//! Player-controller commands and exact receipts for the canonical TLS path.
//! Polling never acknowledges an enqueued UI command. No legacy Frame is
//! synthesized, and neither a received observation nor this state grants LOAD.
use super::{
    application_protocol as app, battle_action::Guard, battle_follow::Observation,
    battle_operation as operation, battle_state::Snapshot, battle_world as world, rooms,
    transport::ModernTlsConnection,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

#[derive(Clone, PartialEq, Eq)]
pub(crate) enum Operation {
    Move(super::world_protocol::Request),
    Attack(super::combat_protocol::Request),
    Spell(super::spell_protocol::CastRequest),
    Item(super::item_protocol::Request),
    Ammo(super::ammo_protocol::Request),
    Npc(super::npc_protocol::Request),
    NpcSpell(super::npc_spell_protocol::Request),
    Siege(super::siege_protocol::Request),
    Disintegrate(super::disintegrate_cast_protocol::Request),
    NpcDisintegrate(super::npc_disintegrate_protocol::Request),
    NpcFireStorm(super::npc_fire_storm_protocol::Request),
}
impl Operation {
    fn encode(&self, id: u32, version: u16) -> Result<Vec<u8>, app::ApplicationError> {
        Ok(match self {
            Self::Move(q) => super::world_protocol::encode_request(id, q)?,
            Self::Attack(q) => super::combat_protocol::encode_request(id, q)?,
            Self::Spell(q) => super::spell_protocol::encode_cast_version(id, q, version)?,
            Self::Item(q) => super::item_protocol::encode_request(id, *q, version)?,
            Self::Ammo(q) => super::ammo_protocol::encode_request(id, *q, version)?,
            Self::Npc(q) => super::npc_protocol::encode_request(id, q)?,
            Self::NpcSpell(q) => super::npc_spell_protocol::encode_request(id, q, version)?,
            Self::Siege(q) => super::siege_protocol::encode_request(id, q)?,
            Self::Disintegrate(q) => {
                super::disintegrate_cast_protocol::encode_request(id, q, version)?
            }
            Self::NpcDisintegrate(q) => {
                super::npc_disintegrate_protocol::encode_request(id, q, version)?
            }
            Self::NpcFireStorm(q) => {
                super::npc_fire_storm_protocol::encode_request(id, q, version)?
            }
        })
    }
}
#[derive(Clone, PartialEq, Eq)]
enum Kind {
    GreaterHaste(super::battle_greater_haste::Intent),
    TargetHaste(super::battle_target_haste::Intent),
    Slow(super::battle_slow::Intent),
    PlayerHealing(super::battle_player_healing::Intent),
    World(world::Request),
    VipWorld(world::Request),
    VipEquipment(super::battle_vip_equipment::Request),
    Operation(Guard, Operation),
    Revival(super::battle_revival::Request),
    Equipment(super::battle_equipment::Request),
}
#[derive(Clone, PartialEq, Eq)]
pub struct Command(Kind);
impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CanonicalPlayerCommand[redacted]")
    }
}
impl Command {
    /// Test-only operation class; never expose raw commands, identities or text.
    #[cfg(test)]
    pub(crate) fn audit_kind(&self) -> &'static str {
        match &self.0 {
            Kind::GreaterHaste(_) => "greater-haste",
            Kind::TargetHaste(_) => "target-haste",
            Kind::Slow(_) => "slow",
            Kind::PlayerHealing(_) => "player-healing",
            Kind::World(q) => match q.action {
                world::Action::Read => "world-read",
                world::Action::Prepare => "world-prepare",
                world::Action::Loaded => "world-loaded",
                world::Action::Leave => "world-leave",
            },
            Kind::VipWorld(q) => match q.action {
                world::Action::Read => "vip-world-read",
                world::Action::Prepare => "vip-world-prepare",
                world::Action::Loaded => "vip-world-loaded",
                world::Action::Leave => "vip-world-leave",
            },
            Kind::VipEquipment(_) => "vip-equipment",
            Kind::Operation(_, operation) => match operation {
                Operation::Move(_) => "move",
                Operation::Attack(_) => "attack",
                Operation::Spell(_) => "spell",
                Operation::Item(_) => "item",
                Operation::Ammo(_) => "ammo",
                Operation::Npc(_) => "npc-attack",
                Operation::NpcSpell(_) => "npc-spell",
                Operation::Siege(_) => "siege",
                Operation::Disintegrate(_) => "disintegrate",
                Operation::NpcDisintegrate(_) => "npc-disintegrate",
                Operation::NpcFireStorm(_) => "npc-fire-storm",
            },
            Kind::Revival(_) => "revival",
            Kind::Equipment(_) => "equipment",
        }
    }
    pub(crate) fn slow(q: super::battle_slow::Intent) -> Self {
        Self(Kind::Slow(q))
    }
    pub(crate) fn player_healing(q: super::battle_player_healing::Intent) -> Self {
        Self(Kind::PlayerHealing(q))
    }
    pub(crate) fn target_haste(q: super::battle_target_haste::Intent) -> Self {
        Self(Kind::TargetHaste(q))
    }
    pub(crate) fn greater_haste(q: super::battle_greater_haste::Intent) -> Self {
        Self(Kind::GreaterHaste(q))
    }
    pub(crate) fn equipment(q: super::battle_equipment::Request) -> Self {
        Self(Kind::Equipment(q))
    }
    pub(crate) fn revival(q: super::battle_revival::Request) -> Self {
        Self(Kind::Revival(q))
    }
    pub(crate) fn world(q: world::Request) -> Self {
        Self(Kind::World(q))
    }
    pub(crate) fn vip_world(q: world::Request) -> Self {
        Self(Kind::VipWorld(q))
    }
    pub(crate) fn vip_equipment(q: super::battle_vip_equipment::Request) -> Self {
        Self(Kind::VipEquipment(q))
    }
    pub(crate) fn operation(guard: Guard, q: Operation) -> Self {
        Self(Kind::Operation(guard, q))
    }
    pub(crate) fn leaving(&self) -> bool {
        matches!(&self.0, Kind::World(q) | Kind::VipWorld(q) if q.action == world::Action::Leave)
    }
    pub(crate) fn scope(&self) -> (app::SelectionContext, u8) {
        match &self.0 {
            Kind::World(q) | Kind::VipWorld(q) => (q.context, q.channel),
            Kind::VipEquipment(q) => (q.stamp.context, q.stamp.channel),
            Kind::Operation(g, _) => (g.stamp.context, g.stamp.channel),
            Kind::Revival(q) => (q.stamp.context, q.stamp.channel),
            Kind::Equipment(q) => (q.stamp.context, q.stamp.channel),
            Kind::GreaterHaste(q) => (q.stamp.context, q.stamp.channel),
            Kind::TargetHaste(q) => (q.stamp.context, q.stamp.channel),
            Kind::Slow(q) => (q.stamp.context, q.stamp.channel),
            Kind::PlayerHealing(q) => (q.stamp.context, q.stamp.channel),
        }
    }
    pub(crate) fn validate(&self) -> Result<(), String> {
        match &self.0 {
            Kind::GreaterHaste(q) => {
                super::battle_greater_haste::encode_intent(q, super::battle_greater_haste::VERSION)
            }
            Kind::TargetHaste(q) => {
                super::battle_target_haste::encode_intent(q, super::battle_target_haste::VERSION)
            }
            Kind::Slow(q) => super::battle_slow::encode_intent(q, super::battle_slow::VERSION),
            // Syntax only. Complete selection is checked by current() and the
            // TLS owner; a serializable command is not permission to send.
            Kind::PlayerHealing(q) => super::battle_player_healing::encode_candidate_intent(q),
            Kind::Equipment(q) => {
                super::battle_equipment::encode_request(1, q, super::battle_equipment::VERSION)
            }
            Kind::Revival(q) => {
                super::battle_revival::encode_request(1, q, super::battle_revival::VERSION)
            }
            Kind::World(q) | Kind::VipWorld(q) => world::encode_request(1, q, world::VERSION),
            Kind::VipEquipment(q) => q.validate().map(|_| Vec::new()),
            Kind::Operation(g, q) => q
                .encode(1, world::VERSION)
                .and_then(|inner| operation::encode(*g, &inner, world::VERSION)),
        }
        .map(|_| ())
        .map_err(|_| "新版戰鬥指令的型別、生命或場次不一致".into())
    }
    pub(crate) fn current(&self, state: &State, nav: Option<&app::NavigationReply>) -> bool {
        if !state.supported
            || !nav.is_some_and(|n| {
                n.snapshot.phase == app::NavigationPhase::Lobby
                    && (n.context, n.snapshot.channel_id) == (self.scope().0, Some(self.scope().1))
            })
            || state
                .snapshot
                .as_ref()
                .is_some_and(|s| s.scope() != self.scope())
        {
            return false;
        }
        match &self.0 {
            Kind::PlayerHealing(q) => {
                state.player_healing_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| q.current(o))
            }
            Kind::GreaterHaste(q) => {
                state.greater_haste_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| q.current(o))
            }
            Kind::TargetHaste(q) => {
                state.target_haste_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| q.current(o))
            }
            Kind::Slow(q) => {
                state.slow_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| q.current(o))
            }
            Kind::Equipment(q) => {
                state.equipment_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| q.current(o))
            }
            Kind::Revival(q) => {
                state.revival_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.observation().is_some_and(|o| {
                        q.stamp.current(o)
                            && super::battle_revival::Request::from_observation(o)
                                .is_some_and(|current| current.death_revision == q.death_revision)
                    })
            }
            Kind::VipEquipment(q) => {
                state.vip_equipment_supported
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
                    && state.vip_observation().is_some_and(|o| q.current(o))
            }
            Kind::World(q) | Kind::VipWorld(q) => {
                if matches!(&self.0, Kind::VipWorld(_)) != state.vip_world_supported {
                    return false;
                }
                match state.snapshot.as_ref().and_then(|s| s.attachment()) {
                    Some(a) => {
                        q.action != world::Action::Prepare
                            && (q.match_id == 0
                                || [q.room, q.match_id, q.generation]
                                    == [a.world.room, a.world.match_id, a.world.generation])
                    }
                    None => {
                        q.match_id == 0
                            && matches!(q.action, world::Action::Read | world::Action::Prepare)
                    }
                }
            }
            Kind::Operation(g, _) => {
                state.observation().is_some_and(|o| g.stamp.current(o))
                    && state
                        .lifecycle()
                        .is_some_and(|l| l.phase == super::world_protocol::Phase::Active)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Outcome {
    NotSent,
    VipWorld(super::world_protocol::Status, super::world_protocol::Reason),
    VipEquipment(
        super::battle_vip_equipment::Rejection,
        super::battle_vip_equipment::Outcome,
    ),
    GreaterHaste(super::battle_greater_haste::Reason),
    TargetHaste(super::battle_target_haste::Reason),
    Slow(super::battle_slow::Reason),
    PlayerHealing(super::battle_player_healing::Reason),
    Equipment(
        super::battle_equipment::Reason,
        super::battle_equipment::Disposition,
        Option<super::equipment::Status>,
    ),
    Revival(super::battle_revival::Reason),
    World(super::world_protocol::Status, super::world_protocol::Reason),
    Operation(
        super::battle_action::Rejection,
        operation::Disposition,
        operation::Reason,
    ),
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Receipt {
    epoch: u64,
    command: Command,
    outcome: Outcome,
    equipment: Option<Arc<super::equipment::Reply>>,
    vip_equipment: Option<(
        Arc<super::battle_vip_equipment::Reply>,
        Option<super::battle_vip_equipment::Quote>,
    )>,
}
#[derive(Clone, Debug, Default)]
pub struct State {
    supported: bool,
    revival_supported: bool,
    equipment_supported: bool,
    greater_haste_supported: bool,
    target_haste_supported: bool,
    slow_supported: bool,
    player_healing_supported: bool,
    vip_world_supported: bool,
    vip_equipment_supported: bool,
    snapshot: Option<Arc<Snapshot>>,
    receipt: Option<Arc<Receipt>>,
}
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.supported == other.supported
            && self.revival_supported == other.revival_supported
            && self.equipment_supported == other.equipment_supported
            && self.greater_haste_supported == other.greater_haste_supported
            && self.target_haste_supported == other.target_haste_supported
            && self.slow_supported == other.slow_supported
            && self.player_healing_supported == other.player_healing_supported
            && self.vip_world_supported == other.vip_world_supported
            && self.vip_equipment_supported == other.vip_equipment_supported
            && match (&self.snapshot, &other.snapshot) {
                (Some(a), Some(b)) => Arc::ptr_eq(a, b),
                (None, None) => true,
                _ => false,
            }
            && self.receipt == other.receipt
    }
}
impl Eq for State {}
impl State {
    pub(crate) fn vip_world_supported(&self) -> bool {
        self.vip_world_supported
    }
    pub(crate) fn leave_command(&self) -> Option<Command> {
        let s = self.snapshot.as_ref()?;
        let a = s.attachment()?;
        let q = world::Request {
            action: world::Action::Leave,
            ..world::Request::read(s.scope().0, s.scope().1, Some(&a.world))
        };
        Some(if s.full_vip() {
            Command::vip_world(q)
        } else {
            Command::world(q)
        })
    }
    /// Full-VIP authority data. A separate owner-checked player adapter may
    /// seed read-only preview pixels; this accessor never grants shared LOAD
    /// or legacy gameplay-command eligibility.
    pub(crate) fn vip_observation(&self) -> Option<&Observation> {
        let snapshot = self.snapshot.as_ref().filter(|s| s.full_vip())?;
        snapshot.attachment().filter(|a| {
            !matches!(
                a.lifecycle.phase,
                super::world_protocol::Phase::Preparing | super::world_protocol::Phase::Aborted
            )
        })?;
        snapshot.observation()
    }
    pub(crate) fn vip_equipment_reply(&self) -> Option<&super::battle_vip_equipment::Reply> {
        Some(self.receipt.as_ref()?.vip_equipment.as_ref()?.0.as_ref())
    }
    pub(crate) fn vip_quote(&self) -> Option<super::battle_vip_equipment::Quote> {
        let quote = self.receipt.as_ref()?.vip_equipment.as_ref()?.1?;
        quote.current(self.vip_observation()?).then_some(quote)
    }
    pub(crate) fn player_healing_supported(&self) -> bool {
        self.player_healing_supported
    }
    pub(crate) fn slow_supported(&self) -> bool {
        self.slow_supported
    }
    pub(crate) fn target_haste_supported(&self) -> bool {
        self.target_haste_supported
    }
    pub(crate) fn greater_haste_supported(&self) -> bool {
        self.greater_haste_supported
    }
    pub(crate) fn equipment_supported(&self) -> bool {
        self.equipment_supported
    }
    pub(crate) fn equipment_reply(&self) -> Option<&super::equipment::Reply> {
        self.receipt.as_ref()?.equipment.as_deref()
    }
    pub(crate) fn supported(&self) -> bool {
        self.supported
    }
    pub(crate) fn snapshot(&self) -> Option<&Arc<Snapshot>> {
        self.snapshot.as_ref()
    }
    pub(crate) fn attached(&self) -> bool {
        self.snapshot
            .as_ref()
            .is_some_and(|s| s.attachment().is_some())
    }
    pub(crate) fn lifecycle(&self) -> Option<&operation::Lifecycle> {
        Some(&self.snapshot.as_ref()?.attachment()?.lifecycle)
    }
    pub(crate) fn observation(&self) -> Option<&Observation> {
        self.snapshot.as_ref()?.presentation_observation()
    }
    pub(crate) fn confirmed(&self, command: &Command, epoch: u64) -> bool {
        self.receipt
            .as_ref()
            .is_some_and(|r| r.epoch == epoch && r.command == *command)
    }
    pub(crate) fn outcome(&self) -> Option<Outcome> {
        self.receipt.as_ref().map(|r| r.outcome)
    }
    #[cfg(test)]
    pub(crate) fn receipt_audit(&self) -> Option<(u64, &'static str, Outcome)> {
        self.receipt
            .as_ref()
            .map(|r| (r.epoch, r.command.audit_kind(), r.outcome))
    }
    pub(crate) fn receipt_epoch(&self) -> Option<u64> {
        self.receipt.as_ref().map(|r| r.epoch)
    }
    #[cfg(test)]
    pub(crate) fn test_receipt(command: Command, epoch: u64, outcome: Outcome) -> Self {
        Self { receipt: Some(Arc::new(Receipt { epoch, command, outcome,
            equipment: None, vip_equipment: None })), ..Self::default() }
    }
    /// A route must see this exact command's new receipt, not a passive pose
    /// update or an old receipt from before local admission.
    pub(crate) fn received_after(&self, command: &Command, prior: Option<u64>) -> Option<Outcome> {
        self.receipt
            .as_ref()
            .filter(|r| r.command == *command && prior.is_none_or(|old| r.epoch > old))
            .map(|r| r.outcome)
    }
    pub(crate) fn notice(&self) -> &'static str {
        use super::world_protocol::{Phase as P, SettlementState as S};
        if self.vip_world_supported
            && self
                .lifecycle()
                .is_some_and(|l| matches!(l.phase, P::Loading | P::Countdown | P::Active))
        {
            return "完整數值世界已確認；共享資源呈現尚未完成，不代表畫面已就緒。";
        }
        match self.lifecycle() {
            None if self.snapshot.is_none() => "尚未確認世界附著。",
            None => "已確認返回房間；世界附著已清理。",
            Some(l) => match l.phase {
                P::Preparing => "伺服器正在準備戰局；尚未取得引擎或資源載入資格。",
                P::Loading => "正在載入共享資源；完整畫面與資源就緒後才確認進場。",
                P::Countdown => "玩家已完成載入，等待伺服器倒數。",
                P::Active => "戰鬥已開始。",
                P::Aborted => "本場已中止；請確認離場後再準備下一場。",
                P::Finished => match l.settlement.or(l.race_settlement) {
                    Some(S::Saved) => "本場結束，戰績已由伺服器確認保存。",
                    Some(S::Pending) => "本場結束，伺服器仍在保存戰績。",
                    Some(S::Failed) => "本場結束，但戰績保存失敗；沒有宣稱已儲存。",
                    None => "本場已結束。",
                },
            },
        }
    }
    pub(crate) fn command_notice(&self) -> &'static str {
        match self.outcome() {
            Some(Outcome::VipEquipment(_, outcome)) => {
                use super::battle_vip_equipment::Outcome as V;
                match outcome {
                    V::Inspected => "完整數值裝備已重新檢查；尚未執行穿脫。",
                    V::Applied => "完整數值裝備已保存並套用本場；畫面仍需完成資源呈現。",
                    V::Unchanged => "伺服器確認完整數值裝備未變；沒有重複保存。",
                    V::PersistedRetired => "裝備已保存但本場已失效；請離場重新讀取，不要重送。",
                    V::Conflict => "裝備来源或保存版本已變更；沒有提交，請重新檢查。",
                    V::Denied => "裝備條件不符；伺服器未接受穿脫。",
                    V::Rejected => "生命、能力、期限或場次已變更；未提交，沒有自動重送。",
                }
            }
            Some(Outcome::GreaterHaste(reason)) => {
                use super::battle_greater_haste::Reason as R;
                match reason {
                    R::None => "強力加速術已由伺服器確認；耗魔與加速期限採權威結果。",
                    R::TooFast => "施法間隔尚未結束；沒有耗魔或刷新加速，也沒有自動重送。",
                    R::NoMp => "魔力不足；強力加速術未生效。",
                    R::TooHeavy => "負重過高；強力加速術未生效。",
                    R::Unlearned | R::Unsupported => {
                        "角色未學會或来源模板未支援強力加速術；沒有施放。"
                    }
                    R::Dead | R::StaleState => {
                        "生命、裝備、位置或加速狀態已變更；請依新畫面重新確認。"
                    }
                    R::NotActive => "本場尚未開始或已結束；強力加速術未生效。",
                    R::Numeric => "伺服器無法安全計算強力加速術；沒有宣稱生效。",
                }
            }
            Some(Outcome::TargetHaste(reason)) => {
                use super::battle_target_haste::Reason as R;
                match reason {
                    R::None => "加速術已由伺服器確認；耗魔與加速期限採權威結果。",
                    R::TooFast => "施法間隔尚未結束；沒有耗魔或刷新加速，也沒有自動重送。",
                    R::NoMp => "魔力不足；加速術未生效。",
                    R::TooHeavy => "負重過高；加速術未生效。",
                    R::Unlearned | R::Unsupported => "角色未學會或來源模板未支援加速術；沒有施放。",
                    R::Dead | R::StaleState => {
                        "生命、裝備、位置或加速狀態已變更；請依新畫面重新確認。"
                    }
                    R::NotActive => "本場尚未開始或已結束；加速術未生效。",
                    R::TargetDead | R::StaleTarget => "目標生命、位置或效果已更新；請重新選取。",
                    R::WrongInstance => "目標不屬於目前戰鬥空間。",
                    R::OutOfRange => "目標超出施法範圍。",
                    R::Numeric => "伺服器無法安全計算加速術；沒有宣稱生效。",
                }
            }
            Some(Outcome::Slow(reason)) => {
                use super::battle_slow::Reason as R;
                match reason {
                    R::None => "緩速施法已確認；扣魔、材料、命中或免疫結果以伺服器為準。",
                    R::NoMaterial => "魔法寶石不足；未扣魔或施放。",
                    R::TooFast => "施法間隔尚未結束；沒有自動重送。",
                    R::NoMp => "魔力不足；未施放緩速術。",
                    R::TooHeavy => "負重過高；未施放緩速術。",
                    R::Unlearned | R::Unsupported => "角色未學會或來源模板尚未支援緩速術。",
                    R::Dead | R::StaleState | R::StaleTarget | R::TargetDead => {
                        "角色或目標狀態已變更；請依新畫面重新選取。"
                    }
                    R::NotActive => "本場尚未開始或已結束。",
                    R::WrongInstance | R::SameTeam => "目標不符合本場空間或隊伍規則。",
                    R::OutOfRange | R::Blocked => "目標超出範圍或施法路徑受阻。",
                    R::Numeric => "伺服器無法安全計算緩速術；未宣稱生效。",
                }
            }
            Some(Outcome::PlayerHealing(reason)) => {
                use super::battle_player_healing::Reason as R;
                match reason {
                    R::None => "治癒已由伺服器確認；恢復量及耗魔以回覆為準。",
                    R::TooFast => "施法間隔尚未結束；沒有自動重送。",
                    R::NoMp => "魔力不足；未施放治癒術。",
                    R::TooHeavy => "負重過高；未施放治癒術。",
                    R::Unlearned | R::Unsupported => "角色未學會或來源尚未支援治癒術。",
                    R::NotActive => "本場尚未開始或已結束；治癒未生效。",
                    R::Dead | R::TargetDead | R::StaleState | R::StaleTarget => {
                        "角色或目標生命、裝備或位置已更新；請重新確認。"
                    }
                    R::WrongInstance => "目標不符合本場空間或隊伍規則。",
                    R::OutOfRange | R::Blocked => "目標超出範圍或施法路徑受阻。",
                    R::Numeric => "伺服器無法安全計算治癒；未宣稱生效。",
                }
            }
            Some(Outcome::NotSent) => {
                "角色生命、場次或裝備版本在送出前已更新；本次沒有送出，請重新確認。"
            }
            Some(Outcome::Equipment(reason, disposition, status)) => {
                use super::battle_equipment::{Disposition as D, Reason as R};
                use super::equipment::Status as S;
                match (reason, disposition, status) {
                    (_, D::PersistedRetired, _) => "裝備已保存，但本場已失效；未套用即時戰鬥。返回房間後重新讀取，請勿重送。",
                    (_, D::Applied, _) => "穿脫已由伺服器保存並套用本場；等待共享場景更新。",
                    (_, D::Unchanged, _) => "伺服器確認裝備已是指定狀態，沒有重複變更。",
                    (R::StaleLife | R::Dead, _, _) => "生命已更新或角色已死亡；本次沒有穿脫，請依新畫面重新確認。",
                    (R::NotActive, _, _) => "本場尚未開始或已結束；本次沒有保存裝備變更。",
                    (_, _, Some(S::Conflict)) => "裝備版本或來源已變更；本次未提交。若保存資料與本場不同，請返回房間重新讀取。",
                    (R::Busy, _, _) | (_, _, Some(S::Busy)) => "裝備操作忙碌；本次未提交，也沒有自動重送。",
                    (_, _, Some(S::Denied)) => "此物品的職業、欄位、期限或穿戴條件未符合；伺服器未接受穿脫。",
                    _ => "裝備變更未獲確認；保留權威世界，沒有自動重送。",
                }
            }
            Some(Outcome::Revival(reason)) => match reason {
                super::battle_revival::Reason::Revived => {
                    "伺服器已確認復活；等待新生命的共享場景就緒。"
                }
                super::battle_revival::Reason::SpawnBlocked => {
                    "出生位置暫時被佔用；尚未復活，請稍後重新確認。"
                }
                super::battle_revival::Reason::StaleLife
                | super::battle_revival::Reason::NotDead => {
                    "死亡或生命狀態已更新；本次沒有再次復活，也沒有自動重送。"
                }
                super::battle_revival::Reason::NotActive => "本場已不可復活；保留伺服器確認狀態。",
                super::battle_revival::Reason::Numeric => {
                    "伺服器無法安全完成復活計算；角色尚未復活。"
                }
            },
            Some(Outcome::Operation(_, operation::Disposition::Applied, _)) => {
                "新版戰鬥操作已由伺服器確認。"
            }
            Some(Outcome::Operation(_, operation::Disposition::Rejected, _)) => {
                "操作的生命或目標已失效；已採納權威狀態，沒有自動重送。"
            }
            Some(Outcome::Operation(_, operation::Disposition::Denied, reason)) => match reason {
                operation::Reason::TooFast | operation::Reason::Wait => {
                    "動作間隔尚未結束；本次操作未生效，請稍後重新操作。"
                }
                operation::Reason::StalePosition | operation::Reason::StaleState => {
                    "角色位置或狀態已更新；本次操作未生效，請依目前畫面重新操作。"
                }
                operation::Reason::OutOfRange => "目標超出有效距離；本次操作未生效。",
                operation::Reason::Blocked => "路徑或視線受阻；本次操作未生效。",
                operation::Reason::NoMp => "魔力不足；本次施法未生效。",
                operation::Reason::Unlearned => "角色尚未學會此技能；本次施法未生效。",
                operation::Reason::TooHeavy => "負重超過操作限制；本次操作未生效。",
                _ => "伺服器拒絕本次操作；已更新完整狀態，沒有自動重送。",
            },
            Some(Outcome::Operation(_, operation::Disposition::Unavailable, _)) => {
                "伺服器拒絕本次操作；已更新完整狀態，沒有自動重送。"
            }
            Some(Outcome::World(status, _) | Outcome::VipWorld(status, _))
                if status != super::world_protocol::Status::Ok =>
            {
                "伺服器尚未接受世界操作；保留已確認狀態。"
            }
            _ => self.notice(),
        }
    }
    pub(crate) fn command_status(&self) -> String {
        self.equipment_reply()
            .and_then(super::equipment::Reply::denial_notice)
            .unwrap_or_else(|| self.command_notice().into())
    }
    /// Diagnostics (audio/route/local errors) must not replace an exact receipt.
    /// While a newer command is pending, do not present the old receipt as its ACK.
    pub(crate) fn display_status(&self, pending: bool, diagnostic: Option<&str>) -> String {
        let lifecycle = self.notice();
        let command = if pending {
            "等待伺服器確認目前操作；尚未取得本次結果。".into()
        } else {
            self.command_status()
        };
        let mut text = lifecycle.to_owned();
        if command != lifecycle {
            text.push_str("\r\n");
            text.push_str(&command);
        }
        if let Some(extra) =
            diagnostic.filter(|d| !d.is_empty() && *d != lifecycle && *d != command)
        {
            text.push_str("\r\n");
            text.push_str(extra);
        }
        text
    }
    pub(crate) fn hide(&mut self) {
        self.snapshot = None;
    }
    pub(crate) fn retain(&mut self, scope: Option<(app::SelectionContext, u8)>) {
        if self
            .snapshot
            .as_ref()
            .is_some_and(|s| Some(s.scope()) != scope)
        {
            self.snapshot = None;
            self.receipt = None;
        }
        if self
            .receipt
            .as_ref()
            .is_some_and(|r| Some(r.command.scope()) != scope)
        {
            self.receipt = None;
        }
    }
    pub(crate) fn align(&mut self, connection: &ModernTlsConnection) {
        self.supported = connection.supports_battle_world();
        self.revival_supported = connection.supports_battle_revival();
        self.equipment_supported = connection.supports_battle_equipment();
        self.greater_haste_supported = connection.supports_greater_haste();
        self.target_haste_supported = connection.supports_target_haste();
        self.slow_supported = connection.supports_slow();
        self.player_healing_supported = connection.supports_player_healing();
        self.vip_world_supported = connection.supports_vip_world();
        self.vip_equipment_supported = self.vip_world_supported
            && connection
                .negotiated_selection()
                .is_some_and(|p| p.allows_vip_equipment());
        let next = connection.battle_snapshot().cloned();
        if self.snapshot.as_ref().map(|s| s.scope()) != next.as_ref().map(|s| s.scope()) {
            self.receipt = None;
        }
        self.snapshot = next;
    }
    pub(crate) fn execute(
        &mut self,
        connection: &mut ModernTlsConnection,
        command: Command,
        epoch: u64,
    ) -> Result<(), String> {
        if epoch == 0 || self.receipt.as_ref().is_some_and(|r| epoch <= r.epoch) {
            return Err("新版戰鬥操作世代已失效".into());
        }
        command.validate()?;
        // Recheck on the owned worker immediately before writing. A queued
        // click may outlive a poll, respawn or equipment update.
        self.align(connection);
        if !command.current(self, connection.navigation()) {
            self.receipt = Some(Arc::new(Receipt {
                epoch,
                command,
                outcome: Outcome::NotSent,
                equipment: None,
                vip_equipment: None,
            }));
            return Ok(());
        }
        let mut equipment = None;
        let mut vip_equipment = None;
        let result: Result<Outcome, String> = (|| {
            Ok(match &command.0 {
                Kind::VipWorld(q) => {
                    let r = connection.vip_world(q.clone()).map_err(|e| e.to_string())?;
                    Outcome::VipWorld(r.status(), r.reason())
                }
                Kind::VipEquipment(q) => {
                    let (r, quote) = connection.vip_equipment(*q).map_err(|e| e.to_string())?;
                    let outcome = Outcome::VipEquipment(r.rejection, r.outcome);
                    vip_equipment = Some((Arc::new(r), quote));
                    outcome
                }
                Kind::PlayerHealing(q) => Outcome::PlayerHealing(
                    connection
                        .player_healing(*q)
                        .map_err(|e| e.to_string())?
                        .reason,
                ),
                Kind::GreaterHaste(q) => Outcome::GreaterHaste(
                    connection
                        .greater_haste(*q)
                        .map_err(|e| e.to_string())?
                        .reason,
                ),
                Kind::TargetHaste(q) => Outcome::TargetHaste(
                    connection
                        .target_haste(*q)
                        .map_err(|e| e.to_string())?
                        .reason,
                ),
                Kind::Slow(q) => {
                    Outcome::Slow(connection.slow(*q).map_err(|e| e.to_string())?.reason)
                }
                Kind::Equipment(q) => {
                    let r = connection.battle_equipment(*q).map_err(|e| e.to_string())?;
                    let status = r.equipment.as_ref().map(|e| e.status);
                    equipment = r.equipment.map(Arc::new);
                    Outcome::Equipment(r.reason, r.disposition, status)
                }
                Kind::Revival(q) => Outcome::Revival(
                    connection
                        .battle_revival(*q)
                        .map_err(|e| e.to_string())?
                        .reason,
                ),
                Kind::World(q) => {
                    let r = connection
                        .battle_world(q.clone())
                        .map_err(|e| e.to_string())?;
                    Outcome::World(r.status, r.reason)
                }
                Kind::Operation(g, q) => {
                    let r = connection
                        .battle_operation(*g, |id, version| q.encode(id, version))
                        .map_err(|e| e.to_string())?;
                    Outcome::Operation(r.rejection, r.disposition, r.reason)
                }
            })
        })();
        self.align(connection);
        let outcome = match result {
            Ok(outcome) => outcome,
            Err(error) => {
                self.receipt = None;
                return Err(error);
            }
        };
        self.receipt = Some(Arc::new(Receipt {
            epoch,
            command,
            outcome,
            equipment,
            vip_equipment,
        }));
        Ok(())
    }
}

pub(crate) fn scope_for(
    view: &super::login_window::LoginSnapshot,
) -> Option<(app::SelectionContext, u8)> {
    if view.phase != super::login_window::LoginPhase::InLobby {
        return None;
    }
    let nav = view.navigation.as_ref()?;
    if nav.snapshot.phase != app::NavigationPhase::Lobby
        || view.selected_source != Some(nav.context.key.source_id)
    {
        return None;
    }
    Some((nav.context, nav.snapshot.channel_id?))
}

#[cfg(test)]
#[path = "battle_control_tests.rs"]
pub(crate) mod tests;

/// Auto preparation is serialized by the worker's existing command barrier.
/// Only explicit pre-capture CAPACITY refusal is retryable; never replay a
/// command after an uncertain exchange or skip a previous world's LEAVE.
#[derive(Default)]
pub(crate) struct Poller {
    attempted: Option<(app::SelectionContext, u8, u64, i64)>,
    retry_after: Option<Instant>,
}
impl Poller {
    /// The worker, not a periodic receipt, selects the explicitly negotiated
    /// family. A passive exchange never acknowledges an enqueued command.
    pub(crate) fn exchange(
        &mut self,
        c: &mut ModernTlsConnection,
        q: world::Request,
    ) -> Result<(), String> {
        if c.supports_vip_world() {
            let r = c.vip_world(q).map_err(|e| e.to_string())?;
            self.receive_fields(
                r.echo(),
                r.status(),
                r.reason(),
                r.attachment().is_some(),
                Instant::now(),
            );
        } else {
            let r = c.battle_world(q).map_err(|e| e.to_string())?;
            self.receive(&r, Instant::now());
        }
        Ok(())
    }
    pub(crate) fn request(
        &self,
        state: &State,
        nav: &app::NavigationReply,
        directory: Option<&rooms::Reply>,
        now: Instant,
    ) -> Option<world::Request> {
        if !state.supported || nav.snapshot.phase != app::NavigationPhase::Lobby {
            return None;
        }
        let channel = nav.snapshot.channel_id?;
        let attached = state.snapshot.as_ref().and_then(|s| s.attachment());
        let mut q = world::Request::read(nav.context, channel, attached.map(|a| &a.world));
        if attached.is_none() {
            let directory = directory?;
            if directory.common.context != nav.context || directory.common.channel_id != channel {
                return None;
            }
            let joined = directory.joined.as_ref()?;
            let s = &joined.summary;
            if joined.owner == nav.context.key
                && s.satisfied
                && self.attempted != Some((nav.context, channel, s.room_id, s.revision))
                && self.retry_after.is_none_or(|after| now >= after)
                && directory.options.iter().any(|o| {
                    o.mode == s.settings.mode && o.map_id == s.settings.map_id && o.world_available
                })
            {
                q.action = world::Action::Prepare;
                q.room = s.room_id.try_into().ok()?;
                q.room_revision = s.revision;
            }
        }
        Some(q)
    }
    pub(crate) fn receive(&mut self, r: &world::Reply, now: Instant) {
        self.receive_fields(&r.echo, r.status, r.reason, r.attachment.is_some(), now);
    }
    fn receive_fields(
        &mut self,
        q: &world::Request,
        status: super::world_protocol::Status,
        reason: super::world_protocol::Reason,
        attached: bool,
        now: Instant,
    ) {
        if q.action == world::Action::Prepare {
            self.attempted = Some((q.context, q.channel, q.room as u64, q.room_revision));
            if status == super::world_protocol::Status::Busy
                && reason == super::world_protocol::Reason::Capacity
                && !attached
            {
                self.attempted = None;
                self.retry_after = Some(now + Duration::from_secs(2));
            }
        }
    }
}
