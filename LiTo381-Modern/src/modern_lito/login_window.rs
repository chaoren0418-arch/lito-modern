//! Real same-EXE modern login/source/character UI. No native BIN, address hook,
//! scene-preview fallback, SQL connection or fabricated enter-world button.
//!
//! The controller owns async network workers. Every trait call must be bounded,
//! nonblocking and secret-redacted. WM_TIMER polls independently of focus; normal
//! close keeps pumping messages until shutdown has made the worker quiescent.

use super::application_protocol::{
    ChannelSummary, CharacterKey, NavigationPhase, NavigationReply, SelectionContext,
};
use super::character_data::{CharacterDataFailure, CharacterDataState, ItemKind};
use super::descriptions::DescriptionCatalog;
use super::equipment;
use super::matchmaking_protocol as matching;
use super::profile::ModernProfile;
use super::record_protocol as record;
use super::room_window::{FormOutcome, RoomForm};
use super::rooms;
use super::shop_window::{Outcome as ShopOutcome, ShopWindow};
use super::world_navigation::Navigation as WorldNavigation;
use super::world_protocol as world;
use super::world_runtime::{RuntimeStatus, WorldRuntime};
use super::world_window::{Action as WorldAction, CombatSurface, HitSurface, WorldWindow};
use super::{item_protocol as item, shop_protocol as shop};
use anyhow::{ensure, Result};
use std::cell::{Cell, RefCell};
use std::fmt::Write as _;
use std::path::Path;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc,
};
use std::thread::{self, JoinHandle};
use std::time::Instant;
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreateFontW, DeleteObject, DrawTextW, FillRect, GetSysColor, GetSysColorBrush, InvalidateRect,
            RedrawWindow, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS,
            DT_END_ELLIPSIS, DT_NOPREFIX, DT_SINGLELINE, DT_VCENTER,
            RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE,
            REDRAW_WINDOW_FLAGS, SRCCOPY,
            CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS, COLOR_BTNFACE, COLOR_HIGHLIGHT,
            COLOR_HIGHLIGHTTEXT, COLOR_WINDOW, COLOR_WINDOWTEXT, DEFAULT_CHARSET, HFONT,
            OUT_DEFAULT_PRECIS,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                DRAWITEMSTRUCT, EM_EMPTYUNDOBUFFER, EM_GETFIRSTVISIBLELINE, EM_GETSEL, EM_LINESCROLL,
                EM_SETLIMITTEXT, EM_SETSEL, ODS_SELECTED,
            },
            HiDpi::{
                AdjustWindowRectExForDpi, GetDpiForWindow, SetThreadDpiAwarenessContext,
                DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            },
            Input::KeyboardAndMouse::{EnableWindow, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};
use zeroize::{Zeroize, Zeroizing};

#[path = "login_canonical_world.rs"]
pub(crate) mod canonical_world;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoginPhase {
    Disconnected,
    Connecting,
    LoginReady,
    Authenticating,
    ChoosingSource,
    LoadingCharacters,
    CharactersReady,
    SelectingCharacter,
    ChoosingChannel,
    JoiningLobby,
    InLobby,
    LeavingLobby,
    ReturningCharacters,
    Disconnecting,
    Failed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceChoice {
    pub id: u32,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharacterRow {
    pub source_id: u32,
    pub object_id: u64,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoginSnapshot {
    pub phase: LoginPhase,
    /// Already redacted, bounded status. Never include account/password values.
    pub status: String,
    pub sources: Vec<SourceChoice>,
    pub selected_source: Option<u32>,
    pub characters: Vec<CharacterRow>,
    /// The generation of the actually displayed roster, never a row index.
    pub roster_epoch: u64,
    /// Only an acknowledged server snapshot; pending UI state does not invent membership.
    pub navigation: Option<NavigationReply>,
    /// Shared immutable source snapshot; never a live/effective combat status.
    pub character_data: CharacterDataState,
    pub equipment: equipment::State,
    /// An owned request is in flight; retained state is not an optimistic mutation.
    pub equipment_pending: bool,
    pub rooms: rooms::State,
    pub room_pending: bool,
    pub matchmaking: super::matchmaking_protocol::State,
    /// Negotiated v7 implementation only, not a cached account GM grant.
    pub gm_supported: bool,
    pub world: world::State,
    /// Followed whole-battle view; not a command ACK, LOAD proof or saved receipt.
    pub battle: super::battle_session::State,
    /// Complete canonical lifecycle/receipt state; never a legacy Frame.
    pub canonical: super::battle_control::State,
    pub battle_supported: bool,
    pub action_supported: bool,
    pub world_pending: bool,
    pub combat: Option<Arc<super::combat_protocol::Frame>>,
    /// Complete v18 NPC authority, including preflight; never inferred from player vitals.
    pub npc: Option<Arc<super::npc_protocol::Reply>>,
    pub combat_pending: bool,
    /// Connection capability only. A v15 descriptor decides this match's data.
    pub combat_supported: bool,
    pub skills: super::spell_protocol::SkillsState,
    pub spell_pending: bool,
    pub record: record::State,
    pub record_pending: bool,
    pub chat: super::chat_state::State,
    pub chat_pending: bool,
    pub direct_chat: super::direct_chat_state::State,
    pub direct_chat_pending: bool,
    pub shop: shop::State,
    pub shop_pending: bool,
    pub item_pending: bool,
    pub ammo_pending: bool,
}

impl LoginSnapshot {
    /// Both world implementations own navigation until a confirmed departure.
    /// A hidden/pending picture is not permission to use cached lobby controls.
    pub(crate) fn world_owns_lobby(&self) -> bool {
        self.world_pending
            || self.combat_pending
            || self.world.frame().is_some()
            || self.canonical.attached()
    }
    pub(crate) fn conversation_pending(&self) -> bool {
        self.chat_pending || self.direct_chat_pending
    }
    pub(crate) fn world_npc_enabled(&self) -> bool {
        self.world.frame().is_some_and(|f| {
            f.descriptor
                .is_some_and(|d| d.has(world::MatchDescriptor::NPC))
        })
    }
    pub(crate) fn world_combat_enabled(&self) -> bool {
        self.combat_supported
            && self
                .world
                .frame()
                .is_some_and(|frame| frame.combat_enabled(self.combat_supported))
    }
}

/// Intentionally not Debug/Clone: credentials must not leak into logging or be
/// retained by UI snapshots. The controller consumes this one ephemeral command.
pub enum LoginCommand {
    Canonical(super::battle_control::Command),
    /// Resolve the target only after the controller's final authority poll.
    PaintedTarget(super::battle_window::Intent),
    /// Player-window ownership survives the controller's final pre-send poll.
    PaintedLocal {
        intent: super::battle_window::Intent,
        command: Box<LoginCommand>,
    },
    Connect,
    Login {
        account: String,
        password: String,
    },
    SelectSource(u32),
    SelectCharacter(SelectionContext),
    EnterChannel {
        context: SelectionContext,
        channel_id: u8,
    },
    ReturnCharacters(SelectionContext),
    LeaveLobby(SelectionContext),
    Equipment(equipment::Request),
    Rooms(rooms::Request),
    Matchmaking(super::matchmaking_protocol::Request),
    GmBots {
        count: u16,
    },
    World(world::Request),
    CombatAttack {
        target_entity_id: u64,
    },
    NpcAttack {
        target: super::npc_protocol::Id,
        life: u64,
    },
    QuestAttack {
        target: super::quest_protocol::Id,
    },
    TimeAttackTowerAttack {
        target: super::time_attack_protocol::TowerId,
    },
    SiegeAttack {
        target: super::siege_protocol::Target,
    },
    SiegePortal {
        scope: super::siege_protocol::Scope,
    },
    QuestRevive {
        proof: super::quest_life::Confirmation,
    },
    ReadSkills,
    ReadRecord,
    DirectChat {
        token: super::direct_chat_state::Token,
        action: super::direct_chat_protocol::Action,
        target: Option<super::direct_chat_protocol::Address>,
        preferences: Option<super::direct_chat_protocol::Preferences>,
        text: String,
    },
    ChatSend {
        token: super::chat_state::Token,
        text: String,
    },
    FilteredChatSend {
        token: super::chat_state::Token,
        kind: super::filtered_chat_protocol::Kind,
        text: String,
    },
    Shop(shop::Request),
    UseItem {
        item_id: i64,
    },
    SelectAmmo {
        item_id: i64,
    },
    CastTeleport,
    CastLightArrow {
        target_entity_id: u64,
    },
    CastNpcLightArrow {
        target: super::npc_protocol::Id,
        life: u64,
    },
    CastBasicHeal {
        target_entity_id: u64,
    },
    CastFireStorm,
    CastDisintegrate(super::disintegrate_cast_protocol::Request),
    CastNpcDisintegrate(super::npc_disintegrate_protocol::Request),
    CastNpcFireStorm(super::npc_fire_storm_protocol::Request),
    Disconnect,
    Shutdown,
}

pub trait LoginController {
    /// Only reap workers already finished; no blocking network IO or join here.
    fn poll(&mut self);
    fn snapshot(&self) -> LoginSnapshot;
    /// Errors must be redacted; no credential values or raw server payloads.
    fn command(&mut self, command: LoginCommand) -> std::result::Result<(), String>;
    /// True only when all workers, queued credentials and owned sockets retired.
    fn is_quiescent(&self) -> bool;
}

const CLASS: PCWSTR = w!("ModernLiToLoginWindow");
const TIMER: usize = 1;
const ACCOUNT_UTF16_LIMIT: usize = 64;
const PASSWORD_UTF16_LIMIT: usize = 256;
const MAX_SOURCES: usize = super::application_protocol::MAX_SOURCES;
const MAX_CHARACTERS: usize = super::application_protocol::MAX_CHARACTERS;
const MIN_WIDTH_DIP: i32 = 640;
const MIN_HEIGHT_DIP: i32 = 840;
const STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_CLIPCHILDREN.0);
const EX_STYLE: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
// STATIC control styles live in the optional SystemServices bindings. Keep the
// two documented style bits local rather than add a Cargo feature for constants.
const STATIC_NO_PREFIX: u32 = 0x0000_0080;
const STATIC_PATH_ELLIPSIS: u32 = 0x0000_8000;
const STATIC_OWNER_DRAW: u32 = 0x0000_000D;

const PROFILE: usize = 0;
const ENDPOINT: usize = 1;
const ROOT: usize = 2;
const STATUS: usize = 3;
const ACCOUNT_LABEL: usize = 4;
const ACCOUNT: usize = 5;
const PASSWORD_LABEL: usize = 6;
const PASSWORD: usize = 7;
const CONNECT: usize = 8;
const LOGIN: usize = 9;
const DISCONNECT: usize = 10;
const SOURCE_LABEL: usize = 11;
const SOURCE: usize = 12;
const CHARACTER_LABEL: usize = 13;
const CHARACTERS: usize = 14;
const DETAILS: usize = 15;
const FOOTNOTE: usize = 16;
const EXIT: usize = 17;
const CHANNELS: usize = 18;
const SELECT_CHARACTER: usize = 19;
const ENTER_CHANNEL: usize = 20;
const RETURN_CHARACTERS: usize = 21;
const SHOW_STATS: usize = 22;
const SHOW_ITEMS: usize = 23;
const GEAR_ITEM: usize = 24;
const GEAR_EQUIP: usize = 25;
const GEAR_UNEQUIP: usize = 26;
const GEAR_READ: usize = 27;
const ROOM_LIST: usize = 28;
const ROOM_CREATE: usize = 29;
const ROOM_JOIN: usize = 30;
const ROOM_CONFIGURE: usize = 31;
const ROOM_LEAVE: usize = 32;
const ROOM_TEAM0: usize = 33;
const ROOM_TEAM1: usize = 34;
const ROOM_READY: usize = 35;
const ROOM_READ: usize = 36;
const SHOW_SKILLS: usize = 37;
const SKILLS_READ: usize = 38;
const CAST_TELEPORT: usize = 39;
const SHOW_RECORD: usize = 40;
const RECORD_READ: usize = 41;
const GM_LABEL: usize = 42;
const GM_COUNT: usize = 43;
const GM_SEND: usize = 44;
const SHOP_OPEN: usize = 45;
const SETTINGS: usize = 46;
const CHAT: usize = 47;
const MATCH_START: usize = 48;
const MATCH_CANCEL: usize = 49;
const VIP_STATUS_STRIP: usize = 50;
const VIP_ITEMS: usize = 51;
const CONTROL_COUNT: usize = 52;
// 512 complete rows: each may contain a 256-byte source token AND a 512-byte
// expanded shared name, plus all signed-64-bit fields. Keep the proven row bound
// rather than silently truncating records to fit the old token-only budget.
const MAX_DETAILS_UTF16: usize = 1024 * 1024;
const FIRST_ID: usize = 100;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BoxRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn scale(value: i32, dpi: u32) -> i32 {
    // Win32 dimensions are signed. Defend layout arithmetic independently of
    // externally delivered messages; desktop/window position is never an input.
    ((i64::from(value) * i64::from(dpi.clamp(48, 768)) + 48) / 96) as i32
}

fn layout(width: i32, height: i32, dpi: u32) -> [BoxRect; CONTROL_COUNT] {
    let width = width.max(scale(MIN_WIDTH_DIP, dpi));
    let height = height.max(scale(MIN_HEIGHT_DIP, dpi));
    let margin = scale(16, dpi);
    let gap = scale(8, dpi);
    let full = width - margin * 2;
    let mut boxes = [BoxRect::default(); CONTROL_COUNT];
    let mut row = |index, y, row_height| {
        boxes[index] = BoxRect {
            x: margin,
            y: scale(y, dpi),
            width: full,
            height: scale(row_height, dpi),
        };
    };
    row(PROFILE, 16, 26);
    row(ENDPOINT, 46, 24);
    row(ROOT, 72, 24);
    row(STATUS, 104, 54);
    row(SOURCE_LABEL, 286, 24);
    // A dropdown's height includes its popup list, not an overlapping edit.
    row(SOURCE, 312, 220);
    row(CHARACTER_LABEL, 352, 26);
    let label_width = scale(72, dpi);
    for (label, edit, y) in [
        (ACCOUNT_LABEL, ACCOUNT, 168),
        (PASSWORD_LABEL, PASSWORD, 208),
    ] {
        boxes[label] = BoxRect {
            x: margin,
            y: scale(y + 4, dpi),
            width: label_width,
            height: scale(26, dpi),
        };
        boxes[edit] = BoxRect {
            x: margin + label_width + gap,
            y: scale(y, dpi),
            width: full - label_width - gap,
            height: scale(30, dpi),
        };
    }
    let button_width = (full - gap * 2) / 3;
    for (offset, index) in [CONNECT, LOGIN, DISCONNECT].into_iter().enumerate() {
        boxes[index] = BoxRect {
            x: margin + offset as i32 * (button_width + gap),
            y: scale(248, dpi),
            width: button_width,
            height: scale(30, dpi),
        };
    }
    let exit_width = scale(104, dpi);
    boxes[EXIT] = BoxRect {
        x: width - margin - exit_width,
        y: height - margin - scale(32, dpi),
        width: exit_width,
        height: scale(32, dpi),
    };
    boxes[FOOTNOTE] = BoxRect {
        x: margin,
        y: boxes[EXIT].y,
        width: full - exit_width - gap,
        height: scale(40, dpi),
    };
    boxes[SETTINGS] = BoxRect {
        x: boxes[EXIT].x - gap - scale(136, dpi),
        y: boxes[EXIT].y,
        width: scale(136, dpi),
        height: scale(32, dpi),
    };
    boxes[CHAT] = BoxRect {
        x: boxes[SETTINGS].x - gap - scale(104, dpi),
        y: boxes[EXIT].y,
        width: scale(104, dpi),
        height: scale(32, dpi),
    };
    boxes[FOOTNOTE].width = boxes[CHAT].x - margin - gap;
    boxes[DETAILS] = BoxRect {
        x: margin,
        y: boxes[EXIT].y - scale(248, dpi),
        width: full,
        height: scale(240, dpi),
    };
    boxes[CHARACTERS] = BoxRect {
        x: margin,
        y: scale(382, dpi),
        width: full,
        height: boxes[DETAILS].y - scale(470, dpi),
    };
    boxes[CHANNELS] = boxes[CHARACTERS]; // Mutually exclusive standard listboxes.
    boxes[ROOM_LIST] = boxes[CHARACTERS];
    for (offset, index) in [
        ROOM_CREATE,
        ROOM_JOIN,
        ROOM_CONFIGURE,
        MATCH_START,
        ROOM_LEAVE,
        ROOM_TEAM0,
        ROOM_TEAM1,
        MATCH_CANCEL,
    ]
    .into_iter()
    .enumerate()
    {
        boxes[index] = BoxRect {
            x: margin + (offset % 4) as i32 * ((full - 3 * gap) / 4 + gap),
            y: scale(168 + (offset / 4) as i32 * 40, dpi),
            width: (full - 3 * gap) / 4,
            height: scale(30, dpi),
        };
    }
    boxes[ROOM_READY] = boxes[CONNECT];
    boxes[ROOM_READ] = boxes[LOGIN];
    // Mutually exclusive with the source controls after entering the room UI.
    boxes[GM_LABEL] = boxes[SOURCE_LABEL];
    boxes[GM_COUNT] = BoxRect {
        x: margin,
        y: scale(312, dpi),
        width: button_width,
        height: scale(30, dpi),
    };
    boxes[GM_SEND] = BoxRect {
        x: margin + button_width + gap,
        y: scale(312, dpi),
        width: button_width,
        height: scale(30, dpi),
    };
    boxes[SHOP_OPEN] = BoxRect {
        x: margin + 2 * (button_width + gap),
        y: scale(312, dpi),
        width: button_width,
        height: scale(30, dpi),
    };
    for (offset, index) in [SELECT_CHARACTER, ENTER_CHANNEL, RETURN_CHARACTERS]
        .into_iter()
        .enumerate()
    {
        boxes[index] = BoxRect {
            x: margin + offset as i32 * (button_width + gap),
            y: boxes[DETAILS].y - scale(80, dpi),
            width: button_width,
            height: scale(32, dpi),
        };
    }
    let detail_button_width = (full - gap * 3) / 4;
    for (offset, index) in [SHOW_STATS, SHOW_ITEMS, GEAR_EQUIP, GEAR_UNEQUIP]
        .into_iter()
        .enumerate()
    {
        boxes[index] = BoxRect {
            x: margin + offset as i32 * (detail_button_width + gap),
            y: boxes[DETAILS].y - scale(40, dpi),
            width: detail_button_width,
            height: scale(32, dpi),
        };
    }
    boxes[GEAR_ITEM] = BoxRect {
        x: margin,
        y: boxes[DETAILS].y,
        width: full - button_width - gap,
        height: scale(220, dpi),
    };
    boxes[GEAR_READ] = BoxRect {
        x: margin + full - button_width,
        y: boxes[DETAILS].y,
        width: button_width,
        height: scale(32, dpi),
    };
    boxes[DETAILS].y += scale(40, dpi);
    boxes[DETAILS].height -= scale(40, dpi);
    boxes[VIP_STATUS_STRIP] = BoxRect {
        x: boxes[DETAILS].x,
        y: boxes[DETAILS].y,
        width: boxes[DETAILS].width,
        height: scale(32, dpi),
    };
    boxes[VIP_ITEMS] = boxes[DETAILS];
    let utility_width = (full - gap * 4) / 5;
    for (offset, index) in [
        SHOW_SKILLS,
        SKILLS_READ,
        CAST_TELEPORT,
        SHOW_RECORD,
        RECORD_READ,
    ]
    .into_iter()
    .enumerate()
    {
        boxes[index] = BoxRect {
            x: margin + offset as i32 * (utility_width + gap),
            y: boxes[DETAILS].y,
            width: utility_width,
            height: scale(32, dpi),
        };
    }
    boxes[DETAILS].y += scale(40, dpi);
    boxes[DETAILS].height -= scale(40, dpi);
    boxes
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Enabled {
    connect: bool,
    credentials: bool,
    source: bool,
    disconnect: bool,
}

fn enabled(phase: LoginPhase, closing: bool) -> Enabled {
    Enabled {
        connect: !closing && matches!(phase, LoginPhase::Disconnected | LoginPhase::Failed),
        credentials: !closing && phase == LoginPhase::LoginReady,
        // A new choice during an in-flight source read is real: the controller
        // coalesces it and discards the old epoch, instead of freezing the list.
        source: !closing
            && matches!(
                phase,
                LoginPhase::ChoosingSource
                    | LoginPhase::LoadingCharacters
                    | LoginPhase::CharactersReady
            ),
        disconnect: !closing
            && !matches!(phase, LoginPhase::Disconnected | LoginPhase::Disconnecting),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ListChanges {
    sources: bool,
    characters: bool,
}

fn list_changes(previous: Option<&LoginSnapshot>, next: &LoginSnapshot) -> ListChanges {
    ListChanges {
        sources: previous.is_none_or(|old| old.sources != next.sources),
        characters: previous.is_none_or(|old| old.characters != next.characters),
    }
}

/// Native combo selection changes before CBN_SELCHANGE reaches our controller.
/// A rejected command leaves the authoritative snapshot unchanged, so comparing
/// old/new snapshots alone cannot restore the UI. Reconcile only the current
/// index, never rebuild the sources or reset the character highlight.
fn source_selection_to_apply(actual: isize, wanted: Option<usize>) -> Option<usize> {
    let current = usize::try_from(actual).ok();
    if current == wanted {
        None
    } else {
        Some(wanted.unwrap_or(usize::MAX)) // CB_SETCURSEL(-1) clears selection.
    }
}

fn safe_label(value: &str, max_bytes: usize) -> bool {
    !value.is_empty() && value.len() <= max_bytes && !value.chars().any(char::is_control)
}

/// Actual window and headless interop use the same resource submission gate.
/// NPC encounters cannot enter the ordinary combat loader or ACK early.
pub(crate) fn update_world_runtime(
    runtime: &mut WorldRuntime,
    snapshot: &LoginSnapshot,
    entity: u64,
) -> Result<()> {
    let frame = snapshot
        .world
        .frame()
        .ok_or_else(|| anyhow::anyhow!("world absent"))?;
    if snapshot.world_npc_enabled() {
        let reply = snapshot
            .npc
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NPC preflight absent"))?;
        let npc = reply
            .frame
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("NPC snapshot absent"))?;
        ensure!(
            npc.combat.world == *frame
                && snapshot.combat.as_deref() == Some(&npc.combat)
                && snapshot
                    .navigation
                    .as_ref()
                    .is_some_and(|n| n.context == reply.echo.context
                        && n.snapshot.channel_id == Some(reply.echo.channel_id)),
            "NPC authority differs from selected world"
        );
        let plan = Arc::new(super::npc_presentation::Plan::from_reply(reply)?);
        runtime.update_npc(reply.clone(), entity, plan)
    } else {
        runtime.update_combat(Arc::new(frame.clone()), entity, snapshot.combat.clone())
    }
}

/// ACK requires a successfully rendered resource generation, not merely a
/// world/combat packet. Used by the real window and true-service integration.
pub(crate) fn loaded_request(
    snapshot: &LoginSnapshot,
    rendered: &super::world_runtime::RenderedWorld,
) -> Option<world::Request> {
    let frame = snapshot.world.frame()?;
    let nav = snapshot.navigation.as_ref()?;
    let actor = frame
        .actors
        .iter()
        .find(|a| a.matches_character(nav.context.key))?;
    if frame.phase != world::Phase::Loading
        || actor.loaded
        || !rendered.ready_to_ack
        || rendered.match_id != frame.match_id
        || rendered.generation != frame.generation
        || rendered.selected_entity_id != actor.entity_id
        || rendered.manifest_digest != frame.manifest.digest
        || rendered.revision > frame.revision
        || rendered.width != 800
        || rendered.height != 600
    {
        return None;
    }
    if snapshot.world_npc_enabled() {
        let current = snapshot.npc.as_ref()?;
        let drawn = rendered.npc.as_ref()?;
        let current_frame = current.frame.as_ref()?;
        let drawn_frame = drawn.frame.as_ref()?;
        if current.echo.context != nav.context
            || drawn.echo.context != nav.context
            || Some(current.echo.channel_id) != nav.snapshot.channel_id
            || drawn.echo.channel_id != current.echo.channel_id
            || current_frame.combat.world != *frame
            || snapshot.combat.as_deref() != Some(&current_frame.combat)
            || drawn_frame.combat.world.revision != rendered.revision
            || drawn_frame.combat.world.phase != world::Phase::Loading
            || drawn_frame.combat.world.manifest.digest != rendered.manifest_digest
            || drawn_frame.resources.is_none()
            || drawn_frame.resources != current_frame.resources
            || drawn_frame.scope != current_frame.scope
            || rendered.combat.as_deref() != Some(&drawn_frame.combat)
        {
            return None;
        }
    }
    let mut request = world_request(snapshot, world::Action::Loaded)?;
    request.manifest_digest = rendered.manifest_digest;
    Some(request)
}

pub(crate) fn validate_snapshot(snapshot: &LoginSnapshot) -> Result<()> {
    // A pending return still carries the last acknowledged Lobby snapshot.
    // It may be displayed read-only; it is not a new chat admission.
    let chat_presentation = matches!(
        snapshot.phase,
        LoginPhase::InLobby | LoginPhase::LeavingLobby | LoginPhase::ReturningCharacters
    );
    if let Some(scope) = snapshot.chat.scope() {
        let current = super::chat_state::Scope::current(
            snapshot.navigation.as_ref(),
            match &snapshot.rooms {
                rooms::State::Ready(reply) => Some(reply.as_ref()),
                _ => None,
            },
        );
        ensure!(
            chat_presentation && current == Some(scope),
            "chat belongs to another audience"
        );
    }
    ensure!(
        !snapshot.chat_pending
            || snapshot.phase == LoginPhase::InLobby && snapshot.chat.token().is_some(),
        "unconfirmed chat audience cannot send"
    );
    if let Some(e) = snapshot.direct_chat.enrollment() {
        ensure!(
            chat_presentation
                && snapshot
                    .navigation
                    .as_ref()
                    .is_some_and(|n| n.snapshot.phase == NavigationPhase::Lobby
                        && n.context == e.context
                        && n.snapshot.channel_id == Some(e.channel_id)),
            "private conversation belongs to another enrollment"
        );
    }
    if let Some(t) = snapshot.direct_chat.token() {
        ensure!(
            snapshot
                .chat
                .token()
                .is_some_and(|p| p.scope.context == t.enrollment.context
                    && p.scope.channel_id == t.enrollment.channel_id
                    && p.scope.room_id == t.room_id
                    && p.membership == t.membership),
            "private admission is stale"
        );
    }
    ensure!(
        !snapshot.direct_chat_pending
            || snapshot.phase == LoginPhase::InLobby && snapshot.direct_chat.token().is_some(),
        "unconfirmed private audience cannot mutate"
    );
    if let Some(reply) = &snapshot.npc {
        reply.validate()?;
        ensure!(
            snapshot.world_npc_enabled()
                && snapshot
                    .navigation
                    .as_ref()
                    .is_some_and(|n| n.context == reply.echo.context
                        && n.snapshot.channel_id == Some(reply.echo.channel_id)),
            "NPC selection mismatch"
        );
        if let Some(frame) = &reply.frame {
            ensure!(
                frame.resources.is_some()
                    && snapshot.world.frame() == Some(&frame.combat.world)
                    && snapshot.combat.as_deref() == Some(&frame.combat),
                "NPC world/preflight mismatch"
            );
        }
    }
    let pending = [
        snapshot.equipment_pending,
        snapshot.room_pending,
        snapshot.world_pending,
        snapshot.combat_pending,
        snapshot.spell_pending,
        snapshot.record_pending,
        snapshot.chat_pending,
        snapshot.direct_chat_pending,
        snapshot.shop_pending,
        snapshot.item_pending,
        snapshot.ammo_pending,
    ];
    ensure!(
        pending.iter().filter(|p| **p).count() <= 1,
        "UI mutations must remain single-flight"
    );
    if let Some(reply) = snapshot.shop.reply() {
        reply.validate()?;
        ensure!(
            snapshot
                .navigation
                .as_ref()
                .is_some_and(|nav| nav.context == reply.echo.context
                    && nav.snapshot.channel_id == Some(reply.echo.channel_id)),
            "shop belongs to another selection/channel"
        );
        ensure!(
            snapshot.world.frame().is_none(),
            "shop stale across world preparation"
        );
    }
    if let shop::State::Unavailable {
        context,
        channel_id,
        status,
    } = &snapshot.shop
    {
        ensure!(
            snapshot.navigation.as_ref().is_some_and(
                |nav| nav.context == *context && nav.snapshot.channel_id == Some(*channel_id)
            ) && matches!(
                status,
                shop::Status::Busy | shop::Status::Unavailable | shop::Status::Frozen
            ),
            "invalid failed shop state"
        );
    }
    ensure!(
        !snapshot.shop_pending
            || (snapshot.phase == LoginPhase::InLobby
                && snapshot.world.frame().is_none()
                && snapshot.navigation.is_some()),
        "invalid shop pending state"
    );
    ensure!(
        !snapshot.item_pending
            || (snapshot.phase == LoginPhase::InLobby
                && snapshot.world.frame().is_some()
                && snapshot
                    .combat
                    .as_ref()
                    .is_some_and(|c| !c.items.is_empty())),
        "invalid item pending state"
    );
    ensure!(!snapshot.ammo_pending || snapshot.phase==LoginPhase::InLobby
        && snapshot.combat.as_ref().is_some_and(|c|!c.ammo.is_empty() && snapshot.world.frame()==Some(&c.world)),
        "invalid ammunition pending state");
    if let Some(context) = snapshot.record.context() {
        ensure!(
            snapshot
                .navigation
                .as_ref()
                .is_some_and(|nav| nav.context == context),
            "record belongs to a different selection"
        );
    }
    match &snapshot.record {
        record::State::Ready {
            confirmed,
            refresh_failure,
        } => {
            confirmed.validate()?;
            ensure!(
                confirmed.status == record::Status::Ok
                    && refresh_failure.is_none_or(|status| status != record::Status::Ok),
                "invalid confirmed record state"
            );
        }
        record::State::Unavailable { status, .. } => ensure!(
            *status != record::Status::Ok,
            "invalid unavailable record state"
        ),
        _ => {}
    }
    ensure!(
        !snapshot.record_pending
            || snapshot.navigation.is_some()
                && !snapshot.spell_pending
                && !snapshot.conversation_pending()
                && !snapshot.world_pending
                && !snapshot.combat_pending
                && !snapshot.room_pending
                && !snapshot.equipment_pending
                && snapshot.world.frame().is_none(),
        "invalid record pending state"
    );
    if let super::spell_protocol::SkillsState::Ready(skills) = &snapshot.skills {
        skills.validate()?;
        ensure!(
            snapshot
                .navigation
                .as_ref()
                .is_some_and(|nav| nav.context == skills.context),
            "skills belong to a different selection"
        );
    }
    ensure!(
        !snapshot.spell_pending
            || snapshot.navigation.is_some()
                && !snapshot.world_pending
                && !snapshot.combat_pending
                && !snapshot.room_pending
                && !snapshot.equipment_pending,
        "invalid spell pending state"
    );
    ensure!(
        snapshot.status.len() <= 2048 && !snapshot.status.contains('\0'),
        "invalid UI status"
    );
    ensure!(
        snapshot.sources.len() <= MAX_SOURCES,
        "too many source choices"
    );
    ensure!(
        snapshot.characters.len() <= MAX_CHARACTERS,
        "too many character rows"
    );
    let mut sources = std::collections::BTreeSet::new();
    for source in &snapshot.sources {
        ensure!(
            safe_label(&source.label, 512)
                && source.id > 0
                && source.id <= i32::MAX as u32
                && sources.insert(source.id),
            "invalid or duplicate source choice"
        );
    }
    if let Some(selected) = snapshot.selected_source {
        ensure!(
            sources.contains(&selected),
            "selected source is missing from choices"
        );
    }
    let mut characters = std::collections::BTreeSet::new();
    for row in &snapshot.characters {
        ensure!(
            safe_label(&row.label, 1024)
                && row.source_id > 0
                && row.source_id <= i32::MAX as u32
                && row.object_id > 0
                && row.object_id <= i64::MAX as u64
                && snapshot.selected_source == Some(row.source_id)
                && characters.insert((row.source_id, row.object_id)),
            "invalid, stale-source or duplicate character row"
        );
    }
    if matches!(
        snapshot.phase,
        LoginPhase::CharactersReady | LoginPhase::SelectingCharacter
    ) {
        ensure!(
            snapshot.roster_epoch != 0
                && snapshot.selected_source.is_some()
                && snapshot.navigation.is_none(),
            "character selection requires a current roster epoch"
        );
    } else {
        ensure!(
            snapshot.roster_epoch == 0 && snapshot.characters.is_empty(),
            "stale roster outside character selection"
        );
    }
    if let Some(reply) = &snapshot.navigation {
        ensure!(
            matches!(
                snapshot.phase,
                LoginPhase::ChoosingChannel
                    | LoginPhase::JoiningLobby
                    | LoginPhase::InLobby
                    | LoginPhase::LeavingLobby
                    | LoginPhase::ReturningCharacters
            ),
            "navigation snapshot in an unrelated UI phase"
        );
        let navigation = &reply.snapshot;
        let selected = navigation
            .selected
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing selected character"))?;
        ensure!(
            reply.context.epoch != 0
                && selected.key == reply.context.key
                && snapshot.selected_source == Some(selected.key.source_id)
                && selected.key.object_id > 0
                && selected.key.object_id <= i64::MAX as u64
                && safe_label(&selected.name, 256)
                && selected.level > 0
                && selected.sex <= 1
                && navigation.channels.len() == 4,
            "invalid navigation identity or channel count"
        );
        for (index, channel) in navigation.channels.iter().enumerate() {
            ensure!(
                channel.id as usize == index
                    && safe_label(&channel.name, 256)
                    && channel.capacity > 0
                    && channel.capacity <= 65535
                    && channel.population <= channel.capacity
                    && channel.minimum_level > 0
                    && channel.maximum_level >= channel.minimum_level,
                "invalid server channel snapshot"
            );
        }
        ensure!(
            match snapshot.phase {
                LoginPhase::ChoosingChannel | LoginPhase::JoiningLobby =>
                    navigation.phase == NavigationPhase::Channels && navigation.channel_id.is_none(),
                LoginPhase::InLobby | LoginPhase::LeavingLobby =>
                    navigation.phase == NavigationPhase::Lobby
                        && navigation.channel_id.is_some_and(|id| id < 4),
                LoginPhase::ReturningCharacters => match navigation.phase {
                    NavigationPhase::Channels => navigation.channel_id.is_none(),
                    NavigationPhase::Lobby => navigation.channel_id.is_some_and(|id| id < 4),
                    NavigationPhase::Characters => false,
                },
                _ => false,
            },
            "UI phase disagrees with server-confirmed navigation"
        );
    } else {
        ensure!(
            !matches!(
                snapshot.phase,
                LoginPhase::ChoosingChannel
                    | LoginPhase::JoiningLobby
                    | LoginPhase::InLobby
                    | LoginPhase::LeavingLobby
                    | LoginPhase::ReturningCharacters
            ),
            "channel and lobby UI require server confirmation"
        );
    }
    match &snapshot.character_data {
        CharacterDataState::NotRequested => {}
        CharacterDataState::Unsupported | CharacterDataState::Unavailable(_) => {
            ensure!(
                snapshot.navigation.is_some(),
                "character data state outside selected identity"
            );
        }
        CharacterDataState::Ready(data) => {
            ensure!(
                snapshot.navigation.as_ref().map(|reply| reply.context) == Some(data.context),
                "stale source character data"
            );
            data.validate()?;
        }
    }
    if let equipment::State::Ready(gear) = &snapshot.equipment {
        gear.validate()?;
        ensure!(
            snapshot.navigation.as_ref().map(|reply| reply.context) == Some(gear.source.context),
            "stale equipment context"
        );
        ensure!(
            matches!(&snapshot.character_data, CharacterDataState::Ready(data) if data.as_ref() == gear.source.as_ref()),
            "equipment source does not match displayed source data"
        );
    } else if !matches!(snapshot.equipment, equipment::State::NotRequested) {
        ensure!(
            snapshot.navigation.is_some(),
            "equipment outside selected identity"
        );
    }
    if snapshot.equipment_pending {
        ensure!(
            matches!(
                snapshot.phase,
                LoginPhase::ChoosingChannel | LoginPhase::InLobby
            ) && !matches!(
                snapshot.equipment,
                equipment::State::NotRequested | equipment::State::Unsupported
            ),
            "equipment pending outside its selected page"
        );
    }
    if let rooms::State::Ready(reply) = &snapshot.rooms {
        reply.validate()?;
        ensure!(
            snapshot
                .navigation
                .as_ref()
                .is_some_and(|nav| nav.context == reply.common.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(reply.common.channel_id)),
            "stale room context"
        );
    }
    if let matching::State::Ready(reply) = &snapshot.matchmaking {
        reply.validate()?;
        // Navigation intent keeps the last confirmed queue until its own ACK.
        ensure!(
            matches!(
                snapshot.phase,
                LoginPhase::InLobby | LoginPhase::LeavingLobby | LoginPhase::ReturningCharacters
            ) && snapshot
                .navigation
                .as_ref()
                .is_some_and(|nav| nav.context == reply.echo.context
                    && nav.snapshot.phase == NavigationPhase::Lobby
                    && nav.snapshot.channel_id == Some(reply.echo.channel_id)),
            "stale matchmaking context"
        );
    }
    ensure!(
        !snapshot.room_pending
            || (snapshot.phase == LoginPhase::InLobby
                && !snapshot.equipment_pending
                && matches!(snapshot.rooms, rooms::State::Ready(_))),
        "invalid room pending state"
    );
    if let world::State::Ready(reply) = &snapshot.world {
        reply.validate()?;
        ensure!(
            (snapshot.phase == LoginPhase::InLobby
                || reply.frame.is_none()
                    && matches!(
                        snapshot.phase,
                        LoginPhase::LeavingLobby | LoginPhase::ReturningCharacters
                    ))
                && snapshot
                    .navigation
                    .as_ref()
                    .is_some_and(|nav| nav.context == reply.echo.context
                        && nav.snapshot.channel_id == Some(reply.echo.channel_id)),
            "stale world context"
        );
    }
    if let Some(combat) = &snapshot.combat {
        combat.validate()?;
        ensure!(
            snapshot.world_combat_enabled() && snapshot.world.frame() == Some(&combat.world),
            "combat UI requires the exact confirmed world revision"
        );
    }
    ensure!(
        !snapshot.combat_pending
            || snapshot.world_combat_enabled()
                && snapshot.phase == LoginPhase::InLobby
                && snapshot.world.frame().is_some()
                && !snapshot.world_pending
                && !snapshot.room_pending
                && !snapshot.equipment_pending,
        "invalid combat pending state"
    );
    ensure!(
        !snapshot.world_pending
            || (snapshot.phase == LoginPhase::InLobby
                && !snapshot.room_pending
                && !snapshot.equipment_pending
                && (matches!(snapshot.world, world::State::Ready(_))
                    || snapshot.canonical.supported()
                        && matches!(snapshot.world, world::State::NotRequested)
                        && snapshot.combat.is_none()
                        && snapshot.npc.is_none()
                        && super::battle_control::scope_for(snapshot).is_some()
                        // Departure hides the old attachment while its ACK is pending.
                        && snapshot.canonical.snapshot().is_none_or(
                            |s| Some(s.scope()) == super::battle_control::scope_for(snapshot)
                        ))),
        "invalid world pending state"
    );
    Ok(())
}

fn character_context(snapshot: &LoginSnapshot, index: isize) -> Option<SelectionContext> {
    if snapshot.phase != LoginPhase::CharactersReady || snapshot.roster_epoch == 0 {
        return None;
    }
    let row = snapshot.characters.get(usize::try_from(index).ok()?)?;
    if snapshot.selected_source != Some(row.source_id)
        || row.source_id == 0
        || row.source_id > i32::MAX as u32
        || row.object_id == 0
        || row.object_id > i64::MAX as u64
    {
        return None;
    }
    Some(SelectionContext {
        key: CharacterKey {
            source_id: row.source_id,
            object_id: row.object_id,
        },
        epoch: snapshot.roster_epoch,
    })
}

fn channel_command(snapshot: &LoginSnapshot, index: isize) -> Option<LoginCommand> {
    if snapshot.phase != LoginPhase::ChoosingChannel
        || snapshot.equipment_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
    {
        return None;
    }
    let reply = snapshot.navigation.as_ref()?;
    let channel = reply.snapshot.channels.get(usize::try_from(index).ok()?)?;
    // Recommended tier and cached occupancy are display data, not authorization.
    Some(LoginCommand::EnterChannel {
        context: reply.context,
        channel_id: channel.id,
    })
}

/// The one visible Back control follows the acknowledged server page, not a
/// shortcut from the lobby that silently releases the selected character.
fn return_command(snapshot: &LoginSnapshot) -> Option<LoginCommand> {
    if snapshot.world_owns_lobby()
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
        || snapshot.equipment_pending
        || snapshot.room_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.rooms.joined().is_some()
    {
        return None;
    }
    let reply = snapshot.navigation.as_ref()?;
    match (snapshot.phase, reply.snapshot.phase) {
        (LoginPhase::InLobby, NavigationPhase::Lobby) => {
            Some(LoginCommand::LeaveLobby(reply.context))
        }
        (LoginPhase::ChoosingChannel, NavigationPhase::Channels) => {
            Some(LoginCommand::ReturnCharacters(reply.context))
        }
        _ => None,
    }
}

fn return_label(phase: LoginPhase) -> &'static str {
    if matches!(phase, LoginPhase::InLobby | LoginPhase::LeavingLobby) {
        "返回頻道"
    } else {
        "返回選角"
    }
}

fn channel_label(channel: &ChannelSummary) -> String {
    let tier = if channel.maximum_level == i64::MAX {
        format!("{}+", channel.minimum_level)
    } else {
        format!("{}–{}", channel.minimum_level, channel.maximum_level)
    };
    format!(
        "{}  ·  人數 {}/{}  ·  推薦等級 {tier}",
        channel.name, channel.population, channel.capacity
    )
}

fn navigation_details(snapshot: &LoginSnapshot) -> Option<String> {
    let reply = snapshot.navigation.as_ref()?;
    let selected = reply.snapshot.selected.as_ref()?;
    let source = snapshot
        .sources
        .iter()
        .find(|source| source.id == selected.key.source_id)?;
    let destination = reply.snapshot.channel_id.and_then(|id| {
        reply
            .snapshot
            .channels
            .iter()
            .find(|channel| channel.id == id)
    });
    Some(if let Some(channel) = destination {
        format!(
            "{}({})  ·  {} 大廳  ·  人數 {}/{}\r\n來源 ID：{}  ·  角色識別碼：{}",
            selected.name,
            source.label,
            channel.name,
            channel.population,
            channel.capacity,
            selected.key.source_id,
            selected.key.object_id
        )
    } else {
        format!(
            "{}({})  ·  Lv.{}\r\n請選擇伺服器提供的頻道；返回選角會由伺服器解除選角狀態。",
            selected.name, source.label, selected.level
        )
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DetailView {
    Stats,
    Items,
    Skills,
    Record,
}
fn detail_mnemonic_target(page: super::world_window::DetailsPage) -> (usize, DetailView) {
    use super::world_window::DetailsPage;
    match page {
        DetailsPage::Character => (SHOW_STATS, DetailView::Stats),
        DetailsPage::Inventory => (SHOW_ITEMS, DetailView::Items),
        DetailsPage::Skills => (SHOW_SKILLS, DetailView::Skills),
    }
}
fn canonical_detail_page(
    snapshot: &LoginSnapshot,
    view: DetailView,
) -> Option<super::world_window::DetailsPage> {
    use super::world_window::DetailsPage;
    if !snapshot.canonical.supported()
        || !snapshot.world_owns_lobby()
        || snapshot.canonical.snapshot().is_some_and(|s| s.full_vip())
    {
        return None;
    }
    match view {
        DetailView::Stats => Some(DetailsPage::Character),
        DetailView::Items => Some(DetailsPage::Inventory),
        DetailView::Skills => Some(DetailsPage::Skills),
        // Central saved records remain explicitly distinct from live combat.
        DetailView::Record => None,
    }
}
fn vip_detail_observation(snapshot: &LoginSnapshot) -> Option<&super::battle_follow::Observation> {
    if !snapshot.world_owns_lobby() {
        return None;
    }
    let scope = super::battle_control::scope_for(snapshot)?;
    let state = &snapshot.canonical;
    if !state.supported() || state.snapshot()?.scope() != scope {
        return None;
    }
    state.vip_observation()
}
fn full_vip_details_route(snapshot: &LoginSnapshot) -> bool {
    snapshot.world_owns_lobby()
        && snapshot.canonical.snapshot().is_some_and(|s| s.full_vip())
}
fn vip_detail_identity(snapshot: &LoginSnapshot) -> Option<(SelectionContext, u8, [i64; 3], i64, i64, i64)> {
    let (context, channel) = super::battle_control::scope_for(snapshot)?;
    let data = vip_detail_observation(snapshot)?.data();
    let owner = &data.vip.as_ref()?.owner;
    Some((context, channel, [data.world.identity.room,
        data.world.identity.match_id, data.world.identity.generation],
        owner.entity, owner.life, owner.versions.lifecycle))
}
fn can_show_player_detail(
    snapshot: &LoginSnapshot,
    view: DetailView,
    world_available: bool,
) -> bool {
    if canonical_detail_page(snapshot, view).is_some() {
        return world_available;
    }
    if snapshot.world_owns_lobby()
        && snapshot.canonical.snapshot().is_some_and(|s| s.full_vip())
    {
        return matches!(view, DetailView::Stats | DetailView::Items | DetailView::Skills)
            && vip_detail_observation(snapshot).is_some();
    }
    match view {
        DetailView::Stats | DetailView::Items => {
            matches!(snapshot.character_data, CharacterDataState::Ready(_))
        }
        DetailView::Skills | DetailView::Record => snapshot.navigation.is_some(),
    }
}
unsafe fn show_player_detail(hwnd: HWND, state: &WindowState, detail: DetailView) {
    let snapshot = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        if ui.closing {
            return;
        }
        ui.controller.snapshot()
    };
    let world_hwnd = state
        .world_window
        .borrow()
        .as_ref()
        .and_then(WorldWindow::window_handle);
    if let Some(page) = canonical_detail_page(&snapshot, detail) {
        if let Some(world) = world_hwnd {
            // Clearing this cache is required even when reopening the same
            // page: show_details discards old text and equipment proofs.
            state.world_details_cache.borrow_mut().take();
            if WorldWindow::show_details_window(world, page).is_err() {
                local_error(hwnd, state, "目前戰鬥視窗已關閉；未改開來源保存資料頁。");
            }
        } else {
            local_error(
                hwnd,
                state,
                "新版戰鬥視窗尚未就緒；不以來源保存資料代替本場資料。",
            );
        }
    } else if can_show_player_detail(&snapshot, detail, world_hwnd.is_some()) {
        state.detail_view.set(detail);
    }
    refresh(hwnd, state);
}
pub(crate) fn can_read_record(snapshot: &LoginSnapshot) -> bool {
    matches!(
        snapshot.phase,
        LoginPhase::ChoosingChannel | LoginPhase::InLobby
    ) && snapshot.navigation.is_some()
        && !snapshot.world_owns_lobby()
        && !snapshot.record_pending
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.room_pending
        && !snapshot.equipment_pending
        && !matches!(snapshot.record, record::State::Unsupported)
}
fn record_text(snapshot: &LoginSnapshot) -> String {
    let mut text = String::from("中央累計戰績（只讀取已提交保存值，非本場推算）\r\n");
    if let Some(nav) = &snapshot.navigation {
        let _ = write!(
            text,
            "來源 ID：{} · 角色識別碼：{}\r\n",
            nav.context.key.source_id, nav.context.key.object_id
        );
    }
    match &snapshot.record {
        record::State::NotRequested => text.push_str("尚未讀取；不代表零勝敗。\r\n"),
        record::State::Unsupported => text.push_str("此連線未協商 v5 戰績查詢；不代表零勝敗。\r\n"),
        record::State::Unavailable { status, .. } => {
            let _ = write!(
                text,
                "戰績查詢未更新（{:?}）；目前沒有已確認數字。\r\n",
                status
            );
        }
        record::State::Ready {
            confirmed,
            refresh_failure,
        } => {
            if let Some(totals) = confirmed.totals {
                let _ = write!(
                    text,
                    "勝利：{} · 敗北：{} · 保存版本：{}\r\n",
                    totals.wins, totals.losses, totals.revision
                );
                if !totals.stored {
                    text.push_str("查詢成功：中央尚無此角色的保存紀錄。\r\n");
                }
            }
            if let Some(status) = refresh_failure {
                let _ = write!(
                    text,
                    "本次未更新（{:?}）；以上為同角色上次確認值。\r\n",
                    status
                );
            }
        }
    }
    if snapshot.record_pending {
        text.push_str("正在重新查詢；畫面不預先加減勝敗。\r\n");
    }
    text.push_str("未完成保存的場次可能尚未計入；返回房間後有限次自動刷新，仍可手動刷新。不得由數字差額推定某一場已保存。\r\n");
    text
}
fn equipment_command(
    snapshot: &LoginSnapshot,
    item_id: Option<u64>,
    desired: Option<bool>,
) -> Option<LoginCommand> {
    if snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
        || snapshot.equipment_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.room_pending
        || snapshot.world_pending
        || snapshot.world.frame().is_some()
        || snapshot.canonical.attached()
        || !matches!(
            snapshot.phase,
            LoginPhase::ChoosingChannel | LoginPhase::InLobby
        )
    {
        return None;
    }
    let context = snapshot.navigation.as_ref()?.context;
    if matches!(
        snapshot.equipment,
        equipment::State::NotRequested | equipment::State::Unsupported
    ) {
        return None;
    }
    let request = if let Some(desired) = desired {
        let equipment::State::Ready(gear) = &snapshot.equipment else {
            return None;
        };
        let item_id = item_id?;
        if gear.source.context != context || gear.equipped(item_id)? == desired {
            return None;
        }
        if desired
            && gear
                .lifetime(item_id)
                .is_some_and(|row| !row.state.usable())
        {
            return None;
        }
        equipment::Request::set(gear, item_id, desired)
    } else {
        equipment::Request::read(context)
    };
    Some(LoginCommand::Equipment(request))
}

enum DescriptionState {
    Loading,
    Ready(Arc<DescriptionCatalog>),
    Unavailable(String),
}

impl DescriptionState {
    fn item_label(&self, token: &str) -> String {
        match self {
            Self::Loading => "名稱載入中（共享 Lineage Text）".into(),
            Self::Unavailable(reason) => format!("名稱無法解析（{reason}）"),
            Self::Ready(catalog) => match catalog.resolve_name(token) {
                Ok(name) => name.text().to_owned(),
                Err(error) => format!("名稱無法解析（{error}）"),
            },
        }
    }
}

/// One immutable profile owns one load. The UI and network heartbeat never do
/// archive I/O. A close cancels cooperatively and keeps pumping until the owned
/// thread has actually finished; synchronous filesystem reads have no invented
/// short cancellation deadline.
struct DescriptionJob {
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<std::result::Result<DescriptionCatalog, String>>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for DescriptionJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            // Exceptional construction/message-loop errors still must not detach
            // a worker. Normal close only drops after is_finished() was observed.
            let _ = thread.join();
        }
    }
}

struct Descriptions {
    state: DescriptionState,
    job: Option<DescriptionJob>,
    revision: u8,
}

impl Descriptions {
    fn start(root: &Path) -> Self {
        let root = root.to_owned();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        match thread::Builder::new()
            .name("lito-shared-descriptions".into())
            .spawn(move || {
                let result = DescriptionCatalog::load_shared_cancellable(&root, &worker_cancel)
                    .map_err(|error| {
                        error
                            .to_string()
                            .chars()
                            .filter(|c| !c.is_control())
                            .take(160)
                            .collect()
                    });
                let _ = sender.send(result);
            }) {
            Ok(thread) => Self {
                state: DescriptionState::Loading,
                job: Some(DescriptionJob {
                    cancel,
                    receiver,
                    thread: Some(thread),
                }),
                revision: 0,
            },
            Err(_) => Self {
                state: DescriptionState::Unavailable("無法建立資源工作執行緒".into()),
                job: None,
                revision: 1,
            },
        }
    }

    fn cancel(&self) {
        if let Some(job) = &self.job {
            job.cancel.store(true, Ordering::Release);
        }
    }

    fn poll(&mut self, closing: bool) {
        if closing {
            self.cancel();
        }
        if !self
            .job
            .as_ref()
            .is_some_and(|job| job.thread.as_ref().is_some_and(JoinHandle::is_finished))
        {
            return;
        }
        let mut job = self.job.take().expect("finished owned description job");
        let joined = job.thread.take().expect("owned description thread").join();
        self.state = if closing {
            DescriptionState::Unavailable("資源載入已取消".into())
        } else if joined.is_err() {
            DescriptionState::Unavailable("資源工作執行緒未正常完成".into())
        } else {
            match job.receiver.try_recv() {
                Ok(Ok(catalog)) => DescriptionState::Ready(Arc::new(catalog)),
                Ok(Err(reason)) => DescriptionState::Unavailable(reason),
                Err(_) => DescriptionState::Unavailable("資源工作未傳回結果".into()),
            }
        };
        self.revision = 1;
    }

    fn is_quiescent(&self) -> bool {
        self.job.is_none()
    }
}

fn source_data_text(
    snapshot: &LoginSnapshot,
    view: DetailView,
    descriptions: &DescriptionState,
) -> Option<String> {
    use std::fmt::Write;
    let mut text = navigation_details(snapshot)?;
    text.push_str("\r\n\r\n");
    if full_vip_details_route(snapshot) {
        match view {
            DetailView::Stats => {
                if let Some(o) = vip_detail_observation(snapshot) {
                    text.push_str(&super::battle_details::vip_character(o)?);
                } else {
                    text.push_str("完整 VIP 本場權威資料尚未與目前來源／頻道一致；不顯示舊角色保存值。");
                }
            }
            DetailView::Items => {
                if let Some(o) = vip_detail_observation(snapshot) {
                    text.push_str(&super::battle_details::vip_inventory(o,
                        |token| descriptions.item_label(token))?);
                } else {
                    text.push_str("完整 VIP 本場背包尚未與目前來源／頻道一致；不顯示舊角色保存背包。");
                }
            }
            DetailView::Record => text.push_str(&record_text(snapshot)),
            DetailView::Skills => {
                if let Some(o) = vip_detail_observation(snapshot) {
                    text.push_str(&super::battle_details::vip_skills(o,
                        |token| descriptions.item_label(token))?);
                } else {
                    text.push_str("完整 VIP 本場技能尚未與目前來源／頻道一致；不顯示舊角色保存技能。");
                }
            }
        }
        return Some(text);
    }
    if canonical_detail_page(snapshot, view).is_some() {
        text.push_str("新版戰鬥正在使用本場確認資料。請按「素質／裝備加成」、「背包資料」或「魔法／技能」開啟目前戰鬥視窗的對應頁面（Alt+C／Alt+I／Alt+P）。\r\n來源保存值不是本場即時素質；此處不顯示舊頁資料。等待進場或離場確認時不會退回舊頁。");
        return Some(text);
    }
    if matches!(view, DetailView::Stats | DetailView::Record) {
        text.push_str(&record_text(snapshot));
        text.push_str("\r\n");
        if view == DetailView::Record {
            return Some(text);
        }
    }
    if view == DetailView::Skills {
        use super::spell_protocol::{SkillsState, Support};
        text.push_str("已學技能（伺服器確認，非原生 BIN 記憶體清單）\r\n依協商版本、來源資格與本場規則逐項開放；請以各列的可用狀態及目標限制為準。目的地、傷害與扣魔均由伺服器確認。\r\n");
        match &snapshot.skills {
            SkillsState::NotRequested => text.push_str("尚未讀取技能；不代表未學會。"),
            SkillsState::Unsupported => text.push_str("此連線未協商 v4 技能能力。"),
            SkillsState::Ready(reply) => {
                let _ = write!(text, "讀取狀態：{:?}\r\n", reply.status);
                for skill in &reply.skills {
                    let availability = match skill.support {
                        Support::Teleport => "可用（僅已開放的戰鬥世界）",
                        Support::LightArrow => {
                            "可用光箭（v6；在世界選取光箭後左鍵目標，實際扣魔依凍結素質）"
                        }
                        Support::BasicHeal => {
                            "初級治癒術（僅本場同隊存活守護塔，實際扣魔依凍結素質）"
                        }
                        Support::FireStorm => "可用（本場開放時按 F，自身 3 格範圍；不指定目標）",
                        Support::Slow => "緩速術來源已資格化（v47；需新版戰鬥目標操作與材料，不由此舊頁面施放）",
                        Support::Haste => "加速術已資格化（v44；由魔法頁選取自己／角色／NPC；耗魔及期限依伺服器）",
                        Support::GreaterHaste => "強力加速術已資格化（v42；本場開放時按 G，僅自身；耗魔及期限依伺服器）",
                        Support::Unimplemented => "已學，尚未實作／不可施放",
                        Support::Disintegrate => "究極光裂術已資格化；限本場明確開放、合法目標及正義值至少500，實際耗魔依場次",
                        Support::UnsupportedTemplate => "來源模板未資格化／不可施放",
                    };
                    let _ = write!(
                        text,
                        "#{} {} · {} · MP {} · 來源重用間隔 {}ms · 原始名稱 token：{}\r\n",
                        skill.skill_id,
                        descriptions.item_label(&skill.name),
                        availability,
                        skill.mp_cost,
                        skill.reuse_millis,
                        skill.name
                    );
                }
            }
        }
        return Some(text);
    }
    let data = match &snapshot.character_data {
        CharacterDataState::Ready(data) => data,
        state => {
            text.push_str(match state {
                CharacterDataState::NotRequested => "尚未讀取來源保存資料；不代表空背包或零素質。",
                CharacterDataState::Unsupported => {
                    "此連線未協商人物／背包資料能力，無可顯示的保存資料。"
                }
                CharacterDataState::Unavailable(CharacterDataFailure::Denied) => {
                    "伺服器拒絕讀取此角色的來源保存資料；目前仍保留已確認的選角狀態。"
                }
                CharacterDataState::Unavailable(CharacterDataFailure::BackendUnavailable) => {
                    "來源保存資料目前無法使用或尚未配置；不代表空背包或零素質。"
                }
                CharacterDataState::Unavailable(CharacterDataFailure::Busy) => {
                    "來源保存資料讀取忙碌；可返回選角後重試。"
                }
                CharacterDataState::Ready(_) => unreachable!(),
            });
            return Some(text);
        }
    };
    match view {
        DetailView::Skills | DetailView::Record => unreachable!(),
        DetailView::Stats => {
            text.push_str("人物：來源保存值，非即時戰鬥素質\r\nAC 可能已含裝備加成；MaxHP/MaxMP 是來源基礎上限，保存 HP/MP 可能高於它們。\r\n");
            let _ = write!(
                text,
                "保存角色：{}  ·  等級 {}  ·  職業 {}  ·  性別 {}\r\n",
                data.character.name, data.character.level, data.character.class, data.character.sex
            );
            let values = &data.stats;
            for (label, value) in [
                ("基礎最大 HP", values.max_hp),
                ("基礎最大 MP", values.max_mp),
                ("保存 HP", values.saved_hp),
                ("保存 MP", values.saved_mp),
                ("保存 AC", values.armor_class),
                ("力量 STR", values.strength),
                ("敏捷 DEX", values.dexterity),
                ("體質 CON", values.constitution),
                ("智力 INT", values.intelligence),
                ("精神 WIS", values.wisdom),
                ("魅力 CHA", values.charisma),
                ("經驗值", values.experience),
                ("正義值", values.lawful),
                ("初始 STR", values.original_strength),
                ("初始 DEX", values.original_dexterity),
                ("初始 CON", values.original_constitution),
                ("初始 INT", values.original_intelligence),
                ("初始 WIS", values.original_wisdom),
            ] {
                let _ = write!(text, "{label}：{value}\r\n");
            }
            let _ = write!(
                text,
                "飽食度：{}/225（{}%）\r\n",
                values.food225,
                values
                    .food_percent()
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "無效".into())
            );
            if let equipment::State::Ready(gear) = &snapshot.equipment {
                let _ = write!(
                    text,
                    "\r\n裝備加成（伺服器確認，版本 {}；非完整即時素質）\r\n計算時間 {} 毫秒；下次期限重算（魔石／物品） {:?}\r\n",
                    gear.revision, gear.evaluated_at_millis, gear.next_stone_expiry
                );
                for (name, value) in equipment::STAT_NAMES.iter().zip(gear.modifiers) {
                    let _ = write!(text, "{name}：{value}\r\n");
                }
                let _ = write!(
                    text,
                    "magicReductionBase：{}\r\n",
                    gear.magic_reduction_base
                );
            }
        }
        DetailView::Items => {
            let authority = if let equipment::State::Ready(gear) = &snapshot.equipment {
                Some(gear)
            } else {
                None
            };
            if let Some(gear) = authority {
                let _ = write!(
                    text,
                    "LiTo 裝備狀態：伺服器確認版本 {}。穿脫只在確認回覆後更新；保存旗標另外列出。\r\n",
                    gear.revision
                );
            } else {
                text.push_str("來源背包保存資料（尚未合併 LiTo 私有狀態）\r\n已裝備標記來自來源 character_items；此頁僅唯讀，不提供穿脫、使用或交易。\r\n");
            }
            text.push_str("名稱僅使用此設定的共享 Lineage Text/desc-c.tbl；未解析的名稱保留原始 token，不讀取 LiTo 專用包。\r\n");
            let _ = write!(
                text,
                "共 {} 筆物品，其中 {} 筆保存為已裝備。\r\n",
                data.items.len(),
                data.items.iter().filter(|item| item.equipped).count()
            );
            for item in &data.items {
                let kind = match item.kind {
                    ItemKind::Weapon => "武器",
                    ItemKind::Armor => "防具",
                    ItemKind::Etc => "其他",
                };
                let flag = match authority.and_then(|gear| gear.equipped(item.object_id)) {
                    Some(true) => "[已裝備（LiTo 確認）]",
                    Some(false) => "[背包（LiTo 確認）]",
                    None if item.equipped => "[已裝備（保存）]",
                    None => "[背包（保存）]",
                };
                let _ = write!(
                    text,
                    "\r\n{} · {kind} · {}\r\n原始名稱 token：{}\r\n實例 ID {}／目錄 ID {}／數量 {}\r\n強化 {}／損壞值 {}／祝福 {}／剩餘次數 {}\r\n已鑑定 {}／封印 {}／背包圖 {}／地面圖 {}／重量保存值（千分單位） {}\r\n",
                    flag,
                    descriptions.item_label(&item.name_token),
                    item.name_token,
                    item.object_id,
                    item.catalog_id,
                    item.quantity,
                    item.enchantment,
                    item.durability,
                    item.blessing,
                    item.charge_count,
                    item.identified,
                    item.sealed,
                    item.inventory_graphic,
                    item.ground_graphic,
                    item.unit_weight
                );
                if authority.is_some() {
                    let _ = write!(text, "來源 SQL 保存旗標：{}\r\n", item.equipped);
                }
                if let Some(gear) = authority {
                    let _ = write!(text, "{}\r\n", item_lifetime_text(gear, item.object_id));
                }
            }
            if let Some(gear) = authority {
                for row in &gear.stale {
                    let _ = write!(
                        text,
                        "\r\n失效的舊覆寫：實例 {}／目錄 {}／{}（不代表物品期限到期；未自動刪除或重綁）\r\n",
                        row.item_id,
                        row.catalog_id,
                        stale_reason_text(row.reason)
                    );
                }
            }
        }
    }
    if let equipment::State::Ready(gear) = &snapshot.equipment {
        text.push_str("\r\n");
        if let Some(weight) = &gear.weight {
            text.push_str(&weight.notice());
            text.push_str("\r\n此為裝備讀取時的來源基準／容量；不代表未同步的本場補給重量。\r\n");
        } else {
            text.push_str("比賽負重／容量未由此連線提供，不以來源背包自行推算。\r\n");
        }
    }
    Some(text)
}

/// Read-only world inspection. Saved source values are never promoted to live
/// combat authority, and bot object IDs never match a selected human identity.
fn world_details_text(
    snapshot: &LoginSnapshot,
    page: super::world_window::DetailsPage,
    descriptions: &DescriptionState,
) -> Option<String> {
    use super::world_window::DetailsPage;
    use std::fmt::Write;
    let scope = world_ui_scope(snapshot)?;
    let nav = snapshot.navigation.as_ref()?;
    let frame = snapshot.world.frame()?;
    if snapshot.phase != LoginPhase::InLobby
        || nav.context != scope.context
        || nav.snapshot.phase != NavigationPhase::Lobby
        || nav.snapshot.selected.as_ref().map(|c| c.key) != Some(scope.context.key)
        || nav.snapshot.channel_id != Some(scope.channel_id)
        || snapshot.selected_source != Some(scope.context.key.source_id)
        || frame.validate(scope.context.key).is_err()
    {
        return None;
    }
    if let CharacterDataState::Ready(data) = &snapshot.character_data {
        if data.context != scope.context || data.validate().is_err() {
            return None;
        }
    }
    if let equipment::State::Ready(gear) = &snapshot.equipment {
        if gear.source.context != scope.context
            || gear.validate().is_err()
            || !matches!(&snapshot.character_data, CharacterDataState::Ready(data) if data.as_ref() == gear.source.as_ref())
        {
            return None;
        }
    }
    if let super::spell_protocol::SkillsState::Ready(skills) = &snapshot.skills {
        if skills.context != scope.context || skills.validate().is_err() {
            return None;
        }
    }
    let actor = frame
        .actors
        .iter()
        .find(|a| a.matches_character(scope.context.key))?;
    let member = frame
        .room
        .members
        .iter()
        .find(|m| m.identity() == actor.identity())?;
    let combat = snapshot
        .combat
        .as_ref()
        .filter(|c| c.world == *frame && c.validate().is_ok());
    let mut text = format!(
        "{}({}) · 來源 {}／角色 {}\r\n場次 {}／世代 {}／世界版本 {} · {:?}\r\nEsc 關閉檢視；返回房間仍由獨立按鈕及伺服器確認。\r\n\r\n",
        member.name, member.source_name, scope.context.key.source_id, scope.context.key.object_id,
        frame.match_id, frame.generation, frame.revision, frame.phase
    );
    match page {
        DetailsPage::Character => {
            text.push_str("【本場即時能力：伺服器確認】\r\n");
            if let Some(v) = combat.and_then(|c| c.vital(actor.entity_id)) {
                let _ = write!(
                    text,
                    "HP：{} / {}\r\nMP：{} / {}\r\nAC：{}\r\n武器類型：{}\r\n攻擊間隔：{} ns\r\n",
                    v.hp, v.max_hp, v.mp, v.max_mp, v.ac, v.weapon_type, v.attack_interval_nanos
                );
                if let Some(weight) = combat.and_then(|c| c.weight(actor.entity_id)) {
                    let _ = write!(
                        text,
                        "本場負重：{} / {} · 階段 {}/240 · 命中減值 {}\r\n",
                        weight.total,
                        weight.maximum(),
                        weight.stage,
                        weight.hit_penalty
                    );
                }
                text.push_str("未由本場封包提供的能力／飽食度不推算；下方明確列為來源保存值。\r\n");
            } else if frame.combat_enabled(snapshot.combat_supported) {
                text.push_str("即時能力等待同一世界版本的戰鬥確認；未以保存HP／AC代替。\r\n");
            } else {
                text.push_str("本場非戰鬥規則，沒有即時HP／MP／AC。\r\n");
            }
            text.push_str("\r\n【來源保存資料／裝備確認：不是即時戰鬥能力】\r\n");
            text.push_str(&source_data_text(
                snapshot,
                DetailView::Stats,
                descriptions,
            )?);
        }
        DetailsPage::Inventory => {
            text.push_str("【本場背包：不等於來源背包】\r\n本場補給在結算後重設；使用請按下方補給按鈕，僅伺服器確認後扣量。\r\n");
            if let Some(items) = combat.and_then(|c| c.item(actor.entity_id)) {
                if items.stocks.is_empty() {
                    text.push_str("本場確認沒有補給。\r\n");
                }
                for stock in &items.stocks {
                    let _ = write!(
                        text,
                        "補給 #{}：剩餘 {}／開場 {}\r\n",
                        stock.item_id, stock.remaining, stock.initial
                    );
                }
            } else {
                text.push_str("本場補給未開放或等待同版確認；不代表來源背包是空的。\r\n");
            }
            if let Some(ammo) = combat.and_then(|c| c.ammo(actor.entity_id)) {
                for stock in &ammo.stocks {
                    let _ = write!(
                        text,
                        "箭矢 #{}{}：剩餘 {}／開場 {}\r\n",
                        stock.facts.item_id,
                        if ammo.selected_item_id == stock.facts.item_id {
                            " [已選取]"
                        } else {
                            ""
                        },
                        stock.remaining,
                        stock.initial
                    );
                }
            }
            text.push_str("\r\n【來源保存背包／LiTo裝備旗標：檢視用】\r\n此頁不在戰鬥中穿脫裝備，也不把來源道具當作本場可消耗物。\r\n");
            text.push_str(&source_data_text(
                snapshot,
                DetailView::Items,
                descriptions,
            )?);
        }
        DetailsPage::Skills => {
            use super::spell_protocol::{SkillsState, Support};
            text.push_str("【已學技能與本場規則】\r\n此頁僅檢視，不會因選取文字而施法。關閉檢視後使用世界技能選單、T瞬移或F火風暴；MP、目標、存活狀態及間隔仍由既有施法流程與伺服器確認。\r\n來源模板MP不是本場最終耗魔。\r\n");
            match &snapshot.skills {
                SkillsState::NotRequested => text.push_str("已學技能尚未確認；不是未學會。\r\n"),
                SkillsState::Unsupported => text.push_str("此連線未協商技能目錄。\r\n"),
                SkillsState::Ready(reply) if reply.status != super::spell_protocol::Status::Ok => {
                    let _ = write!(
                        text,
                        "技能目錄未確認：{:?}；不是空技能表。\r\n",
                        reply.status
                    );
                }
                SkillsState::Ready(reply) => {
                    for skill in &reply.skills {
                        let enabled = match skill.support {
                            Support::Teleport => frame.cast(actor.entity_id).is_some(),
                            Support::LightArrow => combat.is_some_and(|c| {
                                c.damage(actor.entity_id).is_some()
                                    || c.npc_spell(actor.entity_id).is_some()
                            }),
                            Support::BasicHeal => combat.is_some_and(|c| c.honor.is_some()),
                            Support::FireStorm => {
                                combat.is_some() && fire_storm_available(snapshot, actor.entity_id)
                            }
                            Support::Disintegrate => combat.is_some_and(|c| {
                                c.disintegrate(actor.entity_id).is_some()
                                    || c.npc_disintegrate(actor.entity_id).is_some()
                            }),
                            Support::Unimplemented
                            | Support::UnsupportedTemplate
                            | Support::GreaterHaste
                            | Support::Haste
                            | Support::Slow => false,
                        };
                        let status = match skill.support {
                            Support::Unimplemented => "尚未實作，不可施放",
                            Support::UnsupportedTemplate => "來源模板未資格化，不可施放",
                            _ if enabled => "本場規則開放，施放條件仍需確認",
                            _ => "本場未開放／等待同版確認",
                        };
                        let _ = write!(
                            text,
                            "#{} {} · {} · 模板MP {} · 模板間隔 {} ms · 原始名稱 token：{}\r\n",
                            skill.skill_id,
                            descriptions.item_label(&skill.name),
                            status,
                            skill.mp_cost,
                            skill.reuse_millis,
                            skill.name
                        );
                    }
                }
            }
        }
    }
    if text.contains('\0') || text.encode_utf16().count() > super::world_window::MAX_DETAILS_UTF16 {
        return None;
    }
    Some(text)
}

#[cfg(test)]
pub(crate) fn test_world_details_text(
    snapshot: &LoginSnapshot,
    page: super::world_window::DetailsPage,
) -> Option<String> {
    world_details_text(snapshot, page, &DescriptionState::Loading)
}

/// Retained Arcs prevent pointer-reuse cache hits. Refresh only when confirmed
/// inputs/page change, not for every paint, cursor move or world-runtime poll.
#[derive(Clone, PartialEq, Eq)]
struct WorldDetailsCache {
    canonical: Option<(super::battle_window::Stamp, i64, i64)>,
    inventory: Option<super::battle_inventory::Selection>,
    character_equipped: Option<u64>,
    page: super::world_window::DetailsPage,
    scope: Option<WorldUiScope>,
    phase: LoginPhase,
    source: Option<u32>,
    navigation: Option<NavigationReply>,
    world: world::State,
    combat: Option<Arc<super::combat_protocol::Frame>>,
    character_data: CharacterDataState,
    equipment: equipment::State,
    skills: super::spell_protocol::SkillsState,
    record: record::State,
    record_pending: bool,
    combat_supported: bool,
    descriptions: u8,
}
impl WorldDetailsCache {
    fn new(
        snapshot: &LoginSnapshot,
        page: super::world_window::DetailsPage,
        descriptions: u8,
    ) -> Self {
        Self {
            canonical: None,
            inventory: None,
            character_equipped: None,
            page,
            scope: world_ui_scope(snapshot),
            phase: snapshot.phase,
            source: snapshot.selected_source,
            navigation: snapshot.navigation.clone(),
            world: snapshot.world.clone(),
            combat: snapshot.combat.clone(),
            character_data: snapshot.character_data.clone(),
            equipment: snapshot.equipment.clone(),
            skills: snapshot.skills.clone(),
            record: snapshot.record.clone(),
            record_pending: snapshot.record_pending,
            combat_supported: snapshot.combat_supported,
            descriptions,
        }
    }
}

fn stale_reason_text(reason: equipment::StaleReason) -> &'static str {
    match reason {
        equipment::StaleReason::Missing => "來源已無此物件",
        equipment::StaleReason::CatalogChanged => "來源目錄 ID 已變更",
        equipment::StaleReason::FactsChanged => "來源物件事實已變更",
    }
}
fn item_lifetime_text(gear: &equipment::Snapshot, item_id: u64) -> String {
    match gear.lifetime(item_id) {
        None => "物品期限：版本 1 未提供；不可據此視為永久物品。".into(),
        Some(row) => {
            let mut text = format!("物品期限：{}。", row.state.label());
            if let Some(cutoff) = row.expires_at_millis {
                let _ = write!(
                    text,
                    " 伺服器保守截止值 {cutoff}（Unix 毫秒；非原 SQL 日期時間）。"
                );
            }
            text.push_str(" 未刪除來源 SQL 物品；顯示依伺服器確認，不依本機時鐘自行改穿戴旗標。");
            text
        }
    }
}

fn validate_credentials(account: &str, password: &str) -> std::result::Result<(), &'static str> {
    if account.is_empty() || account.len() > 64 || account.chars().any(char::is_control) {
        return Err("帳號需為 1–64 UTF-8 位元組，不可包含控制字元。");
    }
    if password.is_empty() || password.len() > 256 || password.chars().any(char::is_control) {
        return Err("密碼需為 1–256 UTF-8 位元組，不可包含控制字元。");
    }
    Ok(())
}

fn validated_login_command(
    mut account: Zeroizing<String>,
    mut password: Zeroizing<String>,
) -> std::result::Result<LoginCommand, &'static str> {
    validate_credentials(&account, &password)?;
    // Transfer the same String allocations, not plaintext clones. The controller
    // must wrap them immediately in its own Credentials guard when accepting.
    Ok(LoginCommand::Login {
        account: std::mem::take(&mut *account),
        password: std::mem::take(&mut *password),
    })
}

fn zeroize_unsubmitted(command: &mut LoginCommand) {
    if let LoginCommand::Login { account, password } = command {
        account.zeroize();
        password.zeroize();
    }
}

fn snapshot_changed(previous: Option<&LoginSnapshot>, next: &LoginSnapshot) -> bool {
    previous.is_some_and(|previous| previous != next)
}

fn clear_credentials_on_transition(previous: Option<&LoginSnapshot>, next: &LoginSnapshot) -> bool {
    matches!(
        next.phase,
        LoginPhase::Failed | LoginPhase::Disconnected | LoginPhase::Disconnecting
    ) && previous.is_none_or(|previous| previous.phase != next.phase)
}

struct Ui {
    controller: Box<dyn LoginController>,
    last: Option<LoginSnapshot>,
    local_error: Option<String>,
    closing: bool,
    fatal: bool,
}

impl Ui {
    fn begin_close(&mut self) {
        if !self.closing {
            self.closing = true;
            if self.controller.command(LoginCommand::Shutdown).is_err() {
                self.local_error = Some("取消要求未完成，等待控制器釋放連線。".into());
            }
        }
    }
}

/// Presentation state is scoped independently of the latest-snapshot mailbox:
/// the UI is not guaranteed to observe the intervening world=None after LEAVE.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct WorldUiScope {
    context: SelectionContext,
    channel_id: u8,
    match_id: u64,
    generation: u64,
}

fn world_ui_scope(snapshot: &LoginSnapshot) -> Option<WorldUiScope> {
    if snapshot.canonical.supported() {
        let scope = super::battle_control::scope_for(snapshot)?;
        let state = snapshot.canonical.snapshot()?;
        if state.scope() != scope {
            return None;
        }
        let world = &state.attachment()?.world;
        return Some(WorldUiScope {
            context: scope.0,
            channel_id: scope.1,
            match_id: world.match_id as u64,
            generation: world.generation as u64,
        });
    }
    let world::State::Ready(reply) = &snapshot.world else {
        return None;
    };
    let frame = reply.frame.as_ref()?;
    Some(WorldUiScope {
        context: reply.echo.context,
        channel_id: reply.echo.channel_id,
        match_id: frame.match_id,
        generation: frame.generation,
    })
}

#[derive(Default)]
struct WorldUiScopeState {
    current: Cell<Option<WorldUiScope>>,
    min_hit_revision: Cell<i64>,
    failure: Cell<Option<&'static str>>,
    leave_queued: Cell<bool>,
    leave_inflight: Cell<bool>,
}
impl WorldUiScopeState {
    /// Returns true before the caller may read a queued world-window action.
    /// This only retires UI intent; it cannot acknowledge/cancel a TLS command.
    fn replace(&self, next: Option<WorldUiScope>) -> bool {
        if self.current.replace(next) == next {
            return false;
        }
        self.min_hit_revision.set(0);
        self.failure.set(None);
        self.leave_queued.set(false);
        self.leave_inflight.set(false);
        true
    }
}

fn scoped_world_action(changed: bool, pending: Option<WorldAction>) -> Option<WorldAction> {
    if changed {
        None
    } else {
        pending
    }
}

struct WindowState {
    test_deadline: Option<Instant>,
    settings: RefCell<super::settings_window::SettingsController>,
    chat_window: RefCell<Option<super::chat_window::ChatWindow>>,
    direct_chat_window: RefCell<Option<super::direct_chat_window::DirectChatWindow>>,
    ui: RefCell<Ui>,
    controls: Cell<Option<[HWND; CONTROL_COUNT]>>,
    rendering: Cell<bool>,
    rendered_status: RefCell<String>,
    detail_view: Cell<DetailView>,
    rendered_detail_view: Cell<Option<DetailView>>,
    descriptions: RefCell<Descriptions>,
    vip_status: RefCell<super::vip_status_preview::Preview>,
    vip_inventory: RefCell<super::vip_inventory_preview::Preview>,
    rendered_vip_inventory: RefCell<Option<super::vip_inventory_preview::Model>>,
    vip_inventory_visible: Cell<bool>,
    vip_inventory_rendered_revision: Cell<u64>,
    vip_inventory_description_revision: Cell<Option<u8>>,
    vip_inventory_requested: Cell<bool>,
    vip_status_visible: Cell<bool>,
    vip_status_rendered_revision: Cell<u64>,
    rendered_vip_status_text_revision: Cell<Option<u64>>,
    effect_catalog_path: Option<std::path::PathBuf>,
    rendered_description_revision: Cell<Option<u8>>,
    rendered_equipment: RefCell<Option<Arc<equipment::Snapshot>>>,
    rendered_gear_description_revision: Cell<Option<u8>>,
    room_form: RefCell<Option<RoomForm>>,
    room_form_context: Cell<Option<rooms::Common>>,
    shop_window: RefCell<Option<ShopWindow>>,
    shared_root: std::path::PathBuf,
    world_runtime: RefCell<WorldRuntime>,
    world_audio: RefCell<super::world_audio::WorldAudioOutput>,
    battle_runtime: RefCell<super::battle_runtime::BattleRuntime>,
    battle_audio: RefCell<super::battle_playback::BattleAudioOutput>,
    world_navigation: RefCell<WorldNavigation>,
    canonical_route: RefCell<super::battle_route::Route>,
    world_window: RefCell<Option<WorldWindow>>,
    world_details_cache: RefCell<Option<WorldDetailsCache>>,
    world_load_ack: Cell<Option<(u64, u64)>>,
    world_last_step: Cell<Option<Instant>>,
    world_last_move_ack: Cell<Option<(u64, u64, u64, u64)>>,
    world_last_attack: Cell<Option<Instant>>,
    world_last_attack_ack: Cell<Option<(u64, u64, u64, u64)>>,
    world_last_cast: Cell<Option<(Instant, u32)>>,
    world_last_item: Cell<Option<(u64, u64, u64, i64, Instant, u32)>>,
    world_item_observation: RefCell<Option<(u64, u64, item::ItemState)>>,
    world_combat_seen: Cell<Option<(u64, u64)>>,
    world_casts: RefCell<Vec<super::spell_protocol::CastState>>,
    world_damage_casts: RefCell<Vec<super::spell_protocol::DamageCastState>>,
    world_npc_casts: RefCell<Vec<super::npc_spell_protocol::State>>,
    world_honor: RefCell<Option<super::honor_protocol::State>>,
    world_fire_casts: RefCell<Vec<super::fire_storm_presentation::Event>>,
    world_last_fire: Cell<Option<Instant>>,
    world_dis_casts: RefCell<Vec<super::disintegrate_presentation::Receipt>>,
    world_last_dis: Cell<Option<Instant>>,
    world_scope: WorldUiScopeState,
    text_fault: Cell<bool>,
    font: Cell<HFONT>,
}

impl Drop for WindowState {
    fn drop(&mut self) {
        let font = self.font.get();
        if !font.0.is_null() {
            unsafe {
                let _ = DeleteObject(font.into());
            }
        }
    }
}

/// The controller should be idle on entry; connecting is an explicit UI action.
/// This function does not run a native BIN or persist credentials/settings.
pub fn run(profile: &ModernProfile, controller: Box<dyn LoginController>) -> Result<()> {
    ensure!(
        controller.is_quiescent(),
        "login window must receive an idle controller"
    );
    run_owner(profile, controller, None)
}

/// Test-classpath entry only: fixture authentication is programmatic, not UI
/// automation of a user's account. All subsequent HWND logic is the real owner.
#[cfg(test)]
pub(crate) fn run_connected_test(
    profile: &ModernProfile,
    controller: Box<dyn LoginController>,
    deadline: Instant,
) -> Result<()> {
    let view = controller.snapshot();
    ensure!(
        view.phase == LoginPhase::InLobby
            && view.canonical.supported()
            && view.canonical.attached()
            && view.world.frame().is_none(),
        "test needs canonical fixture attachment"
    );
    run_owner(profile, controller, Some(deadline))
}

/// Independent source-reselection HWND test. Keep the attached-world entry's
/// preconditions unchanged; this entry accepts only a fully cleared roster.
#[cfg(test)]
pub(crate) fn run_source_selection_test(
    profile: &ModernProfile,
    controller: Box<dyn LoginController>,
    deadline: Instant,
) -> Result<()> {
    let view = controller.snapshot();
    ensure!(
        !controller.is_quiescent()
            && deadline > Instant::now()
            && view.phase == LoginPhase::CharactersReady
            && view.selected_source.is_some()
            && view.roster_epoch > 0
            && !view.characters.is_empty()
            && view
                .characters
                .iter()
                .all(|row| Some(row.source_id) == view.selected_source)
            && view.navigation.is_none()
            && matches!(view.character_data, CharacterDataState::NotRequested)
            && matches!(view.equipment, equipment::State::NotRequested)
            && matches!(
                view.skills,
                super::spell_protocol::SkillsState::NotRequested
            )
            && matches!(view.record, record::State::NotRequested)
            && view.rooms.joined().is_none()
            && !view.canonical.attached()
            && view.world.frame().is_none()
            && !view.world_pending,
        "source test needs an acknowledged, cleared character roster"
    );
    run_owner(profile, controller, Some(deadline))
}

fn run_owner(
    profile: &ModernProfile,
    controller: Box<dyn LoginController>,
    test_deadline: Option<Instant>,
) -> Result<()> {
    let mut settings = super::settings_window::SettingsController::default();
    #[cfg(test)]
    if test_deadline.is_some() {
        settings.start_isolated_test()?;
    } else {
        settings.start()?;
    }
    #[cfg(not(test))]
    settings.start()?;
    let mut state = Box::new(WindowState {
        test_deadline,
        settings: RefCell::new(settings),
        chat_window: RefCell::new(None),
        direct_chat_window: RefCell::new(None),
        ui: RefCell::new(Ui {
            controller,
            last: None,
            local_error: None,
            closing: false,
            fatal: false,
        }),
        controls: Cell::new(None),
        rendering: Cell::new(false),
        rendered_status: RefCell::new(String::new()),
        detail_view: Cell::new(DetailView::Stats),
        rendered_detail_view: Cell::new(None),
        descriptions: RefCell::new(Descriptions::start(profile.shared_resource_root())),
        vip_status: RefCell::new(super::vip_status_preview::Preview::default()),
        vip_inventory: RefCell::new(super::vip_inventory_preview::Preview::default()),
        rendered_vip_inventory: RefCell::new(None),
        vip_inventory_visible: Cell::new(false),
        vip_inventory_rendered_revision: Cell::new(0),
        vip_inventory_description_revision: Cell::new(None),
        vip_inventory_requested: Cell::new(false),
        vip_status_visible: Cell::new(false),
        vip_status_rendered_revision: Cell::new(0),
        rendered_vip_status_text_revision: Cell::new(None),
        effect_catalog_path: profile.effect_catalog_path().map(std::path::Path::to_path_buf),
        rendered_description_revision: Cell::new(None),
        rendered_equipment: RefCell::new(None),
        rendered_gear_description_revision: Cell::new(None),
        room_form: RefCell::new(None),
        room_form_context: Cell::new(None),
        shop_window: RefCell::new(None),
        shared_root: profile.shared_resource_root().to_path_buf(),
        world_runtime: RefCell::new(WorldRuntime::with_preferences(
            profile.shared_resource_root().to_path_buf(),
            super::preferences::shared(),
        )?),
        world_audio: RefCell::new(super::world_audio::WorldAudioOutput::new()?),
        battle_runtime: RefCell::new(
            super::battle_runtime::BattleRuntime::with_preferences_and_effects(
                profile.shared_resource_root().to_path_buf(),
                profile
                    .effect_catalog_path()
                    .map(std::path::Path::to_path_buf),
                super::preferences::shared(),
            )?,
        ),
        battle_audio: RefCell::new(super::battle_playback::BattleAudioOutput::new()?),
        world_navigation: RefCell::new(WorldNavigation::default()),
        canonical_route: RefCell::new(super::battle_route::Route::default()),
        world_window: RefCell::new(None),
        world_details_cache: RefCell::new(None),
        world_load_ack: Cell::new(None),
        world_last_step: Cell::new(None),
        world_last_move_ack: Cell::new(None),
        world_last_attack: Cell::new(None),
        world_last_attack_ack: Cell::new(None),
        world_last_cast: Cell::new(None),
        world_last_item: Cell::new(None),
        world_item_observation: RefCell::new(None),
        world_combat_seen: Cell::new(None),
        world_casts: RefCell::new(Vec::new()),
        world_damage_casts: RefCell::new(Vec::new()),
        world_npc_casts: RefCell::new(Vec::new()),
        world_honor: RefCell::new(None),
        world_fire_casts: RefCell::new(Vec::new()),
        world_last_fire: Cell::new(None),
        world_dis_casts: RefCell::new(Vec::new()),
        world_last_dis: Cell::new(None),
        world_scope: WorldUiScopeState::default(),
        text_fault: Cell::new(false),
        font: Cell::new(HFONT::default()),
    });
    unsafe {
        // Own a checked per-thread scope instead of relying on a process-wide
        // override or the launching program's current DPI context,
        // and restore it after all modern HWNDs have been destroyed.
        let previous_dpi = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        ensure!(
            !previous_dpi.0.is_null(),
            "set modern login thread DPI context"
        );
        let _dpi = ThreadDpiGuard(previous_dpi);
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        ensure!(
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: CLASS,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                ..Default::default()
            }) != 0,
            "register modern login window"
        );
        let _class = ClassGuard(instance);
        let hwnd = CreateWindowExW(
            EX_STYLE,
            CLASS,
            w!("LiTo 現代客戶端 — 登入與角色"),
            STYLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            820,
            750,
            None,
            None,
            Some(instance),
            Some((&mut *state as *mut WindowState).cast()),
        )?;
        let _window = WindowGuard(hwnd);
        let controls = create_controls(hwnd, instance)?;
        state.controls.set(Some(controls));
        set_text_checked(
            controls[PROFILE],
            &format!("伺服器：{}", profile.display_name()),
        )?;
        set_text_checked(
            controls[ENDPOINT],
            &format!(
                "連線端點：{}  ·  TLS：{}",
                profile.endpoint(),
                profile.tls_server_name()
            ),
        )?;
        set_text_checked(
            controls[ROOT],
            &format!("共享資源：{}", profile.shared_resource_root().display()),
        )?;
        ensure!(
            SetTimer(Some(hwnd), TIMER, 30, None) != 0,
            "create login poll timer"
        );
        replace_font(hwnd, &state)?;
        let dpi = GetDpiForWindow(hwnd);
        let mut initial = RECT {
            left: 0,
            top: 0,
            right: scale(760, dpi),
            bottom: scale(MIN_HEIGHT_DIP, dpi),
        };
        AdjustWindowRectExForDpi(&mut initial, STYLE, false, EX_STYLE, dpi)?;
        SetWindowPos(
            hwnd,
            None,
            0,
            0,
            initial.right - initial.left,
            initial.bottom - initial.top,
            SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
        )?;
        resize(hwnd, &state);
        // Show the owner first. The first refresh may create and activate an
        // already-attached world; never raise the owner over it afterwards.
        let _ = ShowWindow(hwnd, SW_SHOW);
        refresh(hwnd, &state);
        if state.world_window.borrow().is_none() {
            use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
            if let Some(control) = [CONNECT, ACCOUNT, SOURCE, CHARACTERS, CHANNELS, EXIT]
                .into_iter()
                .map(|index| controls[index])
                .find(|control| IsWindowEnabled(*control).as_bool())
            {
                let _ = SetFocus(Some(control));
            }
        }
        let mut message = MSG::default();
        loop {
            match GetMessageW(&mut message, None, 0, 0).0 {
                -1 => {
                    // The normal shutdown path below is asynchronous. GetMessage
                    // failure is exceptional and must still request cancellation.
                    let error = std::io::Error::last_os_error();
                    request_close(hwnd, &state);
                    return Err(error.into());
                }
                0 => {
                    let quiescent = state.ui.borrow().controller.is_quiescent()
                        && state.settings.borrow().is_quiescent()
                        && state.descriptions.borrow().is_quiescent()
                        && state.vip_status.borrow().is_quiescent()
                        && state.vip_inventory.borrow().is_quiescent()
                        && state.world_runtime.borrow().is_quiescent()
                        && state.battle_runtime.borrow().is_quiescent()
                        && state.canonical_route.borrow().is_quiescent()
                        && state
                            .room_form
                            .borrow()
                            .as_ref()
                            .is_none_or(RoomForm::is_quiescent)
                        && state
                            .shop_window
                            .borrow()
                            .as_ref()
                            .is_none_or(ShopWindow::is_quiescent);
                    if quiescent {
                        clear_credentials(&state, true);
                        break;
                    }
                    // Ignore an early external WM_QUIT until workers retire.
                    request_close(hwnd, &state);
                    ensure!(
                        IsWindow(Some(hwnd)).as_bool(),
                        "login window destroyed before workers retired"
                    );
                }
                _ => {
                    #[cfg(test)]
                    let _navigation_trace = NavigationTrace::begin(hwnd, controls, &message);
                    if state
                        .direct_chat_window
                        .borrow()
                        .as_ref()
                        .is_some_and(|w| w.pre_translate(&message))
                    {
                        continue;
                    }
                    if state
                        .chat_window
                        .borrow()
                        .as_ref()
                        .is_some_and(|chat| chat.pre_translate(&message))
                    {
                        continue;
                    }
                    if state.settings.borrow().pre_translate(&message) {
                        continue;
                    }
                    let shop_hwnd = state
                        .shop_window
                        .borrow()
                        .as_ref()
                        .and_then(ShopWindow::window_handle);
                    if shop_hwnd
                        .is_some_and(|shop| ShopWindow::pre_translate_window(shop, &message))
                    {
                        continue;
                    }
                    let world_hwnd = state
                        .world_window
                        .borrow()
                        .as_ref()
                        .and_then(WorldWindow::window_handle);
                    if world_hwnd
                        .is_some_and(|world| WorldWindow::pre_translate_window(world, &message))
                    {
                        continue;
                    }
                    let form_hwnd = state
                        .room_form
                        .borrow()
                        .as_ref()
                        .and_then(RoomForm::window_handle);
                    if form_hwnd.is_some_and(|form| RoomForm::pre_translate_window(form, &message))
                    {
                        continue;
                    }
                    if message.hwnd == hwnd || IsChild(hwnd, message.hwnd).as_bool() {
                        // The world's local UI shortcuts remain available if
                        // focus returned to the lobby: inspection Esc first,
                        // then confirmed Leave; F10/F11 remain local UI.
                        // Use the copied HWND, without holding the RefCell
                        // borrow across SetFocus/native window re-entry.
                        if world_hwnd.is_some_and(|world| {
                            WorldWindow::pre_translate_owner_shortcut(world, &message)
                        }) {
                            continue;
                        }
                        // Like chat history, the read-only multiline EDIT asks
                        // to keep Tab. Preserve native dialog traversal without
                        // changing its selection, copy, scroll, or text handling.
                        if super::chat_window::readonly_tab(hwnd, &message, &[controls[DETAILS]]) {
                            continue;
                        }
                        if let Some(page) = super::world_window::details_mnemonic(&message) {
                            use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
                            let (button, detail) = detail_mnemonic_target(page);
                            if message.lParam.0 & (1 << 30) == 0
                                && IsWindowEnabled(controls[button]).as_bool()
                            {
                                show_player_detail(hwnd, &state, detail);
                            }
                            continue;
                        }
                    }
                    if let Some(action) = fallback_escape(&message) {
                        if action == EscapeFallback::Close {
                            request_close(hwnd, &state);
                        }
                    } else if message.message == WM_KEYDOWN
                        && message.wParam.0 == 0x0d
                        && matches!(message.hwnd, handle if handle == controls[ACCOUNT] || handle == controls[PASSWORD])
                    {
                        submit_login(hwnd, &state);
                    } else if !IsDialogMessageW(hwnd, &message).as_bool() {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
        }
    }
    let ui = state.ui.get_mut();
    if ui.fatal || state.text_fault.get() {
        anyhow::bail!("modern login UI closed after an internal state error");
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EscapeFallback {
    Consume,
    Close,
}
fn fallback_escape(message: &MSG) -> Option<EscapeFallback> {
    if message.message != WM_KEYDOWN || message.wParam.0 != 0x1b {
        return None;
    }
    // Returning from a world can destroy its HWND before key-up. Consume held
    // Esc here so that one physical press cannot both return and exit the app.
    Some(if message.lParam.0 & (1 << 30) != 0 {
        EscapeFallback::Consume
    } else {
        EscapeFallback::Close
    })
}

// Test-only observation of native navigation routing. Do not record text,
// arbitrary keys, credentials, window addresses, or mutate focus/messages.
#[cfg(test)]
fn traced_navigation_key(message: &MSG) -> Option<&'static str> {
    if !matches!(message.message, WM_KEYDOWN | WM_KEYUP) {
        return None;
    }
    match message.wParam.0 {
        0x09 => Some("tab"),
        0x0d => Some("return"),
        0x1b => Some("escape"),
        _ => None,
    }
}

#[cfg(test)]
#[derive(Debug, PartialEq, Eq)]
enum NavigationFocus {
    Owner,
    Control(usize),
    Other,
}

#[cfg(test)]
struct NavigationTrace {
    owner: HWND,
    controls: [HWND; CONTROL_COUNT],
    before: NavigationFocus,
    destination: NavigationFocus,
    key: &'static str,
    down: bool,
    repeated: bool,
    time: u32,
}

#[cfg(test)]
impl NavigationTrace {
    fn role(owner: HWND, controls: &[HWND; CONTROL_COUNT], handle: HWND) -> NavigationFocus {
        if handle == owner {
            NavigationFocus::Owner
        } else if let Some(index) = controls.iter().position(|control| *control == handle) {
            NavigationFocus::Control(index)
        } else {
            NavigationFocus::Other
        }
    }

    fn begin(owner: HWND, controls: [HWND; CONTROL_COUNT], message: &MSG) -> Option<Self> {
        let key = traced_navigation_key(message)?;
        let focus = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
        Some(Self {
            owner,
            controls,
            before: Self::role(owner, &controls, focus),
            destination: Self::role(owner, &controls, message.hwnd),
            key,
            down: message.message == WM_KEYDOWN,
            repeated: message.lParam.0 & (1 << 30) != 0,
            time: message.time,
        })
    }
}

#[cfg(test)]
impl Drop for NavigationTrace {
    fn drop(&mut self) {
        let focus = unsafe { windows::Win32::UI::Input::KeyboardAndMouse::GetFocus() };
        let after = Self::role(self.owner, &self.controls, focus);
        eprintln!(
            "LOGIN_NAV time={} key={} down={} repeated={} destination={:?} before={:?} after={:?}",
            self.time, self.key, self.down, self.repeated, self.destination, self.before, after,
        );
    }
}

fn restore_disabled_control_focus(
    closing: bool,
    owner_foreground: bool,
    previous_usable: bool,
    focus_empty: bool,
) -> bool {
    !closing && owner_foreground && !previous_usable && focus_empty
}

unsafe fn repair_refresh_focus(
    owner: HWND,
    controls: &[HWND; CONTROL_COUNT],
    previous: Option<HWND>,
    closing: bool,
) {
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetFocus, IsWindowEnabled};
    let Some(previous) = previous else {
        return;
    };
    // Disabling the focused pushbutton (for an in-flight room/navigation
    // request) clears keyboard focus. Only repair that loss in our foreground
    // owner; never activate the app or take focus from a child dialog/control.
    if !restore_disabled_control_focus(
        closing,
        GetForegroundWindow() == owner && IsWindowEnabled(owner).as_bool(),
        IsWindowVisible(previous).as_bool() && IsWindowEnabled(previous).as_bool(),
        GetFocus().0.is_null(),
    ) {
        return;
    }
    // Prefer the current page's selectable list, then its read-only status.
    // This is focus only: it must not select a row or submit another request.
    if let Some(next) = [
        CHARACTERS, CHANNELS, SOURCE, ACCOUNT, ROOM_LIST, DETAILS, CONNECT, DISCONNECT,
    ]
    .into_iter()
    .map(|index| controls[index])
    .find(|control| IsWindowVisible(*control).as_bool() && IsWindowEnabled(*control).as_bool())
    {
        #[cfg(test)]
        eprintln!(
            "LOGIN_FOCUS_REPAIR previous={:?} next={:?} hidden={} disabled={}",
            NavigationTrace::role(owner, controls, previous),
            NavigationTrace::role(owner, controls, next),
            !IsWindowVisible(previous).as_bool(),
            !IsWindowEnabled(previous).as_bool(),
        );
        let _ = SetFocus(Some(next));
    }
}

struct ThreadDpiGuard(DPI_AWARENESS_CONTEXT);
impl Drop for ThreadDpiGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = SetThreadDpiAwarenessContext(self.0);
        }
    }
}

struct WindowGuard(HWND);
impl Drop for WindowGuard {
    fn drop(&mut self) {
        unsafe {
            if IsWindow(Some(self.0)).as_bool() {
                let _ = DestroyWindow(self.0);
            }
        }
    }
}
struct ClassGuard(HINSTANCE);
impl Drop for ClassGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterClassW(CLASS, Some(self.0));
        }
    }
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
fn checked_text_units(text: &str) -> Result<Vec<u16>> {
    ensure!(!text.contains('\0'), "UI text contains NUL");
    let units = wide(text);
    ensure!(
        units.len() - 1 <= MAX_DETAILS_UTF16,
        "UI text exceeds safe limit"
    );
    Ok(units)
}

unsafe fn set_text_checked(hwnd: HWND, text: &str) -> Result<()> {
    let units = checked_text_units(text)?;
    SetWindowTextW(hwnd, PCWSTR(units.as_ptr()))?;
    ensure!(
        GetWindowTextLengthW(hwnd) as usize == units.len() - 1,
        "UI text was not fully accepted"
    );
    // These are presentation strings or empty credential clears, never input
    // credentials. Verify the actual control accepted ALL rows, not only that
    // formatting a large String happened to succeed.
    let mut accepted = vec![0u16; units.len()];
    let count = GetWindowTextW(hwnd, &mut accepted);
    ensure!(
        count as usize == units.len() - 1 && accepted == units,
        "UI text readback mismatch"
    );
    Ok(())
}

unsafe fn paint_text(state: &WindowState, hwnd: HWND, text: &str) {
    if set_text_checked(hwnd, text).is_err() {
        // Sticky even when a subsequent snapshot/status changes. No recursive
        // Win32 painting or RefCell borrowing on this synchronous failure path.
        state.text_fault.set(true);
    }
}

/// DETAILS is a scrollable read-only EDIT. Identical text must not reset the
/// native selection/scroll; a same-life VIP update preserves both.
unsafe fn paint_details_text(state: &WindowState, hwnd: HWND, text: &str, preserve: bool) {
    let result = (|| -> Result<()> {
        let desired = checked_text_units(text)?;
        if GetWindowTextLengthW(hwnd) as usize == desired.len() - 1 {
            let mut current = vec![0u16; desired.len()];
            let count = GetWindowTextW(hwnd, &mut current);
            if count as usize == desired.len() - 1 && current == desired {
                if !preserve {
                    SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(0)));
                    let first = SendMessageW(hwnd, EM_GETFIRSTVISIBLELINE, None, None);
                    SendMessageW(hwnd, EM_LINESCROLL, None, Some(LPARAM(-first.0)));
                }
                return Ok(());
            }
        }
        let (mut start, mut end) = (0u32, 0u32);
        let first = if preserve {
            SendMessageW(hwnd, EM_GETSEL,
                Some(WPARAM((&mut start as *mut u32) as usize)),
                Some(LPARAM((&mut end as *mut u32) as isize)));
            Some(SendMessageW(hwnd, EM_GETFIRSTVISIBLELINE, None, None))
        } else {
            None
        };
        set_text_checked(hwnd, text)?;
        if let Some(first) = first {
            SendMessageW(hwnd, EM_SETSEL,
                Some(WPARAM((start as usize).min(desired.len() - 1))),
                Some(LPARAM((end as usize).min(desired.len() - 1) as isize)));
            let current = SendMessageW(hwnd, EM_GETFIRSTVISIBLELINE, None, None);
            SendMessageW(hwnd, EM_LINESCROLL, None, Some(LPARAM(first.0 - current.0)));
        } else {
            SendMessageW(hwnd, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(0)));
            let first = SendMessageW(hwnd, EM_GETFIRSTVISIBLELINE, None, None);
            SendMessageW(hwnd, EM_LINESCROLL, None, Some(LPARAM(-first.0)));
        }
        Ok(())
    })();
    if result.is_err() {
        state.text_fault.set(true);
    }
}

unsafe fn create_controls(parent: HWND, instance: HINSTANCE) -> Result<[HWND; CONTROL_COUNT]> {
    let mut controls = [HWND::default(); CONTROL_COUNT];
    for index in 0..CONTROL_COUNT {
        let (class, label, extra, ex_style) = match index {
            GM_COUNT => (
                w!("EDIT"),
                "1",
                WS_TABSTOP.0 | ES_AUTOHSCROLL as u32 | ES_NUMBER as u32,
                WS_EX_CLIENTEDGE,
            ),
            GM_SEND => (
                w!("BUTTON"),
                "送出假人操作（伺服器查驗 GM）",
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            ACCOUNT => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | ES_AUTOHSCROLL as u32,
                WS_EX_CLIENTEDGE,
            ),
            PASSWORD => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | ES_AUTOHSCROLL as u32 | ES_PASSWORD as u32,
                WS_EX_CLIENTEDGE,
            ),
            DETAILS => (
                w!("EDIT"),
                "尚未選取角色。",
                WS_TABSTOP.0
                    | WS_VSCROLL.0
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
                WS_EX_CLIENTEDGE,
            ),
            VIP_STATUS_STRIP => (
                w!("STATIC"),
                "",
                STATIC_OWNER_DRAW,
                WINDOW_EX_STYLE::default(),
            ),
            VIP_ITEMS => (
                w!("LISTBOX"),
                "",
                WS_VSCROLL.0 | LBS_OWNERDRAWFIXED as u32 | LBS_HASSTRINGS as u32
                    | LBS_NOTIFY as u32 | WS_TABSTOP.0,
                WS_EX_CLIENTEDGE,
            ),
            SHOW_SKILLS | SKILLS_READ | CAST_TELEPORT | SHOW_RECORD | RECORD_READ | SHOP_OPEN => (
                w!("BUTTON"),
                match index {
                    SHOW_SKILLS => "魔法／技能(&P)",
                    SKILLS_READ => "重新讀取技能",
                    SHOW_RECORD => "中央戰績",
                    RECORD_READ => "刷新戰績",
                    SHOP_OPEN => "本場補給商店",
                    _ => "瞬間移動（T）",
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            SHOW_STATS | SHOW_ITEMS => (
                w!("BUTTON"),
                if index == SHOW_STATS {
                    "素質／裝備加成(&C)"
                } else {
                    "背包資料(&I)"
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            GEAR_EQUIP | GEAR_UNEQUIP | GEAR_READ => (
                w!("BUTTON"),
                match index {
                    GEAR_EQUIP => "穿上",
                    GEAR_UNEQUIP => "脫下",
                    _ => "重新讀取裝備",
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            CONNECT => (
                w!("BUTTON"),
                "連線",
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            LOGIN => (
                w!("BUTTON"),
                "登入",
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            DISCONNECT => (
                w!("BUTTON"),
                "中斷／取消操作",
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            EXIT | SETTINGS | CHAT => (
                w!("BUTTON"),
                if index == CHAT {
                    "聊天"
                } else if index == SETTINGS {
                    "遊戲設定"
                } else {
                    "退出"
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            SELECT_CHARACTER | ENTER_CHANNEL | RETURN_CHARACTERS => (
                w!("BUTTON"),
                match index {
                    SELECT_CHARACTER => "選擇角色",
                    ENTER_CHANNEL => "進入大廳",
                    _ => "返回選角",
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            SOURCE | GEAR_ITEM => (
                w!("COMBOBOX"),
                "",
                WS_TABSTOP.0 | WS_VSCROLL.0 | CBS_DROPDOWNLIST as u32,
                WINDOW_EX_STYLE::default(),
            ),
            CHARACTERS | CHANNELS | ROOM_LIST => (
                w!("LISTBOX"),
                "",
                WS_TABSTOP.0 | WS_VSCROLL.0 | LBS_NOTIFY as u32 | LBS_NOINTEGRALHEIGHT as u32,
                WS_EX_CLIENTEDGE,
            ),
            ROOM_CREATE | ROOM_JOIN | ROOM_CONFIGURE | ROOM_LEAVE | ROOM_TEAM0 | ROOM_TEAM1
            | ROOM_READY | ROOM_READ | MATCH_START | MATCH_CANCEL => (
                w!("BUTTON"),
                match index {
                    ROOM_CREATE => "建立房間",
                    ROOM_JOIN => "加入房間",
                    ROOM_CONFIGURE => "房間設定",
                    ROOM_LEAVE => "離開房間",
                    ROOM_TEAM0 => "加入隊伍 A",
                    ROOM_TEAM1 => "加入隊伍 B",
                    ROOM_READY => "準備／取消準備",
                    MATCH_START => "配對",
                    MATCH_CANCEL => "取消配對",
                    _ => "重新讀取房間",
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            _ => (
                w!("STATIC"),
                match index {
                    ACCOUNT_LABEL => "帳號",
                    PASSWORD_LABEL => "密碼",
                    SOURCE_LABEL => "資料來源／伺服器（切換即重新載入角色）",
                    GM_LABEL => "GM 假人：1–100 新增，0 清除房內假人（不代表目前帳號有權限）",
                    CHARACTER_LABEL => "角色列表（選取後按「選擇角色」）",
                    DETAILS => "尚未選取角色。",
                    FOOTNOTE => "功能依本場規則開放；未實作技能不可施放。",
                    _ => "",
                },
                STATIC_NO_PREFIX
                    | if index == ROOT {
                        STATIC_PATH_ELLIPSIS
                    } else {
                        0
                    },
                WINDOW_EX_STYLE::default(),
            ),
        };
        let label = wide(label);
        controls[index] = CreateWindowExW(
            ex_style,
            class,
            PCWSTR(label.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | if matches!(index, VIP_STATUS_STRIP | VIP_ITEMS) { 0 } else { WS_VISIBLE.0 } | extra),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU((FIRST_ID + index) as *mut _)),
            Some(instance),
            None,
        )?;
    }
    let _ = SendMessageW(controls[GM_COUNT], EM_SETLIMITTEXT, Some(WPARAM(3)), None);
    let _ = SendMessageW(
        controls[ACCOUNT],
        EM_SETLIMITTEXT,
        Some(WPARAM(ACCOUNT_UTF16_LIMIT)),
        None,
    );
    let _ = SendMessageW(
        controls[PASSWORD],
        EM_SETLIMITTEXT,
        Some(WPARAM(PASSWORD_UTF16_LIMIT)),
        None,
    );
    let _ = SendMessageW(
        controls[DETAILS],
        EM_SETLIMITTEXT,
        Some(WPARAM(MAX_DETAILS_UTF16)),
        None,
    );
    Ok(controls)
}

unsafe fn replace_font(hwnd: HWND, state: &WindowState) -> Result<()> {
    let font = CreateFontW(
        -scale(16, GetDpiForWindow(hwnd)),
        0,
        0,
        0,
        400,
        0,
        0,
        0,
        DEFAULT_CHARSET,
        OUT_DEFAULT_PRECIS,
        CLIP_DEFAULT_PRECIS,
        CLEARTYPE_QUALITY,
        0,
        w!("Segoe UI"),
    );
    ensure!(!font.0.is_null(), "create DPI-scaled UI font");
    if let Some(controls) = state.controls.get() {
        for control in controls {
            let _ = SendMessageW(
                control,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
    }
    let previous = state.font.replace(font);
    if !previous.0.is_null() {
        let _ = DeleteObject(previous.into());
    }
    Ok(())
}

unsafe fn resize(hwnd: HWND, state: &WindowState) {
    let Some(controls) = state.controls.get() else {
        return;
    };
    let mut client = RECT::default();
    if GetClientRect(hwnd, &mut client).is_err() || client.right <= 0 || client.bottom <= 0 {
        return;
    }
    let dpi = GetDpiForWindow(hwnd);
    let boxes = arranged_layout(client.right, client.bottom, dpi,
        state.vip_status_visible.get(), state.vip_inventory_visible.get());
    let _ = SendMessageW(controls[VIP_ITEMS], LB_SETITEMHEIGHT,
        Some(WPARAM(0)), Some(LPARAM(scale(40, dpi) as isize)));
    for (control, rect) in controls.into_iter().zip(boxes) {
        let _ = MoveWindow(control, rect.x, rect.y, rect.width, rect.height, true);
    }
}

fn arranged_layout(width: i32, height: i32, dpi: u32,
    status_visible: bool, inventory_visible: bool) -> [BoxRect; CONTROL_COUNT] {
    let mut boxes = layout(width, height, dpi);
    if status_visible {
        boxes[DETAILS].y += scale(40, dpi);
        boxes[DETAILS].height -= scale(40, dpi);
    }
    if inventory_visible {
        let list_height = scale(96, dpi).min((boxes[DETAILS].height / 2).max(0));
        boxes[VIP_ITEMS] = BoxRect { x: boxes[DETAILS].x, y: boxes[DETAILS].y,
            width: boxes[DETAILS].width, height: list_height };
        boxes[DETAILS].y += list_height + scale(4, dpi);
        boxes[DETAILS].height = (boxes[DETAILS].height - list_height - scale(4, dpi)).max(0);
    }
    boxes
}

unsafe fn clear_credentials(state: &WindowState, clear_account: bool) {
    if let Some(controls) = state.controls.get() {
        paint_text(state, controls[PASSWORD], "");
        let _ = SendMessageW(controls[PASSWORD], EM_EMPTYUNDOBUFFER, None, None);
        if clear_account {
            paint_text(state, controls[ACCOUNT], "");
            let _ = SendMessageW(controls[ACCOUNT], EM_EMPTYUNDOBUFFER, None, None);
        }
    }
}

unsafe fn read_edit(hwnd: HWND, max_units: usize) -> std::result::Result<String, &'static str> {
    let length = GetWindowTextLengthW(hwnd);
    if length < 0 || length as usize > max_units {
        return Err("輸入長度超出限制。");
    }
    let mut text = vec![0u16; length as usize + 1];
    let count = GetWindowTextW(hwnd, &mut text);
    let result = if count < 0 || count as usize != length as usize {
        Err("無法完整讀取輸入內容。")
    } else if char::decode_utf16(text[..count as usize].iter().copied()).any(|unit| unit.is_err()) {
        // Validate without allocating a partial UTF-8 String that an invalid
        // surrogate could otherwise drop before we install a zeroizing guard.
        Err("輸入內容不是有效的 Unicode。")
    } else {
        String::from_utf16(&text[..count as usize]).map_err(|_| "輸入內容不是有效的 Unicode。")
    };
    // Best-effort wipe of our UTF-16 copy; Windows owns the control's storage.
    for unit in &mut text {
        std::ptr::write_volatile(unit, 0);
    }
    result
}

unsafe fn local_error(hwnd: HWND, state: &WindowState, error: &str) {
    if let Ok(mut ui) = state.ui.try_borrow_mut() {
        ui.local_error = Some(error.chars().filter(|c| *c != '\0').take(512).collect());
    }
    refresh(hwnd, state);
}

unsafe fn submit_login(hwnd: HWND, state: &WindowState) {
    if state.text_fault.get() {
        return;
    }
    let ready = state.ui.try_borrow().ok().is_some_and(|ui| {
        !ui.closing
            && ui
                .last
                .as_ref()
                .is_some_and(|s| s.phase == LoginPhase::LoginReady)
    });
    if !ready {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    let credentials = (|| {
        let account = Zeroizing::new(read_edit(controls[ACCOUNT], ACCOUNT_UTF16_LIMIT)?);
        let password = Zeroizing::new(read_edit(controls[PASSWORD], PASSWORD_UTF16_LIMIT)?);
        validated_login_command(account, password)
    })();
    clear_credentials(state, false);
    match credentials {
        Ok(command) => send(hwnd, state, command),
        Err(error) => local_error(hwnd, state, error),
    }
}

unsafe fn send(hwnd: HWND, state: &WindowState, mut command: LoginCommand) {
    if matches!(command, LoginCommand::Disconnect | LoginCommand::Shutdown) {
        cancel_room_form(state);
        cancel_shop(state);
        clear_credentials(state, true);
    } else if matches!(command, LoginCommand::SelectSource(_)) {
        clear_credentials(state, false);
    }
    let outcome = {
        let Ok(mut ui) = state.ui.try_borrow_mut() else {
            zeroize_unsubmitted(&mut command);
            return;
        };
        if ui.closing || state.text_fault.get() {
            zeroize_unsubmitted(&mut command);
            return;
        }
        ui.local_error = None;
        ui.controller.command(command)
    };
    if let Err(error) = outcome {
        local_error(hwnd, state, &error);
    } else {
        refresh(hwnd, state);
    }
}

fn cancel_room_form(state: &WindowState) {
    let form = state.room_form.borrow_mut().take();
    if let Some(mut form) = form {
        form.cancel();
        if !form.is_quiescent() {
            *state.room_form.borrow_mut() = Some(form);
        } else {
            state.room_form_context.set(None);
        }
    }
}
fn cancel_shop(state: &WindowState) {
    let shop = state.shop_window.borrow_mut().take();
    if let Some(mut shop) = shop {
        shop.cancel();
        *state.shop_window.borrow_mut() = Some(shop);
    }
}
fn shop_view(state: &WindowState, snapshot: &LoginSnapshot) -> Option<super::shop_window::View> {
    if snapshot.phase != LoginPhase::InLobby
        || snapshot.world_owns_lobby()
        || matches!(snapshot.shop, shop::State::Unsupported)
    {
        return None;
    }
    let nav = snapshot.navigation.as_ref()?;
    let rooms::State::Ready(room) = &snapshot.rooms else {
        return None;
    };
    let request = shop::Request::read(nav.context, nav.snapshot.channel_id?, room.joined.as_ref());
    let labels = {
        let descriptions = state.descriptions.borrow();
        snapshot
            .shop
            .snapshot()
            .map_or_else(std::collections::BTreeMap::new, |s| {
                s.offers
                    .iter()
                    .map(|o| (o.item_id, descriptions.state.item_label(&o.name)))
                    .collect()
            })
    };
    let status = state
        .ui
        .try_borrow()
        .ok()
        .and_then(|ui| ui.local_error.clone())
        .unwrap_or_else(|| snapshot.status.clone());
    Some(super::shop_window::View {
        request,
        state: snapshot.shop.clone(),
        pending: snapshot.shop_pending,
        status,
        labels,
    })
}
unsafe fn open_shop(hwnd: HWND, state: &WindowState) {
    if state.shop_window.borrow().is_some() || state.room_form.borrow().is_some() {
        return;
    }
    let snapshot = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        if ui.closing {
            return;
        }
        ui.controller.snapshot()
    };
    if snapshot.equipment_pending
        || snapshot.room_pending
        || snapshot.world_pending
        || snapshot.combat_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
    {
        return;
    }
    let Some(view) = shop_view(state, &snapshot) else {
        return;
    };
    let request = view.request;
    match ShopWindow::open(hwnd, &state.shared_root, view) {
        Ok(shop) => {
            *state.shop_window.borrow_mut() = Some(shop);
            send(hwnd, state, LoginCommand::Shop(request));
        }
        Err(_) => local_error(hwnd, state, "無法開啟本場補給商店。"),
    }
}
unsafe fn refresh_shop(
    hwnd: HWND,
    state: &WindowState,
    snapshot: &LoginSnapshot,
    closing: bool,
) -> bool {
    let shop = state.shop_window.borrow_mut().take();
    let Some(mut shop) = shop else { return true };
    let view = shop_view(state, snapshot);
    // A cancelled shop may still own its icon worker after TLS reconnects.
    // Scope retirement must not apply the new character's data or be reported
    // as a fatal text error; refresh still polls the real worker to completion.
    let result = shop.refresh(view, closing);
    if result.is_err() {
        state.text_fault.set(true);
        shop.cancel();
    }
    let quiet = shop.is_quiescent();
    if !quiet {
        *state.shop_window.borrow_mut() = Some(shop);
    } else {
        drop(shop);
        let _ = SetFocus(Some(hwnd));
    }
    quiet
}
fn form_context_current(common: rooms::Common, snapshot: &LoginSnapshot) -> bool {
    snapshot.phase == LoginPhase::InLobby
        && snapshot.navigation.as_ref().is_some_and(|nav| {
            nav.context == common.context && nav.snapshot.channel_id == Some(common.channel_id)
        })
}
fn room_surface(snapshot: &LoginSnapshot) -> bool {
    snapshot.phase == LoginPhase::InLobby && matches!(snapshot.rooms, rooms::State::Ready(_))
}
fn source_selector_visible(snapshot: &LoginSnapshot) -> bool {
    snapshot.phase != LoginPhase::InLobby
}
fn unsupported_room_hint(snapshot: &LoginSnapshot) -> Option<&'static str> {
    (snapshot.phase == LoginPhase::InLobby && matches!(snapshot.rooms, rooms::State::Unsupported))
        .then_some("此伺服器尚未提供房間功能；仍可檢視角色、操作裝備或返回頻道。")
}
fn room_ready_value(snapshot: &LoginSnapshot) -> u8 {
    let rooms::State::Ready(directory) = &snapshot.rooms else {
        return 1;
    };
    u8::from(
        !directory
            .joined
            .as_ref()
            .and_then(|room| room.member(directory.common.context.key))
            .is_some_and(|member| member.ready),
    )
}
fn room_directory_label(reply: &rooms::Reply) -> String {
    if let Some(room) = &reply.joined {
        let available = reply
            .mode_option(room.summary.settings.mode, room.summary.settings.map_id)
            .is_some_and(|o| o.world_available);
        format!(
            "戰鬥室 #{} {} · 版本{} · {}",
            room.summary.room_id,
            room.summary.settings.name,
            room.summary.revision,
            if !available {
                "地圖未開放；請更換地圖或離房"
            } else if room.summary.satisfied {
                "準備條件滿足"
            } else {
                "等待準備"
            }
        )
    } else {
        let available = reply.options.iter().filter(|o| o.world_available).count();
        format!(
            "房間列表 · {} 間 · 已綁定地圖 {}/{}{}",
            reply.rooms.len(),
            available,
            reply.options.len(),
            if available == 0 {
                " · 尚無可進場地圖，可重新整理或返回頻道"
            } else {
                ""
            }
        )
    }
}
pub(crate) fn room_common(
    snapshot: &LoginSnapshot,
    action: rooms::Action,
    selected: Option<u64>,
    value: u8,
) -> Option<rooms::Common> {
    if snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
        || snapshot.phase != LoginPhase::InLobby
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.room_pending
        || snapshot.equipment_pending
        || snapshot.world_owns_lobby()
    {
        return None;
    }
    let rooms::State::Ready(directory) = &snapshot.rooms else {
        return None;
    };
    let joined = directory.joined.as_ref();
    let row = match action {
        rooms::Action::Read => None,
        rooms::Action::Create
            if joined.is_none() && directory.options.iter().any(|o| o.world_available) =>
        {
            None
        }
        rooms::Action::Join if joined.is_none() => Some(
            directory
                .rooms
                .iter()
                .find(|room| Some(room.room_id) == selected)?,
        ),
        rooms::Action::Leave
        | rooms::Action::Team
        | rooms::Action::Ready
        | rooms::Action::Configure => Some(&joined?.summary),
        _ => return None,
    };
    if action == rooms::Action::Configure && joined?.owner != directory.common.context.key {
        return None;
    }
    if (action == rooms::Action::Join || action == rooms::Action::Ready && value == 1)
        && !directory
            .mode_option(row?.settings.mode, row?.settings.map_id)?
            .world_available
    {
        return None;
    }
    if action == rooms::Action::Team
        && !directory
            .mode_option(row?.settings.mode, row?.settings.map_id)?
            .team_based
    {
        return None;
    }
    Some(rooms::Common {
        context: directory.common.context,
        channel_id: directory.common.channel_id,
        action,
        value,
        room_id: row.map(|row| row.room_id).unwrap_or(0),
        expected_revision: row.map(|row| row.revision).unwrap_or(0),
    })
}
fn matchmaking_common(snapshot: &LoginSnapshot, cancel: bool) -> Option<rooms::Common> {
    if !snapshot.matchmaking.supported() {
        return None;
    }
    let common = room_common(snapshot, rooms::Action::Read, None, 0)?;
    let rooms::State::Ready(directory) = &snapshot.rooms else {
        return None;
    };
    if directory.joined.is_some() {
        return None;
    }
    if cancel {
        snapshot.matchmaking.waiting()?;
    } else if snapshot.matchmaking.waiting().is_some()
        || !directory.options.iter().any(|o| o.world_available)
    {
        return None;
    }
    Some(common)
}
unsafe fn matchmaking_click(hwnd: HWND, state: &WindowState, cancel: bool) {
    if state.room_form.borrow().is_some() || state.shop_window.borrow().is_some() {
        return;
    }
    let Some(snapshot) = state.ui.borrow().last.clone() else {
        return;
    };
    let Some(common) = matchmaking_common(&snapshot, cancel) else {
        return;
    };
    if cancel {
        let ticket_id = snapshot.matchmaking.waiting().unwrap().ticket_id;
        send(
            hwnd,
            state,
            LoginCommand::Matchmaking(matching::Request {
                context: common.context,
                channel_id: common.channel_id,
                action: matching::Action::Cancel,
                ticket_id,
                criteria: None,
            }),
        );
    } else if let rooms::State::Ready(directory) = &snapshot.rooms {
        match RoomForm::open_matchmaking(hwnd, common, directory.options.clone()) {
            Ok(form) => {
                *state.room_form.borrow_mut() = Some(form);
                state.room_form_context.set(Some(common));
            }
            Err(_) => local_error(hwnd, state, "無法開啟配對條件視窗，尚未排隊。"),
        }
        refresh(hwnd, state);
    }
}
unsafe fn room_click(hwnd: HWND, state: &WindowState, button: usize) {
    if state.room_form.borrow().is_some() {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    let snapshot = state.ui.borrow().last.clone();
    let Some(snapshot) = snapshot else {
        return;
    };
    let rooms::State::Ready(directory) = &snapshot.rooms else {
        return;
    };
    let index = SendMessageW(controls[ROOM_LIST], LB_GETCURSEL, None, None).0;
    let selected = usize::try_from(index)
        .ok()
        .and_then(|index| directory.rooms.get(index))
        .map(|row| row.room_id);
    let (action, value) = match button {
        ROOM_CREATE => (rooms::Action::Create, 0),
        ROOM_JOIN => (rooms::Action::Join, 0),
        ROOM_CONFIGURE => (rooms::Action::Configure, 0),
        ROOM_LEAVE => (rooms::Action::Leave, 0),
        ROOM_TEAM0 => (rooms::Action::Team, 0),
        ROOM_TEAM1 => (rooms::Action::Team, 1),
        ROOM_READY => (rooms::Action::Ready, room_ready_value(&snapshot)),
        ROOM_READ => (rooms::Action::Read, 0),
        _ => return,
    };
    let Some(common) = room_common(&snapshot, action, selected, value) else {
        return;
    };
    if matches!(
        action,
        rooms::Action::Create | rooms::Action::Configure | rooms::Action::Join
    ) {
        let initial = if action == rooms::Action::Create {
            None
        } else {
            directory
                .rooms
                .iter()
                .find(|row| row.room_id == common.room_id)
                .map(|row| row.settings.clone())
        };
        match RoomForm::open(hwnd, common, directory.options.clone(), initial) {
            Ok(form) => {
                *state.room_form.borrow_mut() = Some(form);
                state.room_form_context.set(Some(common));
            }
            Err(_) => local_error(hwnd, state, "無法開啟房間設定視窗，尚未送出任何操作。"),
        }
        refresh(hwnd, state);
    } else if let Ok(request) = rooms::Request::simple(common) {
        send(hwnd, state, LoginCommand::Rooms(request));
    }
}

unsafe fn request_close(hwnd: HWND, state: &WindowState) {
    let chat = state.chat_window.borrow_mut().take();
    drop(chat);
    drop(state.direct_chat_window.borrow_mut().take());
    state.settings.borrow_mut().close();
    cancel_shop(state);
    state.world_audio.borrow_mut().stop();
    state.world_runtime.borrow_mut().shutdown();
    state.battle_audio.borrow_mut().stop();
    state.battle_runtime.borrow_mut().shutdown();
    cancel_room_form(state);
    state.descriptions.borrow().cancel();
    state.vip_status.borrow().cancel();
    state.vip_inventory.borrow().cancel();
    clear_credentials(state, true);
    {
        let Ok(mut ui) = state.ui.try_borrow_mut() else {
            return;
        };
        ui.begin_close();
    }
    refresh(hwnd, state);
}

fn world_request(snapshot: &LoginSnapshot, action: world::Action) -> Option<world::Request> {
    if snapshot.phase != LoginPhase::InLobby
        || snapshot.world_pending
        || snapshot.room_pending
        || snapshot.equipment_pending
        || snapshot.combat_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
    {
        return None;
    }
    let world::State::Ready(reply) = &snapshot.world else {
        return None;
    };
    let frame = reply.frame.as_ref()?;
    let mut request = world::Request::read(reply.echo.context, reply.echo.channel_id);
    if action == world::Action::Move
        && frame.race.is_some()
        && (frame.phase != world::Phase::Active
            || !frame.actors.iter().any(|a| {
                a.matches_character(reply.echo.context.key) && frame.actor_occupies(a.entity_id)
            }))
    {
        return None;
    }
    request.action = action;
    request.match_id = frame.match_id;
    request.generation = frame.generation;
    Some(request)
}
#[cfg(test)]
fn can_cast_teleport(snapshot: &LoginSnapshot) -> bool {
    can_cast_skill(snapshot, super::spell_protocol::TELEPORT)
}
/// Match support, not permission to cast: keep a learned button visible while
/// loading, dead, on cooldown, or awaiting a receipt. Its enabled state still
/// uses the normal singleflight/life/cadence and painted-scene checks.
pub(crate) fn fire_storm_available(snapshot: &LoginSnapshot, entity: u64) -> bool {
    let (Some(nav), Some(frame)) = (&snapshot.navigation, &snapshot.combat) else {
        return false;
    };
    if snapshot.skills.fire_storm().is_none()
        || !matches!(&snapshot.skills, super::spell_protocol::SkillsState::Ready(s) if s.context == nav.context)
        || snapshot.selected_source != Some(nav.context.key.source_id)
        || nav.snapshot.selected.as_ref().map(|c| c.key) != Some(nav.context.key)
        || snapshot.world.frame() != Some(&frame.world)
        || frame
            .world
            .actor(entity)
            .is_none_or(|a| !a.matches_character(nav.context.key))
    {
        return false;
    }
    if snapshot.world_npc_enabled() {
        // NPC70 has its own authoritative section and can never also have the
        // PvP70 section. Do not hide its button by looking only in frame.fires.
        frame.fire(entity).is_none()
            && frame.npc_fire(entity).is_some()
            && snapshot.npc.as_ref().is_some_and(|reply| {
                reply.echo.context == nav.context
                    && Some(reply.echo.channel_id) == nav.snapshot.channel_id
                    && reply
                        .frame
                        .as_ref()
                        .is_some_and(|npc| &npc.combat == frame.as_ref())
            })
    } else {
        frame.fire(entity).is_some() && frame.npc_fire(entity).is_none()
    }
}
fn can_cast_skill(snapshot: &LoginSnapshot, skill: u32) -> bool {
    if snapshot.phase != LoginPhase::InLobby
        || match skill {
            super::spell_protocol::TELEPORT => snapshot.skills.teleport().is_none(),
            super::spell_protocol::LIGHT_ARROW => snapshot.skills.light_arrow().is_none(),
            super::spell_protocol::BASIC_HEAL => snapshot.skills.basic_heal().is_none(),
            super::spell_protocol::FIRE_STORM => snapshot.skills.fire_storm().is_none(),
            super::spell_protocol::DISINTEGRATE => snapshot.skills.disintegrate().is_none(),
            _ => true,
        }
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
        || snapshot.world_pending
        || snapshot.combat_pending
        || snapshot.equipment_pending
        || snapshot.room_pending
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
    {
        return false;
    }
    let Some(frame) = snapshot.combat.as_ref() else {
        return false;
    };
    let Some(nav) = snapshot.navigation.as_ref() else {
        return false;
    };
    snapshot.world.frame() == Some(&frame.world)
        && (!matches!(
            skill,
            super::spell_protocol::DISINTEGRATE | super::spell_protocol::FIRE_STORM
        ) || matches!(&snapshot.skills, super::spell_protocol::SkillsState::Ready(s) if s.context == nav.context))
        && frame.world.phase == world::Phase::Active
        && frame
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(nav.context.key))
            .is_some_and(|a| {
                frame.world.cast(a.entity_id).is_some()
                    && frame.weight(a.entity_id).is_none_or(|w| w.can_attack())
                    && (skill != super::spell_protocol::LIGHT_ARROW
                        || frame.damage(a.entity_id).is_some()
                        || frame.npc_spell(a.entity_id).is_some()
                            && snapshot
                                .npc
                                .as_ref()
                                .and_then(|n| n.frame.as_ref())
                                .is_some_and(|n| &n.combat == frame.as_ref()))
                    && (skill != super::spell_protocol::BASIC_HEAL || frame.honor.is_some())
                    && (skill != super::spell_protocol::FIRE_STORM
                        || fire_storm_available(snapshot, a.entity_id)
                            && (!snapshot.world_npc_enabled()
                                || npc_fire_storm_input_request(snapshot).is_some()))
                    && (skill != super::spell_protocol::DISINTEGRATE
                        || frame
                            .disintegrate(a.entity_id)
                            .map(|d| (d.mp_cost, d.lawful))
                            .or_else(|| {
                                snapshot
                                    .npc
                                    .as_ref()
                                    .and_then(|n| n.frame.as_ref())
                                    .filter(|n| &n.combat == frame.as_ref())
                                    .and_then(|_| frame.npc_disintegrate(a.entity_id))
                                    .map(|d| (d.mp_cost, d.lawful))
                            })
                            .is_some_and(|(cost, lawful)| {
                                lawful >= 500
                                    && frame
                                        .vital(a.entity_id)
                                        .is_some_and(|v| v.mp >= i64::from(cost))
                            }))
                    && frame.vital(a.entity_id).is_some_and(|v| v.hp > 0)
            })
}

fn disintegrate_input_request(
    snapshot: &LoginSnapshot,
    combat: &super::combat_protocol::Frame,
    surface: super::world_window::HitSurface,
    target_entity_id: u64,
    caster_life: u64,
    target_life: u64,
) -> Option<super::disintegrate_cast_protocol::Request> {
    let nav = snapshot.navigation.as_ref()?;
    if snapshot.world.frame() != Some(&combat.world)
        || snapshot.combat.as_deref() != Some(combat)
        || surface.match_id != combat.world.match_id
        || surface.generation != combat.world.generation
        || surface.revision > combat.world.revision
        || surface.revision < 1
        || !combat
            .world
            .actor(surface.selected_entity_id)?
            .matches_character(nav.context.key)
    {
        return None;
    }
    let request = super::disintegrate_cast_protocol::Request {
        context: nav.context,
        channel_id: nav.snapshot.channel_id?,
        match_id: surface.match_id,
        generation: surface.generation,
        sequence: combat
            .world
            .cast(surface.selected_entity_id)?
            .request_sequence
            .checked_add(1)?,
        target_entity_id,
        caster_life,
        target_life,
    };
    request.ready_in(combat).then_some(request)
}
pub(crate) fn npc_fire_storm_input_request(
    snapshot: &LoginSnapshot,
) -> Option<super::npc_fire_storm_protocol::Request> {
    let nav = snapshot.navigation.as_ref()?;
    let reply = snapshot.npc.as_ref()?;
    let f = reply.frame.as_ref()?;
    if reply.echo.context != nav.context
        || nav.snapshot.channel_id != Some(reply.echo.channel_id)
        || snapshot.world.frame() != Some(&f.combat.world)
        || snapshot.combat.as_deref() != Some(&f.combat)
    {
        return None;
    }
    let actor = f
        .combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(nav.context.key))?;
    let r = super::npc_fire_storm_protocol::Request {
        context: nav.context,
        channel_id: nav.snapshot.channel_id?,
        scope: f.scope,
        sequence: f
            .combat
            .world
            .cast(actor.entity_id)?
            .request_sequence
            .checked_add(1)?,
        caster_life: super::quest_life::life(&f.combat.world, actor.entity_id)
            .map_or(1, |l| l.incarnation()),
    };
    r.ready_in(f).then_some(r)
}
fn npc_disintegrate_input_request(
    snapshot: &LoginSnapshot,
    surface: super::world_window::HitSurface,
    target: super::npc_protocol::Id,
    caster_life: u64,
    target_life: u64,
) -> Option<super::npc_disintegrate_protocol::Request> {
    let nav = snapshot.navigation.as_ref()?;
    let f = snapshot.npc.as_ref()?.frame.as_ref()?;
    let w = &f.combat.world;
    if snapshot.world.frame() != Some(w)
        || snapshot.combat.as_deref() != Some(&f.combat)
        || surface.match_id != w.match_id
        || surface.generation != w.generation
        || surface.revision < 1
        || surface.revision > w.revision
        || !w
            .actor(surface.selected_entity_id)?
            .matches_character(nav.context.key)
    {
        return None;
    }
    let r = super::npc_disintegrate_protocol::Request {
        context: nav.context,
        channel_id: nav.snapshot.channel_id?,
        target,
        caster_life,
        target_life,
        sequence: w
            .cast(surface.selected_entity_id)?
            .request_sequence
            .checked_add(1)?,
    };
    r.ready_in(f).then_some(r)
}
fn submit_world(state: &WindowState, request: world::Request) -> bool {
    submit_world_bound(state, request, None)
}
fn submit_world_bound(
    state: &WindowState,
    request: world::Request,
    intent: Option<super::battle_window::Intent>,
) -> bool {
    let Ok(mut ui) = state.ui.try_borrow_mut() else {
        return false;
    };
    match ui
        .controller
        .command(super::battle_player::bind_local_command(
            intent,
            LoginCommand::World(request),
        )) {
        Ok(()) => {
            ui.local_error = None;
            true
        }
        Err(error) => {
            ui.local_error = Some(error);
            false
        }
    }
}

fn combat_alive(supported: bool, vital: Option<&super::combat_protocol::Vital>) -> bool {
    vital.map_or(!supported, |v| v.hp > 0)
}
fn combat_dead(vital: Option<&super::combat_protocol::Vital>) -> bool {
    vital.is_some_and(|v| v.hp == 0)
}
fn honor_respawn_notice(phase: world::Phase, dead: bool) -> &'static str {
    if phase == world::Phase::Active && dead {
        "等待伺服器原地復活；本機不復活、不恢復MP。"
    } else {
        ""
    }
}
fn ammo_input_ready(snapshot: &LoginSnapshot, entity: u64, item_id: i64) -> bool {
    if snapshot.phase != LoginPhase::InLobby
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
        || snapshot.equipment_pending
        || snapshot.room_pending
        || snapshot.world_pending
        || snapshot.combat_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
    {
        return false;
    }
    snapshot.combat.as_ref().is_some_and(|c| {
        snapshot.world.frame() == Some(&c.world)
            && c.world.phase == world::Phase::Active
            && c.vital(entity).is_some_and(|v| v.hp > 0)
            && super::ammo_protocol::supported(item_id)
            && c.ammo(entity)
                .and_then(|s| s.stock(item_id))
                .is_some_and(|s| s.remaining > 0)
    })
}
fn item_input_ready(
    snapshot: &LoginSnapshot,
    entity: u64,
    item_id: i64,
    last: Option<(u64, u64, u64, i64, Instant, u32)>,
    now: Instant,
) -> bool {
    if snapshot.phase != LoginPhase::InLobby
        || snapshot.shop_pending
        || snapshot.item_pending
        || snapshot.ammo_pending
        || snapshot.equipment_pending
        || snapshot.room_pending
        || snapshot.world_pending
        || snapshot.combat_pending
        || snapshot.spell_pending
        || snapshot.conversation_pending()
        || snapshot.record_pending
    {
        return false;
    }
    let Some(combat) = snapshot.combat.as_ref().filter(|c| {
        snapshot.world.frame() == Some(&c.world) && c.world.phase == world::Phase::Active
    }) else {
        return false;
    };
    if !combat.vital(entity).is_some_and(|v| v.hp > 0)
        || !combat
            .item(entity)
            .and_then(|s| s.stock(item_id))
            .is_some_and(|s| s.remaining > 0)
    {
        return false;
    }
    if let Some(kind) = super::timed_protocol::Kind::from_item(item_id) {
        return !combat.timed.is_empty() && timed_item_class_eligible(&combat.world, entity, kind);
    }
    if item::healing(item_id).is_none() {
        return false;
    }
    last.filter(|p| (p.0, p.1, p.2) == (combat.world.match_id, combat.world.generation, entity))
        .is_none_or(|p| attack_interval_ready(Some(p.4), u64::from(p.5) * 1_000_000, now))
}
fn timed_item_class_eligible(
    frame: &world::Frame,
    entity: u64,
    kind: super::timed_protocol::Kind,
) -> bool {
    // A projected force keeps global entity IDs; its visible actor index is
    // not the index in the full public roster. Hidden/unknown actors fail closed.
    frame.actor(entity).is_some() && frame.member(entity).is_some_and(|m| kind.eligible(m.class))
}
#[cfg(test)]
fn action_cadence_ready(
    last_move: Option<Instant>,
    step_nanos: u64,
    last_attack: Option<Instant>,
    vital: Option<&super::combat_protocol::Vital>,
    now: Instant,
) -> bool {
    attack_interval_ready(last_move, step_nanos, now)
        && vital.is_none_or(|v| attack_interval_ready(last_attack, v.attack_interval_nanos, now))
}
fn attack_interval_ready(last: Option<Instant>, interval_nanos: u64, now: Instant) -> bool {
    last.is_none_or(|last| {
        now.checked_duration_since(last)
            .is_some_and(|elapsed| elapsed.as_nanos() >= u128::from(interval_nanos))
    })
}
#[cfg(test)]
fn cast_interval_ready(last: Option<(Instant, u32)>, now: Instant) -> bool {
    last.is_none_or(|(receipt, millis)| {
        attack_interval_ready(Some(receipt), u64::from(millis) * 1_000_000, now)
    })
}
fn confirmed_cadence_ready(
    combat: Option<&super::combat_protocol::Frame>,
    actor: &world::Actor,
    last_move: Option<Instant>,
    last_attack: Option<Instant>,
    last_cast: Option<(Instant, u32)>,
    now: Instant,
) -> bool {
    let effective = |base, pace| {
        combat.map_or(base, |c| {
            super::timed_protocol::effective_nanos(c, actor.entity_id, base, pace)
        })
    };
    attack_interval_ready(
        last_move,
        effective(actor.step_interval_nanos, super::timed_protocol::Pace::Move),
        now,
    ) && combat
        .and_then(|c| c.vital(actor.entity_id))
        .is_none_or(|v| {
            attack_interval_ready(
                last_attack,
                effective(v.attack_interval_nanos, super::timed_protocol::Pace::Attack),
                now,
            )
        })
        && last_cast.is_none_or(|(at, ms)| {
            attack_interval_ready(
                Some(at),
                effective(
                    u64::from(ms) * 1_000_000,
                    super::timed_protocol::Pace::Spell,
                ),
                now,
            )
        })
}
type HealingReceipt = (u64, u64, u64, i64, Instant, u32);
fn observe_healing_receipt(
    last: Option<HealingReceipt>,
    previous: Option<&(u64, u64, item::ItemState)>,
    current: &item::ItemState,
    match_id: u64,
    generation: u64,
    now: Instant,
) -> Option<HealingReceipt> {
    let same = previous
        .filter(|p| p.0 == match_id && p.1 == generation && p.2.entity_id == current.entity_id);
    let healed = item::HEALING.iter().any(|k| {
        current.stock(k.item_id).is_some_and(|s| {
            s.remaining
                < same
                    .and_then(|p| p.2.stock(k.item_id))
                    .map_or(s.initial, |p| p.remaining)
        })
    });
    let last = last.filter(|p| (p.0, p.1, p.2) == (match_id, generation, current.entity_id));
    if healed {
        Some((
            match_id,
            generation,
            current.entity_id,
            current.use_revision,
            now,
            if item::healing(current.item_id).is_some() {
                current.delay_millis as u32
            } else {
                1000
            },
        ))
    } else {
        last
    }
}
fn cast_has_later_action(
    cast: &super::spell_protocol::CastState,
    actor: &world::Actor,
    vital: Option<&super::combat_protocol::Vital>,
) -> bool {
    actor.position != cast.to
        || vital.is_some_and(|v| {
            v.attack_revision > cast.cast_revision || v.death_revision > cast.cast_revision
        })
}
fn spell_input_ready(state: &WindowState, snapshot: &LoginSnapshot, now: Instant) -> bool {
    skill_input_ready(state, snapshot, super::spell_protocol::TELEPORT, now)
}
fn skill_input_ready(
    state: &WindowState,
    snapshot: &LoginSnapshot,
    skill: u32,
    now: Instant,
) -> bool {
    if state.world_scope.failure.get().is_some()
        || state.world_scope.leave_queued.get()
        || state.world_scope.leave_inflight.get()
        || !can_cast_skill(snapshot, skill)
        || skill == super::spell_protocol::FIRE_STORM
            && state.world_last_fire.get().is_some_and(|t| {
                now.saturating_duration_since(t) < std::time::Duration::from_millis(2000)
            })
        || skill == super::spell_protocol::DISINTEGRATE
            && state.world_last_dis.get().is_some_and(|t| {
                now.saturating_duration_since(t) < std::time::Duration::from_millis(5000)
            })
    {
        return false;
    }
    let Some(combat) = &snapshot.combat else {
        return false;
    };
    let Some(nav) = &snapshot.navigation else {
        return false;
    };
    let Some(actor) = combat
        .world
        .actors
        .iter()
        .find(|a| a.matches_character(nav.context.key))
    else {
        return false;
    };
    confirmed_cadence_ready(
        Some(combat),
        actor,
        state.world_last_step.get(),
        state.world_last_attack.get(),
        state.world_last_cast.get(),
        now,
    )
}

/// Display only the authority verdict for this exact source/object identity.
/// SAVED is a receipt for this result, not a claim about SQL lifetime totals.
fn completion_notice(
    frame: &world::Frame,
    key: super::application_protocol::CharacterKey,
) -> Option<String> {
    if frame.race.is_some() {
        return race_notice(frame, key);
    }
    if frame.time_attack.is_some() {
        return super::time_attack_presentation::result_notice(frame, key);
    }
    if frame.siege.is_some() {
        return super::siege_presentation::result_notice(frame, key);
    }
    let completion = frame.completion.as_ref()?;
    let participant = completion
        .event
        .participants
        .iter()
        .find(|p| p.matches_character(key))?;
    let verdict = match participant.verdict {
        world::Verdict::Win => "勝利",
        world::Verdict::Loss => "敗北",
    };
    let state = match completion.state {
        world::SettlementState::Pending => "戰績保存尚未確認，請等待伺服器回覆。",
        world::SettlementState::Saved => "本場結果已保存；此畫面不代表資料庫累計場數。",
        world::SettlementState::Failed => {
            "保存未確認／回覆失敗，可能已入帳；不可重複計入或假定未保存。"
        }
    };
    Some(format!(
        "本場{verdict}（伺服器判定，包含同隊成員）。{state} 可返回原房間。"
    ))
}

fn quest_notice(frame: &world::Frame) -> Option<String> {
    use super::quest_protocol::{Flow, Phase};
    let q = frame.quest.as_ref()?;
    let lane = q.lanes.first()?;
    let objective = match q.flow {
        Flow::FinalGuardian => "最終守護者",
        Flow::RoomSymbols => "關卡符號",
    };
    let task = match lane.phase {
        Phase::SpawnPending => "等待伺服器開放下一波".into(),
        Phase::Fighting => format!("尚有 {} 個怪物", lane.living),
        Phase::Objective => q
            .actors
            .first()
            .and_then(|a| q.target(a.slot).map(|t| (a, t)))
            .map_or_else(
                || format!("等待{objective}出生"),
                |(a, t)| format!("{objective}：{}/{}", a.hp, t.max_hp),
            ),
    };
    Some(format!("己方第 {}/4 關｜{task}。", lane.stage))
}

fn race_notice(
    frame: &world::Frame,
    key: super::application_protocol::CharacterKey,
) -> Option<String> {
    use super::race_protocol::{DnfReason, RaceState, Settlement};
    let race = frame.race.as_ref()?;
    let actor = frame.actors.iter().find(|a| a.matches_character(key))?;
    let runner = race.runner(actor.entity_id)?;
    let progress = if runner.placement > 0 {
        format!(
            "第 {} 名，完賽 {:.3} 秒",
            runner.placement,
            runner.finish_elapsed_nanos as f64 / 1_000_000_000.0
        )
    } else if runner.dnf != DnfReason::None {
        let reason = match runner.dnf {
            DnfReason::Left => "中途離場",
            DnfReason::Disconnected => "連線中斷",
            DnfReason::WindowClosed => "收尾時間結束",
            DnfReason::None => unreachable!(),
        };
        format!("未完賽（{reason}），已完成 {} 圈", runner.completed_laps)
    } else {
        format!(
            "已完成 {}/4 圈，下一檢查點 {}/{}",
            runner.completed_laps,
            runner.next_checkpoint + 1,
            race.gates.len()
        )
    };
    let phase = match race.state {
        RaceState::Waiting => "等待全員載入及伺服器倒數。",
        RaceState::Running => "競賽中。",
        RaceState::FinishWindow => "首位已完成，伺服器收尾中（首位後固定 10 秒）。",
        RaceState::Aborted => "本場已中止，沒有競賽結算。",
        RaceState::Complete => match race.settlement {
            Settlement::Saved => "本場競賽結果已獨立保存，不計入兩隊戰鬥勝敗。",
            Settlement::Failed => "保存尚未確認，可能已入帳；請勿重複計入。",
            _ => "等待伺服器確認競賽結果保存。",
        },
    };
    Some(format!("寵物競賽：{progress}。{phase}"))
}

fn dispatch_canonical_action(
    state: &WindowState,
    snapshot: &LoginSnapshot,
    intent: super::battle_window::Intent,
    can_move: bool,
    can_attack: bool,
    ready: bool,
) {
    use super::battle_window::Kind;
    if intent.kind == Kind::Cancel {
        state.world_navigation.borrow_mut().cancel();
        return;
    }
    let result = (|| -> Result<()> {
        super::battle_player::validate_intent(intent, snapshot)?;
        if let Kind::Navigate(_) = intent.kind {
            ensure!(can_move, "movement is not ready");
            super::battle_player::navigate(
                &state.battle_runtime.borrow(),
                snapshot,
                intent,
                &mut state.world_navigation.borrow_mut(),
            )?;
            return Ok(());
        }
        state.world_navigation.borrow_mut().cancel();
        let available = match intent.kind {
            Kind::Attack(_) => can_attack,
            Kind::Portal => can_move,
            Kind::LightArrow(_) => {
                ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::LIGHT_ARROW,
                        Instant::now(),
                    )
            }
            Kind::Heal(_) => {
                ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::BASIC_HEAL,
                        Instant::now(),
                    )
            }
            Kind::Disintegrate(_) => {
                ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::DISINTEGRATE,
                        Instant::now(),
                    )
            }
            _ => false,
        };
        ensure!(available, "command is not ready");
        // Recheck/rebuild after the controller's final poll too; an entity ID
        // must not silently acquire a successor life between these two calls.
        super::battle_player::command(intent, snapshot)?;
        state
            .ui
            .try_borrow_mut()
            .map_err(|_| anyhow::anyhow!("controller busy"))?
            .controller
            .command(LoginCommand::PaintedTarget(intent))
            .map_err(anyhow::Error::msg)?;
        if matches!(intent.kind, Kind::Attack(_)) {
            state.world_last_attack.set(Some(Instant::now()));
        }
        Ok(())
    })();
    if let Ok(mut ui) = state.ui.try_borrow_mut() {
        ui.local_error = result
            .err()
            .map(|_| "點選場景／目標已更新，或前一操作／動作間隔未完成；未送出重試指令。".into());
    }
}

/// Coordinates immutable snapshots. All asset reads and painting run on owned workers.
unsafe fn refresh_world(hwnd: HWND, state: &WindowState, snapshot: &LoginSnapshot, closing: bool) {
    if snapshot.canonical.supported() {
        canonical_world::refresh(hwnd, state, snapshot, closing);
        return;
    }
    state.canonical_route.borrow_mut().cancel();
    state.canonical_route.borrow_mut().poll();
    let foreground = GetAncestor(GetForegroundWindow(), GA_ROOTOWNER) == hwnd;
    state
        .world_audio
        .borrow_mut()
        .configure(super::preferences::shared().get().audible_gain(foreground));
    state.world_audio.borrow_mut().poll();
    let frame = snapshot.world.frame();
    let scope_changed = state.world_scope.replace(if closing {
        None
    } else {
        world_ui_scope(snapshot)
    });
    if closing || frame.is_none() || scope_changed {
        // Discard every old action (including unscoped keyboard/Leave intents)
        // and painted hit before any successor frame may consume input. Drop
        // happens outside the RefCell borrow and never acknowledges a command.
        let retired = state.world_window.borrow_mut().take();
        drop(retired);
        state.world_details_cache.borrow_mut().take();
        state.world_audio.borrow_mut().stop();
        state.battle_audio.borrow_mut().stop();
        if closing {
            state.battle_runtime.borrow_mut().shutdown();
        } else {
            state.battle_runtime.borrow_mut().clear();
        }
        let _ = state.battle_runtime.borrow_mut().poll();
        state.world_navigation.borrow_mut().cancel();
        state.world_navigation.borrow_mut().poll();
        if closing {
            state.world_runtime.borrow_mut().shutdown();
        } else {
            state.world_runtime.borrow_mut().clear();
        }
        let _ = state.world_runtime.borrow_mut().poll();
        state.world_load_ack.set(None);
        state.world_last_step.set(None);
        state.world_last_move_ack.set(None);
        state.world_last_attack.set(None);
        state.world_last_attack_ack.set(None);
        state.world_last_cast.set(None);
        state.world_last_item.set(None);
        *state.world_item_observation.borrow_mut() = None;
        state.world_combat_seen.set(None);
        state.world_casts.borrow_mut().clear();
        state.world_damage_casts.borrow_mut().clear();
        state.world_npc_casts.borrow_mut().clear();
        *state.world_honor.borrow_mut() = None;
        state.world_fire_casts.borrow_mut().clear();
        state.world_last_fire.set(None);
        state.world_dis_casts.borrow_mut().clear();
        state.world_last_dis.set(None);
        state.world_scope.min_hit_revision.set(0);
        state.world_scope.failure.set(None);
        state.world_scope.leave_queued.set(false);
        state.world_scope.leave_inflight.set(false);
        if closing || frame.is_none() {
            return;
        }
    }
    let frame = frame.unwrap();
    let canonical_status =
        super::battle_player::update(&mut state.battle_runtime.borrow_mut(), snapshot, closing);
    let mut canonical_surface = match canonical_status {
        Ok(super::battle_runtime::Status::Ready(surface))
            if super::battle_player::admitted(&surface, snapshot) =>
        {
            Some(surface)
        }
        Ok(super::battle_runtime::Status::Failed(_)) | Err(_) => {
            state
                .world_scope
                .failure
                .set(Some("新戰鬥場景驗證／繪製失敗，正在返回房間。"));
            state.world_scope.leave_queued.set(true);
            None
        }
        _ => None,
    };
    if state.world_scope.leave_queued.get() || state.world_scope.leave_inflight.get() {
        state.battle_runtime.borrow_mut().clear();
        canonical_surface = None;
    }
    let canonical_audio = state.battle_audio.borrow_mut().tick(
        &state.battle_runtime.borrow(),
        foreground,
        Instant::now(),
    );
    if snapshot.battle_supported {
        state.world_audio.borrow_mut().stop();
    }
    let teleported = state
        .world_casts
        .borrow()
        .iter()
        .zip(&frame.casts)
        .any(|(old, new)| old.entity_id == new.entity_id && new.cast_revision > old.cast_revision);
    if teleported {
        let selected = snapshot.navigation.as_ref().map(|nav| nav.context.key);
        let selected_teleported =
            state
                .world_casts
                .borrow()
                .iter()
                .zip(&frame.casts)
                .any(|(old, new)| {
                    new.cast_revision > old.cast_revision
                        && frame.actors.iter().any(|actor| {
                            actor.entity_id == new.entity_id
                                && selected.is_some_and(|key| actor.matches_character(key))
                        })
                });
        if selected_teleported {
            state.world_navigation.borrow_mut().cancel();
            if let Some(actor) = frame
                .actors
                .iter()
                .find(|a| selected.is_some_and(|key| a.matches_character(key)))
            {
                if let Some(cast) = frame.cast(actor.entity_id) {
                    let vital = snapshot
                        .combat
                        .as_ref()
                        .filter(|combat| combat.world == *frame)
                        .and_then(|combat| combat.vital(actor.entity_id));
                    state
                        .world_last_cast
                        .set(if cast_has_later_action(cast, actor, vital) {
                            None
                        } else {
                            Some((Instant::now(), cast.cast_interval_millis))
                        });
                }
            }
        }
        state.world_scope.min_hit_revision.set(frame.revision);
        if let Some(window) = state.world_window.borrow().as_ref() {
            window.invalidate_input();
        }
    }
    *state.world_casts.borrow_mut() = frame.casts.clone();
    if let Some(combat) = snapshot
        .combat
        .as_ref()
        .filter(|combat| combat.world == *frame)
    {
        let selected = snapshot.navigation.as_ref().map(|nav| nav.context.key);
        let old = state.world_damage_casts.borrow();
        let changed = old
            .iter()
            .zip(&combat.damages)
            .any(|(a, b)| b.cast_revision > a.cast_revision);
        if changed {
            if let Some(actor) = frame
                .actors
                .iter()
                .find(|a| selected.is_some_and(|key| a.matches_character(key)))
            {
                if let (Some(before), Some(after)) = (
                    old.iter().find(|d| d.entity_id == actor.entity_id),
                    combat.damage(actor.entity_id),
                ) {
                    if after.cast_revision > before.cast_revision {
                        state.world_navigation.borrow_mut().cancel();
                        let later = actor.position != after.from
                            || combat.vital(actor.entity_id).is_some_and(|v| {
                                v.attack_revision as u64 > after.cast_revision
                                    || v.death_revision as u64 > after.cast_revision
                            })
                            || frame
                                .cast(actor.entity_id)
                                .is_some_and(|c| c.cast_revision as u64 > after.cast_revision);
                        if !later {
                            if let Some(cast) = frame.cast(actor.entity_id) {
                                state
                                    .world_last_cast
                                    .set(Some((Instant::now(), cast.cast_interval_millis)));
                            }
                        }
                    }
                }
            }
            state.world_scope.min_hit_revision.set(frame.revision);
            if let Some(window) = state.world_window.borrow().as_ref() {
                window.invalidate_input();
            }
        }
        drop(old);
        *state.world_damage_casts.borrow_mut() = combat.damages.clone();
        let old = state.world_npc_casts.borrow();
        let changed = old
            .iter()
            .zip(&combat.npc_spells)
            .any(|(a, b)| a.entity_id == b.entity_id && b.revision > a.revision);
        if changed {
            if let Some(actor) = frame
                .actors
                .iter()
                .find(|a| selected.is_some_and(|key| a.matches_character(key)))
            {
                if let Some((before, after)) = old
                    .iter()
                    .find(|s| s.entity_id == actor.entity_id)
                    .zip(combat.npc_spell(actor.entity_id))
                {
                    if after.revision > before.revision {
                        state.world_navigation.borrow_mut().cancel();
                        let later = after
                            .receipt
                            .as_ref()
                            .is_none_or(|r| r.from != actor.position)
                            || combat.vital(actor.entity_id).is_some_and(|v| {
                                v.attack_revision.max(v.death_revision) > after.revision
                            })
                            || frame
                                .cast(actor.entity_id)
                                .is_some_and(|c| c.cast_revision > after.revision)
                            || super::quest_life::respawn_revision(combat, actor.entity_id)
                                > after.revision;
                        if !later {
                            if let Some(cast) = frame.cast(actor.entity_id) {
                                state
                                    .world_last_cast
                                    .set(Some((Instant::now(), cast.cast_interval_millis)));
                            }
                        }
                    }
                }
            }
            state.world_scope.min_hit_revision.set(frame.revision);
            if let Some(window) = state.world_window.borrow().as_ref() {
                window.invalidate_input();
            }
        }
        drop(old);
        *state.world_npc_casts.borrow_mut() = combat.npc_spells.clone();
        if let (Some(old), Some(new)) = (state.world_honor.borrow().as_ref(), combat.honor.as_ref())
        {
            let changed = old.lives.iter().zip(&new.lives).any(|(a, b)| {
                a.last_death_revision != b.last_death_revision
                    || a.respawn_revision != b.respawn_revision
            }) || old
                .heals
                .iter()
                .zip(&new.heals)
                .any(|(a, b)| a.revision != b.revision);
            if changed {
                if let Some(actor) = frame
                    .actors
                    .iter()
                    .find(|a| selected.is_some_and(|key| a.matches_character(key)))
                {
                    let i = actor.entity_id as usize - 1;
                    if old.lives[i] != new.lives[i]
                        || old.heals[i].revision != new.heals[i].revision
                    {
                        state.world_navigation.borrow_mut().cancel();
                    }
                    let heal = &new.heals[i];
                    if heal.revision > old.heals[i].revision {
                        let later = actor.position != heal.from
                            || combat.vital(actor.entity_id).is_some_and(|v| {
                                v.attack_revision.max(v.death_revision) > heal.revision
                            })
                            || new.lives[i].respawn_revision > heal.revision
                            || frame
                                .cast(actor.entity_id)
                                .is_some_and(|c| c.cast_revision > heal.revision)
                            || combat
                                .damage(actor.entity_id)
                                .is_some_and(|c| c.cast_revision > heal.revision as u64);
                        if !later {
                            if let Some(cast) = frame.cast(actor.entity_id) {
                                state
                                    .world_last_cast
                                    .set(Some((Instant::now(), cast.cast_interval_millis)));
                            }
                        }
                    }
                }
                state.world_scope.min_hit_revision.set(frame.revision);
                if let Some(window) = state.world_window.borrow().as_ref() {
                    window.invalidate_input();
                }
            }
        }
        *state.world_honor.borrow_mut() = combat.honor.clone();
        let current = super::fire_storm_presentation::events(combat);
        let old = state.world_fire_casts.borrow();
        let changed = old
            .iter()
            .zip(&current)
            .any(|(a, b)| a.cast_revision != b.cast_revision);
        if changed {
            if let Some(actor) = frame
                .actors
                .iter()
                .find(|a| selected.is_some_and(|key| a.matches_character(key)))
            {
                if let Some((before, after)) = old
                    .iter()
                    .find(|s| s.entity_id == actor.entity_id)
                    .zip(current.iter().find(|s| s.entity_id == actor.entity_id))
                {
                    if after.cast_revision > before.cast_revision {
                        state.world_navigation.borrow_mut().cancel();
                        state.world_last_fire.set(Some(Instant::now()));
                        let later = actor.position != after.from
                            || combat.vital(actor.entity_id).is_some_and(|v| {
                                v.attack_revision.max(v.death_revision) > after.cast_revision
                            })
                            || frame
                                .cast(actor.entity_id)
                                .is_some_and(|c| c.cast_revision > after.cast_revision)
                            || combat
                                .damage(actor.entity_id)
                                .is_some_and(|c| c.cast_revision > after.cast_revision as u64)
                            || super::honor_protocol::heal_revision(combat, actor.entity_id)
                                > after.cast_revision
                            || super::honor_protocol::respawn_revision(combat, actor.entity_id)
                                > after.cast_revision;
                        let later = later
                            || super::disintegrate_presentation::revision(combat, actor.entity_id)
                                > after.cast_revision
                            || super::quest_life::respawn_revision(combat, actor.entity_id)
                                > after.cast_revision
                            || combat
                                .npc_spell(actor.entity_id)
                                .is_some_and(|s| s.revision > after.cast_revision);
                        if !later {
                            if let Some(cast) = frame.cast(actor.entity_id) {
                                state
                                    .world_last_cast
                                    .set(Some((Instant::now(), cast.cast_interval_millis)));
                            }
                        }
                    }
                }
            }
            state.world_scope.min_hit_revision.set(frame.revision);
            if let Some(window) = state.world_window.borrow().as_ref() {
                window.invalidate_input();
            }
        }
        drop(old);
        *state.world_fire_casts.borrow_mut() = current;
        let current = super::disintegrate_presentation::receipts(combat).collect::<Vec<_>>();
        let old = state.world_dis_casts.borrow();
        let changed = old
            .iter()
            .zip(&current)
            .any(|(a, b)| a.revision != b.revision);
        if changed {
            if let Some(actor) = frame
                .actors
                .iter()
                .find(|a| selected.is_some_and(|key| a.matches_character(key)))
            {
                if old
                    .iter()
                    .find(|s| s.entity_id == actor.entity_id)
                    .zip(current.iter().find(|s| s.entity_id == actor.entity_id))
                    .is_some_and(|(a, b)| b.revision > a.revision)
                {
                    let d = current
                        .iter()
                        .find(|s| s.entity_id == actor.entity_id)
                        .unwrap();
                    state.world_navigation.borrow_mut().cancel();
                    state.world_last_dis.set(Some(Instant::now()));
                    let later = d.hit.as_ref().is_none_or(|h| actor.position != h.from)
                        || combat
                            .vital(actor.entity_id)
                            .is_some_and(|v| v.attack_revision.max(v.death_revision) > d.revision)
                        || frame
                            .cast(actor.entity_id)
                            .is_some_and(|c| c.cast_revision > d.revision)
                        || combat
                            .damage(actor.entity_id)
                            .is_some_and(|c| c.cast_revision > d.revision as u64)
                        || super::fire_storm_presentation::revision(combat, actor.entity_id)
                            > d.revision
                        || super::honor_protocol::heal_revision(combat, actor.entity_id)
                            > d.revision
                        || super::honor_protocol::respawn_revision(combat, actor.entity_id)
                            > d.revision
                        || super::quest_life::respawn_revision(combat, actor.entity_id)
                            > d.revision
                        || combat
                            .npc_spell(actor.entity_id)
                            .is_some_and(|c| c.revision > d.revision);
                    if !later {
                        if let Some(c) = frame.cast(actor.entity_id) {
                            state
                                .world_last_cast
                                .set(Some((Instant::now(), c.cast_interval_millis)));
                        }
                    }
                }
            }
            state.world_scope.min_hit_revision.set(frame.revision);
            if let Some(window) = state.world_window.borrow().as_ref() {
                window.invalidate_input();
            }
        }
        drop(old);
        *state.world_dis_casts.borrow_mut() = current;
    }
    let action = scoped_world_action(
        scope_changed,
        state
            .world_window
            .borrow()
            .as_ref()
            .and_then(WorldWindow::take_action),
    );
    let local_intent = match action {
        Some(WorldAction::Canonical(intent))
            if matches!(intent.kind, super::battle_window::Kind::Local(_)) =>
        {
            Some(intent)
        }
        _ => None,
    };
    let action = action.and_then(|action| {
        match super::battle_player::admit_local_action(action, snapshot) {
            Ok(action) => Some(action),
            Err(_) => {
                state.world_navigation.borrow_mut().cancel();
                if let Ok(mut ui) = state.ui.try_borrow_mut() {
                    ui.local_error =
                        Some("操作的角色、生命或場景已更新；未送出，請重新操作。".into());
                }
                None
            }
        }
    });
    let settings_requested = state
        .world_window
        .borrow()
        .as_ref()
        .is_some_and(WorldWindow::take_settings_request);
    if settings_requested && !closing {
        if state.settings.borrow_mut().show(hwnd).is_err() {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("無法開啟遊戲設定；未送出遊戲操作。".into());
            }
        }
    }
    let chat_requested = state
        .world_window
        .borrow()
        .as_ref()
        .is_some_and(WorldWindow::take_chat_request);
    if chat_requested && !closing {
        open_chat(hwnd, state);
    }
    if action == Some(WorldAction::Leave) {
        state.world_navigation.borrow_mut().cancel();
        state.world_scope.leave_queued.set(true);
        // A new explicit click can retry a confirmed refusal; pending commands still block submission.
        if !snapshot.world_pending {
            state.world_scope.leave_inflight.set(false);
        }
    }
    if state.world_window.borrow().is_none() {
        let preserve_chat_focus = state
            .chat_window
            .borrow()
            .as_ref()
            .is_some_and(super::chat_window::ChatWindow::has_keyboard_focus)
            || state
                .direct_chat_window
                .borrow()
                .as_ref()
                .is_some_and(super::direct_chat_window::DirectChatWindow::has_keyboard_focus);
        match WorldWindow::open_with_activation(hwnd, !preserve_chat_focus) {
            Ok(window) => {
                *state.world_window.borrow_mut() = Some(window);
            }
            Err(_) => {
                state
                    .world_scope
                    .failure
                    .set(Some("無法建立世界視窗，正在返回房間。"));
                state.world_scope.leave_queued.set(true);
            }
        }
    }
    if let Some(window) = state.world_window.borrow().as_ref() {
        window.set_chat_available(snapshot.chat.supported() && !closing);
        if let Some(page) = window.details_page() {
            let descriptions = state.descriptions.borrow();
            let observation = window.painted_observation().filter(|o| {
                !closing
                    && !state.world_scope.leave_queued.get()
                    && !state.world_scope.leave_inflight.get()
                    && super::battle_player::admitted_observation(o, snapshot)
            });
            let mut key = WorldDetailsCache::new(snapshot, page, descriptions.revision);
            key.canonical = observation.as_ref().map(|o| {
                (
                    super::battle_window::Stamp::of(o),
                    o.data().world.loading_elapsed_nanos,
                    o.data().world.active_elapsed_nanos,
                )
            });
            if window.details_need_text()
                || state.world_details_cache.borrow().as_ref() != Some(&key)
            {
                let text = (if snapshot.battle_supported {
                    observation.as_ref().and_then(|o| {
                        super::battle_details::text(o, page, |token| {
                            descriptions.state.item_label(token)
                        })
                    })
                } else {
                    world_details_text(snapshot, page, &descriptions.state)
                })
                .unwrap_or_else(|| {
                    "目前場次／角色資料尚未一致，已清除舊檢視；等待伺服器確認。".into()
                });
                if window.update_details(&text).is_err() {
                    state.text_fault.set(true);
                }
                *state.world_details_cache.borrow_mut() = Some(key);
            }
        } else {
            state.world_details_cache.borrow_mut().take();
        }
    }
    let context = snapshot.navigation.as_ref().unwrap().context; // validate_snapshot ran before this function.
    let Some(actor) = frame
        .actors
        .iter()
        .find(|actor| actor.matches_character(context.key))
    else {
        state.text_fault.set(true);
        return;
    };
    let combat = snapshot
        .combat
        .as_ref()
        .filter(|combat| combat.world == *frame);
    if let Some(item) = combat.and_then(|c| c.item(actor.entity_id)) {
        state.world_last_item.set(observe_healing_receipt(
            state.world_last_item.get(),
            state.world_item_observation.borrow().as_ref(),
            item,
            frame.match_id,
            frame.generation,
            Instant::now(),
        ));
        *state.world_item_observation.borrow_mut() =
            Some((frame.match_id, frame.generation, item.clone()));
    }
    if combat.is_some() {
        state
            .world_combat_seen
            .set(Some((frame.match_id, frame.generation)));
    }
    let npc = snapshot.npc.as_ref().filter(|r| {
        r.frame.as_ref().is_some_and(|n| {
            n.combat.world == *frame
                && combat.is_some_and(|c| **c == n.combat)
                && n.resources.is_some()
        })
    });
    let match_combat_enabled = snapshot.world_combat_enabled();
    let waiting_combat = (match_combat_enabled
        || state.world_combat_seen.get() == Some((frame.match_id, frame.generation)))
        && (combat.is_none() || snapshot.world_npc_enabled() && npc.is_none());
    let vital = combat.and_then(|combat| {
        combat
            .vitals
            .iter()
            .find(|v| v.entity_id == actor.entity_id)
    });
    if let Some(vital) = vital {
        let ack = (
            frame.match_id,
            frame.generation,
            actor.entity_id,
            vital.attack_sequence,
        );
        if let Some(old) = state.world_last_attack_ack.replace(Some(ack)) {
            if (old.0, old.1, old.2) != (ack.0, ack.1, ack.2) {
                state.world_last_attack.set(None);
            } else if old.3 != ack.3 {
                state.world_last_attack.set(Some(Instant::now()));
            }
        }
    }
    let alive = combat_alive(match_combat_enabled, vital) && !waiting_combat;
    if combat_dead(vital) || frame.phase == world::Phase::Finished {
        state.world_navigation.borrow_mut().cancel();
    }
    let ack = (
        frame.match_id,
        frame.generation,
        actor.entity_id,
        actor.sequence,
    );
    if let Some(previous) = state.world_last_move_ack.replace(Some(ack)) {
        if (previous.0, previous.1, previous.2) != (ack.0, ack.1, ack.2) {
            state.world_last_step.set(None);
        } else if previous.3 != ack.3 {
            // Applies across mouse retargets, cancellation and manual steps.
            // A changed authoritative sequence includes confirmed denials;
            // never reset its interval by submitting a fresh route intent.
            state.world_last_step.set(Some(Instant::now()));
        }
    }
    if state.world_scope.leave_queued.get()
        || state.world_scope.leave_inflight.get()
        || matches!(frame.phase, world::Phase::Preparing | world::Phase::Aborted)
    {
        state.world_navigation.borrow_mut().cancel();
        state.world_runtime.borrow_mut().clear();
        state.world_audio.borrow_mut().stop();
    } else if snapshot.battle_supported {
        // The canonical worker now owns complete resource qualification as
        // well as pixels. Keep old workers only for non-canonical sessions.
        state.world_runtime.borrow_mut().clear();
    } else if state.world_scope.failure.get().is_none() && !waiting_combat {
        if update_world_runtime(
            &mut state.world_runtime.borrow_mut(),
            snapshot,
            actor.entity_id,
        )
        .is_err()
        {
            state
                .world_scope
                .failure
                .set(Some("世界資源狀態不一致，正在返回房間。"));
            state.world_scope.leave_queued.set(true);
        }
    }
    let runtime = state.world_runtime.borrow_mut().poll();
    let mut image = None;
    let mut hit = None;
    let mut navigation_map = None;
    let mut ready = false;
    let mut combat_surface = None;
    let mut player_surface = None;
    let mut presentation_notice = None;
    if let RuntimeStatus::Ready(rendered) = &runtime {
        if rendered.match_id == frame.match_id
            && rendered.generation == frame.generation
            && rendered.selected_entity_id == actor.entity_id
            && rendered.manifest_digest == frame.manifest.digest
            && rendered.revision <= frame.revision
            && rendered.width == 800
            && rendered.height == 600
        {
            image = Some(rendered.bgra.clone());
            presentation_notice = if snapshot.battle_supported {
                canonical_surface
                    .as_ref()
                    .and_then(|s| super::battle_window::presentation_notice(s))
            } else {
                rendered.presentation_notice
            };
            if !snapshot.battle_supported
                && matches!(frame.phase, world::Phase::Active | world::Phase::Finished)
                && !state.world_scope.leave_queued.get()
                && !state.world_scope.leave_inflight.get()
                && state.world_scope.failure.get().is_none()
            {
                if let Some(audio) = &rendered.audio {
                    state.world_audio.borrow_mut().update_at_gain(
                        (rendered.match_id, rendered.generation),
                        audio,
                        Instant::now(),
                        rendered.audio_gain,
                    );
                }
            }
            ready = rendered.ready_to_ack;
            navigation_map = rendered.navigation_map();
            if navigation_map.is_some()
                && rendered.revision >= state.world_scope.min_hit_revision.get()
            {
                hit = Some(HitSurface {
                    match_id: rendered.match_id,
                    generation: rendered.generation,
                    revision: rendered.revision,
                    selected_entity_id: rendered.selected_entity_id,
                    projection: rendered.projection,
                });
                combat_surface = rendered.combat.as_ref().map(|combat| CombatSurface {
                    frame: combat.clone(),
                    actors: rendered.actors.clone(),
                    objectives: rendered.objectives.clone(),
                    quests: rendered.quests.clone(),
                    time_attack_tower: rendered.time_attack_tower.clone(),
                    siege_towers: rendered.siege_towers.clone(),
                    npc: rendered
                        .npc
                        .as_ref()
                        .map(|reply| super::world_window::NpcSurface {
                            reply: reply.clone(),
                            placements: rendered.npcs.clone(),
                        }),
                });
                player_surface =
                    rendered
                        .combat
                        .is_none()
                        .then(|| super::world_window::PlayerSurface {
                            frame: rendered.world.clone(),
                            actors: rendered.actors.clone(),
                        });
            }
        }
    } else if matches!(runtime, RuntimeStatus::Failed(_)) {
        state.world_audio.borrow_mut().stop();
        state
            .world_scope
            .failure
            .set(Some("共用地圖或角色資源驗證／繪製失敗，正在返回房間。"));
        state.world_scope.leave_queued.set(true);
    }
    // Never mix a retired legacy worker with canonical readiness or controls.
    let input_picture_ready = if snapshot.battle_supported {
        ready = canonical_surface
            .as_ref()
            .is_some_and(|surface| state.battle_runtime.borrow().prepared(surface));
        image = None;
        hit = None;
        combat_surface = None;
        player_surface = None;
        presentation_notice = canonical_surface
            .as_ref()
            .and_then(|s| super::battle_window::presentation_notice(s));
        canonical_surface
            .as_ref()
            .is_some_and(|s| super::battle_player::controls_ready(s, snapshot))
    } else {
        hit.is_some()
    };
    let cadence_ready = confirmed_cadence_ready(
        combat.map(AsRef::as_ref),
        actor,
        state.world_last_step.get(),
        state.world_last_attack.get(),
        state.world_last_cast.get(),
        Instant::now(),
    );
    let can_move = ready
        && frame.actor_occupies(actor.entity_id)
        && input_picture_ready
        && alive
        && frame.phase == world::Phase::Active
        && !snapshot.world_pending
        && !snapshot.combat_pending
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.record_pending
        && !snapshot.shop_pending
        && !snapshot.item_pending
        && !snapshot.ammo_pending
        && !state.world_scope.leave_queued.get()
        && !state.world_scope.leave_inflight.get()
        && state.world_scope.failure.get().is_none()
        && cadence_ready;
    let can_attack = ready
        && input_picture_ready
        && alive
        && frame.phase == world::Phase::Active
        && !snapshot.world_pending
        && !snapshot.combat_pending
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.record_pending
        && !snapshot.shop_pending
        && !snapshot.item_pending
        && !snapshot.ammo_pending
        && !state.world_scope.leave_queued.get()
        && !state.world_scope.leave_inflight.get()
        && state.world_scope.failure.get().is_none()
        && vital.is_some_and(|v| {
            !super::ammo_protocol::bow(v.weapon_type)
                || combat
                    .and_then(|c| c.ammo(actor.entity_id))
                    .is_some_and(|s| s.available())
        })
        && cadence_ready
        && combat
            .and_then(|c| c.weight(actor.entity_id))
            .is_none_or(|w| w.can_attack());
    let death_confirmation =
        combat.and_then(|c| super::quest_life::confirmation(c, actor.entity_id));
    let can_confirm_death = ready
        && death_confirmation.is_some()
        && !snapshot.world_pending
        && !snapshot.combat_pending
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.item_pending
        && !snapshot.ammo_pending
        && !snapshot.shop_pending
        && !snapshot.record_pending
        && !state.world_scope.leave_queued.get()
        && !state.world_scope.leave_inflight.get()
        && state.world_scope.failure.get().is_none();
    if matches!(
        action,
        Some(
            WorldAction::Step(_)
                | WorldAction::CancelRoute
                | WorldAction::Attack { .. }
                | WorldAction::AttackNpc { .. }
                | WorldAction::AttackQuestObjective { .. }
                | WorldAction::AttackTimeAttackTower { .. }
                | WorldAction::AttackSiegeTower { .. }
                | WorldAction::UseSiegePortal { .. }
                | WorldAction::ConfirmDeath(_)
                | WorldAction::CastLightArrow { .. }
                | WorldAction::CastNpcLightArrow { .. }
                | WorldAction::CastBasicHeal { .. }
                | WorldAction::CastDisintegrate { .. }
                | WorldAction::CastNpcDisintegrate { .. }
                | WorldAction::Teleport
                | WorldAction::FireStorm
                | WorldAction::GreaterHaste
                | WorldAction::UseItem(_)
                | WorldAction::SelectAmmo(_)
        )
    ) {
        state.world_navigation.borrow_mut().cancel();
    }
    match action {
        Some(WorldAction::Canonical(intent)) => {
            dispatch_canonical_action(state, snapshot, intent, can_move, can_attack, ready);
        }
        Some(WorldAction::SelectAmmo(item_id)) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                if !ready
                    || !input_picture_ready
                    || !ammo_input_ready(snapshot, actor.entity_id, item_id)
                    || state.world_scope.leave_queued.get()
                    || state.world_scope.leave_inflight.get()
                    || state.world_scope.failure.get().is_some()
                {
                    ui.local_error =
                        Some("箭矢未確認、已用完或前一操作尚未完成；未重送選箭。".into());
                } else {
                    match ui
                        .controller
                        .command(super::battle_player::bind_local_command(
                            local_intent,
                            LoginCommand::SelectAmmo { item_id },
                        )) {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::UseItem(item_id)) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                let available = ready
                    && input_picture_ready
                    && item_input_ready(
                        snapshot,
                        actor.entity_id,
                        item_id,
                        state.world_last_item.get(),
                        Instant::now(),
                    )
                    && !state.world_scope.leave_queued.get()
                    && !state.world_scope.leave_inflight.get()
                    && state.world_scope.failure.get().is_none();
                if !available {
                    ui.local_error =
                        Some("本場補給未確認、已用完、仍在冷卻或前一操作未完成；沒有重送。".into());
                } else {
                    match ui
                        .controller
                        .command(super::battle_player::bind_local_command(
                            local_intent,
                            LoginCommand::UseItem { item_id },
                        )) {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::CastLightArrow {
            surface,
            target_entity_id,
        })
        | Some(WorldAction::CastBasicHeal {
            surface,
            target_entity_id,
        }) => {
            let heal = matches!(action, Some(WorldAction::CastBasicHeal { .. }));
            let skill = if heal {
                super::spell_protocol::BASIC_HEAL
            } else {
                super::spell_protocol::LIGHT_ARROW
            };
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                if !skill_input_ready(state, snapshot, skill, Instant::now())
                    || surface.match_id != frame.match_id
                    || surface.generation != frame.generation
                    || surface.selected_entity_id != actor.entity_id
                    || surface.revision > frame.revision
                    || target_entity_id == actor.entity_id
                    || !combat.is_some_and(|c| {
                        super::honor_protocol::target_hp(c, target_entity_id)
                            .is_some_and(|hp| hp > 0)
                            && (!heal
                                || c.honor
                                    .as_ref()
                                    .and_then(|h| h.objective(target_entity_id))
                                    .is_some()
                                    && super::honor_protocol::target_team(c, target_entity_id)
                                        == super::honor_protocol::target_team(c, actor.entity_id))
                    })
                {
                    ui.local_error =
                        Some("魔法／目標尚未確認，或前一操作／施法間隔未完成；未重送。".into());
                } else {
                    match ui.controller.command(if heal {
                        LoginCommand::CastBasicHeal { target_entity_id }
                    } else {
                        LoginCommand::CastLightArrow { target_entity_id }
                    }) {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::CastNpcDisintegrate {
            surface,
            target,
            caster_life,
            target_life,
        }) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                let request = npc_disintegrate_input_request(
                    snapshot,
                    surface,
                    target,
                    caster_life,
                    target_life,
                );
                if !ready
                    || !skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::DISINTEGRATE,
                        Instant::now(),
                    )
                    || request.is_none()
                {
                    ui.local_error =
                        Some("NPC 究極光裂術目標、生命或畫面已改變；未送出施法。".into());
                } else {
                    match ui
                        .controller
                        .command(LoginCommand::CastNpcDisintegrate(request.unwrap()))
                    {
                        Ok(()) => ui.local_error = None,
                        Err(e) => ui.local_error = Some(e),
                    }
                }
            }
        }
        Some(WorldAction::CastDisintegrate {
            surface,
            target_entity_id,
            caster_life,
            target_life,
        }) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                let request = combat.and_then(|c| {
                    disintegrate_input_request(
                        snapshot,
                        c,
                        surface,
                        target_entity_id,
                        caster_life,
                        target_life,
                    )
                });
                if !ready
                    || !skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::DISINTEGRATE,
                        Instant::now(),
                    )
                    || request.is_none()
                {
                    ui.local_error = Some(
                        "究極光裂術未就緒、間隔未到或目標生命已改變；請重新選取角色，未送出施法。"
                            .into(),
                    );
                } else {
                    match ui
                        .controller
                        .command(LoginCommand::CastDisintegrate(request.unwrap()))
                    {
                        Ok(()) => ui.local_error = None,
                        Err(e) => ui.local_error = Some(e),
                    }
                }
            }
        }
        Some(WorldAction::GreaterHaste) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error =
                    Some("強力加速術需要已確認的新版戰鬥畫面；沒有送出舊版施法。".into());
            }
        }
        Some(WorldAction::FireStorm) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                if !ready
                    || !input_picture_ready
                    || !skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::FIRE_STORM,
                        Instant::now(),
                    )
                {
                    ui.local_error =
                        Some("火風暴尚未開放、正在等待間隔或前一操作回覆；未重送。".into());
                } else {
                    let command = if snapshot.world_npc_enabled() {
                        let Some(request) = npc_fire_storm_input_request(snapshot) else {
                            ui.local_error =
                                Some("NPC 火風暴場次或角色生命已改變；未送出施法。".into());
                            return;
                        };
                        LoginCommand::CastNpcFireStorm(request)
                    } else {
                        LoginCommand::CastFireStorm
                    };
                    match ui
                        .controller
                        .command(super::battle_player::bind_local_command(
                            local_intent,
                            command,
                        )) {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::Teleport) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                if state.world_scope.leave_queued.get() || state.world_scope.leave_inflight.get() {
                    ui.local_error = Some("正在返回房間，不能施法。".into());
                } else if !ready
                    || !input_picture_ready
                    || !spell_input_ready(state, snapshot, Instant::now())
                {
                    ui.local_error =
                        Some("魔法尚未就緒、正在等待動作間隔或前一個操作回覆；未重送施法。".into());
                } else {
                    match ui
                        .controller
                        .command(super::battle_player::bind_local_command(
                            local_intent,
                            LoginCommand::CastTeleport,
                        )) {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::Leave) => state.world_scope.leave_queued.set(true),
        Some(WorldAction::ConfirmDeath(proof))
            if can_confirm_death && death_confirmation == Some(proof) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui.controller.command(LoginCommand::QuestRevive { proof }) {
                    Ok(()) => ui.local_error = None,
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::AttackQuestObjective { surface, target })
            if can_attack
                && surface.match_id == frame.match_id
                && surface.generation == frame.generation
                && surface.selected_entity_id == actor.entity_id
                && surface.revision <= frame.revision
                && super::quest_protocol::target(frame, target).is_some_and(|(_, a)| a.hp > 0) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui.controller.command(LoginCommand::QuestAttack { target }) {
                    Ok(()) => {
                        state.world_last_attack.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::AttackSiegeTower { surface, target })
            if can_attack
                && surface.match_id == frame.match_id
                && surface.generation == frame.generation
                && surface.revision <= frame.revision
                && surface.selected_entity_id == actor.entity_id
                && target.scope == super::siege_protocol::Scope::of(frame)
                && frame.siege.as_ref().is_some_and(|d| {
                    frame
                        .member(actor.entity_id)
                        .is_some_and(|m| d.attackable(m.team, target.slot))
                }) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui.controller.command(LoginCommand::SiegeAttack { target }) {
                    Ok(()) => {
                        state.world_last_attack.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::UseSiegePortal { surface, scope })
            if can_move
                && surface.match_id == frame.match_id
                && surface.generation == frame.generation
                && surface.revision == frame.revision
                && surface.selected_entity_id == actor.entity_id
                && scope == super::siege_protocol::Scope::of(frame)
                && combat.is_some_and(|f| {
                    super::siege_presentation::portal_ready(f, actor.entity_id)
                }) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui.controller.command(LoginCommand::SiegePortal { scope }) {
                    Ok(()) => {
                        state.world_last_step.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::AttackTimeAttackTower { surface, target })
            if can_attack
                && surface.match_id == frame.match_id
                && surface.generation == frame.generation
                && surface.selected_entity_id == actor.entity_id
                && surface.revision <= frame.revision
                && super::time_attack_presentation::target(frame, target).is_some_and(|d| {
                    d.phase == super::time_attack_protocol::Phase::WaitingForTower && d.tower_hp > 0
                }) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui
                    .controller
                    .command(LoginCommand::TimeAttackTowerAttack { target })
                {
                    Ok(()) => {
                        state.world_last_attack.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::CastNpcLightArrow {
            surface,
            target,
            life,
        }) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                if !ready
                    || !input_picture_ready
                    || !skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::LIGHT_ARROW,
                        Instant::now(),
                    )
                    || surface.match_id != frame.match_id
                    || surface.generation != frame.generation
                    || surface.selected_entity_id != actor.entity_id
                    || surface.revision > frame.revision
                    || !npc.is_some_and(|r| {
                        r.frame.as_ref().is_some_and(|f| {
                            f.actor(target).is_some_and(|n| n.alive() && n.life == life)
                        })
                    })
                {
                    ui.local_error = Some("NPC 光箭目標或畫面已改變，未送出施法。".into());
                } else {
                    match ui
                        .controller
                        .command(LoginCommand::CastNpcLightArrow { target, life })
                    {
                        Ok(()) => ui.local_error = None,
                        Err(error) => ui.local_error = Some(error),
                    }
                }
            }
        }
        Some(WorldAction::AttackNpc {
            surface,
            target,
            life,
        }) if can_attack
            && surface.match_id == frame.match_id
            && surface.generation == frame.generation
            && surface.selected_entity_id == actor.entity_id
            && surface.revision <= frame.revision
            && npc.is_some_and(|r| {
                r.frame
                    .as_ref()
                    .is_some_and(|f| f.actor(target).is_some_and(|n| n.alive() && n.life == life))
            }) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui
                    .controller
                    .command(LoginCommand::NpcAttack { target, life })
                {
                    Ok(()) => {
                        state.world_last_attack.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::Attack {
            surface,
            target_entity_id,
        }) if can_attack
            && surface.match_id == frame.match_id
            && surface.generation == frame.generation
            && surface.selected_entity_id == actor.entity_id
            && surface.revision <= frame.revision
            && target_entity_id != actor.entity_id
            && combat.is_some_and(|c| {
                super::honor_protocol::target_hp(c, target_entity_id).is_some_and(|hp| hp > 0)
            }) =>
        {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                match ui
                    .controller
                    .command(LoginCommand::CombatAttack { target_entity_id })
                {
                    Ok(()) => {
                        state.world_last_attack.set(Some(Instant::now()));
                        ui.local_error = None;
                    }
                    Err(error) => ui.local_error = Some(error),
                }
            }
        }
        Some(WorldAction::Step(heading)) if can_move => {
            if let Some(mut request) = world_request(snapshot, world::Action::Move) {
                if let Some(sequence) = actor
                    .sequence
                    .checked_add(1)
                    .filter(|n| *n <= i64::MAX as u64)
                {
                    request.heading = heading;
                    request.sequence = sequence;
                    request.from_x = actor.position.x;
                    request.from_y = actor.position.y;
                    if submit_world_bound(state, request, local_intent) {
                        state.world_last_step.set(Some(Instant::now()));
                    }
                } else {
                    state.world_scope.leave_queued.set(true);
                }
            }
        }
        Some(WorldAction::Navigate { surface, goal })
            if can_move
                && surface.match_id == frame.match_id
                && surface.generation == frame.generation
                && surface.selected_entity_id == actor.entity_id
                && surface.revision <= frame.revision =>
        {
            if let Some(map) = navigation_map {
                let result = if let Some(reply) = npc {
                    state.world_navigation.borrow_mut().request_npc(
                        map,
                        reply.clone(),
                        actor.entity_id,
                        goal,
                    )
                } else if let Some(combat) = combat {
                    state.world_navigation.borrow_mut().request_combat(
                        map,
                        combat.clone(),
                        actor.entity_id,
                        goal,
                    )
                } else {
                    state.world_navigation.borrow_mut().request(
                        map,
                        Arc::new(frame.clone()),
                        actor.entity_id,
                        goal,
                    )
                };
                if result.is_err() {
                    state.world_navigation.borrow_mut().cancel();
                }
            }
        }
        _ => {}
    }
    if frame.phase != world::Phase::Active
        || !frame.actor_occupies(actor.entity_id)
        || combat_dead(vital)
        || state.world_scope.leave_queued.get()
        || state.world_scope.leave_inflight.get()
        || state.world_scope.failure.get().is_some()
    {
        state.world_navigation.borrow_mut().cancel();
    }
    state.world_navigation.borrow_mut().poll();
    let next = if snapshot.world_npc_enabled() {
        npc.and_then(|reply| {
            state.world_navigation.borrow_mut().next_step_npc(
                reply,
                actor.entity_id,
                Instant::now(),
            )
        })
    } else if let Some(combat) = combat {
        state.world_navigation.borrow_mut().next_step_combat(
            combat,
            actor.entity_id,
            Instant::now(),
        )
    } else {
        state
            .world_navigation
            .borrow_mut()
            .next_step(frame, actor.entity_id, Instant::now())
    };
    if let Some(intent) = next.filter(|_| can_move) {
        if let Some(mut request) = world_request(snapshot, world::Action::Move) {
            request.heading = intent.heading;
            request.sequence = intent.sequence;
            request.from_x = intent.from.x;
            request.from_y = intent.from.y;
            if submit_world(state, request) {
                state.world_last_step.set(Some(Instant::now()));
                if state
                    .world_navigation
                    .borrow_mut()
                    .submitted(intent)
                    .is_err()
                {
                    // An accepted command is never resent after a local mismatch.
                    state.world_navigation.borrow_mut().cancel();
                    state.world_scope.leave_queued.set(true);
                }
            }
        }
    }
    let route_status = state.world_navigation.borrow().status().text();
    let completion_notice = completion_notice(frame, context.key);
    let local_notice = state
        .ui
        .try_borrow()
        .ok()
        .and_then(|ui| ui.local_error.clone());
    let status = state
        .world_scope
        .failure
        .get()
        .or(completion_notice.as_deref())
        .or(local_notice.as_deref())
        .or(presentation_notice)
        .or(canonical_audio.notice)
        .or((snapshot.battle_supported && canonical_surface.is_none())
            .then_some("正在載入新戰鬥場景；已保留伺服器狀態，尚未開放操作。"))
        .or((!cadence_ready)
            .then_some("等待伺服器指定的施法動作間隔；位置與魔力已確認，不會重送。"))
        .or(route_status)
        .or(state
            .world_audio
            .borrow()
            .failed()
            .then_some("音訊裝置失敗，音效已停用；遊戲與返回仍可操作。"))
        .unwrap_or(&snapshot.status);
    let honor_status = combat.and_then(|c| c.honor.as_ref()).map(|h| {
        format!(
            "{}\r\n守護塔：隊0 {}/{}；隊1 {}/{}。{}治癒呈現：SharedClassicBasicHealV1（744）。",
            status,
            h.objectives[0].hp,
            h.objectives[0].max_hp,
            h.objectives[1].hp,
            h.objectives[1].max_hp,
            honor_respawn_notice(frame.phase, combat_dead(vital))
        )
    });
    let status = honor_status.as_deref().unwrap_or(status);
    let quest_status = quest_notice(frame).map(|notice| format!("{status}\r\n{notice}"));
    let status = quest_status.as_deref().unwrap_or(status);
    if let Some(window) = state.world_window.borrow().as_ref() {
        if window
            .update_match_descriptor(frame.descriptor.as_ref())
            .is_err()
        {
            state.text_fault.set(true);
        }
        if window
            .update_with_roster(
                image,
                hit,
                status,
                can_move,
                combat_surface,
                can_attack,
                ready
                    && input_picture_ready
                    && spell_input_ready(state, snapshot, Instant::now())
                    && !state.world_scope.leave_queued.get()
                    && !state.world_scope.leave_inflight.get(),
                match_combat_enabled
                    && snapshot.skills.light_arrow().is_some()
                    && combat.map_or(window.light_arrow_available(), |c| {
                        c.damage(actor.entity_id).is_some()
                            || c.npc_spell(actor.entity_id).is_some()
                    }),
                ready
                    && input_picture_ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::LIGHT_ARROW,
                        Instant::now(),
                    ),
                player_surface,
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
        if let Some(surface) = canonical_surface.as_ref() {
            if window.update_canonical(surface.clone()).is_err() {
                state.text_fault.set(true);
            }
        }
        if window
            .update_honor(
                snapshot.skills.basic_heal().is_some() && combat.is_some_and(|c| c.honor.is_some()),
                ready
                    && input_picture_ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::BASIC_HEAL,
                        Instant::now(),
                    ),
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
        if window
            .update_disintegrate(
                snapshot.skills.disintegrate().is_some()
                    && combat.is_some_and(|c| {
                        c.disintegrate(actor.entity_id).is_some()
                            || c.npc_disintegrate(actor.entity_id).is_some()
                    }),
                ready
                    && input_picture_ready
                    && skill_input_ready(
                        state,
                        snapshot,
                        super::spell_protocol::DISINTEGRATE,
                        Instant::now(),
                    ),
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
        window.update_fire_storm(
            fire_storm_available(snapshot, actor.entity_id),
            ready
                && input_picture_ready
                && skill_input_ready(
                    state,
                    snapshot,
                    super::spell_protocol::FIRE_STORM,
                    Instant::now(),
                ),
        );
        window.update_quest_confirmation(death_confirmation, can_confirm_death);
        let ammo_state = combat.and_then(|c| c.ammo(actor.entity_id));
        let ammo_labels = std::array::from_fn(|i| {
            let id = super::ammo_protocol::UI_ITEMS[i];
            format!(
                "{} ×{}（F{}）{}{}",
                if i == 0 { "銀箭" } else { "米索莉箭" },
                ammo_state
                    .and_then(|s| s.stock(id))
                    .map_or(0, |s| s.remaining),
                i + 8,
                if ammo_state.is_some_and(|s| s.selected_item_id == id) {
                    "／已選用"
                } else {
                    ""
                },
                if vital.is_some_and(|v| !super::ammo_protocol::bow(v.weapon_type)) {
                    "／需普通弓才可射擊"
                } else {
                    ""
                }
            )
        });
        let can_select = std::array::from_fn(|i| {
            ready
                && input_picture_ready
                && !state.world_scope.leave_queued.get()
                && !state.world_scope.leave_inflight.get()
                && state.world_scope.failure.get().is_none()
                && ammo_input_ready(snapshot, actor.entity_id, super::ammo_protocol::UI_ITEMS[i])
        });
        if window
            .update_ammo(ammo_labels, ammo_state.is_some(), can_select)
            .is_err()
        {
            state.text_fault.set(true);
        }
        let item_state = combat.and_then(|c| c.item(actor.entity_id));
        let labels = {
            let descriptions = state.descriptions.borrow();
            std::array::from_fn(|i| {
                let token = if i < 3 {
                    ["$26", "$235", "$238"][i].to_string()
                } else {
                    format!(
                        "${}",
                        super::timed_protocol::Kind::ALL[i - 3].description_id()
                    )
                };
                let restricted = i >= 3
                    && !super::timed_protocol::Kind::ALL[i - 3].eligible(
                        frame
                            .room
                            .members
                            .iter()
                            .find(|m| m.matches_character(context.key))
                            .map_or(u16::MAX, |m| m.class),
                    );
                format!(
                    "{} ×{}（F{}）{}",
                    descriptions.state.item_label(&token),
                    item_state
                        .and_then(|s| s.stock(item::UI_ITEMS[i]))
                        .map_or(0, |s| s.remaining),
                    i + 1,
                    if restricted { "／職業不符" } else { "" }
                )
            })
        };
        let can_use = std::array::from_fn(|i| {
            ready
                && input_picture_ready
                && !state.world_scope.leave_queued.get()
                && !state.world_scope.leave_inflight.get()
                && state.world_scope.failure.get().is_none()
                && item_input_ready(
                    snapshot,
                    actor.entity_id,
                    item::UI_ITEMS[i],
                    state.world_last_item.get(),
                    Instant::now(),
                )
        });
        if window
            .update_supplies(
                labels,
                item_state.is_some(),
                combat.is_some_and(|c| !c.timed.is_empty()),
                can_use,
            )
            .is_err()
        {
            state.text_fault.set(true);
        }
    }
    if state.world_scope.leave_queued.get() && !state.world_scope.leave_inflight.get() {
        if let Some(request) = world_request(snapshot, world::Action::Leave) {
            if submit_world(state, request) {
                state.world_scope.leave_inflight.set(true);
            }
        }
    } else if !state.world_scope.leave_queued.get()
        && !state.world_scope.leave_inflight.get()
        && ready
        && frame.phase == world::Phase::Loading
        && !actor.loaded
        && state.world_load_ack.get() != Some((frame.match_id, frame.generation))
    {
        let request = if snapshot.battle_supported {
            canonical_surface.as_ref().and_then(|surface| {
                super::battle_player::loaded_request(
                    &state.battle_runtime.borrow(),
                    surface,
                    snapshot,
                )
            })
        } else if let RuntimeStatus::Ready(rendered) = &runtime {
            loaded_request(snapshot, rendered)
        } else {
            None
        };
        if let Some(request) = request {
            if submit_world(state, request) {
                state
                    .world_load_ack
                    .set(Some((frame.match_id, frame.generation)));
            }
        }
    }
}

struct Rendering<'a>(&'a Cell<bool>);
impl Drop for Rendering<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

fn vip_inventory_changed(old: &LoginSnapshot, new: &LoginSnapshot) -> bool {
    match (vip_detail_observation(old), vip_detail_observation(new)) {
        (None, None) => false,
        (Some(old), Some(new)) => {
            let a = old.data();
            let b = new.data();
            let (Some(av), Some(bv)) = (a.vip.as_ref(), b.vip.as_ref()) else {
                return true;
            };
            av.owner.player != bv.owner.player
                || av.owner.entity != bv.owner.entity
                || av.owner.versions != bv.owner.versions
                || av.owner.items != bv.owner.items
                || a.personal.equipment.as_ref().map(|e| (&e.inventory, &e.equipped, &e.lifetimes))
                    != b.personal.equipment.as_ref().map(|e| (&e.inventory, &e.equipped, &e.lifetimes))
                || a.world.identity != b.world.identity
                || a.world.phase != b.world.phase
        }
        _ => true,
    }
}

fn vip_skills_changed(old: &LoginSnapshot, new: &LoginSnapshot) -> bool {
    match (vip_detail_observation(old), vip_detail_observation(new)) {
        (None, None) => false,
        (Some(a), Some(b)) => {
            let a = a.data();
            let b = b.data();
            a.personal.skills != b.personal.skills
                || a.vip.as_ref().map(|v| (&v.owner.player, &v.owner.entity,
                    &v.owner.life, &v.owner.versions))
                    != b.vip.as_ref().map(|v| (&v.owner.player, &v.owner.entity,
                        &v.owner.life, &v.owner.versions))
                || a.world.identity != b.world.identity
                || a.world.phase != b.world.phase
        }
        _ => true,
    }
}

fn detail_snapshot_changed(old: &LoginSnapshot, new: &LoginSnapshot, view: DetailView) -> bool {
    old.navigation != new.navigation
        || canonical_detail_page(old, view) != canonical_detail_page(new, view)
        // The full-VIP placeholder replaces the saved Items/Skills pages too.
        // Their stored data and navigation can remain identical across attach/exit.
        || full_vip_details_route(old) != full_vip_details_route(new)
        || old.character_data != new.character_data
        || old.equipment != new.equipment
        || old.skills != new.skills
        || old.record != new.record
        || old.record_pending != new.record_pending
        || (view == DetailView::Stats
            && vip_detail_observation(old).and_then(super::battle_details::vip_character)
                != vip_detail_observation(new).and_then(super::battle_details::vip_character))
        || (view == DetailView::Items && vip_inventory_changed(old, new))
        || (view == DetailView::Skills && vip_skills_changed(old, new))
}

fn preserve_detail_reading(
    old: Option<&LoginSnapshot>,
    new: &LoginSnapshot,
    view: DetailView,
    rendered_view: Option<DetailView>,
) -> bool {
    let reading_scope = |snapshot: &LoginSnapshot| {
        snapshot.navigation.as_ref().map(|nav| {
            (nav.context, nav.snapshot.phase, nav.snapshot.channel_id,
                vip_detail_identity(snapshot))
        })
    };
    rendered_view == Some(view)
        && reading_scope(new).is_some()
        && old.and_then(reading_scope) == reading_scope(new)
}

fn vip_status_desired(
    snapshot: &LoginSnapshot,
) -> Option<(super::vip_status_preview::Key, &super::battle_follow::Observation)> {
    let observation = vip_detail_observation(snapshot)?;
    let (context, channel) = super::battle_control::scope_for(snapshot)?;
    let key = super::vip_status_preview::Key::from_observation(
        context, channel, observation).ok()??;
    Some((key, observation))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InventoryReadingPosition {
    top: usize,
    selected: Option<usize>,
}

fn restore_inventory_reading(
    old: &super::vip_inventory_preview::Model,
    next: &super::vip_inventory_preview::Model,
    top: usize,
    selected: Option<usize>,
) -> Option<InventoryReadingPosition> {
    if !old.key.same_reading_scope(&next.key) { return None; }
    let top_object = old.rows.get(top).map(|row| row.object);
    let top = top_object.and_then(|object|
        next.rows.iter().position(|row| row.object == object))
        .unwrap_or_else(|| top.min(next.rows.len().saturating_sub(1)));
    let selected = selected.and_then(|index| old.rows.get(index))
        .and_then(|row| next.rows.iter().position(|next| next.object == row.object));
    Some(InventoryReadingPosition { top, selected })
}

unsafe fn refresh_vip_inventory_control(
    hwnd: HWND, state: &WindowState, controls: [HWND; CONTROL_COUNT],
    snapshot: &LoginSnapshot, closing: bool, revision: u64,
) {
    let show = !closing && state.detail_view.get() == DetailView::Items
        && full_vip_details_route(snapshot)
        && state.vip_inventory.borrow().model().is_some();
    let changed = {
        let inventory = state.vip_inventory.borrow();
        let rendered = state.rendered_vip_inventory.borrow();
        rendered.as_ref() != if show { inventory.model() } else { None }
    };
    if changed {
        let model = if show { state.vip_inventory.borrow().model().cloned() } else { None };
        let previous = state.rendered_vip_inventory.borrow().clone();
        let top = SendMessageW(controls[VIP_ITEMS], LB_GETTOPINDEX, None, None).0.max(0) as usize;
        let selected = SendMessageW(controls[VIP_ITEMS], LB_GETCURSEL, None, None).0;
        let position = previous.as_ref().zip(model.as_ref()).and_then(|(old, next)|
            restore_inventory_reading(old, next, top,
                (selected >= 0).then_some(selected as usize)));
        if position.is_none() { let _ = ShowWindow(controls[VIP_ITEMS], SW_HIDE); }
        else { let _ = SendMessageW(controls[VIP_ITEMS], WM_SETREDRAW, Some(WPARAM(0)), None); }
        let _ = SendMessageW(controls[VIP_ITEMS], LB_RESETCONTENT, None, None);
        *state.rendered_vip_inventory.borrow_mut() = model.clone();
        let mut complete = true;
        if let Some(model) = &model {
            for (index, row) in model.rows.iter().enumerate() {
                let equipped = match row.equipped {
                    Some(true) => "已裝備", Some(false) => "背包", None => "穿戴未確認",
                };
                let label = wide(&format!("{} ×{} · {} · 實例 {}",
                    row.label, row.quantity, equipped, row.object));
                if SendMessageW(controls[VIP_ITEMS], LB_ADDSTRING, None,
                    Some(LPARAM(label.as_ptr() as isize))).0 != index as isize
                { complete = false; break; }
            }
        }
        if !complete {
            let _ = SendMessageW(controls[VIP_ITEMS], LB_RESETCONTENT, None, None);
            *state.rendered_vip_inventory.borrow_mut() = None;
        }
        if complete {
            if let Some(position) = position {
                // LB_SETCURSEL can scroll the chosen row into view. Restore
                // the reader's top item *last*, even when selection is empty.
                let _ = SendMessageW(controls[VIP_ITEMS], LB_SETCURSEL,
                    Some(WPARAM(position.selected.unwrap_or(usize::MAX))), None);
                if model.as_ref().is_some_and(|m| !m.rows.is_empty()) {
                    let _ = SendMessageW(controls[VIP_ITEMS], LB_SETTOPINDEX,
                        Some(WPARAM(position.top)), None);
                }
            }
        }
        if position.is_some() {
            let _ = SendMessageW(controls[VIP_ITEMS], WM_SETREDRAW, Some(WPARAM(1)), None);
        }
        let visible = show && complete;
        if state.vip_inventory_visible.replace(visible) != visible { resize(hwnd, state); }
        if visible {
            if position.is_none() { let _ = ShowWindow(controls[VIP_ITEMS], SW_SHOWNA); }
            let _ = RedrawWindow(Some(controls[VIP_ITEMS]), None, None,
                REDRAW_WINDOW_FLAGS(RDW_ERASE.0 | RDW_FRAME.0
                    | RDW_INVALIDATE.0 | RDW_ALLCHILDREN.0));
        } else {
            // WM_SETREDRAW(TRUE) can restore WS_VISIBLE; final hide comes last.
            let _ = ShowWindow(controls[VIP_ITEMS], SW_HIDE);
        }
    }
    if state.vip_inventory_rendered_revision.replace(revision) != revision && show {
        let _ = InvalidateRect(Some(controls[VIP_ITEMS]), None, false);
    }
}

unsafe fn refresh(hwnd: HWND, state: &WindowState) {
    let Some(controls) = state.controls.get() else {
        return;
    };
    if state.rendering.replace(true) {
        return;
    }
    let _rendering = Rendering(&state.rendering);
    if state
        .test_deadline
        .is_some_and(|deadline| Instant::now() >= deadline)
    {
        if let Ok(mut ui) = state.ui.try_borrow_mut() {
            ui.begin_close();
        }
    }
    let settings_quiescent = {
        let mut settings = state.settings.borrow_mut();
        if state.text_fault.get() || state.ui.try_borrow().ok().is_some_and(|ui| ui.closing) {
            settings.close();
        }
        settings.poll();
        settings.is_quiescent()
    };
    let form_outcome = state
        .room_form
        .borrow_mut()
        .as_mut()
        .and_then(RoomForm::poll);
    if let Some(outcome) = form_outcome {
        if state
            .room_form
            .borrow()
            .as_ref()
            .is_some_and(RoomForm::is_quiescent)
        {
            let retired_form = state.room_form.borrow_mut().take();
            drop(retired_form); // Native Drop cannot run while the RefCell is borrowed.
            state.room_form_context.set(None);
        }
        match outcome {
            FormOutcome::Submitted(request) => send(hwnd, state, LoginCommand::Rooms(request)),
            FormOutcome::Matchmaking(request) => {
                send(hwnd, state, LoginCommand::Matchmaking(request))
            }
            FormOutcome::Cancelled => {}
            FormOutcome::Failed(reason) => local_error(hwnd, state, reason),
        }
    }
    let shop_intent = {
        let mut shop = state.shop_window.borrow_mut();
        shop.as_mut().and_then(ShopWindow::poll)
    };
    if let Some(intent) = shop_intent {
        if let Some(shop) = state.shop_window.borrow_mut().as_mut() {
            shop.dispatched();
        }
        match intent {
            ShopOutcome::Request(request) => send(hwnd, state, LoginCommand::Shop(request)),
            ShopOutcome::Failed(reason) => local_error(hwnd, state, reason),
        }
    }
    let shop_quiescent = state
        .shop_window
        .borrow()
        .as_ref()
        .is_none_or(ShopWindow::is_quiescent);
    let form_quiescent = state
        .room_form
        .borrow()
        .as_ref()
        .is_none_or(RoomForm::is_quiescent);
    let descriptions_quiescent = {
        let closing =
            state.text_fault.get() || state.ui.try_borrow().ok().is_some_and(|ui| ui.closing);
        let mut descriptions = state.descriptions.borrow_mut();
        descriptions.poll(closing);
        descriptions.is_quiescent()
    };
    let worlds_quiescent = {
        let mut runtime = state.world_runtime.borrow_mut();
        let closing =
            state.text_fault.get() || state.ui.try_borrow().ok().is_some_and(|ui| ui.closing);
        // For an open window, refresh_world must consume queued Leave before poll may spawn a pending loader.
        if closing {
            runtime.shutdown();
            let _ = runtime.poll();
            state.world_navigation.borrow_mut().cancel();
            state.world_navigation.borrow_mut().poll();
        }
        let mut canonical = state.battle_runtime.borrow_mut();
        if closing {
            canonical.shutdown();
            canonical.poll();
            state.canonical_route.borrow_mut().cancel();
            state.canonical_route.borrow_mut().poll();
            state
                .battle_audio
                .borrow_mut()
                .tick(&canonical, false, Instant::now());
        }
        runtime.is_quiescent()
            && canonical.is_quiescent()
            && state.world_navigation.borrow().is_quiescent()
            && state.canonical_route.borrow().is_quiescent()
    };
    let (old, snapshot, closing, error, clear_secrets, mut should_destroy) = {
        let Ok(mut ui) = state.ui.try_borrow_mut() else {
            return;
        };
        ui.controller.poll();
        if state.text_fault.get() {
            ui.fatal = true;
            ui.begin_close();
        }
        let mut snapshot = ui.controller.snapshot();
        // A local validation/busy message is useful while its snapshot remains
        // current, but must not hide a later auth result or connection failure.
        if snapshot_changed(ui.last.as_ref(), &snapshot) {
            ui.local_error = None;
        }
        if validate_snapshot(&snapshot).is_err() {
            ui.local_error = Some("控制器傳回不一致或超出限制的列表，已取消連線。".into());
            ui.fatal = true;
            ui.begin_close();
            // Never render the unvalidated/oversized value after rejecting it.
            snapshot = LoginSnapshot {
                phase: LoginPhase::Disconnecting,
                status: String::new(),
                sources: vec![],
                selected_source: None,
                characters: vec![],
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
                battle: crate::modern_lito::battle_session::State::default(),
                canonical: crate::modern_lito::battle_control::State::default(),
                battle_supported: false,
                action_supported: false,
                world_pending: false,
                combat: None,
                npc: None,
                combat_pending: false,
                combat_supported: false,
                skills: super::spell_protocol::SkillsState::NotRequested,
                spell_pending: false,
                record: record::State::NotRequested,
                record_pending: false,
                chat: crate::modern_lito::chat_state::State::default(),
                chat_pending: false,
                direct_chat: crate::modern_lito::direct_chat_state::State::default(),
                direct_chat_pending: false,
                shop: shop::State::NotRequested,
                shop_pending: false,
                item_pending: false,
                ammo_pending: false,
            };
        }
        let should_destroy = ui.closing
            && ui.controller.is_quiescent()
            && descriptions_quiescent
            && form_quiescent
            && shop_quiescent
            && settings_quiescent
            && worlds_quiescent;
        let clear_secrets = clear_credentials_on_transition(ui.last.as_ref(), &snapshot);
        let old = ui.last.replace(snapshot.clone());
        (
            old,
            snapshot,
            ui.closing,
            ui.local_error.clone(),
            clear_secrets,
            should_destroy,
        )
    };
    let (status_quiescent, status_revision, status_ready) = {
        let mut status = state.vip_status.borrow_mut();
        status.update(vip_status_desired(&snapshot), &state.shared_root,
            state.effect_catalog_path.as_deref(), closing);
        (status.is_quiescent(), status.revision(), status.image().is_some())
    };
    should_destroy &= status_quiescent;
    let inventory_description_revision = state.descriptions.borrow().revision;
    let want_inventory = !closing && state.detail_view.get() == DetailView::Items
        && full_vip_details_route(&snapshot);
    let inventory_changed = old.as_ref().is_none_or(|previous|
        previous.phase != snapshot.phase
            || previous.selected_source != snapshot.selected_source
            || previous.navigation != snapshot.navigation
            || vip_inventory_changed(previous, &snapshot))
        || state.vip_inventory_description_revision.get() != Some(inventory_description_revision)
        || state.vip_inventory_requested.replace(want_inventory) != want_inventory;
    let (inventory_quiescent, inventory_revision) = {
        let mut inventory = state.vip_inventory.borrow_mut();
        if inventory_changed || closing {
            let desired = if !want_inventory { None } else {
                super::battle_control::scope_for(&snapshot)
                    .zip(vip_detail_observation(&snapshot))
                    .and_then(|((context, channel), observation)| {
                        let descriptions = state.descriptions.borrow();
                        super::vip_inventory_preview::Model::from_observation(
                            context, channel, observation,
                            |token| descriptions.state.item_label(token))
                    })
            };
            inventory.reconcile(desired);
            state.vip_inventory_description_revision.set(Some(inventory_description_revision));
        }
        inventory.poll(&state.shared_root, closing);
        (inventory.is_quiescent(), inventory.revision())
    };
    should_destroy &= inventory_quiescent;
    // All borrows end before synchronous Win32 sends, which may reenter WndProc.
    if closing
        || state
            .room_form_context
            .get()
            .is_some_and(|common| !form_context_current(common, &snapshot))
    {
        cancel_room_form(state);
    }
    if clear_secrets || should_destroy {
        clear_credentials(state, true);
    }
    refresh_world(hwnd, state, &snapshot, closing);
    refresh_shop(hwnd, state, &snapshot, closing);
    refresh_chat(hwnd, state, closing);
    refresh_direct_chat(hwnd, state, closing);
    if should_destroy {
        let _ = DestroyWindow(hwnd);
        return;
    }
    let show_status = !closing && status_ready
        && matches!(state.detail_view.get(), DetailView::Stats | DetailView::Items)
        && full_vip_details_route(&snapshot);
    if state.vip_status_visible.replace(show_status) != show_status {
        if !show_status {
            let _ = ShowWindow(controls[VIP_STATUS_STRIP], SW_HIDE);
        }
        resize(hwnd, state);
        if show_status {
            let _ = ShowWindow(controls[VIP_STATUS_STRIP], SW_SHOWNA);
        }
    }
    if state.vip_status_rendered_revision.replace(status_revision) != status_revision && show_status {
        let _ = InvalidateRect(Some(controls[VIP_STATUS_STRIP]), None, false);
    }
    refresh_vip_inventory_control(hwnd, state, controls, &snapshot, closing, inventory_revision);
    let focus_before_refresh = {
        let focused = windows::Win32::UI::Input::KeyboardAndMouse::GetFocus();
        (GetForegroundWindow() == hwnd && controls.contains(&focused)).then_some(focused)
    };
    let controls_enabled = enabled(snapshot.phase, closing);
    let show_rooms = room_surface(&snapshot);
    let _ = EnableWindow(controls[SETTINGS], !closing);
    let _ = EnableWindow(
        controls[CHAT],
        !closing && snapshot.phase == LoginPhase::InLobby && snapshot.chat.supported(),
    );
    let form_open = state.room_form.borrow().is_some() || state.shop_window.borrow().is_some();
    for (index, cancel) in [(MATCH_START, false), (MATCH_CANCEL, true)] {
        let _ = ShowWindow(
            controls[index],
            if show_rooms && snapshot.matchmaking.supported() {
                SW_SHOWNA
            } else {
                SW_HIDE
            },
        );
        let _ = EnableWindow(
            controls[index],
            !closing && !form_open && matchmaking_common(&snapshot, cancel).is_some(),
        );
    }
    let show_shop = show_rooms && !matches!(snapshot.shop, shop::State::Unsupported);
    let _ = ShowWindow(
        controls[SHOP_OPEN],
        if show_shop { SW_SHOWNA } else { SW_HIDE },
    );
    let _ = EnableWindow(
        controls[SHOP_OPEN],
        !closing && !form_open && can_open_shop(&snapshot),
    );
    let show_gm = show_rooms && snapshot.gm_supported;
    for index in [GM_LABEL, GM_COUNT, GM_SEND] {
        let _ = ShowWindow(controls[index], if show_gm { SW_SHOWNA } else { SW_HIDE });
    }
    // The source selector belongs to login/character navigation.  A lobby
    // with neither GM nor shop support must not accidentally expose it over
    // the room controls (nor leave an orphan label when only shop is present).
    for index in [SOURCE_LABEL, SOURCE] {
        let _ = ShowWindow(
            controls[index],
            if source_selector_visible(&snapshot) {
                SW_SHOWNA
            } else {
                SW_HIDE
            },
        );
    }
    for index in [GM_COUNT, GM_SEND] {
        let _ = EnableWindow(
            controls[index],
            !closing && !form_open && gm_input_ready(&snapshot),
        );
    }
    for index in [
        ACCOUNT_LABEL,
        ACCOUNT,
        PASSWORD_LABEL,
        PASSWORD,
        CONNECT,
        LOGIN,
    ] {
        let _ = ShowWindow(
            controls[index],
            if show_rooms { SW_HIDE } else { SW_SHOWNA },
        );
    }
    for index in ROOM_CREATE..=ROOM_READ {
        let _ = ShowWindow(
            controls[index],
            if show_rooms { SW_SHOWNA } else { SW_HIDE },
        );
    }
    let _ = ShowWindow(
        controls[ROOM_LIST],
        if show_rooms { SW_SHOWNA } else { SW_HIDE },
    );
    let _ = EnableWindow(controls[CONNECT], controls_enabled.connect);
    for index in [ACCOUNT, PASSWORD, LOGIN] {
        let _ = EnableWindow(controls[index], controls_enabled.credentials);
    }
    let _ = EnableWindow(
        controls[SOURCE],
        controls_enabled.source && !snapshot.sources.is_empty(),
    );
    let _ = EnableWindow(controls[DISCONNECT], controls_enabled.disconnect);
    let _ = EnableWindow(
        controls[CHARACTERS],
        !closing && snapshot.phase == LoginPhase::CharactersReady,
    );
    let show_channels = snapshot.navigation.is_some();
    {
        let _ = ShowWindow(
            controls[CHARACTERS],
            if show_channels { SW_HIDE } else { SW_SHOWNA },
        );
        let _ = ShowWindow(
            controls[CHANNELS],
            if show_channels && !show_rooms {
                SW_SHOWNA
            } else {
                SW_HIDE
            },
        );
    }
    if old.as_ref().is_none_or(|old| old.rooms != snapshot.rooms) {
        let index = SendMessageW(controls[ROOM_LIST], LB_GETCURSEL, None, None).0;
        let old_directory = old.as_ref().and_then(|old| {
            if let rooms::State::Ready(reply) = &old.rooms {
                Some(reply)
            } else {
                None
            }
        });
        let retained = old_directory
            .filter(|old| old.joined.is_none())
            .and_then(|old| {
                usize::try_from(index)
                    .ok()
                    .and_then(|i| old.rooms.get(i))
                    .map(|r| r.room_id)
            });
        let _ = SendMessageW(controls[ROOM_LIST], LB_RESETCONTENT, None, None);
        if let rooms::State::Ready(reply) = &snapshot.rooms {
            let labels: Vec<String> = if let Some(room) = &reply.joined {
                room.members
                    .iter()
                    .map(|m| {
                        format!(
                            "{}({}) · {}{} / {} · Lv.{} · 隊{} · {}{}",
                            m.name,
                            m.source_name,
                            if m.kind == rooms::MemberKind::ServerBot {
                                "假人來源"
                            } else {
                                "真人來源"
                            },
                            m.key.source_id,
                            m.key.object_id,
                            m.level,
                            if m.team == 0 { "A" } else { "B" },
                            if m.ready { "已準備" } else { "未準備" },
                            if m.matches_character(room.owner) {
                                " · 房主"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect()
            } else {
                reply
                    .rooms
                    .iter()
                    .map(|room| {
                        format!(
                            "#{} {} · 模式{} / 地圖{} · {}/{} · 準備{}{}",
                            room.room_id,
                            room.settings.name,
                            room.settings.mode,
                            room.settings.map_id,
                            room.users,
                            room.settings.capacity,
                            room.ready,
                            if room.settings.attributes & 4 != 0 {
                                " · 有密碼"
                            } else {
                                ""
                            }
                        )
                    })
                    .collect()
            };
            for (i, label) in labels.iter().enumerate() {
                let label = wide(label);
                if SendMessageW(
                    controls[ROOM_LIST],
                    LB_ADDSTRING,
                    None,
                    Some(LPARAM(label.as_ptr() as isize)),
                )
                .0 != i as isize
                {
                    state.text_fault.set(true);
                    return;
                }
            }
            if reply.joined.is_none() {
                if let Some(index) =
                    retained.and_then(|id| reply.rooms.iter().position(|r| r.room_id == id))
                {
                    let _ =
                        SendMessageW(controls[ROOM_LIST], LB_SETCURSEL, Some(WPARAM(index)), None);
                }
            }
            paint_text(
                state,
                controls[CHARACTER_LABEL],
                &room_directory_label(reply),
            );
        }
    }
    let selected_room_index = SendMessageW(controls[ROOM_LIST], LB_GETCURSEL, None, None).0;
    let selected_room = if let rooms::State::Ready(reply) = &snapshot.rooms {
        usize::try_from(selected_room_index)
            .ok()
            .and_then(|i| reply.rooms.get(i))
            .map(|r| r.room_id)
    } else {
        None
    };
    for (index, action, value) in [
        (ROOM_CREATE, rooms::Action::Create, 0),
        (ROOM_JOIN, rooms::Action::Join, 0),
        (ROOM_CONFIGURE, rooms::Action::Configure, 0),
        (ROOM_LEAVE, rooms::Action::Leave, 0),
        (ROOM_TEAM0, rooms::Action::Team, 0),
        (ROOM_TEAM1, rooms::Action::Team, 1),
        (
            ROOM_READY,
            rooms::Action::Ready,
            room_ready_value(&snapshot),
        ),
        (ROOM_READ, rooms::Action::Read, 0),
    ] {
        let _ = EnableWindow(
            controls[index],
            !closing
                && !form_open
                && room_common(&snapshot, action, selected_room, value).is_some(),
        );
    }
    let _ = EnableWindow(
        controls[ROOM_LIST],
        !closing
            && !form_open
            && !snapshot.room_pending
            && !snapshot.equipment_pending
            && !snapshot.world_owns_lobby()
            && !snapshot.spell_pending
            && !snapshot.conversation_pending()
            && !snapshot.record_pending,
    );
    let _ = EnableWindow(
        controls[CHANNELS],
        !closing
            && !snapshot.equipment_pending
            && !snapshot.spell_pending
            && !snapshot.conversation_pending()
            && !snapshot.record_pending
            && snapshot.phase == LoginPhase::ChoosingChannel,
    );
    if old
        .as_ref()
        .is_none_or(|previous| previous.phase != snapshot.phase)
    {
        if !show_rooms {
            paint_text(
                state,
                controls[CHARACTER_LABEL],
                if let Some(hint) = unsupported_room_hint(&snapshot) {
                    hint
                } else if matches!(
                    snapshot.phase,
                    LoginPhase::InLobby | LoginPhase::LeavingLobby
                ) {
                    "已進入大廳（伺服器確認的頻道狀態）"
                } else if show_channels {
                    "頻道列表（同一端點的四個大廳，選取後按「進入大廳」）"
                } else {
                    "角色列表（選取後按「選擇角色」）"
                },
            );
        }
        paint_text(
            state,
            controls[RETURN_CHARACTERS],
            return_label(snapshot.phase),
        );
    }
    let status = if closing {
        "正在取消連線並等待工作執行緒結束；視窗仍可移動。".to_string()
    } else if let Some(error) = error {
        error
    } else {
        format!("{}\r\n{}", phase_label(snapshot.phase), snapshot.status)
    };
    let status_changed = {
        let mut previous = state.rendered_status.borrow_mut();
        if *previous == status {
            false
        } else {
            *previous = status.clone();
            true
        }
    };
    if status_changed {
        paint_text(state, controls[STATUS], &status);
        // A readable title permits ordinary UI observation without inspecting
        // password EDIT memory or introducing an automation-only login bypass.
        let title_status: String = status
            .chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .take(240)
            .collect();
        paint_text(state, hwnd, &format!("LiTo 現代客戶端 — {title_status}"));
    }
    let changes = list_changes(old.as_ref(), &snapshot);
    if changes.sources {
        let _ = SendMessageW(controls[SOURCE], CB_RESETCONTENT, None, None);
        for source in &snapshot.sources {
            let label = wide(&source.label);
            let _ = SendMessageW(
                controls[SOURCE],
                CB_ADDSTRING,
                None,
                Some(LPARAM(label.as_ptr() as isize)),
            );
        }
    }
    let wanted = snapshot
        .sources
        .iter()
        .position(|source| Some(source.id) == snapshot.selected_source);
    let actual = SendMessageW(controls[SOURCE], CB_GETCURSEL, None, None).0;
    if let Some(selected) = source_selection_to_apply(actual, wanted) {
        let _ = SendMessageW(controls[SOURCE], CB_SETCURSEL, Some(WPARAM(selected)), None);
    }
    if changes.characters {
        let _ = SendMessageW(controls[CHARACTERS], LB_RESETCONTENT, None, None);
        for row in &snapshot.characters {
            let label = wide(&row.label);
            let _ = SendMessageW(
                controls[CHARACTERS],
                LB_ADDSTRING,
                None,
                Some(LPARAM(label.as_ptr() as isize)),
            );
        }
        paint_details_text(
            state,
            controls[DETAILS],
            if snapshot.phase == LoginPhase::LoadingCharacters {
                "正在載入所選來源的角色，不顯示舊來源資料。"
            } else if snapshot.characters.is_empty() {
                "此階段沒有可顯示的角色。"
            } else {
                "請選取角色以檢視其來源與識別碼，再按「選擇角色」。"
            },
            false,
        );
    }
    let old_channels = old
        .as_ref()
        .and_then(|view| view.navigation.as_ref())
        .map(|reply| &reply.snapshot.channels);
    let channels = snapshot
        .navigation
        .as_ref()
        .map(|reply| &reply.snapshot.channels);
    if old_channels != channels {
        let old_index = SendMessageW(controls[CHANNELS], LB_GETCURSEL, None, None).0;
        let same_context = old
            .as_ref()
            .and_then(|view| view.navigation.as_ref())
            .map(|reply| reply.context)
            == snapshot.navigation.as_ref().map(|reply| reply.context);
        let retain_id = if same_context {
            usize::try_from(old_index)
                .ok()
                .and_then(|index| old_channels?.get(index))
                .map(|channel| channel.id)
        } else {
            None
        };
        let _ = SendMessageW(controls[CHANNELS], LB_RESETCONTENT, None, None);
        if let Some(channels) = channels {
            for channel in channels {
                let label = wide(&channel_label(channel));
                let _ = SendMessageW(
                    controls[CHANNELS],
                    LB_ADDSTRING,
                    None,
                    Some(LPARAM(label.as_ptr() as isize)),
                );
            }
            if let Some(id) = snapshot
                .navigation
                .as_ref()
                .and_then(|reply| reply.snapshot.channel_id)
                .or(retain_id)
            {
                if let Some(index) = channels.iter().position(|channel| channel.id == id) {
                    let _ =
                        SendMessageW(controls[CHANNELS], LB_SETCURSEL, Some(WPARAM(index)), None);
                }
            }
        }
    }
    if let Some(id) = snapshot
        .navigation
        .as_ref()
        .and_then(|reply| reply.snapshot.channel_id)
    {
        // The lobby highlight is server-owned, not the last locally clicked row.
        let index = snapshot.navigation.as_ref().and_then(|reply| {
            reply
                .snapshot
                .channels
                .iter()
                .position(|channel| channel.id == id)
        });
        let actual = SendMessageW(controls[CHANNELS], LB_GETCURSEL, None, None).0;
        if let Some(index) = source_selection_to_apply(actual, index) {
            let _ = SendMessageW(controls[CHANNELS], LB_SETCURSEL, Some(WPARAM(index)), None);
        }
    }
    let detail_view = state.detail_view.get();
    let description_revision = state.descriptions.borrow().revision;
    let preserve_details_reading = preserve_detail_reading(
        old.as_ref(), &snapshot, detail_view, state.rendered_detail_view.get());
    let detail_changed = old.as_ref().is_none_or(|old| {
        detail_snapshot_changed(old, &snapshot, detail_view)
    }) || state.rendered_detail_view.get() != Some(detail_view)
        || (matches!(detail_view, DetailView::Stats | DetailView::Items)
            && full_vip_details_route(&snapshot)
            && state.rendered_vip_status_text_revision.get() != Some(status_revision))
        || (matches!(detail_view, DetailView::Items | DetailView::Skills)
            && state.rendered_description_revision.get() != Some(description_revision));
    if detail_changed {
        state.rendered_detail_view.set(Some(detail_view));
        state
            .rendered_description_revision
            .set(Some(description_revision));
        state.rendered_vip_status_text_revision.set(Some(status_revision));
        let mut details = source_data_text(&snapshot, detail_view, &state.descriptions.borrow().state);
        if matches!(detail_view, DetailView::Stats | DetailView::Items)
            && full_vip_details_route(&snapshot)
            && vip_detail_observation(&snapshot).is_some_and(|o|
                !o.data().vip.as_ref().unwrap().owner.icons.0.is_empty())
        {
            if let Some(details) = details.as_mut() {
                let status = state.vip_status.borrow();
                if let Some(reason) = status.error() {
                    let _ = write!(details, "\r\n狀態圖尚未顯示：{reason}。不替代或省略圖像。\r\n");
                } else if status.image().is_none() {
                    details.push_str("\r\n狀態圖正在從已設定的共享資源載入；尚未顯示。\r\n");
                }
            }
        }
        // Release catalog borrows before any synchronous Win32 notifications.
        if let Some(details) = details {
            // The source model's 512-row/text budgets make this bound achievable
            // without truncating items. Never silently cut off the last rows.
            if details.encode_utf16().count() <= MAX_DETAILS_UTF16 {
                paint_details_text(state, controls[DETAILS], &details, preserve_details_reading);
            } else {
                paint_details_text(
                    state,
                    controls[DETAILS],
                    "來源保存資料顯示超出安全限制，未顯示截斷資料。",
                    false,
                );
            }
        } else if old.as_ref().is_some_and(|old| old.navigation.is_some()) {
            paint_details_text(
                state,
                controls[DETAILS],
                "已清除前一個角色的來源保存資料；請選擇目前來源的角色。",
                false,
            );
        }
    }
    let world_available = state
        .world_window
        .borrow()
        .as_ref()
        .and_then(WorldWindow::window_handle)
        .is_some();
    for (index, detail) in [
        (SHOW_STATS, DetailView::Stats),
        (SHOW_ITEMS, DetailView::Items),
    ] {
        let _ = EnableWindow(
            controls[index],
            !closing && can_show_player_detail(&snapshot, detail, world_available),
        );
    }
    let _ = EnableWindow(
        controls[SHOW_SKILLS],
        !closing && can_show_player_detail(&snapshot, DetailView::Skills, world_available),
    );
    let _ = EnableWindow(
        controls[SHOW_RECORD],
        !closing && snapshot.navigation.is_some(),
    );
    let _ = EnableWindow(
        controls[RECORD_READ],
        !closing
            && !form_open
            && !snapshot.shop_pending
            && !snapshot.item_pending
            && !snapshot.ammo_pending
            && can_read_record(&snapshot),
    );
    let _ = EnableWindow(
        controls[SKILLS_READ],
        !closing
            && !form_open
            && !snapshot.shop_pending
            && !snapshot.item_pending
            && !snapshot.ammo_pending
            && snapshot.navigation.is_some()
            && !snapshot.spell_pending
            && !snapshot.conversation_pending()
            && !snapshot.record_pending
            && !snapshot.world_pending
            && !snapshot.combat_pending
            && !snapshot.room_pending
            && !snapshot.equipment_pending
            && !matches!(
                snapshot.skills,
                super::spell_protocol::SkillsState::Unsupported
            ),
    );
    let _ = EnableWindow(
        controls[CAST_TELEPORT],
        !closing && spell_input_ready(state, &snapshot, Instant::now()),
    );
    let character_index = SendMessageW(controls[CHARACTERS], LB_GETCURSEL, None, None).0;
    let current_gear = if let equipment::State::Ready(gear) = &snapshot.equipment {
        Some(gear.clone())
    } else {
        None
    };
    let old_gear = state.rendered_equipment.borrow().clone();
    if old_gear != current_gear
        || state.rendered_gear_description_revision.get() != Some(description_revision)
    {
        let index = SendMessageW(controls[GEAR_ITEM], CB_GETCURSEL, None, None).0;
        let retained = old_gear.as_ref().and_then(|gear| {
            if current_gear
                .as_ref()
                .is_some_and(|new| new.source.context == gear.source.context)
            {
                usize::try_from(index)
                    .ok()
                    .and_then(|index| gear.flags.get(index))
                    .map(|row| row.item_id)
            } else {
                None
            }
        });
        let labels: Vec<String> = if let Some(gear) = &current_gear {
            let descriptions = state.descriptions.borrow();
            gear.source
                .items
                .iter()
                .zip(&gear.flags)
                .map(|(item, flag)| {
                    format!(
                        "{} · {} · 實例 {}{}",
                        if flag.equipped { "已裝備" } else { "背包" },
                        descriptions.state.item_label(&item.name_token),
                        item.object_id,
                        gear.lifetime(item.object_id)
                            .map(|row| format!(" · {}", row.state.label()))
                            .unwrap_or_default()
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        // No RefCell/Mutex borrow survives a synchronous native-control message.
        let _ = SendMessageW(controls[GEAR_ITEM], CB_RESETCONTENT, None, None);
        for (index, label) in labels.iter().enumerate() {
            let label = wide(label);
            if SendMessageW(
                controls[GEAR_ITEM],
                CB_ADDSTRING,
                None,
                Some(LPARAM(label.as_ptr() as isize)),
            )
            .0 != index as isize
            {
                state.text_fault.set(true);
                return;
            }
        }
        if let Some(index) = retained.and_then(|id| {
            current_gear
                .as_ref()?
                .flags
                .iter()
                .position(|row| row.item_id == id)
        }) {
            let _ = SendMessageW(controls[GEAR_ITEM], CB_SETCURSEL, Some(WPARAM(index)), None);
        }
        state
            .rendered_gear_description_revision
            .set(Some(description_revision));
    }
    // A fresh READ can be value-equal yet own a new Arc. The rows remain valid,
    // but clicks must capture the currently published authority snapshot.
    *state.rendered_equipment.borrow_mut() = current_gear.clone();
    let selected_gear_index = SendMessageW(controls[GEAR_ITEM], CB_GETCURSEL, None, None).0;
    let selected_item = usize::try_from(selected_gear_index)
        .ok()
        .and_then(|index| current_gear.as_ref()?.flags.get(index))
        .map(|row| row.item_id);
    for (index, desired) in [
        (GEAR_EQUIP, Some(true)),
        (GEAR_UNEQUIP, Some(false)),
        (GEAR_READ, None),
    ] {
        let _ = EnableWindow(
            controls[index],
            !closing
                && !form_open
                && equipment_command(&snapshot, selected_item, desired).is_some(),
        );
    }
    let _ = EnableWindow(
        controls[GEAR_ITEM],
        !closing && !snapshot.equipment_pending && current_gear.is_some(),
    );
    let channel_index = SendMessageW(controls[CHANNELS], LB_GETCURSEL, None, None).0;
    let _ = EnableWindow(
        controls[SELECT_CHARACTER],
        !closing && character_context(&snapshot, character_index).is_some(),
    );
    let _ = EnableWindow(
        controls[ENTER_CHANNEL],
        !closing && channel_command(&snapshot, channel_index).is_some(),
    );
    let _ = EnableWindow(
        controls[RETURN_CHARACTERS],
        !closing && !form_open && return_command(&snapshot).is_some(),
    );
    repair_refresh_focus(hwnd, &controls, focus_before_refresh, closing);
}

fn phase_label(phase: LoginPhase) -> &'static str {
    match phase {
        LoginPhase::Disconnected => "1／4 連線：尚未連線",
        LoginPhase::Connecting => "1／4 連線：驗證伺服器與加密連線中",
        LoginPhase::LoginReady => "2／4 登入：連線已驗證，請輸入帳號密碼",
        LoginPhase::Authenticating => "2／4 登入：驗證帳號中",
        LoginPhase::ChoosingSource => "3／4 選擇資料來源／伺服器",
        LoginPhase::LoadingCharacters => "4／4 角色：正在讀取所選來源",
        LoginPhase::CharactersReady => "4／4 角色：已收到所選來源的角色列表",
        LoginPhase::SelectingCharacter => "選角：等待伺服器驗證角色擁有者",
        LoginPhase::ChoosingChannel => "頻道：角色已確認，請選擇大廳",
        LoginPhase::JoiningLobby => "大廳：等待伺服器確認加入",
        LoginPhase::InLobby => "大廳：已由伺服器確認加入",
        LoginPhase::LeavingLobby => "返回頻道：等待伺服器解除大廳人數占用，保留已選角色",
        LoginPhase::ReturningCharacters => "返回選角：等待伺服器解除角色與頻道狀態",
        LoginPhase::Disconnecting => "正在取消或中斷連線",
        LoginPhase::Failed => "連線／操作失敗；可檢查設定後重新連線",
    }
}

unsafe fn selection(hwnd: HWND, state: &WindowState, source: bool) {
    if state.rendering.get() {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    let (control, message) = if source {
        (controls[SOURCE], CB_GETCURSEL)
    } else {
        (controls[CHARACTERS], LB_GETCURSEL)
    };
    let index = SendMessageW(control, message, None, None).0;
    if index < 0 {
        return;
    }
    let chosen = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        if ui.closing {
            return;
        }
        let Some(snapshot) = &ui.last else {
            return;
        };
        if source {
            if !enabled(snapshot.phase, false).source {
                return;
            }
            snapshot
                .sources
                .get(index as usize)
                .map(|s| (Some(s.id), String::new()))
        } else {
            if snapshot.phase != LoginPhase::CharactersReady {
                return;
            }
            snapshot.characters.get(index as usize).map(|row| {
                (
                    None,
                    format!(
                        "{}\r\n來源 ID：{}  ·  角色識別碼：{}",
                        row.label, row.source_id, row.object_id
                    ),
                )
            })
        }
    };
    if let Some((source_id, text)) = chosen {
        if let Some(id) = source_id {
            send(hwnd, state, LoginCommand::SelectSource(id));
        } else {
            paint_details_text(state, controls[DETAILS], &text, false);
            refresh(hwnd, state);
        }
    }
}

unsafe fn navigation_click(hwnd: HWND, state: &WindowState, button: usize) {
    if state.rendering.get() {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    // Read native selection before borrowing UI; sending may synchronously reenter.
    let index = match button {
        SELECT_CHARACTER => SendMessageW(controls[CHARACTERS], LB_GETCURSEL, None, None).0,
        ENTER_CHANNEL => SendMessageW(controls[CHANNELS], LB_GETCURSEL, None, None).0,
        _ => -1,
    };
    let command = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        if ui.closing {
            return;
        }
        let Some(snapshot) = &ui.last else {
            return;
        };
        match button {
            SELECT_CHARACTER => {
                character_context(snapshot, index).map(LoginCommand::SelectCharacter)
            }
            ENTER_CHANNEL => channel_command(snapshot, index),
            RETURN_CHARACTERS => return_command(snapshot),
            _ => None,
        }
    };
    if let Some(command) = command {
        send(hwnd, state, command);
    }
}
unsafe fn equipment_click(hwnd: HWND, state: &WindowState, button: usize) {
    if state.rendering.get() || state.text_fault.get() {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    let index = SendMessageW(controls[GEAR_ITEM], CB_GETCURSEL, None, None).0;
    let rendered = state.rendered_equipment.borrow().clone();
    let item_id = usize::try_from(index)
        .ok()
        .and_then(|index| rendered.as_ref()?.flags.get(index))
        .map(|row| row.item_id);
    let desired = match button {
        GEAR_EQUIP => Some(true),
        GEAR_UNEQUIP => Some(false),
        _ => None,
    };
    let command = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        if ui.closing {
            return;
        }
        let Some(snapshot) = ui.last.as_ref() else {
            return;
        };
        if desired.is_some()
            && !matches!((&snapshot.equipment, &rendered), (equipment::State::Ready(current), Some(old)) if Arc::ptr_eq(current, old))
        {
            return;
        }
        equipment_command(snapshot, item_id, desired)
    };
    if let Some(command) = command {
        send(hwnd, state, command);
    }
}

fn chat_view(snapshot: &LoginSnapshot, ui_busy: bool) -> super::chat_window::View {
    super::chat_window::View {
        chat: snapshot.chat.clone(),
        busy: ui_busy
            || snapshot.phase != LoginPhase::InLobby
            || snapshot.conversation_pending()
            || snapshot.room_pending
            || snapshot.equipment_pending
            || snapshot.world_pending
            || snapshot.combat_pending
            || snapshot.spell_pending
            || snapshot.record_pending
            || snapshot.shop_pending
            || snapshot.item_pending
            || snapshot.ammo_pending,
        chat_pending: snapshot.chat_pending,
        gm_pending: snapshot.room_pending,
        gm_ready: gm_input_ready(snapshot) && !ui_busy,
        direct_supported: snapshot.direct_chat.supported(),
        server_status: snapshot.status.clone(),
    }
}
pub(crate) fn chat_command(
    snapshot: &LoginSnapshot,
    intent: super::chat_window::Intent,
) -> std::result::Result<LoginCommand, String> {
    if chat_view(snapshot, false).busy || snapshot.chat.token() != Some(intent.token) {
        return Err("聊天範圍已改變或有操作待確認；未送出。".into());
    }
    let kind = super::chat_window::parse_kind(&intent.text)?;
    let category_valid = match intent.kind {
        super::chat_window::Kind::Public(category) => {
            kind == super::chat_window::Kind::Message
                && category != super::filtered_chat_protocol::Kind::Normal
                && snapshot.chat.filtered()
        }
        _ => kind == intent.kind,
    };
    if !category_valid {
        return Err("聊天操作內容不一致，未送出。".into());
    }
    match intent.kind {
        super::chat_window::Kind::Public(kind) => Ok(LoginCommand::FilteredChatSend {
            token: intent.token,
            kind,
            text: intent.text,
        }),
        super::chat_window::Kind::Message => Ok(LoginCommand::ChatSend {
            token: intent.token,
            text: intent.text,
        }),
        super::chat_window::Kind::GmBots(count) if gm_input_ready(snapshot) => {
            Ok(LoginCommand::GmBots { count })
        }
        _ => Err("目前不適合操作假人；需在尚未開局的房間，由伺服器查驗權限與容量。".into()),
    }
}
unsafe fn open_chat(hwnd: HWND, state: &WindowState) {
    let view = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        let snapshot = ui.controller.snapshot();
        if ui.closing || snapshot.phase != LoginPhase::InLobby || !snapshot.chat.supported() {
            return;
        }
        chat_view(
            &snapshot,
            state.room_form.borrow().is_some() || state.shop_window.borrow().is_some(),
        )
    };
    let retired = if state
        .chat_window
        .borrow()
        .as_ref()
        .is_some_and(|c| c.handle().is_none())
    {
        state.chat_window.borrow_mut().take()
    } else {
        None
    };
    drop(retired);
    if let Some(chat) = state.chat_window.borrow().as_ref() {
        chat.show();
        return;
    }
    // Stop click-to-walk, but do not change authoritative movement or world state.
    state.world_navigation.borrow_mut().cancel();
    match super::chat_window::ChatWindow::open(hwnd, view) {
        Ok(chat) => {
            *state.chat_window.borrow_mut() = Some(chat);
        }
        Err(_) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("無法開啟聊天視窗；沒有送出訊息。".into());
            }
        }
    }
}
unsafe fn refresh_chat(hwnd: HWND, state: &WindowState, closing: bool) {
    let snapshot = match state.ui.try_borrow() {
        Ok(ui) => ui.controller.snapshot(),
        Err(_) => return,
    };
    if closing
        || snapshot.phase != LoginPhase::InLobby
        || !snapshot.chat.supported()
        || state
            .chat_window
            .borrow()
            .as_ref()
            .is_some_and(|c| c.handle().is_none())
    {
        let retired = state.chat_window.borrow_mut().take();
        drop(retired);
        return;
    }
    let busy = state.room_form.borrow().is_some() || state.shop_window.borrow().is_some();
    let intent = {
        let window = state.chat_window.borrow();
        let Some(window) = window.as_ref() else {
            return;
        };
        if window.update(chat_view(&snapshot, busy)).is_err() {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("聊天視窗更新失敗，沒有送出新的訊息。".into());
            }
            return;
        }
        window.take_action()
    };
    let open_direct = state
        .chat_window
        .borrow()
        .as_ref()
        .is_some_and(|w| w.take_open_direct());
    if open_direct {
        open_direct_chat(hwnd, state);
    }
    if let Some(intent) = intent {
        let kind = intent.kind;
        let result = if busy {
            Err("請先完成房間或商店操作，尚未送出。".into())
        } else {
            chat_command(&snapshot, intent).and_then(|command| {
                state
                    .ui
                    .try_borrow_mut()
                    .map_err(|_| "介面正在處理其他操作，未送出。".to_string())?
                    .controller
                    .command(command)
            })
        };
        if let Some(window) = state.chat_window.borrow().as_ref() {
            window.dispatched(kind, result);
        }
    }
}

fn direct_chat_view(snapshot: &LoginSnapshot, ui_busy: bool) -> super::direct_chat_window::View {
    super::direct_chat_window::View {
        chat: snapshot.direct_chat.clone(),
        filtered_public: snapshot.chat.filtered(),
        sources: snapshot.sources.clone(),
        busy: chat_view(snapshot, ui_busy).busy,
        pending: snapshot.direct_chat_pending,
    }
}
pub(crate) fn direct_chat_command(
    snapshot: &LoginSnapshot,
    intent: super::direct_chat_window::Intent,
) -> std::result::Result<LoginCommand, String> {
    if direct_chat_view(snapshot, false).busy
        || snapshot.direct_chat.token() != Some(intent.token)
        || intent
            .target
            .as_ref()
            .is_some_and(|t| !snapshot.sources.iter().any(|s| s.id == t.source_id))
    {
        return Err("私訊頁面或來源已改變，或仍有操作待確認；未送出。".into());
    }
    // Legacy peers expose only whispers. Category controls require the confirmed
    // recipient-local public profile; never silently alter unexposed flags.
    if !snapshot.chat.filtered()
        && intent.preferences.is_some_and(|p| {
            snapshot
                .direct_chat
                .preferences()
                .is_none_or(|confirmed| p.world != confirmed.world || p.trade != confirmed.trade)
        })
    {
        return Err("私訊設定含有本介面未提供的變更，未送出。".into());
    }
    snapshot.direct_chat.command(
        intent.token,
        intent.action,
        intent.target.clone(),
        intent.preferences,
        intent.text.clone(),
    )?;
    Ok(LoginCommand::DirectChat {
        token: intent.token,
        action: intent.action,
        target: intent.target,
        preferences: intent.preferences,
        text: intent.text,
    })
}
unsafe fn open_direct_chat(hwnd: HWND, state: &WindowState) {
    let view = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        let snapshot = ui.controller.snapshot();
        if ui.closing || snapshot.phase != LoginPhase::InLobby || !snapshot.direct_chat.supported()
        {
            return;
        }
        direct_chat_view(
            &snapshot,
            state.room_form.borrow().is_some() || state.shop_window.borrow().is_some(),
        )
    };
    let closed = state
        .direct_chat_window
        .borrow()
        .as_ref()
        .is_some_and(|w| w.handle().is_none());
    if closed {
        drop(state.direct_chat_window.borrow_mut().take());
    }
    if let Some(w) = state.direct_chat_window.borrow().as_ref() {
        w.show();
        return;
    }
    state.world_navigation.borrow_mut().cancel();
    match super::direct_chat_window::DirectChatWindow::open(hwnd, view) {
        Ok(w) => *state.direct_chat_window.borrow_mut() = Some(w),
        Err(_) => {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("無法開啟私訊視窗；沒有送出任何操作。".into());
            }
        }
    }
}
unsafe fn refresh_direct_chat(_hwnd: HWND, state: &WindowState, closing: bool) {
    let snapshot = match state.ui.try_borrow() {
        Ok(ui) => ui.controller.snapshot(),
        Err(_) => return,
    };
    if closing
        || snapshot.phase != LoginPhase::InLobby
        || !snapshot.direct_chat.supported()
        || state
            .direct_chat_window
            .borrow()
            .as_ref()
            .is_some_and(|w| w.handle().is_none())
    {
        drop(state.direct_chat_window.borrow_mut().take());
        return;
    }
    let busy = state.room_form.borrow().is_some() || state.shop_window.borrow().is_some();
    let intent = {
        let window = state.direct_chat_window.borrow();
        let Some(window) = window.as_ref() else {
            return;
        };
        if window.update(direct_chat_view(&snapshot, busy)).is_err() {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("私訊視窗更新失敗，沒有送出新的操作。".into());
            }
            return;
        }
        window.take_action()
    };
    if let Some(intent) = intent {
        let action = intent.action;
        let result = if busy {
            Err("請先完成房間或商店操作，尚未送出。".into())
        } else {
            direct_chat_command(&snapshot, intent).and_then(|command| {
                state
                    .ui
                    .try_borrow_mut()
                    .map_err(|_| "介面正在處理其他操作，未送出。".to_string())?
                    .controller
                    .command(command)
            })
        };
        if let Some(window) = state.direct_chat_window.borrow().as_ref() {
            window.dispatched(action, result);
        }
    }
}
fn parse_bot_count(text: &str) -> Option<u16> {
    if text.is_empty() || text.len() > 3 || !text.bytes().all(|c| c.is_ascii_digit()) {
        return None;
    }
    text.parse::<u16>().ok().filter(|n| *n <= 100)
}
pub(crate) fn can_open_shop(snapshot: &LoginSnapshot) -> bool {
    snapshot.phase == LoginPhase::InLobby
        && !matches!(snapshot.shop, shop::State::Unsupported)
        && !snapshot.world_owns_lobby()
        && !snapshot.room_pending
        && !snapshot.equipment_pending
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.record_pending
        && !snapshot.shop_pending
        && !snapshot.item_pending
        && !snapshot.ammo_pending
}
pub(crate) fn gm_input_ready(snapshot: &LoginSnapshot) -> bool {
    snapshot.gm_supported
        && snapshot.phase == LoginPhase::InLobby
        && !snapshot.room_pending
        && !snapshot.equipment_pending
        && !snapshot.world_owns_lobby()
        && !snapshot.spell_pending
        && !snapshot.conversation_pending()
        && !snapshot.record_pending
        && snapshot.navigation.as_ref().is_some_and(|nav| {
            snapshot
                .rooms
                .joined()
                .is_some_and(|r| r.member(nav.context.key).is_some())
        })
}
unsafe fn gm_click(hwnd: HWND, state: &WindowState) {
    if state.rendering.get() || state.text_fault.get() || state.room_form.borrow().is_some() {
        return;
    }
    let Some(controls) = state.controls.get() else {
        return;
    };
    // Native sends happen before the UI borrow; no synchronous re-entry while borrowed.
    let count = read_edit(controls[GM_COUNT], 3)
        .ok()
        .and_then(|s| parse_bot_count(&s));
    let ready = state
        .ui
        .try_borrow()
        .ok()
        .is_some_and(|ui| !ui.closing && ui.last.as_ref().is_some_and(gm_input_ready));
    if ready {
        if let Some(count) = count {
            send(hwnd, state, LoginCommand::GmBots { count });
        } else {
            if let Ok(mut ui) = state.ui.try_borrow_mut() {
                ui.local_error = Some("假人數量需填 0 至 100 的整數；0 表示清除。".into());
            }
            refresh(hwnd, state);
        }
    }
}
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize as _);
    }
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const WindowState;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &*pointer;
    match message {
        WM_DRAWITEM if lparam.0 != 0 => {
            let item = &*(lparam.0 as *const DRAWITEMSTRUCT);
            if item.CtlID == (FIRST_ID + VIP_ITEMS) as u32 {
                let selected = (item.itemState.0 & ODS_SELECTED.0) != 0;
                let brush = if selected { COLOR_HIGHLIGHT } else { COLOR_WINDOW };
                let _ = FillRect(item.hDC, &item.rcItem, GetSysColorBrush(brush));
                if state.vip_inventory_visible.get() && item.itemID != u32::MAX {
                    let inventory = state.vip_inventory.borrow();
                    if let Some(row) = inventory.model().and_then(|m| m.rows.get(item.itemID as usize)) {
                        let art = inventory.art(row.graphic);
                        if let Some(Ok(art)) = art {
                            if art.bgra.len() == (super::vip_inventory_preview::ICON_SIDE
                                * super::vip_inventory_preview::ICON_SIDE * 4) as usize {
                                let bitmap = BITMAPINFO {
                                    bmiHeader: BITMAPINFOHEADER {
                                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                                        biWidth: super::vip_inventory_preview::ICON_SIDE as i32,
                                        biHeight: -(super::vip_inventory_preview::ICON_SIDE as i32),
                                        biPlanes: 1, biBitCount: 32, biCompression: BI_RGB.0,
                                        ..Default::default()
                                    }, ..Default::default()
                                };
                                let side = scale(32, GetDpiForWindow(hwnd))
                                    .min(item.rcItem.bottom - item.rcItem.top - 4).max(0);
                                if side > 0 {
                                    let _ = StretchDIBits(item.hDC, item.rcItem.left + 4,
                                        item.rcItem.top + 2, side, side, 0, 0,
                                        super::vip_inventory_preview::ICON_SIDE as i32,
                                        super::vip_inventory_preview::ICON_SIDE as i32,
                                        Some(art.bgra.as_ptr().cast()), &bitmap,
                                        DIB_RGB_COLORS, SRCCOPY);
                                }
                            }
                        }
                        let status = match art {
                            Some(Ok(_)) => "",
                            Some(Err(_)) => " · 圖像缺失",
                            None if row.graphic < 0 => " · 無有效圖號",
                            None => " · 圖像載入中",
                        };
                        let equipped = match row.equipped {
                            Some(true) => "已裝備", Some(false) => "背包", None => "穿戴未確認",
                        };
                        let line = format!("{} ×{} · {} · 實例 {}{}",
                            row.label, row.quantity, equipped, row.object, status);
                        let mut wide_line = line.encode_utf16().collect::<Vec<_>>();
                        let mut rect = item.rcItem;
                        rect.left += scale(44, GetDpiForWindow(hwnd));
                        rect.right -= 4;
                        let prior_color = windows::Win32::Graphics::Gdi::SetTextColor(
                            item.hDC, windows::Win32::Foundation::COLORREF(
                                GetSysColor(if selected { COLOR_HIGHLIGHTTEXT }
                                    else { COLOR_WINDOWTEXT })));
                        let prior_mode = windows::Win32::Graphics::Gdi::SetBkMode(
                            item.hDC, windows::Win32::Graphics::Gdi::TRANSPARENT);
                        let _ = DrawTextW(item.hDC, &mut wide_line, &mut rect,
                            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX);
                        let _ = windows::Win32::Graphics::Gdi::SetTextColor(item.hDC, prior_color);
                        let _ = windows::Win32::Graphics::Gdi::SetBkMode(item.hDC,
                            windows::Win32::Graphics::Gdi::BACKGROUND_MODE(prior_mode as u32));
                    }
                }
                return LRESULT(1);
            }
            if item.CtlID != (FIRST_ID + VIP_STATUS_STRIP) as u32 {
                return DefWindowProcW(hwnd, message, wparam, lparam);
            }
            let _ = FillRect(item.hDC, &item.rcItem, GetSysColorBrush(COLOR_BTNFACE));
            if state.vip_status_visible.get() {
                if let Some(image) = state.vip_status.borrow().image() {
                    let bitmap = BITMAPINFO {
                        bmiHeader: BITMAPINFOHEADER {
                            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                            biWidth: super::vip_status_preview::WIDTH as i32,
                            biHeight: -(super::vip_status_preview::HEIGHT as i32),
                            biPlanes: 1,
                            biBitCount: 32,
                            biCompression: BI_RGB.0,
                            ..Default::default()
                        },
                        ..Default::default()
                    };
                    let dpi = GetDpiForWindow(hwnd);
                    let width = scale(super::vip_status_preview::WIDTH as i32, dpi)
                        .min(item.rcItem.right - item.rcItem.left);
                    let height = scale(super::vip_status_preview::HEIGHT as i32, dpi)
                        .min(item.rcItem.bottom - item.rcItem.top);
                    if width > 0 && height > 0 {
                        let _ = StretchDIBits(
                            item.hDC,
                            item.rcItem.left,
                            item.rcItem.top,
                            width, height,
                            0, 0,
                            super::vip_status_preview::WIDTH as i32,
                            super::vip_status_preview::HEIGHT as i32,
                            Some(image.bgra.as_ptr().cast()),
                            &bitmap, DIB_RGB_COLORS, SRCCOPY,
                        );
                    }
                }
            }
            LRESULT(1)
        }
        WM_TIMER if wparam.0 == TIMER => {
            refresh(hwnd, state);
            LRESULT(0)
        }
        WM_COMMAND => {
            if state.rendering.get() || state.text_fault.get() {
                return LRESULT(0);
            }
            let id = wparam.0 & 0xffff;
            let notification = (wparam.0 >> 16) as u32;
            if notification == BN_CLICKED {
                match id.checked_sub(FIRST_ID) {
                    Some(SETTINGS) => {
                        let ready = state.ui.try_borrow().ok().is_some_and(|ui| !ui.closing);
                        if ready {
                            let result = state.settings.borrow_mut().show(hwnd);
                            if result.is_err() {
                                local_error(hwnd, state, "無法開啟遊戲設定；未送出遊戲操作。");
                            }
                        }
                    }
                    Some(CHAT) => open_chat(hwnd, state),
                    Some(SHOW_SKILLS) => show_player_detail(hwnd, state, DetailView::Skills),
                    Some(SKILLS_READ) => send(hwnd, state, LoginCommand::ReadSkills),
                    Some(SHOW_RECORD) => {
                        state.detail_view.set(DetailView::Record);
                        refresh(hwnd, state);
                    }
                    Some(RECORD_READ) => send(hwnd, state, LoginCommand::ReadRecord),
                    Some(GM_SEND) => gm_click(hwnd, state),
                    Some(SHOP_OPEN) => open_shop(hwnd, state),
                    Some(CAST_TELEPORT) => {
                        state.world_navigation.borrow_mut().cancel();
                        let ready = state.ui.try_borrow().ok().is_some_and(|ui| {
                            !ui.closing
                                && ui.last.as_ref().is_some_and(|snapshot| {
                                    spell_input_ready(state, snapshot, Instant::now())
                                })
                        });
                        if ready {
                            send(hwnd, state, LoginCommand::CastTeleport);
                        } else if let Ok(mut ui) = state.ui.try_borrow_mut() {
                            ui.local_error =
                                Some("魔法尚未就緒或正在等待動作間隔；未送出施法。".into());
                        }
                    }
                    Some(CONNECT) => send(hwnd, state, LoginCommand::Connect),
                    Some(LOGIN) => submit_login(hwnd, state),
                    Some(DISCONNECT) => send(hwnd, state, LoginCommand::Disconnect),
                    Some(EXIT) => request_close(hwnd, state),
                    Some(MATCH_START) => matchmaking_click(hwnd, state, false),
                    Some(MATCH_CANCEL) => matchmaking_click(hwnd, state, true),
                    Some(
                        button @ (ROOM_CREATE | ROOM_JOIN | ROOM_CONFIGURE | ROOM_LEAVE
                        | ROOM_TEAM0 | ROOM_TEAM1 | ROOM_READY | ROOM_READ),
                    ) => room_click(hwnd, state, button),
                    Some(button @ (GEAR_EQUIP | GEAR_UNEQUIP | GEAR_READ)) => {
                        equipment_click(hwnd, state, button)
                    }
                    Some(button @ (SHOW_STATS | SHOW_ITEMS)) => {
                        show_player_detail(
                            hwnd,
                            state,
                            if button == SHOW_STATS {
                                DetailView::Stats
                            } else {
                                DetailView::Items
                            },
                        );
                    }
                    Some(button @ (SELECT_CHARACTER | ENTER_CHANNEL | RETURN_CHARACTERS)) => {
                        navigation_click(hwnd, state, button)
                    }
                    _ => {}
                }
            } else if id == FIRST_ID + SOURCE && notification == CBN_SELCHANGE {
                selection(hwnd, state, true);
            } else if id == FIRST_ID + CHARACTERS && notification == LBN_SELCHANGE {
                selection(hwnd, state, false);
            } else if id == FIRST_ID + CHANNELS && notification == LBN_SELCHANGE {
                refresh(hwnd, state);
            } else if id == FIRST_ID + ROOM_LIST && notification == LBN_SELCHANGE {
                refresh(hwnd, state);
            } else if id == FIRST_ID + GEAR_ITEM && notification == CBN_SELCHANGE {
                refresh(hwnd, state);
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            request_close(hwnd, state);
            LRESULT(0)
        }
        WM_SIZE => {
            resize(hwnd, state);
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let dpi = GetDpiForWindow(hwnd).max(96);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: scale(MIN_WIDTH_DIP, dpi),
                bottom: scale(MIN_HEIGHT_DIP, dpi),
            };
            if AdjustWindowRectExForDpi(&mut rect, STYLE, false, EX_STYLE, dpi).is_ok() {
                let info = &mut *(lparam.0 as *mut MINMAXINFO);
                info.ptMinTrackSize.x = rect.right - rect.left;
                info.ptMinTrackSize.y = rect.bottom - rect.top;
            }
            LRESULT(0)
        }
        WM_DPICHANGED => {
            let rect = &*(lparam.0 as *const RECT);
            let _ = SetWindowPos(
                hwnd,
                None,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            );
            if replace_font(hwnd, state).is_err() {
                local_error(hwnd, state, "無法更新顯示字型。");
            }
            resize(hwnd, state);
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = KillTimer(Some(hwnd), TIMER);
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            state.controls.set(None);
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    #[test]
    fn refresh_focus_repair_never_steals_existing_or_background_focus() {
        assert!(restore_disabled_control_focus(false, true, false, true));
        assert!(!restore_disabled_control_focus(true, true, false, true));
        assert!(!restore_disabled_control_focus(false, false, false, true));
        assert!(!restore_disabled_control_focus(false, true, true, true));
        assert!(!restore_disabled_control_focus(false, true, false, false));
    }

    #[test]
    fn navigation_trace_records_only_three_navigation_keys_and_no_text() {
        for message in [
            WM_KEYDOWN,
            WM_KEYUP,
            WM_SYSKEYDOWN,
            WM_SYSKEYUP,
            WM_CHAR,
            WM_COMMAND,
        ] {
            for key in 0..=255 {
                let observed = traced_navigation_key(&MSG {
                    message,
                    wParam: WPARAM(key),
                    ..Default::default()
                });
                let expected = if matches!(message, WM_KEYDOWN | WM_KEYUP) {
                    match key {
                        0x09 => Some("tab"),
                        0x0d => Some("return"),
                        0x1b => Some("escape"),
                        _ => None,
                    }
                } else {
                    None
                };
                assert_eq!(observed, expected);
            }
        }
    }

    #[test]
    fn fallback_escape_needs_a_new_press_after_world_retirement() {
        for message in [WM_KEYDOWN, WM_SYSKEYDOWN, WM_KEYUP, WM_SYSKEYUP, WM_CHAR] {
            for key in [0x1b, 0x0d, 0x09, 0x43, 0x79, 0x7a] {
                for repeated in [false, true] {
                    let message = MSG {
                        message,
                        wParam: WPARAM(key),
                        lParam: LPARAM(1 | if repeated { 1 << 30 } else { 0 }),
                        ..Default::default()
                    };
                    let expected = if message.message == WM_KEYDOWN && key == 0x1b {
                        Some(if repeated {
                            EscapeFallback::Consume
                        } else {
                            EscapeFallback::Close
                        })
                    } else {
                        None
                    };
                    assert_eq!(fallback_escape(&message), expected);
                }
            }
        }
    }
    fn details_snapshot(reply: crate::modern_lito::combat_protocol::Reply) -> LoginSnapshot {
        let mut view = channel_snapshot();
        let world = reply.world_reply().unwrap();
        let context = world.echo.context;
        let member = world
            .frame
            .as_ref()
            .unwrap()
            .room
            .member(context.key)
            .unwrap();
        view.phase = LoginPhase::InLobby;
        view.selected_source = Some(context.key.source_id);
        view.sources = vec![SourceChoice {
            id: context.key.source_id,
            label: member.source_name.clone(),
        }];
        let nav = view.navigation.as_mut().unwrap();
        nav.context = context;
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(world.echo.channel_id);
        nav.snapshot.selected = Some(crate::modern_lito::application_protocol::CharacterSummary {
            key: context.key,
            name: member.name.clone(),
            class: member.class,
            sex: member.sex,
            level: member.level,
        });
        let mut data = crate::modern_lito::character_data::tests::sample_data();
        data.context = context;
        data.character = nav.snapshot.selected.clone().unwrap();
        data.validate().unwrap();
        view.character_data = CharacterDataState::Ready(Arc::new(data));
        view.world = world::State::Ready(Arc::new(world));
        view.combat = reply.frame.map(Arc::new);
        view.combat_supported = true;
        view
    }
    #[test]
    fn world_details_keep_live_signed_64_bit_vitals_separate_from_source_food_and_hp() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let mut reply = crate::modern_lito::item_protocol::tests::initial();
        let vital = &mut reply.frame.as_mut().unwrap().vitals[0];
        vital.hp = i64::MAX - 1;
        vital.max_hp = i64::MAX;
        vital.ac = i64::MIN;
        let view = details_snapshot(reply);
        let before = view.clone();
        let text = world_details_text(&view, P::Character, &DescriptionState::Loading).unwrap();
        assert!(text.contains("HP：9223372036854775806 / 9223372036854775807"));
        assert!(text.contains("AC：-9223372036854775808"));
        assert!(text.contains("保存 HP：9223372036854775807"));
        assert!(text.contains("飽食度：225/225（100%）"));
        assert!(text.contains("不是即時戰鬥能力"));
        assert_eq!(
            view, before,
            "inspection cannot enqueue a mutation or edit HP"
        );
    }
    #[test]
    fn world_details_do_not_promote_stale_or_missing_combat_to_live_source_stats() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        for missing in [false, true] {
            let mut stale = view.clone();
            if missing {
                stale.combat = None;
            } else {
                Arc::make_mut(stale.combat.as_mut().unwrap()).world.revision += 1;
            }
            let text =
                world_details_text(&stale, P::Character, &DescriptionState::Loading).unwrap();
            assert!(text.contains("等待同一世界版本"));
            assert!(!text.contains("HP：30 / 100"));
            assert!(text.contains("保存 HP：9223372036854775807"));
            let inventory =
                world_details_text(&stale, P::Inventory, &DescriptionState::Loading).unwrap();
            assert!(!inventory.contains("補給 #40010"));
            assert!(inventory.contains("不代表來源背包是空的"));
        }
    }
    #[test]
    fn world_details_inventory_has_real_match_counts_and_every_source_row_without_merging() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let mut view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        let CharacterDataState::Ready(data) = &mut view.character_data else {
            unreachable!()
        };
        let data = Arc::make_mut(data);
        let template = data.items[0].clone();
        data.items = (1..=512)
            .map(|i| {
                let mut row = template.clone();
                row.object_id = i;
                row.name_token = format!("物品{i} {}", "名".repeat(75));
                row
            })
            .collect();
        data.validate().unwrap();
        let text = world_details_text(&view, P::Inventory, &DescriptionState::Loading).unwrap();
        assert!(text.contains("補給 #40010：剩餘 3／開場 3"));
        assert!(text.contains("補給 #40011：剩餘 1／開場 1"));
        assert!(text.contains("物品1 ") && text.contains("物品512 "));
        assert!(text.contains("數量 9223372036854775807"));
        assert!(text.contains("此頁不在戰鬥中穿脫裝備"));
        assert!(text.encode_utf16().count() <= crate::modern_lito::world_window::MAX_DETAILS_UTF16);
    }
    #[test]
    fn world_details_reject_foreign_source_epoch_channel_and_disconnect() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        for case in 0..6 {
            let mut bad = view.clone();
            match case {
                0 => bad.selected_source = Some(999),
                1 => bad.navigation.as_mut().unwrap().context.epoch += 1,
                2 => bad.navigation.as_mut().unwrap().snapshot.channel_id = Some(3),
                3 => bad.phase = LoginPhase::Disconnecting,
                4 => {
                    let CharacterDataState::Ready(data) = &mut bad.character_data else {
                        unreachable!()
                    };
                    Arc::make_mut(data).context.key.source_id += 1;
                }
                _ => bad.world = world::State::NotRequested,
            }
            for page in [P::Character, P::Inventory, P::Skills] {
                assert!(
                    world_details_text(&bad, page, &DescriptionState::Loading).is_none(),
                    "{case:?} {page:?}"
                );
            }
        }
    }
    #[test]
    fn world_details_skills_distinguish_learned_support_from_this_match_admission() {
        use crate::modern_lito::{spell_protocol as spell, world_window::DetailsPage as P};
        let mut view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: view.navigation.as_ref().unwrap().context,
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
                    skill_id: 6,
                    support: spell::Support::Unimplemented,
                    mp_cost: 9,
                    reuse_millis: 10,
                    name: "未實作技能".into(),
                },
            ],
        }));
        let loading = world_details_text(&view, P::Skills, &DescriptionState::Loading).unwrap();
        assert!(loading.contains("名稱載入中（共享 Lineage Text）"));
        assert!(loading.contains("原始名稱 token：光箭"));
        let descriptions = DescriptionState::Ready(Arc::new(
            DescriptionCatalog::parse_cp950(b"1\r\nname").unwrap(),
        ));
        let text = world_details_text(&view, P::Skills, &descriptions).unwrap();
        assert!(text.contains("#4 光箭 · 本場未開放"), "{text}");
        assert!(text.contains("#6 未實作技能 · 尚未實作，不可施放"));
        let mut enabled =
            details_snapshot(crate::modern_lito::spell_protocol::tests::initial_damage());
        enabled.skills = view.skills.clone();
        let text = world_details_text(&enabled, P::Skills, &descriptions).unwrap();
        assert!(text.contains("#4 光箭 · 本場規則開放"));
        assert!(text.contains("來源模板MP不是本場最終耗魔"));
        let spell::SkillsState::Ready(skills) = &mut enabled.skills else {
            unreachable!()
        };
        Arc::make_mut(skills).context.epoch += 1;
        assert!(world_details_text(&enabled, P::Skills, &DescriptionState::Loading).is_none());
    }
    #[test]
    fn world_details_cache_observes_new_authority_and_does_not_refresh_for_mouse_or_status() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        let old = WorldDetailsCache::new(&view, P::Character, 0);
        let mut noise = view.clone();
        noise.status = "another timer or cursor event".into();
        assert!(old == WorldDetailsCache::new(&noise, P::Character, 0));
        assert!(old != WorldDetailsCache::new(&view, P::Inventory, 0));
        assert!(old != WorldDetailsCache::new(&view, P::Character, 1));
        let mut fresh = view.clone();
        Arc::make_mut(fresh.combat.as_mut().unwrap()).vitals[0].hp += 1;
        assert!(old != WorldDetailsCache::new(&fresh, P::Character, 0));
        fresh = view.clone();
        fresh.record_pending = true;
        assert!(old != WorldDetailsCache::new(&fresh, P::Character, 0));
        fresh = view.clone();
        fresh.navigation.as_mut().unwrap().context.epoch += 1;
        assert!(old != WorldDetailsCache::new(&fresh, P::Character, 0));
        assert!(
            old == WorldDetailsCache::new(&view, P::Character, 0),
            "retained inputs were not mutated"
        );
    }
    #[test]
    fn world_details_cache_refreshes_when_only_inventory_focus_changes() {
        use crate::modern_lito::{
            battle_inventory::{Key, Selection},
            world_window::DetailsPage,
        };
        let view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        let summary = WorldDetailsCache::new(&view, DetailsPage::Inventory, 0);
        let stamp = crate::modern_lito::battle_window::Stamp::of(
            &crate::modern_lito::battle_window::tests::scene().observation,
        );
        let mut first = summary.clone();
        first.inventory = Some(Selection {
            stamp,
            key: Key::Source(40010),
        });
        let mut second = first.clone();
        second.inventory.as_mut().unwrap().key = Key::Supply(40010);
        assert!(summary != first && first != second && summary != second);
        second.inventory = None;
        assert!(second == summary);
    }
    #[test]
    fn world_details_sparse_npc_uses_typed_member_not_vector_index_or_bot_key() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let reply = crate::modern_lito::npc_protocol::tests::isolated_guest().combat_reply();
        let view = details_snapshot(reply);
        let text = world_details_text(&view, P::Character, &DescriptionState::Loading).unwrap();
        assert!(text.starts_with("human-1(server-42) · 來源 42／角色 1"));
        assert!(text.contains("HP：9007199254740993 / 9007199254740993"));
        assert!(!text.contains("BOT1"));
        let mut wrong = view.clone();
        wrong
            .navigation
            .as_mut()
            .unwrap()
            .snapshot
            .selected
            .as_mut()
            .unwrap()
            .key
            .source_id = 7;
        assert!(world_details_text(&wrong, P::Character, &DescriptionState::Loading).is_none());
    }
    #[test]
    fn world_details_noncombat_race_does_not_invent_live_vitals() {
        use crate::modern_lito::{world_protocol as wp, world_window::DetailsPage as P};
        let mut view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        let frame = crate::modern_lito::race_protocol::tests::running();
        let context = view.navigation.as_ref().unwrap().context;
        assert_eq!(frame.actors[0].key, context.key);
        view.world = world::State::Ready(Arc::new(wp::Reply {
            echo: wp::Request::read(
                context,
                view.navigation
                    .as_ref()
                    .unwrap()
                    .snapshot
                    .channel_id
                    .unwrap(),
            ),
            status: wp::Status::Ok,
            reason: wp::Reason::None,
            frame: Some(frame),
        }));
        view.combat = None;
        let text = world_details_text(&view, P::Character, &DescriptionState::Loading).unwrap();
        assert!(text.contains("本場非戰鬥規則，沒有即時HP／MP／AC"));
        assert!(!text.contains("HP：30 / 100"));
    }
    #[test]
    fn world_details_reject_equipment_with_same_context_but_different_source_snapshot() {
        use crate::modern_lito::world_window::DetailsPage as P;
        let mut view = details_snapshot(crate::modern_lito::item_protocol::tests::initial());
        let gear = equipment::tests::sample_snapshot(view.navigation.as_ref().unwrap().context);
        view.character_data = CharacterDataState::Ready(gear.source.clone());
        view.equipment = equipment::State::Ready(gear);
        for page in [P::Character, P::Inventory, P::Skills] {
            assert!(world_details_text(&view, page, &DescriptionState::Loading).is_some());
        }
        let CharacterDataState::Ready(source) = &mut view.character_data else {
            unreachable!()
        };
        Arc::make_mut(source).character.name.push('X');
        for page in [P::Character, P::Inventory, P::Skills] {
            assert!(world_details_text(&view, page, &DescriptionState::Loading).is_none());
        }
    }
    #[test]
    fn matchmaking_controls_follow_confirmed_queue_and_all_pending_gates() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        let mut directory = rooms::tests::sample(context, false);
        directory
            .options
            .iter_mut()
            .for_each(|o| o.world_available = true);
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert!(matchmaking_common(&view, false).is_none());
        view.matchmaking = matching::State::NotRequested;
        assert!(matchmaking_common(&view, false).is_some());
        assert!(matchmaking_common(&view, true).is_none());
        let mut reply = super::super::matchmaking_protocol::tests::queued();
        reply.echo.context = context;
        reply.echo.channel_id = 0;
        reply.snapshot.as_mut().unwrap().rooms = directory;
        view.matchmaking = matching::State::Ready(Arc::new(reply.clone()));
        assert!(matchmaking_common(&view, false).is_none());
        assert!(matchmaking_common(&view, true).is_some());
        validate_snapshot(&view).unwrap();
        // A previous room poll can confirm that no world exists. That empty
        // receipt must not invalidate a subsequent lobby-return intent.
        view.world = world::State::Ready(Arc::new(world::Reply {
            echo: world::Request::read(context, 0),
            status: world::Status::Ok,
            reason: world::Reason::None,
            frame: None,
        }));
        for phase in [LoginPhase::LeavingLobby, LoginPhase::ReturningCharacters] {
            let mut pending = view.clone();
            pending.phase = phase;
            validate_snapshot(&pending).unwrap();
            assert!(matchmaking_common(&pending, true).is_none());
        }
        for kind in 0..10 {
            let mut busy = view.clone();
            match kind {
                0 => busy.room_pending = true,
                1 => busy.equipment_pending = true,
                2 => busy.world_pending = true,
                3 => busy.combat_pending = true,
                4 => busy.spell_pending = true,
                5 => busy.record_pending = true,
                6 => busy.chat_pending = true,
                7 => busy.shop_pending = true,
                8 => busy.item_pending = true,
                _ => busy.ammo_pending = true,
            }
            assert!(matchmaking_common(&busy, true).is_none(), "pending {kind}");
        }
        reply.echo.context.epoch += 1;
        reply.snapshot.as_mut().unwrap().rooms.common.context = reply.echo.context;
        view.matchmaking = matching::State::Ready(Arc::new(reply));
        assert!(validate_snapshot(&view).is_err());
    }
    #[test]
    fn matchmaking_room_controls_do_not_overlap_at_supported_dpi() {
        for dpi in [96, 120, 144, 192, 288, 768] {
            let boxes = layout(scale(MIN_WIDTH_DIP, dpi), scale(MIN_HEIGHT_DIP, dpi), dpi);
            let shown = [
                ROOM_CREATE,
                ROOM_JOIN,
                ROOM_CONFIGURE,
                ROOM_LEAVE,
                ROOM_TEAM0,
                ROOM_TEAM1,
                ROOM_READY,
                ROOM_READ,
                MATCH_START,
                MATCH_CANCEL,
                DISCONNECT,
            ];
            for (n, &i) in shown.iter().enumerate() {
                let a = boxes[i];
                assert!(a.width >= scale(130, dpi));
                for &j in &shown[n + 1..] {
                    let b = boxes[j];
                    assert!(
                        a.x + a.width <= b.x
                            || b.x + b.width <= a.x
                            || a.y + a.height <= b.y
                            || b.y + b.height <= a.y,
                        "{i}/{j} dpi {dpi}"
                    );
                }
            }
        }
    }
    #[test]
    fn healing_then_timed_use_preserves_healing_cooldown_and_coalesced_heal_is_conservative() {
        use super::*;
        let t = Instant::now();
        let initial = item::tests::initial();
        let old = initial.frame.as_ref().unwrap().items[0].clone();
        let healed = item::tests::used(&initial);
        let heal = healed.frame.as_ref().unwrap().items[0].clone();
        let previous = (1, 1, old);
        let receipt = observe_healing_receipt(None, Some(&previous), &heal, 1, 1, t).unwrap();
        assert_eq!(receipt.5, 500);
        let mut buff = heal.clone();
        buff.item_id = 40013;
        buff.effect = 191;
        buff.restored = 0;
        buff.hp_before = buff.hp_after;
        buff.delay_millis = 0;
        buff.use_revision += 1;
        let same = observe_healing_receipt(
            Some(receipt),
            Some(&(1, 1, heal)),
            &buff,
            1,
            1,
            t + Duration::from_millis(100),
        );
        assert_eq!(same, Some(receipt));
        assert!(!attack_interval_ready(
            Some(same.unwrap().4),
            u64::from(same.unwrap().5) * 1_000_000,
            t + Duration::from_millis(499)
        ));
        let coalesced = observe_healing_receipt(None, Some(&previous), &buff, 1, 1, t).unwrap();
        assert_eq!(coalesced.5, 1000);
        assert!(observe_healing_receipt(
            Some(receipt),
            Some(&(1, 1, buff.clone())),
            &buff,
            2,
            1,
            t
        )
        .is_some());
    }
    #[test]
    fn confirmed_timed_pace_changes_cadence_but_local_time_never_expires_authority() {
        use super::*;
        use crate::modern_lito::timed_protocol::{Kind, State};
        let mut combat = crate::modern_lito::world_runtime::tests::combat_frame();
        let actor = combat.world.actors[0].clone();
        let t = Instant::now();
        let mut s = State::initial(actor.entity_id);
        s.revision = 1;
        s.haste_until_nanos = Kind::Haste.duration_millis() * 1_000_000;
        combat.timed.push(s);
        let move_base = actor.step_interval_nanos;
        let fast = super::super::timed_protocol::effective_nanos(
            &combat,
            actor.entity_id,
            move_base,
            super::super::timed_protocol::Pace::Move,
        );
        assert!(fast < move_base);
        assert!(!confirmed_cadence_ready(
            Some(&combat),
            &actor,
            Some(t),
            None,
            None,
            t + Duration::from_nanos(fast - 1)
        ));
        assert!(confirmed_cadence_ready(
            Some(&combat),
            &actor,
            Some(t),
            None,
            None,
            t + Duration::from_nanos(fast)
        ));
        combat.timed[0].notice_estimate(Duration::from_secs(999));
        assert!(combat.timed[0].haste());
        combat.timed[0].haste_until_nanos = 0;
        assert!(!confirmed_cadence_ready(
            Some(&combat),
            &actor,
            Some(t),
            None,
            None,
            t + Duration::from_nanos(fast)
        ));
        assert!(confirmed_cadence_ready(
            Some(&combat),
            &actor,
            Some(t),
            None,
            None,
            t + Duration::from_nanos(move_base)
        ));
    }
    #[test]
    fn natural_recovery_does_not_reset_action_cadence_or_invent_a_healing_receipt() {
        use super::*;
        let old = crate::modern_lito::regen_protocol::tests::initial();
        let new = crate::modern_lito::regen_protocol::tests::recovered(&old, 1, -5, -2);
        let a = old.frame.as_ref().unwrap();
        let b = new.frame.as_ref().unwrap();
        let actor = &a.world.actors[0];
        let t = Instant::now();
        for delta in [0, 199, 200, 699, 700, 1199, 1200] {
            let now = t + Duration::from_millis(delta);
            assert_eq!(
                confirmed_cadence_ready(Some(a), actor, Some(t), Some(t), Some((t, 1200)), now),
                confirmed_cadence_ready(
                    Some(b),
                    &b.world.actors[0],
                    Some(t),
                    Some(t),
                    Some((t, 1200)),
                    now
                )
            );
        }
        assert!(observe_healing_receipt(
            None,
            Some(&(a.world.match_id, a.world.generation, a.items[0].clone())),
            &b.items[0],
            b.world.match_id,
            b.world.generation,
            t
        )
        .is_none());
        assert_eq!(a.items, b.items);
        assert_eq!(a.world.casts, b.world.casts);
        assert_eq!(a.timed, b.timed);
        assert_eq!(a.ammo, b.ammo);
    }
    use super::*;
    use std::time::Duration;

    #[test]
    fn completion_text_is_authority_identity_scoped_and_preserves_commit_uncertainty() {
        let mut frame = super::super::world_protocol::tests::finished();
        let first = frame.actors[0].key;
        let second = frame.actors[1].key;
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("本場勝利"));
        assert!(completion_notice(&frame, second)
            .unwrap()
            .contains("本場敗北"));
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("尚未確認"));
        frame.completion.as_mut().unwrap().state = world::SettlementState::Saved;
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("本場結果已保存"));
        frame.completion.as_mut().unwrap().state = world::SettlementState::Failed;
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("可能已入帳"));
        assert!(completion_notice(
            &frame,
            super::super::application_protocol::CharacterKey {
                source_id: 99,
                object_id: first.object_id
            }
        )
        .is_none());
        frame.completion = None;
        assert!(completion_notice(&frame, first).is_none());
    }

    #[test]
    fn quest_progress_and_result_text_use_own_lane_and_source_qualified_public_verdict() {
        use super::super::quest_protocol::tests::sample;
        for (name, verdict) in [("win", "本場勝利"), ("loss", "本場敗北")] {
            let reply = sample(name);
            let frame = &reply.frame.as_ref().unwrap().combat.world;
            let text = completion_notice(frame, frame.actors[0].key).unwrap();
            assert!(text.contains(verdict));
            assert!(text.contains("尚未確認")); // Java golden is durable-PENDING, not SAVED.
            assert!(quest_notice(frame).unwrap().starts_with("己方第 "));
        }
        let before = sample("before");
        assert_eq!(
            quest_notice(&before.frame.as_ref().unwrap().combat.world).unwrap(),
            "己方第 4/4 關｜最終守護者：1/1。"
        );
    }

    #[test]
    fn race_notice_keeps_source_scoped_placement_dnf_and_commit_uncertainty_separate() {
        use super::super::race_protocol::{tests, Settlement};
        let mut frame = tests::complete();
        let first = frame.actors[0].key;
        let second = frame.actors[1].key;
        let placed = completion_notice(&frame, first).unwrap();
        assert!(
            placed.contains("第 1 名") && placed.contains("1.000 秒") && placed.contains("等待")
        );
        assert!(!placed.contains("本場勝利") && !placed.contains("本場敗北"));
        let dnf = completion_notice(&frame, second).unwrap();
        assert!(dnf.contains("未完賽") && dnf.contains("收尾時間結束") && !dnf.contains("第 1 名"));
        assert!(completion_notice(
            &frame,
            super::super::application_protocol::CharacterKey {
                source_id: 99,
                ..first
            }
        )
        .is_none());
        frame.revision += 1;
        frame.race.as_mut().unwrap().settlement = Settlement::Saved;
        frame.validate(first).unwrap();
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("獨立保存"));
        frame.race.as_mut().unwrap().settlement = Settlement::Failed;
        assert!(completion_notice(&frame, first)
            .unwrap()
            .contains("可能已入帳"));
        let running = tests::running();
        assert!(race_notice(&running, first)
            .unwrap()
            .contains("已完成 0/4 圈"));
        assert!(race_notice(&running, first)
            .unwrap()
            .contains("下一檢查點 1/2"));
    }

    #[test]
    fn combat_unknown_dead_and_receipt_cooldown_do_not_allow_fabricated_alive_or_retarget_reset() {
        use std::time::Duration;
        assert!(combat_alive(false, None));
        assert!(!combat_alive(true, None));
        let mut vital = super::super::world_runtime::tests::combat_frame()
            .vitals
            .remove(0);
        assert!(combat_alive(true, Some(&vital)));
        vital.hp = 0;
        vital.death_revision = 1;
        assert!(!combat_alive(true, Some(&vital)));
        let receipt = Instant::now();
        assert!(!attack_interval_ready(
            Some(receipt),
            1_000_000_000,
            receipt + Duration::from_millis(999)
        ));
        assert!(attack_interval_ready(
            Some(receipt),
            1_000_000_000,
            receipt + Duration::from_secs(1)
        ));
        assert!(!attack_interval_ready(
            Some(receipt),
            1,
            receipt - Duration::from_nanos(1)
        ));
    }

    #[test]
    fn interim_combat_none_pauses_but_keeps_route_and_both_action_cooldowns() {
        use super::super::{
            resources::SharedMapSectorKey,
            world_map::{SectorInput, WorldCell, WorldMap},
        };
        use std::time::Duration;
        let cancel = AtomicBool::new(false);
        let map = Arc::new(
            WorldMap::from_sector_bytes(
                vec![SectorInput {
                    key: SharedMapSectorKey {
                        map_id: 514,
                        sector_x: 0x7fff,
                        sector_y: 0x8000,
                    },
                    bytes: vec![0; 49170],
                }],
                &cancel,
            )
            .unwrap(),
        );
        let mut combat = super::super::world_runtime::tests::combat_frame();
        combat.world.manifest.digest = map.canonical_digest();
        combat.world.manifest.sectors = map
            .bindings()
            .into_iter()
            .map(|b| world::Sector {
                x: b.key.sector_x,
                y: b.key.sector_y,
                byte_length: b.byte_length,
                sha256: b.sha256,
            })
            .collect();
        let mut route = WorldNavigation::default();
        route
            .request(
                map,
                Arc::new(combat.world.clone()),
                1,
                WorldCell { x: 32724, y: 32785 },
            )
            .unwrap();
        assert!(!route.is_quiescent());
        assert!(!combat_alive(true, None)); // Unknown state blocks submitting.
        if combat_dead(None) {
            route.cancel();
        }
        assert!(!route.is_quiescent()); // But does NOT discard pending intent.
        let receipt = Instant::now();
        let vital = &combat.vitals[0];
        assert!(!action_cadence_ready(
            Some(receipt),
            200_000_000,
            None,
            Some(vital),
            receipt + Duration::from_millis(199)
        ));
        assert!(!action_cadence_ready(
            None,
            200_000_000,
            Some(receipt),
            Some(vital),
            receipt + Duration::from_millis(999)
        ));
        assert!(action_cadence_ready(
            Some(receipt),
            200_000_000,
            Some(receipt),
            Some(vital),
            receipt + Duration::from_secs(1)
        ));
        assert!(combat_alive(true, Some(vital)));
        assert!(!combat_dead(Some(vital)));
        assert!(!route.is_quiescent());
        let mut dead = vital.clone();
        dead.hp = 0;
        dead.death_revision = 1;
        if combat_dead(Some(&dead)) {
            route.cancel();
        }
        assert!(route.is_quiescent());
    }

    #[test]
    fn honor_respawn_notice_requires_an_active_server_match_and_a_dead_character() {
        for phase in [
            world::Phase::Preparing,
            world::Phase::Loading,
            world::Phase::Countdown,
            world::Phase::Active,
            world::Phase::Aborted,
            world::Phase::Finished,
        ] {
            assert_eq!(
                !honor_respawn_notice(phase, true).is_empty(),
                phase == world::Phase::Active
            );
            assert!(honor_respawn_notice(phase, false).is_empty());
        }
    }

    #[test]
    fn match_descriptor_controls_ui_readiness_without_mutating_connection_capability() {
        let mut view = snapshot();
        view.combat_supported = true;
        assert!(!view.world_combat_enabled());
        let mut reply = super::super::combat_protocol::tests::active_reply()
            .world_reply()
            .unwrap();
        reply.frame.as_mut().unwrap().descriptor =
            Some(super::super::world_runtime::tests::match_descriptor(false));
        reply.validate().unwrap();
        view.world = world::State::Ready(Arc::new(reply.clone()));
        assert!(!view.world_combat_enabled());
        assert!(view.combat_supported);
        assert!(
            combat_alive(view.world_combat_enabled(), None),
            "explicit scene-only can move without fake vitals"
        );
        reply.frame.as_mut().unwrap().descriptor =
            Some(super::super::world_runtime::tests::match_descriptor(true));
        reply.validate().unwrap();
        view.world = world::State::Ready(Arc::new(reply.clone()));
        assert!(view.world_combat_enabled());
        assert!(
            !combat_alive(view.world_combat_enabled(), None),
            "active combat cannot downgrade when the frame is late"
        );
        view.combat_supported = false;
        assert!(
            !view.world_combat_enabled(),
            "a descriptor is not a connection capability grant"
        );
        reply.frame.as_mut().unwrap().descriptor = None;
        view.world = world::State::Ready(Arc::new(reply));
        assert!(!view.world_combat_enabled());
        view.combat_supported = true;
        assert!(
            view.world_combat_enabled(),
            "legacy uses the connection contract"
        );
    }

    #[test]
    fn canonical_shortcuts_cannot_fallback_to_unstamped_legacy_inputs() {
        use super::super::{battle_player, world_window::Action as A};
        let mut view = snapshot();
        for a in [
            A::Step(3),
            A::Teleport,
            A::FireStorm,
            A::UseItem(item::UI_ITEMS[0]),
            A::SelectAmmo(super::super::ammo_protocol::UI_ITEMS[0]),
        ] {
            view.battle_supported = true;
            assert!(battle_player::admit_local_action(a, &view).is_err());
            view.battle_supported = false;
            assert_eq!(battle_player::admit_local_action(a, &view).unwrap(), a);
        }
        view.battle_supported = true;
        for a in [A::Leave, A::CancelRoute] {
            assert_eq!(battle_player::admit_local_action(a, &view).unwrap(), a);
        }
    }

    fn snapshot() -> LoginSnapshot {
        LoginSnapshot {
            phase: LoginPhase::CharactersReady,
            status: "角色資料已載入".into(),
            sources: vec![SourceChoice {
                id: 3,
                label: "太陽神".into(),
            }],
            selected_source: Some(3),
            characters: vec![CharacterRow {
                source_id: 3,
                object_id: 3333,
                label: "3333(太陽神)".into(),
            }],
            roster_epoch: 9,
            navigation: None,
            character_data: CharacterDataState::NotRequested,
            equipment: equipment::State::NotRequested,
            equipment_pending: false,
            rooms: rooms::State::NotRequested,
            room_pending: false,
            matchmaking: matching::State::default(),
            gm_supported: false,
            world: world::State::NotRequested,
            battle: crate::modern_lito::battle_session::State::default(),
            canonical: crate::modern_lito::battle_control::State::default(),
            battle_supported: false,
            action_supported: false,
            world_pending: false,
            combat: None,
            npc: None,
            combat_pending: false,
            combat_supported: false,
            skills: crate::modern_lito::spell_protocol::SkillsState::NotRequested,
            spell_pending: false,
            record: record::State::NotRequested,
            record_pending: false,
            chat: crate::modern_lito::chat_state::State::default(),
            chat_pending: false,
            direct_chat: crate::modern_lito::direct_chat_state::State::default(),
            direct_chat_pending: false,
            shop: shop::State::NotRequested,
            shop_pending: false,
            item_pending: false,
            ammo_pending: false,
        }
    }

    fn channel_snapshot() -> LoginSnapshot {
        use super::super::application_protocol::{
            ChannelSummary, CharacterSummary, NavigationAction, NavigationSnapshot,
            NavigationStatus,
        };
        let mut snapshot = snapshot();
        let context = character_context(&snapshot, 0).unwrap();
        snapshot.phase = LoginPhase::ChoosingChannel;
        snapshot.characters.clear();
        snapshot.roster_epoch = 0;
        snapshot.navigation = Some(NavigationReply {
            action: NavigationAction::SelectCharacter,
            status: NavigationStatus::Ok,
            context,
            snapshot: NavigationSnapshot {
                phase: NavigationPhase::Channels,
                channel_id: None,
                selected: Some(CharacterSummary {
                    key: context.key,
                    name: "勇者".into(),
                    class: 5,
                    sex: 1,
                    level: 50,
                }),
                channels: (0..4)
                    .map(|id| ChannelSummary {
                        id,
                        name: format!("伺服器提供的頻道{id}"),
                        population: 2,
                        capacity: 20,
                        minimum_level: 1,
                        maximum_level: 100,
                    })
                    .collect(),
            },
        });
        snapshot
    }

    fn npc70_ui_snapshot() -> LoginSnapshot {
        use super::super::{npc_fire_storm_protocol as fire, spell_protocol as spell};
        let r = fire::tests::snapshot("normal", "before");
        let f = r.frame.as_ref().unwrap();
        let mut v = channel_snapshot();
        v.phase = LoginPhase::InLobby;
        v.selected_source = Some(r.echo.context.key.source_id);
        let nav = v.navigation.as_mut().unwrap();
        nav.context = r.echo.context;
        nav.snapshot.channel_id = Some(r.echo.channel_id);
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.selected.as_mut().unwrap().key = r.echo.context.key;
        v.world = world::State::Ready(Arc::new(r.combat_reply().world_reply().unwrap()));
        v.combat = Some(Arc::new(f.combat.clone()));
        v.npc = Some(Arc::new(r.clone()));
        v.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: r.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 70,
                support: spell::Support::FireStorm,
                mp_cost: 48,
                reuse_millis: 2000,
                name: "npc70".into(),
            }],
        }));
        v
    }

    #[test]
    fn npc70_input_keeps_selected_source_epoch_channel_and_atomic_snapshot() {
        use super::super::{npc_fire_storm_protocol as fire, spell_protocol as spell};
        let v = npc70_ui_snapshot();
        let f = v.npc.as_ref().unwrap().frame.as_ref().unwrap();
        assert!(can_cast_skill(&v, 70));
        assert!(
            fire_storm_available(&v, f.combat.world.actors[0].entity_id),
            "a qualified NPC70 cast must have a visible normal-world button"
        );
        let request = npc_fire_storm_input_request(&v).unwrap();
        assert!(request.ready_in(f));
        let (expected, _, _) = fire::tests::sample("normal", "hit");
        assert_eq!(request, expected);
        for kind in 0..8 {
            let mut bad = v.clone();
            match kind {
                0 => bad.npc = None,
                1 => bad.spell_pending = true,
                2 => bad.skills = spell::SkillsState::NotRequested,
                3 => Arc::make_mut(bad.combat.as_mut().unwrap()).world.revision += 1,
                4 => bad.navigation.as_mut().unwrap().context.epoch += 1,
                5 => bad.navigation.as_mut().unwrap().snapshot.channel_id = Some(3),
                6 => Arc::make_mut(bad.npc.as_mut().unwrap()).echo.context.epoch += 1,
                _ => bad.world = world::State::NotRequested,
            }
            assert!(!can_cast_skill(&bad, 70), "case {kind}");
            assert_eq!(
                fire_storm_available(&bad, f.combat.world.actors[0].entity_id),
                kind == 1,
                "only pending keeps the qualified button visible: {kind}"
            );
        }
    }

    #[test]
    fn npc70_button_stays_visible_but_unusable_during_every_singleflight_wait() {
        let original = npc70_ui_snapshot();
        let entity = original.combat.as_ref().unwrap().world.actors[0].entity_id;
        for gate in 0..11 {
            let mut pending = original.clone();
            match gate {
                0 => pending.world_pending = true,
                1 => pending.spell_pending = true,
                2 => pending.combat_pending = true,
                3 => pending.item_pending = true,
                4 => pending.ammo_pending = true,
                5 => pending.room_pending = true,
                6 => pending.equipment_pending = true,
                7 => pending.shop_pending = true,
                8 => pending.record_pending = true,
                9 => pending.chat_pending = true,
                _ => pending.direct_chat_pending = true,
            }
            assert!(fire_storm_available(&pending, entity), "gate {gate}");
            assert!(!can_cast_skill(&pending, 70), "gate {gate}");
        }
        assert!(
            can_cast_skill(&original, 70),
            "retiring a pending flag can restore input without rebuilding the catalog"
        );
    }

    #[test]
    fn npc70_button_and_skill_inspection_share_qualified_match_support() {
        use super::super::spell_protocol::{SkillsState, Support};
        let original = npc70_ui_snapshot();
        let entity = original.combat.as_ref().unwrap().world.actors[0].entity_id;
        assert!(fire_storm_available(&original, entity));
        assert!(!fire_storm_available(&original, entity + 1));
        assert!(!fire_storm_available(&original, u64::MAX));
        let text =
            test_world_details_text(&original, super::super::world_window::DetailsPage::Skills)
                .unwrap();
        assert!(text.contains("本場規則開放，施放條件仍需確認"));
        for mutation in 0..7 {
            let mut bad = original.clone();
            match mutation {
                0 => bad.navigation = None,
                1 => bad.combat = None,
                2 | 3 => {
                    let SkillsState::Ready(skills) = &mut bad.skills else {
                        unreachable!()
                    };
                    let skills = Arc::make_mut(skills);
                    if mutation == 2 {
                        skills.context.epoch += 1;
                    } else {
                        skills.skills[0].support = Support::Unimplemented;
                    }
                }
                4 => {
                    Arc::make_mut(bad.npc.as_mut().unwrap())
                        .frame
                        .as_mut()
                        .unwrap()
                        .combat
                        .npc_fires
                        .clear();
                }
                5 => bad.selected_source = None,
                _ => bad.navigation.as_mut().unwrap().snapshot.selected = None,
            }
            assert!(!fire_storm_available(&bad, entity), "mutation {mutation}");
            assert!(!can_cast_skill(&bad, 70), "mutation {mutation}");
        }
    }
    #[test]
    fn npc77_input_keeps_selected_source_scope_lives_and_atomic_snapshot() {
        use super::super::{
            npc_protocol as npc, spell_protocol as spell, world_window::HitSurface,
        };
        let r = super::super::disintegrate_presentation::npc_tests::reply("normal-before");
        let f = r.frame.as_ref().unwrap();
        let selected = &f.combat.world.actors[0];
        let mut v = channel_snapshot();
        v.phase = LoginPhase::InLobby;
        let nav = v.navigation.as_mut().unwrap();
        nav.context = r.echo.context;
        nav.snapshot.channel_id = Some(r.echo.channel_id);
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.selected.as_mut().unwrap().key = r.echo.context.key;
        v.world = world::State::Ready(Arc::new(r.combat_reply().world_reply().unwrap()));
        v.combat = Some(Arc::new(f.combat.clone()));
        v.npc = Some(Arc::new(r.clone()));
        v.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context: r.echo.context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 77,
                support: spell::Support::Disintegrate,
                mp_cost: 70,
                reuse_millis: 5000,
                name: "npc77".into(),
            }],
        }));
        let s = HitSurface {
            match_id: f.scope.match_id,
            generation: f.scope.generation,
            revision: f.combat.world.revision,
            selected_entity_id: selected.entity_id,
            projection: super::super::world_map::ViewProjection {
                anchor_cell: selected.position,
                anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
            },
        };
        let target = npc::Id {
            scope: f.scope,
            serial: f.actors[0].serial,
        };
        assert!(can_cast_skill(&v, 77));
        let request = npc_disintegrate_input_request(&v, s, target, 1, 1).unwrap();
        assert!(request.ready_in(f));
        for (caster, life) in [(2, 1), (1, 2)] {
            assert!(npc_disintegrate_input_request(&v, s, target, caster, life).is_none());
        }
        for surface in [
            HitSurface {
                generation: s.generation + 1,
                ..s
            },
            HitSurface {
                revision: s.revision + 1,
                ..s
            },
            HitSurface {
                selected_entity_id: 2,
                ..s
            },
        ] {
            assert!(npc_disintegrate_input_request(&v, surface, target, 1, 1).is_none());
        }
        for kind in 0..5 {
            let mut bad = v.clone();
            match kind {
                0 => bad.npc = None,
                1 => bad.spell_pending = true,
                2 => bad.skills = spell::SkillsState::NotRequested,
                3 => Arc::make_mut(bad.combat.as_mut().unwrap()).world.revision += 1,
                _ => bad.navigation.as_mut().unwrap().context.epoch += 1,
            }
            assert!(!can_cast_skill(&bad, 77));
        }
    }
    #[test]
    fn disintegrate_input_rechecks_selection_and_painted_life_without_retargeting_respawn() {
        use super::super::{
            disintegrate_cast_protocol as dis, spell_protocol as spell, world_window::HitSurface,
        };
        let make = |name| {
            let r = super::super::disintegrate_presentation::tests::reply(name);
            let mut v = channel_snapshot();
            v.phase = LoginPhase::InLobby;
            let nav = v.navigation.as_mut().unwrap();
            nav.context = r.echo.context;
            nav.snapshot.selected.as_mut().unwrap().key = r.echo.context.key;
            nav.snapshot.channel_id = Some(r.echo.channel_id);
            nav.snapshot.phase = NavigationPhase::Lobby;
            v.world = world::State::Ready(Arc::new(r.world_reply().unwrap()));
            v.combat = Some(Arc::new(r.frame.unwrap()));
            v.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
                context: nav.context,
                status: spell::Status::Ok,
                skills: vec![spell::Skill {
                    skill_id: 77,
                    support: spell::Support::Disintegrate,
                    mp_cost: 70,
                    reuse_millis: 5000,
                    name: "dis".into(),
                }],
            }));
            v
        };
        let v = make("honor-active");
        let f = v.combat.as_ref().unwrap();
        let selected = f
            .world
            .actors
            .iter()
            .find(|a| a.matches_character(v.navigation.as_ref().unwrap().context.key))
            .unwrap();
        let target = f
            .world
            .actors
            .iter()
            .find(|a| a.entity_id != selected.entity_id)
            .unwrap()
            .entity_id;
        let s = HitSurface {
            match_id: f.world.match_id,
            generation: f.world.generation,
            revision: f.world.revision,
            selected_entity_id: selected.entity_id,
            projection: super::super::world_map::ViewProjection {
                anchor_cell: selected.position,
                anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
            },
        };
        assert!(can_cast_skill(&v, 77));
        let request = disintegrate_input_request(&v, f, s, target, 1, 1).unwrap();
        assert!(request.ready_in(f));
        let reborn = make("honor-respawn");
        let current = reborn.combat.as_ref().unwrap();
        assert_eq!(dis::life(current, target), Some(2));
        assert!(disintegrate_input_request(&reborn, current, s, target, 1, 1).is_none());
        let fresh = HitSurface {
            revision: current.world.revision,
            ..s
        };
        assert!(disintegrate_input_request(&reborn, current, fresh, target, 1, 2).is_some());
        for surface in [
            HitSurface {
                generation: s.generation + 1,
                ..s
            },
            HitSurface {
                match_id: s.match_id + 1,
                ..s
            },
            HitSurface {
                revision: current.world.revision + 1,
                ..s
            },
            HitSurface {
                selected_entity_id: target,
                ..s
            },
        ] {
            assert!(disintegrate_input_request(&reborn, current, surface, target, 1, 2).is_none());
        }
        for mutation in 0..4 {
            let mut bad = v.clone();
            match mutation {
                0 => bad.skills = spell::SkillsState::NotRequested,
                1 => bad.spell_pending = true,
                2 => {
                    let spell::SkillsState::Ready(r) = &mut bad.skills else {
                        unreachable!()
                    };
                    Arc::make_mut(r).context.epoch -= 1;
                }
                _ => bad.world = world::State::NotRequested,
            }
            assert!(!can_cast_skill(&bad, 77));
        }
    }

    fn seed_old_world_intent(state: &WorldUiScopeState) {
        state.min_hit_revision.set(90);
        state.failure.set(Some("old world failed"));
        state.leave_queued.set(true);
        state.leave_inflight.set(true);
    }
    fn assert_world_intent_retired(state: &WorldUiScopeState) {
        assert_eq!(state.min_hit_revision.get(), 0);
        assert_eq!(state.failure.get(), None);
        assert!(!state.leave_queued.get());
        assert!(!state.leave_inflight.get());
    }
    #[test]
    fn coalesced_world_rotation_retires_old_leave_failure_and_hit_revision_without_none() {
        let initial = super::super::item_protocol::tests::initial();
        let mut view = snapshot();
        view.phase = LoginPhase::InLobby;
        let state = WorldUiScopeState::default();
        let mut prior = None;
        // No intermediate world=None is delivered to this UI mailbox.
        for (index, map_id) in [509, 514, 509].into_iter().enumerate() {
            let mut reply = initial.world_reply().unwrap();
            let frame = reply.frame.as_mut().unwrap();
            frame.match_id = 100 + index as u64;
            frame.generation = 200 + index as u64;
            frame.manifest.map_id = map_id;
            view.world = world::State::Ready(Arc::new(reply));
            seed_old_world_intent(&state);
            let scope = world_ui_scope(&view).unwrap();
            assert_ne!(Some(scope), prior);
            let changed = state.replace(Some(scope));
            assert!(changed);
            assert_world_intent_retired(&state);
            let surface = HitSurface {
                match_id: scope.match_id.saturating_sub(1),
                generation: scope.generation.saturating_sub(1),
                revision: 90,
                selected_entity_id: 1,
                projection: super::super::world_map::ViewProjection {
                    anchor_cell: super::super::world_map::WorldCell { x: 32724, y: 32784 },
                    anchor_pixel: super::super::world_map::PixelPoint { x: 400, y: 300 },
                },
            };
            for action in [
                WorldAction::Leave,
                WorldAction::Step(3),
                WorldAction::Teleport,
                WorldAction::UseItem(40010),
                WorldAction::SelectAmmo(40744),
                WorldAction::Navigate {
                    surface,
                    goal: surface.projection.anchor_cell,
                },
                WorldAction::Attack {
                    surface,
                    target_entity_id: 2,
                },
                WorldAction::CastLightArrow {
                    surface,
                    target_entity_id: 2,
                },
            ] {
                assert_eq!(scoped_world_action(changed, Some(action)), None);
            }
            assert!(!state.replace(Some(scope)));
            assert_eq!(
                scoped_world_action(false, Some(WorldAction::Leave)),
                Some(WorldAction::Leave)
            );
            prior = Some(scope);
        }
    }
    #[test]
    fn same_world_receipts_do_not_clear_pending_leave_or_failure() {
        let reply = super::super::item_protocol::tests::initial()
            .world_reply()
            .unwrap();
        let mut view = snapshot();
        view.world = world::State::Ready(Arc::new(reply));
        let state = WorldUiScopeState::default();
        let scope = world_ui_scope(&view);
        assert!(state.replace(scope));
        seed_old_world_intent(&state);
        let world::State::Ready(reply) = &mut view.world else {
            unreachable!()
        };
        Arc::make_mut(reply).frame.as_mut().unwrap().revision += 1;
        assert!(!state.replace(world_ui_scope(&view)));
        assert_eq!(state.min_hit_revision.get(), 90);
        assert_eq!(state.failure.get(), Some("old world failed"));
        assert!(state.leave_queued.get());
        assert!(state.leave_inflight.get());
        assert!(state.replace(None));
        assert_world_intent_retired(&state);
        assert!(!state.replace(None));
    }
    #[test]
    fn world_scope_is_bound_to_source_character_selection_epoch_channel_and_generation() {
        let reply = super::super::item_protocol::tests::initial()
            .world_reply()
            .unwrap();
        let mut view = snapshot();
        view.world = world::State::Ready(Arc::new(reply));
        let old = world_ui_scope(&view).unwrap();
        for field in 0..6 {
            let mut next = old;
            match field {
                0 => next.context.key.source_id += 1,
                1 => next.context.key.object_id += 1,
                2 => next.context.epoch += 1,
                3 => next.channel_id += 1,
                4 => next.match_id += 1,
                _ => next.generation += 1,
            }
            let state = WorldUiScopeState::default();
            assert!(state.replace(Some(old)));
            seed_old_world_intent(&state);
            assert!(state.replace(Some(next)), "scope component {field}");
            assert_world_intent_retired(&state);
        }
    }
    #[test]
    fn retiring_old_world_ui_never_acknowledges_successor_tls_pending_command() {
        let reply = super::super::item_protocol::tests::initial()
            .world_reply()
            .unwrap();
        let mut view = snapshot();
        view.phase = LoginPhase::InLobby;
        view.world = world::State::Ready(Arc::new(reply));
        let next = world_ui_scope(&view).unwrap();
        let state = WorldUiScopeState::default();
        assert!(state.replace(Some(WorldUiScope {
            generation: next.generation + 1,
            ..next
        })));
        seed_old_world_intent(&state);
        view.world_pending = true;
        let confirmed = view.clone();
        assert!(state.replace(Some(next)));
        assert_world_intent_retired(&state);
        assert_eq!(view, confirmed);
        assert!(world_request(&view, world::Action::Leave).is_none());
        assert!(world_request(&view, world::Action::Loaded).is_none());
        view.world_pending = false;
        let request = world_request(&view, world::Action::Leave).unwrap();
        assert_eq!(
            (request.match_id, request.generation),
            (next.match_id, next.generation)
        );
        assert!(
            !state.leave_queued.get(),
            "only a new explicit action may request Leave"
        );
    }
    #[test]
    fn timed_item_class_lookup_uses_global_roster_identity_and_rejects_hidden_actors() {
        use crate::modern_lito::timed_protocol::Kind;
        let mut reply = crate::modern_lito::npc_protocol::tests::isolated_guest();
        let frame = &mut reply.frame.as_mut().unwrap().combat.world;
        frame.room.members[0].class = 1;
        frame.room.members[1].class = 3;
        // Valid sparse world, but this does not enable timed supplies for NPC
        // matches: only the shared UI identity lookup is under test here.
        assert_eq!(frame.actors.len(), 1);
        assert_eq!(frame.actors[0].entity_id, 2);
        assert!(timed_item_class_eligible(frame, 2, Kind::Wisdom));
        assert!(!timed_item_class_eligible(frame, 2, Kind::ElfBrave));
        assert!(!timed_item_class_eligible(frame, 1, Kind::Haste));
        assert!(!timed_item_class_eligible(frame, 0, Kind::Haste));
        assert!(!timed_item_class_eligible(frame, u64::MAX, Kind::Haste));
        frame.room.members[0].class = 3;
        frame.room.members[1].class = 2;
        assert!(timed_item_class_eligible(frame, 2, Kind::ElfBrave));
        assert!(!timed_item_class_eligible(frame, 2, Kind::Wisdom));
        reply.validate().unwrap();
    }
    #[test]
    fn timed_item_buttons_keep_confirmed_class_and_stock_requirements() {
        let combat = crate::modern_lito::timed_protocol::tests::initial();
        let mut view = snapshot();
        view.phase = LoginPhase::InLobby;
        view.world = world::State::Ready(Arc::new(combat.world_reply().unwrap()));
        view.combat = combat.frame.clone().map(Arc::new);
        let now = Instant::now();
        assert!(item_input_ready(&view, 1, 40016, None, now));
        assert!(!item_input_ready(&view, 1, 40068, None, now));
        assert!(!item_input_ready(&view, 2, 40068, None, now));
        assert!(!item_input_ready(&view, 0, 40016, None, now));
        view.item_pending = true;
        assert!(!item_input_ready(&view, 1, 40016, None, now));
    }
    #[test]
    fn supply_buttons_require_atomic_live_world_real_stock_singleflight_and_shared_cooldown() {
        let combat = super::super::item_protocol::tests::initial();
        let mut view = snapshot();
        view.phase = LoginPhase::InLobby;
        view.world = world::State::Ready(Arc::new(combat.world_reply().unwrap()));
        view.combat = combat.frame.clone().map(Arc::new);
        let now = Instant::now();
        assert!(item_input_ready(&view, 1, 40010, None, now));
        assert!(!item_input_ready(&view, 1, 40012, None, now));
        assert!(!item_input_ready(&view, 2, 40010, None, now));
        let f = view.combat.as_ref().unwrap();
        let cooldown = Some((f.world.match_id, f.world.generation, 1, 2, now, 500));
        assert!(!item_input_ready(
            &view,
            1,
            40011,
            cooldown,
            now + Duration::from_millis(499)
        ));
        assert!(item_input_ready(
            &view,
            1,
            40011,
            cooldown,
            now + Duration::from_millis(500)
        ));
        for which in 0..8 {
            let mut pending = view.clone();
            match which {
                0 => pending.item_pending = true,
                1 => pending.shop_pending = true,
                2 => pending.world_pending = true,
                3 => pending.combat_pending = true,
                4 => pending.room_pending = true,
                5 => pending.equipment_pending = true,
                6 => pending.spell_pending = true,
                _ => pending.record_pending = true,
            }
            assert!(
                !item_input_ready(&pending, 1, 40010, None, now),
                "pending {which}"
            );
        }
        view.combat = None;
        assert!(!item_input_ready(&view, 1, 40010, None, now));
    }
    #[test]
    fn arrows_are_selection_only_and_require_atomic_live_stock_without_bypassing_singleflight() {
        let combat = crate::modern_lito::ammo_protocol::tests::initial();
        let mut view = snapshot();
        view.phase = LoginPhase::InLobby;
        view.world = world::State::Ready(Arc::new(combat.world_reply().unwrap()));
        view.combat = combat.frame.clone().map(Arc::new);
        assert!(ammo_input_ready(&view, 1, 40744));
        assert!(ammo_input_ready(&view, 1, 40746));
        assert!(!ammo_input_ready(&view, 1, 40010));
        assert!(!ammo_input_ready(&view, 2, 40744));
        for which in 0..9 {
            let mut v = view.clone();
            match which {
                0 => v.ammo_pending = true,
                1 => v.item_pending = true,
                2 => v.shop_pending = true,
                3 => v.world_pending = true,
                4 => v.combat_pending = true,
                5 => v.equipment_pending = true,
                6 => v.room_pending = true,
                7 => v.spell_pending = true,
                _ => v.record_pending = true,
            }
            assert!(!ammo_input_ready(&v, 1, 40744));
        }
        let mut mismatch = view.clone();
        Arc::make_mut(mismatch.combat.as_mut().unwrap())
            .world
            .revision += 1;
        assert!(!ammo_input_ready(&mismatch, 1, 40744));
        let mut unarmed = view.clone();
        Arc::make_mut(unarmed.combat.as_mut().unwrap()).vitals[0].weapon_type = 0;
        assert!(
            ammo_input_ready(&unarmed, 1, 40744),
            "selection does not fake an equipped bow"
        );
        view.combat = None;
        assert!(!ammo_input_ready(&view, 1, 40744));
    }

    #[test]
    fn fire_ui_requires_learned_full_current_world_and_all_singleflight_gates() {
        use super::super::spell_protocol as spell;
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let mut reply = super::super::fire_storm_protocol::tests::initial();
        reply.echo.context = context;
        let f = reply.frame.as_mut().unwrap();
        f.world.actors[0].key = context.key;
        f.world.room.members[0].key = context.key;
        f.world.room.owner = context.key;
        view.phase = LoginPhase::InLobby;
        view.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Lobby;
        view.navigation.as_mut().unwrap().snapshot.channel_id = Some(reply.echo.channel_id);
        view.combat = Some(Arc::new(f.clone()));
        view.world = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        assert!(!can_cast_skill(&view, spell::FIRE_STORM));
        assert!(!fire_storm_available(&view, 1));
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 70,
                support: spell::Support::FireStorm,
                mp_cost: 48,
                reuse_millis: 2000,
                name: "火風暴".into(),
            }],
        }));
        assert!(can_cast_skill(&view, spell::FIRE_STORM));
        assert!(fire_storm_available(&view, 1));
        assert!(!fire_storm_available(&view, 2));
        for case in 0..12 {
            let mut bad = view.clone();
            match case {
                0 => bad.world_pending = true,
                1 => bad.spell_pending = true,
                2 => bad.combat_pending = true,
                3 => bad.item_pending = true,
                4 => bad.ammo_pending = true,
                5 => bad.room_pending = true,
                6 => bad.equipment_pending = true,
                7 => bad.shop_pending = true,
                8 => bad.record_pending = true,
                9 => Arc::make_mut(bad.combat.as_mut().unwrap()).fires.clear(),
                10 => Arc::make_mut(bad.combat.as_mut().unwrap()).vitals[0].hp = 0,
                _ => Arc::make_mut(bad.combat.as_mut().unwrap()).world.revision += 1,
            }
            assert!(!can_cast_skill(&bad, spell::FIRE_STORM), "gate {case}");
            assert_eq!(
                fire_storm_available(&bad, 1),
                case != 9 && case != 11,
                "support remains visible while pending or dead: {case}"
            );
        }
    }
    #[test]
    fn overweight_ui_keeps_heal_available_and_never_enables_cast_from_unconfirmed_weight() {
        use super::super::spell_protocol as spell;
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let mut reply = super::super::weight_protocol::tests::weighted_reply();
        reply.echo.context = context;
        let f = reply.frame.as_mut().unwrap();
        f.world.actors[0].key = context.key;
        f.world.room.members[0].key = context.key;
        f.world.room.owner = context.key;
        f.world.casts = f
            .world
            .actors
            .iter()
            .map(|a| spell::CastState::initial(a.entity_id, 1000))
            .collect();
        for v in &mut f.vitals {
            v.mp = 100;
            v.max_mp = 100;
        }
        reply.validate().unwrap();
        view.phase = LoginPhase::InLobby;
        view.combat_supported = true;
        view.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Lobby;
        view.navigation.as_mut().unwrap().snapshot.channel_id = Some(reply.echo.channel_id);
        view.skills = spell::SkillsState::Ready(Arc::new(spell::SkillsReply {
            context,
            status: spell::Status::Ok,
            skills: vec![spell::Skill {
                skill_id: 5,
                support: spell::Support::Teleport,
                mp_cost: 10,
                reuse_millis: 10,
                name: "瞬間移動".into(),
            }],
        }));
        view.world = world::State::Ready(Arc::new(reply.world_reply().unwrap()));
        view.combat = reply.frame.clone().map(Arc::new);
        let now = Instant::now();
        assert!(!can_cast_teleport(&view));
        assert!(item_input_ready(&view, 1, 40010, None, now));
        let text = view.combat.as_ref().unwrap().weight(1).unwrap().notice();
        assert!(text.contains("不含 ETC／金幣"));
        assert!(text.contains("仍可移動"));
        let mut used = super::super::item_protocol::tests::used(&reply);
        let w = &mut used.frame.as_mut().unwrap().weights[0].weight;
        w.supplies_weight -= 1;
        w.total -= 1;
        w.stage = 196;
        w.hit_penalty = 5;
        used.validate().unwrap();
        view.combat = used.frame.clone().map(Arc::new);
        assert!(
            !can_cast_teleport(&view),
            "new HP/weight must not mix with the previous world revision"
        );
        view.world = world::State::Ready(Arc::new(used.combat_reply().world_reply().unwrap()));
        assert!(can_cast_teleport(&view));
        view.item_pending = true;
        assert!(!can_cast_teleport(&view));
        assert!(!item_input_ready(&view, 1, 40010, None, now));
    }
    #[test]
    fn supply_pending_never_coexists_with_any_other_command_class() {
        for which in 0..7 {
            let mut view = channel_snapshot();
            view.shop_pending = true;
            match which {
                0 => view.item_pending = true,
                1 => view.equipment_pending = true,
                2 => view.room_pending = true,
                3 => view.world_pending = true,
                4 => view.combat_pending = true,
                5 => view.spell_pending = true,
                _ => view.record_pending = true,
            }
            assert!(validate_snapshot(&view).is_err(), "pending {which}");
        }
    }

    #[test]
    fn room_form_identity_is_cancelled_on_page_failure_epoch_or_channel_change() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let common = rooms::Common::read(nav.context, 0);
        assert!(form_context_current(common, &view));
        for phase in [
            LoginPhase::ChoosingChannel,
            LoginPhase::Failed,
            LoginPhase::Disconnecting,
            LoginPhase::CharactersReady,
        ] {
            let mut changed = view.clone();
            changed.phase = phase;
            assert!(!form_context_current(common, &changed));
        }
        let mut changed = view.clone();
        changed.navigation.as_mut().unwrap().context.epoch += 1;
        assert!(!form_context_current(common, &changed));
        changed = view.clone();
        changed.navigation.as_mut().unwrap().snapshot.channel_id = Some(1);
        assert!(!form_context_current(common, &changed));
    }
    #[test]
    fn legacy_peer_without_room_capability_keeps_navigation_and_hides_room_only_controls() {
        let mut view = channel_snapshot();
        assert!(source_selector_visible(&view));
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        view.rooms = rooms::State::Unsupported;
        assert!(!room_surface(&view));
        assert!(!source_selector_visible(&view));
        assert!(unsupported_room_hint(&view)
            .unwrap()
            .contains("尚未提供房間功能"));
        assert!(matches!(
            return_command(&view),
            Some(LoginCommand::LeaveLobby(_))
        ));
        assert!(room_common(&view, rooms::Action::Create, None, 0).is_none());
        let context = view.navigation.as_ref().unwrap().context;
        view.rooms = rooms::State::Ready(Arc::new(rooms::tests::sample(context, false)));
        assert!(room_surface(&view));
        assert!(!source_selector_visible(&view));
        assert!(unsupported_room_hint(&view).is_none());
        view.phase = LoginPhase::ChoosingChannel;
        assert!(!room_surface(&view));
        assert!(source_selector_visible(&view));
    }
    #[test]
    fn chat_snapshot_requires_current_audience_and_chat_pending_blocks_room_input() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        let room = rooms::tests::sample(context, true);
        view.rooms = rooms::State::Ready(Arc::new(room.clone()));
        let scope = super::super::chat_state::Scope::current(view.navigation.as_ref(), Some(&room));
        view.chat.align(true, scope);
        view.chat
            .receive(super::super::chat_protocol::Reply {
                echo: view.chat.read().unwrap(),
                status: super::super::chat_protocol::Status::Ok,
                membership: 1,
                sequence: 0,
                cursor: 0,
                available: true,
                truncated: false,
                messages: vec![],
            })
            .unwrap();
        validate_snapshot(&view).unwrap();
        assert!(room_common(&view, rooms::Action::Leave, None, 0).is_some());
        let token = view.chat.token().unwrap();
        let intent = |text: &str, kind| super::super::chat_window::Intent {
            token,
            text: text.into(),
            kind,
        };
        use super::super::chat_window::Kind;
        assert!(matches!(
            chat_command(&view, intent("你好", Kind::Message)),
            Ok(LoginCommand::ChatSend { .. })
        ));
        assert!(chat_command(&view, intent(".de 20", Kind::GmBots(20))).is_err());
        view.gm_supported = true;
        assert!(matches!(
            chat_command(&view, intent(".de 20", Kind::GmBots(20))),
            Ok(LoginCommand::GmBots { count: 20 })
        ));
        assert!(chat_command(&view, intent(".de 20", Kind::Message)).is_err());
        assert!(chat_command(&view, intent(".de 101", Kind::GmBots(101))).is_err());
        let mut stale = intent("stale", Kind::Message);
        stale.token.membership += 1;
        assert!(chat_command(&view, stale).is_err());
        view.chat_pending = true;
        assert!(chat_command(&view, intent("busy", Kind::Message)).is_err());
        validate_snapshot(&view).unwrap();
        assert!(room_common(&view, rooms::Action::Leave, None, 0).is_none());
        view.room_pending = true;
        assert!(validate_snapshot(&view).is_err());
        view.room_pending = false;
        view.chat_pending = false;
        view.navigation.as_mut().unwrap().context.epoch += 1;
        assert!(validate_snapshot(&view).is_err());
    }
    #[test]
    fn canonical_pending_world_uses_its_confirmed_scope_not_a_legacy_frame() {
        let mut v = channel_snapshot();
        v.phase = LoginPhase::InLobby;
        let nav = v.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        v.world_pending = true;
        assert!(validate_snapshot(&v).is_err());
        for canonical in [
            super::super::battle_control::tests::detached(context, 0),
            super::super::battle_control::tests::preparing(context, 0),
        ] {
            v.canonical = canonical;
            validate_snapshot(&v).unwrap();
            v.navigation.as_mut().unwrap().context.epoch += 1;
            assert!(validate_snapshot(&v).is_err());
            v.navigation.as_mut().unwrap().context = context;
            v.equipment_pending = true;
            assert!(validate_snapshot(&v).is_err());
            v.equipment_pending = false;
            v.canonical.hide();
            validate_snapshot(&v).unwrap();
            let nav = v.navigation.take();
            assert!(validate_snapshot(&v).is_err());
            v.navigation = nav;
        }
    }
    #[test]
    fn pending_lobby_return_keeps_confirmed_chat_read_only_without_fatal_ui_validation() {
        use super::super::{
            chat_protocol as chat, chat_state, direct_chat_protocol as dc, direct_chat_state as ds,
        };
        let mut v = channel_snapshot();
        v.phase = LoginPhase::InLobby;
        let nav = v.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        let room = rooms::tests::sample(context, false);
        v.chat.align(
            true,
            chat_state::Scope::current(v.navigation.as_ref(), Some(&room)),
        );
        v.rooms = rooms::State::Ready(Arc::new(room));
        v.chat
            .receive(chat::Reply {
                echo: v.chat.read().unwrap(),
                status: chat::Status::Ok,
                membership: 1,
                sequence: 0,
                cursor: 0,
                available: true,
                truncated: false,
                messages: vec![],
            })
            .unwrap();
        let public = v.chat.token().unwrap();
        let enrollment = ds::Enrollment {
            context,
            channel_id: 0,
            generation: 1,
        };
        let token = ds::Token {
            enrollment,
            room_id: 0,
            membership: 1,
        };
        v.direct_chat
            .bind(true, Some(enrollment), Some(token))
            .unwrap();
        v.direct_chat
            .receive(
                token,
                dc::Reply {
                    echo: v.direct_chat.read().unwrap().1,
                    status: dc::Status::Ok,
                    sequence: 0,
                    cursor: 0,
                    available: true,
                    truncated: false,
                    owner: dc::Peer {
                        key: context.key,
                        character_name: "3333".into(),
                        source_name: "太陽神".into(),
                    },
                    preferences: dc::Preferences::default(),
                    blocked: vec![],
                    messages: vec![],
                },
            )
            .unwrap();
        validate_snapshot(&v).unwrap();
        for phase in [LoginPhase::LeavingLobby, LoginPhase::ReturningCharacters] {
            v.phase = phase;
            validate_snapshot(&v).unwrap();
            assert!(chat_view(&v, false).busy && direct_chat_view(&v, false).busy);
            assert!(chat_command(
                &v,
                super::super::chat_window::Intent {
                    token: public,
                    text: "不能送出".into(),
                    kind: super::super::chat_window::Kind::Message
                }
            )
            .is_err());
            assert!(direct_chat_command(
                &v,
                super::super::direct_chat_window::Intent {
                    token,
                    action: dc::Action::ClearBlocks,
                    target: None,
                    preferences: None,
                    text: String::new()
                }
            )
            .is_err());
            v.chat_pending = true;
            assert!(validate_snapshot(&v).is_err());
            v.chat_pending = false;
            v.direct_chat_pending = true;
            assert!(validate_snapshot(&v).is_err());
            v.direct_chat_pending = false;
            v.navigation.as_mut().unwrap().snapshot.channel_id = Some(1);
            assert!(validate_snapshot(&v).is_err());
            v.navigation.as_mut().unwrap().snapshot.channel_id = Some(0);
        }
        for phase in [
            LoginPhase::ChoosingChannel,
            LoginPhase::JoiningLobby,
            LoginPhase::CharactersReady,
            LoginPhase::Disconnected,
        ] {
            v.phase = phase;
            assert!(validate_snapshot(&v).is_err());
        }
    }
    #[test]
    fn room_controls_use_instance_revision_and_real_membership_and_never_offer_start() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        let room = rooms::tests::sample(context, true);
        view.rooms = rooms::State::Ready(Arc::new(room));
        assert!(return_command(&view).is_none());
        assert!(room_common(&view, rooms::Action::Create, None, 0).is_none());
        let leave = room_common(&view, rooms::Action::Leave, None, 0).unwrap();
        assert_eq!(leave.context, context);
        assert_eq!(leave.expected_revision, 1);
        assert_eq!(leave.room_id, 5_000_000_001);
        assert!(room_common(&view, rooms::Action::Configure, None, 0).is_some());
        view.room_pending = true;
        assert!(room_common(&view, rooms::Action::Leave, None, 0).is_none());
        assert!(equipment_command(&view, None, None).is_none());
    }

    #[test]
    fn unbound_rooms_cannot_be_created_joined_or_readied_but_keep_escape_controls() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let mut directory = rooms::tests::sample(nav.context, true);
        let joined = directory.joined.take().unwrap();
        let id = joined.summary.room_id;
        for option in &mut directory.options {
            option.world_available = false;
        }
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert!(room_common(&view, rooms::Action::Create, None, 0).is_none());
        assert!(room_common(&view, rooms::Action::Join, Some(id), 0).is_none());
        assert!(room_common(&view, rooms::Action::Read, None, 0).is_some());
        assert!(matches!(
            return_command(&view),
            Some(LoginCommand::LeaveLobby(_))
        ));
        directory.joined = Some(joined);
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert!(room_common(&view, rooms::Action::Ready, None, 1).is_none());
        for action in [
            rooms::Action::Read,
            rooms::Action::Configure,
            rooms::Action::Leave,
            rooms::Action::Ready,
            rooms::Action::Team,
        ] {
            assert!(room_common(&view, action, None, 0).is_some(), "{action:?}");
        }
        let config = &directory.joined.as_ref().unwrap().summary.settings;
        let pair = (config.mode, config.map_id);
        directory
            .options
            .iter_mut()
            .find(|o| (o.mode, o.map_id) == pair)
            .unwrap()
            .world_available = true;
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert!(room_common(&view, rooms::Action::Ready, None, 1).is_some());
        directory.joined = None;
        view.rooms = rooms::State::Ready(Arc::new(directory));
        assert!(room_common(&view, rooms::Action::Create, None, 0).is_some());
        assert!(room_common(&view, rooms::Action::Join, Some(id), 0).is_some());
    }

    #[test]
    fn ready_button_checks_its_actual_toggle_and_unbound_map_remains_explainable() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let mut directory = rooms::tests::sample(nav.context, true);
        assert!(room_directory_label(&directory).contains("地圖未開放"));
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert_eq!(room_ready_value(&view), 1);
        assert!(room_common(&view, rooms::Action::Ready, None, room_ready_value(&view)).is_none());
        directory.joined.as_mut().unwrap().members[0].ready = true;
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        assert_eq!(room_ready_value(&view), 0);
        assert!(room_common(&view, rooms::Action::Ready, None, room_ready_value(&view)).is_some());
        directory.joined = None;
        assert!(room_directory_label(&directory).contains("已綁定地圖 0/40"));
        assert!(room_directory_label(&directory).contains("返回頻道"));
        directory.options[0].world_available = true;
        assert!(room_directory_label(&directory).contains("已綁定地圖 1/40"));
        assert!(!room_directory_label(&directory).contains("尚無可進場"));
        view.rooms = rooms::State::Ready(Arc::new(directory.clone()));
        // An unrelated enabled pair cannot authorize joining this unbound room.
        assert!(room_common(&view, rooms::Action::Create, None, 0).is_some());
        assert!(room_common(
            &view,
            rooms::Action::Join,
            Some(directory.rooms[0].room_id),
            0
        )
        .is_none());
    }

    #[test]
    fn canonical_attachment_and_hidden_pending_world_gate_cached_lobby_controls() {
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        view.rooms = rooms::State::Ready(Arc::new(rooms::tests::sample(context, true)));
        view.gm_supported = true;
        let check = |v: &LoginSnapshot, ready: bool| {
            for action in [
                rooms::Action::Read,
                rooms::Action::Configure,
                rooms::Action::Leave,
                rooms::Action::Ready,
            ] {
                assert_eq!(room_common(v, action, None, 0).is_some(), ready);
            }
            assert_eq!(gm_input_ready(v), ready);
            assert_eq!(can_open_shop(v), ready);
            assert_eq!(can_read_record(v), ready);
        };
        check(&view, true);
        view.canonical = super::super::battle_control::tests::preparing(context, 0);
        assert!(view.world.frame().is_none());
        check(&view, false);
        view.rooms = rooms::State::NotRequested;
        assert!(
            return_command(&view).is_none(),
            "absent cached room is not LEAVE ACK"
        );
        view.canonical = super::super::battle_control::State::default();
        view.world_pending = true;
        assert!(return_command(&view).is_none());
        view.rooms = rooms::State::Ready(Arc::new(rooms::tests::sample(context, true)));
        check(&view, false);
        view.world_pending = false;
        view.combat_pending = true;
        check(&view, false);
        view.combat_pending = false;
        view.canonical = super::super::battle_control::tests::detached(context, 0);
        check(&view, true);
    }

    #[test]
    fn canonical_inspection_never_falls_back_to_source_pages_or_old_tokens() {
        use super::super::world_window::DetailsPage;
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        let nav = view.navigation.as_mut().unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        let context = nav.context;
        assert_eq!(canonical_detail_page(&view, DetailView::Items), None);
        view.canonical = super::super::battle_control::tests::preparing(context, 0);
        assert!(view.world.frame().is_none());
        for (button, detail, page) in [
            (SHOW_STATS, DetailView::Stats, DetailsPage::Character),
            (SHOW_ITEMS, DetailView::Items, DetailsPage::Inventory),
            (SHOW_SKILLS, DetailView::Skills, DetailsPage::Skills),
        ] {
            assert_eq!(detail_mnemonic_target(page), (button, detail));
            assert_eq!(canonical_detail_page(&view, detail), Some(page));
            assert!(!can_show_player_detail(&view, detail, false));
            assert!(can_show_player_detail(&view, detail, true));
            let text = source_data_text(&view, detail, &DescriptionState::Loading).unwrap();
            assert!(text.contains("此處不顯示舊頁資料"));
        }
        assert_eq!(canonical_detail_page(&view, DetailView::Record), None);
        assert!(
            source_data_text(&view, DetailView::Record, &DescriptionState::Loading)
                .unwrap()
                .contains("中央累計戰績")
        );
        view.canonical = super::super::battle_control::tests::detached(context, 0);
        view.world_pending = true;
        assert_eq!(
            canonical_detail_page(&view, DetailView::Items),
            Some(DetailsPage::Inventory)
        );
        view.world_pending = false;
        assert_eq!(canonical_detail_page(&view, DetailView::Items), None);
        assert!(
            !source_data_text(&view, DetailView::Items, &DescriptionState::Loading)
                .unwrap()
                .contains("此處不顯示舊頁資料")
        );
    }

    pub(crate) fn vip_preview_view(
        observation: &super::super::battle_follow::Observation,
    ) -> LoginSnapshot {
        let player = &observation.data().personal.player;
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        view.selected_source = Some(u32::try_from(player.source).unwrap());
        view.sources[0].id = u32::try_from(player.source).unwrap();
        view.battle_supported = true;
        let nav = view.navigation.as_mut().unwrap();
        nav.context.key.source_id = u32::try_from(player.source).unwrap();
        nav.context.key.object_id = player.object as u64;
        nav.context.epoch = u64::try_from(player.epoch).unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(u8::try_from(player.channel).unwrap());
        nav.snapshot.selected.as_mut().unwrap().key = nav.context.key;
        let context = nav.context;
        view.canonical = super::super::battle_control::tests::vip_inspection(
            context, u8::try_from(player.channel).unwrap(), observation.clone());
        assert!(world_ui_scope(&view).is_some());
        view
    }

    pub(crate) fn assert_vip_read_only_character_route(
        observation: &super::super::battle_follow::Observation,
    ) {
        let player = &observation.data().personal.player;
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        view.selected_source = Some(u32::try_from(player.source).unwrap());
        view.sources[0].id = u32::try_from(player.source).unwrap();
        view.battle_supported = true;
        let nav = view.navigation.as_mut().unwrap();
        nav.context.key.source_id = u32::try_from(player.source).unwrap();
        nav.context.key.object_id = player.object as u64;
        nav.context.epoch = u64::try_from(player.epoch).unwrap();
        nav.snapshot.phase = NavigationPhase::Lobby;
        nav.snapshot.channel_id = Some(0);
        nav.snapshot.selected.as_mut().unwrap().key = nav.context.key;
        let context = nav.context;
        let before = view.clone();
        assert!(!before.world_owns_lobby());
        view.canonical = super::super::battle_control::tests::vip_inspection(
            context, 0, observation.clone());
        assert!(view.world_owns_lobby());
        assert_eq!(before.navigation, view.navigation);
        assert_eq!(before.character_data, view.character_data);
        assert_eq!(before.skills, view.skills);
        for detail in [DetailView::Items, DetailView::Skills] {
            assert!(detail_snapshot_changed(&before, &view, detail));
            assert!(detail_snapshot_changed(&view, &before, detail));
            assert_ne!(source_data_text(&before, detail, &DescriptionState::Loading),
                source_data_text(&view, detail, &DescriptionState::Loading));
        }
        assert!(!detail_snapshot_changed(&view, &view, DetailView::Stats));
        assert!(!detail_snapshot_changed(&view, &view, DetailView::Items));
        assert!(preserve_detail_reading(Some(&view), &view,
            DetailView::Stats, Some(DetailView::Stats)));
        assert!(preserve_detail_reading(Some(&view), &view,
            DetailView::Items, Some(DetailView::Items)));
        assert!(!preserve_detail_reading(Some(&before), &view,
            DetailView::Stats, Some(DetailView::Stats)));
        assert!(!preserve_detail_reading(Some(&view), &before,
            DetailView::Stats, Some(DetailView::Stats)));
        for detail in [DetailView::Stats, DetailView::Items, DetailView::Skills] {
            assert!(canonical_detail_page(&view, detail).is_none());
        }
        assert!(can_show_player_detail(&view, DetailView::Stats, false));
        assert!(can_show_player_detail(&view, DetailView::Items, false));
        assert!(can_show_player_detail(&view, DetailView::Skills, false));
        let text = source_data_text(&view, DetailView::Stats, &DescriptionState::Loading).unwrap();
        assert!(text.contains("完整 VIP 本場權威數值（唯讀）"));
        assert!(text.contains(&format!("能力版本 {}",
            observation.data().vip.as_ref().unwrap().owner.versions.runtime)));
        assert!(!text.contains("此處不顯示舊頁資料"));
        let items = source_data_text(&view, DetailView::Items, &DescriptionState::Loading).unwrap();
        assert!(items.contains("完整 VIP 本場來源背包（唯讀）"));
        assert!(items.contains(&format!("能力版本 {}",
            observation.data().vip.as_ref().unwrap().owner.versions.runtime)));
        for item in &observation.data().personal.equipment.as_ref().unwrap().inventory.0 {
            assert!(items.contains(&format!("實例 {}／目錄 {}", item.object, item.catalog)));
        }
        let inventory = super::super::vip_inventory_preview::Model::from_observation(
            context, 0, observation, |token| format!("驗證:{token}"))
            .expect("current owner must expose validated source inventory");
        let source_items = &observation.data().personal.equipment.as_ref().unwrap().inventory.0;
        assert_eq!(inventory.rows.len(), source_items.len());
        for (row, item) in inventory.rows.iter().zip(source_items) {
            assert_eq!(row.object, item.object);
            assert_eq!(row.graphic, item.inventory_graphic);
            assert_eq!(row.quantity, item.quantity);
            assert_eq!(row.equipped, observation.personal().equipped(item.object));
            assert_eq!(row.label, format!("驗證:{}", item.name.0));
        }
        assert!(inventory.key.graphics.windows(2).all(|pair| pair[0] < pair[1]));
        let mut wrong_context = context;
        wrong_context.key.object_id += 1;
        assert!(super::super::vip_inventory_preview::Model::from_observation(
            wrong_context, 0, observation, |_| "錯誤來源".into()).is_none());
        let mut stale_epoch = context;
        stale_epoch.epoch += 1;
        assert!(super::super::vip_inventory_preview::Model::from_observation(
            stale_epoch, 0, observation, |_| "舊版角色".into()).is_none());
        assert!(super::super::vip_inventory_preview::Model::from_observation(
            context, 1, observation, |_| "錯誤頻道".into()).is_none());
        for item in &observation.data().vip.as_ref().unwrap().owner.items.0 {
            assert!(items.contains(&format!("已套用 VIP：模板 {}", item.applied.catalog)));
        }
        assert!(!items.contains("此頁不退回來源保存資料"));
        let skills = source_data_text(&view, DetailView::Skills, &DescriptionState::Loading).unwrap();
        assert!(skills.contains("完整 VIP 本場已學技能（唯讀）"));
        assert!(skills.contains(&format!("本場確認已學 {} 項", observation.personal().skills().len())));
        for skill in observation.personal().skills() {
            assert!(skills.contains(&format!("#{} 名稱載入中", skill.id)));
            assert!(skills.contains(&format!("MP {} · 來源重用間隔 {}ms",
                skill.mp_cost, skill.reuse_millis)));
        }
        assert!(!skills.contains("此頁不退回來源保存資料"));
        assert!(source_data_text(&view, DetailView::Record, &DescriptionState::Loading)
            .unwrap().contains("中央累計戰績"));
        let attached = view.clone();
        let expected_icons = super::super::vip_status::required(observation).unwrap();
        let status = vip_status_desired(&view);
        assert_eq!(status.as_ref().map(|(key, _)| key.icons.len()),
            (!expected_icons.is_empty()).then_some(expected_icons.len()));
        if let Some((key, _)) = status {
            assert_eq!(key.context, context);
            assert_eq!(key.channel, 0);
            assert_eq!(key.icons, expected_icons.iter()
                .map(|id| i64::from(id.value())).collect::<Vec<_>>());
        }
        view.selected_source = Some(u32::try_from(player.source).unwrap() + 1);
        assert!(!preserve_detail_reading(Some(&attached), &view,
            DetailView::Stats, Some(DetailView::Stats)));
        assert!(detail_snapshot_changed(&attached, &view, DetailView::Stats));
        assert!(!can_show_player_detail(&view, DetailView::Stats, false));
        assert!(!can_show_player_detail(&view, DetailView::Items, false));
        assert!(!can_show_player_detail(&view, DetailView::Skills, false));
        assert!(vip_status_desired(&view).is_none());
        assert!(vip_detail_observation(&view).is_none());
        assert!(source_data_text(&view, DetailView::Stats, &DescriptionState::Loading)
            .unwrap().contains("不顯示舊角色保存值"));
        assert!(source_data_text(&view, DetailView::Items, &DescriptionState::Loading)
            .unwrap().contains("不顯示舊角色保存背包"));
        assert!(source_data_text(&view, DetailView::Skills, &DescriptionState::Loading)
            .unwrap().contains("不顯示舊角色保存技能"));
        view.selected_source = Some(u32::try_from(player.source).unwrap());
        view.navigation.as_mut().unwrap().snapshot.channel_id = Some(1);
        assert!(!can_show_player_detail(&view, DetailView::Stats, false));
        assert!(!can_show_player_detail(&view, DetailView::Items, false));
        assert!(!can_show_player_detail(&view, DetailView::Skills, false));
        assert!(vip_status_desired(&view).is_none());
    }
    #[test]
    fn character_button_carries_displayed_source_object_and_epoch_not_a_list_index() {
        let mut snapshot = snapshot();
        let context = character_context(&snapshot, 0).unwrap();
        assert_eq!(
            (context.key.source_id, context.key.object_id, context.epoch),
            (3, 3333, 9)
        );
        assert!(character_context(&snapshot, -1).is_none());
        assert!(character_context(&snapshot, 1).is_none());
        snapshot.phase = LoginPhase::SelectingCharacter;
        assert!(
            character_context(&snapshot, 0).is_none(),
            "double click cannot submit another mutation"
        );
        snapshot.phase = LoginPhase::CharactersReady;
        snapshot.roster_epoch = 0;
        assert!(character_context(&snapshot, 0).is_none());
    }

    #[test]
    fn channel_button_uses_server_rows_without_turning_recommendations_or_cached_population_into_restrictions(
    ) {
        let mut snapshot = channel_snapshot();
        validate_snapshot(&snapshot).unwrap();
        let Some(LoginCommand::EnterChannel {
            context,
            channel_id,
        }) = channel_command(&snapshot, 2)
        else {
            panic!("valid server channel")
        };
        assert_eq!(context, snapshot.navigation.as_ref().unwrap().context);
        assert_eq!(channel_id, 2);
        assert!(channel_command(&snapshot, -1).is_none());
        assert!(channel_command(&snapshot, 4).is_none());
        snapshot.navigation.as_mut().unwrap().snapshot.channels[2].population = 20;
        assert!(
            channel_command(&snapshot, 2).is_some(),
            "full snapshot can be stale; server must decide"
        );
        snapshot.navigation.as_mut().unwrap().snapshot.channels[2].population = 2;
        snapshot.navigation.as_mut().unwrap().snapshot.channels[2].minimum_level = 51;
        assert!(
            channel_command(&snapshot, 2).is_some(),
            "recommended tier is not an entry restriction"
        );
        snapshot.navigation.as_mut().unwrap().snapshot.channels[2].minimum_level = 1;
        let reply = snapshot.navigation.as_mut().unwrap();
        reply.snapshot.selected.as_mut().unwrap().level = 80;
        reply.snapshot.channels[3].minimum_level = 15;
        reply.snapshot.channels[3].maximum_level = 39;
        assert!(channel_label(&reply.snapshot.channels[3]).contains("推薦等級 15–39"));
        assert!(
            matches!(
                channel_command(&snapshot, 3),
                Some(LoginCommand::EnterChannel { channel_id: 3, .. })
            ),
            "level 80 can request the fourth recommended tier"
        );
        for phase in [
            LoginPhase::JoiningLobby,
            LoginPhase::ReturningCharacters,
            LoginPhase::Disconnected,
        ] {
            snapshot.phase = phase;
            assert!(channel_command(&snapshot, 2).is_none());
            assert!(!enabled(phase, false).source);
        }
    }

    #[test]
    fn server_cast_interval_is_receipt_bound_and_does_not_restart_for_proven_later_actions() {
        use std::time::Duration;
        let now = Instant::now();
        let receipt = Some((now, 1200));
        assert!(!cast_interval_ready(
            receipt,
            now + Duration::from_millis(1199)
        ));
        assert!(cast_interval_ready(
            receipt,
            now + Duration::from_millis(1200)
        ));
        assert!(!cast_interval_ready(
            receipt,
            now - Duration::from_millis(1)
        ));
        let initial = super::super::spell_protocol::tests::initial();
        let success = super::super::spell_protocol::tests::success(&initial);
        let mut frame = success.frame.unwrap();
        let cast = frame.world.casts[0];
        assert!(!cast_has_later_action(
            &cast,
            &frame.world.actors[0],
            Some(&frame.vitals[0])
        ));
        frame.world.actors[0].position.x += 1;
        assert!(cast_has_later_action(
            &cast,
            &frame.world.actors[0],
            Some(&frame.vitals[0])
        ));
        frame.world.actors[0].position = cast.to;
        frame.vitals[0].attack_revision = cast.cast_revision + 1;
        assert!(cast_has_later_action(
            &cast,
            &frame.world.actors[0],
            Some(&frame.vitals[0])
        ));
    }

    #[test]
    fn central_record_display_distinguishes_absence_unavailable_stale_and_other_source() {
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let text = source_data_text(&view, DetailView::Record, &DescriptionState::Loading).unwrap();
        assert!(text.contains("尚未讀取"));
        assert!(!text.contains("勝利：0"));
        view.record = record::State::Unsupported;
        assert!(!can_read_record(&view));
        assert!(record_text(&view).contains("v5"));
        view.record
            .receive(record::Reply {
                context,
                status: record::Status::Unavailable,
                totals: None,
            })
            .unwrap();
        assert!(!record_text(&view).contains("勝利：0"));
        view.record
            .receive(record::Reply {
                context,
                status: record::Status::Ok,
                totals: Some(record::Totals {
                    stored: false,
                    wins: 0,
                    losses: 0,
                    revision: 0,
                }),
            })
            .unwrap();
        assert!(record_text(&view).contains("查詢成功：中央尚無"));
        view.record
            .receive(record::Reply {
                context,
                status: record::Status::Ok,
                totals: Some(record::Totals {
                    stored: true,
                    wins: 4_000_000_000,
                    losses: 9,
                    revision: 4_000_000_009,
                }),
            })
            .unwrap();
        view.record
            .receive(record::Reply {
                context,
                status: record::Status::Busy,
                totals: None,
            })
            .unwrap();
        let text = source_data_text(&view, DetailView::Record, &DescriptionState::Loading).unwrap();
        assert!(text.contains("4000000000"));
        assert!(text.contains("上次確認值"));
        assert!(text.contains(&context.key.object_id.to_string()));
        assert!(text.contains("太陽神"));
        assert!(can_read_record(&view));
        validate_snapshot(&view).unwrap();
        view.record_pending = true;
        assert!(!can_read_record(&view));
        assert!(return_command(&view).is_none());
        assert!(channel_command(&view, 0).is_none());
        validate_snapshot(&view).unwrap();
        view.spell_pending = true;
        assert!(validate_snapshot(&view).is_err());
        view.spell_pending = false;
        view.record_pending = false;
        view.navigation.as_mut().unwrap().context.key.source_id = 42;
        assert!(validate_snapshot(&view).is_err());
    }

    #[test]
    fn magic_names_use_shared_text_without_hiding_catalog_names_or_unimplemented_rows() {
        use super::super::spell_protocol::{
            self as spell, Skill, SkillsReply, SkillsState, Support,
        };
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let mut reply = SkillsReply {
            context,
            status: spell::Status::Ok,
            skills: vec![
                Skill {
                    skill_id: 5,
                    support: Support::Teleport,
                    mp_cost: 10,
                    reuse_millis: 10,
                    name: "$0".into(),
                },
                Skill {
                    skill_id: 87,
                    support: Support::Unimplemented,
                    mp_cost: 15,
                    reuse_millis: 8000,
                    name: "衝擊之暈".into(),
                },
            ],
        };
        let names = DescriptionState::Ready(Arc::new(
            DescriptionCatalog::parse_cp950(b"1\r\nShared teleport\r\n").unwrap(),
        ));
        view.skills = SkillsState::Ready(Arc::new(reply.clone()));
        let text = source_data_text(&view, DetailView::Skills, &names).unwrap();
        assert!(text.contains("#5 Shared teleport"));
        assert!(text.contains("原始名稱 token：$0"));
        assert!(text.contains("#87 衝擊之暈"));
        assert!(text.contains("已學，尚未實作／不可施放"));
        reply.skills[0].name = "$1".into();
        view.skills = SkillsState::Ready(Arc::new(reply));
        let text = source_data_text(&view, DetailView::Skills, &names).unwrap();
        assert!(text.contains("#5 名稱無法解析"));
        assert!(text.contains("原始名稱 token：$1"));
        assert!(text.contains("#87 衝擊之暈"));
        for state in [
            DescriptionState::Loading,
            DescriptionState::Unavailable("shared text unavailable".into()),
        ] {
            let text = source_data_text(&view, DetailView::Skills, &state).unwrap();
            assert!(text.contains("#5 "));
            assert!(text.contains("#87 "));
            assert!(text.contains("原始名稱 token：衝擊之暈"));
            assert!(!can_cast_teleport(&view));
        }
    }

    #[test]
    fn magic_view_reports_real_support_and_cast_requires_atomic_alive_confirmed_world() {
        use super::super::spell_protocol::{
            self as spell, Skill, SkillsReply, SkillsState, Support,
        };
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        view.skills = SkillsState::Ready(Arc::new(SkillsReply {
            context,
            status: spell::Status::Ok,
            skills: vec![
                Skill {
                    skill_id: 5,
                    support: Support::Teleport,
                    mp_cost: 10,
                    reuse_millis: 10,
                    name: "瞬間移動".into(),
                },
                Skill {
                    skill_id: 220,
                    support: Support::Unimplemented,
                    mp_cost: 100,
                    reuse_millis: 1,
                    name: "尚未完成技能".into(),
                },
            ],
        }));
        let text = source_data_text(&view, DetailView::Skills, &DescriptionState::Loading).unwrap();
        assert!(text.contains("瞬間移動"));
        assert!(text.contains("尚未完成技能"));
        assert!(text.contains("未實作"));
        assert!(!can_cast_teleport(&view));
        let mut combat = spell::tests::initial();
        combat.echo.context = context;
        // Use the selected row identity, not the source label or roster index.
        let frame = combat.frame.as_mut().unwrap();
        frame.world.actors[0].key = context.key;
        frame.world.room.members[0].key = context.key;
        frame.world.room.owner = context.key;
        view.phase = LoginPhase::InLobby;
        view.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Lobby;
        view.navigation.as_mut().unwrap().snapshot.channel_id = Some(combat.echo.channel_id);
        view.combat_supported = true;
        view.combat = Some(Arc::new(combat.frame.clone().unwrap()));
        view.world = world::State::Ready(Arc::new(combat.world_reply().unwrap()));
        assert!(can_cast_teleport(&view));
        assert!(!can_cast_skill(&view, spell::LIGHT_ARROW));
        let SkillsState::Ready(skills) = &view.skills else {
            panic!("skills");
        };
        let mut learned = (**skills).clone();
        learned.skills.insert(
            0,
            Skill {
                skill_id: 4,
                support: Support::LightArrow,
                mp_cost: 3,
                reuse_millis: 10,
                name: "光箭".into(),
            },
        );
        view.skills = SkillsState::Ready(Arc::new(learned));
        assert!(
            !can_cast_skill(&view, spell::LIGHT_ARROW),
            "a learned row cannot enable an unqualified match"
        );
        let mut current = (**view.combat.as_ref().unwrap()).clone();
        current.damages = current
            .world
            .actors
            .iter()
            .map(|a| spell::DamageCastState::initial(a.entity_id, 1))
            .collect();
        view.combat = Some(Arc::new(current));
        assert!(can_cast_skill(&view, spell::LIGHT_ARROW));
        assert!(
            can_cast_teleport(&view),
            "target magic must not replace teleport"
        );
        for case in 0..6 {
            let mut bad = view.clone();
            match case {
                0 => bad.spell_pending = true,
                1 => bad.world_pending = true,
                2 => bad.combat = None,
                3 => bad.skills = SkillsState::Unsupported,
                4 => {
                    let mut frame = (**bad.combat.as_ref().unwrap()).clone();
                    frame.vitals[0].hp = 0;
                    bad.combat = Some(Arc::new(frame));
                }
                5 => bad.skills = SkillsState::NotRequested,
                _ => unreachable!(),
            };
            assert!(!can_cast_teleport(&bad), "case {case}");
            assert!(
                !can_cast_skill(&bad, spell::LIGHT_ARROW),
                "targeted case {case}"
            );
        }
        view.spell_pending = true;
        assert!(return_command(&view).is_none());
        assert!(room_common(&view, rooms::Action::Read, None, 0).is_none());
        view.phase = LoginPhase::ChoosingChannel;
        assert!(channel_command(&view, 0).is_none());
    }

    #[test]
    fn lobby_display_requires_confirmed_matching_identity_and_exact_four_server_channels() {
        let mut snapshot = channel_snapshot();
        snapshot.phase = LoginPhase::InLobby;
        let reply = snapshot.navigation.as_mut().unwrap();
        reply.snapshot.phase = NavigationPhase::Lobby;
        reply.snapshot.channel_id = Some(2);
        validate_snapshot(&snapshot).unwrap();
        let details = navigation_details(&snapshot).unwrap();
        assert!(details.contains("勇者(太陽神)"));
        assert!(details.contains("伺服器提供的頻道2 大廳"));
        assert!(details.contains("2/20"));
        let mut invalid = snapshot.clone();
        invalid.navigation.as_mut().unwrap().context.key.source_id = 42;
        assert!(validate_snapshot(&invalid).is_err());
        invalid = snapshot.clone();
        invalid.navigation.as_mut().unwrap().snapshot.channel_id = None;
        assert!(validate_snapshot(&invalid).is_err());
        invalid = snapshot.clone();
        invalid.navigation.as_mut().unwrap().snapshot.channels.pop();
        assert!(validate_snapshot(&invalid).is_err());
        invalid = snapshot.clone();
        invalid.roster_epoch = 9;
        assert!(
            validate_snapshot(&invalid).is_err(),
            "a lobby cannot carry a stale selectable roster"
        );
        invalid = snapshot;
        invalid.navigation = None;
        assert!(
            validate_snapshot(&invalid).is_err(),
            "lobby UI cannot be synthesized without a server reply"
        );
    }

    #[test]
    fn back_control_follows_lobby_to_channels_to_characters_and_blocks_pending_mutations() {
        let mut snapshot = channel_snapshot();
        let context = snapshot.navigation.as_ref().unwrap().context;
        assert!(
            matches!(return_command(&snapshot), Some(LoginCommand::ReturnCharacters(value)) if value == context)
        );
        assert_eq!(return_label(snapshot.phase), "返回選角");
        snapshot.phase = LoginPhase::InLobby;
        snapshot.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Lobby;
        snapshot.navigation.as_mut().unwrap().snapshot.channel_id = Some(2);
        assert!(
            matches!(return_command(&snapshot), Some(LoginCommand::LeaveLobby(value)) if value == context)
        );
        assert_eq!(return_label(snapshot.phase), "返回頻道");
        snapshot.phase = LoginPhase::LeavingLobby;
        validate_snapshot(&snapshot).unwrap();
        assert_eq!(return_label(snapshot.phase), "返回頻道");
        assert!(return_command(&snapshot).is_none());
        assert!(channel_command(&snapshot, 2).is_none());
        assert!(!enabled(snapshot.phase, false).source);
        assert!(enabled(snapshot.phase, false).disconnect);
        let mut invalid = snapshot.clone();
        invalid.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Channels;
        invalid.navigation.as_mut().unwrap().snapshot.channel_id = None;
        assert!(
            validate_snapshot(&invalid).is_err(),
            "pending leave must retain the previous lobby ACK"
        );
        snapshot.phase = LoginPhase::ChoosingChannel;
        snapshot.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Channels;
        snapshot.navigation.as_mut().unwrap().snapshot.channel_id = None;
        validate_snapshot(&snapshot).unwrap();
        assert!(
            matches!(return_command(&snapshot), Some(LoginCommand::ReturnCharacters(value)) if value == context)
        );
        assert_eq!(return_label(snapshot.phase), "返回選角");
        snapshot.phase = LoginPhase::ReturningCharacters;
        assert!(return_command(&snapshot).is_none());
    }

    #[test]
    fn readonly_data_views_preserve_full_source_values_tokens_and_all_budgeted_items() {
        use super::super::character_data;
        let mut snapshot = channel_snapshot();
        let mut data = character_data::tests::sample_data();
        data.context = snapshot.navigation.as_ref().unwrap().context;
        data.character.key = data.context.key;
        snapshot.character_data = CharacterDataState::Ready(std::sync::Arc::new(data.clone()));
        validate_snapshot(&snapshot).unwrap();
        let descriptions = DescriptionState::Loading;
        let stats = source_data_text(&snapshot, DetailView::Stats, &descriptions).unwrap();
        assert!(stats.contains("來源保存值，非即時戰鬥"));
        assert!(stats.contains("保存 HP：9223372036854775807"));
        assert!(stats.contains("保存 AC：-9223372036854775808"));
        assert!(stats.contains("飽食度：225/225（100%）"));
        assert!(
            stats.contains("等級 4000000000"),
            "fresh data not overwritten by navigation level"
        );
        let items = source_data_text(&snapshot, DetailView::Items, &descriptions).unwrap();
        assert!(items.contains("來源背包保存資料（尚未合併 LiTo 私有狀態）"));
        assert!(items.contains("原始名稱 token：$123"));
        assert!(items.contains("[已裝備（保存）]"));
        let template = data.items[0].clone();
        data.items = (1..=512)
            .map(|id| {
                let mut item = template.clone();
                item.object_id = id;
                item.name_token = "x".repeat(256);
                item
            })
            .collect();
        snapshot.character_data = CharacterDataState::Ready(std::sync::Arc::new(data));
        validate_snapshot(&snapshot).unwrap();
        let full = source_data_text(&snapshot, DetailView::Items, &descriptions).unwrap();
        assert!(full.contains("實例 ID 512／"));
        assert!(full.encode_utf16().count() < MAX_DETAILS_UTF16);
        snapshot.navigation.as_mut().unwrap().context.epoch += 1;
        assert!(
            validate_snapshot(&snapshot).is_err(),
            "old source data must not survive a new selection generation"
        );
        snapshot.character_data =
            CharacterDataState::Unavailable(CharacterDataFailure::BackendUnavailable);
        let unavailable = source_data_text(&snapshot, DetailView::Items, &descriptions).unwrap();
        assert!(unavailable.contains("不代表空背包或零素質"));
        snapshot.character_data = CharacterDataState::Unsupported;
        assert!(
            source_data_text(&snapshot, DetailView::Stats, &descriptions)
                .unwrap()
                .contains("未協商")
        );
        snapshot.navigation = None;
        snapshot.character_data = CharacterDataState::NotRequested;
        assert!(source_data_text(&snapshot, DetailView::Stats, &descriptions).is_none());
    }

    #[test]
    fn shared_names_are_resolved_without_hiding_unresolved_items_or_source_identity() {
        use super::super::character_data;
        let mut snapshot = channel_snapshot();
        let mut data = character_data::tests::sample_data();
        data.context = snapshot.navigation.as_ref().unwrap().context;
        data.character.key = data.context.key;
        data.items[0].name_token = "$0".into();
        let catalog = DescriptionCatalog::parse_cp950(b"2\r\nShared sword\r\n").unwrap();
        let names = DescriptionState::Ready(Arc::new(catalog));
        snapshot.character_data = CharacterDataState::Ready(Arc::new(data.clone()));
        let text = source_data_text(&snapshot, DetailView::Items, &names).unwrap();
        assert!(text.contains("Shared sword"));
        assert!(text.contains("原始名稱 token：$0"));
        assert!(text.contains(&format!("實例 ID {}／", data.items[0].object_id)));
        data.items[0].name_token = "$1".into();
        snapshot.character_data = CharacterDataState::Ready(Arc::new(data));
        let text = source_data_text(&snapshot, DetailView::Items, &names).unwrap();
        assert!(text.contains("名稱無法解析"));
        assert!(text.contains("原始名稱 token：$1"));
        let unavailable = DescriptionState::Unavailable("測試資源無法讀取".into());
        let text = source_data_text(&snapshot, DetailView::Items, &unavailable).unwrap();
        assert!(text.contains("測試資源無法讀取"));
        assert!(
            text.contains("實例 ID"),
            "name failure cannot erase inventory"
        );

        let mut full_data = character_data::tests::sample_data();
        full_data.context = snapshot.navigation.as_ref().unwrap().context;
        full_data.character.key = full_data.context.key;
        let template = full_data.items[0].clone();
        full_data.items = (1..=512)
            .map(|id| {
                let mut item = template.clone();
                item.object_id = id;
                item.name_token = "$0".repeat(128);
                item
            })
            .collect();
        snapshot.character_data = CharacterDataState::Ready(Arc::new(full_data));
        let names = DescriptionState::Ready(Arc::new(
            DescriptionCatalog::parse_cp950(b"1\r\nname").unwrap(),
        ));
        let full = source_data_text(&snapshot, DetailView::Items, &names).unwrap();
        assert!(full.contains(&"name".repeat(128)));
        assert!(full.contains("實例 ID 512／"));
        assert!(
            checked_text_units(&full).is_ok(),
            "maximum expanded names must retain every row"
        );
    }

    #[test]
    fn ui_text_contract_rejects_embedded_nul_and_oversize_without_truncation() {
        assert_eq!(checked_text_units("").unwrap(), vec![0]);
        assert_eq!(checked_text_units("道具🗡").unwrap().len(), 5);
        assert!(checked_text_units("before\0after").is_err());
        assert_eq!(
            checked_text_units(&"x".repeat(MAX_DETAILS_UTF16))
                .unwrap()
                .len(),
            MAX_DETAILS_UTF16 + 1
        );
        assert!(checked_text_units(&"x".repeat(MAX_DETAILS_UTF16 + 1)).is_err());
        assert!(checked_text_units(&"🗡".repeat(MAX_DETAILS_UTF16 / 2 + 1)).is_err());
    }

    #[test]
    fn owned_description_job_keeps_polling_nonblocking_and_discards_cancelled_completion() {
        use std::time::Duration;
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (release, wait) = mpsc::sync_channel(1);
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(worker_cancel.load(Ordering::Acquire));
            let _ = sender.send(Ok(DescriptionCatalog::parse_cp950(b"1\r\nsword").unwrap()));
        });
        let mut descriptions = Descriptions {
            state: DescriptionState::Loading,
            job: Some(DescriptionJob {
                cancel,
                receiver,
                thread: Some(handle),
            }),
            revision: 0,
        };
        descriptions.poll(true);
        assert!(
            !descriptions.is_quiescent(),
            "cancellation is not completion"
        );
        assert_eq!(descriptions.revision, 0);
        release.send(()).unwrap();
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        while !descriptions.is_quiescent() {
            descriptions.poll(true);
            assert!(
                std::time::Instant::now() < deadline,
                "owned load did not retire"
            );
            thread::yield_now();
        }
        assert!(matches!(
            descriptions.state,
            DescriptionState::Unavailable(_)
        ));
        assert_eq!(descriptions.revision, 1);
        descriptions.poll(false);
        assert!(
            matches!(descriptions.state, DescriptionState::Unavailable(_)),
            "late result cannot resurrect cancelled resource"
        );
    }

    #[test]
    fn completed_description_load_publishes_once_after_join_and_retains_catalog() {
        use std::time::{Duration, Instant};
        let (sender, receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            sender
                .send(Ok(DescriptionCatalog::parse_cp950(b"1\r\nsword").unwrap()))
                .unwrap();
        });
        let mut descriptions = Descriptions {
            state: DescriptionState::Loading,
            job: Some(DescriptionJob {
                cancel: Arc::new(AtomicBool::new(false)),
                receiver,
                thread: Some(handle),
            }),
            revision: 0,
        };
        let deadline = Instant::now() + Duration::from_secs(5);
        while !descriptions.is_quiescent() {
            descriptions.poll(false);
            assert!(Instant::now() < deadline);
            thread::yield_now();
        }
        assert_eq!(descriptions.state.item_label("$0"), "sword");
        assert_eq!(descriptions.revision, 1);
        let catalog = match &descriptions.state {
            DescriptionState::Ready(catalog) => catalog.clone(),
            _ => panic!("successful owned load did not publish"),
        };
        for _ in 0..100 {
            descriptions.poll(false);
            assert_eq!(
                descriptions.revision, 1,
                "heartbeat does not invalidate names"
            );
            assert!(
                matches!(&descriptions.state, DescriptionState::Ready(current) if Arc::ptr_eq(current, &catalog))
            );
        }
    }

    #[test]
    fn equipment_ui_commands_use_instance_context_revision_and_token_without_optimistic_flags() {
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let gear = equipment::tests::sample_snapshot(context);
        view.character_data = CharacterDataState::Ready(gear.source.clone());
        view.equipment = equipment::State::Ready(gear.clone());
        validate_snapshot(&view).unwrap();
        let Some(LoginCommand::Equipment(request)) = equipment_command(&view, Some(1), Some(true))
        else {
            panic!()
        };
        assert_eq!(request, equipment::Request::set(&gear, 1, true));
        assert_eq!(gear.equipped(1), Some(false));
        assert!(equipment_command(&view, Some(1), Some(false)).is_none());
        assert!(equipment_command(&view, Some(99999), Some(true)).is_none());
        assert!(
            matches!(equipment_command(&view, None, None), Some(LoginCommand::Equipment(request)) if request == equipment::Request::read(context))
        );
        let text = source_data_text(&view, DetailView::Items, &DescriptionState::Loading).unwrap();
        assert!(text.contains("背包（LiTo 確認）"));
        assert!(text.contains("來源 SQL 保存旗標：true"));
        let stats = source_data_text(&view, DetailView::Stats, &DescriptionState::Loading).unwrap();
        assert!(stats.contains("裝備加成"));
        assert!(stats.contains("非完整即時素質"));
        for name in equipment::STAT_NAMES {
            assert!(stats.contains(name));
        }
        view.equipment_pending = true;
        assert!(equipment_command(&view, Some(1), Some(true)).is_none());
        assert!(equipment_command(&view, None, None).is_none());
        assert!(return_command(&view).is_none());
        assert!(channel_command(&view, 0).is_none());
        view.equipment_pending = false;
        view.equipment = equipment::State::Unsupported;
        assert!(equipment_command(&view, None, None).is_none());
        view.equipment = equipment::State::Unavailable(equipment::Status::Busy);
        assert!(equipment_command(&view, Some(1), Some(true)).is_none());
        assert!(equipment_command(&view, None, None).is_some());
    }
    #[test]
    fn expired_and_unqualified_lifetimes_preserve_sql_inventory_and_disable_only_known_unsafe_equip(
    ) {
        let mut view = channel_snapshot();
        let context = view.navigation.as_ref().unwrap().context;
        let legacy = equipment::tests::sample_snapshot(context);
        assert!(item_lifetime_text(&legacy, 1).contains("版本 1 未提供"));
        let mut snapshot = (*legacy).clone();
        snapshot.lifetimes = Some(vec![
            equipment::LifetimeView {
                item_id: 1,
                state: equipment::LifetimeState::Expired,
                expires_at_millis: Some(123),
            },
            equipment::LifetimeView {
                item_id: 2,
                state: equipment::LifetimeState::Permanent,
                expires_at_millis: None,
            },
        ]);
        snapshot.stale = vec![equipment::StaleItem {
            item_id: 99,
            catalog_id: 88,
            reason: equipment::StaleReason::FactsChanged,
        }];
        snapshot.validate().unwrap();
        let gear = Arc::new(snapshot);
        view.character_data = CharacterDataState::Ready(gear.source.clone());
        view.equipment = equipment::State::Ready(gear.clone());
        validate_snapshot(&view).unwrap();
        assert!(gear.source.items[0].equipped); // Exact source SQL flag is not rewritten.
        assert_eq!(gear.equipped(1), Some(false));
        assert!(equipment_command(&view, Some(1), Some(true)).is_none());
        assert!(equipment_command(&view, Some(2), Some(true)).is_some());
        assert!(equipment_command(&view, None, None).is_some());
        let text = source_data_text(&view, DetailView::Items, &DescriptionState::Loading).unwrap();
        assert!(text.contains("已到期；保留來源背包，不可裝備"));
        assert!(text.contains("來源 SQL 保存旗標：true"));
        assert!(text.contains("未刪除來源 SQL 物品"));
        assert!(text.contains("失效的舊覆寫"));
        assert!(text.contains("來源物件事實已變更"));
        assert!(!text.contains("過期覆寫"));
        let mut next = (*gear).clone();
        next.lifetimes.as_mut().unwrap()[0] = equipment::LifetimeView {
            item_id: 1,
            state: equipment::LifetimeState::Unqualified,
            expires_at_millis: None,
        };
        view.equipment = equipment::State::Ready(Arc::new(next));
        assert!(equipment_command(&view, Some(1), Some(true)).is_none());
        assert!(equipment_command(&view, None, None).is_some());
    }
    #[test]
    fn client_layout_scales_without_desktop_coordinates_or_overlapping_edits() {
        for dpi in [96, 120, 144, 192, 288] {
            let width = scale(MIN_WIDTH_DIP, dpi);
            let height = scale(MIN_HEIGHT_DIP, dpi);
            let boxes = layout(width, height, dpi);
            for (index, rect) in boxes.iter().enumerate() {
                assert!(
                    rect.x >= 0 && rect.y >= 0 && rect.width > 0 && rect.height > 0,
                    "{index}: {rect:?}"
                );
                assert!(rect.x + rect.width <= width);
                assert!(rect.y + rect.height <= height || matches!(index, SOURCE | GEAR_ITEM));
            }
            assert!(boxes[ACCOUNT].y + boxes[ACCOUNT].height <= boxes[PASSWORD].y);
            assert!(boxes[GM_LABEL].y + boxes[GM_LABEL].height <= boxes[GM_COUNT].y);
            assert!(boxes[GM_COUNT].x + boxes[GM_COUNT].width <= boxes[GM_SEND].x);
            assert!(boxes[GM_COUNT].y + boxes[GM_COUNT].height <= boxes[CHARACTER_LABEL].y);
            assert!(boxes[PASSWORD].y + boxes[PASSWORD].height <= boxes[LOGIN].y);
            assert!(boxes[CHARACTERS].y + boxes[CHARACTERS].height <= boxes[DETAILS].y);
            assert_eq!(boxes[CHARACTERS], boxes[CHANNELS]);
            assert!(boxes[CHARACTERS].y + boxes[CHARACTERS].height <= boxes[SELECT_CHARACTER].y);
            assert!(boxes[SELECT_CHARACTER].y + boxes[SELECT_CHARACTER].height <= boxes[DETAILS].y);
            assert!(
                boxes[SELECT_CHARACTER].y + boxes[SELECT_CHARACTER].height <= boxes[SHOW_STATS].y
            );
            assert!(boxes[SHOW_STATS].y + boxes[SHOW_STATS].height <= boxes[DETAILS].y);
            assert!(boxes[SHOW_STATS].x + boxes[SHOW_STATS].width <= boxes[SHOW_ITEMS].x);
            assert!(boxes[DETAILS].height >= scale(160, dpi));
            assert!(boxes[GEAR_READ].y + boxes[GEAR_READ].height <= boxes[SHOW_SKILLS].y);
            assert!(boxes[SHOW_SKILLS].y + boxes[SHOW_SKILLS].height <= boxes[DETAILS].y);
            assert!(boxes[SHOW_SKILLS].width >= scale(110, dpi));
            assert!(boxes[SKILLS_READ].x + boxes[SKILLS_READ].width <= boxes[CAST_TELEPORT].x);
            assert!(boxes[CAST_TELEPORT].x + boxes[CAST_TELEPORT].width <= boxes[SHOW_RECORD].x);
            assert!(boxes[SHOW_RECORD].x + boxes[SHOW_RECORD].width <= boxes[RECORD_READ].x);
            assert!(boxes[RECORD_READ].x + boxes[RECORD_READ].width <= width - scale(16, dpi));
            assert!(boxes[DETAILS].y + boxes[DETAILS].height <= boxes[EXIT].y);
            assert!(boxes[FOOTNOTE].x + boxes[FOOTNOTE].width <= boxes[CHAT].x);
            assert!(boxes[CHAT].x + boxes[CHAT].width <= boxes[SETTINGS].x);
            assert!(boxes[SETTINGS].x + boxes[SETTINGS].width <= boxes[EXIT].x);
            assert_eq!(boxes[ACCOUNT].x, boxes[PASSWORD].x);
        }
    }
    #[test]
    fn full_vip_status_inventory_and_text_have_nonoverlapping_minimum_layouts() {
        for dpi in [96, 120, 144, 192, 288] {
            let width = scale(MIN_WIDTH_DIP, dpi);
            let height = scale(MIN_HEIGHT_DIP, dpi);
            for status in [false, true] {
                let boxes = arranged_layout(width, height, dpi, status, true);
                if status {
                    assert!(boxes[VIP_STATUS_STRIP].y + boxes[VIP_STATUS_STRIP].height
                        <= boxes[VIP_ITEMS].y);
                }
                assert!(boxes[VIP_ITEMS].height >= scale(56, dpi));
                assert!(boxes[VIP_ITEMS].y + boxes[VIP_ITEMS].height < boxes[DETAILS].y);
                assert!(boxes[DETAILS].height >= scale(52, dpi));
                assert!(boxes[DETAILS].y + boxes[DETAILS].height <= boxes[EXIT].y);
                assert_eq!(boxes[VIP_ITEMS].width, boxes[DETAILS].width);
            }
        }
    }
    #[test]
    fn vip_inventory_reading_follows_instance_across_refresh_but_not_owner_change() {
        use super::super::vip_inventory_preview::{Key, Model, Row};
        let key = Key {
            context: SelectionContext { key: CharacterKey {
                source_id: 7, object_id: 3333 }, epoch: 9 },
            channel: 0, room: 1, match_id: 2, generation: 3,
            entity: 4, life: 5, graphics: vec![1],
        };
        let row = |object| Row { object, graphic: 1, label: format!("物品 {object}"),
            quantity: 1, equipped: Some(false) };
        let old = Model { key, rows: vec![row(10), row(20), row(30), row(40)] };
        let mut changed = old.clone();
        changed.rows[2].quantity = 99;
        changed.rows[3].equipped = Some(true);
        assert_eq!(restore_inventory_reading(&old, &changed, 2, Some(3)),
            Some(InventoryReadingPosition { top: 2, selected: Some(3) }));
        let mut rearranged = changed.clone();
        rearranged.rows = vec![row(30), row(10), row(40), row(20)];
        assert_eq!(restore_inventory_reading(&old, &rearranged, 2, Some(3)),
            Some(InventoryReadingPosition { top: 0, selected: Some(2) }));
        rearranged.rows = vec![row(10), row(40), row(20)];
        assert_eq!(restore_inventory_reading(&old, &rearranged, 2, Some(3)),
            Some(InventoryReadingPosition { top: 2, selected: Some(1) }));
        rearranged.rows = vec![row(10), row(20)];
        assert_eq!(restore_inventory_reading(&old, &rearranged, 2, Some(3)),
            Some(InventoryReadingPosition { top: 1, selected: None }));
        rearranged.key.context.key.source_id = 42;
        assert_eq!(restore_inventory_reading(&old, &rearranged, 2, Some(3)), None);
        rearranged.key.context.key.source_id = 7;
        rearranged.key.life += 1;
        assert_eq!(restore_inventory_reading(&old, &rearranged, 2, Some(3)), None);
    }
    #[test]
    fn gm_counts_are_bounded_and_controls_do_not_mistake_capability_for_permission() {
        for (value, expected) in [
            ("0", Some(0)),
            ("1", Some(1)),
            ("100", Some(100)),
            ("", None),
            ("101", None),
            ("-1", None),
            ("1.0", None),
            (" 1", None),
            ("１", None),
            ("1000", None),
        ] {
            assert_eq!(parse_bot_count(value), expected);
        }
        let mut view = channel_snapshot();
        view.phase = LoginPhase::InLobby;
        view.navigation.as_mut().unwrap().snapshot.phase = NavigationPhase::Lobby;
        view.navigation.as_mut().unwrap().snapshot.channel_id = Some(0);
        let context = view.navigation.as_ref().unwrap().context;
        view.rooms = rooms::State::Ready(Arc::new(rooms::tests::sample(context, true)));
        assert!(!gm_input_ready(&view));
        view.gm_supported = true;
        assert!(
            gm_input_ready(&view),
            "UI offers command but server freshly authorizes it"
        );
        for case in 0..6 {
            let mut pending = view.clone();
            match case {
                0 => pending.room_pending = true,
                1 => pending.equipment_pending = true,
                2 => pending.world_pending = true,
                3 => pending.combat_pending = true,
                4 => pending.spell_pending = true,
                _ => pending.record_pending = true,
            }
            assert!(!gm_input_ready(&pending));
        }
        let rooms::State::Ready(room) = &view.rooms else {
            panic!()
        };
        let mut bot_only = (**room).clone();
        bot_only.joined.as_mut().unwrap().members[0].kind = rooms::MemberKind::ServerBot;
        view.rooms = rooms::State::Ready(Arc::new(bot_only));
        assert!(!gm_input_ready(&view));
    }

    #[test]
    fn pending_source_is_interactive_but_credentials_and_closing_are_gated() {
        assert!(enabled(LoginPhase::LoginReady, false).credentials);
        for phase in [
            LoginPhase::Connecting,
            LoginPhase::Authenticating,
            LoginPhase::LoadingCharacters,
        ] {
            assert!(!enabled(phase, false).credentials);
        }
        assert!(enabled(LoginPhase::LoadingCharacters, false).source);
        assert!(enabled(LoginPhase::Disconnected, false).connect);
        assert!(!enabled(LoginPhase::CharactersReady, false).connect);
        for phase in [
            LoginPhase::LoginReady,
            LoginPhase::LoadingCharacters,
            LoginPhase::Failed,
        ] {
            assert_eq!(
                enabled(phase, true),
                Enabled {
                    connect: false,
                    credentials: false,
                    source: false,
                    disconnect: false
                }
            );
        }
    }

    #[test]
    fn snapshot_rejects_duplicate_stale_or_oversized_list_data() {
        let good = snapshot();
        assert!(validate_snapshot(&good).is_ok());
        let mut bad = good.clone();
        bad.characters[0].source_id = 4;
        assert!(validate_snapshot(&bad).is_err());
        let mut bad = good.clone();
        bad.sources.push(bad.sources[0].clone());
        assert!(validate_snapshot(&bad).is_err());
        let mut bad = good.clone();
        bad.characters.push(bad.characters[0].clone());
        assert!(validate_snapshot(&bad).is_err());
        let mut bad = good.clone();
        bad.selected_source = Some(99);
        assert!(validate_snapshot(&bad).is_err());
        let mut bad = good.clone();
        bad.characters[0].label = "x\0hidden".into();
        assert!(validate_snapshot(&bad).is_err());
        let mut bad = good;
        bad.sources = vec![
            SourceChoice {
                id: 1,
                label: "x".into()
            };
            MAX_SOURCES + 1
        ];
        assert!(validate_snapshot(&bad).is_err());
    }

    #[test]
    fn credentials_are_limited_in_utf8_not_only_edit_utf16_units() {
        assert!(validate_credentials("3333", "3333").is_ok());
        assert!(validate_credentials(&"字".repeat(21), &"字".repeat(85)).is_ok());
        assert!(validate_credentials(&"字".repeat(22), "secret").is_err());
        assert!(validate_credentials("account", &"字".repeat(86)).is_err());
        assert!(validate_credentials("", "secret").is_err());
        assert!(validate_credentials("account", "").is_err());
        assert!(validate_credentials("account\n", "secret").is_err());
        assert!(validate_credentials("account", "secret\0").is_err());
    }

    #[test]
    fn status_or_loading_updates_do_not_reset_native_source_or_character_selection() {
        let previous = snapshot();
        let mut next = previous.clone();
        next.status = "收到連線保活回覆".into();
        assert_eq!(
            list_changes(Some(&previous), &next),
            ListChanges {
                sources: false,
                characters: false,
            }
        );
        next.phase = LoginPhase::LoadingCharacters;
        assert_eq!(
            list_changes(Some(&previous), &next),
            ListChanges {
                sources: false,
                characters: false,
            }
        );
        next.sources.push(SourceChoice {
            id: 4,
            label: "月亮神".into(),
        });
        next.selected_source = Some(4);
        next.characters.clear();
        assert_eq!(
            list_changes(Some(&previous), &next),
            ListChanges {
                sources: true,
                characters: true,
            }
        );
    }

    #[test]
    fn rejected_source_change_restores_native_index_without_rebuilding_lists() {
        let previous = snapshot();
        let unchanged = previous.clone();
        assert_eq!(
            list_changes(Some(&previous), &unchanged),
            ListChanges {
                sources: false,
                characters: false,
            }
        );
        // Native combo already moved to B (index 1), but the controller rejected
        // the command as busy; accepted source A remains index 0 in its snapshot.
        assert_eq!(source_selection_to_apply(1, Some(0)), Some(0));
        assert_eq!(source_selection_to_apply(0, Some(0)), None);
        // Accepted B is already selected natively: no redundant SETCURSEL.
        assert_eq!(source_selection_to_apply(1, Some(1)), None);
        assert_eq!(source_selection_to_apply(1, None), Some(usize::MAX));
        assert_eq!(source_selection_to_apply(-1, None), None);
        assert_eq!(source_selection_to_apply(-1, Some(0)), Some(0));
    }

    #[test]
    fn unsolicited_disconnect_clears_unsubmitted_edits_once_not_on_every_timer() {
        let mut before = snapshot();
        before.phase = LoginPhase::LoginReady;
        for phase in [
            LoginPhase::Failed,
            LoginPhase::Disconnected,
            LoginPhase::Disconnecting,
        ] {
            let mut after = before.clone();
            after.phase = phase;
            assert!(clear_credentials_on_transition(Some(&before), &after));
            assert!(!clear_credentials_on_transition(Some(&after), &after));
        }
        assert!(!clear_credentials_on_transition(Some(&before), &before));
        let mut authentication = before.clone();
        authentication.phase = LoginPhase::Authenticating;
        assert!(!clear_credentials_on_transition(
            Some(&before),
            &authentication
        ));
    }

    #[test]
    fn local_busy_error_expires_when_authoritative_status_or_phase_changes() {
        let previous = snapshot();
        assert!(!snapshot_changed(Some(&previous), &previous));
        let mut changed = previous.clone();
        changed.status = "連線已中斷".into();
        assert!(snapshot_changed(Some(&previous), &changed));
        changed.status = previous.status.clone();
        changed.phase = LoginPhase::Failed;
        assert!(snapshot_changed(Some(&previous), &changed));
        assert!(!snapshot_changed(None, &previous));
    }

    #[test]
    fn locally_rejected_credentials_are_zeroized_without_changing_command_api() {
        let mut command = validated_login_command(
            Zeroizing::new("test-account".into()),
            Zeroizing::new("test-password".into()),
        )
        .unwrap();
        if let LoginCommand::Login { account, password } = &command {
            assert_eq!(account, "test-account");
            assert_eq!(password, "test-password");
        } else {
            panic!("expected a login command");
        }
        // Used by the closing/reentrant dispatch rejection branches. The
        // zeroize crate wipes this owned allocation before emptying the String.
        zeroize_unsubmitted(&mut command);
        if let LoginCommand::Login { account, password } = &command {
            assert!(account.is_empty() && password.is_empty());
        } else {
            panic!("expected unchanged command variant");
        }
        assert!(validated_login_command(
            Zeroizing::new("x".repeat(65)),
            Zeroizing::new("test-password".into()),
        )
        .is_err());
        let mut source = LoginCommand::SelectSource(3);
        zeroize_unsubmitted(&mut source);
        assert!(matches!(source, LoginCommand::SelectSource(3)));
    }

    struct ClosingController {
        shutdowns: std::rc::Rc<Cell<usize>>,
        polls_remaining: usize,
    }
    impl LoginController for ClosingController {
        fn poll(&mut self) {
            self.polls_remaining = self.polls_remaining.saturating_sub(1);
        }
        fn snapshot(&self) -> LoginSnapshot {
            snapshot()
        }
        fn command(&mut self, command: LoginCommand) -> std::result::Result<(), String> {
            assert!(matches!(command, LoginCommand::Shutdown));
            self.shutdowns.set(self.shutdowns.get() + 1);
            Ok(())
        }
        fn is_quiescent(&self) -> bool {
            self.polls_remaining == 0
        }
    }

    #[test]
    fn repeated_close_cancels_once_and_waits_for_worker_quiescence_without_joining() {
        let shutdowns = std::rc::Rc::new(Cell::new(0));
        let mut ui = Ui {
            controller: Box::new(ClosingController {
                shutdowns: shutdowns.clone(),
                polls_remaining: 3,
            }),
            last: None,
            local_error: None,
            closing: false,
            fatal: false,
        };
        ui.begin_close();
        ui.begin_close();
        assert_eq!(shutdowns.get(), 1);
        assert!(ui.closing);
        assert!(!ui.controller.is_quiescent());
        for _ in 0..2 {
            ui.controller.poll();
            assert!(!ui.controller.is_quiescent());
        }
        ui.controller.poll();
        assert!(ui.controller.is_quiescent());
    }
}
