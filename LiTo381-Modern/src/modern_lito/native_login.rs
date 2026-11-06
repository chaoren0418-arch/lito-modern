//! Native LOGIN endpoint state and MS950 envelopes. No credentials enter snapshots.
use super::native_connection::{Controller, Event, Limits, Phase};
use anyhow::{bail, ensure, Context, Result};
use std::{
    net::{Ipv4Addr, SocketAddr},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[cfg(test)]
#[path = "native_rental_return_tests.rs"]
mod rental_return_tests;
#[cfg(test)]
#[path = "native_battle_withdrawal_tests.rs"]
mod battle_withdrawal_tests;
#[cfg(test)]
#[path = "native_return_server_negative_tests.rs"]
mod return_server_negative_tests;

pub fn decode_text(bytes: &[u8]) -> Result<String> {
    decode_message(bytes, false)
}
pub(crate) fn decode_message(bytes: &[u8], layout_controls: bool) -> Result<String> {
    use windows::Win32::Globalization::{MultiByteToWideChar, MB_ERR_INVALID_CHARS};
    ensure!(
        !bytes.is_empty() && bytes.len() <= 256,
        "native text length"
    );
    let mut wide = Zeroizing::new(vec![0u16; bytes.len()]);
    let count = unsafe { MultiByteToWideChar(950, MB_ERR_INVALID_CHARS, bytes, Some(&mut wide)) };
    ensure!(count > 0, "invalid MS950 text");
    let text = String::from_utf16(&wide[..count as usize])?;
    ensure!(
        !text
            .chars()
            .any(|c| c.is_control() && !(layout_controls && matches!(c, '\n' | '\r' | '\t'))),
        "control in native text"
    );
    Ok(text)
}
pub fn encode_text(text: &str, max: usize) -> Result<Zeroizing<Vec<u8>>> {
    use windows::{
        core::PCSTR,
        Win32::{
            Foundation::BOOL,
            Globalization::{WideCharToMultiByte, WC_NO_BEST_FIT_CHARS},
        },
    };
    ensure!(
        !text.is_empty() && text.len() <= 1024 && !text.chars().any(char::is_control),
        "native text shape"
    );
    let wide = Zeroizing::new(text.encode_utf16().collect::<Vec<_>>());
    let mut fallback = BOOL(0);
    let mut bytes = Zeroizing::new(vec![0; wide.len() * 2]);
    let count = unsafe {
        WideCharToMultiByte(
            950,
            WC_NO_BEST_FIT_CHARS,
            &wide,
            Some(&mut bytes),
            PCSTR::null(),
            Some(&mut fallback),
        )
    };
    ensure!(
        count > 0 && !fallback.as_bool(),
        "text cannot be represented exactly in MS950"
    );
    bytes.truncate(count as usize);
    ensure!(bytes.len() <= max, "native encoded text too long");
    ensure!(
        Zeroizing::new(decode_text(&bytes)?).as_str() == text,
        "MS950 roundtrip changed text"
    );
    Ok(bytes)
}
pub fn login_request(account: &str, password: &str, local: Ipv4Addr) -> Result<Zeroizing<Vec<u8>>> {
    ensure!(
        !account.trim().is_empty() && account.encode_utf16().count() <= 64,
        "native account length"
    );
    ensure!(
        !password.trim().is_empty() && password.encode_utf16().count() <= 128,
        "native password length"
    );
    let account = encode_text(account, 128)?;
    let password = encode_text(password, 256)?;
    let mut body = Zeroizing::new(Vec::with_capacity(7 + account.len() + password.len()));
    body.push(1);
    body.extend_from_slice(&account);
    body.push(0);
    body.extend_from_slice(&password);
    body.push(0);
    body.extend_from_slice(&local.octets());
    Ok(body)
}
pub fn source_request(name: &str) -> Result<Zeroizing<Vec<u8>>> {
    let name = encode_text(name, 63)?;
    let mut body = Zeroizing::new(vec![0xc8]);
    body.extend_from_slice(&name);
    body.push(0);
    Ok(body)
}
struct Reader<'a> {
    data: &'a [u8],
    at: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        ensure!(
            n <= self.data.len() - self.at,
            "truncated native login packet"
        );
        let at = self.at;
        self.at += n;
        Ok(&self.data[at..self.at])
    }
    fn c(&mut self) -> Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn h(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn d(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn text(&mut self, max: usize) -> Result<String> {
        let n = self.data[self.at..]
            .iter()
            .position(|b| *b == 0)
            .context("unterminated native text")?;
        ensure!(n > 0 && n <= max, "native text bound");
        let s = decode_text(self.take(n)?)?;
        self.take(1)?;
        Ok(s)
    }
    fn end(self) -> Result<()> {
        ensure!(
            self.at == self.data.len(),
            "trailing native login packet bytes"
        );
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Source {
    pub name: String,
    pub endpoint: SocketAddr,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct LoginCheck {
    pub result: u8,
    pub value: u32,
    pub server_name: String,
    pub flag: u8,
    pub native_word: u16,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Packet {
    Services(super::native_services::Directory),
    LoginOk,
    LoginCheck(LoginCheck),
    Sources(Vec<Source>),
    TimeLimit(u32),
    Redirect { index: u8, endpoint: SocketAddr },
    Other(u8),
}
pub fn parse(body: &[u8]) -> Result<Packet> {
    let mut r = Reader { data: body, at: 0 };
    let opcode = r.c()?;
    let packet = match opcode {
        0xfb=>return Ok(Packet::Services(super::native_services::parse(body)?)),
        0xe7 => Packet::LoginOk,
        0x00 => {
            let result = r.c()?;
            ensure!(
                [0, 5, 8, 0x1a, 0x16].contains(&result),
                "unknown native login result"
            );
            Packet::LoginCheck(LoginCheck {
                result,
                value: r.d()?,
                server_name: r.text(40)?,
                flag: r.c()?,
                native_word: r.h()?,
            })
        }
        0xe9 => Packet::TimeLimit(r.d()?),
        0x3b => {
            let count = r.c()?;
            ensure!((1..=127).contains(&count), "native source count");
            let mut sources = Vec::with_capacity(count as usize);
            for _ in 0..count {
                let name = r.text(63)?;
                let ip = Ipv4Addr::from(r.d()?.to_le_bytes());
                let port = r.h()?;
                ensure!(port != 0, "native source port");
                ensure!(
                    !sources.iter().any(|s: &Source| s.name == name),
                    "duplicate native source name"
                );
                sources.push(Source {
                    name,
                    endpoint: (ip, port).into(),
                });
            }
            Packet::Sources(sources)
        }
        0xf4 => {
            let index = r.c()?;
            ensure!(index < 10, "native redirect index");
            let ip = Ipv4Addr::from(r.d()?.to_le_bytes());
            let port = r.h()?;
            ensure!(port != 0, "native redirect port");
            Packet::Redirect {
                index,
                endpoint: (ip, port).into(),
            }
        }
        _ => return Ok(Packet::Other(opcode)),
    };
    r.end()?;
    Ok(packet)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LoginPhase {
    Disconnected,
    Connecting,
    ReadyForLogin,
    LoginPending,
    AwaitingSources,
    SourcesReady,
    RedirectPending,
    RedirectReady,
    HandoffClosing,
    TeleportQuitPending,
    GameConnecting,
    GameLoginPending,
    LoadingCharacters,
    CharactersReady,
    CharacterPending,
    CharacterSelected,
    ClassicEntryPending,
    ClassicCharacterSelected,
    RentalSetPending,
    RentalReady,
    RentalArmPending,
    RentalAwaitPermission,
    RentalRestartPending,
    ChannelPending,
    AwaitingChannelInfo,
    LoadingRooms,
    LobbyReady,
    ReturningCharacters,
    RoomJoining,
    RoomCreating,
    RoomEditing,
    KickPending,
    ObserverPending,
    ModePending,
    MapPending,
    RoomAwaitSelf,
    RoomReady,
    ReadyPending,
    ForcePending,
    RoomLeaving,
    BattlePending,
    WorldEntering,
    WorldReady,
    BattleResult,
    ResultReturning,
    Closing,
    Closed,
    Failed,
}
impl LoginPhase {
    pub fn active_world(self)->bool {matches!(self,Self::BattlePending|Self::WorldReady)}
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Snapshot {
    pub pickup_feedback: super::native_ground_pickup::Feedback,
    pub item_feedback: super::native_item_actions::Feedback,
    pub skill_feedback: super::native_skills::Feedback,
    pub attack_feedback: super::native_combat::Feedback,
    pub phase: LoginPhase,
    pub sources: Vec<Source>,
    pub selected_source: Option<String>,
    pub redirect: Option<SocketAddr>,
    pub login_result: Option<LoginCheck>,
    pub error: Option<String>,
    pub characters: Vec<super::native_game::Character>,
    pub notice: Option<super::native_game::Notice>,
    pub selected_character: Option<String>,
    pub character_result: Option<u8>,
    pub classic_entry_result: Option<super::native_game::ClassicEntryCheck>,
    pub services: Option<super::native_services::Directory>,
    pub rental_set: Option<u8>,
    pub rental_restart_acks: u64,
    pub battle_withdrawal_pending: bool,
    pub shop_points: Option<u32>,
    pub merchant: super::native_merchant::Session,
    pub game_connected: bool,
    pub channel: Option<super::native_lobby::Channel>,
    pub channel_result: Option<u8>,
    pub rooms: Vec<super::native_lobby::Room>,
    pub directory_generation: u64,
    pub joined_room: Option<super::native_room::Room>,
    pub world: super::native_world::World,
    pub environment: super::native_environment::State,
    pub inventory: super::native_inventory::Inventory,
    pub room_result: Option<u8>,
    pub observer: bool,
    pub room_kick_result: Option<bool>,
    pub last_room_kick: Option<String>,
    pub acknowledgement_opcodes: Vec<u8>,
}
struct TeleportRecovery {
    login_peer: SocketAddr,
    account: String,
    source: String,
    character: String,
}
struct RentalReturnReconnect {
    login_peer: SocketAddr,
    source: String,
    closing: bool,
}
pub struct LoginController {
    teleport: Option<super::native_teleport::Pending>,
    teleport_quit_tx: Option<u64>,
    login_peer: Option<SocketAddr>,
    rental_return_reconnect: Option<RentalReturnReconnect>,
    teleport_recovery: Option<TeleportRecovery>,
    recovery_character: Option<String>,
    portal_entry: Option<super::native_portal::EntryPending>,
    portal_feedback: super::native_portal::EntryFeedback,
    door_pending: Option<super::native_doors::Pending>,
    door_feedback: super::native_doors::Feedback,
    pickup_pending: Option<super::native_ground_pickup::Pending>,
    item_pending: Option<super::native_item_actions::Pending>,
    skill_pending: Option<super::native_skills::Pending>,
    skill_ready: Option<Instant>,
    world_tree_request: Option<(super::native_navigation::InputStamp,Instant)>,
    attack_pending: Option<super::native_combat::Pending>,
    transport: Controller,
    state: Snapshot,
    deadline: Option<Instant>,
    credentials: Option<(Zeroizing<String>, Zeroizing<String>)>,
    limits: Limits,
    game: bool,
    readiness_acked: bool,
    confirm_initial_source: bool,
    directory: super::native_game::Directory,
    requested_channel: Option<u8>,
    requested_room: Option<u32>,
    kick_pending: Option<(u32,String,bool)>,
    observer_return: Option<LoginPhase>,
    rental_set_pending: Option<u8>,
    rental_restart_quit_pending: bool,
    retired_departures: super::native_room::RetiredDepartures,
    pending_roster_forces: super::native_room::PendingRosterForces,
    early_roster: super::native_room::EarlyRoster,
    pending_rooms: Option<Vec<super::native_lobby::Room>>,
}
impl Default for LoginController {
    fn default() -> Self {
        Self {
            teleport: None,
            teleport_quit_tx: None,
            login_peer: None,
            rental_return_reconnect: None,
            teleport_recovery: None,
            recovery_character: None,
            portal_entry: None,
            portal_feedback: Default::default(),
            door_pending: None,
            door_feedback: Default::default(),
            pickup_pending: None,
            item_pending: None,
            attack_pending: None,
            skill_pending: None,
            skill_ready: None,
            world_tree_request: None,
            transport: Controller::default(),
            state: Snapshot {
                pickup_feedback: Default::default(),
                item_feedback: Default::default(),
                attack_feedback: Default::default(),
                skill_feedback: Default::default(),
                phase: LoginPhase::Disconnected,
                sources: vec![],
                selected_source: None,
                redirect: None,
                login_result: None,
                error: None,
                characters: vec![],
                notice: None,
                selected_character: None,
                character_result: None,
                classic_entry_result: None,
                services: None,
                rental_set: None,
                rental_restart_acks: 0,
                battle_withdrawal_pending: false,
                shop_points: None,
                merchant: Default::default(),
                game_connected: false,
                channel: None,
                channel_result: None,
                rooms: vec![],
                directory_generation: 0,
                joined_room: None,
                world: Default::default(),
                environment: Default::default(),
                inventory: Default::default(),
                room_result: None,
                observer: false,
                room_kick_result: None,
                last_room_kick: None,
                acknowledgement_opcodes: vec![],
            },
            deadline: None,
            credentials: None,
            limits: Limits::default(),
            game: false,
            readiness_acked: false,
            confirm_initial_source: false,
            directory: super::native_game::Directory::default(),
            requested_channel: None,
            requested_room: None,
            kick_pending: None,
            observer_return: None,
            rental_set_pending: None,
            rental_restart_quit_pending: false,
            retired_departures: Default::default(),
            pending_roster_forces: Default::default(),
            early_roster: Default::default(),
            pending_rooms: None,
        }
    }
}
impl LoginController {
    pub(crate) fn merchant_owner(&self)->Option<super::native_merchant::Owner>{
        if self.state.phase!=LoginPhase::BattlePending||!self.state.world.ready()||self.state.observer||!self.state.joined_room.as_ref().is_some_and(|r|r.mode==6)
            ||self.state.world.battle_result.is_some()||!self.state.world.hp.is_some_and(|v|v[0]>0)||!self.state.world.own_actor().is_some_and(|a|!a.dead){return None;}
        Some(super::native_merchant::Owner{connection:self.transport.snapshot().epoch,generation:self.state.world.generation,incarnation:self.state.world.own_actor()?.incarnation,control_interruptions:self.state.world.control_interruptions,inventory:self.state.inventory.owner.clone()?})
    }
    fn merchant_target(&self,npc:u32)->Result<super::native_merchant::Owner>{
        ensure!(self.portal_entry.is_none(),"等待地圖入口回覆");
        let owner=self.merchant_owner().context("租裝 NPC 互動目前不可用")?;ensure!(self.state.world.movement_blocks.is_empty(),"狀態限制 NPC 操作");
        let a=self.state.world.own_actor().context("own NPC actor")?;let target=self.state.world.actors.get(&npc).context("NPC 已離開場景")?;
        ensure!(npc!=a.id&&!target.dead&&a.x.abs_diff(target.x).max(a.y.abs_diff(target.y))<=5,"NPC 不在互動範圍");Ok(owner)
    }
    pub fn rental_npc_talk(&mut self,npc:u32)->Result<()>{
        let owner=self.merchant_target(npc)?;ensure!(!self.state.merchant.waiting,"請等待 NPC 回覆");
        self.transport.send(owner.connection,Zeroizing::new(super::native_merchant::talk(npc)?))?;
        self.teleport=None;
        self.state.merchant.begin(owner,npc,Instant::now())
    }
    pub fn rental_npc_action(&mut self,npc:u32,action:&str)->Result<()>{
        let owner=self.merchant_target(npc)?;ensure!(self.state.merchant.owner.as_ref()==Some(&owner)&&self.state.merchant.npc==Some(npc)&&!self.state.merchant.waiting,"請先開啟此 NPC 對話");
        let teleport=self.state.merchant.dialog.as_ref().and_then(|dialog|
            super::native_teleport::Pending::new(owner.clone(),self.state.selected_character.clone().unwrap_or_default(),&dialog.html,action));
        if teleport.is_some(){ensure!(self.credentials.is_some(),"missing teleport session credentials");}
        self.transport.send(owner.connection,Zeroizing::new(super::native_merchant::action(npc,action)?))?;
        self.teleport=teleport;
        self.state.merchant.begin(owner,npc,Instant::now())
    }
    pub fn rental_merchant_order(&mut self,revision:u64,index:usize,quantity:u32)->Result<()>{
        let npc=self.state.merchant.npc.context("no merchant target")?;let owner=self.merchant_target(npc)?;
        ensure!(self.state.merchant.owner.as_ref()==Some(&owner),"商店場景已變更");
        let body=self.state.merchant.order(revision,index,quantity,Instant::now())?;
        if let Some(super::native_merchant::Listing::Sell{rows,..})=&self.state.merchant.listing{
            let row=rows.get(index).context("sell row")?;ensure!(self.state.inventory.items.get(&row.id).is_some_and(|v|v.count>=quantity),"歸還數量超過持有數量");
        }
        self.transport.send(owner.connection,Zeroizing::new(body))?;self.state.merchant.submitted();Ok(())
    }
    pub fn rental_npc_quantity(&mut self,revision:u64,quantity:u32)->Result<()>{
        let npc=self.state.merchant.npc.context("no NPC quantity target")?;let owner=self.merchant_target(npc)?;
        ensure!(self.state.merchant.owner.as_ref()==Some(&owner),"NPC quantity character changed");
        let body=self.state.merchant.input_reply(revision,quantity,Instant::now())?;
        self.transport.send(owner.connection,Zeroizing::new(body))?;self.state.merchant.begin(owner,npc,Instant::now())
    }
    pub fn pickup(
        &mut self,
        choice: super::native_ground_pickup::Choice,
        quantity: u32,
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        ensure!(self.portal_entry.is_none(),"等待地圖入口回覆");
        ensure!(
            self.item_pending.is_none() && self.pickup_pending.is_none(),
            "請等待上一筆物品更新"
        );
        ensure!(
            choice.owner.connection == self.transport.snapshot().epoch
                && frame.input_ready(&self.state.world)
                && frame.movement_ready(&self.state.world),
            "請等待目前角色動作或畫面完成"
        );
        ensure!(
            frame.ground_items.iter().any(|a| a == &choice.item),
            "地面物品畫面已失效"
        );
        let body = super::native_ground_pickup::request(&self.state, &choice, quantity)?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.pickup_pending = Some(super::native_ground_pickup::Pending::new(
            &self.state,
            choice,
            Instant::now(),
        ));
        self.state.pickup_feedback = super::native_ground_pickup::Feedback::Pending;
        Ok(())
    }
    pub fn item_action(
        &mut self,
        choice: super::native_item_actions::Choice,
        action: super::native_item_actions::Action,
    ) -> Result<()> {
        ensure!(self.portal_entry.is_none(),"等待地圖入口回覆");
        ensure!(
            self.item_pending.is_none() && self.pickup_pending.is_none(),
            "請等待上一筆物品更新"
        );
        ensure!(
            choice.owner.connection == self.transport.snapshot().epoch,
            "物品連線已變更"
        );
        let body = super::native_item_actions::request(&self.state, &choice, action)?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.item_pending = Some(super::native_item_actions::Pending::new(
            &self.state,
            choice,
            Instant::now(),
        ));
        self.state.item_feedback = super::native_item_actions::Feedback::Pending;
        Ok(())
    }
    pub fn item_target_action(
        &mut self,
        cursor: super::native_item_actions::TargetChoice,
        target: u32,
        position: [u16; 2],
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        ensure!(self.portal_entry.is_none(),"等待地圖入口回覆");
        ensure!(self.item_pending.is_none() && self.pickup_pending.is_none(),
            "請等待上一筆物品更新");
        ensure!(cursor.choice().owner.connection == self.transport.snapshot().epoch,
            "物品連線已變更");
        ensure!(frame.input_ready(&self.state.world) && frame.movement_ready(&self.state.world),
            "等待目前場景畫面");
        ensure!(frame.navigation.standing(i32::from(position[0]), i32::from(position[1])),
            "目標位置不可使用");
        if target != 0 {
            ensure!(frame.visibility.iter().any(|(id, _)| *id == target), "目標不在目前畫面");
        }
        let body = super::native_item_actions::target_request(&self.state, &cursor, target, position)?;
        self.transport.send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.item_pending = Some(super::native_item_actions::Pending::new_target(
            &self.state, cursor.choice().clone(), Instant::now()));
        self.state.item_feedback = super::native_item_actions::Feedback::Pending;
        Ok(())
    }
    pub fn item_selection_action(
        &mut self,
        selection: super::native_item_actions::Selection,
        target: Option<super::native_item_actions::Choice>,
        text: &str,
    ) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待地圖入口回覆");
        ensure!(self.item_pending.is_none() && self.pickup_pending.is_none(), "請等待上一筆物品更新");
        ensure!(selection.source.owner.connection == self.transport.snapshot().epoch, "物品連線已變更");
        let body = if let Some(target) = target {
            selection.item_request(&self.state, &target)?
        } else {
            selection.text_request(&self.state, text)?
        };
        self.transport.send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.item_pending = Some(super::native_item_actions::Pending::new(
            &self.state, selection.source, Instant::now()));
        self.state.item_feedback = super::native_item_actions::Feedback::Pending;
        Ok(())
    }
    pub fn item_letter_action(&mut self, selection: super::native_item_actions::Selection,
        receiver: &str, subject: &str, content: &str) -> Result<()> {
        ensure!(self.portal_entry.is_none(), "等待地圖入口回覆");
        ensure!(self.item_pending.is_none() && self.pickup_pending.is_none(), "請等待上一筆物品更新");
        ensure!(selection.source.owner.connection == self.transport.snapshot().epoch, "物品連線已變更");
        let body = selection.letter_request(&self.state, receiver, subject, content)?;
        self.transport.send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.item_pending = Some(super::native_item_actions::Pending::new(
            &self.state, selection.source, Instant::now()));
        self.state.item_feedback = super::native_item_actions::Feedback::Pending;
        Ok(())
    }
    pub fn continue_attack(
        &mut self,
        held: &mut super::native_combat::HeldAttack,
        frame: &super::native_world_scene::Frame,
    ) -> Result<bool> {
        let Some(target) = held.ready(
            self.transport.snapshot().epoch,
            &self.state.world,
            self.state.attack_feedback,
            self.state.phase.active_world(),
            frame.movement_ready(&self.state.world),
        ) else {
            return Ok(false);
        };
        if let Err(error) = self.attack(target) {
            held.clear();
            return Err(error);
        }
        held.sent(&self.state.world);
        Ok(true)
    }
    pub fn cast(
        &mut self,
        skill: u16,
        target: Option<u32>,
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        self.cast_request(skill, target, None, None, frame)
    }
    pub fn cast_ground(
        &mut self,
        skill: u16,
        position: [u16; 2],
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        ensure!(
            frame
                .navigation
                .contains(i32::from(position[0]), i32::from(position[1])),
            "地面選點不在已載入地圖"
        );
        self.cast_request(skill, None, Some(position), None, frame)
    }
    pub fn cast_item(&mut self, skill: u16, item: super::native_item_actions::Choice,
        frame: &super::native_world_scene::Frame) -> Result<()> {
        ensure!(item.owner.connection == self.transport.snapshot().epoch, "物品連線已變更");
        self.cast_request(skill, None, None, Some(&item), frame)
    }
    fn cast_request(
        &mut self,
        skill: u16,
        target: Option<u32>,
        ground: Option<[u16; 2]>,
        item: Option<&super::native_item_actions::Choice>,
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        ensure!(self.world_tree_request.is_none(),"等待世界樹傳送回覆");
        ensure!(!self.state.observer,"observer cannot cast");
        ensure!(
            self.state.phase.active_world()
                && self.attack_pending.is_none()
                && self.skill_pending.is_none() && self.door_pending.is_none() && self.portal_entry.is_none(),
            "等待目前動作回覆"
        );
        let now = Instant::now();
        ensure!(
            self.skill_ready.is_none_or(|deadline| now >= deadline)
                && frame.movement_ready(&self.state.world),
            "等待施法動作完成"
        );
        let body = if let Some(item) = item {
            super::native_skills::item_request(&self.state, skill, item)?
        } else {
            match ground {
                Some(position) => {
                    super::native_skills::ground_request(&self.state.world, skill, position)?
                }
                None => super::native_skills::request(&self.state.world, skill, target)?,
            }
        };
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.world_tree_request = (skill==131).then(||(super::native_navigation::InputStamp::capture(self.transport.snapshot().epoch,&self.state.world),now));
        self.skill_pending = Some(super::native_skills::Pending {
            since: now,
            casts: self.state.world.own_cast_packets,
            messages: self.state.world.message_count,
            generation: self.state.world.generation,
            incarnation: self.state.world.own_actor().unwrap().incarnation,
        });
        // Formal reuse, recovered client cast clock, body completion and
        // pending acknowledgement are separate admission gates.
        self.skill_ready = Some(now + Duration::from_millis(super::native_skills::reuse_millis(skill).max(super::native_skills::client_clock_millis(skill))));
        self.state.skill_feedback = super::native_skills::Feedback::Pending;
        Ok(())
    }
    pub fn attack(&mut self, target: u32) -> Result<()> {
        ensure!(self.world_tree_request.is_none(),"等待世界樹傳送回覆");
        ensure!(!self.state.observer,"observer cannot attack");
        ensure!(
            self.state.phase.active_world()
                && self.attack_pending.is_none()
                && self.skill_pending.is_none() && self.door_pending.is_none() && self.portal_entry.is_none(),
            "attack unavailable or awaiting server"
        );
        let body = self.state.world.attack_request(target)?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.attack_pending = Some(super::native_combat::Pending {
            since: Instant::now(),
            count: self.state.world.own_attack_packets,
            target,
            generation: self.state.world.generation,
        });
        self.state.attack_feedback = super::native_combat::Feedback::Pending;
        Ok(())
    }
    pub fn confirm_death(&mut self, frame: &super::native_world_scene::Frame) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::BattlePending && frame.death_ready(&self.state.world),
            "death animation/authority not ready"
        );
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0xe0]))?;
        self.state.world.death_confirmation_pending = false;
        self.state.world.death_confirmations_sent += 1;
        self.state.world.last_death_confirmed_action =
            self.state.world.own_actor().map(|a| a.action_sequence);
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn request_battle_withdrawal(&mut self) -> Result<()> {
        ensure!(self.state.phase==LoginPhase::BattlePending
            && !self.state.battle_withdrawal_pending && !self.state.observer
            && self.state.rental_set.is_none()
            && self.state.joined_room.as_ref().is_some_and(|r|r.mode!=6)
            && self.state.world.ready(),"目前無法申請退出戰鬥");
        self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0xd9]))?;
        // The authority either settles the battle or refuses during combat.
        // Keep the live scene until that response, and never send a second D9.
        self.state.battle_withdrawal_pending=true;
        self.deadline=Some(Instant::now()+Duration::from_secs(8));
        Ok(())
    }
    pub fn return_from_battle(&mut self) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::BattleResult,
            "battle result unavailable"
        );
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0xd9]))?;
        self.state.phase = LoginPhase::ResultReturning;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn move_checked(
        &mut self,
        heading: u8,
        navigation: &super::native_navigation::Navigation,
    ) -> Result<()> {
        ensure!(
            self.state.world.map_id.map(u32::from) == Some(navigation.map_id),
            "movement map mismatch"
        );
        let own = self.state.world.own_actor().context("local actor absent")?;
        ensure!(
            navigation.can_step(i32::from(own.x), i32::from(own.y), heading),
            "map edge blocks movement"
        );
        self.move_step(heading)
    }
    pub fn move_step(&mut self, heading: u8) -> Result<()> {
        ensure!(self.world_tree_request.is_none(),"等待世界樹傳送回覆");
        ensure!(!self.state.observer,"observer cannot move a player");
        ensure!(
            self.attack_pending.is_none() && self.skill_pending.is_none() && self.door_pending.is_none() && self.portal_entry.is_none(),
            "awaiting action response"
        );
        ensure!(
            self.state.phase.active_world(),
            "movement outside world"
        );
        let body = self.state.world.move_request(heading)?;
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(body))?;
        self.state.world.predict_step(heading)
    }
    pub fn pan_observer(&mut self, heading:u8)->Result<bool> {
        ensure!(self.state.observer&&matches!(self.state.phase,LoginPhase::BattlePending|LoginPhase::BattleResult),"observer view inactive");
        self.state.world.pan_observer(heading)
    }
    /// Disjoint presentation inputs; gameplay authority remains read-only.
    pub(crate) fn presentation_inputs(&mut self)->(&super::native_world::World,&mut super::native_environment::State) {
        (&self.state.world,&mut self.state.environment)
    }
    pub fn snapshot(&self) -> &Snapshot {
        &self.state
    }
    pub fn portal_entry_feedback(&self)->super::native_portal::EntryFeedback {self.portal_feedback}
    pub fn enter_map_portal(&mut self,choice:super::native_portal::EntryChoice,frame:&super::native_world_scene::Frame)->Result<()> {
        ensure!(self.state.phase==LoginPhase::BattlePending&&!self.state.observer&&self.portal_entry.is_none()
            &&self.attack_pending.is_none()&&self.skill_pending.is_none()&&self.item_pending.is_none()
            &&self.pickup_pending.is_none()&&self.door_pending.is_none()&&!self.state.merchant.waiting,"等待目前操作回覆");
        ensure!(frame.input_ready(&self.state.world)&&frame.movement_ready(&self.state.world),"等待目前世界畫面");
        let epoch=self.transport.snapshot().epoch;
        let body=choice.request(epoch,&self.state.world,&frame.navigation)?;
        self.transport.send(epoch,Zeroizing::new(body))?;
        self.portal_entry=Some(super::native_portal::EntryPending{choice,since:Instant::now()});
        self.portal_feedback=super::native_portal::EntryFeedback::Pending;
        Ok(())
    }
    pub fn door_feedback(&self)->super::native_doors::Feedback {self.door_feedback}
    pub fn toggle_door(&mut self,choice:super::native_doors::Choice,frame:&super::native_world_scene::Frame)->Result<()> {
        ensure!(self.state.phase==LoginPhase::BattlePending && self.door_pending.is_none() && self.portal_entry.is_none()
            &&self.attack_pending.is_none()&&self.skill_pending.is_none(),"等待目前操作回覆");
        ensure!(frame.movement_ready(&self.state.world)&&frame.input_ready(&self.state.world),"等待目前世界畫面");
        let epoch=self.transport.snapshot().epoch;
        let body=choice.request(epoch,&self.state.world)?;
        self.transport.send(epoch,Zeroizing::new(body))?;
        self.door_pending=Some(super::native_doors::Pending{choice,since:Instant::now()});
        self.door_feedback=super::native_doors::Feedback::Pending;
        Ok(())
    }
    pub fn transport(&self) -> &Controller {
        &self.transport
    }
    pub fn is_quiescent(&self) -> bool {
        self.transport.is_quiescent()
    }
    pub fn start(&mut self, peer: SocketAddr, limits: Limits) -> Result<()> {
        self.transport.start(peer, limits)?;
        if self.teleport_recovery.as_ref().is_some_and(|r|r.login_peer!=peer){self.teleport_recovery=None;}
        self.rental_return_reconnect=None;
        self.login_peer=Some(peer);self.recovery_character=None;
        self.teleport=None;
        self.teleport_quit_tx=None;
        self.portal_entry=None;self.portal_feedback=super::native_portal::EntryFeedback::Idle;self.door_pending=None;self.door_feedback=Default::default();
        self.pickup_pending = None;
        self.state.pickup_feedback = super::native_ground_pickup::Feedback::Interrupted;
        self.item_pending = None;
        let generation = self.state.world.generation;
        self.state = Self::default().state;
        self.state.world.generation = generation;
        self.state.phase = LoginPhase::Connecting;
        self.credentials = None;
        self.limits = limits;
        self.game = false;
        self.requested_channel = None;
        self.requested_room = None;
        self.kick_pending = None;
        self.observer_return = None;
        self.rental_set_pending = None;
        self.rental_restart_quit_pending = false;
        self.pending_rooms = None;
        self.retired_departures.clear();
        self.pending_roster_forces.clear();self.early_roster.clear();
        self.attack_pending = None;
        self.skill_pending = None;
        self.world_tree_request=None;
        self.skill_ready = None;
        self.readiness_acked = false;
        self.confirm_initial_source = false;
        self.directory = super::native_game::Directory::default();
        self.deadline = None;
        Ok(())
    }
    pub fn login(&mut self, account: &str, password: &str) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::ReadyForLogin,
            "native login is not ready"
        );
        let info = self
            .transport
            .snapshot()
            .negotiated
            .as_ref()
            .context("missing native transport")?;
        let local = match info.local.ip() {
            std::net::IpAddr::V4(ip) => ip,
            _ => Ipv4Addr::UNSPECIFIED,
        };
        self.transport.send(
            self.transport.snapshot().epoch,
            login_request(account, password, local)?,
        )?;
        self.credentials = Some((
            Zeroizing::new(account.to_owned()),
            Zeroizing::new(password.to_owned()),
        ));
        self.state.login_result = None;
        self.state.phase = LoginPhase::LoginPending;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn select_source(&mut self, index: usize) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::SourcesReady,
            "native source selection is not ready"
        );
        let source = self
            .state
            .sources
            .get(index)
            .context("native source index")?;
        self.transport.send(
            self.transport.snapshot().epoch,
            source_request(&source.name)?,
        )?;
        self.state.selected_source = Some(source.name.clone());
        if let Some(recovery)=self.teleport_recovery.take(){
            if recovery.source==source.name && self.credentials.as_ref().is_some_and(|(account,_)|**account==recovery.account) {
                self.recovery_character=Some(recovery.character);
            }
        }
        self.state.phase = LoginPhase::RedirectPending;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn begin_game(&mut self) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::RedirectReady,
            "GAME handoff is not ready"
        );
        let target = self.state.redirect.context("missing GAME redirect")?;
        let source = self
            .state
            .sources
            .iter()
            .find(|s| Some(&s.name) == self.state.selected_source.as_ref())
            .context("missing selected source")?;
        ensure!(
            source.endpoint == target,
            "GAME redirect differs from selected advertised endpoint"
        );
        ensure!(self.credentials.is_some(), "missing handoff credentials");
        // LOGIN source handoffs expire while a user waits at the redirect.
        // Rebind an ordinary initial GAME through its public C8 before
        // publishing any character directory. Staged rental continuations
        // retain the formal bootstrap auto-selection and must not replay C8.
        self.confirm_initial_source = self.teleport.is_none() && self.recovery_character.is_none();
        self.transport.cancel();
        self.state.phase = LoginPhase::HandoffClosing;
        self.deadline = None;
        Ok(())
    }
    pub fn acknowledge_notice(&mut self) -> Result<()> {
        ensure!(
            self.game
                && matches!(
                    self.state.phase,
                    LoginPhase::GameLoginPending
                        | LoginPhase::LoadingCharacters
                        | LoginPhase::CharactersReady
                ),
            "notice outside GAME bootstrap"
        );
        let opcode = self
            .state
            .notice
            .as_ref()
            .context("no pending notice")?
            .acknowledgement();
        if let Some(opcode) = opcode {
            ensure!(
                self.state.acknowledgement_opcodes.len() < 64,
                "too many GAME notices"
            );
            self.transport.send(
                self.transport.snapshot().epoch,
                Zeroizing::new(vec![opcode]),
            )?;
            self.state.acknowledgement_opcodes.push(opcode);
        }
        self.state.notice = None;
        if self.state.phase != LoginPhase::CharactersReady {
            self.deadline = Some(Instant::now() + Duration::from_secs(8));
        }
        Ok(())
    }
    pub fn select_character(&mut self, index: usize) -> Result<()> {
        self.select_character_kind(index,false)
    }
    /// C05 uses the exact received directory identity. Acceptance alone does not
    /// establish a world scene or authorize movement.
    pub fn request_classic_entry(&mut self,index:usize)->Result<()> {
        self.select_character_kind(index,true)
    }
    fn select_character_kind(&mut self,index:usize,classic:bool)->Result<()> {
        ensure!(
            self.state.phase == LoginPhase::CharactersReady && self.state.notice.is_none(),
            "character selection not ready"
        );
        self.transport.send(
            self.transport.snapshot().epoch,
            if classic {super::native_game::classic_entry_request(&self.state.characters.get(index).context("classic character index")?.name)?}else{self.directory.select(index)?},
        )?;
        self.state.selected_character = Some(
            self.state
                .characters
                .get(index)
                .context("character index")?
                .name
                .clone(),
        );
        self.state.character_result = None;
        self.state.classic_entry_result = None;
        self.state.rental_set=None;self.rental_set_pending=None;self.state.shop_points=None;
        self.state.world.clear_preserving_generation();self.state.environment.retire_scene();
        self.state.inventory.begin(
            self.transport.snapshot().epoch,
            self.state.selected_character.as_deref().unwrap(),
        )?;
        self.skill_pending = None;
        self.world_tree_request=None;
        self.skill_ready = None;
        self.state.skill_feedback = Default::default();
        self.state.phase = if classic {LoginPhase::ClassicEntryPending}else{LoginPhase::CharacterPending};
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn join_room(&mut self, index: usize, password: &str) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::LobbyReady,
            "room join requires live lobby"
        );
        let id = self
            .state
            .rooms
            .get(index)
            .context("unreceived room index")?
            .id;
        self.transport.send(
            self.transport.snapshot().epoch,
            super::native_room::join(id, password)?,
        )?;
        self.early_roster.clear();
        self.requested_room = Some(id);
        self.state.room_result = None;
        self.state.joined_room = None;
        self.state.phase = LoginPhase::RoomJoining;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn create_room(&mut self,settings:&super::native_room::Settings<'_>)->Result<()> {
        ensure!(self.state.phase==LoginPhase::LobbyReady,"room creation requires lobby");
        ensure!((2..=100).contains(&settings.capacity),"native room capacity outside 2..100");
        self.transport.send(self.transport.snapshot().epoch,settings.request(0xcc)?)?;
        self.early_roster.clear();self.requested_room=None;self.state.room_result=None;self.state.joined_room=None;
        self.state.phase=LoginPhase::RoomCreating;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn select_rental_set(&mut self,index:u8)->Result<()> {
        ensure!(matches!(self.state.phase,LoginPhase::CharacterSelected|LoginPhase::RentalReady),"rental selection requires selected character");
        ensure!(self.state.services.as_ref().is_some_and(|d|!d.rooms.is_empty()),"no advertised rental service");
        ensure!(index<=3,"rental selection wire index");
        self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0xca,index]))?;
        self.rental_set_pending=Some(index);self.state.rental_set=None;self.state.channel_result=None;
        self.state.phase=LoginPhase::RentalSetPending;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn return_from_rental(&mut self)->Result<()> {
        ensure!(self.merchant_owner().is_some(),"目前不是可返回的租裝村角色");
        ensure!(!self.state.merchant.waiting,"請等待 NPC 回覆後返回");
        self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0x10]))?;
        self.state.merchant.clear();self.state.rental_restart_acks=0;self.rental_restart_quit_pending=false;
        self.state.phase=LoginPhase::RentalRestartPending;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn join_rental_room(&mut self,index:usize)->Result<()> {
        ensure!(self.state.phase==LoginPhase::RentalReady&&self.state.rental_set.is_some(),"rental loadout not selected");
        let room=self.state.services.as_ref().and_then(|d|d.rooms.get(index)).context("rental directory index")?;
        ensure!(Some(room.endpoint)==self.state.redirect,"rental service requires a separate endpoint handoff");
        self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0xd9]))?;
        self.requested_room=Some(room.id);self.state.room_result=None;self.state.joined_room=None;
        self.state.phase=LoginPhase::RentalArmPending;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn edit_room(&mut self,settings:&super::native_room::Settings<'_>)->Result<()> {
        ensure!(self.state.phase==LoginPhase::RoomReady,"room editing requires room");
        let room=self.state.joined_room.as_ref().context("room editing missing room")?;
        ensure!(self.state.selected_character.as_deref()==Some(room.owner.as_str()),"only owner may edit room");
        ensure!((2..=100).contains(&settings.capacity),"native room capacity outside 2..100");
        self.transport.send(self.transport.snapshot().epoch,settings.request(0xdd)?)?;
        self.state.phase=LoginPhase::RoomEditing;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn kick_member(&mut self,name:&str)->Result<()> {
        ensure!(self.state.phase==LoginPhase::RoomReady,"kick requires room");
        let room=self.state.joined_room.as_ref().context("kick missing room")?;
        ensure!(self.state.selected_character.as_deref()==Some(room.owner.as_str()),"only owner may kick");
        ensure!(name!=room.owner && room.members.iter().any(|m|m.name==name),"kick target is not a current peer");
        self.transport.send(self.transport.snapshot().epoch,super::native_room::kick_request(name)?)?;
        self.kick_pending=Some((room.id,name.to_owned(),false));self.state.room_kick_result=None;
        self.state.phase=LoginPhase::KickPending;self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn set_observer(&mut self,enabled:bool)->Result<()> {
        ensure!(matches!(self.state.phase,LoginPhase::CharacterSelected|LoginPhase::LobbyReady|LoginPhase::RoomReady),"observer selection unavailable");
        self.transport.send(self.transport.snapshot().epoch,super::native_room::observer_request(enabled))?;
        self.observer_return=Some(self.state.phase);self.state.phase=LoginPhase::ObserverPending;
        self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn set_force(&mut self, force: u8) -> Result<()> {
        ensure!(self.state.phase == LoginPhase::RoomReady, "force selection requires room");
        let room = self.state.joined_room.as_ref().context("force selection missing room")?;
        let own = room.members.iter().find(|m| Some(m.name.as_str()) == self.state.selected_character.as_deref())
            .context("force selection missing participant")?;
        ensure!(own.state == 0, "cancel readiness before changing force");
        self.transport.send(self.transport.snapshot().epoch, super::native_room::force_request(force)?)?;
        self.state.phase = LoginPhase::ForcePending;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn step_room_mode(&mut self, direction:u8) -> Result<()> {
        ensure!(self.state.phase==LoginPhase::RoomReady,"mode selection requires room");
        let room=self.state.joined_room.as_ref().context("mode selection missing room")?;
        ensure!(self.state.selected_character.as_deref()==Some(room.owner.as_str()),"only room owner may change mode");
        ensure!(room.members.iter().all(|m|m.state==0),"cancel readiness before changing mode");
        self.transport.send(self.transport.snapshot().epoch,super::native_room::mode_step(direction)?)?;
        self.state.phase=LoginPhase::ModePending;
        self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn select_room_map(&mut self, map:u16) -> Result<()> {
        ensure!(self.state.phase==LoginPhase::RoomReady,"map selection requires room");
        let room=self.state.joined_room.as_ref().context("map selection missing room")?;
        ensure!(self.state.selected_character.as_deref()==Some(room.owner.as_str()),"only room owner may change map");
        ensure!(room.members.iter().all(|m|m.state==0),"cancel readiness before changing map");
        self.transport.send(self.transport.snapshot().epoch,super::native_room::map_selection(room.mode,map)?)?;
        self.state.phase=LoginPhase::MapPending;
        self.deadline=Some(Instant::now()+Duration::from_secs(8));Ok(())
    }
    pub fn set_ready(&mut self, ready: bool) -> Result<()> {
        ensure!(!ready || !self.state.observer,"observer cannot become a battle participant");
        ensure!(
            self.state.phase == LoginPhase::RoomReady,
            "room readiness unavailable"
        );
        self.transport.send(
            self.transport.snapshot().epoch,
            Zeroizing::new(vec![0xd2, u8::from(ready)]),
        )?;
        self.state.phase = LoginPhase::ReadyPending;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn leave_room(&mut self) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::RoomReady,
            "room departure unavailable"
        );
        self.transport
            .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0xd0]))?;
        self.state.phase = LoginPhase::RoomLeaving;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    fn accept_room_member(&mut self,room:u32,member:super::native_room::Member)->Result<()> {
        let active=self.state.joined_room.as_mut().context("participant before joined room")?;
        ensure!(active.id==room,"participant for a different room");
        let own=self.state.selected_character.as_deref()==Some(member.name.as_str());
        self.pending_roster_forces.member_received(room,&member.name)?;
        super::native_room::upsert(active,member)?;
        if own&&self.state.phase==LoginPhase::RoomAwaitSelf{self.state.phase=LoginPhase::RoomReady;self.deadline=None;}
        Ok(())
    }
    fn accept_room(&mut self, body: &[u8]) -> Result<bool> {
        use super::native_room::{self, Packet as R};
        match native_room::parse(body)? {
            R::Observer(enabled) => {
                self.state.observer=enabled;
                self.state.world.set_observer_mode(enabled);
                if let Some(phase)=self.observer_return.take(){self.state.phase=phase;self.deadline=None;}
            }
            R::KickStatus(accepted) => {
                ensure!(self.state.phase==LoginPhase::KickPending,"unsolicited kick result");
                self.state.room_kick_result=Some(accepted);
                if accepted {self.kick_pending.as_mut().context("kick result without request")?.2=true;}
                else {self.kick_pending=None;self.state.phase=LoginPhase::RoomReady;self.deadline=None;}
            }
            R::Kicked(name) => {
                ensure!(self.state.joined_room.as_ref().is_some_and(|r|r.members.iter().any(|m|m.name==name)),"kick notice for unknown member");
                self.state.last_room_kick=Some(name);
            }
            R::Info { room, update } => {
                let map_complete=matches!(update,native_room::Info::Map(_));
                let complete=map_complete||matches!(update,native_room::Info::Summary{..});
                let configuration=matches!(update,native_room::Info::Configuration{..});
                native_room::apply_info(self.state.joined_room.as_mut().context("game info without room")?,room,update)?;
                if (self.state.phase==LoginPhase::ModePending && complete) || (self.state.phase==LoginPhase::MapPending && map_complete) {
                    self.state.phase=LoginPhase::RoomReady;self.deadline=None;
                }
                if self.state.phase==LoginPhase::RoomEditing && configuration {
                    // A refusal also returns the unchanged authoritative snapshot.
                    self.state.phase=LoginPhase::RoomReady;self.deadline=None;
                }
            }
            R::Joined(room) => {
                ensure!(
                    (self.state.phase == LoginPhase::RoomJoining && self.requested_room == Some(room.id))
                        || (self.state.phase==LoginPhase::RoomCreating && self.requested_room.is_none()),
                    "unsolicited room acceptance"
                );
                let early=self.early_roster.take(self.transport.snapshot().epoch,room.id)?;
                self.state.room_result = Some(0);
                self.requested_room=Some(room.id);
                self.retired_departures.clear();
                self.pending_roster_forces.clear();self.early_roster.clear();
                self.state.joined_room = Some(room);
                self.state.phase = LoginPhase::RoomAwaitSelf;
                for member in early {self.accept_room_member(self.requested_room.unwrap(),member)?;}
            }
            R::Rejected(result) => {
                ensure!(
                    matches!(self.state.phase,LoginPhase::RoomJoining|LoginPhase::RoomCreating),
                    "unsolicited room rejection"
                );
                self.state.room_result = Some(result);
                self.early_roster.clear();
                self.requested_room = None;
                self.state.phase = if self.state.rental_set.is_some()&&self.state.channel.is_none(){LoginPhase::RentalReady}else{LoginPhase::LobbyReady};
                self.deadline = None;
            }
            R::Member { room, member } => {
                if self.state.joined_room.is_none(){
                    ensure!(matches!(self.state.phase,LoginPhase::RoomJoining|LoginPhase::RoomCreating),"participant outside pending join");
                    self.early_roster.push(self.transport.snapshot().epoch,self.requested_room,room,member)?;
                }else{self.accept_room_member(room,member)?;}
            }

            R::Ready { name, state } => {
                let active = self
                    .state
                    .joined_room
                    .as_mut()
                    .context("readiness without room")?;
                let member = active
                    .members
                    .iter_mut()
                    .find(|m| m.name == name)
                    .context("readiness for unknown member")?;
                member.state = state;
                if self.state.selected_character.as_deref() == Some(name.as_str())
                    && self.state.phase == LoginPhase::ReadyPending
                {
                    self.state.phase = LoginPhase::RoomReady;
                    self.deadline = None;
                }
            }
            R::Force { name, force } => {
                let room = self.state.joined_room.as_mut().context("force update without room")?;
                let applied=self.pending_roster_forces.apply(room,&name,force,
                    self.state.phase==LoginPhase::RoomAwaitSelf,Instant::now())?;
                if !applied {return Ok(true);}
                if self.state.selected_character.as_deref() == Some(name.as_str())
                    && self.state.phase == LoginPhase::ForcePending {
                    self.state.phase = LoginPhase::RoomReady;
                    self.deadline = None;
                }
            }
            R::Departed { room, name } => {
                let own = self.state.selected_character.as_deref().context("departure without character")?;
                if self.state.joined_room.is_none() && matches!(self.state.phase,LoginPhase::RoomJoining|LoginPhase::RoomCreating) {
                    ensure!(name!=own,"self departure before room acceptance");
                    // A known old-room peer is consumed once before it can
                    // touch the next join's room binding or early roster.
                    if self.retired_departures.consume(room,&name) { return Ok(true); }
                    self.early_roster.depart(self.transport.snapshot().epoch,self.requested_room,room,&name)?;
                    return Ok(true);
                }
                if name == own {
                    ensure!(!self.rental_restart_quit_pending,
                        "rental departure preceded quit acknowledgement");
                    ensure!(
                        matches!(
                            self.state.phase,
                            LoginPhase::RoomLeaving
                                | LoginPhase::ReturningCharacters
                                | LoginPhase::RoomReady
                                | LoginPhase::ReadyPending
                                | LoginPhase::ForcePending
                                | LoginPhase::ModePending
                                | LoginPhase::MapPending
                                | LoginPhase::RoomEditing
                                | LoginPhase::KickPending
                                | LoginPhase::ObserverPending
                        ),
                        "self departure phase"
                    );
                }
                if native_room::depart(&mut self.state.joined_room, &mut self.retired_departures,
                    room, &name, own)? == native_room::Departure::SelfLeft {
                    self.requested_room = None;
                    self.pending_roster_forces.clear();self.early_roster.clear();
                    self.kick_pending=None;self.observer_return=None;
                    self.state.world.clear_preserving_generation();self.state.environment.retire_scene();
                    if self.state.phase==LoginPhase::ReturningCharacters {
                        return Ok(true);
                    }
                    self.state.phase = LoginPhase::LobbyReady;
                    self.refresh_rooms()?;
                } else if self.kick_pending.as_ref().is_some_and(|(id,target,accepted)|*id==room && target==&name && *accepted) {
                    self.kick_pending=None;self.state.phase=LoginPhase::RoomReady;self.deadline=None;
                }
            }
            R::Deleted(id) => {
                ensure!(
                    self.state.joined_room.as_ref().is_none_or(|r| r.id != id),
                    "active room deleted without departure"
                );
                self.state.rooms.retain(|r| r.id != id);
                if let Some(rows) = &mut self.pending_rooms {
                    rows.retain(|r| r.id != id);
                }
            }
            R::Owner { room, name } => {
                if let Some(active) = &mut self.state.joined_room {
                    ensure!(active.id == room, "owner for different room");
                    active.owner = name;
                }
            }
            R::Countdown(value) => {
                if let Some(active) = &mut self.state.joined_room {
                    active.countdown = Some(value);
                }
            }
            R::BattleStart => {
                bail!("battle start without preceding world assignment");
            }
            R::Other => return Ok(false),
        }
        Ok(true)
    }
    pub fn enter_channel(&mut self, id: u8) -> Result<()> {
        ensure!(
            matches!(
                self.state.phase,
                LoginPhase::CharacterSelected | LoginPhase::LobbyReady
            ),
            "channel selection is not ready"
        );
        ensure!(self.state.phase != LoginPhase::CharacterSelected
            || self.state.services.as_ref().is_none_or(|directory| directory.rooms.is_empty()),
            "此入口提供租裝服務，請確認配裝後進入租裝村");
        let mut frames = vec![];
        if self.state.phase == LoginPhase::LobbyReady {
            frames.push(Zeroizing::new(vec![0xcb]));
        }
        frames.push(Zeroizing::new(super::native_lobby::enter(id)?));
        self.transport
            .send_batch(self.transport.snapshot().epoch, frames)?;
        self.requested_channel = Some(id);
        self.state.channel_result = None;
        self.state.channel = None;
        self.state.rooms.clear();
        self.pending_rooms = None;
        self.state.phase = LoginPhase::ChannelPending;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn refresh_rooms(&mut self) -> Result<()> {
        ensure!(
            self.state.phase == LoginPhase::LobbyReady,
            "room refresh is not ready"
        );
        let id = self
            .state
            .channel
            .as_ref()
            .context("missing active channel")?
            .id;
        // C_CA on the current channel is an idempotent S_CA refresh. Its reply
        // follows all C_CE rows on this socket and closes this refresh batch.
        self.transport.send_batch(
            self.transport.snapshot().epoch,
            vec![
                Zeroizing::new(vec![0xce]),
                Zeroizing::new(super::native_lobby::enter(id)?),
            ],
        )?;
        self.pending_rooms = Some(vec![]);
        self.state.phase = LoginPhase::LoadingRooms;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    pub fn return_characters(&mut self) -> Result<()> {
        ensure!(
            matches!(
                self.state.phase,
                LoginPhase::LobbyReady | LoginPhase::CharacterSelected | LoginPhase::ClassicCharacterSelected
            ),
            "return to characters is not ready"
        );
        let mut frames = vec![];
        if self.state.phase == LoginPhase::LobbyReady {
            frames.push(Zeroizing::new(vec![0xcb]));
        }
        frames.push(Zeroizing::new(vec![0xca, 100]));
        self.transport
            .send_batch(self.transport.snapshot().epoch, frames)?;
        self.state.phase = LoginPhase::ReturningCharacters;
        self.deadline = Some(Instant::now() + Duration::from_secs(8));
        Ok(())
    }
    fn accept_lobby(&mut self, body: &[u8]) -> Result<bool> {
        use super::native_lobby::{self, Packet as L};
        match native_lobby::parse(body)? {
            L::Enter {
                result, channel, ..
            } => {
                if self.state.phase==LoginPhase::RentalSetPending {
                    ensure!(self.rental_set_pending==Some(channel),"rental selection identity changed");
                    self.rental_set_pending=None;self.state.channel_result=Some(result);
                    self.state.rental_set=if result==0{Some(channel)}else{None};
                    self.state.phase=if result==0{LoginPhase::RentalReady}else{LoginPhase::CharacterSelected};
                    self.deadline=None;return Ok(true);
                }
                ensure!(
                    self.state.phase == LoginPhase::ChannelPending
                        && self.requested_channel == Some(channel),
                    "unsolicited channel result"
                );
                self.state.channel_result = Some(result);
                if result == 0 {
                    self.state.phase = LoginPhase::AwaitingChannelInfo;
                } else {
                    self.state.phase = LoginPhase::CharacterSelected;
                    self.deadline = None;
                }
            }
            L::Channel(channel) => {
                ensure!(
                    self.requested_channel == Some(channel.id),
                    "channel info changed identity"
                );
                match self.state.phase {
                    LoginPhase::AwaitingChannelInfo => {
                        self.state.channel = Some(channel);
                        self.state.phase = LoginPhase::LobbyReady;
                        self.refresh_rooms()?;
                    }
                    LoginPhase::LoadingRooms => {
                        self.state.channel = Some(channel);
                        self.state.rooms =
                            self.pending_rooms.take().context("missing room refresh")?;
                        self.state.phase = LoginPhase::LobbyReady;
                        self.deadline = None;
                    }
                    LoginPhase::LobbyReady => self.state.channel = Some(channel),
                    // An old population notification does not revive a retired UI.
                    LoginPhase::ReturningCharacters => {}
                    _ => anyhow::bail!("unsolicited channel info"),
                }
            }
            L::Room(room) => match self.state.phase {
                LoginPhase::LoadingRooms => native_lobby::upsert(
                    self.pending_rooms.as_mut().context("missing room buffer")?,
                    room,
                )?,
                LoginPhase::LobbyReady
                | LoginPhase::RoomJoining
                | LoginPhase::RoomCreating
                | LoginPhase::RoomEditing
                | LoginPhase::KickPending
                | LoginPhase::ObserverPending
                | LoginPhase::ModePending
                                | LoginPhase::MapPending
                | LoginPhase::RoomAwaitSelf
                | LoginPhase::RoomReady
                | LoginPhase::ReadyPending
                | LoginPhase::ForcePending
                | LoginPhase::RoomLeaving => native_lobby::upsert(&mut self.state.rooms, room)?,
                LoginPhase::ReturningCharacters => {}
                _ => anyhow::bail!("room row outside lobby"),
            },
            L::SourceRefresh => {
                ensure!(!self.rental_restart_quit_pending,"source refresh preceded rental quit acknowledgement");
                ensure!(self.state.rental_restart_acks==0 || self.state.rental_set.is_none()
                    || self.state.joined_room.is_none(),"source refresh preceded rental departure");
                ensure!(
                    self.state.phase == LoginPhase::ReturningCharacters,
                    "unsolicited source refresh permission"
                );
                let source = self
                    .state
                    .selected_source
                    .as_ref()
                    .context("missing return source")?;
                self.transport
                    .send(self.transport.snapshot().epoch, source_request(source)?)?;
                self.directory = super::native_game::Directory::default();
                self.state.characters.clear();
                self.state.selected_character = None;
                self.state.inventory.suspend();
                self.pending_roster_forces.clear();self.early_roster.clear();
                self.state.joined_room=None;self.state.world.clear_preserving_generation();self.state.environment.retire_scene();self.state.merchant.clear();
                self.state.rental_set=None;self.rental_set_pending=None;self.state.shop_points=None;
                self.state.classic_entry_result=None;
                self.state.character_result = None;
                self.state.channel = None;
                self.state.channel_result = None;
                self.state.rooms.clear();
                self.requested_channel = None;
                self.requested_room = None;
                self.pending_rooms = None;
                self.state.phase = LoginPhase::LoadingCharacters;
                self.deadline = Some(Instant::now() + Duration::from_secs(8));
            }
            L::Other => return Ok(false),
        }
        Ok(true)
    }
    fn accept_game(&mut self, body: &[u8]) -> Result<()> {
        if self.state.phase==LoginPhase::TeleportQuitPending {
            if body.first()==Some(&0xf0) {
                ensure!(body==[0xf0,0],"malformed teleport quit acknowledgement");
                self.retire_teleport_scene();
                self.transport.cancel();
                self.state.phase=LoginPhase::HandoffClosing;
                self.deadline=None;
            }
            // Only the quit acknowledgement can advance this retired world.
            // Already queued old-world updates cannot repopulate its authority.
            return Ok(());
        }
        let environment_packet = self.state.environment.accept(self.transport.snapshot().epoch, body)?;
        if environment_packet && !matches!(body.first(), Some(0x0c | 0x21)) {
            return Ok(());
        }
        if body.first()==Some(&0xf3) && self.state.phase==LoginPhase::BattlePending {
            let owner=self.merchant_owner().context("teleport status outside live rental world")?;
            ensure!(self.state.merchant.waiting,"teleport action no longer pending");
            self.teleport.as_mut().context("unsolicited teleport status")?
                .status(&owner,super::native_services::parse_availability(body)?)?;
            return Ok(());
        }
        if body.first()==Some(&0xf4) {
            ensure!(self.state.phase==LoginPhase::BattlePending&&self.state.merchant.waiting,"unsolicited teleport endpoint");
            let owner=self.merchant_owner().context("teleport endpoint outside live rental world")?;
            let current=self.transport.snapshot().negotiated.as_ref().context("teleport transport")?.peer;
            let target=self.teleport.as_mut().context("no pending teleport action")?
                .endpoint(&owner,super::native_services::parse_endpoint(body)?,current)?;
            self.teleport.as_mut().unwrap().remember_same_endpoint_services(current,self.state.services.as_ref());
            self.teleport_quit_tx=Some(self.transport.send_final(owner.connection,Zeroizing::new(vec![0xd9]))?);
            self.state.redirect=Some(target);
            self.state.phase=LoginPhase::TeleportQuitPending;
            self.state.merchant.clear();
            self.state.world.clear_preserving_generation();self.state.environment.retire_scene();
            self.state.environment.retire();
            self.state.world.relations.retire();
            self.interrupt_world_actions();
            self.deadline=Some(Instant::now()+Duration::from_secs(8));
            return Ok(());
        }
        if body.first()==Some(&0x65) {
            ensure!(body==[0x65]&&self.state.phase==LoginPhase::RentalRestartPending,"unsolicited or malformed rental restart acknowledgement");
            self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0xd9]))?;
            self.state.rental_restart_acks+=1;self.rental_restart_quit_pending=true;self.state.phase=LoginPhase::ReturningCharacters;
            self.deadline=Some(Instant::now()+Duration::from_secs(8));return Ok(());
        }
        if body.first()==Some(&0xf0) && (self.state.phase==LoginPhase::RentalRestartPending
            || (self.state.phase==LoginPhase::ReturningCharacters
                && self.state.rental_restart_acks>0 && self.state.rental_set.is_some())) {
            ensure!(body==[0xf0,0] && self.rental_restart_quit_pending
                && self.state.phase==LoginPhase::ReturningCharacters,
                "unsolicited, duplicate or malformed rental quit acknowledgement");
            // Rental gameplay admits C_D0, while its opcode filter rejects C_CB.
            // S_D0 retires the room; S_F5 then permits the source refresh.
            self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(vec![0xd0]))?;
            self.rental_restart_quit_pending=false;
            self.deadline=Some(Instant::now()+Duration::from_secs(8));return Ok(());
        }
        if body.first()==Some(&0x5b){
            let owner=self.merchant_owner();self.state.merchant.tick(owner.as_ref(),Instant::now());
            self.state.merchant.accept_input(super::native_merchant::parse_input(body)?,Instant::now())?;return Ok(());
        }
        if body.first().is_some_and(|op|[0x2a,0x2b,0x2c].contains(op)){
            let owner=self.merchant_owner();self.state.merchant.tick(owner.as_ref(),Instant::now());
            self.state.merchant.accept(super::native_merchant::parse(body)?,Instant::now())?;return Ok(());
        }
        if body.first()==Some(&0xdb)&&self.state.world.battle_result.is_none(){
            let(name,points)=super::native_points::parse(body)?;
            ensure!(self.state.selected_character.as_deref()==Some(name.as_str()),"points for a different character");
            self.state.shop_points=Some(points);return Ok(());
        }
        if body.first()==Some(&0xfb){self.state.services=Some(super::native_services::parse(body)?);return Ok(());}
        if body.first()==Some(&0xf0)&&self.state.phase==LoginPhase::RentalArmPending {
            ensure!(body==[0xf0,0],"rental arm acknowledgement");self.state.phase=LoginPhase::RentalAwaitPermission;return Ok(());
        }
        if body.first()==Some(&0xf5)&&self.state.phase==LoginPhase::RentalAwaitPermission {
            ensure!(body==[0xf5],"rental permission envelope");
            self.transport.send(self.transport.snapshot().epoch,super::native_room::join(self.requested_room.context("rental room lost")?,"")?)?;
            self.state.phase=LoginPhase::RoomJoining;return Ok(());
        }
        if self
            .state
            .inventory
            .accept(self.transport.snapshot().epoch, body)?
        {
            return Ok(());
        }
        if body.first() == Some(&0xf0) && self.state.phase == LoginPhase::ResultReturning {
            ensure!(
                body == [0xf0, 0],
                "unsupported fight return acknowledgement"
            );
            self.transport
                .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0xd0]))?;
            self.state.phase = LoginPhase::RoomLeaving;
            self.deadline = Some(Instant::now() + Duration::from_secs(8));
            return Ok(());
        }
        if body.first()==Some(&0x5a){
            let destination=super::native_portal::parse(body)?;
            let tree=self.world_tree_request.take().filter(|(stamp,since)|
                *stamp==super::native_navigation::InputStamp::capture(self.transport.snapshot().epoch,&self.state.world)
                && since.elapsed()<Duration::from_secs(2));
            if tree.is_some(){
                super::native_portal::admit_world_tree(&self.state,destination)?;
                self.state.phase=LoginPhase::WorldEntering;
            }else{super::native_portal::admit(&self.state,destination)?;}
        }
        if body.first() == Some(&0x28) {
            self.state.merchant.clear();
            ensure!(
                self.state.joined_room.is_some(),
                "world before room assignment"
            );
        }
        let previous_world_generation=self.state.world.generation;
        if self.state.world.accept(body)? {
            if matches!(body.first(), Some(0x28 | 0x5a)) {
                self.state.environment.map_changed(self.transport.snapshot().epoch,
                    self.state.world.map_id.context("accepted environment map")?,
                    (body.first() == Some(&0x28)).then_some(self.state.world.underwater))?;
                if self.state.world.generation!=previous_world_generation {
                    self.state.environment.ambient_scene_changed(self.transport.snapshot().epoch,self.state.world.map_id.context("ambient destination map")?)?;
                }
            }
            self.state.world.set_observer_mode(self.state.observer);
            if body.first()==Some(&0x0f) && self.state.battle_withdrawal_pending
                && self.state.world.last_system_message.as_ref().is_some_and(|(kind,text)|
                    *kind==9 && text.starts_with("戰鬥中無法退出，請停止攻擊並等待 ") && text.ends_with(" 秒。")) {
                self.state.battle_withdrawal_pending=false;
                self.deadline=None;
            }
            if body.first()==Some(&0x5a){
                if self.portal_entry.take().is_some(){self.portal_feedback=super::native_portal::EntryFeedback::SceneUpdated;}
                self.state.item_feedback = self.item_pending.as_ref()
                    .map(|p|p.observe(&self.state,Instant::now()))
                    .filter(|f|*f==super::native_item_actions::Feedback::SceneUpdated)
                    .unwrap_or(super::native_item_actions::Feedback::Interrupted);
                self.state.merchant.clear();self.attack_pending=None;self.skill_pending=None;self.item_pending=None;self.pickup_pending=None;self.door_pending=None;self.skill_ready=None;
                self.state.attack_feedback=super::native_combat::Feedback::Interrupted;self.state.skill_feedback=super::native_skills::Feedback::Interrupted;
                self.transport.send(self.transport.snapshot().epoch,Zeroizing::new(super::native_portal::ACK.to_vec()))?;
                self.deadline=Some(Instant::now()+Duration::from_secs(8));
            }
            if self.state.world.finish_portal_if_ready()?{
                if self.state.phase==LoginPhase::WorldEntering {
                    // Formal notifyPeersOfDisconnect sends S_D0 to peers only.
                    // The exact pending destination plus rebuilt self is our own
                    // confirmation of completeClassicWorldPortal's leaveRoom/leaveChannel.
                    if let Some(room)=self.state.joined_room.as_ref(){
                        let id=room.id;let own=self.state.selected_character.as_deref().context("WORLD character")?;
                        ensure!(super::native_room::depart(&mut self.state.joined_room,&mut self.retired_departures,id,own,own)?==super::native_room::Departure::SelfLeft,"WORLD retirement ownership");
                    }
                    self.requested_room=None;self.pending_roster_forces.clear();self.early_roster.clear();self.kick_pending=None;self.observer_return=None;
                    self.state.channel=None;self.requested_channel=None;self.state.rooms.clear();self.pending_rooms=None;
                    self.state.phase=LoginPhase::WorldReady;
                }
                if let Some(room)=self.state.joined_room.as_mut(){room.map_id=self.state.world.map_id.context("portal map")?;}
                self.deadline=None;
            }
            if body.first() == Some(&0x11) {
                if let Some(event) = self.state.world.last_resurrection {
                    let own = self.state.world.own_actor().is_some_and(|a| a.id == event.target);
                    if own || self.attack_pending.as_ref().is_some_and(|p| p.target == event.target) {
                        self.attack_pending = None;
                        self.state.attack_feedback = super::native_combat::Feedback::Interrupted;
                    }
                    if own {
                        // S_RESURRECT completes C_CONFIRM_DEATH's response wait;
                        // keeping that deadline disconnects a live honor player.
                        self.deadline = None;
                        self.skill_pending = None;
        self.world_tree_request=None;
                        self.state.skill_feedback = super::native_skills::Feedback::Interrupted;
                        self.door_pending = None;
                        self.door_feedback = super::native_doors::Feedback::Interrupted;
                    }
                }
            }
            if body.first() == Some(&0x28) {
                self.attack_pending = None;
                self.state.attack_feedback = super::native_combat::Feedback::Idle;
            }
            if body.first() == Some(&0x63) {
                self.transport
                    .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0x5a]))?;
            }
            if body.first() == Some(&0xd2) {
                self.state.battle_withdrawal_pending=false;
                self.state.phase = LoginPhase::BattlePending;
                self.deadline = None;
            }
            if body.first() == Some(&0xd3) {
                self.state.battle_withdrawal_pending=false;
                if let Some(pending) = &self.attack_pending {
                    self.state.attack_feedback = pending.observe(&self.state.world, Instant::now());
                }
                self.state.phase = LoginPhase::BattleResult;
                self.deadline = None;
                self.attack_pending = None;
            }
            return Ok(());
        }
        if self.accept_room(body)? {
            return Ok(());
        }
        if self.accept_lobby(body)? {
            return Ok(());
        }

        use super::native_game::{Notice, Packet as G};
        if body.first() == Some(&0) {
            ensure!(
                self.state.phase == LoginPhase::GameLoginPending,
                "unsolicited GAME authentication result"
            );
            if let Packet::LoginCheck(check) = parse(body)? {
                ensure!(
                    check.result == 0,
                    "GAME authentication rejected: {}",
                    check.result
                );
            }
            return Ok(());
        }
        match super::native_game::parse(body)? {
            G::Notice(id) => {
                ensure!(
                    self.state.phase == LoginPhase::GameLoginPending && self.state.notice.is_none(),
                    "unsolicited GAME notice"
                );
                self.state.notice = Some(Notice {
                    message_id: Some(id),
                    text: None,
                });
                self.deadline = None;
            }
            G::Ready => {
                ensure!(
                    self.state.phase == LoginPhase::GameLoginPending
                        && !self.readiness_acked
                        && self.state.notice.is_none(),
                    "unsolicited GAME readiness"
                );
                self.transport
                    .send(self.transport.snapshot().epoch, Zeroizing::new(vec![0xe2]))?;
                self.state.acknowledgement_opcodes.push(0xe2);
                self.readiness_acked = true;
                self.state.phase = LoginPhase::LoadingCharacters;
                self.deadline = Some(Instant::now() + Duration::from_secs(8));
            }
            G::News(text) => {
                ensure!(
                    self.state.phase == LoginPhase::LoadingCharacters
                        && self.state.notice.is_none(),
                    "unsolicited GAME common news"
                );
                self.state.notice = Some(Notice {
                    message_id: None,
                    text: Some(text),
                });
                self.deadline = None;
            }
            G::Count(count) => {
                ensure!(
                    self.state.phase == LoginPhase::LoadingCharacters && self.readiness_acked,
                    "character count before readiness"
                );
                self.directory.start(count)?;
            }
            G::Row(row) => {
                ensure!(
                    self.state.phase == LoginPhase::LoadingCharacters && self.readiness_acked,
                    "character row outside directory"
                );
                self.directory.push(row)?;
            }
            G::ClassicEntry(check) => {
                ensure!(self.state.phase==LoginPhase::ClassicEntryPending,"unsolicited classic entry check");
                self.state.phase=if check.accepted {self.state.inventory.confirm_selection();LoginPhase::ClassicCharacterSelected}
                    else {self.state.selected_character=None;self.state.inventory.reject_selection();LoginPhase::CharactersReady};
                if check.accepted {self.transport.start_game_timers(self.transport.snapshot().epoch)?;}
                self.state.classic_entry_result=Some(check);self.deadline=None;
            }
            G::Selected(result) => {
                ensure!(
                    self.state.phase == LoginPhase::CharacterPending,
                    "unsolicited character selection result"
                );
                self.state.character_result = Some(result);
                self.state.phase = if result == 0 {
                    self.state.inventory.confirm_selection();
                    self.transport.start_game_timers(self.transport.snapshot().epoch)?;
                    LoginPhase::CharacterSelected
                } else {
                    self.state.selected_character = None;
                    self.state.inventory.reject_selection();
                    LoginPhase::CharactersReady
                };
                self.deadline = None;
                if self.teleport.is_some() || self.recovery_character.is_some() {
                    ensure!(result==0,"destination rejected teleport continuation");
                    if let Some(pending)=self.teleport.as_mut() {
                        let transport=self.transport.snapshot();
                        let restored=transport.negotiated.as_ref().and_then(|connection|
                            pending.take_same_endpoint_services(connection.peer,transport.epoch,self.state.selected_character.as_deref().unwrap_or("")));
                        // A directory actually sent by the new GAME takes priority.
                        if self.state.services.is_none() { self.state.services=restored; }
                    }
                    self.teleport=None;self.recovery_character=None;
                }
            }
            G::Other(_) => {}
        }
        if self.state.phase == LoginPhase::LoadingCharacters && self.directory.complete() {
            if self.confirm_initial_source {
                let source = self.state.selected_source.as_ref().context("missing initial GAME source")?;
                self.transport.send(self.transport.snapshot().epoch, source_request(source)?)?;
                self.confirm_initial_source = false;
                self.directory = super::native_game::Directory::default();
                self.deadline = Some(Instant::now() + Duration::from_secs(8));
                return Ok(());
            }
            self.state.characters = self.directory.view()?;
            self.state.directory_generation = self
                .state
                .directory_generation
                .checked_add(1)
                .context("directory generation exhausted")?;
            if let Some(character)=self.teleport.as_ref().filter(|t|t.destination.is_some()).map(|t|t.character.as_str()).or(self.recovery_character.as_deref()) {
                ensure!(self.state.characters.iter().any(|c|c.name==character),
                    "teleport character absent from destination directory");
                self.state.selected_character=Some(character.to_owned());
                self.state.inventory.begin(self.transport.snapshot().epoch,character)?;
                // Formal GAME bootstrap auto-selects the staged rental loadout.
                // A second C_C9 would race that one-shot server continuation.
                self.state.phase=LoginPhase::CharacterPending;
                self.deadline=Some(Instant::now()+Duration::from_secs(8));
            } else {
                self.state.phase = LoginPhase::CharactersReady;
                self.deadline = None;
            }
        }
        Ok(())
    }
    pub fn cancel(&mut self) {
        self.rental_return_reconnect=None;
        self.confirm_initial_source = false;
        self.state.environment.retire();
        self.state.world.relations.retire();
        // Cancellation closes transport and clears credentials/authority. Keep
        // only one correlation for a later explicit authenticated LOGIN: the
        // server may still have the loadout staged after C_D9 was written.
        self.teleport_recovery=self.teleport_recovery_candidate();
        self.retire_teleport_scene();
        self.teleport=None;self.recovery_character=None;
        self.state.battle_withdrawal_pending=false;
        self.rental_restart_quit_pending=false;
        self.state.merchant.clear();
        if self.portal_entry.take().is_some(){self.portal_feedback=super::native_portal::EntryFeedback::Interrupted;}self.door_pending=None;self.door_feedback=super::native_doors::Feedback::Interrupted;
        self.retired_departures.clear();
        self.pending_roster_forces.clear();self.early_roster.clear();
        self.state.world.emblems = Default::default();
        self.pickup_pending = None;
        self.state.pickup_feedback = super::native_ground_pickup::Feedback::Interrupted;
        self.item_pending = None;
        self.state.item_feedback = super::native_item_actions::Feedback::Interrupted;
        self.state.inventory.clear();
        self.credentials = None;
        self.state.notice = None;
        self.transport.cancel();
        self.deadline = None;
        if self.state.phase==LoginPhase::HandoffClosing || !self.is_quiescent() {
            self.state.phase = LoginPhase::Closing;
        }
    }
    fn accept(&mut self, packet: Packet) -> Result<()> {
        match packet {
            Packet::Services(directory)=>self.state.services=Some(directory),
            Packet::LoginOk | Packet::LoginCheck(LoginCheck { result: 0, .. }) => {
                ensure!(
                    matches!(
                        self.state.phase,
                        LoginPhase::LoginPending | LoginPhase::AwaitingSources
                    ),
                    "unsolicited native login success"
                );
                self.state.phase = LoginPhase::AwaitingSources;
            }
            Packet::LoginCheck(check) => {
                ensure!(
                    self.state.phase == LoginPhase::LoginPending,
                    "unsolicited native login failure"
                );
                self.credentials = None;
                self.state.login_result = Some(check);
                self.state.phase = LoginPhase::ReadyForLogin;
                self.deadline = None;
            }
            Packet::Sources(sources) => {
                ensure!(
                    matches!(
                        self.state.phase,
                        LoginPhase::AwaitingSources | LoginPhase::SourcesReady
                    ),
                    "native sources before authentication"
                );
                self.state.sources = sources;
                self.state.phase = LoginPhase::SourcesReady;
                self.deadline = None;
            }
            Packet::Redirect { endpoint, .. } => {
                ensure!(
                    self.state.phase == LoginPhase::RedirectPending,
                    "unsolicited native redirect"
                );
                self.state.redirect = Some(endpoint);
                self.state.phase = LoginPhase::RedirectReady;
                self.deadline = None;
            }
            Packet::TimeLimit(_) | Packet::Other(_) => {}
        };
        Ok(())
    }
    pub fn poll(&mut self) {
        for event in self.transport.poll() {
            if matches!(
                self.state.phase,
                LoginPhase::Failed | LoginPhase::Closing | LoginPhase::HandoffClosing
            ) {
                continue;
            }
            let result = match event {
                Event::Negotiated { epoch, .. } => {
                    if epoch != self.transport.snapshot().epoch { continue; }
                    if self.game {
                        let result = (|| -> Result<()> {
                            let (account, password) = self
                                .credentials
                                .as_ref()
                                .context("missing GAME credentials")?;
                            let info = self
                                .transport
                                .snapshot()
                                .negotiated
                                .as_ref()
                                .context("missing GAME transport")?;
                            let local = match info.local.ip() {
                                std::net::IpAddr::V4(ip) => ip,
                                _ => Ipv4Addr::UNSPECIFIED,
                            };
                            let body = login_request(&account, &password, local)?;
                            self.transport.send(self.transport.snapshot().epoch, body)
                        })();
                        if result.is_ok() {
                            if let Err(error) = self.state.environment.begin(epoch) {
                                self.fail(format!("GAME environment: {error:#}"));
                                continue;
                            }
                            if let Err(error)=self.state.world.relations.begin(epoch) {
                                self.fail(format!("GAME relations: {error:#}"));
                                continue;
                            }
                            self.state.phase = LoginPhase::GameLoginPending;
                            self.state.game_connected = true;
                            self.deadline = Some(Instant::now() + Duration::from_secs(8));
                        }
                        result
                    } else {
                        self.state.phase = LoginPhase::ReadyForLogin;
                        Ok(())
                    }
                }
                Event::Packet { epoch, body, .. } => {
                    if epoch != self.transport.snapshot().epoch { continue; }
                    if self.game {
                        self.accept_game(&body)
                    } else {
                        parse(&body).and_then(|packet| self.accept(packet))
                    }
                }
                Event::Sent { epoch, tx_counter } => {
                    if self.state.phase==LoginPhase::TeleportQuitPending && epoch==self.transport.snapshot().epoch
                        && self.teleport_quit_tx==Some(tx_counter) {
                        // The native cross-endpoint path resets after C_D9 was
                        // actually written; its old S_F0 reply is best effort.
                        self.retire_teleport_scene();self.transport.cancel();
                        self.state.phase=LoginPhase::HandoffClosing;self.deadline=None;
                    }
                    Ok(())
                },
            };
            if let Err(error) = result {
                self.fail(format!("{error:#}"));
            }
        }
        if let Some(epoch)=self.state.environment.epoch {
            let generation=self.state.world.view_ready().then_some(self.state.world.generation);
            if let Err(error)=self.state.environment.water_poll(epoch,generation) {
                self.fail(format!("GAME visual clock: {error:#}"));
            }
        }
        let owner=self.merchant_owner();self.state.merchant.tick(owner.as_ref(),Instant::now());
        if self.state.phase==LoginPhase::BattlePending && self.teleport.as_ref().is_some_and(|t|
            owner.as_ref()!=Some(&t.owner)||!self.state.merchant.waiting) {self.teleport=None;}
        if self.pending_roster_forces.expired(Instant::now()) {
            self.fail("room force notification was not followed by membership".into());
        }
        if self.world_tree_request.is_some_and(|(_,since)|since.elapsed()>=Duration::from_secs(2)){self.world_tree_request=None;}
        if let Some(pending)=&self.portal_entry {
            self.portal_feedback=if self.state.phase==LoginPhase::BattlePending {
                pending.observe(self.transport.snapshot().epoch,&self.state.world,Instant::now())
            }else{super::native_portal::EntryFeedback::Interrupted};
            if self.portal_feedback!=super::native_portal::EntryFeedback::Pending{self.portal_entry=None;}
        }
        if let Some(pending)=&self.door_pending {
            self.door_feedback=if self.state.phase==LoginPhase::BattlePending {
                pending.observe(self.transport.snapshot().epoch,&self.state.world,Instant::now())
            }else{super::native_doors::Feedback::Interrupted};
            if self.door_feedback!=super::native_doors::Feedback::Pending{self.door_pending=None;}
        }
        if let Some(pending) = &self.pickup_pending {
            let feedback = pending.observe(&self.state, Instant::now());
            self.state.pickup_feedback = feedback;
            if feedback != super::native_ground_pickup::Feedback::Pending {
                self.pickup_pending = None;
            }
        }
        if let Some(pending) = &self.item_pending {
            let feedback = pending.observe(&self.state, Instant::now());
            self.state.item_feedback = feedback;
            if feedback != super::native_item_actions::Feedback::Pending {
                self.item_pending = None;
            }
        }
        if let Some(pending) = &self.skill_pending {
            let feedback = if self.state.phase.active_world() {
                pending.observe(&self.state.world, Instant::now())
            } else {
                super::native_skills::Feedback::Interrupted
            };
            self.state.skill_feedback = feedback;
            if feedback != super::native_skills::Feedback::Pending {
                self.skill_pending = None;
                if feedback!=super::native_skills::Feedback::CastAction {self.world_tree_request=None;}
            }
        }
        if let Some(pending) = &self.attack_pending {
            let feedback = if self.state.phase.active_world() {
                pending.observe(&self.state.world, Instant::now())
            } else {
                super::native_combat::Feedback::Interrupted
            };
            self.state.attack_feedback = feedback;
            if feedback != super::native_combat::Feedback::Pending {
                self.attack_pending = None;
            }
        }
        if self.deadline.is_some_and(|time| Instant::now() >= time) {
            if self.state.phase==LoginPhase::ReturningCharacters
                && self.rental_restart_quit_pending && self.state.rental_restart_acks==1 {
                // Formal rental BATTLE may acknowledge C10 but never finish
                // quitFight. Keep the original8s deadline; retire that peer,
                // then obtain a fresh source directory through ordinary LOGIN.
                if let Err(error)=self.begin_rental_return_reconnect() {
                    self.fail(format!("rental return recovery: {error:#}"));
                }
            } else if self.state.phase==LoginPhase::CharacterPending
                && self.teleport.is_none() && self.recovery_character.is_some()
                && self.state.character_result.is_none() && self.state.inventory.batches==0 {
                // A fresh LOGIN can outlive the server's staged handoff. The
                // protocol has no explicit "stage expired" response. With no
                // selection ack or inventory, retire this one expectation and
                // return to the current directory; do not invent authority or
                // send a second C_C9 while server continuation is pending.
                self.recovery_character=None;self.state.selected_character=None;
                self.state.inventory.clear();self.deadline=None;
                self.state.phase=LoginPhase::CharactersReady;
            } else {self.fail("native login response deadline exceeded".into());}
        }
        if self.state.phase.active_world() && self.state.world.ready() {
            let wanted = self.state.world.actors.values()
                .filter(|a| a.emblem != 0 && self.state.world.actor_visible(a.id))
                .map(|a| a.emblem).collect();
            // Bound per-poll work independently of the 32 outstanding requests.
            for _ in 0..4 {
                let now = Instant::now();
                let Some(id) = self.state.world.emblems.next_request(&wanted, now) else { break; };
                let sent = super::native_emblems::request(id).and_then(|body|
                    self.transport.send(self.transport.snapshot().epoch, Zeroizing::new(body)));
                if let Err(error) = sent.and_then(|_| self.state.world.emblems.mark_sent(id, now)) {
                    self.fail(format!("emblem request: {error:#}"));
                    break;
                }
            }
        }
        if self.transport.is_quiescent() {
            self.deadline = None;
            if self.rental_return_reconnect.as_ref().is_some_and(|r|r.closing) {
                let mut pending=self.rental_return_reconnect.take().unwrap();
                let Some(credentials)=self.credentials.take() else {
                    self.fail("rental return credentials retired".into());
                    return;
                };
                match self.start(pending.login_peer,self.limits) {
                    Ok(())=>{
                        pending.closing=false;
                        self.credentials=Some(credentials);
                        self.rental_return_reconnect=Some(pending);
                    }
                    Err(error)=>self.fail(format!("rental return LOGIN: {error:#}")),
                }
            } else if self.state.phase == LoginPhase::HandoffClosing {
                let target = self.state.redirect.unwrap();
                match self.transport.start(target, self.limits) {
                    Ok(_) => {
                        self.game = true;
                        self.state.phase = LoginPhase::GameConnecting;
                    }
                    Err(error) => self.fail(format!("{error:#}")),
                }
            } else if self.state.phase == LoginPhase::Closing {
                self.state.phase = LoginPhase::Closed;
            } else if self.transport.snapshot().phase == Phase::Failed
                && self.state.phase != LoginPhase::Failed
            {
                self.fail(
                    self.transport
                        .snapshot()
                        .error
                        .clone()
                        .unwrap_or_else(|| "native connection ended".into()),
                );
            }
        }
        if let Err(error)=self.poll_rental_return_reconnect() {
            self.fail(format!("rental return source refresh: {error:#}"));
        }
    }
    fn begin_rental_return_reconnect(&mut self)->Result<()> {
        ensure!(self.state.phase==LoginPhase::ReturningCharacters
            && self.rental_restart_quit_pending && self.state.rental_restart_acks==1,
            "rental restart acknowledgement required");
        let login_peer=self.login_peer.context("original LOGIN peer missing")?;
        ensure!(self.credentials.is_some(),"rental return credentials missing");
        let source=self.state.selected_source.clone().filter(|s|!s.is_empty())
            .context("original rental source missing")?;
        ensure!(self.state.game_connected && self.state.selected_character.is_some()
            && self.state.rental_set.is_some()
            && self.state.joined_room.as_ref().is_some_and(|room|room.mode==6),
            "owned rental return context missing");
        self.teleport=None;self.teleport_recovery=None;self.recovery_character=None;
        self.retire_teleport_scene();
        self.rental_return_reconnect=Some(RentalReturnReconnect{login_peer,source,closing:true});
        self.transport.cancel();self.deadline=None;self.state.phase=LoginPhase::HandoffClosing;
        Ok(())
    }
    fn poll_rental_return_reconnect(&mut self)->Result<()> {
        let Some(pending)=self.rental_return_reconnect.as_ref() else{return Ok(())};
        if pending.closing{return Ok(())}
        let source=pending.source.clone();
        match self.state.phase {
            LoginPhase::ReadyForLogin=>{
                let (account,password)=self.credentials.as_ref().context("return credentials missing")?;
                let account=Zeroizing::new(account.as_str().to_owned());
                let password=Zeroizing::new(password.as_str().to_owned());
                self.login(account.as_str(),password.as_str())?;
            }
            LoginPhase::SourcesReady=>{
                let index=self.state.sources.iter().position(|entry|entry.name==source)
                    .context("original rental source no longer advertised")?;
                self.select_source(index)?;
            }
            LoginPhase::RedirectReady=>self.begin_game()?,
            LoginPhase::CharactersReady=>{
                ensure!(self.state.selected_source.as_deref()==Some(source.as_str())
                    && self.state.selected_character.is_none() && self.state.joined_room.is_none()
                    && self.state.inventory.items.is_empty() && !self.state.world.ready(),
                    "return source directory retained retired rental authority");
                self.rental_return_reconnect=None;
            }
            _=>{}
        }
        Ok(())
    }
    fn teleport_recovery_candidate(&self)->Option<TeleportRecovery> {
        let teleport=self.teleport.as_ref().filter(|t|t.destination.is_some())?;
        Some(TeleportRecovery{login_peer:self.login_peer?,account:self.credentials.as_ref()?.0.as_str().to_owned(),
            source:self.state.selected_source.clone()?,character:teleport.character.clone()})
    }
    fn fail(&mut self, error: String) {
        self.rental_return_reconnect=None;
        self.confirm_initial_source = false;
        self.state.environment.retire();
        self.state.world.relations.retire();
        if let Some(recovery)=self.teleport_recovery_candidate() {
            // No password survives failure. A one-attempt correlation survives
            // only a fresh authenticated LOGIN to the same peer/account/source.
            self.teleport_recovery=Some(recovery);
        }
        self.retire_teleport_scene();
        self.teleport=None;
        self.recovery_character=None;
        self.state.battle_withdrawal_pending=false;
        self.rental_restart_quit_pending=false;
        self.pending_roster_forces.clear();self.early_roster.clear();
        self.state.merchant.clear();
        if self.portal_entry.take().is_some(){self.portal_feedback=super::native_portal::EntryFeedback::Interrupted;}self.door_pending=None;self.door_feedback=super::native_doors::Feedback::Interrupted;
        self.state.world.emblems = Default::default();
        self.pickup_pending = None;
        self.state.pickup_feedback = super::native_ground_pickup::Feedback::Interrupted;
        self.item_pending = None;
        self.state.item_feedback = super::native_item_actions::Feedback::Interrupted;
        self.state.inventory.clear();
        self.credentials = None;
        self.state.notice = None;
        self.state.phase = LoginPhase::Failed;
        self.state.error = Some(error);
        self.deadline = None;
        self.transport.cancel();
    }
    fn interrupt_world_actions(&mut self) {
        self.attack_pending=None;self.skill_pending=None;self.skill_ready=None;self.world_tree_request=None;
        self.item_pending=None;self.pickup_pending=None;if self.portal_entry.take().is_some(){self.portal_feedback=super::native_portal::EntryFeedback::Interrupted;}self.door_pending=None;
        self.state.attack_feedback=super::native_combat::Feedback::Interrupted;
        self.state.skill_feedback=super::native_skills::Feedback::Interrupted;
        self.state.item_feedback=super::native_item_actions::Feedback::Interrupted;
        self.state.pickup_feedback=super::native_ground_pickup::Feedback::Interrupted;
        self.door_feedback=super::native_doors::Feedback::Interrupted;
    }
    fn retire_teleport_scene(&mut self) {
        self.state.environment.retire();
        self.state.world.relations.retire();
        self.teleport_quit_tx=None;
        self.state.world.clear_preserving_generation();self.state.environment.retire_scene();self.state.inventory.clear();
        self.state.merchant.clear();self.interrupt_world_actions();
        self.state.joined_room=None;self.state.rooms.clear();self.state.channel=None;
        self.state.services=None;self.state.rental_set=None;self.state.shop_points=None;
        self.state.selected_character=None;self.state.characters.clear();
        self.state.notice=None;self.state.character_result=None;self.state.observer=false;
        self.state.game_connected=false;self.state.acknowledgement_opcodes.clear();
        self.state.battle_withdrawal_pending=false;
        self.requested_channel=None;self.requested_room=None;self.kick_pending=None;
        self.observer_return=None;self.rental_set_pending=None;self.rental_restart_quit_pending=false;
        self.retired_departures.clear();self.pending_roster_forces.clear();self.early_roster.clear();self.pending_rooms=None;
        self.readiness_acked=false;self.directory=Default::default();
    }
}

#[cfg(any(test, feature = "teleport-wire-test"))]
#[doc(hidden)]
#[path = "native_teleport_controller_tests.rs"]
pub mod teleport_wire_tests;

/// Exercises real authentication using isolated test credentials supplied via stdin.
pub fn probe(
    peer: SocketAddr,
    account: &str,
    password: &str,
    retry: Option<&str>,
    source_index: usize,
) -> Result<serde_json::Value> {
    ensure!(
        peer.ip().is_loopback() && peer.port() != 0,
        "login probe requires loopback"
    );
    let mut c = LoginController::default();
    c.start(
        peer,
        Limits {
            read_chunk: 2,
            write_chunk: 3,
            ..Limits::default()
        },
    )?;
    let deadline = Instant::now() + Duration::from_secs(20);
    let mut submitted = false;
    let mut retried = false;
    let mut failures = Vec::new();
    let outcome = (|| -> Result<()> {
        loop {
            c.poll();
            ensure!(Instant::now() < deadline, "native login probe timeout");
            match c.snapshot().phase {
                LoginPhase::ReadyForLogin if !submitted => {
                    c.login(account, password)?;
                    submitted = true;
                }
                LoginPhase::ReadyForLogin if submitted => {
                    let result = c
                        .snapshot()
                        .login_result
                        .as_ref()
                        .context("missing login rejection")?;
                    failures.push(result.result);
                    if let Some(next) = retry.filter(|_| !retried) {
                        c.login(account, next)?;
                        retried = true;
                    } else {
                        break;
                    }
                }
                LoginPhase::SourcesReady => c.select_source(source_index)?,
                LoginPhase::RedirectReady => break,
                LoginPhase::Failed => bail!(
                    "{}",
                    c.snapshot()
                        .error
                        .as_deref()
                        .unwrap_or("native login failed")
                ),
                _ => {}
            };
            std::thread::sleep(Duration::from_millis(3));
        }
        Ok(())
    })();
    let result = c.snapshot().clone();
    let transport = c.transport.snapshot().clone();
    c.cancel();
    while !c.is_quiescent() {
        c.poll();
        std::thread::sleep(Duration::from_millis(3));
    }
    outcome?;
    Ok(
        serde_json::json!({"schema":1,"status":"native_login_probe_completed","pointer_bits":usize::BITS,"login":result,"transport":transport,"rejection_results":failures,"retried":retried,"worker_quiescent":c.is_quiescent(),"shutdown":c.snapshot().phase,"redirect_followed":false,"sql_access":false,"playable_game":false}),
    )
}

/// Full isolated LOGIN-to-GAME flow, sharing the same owner/controller as the UI.
pub fn probe_game(
    peer: SocketAddr,
    account: &str,
    password: &str,
    source: usize,
    select: bool,
) -> Result<serde_json::Value> {
    probe_flow(peer, account, password, source, select, None)
}
pub fn probe_lobby(
    peer: SocketAddr,
    account: &str,
    password: &str,
    source: usize,
    mode: &str,
) -> Result<serde_json::Value> {
    if matches!(mode, "recovery-room" | "recovery-world") {
        return super::native_recovery_probe::run(peer, account, password, source,
            mode.strip_prefix("recovery-").unwrap());
    }
    if matches!(mode, "room-world-peer-a" | "room-world-peer-b" | "room-world-peer-c") {
        return super::native_peer_probe::run(peer, account, password, source,
            if mode.ends_with("-a") { "a" } else if mode.ends_with("-b") { "b" } else {"c"});
    }
    ensure!(
        [
            "enter",
            "inventory",
            "inventory-reselect",
            "inventory-actions",
            "inventory-actions-refused",
            "switch",
            "return",
            "reselect",
            "restricted",
            "room-join",
            "room-ready",
            "room-leave",
            "room-rejoin",
            "room-reject",
            "room-retry",
            "room-world",
            "room-world-move",
            "room-world-correct",
            "room-world-route",
            "room-world-states",
            "room-world-labels",
            "room-world-emblems",
            "room-world-attack",
            "room-world-victory",
            "room-world-defeat",
            "room-world-refused",
            "room-world-cycle",
            "room-world-ranged",
            "room-world-heal",
            "room-world-shield",
            "room-world-bolt",
            "room-world-bolt-refused",
            "room-world-pickup",
            "room-world-pickup-stale",
            "room-world-stream",
            "room-world-stream-no-reagent",
            "room-world-stream-no-mana",
            "room-world-wall",
            "room-world-wall-refused",
            "room-world-item-heal",
            "room-world-item-heal-194",
            "room-world-item-heal-197",
            "room-world-item-drop",
            "room-world-item-drop-refused"
        ]
        .contains(&mode),
        "unknown lobby probe mode"
    );
    probe_flow(peer, account, password, source, true, Some(mode))
}
fn probe_flow(
    peer: SocketAddr,
    account: &str,
    password: &str,
    source: usize,
    select: bool,
    lobby: Option<&str>,
) -> Result<serde_json::Value> {
    ensure!(
        peer.ip().is_loopback() && peer.port() != 0,
        "GAME probe requires loopback"
    );
    let mut c = LoginController::default();
    c.start(
        peer,
        Limits {
            read_chunk: 2,
            write_chunk: 3,
            ..Limits::default()
        },
    )?;
    let end = Instant::now()
        + Duration::from_secs(
            if matches!(
                lobby,
                Some(
                    "room-world-route"
                        | "room-world-states"
                        | "room-world-labels"
                        | "room-world-emblems"
                        | "room-world-attack"
                        | "room-world-victory"
                        | "room-world-defeat"
                        | "room-world-refused"
                        | "room-world-cycle"
                        | "room-world-ranged"
                        | "room-world-heal"
                        | "room-world-shield"
                        | "room-world-bolt"
                        | "room-world-bolt-refused"
                        | "room-world-pickup"
                        | "room-world-pickup-stale"
                        | "room-world-stream"
                        | "room-world-stream-no-reagent"
                        | "room-world-stream-no-mana"
                        | "room-world-wall"
                        | "room-world-wall-refused"
                        | "room-world-item-heal"
                        | "room-world-item-heal-194"
                        | "room-world-item-heal-197"
                        | "room-world-item-drop"
                        | "room-world-item-drop-refused"
                )
            ) {
                90
            } else {
                40
            },
        );
    let mut lobby_history = Vec::new();
    let mut returned = false;
    let mut inventory_return_cleared = false;
    let mut item_probe = super::native_item_probe::Probe::default();
    let mut room_history = Vec::new();
    let mut room_step = 0;
    let mut room_rejections = Vec::new();
    let mut room_retry = false;
    let mut submitted = false;
    let mut login_epoch = 0;
    let world_mode = lobby.is_some_and(|m| m.starts_with("room-world"));
    let mut motion_before = None;
    let mut motion_sent = None;
    let mut motion_tx = 0;
    let mut route_model = None;
    let mut route_load_ms = 0;
    let mut route_samples = Vec::new();
    let mut route_frames = 0;
    let mut route_start = None;
    let mut route_last_frame = None;
    let mut route_path = Vec::new();
    let mut route_index = 0;
    let mut route_wall = 0;
    let mut route_blocked = false;
    let mut navigation_trace = Vec::new();
    let combat_mode = matches!(
        lobby,
        Some(
            "room-world-states"
                | "room-world-labels"
                        | "room-world-emblems"
                | "room-world-attack"
                | "room-world-victory"
                | "room-world-defeat"
                | "room-world-refused"
                | "room-world-cycle"
                | "room-world-ranged"
                | "room-world-heal"
                | "room-world-shield"
                | "room-world-bolt"
                | "room-world-bolt-refused"
                | "room-world-pickup"
                | "room-world-pickup-stale"
                | "room-world-stream"
                | "room-world-stream-no-reagent"
                | "room-world-stream-no-mana"
                | "room-world-wall"
                | "room-world-wall-refused"
                | "room-world-item-heal"
                | "room-world-item-heal-194"
                | "room-world-item-heal-197"
                | "room-world-item-drop"
                | "room-world-item-drop-refused"
        )
    );
    let mut combat = super::native_combat_probe::Probe::default();
    let mut combat_rounds = Vec::new();
    let outcome = (|| -> Result<()> {
        loop {
            c.poll();
            ensure!(Instant::now() < end, "GAME probe deadline");
            if c.snapshot().notice.is_some() {
                c.acknowledge_notice()?;
            }
            if combat_mode
                && (matches!(
                    c.snapshot().phase,
                    LoginPhase::BattlePending
                        | LoginPhase::BattleResult
                        | LoginPhase::ResultReturning
                        | LoginPhase::RoomLeaving
                ) || (c.snapshot().phase == LoginPhase::LobbyReady && combat.result.is_some()))
            {
                if combat.tick(&mut c, lobby.unwrap())? {
                    if lobby == Some("room-world-cycle") {
                        combat_rounds.push(combat.receipt());
                        if combat_rounds.len() == 1 {
                            ensure!(
                                c.snapshot().phase == LoginPhase::LobbyReady
                                    && c.snapshot().joined_room.is_none(),
                                "first round did not leave room"
                            );
                            combat.restart()?;
                            room_step = 0;
                            continue;
                        }
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(3));
                continue;
            }
            match c.snapshot().phase {
                LoginPhase::ReadyForLogin if !submitted => {
                    c.login(account, password)?;
                    submitted = true;
                    login_epoch = c.transport.snapshot().epoch;
                }
                LoginPhase::ReadyForLogin => bail!("LOGIN authentication rejected"),
                LoginPhase::SourcesReady => c.select_source(source)?,
                LoginPhase::RedirectReady => {
                    ensure!(
                        c.snapshot().redirect.unwrap().ip().is_loopback(),
                        "isolated GAME redirect must be loopback"
                    );
                    c.begin_game()?;
                }
                LoginPhase::CharactersReady if lobby == Some("return") && returned => break,
                LoginPhase::CharacterSelected if lobby.is_some() => {
                    if lobby == Some("restricted") && c.snapshot().channel_result == Some(10) {
                        break;
                    }
                    ensure!(
                        c.snapshot().channel_result.is_none(),
                        "unexpected channel rejection"
                    );
                    c.enter_channel(
                        if returned || matches!(lobby, Some("room-reject" | "room-retry")) {
                            3
                        } else {
                            0
                        },
                    )?;
                }
                LoginPhase::RoomReady if lobby.is_some_and(|m| m.starts_with("room-")) => {
                    let room = c
                        .snapshot()
                        .joined_room
                        .as_ref()
                        .context("probe room missing")?;
                    let own = room
                        .members
                        .iter()
                        .find(|m| {
                            Some(m.name.as_str()) == c.snapshot().selected_character.as_deref()
                        })
                        .context("probe own participant missing")?;
                    if !world_mode || room_step == 0 {
                        room_history.push(c.snapshot().clone());
                    }
                    match lobby.unwrap() {
                        m if m.starts_with("room-world") && room_step == 0 => {
                            room_step = 1;
                            c.set_ready(true)?;
                        }
                        m if m.starts_with("room-world") => {}
                        "room-ready" if room_step == 0 => {
                            ensure!(own.state == 0, "initial ready state");
                            room_step = 1;
                            c.set_ready(true)?;
                        }
                        "room-ready" if room_step == 1 => {
                            ensure!(own.state == 1, "ready acknowledgement");
                            room_step = 2;
                            c.set_ready(false)?;
                        }
                        "room-ready" => {
                            ensure!(own.state == 0, "not-ready acknowledgement");
                            break;
                        }
                        "room-leave" | "room-rejoin" if room_step == 0 => {
                            room_step = 1;
                            c.leave_room()?;
                        }
                        _ => break,
                    }
                }
                LoginPhase::BattlePending
                    if world_mode
                        && c.snapshot().world.ready()
                        && c.snapshot().world.integrity_challenge.is_some() =>
                {
                    if lobby == Some("room-world") {
                        break;
                    }
                    if lobby == Some("room-world-route") {
                        if route_model.is_none() {
                            let load_start = Instant::now();
                            let model = super::native_world_scene::Model::load(
                                std::path::Path::new(r"D:\lineage開發\0-客戶端"),
                                &c.snapshot().world,
                                &std::sync::atomic::AtomicBool::new(false),
                            )?;
                            let a = c.snapshot().world.own_actor().unwrap();
                            let (path, wall) = model.navigation().wall_route([a.x, a.y])?;
                            route_load_ms = load_start.elapsed().as_millis();
                            route_path = path;
                            route_wall = wall;
                            route_start = Some(Instant::now());
                            route_model = Some(model);
                            motion_before = Some(c.snapshot().world.clone());
                        }
                        ensure!(
                            c.snapshot().world.local_corrections == 0,
                            "route unexpectedly corrected by server"
                        );
                        if route_last_frame
                            .is_none_or(|t: Instant| t.elapsed() >= Duration::from_millis(50))
                        {
                            route_last_frame = Some(Instant::now());
                            let frame = route_model.as_mut().unwrap().draw_at(
                                &c.snapshot().world,
                                &std::sync::atomic::AtomicBool::new(false),
                                route_start.unwrap().elapsed().as_nanos() as u64,
                            )?;
                            route_frames += 1;
                            ensure!(route_samples.len() < 2048, "route sample budget");
                            route_samples.push(frame.receipt["render_profile_us"].clone());
                            if frame.movement_ready(&c.snapshot().world) {
                                if route_index == route_path.len() && !route_blocked {
                                    let before = serde_json::to_value(&c.snapshot().world)?;
                                    let tx = c
                                        .transport
                                        .snapshot()
                                        .negotiated
                                        .as_ref()
                                        .unwrap()
                                        .tx_counter;
                                    ensure!(
                                        c.move_checked(route_wall, &frame.navigation).is_err(),
                                        "wall move accepted locally"
                                    );
                                    ensure!(
                                        before == serde_json::to_value(&c.snapshot().world)?
                                            && tx
                                                == c.transport
                                                    .snapshot()
                                                    .negotiated
                                                    .as_ref()
                                                    .unwrap()
                                                    .tx_counter,
                                        "blocked move changed world or transport"
                                    );
                                    navigation_trace.push(serde_json::json!({"blocked":true,"heading":route_wall,"tx_unchanged":true,"predictions_unchanged":true,"own":c.snapshot().world.own_actor()}));
                                    route_blocked = true;
                                }
                                if route_index == route_path.len() * 2 {
                                    break;
                                }
                                let h = if route_index < route_path.len() {
                                    route_path[route_index]
                                } else {
                                    (route_path[route_path.len() * 2 - 1 - route_index] + 4) % 8
                                };
                                c.move_checked(h, &frame.navigation)?;
                                route_index += 1;
                                navigation_trace.push(serde_json::json!({"blocked":false,"step":route_index,"heading":h,"elapsed_ms":route_start.unwrap().elapsed().as_millis(),"own":c.snapshot().world.own_actor()}));
                            }
                        }
                        std::thread::sleep(Duration::from_millis(3));
                        continue;
                    }
                    if motion_before.is_none() {
                        motion_before = Some(c.snapshot().world.clone());
                        motion_tx = c
                            .transport
                            .snapshot()
                            .negotiated
                            .as_ref()
                            .context("motion transport")?
                            .tx_counter;
                        if lobby == Some("room-world-move") {
                            c.move_step(0)?;
                        } else {
                            // Isolated loopback diagnostic: intentionally stale source,
                            // with a visible local prediction that must be rolled back.
                            let mut body = c.state.world.move_request(0)?;
                            let x = c
                                .state
                                .world
                                .own_actor()
                                .unwrap()
                                .x
                                .checked_add(5)
                                .context("stale source")?;
                            body[1..3].copy_from_slice(&x.to_le_bytes());
                            c.transport
                                .send(c.transport.snapshot().epoch, Zeroizing::new(body))?;
                            c.state.world.predict_step(0)?;
                        }
                        motion_sent = Some(Instant::now());
                    } else {
                        let tx = c
                            .transport
                            .snapshot()
                            .negotiated
                            .as_ref()
                            .context("motion transport")?
                            .tx_counter;
                        let corrected = lobby == Some("room-world-correct");
                        let bytes = if corrected { 7 } else { 6 };
                        if tx == motion_tx.wrapping_add(bytes)
                            && motion_sent.unwrap().elapsed() >= Duration::from_millis(300)
                            && (!corrected
                                || (c.state.world.local_corrections > 0
                                    && c.state.world.movement_locks > 0))
                        {
                            break;
                        }
                    }
                }
                LoginPhase::LobbyReady if lobby.is_some_and(|m| m.starts_with("room-")) => {
                    lobby_history.push(c.snapshot().clone());
                    if c.snapshot().room_result.is_some_and(|r| r != 0) && !room_retry {
                        room_rejections.push(c.snapshot().room_result.unwrap());
                        ensure!(
                            c.snapshot().room_result == Some(10),
                            "unexpected room refusal"
                        );
                        if lobby == Some("room-reject") {
                            break;
                        }
                        ensure!(lobby == Some("room-retry"), "room rejected");
                        room_retry = true;
                        c.enter_channel(0)?;
                    } else if lobby == Some("room-leave") && room_step == 1 {
                        break;
                    } else {
                        c.join_room(0, "")?;
                    }
                }
                LoginPhase::LobbyReady
                    if matches!(
                        lobby,
                        Some("inventory-actions" | "inventory-actions-refused")
                    ) =>
                {
                    if item_probe.tick(&mut c, lobby == Some("inventory-actions-refused"))? {
                        lobby_history.push(c.snapshot().clone());
                        break;
                    }
                }
                LoginPhase::LobbyReady if lobby.is_some() => {
                    lobby_history.push(c.snapshot().clone());
                    match lobby.unwrap() {
                        "switch" if lobby_history.len() == 1 => c.enter_channel(3)?,
                        "return" | "reselect" | "inventory-reselect" if !returned => {
                            c.return_characters()?;
                            returned = true;
                        }
                        _ => break,
                    }
                }
                LoginPhase::CharactersReady if select && !c.snapshot().characters.is_empty() => {
                    if returned && lobby == Some("inventory-reselect") {
                        ensure!(
                            c.state.inventory.owner.is_none() && c.state.inventory.items.is_empty(),
                            "inventory survived character return"
                        );
                        inventory_return_cleared = true;
                    }
                    c.select_character(0)?
                }
                LoginPhase::CharactersReady | LoginPhase::CharacterSelected => break,
                LoginPhase::Failed => bail!(
                    "{}",
                    c.snapshot().error.as_deref().unwrap_or("GAME failure")
                ),
                _ => {}
            };
            std::thread::sleep(Duration::from_millis(3));
        }
        Ok(())
    })();
    let snapshot = c.snapshot().clone();
    let transport = c.transport.snapshot().clone();
    c.cancel();
    while !c.is_quiescent() {
        c.poll();
        std::thread::sleep(Duration::from_millis(3));
    }
    outcome.with_context(|| {
        format!("world route step={route_index} frames={route_frames} load_ms={route_load_ms}")
    })?;
    let world_render = if world_mode && !combat_mode {
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut model = super::native_world_scene::Model::load(
            std::path::Path::new(r"D:\lineage開發\0-客戶端"),
            &snapshot.world,
            &cancel,
        )?;
        let output =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification");
        super::resources::reject_reparse_chain(&output)?;
        let save =
            |frame: super::native_world_scene::Frame, name: &str| -> Result<serde_json::Value> {
                let path = output.join(name);
                if path.exists() {
                    super::resources::reject_reparse_chain(&path)?;
                }
                let mut png = png::Encoder::new(std::fs::File::create(&path)?, 800, 600);
                png.set_color(png::ColorType::Rgba);
                png.set_depth(png::BitDepth::Eight);
                png.write_header()?.write_image_data(frame.canvas.rgba())?;
                Ok(serde_json::json!({"png":path,"frame":frame.receipt}))
            };
        let before = motion_before
            .as_ref()
            .map(|world| {
                save(
                    model.draw(world, &cancel)?,
                    &format!("{}-before.png", lobby.unwrap()),
                )
            })
            .transpose()?;
        let filename = if lobby == Some("room-world") {
            "native-world.png".into()
        } else {
            format!("{}.png", lobby.unwrap())
        };
        let mut timeline = Vec::new();
        for elapsed_ns in [
            0,
            166_666_667,
            333_333_334,
            500_000_001,
            666_666_668,
            1_000_000_000,
        ] {
            timeline.push(save(
                model.draw_at(&snapshot.world, &cancel, elapsed_ns)?,
                &format!("{}-animation-{}.png", lobby.unwrap(), elapsed_ns),
            )?);
        }
        let mut result = save(
            model.draw_at(&snapshot.world, &cancel, 1_000_000_000)?,
            &filename,
        )?;
        result["before"] = serde_json::to_value(before)?;
        result["timeline"] = serde_json::to_value(timeline)?;
        result["model_reused"] = serde_json::json!(motion_before.is_some());
        Some(result)
    } else {
        None
    };
    let inventory_names = if matches!(lobby, Some("inventory" | "inventory-reselect")) {
        let mut names = super::native_inventory_names::Service::start(std::path::PathBuf::from(
            r"D:\lineage開發\0-客戶端",
        ))?;
        let deadline = Instant::now() + Duration::from_secs(15);
        while !names.quiescent() {
            names.poll();
            ensure!(Instant::now() < deadline, "inventory name loader timeout");
            std::thread::sleep(Duration::from_millis(3));
        }
        let catalog = names
            .catalog()
            .context("inventory name catalog unavailable")?;
        let views: Vec<Vec<_>> = lobby_history
            .iter()
            .map(|h: &Snapshot| {
                h.inventory
                    .items
                    .values()
                    .map(|i| super::native_inventory_names::label(i, Some(catalog)))
                    .collect()
            })
            .collect();
        let mut result = serde_json::json!({"sources":catalog.sources,"views":views,"worker_quiescent":names.quiescent()});
        names.cancel();
        result["cancelled_catalog_empty"] = serde_json::json!(names.catalog().is_none());
        Some(result)
    } else {
        None
    };
    let inventory_icons = if matches!(lobby, Some("inventory" | "inventory-reselect")) {
        let mut icons = super::native_inventory_icons::Service::new(std::path::PathBuf::from(
            r"D:\lineage開發\0-客戶端",
        ));
        let mut views = Vec::new();
        for h in &lobby_history {
            let ids: Vec<_> = h.inventory.items.values().map(|i| i.icon).collect();
            let deadline = Instant::now() + Duration::from_secs(15);
            loop {
                icons.tick(&ids);
                if icons.quiescent() {
                    break;
                }
                ensure!(Instant::now() < deadline, "inventory icon loader timeout");
                std::thread::sleep(Duration::from_millis(3));
            }
            let mut rows = Vec::new();
            for i in h.inventory.items.values() {
                let icon = icons.icon(i.icon).context("inventory icon unavailable")?;
                rows.push(serde_json::json!({"object":i.id,"graphic":i.icon,"icon":icon}));
            }
            views.push(rows);
        }
        icons.cancel();
        ensure!(icons.quiescent(), "inventory icons did not close");
        Some(
            serde_json::json!({"views":views,"worker_quiescent":icons.quiescent(),"cancelled_cache_empty":icons.cached()==0}),
        )
    } else {
        None
    };
    ensure!(
        transport.epoch == login_epoch + 1 && snapshot.game_connected,
        "GAME did not get a fresh owner epoch"
    );
    Ok(
        serde_json::json!({"schema":1,"status":"native_game_probe_completed","pointer_bits":usize::BITS,"game":snapshot,"transport":transport,"login_epoch":login_epoch.to_string(),"lobby_history":lobby_history,"room_history":room_history,"world_render":world_render,"combat_rounds":combat_rounds,"combat":if combat_mode {Some(combat.receipt())} else {None},"motion_before":motion_before,"navigation_trace":navigation_trace,"navigation_load_ms":route_load_ms,"navigation_frames":route_frames,"navigation_render_samples":route_samples,"movement_self_ack":false,"room_rejections":room_rejections,"returned_to_characters":returned,"item_actions":item_probe.rows,"inventory_names":inventory_names,"inventory_icons":inventory_icons,"inventory_return_cleared":inventory_return_cleared,"inventory_shutdown_cleared":c.snapshot().inventory.owner.is_none() && c.snapshot().inventory.items.is_empty(),"worker_quiescent":true,"shutdown":c.snapshot().phase,"sql_access":false,"playable_game":false}),
    )
}

#[cfg(test)]
#[path = "native_pending_join_departure_tests.rs"]
mod pending_join_departure_tests;
