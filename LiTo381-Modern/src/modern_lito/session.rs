//! One persistent TLS owner for the modern login/source/character workflow.
//! No network call, disk access or worker join occurs in the UI poll path.

use super::ammo_protocol as ammo;
use super::application_protocol as app;
use super::application_protocol::{
    Credentials, NavigationAction, NavigationPhase, NavigationReply, NavigationStatus, Principal,
    SelectionContext,
};
use super::battle_control as canonical;
use super::character_data::{CharacterDataReply, CharacterDataState};
use super::combat_protocol as combat;
use super::disintegrate_cast_protocol as dis;
use super::equipment;
use super::filtered_chat_protocol as filtered_chat;
use super::gm_protocol as gm;
use super::login_window::{
    CharacterRow, LoginCommand, LoginController, LoginPhase, LoginSnapshot, SourceChoice,
};
use super::matchmaking_protocol as matching;
use super::npc_protocol as npc;
use super::npc_spell_protocol as npc_spell;
use super::profile::{GameplayProtocol, ModernProfile, Trust};
use super::protocol::Hello;
use super::record_protocol as record;
use super::rooms;
use super::spell_protocol as spell;
use super::transport::{connect_and_negotiate, Cancellation, TransportLimits};
use super::world_protocol as world;
use super::{chat_protocol as chat, chat_state};
use super::{direct_chat_protocol as direct, direct_chat_state as direct_state};
use super::{item_protocol as item, shop_protocol as shop};
use std::sync::{mpsc, Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

#[path = "profile_probe.rs"]
pub mod profile_probe;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);
const CHAT_POLL_INTERVAL: Duration = Duration::from_millis(750);
const WORLD_POLL_INTERVAL: Duration = Duration::from_millis(200);

/// Both world owners need a deadline in the actual socket-worker sleep, not
/// just an elapsed-time check after some unrelated room/chat event wakes it.
/// Eligibility remains separate: an idle lobby must not spin on an absent
/// canonical request, and this timer cannot acknowledge/send any UI mutation.
fn world_poll_wait(
    current: Duration,
    legacy_ready: bool,
    canonical_ready: bool,
    elapsed: Duration,
) -> Duration {
    if legacy_ready || canonical_ready {
        current.min(WORLD_POLL_INTERVAL.saturating_sub(elapsed))
    } else {
        current
    }
}
const SKILLS_CONFIRMED_STATUS: &str =
    "已學技能資料已由伺服器確認；可施放項目以已學技能與本場開放能力為準。";

/// Presentation only, called after the successful reply has passed authority
/// and cross-frame checks. Unknown future skills must not be mislabeled as TP.
fn confirmed_cast_status(skill_id: u32, finished: bool) -> &'static str {
    match skill_id {
        spell::TELEPORT => "瞬間移動已由伺服器確認；目的地與扣魔皆採權威結果。",
        spell::LIGHT_ARROW if finished => {
            "光箭與勝敗已由伺服器確認；本場結束，請查看保存狀態後返回房間。"
        }
        spell::LIGHT_ARROW => "光箭施法、扣魔與目標生命值已由伺服器同版確認。",
        spell::BASIC_HEAL => "初級治癒術施法、扣魔與治癒結果已由伺服器同版確認。",
        spell::DISINTEGRATE if finished => {
            "究極光裂術與勝敗已由伺服器確認；本場結束，請查看保存狀態後返回房間。"
        }
        spell::DISINTEGRATE => "究極光裂術、耗魔、正義值與目標生命值已由伺服器同版確認。",
        super::fire_storm_protocol::SKILL_ID if finished => {
            "火風暴與勝敗已由伺服器確認；本場結束，請查看保存狀態後返回房間。"
        }
        super::fire_storm_protocol::SKILL_ID => {
            "火風暴施法、扣魔與範圍目標結果已由伺服器同版確認。"
        }
        _ => "施法結果已由伺服器確認。",
    }
}

/// At most one initial read plus five retries, using the existing TLS worker.
/// A totals revision cannot prove that any particular pending UUID was committed.
struct RecordRefresh {
    context: SelectionContext,
    channel_id: Option<u8>,
    due: Instant,
    retries: usize,
    pending_settlement: bool,
}
impl RecordRefresh {
    const BACKOFF: [u64; 5] = [1, 2, 4, 8, 15];
    fn new(
        context: SelectionContext,
        channel_id: Option<u8>,
        now: Instant,
        pending_settlement: bool,
    ) -> Self {
        Self {
            context,
            channel_id,
            due: now,
            retries: 0,
            pending_settlement,
        }
    }
    fn current(&self, navigation: Option<&NavigationReply>, in_world: bool) -> bool {
        !in_world
            && navigation.is_some_and(|nav| {
                nav.context == self.context
                    && (self.channel_id.is_none() || nav.snapshot.channel_id == self.channel_id)
                    && matches!(
                        nav.snapshot.phase,
                        NavigationPhase::Channels | NavigationPhase::Lobby
                    )
            })
    }
    /// false retires the schedule. A successful read is not a settlement receipt.
    fn completed(&mut self, status: record::Status, now: Instant) -> bool {
        if !self.pending_settlement && status == record::Status::Ok {
            return false;
        }
        let Some(seconds) = Self::BACKOFF.get(self.retries) else {
            return false;
        };
        let Some(due) = now.checked_add(Duration::from_secs(*seconds)) else {
            return false;
        };
        self.retries += 1;
        self.due = due;
        true
    }
}
fn received_record(
    view: &mut LoginSnapshot,
    state: &mut record::State,
    reply: record::Reply,
) -> Result<(), String> {
    if !view
        .navigation
        .as_ref()
        .is_some_and(|nav| nav.context == reply.context)
    {
        return Err("戰績來源／角色／選角世代不一致".into());
    }
    state.receive(reply).map_err(|_| "中央戰績回覆格式不正確")?;
    view.record = state.clone();
    view.record_pending = false;
    Ok(())
}
fn should_poll_combat(frame: Option<&world::Frame>, negotiated: bool) -> bool {
    negotiated
        && frame.is_some_and(|frame| {
            frame.phase != world::Phase::Preparing
                && frame.combat_enabled(negotiated)
                && !frame
                    .descriptor
                    .is_some_and(|d| d.has(world::MatchDescriptor::NPC))
        })
}

fn should_poll_npc(frame: Option<&world::Frame>, negotiated: bool) -> bool {
    negotiated
        && frame.is_some_and(|f| {
            // Preparation can abort before an NPC/combat engine exists. Keep
            // polling the world/navigation authority, not an absent NPC frame.
            matches!(
                f.phase,
                world::Phase::Loading
                    | world::Phase::Countdown
                    | world::Phase::Active
                    | world::Phase::Finished
            ) && f
                .descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::NPC))
        })
}
fn npc_read(context: SelectionContext, channel_id: u8, frame: &world::Frame) -> npc::Request {
    npc::Request {
        context,
        channel_id,
        action: npc::Action::Read,
        world: npc::Scope {
            room_id: frame.room.summary.room_id,
            match_id: frame.match_id,
            generation: frame.generation,
        },
        sequence: 0,
        serial: 0,
        life: 0,
    }
}
/// This game controller owns NPC and Disintegrate preflight/renderer/typed input.
/// Diagnostic callers of Hello::current keep their narrower default offer.
fn game_hello(request_id: u32) -> Hello {
    let mut hello = Hello::current(request_id);
    hello.max_version = super::protocol::RUNTIME_WORLD_VERSION;
    hello.supported_capabilities |= super::protocol::CAP_NPC
        | super::protocol::CAP_DISINTEGRATE
        | super::protocol::CAP_NPC_DISINTEGRATE
        | super::protocol::CAP_NPC_FIRE_STORM
        | super::protocol::CAP_CHAT
        | super::protocol::CAP_DIRECT_CHAT
        | super::protocol::CAP_FILTERED_CHAT
        | super::protocol::CAP_MATCHMAKING
        | super::protocol::CAP_LARGE_MAPS
        | super::protocol::CAP_BATTLE_DATA
        | super::protocol::CAP_ACTION_LIFE;
    hello
}
#[cfg(any(debug_assertions, test))]
fn target_haste_hello(request_id: u32, version: u16) -> Hello {
    use super::protocol as p;
    let mut h = canonical_hello(request_id);
    h.min_version = version;
    h.max_version = version;
    if version == 47 {
        h.supported_capabilities |= p::CAP_SLOW_DATA;
        h.required_capabilities |= p::CAP_SLOW_DATA;
    }
    h.supported_capabilities |= p::CAP_HASTE;
    if version != 47 {
        h.required_capabilities |= p::CAP_HASTE;
    }
    h.required_capabilities |= p::CAP_TIMED
        | p::CAP_WEIGHT
        | p::CAP_SHOP
        | p::CAP_COMBAT
        | p::CAP_CHARACTER_DATA
        | p::CAP_EQUIPMENT;
    h
}
/// One player entry for qualified modes. Capabilities advertise implementations,
/// not permission to enter a map or cast a spell. A race server has no combat or
/// shop authority; a siege binding need not enable Slow. Each descriptor and
/// action still requires its negotiated capabilities and resource qualification.
/// Keep v47 exact: this does not retry or downgrade to a development contract.
fn modern_hello(request_id: u32) -> Hello {
    let mut hello = canonical_hello(request_id);
    hello.min_version = 47;
    hello.max_version = 47;
    hello.supported_capabilities |= super::protocol::CAP_HASTE | super::protocol::CAP_SLOW_DATA;
    hello
}
#[cfg(test)]
mod target_haste_offer_tests {
    use super::*;
    #[test]
    fn opt_in_target_haste_requires_complete_dependencies_without_changing_defaults() {
        use crate::modern_lito::protocol as p;
        let h = target_haste_hello(1, 44);
        assert_eq!((h.min_version, h.max_version), (44, 44));
        p::encode(p::Message::Hello(h)).unwrap();
        assert_ne!(h.required_capabilities & p::CAP_HASTE, 0);
        let cooperative = target_haste_hello(1, 45);
        assert_eq!((cooperative.min_version, cooperative.max_version), (45, 45));
        p::encode(p::Message::Hello(cooperative)).unwrap();
        assert_eq!(cooperative.required_capabilities, h.required_capabilities);
        assert_eq!(cooperative.supported_capabilities, h.supported_capabilities);
        let slow = target_haste_hello(1, 47);
        assert_eq!((slow.min_version, slow.max_version), (47, 47));
        p::encode(p::Message::Hello(slow)).unwrap();
        assert_eq!(
            slow.required_capabilities,
            (h.required_capabilities & !p::CAP_HASTE) | p::CAP_SLOW_DATA
        );
        assert_eq!(
            slow.supported_capabilities,
            h.supported_capabilities | p::CAP_SLOW_DATA
        );
        assert_eq!(canonical_hello(1).max_version, 43);
        assert_eq!(game_hello(1).max_version, 37);
        assert_eq!(
            (canonical_hello(1).supported_capabilities | game_hello(1).supported_capabilities)
                & p::CAP_HASTE,
            0
        );
    }
}
fn canonical_hello(request_id: u32) -> Hello {
    use super::protocol;
    let mut hello = game_hello(request_id);
    hello.min_version = super::battle_world::VERSION;
    hello.max_version = super::battle_payload::npc_motion::VERSION;
    hello.supported_capabilities |= protocol::CAP_GREATER_HASTE;
    hello.required_capabilities |= protocol::CAP_ROOMS
        | protocol::CAP_WORLD
        | protocol::CAP_MATCH_DESCRIPTOR
        | protocol::CAP_BATTLE_DATA
        | protocol::CAP_ACTION_LIFE;
    hello
}
fn receive_npc(
    view: &mut LoginSnapshot,
    state: &mut world::State,
    reply: npc::Reply,
) -> Result<(), String> {
    if !view.world_npc_enabled()
        || !npc::follows(view.npc.as_deref(), &reply)
        || reply.frame.as_ref().is_some_and(|f| f.resources.is_none())
    {
        return Err("NPC 場次、資源清單或生命世代不一致".into());
    }
    receive_combat(view, state, &reply.combat_reply())?;
    view.status = match reply.status {
        npc::Status::Ok if reply.echo.action == npc::Action::ConfirmDeath => {
            "已由伺服器確認復活並返回己方出生點。".into()
        }
        npc::Status::Denied if reply.reason == npc::Reason::SpawnBlocked => {
            "己方出生點暫時被占用；尚未復活，可稍後再次確認。".into()
        }
        npc::Status::Denied if reply.reason == npc::Reason::StaleLife => {
            "死亡世代已更新，本次舊確認未生效。".into()
        }
        npc::Status::Ok
            if state
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Active) =>
        {
            "NPC 位置、攻擊與生命值已由伺服器確認；Quest 完整流程仍在開發。".into()
        }
        npc::Status::Ok => confirmed_world_notice(state.frame()).into(),
        _ => format!(
            "伺服器未接受 NPC 攻擊或尚未就緒：{:?}；不會預先扣血或自動重送。",
            reply.reason
        ),
    };
    view.npc = Some(Arc::new(reply));
    Ok(())
}

fn receive_combat(
    view: &mut LoginSnapshot,
    state: &mut world::State,
    reply: &combat::Reply,
) -> Result<(), String> {
    reply.validate().map_err(|_| "戰鬥回覆格式不正確")?;
    if !view.navigation.as_ref().is_some_and(|nav| {
        nav.context == reply.echo.context && nav.snapshot.channel_id == Some(reply.echo.channel_id)
    }) {
        return Err("戰鬥角色來源與目前選角不一致".into());
    }
    if !state.frame().is_some_and(|f| {
        f.match_id == reply.echo.match_id
            && f.generation == reply.echo.generation
            && f.combat_enabled(true)
    }) {
        return Err("戰鬥世代與目前世界不一致".into());
    }
    if let Some(next) = reply.world_reply() {
        let prior = match &*state {
            world::State::Ready(prior) => Some(prior.as_ref()),
            _ => None,
        };
        if !world::follows(prior, &next) {
            return Err("戰鬥回覆的世界版本已失效".into());
        }
        *state = world::State::Ready(Arc::new(next));
    }
    view.world = state.clone();
    view.combat = reply.frame.clone().map(Arc::new);
    view.npc = None;
    view.combat_supported = true;
    view.status = match reply.status {
        combat::Status::Ok
            if state
                .frame()
                .is_some_and(|f| f.phase == world::Phase::Active) =>
        {
            "普通攻擊與即時素質由伺服器確認；技能以已學清單的可用狀態為準。".into()
        }
        combat::Status::Ok => confirmed_world_notice(state.frame()).into(),
        combat::Status::Denied if reply.reason == combat::Reason::StaleRegen => {
            "自然恢復已改變目標 HP，本次攻擊未提交；未扣箭／血，不會自動重送。".into()
        }
        _ => format!(
            "伺服器未接受攻擊／戰鬥尚未就緒：{:?}；沒有預先扣血。",
            reply.reason
        ),
    };
    Ok(())
}
fn world_notice(reply: &world::Reply) -> &'static str {
    if reply.status != world::Status::Ok {
        return "伺服器未接受此世界操作；保留已確認的位置與狀態。";
    }
    confirmed_world_notice(reply.frame.as_ref())
}
// Presentation follows the validated authority phase, never a local timer
// started from the descriptor's static policy duration.
fn confirmed_world_notice(frame: Option<&world::Frame>) -> &'static str {
    if let Some(race) = frame.and_then(|f| f.race.as_ref()) {
        use super::race_protocol::{RaceState, Settlement};
        match race.state {
            RaceState::Running => return "寵物競賽已開始：新版規則 4 圈；圈數與名次由伺服器確認，本模式不開放攻擊／道具。",
            RaceState::FinishWindow => return "已有選手完賽，伺服器正在執行首位後 10 秒收尾；未完賽者仍可繼續。",
            RaceState::Complete => return match race.settlement {
                Settlement::Saved => "寵物競賽結果已保存至獨立競賽紀錄，不列入兩隊戰鬥勝敗。",
                Settlement::Failed => "競賽結束，但保存結果尚未確認；不可假定未入帳或重複計入。",
                _ => "競賽結束，等待伺服器確認結果保存；未完賽原因與名次分開列示。",
            },
            _ => {},
        }
    }
    match frame.map(|frame| frame.phase) {
        None => "世界已返回待機狀態；房間操作仍需伺服器確認。",
        Some(world::Phase::Preparing) => "伺服器正在重新核對參賽角色資料…",
        Some(world::Phase::Loading) => "正在準備共用地圖與角色資源，等待全員載入…",
        Some(world::Phase::Countdown) => "全員載入完成，伺服器倒數中…",
        Some(world::Phase::Active) => {
            "世界已啟動；移動與普通攻擊由伺服器核准，結算依已協商版本與模式開放。"
        }
        Some(world::Phase::Aborted)
            if frame.is_some_and(|f| f.abort_reason == world::AbortReason::ActiveTimeout) =>
        {
            "本場已達伺服器設定的安全時間上限，已中止；請返回房間，不計算勝敗。"
        }
        Some(world::Phase::Aborted)
            if frame.is_some_and(|f| f.abort_reason == world::AbortReason::LoadTimeout) =>
        {
            "全員載入逾時，世界已中止；請返回房間，不計算勝敗。"
        }
        Some(world::Phase::Aborted) => "世界已中止，請返回房間；此中止不會計算勝敗。",
        Some(world::Phase::Finished) => "本場已結束，勝敗由伺服器確認；請查看保存狀態後返回房間。",
    }
}
fn room_notice(reply: &rooms::Reply) -> &'static str {
    match reply.status {
        rooms::Status::Ok => {
            "房間狀態已由伺服器確認；世界進場依伺服器的資源綁定開放，完整戰鬥尚在開發。"
        }
        rooms::Status::Conflict => "房間版本已更新，已載入現況；請確認後重新操作。",
        rooms::Status::Full => "房間人數已滿，未加入房間。",
        rooms::Status::Unavailable => "房間版本資源已用盡，目前無法變更。",
        rooms::Status::Denied => match reply.reason {
            rooms::Reason::BadPassword => "房間密碼不正確，未加入房間。",
            rooms::Reason::LevelRestricted => "角色等級不符合此房間／頻道的限制。",
            rooms::Reason::NotOwner => "只有房主可以修改此房間設定。",
            rooms::Reason::RoomNotFound => "此房間已不存在，已更新房間列表。",
            _ => "伺服器拒絕此房間操作，畫面已更新為目前狀態。",
        },
    }
}

fn receive_matchmaking(
    view: &mut LoginSnapshot,
    state: &mut matching::State,
    room_state: &mut rooms::State,
    reply: matching::Reply,
) -> Result<(), String> {
    reply.validate().map_err(|_| "配對回覆格式不正確")?;
    if !view.navigation.as_ref().is_some_and(|n| {
        n.context == reply.echo.context
            && n.snapshot.phase == NavigationPhase::Lobby
            && n.snapshot.channel_id == Some(reply.echo.channel_id)
    }) {
        return Err("配對回覆的角色來源／頻道已失效".into());
    }
    view.status = match (&reply.status, reply.snapshot.as_ref()) {
        (matching::Status::Busy, _) => "中央戰績讀取忙碌；尚未確認入列，可稍後再試。",
        (matching::Status::Unavailable, _) => "中央戰績暫不可用；未以零勝敗代替，尚未確認入列。",
        (_, Some(s)) if s.rooms.status != rooms::Status::Ok => room_notice(&s.rooms),
        (_, Some(s)) if s.phase == matching::Phase::Waiting => {
            "已由伺服器確認排隊；等候符合模式／地圖／人數的玩家，可取消配對。"
        }
        (_, Some(s)) if s.phase == matching::Phase::Matched => {
            "配對完成並已確認入房；請選擇準備，全員就緒後由伺服器啟動。"
        }
        _ => "目前沒有等待中的配對；取消配對不會自動離開已加入的房間。",
    }
    .into();
    if let Some(s) = &reply.snapshot {
        *room_state = rooms::State::Ready(Arc::new(s.rooms.clone()));
        view.rooms = room_state.clone();
    }
    *state = matching::State::Ready(Arc::new(reply));
    view.matchmaking = state.clone();
    view.room_pending = false;
    Ok(())
}
fn empty(phase: LoginPhase, status: impl Into<String>) -> LoginSnapshot {
    LoginSnapshot {
        phase,
        status: status.into(),
        sources: Vec::new(),
        selected_source: None,
        characters: Vec::new(),
        roster_epoch: 0,
        navigation: None,
        character_data: CharacterDataState::NotRequested,
        equipment: equipment::State::NotRequested,
        equipment_pending: false,
        rooms: rooms::State::NotRequested,
        room_pending: false,
        matchmaking: matching::State::default(),
        gm_supported: false,
        world: world::State::NotRequested,
        battle: super::battle_session::State::default(),
        canonical: canonical::State::default(),
        battle_supported: false,
        action_supported: false,
        world_pending: false,
        combat: None,
        npc: None,
        combat_pending: false,
        combat_supported: false,
        skills: spell::SkillsState::NotRequested,
        spell_pending: false,
        record: record::State::NotRequested,
        record_pending: false,
        chat: super::chat_state::State::default(),
        chat_pending: false,
        direct_chat: direct_state::State::default(),
        direct_chat_pending: false,
        shop: shop::State::NotRequested,
        shop_pending: false,
        item_pending: false,
        ammo_pending: false,
    }
}

#[derive(Clone, Copy)]
struct Selection {
    source: u32,
    epoch: u64,
}

#[derive(Clone, Copy)]
struct NavigationRequest {
    action: NavigationAction,
    context: SelectionContext,
    channel_id: Option<u8>,
    // Reserve a fresh roster epoch BEFORE sending any mutation. If the server
    // returns to Characters, even the same source must be re-read with this epoch.
    refresh_epoch: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SpellRequest {
    Player(spell::CastRequest),
    Npc(npc_spell::Request),
    Disintegrate(dis::Request),
    NpcDisintegrate(super::npc_disintegrate_protocol::Request),
    NpcFireStorm(super::npc_fire_storm_protocol::Request),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CombatCommand {
    Player(combat::Request),
    Siege(super::siege_protocol::Request),
}

#[derive(Default)]
struct Inbox {
    canonical: Option<(canonical::Command, u64)>,
    action_guard: Option<super::battle_action::Guard>,
    credentials: Option<Credentials>,
    selection: Option<Selection>,
    navigation: Option<NavigationRequest>,
    equipment: Option<(equipment::Request, u64)>,
    room: Option<(rooms::Request, u64)>,
    matchmaking: Option<(matching::Request, u64)>,
    gm: Option<(gm::Request, u64)>,
    world: Option<(world::Request, u64)>,
    combat: Option<(CombatCommand, u64)>,
    npc: Option<(npc::Request, u64)>,
    spell: Option<(Option<SpellRequest>, u64)>,
    record: Option<(SelectionContext, u64)>,
    chat: Option<(chat::Request, u64)>,
    filtered_chat: Option<(filtered_chat::Request, u64)>,
    direct_chat: Option<(direct_state::Token, direct::Request, u64)>,
    shop: Option<(shop::Request, u64)>,
    item: Option<(item::Request, u64)>,
    ammo: Option<(ammo::Request, u64)>,
    auto_world: bool,
    shutdown: bool,
}
impl Inbox {
    fn busy(&self) -> bool {
        self.shutdown
            || self.canonical.is_some()
            || self.action_guard.is_some()
            || self.auto_world
            || self.credentials.is_some()
            || self.selection.is_some()
            || self.navigation.is_some()
            || self.equipment.is_some()
            || self.room.is_some()
            || self.matchmaking.is_some()
            || self.gm.is_some()
            || self.world.is_some()
            || self.combat.is_some()
            || self.npc.is_some()
            || self.spell.is_some()
            || self.record.is_some()
            || self.chat.is_some()
            || self.filtered_chat.is_some()
            || self.direct_chat.is_some()
            || self.shop.is_some()
            || self.item.is_some()
            || self.ammo.is_some()
    }
}

#[derive(Clone)]
struct Published {
    view: LoginSnapshot,
    source_epoch: u64,
    navigation_epoch: u64,
}

struct Worker {
    pending_canonical: Option<(canonical::Command, u64)>,
    cancel: Cancellation,
    inbox: Arc<Mutex<Inbox>>,
    wake: mpsc::SyncSender<()>,
    published: Arc<Mutex<Published>>,
    handle: JoinHandle<()>,
    desired: Option<Selection>,
    pending_navigation: Option<NavigationRequest>,
    pending_equipment: Option<(equipment::Request, u64)>,
    pending_room: Option<(rooms::Expectation, u64)>,
    pending_matchmaking: Option<(matching::Request, u64)>,
    pending_world: Option<(world::Request, u64)>,
    pending_combat: Option<(CombatCommand, u64)>,
    pending_npc: Option<(npc::Request, u64)>,
    pending_spell: Option<(Option<SpellRequest>, u64)>,
    pending_record: Option<(SelectionContext, u64)>,
    pending_chat: Option<u64>,
    pending_direct_chat: Option<u64>,
    pending_shop: Option<(shop::Request, u64)>,
    pending_item: Option<(item::Request, u64)>,
    pending_ammo: Option<(ammo::Request, u64)>,
}

impl Worker {
    fn any_pending(&self) -> bool {
        self.pending_navigation.is_some()
            || self.pending_canonical.is_some()
            || self.pending_equipment.is_some()
            || self.pending_room.is_some()
            || self.pending_matchmaking.is_some()
            || self.pending_world.is_some()
            || self.pending_combat.is_some()
            || self.pending_npc.is_some()
            || self.pending_spell.is_some()
            || self.pending_record.is_some()
            || self.pending_chat.is_some()
            || self.pending_direct_chat.is_some()
            || self.pending_shop.is_some()
            || self.pending_item.is_some()
            || self.pending_ammo.is_some()
    }
    fn cancel(&self) {
        self.cancel.cancel();
        // A paused worker must never block cancellation on the UI thread. If
        // clearing is busy, the atomic token still cancels and the mailbox owns
        // the queued credentials until the worker retires.
        if let Ok(mut inbox) = self.inbox.try_lock() {
            inbox.shutdown = true;
            inbox.canonical = None;
            inbox.action_guard = None;
            inbox.credentials = None;
            inbox.selection = None;
            inbox.navigation = None;
            inbox.equipment = None;
            inbox.room = None;
            inbox.matchmaking = None;
            inbox.gm = None;
            inbox.world = None;
            inbox.combat = None;
            inbox.npc = None;
            inbox.spell = None;
            inbox.record = None;
            inbox.chat = None;
            inbox.filtered_chat = None;
            inbox.direct_chat = None;
            inbox.shop = None;
            inbox.item = None;
            inbox.ammo = None;
        }
        let _ = self.wake.try_send(());
    }
}

// Both the ordinary login owner and the explicit no-account probe use this
// exact trust, deadline and one-attempt transport path. No fallback HELLO.
fn connect_profile(
    profile: &ModernProfile,
    hello: Hello,
    extensions: Option<super::protocol::extensions::Offer>,
    cancel: Cancellation,
) -> Result<super::transport::ModernTlsConnection, super::transport::TransportError> {
    let root = match profile.trust() {
        Trust::WebPki => None,
        Trust::PrivateCa(der) => Some(der.as_slice()),
    };
    let limits = TransportLimits {
        connect_timeout: profile.connect_timeout(),
        total_timeout: profile.connect_timeout() + profile.exchange_timeout(),
        ..Default::default()
    };
    let mut connection = match extensions {
        Some(offer) => super::transport::connect_and_negotiate_extended(
            profile.endpoint(),
            profile.tls_server_name(),
            root,
            hello,
            offer,
            limits,
            cancel,
        ),
        None => connect_and_negotiate(
            profile.endpoint(),
            profile.tls_server_name(),
            root,
            hello,
            limits,
            cancel,
        ),
    }?;
    connection.set_exchange_timeout(profile.exchange_timeout())?;
    Ok(connection)
}

pub struct SessionController {
    canonical_mode: bool,
    handshake: Hello,
    extension_offer: Option<super::protocol::extensions::Offer>,
    queued_guard: Option<super::battle_action::Guard>,
    profile: ModernProfile,
    view: LoginSnapshot,
    active: Option<Worker>,
    retiring: Vec<Worker>,
    next_connection: u32,
    next_selection: u64,
    closing: bool,
}

impl SessionController {
    /// Test-only network scheduling fixture: retain the EXACT request/proof
    /// captured before death, then enqueue it later. Never fabricates a view,
    /// HP, life, ACK or server state.
    #[cfg(test)]
    pub(crate) fn submit_deferred_npc_fixture(
        &mut self,
        request: npc::Request,
        guard: super::battle_action::Guard,
    ) -> Result<(), String> {
        self.poll();
        super::battle_action::encode(
            guard,
            &npc::encode_request(1, &request).map_err(|e| e.to_string())?,
            37,
        )
        .map_err(|e| e.to_string())?;
        let next = self
            .next_selection
            .checked_add(1)
            .ok_or("operation epoch exhausted")?;
        let active = self.active.as_mut().ok_or("no worker")?;
        let mut inbox = active.inbox.try_lock().map_err(|_| "worker busy")?;
        if inbox.busy() || active.any_pending() {
            return Err("operation pending".into());
        }
        inbox.action_guard = Some(guard);
        inbox.npc = Some((request, self.next_selection));
        active.pending_npc = inbox.npc;
        self.next_selection = next;
        self.view.combat_pending = true;
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    pub fn new(profile: ModernProfile) -> Self {
        let modern = profile.gameplay_protocol() == GameplayProtocol::ModernV47;
        let mut controller = Self::disconnected(profile);
        if modern {
            controller.canonical_mode = true;
            controller.handshake = modern_hello(1);
            controller.view.status = "Modern v47（尚在驗收）；請連線至相容伺服器。".into();
            if let Some(extension) = controller.profile.gameplay_extension() {
                use super::profile::GameplayExtension as Extension;
                use super::protocol::extensions as e;
                controller.handshake.supported_capabilities |= e::DEPENDENCIES;
                controller.handshake.required_capabilities |= e::DEPENDENCIES;
                let (offer, status) = match extension {
                    Extension::FullVipV2 => (e::FULL_VIP_V2,
                        "Modern v47 完整数值與公開外觀（共享呈現尚未完成）；需新版專用伺服器，不會降級。"),
                    Extension::FullVipV1 => (e::FULL_VIP,
                        "Modern v47 完整數值（共享呈現尚未完成）；需專用完整數值伺服器，不會降級。"),
                    Extension::PlayerHealingV1 => (e::HEALING,
                        "Modern v47 角色治癒（尚在驗收）；需明確支援擴充的伺服器，不會降級。"),
                    Extension::ArmorProtectionV1 => (e::ARMOR,
                        "Modern v47 護甲資料（呈現尚未完成）；需明確支援擴充的伺服器，不會降級。"),
                    Extension::PlayerHealingArmorV1 => (e::HEALING_ARMOR,
                        "Modern v47 治癒與護甲資料（護甲呈現尚未完成）；需明確支援擴充的伺服器，不會降級。"),
                    Extension::ArmorPresentationV1 => (e::ARMOR_PRESENTATION,
                        "Modern v47 護甲特效資料（資源與畫面尚在驗收）；需明確支援新版擴充，不會降級。"),
                    Extension::PlayerHealingArmorPresentationV1 => (e::HEALING_ARMOR_PRESENTATION,
                        "Modern v47 治癒與護甲特效資料（資源與畫面尚在驗收）；需明確支援新版擴充，不會降級。"),
                };
                controller.extension_offer = Some(offer);
                controller.view.status = status.into();
            }
        }
        controller
    }

    /// No IO, connection, credential, or retained world state. Historical
    /// explicit development constructors keep their own pinned contracts.
    fn disconnected(profile: ModernProfile) -> Self {
        Self {
            canonical_mode: false,
            handshake: game_hello(1),
            extension_offer: None,
            queued_guard: None,
            profile,
            view: empty(LoginPhase::Disconnected, "尚未連線"),
            active: None,
            retiring: Vec::new(),
            next_connection: 1,
            next_selection: 1,
            closing: false,
        }
    }

    /// Same login/room/heartbeat worker, with the complete world contract.
    /// Historical explicit canonical entry; schema1's ordinary window stays
    /// on v37. A schema2 ordinary entry selects its pinned v47 above.
    pub(crate) fn new_canonical(profile: ModernProfile) -> Self {
        let mut controller = Self::disconnected(profile);
        controller.canonical_mode = true;
        controller.handshake = canonical_hello(1);
        controller
    }

    /// Explicit same-EXE development entry. Uses the real controller/worker;
    /// no synthetic login, bundled credentials, fallback or fixture dependency.
    #[cfg(any(debug_assertions, test))]
    pub fn new_canonical_development(profile: ModernProfile) -> Self {
        let mut controller = Self::new_canonical(profile);
        controller.view.status = "新版戰鬥開發驗證（尚未交付）；請連線至相容伺服器。".into();
        controller
    }

    /// Explicit same-EXE v44 development entry. Require target-haste and its
    /// dependencies; do not fall back to another version or connect implicitly.
    #[cfg(any(debug_assertions, test))]
    pub fn new_target_haste_development(profile: ModernProfile) -> Self {
        let mut c = Self::new_canonical(profile);
        c.handshake = target_haste_hello(1, 44);
        c.view.status =
            "v44 指定目標加速術開發驗證（尚未交付）；請連線至明確支援此版本的伺服器。".into();
        c
    }

    /// Explicit v45 opt-in for cooperative TimeAttack+Haste; old entries stay frozen.
    #[cfg(any(debug_assertions, test))]
    pub fn new_cooperative_haste_development(profile: ModernProfile) -> Self {
        let mut c = Self::new_target_haste_development(profile);
        c.handshake = target_haste_hello(1, 45);
        c.view.status = "v45 合作限時戰加速術開發驗證（尚未交付）；不會降級至 v44。".into();
        c
    }

    /// Same Modern EXE; explicit v47 development only, no implicit connection.
    #[cfg(any(debug_assertions, test))]
    pub fn new_slow_development(profile: ModernProfile) -> Self {
        let mut c = Self::new_canonical(profile);
        c.handshake = target_haste_hello(1, 47);
        c.view.status = "v47 緩速術開發驗證（尚未交付）；需相容伺服器與共享資源。".into();
        c
    }

    fn connect(&mut self) -> Result<(), String> {
        if self.closing {
            return Err("視窗正在關閉".into());
        }
        self.reap();
        if !matches!(
            self.view.phase,
            LoginPhase::Disconnected | LoginPhase::Failed
        ) {
            return Err("請先中斷目前的連線".into());
        }
        if self.retiring.len() + usize::from(self.active.is_some()) >= 2 {
            return Err("上一個連線仍在取消，請稍候".into());
        }
        self.next_selection
            .checked_add(1)
            .ok_or("操作世代已用盡，請重新開啟程式")?;
        let request_id = self.next_connection;
        let next = request_id
            .checked_add(1)
            .ok_or("連線世代已用盡，請重新開啟程式")?;
        let cancel = Cancellation::new();
        let inbox = Arc::new(Mutex::new(Inbox::default()));
        let published = Arc::new(Mutex::new(Published {
            view: empty(LoginPhase::Connecting, "正在建立安全連線…"),
            source_epoch: 0,
            navigation_epoch: 0,
        }));
        let (wake, receiver) = mpsc::sync_channel(1);
        let profile = self.profile.clone();
        let hello = Hello {
            request_id,
            ..self.handshake
        };
        let worker_cancel = cancel.clone();
        let extension_offer = self.extension_offer;
        let worker_inbox = inbox.clone();
        let worker_published = published.clone();
        let handle = thread::Builder::new()
            .name(format!("lito-session-{request_id}"))
            .spawn(move || {
                run_worker(
                    profile,
                    hello,
                    extension_offer,
                    worker_cancel,
                    worker_inbox,
                    worker_published,
                    receiver,
                )
            })
            .map_err(|_| "無法建立連線工作執行緒")?;
        self.install_connection(
            next,
            Worker {
                pending_canonical: None,
                cancel,
                inbox,
                wake,
                published,
                handle,
                desired: None,
                pending_navigation: None,
                pending_equipment: None,
                pending_room: None,
                pending_matchmaking: None,
                pending_world: None,
                pending_combat: None,
                pending_npc: None,
                pending_spell: None,
                pending_record: None,
                pending_chat: None,
                pending_direct_chat: None,
                pending_shop: None,
                pending_item: None,
                pending_ammo: None,
            },
        );
        Ok(())
    }

    fn install_connection(&mut self, next_connection: u32, worker: Worker) {
        self.retire_active();
        self.next_connection = next_connection;
        // SelectionContext is also carried by local UI commands. Its epoch must
        // remain unique for this controller's lifetime, not merely one socket:
        // reconnecting to the same source/object must not revive an old click.
        self.view = empty(LoginPhase::Connecting, "正在建立安全連線…");
        self.active = Some(worker);
    }

    fn canonical_command(&mut self, command: canonical::Command) -> Result<(), String> {
        if !self.canonical_mode
            || self.closing
            || self.view.phase != LoginPhase::InLobby
            || !command.current(&self.view.canonical, self.view.navigation.as_ref())
        {
            return Err("新版戰鬥尚未就緒，或畫面的生命／場次已更新".into());
        }
        command.validate()?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成".into());
        }
        let pending = (command, self.next_selection);
        inbox.canonical = Some(pending.clone());
        active.pending_canonical = Some(pending);
        self.next_selection = next;
        self.view.world_pending = true;
        self.view.status = "等待伺服器確認新版戰鬥操作…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }

    fn login(&mut self, account: String, password: String) -> Result<(), String> {
        // Construct the non-Debug/zeroizing credential value before state checks,
        // so even an invalid UI command does not retain its plaintext password.
        let credentials =
            Credentials::new(account, password).map_err(|_| "帳號或密碼格式不符合限制")?;
        if self.view.phase != LoginPhase::LoginReady {
            return Err("請先完成安全連線".into());
        }
        let active = self.active.as_ref().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候再試")?;
        if inbox.credentials.is_some() || inbox.shutdown {
            return Err("登入請求已送出或連線正在關閉".into());
        }
        inbox.credentials = Some(credentials);
        self.view.phase = LoginPhase::Authenticating;
        self.view.status = "正在驗證帳號…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }

    fn select_source(&mut self, source: u32) -> Result<(), String> {
        if !matches!(
            self.view.phase,
            LoginPhase::ChoosingSource
                | LoginPhase::LoadingCharacters
                | LoginPhase::CharactersReady
        ) || !self.view.sources.iter().any(|entry| entry.id == source)
        {
            return Err("尚未登入或伺服器來源未授權".into());
        }
        if self.view.selected_source == Some(source)
            && matches!(
                self.view.phase,
                LoginPhase::LoadingCharacters | LoginPhase::CharactersReady
            )
        {
            return Ok(());
        }
        let next = self
            .next_selection
            .checked_add(1)
            .ok_or("來源選擇世代已用盡，請重新開啟程式")?;
        let selection = Selection {
            source,
            epoch: self.next_selection,
        };
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候再試")?;
        if inbox.shutdown {
            return Err("連線正在關閉".into());
        }
        // Latest selection replaces only a not-yet-sent request; never changes
        // the identity/epoch of an already in-flight response.
        inbox.selection = Some(selection);
        active.desired = Some(selection);
        self.next_selection = next;
        self.view.phase = LoginPhase::LoadingCharacters;
        self.view.status = "正在讀取選定來源的角色…".into();
        self.view.selected_source = Some(source);
        self.view.characters.clear();
        self.view.roster_epoch = 0;
        self.view.navigation = None;
        self.view.character_data = CharacterDataState::NotRequested;
        self.view.equipment = equipment::State::NotRequested;
        self.view.equipment_pending = false;
        self.view.rooms = rooms::State::NotRequested;
        self.view.room_pending = false;
        self.view.chat = chat_state::State::default();
        self.view.chat_pending = false;
        self.view.direct_chat_pending = false;
        self.view.direct_chat.clear();
        self.view.record = record::State::NotRequested;
        self.view.record_pending = false;
        self.view.shop = shop::State::NotRequested;
        self.view.shop_pending = false;
        self.view.item_pending = false;
        self.view.ammo_pending = false;
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }

    fn navigate(
        &mut self,
        action: NavigationAction,
        context: SelectionContext,
        channel_id: Option<u8>,
    ) -> Result<(), String> {
        if self.closing {
            return Err("視窗正在關閉".into());
        }
        if self.view.equipment_pending {
            return Err("裝備操作尚未完成，請稍候或中斷連線".into());
        }
        if self.view.room_pending {
            return Err("房間操作尚未完成，請稍候或中斷連線".into());
        }
        if matches!(
            action,
            NavigationAction::LeaveLobby | NavigationAction::ReturnCharacters
        ) && self.view.rooms.joined().is_some()
        {
            return Err("請先離開戰鬥室，再離開大廳或返回角色".into());
        }
        match action {
            NavigationAction::SelectCharacter => {
                if self.view.phase != LoginPhase::CharactersReady
                    || self.view.roster_epoch == 0
                    || self.view.roster_epoch != context.epoch
                    || self.view.selected_source != Some(context.key.source_id)
                    || !self.view.characters.iter().any(|row| {
                        row.source_id == context.key.source_id
                            && row.object_id == context.key.object_id
                    })
                    || channel_id.is_some()
                {
                    return Err("角色列表已改變，請重新選擇目前來源的角色".into());
                }
            }
            NavigationAction::EnterChannel
            | NavigationAction::ReturnCharacters
            | NavigationAction::LeaveLobby => {
                let navigation = self.view.navigation.as_ref().ok_or("尚未選擇角色")?;
                if navigation.context != context {
                    return Err("角色選擇已改變，請重新確認".into());
                }
                if action == NavigationAction::EnterChannel {
                    channel_id
                        .and_then(|id| {
                            navigation
                                .snapshot
                                .channels
                                .iter()
                                .find(|channel| channel.id == id)
                        })
                        .ok_or("請選擇伺服器提供的頻道")?;
                    if self.view.phase != LoginPhase::ChoosingChannel
                        || navigation.snapshot.phase != NavigationPhase::Channels
                    {
                        return Err("目前操作尚未完成，請稍候或中斷連線".into());
                    }
                    // Tiers describe recommendations, not admission. Population is
                    // a potentially stale snapshot; only the server decides entry.
                } else if action == NavigationAction::LeaveLobby {
                    if channel_id.is_some()
                        || self.view.phase != LoginPhase::InLobby
                        || navigation.snapshot.phase != NavigationPhase::Lobby
                    {
                        return Err("尚未進入大廳，或上一個操作尚未完成".into());
                    }
                } else if channel_id.is_some()
                    || !matches!(
                        self.view.phase,
                        LoginPhase::ChoosingChannel | LoginPhase::InLobby
                    )
                {
                    return Err("目前操作尚未完成，請稍候或中斷連線".into());
                }
            }
        }
        let next = self
            .next_selection
            .checked_add(1)
            .ok_or("操作世代已用盡，請重新開啟程式")?;
        let request = NavigationRequest {
            action,
            context,
            channel_id,
            refresh_epoch: self.next_selection,
        };
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候再試")?;
        if inbox.shutdown
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || active.pending_navigation.is_some()
            || inbox.auto_world
            || inbox.equipment.is_some()
            || active.pending_equipment.is_some()
            || inbox.room.is_some()
            || active.pending_room.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        inbox.navigation = Some(request);
        active.pending_navigation = Some(request);
        self.next_selection = next;
        self.view.phase = match action {
            NavigationAction::SelectCharacter => LoginPhase::SelectingCharacter,
            NavigationAction::EnterChannel => LoginPhase::JoiningLobby,
            NavigationAction::ReturnCharacters => LoginPhase::ReturningCharacters,
            NavigationAction::LeaveLobby => LoginPhase::LeavingLobby,
        };
        self.view.status = "等待伺服器確認操作；尚未變更角色／頻道狀態…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }

    fn equipment(&mut self, request: equipment::Request) -> Result<(), String> {
        request.validate().map_err(|_| "裝備命令格式不正確")?;
        if self.closing
            || self.view.equipment_pending
            || self.view.room_pending
            || !matches!(
                self.view.phase,
                LoginPhase::ChoosingChannel | LoginPhase::InLobby
            )
            || self.view.navigation.as_ref().map(|reply| reply.context) != Some(request.context)
            || matches!(
                self.view.equipment,
                equipment::State::NotRequested | equipment::State::Unsupported
            )
        {
            return Err("裝備尚未就緒，或上一個操作尚未完成".into());
        }
        if request.action == equipment::Action::Set {
            let equipment::State::Ready(current) = &self.view.equipment else {
                return Err("請先重新讀取裝備狀態".into());
            };
            if request.expected_revision != current.revision
                || request.expected_view_token != current.view_token
                || current.equipped(request.item_id).is_none()
            {
                return Err("裝備畫面已改變，請確認目前物件與版本".into());
            }
        }
        let next = self
            .next_selection
            .checked_add(1)
            .ok_or("操作世代已用盡，請重新開啟程式")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候再試")?;
        if inbox.shutdown
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.auto_world
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || inbox.room.is_some()
            || active.pending_room.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        let operation = (request, self.next_selection);
        inbox.equipment = Some(operation);
        active.pending_equipment = Some(operation);
        self.next_selection = next;
        self.view.equipment_pending = true;
        self.view.status = "等待伺服器確認裝備操作；尚未變更已確認的穿戴狀態…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn matchmaking(&mut self, request: matching::Request) -> Result<(), String> {
        request.validate().map_err(|_| "配對條件不正確")?;
        let rooms::State::Ready(directory) = &self.view.rooms else {
            return Err("尚未讀取房間與地圖選項".into());
        };
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || !self.view.matchmaking.supported()
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.world_pending
            || self.view.world.frame().is_some()
            || !self.view.navigation.as_ref().is_some_and(|n| {
                n.context == request.context && n.snapshot.channel_id == Some(request.channel_id)
            })
            || match request.action {
                matching::Action::Start => {
                    directory.joined.is_some()
                        || self.view.matchmaking.waiting().is_some()
                        || !directory
                            .options
                            .iter()
                            .any(|o| request.criteria.unwrap().accepts(o))
                }
                matching::Action::Cancel => self
                    .view
                    .matchmaking
                    .waiting()
                    .is_none_or(|s| s.ticket_id != request.ticket_id),
                matching::Action::Read => {
                    request.ticket_id != 0
                        && self
                            .view
                            .matchmaking
                            .snapshot()
                            .is_none_or(|s| s.ticket_id != request.ticket_id)
                }
            }
        {
            return Err("配對尚未就緒、地圖不可用，或本次配對編號已變更".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        if active.any_pending() {
            return Err("上一個操作尚未完成".into());
        }
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        let operation = (request, self.next_selection);
        inbox.matchmaking = Some(operation);
        active.pending_matchmaking = Some(operation);
        self.next_selection = next;
        self.view.room_pending = true;
        self.view.status = "等待伺服器確認配對操作；不預先入列、取消或進入房間…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn rooms(&mut self, request: rooms::Request) -> Result<(), String> {
        request.validate().map_err(|_| "房間命令格式不正確")?;
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || self.view.room_pending
            || self.view.equipment_pending
            || !matches!(self.view.rooms, rooms::State::Ready(_))
            || !self.view.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.common.context
                    && nav.snapshot.channel_id == Some(request.common.channel_id)
            })
        {
            return Err("房間尚未就緒，或上一個操作尚未完成".into());
        }
        if let rooms::State::Ready(directory) = &self.view.rooms {
            if !request.capacity_allowed(directory) {
                return Err("房間人數超出此模式／地圖的伺服器上限".into());
            }
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.auto_world
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        active.pending_room = Some((request.expectation(), self.next_selection));
        inbox.room = Some((request, self.next_selection));
        self.next_selection = next;
        self.view.room_pending = true;
        self.view.status = "等待伺服器確認房間操作；畫面仍為上次已確認狀態…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn world(&mut self, request: world::Request) -> Result<(), String> {
        request.validate().map_err(|_| "世界命令格式不正確")?;
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.world_pending
            || self.view.combat_pending
            || !matches!(self.view.world, world::State::Ready(_))
            || !self.view.navigation.as_ref().is_some_and(|nav| {
                nav.context == request.context
                    && nav.snapshot.channel_id == Some(request.channel_id)
            })
        {
            return Err("世界尚未就緒，或上一個操作尚未完成".into());
        }
        if request.action != world::Action::Read && request.action != world::Action::Prepare {
            let frame = self.view.world.frame().ok_or("尚未確認世界進場")?;
            if request.match_id != frame.match_id || request.generation != frame.generation {
                return Err("世界世代已改變，請等待最新狀態".into());
            }
            if request.action == world::Action::Move
                && frame.race.is_some()
                && (frame.phase != world::Phase::Active
                    || !frame.actors.iter().any(|a| {
                        a.matches_character(request.context.key)
                            && frame.actor_occupies(a.entity_id)
                    }))
            {
                return Err("本角色已完賽／退出，或競賽尚未開始；不再提交移動。".into());
            }
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.auto_world
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.world.is_some()
            || inbox.combat.is_some()
            || inbox.npc.is_some()
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
            || active.pending_world.is_some()
            || active.pending_combat.is_some()
            || active.pending_npc.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.world = Some((request, self.next_selection));
        active.pending_world = Some((request, self.next_selection));
        self.next_selection = next;
        self.view.world_pending = true;
        self.view.status = "等待伺服器確認世界操作…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn gm_bots(&mut self, count: u16) -> Result<(), String> {
        let nav = self.view.navigation.as_ref().ok_or("請先進入房間")?;
        let room = self.view.rooms.joined().ok_or("請先進入房間")?;
        if self.closing
            || !self.view.gm_supported
            || self.view.phase != LoginPhase::InLobby
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.spell_pending
            || self.view.record_pending
            || self.view.world.frame().is_some()
            || room.member(nav.context.key).is_none()
        {
            return Err("假人操作尚未就緒，或上一個操作尚未完成".into());
        }
        let request = gm::Request {
            context: nav.context,
            channel_id: nav.snapshot.channel_id.ok_or("缺少頻道")?,
            room_id: room.summary.room_id,
            expected_revision: room.summary.revision,
            count,
        };
        request.validate().map_err(|_| "假人數量必須是 0 至 100")?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.auto_world
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.gm.is_some()
            || inbox.world.is_some()
            || inbox.combat.is_some()
            || inbox.npc.is_some()
            || inbox.spell.is_some()
            || inbox.record.is_some()
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
            || active.pending_world.is_some()
            || active.pending_combat.is_some()
            || active.pending_npc.is_some()
            || active.pending_spell.is_some()
            || active.pending_record.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        // Reuse the room mutation reservation/epoch. The exact GM expectation
        // is checked by transport before this epoch can be published.
        active.pending_room = Some((
            rooms::Expectation {
                common: request.read_common(),
                draft: None,
            },
            self.next_selection,
        ));
        inbox.gm = Some((request, self.next_selection));
        self.next_selection = next;
        self.view.room_pending = true;
        self.view.status = "等待伺服器重新查驗 GM 權限與房間；尚未增減假人…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn supply_busy(&self) -> bool {
        self.closing
            || self.view.equipment_pending
            || self.view.room_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.spell_pending
            || self.view.record_pending
            || self.view.shop_pending
            || self.view.item_pending
            || self.view.ammo_pending
    }
    fn shop(&mut self, request: shop::Request) -> Result<(), String> {
        request.validate().map_err(|_| "商店命令格式不正確")?;
        if self.supply_busy()
            || self.view.phase != LoginPhase::InLobby
            || self.view.world.frame().is_some()
            || matches!(self.view.shop, shop::State::Unsupported)
            || !self.view.navigation.as_ref().is_some_and(|n| {
                n.context == request.context && n.snapshot.channel_id == Some(request.channel_id)
            })
            || self.view.rooms.joined().map_or(0, |r| r.summary.room_id) != request.room_id
            || self.view.rooms.joined().map_or(0, |r| r.summary.revision)
                != request.expected_room_revision
        {
            return Err("商店尚未就緒或房間已改變，請重新讀取".into());
        }
        if request.action != shop::Action::Read
            && !self.view.shop.snapshot().is_some_and(|s| {
                s.round_id == request.round_id
                    && s.revision == request.expected_revision
                    && s.token == request.token
            })
        {
            return Err("商品／補給版本已改變，請重新確認".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        active.pending_shop = Some((request, self.next_selection));
        inbox.shop = active.pending_shop;
        self.next_selection = next;
        self.view.shop_pending = true;
        self.view.status = "等待伺服器確認本場補給；不預先扣款或增減數量…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn use_item(&mut self, item_id: i64) -> Result<(), String> {
        if self.supply_busy() || self.view.phase != LoginPhase::InLobby {
            return Err("補給操作尚未就緒，請等候目前操作完成".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("尚未確認選角")?;
        let frame = self.view.combat.as_ref().ok_or("尚未確認戰鬥與補給")?;
        if self.view.world.frame() != Some(&frame.world)
            || frame.world.phase != world::Phase::Active
        {
            return Err("僅已確認的戰鬥中可使用補給".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("目前角色不在世界")?;
        let state = frame.item(actor.entity_id).ok_or("此世界未開放本場補給")?;
        if !frame.vital(actor.entity_id).is_some_and(|v| v.hp > 0) || !item::supported(item_id) {
            return Err("目前角色不能使用這項補給".into());
        }
        if let Some(kind) = super::timed_protocol::Kind::from_item(item_id) {
            let member = frame
                .world
                .room
                .members
                .iter()
                .find(|m| m.matches_character(nav.context.key))
                .ok_or("目前角色不在凍結名單")?;
            if frame.timed.is_empty() {
                return Err("此世界未開放限時補給".into());
            }
            if !kind.eligible(member.class) {
                return Err("角色職業不能使用這項限時補給".into());
            }
        }
        let request = item::Request {
            context: nav.context,
            channel_id: nav.snapshot.channel_id.ok_or("缺少頻道")?,
            action: item::Action::Use,
            match_id: frame.world.match_id,
            generation: frame.world.generation,
            sequence: state
                .request_sequence
                .checked_add(1)
                .ok_or("用藥序列已用盡")?,
            item_id,
        };
        request.validate().map_err(|_| "補給命令格式不正確")?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        active.pending_item = Some((request, self.next_selection));
        inbox.item = active.pending_item;
        self.next_selection = next;
        self.view.item_pending = true;
        self.view.status = "等待伺服器確認用藥；尚未改動HP或數量…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn select_ammo(&mut self, item_id: i64) -> Result<(), String> {
        if self.supply_busy() || self.view.phase != LoginPhase::InLobby {
            return Err("箭矢操作尚未就緒，請等候目前操作完成".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("尚未確認選角")?;
        let frame = self.view.combat.as_ref().ok_or("尚未確認戰鬥與箭袋")?;
        if self.view.world.frame() != Some(&frame.world)
            || frame.world.phase != world::Phase::Active
        {
            return Err("僅已確認的戰鬥中可選箭".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("目前角色不在世界")?;
        let state = frame.ammo(actor.entity_id).ok_or("此世界未開放箭袋")?;
        if !ammo::supported(item_id)
            || !frame.vital(actor.entity_id).is_some_and(|v| v.hp > 0)
            || !state.stock(item_id).is_some_and(|s| s.remaining > 0)
        {
            return Err("目前不能選用這種箭矢，請確認剩餘數量".into());
        }
        let request = ammo::Request {
            action: ammo::Action::Select,
            item_id,
            sequence: state
                .request_sequence
                .checked_add(1)
                .ok_or("選箭序號已用盡")?,
            ..ammo::Request::read(
                nav.context,
                nav.snapshot.channel_id.ok_or("缺少頻道")?,
                &frame.world,
            )
        };
        request.validate().map_err(|_| "選箭命令格式不正確")?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        active.pending_ammo = Some((request, self.next_selection));
        inbox.ammo = active.pending_ammo;
        self.next_selection = next;
        self.view.ammo_pending = true;
        self.view.status = "等待伺服器確認選箭；尚未改變箭種或數量…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn attack(&mut self, target_entity_id: u64) -> Result<(), String> {
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || !self.view.world_combat_enabled()
            || self.view.world_npc_enabled()
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
        {
            return Err("戰鬥尚未確認，或上一個操作尚未完成".into());
        }
        let frame = self
            .view
            .combat
            .as_ref()
            .ok_or("正在等待伺服器確認即時戰鬥資料")?;
        if self.view.world.frame() != Some(&frame.world)
            || frame.world.phase != world::Phase::Active
        {
            return Err("戰鬥畫面版本尚未確認".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("尚未確認角色")?;
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("戰鬥角色不一致")?;
        let vital = frame.vital(actor.entity_id).ok_or("缺少戰鬥素質")?;
        if ammo::bow(vital.weapon_type)
            && frame.ammo(actor.entity_id).is_none_or(|s| !s.available())
        {
            return Err("沒有可用的已選箭矢；請先選箭，耗盡後不會自動換箭".into());
        }
        if frame
            .weight(actor.entity_id)
            .is_some_and(|w| !w.can_attack())
        {
            return Err("比賽負重階段達 197：不能普攻；可移動或使用補給減重".into());
        }
        if vital.hp == 0
            || target_entity_id == actor.entity_id
            || super::honor_protocol::target(frame, target_entity_id).is_none()
        {
            return Err("攻擊目標或自身狀態不合法".into());
        }
        let request = combat::Request {
            action: combat::Action::Attack,
            target_entity_id,
            sequence: vital
                .attack_sequence
                .checked_add(1)
                .filter(|v| *v <= i64::MAX as u64)
                .ok_or("攻擊序號已用盡")?,
            ..combat::Request::read(
                nav.context,
                nav.snapshot.channel_id.ok_or("缺少頻道")?,
                &frame.world,
            )
        };
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.auto_world
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.world.is_some()
            || inbox.combat.is_some()
            || inbox.npc.is_some()
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
            || active.pending_world.is_some()
            || active.pending_combat.is_some()
            || active.pending_npc.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.combat = Some((CombatCommand::Player(request), self.next_selection));
        active.pending_combat = Some((CombatCommand::Player(request), self.next_selection));
        self.next_selection = next;
        self.view.combat_pending = true;
        self.view.status = "等待伺服器確認普通攻擊…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn siege(
        &mut self,
        scope: super::siege_protocol::Scope,
        action: super::siege_protocol::Action,
    ) -> Result<(), String> {
        use super::siege_protocol as siege;
        if self.closing || self.view.phase != LoginPhase::InLobby || !self.view.combat_supported {
            return Err("攻城操作尚未就緒".into());
        }
        let frame = self.view.combat.as_deref().ok_or("缺少完整戰鬥資料")?;
        let nav = self.view.navigation.as_ref().ok_or("尚未確認角色")?;
        if self.view.world.frame() != Some(&frame.world) {
            return Err("攻城畫面已失效".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("攻城角色不一致")?;
        let sequence = match action {
            siege::Action::AttackTower(_) => {
                if frame
                    .weight(actor.entity_id)
                    .is_some_and(|w| !w.can_attack())
                {
                    return Err("負重過高，不能攻擊塔".into());
                }
                frame
                    .vital(actor.entity_id)
                    .ok_or("缺少戰鬥素質")?
                    .attack_sequence
            }
            siege::Action::Portal => actor.sequence,
        }
        .checked_add(1)
        .filter(|s| *s <= i64::MAX as u64)
        .ok_or("攻城操作序號已用盡")?;
        let request = siege::Request {
            context: nav.context,
            channel_id: nav.snapshot.channel_id.ok_or("缺少頻道")?,
            scope,
            sequence,
            action,
        };
        request.validate().map_err(|_| "攻城操作識別失效")?;
        if !request.ready_in(frame) {
            return Err("攻城角色、目標或傳送門位置已失效".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        if active.any_pending() {
            return Err("上一個操作尚未完成".into());
        }
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() {
            return Err("上一個操作尚未完成".into());
        }
        let command = (CombatCommand::Siege(request), self.next_selection);
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.combat = Some(command);
        active.pending_combat = Some(command);
        self.next_selection = next;
        self.view.combat_pending = true;
        self.view.status = "等待伺服器確認攻城操作…".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn attack_npc(&mut self, target: npc::Id, life: u64) -> Result<(), String> {
        self.attack_encounter(npc::Action::Attack, target.scope, target.serial, life, None)
    }
    fn attack_quest(&mut self, target: super::quest_protocol::Id) -> Result<(), String> {
        self.attack_encounter(
            npc::Action::AttackObjective,
            target.scope,
            u64::from(target.slot),
            0,
            None,
        )
    }
    fn attack_time_attack_tower(
        &mut self,
        target: super::time_attack_protocol::TowerId,
    ) -> Result<(), String> {
        self.attack_encounter(npc::Action::AttackTower, target.scope, 0, 0, None)
    }
    fn confirm_quest_death(
        &mut self,
        proof: super::quest_life::Confirmation,
    ) -> Result<(), String> {
        self.attack_encounter(
            npc::Action::ConfirmDeath,
            proof.scope,
            proof.entity_id,
            proof.incarnation,
            Some(proof.death_revision),
        )
    }
    fn attack_encounter(
        &mut self,
        action: npc::Action,
        scope: npc::Scope,
        serial: u64,
        life: u64,
        death_revision: Option<i64>,
    ) -> Result<(), String> {
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || !self.view.world_npc_enabled()
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
        {
            return Err("NPC 戰鬥尚未確認，或上一個操作尚未完成".into());
        }
        let reply = self.view.npc.as_ref().ok_or("正在等待完整 NPC 資料")?;
        let frame = reply.frame.as_ref().ok_or("NPC 場次尚未就緒")?;
        let nav = self.view.navigation.as_ref().ok_or("尚未確認角色")?;
        if self.view.world.frame() != Some(&frame.combat.world)
            || self.view.combat.as_deref() != Some(&frame.combat)
            || frame.combat.world.phase != world::Phase::Active
            || frame.resources.is_none()
            || reply.echo.context != nav.context
            || Some(reply.echo.channel_id) != nav.snapshot.channel_id
            || scope != frame.scope
            || !match action {
                npc::Action::Attack => frame
                    .actor(npc::Id { scope, serial })
                    .is_some_and(|n| n.alive() && n.life == life),
                npc::Action::AttackObjective => u8::try_from(serial).ok().is_some_and(|slot| {
                    life == 0
                        && super::quest_protocol::target(
                            &frame.combat.world,
                            super::quest_protocol::Id { scope, slot },
                        )
                        .is_some_and(|(_, a)| a.hp > 0)
                }),
                npc::Action::Read => false,
                npc::Action::AttackTower => {
                    serial == 0
                        && life == 0
                        && frame.combat.world.time_attack.as_ref().is_some_and(|d| {
                            d.phase == super::time_attack_protocol::Phase::WaitingForTower
                                && d.tower_hp > 0
                        })
                }
                npc::Action::ConfirmDeath => super::quest_life::life(&frame.combat.world, serial)
                    .is_some_and(|l| {
                        l.dead()
                            && l.incarnation() == life
                            && Some(l.death_revision) == death_revision
                    }),
            }
        {
            return Err("NPC 畫面、目標生命或場次版本已失效".into());
        }
        let actor = frame
            .combat
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("NPC 戰鬥角色不一致")?;
        let vital = frame.combat.vital(actor.entity_id).ok_or("缺少戰鬥素質")?;
        if if action == npc::Action::ConfirmDeath {
            vital.hp != 0 || actor.entity_id != serial
        } else {
            vital.hp == 0
                || !frame.combat.world.actor_occupies(actor.entity_id)
                || frame
                    .combat
                    .weight(actor.entity_id)
                    .is_some_and(|w| !w.can_attack())
        } {
            return Err("自身死亡或負重過高，不能攻擊 NPC".into());
        }
        let request = npc::Request {
            action,
            serial,
            life,
            sequence: if action == npc::Action::ConfirmDeath {
                u64::try_from(death_revision.ok_or("缺少死亡版本")?).map_err(|_| "死亡版本失效")?
            } else {
                vital
                    .attack_sequence
                    .checked_add(1)
                    .filter(|v| *v <= i64::MAX as u64)
                    .ok_or("攻擊序號已用盡")?
            },
            ..npc_read(
                nav.context,
                nav.snapshot.channel_id.ok_or("缺少頻道")?,
                &frame.combat.world,
            )
        };
        request.validate().map_err(|_| "NPC 攻擊識別失效")?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        if active.any_pending() {
            return Err("上一個操作尚未完成".into());
        }
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() {
            return Err("上一個操作尚未完成".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.npc = Some((request, self.next_selection));
        active.pending_npc = Some((request, self.next_selection));
        self.next_selection = next;
        self.view.combat_pending = true;
        self.view.status = if action == npc::Action::ConfirmDeath {
            "等待伺服器確認死亡並返回出生點…"
        } else {
            "等待伺服器確認目標攻擊…"
        }
        .into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn send_chat(&mut self, token: chat_state::Token, text: String) -> Result<(), String> {
        if self.view.chat.filtered() {
            return self.send_filtered_chat(token, filtered_chat::Kind::Normal, text);
        }
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || current_chat_scope(&self.view) != Some(token.scope)
        {
            return Err("聊天頁面已改變或正在關閉".into());
        }
        let request = self.view.chat.send(token, text)?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        inbox.chat = Some((request, self.next_selection));
        active.pending_chat = Some(self.next_selection);
        self.next_selection = next;
        self.view.chat_pending = true;
        // Chat delivery never overwrites a world/combat/navigation notice.
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn send_filtered_chat(
        &mut self,
        token: chat_state::Token,
        kind: filtered_chat::Kind,
        text: String,
    ) -> Result<(), String> {
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || current_chat_scope(&self.view) != Some(token.scope)
        {
            return Err("新版聊天頁面已改變或正在關閉".into());
        }
        let request = self.view.chat.filtered_send(token, kind, text)?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        inbox.filtered_chat = Some((request, self.next_selection));
        active.pending_chat = Some(self.next_selection);
        self.next_selection = next;
        self.view.chat_pending = true;
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn direct_chat(
        &mut self,
        token: direct_state::Token,
        action: direct::Action,
        target: Option<direct::Address>,
        preferences: Option<direct::Preferences>,
        text: String,
    ) -> Result<(), String> {
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || current_chat_scope(&self.view).is_none_or(|scope| {
                scope.context != token.enrollment.context
                    || scope.channel_id != token.enrollment.channel_id
                    || scope.room_id != token.room_id
            })
        {
            return Err("私訊頁面已改變或正在關閉".into());
        }
        let request = self
            .view
            .direct_chat
            .command(token, action, target, preferences, text)?;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        inbox.direct_chat = Some((token, request, self.next_selection));
        active.pending_direct_chat = Some(self.next_selection);
        self.next_selection = next;
        self.view.direct_chat_pending = true;
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn read_record(&mut self) -> Result<(), String> {
        if self.closing
            || self.view.record_pending
            || self.view.spell_pending
            || self.view.world_owns_lobby()
            || self.view.room_pending
            || self.view.equipment_pending
            || !matches!(
                self.view.phase,
                LoginPhase::ChoosingChannel | LoginPhase::InLobby
            )
            || matches!(self.view.record, record::State::Unsupported)
        {
            return Err("戰績讀取尚未就緒；請先等候目前操作或返回房間".into());
        }
        let context = self.view.navigation.as_ref().ok_or("尚未確認角色")?.context;
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.auto_world
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.world.is_some()
            || inbox.combat.is_some()
            || inbox.npc.is_some()
            || inbox.spell.is_some()
            || inbox.record.is_some()
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
            || active.pending_world.is_some()
            || active.pending_combat.is_some()
            || active.pending_npc.is_some()
            || active.pending_spell.is_some()
            || active.pending_record.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        inbox.record = Some((context, self.next_selection));
        active.pending_record = Some((context, self.next_selection));
        self.next_selection = next;
        self.view.record_pending = true;
        self.view.status = "正在查詢中央已保存戰績；不預先改變勝敗數。".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn cast_npc(&mut self, target: npc::Id, life: u64) -> Result<(), String> {
        if self.closing
            || self.view.phase != LoginPhase::InLobby
            || !self.view.world_npc_enabled()
            || self.view.spell_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.skills.light_arrow().is_none()
        {
            return Err("NPC 光箭尚未就緒或前一操作尚未完成".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("尚未確認角色")?;
        let f = self
            .view
            .npc
            .as_ref()
            .and_then(|r| r.frame.as_ref())
            .ok_or("缺少 NPC 同版狀態")?;
        if self.view.combat.as_deref() != Some(&f.combat)
            || self.view.world.frame() != Some(&f.combat.world)
        {
            return Err("NPC 與戰鬥快照不同版，未送出施法".into());
        }
        let a = f
            .combat
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("施法角色不一致")?;
        let clock = f.combat.world.cast(a.entity_id).ok_or("本場未開放技能")?;
        let request = npc_spell::Request {
            context: nav.context,
            channel_id: nav.snapshot.channel_id.ok_or("缺少頻道")?,
            target,
            sequence: clock
                .request_sequence
                .checked_add(1)
                .ok_or("施法序號已用盡")?,
            caster_life: super::quest_life::life(&f.combat.world, a.entity_id)
                .map_or(1, |l| l.incarnation()),
            target_life: life,
        };
        if !request.ready_in(f) {
            return Err("NPC 目標、角色生命世代或技能狀態已改變，未送出施法".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，未送出施法".into());
        }
        let queued = (Some(SpellRequest::Npc(request)), self.next_selection);
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.spell = Some(queued);
        active.pending_spell = Some(queued);
        self.next_selection = next;
        self.view.spell_pending = true;
        self.view.status = "等待伺服器確認 NPC 光箭；不預先扣魔或扣血。".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn cast_disintegrate(&mut self, request: dis::Request) -> Result<(), String> {
        if self.closing
            || self.view.spell_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.phase != LoginPhase::InLobby
            || self.view.skills.disintegrate().is_none()
        {
            return Err("究極光裂術尚未習得、來源未資格化或連線未就緒".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("缺少已確認角色")?;
        let frame = self.view.combat.as_deref().ok_or("缺少即時戰鬥資料")?;
        if self.view.world.frame() != Some(&frame.world)
            || nav.context != request.context
            || !matches!(&self.view.skills, spell::SkillsState::Ready(s) if s.context == nav.context)
            || nav.snapshot.channel_id != Some(request.channel_id)
            || !request.ready_in(frame)
        {
            return Err("究極光裂術場次、目標生命或施法序號已改變；請重新選取目標".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("施法角色不一致")?;
        let cast = frame
            .disintegrate(actor.entity_id)
            .ok_or("本場未開放究極光裂術")?;
        if cast.lawful < 500 {
            return Err("究極光裂術需要本場正義值至少500".into());
        }
        if frame
            .vital(actor.entity_id)
            .is_none_or(|v| v.mp < i64::from(cast.mp_cost))
        {
            return Err("究極光裂術魔力不足".into());
        }
        if frame
            .weight(actor.entity_id)
            .is_some_and(|w| !w.can_attack())
        {
            return Err("比賽負重過高，不能施法".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，未送出施法".into());
        }
        let queued = (
            Some(SpellRequest::Disintegrate(request)),
            self.next_selection,
        );
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.spell = Some(queued);
        active.pending_spell = Some(queued);
        self.next_selection = next;
        self.view.spell_pending = true;
        self.view.status = "等待伺服器確認究極光裂術；不預先扣魔、扣血或改正義值。".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn cast_npc_fire_storm(
        &mut self,
        request: super::npc_fire_storm_protocol::Request,
    ) -> Result<(), String> {
        if self.closing
            || self.view.spell_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.phase != LoginPhase::InLobby
            || self.view.skills.fire_storm().is_none()
        {
            return Err("NPC 火風暴尚未習得、來源未資格化或連線未就緒".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("缺少已確認角色")?;
        let frame = self.view.combat.as_deref().ok_or("缺少即時戰鬥資料")?;
        let npc = self
            .view
            .npc
            .as_ref()
            .and_then(|r| r.frame.as_ref())
            .ok_or("缺少 NPC 同版快照")?;
        if &npc.combat != frame
            || !self.view.world_npc_enabled()
            || self.view.world.frame() != Some(&frame.world)
            || nav.context != request.context
            || !matches!(&self.view.skills, spell::SkillsState::Ready(s) if s.context == nav.context)
            || nav.snapshot.channel_id != Some(request.channel_id)
            || self.view.npc.as_ref().is_none_or(|r| {
                r.echo.context != nav.context || r.echo.channel_id != request.channel_id
            })
            || !request.ready_in(npc)
        {
            return Err("NPC 火風暴場次、角色生命或施法序號已改變；請重新施放".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("施法角色不一致")?;
        let cast = frame
            .npc_fire(actor.entity_id)
            .ok_or("本場未開放NPC 火風暴")?;
        if frame
            .vital(actor.entity_id)
            .is_none_or(|v| v.mp < i64::from(cast.mp_cost))
        {
            return Err("NPC 火風暴魔力不足".into());
        }
        if frame
            .weight(actor.entity_id)
            .is_some_and(|w| !w.can_attack())
        {
            return Err("比賽負重過高，不能施法".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，未送出施法".into());
        }
        let queued = (
            Some(SpellRequest::NpcFireStorm(request)),
            self.next_selection,
        );
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.spell = Some(queued);
        active.pending_spell = Some(queued);
        self.next_selection = next;
        self.view.spell_pending = true;
        self.view.status = "等待伺服器確認NPC 火風暴；不預先扣魔、扣血。".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn cast_npc_disintegrate(
        &mut self,
        request: super::npc_disintegrate_protocol::Request,
    ) -> Result<(), String> {
        if self.closing
            || self.view.spell_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
            || self.view.phase != LoginPhase::InLobby
            || self.view.skills.disintegrate().is_none()
        {
            return Err("NPC 究極光裂術尚未習得、來源未資格化或連線未就緒".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("缺少已確認角色")?;
        let frame = self.view.combat.as_deref().ok_or("缺少即時戰鬥資料")?;
        let npc = self
            .view
            .npc
            .as_ref()
            .and_then(|r| r.frame.as_ref())
            .ok_or("缺少 NPC 同版快照")?;
        if &npc.combat != frame
            || !self.view.world_npc_enabled()
            || self.view.world.frame() != Some(&frame.world)
            || nav.context != request.context
            || !matches!(&self.view.skills, spell::SkillsState::Ready(s) if s.context == nav.context)
            || nav.snapshot.channel_id != Some(request.channel_id)
            || !request.ready_in(npc)
        {
            return Err("NPC 究極光裂術場次、目標生命或施法序號已改變；請重新選取目標".into());
        }
        let actor = frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .ok_or("施法角色不一致")?;
        let cast = frame
            .npc_disintegrate(actor.entity_id)
            .ok_or("本場未開放NPC 究極光裂術")?;
        if cast.lawful < 500 {
            return Err("NPC 究極光裂術需要本場正義值至少500".into());
        }
        if frame
            .vital(actor.entity_id)
            .is_none_or(|v| v.mp < i64::from(cast.mp_cost))
        {
            return Err("NPC 究極光裂術魔力不足".into());
        }
        if frame
            .weight(actor.entity_id)
            .is_some_and(|w| !w.can_attack())
        {
            return Err("比賽負重過高，不能施法".into());
        }
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.busy() || active.any_pending() {
            return Err("上一個操作尚未完成，未送出施法".into());
        }
        let queued = (
            Some(SpellRequest::NpcDisintegrate(request)),
            self.next_selection,
        );
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.spell = Some(queued);
        active.pending_spell = Some(queued);
        self.next_selection = next;
        self.view.spell_pending = true;
        self.view.status = "等待伺服器確認NPC 究極光裂術；不預先扣魔、扣血或改正義值。".into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn spell(&mut self, cast: Option<(u32, u64)>) -> Result<(), String> {
        if self.closing
            || self.view.spell_pending
            || self.view.world_pending
            || self.view.combat_pending
            || self.view.room_pending
            || self.view.equipment_pending
            || !matches!(
                self.view.phase,
                LoginPhase::ChoosingChannel | LoginPhase::InLobby
            )
            || matches!(self.view.skills, spell::SkillsState::Unsupported)
        {
            return Err("魔法資料尚未就緒，或上一個操作尚未完成".into());
        }
        let nav = self.view.navigation.as_ref().ok_or("尚未確認角色")?;
        let request = if let Some((skill_id, target_entity_id)) = cast {
            match skill_id {
                spell::TELEPORT => {
                    self.view
                        .skills
                        .teleport()
                        .ok_or("未學會或此伺服器尚未開放瞬間移動")?;
                }
                spell::LIGHT_ARROW => {
                    self.view
                        .skills
                        .light_arrow()
                        .ok_or("未學會或此伺服器尚未開放光箭")?;
                }
                spell::BASIC_HEAL => {
                    self.view
                        .skills
                        .basic_heal()
                        .ok_or("未學會或此伺服器尚未開放初級治癒術")?;
                }
                spell::FIRE_STORM => {
                    self.view
                        .skills
                        .fire_storm()
                        .ok_or("未學會或此伺服器尚未開放火風暴")?;
                }
                _ => return Err("此技能尚未實作，沒有送出施法".into()),
            }
            let frame = self.view.combat.as_ref().ok_or("正在等待即時戰鬥狀態")?;
            if self.view.world.frame() != Some(&frame.world)
                || frame.world.phase != world::Phase::Active
            {
                return Err("戰鬥尚未開始或畫面版本未確認".into());
            }
            let actor = frame
                .world
                .actors
                .iter()
                .find(|a| a.matches_character(nav.context.key))
                .ok_or("施法角色不一致")?;
            let vital = frame.vital(actor.entity_id).ok_or("缺少施法角色素質")?;
            if frame
                .weight(actor.entity_id)
                .is_some_and(|w| !w.can_attack())
            {
                return Err("比賽負重階段達 197：不能施法；可移動或使用補給減重".into());
            }
            if vital.hp == 0 {
                return Err("死亡角色不能施法".into());
            }
            if skill_id == spell::FIRE_STORM
                && (target_entity_id != 0 || frame.fire(actor.entity_id).is_none())
            {
                return Err("本場尚未開放火風暴；它以自身為中心，不指定目標或地面".into());
            }
            if skill_id == spell::LIGHT_ARROW
                && (frame.damage(actor.entity_id).is_none()
                    || target_entity_id == actor.entity_id
                    || super::honor_protocol::target_hp(frame, target_entity_id)
                        .is_none_or(|hp| hp == 0))
            {
                return Err("光箭目標或即時施法能力尚未確認".into());
            }
            if skill_id == spell::BASIC_HEAL
                && (frame
                    .honor
                    .as_ref()
                    .and_then(|h| h.objective(target_entity_id))
                    .is_none_or(|o| o.hp == 0)
                    || super::honor_protocol::target_team(frame, target_entity_id)
                        != super::honor_protocol::target_team(frame, actor.entity_id))
            {
                return Err("初級治癒術只可施放於本場同隊、仍存活的守護塔".into());
            }
            let state = frame
                .world
                .cast(actor.entity_id)
                .ok_or("此場遊戲尚未開放技能")?;
            Some(spell::CastRequest {
                context: nav.context,
                channel_id: nav.snapshot.channel_id.ok_or("缺少頻道")?,
                match_id: frame.world.match_id,
                generation: frame.world.generation,
                sequence: state
                    .request_sequence
                    .checked_add(1)
                    .filter(|v| *v <= i64::MAX as u64)
                    .ok_or("施法序號已用盡")?,
                skill_id,
                target_entity_id,
            })
        } else {
            None
        };
        let next = self.next_selection.checked_add(1).ok_or("操作世代已用盡")?;
        let active = self.active.as_mut().ok_or("連線已結束")?;
        let mut inbox = active
            .inbox
            .try_lock()
            .map_err(|_| "連線工作忙碌，請稍候")?;
        if inbox.shutdown
            || inbox.auto_world
            || inbox.credentials.is_some()
            || inbox.selection.is_some()
            || inbox.navigation.is_some()
            || inbox.equipment.is_some()
            || inbox.room.is_some()
            || inbox.world.is_some()
            || inbox.combat.is_some()
            || inbox.npc.is_some()
            || inbox.spell.is_some()
            || active.pending_navigation.is_some()
            || active.pending_equipment.is_some()
            || active.pending_room.is_some()
            || active.pending_world.is_some()
            || active.pending_combat.is_some()
            || active.pending_npc.is_some()
            || active.pending_spell.is_some()
        {
            return Err("上一個操作尚未完成，請稍候或中斷連線".into());
        }
        if self.queued_guard.is_some() && (inbox.busy() || active.any_pending()) {
            return Err("上一個操作尚未完成，未送出生命綁定指令".into());
        }
        inbox.action_guard = self.queued_guard;
        inbox.spell = Some((request.map(SpellRequest::Player), self.next_selection));
        active.pending_spell = Some((request.map(SpellRequest::Player), self.next_selection));
        self.next_selection = next;
        self.view.spell_pending = true;
        self.view.status = if cast.is_some() {
            "等待伺服器確認施法；不預先扣魔、扣血或改變位置。"
        } else {
            "正在讀取已學技能…"
        }
        .into();
        drop(inbox);
        let _ = active.wake.try_send(());
        Ok(())
    }
    fn retire_active(&mut self) {
        if let Some(worker) = self.active.take() {
            worker.cancel();
            self.retiring.push(worker);
        }
    }

    fn disconnect(&mut self, closing: bool) {
        self.closing |= closing;
        self.retire_active();
        self.view = empty(LoginPhase::Disconnecting, "正在結束連線…");
        self.reap();
        if self.retiring.is_empty() {
            self.view = empty(LoginPhase::Disconnected, "連線已結束");
        }
    }

    fn reap(&mut self) {
        let mut index = 0;
        while index < self.retiring.len() {
            if self.retiring[index].handle.is_finished() {
                let worker = self.retiring.swap_remove(index);
                let _ = worker.handle.join();
            } else {
                index += 1;
            }
        }
    }
}

impl LoginController for SessionController {
    fn poll(&mut self) {
        self.reap();
        if let Some(active) = self.active.as_mut() {
            let published = active.published.try_lock().ok().map(|view| view.clone());
            if let Some(published) = published {
                let mut next = published.view;
                if let Some(desired) = active.desired {
                    if published.source_epoch < desired.epoch
                        && matches!(
                            next.phase,
                            LoginPhase::ChoosingSource
                                | LoginPhase::LoadingCharacters
                                | LoginPhase::CharactersReady
                        )
                    {
                        next.phase = LoginPhase::LoadingCharacters;
                        next.status = "正在讀取選定來源的角色…".into();
                        next.selected_source = Some(desired.source);
                        next.characters.clear();
                        next.roster_epoch = 0;
                        next.navigation = None;
                    }
                }
                // Do not re-enable LoginReady while the worker has not yet
                // consumed the one queued credentials value.
                let terminal = matches!(next.phase, LoginPhase::Failed | LoginPhase::Disconnected);
                let canonical_waiting =
                    active
                        .pending_canonical
                        .as_ref()
                        .is_some_and(|(command, epoch)| {
                            !terminal
                                && (published.navigation_epoch < *epoch
                                    || !next.canonical.confirmed(command, *epoch))
                        });
                let navigation_waiting = active.pending_navigation.is_some_and(|pending| {
                    published.navigation_epoch < pending.refresh_epoch && !terminal
                }) || active
                    .pending_equipment
                    .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_room
                        .as_ref()
                        .is_some_and(|(_, epoch)| published.navigation_epoch < *epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_matchmaking
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_world
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_combat
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_npc
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_spell
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_record
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_shop
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal)
                    || active
                        .pending_item
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal)
                    || active
                        .pending_ammo
                        .is_some_and(|(_, epoch)| published.navigation_epoch < epoch && !terminal);
                let navigation_waiting = navigation_waiting
                    || active
                        .pending_chat
                        .is_some_and(|epoch| published.navigation_epoch < epoch && !terminal)
                    || active
                        .pending_direct_chat
                        .is_some_and(|epoch| published.navigation_epoch < epoch && !terminal);
                if !navigation_waiting
                    && !canonical_waiting
                    && !(self.view.phase == LoginPhase::Authenticating
                        && next.phase == LoginPhase::LoginReady)
                {
                    self.view = next;
                    active.pending_canonical = None;
                    active.pending_navigation = None;
                    active.pending_equipment = None;
                    active.pending_room = None;
                    active.pending_matchmaking = None;
                    active.pending_world = None;
                    active.pending_combat = None;
                    active.pending_npc = None;
                    active.pending_spell = None;
                    active.pending_record = None;
                    active.pending_chat = None;
                    active.pending_direct_chat = None;
                    active.pending_shop = None;
                    active.pending_item = None;
                    active.pending_ammo = None;
                }
            }
            if active.handle.is_finished() {
                let worker = self.active.take().unwrap();
                let result = worker.handle.join();
                // Terminal state may have been published between our earlier
                // snapshot and is_finished(). Re-read after the worker finishes.
                if let Ok(latest) = worker.published.try_lock() {
                    self.view = latest.view.clone();
                }
                if result.is_err()
                    || !matches!(
                        self.view.phase,
                        LoginPhase::Failed | LoginPhase::Disconnected
                    )
                {
                    self.view = empty(LoginPhase::Failed, "連線工作已異常結束，請重新連線");
                }
            }
        } else if self.view.phase == LoginPhase::Disconnecting && self.retiring.is_empty() {
            self.view = empty(LoginPhase::Disconnected, "連線已結束");
        }
    }
    fn snapshot(&self) -> LoginSnapshot {
        let mut view = self.view.clone();
        let request = super::battle_session::request_for(&view);
        view.battle.retain(request.as_ref());
        let canonical_scope = canonical::scope_for(&view);
        view.canonical.retain(canonical_scope);
        if self.closing
            || self.active.as_ref().is_some_and(|active| {
                active.pending_navigation.is_some()
                    || active.pending_room.as_ref().is_some_and(|(r, _)| {
                        matches!(
                            r.common.action,
                            rooms::Action::Leave | rooms::Action::Join | rooms::Action::Create
                        )
                    })
                    || active
                        .pending_world
                        .is_some_and(|(r, _)| r.action == world::Action::Leave)
                    || active
                        .pending_canonical
                        .as_ref()
                        .is_some_and(|(r, _)| r.leaving())
            })
        {
            // A queued departure cannot expose the previous attachment while
            // the worker is waiting for the server. Rejected commands may later
            // re-publish only their server-confirmed current attachment.
            view.battle.clear();
            view.canonical.hide();
        }
        view
    }
    fn command(&mut self, command: LoginCommand) -> Result<(), String> {
        self.poll();
        if self
            .active
            .as_ref()
            .is_some_and(|a| a.pending_canonical.is_some())
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("新版戰鬥操作尚未確認，請稍候或中斷連線".into());
        }
        let command = match command {
            LoginCommand::PaintedTarget(intent) => {
                super::battle_player::command(intent, &self.snapshot())
                    .map_err(|_| "點選的角色、生命或目標已更新；未送出指令".to_owned())?
            }
            command @ LoginCommand::PaintedLocal { .. } => {
                super::battle_player::admit_local_command(command, &self.snapshot())
                    .map_err(|_| "操作的角色、生命或場次已更新；未送出指令".to_owned())?
            }
            command => command,
        };
        if self
            .active
            .as_ref()
            .is_some_and(|a| a.pending_matchmaking.is_some())
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("配對操作尚未確認；請稍候或中斷連線".into());
        }
        if (self.view.chat_pending || self.view.direct_chat_pending)
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("聊天送出結果尚未確認，請稍候或中斷連線".into());
        }
        if (self.view.shop_pending || self.view.item_pending || self.view.ammo_pending)
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("補給操作尚未完成；請稍候或中斷連線".into());
        }
        if self.view.record_pending
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("戰績操作尚未完成；請稍候或中斷連線".into());
        }
        if self.view.spell_pending
            && !matches!(&command, LoginCommand::Disconnect | LoginCommand::Shutdown)
        {
            return Err("魔法操作尚未完成；請稍候或中斷連線".into());
        }
        if self.view.world_owns_lobby()
            && matches!(
                &command,
                LoginCommand::SelectSource(_)
                    | LoginCommand::SelectCharacter(_)
                    | LoginCommand::EnterChannel { .. }
                    | LoginCommand::ReturnCharacters(_)
                    | LoginCommand::LeaveLobby(_)
                    | LoginCommand::Equipment(_)
                    | LoginCommand::Rooms(_)
                    | LoginCommand::Matchmaking(_)
                    | LoginCommand::GmBots { .. }
                    | LoginCommand::Shop(_)
            )
        {
            return Err("請先結束世界進場並等候伺服器確認返回房間".into());
        }
        if self.canonical_mode
            && matches!(
                &command,
                LoginCommand::World(_)
                    | LoginCommand::CombatAttack { .. }
                    | LoginCommand::NpcAttack { .. }
                    | LoginCommand::QuestAttack { .. }
                    | LoginCommand::QuestRevive { .. }
                    | LoginCommand::TimeAttackTowerAttack { .. }
                    | LoginCommand::SiegeAttack { .. }
                    | LoginCommand::SiegePortal { .. }
                    | LoginCommand::UseItem { .. }
                    | LoginCommand::SelectAmmo { .. }
                    | LoginCommand::CastTeleport
                    | LoginCommand::CastLightArrow { .. }
                    | LoginCommand::CastBasicHeal { .. }
                    | LoginCommand::CastFireStorm
                    | LoginCommand::CastNpcLightArrow { .. }
                    | LoginCommand::CastDisintegrate(_)
                    | LoginCommand::CastNpcDisintegrate(_)
                    | LoginCommand::CastNpcFireStorm(_)
            )
        {
            return Err("此連線須使用完整新版戰鬥指令，不能回退舊世界回覆".into());
        }
        self.queued_guard = super::battle_action::for_command(&self.snapshot(), &command)?;
        let result = match command {
            LoginCommand::Canonical(command) => self.canonical_command(command),
            LoginCommand::PaintedLocal { .. } | LoginCommand::PaintedTarget(_) => {
                Err("無效的角色操作封裝".into())
            }
            LoginCommand::Connect => self.connect(),
            LoginCommand::Login { account, password } => self.login(account, password),
            LoginCommand::SelectSource(source) => self.select_source(source),
            LoginCommand::SelectCharacter(context) => {
                self.navigate(NavigationAction::SelectCharacter, context, None)
            }
            LoginCommand::EnterChannel {
                context,
                channel_id,
            } => self.navigate(NavigationAction::EnterChannel, context, Some(channel_id)),
            LoginCommand::ReturnCharacters(context) => {
                self.navigate(NavigationAction::ReturnCharacters, context, None)
            }
            LoginCommand::LeaveLobby(context) => {
                self.navigate(NavigationAction::LeaveLobby, context, None)
            }
            LoginCommand::Equipment(request) => self.equipment(request),
            LoginCommand::Rooms(request) => self.rooms(request),
            LoginCommand::Matchmaking(request) => self.matchmaking(request),
            LoginCommand::GmBots { count } => self.gm_bots(count),
            LoginCommand::World(request) => self.world(request),
            LoginCommand::CombatAttack { target_entity_id } => self.attack(target_entity_id),
            LoginCommand::SiegeAttack { target } => self.siege(
                target.scope,
                super::siege_protocol::Action::AttackTower(target.slot),
            ),
            LoginCommand::SiegePortal { scope } => {
                self.siege(scope, super::siege_protocol::Action::Portal)
            }
            LoginCommand::NpcAttack { target, life } => self.attack_npc(target, life),
            LoginCommand::QuestAttack { target } => self.attack_quest(target),
            LoginCommand::TimeAttackTowerAttack { target } => self.attack_time_attack_tower(target),
            LoginCommand::QuestRevive { proof } => self.confirm_quest_death(proof),
            LoginCommand::ReadSkills => self.spell(None),
            LoginCommand::ReadRecord => self.read_record(),
            LoginCommand::ChatSend { token, text } => self.send_chat(token, text),
            LoginCommand::FilteredChatSend { token, kind, text } => {
                self.send_filtered_chat(token, kind, text)
            }
            LoginCommand::DirectChat {
                token,
                action,
                target,
                preferences,
                text,
            } => self.direct_chat(token, action, target, preferences, text),
            LoginCommand::Shop(request) => self.shop(request),
            LoginCommand::UseItem { item_id } => self.use_item(item_id),
            LoginCommand::SelectAmmo { item_id } => self.select_ammo(item_id),
            LoginCommand::CastTeleport => self.spell(Some((spell::TELEPORT, 0))),
            LoginCommand::CastNpcLightArrow { target, life } => self.cast_npc(target, life),
            LoginCommand::CastLightArrow { target_entity_id } => {
                self.spell(Some((spell::LIGHT_ARROW, target_entity_id)))
            }
            LoginCommand::CastBasicHeal { target_entity_id } => {
                self.spell(Some((spell::BASIC_HEAL, target_entity_id)))
            }
            LoginCommand::CastFireStorm => self.spell(Some((spell::FIRE_STORM, 0))),
            LoginCommand::CastDisintegrate(request) => self.cast_disintegrate(request),
            LoginCommand::CastNpcDisintegrate(request) => self.cast_npc_disintegrate(request),
            LoginCommand::CastNpcFireStorm(request) => self.cast_npc_fire_storm(request),
            LoginCommand::Disconnect => {
                self.disconnect(false);
                Ok(())
            }
            LoginCommand::Shutdown => {
                self.disconnect(true);
                Ok(())
            }
        };
        self.queued_guard = None;
        result
    }
    fn is_quiescent(&self) -> bool {
        self.active.is_none() && self.retiring.is_empty()
    }
}

impl Drop for SessionController {
    fn drop(&mut self) {
        self.disconnect(true);
        // Host normally waits via poll/is_quiescent. Last-resort application
        // teardown owns and joins every worker, never detaches one.
        for worker in self.retiring.drain(..) {
            let _ = worker.handle.join();
        }
    }
}

fn publish(shared: &Mutex<Published>, view: LoginSnapshot, source_epoch: u64) {
    publish_navigation(shared, view, source_epoch, 0);
}

fn current_chat_scope(view: &LoginSnapshot) -> Option<chat_state::Scope> {
    if view.phase != LoginPhase::InLobby {
        return None;
    }
    chat_state::Scope::current(
        view.navigation.as_ref(),
        match &view.rooms {
            rooms::State::Ready(reply) => Some(reply.as_ref()),
            _ => None,
        },
    )
}
fn bind_direct(
    state: &mut direct_state::State,
    offered: bool,
    enrollment: Option<direct_state::Enrollment>,
    admission: Option<chat_state::Token>,
) -> Result<(), String> {
    let enrollment = if offered { enrollment } else { None };
    let token = enrollment.and_then(|enrollment| {
        admission
            .filter(|t| {
                t.scope.context == enrollment.context && t.scope.channel_id == enrollment.channel_id
            })
            .map(|t| direct_state::Token {
                enrollment,
                room_id: t.scope.room_id,
                membership: t.membership,
            })
    });
    state
        .bind(offered, enrollment, token)
        .map_err(str::to_string)
}
fn publish_navigation(
    shared: &Mutex<Published>,
    mut view: LoginSnapshot,
    source_epoch: u64,
    navigation_epoch: u64,
) {
    let battle_request = super::battle_session::request_for(&view);
    view.battle.retain(battle_request.as_ref());
    let canonical_scope = canonical::scope_for(&view);
    view.canonical.retain(canonical_scope);
    let scope = current_chat_scope(&view);
    view.chat.retain_scope(scope);
    // Public admission is narrower than private history. Room moves retire the
    // input token without discarding the same channel enrollment's transcript.
    let enrollment = view.direct_chat.enrollment().filter(|e| {
        view.phase == LoginPhase::InLobby
            && view.navigation.as_ref().is_some_and(|n| {
                e.context == n.context && n.snapshot.channel_id == Some(e.channel_id)
            })
    });
    let offered = view.direct_chat.supported();
    if bind_direct(
        &mut view.direct_chat,
        offered,
        enrollment,
        view.chat.token(),
    )
    .is_err()
    {
        view.direct_chat.clear();
    }
    if let Ok(mut value) = shared.lock() {
        if !matches!(view.phase, LoginPhase::Failed | LoginPhase::Disconnected) {
            view.battle_supported |= value.view.battle_supported;
            view.action_supported |= value.view.action_supported;
        }
        *value = Published {
            view,
            source_epoch,
            navigation_epoch,
        };
    }
}

fn authenticated_view(
    principal: &Principal,
    phase: LoginPhase,
    status: impl Into<String>,
) -> LoginSnapshot {
    LoginSnapshot {
        phase,
        status: status.into(),
        sources: principal
            .sources
            .iter()
            .map(|source| SourceChoice {
                id: source.id,
                label: source.name.clone(),
            })
            .collect(),
        selected_source: None,
        characters: Vec::new(),
        roster_epoch: 0,
        navigation: None,
        character_data: CharacterDataState::NotRequested,
        equipment: equipment::State::NotRequested,
        equipment_pending: false,
        rooms: rooms::State::NotRequested,
        room_pending: false,
        matchmaking: matching::State::default(),
        gm_supported: false,
        world: world::State::NotRequested,
        battle: super::battle_session::State::default(),
        canonical: canonical::State::default(),
        battle_supported: false,
        action_supported: false,
        world_pending: false,
        combat: None,
        npc: None,
        combat_pending: false,
        combat_supported: false,
        skills: spell::SkillsState::NotRequested,
        spell_pending: false,
        record: record::State::NotRequested,
        record_pending: false,
        chat: super::chat_state::State::default(),
        chat_pending: false,
        direct_chat: direct_state::State::default(),
        direct_chat_pending: false,
        shop: shop::State::NotRequested,
        shop_pending: false,
        item_pending: false,
        ammo_pending: false,
    }
}

fn navigation_view(principal: &Principal, reply: NavigationReply) -> Result<LoginSnapshot, String> {
    let source = principal
        .sources
        .iter()
        .find(|source| source.id == reply.context.key.source_id)
        .ok_or("伺服器傳回未授權的角色來源")?;
    let phase = match reply.snapshot.phase {
        NavigationPhase::Characters => return Err("返回選角必須重新讀取角色列表".into()),
        NavigationPhase::Channels => LoginPhase::ChoosingChannel,
        NavigationPhase::Lobby => LoginPhase::InLobby,
    };
    let selected = reply
        .snapshot
        .selected
        .as_ref()
        .ok_or("伺服器未提供已確認的角色")?;
    if selected.key != reply.context.key || reply.context.epoch == 0 {
        return Err("伺服器角色識別資料不一致".into());
    }
    let status = match reply.status {
        NavigationStatus::Ok => format!(
            "{}({})：伺服器已確認角色與頻道狀態。",
            selected.name, source.name
        ),
        NavigationStatus::Denied => "伺服器拒絕此次操作；保留其確認的目前狀態。".into(),
        NavigationStatus::BackendUnavailable => {
            "伺服器資料來源暫時無法使用；可重試或返回選角。".into()
        }
        NavigationStatus::Busy if reply.action == NavigationAction::ReturnCharacters => {
            "裝備保存中，請稍後再返回角色選單；目前角色與頻道保持不變。".into()
        }
        NavigationStatus::Busy => "角色或頻道目前忙碌；可重新選擇或返回選角。".into(),
    };
    let mut view = authenticated_view(principal, phase, status);
    view.selected_source = Some(source.id);
    view.navigation = Some(reply);
    Ok(view)
}

fn busy_return_view(
    principal: &Principal,
    prior: &LoginSnapshot,
    reply: NavigationReply,
) -> Result<LoginSnapshot, String> {
    if !super::transport::confirms_busy_return(prior.navigation.as_ref(), &reply) {
        return Err("返回選角的忙碌回覆改變了目前角色或頻道".into());
    }
    let confirmed = navigation_view(principal, reply)?;
    if prior.phase != confirmed.phase || prior.selected_source != confirmed.selected_source {
        return Err("返回選角的忙碌回覆與目前頁面來源不一致".into());
    }
    // Preserve source/roster scope and the same immutable data Arc. Busy never
    // exposes CharactersReady, clears the selected identity, or requests CHAR.
    let mut view = prior.clone();
    view.status = confirmed.status;
    view.navigation = confirmed.navigation;
    attach_character_data(&mut view, &prior.character_data)?;
    Ok(view)
}

fn navigation_notice(status: NavigationStatus) -> Option<&'static str> {
    match status {
        NavigationStatus::Ok => None,
        NavigationStatus::Denied => Some("伺服器拒絕選擇該角色，已重新讀取角色列表。"),
        NavigationStatus::BackendUnavailable => Some("角色資料來源暫時無法使用，請重試。"),
        NavigationStatus::Busy => Some("該角色目前使用中，請重新選擇角色。"),
    }
}

fn received_character_data(
    reply: CharacterDataReply,
    context: SelectionContext,
) -> Result<CharacterDataState, String> {
    if reply.context != context {
        return Err("來源保存資料與已選角色不一致".into());
    }
    Ok(match reply.result {
        Ok(data) => {
            if data.context != context {
                return Err("來源保存資料與已選角色不一致".into());
            }
            data.validate().map_err(|_| "來源保存資料超出合法格式")?;
            CharacterDataState::Ready(Arc::new(data))
        }
        Err(failure) => CharacterDataState::Unavailable(failure),
    })
}

fn attach_character_data(
    view: &mut LoginSnapshot,
    data: &CharacterDataState,
) -> Result<(), String> {
    if let CharacterDataState::Ready(data) = data {
        if view.navigation.as_ref().map(|reply| reply.context) != Some(data.context) {
            return Err("不可將其他角色或來源的保存資料套用到目前頁面".into());
        }
    }
    view.character_data = data.clone();
    Ok(())
}

fn attach_equipment(view: &mut LoginSnapshot, state: &equipment::State) -> Result<(), String> {
    if let equipment::State::Ready(snapshot) = state {
        if view.navigation.as_ref().map(|reply| reply.context) != Some(snapshot.source.context) {
            return Err("不可將其他角色的裝備套用到目前頁面".into());
        }
        attach_character_data(view, &CharacterDataState::Ready(snapshot.source.clone()))?;
    }
    view.equipment = state.clone();
    view.equipment_pending = false;
    Ok(())
}
fn received_equipment(
    view: &mut LoginSnapshot,
    state: &mut equipment::State,
    reply: equipment::Reply,
) -> Result<(), String> {
    reply.validate().map_err(|_| "裝備回覆格式無效")?;
    if view.navigation.as_ref().map(|reply| reply.context) != Some(reply.request.context) {
        return Err("裝備回覆與目前角色不一致".into());
    }
    let denial_notice = reply.denial_notice();
    if let Some(snapshot) = reply.snapshot {
        *state = equipment::State::Ready(snapshot);
    } else if reply.status != equipment::Status::Busy
        || !matches!(state, equipment::State::Ready(_))
    {
        *state = equipment::State::Unavailable(reply.status);
    }
    attach_equipment(view, state)?;
    view.status = match reply.status {
        equipment::Status::Ok if reply.request.action == equipment::Action::Read => {
            "已重新讀取伺服器確認的裝備與加成。".into()
        }
        equipment::Status::Ok if reply.changed => "裝備變更已由伺服器保存並確認。".into(),
        equipment::Status::Ok => "伺服器確認目前已是指定穿戴狀態，沒有重複變更。".into(),
        equipment::Status::Denied => denial_notice.ok_or("裝備拒絕原因缺失")?,
        equipment::Status::Conflict => {
            "裝備版本／來源已變更，本次未提交；已更新伺服器確認狀態，請重新確認後操作。".into()
        }
        equipment::Status::Busy => {
            "裝備操作忙碌，保留前次確認畫面；請稍後按「重新讀取裝備」。".into()
        }
        equipment::Status::BackendUnavailable => {
            "裝備來源或保存服務暫時無法使用；請重新讀取，不代表空背包。".into()
        }
        equipment::Status::BindingMismatch => {
            "裝備保存綁定與此角色來源不一致；已停用穿脫，未自動清除或重綁。".into()
        }
    };
    Ok(())
}
fn received_return_equipment(
    view: &mut LoginSnapshot,
    state: &mut equipment::State,
    reply: equipment::Reply,
) -> Result<(), String> {
    if reply.request.action != equipment::Action::Read {
        return Err("離場裝備更新只接受重新讀取，不重送穿脫".into());
    }
    let status = reply.status;
    // Busy must not re-enable an old battle token after departure. A later
    // explicit READ may recover; a SET is never automatically replayed.
    let mut refreshed = equipment::State::NotRequested;
    received_equipment(view, &mut refreshed, reply)?;
    *state = refreshed;
    if status == equipment::Status::Busy {
        view.status = "已離場，但裝備重讀忙碌；穿脫暫停，請按「重新讀取裝備」。".into();
    }
    Ok(())
}
fn load_characters(
    connection: &mut super::transport::ModernTlsConnection,
    profile: &ModernProfile,
    principal: &Principal,
    shared: &Mutex<Published>,
    selection: Selection,
    navigation_epoch: u64,
    notice: Option<&str>,
) -> Result<(), String> {
    let source = principal
        .sources
        .iter()
        .find(|source| source.id == selection.source)
        .ok_or("選定來源未授權")?;
    let prefix = notice.map(|value| format!("{value} ")).unwrap_or_default();
    let mut loading = authenticated_view(
        principal,
        LoginPhase::LoadingCharacters,
        format!("{prefix}正在讀取 {} 的角色…", source.name),
    );
    loading.selected_source = Some(source.id);
    loading.combat_supported = connection.supports_combat();
    loading.gm_supported = connection.supports_gm_bots();
    publish_navigation(shared, loading, selection.epoch, navigation_epoch);
    connection
        .set_exchange_timeout(profile.exchange_timeout())
        .map_err(|error| error.to_string())?;
    match connection.characters(selection.source, selection.epoch) {
        Ok(list) => {
            if list.source_id != selection.source || list.epoch != selection.epoch {
                return Err("角色來源／請求世代不一致".into());
            }
            let mut view = authenticated_view(
                principal,
                LoginPhase::CharactersReady,
                format!(
                    "{prefix}{}：{} 個角色，請選擇角色進入頻道。",
                    source.name,
                    list.characters.len()
                ),
            );
            view.selected_source = Some(source.id);
            view.combat_supported = connection.supports_combat();
            view.gm_supported = connection.supports_gm_bots();
            view.roster_epoch = selection.epoch;
            view.characters = list
                .characters
                .into_iter()
                .map(|character| CharacterRow {
                    source_id: character.key.source_id,
                    object_id: character.key.object_id,
                    label: format!(
                        "{}({})  Lv.{}  職業 {}",
                        character.name, source.name, character.level, character.class
                    ),
                })
                .collect();
            publish_navigation(shared, view, selection.epoch, navigation_epoch);
        }
        Err(error) if connection.principal().is_some() => {
            publish_navigation(
                shared,
                authenticated_view(
                    principal,
                    LoginPhase::ChoosingSource,
                    format!("角色讀取失敗，請重新選擇來源：{error}"),
                ),
                selection.epoch,
                navigation_epoch,
            );
        }
        Err(error) => return Err(error.to_string()),
    }
    Ok(())
}

fn run_worker(
    profile: ModernProfile,
    hello: Hello,
    extension_offer: Option<super::protocol::extensions::Offer>,
    cancel: Cancellation,
    inbox: Arc<Mutex<Inbox>>,
    shared: Arc<Mutex<Published>>,
    wake: mpsc::Receiver<()>,
) {
    let result = (|| -> Result<(), String> {
        let mut connection = connect_profile(&profile, hello, extension_offer, cancel.clone())
            .map_err(|error| error.to_string())?;
        let run = (|| -> Result<(), String> {
            publish(
                &shared,
                {
                    let mut view = empty(LoginPhase::LoginReady, "安全連線完成，請輸入帳號密碼");
                    view.battle_supported = connection.supports_battle_data();
                    view.action_supported = connection.supports_battle_actions();
                    view
                },
                0,
            );
            let mut principal: Option<Principal> = None;
            let mut character_data = CharacterDataState::NotRequested;
            let mut equipment_state = equipment::State::NotRequested;
            let mut room_state = rooms::State::NotRequested;
            let mut matchmaking_state = matching::State::Unsupported;
            let mut last_matchmaking_poll = Instant::now();
            let mut world_state = world::State::NotRequested;
            let mut canonical_state = canonical::State::default();
            let mut canonical_poller = canonical::Poller::default();
            let mut battle = super::battle_session::Tracker::new(
                connection
                    .negotiated_selection()
                    .ok_or("連線缺少完整協定確認")?,
            );
            let mut last_battle_poll = Instant::now();
            let mut skills_state = spell::SkillsState::NotRequested;
            let mut record_state = record::State::NotRequested;
            let mut chat_state = chat_state::State::default();
            let mut last_chat_poll = Instant::now();
            let mut direct_chat_state = direct_state::State::default();
            let mut direct_enrollment = None;
            let mut last_direct_poll = Instant::now();
            let mut shop_state = shop::State::NotRequested;
            let mut record_refresh: Option<RecordRefresh> = None;
            let mut last_world_poll = Instant::now();
            let mut attempted_prepare: Option<(u64, i64)> = None;
            let mut prepare_retry_after: Option<Instant> = None;
            let mut last_room_poll = Instant::now();
            let mut last_io = Instant::now();
            loop {
                if cancel.is_cancelled() {
                    return Ok(());
                }
                let (
                    canonical_request,
                    action_guard,
                    credentials,
                    selection,
                    navigation,
                    equipment_request,
                    room_request,
                    matchmaking_request,
                    gm_request,
                    world_request,
                    combat_request,
                    npc_request,
                    spell_request,
                    record_request,
                    chat_request,
                    filtered_chat_request,
                    direct_chat_request,
                    shop_request,
                    item_request,
                    ammo_request,
                    shutdown,
                ) = {
                    let mut commands = inbox.lock().map_err(|_| "連線命令狀態失效")?;
                    (
                        commands.canonical.take(),
                        commands.action_guard.take(),
                        commands.credentials.take(),
                        commands.selection.take(),
                        commands.navigation.take(),
                        commands.equipment.take(),
                        commands.room.take(),
                        commands.matchmaking.take(),
                        commands.gm.take(),
                        commands.world.take(),
                        commands.combat.take(),
                        commands.npc.take(),
                        commands.spell.take(),
                        commands.record.take(),
                        commands.chat.take(),
                        commands.filtered_chat.take(),
                        commands.direct_chat.take(),
                        commands.shop.take(),
                        commands.item.take(),
                        commands.ammo.take(),
                        commands.shutdown,
                    )
                };
                if shutdown {
                    return Ok(());
                }
                if let Some(guard) = action_guard {
                    // A rejected action can continue this loop only if no
                    // unrelated extracted command would be silently dropped.
                    if [
                        canonical_request.is_some(),
                        credentials.is_some(),
                        selection.is_some(),
                        navigation.is_some(),
                        equipment_request.is_some(),
                        room_request.is_some(),
                        matchmaking_request.is_some(),
                        gm_request.is_some(),
                        world_request.is_some(),
                        combat_request.is_some(),
                        npc_request.is_some(),
                        spell_request.is_some(),
                        record_request.is_some(),
                        chat_request.is_some(),
                        filtered_chat_request.is_some(),
                        direct_chat_request.is_some(),
                        shop_request.is_some(),
                        item_request.is_some(),
                        ammo_request.is_some(),
                    ]
                    .into_iter()
                    .filter(|v| *v)
                    .count()
                        != 1
                    {
                        return Err("生命綁定命令佇列不一致；未送出操作".into());
                    }
                    connection
                        .arm_battle_action(guard)
                        .map_err(|e| e.to_string())?;
                }
                macro_rules! action_result {
                    ($call:expr,$epoch:expr) => {
                        match $call {
                            Err(super::transport::TransportError::GuardRejected(reason)) => {
                                let recovery=connection.take_action_recovery().ok_or("操作已拒絕但缺少狀態刷新")?;
                                let mut view=shared.lock().map_err(|_|"連線發布狀態失效")?.view.clone();
                                let context=view.navigation.as_ref().ok_or("刷新缺少角色")?.context;
                                match recovery {
                                    app::Response::Npc(r)=>receive_npc(&mut view,&mut world_state,r)?,
                                    app::Response::Combat(r)=>receive_combat(&mut view,&mut world_state,&r)?,
                                    app::Response::World(r)=>{
                                        world_state=world::State::Ready(Arc::new(r));view.world=world_state.clone();
                                        view.combat=None;view.npc=None;
                                    }
                                    _=>return Err("操作拒絕回覆不是狀態讀取".into())
                                }
                                view.world_pending=false;view.combat_pending=false;view.spell_pending=false;
                                view.item_pending=false;view.ammo_pending=false;
                                // Do not reuse a pre-rejection painted life while
                                // the separate canonical follower catches up.
                                view.battle.clear();
                                view.status=format!("舊操作已捨棄（{reason:?}）；已刷新狀態，沒有重送。");
                                publish_navigation(&shared,view,context.epoch,$epoch);
                                last_io=Instant::now();last_world_poll=last_io;
                                continue;
                            }
                            other=>other
                        }
                    };
                }
                if let Some(credentials) = credentials {
                    if principal.is_some() {
                        return Err("同一連線不能重複登入".into());
                    }
                    publish(
                        &shared,
                        empty(LoginPhase::Authenticating, "正在驗證帳號…"),
                        0,
                    );
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let authenticated = connection
                        .authenticate(credentials)
                        .map_err(|error| error.to_string())?;
                    publish(
                        &shared,
                        {
                            let mut view = authenticated_view(
                                &authenticated,
                                LoginPhase::ChoosingSource,
                                "登入成功，請選擇伺服器來源。",
                            );
                            view.combat_supported = connection.supports_combat();
                            view.gm_supported = connection.supports_gm_bots();
                            view
                        },
                        0,
                    );
                    principal = Some(authenticated);
                    last_io = Instant::now();
                }
                if let Some(selection) = selection {
                    shop_state = shop::State::NotRequested;
                    record_state = record::State::NotRequested;
                    record_refresh = None;
                    let principal = principal.as_ref().ok_or("登入前不可讀取角色")?;
                    load_characters(
                        &mut connection,
                        &profile,
                        principal,
                        &shared,
                        selection,
                        0,
                        None,
                    )?;
                    last_io = Instant::now();
                }
                if let Some(request) = navigation {
                    let principal = principal.as_ref().ok_or("登入前不可選擇角色或頻道")?;
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let reply = match request.action {
                        NavigationAction::SelectCharacter => {
                            connection.select_character(request.context)
                        }
                        NavigationAction::EnterChannel => connection
                            .enter_channel(request.channel_id.ok_or("頻道命令缺少識別碼")?),
                        NavigationAction::ReturnCharacters => connection.return_characters(),
                        NavigationAction::LeaveLobby => connection.leave_lobby(),
                    }
                    .map_err(|error| error.to_string())?;
                    if reply.action != request.action || reply.context != request.context {
                        return Err("伺服器導航回覆與目前操作不一致".into());
                    }
                    if reply.status == NavigationStatus::Ok {
                        // Acknowledged navigation, not room/world READ or another
                        // completed command, owns this private enrollment generation.
                        direct_enrollment = if reply.snapshot.phase == NavigationPhase::Lobby {
                            Some(direct_state::Enrollment {
                                context: reply.context,
                                channel_id: reply.snapshot.channel_id.ok_or("缺少頻道")?,
                                generation: request.refresh_epoch,
                            })
                        } else {
                            None
                        };
                        bind_direct(
                            &mut direct_chat_state,
                            connection.supports_direct_chat(),
                            direct_enrollment,
                            None,
                        )?;
                        matchmaking_state = if connection.supports_matchmaking() {
                            matching::State::NotRequested
                        } else {
                            matching::State::Unsupported
                        };
                        last_matchmaking_poll = Instant::now();
                    }
                    if reply.snapshot.phase == NavigationPhase::Characters {
                        shop_state = shop::State::NotRequested;
                        character_data = CharacterDataState::NotRequested;
                        equipment_state = equipment::State::NotRequested;
                        room_state = rooms::State::NotRequested;
                        world_state = world::State::NotRequested;
                        skills_state = spell::SkillsState::NotRequested;
                        record_state = record::State::NotRequested;
                        record_refresh = None;
                        attempted_prepare = None;
                        // Only an acknowledged server transition reaches this path.
                        // Never reuse the old same-source roster or auto-select a row.
                        load_characters(
                            &mut connection,
                            &profile,
                            principal,
                            &shared,
                            Selection {
                                source: request.context.key.source_id,
                                epoch: request.refresh_epoch,
                            },
                            request.refresh_epoch,
                            navigation_notice(reply.status),
                        )?;
                    } else {
                        if request.action == NavigationAction::SelectCharacter
                            && reply.status == NavigationStatus::Ok
                        {
                            character_data = if connection.supports_character_data() {
                                received_character_data(
                                    connection
                                        .character_data()
                                        .map_err(|error| error.to_string())?,
                                    reply.context,
                                )?
                            } else {
                                CharacterDataState::Unsupported
                            };
                        }
                        let selected_now = reply.action == NavigationAction::SelectCharacter
                            && reply.status == NavigationStatus::Ok;
                        let mut view = if reply.action == NavigationAction::ReturnCharacters
                            && reply.status == NavigationStatus::Busy
                        {
                            let prior = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                            busy_return_view(principal, &prior, reply)?
                        } else {
                            navigation_view(principal, reply)?
                        };
                        shop_state = shop::State::negotiated(connection.supports_shop());
                        view.shop = shop_state.clone();
                        attach_character_data(&mut view, &character_data)?;
                        if selected_now {
                            if connection.supports_equipment() {
                                let context =
                                    view.navigation.as_ref().ok_or("尚未確認角色")?.context;
                                let reply = connection
                                    .equipment(equipment::Request::read(context))
                                    .map_err(|error| error.to_string())?;
                                received_equipment(&mut view, &mut equipment_state, reply)?;
                                character_data = view.character_data.clone();
                            } else {
                                equipment_state = equipment::State::Unsupported;
                                attach_equipment(&mut view, &equipment_state)?;
                            }
                        } else {
                            attach_equipment(&mut view, &equipment_state)?;
                        }
                        if selected_now {
                            skills_state = if connection.supports_spells() {
                                spell::SkillsState::Ready(Arc::new(
                                    connection.skills().map_err(|e| e.to_string())?,
                                ))
                            } else {
                                spell::SkillsState::Unsupported
                            };
                        }
                        view.skills = skills_state.clone();
                        if selected_now {
                            record_state = if connection.supports_record() {
                                let context =
                                    view.navigation.as_ref().ok_or("尚未確認角色")?.context;
                                record_refresh =
                                    Some(RecordRefresh::new(context, None, Instant::now(), false));
                                record::State::NotRequested
                            } else {
                                record::State::Unsupported
                            };
                        }
                        view.record = record_state.clone();
                        room_state = if view.phase == LoginPhase::InLobby {
                            if connection.supports_rooms() {
                                let nav = view.navigation.as_ref().ok_or("尚未確認大廳")?;
                                let request = rooms::Request::read(
                                    nav.context,
                                    nav.snapshot.channel_id.ok_or("缺少頻道")?,
                                )
                                .map_err(|_| "房間識別失效")?;
                                rooms::State::Ready(Arc::new(
                                    connection
                                        .rooms(request)
                                        .map_err(|error| error.to_string())?,
                                ))
                            } else {
                                rooms::State::Unsupported
                            }
                        } else {
                            rooms::State::NotRequested
                        };
                        view.rooms = room_state.clone();
                        view.room_pending = false;
                        view.matchmaking = matchmaking_state.clone();
                        world_state = if view.phase == LoginPhase::InLobby {
                            if connection.supports_battle_world() {
                                let nav = view.navigation.as_ref().ok_or("尚未確認大廳")?;
                                let request = super::battle_world::Request::read(
                                    nav.context,
                                    nav.snapshot.channel_id.ok_or("缺少頻道")?,
                                    None,
                                );
                                canonical_poller.exchange(&mut connection, request)?;
                                world::State::NotRequested
                            } else if connection.supports_world() {
                                let nav = view.navigation.as_ref().ok_or("尚未確認大廳")?;
                                world::State::Ready(Arc::new(
                                    connection
                                        .world(world::Request::read(
                                            nav.context,
                                            nav.snapshot.channel_id.ok_or("缺少頻道")?,
                                        ))
                                        .map_err(|error| error.to_string())?,
                                ))
                            } else {
                                world::State::Unsupported
                            }
                        } else {
                            world::State::NotRequested
                        };
                        view.world = world_state.clone();
                        canonical_state.align(&connection);
                        view.canonical = canonical_state.clone();
                        view.world_pending = false;
                        view.combat = None;
                        view.npc = None;
                        view.combat_pending = false;
                        view.combat_supported = connection.supports_combat();
                        view.gm_supported = connection.supports_gm_bots();
                        last_world_poll = Instant::now();
                        attempted_prepare = None;
                        last_room_poll = Instant::now();
                        view.direct_chat = direct_chat_state.clone();
                        publish_navigation(
                            &shared,
                            view,
                            request.context.epoch,
                            request.refresh_epoch,
                        );
                    }
                    last_io = Instant::now();
                }
                if let Some((request, epoch)) = equipment_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let reply = connection.equipment(request).map_err(|error| {
                        if request.action == equipment::Action::Set {
                            format!(
                                "裝備提交結果尚未確認；請重新連線並完整讀取，勿重複送出：{error}"
                            )
                        } else {
                            error.to_string()
                        }
                    })?;
                    if reply.request != request {
                        return Err("裝備回覆與完整請求不一致".into());
                    }
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    received_equipment(&mut view, &mut equipment_state, reply)?;
                    character_data = view.character_data.clone();
                    if request.action == equipment::Action::Set && room_state.joined().is_some() {
                        let nav = view.navigation.as_ref().ok_or("尚未確認大廳")?;
                        let read = rooms::Request::read(
                            nav.context,
                            nav.snapshot.channel_id.ok_or("缺少頻道")?,
                        )
                        .map_err(|_| "房間識別失效")?;
                        room_state = rooms::State::Ready(Arc::new(
                            connection.rooms(read).map_err(|error| error.to_string())?,
                        ));
                        last_room_poll = Instant::now();
                    }
                    view.rooms = room_state.clone();
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                }
                if let Some((request, epoch)) = room_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let context = request.common.context;
                    let action = request.common.action;
                    let reply = connection.rooms(request).map_err(|error| {
                        format!("房間操作結果尚未確認；請重新連線，不自動重送：{error}")
                    })?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    view.status = room_notice(&reply).into();
                    if reply.status == rooms::Status::Ok
                        && matches!(
                            action,
                            rooms::Action::Create | rooms::Action::Join | rooms::Action::Leave
                        )
                    {
                        matchmaking_state = if connection.supports_matchmaking() {
                            matching::State::NotRequested
                        } else {
                            matching::State::Unsupported
                        };
                    }
                    view.matchmaking = matchmaking_state.clone();
                    room_state = rooms::State::Ready(Arc::new(reply));
                    view.rooms = room_state.clone();
                    view.room_pending = false;
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_room_poll = last_io;
                }
                if let Some((request, epoch)) = matchmaking_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection
                        .matchmaking(request)
                        .map_err(|e| format!("配對結果尚未確認；請重新連線，不自動重送：{e}"))?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    receive_matchmaking(&mut view, &mut matchmaking_state, &mut room_state, reply)?;
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_matchmaking_poll = last_io;
                    last_room_poll = last_io;
                }
                if let Some((request, epoch)) = gm_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.gm_bots(request).map_err(|error| {
                        format!("假人操作結果尚未確認；請重新連線，不自動重送：{error}")
                    })?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    view.status = reply.notice();
                    if let Some(room) = reply.room {
                        room_state = rooms::State::Ready(Arc::new(room));
                    }
                    view.rooms = room_state.clone();
                    view.room_pending = false;
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_room_poll = last_io;
                }
                if let Some((request, epoch)) = world_request {
                    // A single mailbox reservation prevents GM and world admission
                    // from sharing this iteration; never synthesize bot identities.
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let reply =
                        action_result!(connection.world(request), epoch).map_err(|error| {
                            format!("世界操作結果尚未確認；請重新連線，不自動重送：{error}")
                        })?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    if reply.frame.is_some()
                        || request.action == world::Action::Leave
                            && reply.status == world::Status::Ok
                    {
                        shop_state = shop::State::negotiated(connection.supports_shop());
                        view.shop = shop_state.clone();
                    }
                    view.status = world_notice(&reply).into();
                    if request.action == world::Action::Leave && reply.status == world::Status::Ok {
                        let pending_settlement = world_state.frame().is_some_and(|frame| {
                            frame.siege.as_ref().is_some_and(|d| {
                                d.receipt
                                    .as_ref()
                                    .is_none_or(|r| r.state != world::SettlementState::Saved)
                            }) || frame.time_attack.as_ref().is_some_and(|d| {
                                d.receipt
                                    .as_ref()
                                    .is_none_or(|r| r.state != world::SettlementState::Saved)
                            }) || frame.phase == world::Phase::Finished
                                && frame.completion.as_ref().is_some_and(|completion| {
                                    completion.state != world::SettlementState::Saved
                                })
                        });
                        if connection.supports_record()
                            && world_state
                                .frame()
                                .is_some_and(|frame| frame.race.is_none())
                            && reply.frame.is_none()
                        {
                            record_refresh = Some(RecordRefresh::new(
                                request.context,
                                Some(request.channel_id),
                                Instant::now(),
                                pending_settlement,
                            ));
                        }
                        room_state = rooms::State::Ready(Arc::new(
                            connection
                                .rooms(
                                    rooms::Request::read(request.context, request.channel_id)
                                        .map_err(|_| "房間識別失效")?,
                                )
                                .map_err(|error| error.to_string())?,
                        ));
                        view.rooms = room_state.clone();
                        last_room_poll = Instant::now();
                    }
                    world_state = world::State::Ready(Arc::new(reply));
                    view.world = world_state.clone();
                    view.world_pending = false;
                    view.combat = None;
                    view.npc = None;
                    view.combat_pending = false;
                    view.combat_supported = connection.supports_combat();
                    view.gm_supported = connection.supports_gm_bots();
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((command, epoch)) = canonical_request {
                    let (context, channel) = command.scope();
                    let leaving = command.leaving();
                    let result_before = canonical_state
                        .snapshot()
                        .and_then(|s| s.observation())
                        .is_some_and(|o| o.data().result.is_some());
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    canonical_state
                        .execute(&mut connection, command, epoch)
                        .map_err(|e| {
                            format!("新版戰鬥操作未能確認；請重新連線，不自動重送：{e}")
                        })?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    let mut return_equipment_notice = None;
                    if leaving
                        && !canonical_state.attached()
                        && matches!(
                            canonical_state.outcome(),
                            Some(
                                canonical::Outcome::World(world::Status::Ok, _)
                                    | canonical::Outcome::VipWorld(world::Status::Ok, _)
                            )
                        )
                    {
                        room_state = rooms::State::Ready(Arc::new(
                            connection
                                .rooms(
                                    rooms::Request::read(context, channel)
                                        .map_err(|_| "房間識別失效")?,
                                )
                                .map_err(|e| e.to_string())?,
                        ));
                        view.rooms = room_state.clone();
                        if connection.supports_equipment() {
                            // LEAVE has been acknowledged. Refresh saved facts
                            // and CAS token; never replay the preceding SET.
                            let reply = connection
                                .equipment(equipment::Request::read(context))
                                .map_err(|e| {
                                    format!("已確認離場，但裝備重讀未完成；請重新連線：{e}")
                                })?;
                            received_return_equipment(&mut view, &mut equipment_state, reply)?;
                            character_data = view.character_data.clone();
                            return_equipment_notice = Some(view.status.clone());
                        }
                        if result_before && connection.supports_record() {
                            record_refresh = Some(RecordRefresh::new(
                                context,
                                Some(channel),
                                Instant::now(),
                                true,
                            ));
                        }
                    }
                    shop_state = shop::State::negotiated(connection.supports_shop());
                    view.shop = shop_state.clone();
                    if let Some(reply) = canonical_state.equipment_reply() {
                        // Only the exact completed receipt updates the saved
                        // CAS token. Live HP/MP always stay in canonical state.
                        received_equipment(&mut view, &mut equipment_state, reply.clone())?;
                    }
                    view.canonical = canonical_state.clone();
                    view.world_pending = false;
                    view.status = canonical_state.command_status();
                    if let Some(notice) = return_equipment_notice {
                        view.status.push_str(" ");
                        view.status.push_str(&notice);
                    }
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                    last_room_poll = last_io;
                }
                if let Some((context, epoch)) = record_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.record().map_err(|e| e.to_string())?;
                    if reply.context != context {
                        return Err("戰績回覆與請求角色不一致".into());
                    }
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    let status = reply.status;
                    received_record(&mut view, &mut record_state, reply)?;
                    view.status = if status == record::Status::Ok {
                        "中央戰績已確認；尚未完成保存的場次可能尚未計入。"
                    } else {
                        "中央戰績未更新；保留同角色上次確認值，並非零勝敗。"
                    }
                    .into();
                    if let Some(schedule) = &mut record_refresh {
                        if !schedule.current(
                            view.navigation.as_ref(),
                            view.world.frame().is_some() || view.canonical.attached(),
                        ) || !schedule.completed(status, Instant::now())
                        {
                            record_refresh = None;
                        }
                    }
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                }
                if let Some((request, epoch)) = shop_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.shop(request).map_err(|e| {
                        format!("商店結果尚未確認；請重新連線重讀，勿重複購買或賣回：{e}")
                    })?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    if !view.navigation.as_ref().is_some_and(|n| {
                        n.context == request.context
                            && n.snapshot.channel_id == Some(request.channel_id)
                    }) {
                        return Err("商店回覆的選角／頻道已失效".into());
                    }
                    view.status = reply.notice();
                    if let Some(room) = &reply.room {
                        room_state = rooms::State::Ready(Arc::new(room.clone()));
                    }
                    shop_state
                        .receive(reply)
                        .map_err(|_| "商店回覆無效，未套用")?;
                    view.shop = shop_state.clone();
                    view.shop_pending = false;
                    view.rooms = room_state.clone();
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_room_poll = last_io;
                }
                if let Some((request, epoch)) = ammo_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = action_result!(connection.select_ammo(request), epoch)
                        .map_err(|e| format!("選箭結果尚未確認；不自動重送：{e}"))?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    receive_combat(&mut view, &mut world_state, &reply.combat_reply())?;
                    view.ammo_pending = false;
                    view.status = if reply.status == ammo::Status::Ok {
                        "箭種、箭袋及重量已由伺服器同版確認。".into()
                    } else {
                        format!(
                            "伺服器未接受選箭：{:?}；未預先更換或扣除箭矢。",
                            reply.reason
                        )
                    };
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((request, epoch)) = item_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = action_result!(connection.use_item(request), epoch)
                        .map_err(|e| format!("用藥結果尚未確認；不自動重送：{e}"))?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    receive_combat(&mut view, &mut world_state, &reply.combat_reply())?;
                    view.item_pending = false;
                    view.status = if reply.status == item::Status::Ok {
                        "用藥、HP及本場剩餘補給已由伺服器同版確認。".into()
                    } else if reply.reason == item::Reason::StaleRegen {
                        "自然恢復已改變 HP，本次藥水未使用；庫存保留，不會自動重送。".into()
                    } else {
                        format!("伺服器未接受用藥：{:?}；沒有預先回血或消耗。", reply.reason)
                    };
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((request, epoch)) = combat_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let (reply, context, notice) = match request {
                        CombatCommand::Player(q) => (
                            action_result!(connection.combat(q), epoch)
                                .map_err(|e| format!("攻擊結果尚未確認；不自動重送：{e}"))?,
                            q.context,
                            None,
                        ),
                        CombatCommand::Siege(q) => {
                            let r = action_result!(connection.siege(q), epoch)
                                .map_err(|e| format!("攻城結果尚未確認；不自動重送：{e}"))?;
                            let notice = if r.status == combat::Status::Ok {
                                "攻城操作已由伺服器同版確認。".into()
                            } else {
                                format!("伺服器未接受攻城操作：{:?}；未重送。", r.reason)
                            };
                            (r.combat_reply(), q.context, Some(notice))
                        }
                    };
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    receive_combat(&mut view, &mut world_state, &reply)?;
                    view.combat_pending = false;
                    if let Some(notice) = notice {
                        view.status = notice;
                    }
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((request, epoch)) = npc_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = action_result!(connection.npc(request), epoch)
                        .map_err(|e| format!("Quest／NPC 操作結果尚未確認；不自動重送：{e}"))?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    receive_npc(&mut view, &mut world_state, reply)?;
                    view.combat_pending = false;
                    publish_navigation(&shared, view, request.context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((request, epoch)) = spell_request {
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    let context = view.navigation.as_ref().ok_or("尚未確認施法角色")?.context;
                    if let Some(SpellRequest::Disintegrate(request)) = request {
                        let reply = action_result!(connection.disintegrate(request), epoch)
                            .map_err(|e| format!("究極光裂術結果尚未確認；不自動重送：{e}"))?;
                        if reply.frame.is_some() {
                            receive_combat(&mut view, &mut world_state, &reply.combat_reply())?;
                        }
                        view.status = if reply.status == spell::Status::Ok {
                            confirmed_cast_status(
                                spell::DISINTEGRATE,
                                view.world
                                    .frame()
                                    .is_some_and(|f| f.phase == world::Phase::Finished),
                            )
                            .into()
                        } else {
                            format!("究極光裂術未提交：{:?}；沒有預先扣魔或重送。", reply.reason)
                        };
                    } else if let Some(SpellRequest::NpcFireStorm(request)) = request {
                        let reply = action_result!(connection.fire_storm_npc(request), epoch)
                            .map_err(|e| format!("NPC 火風暴結果尚未確認；不自動重送：{e}"))?;
                        receive_npc(&mut view, &mut world_state, reply.npc_reply())?;
                        view.status = if reply.status == npc::Status::Ok {
                            confirmed_cast_status(
                                spell::FIRE_STORM,
                                view.world
                                    .frame()
                                    .is_some_and(|f| f.phase == world::Phase::Finished),
                            )
                            .into()
                        } else {
                            format!("NPC 火風暴未提交：{:?}；沒有預先扣魔或重送。", reply.reason)
                        };
                    } else if let Some(SpellRequest::NpcDisintegrate(request)) = request {
                        let reply = action_result!(connection.disintegrate_npc(request), epoch)
                            .map_err(|e| format!("NPC 究極光裂術結果尚未確認；不自動重送：{e}"))?;
                        receive_npc(&mut view, &mut world_state, reply.npc_reply())?;
                        view.status = if reply.status == npc::Status::Ok {
                            confirmed_cast_status(
                                spell::DISINTEGRATE,
                                view.world
                                    .frame()
                                    .is_some_and(|f| f.phase == world::Phase::Finished),
                            )
                            .into()
                        } else {
                            format!(
                                "NPC 究極光裂術未提交：{:?}；沒有預先扣魔或重送。",
                                reply.reason
                            )
                        };
                    } else if let Some(SpellRequest::Npc(request)) = request {
                        let reply = action_result!(connection.cast_npc(request), epoch)
                            .map_err(|e| format!("NPC 光箭結果尚未確認；不自動重送：{e}"))?;
                        receive_npc(&mut view, &mut world_state, reply.npc_reply())?;
                        view.status = if reply.status == npc::Status::Ok {
                            "NPC 光箭、耗魔與目標生命值已由伺服器同版確認。".into()
                        } else {
                            format!("伺服器未接受 NPC 光箭：{:?}；未扣魔或重送。", reply.reason)
                        };
                    } else if let Some(SpellRequest::Player(request)) = request {
                        let reply = action_result!(connection.cast(request), epoch)
                            .map_err(|e| format!("施法结果尚未確認；不自動重送：{e}"))?;
                        if let Some(combat) = reply.combat_reply() {
                            receive_combat(&mut view, &mut world_state, &combat)?;
                        }
                        view.status = if reply.status == spell::Status::Ok {
                            confirmed_cast_status(
                                request.skill_id,
                                view.world
                                    .frame()
                                    .is_some_and(|f| f.phase == world::Phase::Finished),
                            )
                            .into()
                        } else if reply.reason == spell::Reason::StaleRegen {
                            "自然恢復已改變 HP／MP，本次施法未提交；沒有扣魔或傷害，不會自動重送。"
                                .into()
                        } else {
                            format!(
                                "伺服器未接受施法：{:?}；不預先扣魔、扣血或重送。",
                                reply.reason
                            )
                        };
                    } else {
                        skills_state = spell::SkillsState::Ready(Arc::new(
                            connection.skills().map_err(|e| e.to_string())?,
                        ));
                        view.skills = skills_state.clone();
                        view.status = SKILLS_CONFIRMED_STATUS.into();
                    }
                    view.spell_pending = false;
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if let Some((request, epoch)) = chat_request {
                    // Never reinterpret an old input as a send to the new page.
                    let scope = chat_state::Scope::current(
                        connection.navigation(),
                        connection.room_snapshot(),
                    );
                    chat_state.align_profile(
                        connection.supports_chat(),
                        connection.supports_filtered_chat(),
                        scope,
                    );
                    if chat_state.filtered()
                        || chat_state.token()
                            != Some(chat_state::Token {
                                scope: chat_state::Scope {
                                    context: request.context,
                                    channel_id: request.channel_id,
                                    room_id: request.room_id,
                                },
                                membership: request.membership,
                            })
                    {
                        return Err("聊天送出前角色或房間已改變；未傳送，也不會自動重送".into());
                    }
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let context = request.context;
                    let reply = connection.chat(request).map_err(|e| {
                        format!("聊天送出結果尚未確認；請重新連線，不自動重送：{e}")
                    })?;
                    chat_state.receive(reply)?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    view.chat = chat_state.clone();
                    view.chat_pending = false;
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_chat_poll = last_io;
                }
                if let Some((request, epoch)) = filtered_chat_request {
                    let scope = chat_state::Scope::current(
                        connection.navigation(),
                        connection.room_snapshot(),
                    );
                    chat_state.align_profile(
                        connection.supports_chat(),
                        connection.supports_filtered_chat(),
                        scope,
                    );
                    let token = chat_state::Token {
                        scope: chat_state::Scope {
                            context: request.context,
                            channel_id: request.channel_id,
                            room_id: request.room_id,
                        },
                        membership: request.membership,
                    };
                    if !chat_state.filtered() || chat_state.token() != Some(token) {
                        return Err("新版聊天送出前角色或房間已改變；未傳送，也不會自動重送".into());
                    }
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let context = request.context;
                    let reply = connection.filtered_chat(request).map_err(|e| {
                        format!("新版聊天送出結果尚未確認；請重新連線，不自動重送：{e}")
                    })?;
                    chat_state.receive_filtered(reply)?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    view.chat = chat_state.clone();
                    view.chat_pending = false;
                    publish_navigation(&shared, view, context.epoch, epoch);
                    last_io = Instant::now();
                    last_chat_poll = last_io;
                }
                if let Some((token, request, epoch)) = direct_chat_request {
                    let scope = chat_state::Scope::current(
                        connection.navigation(),
                        connection.room_snapshot(),
                    );
                    chat_state.align_profile(
                        connection.supports_chat(),
                        connection.supports_filtered_chat(),
                        scope,
                    );
                    bind_direct(
                        &mut direct_chat_state,
                        connection.supports_direct_chat(),
                        direct_enrollment,
                        chat_state.token(),
                    )?;
                    if direct_chat_state.token() != Some(token) {
                        return Err("私訊送出前角色或房間已改變；未傳送，也不會自動重送".into());
                    }
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let action = request.action;
                    let reply = connection.direct_chat(request).map_err(|e| {
                        let _ = direct_chat_state.retire_unknown(action);
                        format!("私訊操作結果尚未確認；請重新連線，不自動重送：{e}")
                    })?;
                    direct_chat_state.receive(token, reply)?;
                    let mut view = shared.lock().map_err(|_| "連線發布狀態失效")?.view.clone();
                    view.direct_chat = direct_chat_state.clone();
                    view.direct_chat_pending = false;
                    publish_navigation(&shared, view, token.enrollment.context.epoch, epoch);
                    last_io = Instant::now();
                    last_direct_poll = last_io;
                }
                if last_matchmaking_poll.elapsed() >= Duration::from_millis(500) {
                    if let Some(s) = matchmaking_state.waiting() {
                        let nav = connection.navigation().ok_or("尚未確認配對大廳")?;
                        let request = matching::Request::read(
                            nav.context,
                            nav.snapshot.channel_id.ok_or("缺少頻道")?,
                            s.ticket_id,
                        );
                        {
                            let mut commands = inbox.lock().map_err(|_| "連線命令狀態失效")?;
                            if commands.busy() {
                                continue;
                            }
                            // A poll can commit room membership. Reserve the same
                            // automatic-mutation barrier as world preparation.
                            commands.auto_world = true;
                        }
                        connection
                            .set_exchange_timeout(profile.exchange_timeout())
                            .map_err(|e| e.to_string())?;
                        let reply = connection
                            .matchmaking(request)
                            .map_err(|e| format!("配對入房結果尚未確認；不自動重送：{e}"))?;
                        inbox.lock().map_err(|_| "連線命令狀態失效")?.auto_world = false;
                        let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                        let mut view = previous.view;
                        receive_matchmaking(
                            &mut view,
                            &mut matchmaking_state,
                            &mut room_state,
                            reply,
                        )?;
                        publish_navigation(
                            &shared,
                            view,
                            previous.source_epoch,
                            previous.navigation_epoch,
                        );
                        last_io = Instant::now();
                        last_matchmaking_poll = last_io;
                        last_room_poll = last_io;
                    }
                }
                let room_interval = if room_state.joined().is_some() {
                    Duration::from_secs(1)
                } else {
                    Duration::from_secs(2)
                };
                let can_poll_rooms = connection.supports_rooms()
                    && connection
                        .navigation()
                        .is_some_and(|nav| nav.snapshot.phase == NavigationPhase::Lobby);
                if can_poll_rooms && last_room_poll.elapsed() >= room_interval {
                    let nav = connection.navigation().unwrap();
                    let read = rooms::Request::read(
                        nav.context,
                        nav.snapshot.channel_id.ok_or("缺少頻道")?,
                    )
                    .map_err(|_| "房間識別失效")?;
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let reply = connection.rooms(read).map_err(|error| error.to_string())?;
                    // Keep the last *completed command* epoch. A periodic read must
                    // never pretend to acknowledge a concurrently queued mutation.
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    room_state = rooms::State::Ready(Arc::new(reply));
                    view.rooms = room_state.clone();
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_io = Instant::now();
                    last_room_poll = last_io;
                }
                let can_poll_world = !connection.supports_battle_world()
                    && connection.supports_world()
                    && connection
                        .navigation()
                        .is_some_and(|nav| nav.snapshot.phase == NavigationPhase::Lobby)
                    && (room_state.joined().is_some() || world_state.frame().is_some());
                let world_interval = WORLD_POLL_INTERVAL;
                if connection.supports_battle_world() {
                    canonical_state.align(&connection);
                    if last_world_poll.elapsed() >= world_interval {
                        if let Some(request) = connection.navigation().and_then(|nav| {
                            canonical_poller.request(
                                &canonical_state,
                                nav,
                                connection.room_snapshot(),
                                Instant::now(),
                            )
                        }) {
                            let preparing = request.action == super::battle_world::Action::Prepare;
                            {
                                let mut commands = inbox.lock().map_err(|_| "連線命令狀態失效")?;
                                if commands.busy() {
                                    continue;
                                }
                                if preparing {
                                    commands.auto_world = true;
                                }
                            }
                            connection
                                .set_exchange_timeout(profile.exchange_timeout())
                                .map_err(|e| e.to_string())?;
                            canonical_poller.exchange(&mut connection, request)?;
                            if preparing {
                                inbox.lock().map_err(|_| "連線命令狀態失效")?.auto_world = false;
                            }
                            canonical_state.align(&connection);
                            last_io = Instant::now();
                        }
                        last_world_poll = Instant::now();
                    }
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    if previous.view.canonical != canonical_state {
                        let mut view = previous.view;
                        let lifecycle_key = |s: &canonical::State| {
                            s.lifecycle()
                                .map(|l| (l.phase, l.abort, l.settlement, l.race_settlement))
                        };
                        if lifecycle_key(&view.canonical) != lifecycle_key(&canonical_state) {
                            view.status = canonical_state.notice().into();
                        }
                        view.canonical = canonical_state.clone();
                        if canonical_state.attached() {
                            shop_state = shop::State::negotiated(connection.supports_shop());
                            view.shop = shop_state.clone();
                        }
                        // A READ or automatic PREPARE cannot complete a queued
                        // user command, even when its visible world is newer.
                        publish_navigation(
                            &shared,
                            view,
                            previous.source_epoch,
                            previous.navigation_epoch,
                        );
                    }
                }
                if can_poll_world
                    && last_world_poll.elapsed() >= world_interval
                    && should_poll_npc(world_state.frame(), connection.supports_npc())
                {
                    let nav = connection.navigation().ok_or("尚未確認大廳")?;
                    let request = npc_read(
                        nav.context,
                        nav.snapshot.channel_id.ok_or("缺少頻道")?,
                        world_state.frame().unwrap(),
                    );
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.npc(request).map_err(|e| e.to_string())?;
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    receive_npc(&mut view, &mut world_state, reply)?;
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if can_poll_world
                    && last_world_poll.elapsed() >= world_interval
                    && should_poll_combat(world_state.frame(), connection.supports_combat())
                {
                    let nav = connection.navigation().ok_or("尚未確認大廳")?;
                    let request = combat::Request::read(
                        nav.context,
                        nav.snapshot.channel_id.ok_or("缺少頻道")?,
                        world_state.frame().unwrap(),
                    );
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.combat(request).map_err(|e| e.to_string())?;
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    receive_combat(&mut view, &mut world_state, &reply)?;
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                if can_poll_world && last_world_poll.elapsed() >= world_interval {
                    let nav = connection.navigation().ok_or("尚未確認大廳")?;
                    let mut request = world::Request::read(
                        nav.context,
                        nav.snapshot.channel_id.ok_or("缺少頻道")?,
                    );
                    if let Some(frame) = world_state.frame() {
                        request.match_id = frame.match_id;
                        request.generation = frame.generation;
                    } else if let rooms::State::Ready(directory) = &room_state {
                        if let Some(joined) = &directory.joined {
                            let summary = &joined.summary;
                            if joined.owner == nav.context.key
                                && summary.satisfied
                                && attempted_prepare != Some((summary.room_id, summary.revision))
                                && prepare_retry_after.is_none_or(|after| Instant::now() >= after)
                                && directory.options.iter().any(|option| {
                                    option.mode == summary.settings.mode
                                        && option.map_id == summary.settings.map_id
                                        && option.world_available
                                })
                            {
                                request.action = world::Action::Prepare;
                                request.room_id = summary.room_id;
                                request.expected_room_revision = summary.revision;
                            }
                        }
                    }
                    if request.action == world::Action::Prepare {
                        let mut commands = inbox.lock().map_err(|_| "連線命令狀態失效")?;
                        if commands.busy() {
                            // Preserve this owner's user-command order: do not freeze ahead of queued LEAVE/unready/gear.
                            continue;
                        }
                        commands.auto_world = true;
                        attempted_prepare = Some((request.room_id, request.expected_room_revision));
                    }
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|error| error.to_string())?;
                    let reply = connection
                        .world(request)
                        .map_err(|error| error.to_string())?;
                    if request.action == world::Action::Prepare {
                        inbox.lock().map_err(|_| "連線命令狀態失效")?.auto_world = false;
                    }
                    if request.action == world::Action::Prepare
                        && reply.status == world::Status::Busy
                        && reply.reason == world::Reason::Capacity
                        && reply.frame.is_none()
                    {
                        // Explicit rejection before capture is safe to retry, unlike a lost/uncertain ACK.
                        attempted_prepare = None;
                        prepare_retry_after = Some(Instant::now() + Duration::from_secs(2));
                    }
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    if reply.frame.is_some() || request.action == world::Action::Prepare {
                        view.status = world_notice(&reply).into();
                    }
                    world_state = world::State::Ready(Arc::new(reply));
                    if world_state.frame().is_some() {
                        shop_state = shop::State::negotiated(connection.supports_shop());
                        view.shop = shop_state.clone();
                    }
                    view.world = world_state.clone();
                    view.combat = None;
                    view.npc = None;
                    view.combat_supported = connection.supports_combat();
                    view.gm_supported = connection.supports_gm_bots();
                    // A periodic read / automatic preparation is not ACK for a queued UI command.
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_io = Instant::now();
                    last_world_poll = last_io;
                }
                // Same authenticated owner, after gameplay work. This read
                // preserves the completed command epochs and legacy ACK caches.
                let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                let changed = battle
                    .align(super::battle_session::request_for(&previous.view))
                    .map_err(|_| "戰鬥快照的角色或場次識別失效")?;
                if !connection.supports_battle_world()
                    && (changed || last_battle_poll.elapsed() >= world_interval)
                {
                    if let Some(request) = battle.read_request() {
                        if inbox.lock().map_err(|_| "連線命令狀態失效")?.busy() {
                            continue;
                        }
                        connection
                            .set_exchange_timeout(profile.exchange_timeout())
                            .map_err(|e| e.to_string())?;
                        let reply = connection.read_battle(request).map_err(|e| e.to_string())?;
                        battle
                            .accept(reply)
                            .map_err(|_| "戰鬥快照的內容或更新順序不一致")?;
                        last_io = Instant::now();
                    }
                    if previous.view.battle != battle.state() {
                        let mut view = previous.view;
                        view.battle = battle.state();
                        publish_navigation(
                            &shared,
                            view,
                            previous.source_epoch,
                            previous.navigation_epoch,
                        );
                    }
                    last_battle_poll = Instant::now();
                }
                if last_io.elapsed() >= HEARTBEAT_INTERVAL {
                    connection
                        .set_exchange_timeout(
                            profile.exchange_timeout().min(Duration::from_secs(2)),
                        )
                        .map_err(|error| error.to_string())?;
                    connection.ping().map_err(|error| error.to_string())?;
                    last_io = Instant::now();
                }
                // Automatic records are lower priority than all commands and due
                // room/world/heartbeat IO. Never delay the SELECT/LEAVE publication.
                if record_refresh.as_ref().is_some_and(|schedule| {
                    !schedule.current(
                        connection.navigation(),
                        world_state.frame().is_some() || canonical_state.attached(),
                    )
                }) {
                    record_refresh = None;
                }
                if record_refresh
                    .as_ref()
                    .is_some_and(|schedule| Instant::now() >= schedule.due)
                {
                    let commands_waiting = {
                        let commands = inbox.lock().map_err(|_| "連線命令狀態失效")?;
                        commands.busy()
                    };
                    if commands_waiting {
                        continue;
                    }
                    if cancel.is_cancelled() {
                        return Ok(());
                    }
                    connection
                        .set_exchange_timeout(profile.exchange_timeout())
                        .map_err(|e| e.to_string())?;
                    let reply = connection.record().map_err(|e| e.to_string())?;
                    let status = reply.status;
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    received_record(&mut view, &mut record_state, reply)?;
                    let schedule = record_refresh.as_mut().ok_or("戰績刷新狀態失效")?;
                    let waiting_for_settlement = schedule.pending_settlement;
                    let repeat = schedule.completed(status, Instant::now());
                    if !repeat {
                        record_refresh = None;
                        if waiting_for_settlement || status != record::Status::Ok {
                            view.status =
                                "有限次戰績刷新已完成；保存仍可能尚未完成，可手動刷新中央戰績。"
                                    .into();
                        }
                    }
                    // Auto-read must not acknowledge a UI mutation queued during IO.
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_io = Instant::now();
                }
                // Poll only the current, confirmed audience, after game operations.
                // Keep the completed command epoch: READ must never confirm SEND.
                let scope =
                    chat_state::Scope::current(connection.navigation(), connection.room_snapshot());
                chat_state.align_profile(
                    connection.supports_chat(),
                    connection.supports_filtered_chat(),
                    scope,
                );
                if chat_state.waiting() || last_chat_poll.elapsed() >= CHAT_POLL_INTERVAL {
                    if inbox.lock().map_err(|_| "連線命令狀態失效")?.busy() {
                        continue; // A newly queued game/navigation/SEND owns the next exchange.
                    }
                    if let Some(request) = chat_state.filtered_read() {
                        connection
                            .set_exchange_timeout(profile.exchange_timeout())
                            .map_err(|e| e.to_string())?;
                        chat_state.receive_filtered(
                            connection
                                .filtered_chat(request)
                                .map_err(|e| e.to_string())?,
                        )?;
                        last_io = Instant::now();
                    } else if let Some(request) = chat_state.read() {
                        connection
                            .set_exchange_timeout(profile.exchange_timeout())
                            .map_err(|e| e.to_string())?;
                        chat_state.receive(connection.chat(request).map_err(|e| e.to_string())?)?;
                        last_io = Instant::now();
                    }
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    view.chat = chat_state.clone();
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_chat_poll = Instant::now();
                }
                bind_direct(
                    &mut direct_chat_state,
                    connection.supports_direct_chat(),
                    direct_enrollment,
                    chat_state.token(),
                )?;
                if direct_chat_state.waiting() || last_direct_poll.elapsed() >= CHAT_POLL_INTERVAL {
                    if inbox.lock().map_err(|_| "連線命令狀態失效")?.busy() {
                        continue;
                    }
                    if let Some((token, request)) = direct_chat_state.read() {
                        connection
                            .set_exchange_timeout(profile.exchange_timeout())
                            .map_err(|e| e.to_string())?;
                        direct_chat_state.receive(
                            token,
                            connection.direct_chat(request).map_err(|e| e.to_string())?,
                        )?;
                        last_io = Instant::now();
                    }
                    let previous = shared.lock().map_err(|_| "連線發布狀態失效")?.clone();
                    let mut view = previous.view;
                    view.direct_chat = direct_chat_state.clone();
                    publish_navigation(
                        &shared,
                        view,
                        previous.source_epoch,
                        previous.navigation_epoch,
                    );
                    last_direct_poll = Instant::now();
                }
                let mut wait = HEARTBEAT_INTERVAL.saturating_sub(last_io.elapsed());
                if direct_chat_state.read().is_some() {
                    wait = wait.min(CHAT_POLL_INTERVAL.saturating_sub(last_direct_poll.elapsed()));
                }
                if chat_state.read().is_some() || chat_state.filtered_read().is_some() {
                    wait = wait.min(CHAT_POLL_INTERVAL.saturating_sub(last_chat_poll.elapsed()));
                }
                if can_poll_rooms {
                    wait = wait.min(room_interval.saturating_sub(last_room_poll.elapsed()));
                }
                let canonical_ready = connection.navigation().is_some_and(|nav| {
                    canonical_poller
                        .request(
                            &canonical_state,
                            nav,
                            connection.room_snapshot(),
                            Instant::now(),
                        )
                        .is_some()
                });
                wait = world_poll_wait(
                    wait,
                    can_poll_world,
                    canonical_ready,
                    last_world_poll.elapsed(),
                );
                if battle.read_request().is_some() {
                    wait = wait.min(world_interval.saturating_sub(last_battle_poll.elapsed()));
                }
                if let Some(schedule) = &record_refresh {
                    wait = wait.min(schedule.due.saturating_duration_since(Instant::now()));
                }
                match wake.recv_timeout(wait) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Timeout) => {}
                    Err(mpsc::RecvTimeoutError::Disconnected) => return Ok(()),
                }
            }
        })();
        if run.is_ok() {
            // Only a normal return at a frame boundary may send close_notify.
            // Cancelled/failed exchanges return Err and have already aborted;
            // never drain, retry or report their unconfirmed mutations as done.
            let _ = connection.close_idle_worker();
        }
        run
    })();
    if cancel.is_cancelled() || result.is_ok() {
        publish(&shared, empty(LoginPhase::Disconnected, "連線已結束"), 0);
    } else if let Err(error) = result {
        publish(
            &shared,
            empty(LoginPhase::Failed, format!("連線／登入失敗：{error}")),
            0,
        );
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn both_world_owners_bound_sleep_without_spinning_in_an_idle_lobby() {
        let ms = Duration::from_millis;
        for elapsed in [0, 17, 199, 200, 900] {
            for (legacy, canonical) in [(true, false), (false, true), (true, true)] {
                assert_eq!(
                    world_poll_wait(ms(750), legacy, canonical, ms(elapsed)),
                    ms(200u64.saturating_sub(elapsed))
                );
                assert_eq!(
                    world_poll_wait(ms(11), legacy, canonical, ms(elapsed)),
                    ms(11.min(200u64.saturating_sub(elapsed)))
                );
            }
            assert_eq!(world_poll_wait(ms(750), false, false, ms(elapsed)), ms(750));
        }
    }

    #[test]
    fn canonical_offer_requires_the_complete_capability_dependency_chain() {
        use super::super::protocol as p;
        let hello = canonical_hello(7);
        assert_eq!((hello.min_version, hello.max_version), (39, 43));
        assert_ne!(
            hello.supported_capabilities & super::super::protocol::CAP_GREATER_HASTE,
            0
        );
        assert_eq!(
            hello.required_capabilities & super::super::protocol::CAP_GREATER_HASTE,
            0
        );
        assert_eq!(
            hello.required_capabilities,
            p::REQUIRED_CAPABILITIES
                | p::CAP_ROOMS
                | p::CAP_WORLD
                | p::CAP_MATCH_DESCRIPTOR
                | p::CAP_BATTLE_DATA
                | p::CAP_ACTION_LIFE
        );
        p::ModernClientSession::new(hello).unwrap().begin().unwrap();
        let old = game_hello(7);
        assert_eq!(old.max_version, 37);
        assert_eq!(old.required_capabilities, p::REQUIRED_CAPABILITIES);
    }
    fn ordinary_modern_profile() -> ModernProfile {
        ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":2,"gameplay_protocol":"modern_v47","profile_id":"controller-unit",
            "display_name":"Synthetic controller test","shared_resource_root":"D:\\synthetic-test",
            "endpoint":"127.0.0.1:25400","tls_server_name":"localhost","trust":{"kind":"web_pki"},
            "connect_timeout_ms":100,"exchange_timeout_ms":100
        })).unwrap()).unwrap()
    }
    #[test]
    fn ordinary_schema2_entry_owns_the_v47_controller_without_connecting() {
        let modern = ordinary_modern_profile();
        assert_ne!(modern.context_key(), profile().context_key());
        let controller = SessionController::new(modern);
        assert!(controller.canonical_mode);
        assert_eq!(controller.handshake.max_version, 47);
        assert!(controller.active.is_none() && controller.retiring.is_empty());
        assert_eq!(controller.view.phase, LoginPhase::Disconnected);
        assert!(
            controller.view.status.contains("v47") && controller.view.status.contains("尚在驗收")
        );
        assert!(!controller.view.canonical.attached() && !controller.closing);
        let hello = controller.handshake;
        assert_eq!((hello.min_version, hello.max_version), (47, 47));
        assert_eq!(
            hello.required_capabilities & super::super::protocol::CAP_SLOW_DATA,
            0
        );
        let legacy = SessionController::new(profile());
        assert!(!legacy.canonical_mode && legacy.handshake == game_hello(1));
    }
    #[test]
    fn configured_healing_uses_the_ordinary_worker_without_changing_old_entries() {
        use super::super::protocol::extensions as e;
        let extended = ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":2,"gameplay_protocol":"modern_v47","gameplay_extension":"player_healing_v1",
            "profile_id":"healing-unit","display_name":"test","shared_resource_root":"D:\\synthetic-test",
            "endpoint":"127.0.0.1:25400","tls_server_name":"localhost","trust":{"kind":"web_pki"},
            "connect_timeout_ms":100,"exchange_timeout_ms":100
        })).unwrap()).unwrap();
        let c = SessionController::new(extended.clone());
        assert!(c.canonical_mode && c.active.is_none());
        assert_eq!(c.extension_offer, Some(e::HEALING));
        assert_eq!(
            c.handshake.required_capabilities & e::DEPENDENCIES,
            e::DEPENDENCIES
        );
        super::super::protocol::ModernClientSession::new_extended(c.handshake, e::HEALING).unwrap();
        for old in [
            SessionController::new(profile()),
            SessionController::new(ordinary_modern_profile()),
            SessionController::new_slow_development(extended),
            SessionController::new_canonical(profile()),
        ] {
            assert_eq!(old.extension_offer, None);
        }
    }
    #[test]
    fn configured_extensions_require_exact_bits_without_enabling_render_or_fallback() {
        use super::super::protocol::extensions as e;
        for (name, offer) in [
            ("full_vip_v1", e::FULL_VIP),
            ("full_vip_v2", e::FULL_VIP_V2),
            ("armor_protection_v1", e::ARMOR),
            ("player_healing_armor_v1", e::HEALING_ARMOR),
            ("armor_presentation_v1", e::ARMOR_PRESENTATION),
            (
                "player_healing_armor_presentation_v1",
                e::HEALING_ARMOR_PRESENTATION,
            ),
        ] {
            let profile=ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
                "schema":2,"gameplay_protocol":"modern_v47","gameplay_extension":name,
                "profile_id":"armor-unit","display_name":"test","shared_resource_root":"D:\\synthetic-test",
                "endpoint":"127.0.0.1:25400","tls_server_name":"localhost","trust":{"kind":"web_pki"},
                "connect_timeout_ms":100,"exchange_timeout_ms":100
            })).unwrap()).unwrap();
            let c = SessionController::new(profile.clone());
            assert_eq!(c.extension_offer, Some(offer));
            assert!(c.canonical_mode && c.active.is_none());
            assert_eq!(
                c.handshake.required_capabilities & e::DEPENDENCIES,
                e::DEPENDENCIES
            );
            assert!(c.view.status.contains("尚未完成") || c.view.status.contains("尚在驗收"));
            assert!(!c.view.canonical.attached());
            super::super::protocol::ModernClientSession::new_extended(c.handshake, offer).unwrap();
            for old in [
                SessionController::new(ordinary_modern_profile()),
                SessionController::new_canonical(profile),
            ] {
                assert_eq!(old.extension_offer, None);
            }
        }
    }
    #[test]
    fn ordinary_v47_accepts_race_without_requiring_combat_shop_or_slow() {
        use super::super::protocol as p;
        let controller = SessionController::new(ordinary_modern_profile());
        let hello = controller.handshake;
        let caps = canonical_hello(1).required_capabilities | p::CAP_PET_RACE;
        let mut session = p::ModernClientSession::new(hello).unwrap();
        session.begin().unwrap();
        let ack = p::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: p::NUMERIC_SCHEMA_I64_V1,
            capabilities: caps,
        };
        assert_eq!(
            session
                .accept_ack(&p::encode(p::Message::Ack(ack)).unwrap())
                .unwrap(),
            ack
        );
    }
    #[test]
    fn ordinary_v47_preserves_common_requirements_and_exact_version() {
        use super::super::protocol as p;
        let hello = SessionController::new(ordinary_modern_profile()).handshake;
        assert_eq!(hello, modern_hello(1));
        assert_eq!(
            hello.required_capabilities,
            canonical_hello(1).required_capabilities
        );
        assert_eq!(hello.supported_capabilities, p::KNOWN_CAPABILITIES);
        let ack = p::Ack {
            request_id: 1,
            version: 47,
            numeric_schema: p::NUMERIC_SCHEMA_I64_V1,
            capabilities: p::KNOWN_CAPABILITIES,
        };
        let accepts = |bytes: &[u8]| {
            let mut s = p::ModernClientSession::new(hello).unwrap();
            s.begin().unwrap();
            let accepted = s.accept_ack(bytes).is_ok();
            assert_eq!(s.state() == p::ClientState::Closed, !accepted);
            accepted
        };
        for caps in [
            hello.required_capabilities,
            hello.required_capabilities | p::CAP_CHARACTER_DATA | p::CAP_EQUIPMENT | p::CAP_COMBAT,
            p::KNOWN_CAPABILITIES,
        ] {
            assert!(accepts(
                &p::encode(p::Message::Ack(p::Ack {
                    capabilities: caps,
                    ..ack
                }))
                .unwrap()
            ));
        }
        let bytes = p::encode(p::Message::Ack(ack)).unwrap();
        for bit in (0..32)
            .map(|n| 1u32 << n)
            .filter(|bit| hello.required_capabilities & bit != 0)
        {
            let mut missing = bytes.clone();
            missing[20..24].copy_from_slice(&(ack.capabilities & !bit).to_be_bytes());
            assert!(!accepts(&missing), "missing mandatory bit {bit:#x}");
        }
        for version in [37, 39, 41, 43, 44, 45, 46, 48] {
            let mut wrong = bytes.clone();
            wrong[16..18].copy_from_slice(&u16::to_be_bytes(version));
            assert!(!accepts(&wrong), "must not downgrade/upgrade to {version}");
        }
        // A schema2 document must not change explicitly requested historical
        // development routes, nor their stricter feature requirements.
        assert_eq!(
            SessionController::new_canonical_development(ordinary_modern_profile()).handshake,
            canonical_hello(1)
        );
        assert_eq!(
            SessionController::new_slow_development(ordinary_modern_profile()).handshake,
            target_haste_hello(1, 47)
        );
        assert_ne!(
            target_haste_hello(1, 47).required_capabilities & p::CAP_SLOW_DATA,
            0
        );
    }
    #[test]
    fn same_exe_development_entry_owns_a_fresh_canonical_controller_without_credentials() {
        let controller = SessionController::new_canonical_development(profile());
        assert!(controller.canonical_mode);
        assert_eq!(controller.handshake, canonical_hello(1));
        assert!(controller.active.is_none() && controller.retiring.is_empty());
        assert_eq!(controller.view.phase, LoginPhase::Disconnected);
        assert!(controller.view.status.contains("尚未交付"));
        assert!(!controller.view.canonical.attached());
        let ordinary = SessionController::new(profile());
        assert!(!ordinary.canonical_mode);
        assert_eq!(ordinary.handshake, game_hello(1));
        assert_eq!(ordinary.view.phase, LoginPhase::Disconnected);
    }
    #[test]
    fn target_haste_development_is_a_fresh_explicit_controller_without_credentials() {
        let controller = SessionController::new_target_haste_development(profile());
        assert!(controller.canonical_mode);
        assert_eq!(controller.handshake, target_haste_hello(1, 44));
        assert!(controller.active.is_none() && controller.retiring.is_empty());
        assert_eq!(controller.view.phase, LoginPhase::Disconnected);
        assert!(controller.view.status.contains("v44"));
        assert!(controller.view.status.contains("尚未交付"));
        assert!(!controller.view.canonical.attached());
        assert!(!controller.closing);
    }
    #[test]
    fn cooperative_haste_development_requires_v45_without_implicit_login_or_old_route_changes() {
        let controller = SessionController::new_cooperative_haste_development(profile());
        assert!(controller.canonical_mode);
        assert_eq!(controller.handshake, target_haste_hello(1, 45));
        assert!(controller.active.is_none() && controller.retiring.is_empty());
        assert_eq!(controller.view.phase, LoginPhase::Disconnected);
        assert!(
            controller.view.status.contains("v45") && controller.view.status.contains("尚未交付")
        );
        assert!(!controller.view.canonical.attached() && !controller.closing);
    }
    #[test]
    fn slow_development_starts_disconnected_and_does_not_replace_old_routes() {
        let controller = SessionController::new_slow_development(profile());
        assert!(controller.canonical_mode);
        assert_eq!(controller.handshake, target_haste_hello(1, 47));
        assert!(controller.active.is_none() && controller.retiring.is_empty());
        assert_eq!(controller.view.phase, LoginPhase::Disconnected);
        assert!(
            controller.view.status.contains("v47") && controller.view.status.contains("尚未交付")
        );
        assert!(!controller.view.canonical.attached() && !controller.closing);
        assert_eq!(
            SessionController::new_target_haste_development(profile()).handshake,
            target_haste_hello(1, 44)
        );
        assert_eq!(SessionController::new(profile()).handshake, game_hello(1));
    }
    #[test]
    fn canonical_publication_cannot_survive_wrong_navigation_or_ack_pending_departure() {
        use super::super::battle_session;
        let (mut view, _, _) = npc_view();
        let request = battle_session::request_for(&view).unwrap();
        let mut tracker = battle_session::Tracker::new(super::super::protocol::Ack {
            request_id: 1,
            version: 35,
            numeric_schema: 1,
            capabilities: super::super::protocol::KNOWN_CAPABILITIES,
        });
        tracker.align(Some(request.clone())).unwrap();
        view.battle = tracker.state();
        for field in 0..7 {
            let mut bad = view.clone();
            match field {
                0 => bad.phase = LoginPhase::ChoosingChannel,
                1 => bad.selected_source = Some(request.context.key.source_id + 1),
                2 => bad.navigation.as_mut().unwrap().context.epoch += 1,
                3 => {
                    bad.navigation.as_mut().unwrap().snapshot.channel_id =
                        Some(request.channel_id ^ 1)
                }
                _ => {
                    let world::State::Ready(reply) = &mut bad.world else {
                        unreachable!()
                    };
                    let reply = Arc::make_mut(reply);
                    match field {
                        4 => reply.echo.context.key.object_id += 1,
                        5 => reply.frame.as_mut().unwrap().generation += 1,
                        _ => reply.frame.as_mut().unwrap().match_id += 1,
                    }
                }
            }
            let shared = Mutex::new(Published {
                view: view.clone(),
                source_epoch: 1,
                navigation_epoch: 1,
            });
            publish_navigation(&shared, bad, 1, 1);
            assert!(shared.lock().unwrap().view.battle.request().is_none());
        }
        let (mut controller, release) = controller_with(view.clone());
        let read = world::Request::read(request.context, request.channel_id);
        controller.command(LoginCommand::World(read)).unwrap();
        let worker = controller.active.as_ref().unwrap();
        let (_, epoch) = worker.inbox.lock().unwrap().world.take().unwrap();
        publish_navigation(
            &worker.published,
            view.clone(),
            request.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(
            controller.snapshot().world_pending,
            "canonical READ cannot acknowledge world command"
        );
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().world_pending);
        let leave = world::Request {
            action: world::Action::Leave,
            match_id: request.world.match_id as u64,
            generation: request.world.generation as u64,
            ..read
        };
        let mut room_read = rooms::Request::read(request.context, request.channel_id)
            .unwrap()
            .expectation();
        controller.active.as_mut().unwrap().pending_room = Some((room_read.clone(), epoch + 1));
        assert_eq!(
            controller.snapshot().battle,
            view.battle,
            "room refresh must not detach the battle presenter"
        );
        room_read.common.action = rooms::Action::Leave;
        controller.active.as_mut().unwrap().pending_room = Some((room_read, epoch + 1));
        assert!(controller.snapshot().battle.request().is_none());
        controller.active.as_mut().unwrap().pending_room = None;
        controller.command(LoginCommand::World(leave)).unwrap();
        assert!(controller.snapshot().battle.request().is_none());
        assert_eq!(
            controller.snapshot().world,
            view.world,
            "departure is not optimistically acknowledged"
        );
        let worker = controller.active.as_ref().unwrap();
        let (_, epoch) = worker.inbox.lock().unwrap().world.take().unwrap();
        publish_navigation(&worker.published, view, request.context.epoch, epoch - 1);
        controller.poll();
        assert!(
            controller.snapshot().battle.request().is_none(),
            "late publication cannot revive departing data"
        );
        assert!(controller.snapshot().world_pending);
        controller.command(LoginCommand::Shutdown).unwrap();
        assert!(controller.snapshot().battle.request().is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn matchmaking_owns_one_epoch_and_never_acknowledges_from_background_read() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let mut room = rooms::tests::sample(context(11), false);
        room.common.channel_id = 2;
        room.options
            .iter_mut()
            .for_each(|o| o.world_available = true);
        view.rooms = rooms::State::Ready(Arc::new(room.clone()));
        view.matchmaking = matching::State::NotRequested;
        let request = matching::Request {
            context: context(11),
            channel_id: 2,
            action: matching::Action::Start,
            ticket_id: 0,
            criteria: Some(matching::Criteria {
                mode: 1,
                map_id: 514,
                players: 2,
            }),
        };
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::Matchmaking(request))
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        controller
            .command(LoginCommand::Matchmaking(request))
            .unwrap();
        assert!(controller.snapshot().room_pending);
        assert!(controller.snapshot().matchmaking.waiting().is_none());
        let (_, epoch) = inbox.lock().unwrap().matchmaking.take().unwrap();
        assert!(controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .is_err());
        assert!(controller
            .command(LoginCommand::Matchmaking(request))
            .is_err());
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            11,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().room_pending);
        let mut reply = matching::tests::queued();
        reply.echo = request;
        reply.snapshot.as_mut().unwrap().rooms = room;
        let mut state = matching::State::NotRequested;
        let mut room_state = view.rooms.clone();
        receive_matchmaking(&mut view, &mut state, &mut room_state, reply).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            11,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().room_pending);
        assert!(controller.snapshot().matchmaking.waiting().is_some());
        assert!(controller.snapshot().rooms.joined().is_none());
        let cancel = matching::Request {
            action: matching::Action::Cancel,
            ticket_id: 1,
            criteria: None,
            ..request
        };
        assert!(controller
            .command(LoginCommand::Matchmaking(matching::Request {
                ticket_id: 2,
                ..cancel
            }))
            .is_err());
        controller
            .command(LoginCommand::Matchmaking(cancel))
            .unwrap();
        assert!(
            controller.snapshot().matchmaking.waiting().is_some(),
            "cancel has no optimistic success"
        );
        controller.command(LoginCommand::Shutdown).unwrap();
        assert!(!controller.snapshot().matchmaking.supported());
        assert!(inbox.lock().unwrap().matchmaking.is_none());
        release.send(()).unwrap();
    }
    include!("session_chat_tests.rs");
    include!("session_filtered_chat_tests.rs");
    include!("session_direct_chat_tests.rs");
    include!("session_disintegrate_tests.rs");
    include!("session_npc_disintegrate_tests.rs");
    include!("session_npc_fire_storm_tests.rs");
    fn npc_view() -> (LoginSnapshot, world::State, npc::Reply) {
        let mut reply = npc::tests::read_reply();
        reply.frame.as_mut().unwrap().resources = Some(vec![1649, 2158]);
        encounter_view(reply)
    }
    fn encounter_view(reply: npc::Reply) -> (LoginSnapshot, world::State, npc::Reply) {
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut authenticated = principal();
        authenticated.sources.push(Source {
            id: 42,
            name: "月亮".into(),
        });
        let mut view = navigation_view(&authenticated, nav).unwrap();
        let mut state = world::State::Ready(Arc::new(reply.combat_reply().world_reply().unwrap()));
        view.world = state.clone();
        receive_npc(&mut view, &mut state, reply.clone()).unwrap();
        (view, state, reply)
    }
    #[test]
    fn siege_tower_and_portal_share_busy_cancel_epochs_without_predicting_hp_or_position() {
        use super::super::siege_protocol as siege;
        for (before, after) in [("active", "left"), ("beforeportalplain", "portalplain")] {
            let wire = siege::tests::bytes(after);
            let response = siege::decode_body(&wire[16..], 26).unwrap();
            let mut reply = siege::tests::sample(before);
            reply.echo.context = response.echo.context;
            let mut nav = navigation_reply(
                reply.echo.context,
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            );
            nav.snapshot.channel_id = Some(reply.echo.channel_id);
            let mut authenticated = principal();
            authenticated.sources.push(Source {
                id: 42,
                name: "月亮".into(),
            });
            let mut view = navigation_view(&authenticated, nav).unwrap();
            let mut ws = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
            receive_combat(&mut view, &mut ws, &reply).unwrap();
            let (mut controller, release) = controller_with(view.clone());
            let request = response.echo;
            let mut foreign = request.scope;
            foreign.generation += 1;
            assert!(controller.siege(foreign, request.action).is_err());
            controller.siege(request.scope, request.action).unwrap();
            let inbox = controller.active.as_ref().unwrap().inbox.clone();
            let (queued, epoch) = inbox.lock().unwrap().combat.take().unwrap();
            assert_eq!(queued, CombatCommand::Siege(request));
            assert_eq!(controller.snapshot().combat, view.combat);
            assert_eq!(controller.snapshot().world, view.world);
            assert!(controller.snapshot().combat_pending);
            assert!(controller.siege(request.scope, request.action).is_err());
            assert!(controller.command(LoginCommand::CastTeleport).is_err());
            receive_combat(&mut view, &mut ws, &response.combat_reply()).unwrap();
            view.combat_pending = false;
            publish_navigation(
                &controller.active.as_ref().unwrap().published,
                view.clone(),
                request.context.epoch,
                epoch,
            );
            controller.poll();
            assert!(!controller.snapshot().combat_pending);
            assert_eq!(controller.snapshot().combat, view.combat);
            controller.command(LoginCommand::Disconnect).unwrap();
            assert!(inbox.lock().unwrap().combat.is_none());
            release.send(()).unwrap();
        }
    }
    #[test]
    fn time_attack_tower_command_is_scoped_atomic_and_never_an_npc_or_pc_target() {
        let (view, _, reply) =
            encounter_view(super::super::time_attack_protocol::tests::sample("waiting"));
        let target = super::super::time_attack_protocol::TowerId {
            scope: reply.echo.world,
        };
        let (mut controller, release) = controller_with(view.clone());
        let foreign = super::super::time_attack_protocol::TowerId {
            scope: npc::Scope {
                generation: target.scope.generation + 1,
                ..target.scope
            },
        };
        assert!(controller
            .command(LoginCommand::TimeAttackTowerAttack { target: foreign })
            .is_err());
        controller
            .command(LoginCommand::TimeAttackTowerAttack { target })
            .unwrap();
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let (request, _) = inbox.lock().unwrap().npc.take().unwrap();
        assert_eq!(request.tower(), Some(target));
        assert!(request.target().is_none() && request.objective().is_none());
        assert_eq!((request.serial, request.life), (0, 0));
        assert_eq!(
            request.sequence,
            reply.frame.as_ref().unwrap().combat.vitals[0].attack_sequence + 1
        );
        assert_eq!(controller.snapshot().combat, view.combat);
        assert!(controller.snapshot().combat_pending);
        assert!(controller
            .command(LoginCommand::TimeAttackTowerAttack { target })
            .is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().npc.is_none());
        release.send(()).unwrap();
        for name in [
            "loading",
            "countdown",
            "tower",
            "wave2",
            "timeout",
            "departed",
        ] {
            let (view, _, reply) =
                encounter_view(super::super::time_attack_protocol::tests::sample(name));
            let (mut controller, release) = controller_with(view);
            assert!(
                controller
                    .command(LoginCommand::TimeAttackTowerAttack {
                        target: super::super::time_attack_protocol::TowerId {
                            scope: reply.echo.world
                        }
                    })
                    .is_err(),
                "{name}"
            );
            controller.command(LoginCommand::Disconnect).unwrap();
            release.send(()).unwrap();
        }
    }
    #[test]
    fn quest_command_uses_independent_slot_same_atomic_attack_clock_and_no_optimistic_change() {
        let (view, _, reply) =
            encounter_view(super::super::quest_protocol::tests::sample("before"));
        let target = super::super::quest_protocol::Id {
            scope: reply.echo.world,
            slot: 4,
        };
        let (mut controller, release) = controller_with(view.clone());
        for bad in [
            super::super::quest_protocol::Id { slot: 8, ..target },
            super::super::quest_protocol::Id {
                scope: npc::Scope {
                    generation: target.scope.generation + 1,
                    ..target.scope
                },
                ..target
            },
        ] {
            assert!(controller
                .command(LoginCommand::QuestAttack { target: bad })
                .is_err());
        }
        controller
            .command(LoginCommand::QuestAttack { target })
            .unwrap();
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let (request, _) = inbox.lock().unwrap().npc.take().unwrap();
        assert_eq!(request.action, npc::Action::AttackObjective);
        assert_eq!(request.objective(), Some(target));
        assert!(request.target().is_none());
        assert_eq!((request.sequence, request.serial, request.life), (5, 4, 0));
        assert_eq!(controller.snapshot().combat, view.combat);
        assert!(controller.snapshot().combat_pending);
        assert!(controller
            .command(LoginCommand::QuestAttack { target })
            .is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().npc.is_none());
        release.send(()).unwrap();
    }
    #[test]
    fn npc_controller_offer_and_polling_use_complete_v18_frames_not_pvp_reads() {
        let (view, _, _) = npc_view();
        assert_eq!(game_hello(1).max_version, 37);
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_BATTLE_DATA,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_MATCHMAKING,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_FILTERED_CHAT,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_CHAT,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_NPC_FIRE_STORM,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_NPC_DISINTEGRATE,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_DISINTEGRATE,
            0
        );
        assert_ne!(
            game_hello(1).supported_capabilities & super::super::protocol::CAP_NPC,
            0
        );
        assert!(should_poll_npc(view.world.frame(), true));
        assert!(!should_poll_npc(view.world.frame(), false));
        assert!(!should_poll_combat(view.world.frame(), true));
        let ordinary = super::super::world_runtime::tests::combat_frame();
        assert!(!should_poll_npc(Some(&ordinary.world), true));
        assert!(should_poll_combat(Some(&ordinary.world), true));
    }
    #[test]
    fn npc_preparation_failure_keeps_world_navigation_without_polling_absent_engine() {
        let (view, _, _) = npc_view();
        let mut frame = view.world.frame().unwrap().clone();
        for phase in [world::Phase::Preparing, world::Phase::Aborted] {
            frame.phase = phase;
            assert!(!should_poll_npc(Some(&frame), true));
            assert!(!should_poll_combat(Some(&frame), true));
        }
        for phase in [
            world::Phase::Loading,
            world::Phase::Countdown,
            world::Phase::Active,
            world::Phase::Finished,
        ] {
            frame.phase = phase;
            assert!(should_poll_npc(Some(&frame), true));
        }
    }
    #[test]
    fn npc_receive_rejects_foreign_source_life_plan_and_generic_loader_cannot_ack() {
        let (view, state, reply) = npc_view();
        for case in 0..4 {
            let mut bad = reply.clone();
            match case {
                0 => bad.echo.context.epoch += 1,
                1 => bad.echo.context.key.source_id = 42,
                2 => bad.frame.as_mut().unwrap().resources = None,
                _ => bad.frame.as_mut().unwrap().resources = Some(vec![1649]),
            }
            let mut current = view.clone();
            let mut world = state.clone();
            assert!(receive_npc(&mut current, &mut world, bad).is_err());
            assert_eq!(current, view);
        }
        let mut runtime = super::super::world_runtime::WorldRuntime::new(std::path::PathBuf::from(
            "D:\\synthetic-never-opened",
        ))
        .unwrap();
        let mut missing = view.clone();
        missing.npc = None;
        assert!(
            super::super::login_window::update_world_runtime(&mut runtime, &missing, 1).is_err()
        );
        assert!(runtime.is_quiescent());
        assert!(super::super::login_window::update_world_runtime(&mut runtime, &view, 1).is_ok());
        runtime.clear();
        assert!(runtime.is_quiescent()); // No poll, hence no resource worker or fabricated READY.
    }
    #[test]
    fn npc_commands_keep_typed_target_single_flight_and_poll_is_not_attack_ack() {
        let (view, _, reply) = npc_view();
        let target = npc::Id {
            scope: reply.echo.world,
            serial: reply.frame.as_ref().unwrap().actors[0].serial,
        };
        let (mut controller, release) = controller_with(view.clone());
        assert!(controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 1
            })
            .is_err());
        assert!(controller
            .command(LoginCommand::NpcAttack { target, life: 2 })
            .is_err());
        let mut foreign = target;
        foreign.scope.generation += 1;
        assert!(controller
            .command(LoginCommand::NpcAttack {
                target: foreign,
                life: 1
            })
            .is_err());
        controller
            .command(LoginCommand::NpcAttack { target, life: 1 })
            .unwrap();
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let (request, epoch) = inbox.lock().unwrap().npc.take().unwrap();
        assert_eq!(request.target().unwrap(), target);
        assert_eq!(request.sequence, 2);
        assert!(controller.snapshot().combat_pending);
        assert_eq!(controller.snapshot().combat, view.combat); // No optimistic HP or sequence.
        let published = controller.active.as_ref().unwrap().published.clone();
        publish_navigation(
            &published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().combat_pending);
        assert!(controller
            .command(LoginCommand::NpcAttack { target, life: 1 })
            .is_err());
        assert!(controller.command(LoginCommand::ReadSkills).is_err());
        let mut leave = world::Request::read(reply.echo.context, reply.echo.channel_id);
        leave.action = world::Action::Leave;
        leave.match_id = target.scope.match_id;
        leave.generation = target.scope.generation;
        assert!(controller.command(LoginCommand::World(leave)).is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(controller.snapshot().npc.is_none());
        assert!(inbox.lock().unwrap().npc.is_none());
        publish_navigation(&published, view, reply.echo.context.epoch, epoch);
        controller.poll();
        assert!(controller.snapshot().npc.is_none());
        release.send(()).unwrap();
    }
    #[test]
    fn confirmed_spell_status_names_the_actual_skill_instead_of_teleport_fallback() {
        for (id, name) in [
            (spell::TELEPORT, "瞬間移動"),
            (spell::LIGHT_ARROW, "光箭"),
            (spell::BASIC_HEAL, "初級治癒術"),
            (super::super::fire_storm_protocol::SKILL_ID, "火風暴"),
        ] {
            for finished in [false, true] {
                let message = confirmed_cast_status(id, finished);
                assert!(message.starts_with(name));
                assert!(message.contains("伺服器"));
                if id != spell::TELEPORT {
                    assert!(!message.contains("瞬間移動"));
                    assert!(!message.contains("目的地"));
                }
            }
        }
        assert_eq!(
            confirmed_cast_status(spell::TELEPORT, false),
            "瞬間移動已由伺服器確認；目的地與扣魔皆採權威結果。"
        );
        assert_eq!(
            confirmed_cast_status(spell::LIGHT_ARROW, false),
            "光箭施法、扣魔與目標生命值已由伺服器同版確認。"
        );
    }

    #[test]
    fn npc_magic_controller_keeps_typed_life_mana_and_owner_until_exact_ack() {
        let initial = npc_spell::tests::initial();
        let hit = npc_spell::tests::success();
        let mut nav = navigation_reply(
            initial.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(initial.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world =
            world::State::Ready(Arc::new(initial.combat_reply().world_reply().unwrap()));
        view.world = world.clone();
        view.combat_supported = true;
        receive_npc(&mut view, &mut world, initial.clone()).unwrap();
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: initial.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 4,
                support: spell::Support::LightArrow,
                mp_cost: 3,
                reuse_millis: 10,
                name: "光箭".into(),
            }],
        }));
        let (mut c, release) = controller_with(view.clone());
        let target = hit.echo.target;
        assert!(c
            .command(LoginCommand::CastNpcLightArrow { target, life: 2 })
            .is_err());
        assert!(c
            .command(LoginCommand::CastLightArrow {
                target_entity_id: target.serial
            })
            .is_err());
        let inbox = c.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(c
            .command(LoginCommand::CastNpcLightArrow { target, life: 1 })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        c.command(LoginCommand::CastNpcLightArrow { target, life: 1 })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().spell.take().unwrap();
        let Some(SpellRequest::Npc(request)) = request else {
            panic!("not typed NPC spell")
        };
        assert_eq!(request, hit.echo);
        assert_eq!(c.snapshot().combat, view.combat);
        assert_eq!(c.snapshot().npc, view.npc);
        assert!(c.snapshot().spell_pending);
        for command in [
            LoginCommand::NpcAttack { target, life: 1 },
            LoginCommand::ReadSkills,
            LoginCommand::CastNpcLightArrow { target, life: 1 },
            LoginCommand::CastTeleport,
        ] {
            assert!(c.command(command).is_err());
        }
        let published = c.active.as_ref().unwrap().published.clone();
        publish_navigation(&published, view.clone(), request.context.epoch, epoch - 1);
        c.poll();
        assert!(c.snapshot().spell_pending);
        receive_npc(&mut view, &mut world, hit.npc_reply()).unwrap();
        view.spell_pending = false;
        publish_navigation(&published, view.clone(), request.context.epoch, epoch);
        c.poll();
        assert!(!c.snapshot().spell_pending);
        assert_eq!(c.snapshot().combat.as_ref().unwrap().vitals[0].mp, 27);
        c.command(LoginCommand::CastNpcLightArrow { target, life: 1 })
            .unwrap();
        c.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().spell.is_none());
        publish_navigation(&published, view, request.context.epoch, epoch + 1);
        c.poll();
        assert!(c.snapshot().npc.is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn confirmed_damage_spell_finish_mentions_result_but_not_completed_persistence() {
        for id in [
            spell::LIGHT_ARROW,
            super::super::fire_storm_protocol::SKILL_ID,
        ] {
            let active = confirmed_cast_status(id, false);
            let finished = confirmed_cast_status(id, true);
            assert!(!active.contains("勝敗"));
            assert!(finished.contains("勝敗"));
            assert!(finished.contains("請查看保存狀態"));
            assert!(!finished.contains("已保存"));
        }
        for id in [0, u32::MAX] {
            assert_eq!(confirmed_cast_status(id, false), "施法結果已由伺服器確認。");
            assert_eq!(confirmed_cast_status(id, true), "施法結果已由伺服器確認。");
        }
    }

    #[test]
    fn skill_reload_confirmation_does_not_list_only_pre_v13_skills() {
        assert!(SKILLS_CONFIRMED_STATUS.contains("已學技能"));
        assert!(SKILLS_CONFIRMED_STATUS.contains("本場開放能力"));
        assert!(!SKILLS_CONFIRMED_STATUS.contains("瞬間移動與光箭"));
    }

    #[test]
    fn natural_recovery_poll_is_not_command_ack_and_cannot_publish_after_disconnect() {
        use crate::modern_lito::regen_protocol::tests::{initial, recovered};
        let original = initial();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &original).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2,
            })
            .unwrap();
        let (_, epoch) = inbox.lock().unwrap().combat.take().unwrap();
        let recovered = recovered(&original, 1, -5, -1);
        receive_combat(&mut view, &mut world, &recovered).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            original.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().combat_pending);
        assert!(controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2
            })
            .is_err());
        let published = controller.active.as_ref().unwrap().published.clone();
        controller.command(LoginCommand::Disconnect).unwrap();
        publish_navigation(&published, view, original.echo.context.epoch, epoch);
        controller.poll();
        assert!(controller.snapshot().combat.is_none());
        assert!(!controller.snapshot().combat_pending);
        assert!(inbox.lock().unwrap().combat.is_none());
        release.send(()).unwrap();
    }
    #[test]
    fn natural_recovery_from_same_numeric_object_in_other_source_is_not_applied() {
        let original = crate::modern_lito::regen_protocol::tests::initial();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &original).unwrap();
        let before = view.combat.clone();
        let previous_world = world.clone();
        let mut wrong = crate::modern_lito::regen_protocol::tests::recovered(&original, 1, -1, 0);
        wrong.echo.context.key.source_id = 42;
        assert!(receive_combat(&mut view, &mut world, &wrong).is_err());
        assert_eq!(view.combat, before);
        assert_eq!(world, previous_world);
    }
    use super::super::application_protocol::{
        ChannelSummary, CharacterKey, CharacterSummary, NavigationSnapshot, Source,
    };
    use super::*;
    fn shop_view_fixture() -> LoginSnapshot {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let mut room = rooms::tests::sample(context(11), false);
        room.common.channel_id = 2;
        view.rooms = rooms::State::Ready(Arc::new(room));
        view
    }
    #[test]
    fn shop_read_reserves_one_epoch_and_background_room_read_cannot_ack_or_replay_it() {
        let original = shop_view_fixture();
        let request = shop::Request::read(context(11), 2, None);
        let (mut controller, release) = controller_with(original.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        controller.command(LoginCommand::Shop(request)).unwrap();
        let (_, epoch) = inbox.lock().unwrap().shop.take().unwrap();
        assert!(controller.snapshot().shop_pending);
        for command in [
            LoginCommand::Shop(request),
            LoginCommand::ReadRecord,
            LoginCommand::GmBots { count: 1 },
            LoginCommand::Rooms(rooms::Request::read(context(11), 2).unwrap()),
            LoginCommand::Equipment(equipment::Request::read(context(11))),
            LoginCommand::LeaveLobby(context(11)),
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            original.clone(),
            11,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().shop_pending);
        assert_eq!(controller.snapshot().shop, shop::State::NotRequested);
        let mut completed = original;
        completed.status = "商店後端忙碌，未更新".into();
        completed.shop = shop::State::Unavailable {
            context: context(11),
            channel_id: 2,
            status: shop::Status::Busy,
        };
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            completed,
            11,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().shop_pending);
        assert!(controller.snapshot().shop.snapshot().is_none());
        controller.command(LoginCommand::Shop(request)).unwrap();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().shop.is_none());
        assert!(!controller.snapshot().shop_pending);
        release.send(()).unwrap();
    }
    #[test]
    fn shop_cannot_overtake_auto_capture_inflight_read_or_changed_room_context() {
        let original = shop_view_fixture();
        let request = shop::Request::read(context(11), 2, None);
        let (mut controller, release) = controller_with(original.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller.command(LoginCommand::Shop(request)).is_err());
        inbox.lock().unwrap().auto_world = false;
        for case in 0..3 {
            let mut invalid = request;
            match case {
                0 => invalid.context.key.source_id = 42,
                1 => invalid.context.epoch += 1,
                _ => invalid.channel_id = 1,
            }
            assert!(controller.command(LoginCommand::Shop(invalid)).is_err());
        }
        controller
            .command(LoginCommand::Rooms(
                rooms::Request::read(context(11), 2).unwrap(),
            ))
            .unwrap();
        assert!(inbox.lock().unwrap().room.take().is_some());
        assert!(controller.command(LoginCommand::Shop(request)).is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        release.send(()).unwrap();
        let mut unsupported = original;
        unsupported.shop = shop::State::Unsupported;
        let (mut controller, release) = controller_with(unsupported);
        assert!(controller.command(LoginCommand::Shop(request)).is_err());
        release.send(()).unwrap();
    }
    #[test]
    fn shop_held_mailbox_never_blocks_ui_or_creates_a_pending_purchase() {
        let (mut controller, release) = controller_with(shop_view_fixture());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let guard = inbox.lock().unwrap();
        let now = Instant::now();
        assert!(controller
            .command(LoginCommand::Shop(shop::Request::read(
                context(11),
                2,
                None
            )))
            .is_err());
        assert!(now.elapsed() < Duration::from_millis(100));
        assert!(!controller.snapshot().shop_pending);
        drop(guard);
        release.send(()).unwrap();
    }
    #[test]
    fn ammo_selection_waits_for_exact_epoch_keeps_confirmed_bag_and_does_not_replay_after_cancel() {
        let original = ammo::tests::initial();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &original).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::SelectAmmo { item_id: 40746 })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        controller
            .command(LoginCommand::SelectAmmo { item_id: 40746 })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().ammo.take().unwrap();
        assert!(controller.snapshot().ammo_pending);
        assert_eq!(controller.snapshot().combat, view.combat);
        for command in [
            LoginCommand::SelectAmmo { item_id: 40744 },
            LoginCommand::CombatAttack {
                target_entity_id: 2,
            },
            LoginCommand::UseItem { item_id: 40010 },
            LoginCommand::CastTeleport,
            LoginCommand::World(world::Request::read(
                original.echo.context,
                original.echo.channel_id,
            )),
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().ammo_pending);
        assert_eq!(controller.snapshot().combat, view.combat);
        let reply = ammo::tests::selection(&original, 40746);
        assert_eq!(reply.echo, request);
        receive_combat(&mut view, &mut world, &reply.combat_reply()).unwrap();
        view.ammo_pending = false;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().ammo_pending);
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().ammo[0].selected_item_id,
            40746
        );
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().ammo[0].stocks,
            original.frame.as_ref().unwrap().ammo[0].stocks
        );
        controller
            .command(LoginCommand::SelectAmmo { item_id: 40744 })
            .unwrap();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().ammo.is_none());
        assert!(!controller.snapshot().ammo_pending);
        assert!(controller.snapshot().combat.is_none());
        release.send(()).unwrap();
    }
    #[test]
    fn ammo_selection_never_blocks_held_mailbox_or_overtakes_an_inflight_move() {
        let original = ammo::tests::initial();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &original).unwrap();
        let (mut controller, release) = controller_with(view);
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let guard = inbox.lock().unwrap();
        let start = Instant::now();
        assert!(controller
            .command(LoginCommand::SelectAmmo { item_id: 40744 })
            .is_err());
        assert!(start.elapsed() < Duration::from_millis(100));
        assert!(!controller.snapshot().ammo_pending);
        drop(guard);
        let f = original.frame.as_ref().unwrap();
        let a = &f.world.actors[0];
        let mut request = world::Request::read(original.echo.context, original.echo.channel_id);
        request.action = world::Action::Move;
        request.match_id = f.world.match_id;
        request.generation = f.world.generation;
        request.sequence = a.sequence + 1;
        request.from_x = a.position.x;
        request.from_y = a.position.y;
        request.heading = 2;
        controller.command(LoginCommand::World(request)).unwrap();
        assert!(inbox.lock().unwrap().world.take().is_some());
        assert!(controller
            .command(LoginCommand::SelectAmmo { item_id: 40744 })
            .is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        release.send(()).unwrap();
    }
    #[test]
    fn item_use_waits_for_exact_ack_keeps_atomic_hp_and_quantity_and_cancels_without_replay() {
        let original = item::tests::initial();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &original).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::UseItem { item_id: 40010 })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        controller
            .command(LoginCommand::UseItem { item_id: 40010 })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().item.take().unwrap();
        assert!(controller.snapshot().item_pending);
        assert_eq!(controller.snapshot().combat, view.combat);
        for command in [
            LoginCommand::UseItem { item_id: 40010 },
            LoginCommand::CombatAttack {
                target_entity_id: 2,
            },
            LoginCommand::CastTeleport,
            LoginCommand::World(world::Request::read(
                original.echo.context,
                original.echo.channel_id,
            )),
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().item_pending);
        let reply = item::tests::used(&original);
        assert_eq!(reply.echo, request);
        receive_combat(&mut view, &mut world, &reply.combat_reply()).unwrap();
        view.item_pending = false;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().item_pending);
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().items[0].stocks[0].remaining,
            2
        );
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().vitals[0].hp,
            40
        );
        controller
            .command(LoginCommand::UseItem { item_id: 40010 })
            .unwrap();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().item.is_none());
        assert!(controller.snapshot().combat.is_none());
        assert!(!controller.snapshot().item_pending);
        release.send(()).unwrap();
    }

    #[test]
    fn authoritative_overweight_blocks_attack_and_cast_but_heal_ack_can_restore_admission() {
        let mut original = super::super::weight_protocol::tests::weighted_reply();
        let frame = original.frame.as_mut().unwrap();
        frame.world.casts = frame
            .world
            .actors
            .iter()
            .map(|a| spell::CastState::initial(a.entity_id, 1000))
            .collect();
        for vital in &mut frame.vitals {
            vital.mp = 100;
            vital.max_mp = 100;
        }
        original.validate().unwrap();
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world_state = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut world_state, &original).unwrap();
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: original.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 5,
                support: spell::Support::Teleport,
                mp_cost: 10,
                reuse_millis: 10,
                name: "瞬間移動".into(),
            }],
        }));
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        for command in [
            LoginCommand::CombatAttack {
                target_entity_id: 2,
            },
            LoginCommand::CastTeleport,
        ] {
            assert!(controller.command(command).unwrap_err().contains("負重"));
            assert!(!controller.snapshot().combat_pending);
            assert!(!controller.snapshot().spell_pending);
            assert!(inbox.lock().unwrap().combat.is_none());
            assert!(inbox.lock().unwrap().spell.is_none());
        }
        controller
            .command(LoginCommand::UseItem { item_id: 40010 })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().item.take().unwrap();
        assert_eq!(controller.snapshot().combat, view.combat);
        let mut used = item::tests::used(&original);
        let f = used.frame.as_mut().unwrap();
        let w = &mut f.weights[0].weight;
        w.supplies_weight -= 1;
        w.total -= 1;
        w.stage = 196;
        w.hit_penalty = 5;
        assert_eq!(used.echo, request);
        used.validate().unwrap();
        receive_combat(&mut view, &mut world_state, &used.combat_reply()).unwrap();
        view.item_pending = false;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().item_pending);
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            request.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().item_pending);
        assert_eq!(
            controller
                .snapshot()
                .combat
                .as_ref()
                .unwrap()
                .weight(1)
                .unwrap()
                .stage,
            196
        );
        controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2,
            })
            .unwrap();
        assert!(controller.snapshot().combat_pending);
        release.send(()).unwrap();

        // A heavy actor may MOVE; weight never changes frozen movement pacing.
        let mut nav = navigation_reply(
            original.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(original.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut state = world::State::Ready(Arc::new(original.world_reply().unwrap()));
        receive_combat(&mut view, &mut state, &original).unwrap();
        let (mut moving, release) = controller_with(view);
        let f = original.frame.as_ref().unwrap();
        let actor = &f.world.actors[0];
        let request = world::Request {
            action: world::Action::Move,
            heading: 2,
            match_id: f.world.match_id,
            generation: f.world.generation,
            sequence: actor.sequence + 1,
            from_x: actor.position.x,
            from_y: actor.position.y,
            ..world::Request::read(original.echo.context, original.echo.channel_id)
        };
        moving.command(LoginCommand::World(request)).unwrap();
        assert!(moving.snapshot().world_pending);
        release.send(()).unwrap();
    }

    #[test]
    fn record_refresh_uses_finite_exact_backoff_without_inferring_settlement_from_totals() {
        let now = Instant::now();
        let mut schedule = RecordRefresh::new(context(11), Some(2), now, true);
        let mut at = now;
        for seconds in [1, 2, 4, 8, 15] {
            assert_eq!(schedule.due, at);
            assert!(
                schedule.completed(record::Status::Ok, at),
                "even changed SQL totals cannot identify the pending match UUID"
            );
            assert_eq!(schedule.due, at + Duration::from_secs(seconds));
            assert!(at < schedule.due);
            at = schedule.due;
        }
        assert_eq!(schedule.retries, 5);
        assert!(!schedule.completed(record::Status::Busy, at));
        assert!(!schedule.completed(record::Status::Unavailable, at + Duration::from_secs(1000)));
        assert_eq!(schedule.retries, 5);
        let mut saved = RecordRefresh::new(context(11), Some(2), now, false);
        assert!(!saved.completed(record::Status::Ok, now));
        let mut failed = RecordRefresh::new(context(11), None, now, false);
        assert!(failed.completed(record::Status::Unavailable, now));
        assert_eq!(failed.due, now + Duration::from_secs(1));
        assert!(!failed.completed(record::Status::Ok, failed.due));
    }

    #[test]
    fn record_refresh_cannot_survive_identity_epoch_channel_or_new_world() {
        let now = Instant::now();
        let original = navigation_reply(
            context(11),
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        let schedule = RecordRefresh::new(context(11), Some(2), now, true);
        assert!(schedule.current(Some(&original), false));
        assert!(!schedule.current(Some(&original), true));
        assert!(!schedule.current(None, false));
        for field in 0..5 {
            let mut nav = original.clone();
            match field {
                0 => nav.context.key.source_id = 42,
                1 => nav.context.key.object_id += 1,
                2 => nav.context.epoch += 1,
                3 => nav.snapshot.channel_id = Some(1),
                _ => {
                    nav.snapshot.phase = NavigationPhase::Channels;
                    nav.snapshot.channel_id = None;
                }
            }
            assert!(!schedule.current(Some(&nav), false));
        }
        // The very first selection lookup can follow that same selection into a
        // channel. A settlement retry, unlike it, is pinned to its return lobby.
        let initial = RecordRefresh::new(context(11), None, now, false);
        assert!(initial.current(Some(&original), false));
        let channels = navigation_reply(
            context(11),
            NavigationAction::SelectCharacter,
            NavigationPhase::Channels,
        );
        assert!(initial.current(Some(&channels), false));
    }

    fn record_reply(context: SelectionContext) -> record::Reply {
        record::Reply {
            context,
            status: record::Status::Ok,
            totals: Some(record::Totals {
                stored: true,
                wins: 4_000_000_000,
                losses: 3,
                revision: 4_000_000_003,
            }),
        }
    }
    #[test]
    fn record_received_rejects_wrong_source_object_or_epoch_without_touching_confirmed_values() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let mut state = record::State::NotRequested;
        received_record(&mut view, &mut state, record_reply(context(11))).unwrap();
        let before = view.clone();
        for field in 0..3 {
            let mut reply = record_reply(context(11));
            match field {
                0 => reply.context.key.source_id = 42,
                1 => reply.context.key.object_id += 1,
                _ => reply.context.epoch += 1,
            }
            assert!(received_record(&mut view, &mut state, reply).is_err());
            assert_eq!(view, before);
            assert_eq!(state, before.record);
        }
        received_record(
            &mut view,
            &mut state,
            record::Reply {
                context: context(11),
                status: record::Status::Busy,
                totals: None,
            },
        )
        .unwrap();
        assert_eq!(view.record.totals(), before.record.totals());
        assert!(matches!(
            view.record,
            record::State::Ready {
                refresh_failure: Some(record::Status::Busy),
                ..
            }
        ));
    }

    #[test]
    fn record_command_holds_single_flight_until_exact_epoch_and_disconnect_clears_owned_work() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let mut state = record::State::NotRequested;
        received_record(&mut view, &mut state, record_reply(context(11))).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        controller.command(LoginCommand::ReadRecord).unwrap();
        let active = controller.active.as_ref().unwrap();
        let inbox = active.inbox.clone();
        let (context, epoch) = inbox.lock().unwrap().record.take().unwrap();
        assert_eq!(context, view.navigation.as_ref().unwrap().context);
        assert_eq!(active.pending_record, Some((context, epoch)));
        assert_eq!(controller.snapshot().record, view.record);
        for command in [
            LoginCommand::ReadRecord,
            LoginCommand::ReadSkills,
            LoginCommand::CastTeleport,
            LoginCommand::ReturnCharacters(context),
            LoginCommand::EnterChannel {
                context,
                channel_id: 0,
            },
            LoginCommand::Equipment(equipment::Request::read(context)),
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().record_pending);
        received_record(
            &mut view,
            &mut state,
            record::Reply {
                context,
                status: record::Status::Unavailable,
                totals: None,
            },
        )
        .unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().record_pending);
        assert_eq!(controller.snapshot().record, view.record);
        controller.command(LoginCommand::ReadRecord).unwrap();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().record.is_none());
        assert_eq!(controller.snapshot().record, record::State::NotRequested);
        assert!(!controller.snapshot().record_pending);
        release.send(()).unwrap();
    }

    #[test]
    fn record_request_never_overtakes_auto_prepare_or_inflight_skill_read_and_old_versions_are_explicit(
    ) {
        let view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller.command(LoginCommand::ReadRecord).is_err());
        inbox.lock().unwrap().auto_world = false;
        controller.command(LoginCommand::ReadSkills).unwrap();
        assert!(controller.command(LoginCommand::ReadRecord).is_err());
        assert!(inbox.lock().unwrap().spell.take().is_some());
        assert!(controller.command(LoginCommand::ReadRecord).is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        release.send(()).unwrap();
        let mut unsupported = view;
        unsupported.record = record::State::Unsupported;
        let (mut controller, release) = controller_with(unsupported);
        assert!(controller.command(LoginCommand::ReadRecord).is_err());
        assert!(!controller.snapshot().record_pending);
        release.send(()).unwrap();
    }

    fn profile() -> ModernProfile {
        ModernProfile::parse(&serde_json::to_vec(&serde_json::json!({
            "schema":1,"profile_id":"controller-unit","display_name":"Synthetic controller test",
            "shared_resource_root":"D:\\synthetic-test","endpoint":"127.0.0.1:25400",
            "tls_server_name":"localhost","trust":{"kind":"web_pki"},
            "connect_timeout_ms":100,"exchange_timeout_ms":100
        })).unwrap()).unwrap()
    }

    fn sources(phase: LoginPhase) -> LoginSnapshot {
        let mut view = empty(phase, "synthetic status");
        view.sources = vec![
            SourceChoice {
                id: 7,
                label: "太陽神".into(),
            },
            SourceChoice {
                id: 42,
                label: "月亮".into(),
            },
        ];
        view
    }

    fn row(source: u32, epoch: u64) -> Published {
        let mut view = sources(LoginPhase::CharactersReady);
        view.selected_source = Some(source);
        view.roster_epoch = epoch;
        view.characters = vec![CharacterRow {
            source_id: source,
            object_id: 3333,
            label: format!("source-{source}"),
        }];
        Published {
            view,
            source_epoch: epoch,
            navigation_epoch: 0,
        }
    }

    fn controlled_worker(published: Published) -> (Worker, mpsc::SyncSender<()>) {
        let (release, receiver) = mpsc::sync_channel(1);
        let (wake, _wake_receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            // A failed assertion never leaves a synthetic worker hanging Drop.
            let _ = receiver.recv_timeout(Duration::from_secs(2));
        });
        (
            Worker {
                pending_canonical: None,
                cancel: Cancellation::new(),
                inbox: Arc::new(Mutex::new(Inbox::default())),
                wake,
                published: Arc::new(Mutex::new(published)),
                handle,
                desired: None,
                pending_navigation: None,
                pending_equipment: None,
                pending_room: None,
                pending_matchmaking: None,
                pending_world: None,
                pending_combat: None,
                pending_npc: None,
                pending_spell: None,
                pending_record: None,
                pending_chat: None,
                pending_direct_chat: None,
                pending_shop: None,
                pending_item: None,
                pending_ammo: None,
            },
            release,
        )
    }

    fn controller_with(view: LoginSnapshot) -> (SessionController, mpsc::SyncSender<()>) {
        let (worker, release) = controlled_worker(Published {
            view: view.clone(),
            source_epoch: 0,
            navigation_epoch: 0,
        });
        let mut controller = SessionController::new(profile());
        controller.next_selection = view
            .roster_epoch
            .max(
                view.navigation
                    .as_ref()
                    .map(|reply| reply.context.epoch)
                    .unwrap_or(0),
            )
            .saturating_add(1);
        controller.view = view;
        controller.active = Some(worker);
        (controller, release)
    }

    fn context(epoch: u64) -> SelectionContext {
        SelectionContext {
            key: CharacterKey {
                source_id: 7,
                object_id: 3333,
            },
            epoch,
        }
    }

    fn principal() -> Principal {
        Principal {
            account: "synthetic-owner".into(),
            sources: vec![Source {
                id: 7,
                name: "太陽神".into(),
            }],
        }
    }

    fn navigation_reply(
        context: SelectionContext,
        action: NavigationAction,
        phase: NavigationPhase,
    ) -> NavigationReply {
        NavigationReply {
            action,
            status: NavigationStatus::Ok,
            context,
            snapshot: NavigationSnapshot {
                phase,
                channel_id: if phase == NavigationPhase::Lobby {
                    Some(2)
                } else {
                    None
                },
                selected: if phase == NavigationPhase::Characters {
                    None
                } else {
                    Some(CharacterSummary {
                        key: context.key,
                        name: "勇者".into(),
                        class: 1,
                        sex: 0,
                        level: 50,
                    })
                },
                channels: if phase == NavigationPhase::Characters {
                    vec![]
                } else {
                    (0..4)
                        .map(|id| ChannelSummary {
                            id,
                            name: format!("頻道{id}"),
                            population: 1,
                            capacity: 20,
                            minimum_level: 1,
                            maximum_level: 100,
                        })
                        .collect()
                },
            },
        }
    }

    fn take_navigation(controller: &SessionController) -> NavigationRequest {
        controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .navigation
            .take()
            .unwrap()
    }

    fn acknowledge(
        controller: &mut SessionController,
        request: NavigationRequest,
        phase: NavigationPhase,
    ) {
        let reply = navigation_reply(request.context, request.action, phase);
        let view = navigation_view(&principal(), reply).unwrap();
        let worker = controller.active.as_ref().unwrap();
        publish_navigation(
            &worker.published,
            view,
            request.context.epoch,
            request.refresh_epoch,
        );
        controller.poll();
    }

    #[test]
    fn spell_command_retains_authority_until_exact_epoch_and_disconnect_cancels_owned_inbox() {
        let reply = spell::tests::initial();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world_state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut world_state, &reply).unwrap();
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: reply.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 5,
                support: spell::Support::Teleport,
                mp_cost: 10,
                reuse_millis: 10,
                name: "瞬間移動".into(),
            }],
        }));
        let (mut controller, release) = controller_with(view.clone());
        controller.command(LoginCommand::CastTeleport).unwrap();
        let (request, epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .take()
            .unwrap();
        let SpellRequest::Player(request) = request.unwrap() else {
            panic!("not a player spell")
        };
        assert_eq!(request.sequence, 1);
        assert_eq!(request.skill_id, 5);
        assert_eq!(controller.snapshot().combat, view.combat);
        assert!(controller.snapshot().spell_pending);
        assert!(controller.command(LoginCommand::ReadSkills).is_err());
        assert!(controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2
            })
            .is_err());
        assert!(controller
            .command(LoginCommand::World(world::Request::read(
                reply.echo.context,
                reply.echo.channel_id
            )))
            .is_err());
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().spell_pending);
        let cast = spell::tests::success(&reply);
        assert_eq!(cast.echo, request);
        receive_combat(&mut view, &mut world_state, &cast.combat_reply().unwrap()).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().spell_pending);
        assert_eq!(controller.snapshot().combat, view.combat);
        controller.command(LoginCommand::ReadSkills).unwrap();
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().spell.is_none());
        assert!(controller.snapshot().combat.is_none());
        assert_eq!(
            controller.snapshot().skills,
            spell::SkillsState::NotRequested
        );
        release.send(()).unwrap();
    }
    #[test]
    fn targeted_spell_keeps_confirmed_state_and_cannot_overtake_other_owned_commands() {
        let reply = spell::tests::initial_damage();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut world, &reply).unwrap();
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: reply.echo.context,
            status: spell::Status::Ok,
            skills: vec![
                spell::Skill {
                    skill_id: 4,
                    support: spell::Support::LightArrow,
                    mp_cost: 3,
                    reuse_millis: 10,
                    name: "光箭".into(),
                },
                spell::Skill {
                    skill_id: 5,
                    support: spell::Support::Teleport,
                    mp_cost: 10,
                    reuse_millis: 10,
                    name: "瞬移".into(),
                },
            ],
        }));
        let (mut controller, release) = controller_with(view.clone());
        for target in [0, 1, 3, u64::MAX] {
            assert!(controller
                .command(LoginCommand::CastLightArrow {
                    target_entity_id: target
                })
                .is_err());
        }
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::CastLightArrow {
                target_entity_id: 2
            })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        controller
            .command(LoginCommand::CastLightArrow {
                target_entity_id: 2,
            })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().spell.take().unwrap();
        let SpellRequest::Player(request) = request.unwrap() else {
            panic!("not a player spell")
        };
        assert_eq!(
            (request.skill_id, request.target_entity_id, request.sequence),
            (4, 2, 1)
        );
        assert_eq!(controller.snapshot().combat, view.combat);
        for command in [
            LoginCommand::CastLightArrow {
                target_entity_id: 2,
            },
            LoginCommand::CastTeleport,
            LoginCommand::ReadSkills,
            LoginCommand::CombatAttack {
                target_entity_id: 2,
            },
            LoginCommand::ReadRecord,
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().spell_pending);
        let ack = spell::tests::damage_success(&reply);
        assert_eq!(ack.echo, request);
        receive_combat(&mut view, &mut world, &ack.combat_reply().unwrap()).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().spell_pending);
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().vitals[0].mp,
            97
        );
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().vitals[1].hp,
            40
        );
        controller.command(LoginCommand::CastTeleport).unwrap();
        assert!(controller
            .command(LoginCommand::CastLightArrow {
                target_entity_id: 2
            })
            .is_err());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().spell.is_none());
        assert!(controller.snapshot().combat.is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn fire_storm_is_learned_self_area_singleflight_and_only_ack_changes_authority() {
        let reply = crate::modern_lito::fire_storm_protocol::tests::initial();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut state, &reply).unwrap();
        let (mut unlearned, release) = controller_with(view.clone());
        assert!(unlearned.command(LoginCommand::CastFireStorm).is_err());
        assert!(!unlearned.snapshot().spell_pending);
        release.send(()).unwrap();
        drop(unlearned);
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: reply.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 70,
                support: spell::Support::FireStorm,
                mp_cost: 48,
                reuse_millis: 2000,
                name: "火風暴".into(),
            }],
        }));
        let mut disabled = view.clone();
        let mut frame = (**disabled.combat.as_ref().unwrap()).clone();
        frame.fires.clear();
        disabled.combat = Some(Arc::new(frame));
        let (mut off, release) = controller_with(disabled);
        assert!(off.command(LoginCommand::CastFireStorm).is_err());
        release.send(()).unwrap();
        drop(off);
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller.command(LoginCommand::CastFireStorm).is_err());
        inbox.lock().unwrap().auto_world = false;
        let before = controller.snapshot().combat.clone();
        controller.command(LoginCommand::CastFireStorm).unwrap();
        let (request, epoch) = inbox.lock().unwrap().spell.take().unwrap();
        let SpellRequest::Player(request) = request.unwrap() else {
            panic!("not a player spell")
        };
        assert_eq!(
            (request.skill_id, request.target_entity_id, request.sequence),
            (70, 0, 1)
        );
        assert_eq!(controller.snapshot().combat, before);
        for command in [
            LoginCommand::CastFireStorm,
            LoginCommand::CastTeleport,
            LoginCommand::ReadSkills,
            LoginCommand::UseItem { item_id: 40010 },
            LoginCommand::CombatAttack {
                target_entity_id: 2,
            },
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().spell_pending);
        let ack = crate::modern_lito::fire_storm_protocol::tests::cast(&reply, 1, &[]);
        assert_eq!(ack.echo, request);
        receive_combat(&mut view, &mut state, &ack.combat_reply().unwrap()).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().spell_pending);
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().fires[0].count,
            1
        );
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().vitals[0].mp,
            252
        );
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(controller.snapshot().combat.is_none());
        assert!(inbox.lock().unwrap().spell.is_none());
        release.send(()).unwrap();
    }
    #[test]
    fn basic_heal_requires_a_learned_skill_and_own_objective_and_keeps_singleflight_authority() {
        let reply = crate::modern_lito::honor_protocol::tests::initial();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut state, &reply).unwrap();
        let (mut unlearned, unlearned_release) = controller_with(view.clone());
        assert!(unlearned
            .command(LoginCommand::CastBasicHeal {
                target_entity_id: 3
            })
            .is_err());
        unlearned_release.send(()).unwrap();
        drop(unlearned);
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: reply.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 1,
                support: spell::Support::BasicHeal,
                mp_cost: 4,
                reuse_millis: 10,
                name: "初級治癒術".into(),
            }],
        }));
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        let before = controller.snapshot().combat.clone();
        for target in [0, 1, 2, 4, u64::MAX] {
            assert!(controller
                .command(LoginCommand::CastBasicHeal {
                    target_entity_id: target
                })
                .is_err());
            assert!(inbox.lock().unwrap().spell.is_none());
            assert!(!controller.snapshot().spell_pending);
            assert_eq!(controller.snapshot().combat, before);
        }
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::CastBasicHeal {
                target_entity_id: 3
            })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        controller
            .command(LoginCommand::CastBasicHeal {
                target_entity_id: 3,
            })
            .unwrap();
        let (request, epoch) = inbox.lock().unwrap().spell.take().unwrap();
        let SpellRequest::Player(request) = request.unwrap() else {
            panic!("not a player spell")
        };
        assert_eq!(
            (request.skill_id, request.target_entity_id, request.sequence),
            (1, 3, 1)
        );
        assert_eq!(controller.snapshot().combat, before);
        for command in [
            LoginCommand::CastBasicHeal {
                target_entity_id: 3,
            },
            LoginCommand::UseItem { item_id: 40010 },
            LoginCommand::CombatAttack {
                target_entity_id: 4,
            },
            LoginCommand::ReadSkills,
        ] {
            assert!(controller.command(command).is_err());
        }
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().spell_pending);
        let ack = crate::modern_lito::honor_protocol::tests::heal(&reply, 0);
        assert_eq!(ack.echo, request);
        receive_combat(&mut view, &mut state, &ack.combat_reply().unwrap()).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().spell_pending);
        assert_eq!(
            controller
                .snapshot()
                .combat
                .as_ref()
                .unwrap()
                .honor
                .as_ref()
                .unwrap()
                .heals[0]
                .count,
            1
        );
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().spell.is_none());
        assert!(controller.snapshot().combat.is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn spell_request_never_overtakes_auto_prepare_or_inflight_move() {
        let reply = spell::tests::initial();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut state, &reply).unwrap();
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: reply.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 5,
                support: spell::Support::Teleport,
                mp_cost: 10,
                reuse_millis: 10,
                name: "T".into(),
            }],
        }));
        let (mut controller, release) = controller_with(view.clone());
        controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .auto_world = true;
        assert!(controller.command(LoginCommand::CastTeleport).is_err());
        assert!(!controller.snapshot().spell_pending);
        controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .auto_world = false;
        // Enter through the actual command admission path: a view flag alone
        // is not an owned operation and command() correctly refreshes it from
        // the worker's last confirmed publication.
        let before = reply.frame.as_ref().unwrap();
        let actor = &before.world.actors[0];
        let movement = world::Request {
            action: world::Action::Move,
            heading: 4,
            match_id: before.world.match_id,
            generation: before.world.generation,
            sequence: actor.sequence + 1,
            from_x: actor.position.x,
            from_y: actor.position.y,
            ..world::Request::read(reply.echo.context, reply.echo.channel_id)
        };
        controller.command(LoginCommand::World(movement)).unwrap();
        assert!(controller.snapshot().world_pending);
        assert!(controller.command(LoginCommand::CastTeleport).is_err());
        let active = controller.active.as_ref().unwrap();
        let (queued, epoch) = active.inbox.lock().unwrap().world.take().unwrap();
        assert_eq!(queued, movement);
        assert_eq!(active.pending_world, Some((movement, epoch)));
        // Taking the queued command models the real TLS worker beginning its
        // exchange. It is still owned/pending until the exact operation ACK.
        assert!(controller.command(LoginCommand::CastTeleport).is_err());
        assert!(controller.command(LoginCommand::ReadSkills).is_err());
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().world_pending);
        assert!(controller.command(LoginCommand::CastTeleport).is_err());
        assert!(controller.command(LoginCommand::CastFireStorm).is_err());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .is_none());

        // A correct MOVE ACK may publish position without combat vitals. That
        // interim state is not permission to CAST against stale MP or position.
        let mut moved = before.world.clone();
        moved.revision += 1;
        moved.actors[0].position.y += 1;
        moved.actors[0].heading = movement.heading;
        moved.actors[0].sequence = movement.sequence;
        let movement_ack = world::Reply {
            echo: movement,
            status: world::Status::Ok,
            reason: world::Reason::None,
            frame: Some(moved.clone()),
        };
        movement_ack.validate().unwrap();
        view.world = world::State::Ready(Arc::new(movement_ack));
        view.world_pending = false;
        view.combat = None;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().world_pending);
        assert!(controller.active.as_ref().unwrap().pending_world.is_none());
        assert!(controller.command(LoginCommand::CastTeleport).is_err());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .is_none());

        let mut current = reply.clone();
        current.frame.as_mut().unwrap().world = moved;
        let mut world_state = view.world.clone();
        receive_combat(&mut view, &mut world_state, &current).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            reply.echo.context.epoch,
            epoch,
        );
        controller.command(LoginCommand::CastTeleport).unwrap();
        assert!(controller.snapshot().spell_pending);
        let (spell, spell_epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .spell
            .take()
            .unwrap();
        let SpellRequest::Player(spell) = spell.unwrap() else {
            panic!("not a player spell")
        };
        assert_eq!(spell.context, reply.echo.context);
        assert_eq!(spell.sequence, 1);
        assert!(spell_epoch > epoch);
        release.send(()).unwrap();
    }

    #[test]
    fn combat_command_waits_for_its_epoch_and_never_predicts_health_or_events() {
        let reply = combat::tests::active_reply();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut world_state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut world_state, &reply).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2,
            })
            .unwrap();
        let (request, epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .combat
            .take()
            .unwrap();
        let CombatCommand::Player(request) = request else {
            panic!("ordinary player command expected");
        };
        assert_eq!(request.sequence, 1);
        assert_eq!(request.target_entity_id, 2);
        assert!(controller.snapshot().combat_pending);
        assert_eq!(controller.snapshot().combat, view.combat);
        assert!(controller
            .command(LoginCommand::World(world::Request::read(
                reply.echo.context,
                0
            )))
            .is_err());
        assert!(controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2
            })
            .is_err());
        let worker = controller.active.as_ref().unwrap();
        publish_navigation(
            &worker.published,
            view.clone(),
            reply.echo.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().combat_pending);
        let mut hit = reply.clone();
        hit.echo = request;
        let frame = hit.frame.as_mut().unwrap();
        frame.world.revision += 1;
        frame.vitals[0].attack_sequence = 1;
        frame.vitals[0].attack_revision = frame.world.revision;
        frame.vitals[0].last_target_entity_id = 2;
        frame.vitals[1].hp = 40;
        frame.vitals[1].hurt_revision = frame.world.revision;
        receive_combat(&mut view, &mut world_state, &hit).unwrap();
        view.combat_pending = false;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            reply.echo.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().combat_pending);
        assert_eq!(
            controller.snapshot().combat.as_ref().unwrap().vitals[1].hp,
            40
        );
        assert_eq!(
            controller.snapshot().world.frame(),
            Some(&controller.snapshot().combat.as_ref().unwrap().world)
        );
        release.send(()).unwrap();
    }
    #[test]
    fn combat_reads_present_authoritative_countdown_and_timeout_without_guessing_clock() {
        for (phase, reason, notice) in [
            (
                world::Phase::Loading,
                world::AbortReason::None,
                "等待全員載入",
            ),
            (
                world::Phase::Countdown,
                world::AbortReason::None,
                "伺服器倒數中",
            ),
            (
                world::Phase::Active,
                world::AbortReason::None,
                "普通攻擊與即時素質",
            ),
            (
                world::Phase::Aborted,
                world::AbortReason::ActiveTimeout,
                "安全時間上限",
            ),
            (
                world::Phase::Aborted,
                world::AbortReason::LoadTimeout,
                "載入逾時",
            ),
        ] {
            let mut reply = combat::tests::active_reply();
            let frame = &mut reply.frame.as_mut().unwrap().world;
            frame.phase = phase;
            frame.abort_reason = reason;
            let mut policy = super::super::world_runtime::tests::match_descriptor(true);
            policy.countdown_nanos = 30_000_000_000;
            policy.active_timeout_millis = 50;
            frame.descriptor = Some(policy);
            if phase == world::Phase::Loading {
                for actor in &mut frame.actors {
                    actor.loaded = false;
                }
            }
            reply.validate().unwrap();
            let mut nav = navigation_reply(
                reply.echo.context,
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            );
            nav.snapshot.channel_id = Some(reply.echo.channel_id);
            let mut view = navigation_view(&principal(), nav).unwrap();
            let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
            receive_combat(&mut view, &mut state, &reply).unwrap();
            assert!(view.status.contains(notice), "{phase:?}: {}", view.status);
            assert_eq!(view.world.frame().unwrap().phase, phase);
            assert_eq!(view.world.frame().unwrap().descriptor, Some(policy));
            if phase == world::Phase::Aborted {
                assert!(view.status.contains("返回房間") && view.status.contains("不計算勝敗"));
                assert_eq!(view.status, world_notice(&reply.world_reply().unwrap()));
            }
            assert!(!view.status.contains("剩餘"));
        }
    }

    #[test]
    fn countdown_and_active_timeout_block_attack_but_keep_same_scope_leave_available() {
        for phase in [world::Phase::Countdown, world::Phase::Aborted] {
            let mut reply = combat::tests::active_reply();
            let frame = &mut reply.frame.as_mut().unwrap().world;
            frame.phase = phase;
            if phase == world::Phase::Aborted {
                frame.abort_reason = world::AbortReason::ActiveTimeout;
            }
            frame.descriptor = Some(super::super::world_runtime::tests::match_descriptor(true));
            reply.validate().unwrap();
            let mut nav = navigation_reply(
                reply.echo.context,
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            );
            nav.snapshot.channel_id = Some(reply.echo.channel_id);
            let mut view = navigation_view(&principal(), nav).unwrap();
            let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
            receive_combat(&mut view, &mut state, &reply).unwrap();
            let (mut controller, release) = controller_with(view);
            let before = controller.snapshot();
            assert!(controller
                .command(LoginCommand::CombatAttack {
                    target_entity_id: 2
                })
                .is_err());
            assert_eq!(controller.snapshot(), before);
            assert!(controller
                .active
                .as_ref()
                .unwrap()
                .inbox
                .lock()
                .unwrap()
                .combat
                .is_none());
            let mut leave = world::Request::read(reply.echo.context, reply.echo.channel_id);
            leave.action = world::Action::Leave;
            leave.match_id = reply.echo.match_id;
            leave.generation = reply.echo.generation;
            controller.command(LoginCommand::World(leave)).unwrap();
            assert!(controller.snapshot().world_pending);
            assert_eq!(controller.snapshot().world.frame().unwrap().phase, phase);
            assert_eq!(
                controller
                    .active
                    .as_ref()
                    .unwrap()
                    .inbox
                    .lock()
                    .unwrap()
                    .world
                    .as_ref()
                    .unwrap()
                    .0,
                leave
            );
            release.send(()).unwrap();
        }
    }

    #[test]
    fn combat_polling_uses_this_match_descriptor_not_a_previous_combat_or_global_flag() {
        let mut frame = super::super::world_runtime::tests::combat_frame().world;
        assert!(should_poll_combat(Some(&frame), true));
        assert!(!should_poll_combat(Some(&frame), false));
        assert!(!should_poll_combat(None, true));
        frame.descriptor = Some(super::super::world_runtime::tests::match_descriptor(false));
        assert!(!should_poll_combat(Some(&frame), true));
        frame.descriptor = Some(super::super::world_runtime::tests::match_descriptor(true));
        assert!(should_poll_combat(Some(&frame), true));
        assert!(!should_poll_combat(Some(&frame), false));
        frame.phase = world::Phase::Preparing;
        for actor in &mut frame.actors {
            actor.loaded = false;
        }
        frame.validate(frame.actors[0].key).unwrap();
        assert!(!should_poll_combat(Some(&frame), true));
    }

    #[test]
    fn race_phase_notice_never_grants_combat_or_calls_pending_a_saved_win() {
        use super::super::race_protocol::{tests, Settlement};
        let running = tests::running();
        assert!(!should_poll_combat(Some(&running), true));
        assert!(confirmed_world_notice(Some(&running)).contains("不開放攻擊"));
        let mut complete = tests::complete();
        assert!(confirmed_world_notice(Some(&complete)).contains("等待"));
        complete.race.as_mut().unwrap().settlement = Settlement::Saved;
        assert!(confirmed_world_notice(Some(&complete)).contains("不列入兩隊戰鬥勝敗"));
        complete.race.as_mut().unwrap().settlement = Settlement::Failed;
        assert!(confirmed_world_notice(Some(&complete)).contains("不可假定未入帳"));
        assert!(!should_poll_combat(Some(&complete), true));
    }

    #[test]
    fn scene_descriptor_rejects_combat_reply_without_mutating_controller_state() {
        let reply = combat::tests::active_reply();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(reply.echo.channel_id);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut scene = reply.world_reply().unwrap();
        scene.frame.as_mut().unwrap().descriptor =
            Some(super::super::world_runtime::tests::match_descriptor(false));
        scene.validate().unwrap();
        let mut state = world::State::Ready(Arc::new(scene));
        view.world = state.clone();
        view.combat_supported = true;
        let old = view.clone();
        let old_state = state.clone();
        assert!(receive_combat(&mut view, &mut state, &reply).is_err());
        assert_eq!(view, old);
        assert_eq!(state, old_state);
    }

    #[test]
    fn unsupported_or_stale_atomic_combat_never_queues_attack() {
        let reply = combat::tests::active_reply();
        for variant in 0..3 {
            let mut nav = navigation_reply(
                reply.echo.context,
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            );
            nav.snapshot.channel_id = Some(0);
            let mut view = navigation_view(&principal(), nav).unwrap();
            let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
            receive_combat(&mut view, &mut state, &reply).unwrap();
            match variant {
                0 => view.combat_supported = false,
                1 => view.combat = None,
                2 => {
                    let world::State::Ready(r) = &view.world else {
                        unreachable!()
                    };
                    let mut r = r.as_ref().clone();
                    r.frame.as_mut().unwrap().revision += 1;
                    view.world = world::State::Ready(Arc::new(r));
                }
                _ => unreachable!(),
            }
            let (mut controller, release) = controller_with(view);
            assert!(controller
                .command(LoginCommand::CombatAttack {
                    target_entity_id: 2
                })
                .is_err());
            assert!(controller
                .active
                .as_ref()
                .unwrap()
                .inbox
                .lock()
                .unwrap()
                .combat
                .is_none());
            release.send(()).unwrap();
        }
    }
    #[test]
    fn combat_source_epoch_and_atomic_world_mismatch_cannot_attach_vitals() {
        let reply = combat::tests::active_reply();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(0);
        let view = navigation_view(&principal(), nav).unwrap();
        for change in 0..3 {
            let mut view = view.clone();
            let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
            let mut wrong = reply.clone();
            match change {
                0 => wrong.echo.context.key.source_id = 42,
                1 => wrong.echo.context.epoch += 1,
                2 => {
                    wrong.echo.generation += 1;
                    let f = wrong.frame.as_mut().unwrap();
                    f.world.generation += 1;
                    f.world.room.summary.revision += 1;
                }
                _ => unreachable!(),
            }
            assert!(receive_combat(&mut view, &mut state, &wrong).is_err());
            assert!(view.combat.is_none());
        }
    }
    #[test]
    fn cancelled_combat_command_clears_mailbox_vitals_and_retains_owned_worker() {
        let reply = combat::tests::active_reply();
        let mut nav = navigation_reply(
            reply.echo.context,
            NavigationAction::EnterChannel,
            NavigationPhase::Lobby,
        );
        nav.snapshot.channel_id = Some(0);
        let mut view = navigation_view(&principal(), nav).unwrap();
        let mut state = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        receive_combat(&mut view, &mut state, &reply).unwrap();
        let (mut controller, release) = controller_with(view);
        controller
            .command(LoginCommand::CombatAttack {
                target_entity_id: 2,
            })
            .unwrap();
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        controller.command(LoginCommand::Shutdown).unwrap();
        assert!(inbox.lock().unwrap().combat.is_none());
        assert!(controller.snapshot().combat.is_none());
        assert!(!controller.snapshot().combat_pending);
        assert!(!controller.is_quiescent());
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !controller.is_quiescent() {
            controller.poll();
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
    }
    #[test]
    fn old_world_loaded_and_leave_are_rejected_before_inbox_without_rebinding_current_scope() {
        let context = context(11);
        for action in [world::Action::Loaded, world::Action::Leave] {
            let mut view = navigation_view(
                &principal(),
                navigation_reply(
                    context,
                    NavigationAction::EnterChannel,
                    NavigationPhase::Lobby,
                ),
            )
            .unwrap();
            let channel = view
                .navigation
                .as_ref()
                .unwrap()
                .snapshot
                .channel_id
                .unwrap();
            let frame = super::super::world_runtime::tests::frame(101);
            frame.validate(context.key).unwrap();
            let current = world::Request {
                action,
                match_id: frame.match_id,
                generation: frame.generation,
                manifest_digest: if action == world::Action::Loaded {
                    frame.manifest.digest
                } else {
                    [0; 32]
                },
                ..world::Request::read(context, channel)
            };
            current.validate().unwrap();
            view.world = world::State::Ready(Arc::new(world::Reply {
                echo: world::Request::read(context, channel),
                status: world::Status::Ok,
                reason: world::Reason::None,
                frame: Some(frame),
            }));
            let (mut controller, release) = controller_with(view.clone());
            let epoch = controller.next_selection;
            // Same selected source/character/channel, but an old match, an old
            // generation, or both. None may enter this owner's TLS mailbox.
            for (match_id, generation) in [
                (current.match_id - 1, current.generation),
                (current.match_id, current.generation - 1),
                (current.match_id - 1, current.generation - 1),
            ] {
                let stale = world::Request {
                    match_id,
                    generation,
                    ..current
                };
                stale.validate().unwrap();
                let error = controller.command(LoginCommand::World(stale)).unwrap_err();
                assert!(error.contains("世界世代已改變"), "{action:?}: {error}");
                assert_eq!(controller.snapshot(), view);
                assert_eq!(controller.next_selection, epoch);
                let worker = controller.active.as_ref().unwrap();
                assert_eq!(worker.pending_world, None);
                let inbox = worker.inbox.lock().unwrap();
                assert!(inbox.world.is_none());
                assert!(!inbox.auto_world && !inbox.shutdown);
            }
            // The guard must not reject the equivalent current-scope command
            // or invent its ACK: it becomes exactly one owned pending request.
            controller.command(LoginCommand::World(current)).unwrap();
            assert!(controller.snapshot().world_pending);
            assert_eq!(controller.snapshot().world, view.world);
            let worker = controller.active.as_ref().unwrap();
            assert_eq!(worker.pending_world, Some((current, epoch)));
            assert_eq!(worker.inbox.lock().unwrap().world, Some((current, epoch)));
            assert_eq!(controller.next_selection, epoch + 1);
            controller.command(LoginCommand::Shutdown).unwrap();
            release.send(()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(1);
            while !controller.is_quiescent() {
                controller.poll();
                assert!(
                    Instant::now() < deadline,
                    "controlled world worker must retire"
                );
                thread::yield_now();
            }
        }
    }

    #[test]
    fn world_pending_requires_its_command_epoch_and_auto_capture_reserves_against_ui_mutations() {
        let context = context(11);
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context,
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let channel = view
            .navigation
            .as_ref()
            .unwrap()
            .snapshot
            .channel_id
            .unwrap();
        let mut directory = rooms::tests::sample(context, true);
        directory.common.channel_id = channel;
        view.rooms = rooms::State::Ready(Arc::new(directory));
        let request = world::Request::read(context, channel);
        view.world = world::State::Ready(Arc::new(world::Reply {
            echo: request,
            status: world::Status::Ok,
            reason: world::Reason::None,
            frame: None,
        }));
        let (mut controller, release) = controller_with(view.clone());
        controller.command(LoginCommand::World(request)).unwrap();
        assert!(controller.snapshot().world_pending);
        assert!(controller
            .command(LoginCommand::Rooms(
                rooms::Request::read(context, channel).unwrap()
            ))
            .is_err());
        assert!(controller.command(LoginCommand::World(request)).is_err());
        let worker = controller.active.as_ref().unwrap();
        let (_, epoch) = worker.inbox.lock().unwrap().world.take().unwrap();
        publish_navigation(&worker.published, view.clone(), context.epoch, epoch - 1);
        controller.poll();
        assert!(
            controller.snapshot().world_pending,
            "periodic world reads cannot ACK a queued mutation"
        );
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().world_pending);
        controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .auto_world = true;
        assert!(controller
            .command(LoginCommand::Rooms(
                rooms::Request::read(context, channel).unwrap()
            ))
            .is_err());
        assert!(controller.command(LoginCommand::World(request)).is_err());
        controller.command(LoginCommand::Shutdown).unwrap();
        assert!(!controller.is_quiescent());
        release.send(()).unwrap();
        for _ in 0..100 {
            controller.poll();
            if controller.is_quiescent() {
                break;
            }
            thread::sleep(Duration::from_millis(1));
        }
        assert!(controller.is_quiescent());
    }

    #[test]
    fn equipment_waits_for_exact_ack_keeps_page_and_blocks_navigation_or_duplicate_operations() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let initial = equipment::tests::sample_snapshot(context(11));
        attach_equipment(&mut view, &equipment::State::Ready(initial.clone())).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let request = equipment::Request::set(&initial, 1, true);
        controller
            .command(LoginCommand::Equipment(request))
            .unwrap();
        assert!(controller.snapshot().equipment_pending);
        assert_eq!(controller.snapshot().phase, view.phase);
        assert_eq!(controller.snapshot().equipment, view.equipment);
        assert!(controller
            .command(LoginCommand::Equipment(request))
            .is_err());
        assert_eq!(
            controller
                .command(LoginCommand::ReturnCharacters(context(11)))
                .unwrap_err(),
            "裝備操作尚未完成，請稍候或中斷連線"
        );
        assert!(controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 1
            })
            .is_err());
        assert!(controller.command(LoginCommand::SelectSource(7)).is_err());
        controller.poll();
        assert!(
            controller.snapshot().equipment_pending,
            "old publication must not reenable controls"
        );
        let (queued, epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .equipment
            .take()
            .unwrap();
        assert_eq!(queued, request);
        let super::super::application_protocol::Response::Equipment(reply) =
            super::super::application_protocol::decode_response(&equipment::tests::fixture(
                2,
                request,
                equipment::Status::Ok,
                true,
            ))
            .unwrap()
            .response
        else {
            panic!()
        };
        let mut state = view.equipment.clone();
        received_equipment(&mut view, &mut state, reply).unwrap();
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view.clone(),
            request.context.epoch,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().equipment_pending);
        let equipment::State::Ready(confirmed) = controller.snapshot().equipment else {
            panic!()
        };
        assert_eq!(confirmed.revision, 1);
        assert_eq!(confirmed.equipped(1), Some(true));
        assert_eq!(initial.equipped(1), Some(false));
        assert_eq!(controller.snapshot().navigation, view.navigation);
        assert!(
            controller
                .command(LoginCommand::Equipment(request))
                .is_err(),
            "previous revision/token cannot be submitted again"
        );
        release.send(()).unwrap();
    }

    #[test]
    fn room_pending_retains_arc_and_background_read_cannot_ack_it_or_allow_navigation_and_gear() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let mut room = rooms::tests::sample(context(11), true);
        room.common.channel_id = 2;
        let original = Arc::new(room.clone());
        view.rooms = rooms::State::Ready(original.clone());
        let gear = equipment::tests::sample_snapshot(context(11));
        attach_equipment(&mut view, &equipment::State::Ready(gear.clone())).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        assert!(controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .unwrap_err()
            .contains("先離開戰鬥室"));
        let mut common = room.common;
        common.action = rooms::Action::Ready;
        common.room_id = room.rooms[0].room_id;
        common.expected_revision = 1;
        common.value = 1;
        controller
            .command(LoginCommand::Rooms(rooms::Request::simple(common).unwrap()))
            .unwrap();
        assert!(controller.snapshot().room_pending);
        assert!(controller
            .command(LoginCommand::Rooms(
                rooms::Request::read(context(11), 2).unwrap()
            ))
            .is_err());
        assert!(controller
            .command(LoginCommand::Equipment(equipment::Request::read(context(
                11
            ))))
            .is_err());
        assert!(controller
            .command(LoginCommand::ReturnCharacters(context(11)))
            .is_err());
        room.list_revision = 2;
        room.rooms[0].revision = 2;
        room.joined.as_mut().unwrap().summary = room.rooms[0].clone();
        let mut polled = view.clone();
        polled.rooms = rooms::State::Ready(Arc::new(room.clone()));
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            polled,
            11,
            0,
        );
        controller.poll();
        assert!(controller.snapshot().room_pending);
        let rooms::State::Ready(retained) = controller.snapshot().rooms else {
            panic!()
        };
        assert!(Arc::ptr_eq(&retained, &original));
        let (_, epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .room
            .take()
            .unwrap();
        room.common = common;
        room.rooms[0].ready = 1;
        room.joined.as_mut().unwrap().summary = room.rooms[0].clone();
        room.joined.as_mut().unwrap().members[0].ready = true;
        room.validate().unwrap();
        view.rooms = rooms::State::Ready(Arc::new(room));
        view.room_pending = false;
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            view,
            11,
            epoch,
        );
        controller.poll();
        assert!(!controller.snapshot().room_pending);
        assert!(controller.snapshot().rooms.joined().unwrap().members[0].ready);
        controller.command(LoginCommand::Shutdown).unwrap();
        assert!(matches!(
            controller.snapshot().rooms,
            rooms::State::NotRequested
        ));
        release.send(()).unwrap();
    }
    #[test]
    fn gm_mutation_uses_room_epoch_reservation_and_cancel_retires_queued_command() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let mut room = rooms::tests::sample(context(11), true);
        room.common.channel_id = 2;
        view.rooms = rooms::State::Ready(Arc::new(room));
        view.gm_supported = true;
        let gear = equipment::tests::sample_snapshot(context(11));
        attach_equipment(&mut view, &equipment::State::Ready(gear.clone())).unwrap();
        let (mut controller, release) = controller_with(view.clone());
        let inbox = controller.active.as_ref().unwrap().inbox.clone();
        inbox.lock().unwrap().auto_world = true;
        assert!(controller
            .command(LoginCommand::GmBots { count: 1 })
            .is_err());
        inbox.lock().unwrap().auto_world = false;
        assert!(controller
            .command(LoginCommand::GmBots { count: 101 })
            .is_err());
        controller
            .command(LoginCommand::GmBots { count: 0 })
            .unwrap();
        assert!(controller.snapshot().room_pending);
        assert_eq!(controller.snapshot().rooms, view.rooms);
        assert!(controller
            .command(LoginCommand::Rooms(
                rooms::Request::read(context(11), 2).unwrap()
            ))
            .is_err());
        assert!(controller
            .command(LoginCommand::Equipment(equipment::Request::set(
                &gear, 1, true
            )))
            .is_err());
        assert!(controller
            .command(LoginCommand::GmBots { count: 1 })
            .is_err());
        controller.poll();
        assert!(controller.snapshot().room_pending);
        let (request, epoch) = inbox.lock().unwrap().gm.take().unwrap();
        assert_eq!(request.context, context(11));
        assert_eq!(request.count, 0);
        let published = controller.active.as_ref().unwrap().published.clone();
        let mut rejected = view.clone();
        rejected.status = "GM rejected; confirmed room retained".into();
        publish_navigation(
            &published,
            rejected.clone(),
            request.context.epoch,
            epoch - 1,
        );
        controller.poll();
        assert!(controller.snapshot().room_pending);
        publish_navigation(&published, rejected, request.context.epoch, epoch);
        controller.poll();
        assert!(!controller.snapshot().room_pending);
        assert_eq!(controller.snapshot().rooms, view.rooms);
        controller
            .command(LoginCommand::GmBots { count: 1 })
            .unwrap();
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(inbox.lock().unwrap().gm.is_none());
        assert!(!controller.snapshot().gm_supported);
        release.send(()).unwrap();
    }
    #[test]
    fn equipment_busy_retains_arc_denied_conflict_refresh_and_shutdown_never_apply_late_commit() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let initial = equipment::tests::sample_snapshot(context(11));
        let mut state = equipment::State::Ready(initial.clone());
        attach_equipment(&mut view, &state).unwrap();
        let request = equipment::Request::set(&initial, 1, true);
        for status in [
            equipment::Status::Busy,
            equipment::Status::Denied,
            equipment::Status::Conflict,
        ] {
            let super::super::application_protocol::Response::Equipment(reply) =
                super::super::application_protocol::decode_response(&equipment::tests::fixture(
                    2, request, status, false,
                ))
                .unwrap()
                .response
            else {
                panic!()
            };
            received_equipment(&mut view, &mut state, reply).unwrap();
            let equipment::State::Ready(current) = &state else {
                panic!()
            };
            if status == equipment::Status::Busy {
                assert!(Arc::ptr_eq(current, &initial));
            }
            assert_eq!(view.phase, LoginPhase::InLobby);
            assert!(!view.equipment_pending);
            assert!(!view.status.contains("變更已由伺服器保存"));
        }
        let (mut controller, release) = controller_with(view.clone());
        let equipment::State::Ready(current) = &view.equipment else {
            panic!()
        };
        let request = equipment::Request::set(current, 2, true);
        controller
            .command(LoginCommand::Equipment(request))
            .unwrap();
        let (request, epoch) = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .equipment
            .take()
            .unwrap();
        let publication = controller.active.as_ref().unwrap().published.clone();
        controller.command(LoginCommand::Shutdown).unwrap();
        let super::super::application_protocol::Response::Equipment(reply) =
            super::super::application_protocol::decode_response(&equipment::tests::fixture(
                2,
                request,
                equipment::Status::Ok,
                true,
            ))
            .unwrap()
            .response
        else {
            panic!()
        };
        received_equipment(&mut view, &mut state, reply).unwrap();
        publish_navigation(&publication, view, request.context.epoch, epoch);
        controller.poll();
        assert_eq!(
            controller.snapshot().equipment,
            equipment::State::NotRequested
        );
        assert!(controller.command(LoginCommand::Connect).is_err());
        release.send(()).unwrap();
    }
    #[test]
    fn equipment_unavailable_allows_only_explicit_read_and_wrong_context_is_rejected() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        view.equipment = equipment::State::Unavailable(equipment::Status::BackendUnavailable);
        let (mut controller, release) = controller_with(view.clone());
        assert!(controller
            .command(LoginCommand::Equipment(equipment::Request::read(context(
                12
            ))))
            .is_err());
        let current = equipment::tests::sample_snapshot(context(11));
        assert!(controller
            .command(LoginCommand::Equipment(equipment::Request::set(
                &current, 1, true
            )))
            .is_err());
        controller
            .command(LoginCommand::Equipment(equipment::Request::read(context(
                11,
            ))))
            .unwrap();
        assert!(controller.snapshot().equipment_pending);
        assert_eq!(controller.snapshot().equipment, view.equipment);
        release.send(()).unwrap();
    }
    #[test]
    fn version_two_expired_denial_retains_raw_item_and_busy_keeps_exact_confirmed_lifetime_arc() {
        let mut view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let mut state = equipment::State::NotRequested;
        let read = equipment::Request::read(context(11));
        let lifetime_rows = [
            (1, equipment::LifetimeState::Expired, 123),
            (2, equipment::LifetimeState::Permanent, -1),
        ];
        let bytes = equipment::tests::with_lifetimes(
            equipment::tests::fixture(2, read, equipment::Status::Ok, false),
            &lifetime_rows,
        );
        let super::super::application_protocol::Response::Equipment(reply) =
            super::super::application_protocol::decode_response_version(&bytes, 2)
                .unwrap()
                .response
        else {
            unreachable!()
        };
        received_equipment(&mut view, &mut state, reply).unwrap();
        let equipment::State::Ready(confirmed) = &state else {
            unreachable!()
        };
        let confirmed = confirmed.clone();
        assert!(confirmed.source.items[0].equipped);
        assert_eq!(confirmed.equipped(1), Some(false));
        assert_eq!(
            confirmed.lifetime(1).unwrap().state,
            equipment::LifetimeState::Expired
        );
        let super::super::application_protocol::Response::Equipment(busy) =
            super::super::application_protocol::decode_response_version(
                &equipment::tests::fixture(0, read, equipment::Status::Busy, false),
                2,
            )
            .unwrap()
            .response
        else {
            unreachable!()
        };
        received_equipment(&mut view, &mut state, busy).unwrap();
        assert!(
            matches!(&state, equipment::State::Ready(current) if Arc::ptr_eq(current, &confirmed))
        );
        let request = equipment::Request::set(&confirmed, 1, true);
        let mut denied = equipment::tests::with_lifetimes(
            // A denied SET preserves confirmed flags. The generic SET fixture
            // uses its requested flag as an OK-shaped snapshot, which would
            // incorrectly equip this expired item even though status is Denied.
            equipment::tests::fixture(2, read, equipment::Status::Denied, false),
            &lifetime_rows,
        );
        denied[16..88].copy_from_slice(&equipment::encode_request(19, request).unwrap()[16..]);
        denied[90..92].copy_from_slice(&(equipment::Reason::UnqualifiedItem as u16).to_be_bytes());
        let super::super::application_protocol::Response::Equipment(reply) =
            super::super::application_protocol::decode_response_version(&denied, 2)
                .unwrap()
                .response
        else {
            unreachable!()
        };
        received_equipment(&mut view, &mut state, reply).unwrap();
        assert!(view.status.contains("已到期"));
        assert!(view.status.contains("未刪除 SQL"));
        assert_eq!(view.phase, LoginPhase::ChoosingChannel);
        assert!(!view.equipment_pending);
        assert!(
            matches!(&view.character_data, CharacterDataState::Ready(data) if data.items.len()==2 && data.items[0].equipped)
        );
    }
    #[test]
    fn return_equipment_read_failure_retires_old_token_and_explicit_read_recovers() {
        let read = equipment::Request::read(context(11));
        let decode = |status| {
            let super::super::application_protocol::Response::Equipment(reply) =
                super::super::application_protocol::decode_response(&equipment::tests::fixture(
                    2, read, status, false,
                ))
                .unwrap()
                .response
            else {
                panic!()
            };
            reply
        };
        for status in [
            equipment::Status::Busy,
            equipment::Status::BackendUnavailable,
            equipment::Status::BindingMismatch,
        ] {
            let mut view = navigation_view(
                &principal(),
                navigation_reply(
                    context(11),
                    NavigationAction::EnterChannel,
                    NavigationPhase::Lobby,
                ),
            )
            .unwrap();
            let mut state = equipment::State::NotRequested;
            received_equipment(&mut view, &mut state, decode(equipment::Status::Ok)).unwrap();
            assert!(matches!(state, equipment::State::Ready(_)));
            received_return_equipment(&mut view, &mut state, decode(status)).unwrap();
            assert_eq!(state, equipment::State::Unavailable(status));
            assert_eq!(view.equipment, state);
            assert!(!view.status.contains("已重新讀取"));
            // Failure is not an empty source inventory.
            assert!(
                matches!(&view.character_data, CharacterDataState::Ready(d) if d.items.len() == 2)
            );
            received_equipment(&mut view, &mut state, decode(equipment::Status::Ok)).unwrap();
            assert!(matches!(state, equipment::State::Ready(_)));
            assert!(view.status.contains("已重新讀取"));
        }
    }
    #[test]
    fn navigation_waits_for_server_ack_and_cannot_coalesce_or_reenable_from_old_roster() {
        let (mut controller, release) = controller_with(row(7, 11).view);
        controller
            .command(LoginCommand::SelectCharacter(context(11)))
            .unwrap();
        let request = take_navigation(&controller);
        assert_eq!(request.refresh_epoch, 12);
        assert_eq!(controller.snapshot().phase, LoginPhase::SelectingCharacter);
        assert!(
            controller.snapshot().navigation.is_none(),
            "no fabricated channel page before acknowledgment"
        );
        for _ in 0..4 {
            controller.poll();
        }
        assert_eq!(controller.snapshot().phase, LoginPhase::SelectingCharacter);
        assert!(controller
            .command(LoginCommand::SelectCharacter(context(11)))
            .is_err());
        assert!(controller.command(LoginCommand::SelectSource(42)).is_err());
        assert!(controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 2
            })
            .is_err());
        assert!(controller
            .command(LoginCommand::ReturnCharacters(context(11)))
            .is_err());
        assert_eq!(controller.next_selection, 13);
        acknowledge(&mut controller, request, NavigationPhase::Channels);
        assert_eq!(controller.snapshot().phase, LoginPhase::ChoosingChannel);
        assert!(controller.snapshot().characters.is_empty());
        assert_eq!(controller.snapshot().roster_epoch, 0);
        assert_eq!(
            controller
                .snapshot()
                .navigation
                .unwrap()
                .snapshot
                .channels
                .len(),
            4
        );
        release.send(()).unwrap();
    }

    #[test]
    fn character_source_epoch_and_object_must_match_the_displayed_roster() {
        let (mut controller, release) = controller_with(row(7, 11).view);
        for invalid in [
            context(10),
            SelectionContext {
                key: CharacterKey {
                    source_id: 42,
                    object_id: 3333,
                },
                epoch: 11,
            },
            SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3334,
                },
                epoch: 11,
            },
        ] {
            assert!(controller
                .command(LoginCommand::SelectCharacter(invalid))
                .is_err());
            assert_eq!(controller.snapshot().phase, LoginPhase::CharactersReady);
            assert_eq!(controller.next_selection, 12);
            assert!(controller
                .active
                .as_ref()
                .unwrap()
                .inbox
                .lock()
                .unwrap()
                .navigation
                .is_none());
        }
        controller.next_selection = u64::MAX;
        assert!(controller
            .command(LoginCommand::SelectCharacter(context(11)))
            .is_err());
        assert_eq!(controller.snapshot().phase, LoginPhase::CharactersReady);
        release.send(()).unwrap();
    }

    #[test]
    fn lobby_and_return_keep_authoritative_context_and_reserve_a_fresh_same_source_roster() {
        let (mut controller, release) = controller_with(row(7, 11).view);
        controller
            .command(LoginCommand::SelectCharacter(context(11)))
            .unwrap();
        let select = take_navigation(&controller);
        acknowledge(&mut controller, select, NavigationPhase::Channels);
        controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 2,
            })
            .unwrap();
        let enter = take_navigation(&controller);
        assert_eq!(enter.channel_id, Some(2));
        for _ in 0..3 {
            controller.poll();
        }
        assert_eq!(
            controller.snapshot().phase,
            LoginPhase::JoiningLobby,
            "old channel page cannot reset pending join"
        );
        acknowledge(&mut controller, enter, NavigationPhase::Lobby);
        assert_eq!(controller.snapshot().phase, LoginPhase::InLobby);
        assert_eq!(
            controller
                .snapshot()
                .navigation
                .unwrap()
                .snapshot
                .channel_id,
            Some(2)
        );
        assert!(controller.command(LoginCommand::SelectSource(42)).is_err());
        controller
            .command(LoginCommand::ReturnCharacters(context(11)))
            .unwrap();
        let returning = take_navigation(&controller);
        assert!(returning.refresh_epoch > context(11).epoch);
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::ReturningCharacters);
        let mut loading = sources(LoginPhase::LoadingCharacters);
        loading.selected_source = Some(7);
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            loading,
            returning.refresh_epoch,
            returning.refresh_epoch,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::LoadingCharacters);
        assert!(controller.snapshot().navigation.is_none());
        assert!(controller.snapshot().characters.is_empty());
        let refreshed = row(7, returning.refresh_epoch);
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            refreshed.view,
            returning.refresh_epoch,
            returning.refresh_epoch,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::CharactersReady);
        assert_eq!(controller.snapshot().roster_epoch, returning.refresh_epoch);
        assert!(
            controller
                .command(LoginCommand::SelectCharacter(context(11)))
                .is_err(),
            "old selection capability cannot be reused"
        );
        controller
            .command(LoginCommand::SelectCharacter(context(
                returning.refresh_epoch,
            )))
            .unwrap();
        release.send(()).unwrap();
    }

    #[test]
    fn closed_or_failed_mutation_never_applies_late_lobby_success() {
        let reply = navigation_reply(
            context(11),
            NavigationAction::SelectCharacter,
            NavigationPhase::Channels,
        );
        let view = navigation_view(&principal(), reply).unwrap();
        let (mut controller, release) = controller_with(view);
        controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 2,
            })
            .unwrap();
        let pending = take_navigation(&controller);
        publish(
            &controller.active.as_ref().unwrap().published,
            empty(LoginPhase::Failed, "synthetic failed navigation"),
            0,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::Failed);
        assert!(controller.snapshot().navigation.is_none());
        controller.command(LoginCommand::Shutdown).unwrap();
        let late = navigation_view(
            &principal(),
            navigation_reply(pending.context, pending.action, NavigationPhase::Lobby),
        )
        .unwrap();
        publish_navigation(
            &controller.retiring[0].published,
            late,
            11,
            pending.refresh_epoch,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnecting);
        assert!(controller.snapshot().navigation.is_none());
        assert!(controller.retiring[0].cancel.is_cancelled());
        assert!(controller.retiring[0]
            .inbox
            .lock()
            .unwrap()
            .navigation
            .is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn leaving_lobby_waits_for_its_own_ack_keeps_selection_and_can_rejoin_without_roster_reload() {
        let view = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::EnterChannel,
                NavigationPhase::Lobby,
            ),
        )
        .unwrap();
        let (mut controller, release) = controller_with(view);
        let original = controller.snapshot().navigation.unwrap();
        assert!(controller
            .command(LoginCommand::LeaveLobby(context(10)))
            .is_err());
        controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .unwrap();
        let leaving = take_navigation(&controller);
        assert_eq!(leaving.action, NavigationAction::LeaveLobby);
        assert_eq!(leaving.channel_id, None);
        for _ in 0..4 {
            controller.poll();
        }
        assert_eq!(controller.snapshot().phase, LoginPhase::LeavingLobby);
        assert_eq!(
            controller.snapshot().navigation.as_ref(),
            Some(&original),
            "old server lobby stays visible until leave ACK"
        );
        assert!(controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .is_err());
        assert!(controller
            .command(LoginCommand::ReturnCharacters(context(11)))
            .is_err());
        assert!(controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 2
            })
            .is_err());
        assert!(controller.command(LoginCommand::SelectSource(42)).is_err());
        acknowledge(&mut controller, leaving, NavigationPhase::Channels);
        let channels = controller.snapshot();
        assert_eq!(channels.phase, LoginPhase::ChoosingChannel);
        assert_eq!(
            channels.navigation.as_ref().unwrap().context,
            original.context
        );
        assert_eq!(
            channels.navigation.as_ref().unwrap().snapshot.selected,
            original.snapshot.selected
        );
        assert_eq!(channels.roster_epoch, 0);
        assert!(channels.characters.is_empty());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .selection
            .is_none());
        assert!(controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .is_err());
        controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 2,
            })
            .unwrap();
        let entering = take_navigation(&controller);
        acknowledge(&mut controller, entering, NavigationPhase::Lobby);
        assert_eq!(controller.snapshot().phase, LoginPhase::InLobby);
        controller
            .command(LoginCommand::LeaveLobby(context(11)))
            .unwrap();
        let leaving = take_navigation(&controller);
        controller.command(LoginCommand::Shutdown).unwrap();
        let late = navigation_view(
            &principal(),
            navigation_reply(leaving.context, leaving.action, NavigationPhase::Channels),
        )
        .unwrap();
        publish_navigation(
            &controller.retiring[0].published,
            late,
            11,
            leaving.refresh_epoch,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnecting);
        assert!(controller.snapshot().navigation.is_none());
        assert!(controller.retiring[0].cancel.is_cancelled());
        release.send(()).unwrap();
    }

    #[test]
    fn source_data_retains_fresh_values_for_one_selection_and_unavailable_is_not_an_empty_inventory(
    ) {
        use super::super::character_data::{self, CharacterDataFailure};
        let mut data = character_data::tests::sample_data();
        data.context = context(11);
        data.character.key = data.context.key;
        let state = received_character_data(
            CharacterDataReply {
                context: context(11),
                result: Ok(data.clone()),
            },
            context(11),
        )
        .unwrap();
        for phase in [NavigationPhase::Channels, NavigationPhase::Lobby] {
            let mut view = navigation_view(
                &principal(),
                navigation_reply(context(11), NavigationAction::EnterChannel, phase),
            )
            .unwrap();
            assert_ne!(
                view.navigation
                    .as_ref()
                    .unwrap()
                    .snapshot
                    .selected
                    .as_ref()
                    .unwrap()
                    .level,
                data.character.level
            );
            attach_character_data(&mut view, &state).unwrap();
            assert_eq!(
                view.character_data, state,
                "fresh data level need not equal the older navigation row"
            );
            let CharacterDataState::Ready(shared) = &view.character_data else {
                panic!("ready");
            };
            let CharacterDataState::Ready(original) = &state else {
                panic!("ready");
            };
            assert!(
                Arc::ptr_eq(shared, original),
                "poll snapshots never clone the inventory rows"
            );
            view.navigation.as_mut().unwrap().context.epoch += 1;
            assert!(attach_character_data(&mut view, &state).is_err());
        }
        for failure in [
            CharacterDataFailure::Denied,
            CharacterDataFailure::BackendUnavailable,
            CharacterDataFailure::Busy,
        ] {
            let unavailable = received_character_data(
                CharacterDataReply {
                    context: context(11),
                    result: Err(failure),
                },
                context(11),
            )
            .unwrap();
            assert_eq!(unavailable, CharacterDataState::Unavailable(failure));
        }
        assert!(received_character_data(
            CharacterDataReply {
                context: context(12),
                result: Ok(data)
            },
            context(11)
        )
        .is_err());
        assert_eq!(
            empty(LoginPhase::Disconnected, "closed").character_data,
            CharacterDataState::NotRequested
        );
        assert_eq!(
            authenticated_view(&principal(), LoginPhase::LoadingCharacters, "reloaded")
                .character_data,
            CharacterDataState::NotRequested
        );
    }

    #[test]
    fn busy_return_retains_both_pages_roster_scope_and_exact_data_arc_until_an_actual_retry() {
        use super::super::character_data;
        for phase in [NavigationPhase::Channels, NavigationPhase::Lobby] {
            let mut original = navigation_view(
                &principal(),
                navigation_reply(context(11), NavigationAction::SelectCharacter, phase),
            )
            .unwrap();
            // Preserve even a retained roster; a refusal must not clear or
            // republish it as a freshly queried character selection page.
            original.characters = row(7, 11).view.characters;
            original.roster_epoch = 11;
            let mut data = character_data::tests::sample_data();
            data.context = context(11);
            data.character.key = context(11).key;
            original.character_data = CharacterDataState::Ready(Arc::new(data));
            let (mut controller, release) = controller_with(original.clone());
            controller.active.as_mut().unwrap().desired = Some(Selection {
                source: 7,
                epoch: 11,
            });
            controller
                .command(LoginCommand::ReturnCharacters(context(11)))
                .unwrap();
            let request = take_navigation(&controller);
            assert_eq!(controller.snapshot().phase, LoginPhase::ReturningCharacters);
            assert_eq!(controller.snapshot().navigation, original.navigation);
            let mut reply = original.navigation.clone().unwrap();
            reply.action = NavigationAction::ReturnCharacters;
            reply.status = NavigationStatus::Busy;
            reply.snapshot.channels[0].population += 1;
            let view = busy_return_view(&principal(), &original, reply.clone()).unwrap();
            publish_navigation(
                &controller.active.as_ref().unwrap().published,
                view,
                11,
                request.refresh_epoch,
            );
            controller.poll();
            let busy = controller.snapshot();
            assert_eq!(busy.phase, original.phase);
            assert_eq!(busy.sources, original.sources);
            assert_eq!(busy.selected_source, original.selected_source);
            assert_eq!(busy.characters, original.characters);
            assert_eq!(busy.roster_epoch, 11);
            assert_eq!(busy.navigation, Some(reply));
            assert!(busy.status.contains("裝備保存中"));
            assert!(busy.status.contains("稍後再返回"));
            assert!(!busy.status.contains("重新選擇"));
            let CharacterDataState::Ready(before) = &original.character_data else {
                panic!("data");
            };
            let CharacterDataState::Ready(after) = &busy.character_data else {
                panic!("data");
            };
            assert!(Arc::ptr_eq(before, after));
            let worker = controller.active.as_ref().unwrap();
            assert!(worker.pending_navigation.is_none());
            assert_eq!(worker.desired.unwrap().source, 7);
            assert_eq!(worker.desired.unwrap().epoch, 11);
            assert_eq!(worker.published.lock().unwrap().source_epoch, 11);
            assert!(
                worker.inbox.lock().unwrap().selection.is_none(),
                "Busy never queues CHAR"
            );
            assert!(controller.command(LoginCommand::SelectSource(42)).is_err());
            controller
                .command(LoginCommand::ReturnCharacters(context(11)))
                .unwrap();
            let retry = take_navigation(&controller);
            assert!(retry.refresh_epoch > request.refresh_epoch);
            assert_eq!(retry.context, context(11));
            assert_eq!(controller.snapshot().roster_epoch, 11);
            release.send(()).unwrap();
        }
    }

    #[test]
    fn busy_return_cannot_restore_another_source_phase_or_stale_data() {
        let original = navigation_view(
            &principal(),
            navigation_reply(
                context(11),
                NavigationAction::SelectCharacter,
                NavigationPhase::Channels,
            ),
        )
        .unwrap();
        let mut reply = original.navigation.clone().unwrap();
        reply.action = NavigationAction::ReturnCharacters;
        reply.status = NavigationStatus::Busy;
        for index in 0..5 {
            let mut prior = original.clone();
            match index {
                0 => prior.selected_source = Some(42),
                1 => prior.phase = LoginPhase::CharactersReady,
                2 => prior.navigation.as_mut().unwrap().context.epoch -= 1,
                3 => prior.navigation = None,
                _ => {
                    let mut data = super::super::character_data::tests::sample_data();
                    data.context = context(12);
                    data.character.key = data.context.key;
                    prior.character_data = CharacterDataState::Ready(Arc::new(data));
                }
            }
            assert!(
                busy_return_view(&principal(), &prior, reply.clone()).is_err(),
                "case {index}"
            );
        }
    }

    #[test]
    fn channel_rejection_is_visible_retryable_and_does_not_invent_a_join() {
        let mut reply = navigation_reply(
            context(11),
            NavigationAction::SelectCharacter,
            NavigationPhase::Channels,
        );
        reply.snapshot.selected.as_mut().unwrap().level = 80;
        reply.snapshot.channels[3].minimum_level = 15;
        reply.snapshot.channels[3].maximum_level = 39;
        reply.snapshot.channels[3].population = 20;
        let view = navigation_view(&principal(), reply.clone()).unwrap();
        let (mut controller, release) = controller_with(view);
        assert!(controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 4
            })
            .is_err());
        assert_eq!(controller.snapshot().phase, LoginPhase::ChoosingChannel);
        assert_eq!(controller.next_selection, 12);
        controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 3,
            })
            .unwrap();
        let request = take_navigation(&controller);
        assert_eq!(
            request.channel_id,
            Some(3),
            "level 80 may request tier 15..39 even when the snapshot looked full"
        );
        assert_eq!(controller.snapshot().phase, LoginPhase::JoiningLobby);
        reply.action = NavigationAction::EnterChannel;
        reply.status = NavigationStatus::Busy;
        let refused = navigation_view(&principal(), reply).unwrap();
        assert!(refused.status.contains("忙碌"));
        publish_navigation(
            &controller.active.as_ref().unwrap().published,
            refused,
            11,
            request.refresh_epoch,
        );
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::ChoosingChannel);
        assert_eq!(
            controller.snapshot().navigation.as_ref().unwrap().status,
            NavigationStatus::Busy
        );
        controller
            .command(LoginCommand::EnterChannel {
                context: context(11),
                channel_id: 1,
            })
            .unwrap();
        assert_eq!(take_navigation(&controller).channel_id, Some(1));
        release.send(()).unwrap();
    }

    #[test]
    fn rapid_source_selection_coalesces_and_never_publishes_a_stale_roster() {
        let (mut controller, release) = controller_with(sources(LoginPhase::ChoosingSource));
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        controller.command(LoginCommand::SelectSource(42)).unwrap();
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        let active = controller.active.as_ref().unwrap();
        let queued = active.inbox.lock().unwrap().selection.unwrap();
        assert_eq!((queued.source, queued.epoch), (7, 3));
        *active.published.lock().unwrap() = row(42, 2);
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::LoadingCharacters);
        assert_eq!(controller.snapshot().selected_source, Some(7));
        assert!(controller.snapshot().characters.is_empty());
        // An old response with the SAME source id must still be rejected: the
        // user returned to source 7 in a different request generation.
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = row(7, 1);
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::LoadingCharacters);
        assert!(controller.snapshot().characters.is_empty());
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = row(7, 3);
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::CharactersReady);
        assert_eq!(controller.snapshot().characters[0].source_id, 7);
        assert_eq!(controller.snapshot().characters[0].object_id, 3333);
        let next_epoch = controller.next_selection;
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        assert_eq!(
            controller.next_selection, next_epoch,
            "same-source selection must not reload"
        );
        release.send(()).unwrap();
    }

    #[test]
    fn queued_login_cannot_reenable_or_duplicate_authentication() {
        let (mut controller, release) = controller_with(empty(LoginPhase::LoginReady, "ready"));
        controller
            .command(LoginCommand::Login {
                account: "synthetic".into(),
                password: "secret".into(),
            })
            .unwrap();
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::Authenticating);
        assert!(controller
            .command(LoginCommand::Login {
                account: "another".into(),
                password: "secret".into()
            })
            .is_err());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .credentials
            .is_some());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(controller.retiring[0]
            .inbox
            .lock()
            .unwrap()
            .credentials
            .is_none());
        assert!(controller.snapshot().sources.is_empty());
        assert!(controller.snapshot().characters.is_empty());
        release.send(()).unwrap();
    }

    #[test]
    fn unknown_source_and_epoch_exhaustion_send_nothing_and_preserve_the_view() {
        let (mut controller, release) = controller_with(sources(LoginPhase::ChoosingSource));
        let before = controller.snapshot();
        assert!(controller.command(LoginCommand::SelectSource(99)).is_err());
        assert_eq!(controller.snapshot(), before);
        controller.next_selection = u64::MAX;
        assert!(controller.command(LoginCommand::SelectSource(7)).is_err());
        assert_eq!(controller.snapshot(), before);
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .selection
            .is_none());
        release.send(()).unwrap();
    }

    #[test]
    fn reconnect_same_character_rejects_the_previous_connections_context() {
        let (mut controller, release_first) = controller_with(sources(LoginPhase::ChoosingSource));
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        let first_epoch = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .selection
            .take()
            .unwrap()
            .epoch;
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = row(7, first_epoch);
        controller.poll();
        let stale = context(first_epoch);
        let next_epoch = controller.next_selection;
        controller.command(LoginCommand::Disconnect).unwrap();
        release_first.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !controller.is_quiescent() {
            controller.poll();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }

        // Exercise the same new-owner installation used by connect(), with a
        // controlled worker instead of making a socket to an unrelated service.
        let (worker, release_second) = controlled_worker(Published {
            view: sources(LoginPhase::ChoosingSource),
            source_epoch: 0,
            navigation_epoch: 0,
        });
        let next_connection = controller.next_connection.checked_add(1).unwrap();
        controller.install_connection(next_connection, worker);
        assert_eq!(controller.next_selection, next_epoch);
        controller.poll();
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        let second_epoch = controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .selection
            .take()
            .unwrap()
            .epoch;
        assert!(second_epoch > first_epoch);
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = row(7, second_epoch);
        controller.poll();
        let before = controller.snapshot();
        assert!(controller
            .command(LoginCommand::SelectCharacter(stale))
            .is_err());
        assert_eq!(controller.snapshot(), before);
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .pending_navigation
            .is_none());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .navigation
            .is_none());
        controller
            .command(LoginCommand::SelectCharacter(context(second_epoch)))
            .unwrap();
        assert_eq!(take_navigation(&controller).context.epoch, second_epoch);
        release_second.send(()).unwrap();
    }

    #[test]
    fn reconnect_cannot_reset_an_exhausted_lifetime_selection_epoch() {
        let mut controller = SessionController::new(profile());
        controller.next_selection = u64::MAX;
        let before = controller.snapshot();
        let next_connection = controller.next_connection;
        assert!(controller.command(LoginCommand::Connect).is_err());
        assert_eq!(controller.snapshot(), before);
        assert_eq!(controller.next_selection, u64::MAX);
        assert_eq!(controller.next_connection, next_connection);
        assert!(
            controller.is_quiescent(),
            "exhaustion must reject before spawning a worker"
        );
    }

    #[test]
    fn navigation_epoch_exhaustion_does_not_enqueue_or_mutate_the_selected_page() {
        let (mut controller, release) = controller_with(row(7, u64::MAX - 1).view);
        assert_eq!(controller.next_selection, u64::MAX);
        let before = controller.snapshot();
        assert!(controller
            .command(LoginCommand::SelectCharacter(context(u64::MAX - 1)))
            .is_err());
        assert_eq!(controller.snapshot(), before);
        assert_eq!(controller.next_selection, u64::MAX);
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .pending_navigation
            .is_none());
        assert!(controller
            .active
            .as_ref()
            .unwrap()
            .inbox
            .lock()
            .unwrap()
            .navigation
            .is_none());
        controller.command(LoginCommand::Disconnect).unwrap();
        assert!(controller.retiring[0].cancel.is_cancelled());
        release.send(()).unwrap();
    }

    #[test]
    fn disconnect_ignores_late_success_and_shutdown_prevents_reconnection() {
        let (mut controller, release) = controller_with(sources(LoginPhase::ChoosingSource));
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        controller.command(LoginCommand::Shutdown).unwrap();
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnecting);
        assert!(!controller.is_quiescent());
        assert!(controller.retiring[0].cancel.is_cancelled());
        assert!(controller.retiring[0]
            .inbox
            .lock()
            .unwrap()
            .selection
            .is_none());
        *controller.retiring[0].published.lock().unwrap() = row(7, 1);
        controller.poll();
        assert!(controller.snapshot().characters.is_empty());
        assert!(controller.command(LoginCommand::Connect).is_err());
        release.send(()).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        while !controller.is_quiescent() {
            controller.poll();
            assert!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(controller.snapshot().phase, LoginPhase::Disconnected);
        assert!(controller.command(LoginCommand::Connect).is_err());
    }

    #[test]
    fn terminal_failure_is_not_hidden_by_a_newer_source_selection() {
        let (mut controller, release) = controller_with(sources(LoginPhase::ChoosingSource));
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        controller.command(LoginCommand::SelectSource(42)).unwrap();
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = Published {
            view: empty(LoginPhase::Failed, "synthetic transport failure"),
            source_epoch: 0,
            navigation_epoch: 0,
        };
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::Failed);
        assert!(controller.snapshot().sources.is_empty());
        assert!(controller.snapshot().characters.is_empty());
        release.send(()).unwrap();
    }

    #[test]
    fn worker_cap_and_connection_epoch_cannot_be_bypassed() {
        let mut controller = SessionController::new(profile());
        let (first, release_first) = controlled_worker(Published {
            view: empty(LoginPhase::Disconnected, ""),
            source_epoch: 0,
            navigation_epoch: 0,
        });
        let (second, release_second) = controlled_worker(Published {
            view: empty(LoginPhase::Disconnected, ""),
            source_epoch: 0,
            navigation_epoch: 0,
        });
        controller.retiring = vec![first, second];
        assert!(controller.command(LoginCommand::Connect).is_err());
        assert_eq!(controller.next_connection, 1);
        assert!(controller.active.is_none());
        release_first.send(()).unwrap();
        release_second.send(()).unwrap();
        let mut exhausted = SessionController::new(profile());
        exhausted.next_connection = u32::MAX;
        assert!(exhausted.command(LoginCommand::Connect).is_err());
        assert!(exhausted.is_quiescent());
    }

    #[test]
    fn busy_worker_locks_cannot_block_ui_poll_commands_or_cancellation() {
        let (mut controller, release_worker) =
            controller_with(empty(LoginPhase::LoginReady, "ready"));
        let active = controller.active.as_ref().unwrap();
        let published = active.published.clone();
        let inbox = active.inbox.clone();
        let (ready, started) = mpsc::sync_channel(1);
        let (release_lock, wait_release) = mpsc::sync_channel(1);
        let holder = thread::spawn(move || {
            let _published_guard = published.lock().unwrap();
            let _inbox_guard = inbox.lock().unwrap();
            ready.send(()).unwrap();
            let _ = wait_release.recv_timeout(Duration::from_secs(2));
        });
        started.recv_timeout(Duration::from_secs(2)).unwrap();
        let began = Instant::now();
        controller.poll();
        assert!(controller
            .command(LoginCommand::Login {
                account: "synthetic".into(),
                password: "secret".into()
            })
            .is_err());
        assert_eq!(controller.snapshot().phase, LoginPhase::LoginReady);
        controller.view = sources(LoginPhase::ChoosingSource);
        assert!(controller.command(LoginCommand::SelectSource(7)).is_err());
        assert_eq!(controller.next_selection, 1);
        controller.view = row(7, 1).view;
        assert!(controller
            .command(LoginCommand::SelectCharacter(context(1)))
            .is_err());
        assert_eq!(controller.next_selection, 1);
        controller.command(LoginCommand::Shutdown).unwrap();
        let elapsed = began.elapsed();
        // Always release the holder before asserting, even on a regression.
        let _ = release_lock.try_send(());
        holder.join().unwrap();
        release_worker.send(()).unwrap();
        assert!(
            elapsed < Duration::from_millis(250),
            "UI waited for worker mutex: {elapsed:?}"
        );
        assert!(controller.retiring[0].cancel.is_cancelled());
        assert!(controller.snapshot().characters.is_empty());
    }

    #[test]
    fn old_source_error_does_not_replace_new_loading_and_current_error_is_retryable() {
        let (mut controller, release) = controller_with(sources(LoginPhase::ChoosingSource));
        controller.command(LoginCommand::SelectSource(7)).unwrap();
        controller.command(LoginCommand::SelectSource(42)).unwrap();
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = Published {
            view: sources(LoginPhase::ChoosingSource),
            source_epoch: 1,
            navigation_epoch: 0,
        };
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::LoadingCharacters);
        assert_eq!(controller.snapshot().selected_source, Some(42));
        *controller
            .active
            .as_ref()
            .unwrap()
            .published
            .lock()
            .unwrap() = Published {
            view: sources(LoginPhase::ChoosingSource),
            source_epoch: 2,
            navigation_epoch: 0,
        };
        controller.poll();
        assert_eq!(controller.snapshot().phase, LoginPhase::ChoosingSource);
        controller.command(LoginCommand::SelectSource(42)).unwrap();
        assert_eq!(controller.snapshot().phase, LoginPhase::LoadingCharacters);
        assert_eq!(controller.next_selection, 4);
        release.send(()).unwrap();
    }
}
