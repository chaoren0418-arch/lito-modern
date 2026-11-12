//! Unicode native LOGIN/source UI backed by the persistent TCP owner.
use super::{
    native_connection::Limits,
    native_login::{LoginController, LoginPhase},
};
use anyhow::{ensure, Context, Result};
use std::{cell::RefCell, net::SocketAddr};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                DRAWITEMSTRUCT, EM_EMPTYUNDOBUFFER, EM_SETLIMITTEXT, MEASUREITEMSTRUCT, ODS_FOCUS,
                ODS_SELECTED,
            },
            HiDpi::*,
            Input::KeyboardAndMouse::{EnableWindow, GetAsyncKeyState, GetFocus, IsWindowEnabled, SetFocus, VK_CONTROL},
            WindowsAndMessaging::*,
        },
    },
};
use zeroize::Zeroizing;
use super::native_game::pages::{Pages, Turn};
#[path="native_rental_window.rs"]
mod rental;
#[path = "native_skill_window.rs"]
mod skill_ui;
#[path="native_companion_window.rs"]
mod companions;
const CREATE_ROOM:u16=119;
const EDIT_ROOM:u16=120;
const SAVE_ROOM:u16=121;
const DISCARD_ROOM:u16=122;
const OBSERVER:u16=123;
const KICK_MEMBER:u16=124;
const MODE_PREV:u16=125;
const MODE_NEXT:u16=126;
const MAP_PREV:u16=136;
const MAP_NEXT:u16=137;
const ROOM_NAME:u16=213;
const ROOM_CAPACITY:u16=214;
const ROOM_SECRET:u16=215;
const ROOM_LOCK:u16=216;
const TELEPORT_DEST:u16=217;
const SUMMON_OPTIONS:u16=218;
const ROOM_HELP:u16=313;
const CONNECT: u16 = 101;
const LOGIN: u16 = 102;
const SELECT: u16 = 103;
const CANCEL: u16 = 104;
const GAME: u16 = 105;
const ACK: u16 = 106;
const RETURN: u16 = 107;
const REFRESH: u16 = 108;
const JOIN: u16 = 109;
const READY: u16 = 110;
const UNREADY: u16 = 111;
const EXIT_ROOM: u16 = 112;
const CAST_SKILL: u16 = 113;
const INVENTORY_TOGGLE: u16 = 114;
const INVENTORY_LIST: u16 = 209;
const GROUND_TOGGLE: u16 = 117;
const CHANGE_FORCE: u16 = 118;
const ITEM_USE: u16 = 115;
const ITEM_DROP: u16 = 116;
const ITEM_DESTROY: u16 = 150;
const ITEM_OPTION_SEND: u16 = 151;
const ITEM_OPTION_CANCEL: u16 = 152;
const ITEM_ARGUMENT: u16 = 250;
const ITEM_SUBJECT: u16 = 251;
const ITEM_CONTENT: u16 = 252;
const ITEM_LETTER_HELP: u16 = 317;
const ITEM_MAIL: u16 = 321;
const ITEM_QUANTITY: u16 = 210;
const ITEM_FEEDBACK: u16 = 211;
const ITEM_QUANTITY_LABEL: u16 = 212;
const SKILLS: u16 = 208;
const SKILL_LABEL: u16 = 318;
const ROOM_PASSWORD: u16 = 206;
const ROOMS: u16 = 205;
const ACCOUNT: u16 = 201;
const PASSWORD: u16 = 202;
const SOURCES: u16 = 203;
const STATUS: u16 = 204;
const SCORES: u16 = 207;
const CHARACTER_PREV: u16 = 138;
const CHARACTER_NEXT: u16 = 139;
struct Ui {
    character_pages: Pages,
    rental:rental::View,
    companions:companions::View,
    parent: HWND,
    room_tools_view: bool,
    room_editor: Option<super::native_room_ui::Draft>,
    room_member: Option<super::native_room_ui::MemberChoice>,
    room_choice: Option<super::native_room_ui::RoomChoice>,
    inventory_view: bool,
    ground_view: bool,
    pickup_rows: Vec<super::native_ground_pickup::Choice>,
    pickup_choice: Option<super::native_ground_pickup::Choice>,
    inventory_rows: Vec<(u32, String)>,
    inventory_owner: Option<super::native_inventory::Owner>,
    inventory_names: super::native_inventory_names::Service,
    inventory_icons: super::native_inventory_icons::Service,
    inventory_icon_rows: Vec<(u32, u16)>,
    inventory_choice: Option<super::native_item_actions::Choice>,
    skill_item_target: Option<super::native_skills::ItemTarget>,
    summon_choice: Option<super::native_skills::SummonTarget>,
    teleport_rows: Vec<String>,
    teleport_choice: Option<super::native_teleport_spell::Choice>,
    item_cursor: Option<super::native_item_actions::TargetChoice>,
    item_selection: Option<super::native_item_actions::Selection>,
    destroy_choice: Option<super::native_item_actions::Choice>,
    displayed_mail: Option<(super::native_inventory::Owner, u64)>,
    target: Option<super::native_skill_selection::Target>,
    ground: Option<super::native_skill_selection::Ground>,
    skills: super::native_skill_selection::Model,
    skill_book_open: bool,
    skill_icons: super::native_inventory_icons::Service,
    painted: Option<(u64, u64, super::compositor::Viewport)>,
    painted_input: Option<(u64, super::native_world_scene::InputSurface)>,
    voices: super::combat_audio::CombatVoicePool,
    audio_scope: Option<super::native_audio_delivery::Scope>,
    audio_frame: Option<(u64, u64, u64)>,
    audio_error: Option<String>,
    peer: SocketAddr,
    resource_root: std::path::PathBuf,
    controller: LoginController,
    controls: Vec<(u16, HWND)>,
    names: Vec<String>,
    status: String,
    room_labels: Vec<String>,
    score_labels: Vec<String>,
    result_view: bool,
    world_presentation: super::native_world_view::Presentation,
    world_view: bool,
    held: super::native_navigation::HeldInput,
    held_attack: super::native_combat::HeldAttack,
    closing: bool,
    font: HFONT,
}
struct State(RefCell<Ui>);
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
unsafe fn send(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    SendMessageW(hwnd, msg, Some(wparam), Some(lparam))
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
impl Ui {
    fn character_paging_enabled(&self) -> bool {
        !self.closing && !self.world_view && !self.inventory_view && !self.room_tools_view
            && Pages::enabled(self.controller.snapshot())
    }

    fn control(&self, id: u16) -> HWND {
        self.controls
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, hwnd)| *hwnd)
            .unwrap_or_default()
    }
    unsafe fn text(&self, id: u16, text: &str) {
        let text = wide(text);
        let _ = SetWindowTextW(self.control(id), PCWSTR(text.as_ptr()));
    }
    unsafe fn read(&self, id: u16, max: usize) -> Result<Zeroizing<String>> {
        let hwnd = self.control(id);
        let len = GetWindowTextLengthW(hwnd);
        ensure!(len >= 0 && len as usize <= max, "輸入長度超出限制");
        let mut text = Zeroizing::new(vec![0u16; len as usize + 1]);
        let n = GetWindowTextW(hwnd, &mut text);
        Ok(Zeroizing::new(String::from_utf16(&text[..n as usize])?))
    }
    unsafe fn create(&mut self, parent: HWND) -> Result<()> {
        self.parent = parent;
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        for (id, class, label, extra) in [
            (301, w!("STATIC"), format!("登入伺服器：{}", self.peer), 0),
            (302, w!("STATIC"), "帳號".into(), 0),
            (ACCOUNT, w!("EDIT"), String::new(), ES_AUTOHSCROLL as u32),
            (303, w!("STATIC"), "密碼".into(), 0),
            (
                PASSWORD,
                w!("EDIT"),
                String::new(),
                (ES_PASSWORD | ES_AUTOHSCROLL) as u32,
            ),
            (CONNECT, w!("BUTTON"), "連線".into(), 0),
            (LOGIN, w!("BUTTON"), "登入".into(), 0),
            (304, w!("STATIC"), "來源伺服器".into(), 0),
            (CHARACTER_PREV, w!("BUTTON"), "上一頁".into(), 0),
            (CHARACTER_NEXT, w!("BUTTON"), "下一頁".into(), 0),
            (
                SOURCES,
                w!("COMBOBOX"),
                String::new(),
                CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0,
            ),
            (SELECT, w!("BUTTON"), "選擇來源".into(), 0),
            (CANCEL, w!("BUTTON"), "中斷連線".into(), 0),
            (GAME, w!("BUTTON"), "取得角色".into(), 0),
            (ACK, w!("BUTTON"), "確認通知".into(), 0),
            (RETURN, w!("BUTTON"), "返回選角".into(), 0),
            (REFRESH, w!("BUTTON"), "更新房間".into(), 0),
            (305, w!("STATIC"), "大廳房間".into(), 0),
            (
                ROOMS,
                w!("LISTBOX"),
                String::new(),
                WS_VSCROLL.0 | LBS_NOINTEGRALHEIGHT as u32 | LBS_NOTIFY as u32,
            ),
            (306, w!("STATIC"), "房間密碼".into(), 0),
            (
                ROOM_PASSWORD,
                w!("EDIT"),
                String::new(),
                (ES_PASSWORD | ES_AUTOHSCROLL) as u32,
            ),
            (JOIN, w!("BUTTON"), "進入房間".into(), 0),
            (READY, w!("BUTTON"), "準備".into(), 0),
            (UNREADY, w!("BUTTON"), "取消準備".into(), 0),
            (CHANGE_FORCE, w!("BUTTON"), "切換隊伍".into(), 0),
            (EXIT_ROOM, w!("BUTTON"), "離開房間".into(), 0),
            (CREATE_ROOM,w!("BUTTON"),"建立房間".into(),0),
            (EDIT_ROOM,w!("BUTTON"),"房間設定".into(),0),
            (OBSERVER,w!("BUTTON"),"開啟觀戰".into(),0),
            (KICK_MEMBER,w!("BUTTON"),"移出選取成員".into(),0),
            (MODE_PREV,w!("BUTTON"),"上一模式".into(),0),
            (MODE_NEXT,w!("BUTTON"),"下一模式".into(),0),
            (MAP_PREV,w!("BUTTON"),"上一地圖".into(),0),
            (MAP_NEXT,w!("BUTTON"),"下一地圖".into(),0),
            (ROOM_HELP,w!("STATIC"),"選取下方房間後加入，或建立新房間。\r\n房主可修改設定、切換模式／地圖及移出成員。\r\n切換模式／地圖前，全部成員須先取消準備。".into(),0),
            (314,w!("STATIC"),"房間名稱".into(),0),
            (ROOM_NAME,w!("EDIT"),String::new(),ES_AUTOHSCROLL as u32),
            (315,w!("STATIC"),"人數上限".into(),0),
            (ROOM_CAPACITY,w!("EDIT"),"8".into(),(ES_NUMBER|ES_AUTOHSCROLL) as u32),
            (316,w!("STATIC"),"新密碼".into(),0),
            (ROOM_SECRET,w!("EDIT"),String::new(),(ES_PASSWORD|ES_AUTOHSCROLL) as u32),
            (ROOM_LOCK,w!("BUTTON"),"設定密碼（修改時需重填）".into(),BS_AUTOCHECKBOX as u32),
            (SAVE_ROOM,w!("BUTTON"),"送出設定".into(),0),
            (DISCARD_ROOM,w!("BUTTON"),"取消編輯".into(),0),
            (SKILL_LABEL, w!("STATIC"), "技能".into(), 0),
            (
                SKILLS,
                w!("LISTBOX"),
                String::new(),
                WS_VSCROLL.0 | LBS_OWNERDRAWFIXED as u32 | LBS_HASSTRINGS as u32
                    | LBS_NOINTEGRALHEIGHT as u32 | LBS_NOTIFY as u32,
            ),
            (CAST_SKILL, w!("BUTTON"), "施放技能".into(), 0),
            (SUMMON_OPTIONS,w!("COMBOBOX"),String::new(),CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
            (skill_ui::BOOK, w!("BUTTON"), "技能書".into(), 0),
            (skill_ui::FIRST + 0, w!("BUTTON"), "F5".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 1, w!("BUTTON"), "F6".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 2, w!("BUTTON"), "F7".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 3, w!("BUTTON"), "F8".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 4, w!("BUTTON"), "F9".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 5, w!("BUTTON"), "F10".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 6, w!("BUTTON"), "F11".into(), BS_OWNERDRAW as u32),
            (skill_ui::FIRST + 7, w!("BUTTON"), "F12".into(), BS_OWNERDRAW as u32),
            (TELEPORT_DEST, w!("COMBOBOX"), String::new(), CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
            (INVENTORY_TOGGLE, w!("BUTTON"), "背包".into(), 0),
            (GROUND_TOGGLE, w!("BUTTON"), "附近物品".into(), 0),
            (ITEM_USE, w!("BUTTON"), "使用／裝卸".into(), 0),
            (ITEM_DROP, w!("BUTTON"), "丟棄".into(), 0),
            (ITEM_DESTROY, w!("BUTTON"), "銷毀整疊".into(), 0),
            (ITEM_OPTION_SEND, w!("BUTTON"), "確認選項".into(), 0),
            (ITEM_OPTION_CANCEL, w!("BUTTON"), "取消選項".into(), 0),
            (ITEM_ARGUMENT, w!("EDIT"), String::new(), ES_AUTOHSCROLL as u32),
            (ITEM_SUBJECT, w!("EDIT"), String::new(), ES_AUTOHSCROLL as u32),
            (ITEM_CONTENT, w!("EDIT"), String::new(), ES_MULTILINE as u32 | ES_AUTOVSCROLL as u32 | WS_VSCROLL.0),
            (ITEM_LETTER_HELP, w!("STATIC"), "收件人（下方欄位）／標題（20 字元）／內文（256 字元）".into(), 0),
            (ITEM_MAIL, w!("EDIT"), String::new(), ES_MULTILINE as u32 | ES_READONLY as u32 | ES_AUTOVSCROLL as u32 | WS_VSCROLL.0),
            (
                ITEM_QUANTITY,
                w!("EDIT"),
                "1".into(),
                ES_NUMBER as u32 | ES_AUTOHSCROLL as u32,
            ),
            (ITEM_QUANTITY_LABEL, w!("STATIC"), "數量".into(), 0),
            (ITEM_FEEDBACK, w!("STATIC"), String::new(), 0),
            (
                INVENTORY_LIST,
                w!("LISTBOX"),
                String::new(),
                WS_VSCROLL.0
                    | WS_HSCROLL.0
                    | LBS_NOINTEGRALHEIGHT as u32
                    | LBS_OWNERDRAWFIXED as u32
                    | LBS_HASSTRINGS as u32
                    | LBS_NOTIFY as u32,
            ),
            (STATUS, w!("EDIT"), "請連線後輸入帳密。".into(),
                ES_MULTILINE as u32 | ES_READONLY as u32 | ES_AUTOVSCROLL as u32 | WS_VSCROLL.0),
            (
                SCORES,
                w!("LISTBOX"),
                String::new(),
                WS_VSCROLL.0 | WS_HSCROLL.0 | LBS_NOINTEGRALHEIGHT as u32,
            ),
        ] {
            let text = wide(&label);
            let interactive = id < 300 && id != STATUS;
            let style = WS_CHILD
                | WS_VISIBLE
                | WINDOW_STYLE(extra)
                | if interactive {
                    WS_TABSTOP
                } else {
                    WINDOW_STYLE(0)
                }
                | if matches!(id, ACCOUNT | PASSWORD | ROOM_PASSWORD | ROOM_NAME | ROOM_CAPACITY | ROOM_SECRET) {
                    WS_BORDER
                } else {
                    WINDOW_STYLE(0)
                };
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                PCWSTR(text.as_ptr()),
                style,
                0,
                0,
                0,
                0,
                Some(parent),
                Some(HMENU(id as usize as *mut _)),
                Some(instance),
                None,
            )?;
            self.controls.push((id, hwnd));
        }
        send(
            self.control(ACCOUNT),
            EM_SETLIMITTEXT,
            WPARAM(64),
            LPARAM(0),
        );
        send(
            self.control(PASSWORD),
            EM_SETLIMITTEXT,
            WPARAM(128),
            LPARAM(0),
        );
        send(
            self.control(ROOM_PASSWORD),
            EM_SETLIMITTEXT,
            WPARAM(4),
            LPARAM(0),
        );
        send(
            self.control(ITEM_QUANTITY),
            EM_SETLIMITTEXT,
            WPARAM(10),
            LPARAM(0),
        );
        for (id,max) in [(ROOM_NAME,89),(ROOM_CAPACITY,3),(ROOM_SECRET,4)] {
            send(self.control(id),EM_SETLIMITTEXT,WPARAM(max),LPARAM(0));
        }
        self.create_rental_controls()?;
        self.create_companion_controls()?;
        let ids:std::collections::BTreeSet<_>=self.controls.iter().map(|(id,_)|*id).collect();
        ensure!(ids.len()==self.controls.len(),"duplicate native child control id");
        self.layout(parent, true);
        self.refresh();
        Ok(())
    }
    unsafe fn layout(&mut self, parent: HWND, new_font: bool) {
        let dpi = GetDpiForWindow(parent).max(96);
        let px = |n: i32| n * dpi as i32 / 96;
        let mut area = RECT::default();
        let _ = GetClientRect(parent, &mut area);
        let width = (area.right - px(40)).max(px(280));
        let rows = [
            (CREATE_ROOM,20,20,px(130),32),
            (EDIT_ROOM,170,20,px(130),32),
            (OBSERVER,320,20,px(130),32),
            (KICK_MEMBER,20,62,px(130),32),
            (MODE_PREV,170,62,px(130),32),
            (MODE_NEXT,320,62,px(130),32),
            (MAP_PREV,20,104,px(130),32),
            (MAP_NEXT,170,104,px(130),32),
            (ROOM_HELP,20,157,width,108),
            (314,20,114,px(85),25),
            (ROOM_NAME,110,110,width-px(90),28),
            (315,20,154,px(85),25),
            (ROOM_CAPACITY,110,150,px(70),28),
            (ROOM_LOCK,20,190,width,28),
            (316,20,230,px(85),25),
            (ROOM_SECRET,110,226,px(130),28),
            (SAVE_ROOM,20,270,px(130),32),
            (DISCARD_ROOM,170,270,px(130),32),
            (301, 20, 20, width, 25),
            (302, 20, 62, px(60), 25),
            (ACCOUNT, 90, 58, width - px(70), 28),
            (303, 20, 102, px(60), 25),
            (PASSWORD, 90, 98, width - px(70), 28),
            (CONNECT, 90, 142, px(110), 32),
            (LOGIN, 220, 142, px(110), 32),
            (INVENTORY_TOGGLE, 350, 142, px(120), 32),
            (304, 20, 198, px(290), 25),
            (CHARACTER_PREV, 320, 192, px(80), 30),
            (CHARACTER_NEXT, 410, 192, px(80), 30),
            (SOURCES, 20, 228, width, 150),
            (SELECT, 20, 274, px(130), 32),
            (CANCEL, 170, 274, px(130), 32),
            (GAME, 320, 274, px(130), 32),
            (ACK, 20, 316, px(130), 32),
            (RETURN, 170, 316, px(130), 32),
            (REFRESH, 320, 316, px(130), 32),
            (STATUS, 20, 360, width, 110),
            (305, 20, 476, width, 25),
            (ROOMS, 20, 505, width, 125),
            (INVENTORY_LIST, 20, 505, width, 125),
            (ITEM_USE, 20, 640, px(100), 32),
            (ITEM_QUANTITY_LABEL, 130, 644, px(32), 25),
            (ITEM_QUANTITY, 166, 640, px(60), 28),
            (ITEM_DROP, 240, 640, px(100), 32),
            (ITEM_DESTROY, 350, 640, px(110), 32),
            (ITEM_LETTER_HELP, 20, 290, width, 25),
            (ITEM_SUBJECT, 20, 320, width, 28),
            (ITEM_CONTENT, 20, 352, width, 108),
            (ITEM_MAIL, 20, 290, width, 170),
            (ITEM_ARGUMENT, 20, 466, px(220), 28),
            (ITEM_OPTION_SEND, 250, 466, px(100), 28),
            (ITEM_OPTION_CANCEL, 360, 466, px(100), 28),
            (ITEM_FEEDBACK, 20, 678, width, 36),
            (306, 20, 642, px(80), 25),
            (ROOM_PASSWORD, 105, 640, px(90), 28),
            (JOIN, 210, 638, px(130), 32),
            (CHANGE_FORCE, 350, 638, px(100), 32),
            (READY, 20, 682, px(100), 32),
            (UNREADY, 135, 682, px(130), 32),
            (EXIT_ROOM, 280, 682, px(130), 32),
        ];
        for (id, x, y, w, h) in rows {
            let hwnd = self.control(id);
            if !hwnd.is_invalid() {
                let _ = MoveWindow(hwnd, px(x), px(y), w, px(h), true);
            }
        }
        if new_font {
            let font = CreateFontW(
                -px(16),
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
                w!("Microsoft JhengHei UI"),
            );
            if !font.is_invalid() {
                for (_, hwnd) in &self.controls {
                    send(*hwnd, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
                }
                if !self.font.is_invalid() {
                    let _ = DeleteObject(self.font.into());
                }
                self.font = font;
            }
        }
        for (id, hwnd) in &self.controls {
            let visible = match *id {
                CHARACTER_PREV | CHARACTER_NEXT => matches!(self.controller.snapshot().phase,
                    LoginPhase::CharactersReady | LoginPhase::CharacterPending)
                    && !self.world_view && !self.inventory_view && !self.room_tools_view,
                MAP_PREV|MAP_NEXT => self.room_tools_view && self.room_editor.is_none() && !self.inventory_view,
                CREATE_ROOM|EDIT_ROOM|OBSERVER|KICK_MEMBER|MODE_PREV|MODE_NEXT => self.room_tools_view && !self.inventory_view,
                ROOM_NAME|ROOM_CAPACITY|ROOM_SECRET|ROOM_LOCK|SAVE_ROOM|DISCARD_ROOM|314|315|316 => self.room_tools_view && self.room_editor.is_some() && !self.inventory_view,
                ROOM_HELP => self.room_tools_view && self.room_editor.is_none() && !self.inventory_view,
                301|302|303|304|ACCOUNT|PASSWORD|CONNECT|LOGIN|SOURCES|SELECT|GAME|ACK if self.room_tools_view => false,
                CANCEL | RETURN if self.world_view && self.inventory_view => false,
                ITEM_DROP if self.ground_view => false,
                ITEM_SUBJECT | ITEM_CONTENT | ITEM_LETTER_HELP => self.inventory_view && !self.ground_view
                    && self.item_selection.as_ref().is_some_and(|s| s.source.item.use_type == 12),
                ITEM_MAIL => self.inventory_view && !self.ground_view && self.item_selection.is_none()
                    && self.displayed_mail.is_some() && self.controller.snapshot().inventory.mail.is_some(),
                ITEM_DESTROY => self.inventory_view && !self.ground_view,
                ITEM_ARGUMENT | ITEM_OPTION_SEND | ITEM_OPTION_CANCEL => self.inventory_view
                    && !self.ground_view && (self.item_selection.is_some() || self.destroy_choice.is_some()),
                GROUND_TOGGLE => self.world_view && !self.result_view,
                INVENTORY_LIST | ITEM_USE | ITEM_DROP | ITEM_QUANTITY | ITEM_FEEDBACK
                | ITEM_QUANTITY_LABEL => self.inventory_view,
                JOIN | ROOM_PASSWORD | READY | UNREADY | CHANGE_FORCE | EXIT_ROOM | 306 if self.inventory_view => {
                    false
                }
                INVENTORY_TOGGLE => true,
                ROOMS => !self.world_view && !self.inventory_view,
                SKILL_LABEL => self.world_view && !self.result_view && !self.ground_view && !(self.rental.open || self.companions.open),
                SKILLS | CAST_SKILL => self.world_view && !self.result_view && !self.ground_view,
                SUMMON_OPTIONS => self.world_view && !self.result_view && !self.inventory_view && !self.rental.open && self.skills.selected()==Some(51),
                TELEPORT_DEST => self.world_view && !self.result_view && !self.inventory_view && self.skills.selected()==Some(5),
                SCORES => self.result_view && !self.inventory_view,
                _ => !self.world_view || matches!(*id, CANCEL | STATUS | RETURN),
            };
            let _ = ShowWindow(*hwnd, if visible { SW_SHOW } else { SW_HIDE });
        }
        self.text(
            RETURN,
            if self.world_view && self.controller.snapshot().phase.active_world() {
                "退出戰鬥"
            } else if self.world_view {
                "返回大廳"
            } else {
                "返回選角"
            },
        );
        if self.room_tools_view && !self.world_view {
            let _=MoveWindow(self.control(CANCEL),px(20),px(316),px(130),px(32),true);
            let _=MoveWindow(self.control(INVENTORY_TOGGLE),px(320),px(270),px(130),px(32),true);
        }
        if self.world_view {
            let _ = MoveWindow(
                self.control(GROUND_TOGGLE),
                px(460),
                area.bottom - px(45),
                px(120),
                px(32),
                true,
            );
            let _ = MoveWindow(
                self.control(INVENTORY_TOGGLE),
                px(310),
                area.bottom - px(45),
                px(140),
                px(32),
                true,
            );
            for (id, x, w) in [
                (ITEM_USE, 16, 100),
                (ITEM_QUANTITY_LABEL, 124, 32),
                (ITEM_QUANTITY, 158, 50),
                (ITEM_DROP, 218, 84),
            ] {
                let _ = MoveWindow(
                    self.control(id),
                    px(x),
                    area.bottom - px(45),
                    px(w),
                    px(32),
                    true,
                );
            }
            let _ = MoveWindow(
                self.control(ITEM_FEEDBACK),
                px(16),
                area.bottom - px(if self.inventory_view && !self.ground_view {240} else {200}),
                area.right - px(32),
                px(22),
                true,
            );
            for (id, y, h) in [(ITEM_LETTER_HELP,420,24),(ITEM_SUBJECT,392,28),(ITEM_CONTENT,360,112),(ITEM_MAIL,420,172)] {
                let _ = MoveWindow(self.control(id),px(16),area.bottom-px(y),area.right-px(32),px(h),true);
            }
            for (id, x, width) in [(ITEM_ARGUMENT,16,220),(ITEM_OPTION_SEND,246,110),
                (ITEM_OPTION_CANCEL,366,110),(ITEM_DESTROY,486,110)] {
                let _ = MoveWindow(self.control(id),px(x),area.bottom-px(240),px(width),px(30),true);
            }
            let _ = MoveWindow(
                self.control(INVENTORY_LIST),
                px(16),
                area.bottom - px(if self.inventory_view && !self.ground_view {215} else {175}),
                area.right - px(32),
                px(if self.inventory_view && !self.ground_view {110} else {120}),
                true,
            );
            let _ = MoveWindow(
                self.control(SKILL_LABEL),
                px(16),
                area.bottom - px(86),
                px(40),
                px(25),
                true,
            );
            let _ = MoveWindow(
                self.control(SKILLS),
                px(64),
                area.bottom - px(90),
                (area.right - px(208)).max(px(80)),
                px(220),
                true,
            );
            let _ = MoveWindow(
                self.control(CAST_SKILL),
                (area.right - px(132)).max(px(100)),
                area.bottom - px(90),
                px(116),
                px(32),
                true,
            );
            // Above the F5–F12 hotbar (bottom-155..-97); a closed combo is ~24px.
            let _ = MoveWindow(self.control(SUMMON_OPTIONS),px(16),area.bottom-px(190),area.right-px(32),px(300),true);
            let _ = MoveWindow(self.control(TELEPORT_DEST),px(16),area.bottom-px(190),area.right-px(32),px(220),true);
            let _ = MoveWindow(
                self.control(SCORES),
                px(16),
                area.bottom - px(200),
                area.right - px(32),
                px(145),
                true,
            );
            let _ = MoveWindow(
                self.control(RETURN),
                px(160),
                area.bottom - px(45),
                px(130),
                px(32),
                true,
            );
            let _ = MoveWindow(
                self.control(STATUS),
                px(16),
                area.bottom
                    // World rows: book/cast -90, hotbar -155, teleport -190.
                    - px(if self.inventory_view { if self.ground_view {286} else {326} } else if self.result_view { 250 } else if self.skills.selected()==Some(5) {290} else { 250 }),
                area.right - px(32),
                px(if self.inventory_view { 80 } else if self.result_view { 45 } else { 90 }),
                true,
            );
            let _ = MoveWindow(
                self.control(CANCEL),
                px(16),
                area.bottom - px(45),
                px(130),
                px(32),
                true,
            );
        }
        send(
            self.control(INVENTORY_LIST),
            LB_SETITEMHEIGHT,
            WPARAM(0),
            LPARAM(px(40) as isize),
        );
        self.score_extent();
        self.list_extent(
            INVENTORY_LIST,
            self.inventory_rows.iter().map(|r| r.1.as_str()),
        );
        self.layout_rental();
        self.layout_skill_book(area, dpi);
        self.layout_companions();
    }
    unsafe fn score_extent(&self) {
        self.list_extent(SCORES, self.score_labels.iter().map(String::as_str));
    }
    unsafe fn draw_inventory(&self, draw: &DRAWITEMSTRUCT) {
        let selected = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let background = GetSysColor(if selected {
            COLOR_HIGHLIGHT
        } else {
            COLOR_WINDOW
        });
        FillRect(
            draw.hDC,
            &draw.rcItem,
            GetSysColorBrush(if selected {
                COLOR_HIGHLIGHT
            } else {
                COLOR_WINDOW
            }),
        );
        if self.inventory_owner != self.controller.snapshot().inventory.owner {
            return;
        }
        let Some((id, label)) = self.inventory_rows.get(draw.itemID as usize) else {
            return;
        };
        let dpi = GetDpiForWindow(self.parent).max(96) as i32;
        let px = |v: i32| v * dpi / 96;
        if let Some(item) = self
            .controller
            .snapshot()
            .inventory
            .items
            .get(id)
            .filter(|_| !self.ground_view)
        {
            if let Some(icon) = self.inventory_icons.icon(item.icon) {
                let b = background;
                let pixels = icon.bgra([b as u8, (b >> 8) as u8, (b >> 16) as u8]);
                let mut info = BITMAPINFO::default();
                info.bmiHeader = BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 32,
                    biHeight: -32,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                };
                let previous = SetStretchBltMode(draw.hDC, COLORONCOLOR);
                StretchDIBits(
                    draw.hDC,
                    draw.rcItem.left + px(4),
                    draw.rcItem.top + px(4),
                    px(32),
                    px(32),
                    0,
                    0,
                    32,
                    32,
                    Some(pixels.as_ptr().cast()),
                    &info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                SetStretchBltMode(draw.hDC, STRETCH_BLT_MODE(previous));
            }
        }
        let old_font = SelectObject(draw.hDC, self.font.into());
        let old_color = SetTextColor(
            draw.hDC,
            COLORREF(GetSysColor(if selected {
                COLOR_HIGHLIGHTTEXT
            } else {
                COLOR_WINDOWTEXT
            })),
        );
        let old_mode = SetBkMode(draw.hDC, TRANSPARENT);
        let mut text: Vec<u16> = label.encode_utf16().collect();
        let mut area = draw.rcItem;
        area.left += px(if self.ground_view { 4 } else { 44 });
        area.right -= px(4);
        DrawTextW(
            draw.hDC,
            &mut text,
            &mut area,
            DT_SINGLELINE | DT_VCENTER | DT_NOPREFIX,
        );
        SetBkMode(draw.hDC, BACKGROUND_MODE(old_mode as u32));
        SetTextColor(draw.hDC, old_color);
        SelectObject(draw.hDC, old_font);
        if draw.itemState.0 & ODS_FOCUS.0 != 0 {
            let _ = DrawFocusRect(draw.hDC, &draw.rcItem);
        }
    }
    unsafe fn list_extent<'a>(&self, id: u16, labels: impl Iterator<Item = &'a str>) {
        let hwnd = self.control(id);
        let dc = GetDC(Some(hwnd));
        if dc.0.is_null() {
            return;
        }
        let old = SelectObject(dc, self.font.into());
        let mut width = 0;
        for label in labels {
            let text: Vec<u16> = label.encode_utf16().collect();
            let mut size = SIZE::default();
            if GetTextExtentPoint32W(dc, &text, &mut size).as_bool() {
                width = width.max(size.cx);
            }
        }
        SelectObject(dc, old);
        ReleaseDC(Some(hwnd), dc);
        send(
            hwnd,
            LB_SETHORIZONTALEXTENT,
            WPARAM(
                (width
                    + (if id == INVENTORY_LIST { 64 } else { 24 })
                        * GetDpiForWindow(hwnd).max(96) as i32
                        / 96) as usize,
            ),
            LPARAM(0),
        );
    }
    unsafe fn world_tick(&mut self, parent: HWND) {
        // S5E is rendered only under the current inventory owner, once per reply.
        let inventory = &self.controller.snapshot().inventory;
        if let (Some(owner), Some(mail)) = (&inventory.owner, &inventory.mail) {
            let key = (owner.clone(), inventory.mail_revision);
            if self.displayed_mail.as_ref() != Some(&key) {
                let text = format!("寄件人：{}\r\n收件人：{}\r\n日期：{}\r\n標題：{}\r\n\r\n{}",
                    mail.sender, mail.receiver, mail.date, mail.subject, mail.content);
                self.displayed_mail = Some(key);
                self.text(ITEM_MAIL, &text);
                self.layout(parent, false);
            }
        } else { self.displayed_mail = None; }
        let mut options_changed = false;
        if self.item_selection.as_ref().is_some_and(|c| c.validate(self.controller.snapshot()).is_err()) || self.closing {
            options_changed |= self.item_selection.take().is_some();
        }
        if self.destroy_choice.as_ref().is_some_and(|c| c.validate(self.controller.snapshot()).is_err()) || self.closing {
            options_changed |= self.destroy_choice.take().is_some();
        }
        if options_changed { self.layout(parent, false); }
        if self.item_cursor.as_ref().is_some_and(|c| c.validate(self.controller.snapshot()).is_err())
            || self.closing {
            self.item_cursor = None;
        }
        let active = matches!(
            self.controller.snapshot().phase,
            LoginPhase::BattlePending | LoginPhase::WorldEntering | LoginPhase::WorldReady | LoginPhase::BattleResult | LoginPhase::ResultReturning
        );
        let stamp = if active && !self.closing {
            Some(super::native_navigation::InputStamp::capture(
                self.controller.transport().snapshot().epoch,
                &self.controller.snapshot().world,
            ))
        } else {
            None
        };
        self.held.synchronize_action(&self.controller.snapshot().world);
        if self.held.synchronize(stamp) {
            self.held_attack.clear();
            self.target = None;
            self.ground = None;
            self.painted = None;self.painted_input = None;
            if let Err(e)=self.voices.stop_all() {self.audio_error=Some(e.to_string());}
            self.audio_frame = None;
        }
        if !self.controller.snapshot().world.movement_blocks.is_empty() {
            self.held.clear();
            self.held_attack.clear();
        }

        let result_view = matches!(
            self.controller.snapshot().phase,
            LoginPhase::BattleResult | LoginPhase::ResultReturning
        );
        if (self.world_view != active || self.result_view != result_view) && !self.closing {
            self.world_view = active;
            self.result_view = result_view;
            self.layout(parent, false);
            let _ = InvalidateRect(Some(parent), None, true);
        }
        // Hover is sampled against the actual painted viewport, including when
        // the pointer leaves this window or a child control receives input.
        let hover = (|| {
            if !active || self.closing || self.inventory_view || self.rental.open || self.companions.open || GetForegroundWindow() != parent {
                return None;
            }
            let mut point = POINT::default();
            if GetCursorPos(&mut point).is_err() || WindowFromPoint(point) != parent {
                return None;
            }
            if !ScreenToClient(parent, &mut point).as_bool() {
                return None;
            }
            let mut area = RECT::default();
            if GetClientRect(parent, &mut area).is_err() {
                return None;
            }
            let dpi = GetDpiForWindow(parent).max(96) as i32;
            let view = super::compositor::Viewport::fit(
                area.right,
                area.bottom - (if self.result_view { 260 } else { 250 }) * dpi / 96,
                800,
                600,
            )?;
            let (epoch,surface)=self.painted_input.as_ref()?;
            let world=&self.controller.snapshot().world;
            if *epoch!=self.controller.transport().snapshot().epoch || !surface.input_ready(world)
                || self.painted.is_none_or(|(_,_,painted_view)|painted_view!=view){return None;}
            let (x,y)=view.canvas_point(point.x,point.y,800,600)?;
            let id=surface.pick_current(world,x,y,world.status?.object_id)?;
            super::native_world_labels::Hover::capture(world, id)
        })();
        self.world_presentation.hover = hover;
        let epoch=self.controller.transport().snapshot().epoch;
        let (world,environment)=self.controller.presentation_inputs();
        if self.world_presentation.tick_environment(&self.resource_root,epoch,world,environment,active && !self.closing) {
            let _ = InvalidateRect(Some(parent), None, true);
        }
        let prepared = self.world_presentation.take_static2941_ready();
        if let Err(error) = self.controller.enqueue_static2941(&self.resource_root, prepared) {
            self.world_presentation.error = Some(format!("static2941: {error:#}"));
        }
        if self.world_presentation.prune_static_frame(&self.controller.snapshot().world, epoch) {
            let _ = InvalidateRect(Some(parent), None, true);
        }
        if let Some(frame)=&self.world_presentation.frame {
            if let Err(error)=self.controller.observe_player_frame(frame) {
                self.world_presentation.error=Some(format!("player draw clock: {error:#}"));
            }
        }
        if let Err(e)=self.voices.poll() {self.audio_error=Some(e.to_string());}
        let native_scope=self.world_presentation.audio_scope(self.controller.transport().snapshot().epoch,
            self.controller.snapshot().world.generation);
        if self.audio_scope.as_ref().is_none_or(|old|!old.same_context(&native_scope)) {
            if let Err(e)=self.voices.stop_all() {self.audio_error=Some(e.to_string());}
            self.audio_frame=None;self.audio_scope=Some(native_scope.clone());
        }
        if let Some(frame) = &self.world_presentation.frame {
            let key = (
                self.controller.transport().snapshot().epoch,
                self.controller.snapshot().world.generation,
                frame.receipt["elapsed_ns"].as_u64().unwrap_or(0),
            );
            if self.audio_frame != Some(key) {
                self.audio_frame = Some(key);
                for cue in &frame.legacy_audio {
                    if self.audio_error.is_none() && GetForegroundWindow()==parent {
                        match cue.submit(&native_scope,&mut self.voices) {
                            Err(e)=>self.audio_error=Some(e.to_string()),
                            Ok(super::native_audio_delivery::DeliveryOutcome::Unbound)=>self.audio_error=Some("一般音效尚未綁定場景".into()),
                            _=>{},
                        }
                    } else {cue.discard();}
                }
                for cue in &frame.native_audio {
                    if self.audio_error.is_none() && GetForegroundWindow()==parent {
                        match cue.submit(&native_scope,&mut self.voices) {
                            Err(e)=>self.audio_error=Some(e.to_string()),
                            Ok(super::native_audio_delivery::DeliveryOutcome::Unbound)=>self.audio_error=Some("音效尚未綁定場景".into()),
                            _=>{},
                        }
                    } else {cue.discard();}
                }
            }
        }
    }
    unsafe fn poll_presented(&mut self,parent:HWND) {
        let input_active=!self.closing && self.world_view && !self.inventory_view && !self.rental.open
            && GetForegroundWindow()==parent;
        let before=self.world_presentation.frame.as_ref().and_then(|f|f.receipt["elapsed_ns"].as_u64());
        self.controller.player_input(if input_active {self.held.direction()} else {None},!input_active);
        self.controller.poll_presented(&self.resource_root,&mut self.world_presentation);
        let after=self.world_presentation.frame.as_ref().and_then(|f|f.receipt["elapsed_ns"].as_u64());
        if before!=after {let _=InvalidateRect(Some(parent),None,true);}
    }
    unsafe fn combat_input(&self) -> super::native_forced_combat::Input {
        super::native_forced_combat::Input {
            foreground: self.world_view && !self.closing
                && GetForegroundWindow() == self.parent && IsWindowEnabled(self.parent).as_bool(),
            ctrl: GetAsyncKeyState(i32::from(VK_CONTROL.0)) < 0,
        }
    }
    unsafe fn cast_skill(&mut self, skill: u16) -> Result<()> {
        let input = self.combat_input();
        self.held.clear();
        self.held_attack.clear();
        if skill==51 {
            ensure!(!self.inventory_view && !self.rental.open && !self.closing,"召喚畫面已失效");
            ensure!(self.skills.selected_skill(self.controller.transport().snapshot().epoch,&self.controller.snapshot().world)?==51,"召喚選擇已失效");
            let choice=self.summon_choice.as_ref().context("請先選擇召喚種類")?;
            let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;
            return self.controller.cast_summon(choice,frame);
        }
        if super::native_skills::item_targeted(skill) {
            let epoch = self.controller.transport().snapshot().epoch;
            ensure!(self.skills.selected_skill(epoch, &self.controller.snapshot().world)? == skill, "技能已變更");
            if super::native_skills::weapon_bound(skill) {
                // 12／107：正式伺服器以裝備中的武器為目標，不讓玩家另選背包物品。
                ensure!(!self.ground_view && !self.closing, "目前不能施法");
                let item = super::native_skills::ItemTarget::equipped_weapon(self.controller.snapshot(), epoch)?;
                let frame = self.world_presentation.frame.as_ref().context("等待世界畫面")?;
                return self.controller.cast_item(skill, &item, frame);
            }
            ensure!(self.inventory_view && !self.ground_view && !self.closing, "請開啟背包並選取施法物品");
            let item = self.skill_item_target.as_ref().context("請重新選取施法物品")?;
            let frame = self.world_presentation.frame.as_ref().context("等待世界畫面")?;
            return self.controller.cast_item(skill, item, frame);
        }
        if skill==5 {
            ensure!(!self.inventory_view && !self.closing, "請回到世界畫面選擇傳送目的地");
            let choice=self.teleport_choice.as_ref().context("請選擇傳送目的地")?;
            let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;
            return self.controller.cast_teleport(choice,frame);
        }
        // Hotbar slots and the book share one ID-keyed admission path.
        let choice = self.skills.choice_id_at(
            self.controller.transport().snapshot().epoch,
            &self.controller.snapshot().world, skill, self.target, self.ground)?;
        let frame = self
            .world_presentation
            .frame
            .as_ref()
            .context("等待世界畫面")?;
        match choice.ground {
            Some(position) => self.controller.cast_ground(choice.skill, position, frame),
            None => self.controller.cast_selected_with_input(choice.skill, self.target, frame, input),
        }
    }
    fn enter_nearby_portal(&mut self)->Result<()> {
        self.held.clear();self.held_attack.clear();
        let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;
        let choice=super::native_portal::EntryChoice::nearest(self.controller.transport().snapshot().epoch,&self.controller.snapshot().world,&frame.navigation)?;
        self.controller.enter_map_portal(choice,frame)
    }
    unsafe fn attack_target(&mut self) -> Result<()> {
        let Some(target) = self
            .target
            .and_then(|t| t.resolve(&self.controller.snapshot().world))
        else {
            // No selection: native target-zero swing in the current heading.
            self.held.clear();
            self.held_attack.clear();
            let frame = self.world_presentation.frame.as_ref().context("等待世界畫面")?;
            ensure!(frame.movement_ready(&self.controller.snapshot().world), "等待目前動作完成");
            let heading = self.controller.snapshot().world.own_actor().context("角色尚未就緒")?.heading;
            let input = self.combat_input();
            return self.controller.empty_attack_with_input(heading, None, input);
        };
        self.held.clear();
        ensure!(
            self.controller.snapshot().attack_feedback != super::native_combat::Feedback::Pending,
            "等待攻擊回覆中"
        );
        let epoch = self.controller.transport().snapshot().epoch;
        if self.controller.snapshot().world.attack_out_of_reach(target) {
            self.held_attack.press_chase(target, epoch, &self.controller.snapshot().world);
            self.input_tick();
            return Ok(());
        }
        self.controller.snapshot().world.attack_request(target)?;
        self.held_attack.press(target, epoch, &self.controller.snapshot().world);
        self.input_tick();
        Ok(())
    }
    unsafe fn start_chase(&mut self, id: u32) {
        let epoch = self.controller.transport().snapshot().epoch;
        self.held_attack.press_chase(id, epoch, &self.controller.snapshot().world);
        self.input_tick();
    }
    unsafe fn toggle_selected_door(&mut self)->Result<()> {
        let target=self.target.and_then(|t|t.resolve(&self.controller.snapshot().world)).context("請先點選門")?;
        let choice=super::native_doors::Choice::capture(self.controller.transport().snapshot().epoch,&self.controller.snapshot().world,target)?;
        let frame=self.world_presentation.frame.as_ref().context("等待世界畫面")?;
        self.held.clear();self.held_attack.clear();
        self.controller.toggle_door(choice,frame)
    }
    unsafe fn input_tick(&mut self) {
        if !self.world_view
            || self.closing
            || !self.controller.snapshot().phase.active_world()
        {
            self.held.clear();
            self.held_attack.clear();
            return;
        }
        if self.controller.snapshot().world.death_confirmation_pending {
            self.held.clear();
            self.held_attack.clear();
            if let Some(frame) = &self.world_presentation.frame {
                if frame.death_ready(&self.controller.snapshot().world) {
                    if let Err(e) = self.controller.confirm_death(frame) {
                        self.text(STATUS, &format!("{e:#}"));
                    }
                }
            }
            return;
        }
        if self.inventory_view || self.rental.open || self.companions.open {
            self.held.clear();
            self.held_attack.clear();
            return;
        }
        let input = self.combat_input();
        if let Some(frame) = self.world_presentation.frame.as_ref() {
            if let Err(error) = self
                .controller
                .continue_attack_with_input(&mut self.held_attack, frame, input)
            {
                self.text(STATUS, &format!("{error:#}"));
            }
        }
        if self.held.direction().is_none() && self.held_attack.chasing() {
            if let Some(frame) = self.world_presentation.frame.as_ref() {
                let epoch = self.controller.transport().snapshot().epoch;
                let heading = if frame.movement_ready(&self.controller.snapshot().world) {
                    let feedback = self.controller.snapshot().attack_feedback;
                    let world = &self.controller.snapshot().world;
                    self.held_attack.chase_heading(epoch, world, &frame.navigation, feedback)
                } else { None };
                if let Some(heading) = heading {
                    match self.controller.move_presented(heading, frame) {
                        Ok(()) => self.held_attack.chase_sent(),
                        Err(error) => {
                            self.held_attack.clear();
                            self.text(STATUS, &format!("{error:#}"));
                        }
                    }
                }
            }
        }
        if let Err(error) = self.controller.continue_movement(
            &mut self.held, self.world_presentation.frame.as_ref(),
        ) {
            self.held_attack.clear();
            self.text(STATUS, &format!("{error:#}"));
        }
    }
    unsafe fn paint_world(&mut self, parent: HWND) {
        self.painted = None;self.painted_input = None;
        let mut paint = PAINTSTRUCT::default();
        let dc = BeginPaint(parent, &mut paint);
        let mut area = RECT::default();
        let _ = GetClientRect(parent, &mut area);
        if self.rental.open || self.companions.open {
            FillRect(dc,&area,GetSysColorBrush(COLOR_BTNFACE));
            let _=EndPaint(parent,&paint);self.painted=None;self.painted_input=None;return;
        }
        FillRect(dc, &area, HBRUSH(GetStockObject(BLACK_BRUSH).0));
        if let Some(frame) = self.world_presentation.frame.as_ref().filter(|frame|
            frame.static_current(&self.controller.snapshot().world, self.controller.transport().snapshot().epoch)) {
            let dpi = GetDpiForWindow(parent).max(96) as i32;
            if let Some(view) = super::compositor::Viewport::fit(
                area.right,
                area.bottom
                    - (if self.result_view || self.inventory_view {
                        260
                    } else {
                        if self.skills.selected()==Some(5) {225} else {185}
                    }) * dpi
                        / 96,
                800,
                600,
            ) {
                let mut info = BITMAPINFO::default();
                info.bmiHeader = BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 800,
                    biHeight: -600,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                };
                let drawn = StretchDIBits(
                    dc,
                    view.x,
                    view.y,
                    view.width,
                    view.height,
                    0,
                    0,
                    800,
                    600,
                    Some(frame.bgra.as_ptr().cast()),
                    &info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                if drawn > 0 {
                    let world = &self.controller.snapshot().world;
                    if let Some((ring, [l, t, r, b])) = frame.target_ring(world, self.target.and_then(|t| t.resolve(world))) {
                        // Frame pixels → letterboxed viewport (same mapping as StretchDIBits).
                        let px = |v: i32| view.x + v * view.width / 800;
                        let py = |v: i32| view.y + v * view.height / 600;
                        let [cr, cg, cb] = ring.tone.rgb();
                        let pen = CreatePen(PS_SOLID, (2 * dpi / 96).max(1), COLORREF(u32::from(cr) | u32::from(cg) << 8 | u32::from(cb) << 16));
                        if !pen.is_invalid() {
                            let old_pen = SelectObject(dc, pen.into());
                            let old_brush = SelectObject(dc, GetStockObject(NULL_BRUSH));
                            let _ = Ellipse(dc, px(l), py(t), px(r), py(b));
                            SelectObject(dc, old_brush);
                            SelectObject(dc, old_pen);
                            let _ = DeleteObject(pen.into());
                        }
                    }
                    self.painted = Some((
                        frame.receipt["elapsed_ns"].as_u64().unwrap_or(0),
                        frame.receipt["visual_revision"].as_u64().unwrap_or(0),view));
                    self.painted_input = frame.input_surface(&self.controller.snapshot().world)
                        .map(|surface|(self.controller.transport().snapshot().epoch,surface));
                }
            }
        }
        let _ = EndPaint(parent, &paint);
    }
    unsafe fn refresh_skills(&mut self) {
        let labels=self.controller.teleport_labels();
        if labels!=self.teleport_rows {
            self.teleport_rows=labels;
            self.teleport_choice=None;
            send(self.control(TELEPORT_DEST),CB_RESETCONTENT,WPARAM(0),LPARAM(0));
            for label in std::iter::once("請選擇傳送目的地").chain(self.teleport_rows.iter().map(String::as_str)) {
                let label=wide(label);
                send(self.control(TELEPORT_DEST),CB_ADDSTRING,WPARAM(0),LPARAM(label.as_ptr() as isize));
            }
            send(self.control(TELEPORT_DEST),CB_SETCURSEL,WPARAM(0),LPARAM(0));
        }
        if self.skills.selected()!=Some(5) || self.inventory_view || self.closing
            || self.teleport_choice.as_ref().is_some_and(|choice| !self.controller.teleport_choice_valid(choice)) {
            self.teleport_choice=None;
            send(self.control(TELEPORT_DEST),CB_SETCURSEL,WPARAM(0),LPARAM(0));
        }
        let snapshot = self.controller.snapshot();
        let epoch = self.controller.transport().snapshot().epoch;
        let active = snapshot.phase.active_world() && !self.closing;
        if self.skills.sync(epoch, &snapshot.world, active) {
            let list = self.control(SKILLS);
            let top = send(list, LB_GETTOPINDEX, WPARAM(0), LPARAM(0)).0.max(0) as usize;
            send(list, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for row in self.skills.rows() {
                let label = wide(&row.label);
                send(list, LB_ADDSTRING, WPARAM(0), LPARAM(label.as_ptr() as isize));
            }
            let selected = self.skills.selected().and_then(|id| self.skills.rows().iter().position(|r| r.id == id))
                .unwrap_or(usize::MAX);
            send(list, LB_SETCURSEL, WPARAM(selected), LPARAM(0));
            send(list, LB_SETTOPINDEX, WPARAM(top.min(self.skills.rows().len().saturating_sub(1))), LPARAM(0));
            for id in skill_ui::FIRST..=skill_ui::LAST { let _ = InvalidateRect(Some(self.control(id)), None, true); }
        }
        if !self.inventory_view || self.ground_view || self.closing
            || self.skill_item_target.as_ref().is_some_and(|item| item.validate(snapshot, epoch).is_err()) {
            self.skill_item_target = None;
        }
        if send(self.control(SUMMON_OPTIONS),CB_GETCOUNT,WPARAM(0),LPARAM(0)).0==0 {
            for label in std::iter::once("請選擇召喚種類；無控制能力時依伺服器等級規則".to_string())
                .chain(super::native_skills::SUMMON_SELECTIONS.iter().map(|r|format!("Lv{} {}",r.required_level,r.label))) {
                let label=wide(&label);send(self.control(SUMMON_OPTIONS),CB_ADDSTRING,WPARAM(0),LPARAM(label.as_ptr() as isize));
            }
            send(self.control(SUMMON_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));
        }
        if self.skills.selected()!=Some(51) || self.inventory_view || self.rental.open || self.closing
            || self.summon_choice.as_ref().is_some_and(|c|c.validate(snapshot,epoch).is_err()) {
            self.summon_choice=None;send(self.control(SUMMON_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));
        }
        let choice_valid = if self.skills.selected()==Some(51) {
            self.summon_choice.as_ref().is_some_and(|c|super::native_skills::summon_request(snapshot,epoch,c).is_ok())
        } else if self.skills.selected().is_some_and(super::native_skills::item_targeted) {
            self.skills.selected_skill(epoch, &snapshot.world).and_then(|skill| {
                if super::native_skills::weapon_bound(skill) {
                    let item = super::native_skills::ItemTarget::equipped_weapon(snapshot, epoch)?;
                    return super::native_skills::item_request(snapshot, epoch, skill, &item);
                }
                let item = self.skill_item_target.as_ref().context("請選取背包物品")?;
                super::native_skills::item_request(snapshot, epoch, skill, item)
            }).is_ok()
        } else if self.skills.selected()==Some(5) {
            !self.inventory_view && self.skills.selected_skill(epoch,&snapshot.world).is_ok()
                && self.teleport_choice.as_ref().is_some_and(|choice| self.controller.teleport_choice_valid(choice))
        } else {
            !self.inventory_view && self.skills.choice_at(epoch, &snapshot.world, self.target, self.ground).is_ok()
        };
        let enabled = active && choice_valid && self.world_presentation.frame.as_ref()
            .is_some_and(|f| self.controller.skill_action_ready(f, std::time::Instant::now()));
        let _ = EnableWindow(self.control(CAST_SKILL), enabled);
        let _ = EnableWindow(self.control(SUMMON_OPTIONS), active && self.skills.selected()==Some(51));
        let _ = EnableWindow(self.control(TELEPORT_DEST), active && self.skills.selected()==Some(5));
        let _ = EnableWindow(self.control(SKILLS), active && !self.skills.rows().is_empty());
        self.refresh_skill_art();
    }
    unsafe fn sync_item_choice(&mut self) {
        let index = send(
            self.control(INVENTORY_LIST),
            LB_GETCURSEL,
            WPARAM(0),
            LPARAM(0),
        )
        .0;
        if self.ground_view {
            self.pickup_choice = usize::try_from(index)
                .ok()
                .and_then(|i| self.pickup_rows.get(i))
                .cloned();
            return;
        }
        self.inventory_choice = if self.inventory_view
            && self.inventory_owner == self.controller.snapshot().inventory.owner
        {
            usize::try_from(index)
                .ok()
                .and_then(|i| self.inventory_rows.get(i))
                .and_then(|r| {
                    super::native_item_actions::Choice::capture(self.controller.snapshot(), r.0)
                        .ok()
                })
        } else {
            None
        };
    }
    unsafe fn item_choice(&self) -> Result<super::native_item_actions::Choice> {
        ensure!(
            self.inventory_view
                && self.inventory_owner == self.controller.snapshot().inventory.owner,
            "請開啟目前角色的背包"
        );
        let choice = self.inventory_choice.clone().context("請先選取物品")?;
        choice.validate(self.controller.snapshot())?;
        Ok(choice)
    }
    unsafe fn refresh_item_actions(&self) {
        self.text(
            ITEM_USE,
            if self.ground_view {
                "拾取"
            } else {
                "使用／裝卸"
            },
        );
        if self.ground_view {
            let s = self.controller.snapshot();
            let ready = !self.closing
                && s.pickup_feedback != super::native_ground_pickup::Feedback::Pending
                && s.item_feedback != super::native_item_actions::Feedback::Pending
                && self
                    .pickup_choice
                    .as_ref()
                    .is_some_and(|c| c.validate(s).is_ok())
                && self
                    .world_presentation
                    .frame
                    .as_ref()
                    .is_some_and(|f| f.input_ready(&s.world) && f.movement_ready(&s.world));
            let _ = EnableWindow(self.control(ITEM_USE), ready);
            let _ = EnableWindow(self.control(ITEM_DROP), false);
            let _ = EnableWindow(self.control(ITEM_QUANTITY), ready);
            self.text(ITEM_FEEDBACK, s.pickup_feedback.label());
            return;
        }

        use super::native_item_actions::{self as actions, Action, Feedback};
        let snapshot = self.controller.snapshot();
        let choice = self.item_choice();
        let ready = !self.closing
            && snapshot.item_feedback != Feedback::Pending
            && snapshot.pickup_feedback != super::native_ground_pickup::Feedback::Pending;
        let can_use = ready
            && choice
                .as_ref()
                .is_ok_and(|c| actions::request(snapshot, c, Action::Use).is_ok()
                    || actions::TargetChoice::capture(snapshot, c.item.id).is_ok()
                    || actions::Selection::capture(snapshot, c.clone()).is_ok());
        let can_drop = ready
            && choice
                .as_ref()
                .is_ok_and(|c| actions::request(snapshot, c, Action::Drop(1)).is_ok());
        let _ = EnableWindow(self.control(ITEM_USE), can_use);
        let _ = EnableWindow(self.control(ITEM_DROP), can_drop);
        let _ = EnableWindow(self.control(ITEM_QUANTITY), can_drop);
        let _ = EnableWindow(self.control(ITEM_DESTROY), ready && choice.as_ref()
            .is_ok_and(|c| actions::request(snapshot,c,Action::Destroy).is_ok()));
        let _ = EnableWindow(self.control(ITEM_OPTION_SEND), ready && (
            self.item_selection.as_ref().is_some_and(|c| c.validate(snapshot).is_ok())
            || self.destroy_choice.as_ref().is_some_and(|c| c.validate(snapshot).is_ok())));
        let _ = EnableWindow(self.control(ITEM_ARGUMENT), self.item_selection.as_ref()
            .is_some_and(|c| actions::text_choice_type(c.source.item.use_type) || actions::letter_choice_type(c.source.item.use_type)));
        self.text(
            ITEM_FEEDBACK,
            if self.destroy_choice.is_some() {
                "銷毀會移除選定物品整疊；確認前不會送出，取消可返回"
            } else if let Some(selection) = &self.item_selection {
                if actions::inventory_target_type(selection.source.item.use_type) {
                    "請在背包選取另一件目標物品，再按確認選項"
                } else if actions::letter_choice_type(selection.source.item.use_type) {
                    "填入收件人、標題與內文後確認寄出；Esc 或取消不寄出"
                } else { "請輸入物品選項名稱，再按確認選項" }
            } else if let Some(reason) = choice.as_ref().ok().and_then(|c| actions::server_blocker(c.item.use_type)) {
                reason
            } else if snapshot.item_feedback != Feedback::Idle {
                snapshot.item_feedback.label()
            } else if choice.as_ref().is_ok_and(|c| actions::world_target_type(c.item.use_type)) {
                "使用後點選世界中的目標位置；Esc取消"
            } else if choice
                .as_ref()
                .is_ok_and(|c| !actions::default_use_type(c.item.use_type))
            {
                "使用後選取目標或輸入選項；無可用操作時請查看物品種類與剩餘次數"
            } else {
                "選取物品後使用／裝卸，或輸入要丟棄的數量"
            },
        );
    }
    unsafe fn close_room_editor(&mut self) {
        self.room_editor=None;
        self.text(ROOM_SECRET,"");
        send(self.control(ROOM_SECRET),EM_EMPTYUNDOBUFFER,WPARAM(0),LPARAM(0));
    }
    unsafe fn sync_room_member(&mut self) {
        let index=send(self.control(ROOMS),LB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
        self.room_member=usize::try_from(index).ok().and_then(|i|
            super::native_room_ui::MemberChoice::capture(self.controller.transport().snapshot().epoch,self.controller.snapshot(),i).ok());
        self.room_choice=usize::try_from(index).ok().and_then(|i|
            super::native_room_ui::RoomChoice::capture(self.controller.transport().snapshot().epoch,self.controller.snapshot(),i).ok());
    }
    unsafe fn refresh(&mut self) {
        let epoch=self.controller.transport().snapshot().epoch;
        let room_page=!self.world_view && super::native_room_ui::room_page(self.controller.snapshot());
        let mut relayout=room_page!=self.room_tools_view;
        self.room_tools_view=room_page;
        if self.room_editor.as_ref().is_some_and(|d|d.validate(epoch,self.controller.snapshot()).is_err()) || self.closing {
            relayout|=self.room_editor.is_some();self.close_room_editor();
        }
        if self.room_member.as_ref().is_some_and(|m|m.resolve(epoch,self.controller.snapshot()).is_err()) {
            self.room_member=None;send(self.control(ROOMS),LB_SETCURSEL,WPARAM(usize::MAX),LPARAM(0));
        }
        if self.room_choice.as_ref().is_some_and(|r|r.index(epoch,self.controller.snapshot()).is_err()) {
            self.room_choice=None;send(self.control(ROOMS),LB_SETCURSEL,WPARAM(usize::MAX),LPARAM(0));
        }
        if relayout {self.layout(self.parent,false);let _=InvalidateRect(Some(self.parent),None,true);}
        self.inventory_names.poll();
        let inventory = &self.controller.snapshot().inventory;
        if self.inventory_view
            && (inventory.owner.is_none()
                || self.result_view
                || (self.ground_view
                    && !self.controller.snapshot().phase.active_world()))
        {
            self.inventory_view = false;
            self.ground_view = false;
            self.pickup_choice = None;
            self.pickup_rows.clear();
            self.painted = None;self.painted_input = None;
            self.layout(self.parent, false);
            let _ = InvalidateRect(Some(self.parent), None, true);
        }
        let inventory = &self.controller.snapshot().inventory;
        let mut rows: Vec<_> = inventory
            .items
            .values()
            .map(|i| {
                (
                    i.id,
                    super::native_inventory_names::label(i, self.inventory_names.catalog()).text,
                )
            })
            .collect();
        if self.ground_view {
            let snapshot = self.controller.snapshot();
            if let Some(frame) = self
                .world_presentation
                .frame
                .as_ref()
                .filter(|f| f.input_ready(&snapshot.world))
            {
                self.pickup_rows = frame
                    .ground_items
                    .iter()
                    .filter_map(|a| {
                        super::native_ground_pickup::Choice::capture(snapshot, frame, a.id).ok()
                    })
                    .collect();
            }
            rows = self
                .pickup_rows
                .iter()
                .map(|choice| {
                    let a = &choice.item;
                    let name = self
                        .inventory_names
                        .catalog()
                        .and_then(|c| c.resolve(&a.name).ok())
                        .unwrap_or_else(|| a.name.clone());
                    let name: String = name
                        .chars()
                        .map(|c| if c.is_control() { ' ' } else { c })
                        .collect();
                    (
                        a.id,
                        format!(
                            "{} × {}（{}, {}）",
                            if name.is_empty() {
                                "未命名物品"
                            } else {
                                &name
                            },
                            a.experience,
                            a.x,
                            a.y
                        ),
                    )
                })
                .collect();
        }
        if rows.is_empty() {
            rows.push((
                0,
                if self.ground_view {
                    "三格內沒有可拾取的地面物品"
                } else if inventory.batches == 0 {
                    "尚未收到背包資料"
                } else {
                    "背包是空的"
                }
                .into(),
            ));
        }
        let owner = inventory.owner.clone();
        let enabled = owner.is_some() && !self.closing && !self.result_view;
        let label = if self.inventory_view && !self.ground_view {
            "關閉背包".into()
        } else {
            format!("背包（{}）", inventory.items.len())
        };
        self.text(INVENTORY_TOGGLE, &label);
        self.text(
            GROUND_TOGGLE,
            if self.ground_view {
                "關閉地面物品"
            } else {
                "附近物品"
            },
        );
        let _ = EnableWindow(
            self.control(GROUND_TOGGLE),
            !self.closing && super::native_ground_pickup::available(self.controller.snapshot()),
        );
        let _ = EnableWindow(self.control(INVENTORY_TOGGLE), enabled);
        if rows != self.inventory_rows || owner != self.inventory_owner {
            let hwnd = self.control(INVENTORY_LIST);
            let index = send(hwnd, LB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
            let selected = usize::try_from(index)
                .ok()
                .and_then(|i| self.inventory_rows.get(i))
                .map(|r| r.0);
            let same_owner = owner == self.inventory_owner && !self.ground_view;
            if self.ground_view {
                self.pickup_choice = None;
            }
            let top = if same_owner {
                send(hwnd, LB_GETTOPINDEX, WPARAM(0), LPARAM(0)).0.max(0) as usize
            } else {
                0
            };
            send(hwnd, LB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for (_, label) in &rows {
                let w = wide(label);
                send(hwnd, LB_ADDSTRING, WPARAM(0), LPARAM(w.as_ptr() as isize));
            }
            if same_owner {
                if let Some(index) = rows.iter().position(|r| Some(r.0) == selected) {
                    send(hwnd, LB_SETCURSEL, WPARAM(index), LPARAM(0));
                }
            }
            send(
                hwnd,
                LB_SETTOPINDEX,
                WPARAM(top.min(rows.len().saturating_sub(1))),
                LPARAM(0),
            );
            self.inventory_rows = rows;
            self.inventory_owner = owner;
            self.list_extent(
                INVENTORY_LIST,
                self.inventory_rows.iter().map(|r| r.1.as_str()),
            );
            let _ = InvalidateRect(Some(hwnd), None, true);
        }
        let icon_rows: Vec<_> = self
            .controller
            .snapshot()
            .inventory
            .items
            .values()
            .map(|i| (i.id, i.icon))
            .collect();
        let icon_changed = icon_rows != self.inventory_icon_rows;
        self.inventory_icon_rows = icon_rows;
        let list = self.control(INVENTORY_LIST);
        let visible_ids: Vec<u16> = if self.inventory_view && !self.ground_view && !self.closing {
            let top = send(list, LB_GETTOPINDEX, WPARAM(0), LPARAM(0)).0.max(0) as usize;
            let height = send(list, LB_GETITEMHEIGHT, WPARAM(0), LPARAM(0)).0.max(1) as i32;
            let mut rect = RECT::default();
            let _ = GetClientRect(list, &mut rect);
            self.inventory_icon_rows
                .iter()
                .skip(top)
                .take(((rect.bottom.max(0) / height) + 2).min(32) as usize)
                .map(|r| r.1)
                .collect()
        } else {
            vec![]
        };
        if self.inventory_icons.tick(&visible_ids) || icon_changed {
            let _ = InvalidateRect(Some(list), None, true);
        }
        if !self.ground_view {
            self.sync_item_choice();
        }
        self.refresh_item_actions();
        self.refresh_skills();
        let snapshot = self.controller.snapshot();
        let phase = snapshot.phase;
        let is_character = matches!(
            phase,
            LoginPhase::CharactersReady | LoginPhase::CharacterPending
        );
        self.character_pages.sync(epoch, snapshot);
        for (id, turn) in [(CHARACTER_PREV, Turn::Previous), (CHARACTER_NEXT, Turn::Next)] {
            let _ = ShowWindow(self.control(id), if is_character && !self.inventory_view
                && !self.room_tools_view && !self.world_view { SW_SHOW } else { SW_HIDE });
            let _ = EnableWindow(self.control(id), !self.closing && Pages::enabled(snapshot)
                && self.character_pages.can_turn(turn));
        }
        let is_channel = matches!(
            phase,
            LoginPhase::CharacterSelected
                | LoginPhase::ChannelPending
                | LoginPhase::AwaitingChannelInfo
                | LoginPhase::LoadingRooms
                | LoginPhase::LobbyReady
        );
        let names = if is_channel {
            super::native_lobby::CHANNEL_NAMES
                .iter()
                .enumerate()
                .map(|(i, n)| format!("頻道 {} — {}", i + 1, n))
                .collect::<Vec<_>>()
        } else if is_character {
            snapshot
                .characters
                .iter()
                .skip(self.character_pages.range().start)
                .take(self.character_pages.range().len())
                .map(|c| {
                    format!(
                        "{}  Lv.{}  HP {} / MP {}　血盟：{}",
                        c.name, c.fields.level, c.fields.max_hp, c.fields.max_mp,
                        if c.clan.is_empty() { "無" } else { &c.clan }
                    )
                })
                .collect::<Vec<_>>()
        } else {
            snapshot
                .sources
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
        };
        let character_label = self.character_pages.label();
        self.text(
            304,
            if is_channel {
                "頻道"
            } else if is_character {
                &character_label
            } else {
                "來源伺服器"
            },
        );
        self.text(
            SELECT,
            if is_channel {
                "進入頻道"
            } else if is_character {
                "選擇角色"
            } else {
                "選擇來源"
            },
        );
        if names != self.names {
            send(self.control(SOURCES), CB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for name in &names {
                let name = wide(name);
                send(
                    self.control(SOURCES),
                    CB_ADDSTRING,
                    WPARAM(0),
                    LPARAM(name.as_ptr() as isize),
                );
            }
            if !names.is_empty() {
                send(self.control(SOURCES), CB_SETCURSEL, WPARAM(0), LPARAM(0));
            }
            self.names = names;
        }
        let room_labels = if let Some(room) = &snapshot.joined_room {
            room.members
                .iter()
                .map(|m| {
                    format!(
                        "{}　隊伍{}　{}",
                        m.name,
                        m.force + 1,
                        match m.state {
                            0 => "未準備",
                            1 => "已準備",
                            _ => "商店中",
                        }
                    )
                })
                .collect::<Vec<_>>()
        } else {
            snapshot
                .rooms
                .iter()
                .map(|r| {
                    format!(
                        "#{}  {}  {}/{}{}",
                        r.id,
                        r.name,
                        r.users,
                        r.capacity,
                        if r.attributes & 4 != 0 {
                            "　需密碼"
                        } else {
                            ""
                        }
                    )
                })
                .collect::<Vec<_>>()
        };
        self.text(
            305,
            if self.inventory_view {
                "背包"
            } else if snapshot.joined_room.is_some() {
                "房間成員"
            } else {
                "大廳房間"
            },
        );
        if room_labels != self.room_labels {
            let selected=self.room_choice.as_ref().and_then(|r|r.index(epoch,snapshot).ok());
            self.room_member=None;
            send(self.control(ROOMS), LB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for label in &room_labels {
                let text = wide(label);
                send(
                    self.control(ROOMS),
                    LB_ADDSTRING,
                    WPARAM(0),
                    LPARAM(text.as_ptr() as isize),
                );
            }
            if let Some(index)=selected {send(self.control(ROOMS),LB_SETCURSEL,WPARAM(index),LPARAM(0));}
            self.room_labels = room_labels;
        }
        let score_labels: Vec<_> = snapshot
            .world
            .scores
            .iter()
            .map(super::native_combat::score_label)
            .collect();
        if score_labels != self.score_labels {
            send(self.control(SCORES), LB_RESETCONTENT, WPARAM(0), LPARAM(0));
            for label in &score_labels {
                let text = wide(label);
                send(
                    self.control(SCORES),
                    LB_ADDSTRING,
                    WPARAM(0),
                    LPARAM(text.as_ptr() as isize),
                );
            }
            self.score_labels = score_labels;
            self.score_extent();
        }
        let status = if let Some(notice) = &snapshot.notice {
            notice.text.clone().unwrap_or_else(|| {
                format!(
                    "伺服器通知 #{}（文字資源對應尚未完成）",
                    notice.message_id.unwrap_or(0)
                )
            })
        } else if let Some(error) = &snapshot.error {
            format!("連線失敗：{error}")
        } else {
            match phase {
                LoginPhase::Disconnected | LoginPhase::Closed => "尚未連線。".into(),
                LoginPhase::Connecting => "正在連線…".into(),
                LoginPhase::ReadyForLogin => match snapshot.login_result.as_ref().map(|c| c.result)
                {
                    Some(8) => "帳號或密碼不正確，請重新輸入。".into(),
                    Some(0x1a) => "帳號已被停用。".into(),
                    Some(0x16) => "帳號目前使用中。".into(),
                    Some(_) => "登入未成功，請稍後再試。".into(),
                    None => "連線完成，請輸入帳密。".into(),
                },
                LoginPhase::LoginPending | LoginPhase::AwaitingSources => {
                    "正在登入並取得來源清單…".into()
                }
                LoginPhase::SourcesReady => "登入成功，請選擇來源伺服器。".into(),
                LoginPhase::RedirectPending => "正在選擇來源…".into(),
                LoginPhase::RedirectReady => format!(
                    "已選擇 {}。遊戲連線：{}\r\n按「取得角色」繼續。",
                    snapshot.selected_source.as_deref().unwrap_or(""),
                    snapshot.redirect.map(|a| a.to_string()).unwrap_or_default()
                ),
                LoginPhase::HandoffClosing | LoginPhase::GameConnecting => {
                    "正在交接遊戲連線…".into()
                }
                LoginPhase::TeleportQuitPending => "正在離開村莊並交接傳送…".into(),
                LoginPhase::GameLoginPending => "正在進行遊戲端認證…".into(),
                LoginPhase::LoadingCharacters => "正在載入角色…".into(),
                LoginPhase::CharactersReady => {
                    if snapshot.characters.is_empty() {
                        "此來源目前沒有角色。".into()
                    } else {
                        let refusal = match snapshot.character_result {
                            Some(1) => "伺服器未接受角色選取，請重試。",
                            Some(7) => "此角色目前不能參與，請選擇其他角色。",
                            Some(8) => "伺服器未接受角色名稱，請重新選取。",
                            _ => "請選擇角色。",
                        };
                        format!("{refusal}\r\n每頁 4 筆；PgUp／PgDn、滾輪或上一頁／下一頁翻頁。")
                    }
                }
                LoginPhase::CharacterPending => "正在選擇角色…".into(),
                LoginPhase::CharacterSelected => format!(
                    "已選擇 {}。請選擇頻道。",
                    snapshot.selected_character.as_deref().unwrap_or("")
                ),
                LoginPhase::ChannelPending | LoginPhase::AwaitingChannelInfo => {
                    "正在進入頻道…".into()
                }
                LoginPhase::LoadingRooms => "正在更新大廳房間…".into(),
                LoginPhase::LobbyReady => snapshot
                    .channel
                    .as_ref()
                    .map(|c| {
                        format!(
                            "頻道 {}　{} / {} 人\r\n已收到 {} 個房間，選取後可進入。",
                            c.id + 1,
                            c.users,
                            c.capacity,
                            snapshot.rooms.len()
                        )
                    })
                    .unwrap_or_default(),
                LoginPhase::ReturningCharacters => "正在返回角色清單…".into(),
                LoginPhase::RoomJoining | LoginPhase::RoomAwaitSelf => {
                    "正在加入房間並等待成員資料…".into()
                }
                LoginPhase::ForcePending => "正在等待隊伍更新…".into(),
                LoginPhase::ModePending => "正在等待模式與地圖更新…".into(),
                LoginPhase::MapPending => "正在等待地圖更新…".into(),
                LoginPhase::RoomCreating => "正在等待建立房間…".into(),
                LoginPhase::ClassicEntryPending => "正在等待普通世界入口回覆…".into(),
                LoginPhase::RentalSetPending => "正在等待租裝配裝確認…".into(),
                LoginPhase::RentalReady => "租裝配裝已確認，可進入租裝村。".into(),
                LoginPhase::RentalArmPending | LoginPhase::RentalAwaitPermission => "正在等待租裝村轉場…".into(),
                LoginPhase::RentalRestartPending => "正在等待租裝村返回確認…".into(),
                LoginPhase::ClassicCharacterSelected => "普通世界選角已接受，尚未收到場景。".into(),
                LoginPhase::RoomEditing => "正在等待房間設定回覆…".into(),
                LoginPhase::KickPending => "正在等待成員移出…".into(),
                LoginPhase::ObserverPending => "正在等待觀戰狀態…".into(),
                LoginPhase::RoomReady | LoginPhase::ReadyPending => super::native_room_ui::status(snapshot),
                LoginPhase::RoomLeaving => "正在離開房間…".into(),
                LoginPhase::BattleResult => format!(
                    "戰鬥結束：{}　結算 {} 人。按「返回大廳」繼續。",
                    match snapshot.world.battle_result {
                        Some(0) => "第一隊獲勝",
                        Some(1) => "第二隊獲勝",
                        _ => "無勝方",
                    },
                    snapshot.world.scores.len()
                ),
                LoginPhase::ResultReturning => "正在確認結算並返回大廳…".into(),
                LoginPhase::WorldEntering => "正在進入普通世界…".into(),
                LoginPhase::BattlePending | LoginPhase::WorldReady => {
                    if let Some(error) = &self.world_presentation.error {
                        format!("世界資源載入失敗：{error}")
                    } else if let Some(frame) = &self.world_presentation.frame {
                        let omitted = frame.receipt["actors"].as_array().map_or(0, |rows| {
                            rows.iter().filter(|r| r["rendered"] == false).count()
                        });
                        let target = self
                            .target
                            .and_then(|t| t.resolve_for_skill(&snapshot.world, self.skills.selected().unwrap_or(0)))
                            .and_then(|id| snapshot.world.actors.get(&id))
                            .map(super::native_combat::target_label)
                            .unwrap_or_else(|| "未選取".into());
                        let ground = self
                            .ground
                            .and_then(|c| {
                                c.resolve(
                                    self.controller.transport().snapshot().epoch,
                                    &snapshot.world,
                                )
                            })
                            .map(|c| format!("　選點 {},{}", c[0], c[1]))
                            .unwrap_or_default();
                        let selection = if let Some(skill) = self.skills.selected()
                            .filter(|id| super::native_skills::item_targeted(*id)) {
                            let epoch = self.controller.transport().snapshot().epoch;
                            if super::native_skills::weapon_bound(skill) {
                                super::native_skills::ItemTarget::equipped_weapon(&snapshot, epoch)
                                    .and_then(|item| super::native_skills::item_request(&snapshot,epoch,skill,&item))
                                    .err().map(|e|e.to_string()).unwrap_or_else(|| "可對裝備中的武器施法".into())
                            } else {
                            self.skill_item_target.as_ref().context("請開啟背包並選取施法物品")
                                .and_then(|item| super::native_skills::item_request(&snapshot,epoch,skill,item))
                                .err().map(|e|e.to_string()).unwrap_or_else(|| "可對所選背包物品施法".into())
                            }
                        } else { self
                            .skills
                            .choice_at(
                                self.controller.transport().snapshot().epoch,
                                &snapshot.world,
                                self.target,
                                self.ground,
                            )
                            .err()
                            .map(|e| e.to_string())
                            .unwrap_or_else(|| "可按「施放技能」".into()) };
                        let guidance = if snapshot.battle_withdrawal_pending {
                            "正在等待退出戰鬥確認…".to_owned()
                        } else if self.item_cursor.is_some() {
                            "請點選物品目標位置；Esc取消".to_owned()
                        } else if snapshot.item_feedback == super::native_item_actions::Feedback::TargetRejected {
                            snapshot.world.last_system_message.as_ref()
                                .filter(|(kind,_)|*kind==9).map(|(_,text)|text.clone())
                                .unwrap_or_else(||snapshot.item_feedback.label().to_owned())
                        } else if matches!(snapshot.item_feedback,
                            super::native_item_actions::Feedback::SceneUpdated
                            | super::native_item_actions::Feedback::TargetNoResponse) {
                            snapshot.item_feedback.label().to_owned()
                        } else if let Some((9,text)) = &snapshot.world.last_system_message {
                            if text.starts_with("戰鬥中無法退出，請停止攻擊並等待 ") {
                                text.clone()
                            } else {format!("{selection}；F1 治癒／F2 護盾／F3 光箭")}
                        } else {format!("{selection}；F1 治癒／F2 護盾／F3 光箭")};
                        let buffs = super::native_status_icons::summary(&snapshot.world,
                            super::native_status4183::now_ms(), self.controller.skill_cooldown_ms(std::time::Instant::now()));
                        format!("地圖 {}　HP {}　{}\r\n目標：{}{}　{}　{}\r\n{}\r\n方向鍵移動；Space攻擊；O門；P入口；Esc清除。未顯示 {} 個物件。{}",
                            snapshot.world.map_id.unwrap_or(0),snapshot.world.hp.map(|[current,maximum]|format!("{current}/{maximum}")).unwrap_or_else(||"等待資料".into()),buffs,target,ground,
                            snapshot.attack_feedback.text(),snapshot.skill_feedback.label(),format!("{}　{}",guidance,self.controller.portal_entry_feedback().label()),omitted,
                            self.audio_error.as_ref().map(|e|format!("音效：{e}")).unwrap_or_default())
                    } else {
                        "正在載入伺服器指定的世界資源…".into()
                    }
                }
                LoginPhase::Closing => "正在中斷連線…".into(),
                LoginPhase::Failed => "連線已結束。".into(),
            }
        };
        let status = if phase == LoginPhase::CharacterSelected {
            match snapshot.channel_result {
                Some(code) if code!=0 && self.rental.set_attempt.is_some()=>"伺服器未接受此次配裝，請選擇其他配裝再試。".into(),
                Some(1) => "此頻道目前無法進入，請選擇其他頻道。".into(),
                Some(10) => "此角色等級尚不能進入頻道。".into(),
                _ => status,
            }
        } else {
            status
        };
        let status = if phase == LoginPhase::LobbyReady {
            match snapshot.room_result {
                Some(11) => "房間密碼不正確，請重試。".into(),
                Some(10) => "此角色不能進入這個等級區間的房間。".into(),
                Some(r) if r != 0 => format!("無法進入房間（伺服器結果 {}）。", r),
                _ => status,
            }
        } else {
            status
        };
        let status=if phase==LoginPhase::BattlePending&&snapshot.observer {"觀戰中，方向鍵移動鏡頭。".into()} else if self.rental.open {self.rental_status()}else{status};
        if status != self.status {
            self.text(STATUS, &status);
            self.status = status;
        }
        let owner=phase==LoginPhase::RoomReady && snapshot.joined_room.as_ref().is_some_and(|r|snapshot.selected_character.as_deref()==Some(r.owner.as_str()));
        let no_editor=self.room_editor.is_none();
        let modes=owner && no_editor;
        self.text(OBSERVER,if snapshot.observer {"關閉觀戰"}else{"開啟觀戰"});
        for (id,enabled) in [
            (CREATE_ROOM,phase==LoginPhase::LobbyReady && no_editor),
            (EDIT_ROOM,owner && no_editor),
            (OBSERVER,matches!(phase,LoginPhase::LobbyReady|LoginPhase::RoomReady) && no_editor),
            (MODE_PREV,modes),(MODE_NEXT,modes),
            (MAP_PREV,modes),(MAP_NEXT,modes),
            (KICK_MEMBER,owner && no_editor && self.room_member.is_some()),
            (SAVE_ROOM,self.room_editor.is_some()),(DISCARD_ROOM,self.room_editor.is_some()),
            (ROOM_NAME,self.room_editor.is_some()),(ROOM_CAPACITY,self.room_editor.is_some()),
            (ROOM_LOCK,self.room_editor.is_some()),
            (ROOM_SECRET,self.room_editor.is_some() && send(self.control(ROOM_LOCK),BM_GETCHECK,WPARAM(0),LPARAM(0)).0==1),
        ] {let _=EnableWindow(self.control(id),enabled && !self.closing && !self.inventory_view);}
        for (id, enabled) in [
            (CONNECT, self.controller.is_quiescent()),
            (LOGIN, phase == LoginPhase::ReadyForLogin),
            (ACCOUNT, phase == LoginPhase::ReadyForLogin),
            (PASSWORD, phase == LoginPhase::ReadyForLogin),
            (
                SOURCES,
                phase == LoginPhase::SourcesReady
                    || matches!(
                        phase,
                        LoginPhase::CharacterSelected | LoginPhase::LobbyReady
                    )
                    || (phase == LoginPhase::CharactersReady && snapshot.notice.is_none()),
            ),
            (
                SELECT,
                phase == LoginPhase::SourcesReady
                    || matches!(
                        phase,
                        LoginPhase::CharacterSelected | LoginPhase::LobbyReady
                    )
                    || (phase == LoginPhase::CharactersReady
                        && !snapshot.characters.is_empty()
                        && snapshot.notice.is_none()),
            ),
            (GAME, phase == LoginPhase::RedirectReady),
            (ACK, snapshot.notice.is_some()),
            (
                RETURN,
                matches!(
                    phase,
                    LoginPhase::CharacterSelected
                        | LoginPhase::LobbyReady
                        | LoginPhase::BattleResult
                ) || (phase.active_world() && snapshot.rental_set.is_none()
                    && !snapshot.observer && !snapshot.battle_withdrawal_pending
                    && snapshot.world.ready()
                    && snapshot.joined_room.as_ref().is_some_and(|r|r.mode!=6)),
            ),
            (REFRESH, phase == LoginPhase::LobbyReady),
            (
                JOIN,
                phase == LoginPhase::LobbyReady && !snapshot.rooms.is_empty(),
            ),
            (ROOM_PASSWORD, phase == LoginPhase::LobbyReady),
            (READY, phase == LoginPhase::RoomReady && !snapshot.observer),
            (UNREADY, phase == LoginPhase::RoomReady),
            (CHANGE_FORCE, phase == LoginPhase::RoomReady && snapshot.joined_room.as_ref().is_some_and(|r|
                r.members.iter().any(|m|Some(m.name.as_str())==snapshot.selected_character.as_deref()))),
            (EXIT_ROOM, phase == LoginPhase::RoomReady),
            (CANCEL, !self.controller.is_quiescent()),
        ] {
            // Rental RETURN is governed below by refresh_rental. Temporarily
            // disabling its visible hotel button here retires keyboard focus.
            if id == RETURN
                && matches!(phase, LoginPhase::BattlePending | LoginPhase::RentalRestartPending)
                && snapshot.joined_room.as_ref().is_some_and(|room| room.mode == 6)
            {
                continue;
            }
            let _ = EnableWindow(
                self.control(id),
                enabled
                    && !self.closing
                    && !(self.inventory_view && matches!(id, JOIN | ROOM_PASSWORD)),
            );
        }
        self.refresh_rental();
        self.refresh_companions();
        self.restore_keyboard_focus_after_update();
    }
    unsafe fn restore_keyboard_focus_after_update(&self) {
        if self.closing || GetForegroundWindow() != self.parent { return; }
        let focused = GetFocus();
        let retired = focused.0.is_null() || self.controls.iter().any(|(_, control)|
            *control == focused && !IsWindowEnabled(*control).as_bool());
        if retired { let _ = SetFocus(Some(self.parent)); }
    }
    unsafe fn action(&mut self, id: u16) -> Result<()> {
        match self.companion_action(id) {
            Ok(true)=>return Ok(()), Ok(false)=>{},
            Err(error)=>{self.companions.error=Some(format!("{error:#}"));self.refresh_companions();return Err(error);}
        }
        if self.rental_action(id)? {return Ok(());}
        match id {
            skill_ui::BOOK => {
                self.skill_book_open = !self.skill_book_open;
                self.held.clear(); self.held_attack.clear();
                self.layout(self.parent, false);
                let _ = SetFocus(Some(if self.skill_book_open { self.control(SKILLS) } else { self.parent }));
                let _ = InvalidateRect(Some(self.parent), None, true);
            }
            skill_ui::FIRST..=skill_ui::LAST => self.skill_slot(usize::from(id - skill_ui::FIRST))?,
            CREATE_ROOM|EDIT_ROOM=>{
                let d=super::native_room_ui::Draft::capture(self.controller.transport().snapshot().epoch,self.controller.snapshot(),id==EDIT_ROOM)?;
                self.close_room_editor();self.text(ROOM_NAME,d.name());self.text(ROOM_CAPACITY,&d.capacity().to_string());
                send(self.control(ROOM_LOCK),BM_SETCHECK,WPARAM(usize::from(d.protected())),LPARAM(0));
                self.room_editor=Some(d);self.layout(self.parent,false);let _=SetFocus(Some(self.control(ROOM_NAME)));
            }
            DISCARD_ROOM=>{self.close_room_editor();self.layout(self.parent,false);}
            ROOM_LOCK=>{
                if send(self.control(ROOM_LOCK),BM_GETCHECK,WPARAM(0),LPARAM(0)).0==0 {
                    self.text(ROOM_SECRET,"");send(self.control(ROOM_SECRET),EM_EMPTYUNDOBUFFER,WPARAM(0),LPARAM(0));
                }
            }
            SAVE_ROOM=>{
                let draft=self.room_editor.clone().context("請先開啟房間設定")?;
                let name=self.read(ROOM_NAME,89)?;let password=self.read(ROOM_SECRET,4)?;
                let capacity=self.read(ROOM_CAPACITY,3)?.parse::<u16>().context("請輸入2至100的人數上限")?;
                let protected=send(self.control(ROOM_LOCK),BM_GETCHECK,WPARAM(0),LPARAM(0)).0==1;
                let settings=draft.settings(self.controller.transport().snapshot().epoch,self.controller.snapshot(),&name,&password,protected,capacity)?;
                let result=if draft.editing(){self.controller.edit_room(&settings)}else{self.controller.create_room(&settings)};
                self.close_room_editor();self.layout(self.parent,false);result?;
            }
            OBSERVER=>self.controller.set_observer(!self.controller.snapshot().observer)?,
            MAP_PREV|MAP_NEXT=>{
                let room=self.controller.snapshot().joined_room.as_ref().context("map selection missing room")?;
                let maps=super::native_room::mode_maps(room.mode);
                let at=maps.iter().position(|m|*m==room.map_id).context("目前地圖不在模式清單")?;
                let next=if id==MAP_NEXT {(at+1)%maps.len()}else{(at+maps.len()-1)%maps.len()};
                self.controller.select_room_map(maps[next])?;
            },
            MODE_PREV|MODE_NEXT=>self.controller.step_room_mode(if id==MODE_NEXT {1}else{2})?,
            KICK_MEMBER=>{
                let choice=self.room_member.as_ref().context("請先選取要移出的成員")?;
                let name=choice.resolve(self.controller.transport().snapshot().epoch,self.controller.snapshot())?.to_owned();
                self.controller.kick_member(&name)?;self.room_member=None;
            }
            INVENTORY_TOGGLE | GROUND_TOGGLE => {
                self.rental.open=false;
                ensure!(
                    self.controller.snapshot().inventory.owner.is_some() && !self.result_view,
                    "目前沒有可查看的背包"
                );
                let ground = id == GROUND_TOGGLE;
                ensure!(
                    !ground || super::native_ground_pickup::available(self.controller.snapshot()),
                    "目前不能查看地面物品"
                );
                self.inventory_view = !(self.inventory_view && self.ground_view == ground);
                self.ground_view = ground && self.inventory_view;
                self.pickup_choice = None;
                self.pickup_rows.clear();
                self.inventory_choice = None;
                self.item_cursor = None;
                self.item_selection = None;
                self.destroy_choice = None;
                self.inventory_rows.clear();
                send(
                    self.control(INVENTORY_LIST),
                    LB_RESETCONTENT,
                    WPARAM(0),
                    LPARAM(0),
                );
                self.held.clear();
                self.held_attack.clear();
                self.painted = None;self.painted_input = None;
                self.layout(self.parent, false);
                let _ = InvalidateRect(Some(self.parent), None, true);
                let _ = SetFocus(Some(if self.inventory_view {
                    self.control(INVENTORY_LIST)
                } else {
                    self.parent
                }));
            }
            ITEM_USE if self.ground_view => {
                let choice = self.pickup_choice.clone().context("請先選取地面物品")?;
                let quantity = self
                    .read(ITEM_QUANTITY, 10)?
                    .parse::<u32>()
                    .context("請輸入有效的拾取數量")?;
                self.held.clear();
                self.held_attack.clear();
                let frame = self
                    .world_presentation
                    .frame
                    .as_ref()
                    .context("等待世界畫面")?;
                self.controller.pickup(choice, quantity, frame)?;
                self.refresh_item_actions();
            }
            ITEM_USE | ITEM_DROP => {
                let choice = self.item_choice()?;
                self.destroy_choice = None;
                self.item_selection = None;
                if id == ITEM_USE && (super::native_item_actions::inventory_target_type(choice.item.use_type)
                    || super::native_item_actions::text_choice_type(choice.item.use_type)
                    || super::native_item_actions::letter_choice_type(choice.item.use_type)) {
                    self.item_selection = Some(super::native_item_actions::Selection::capture(
                        self.controller.snapshot(), choice)?);
                    self.text(ITEM_ARGUMENT, "");
                    self.text(ITEM_SUBJECT, "");
                    self.text(ITEM_CONTENT, "");
                    self.layout(self.parent, false);
                    self.refresh_item_actions();
                    return Ok(());
                }
                if id == ITEM_USE && super::native_item_actions::world_target_type(choice.item.use_type) {
                    let cursor = super::native_item_actions::TargetChoice::capture(
                        self.controller.snapshot(), choice.item.id)?;
                    self.held.clear(); self.held_attack.clear();
                    self.item_cursor = Some(cursor);
                    self.inventory_view = false; self.ground_view = false;
                    self.painted = None;self.painted_input = None;
                    self.layout(self.parent, false);
                    self.text(STATUS, "請點選物品目標位置；Esc取消");
                    let _ = InvalidateRect(Some(self.parent), None, true);
                    let _ = SetFocus(Some(self.parent));
                    return Ok(());
                }
                let action = if id == ITEM_USE {
                    super::native_item_actions::Action::Use
                } else {
                    let text = self.read(ITEM_QUANTITY, 10)?;
                    let count = text.parse::<u32>().context("請輸入有效的丟棄數量")?;
                    super::native_item_actions::Action::Drop(count)
                };
                self.held.clear();
                self.held_attack.clear();
                self.controller.item_action(choice, action)?;
                self.refresh_item_actions();
            }
            CAST_SKILL => {
                let skill = self.skills.selected_skill(
                    self.controller.transport().snapshot().epoch,
                    &self.controller.snapshot().world,
                )?;
                self.cast_skill(skill)?;
            }
            ITEM_DESTROY => {
                let choice = self.item_choice()?;
                super::native_item_actions::request(self.controller.snapshot(), &choice,
                    super::native_item_actions::Action::Destroy)?;
                self.item_selection = None;
                self.text(ITEM_ARGUMENT, &choice.item.label());
                self.destroy_choice = Some(choice);
                self.layout(self.parent, false);
                self.refresh_item_actions();
            }
            ITEM_OPTION_CANCEL => {
                if let Some(selection) = self.item_selection.as_mut() { selection.cancel(); }
                self.item_selection = None;
                self.destroy_choice = None;
                self.text(ITEM_ARGUMENT, "");
                self.text(ITEM_SUBJECT, "");
                self.text(ITEM_CONTENT, "");
                self.layout(self.parent, false);
                self.refresh_item_actions();
            }
            ITEM_OPTION_SEND => {
                if let Some(choice) = self.destroy_choice.clone() {
                    ensure!(self.item_choice()? == choice, "選取已變更，請重新確認銷毀物品");
                    self.controller.item_action(choice, super::native_item_actions::Action::Destroy)?;
                    self.destroy_choice = None;
                } else {
                    let selection = self.item_selection.clone().context("請先使用物品")?;
                    let target = if super::native_item_actions::inventory_target_type(selection.source.item.use_type) {
                        Some(self.item_choice()?)
                    } else { None };
                    let text = self.read(ITEM_ARGUMENT,255)?;
                    if super::native_item_actions::letter_choice_type(selection.source.item.use_type) {
                        let subject = self.read(ITEM_SUBJECT, 21)?;
                        let content = self.read(ITEM_CONTENT, 257)?;
                        self.controller.item_letter_action(selection, &text, &subject, &content)?;
                    } else {
                        self.controller.item_selection_action(selection,target,&text)?;
                    }
                    self.item_selection = None;
                }
                self.layout(self.parent, false);
                self.refresh_item_actions();
            }
            CONNECT => self.controller.start(self.peer, Limits::default())?,
            LOGIN => {
                let account = self.read(ACCOUNT, 64)?;
                let password = self.read(PASSWORD, 128)?;
                let result = self.controller.login(&account, &password);
                self.text(PASSWORD, "");
                send(
                    self.control(PASSWORD),
                    EM_EMPTYUNDOBUFFER,
                    WPARAM(0),
                    LPARAM(0),
                );
                result?;
            }
            SELECT => {
                self.rental.set_attempt=None;
                let index = send(self.control(SOURCES), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
                ensure!(index >= 0, "請選擇來源伺服器");
                if matches!(
                    self.controller.snapshot().phase,
                    LoginPhase::CharacterSelected | LoginPhase::LobbyReady
                ) {
                    self.controller.enter_channel(index as u8)?;
                } else if self.controller.snapshot().phase == LoginPhase::CharactersReady {
                    let index = self.character_pages.index(self.controller.transport().snapshot().epoch,
                        self.controller.snapshot(), index as usize)?;
                    self.controller.select_character(index)?;
                } else {
                    self.controller.select_source(index as usize)?;
                }
            }
            CHARACTER_PREV | CHARACTER_NEXT => {
                if self.character_paging_enabled() {
                    self.character_pages.turn(if id == CHARACTER_PREV { Turn::Previous } else { Turn::Next });
                }
            }
            GAME => self.controller.begin_game()?,
            ACK => self.controller.acknowledge_notice()?,
            RETURN => {
                if self.controller.snapshot().phase == LoginPhase::BattleResult {
                    self.controller.return_from_battle()?;
                } else if self.controller.snapshot().phase.active_world() {
                    self.held.clear();self.held_attack.clear();
                    self.controller.request_battle_withdrawal()?;
                } else {
                    self.controller.return_characters()?;
                }
            }
            REFRESH => self.controller.refresh_rooms()?,
            JOIN => {
                self.sync_room_member();
                let i=self.room_choice.as_ref().context("請選擇房間")?
                    .join_index(self.controller.transport().snapshot().epoch,self.controller.snapshot())?;
                let password = self.read(ROOM_PASSWORD, 4)?;
                let result = self.controller.join_room(i, &password);
                self.text(ROOM_PASSWORD, "");
                send(
                    self.control(ROOM_PASSWORD),
                    EM_EMPTYUNDOBUFFER,
                    WPARAM(0),
                    LPARAM(0),
                );
                result?;
            }
            READY => self.controller.set_ready(true)?,
            UNREADY => self.controller.set_ready(false)?,
            CHANGE_FORCE => {
                let s=self.controller.snapshot();
                let own=s.joined_room.as_ref().and_then(|r|r.members.iter()
                    .find(|m|Some(m.name.as_str())==s.selected_character.as_deref())).context("尚未取得房間成員")?;
                self.controller.set_force(1-own.force)?;
            }
            EXIT_ROOM => self.controller.leave_room()?,
            CANCEL => self.controller.cancel(),
            _ => {}
        }
        self.refresh();
        Ok(())
    }
}
impl Drop for Ui {
    fn drop(&mut self) {
        unsafe {
            if !self.font.is_invalid() {
                let _ = DeleteObject(self.font.into());
            }
        }
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
    }
    if msg == WM_MEASUREITEM {
        let measure = &mut *(lparam.0 as *mut MEASUREITEMSTRUCT);
        if matches!(measure.CtlID, id if id == u32::from(INVENTORY_LIST) || id == u32::from(SKILLS)) {
            measure.itemHeight = 40 * GetDpiForWindow(hwnd).max(96) / 96;
            return LRESULT(1);
        }
        if measure.CtlID == u32::from(rental::LINKS) {
            measure.itemHeight = 32 * GetDpiForWindow(hwnd).max(96) / 96;
            return LRESULT(1);
        }
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if !ptr.is_null() {
        let state = &*ptr;
        if let Ok(mut ui) = state.0.try_borrow_mut() {
            match msg {
                WM_DRAWITEM => {
                    let draw = &*(lparam.0 as *const DRAWITEMSTRUCT);
                    if draw.CtlID == u32::from(SKILLS)
                        || (u32::from(skill_ui::FIRST)..=u32::from(skill_ui::LAST)).contains(&draw.CtlID) {
                        ui.draw_skill_control(draw);
                        return LRESULT(1);
                    }
                    if draw.CtlID == u32::from(INVENTORY_LIST) {
                        ui.draw_inventory(draw);
                        return LRESULT(1);
                    }
                    if draw.CtlID == u32::from(rental::LINKS) {
                        ui.draw_rental_choice(draw);
                        return LRESULT(1);
                    }
                }
                WM_LBUTTONDOWN if ui.world_view && !ui.inventory_view && !(ui.rental.open || ui.companions.open) => {
                    let _ = SetFocus(Some(hwnd));
                    let mut area = RECT::default();
                    let _ = GetClientRect(hwnd, &mut area);
                    let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                    let x = (lparam.0 as u16) as i16 as i32;
                    let y = ((lparam.0 >> 16) as u16) as i16 as i32;
                    if let (Some(view), Some((epoch,surface)), Some(own)) = (
                        super::compositor::Viewport::fit(
                            area.right,
                            area.bottom - (if ui.result_view { 260 } else { 250 }) * dpi / 96,
                            800,
                            600,
                        ),
                        ui.painted_input.as_ref(),
                        ui.controller.snapshot().world.own_actor(),
                    ) {
                        if *epoch != ui.controller.transport().snapshot().epoch
                            || !surface.input_ready(&ui.controller.snapshot().world)
                            || ui.painted.is_none_or(|(_,_,painted_view)|painted_view!=view) {
                            return LRESULT(0);
                        }
                        let point = view.canvas_point(x, y, 800, 600);
                        if point.is_some_and(|(x,y)| surface.overlay_contains(x,y)) {return LRESULT(0);}
                        let world = &ui.controller.snapshot().world;
                        let picked =
                            point.and_then(|(x, y)| surface.pick_current(world, x, y, own.id));
                        if let Some(cursor) = ui.item_cursor.clone() {
                            let selected = picked.and_then(|id| world.actors.get(&id)
                                .map(|a| (id, [a.x, a.y])))
                                .or_else(|| point.and_then(|(x,y)| surface.ground_cell(world,x,y))
                                    .map(|cell| (0, [cell.x as u16,cell.y as u16])));
                            if let Some((target, position)) = selected {
                                let Ui { controller, world_presentation, .. } = &mut *ui;
                                let result = controller.item_target_action(cursor, target, position,
                                    world_presentation.frame.as_ref().unwrap());
                                ui.item_cursor = None;
                                match result {
                                    Ok(()) => ui.text(STATUS, "已送出物品目標，等待伺服器更新"),
                                    Err(error) => ui.text(STATUS, &format!("{error:#}")),
                                }
                            }
                            return LRESULT(0);
                        }
                        if picked.is_some_and(|id| surface.is_ground_item(id)) {
                            let choice = super::native_ground_pickup::Choice::capture(
                                ui.controller.snapshot(),ui.world_presentation.frame.as_ref().unwrap(),picked.unwrap());
                            match choice {
                                Ok(choice) => {
                                    ui.target = None;
                                    ui.ground = None;
                                    if let Err(error) = ui.action(GROUND_TOGGLE) {
                                        ui.text(STATUS, &format!("{error:#}"));
                                    } else if let Some(index) =
                                        ui.pickup_rows.iter().position(|c| c == &choice)
                                    {
                                        send(
                                            ui.control(INVENTORY_LIST),
                                            LB_SETCURSEL,
                                            WPARAM(index),
                                            LPARAM(0),
                                        );
                                        ui.pickup_choice = Some(choice);
                                        ui.refresh_item_actions();
                                    }
                                }
                                Err(error) => ui.text(STATUS, &format!("{error:#}")),
                            }
                            return LRESULT(0);
                        }
                        let target = point
                            .and_then(|(x, y)| surface.pick_current(world, x, y, own.id))
                            .and_then(|id| {
                                super::native_skill_selection::Target::capture(world, id)
                            });
                        let ground = point
                            .and_then(|(x, y)| surface.ground_cell(world, x, y))
                            .and_then(|cell| {
                                super::native_skill_selection::Ground::capture(
                                    ui.controller.transport().snapshot().epoch,
                                    world,
                                    cell,
                                )
                            });
                        // Clicking the already-selected attackable target again
                        // starts a mouse attack that walks toward it (bounded
                        // chase through the same checked movement path).
                        let again = target.as_ref().zip(ui.target.as_ref())
                            .is_some_and(|(new, old)| new.id == old.id)
                            && target.as_ref().is_some_and(|t| world.attack_reach(t.id).is_some());
                        ui.target = target;
                        ui.ground = ground;
                        ui.held_attack.clear();
                        if again {
                            if let Some(id) = ui.target.as_ref().map(|t| t.id) {
                                ui.start_chase(id);
                            }
                        }
                    }
                    return LRESULT(0);
                }
                WM_COMMAND if (wparam.0 & 0xffff)==rental::LIST as usize && (wparam.0>>16)==LBN_SELCHANGE as usize => {
                    ui.refresh_rental();return LRESULT(0);
                }
                WM_COMMAND if (wparam.0 & 0xffff)==ROOMS as usize && (wparam.0>>16)==LBN_SELCHANGE as usize => {
                    ui.sync_room_member();ui.refresh();return LRESULT(0);
                }
                WM_COMMAND
                    if (wparam.0 & 0xffff) == INVENTORY_LIST as usize
                        && (wparam.0 >> 16) == LBN_SELCHANGE as usize =>
                {
                    ui.sync_item_choice();
                    ui.skill_item_target = if ui.inventory_view && !ui.ground_view {
                        ui.inventory_choice.as_ref().and_then(|choice| super::native_skills::ItemTarget::capture(
                            ui.controller.snapshot(), ui.controller.transport().snapshot().epoch, choice.item.id).ok())
                    } else { None };
                    ui.refresh_item_actions();
                    ui.refresh_skills();
                    return LRESULT(0);
                }
                WM_COMMAND
                    if (wparam.0 & 0xffff) == SKILLS as usize
                        && (wparam.0 >> 16) == LBN_SELCHANGE as usize =>
                {
                    ui.held.clear();
                    ui.held_attack.clear();
                    let index = send(ui.control(SKILLS), LB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
                    let id = usize::try_from(index).ok()
                        .and_then(|i| ui.skills.rows().get(i))
                        .map(|row| row.id);
                    if let Err(error) = ui.skills.select(id) {
                        ui.text(STATUS, &format!("{error:#}"));
                    }
                    ui.summon_choice=None;send(ui.control(SUMMON_OPTIONS),CB_SETCURSEL,WPARAM(0),LPARAM(0));
                    ui.teleport_choice=None;
                    ui.refresh();
                    ui.layout(hwnd,false);
                    return LRESULT(0);
                }
                WM_COMMAND if (wparam.0 & 0xffff)==SUMMON_OPTIONS as usize && (wparam.0>>16)==CBN_SELCHANGE as usize => {
                    ui.held.clear();ui.held_attack.clear();ui.summon_choice=None;
                    let index=send(ui.control(SUMMON_OPTIONS),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
                    if ui.skills.selected()==Some(51) && !ui.inventory_view && !ui.rental.open && !ui.closing {
                        ui.summon_choice=index.checked_sub(1).and_then(|i|usize::try_from(i).ok())
                            .and_then(|i|super::native_skills::SUMMON_SELECTIONS.get(i))
                            .and_then(|r|super::native_skills::SummonTarget::capture(ui.controller.snapshot(),ui.controller.transport().snapshot().epoch,r.selection).ok());
                    }
                    ui.refresh();return LRESULT(0);
                }
                WM_COMMAND if (wparam.0 & 0xffff)==TELEPORT_DEST as usize && (wparam.0>>16)==CBN_SELCHANGE as usize => {
                    ui.held.clear();ui.held_attack.clear();
                    let index=send(ui.control(TELEPORT_DEST),CB_GETCURSEL,WPARAM(0),LPARAM(0)).0;
                    ui.teleport_choice=index.checked_sub(1).and_then(|i|usize::try_from(i).ok())
                        .and_then(|i|ui.controller.teleport_choice(i).ok());
                    ui.refresh_skills();return LRESULT(0);
                }
                WM_COMMAND if (wparam.0 >> 16) == BN_CLICKED as usize => {
                    let result=ui.action((wparam.0 & 0xffff) as u16);
                    ui.refresh();
                    if let Err(error) = result {
                        ui.text(STATUS, &format!("{error:#}"));
                    }
                    return LRESULT(0);
                }
                WM_TIMER => {
                    ui.poll_presented(hwnd);
                    ui.world_tick(hwnd);
                    ui.input_tick();
                    ui.refresh();
                    if ui.closing
                        && ui.controller.is_quiescent()
                        && ui.world_presentation.quiescent()
                        && ui.inventory_names.quiescent()
                        && ui.inventory_icons.quiescent()
                        && ui.skill_icons.quiescent()
                        && ui.rental.documents.quiescent()
                    {
                        let _ = DestroyWindow(hwnd);
                    }
                    return LRESULT(0);
                }
                WM_ACTIVATEAPP if wparam.0 == 0 => {
                    ui.item_cursor = None;
                    ui.held.clear();
                    ui.held_attack.clear();
                    if let Err(e)=ui.voices.stop_all() {ui.audio_error=Some(e.to_string());}
                }
                WM_KILLFOCUS | WM_CANCELMODE => {
                    ui.item_cursor = None;
                    ui.held.clear();
                    ui.held_attack.clear();
                }
                WM_CLOSE => {
                    ui.item_cursor = None;
                    ui.held.clear();
                    ui.held_attack.clear();
                    if let Err(e)=ui.voices.stop_all() {ui.audio_error=Some(e.to_string());}
                    ui.closing = true;
                    ui.controller.cancel();
                    ui.world_presentation.cancel();
                    ui.inventory_names.cancel();
                    ui.inventory_icons.cancel();
                    ui.skill_icons.cancel();
                    ui.rental.documents.cancel();
                    ui.refresh();
                    if ui.controller.is_quiescent()
                        && ui.world_presentation.quiescent()
                        && ui.inventory_names.quiescent()
                        && ui.inventory_icons.quiescent()
                        && ui.skill_icons.quiescent()
                        && ui.rental.documents.quiescent()
                    {
                        let _ = DestroyWindow(hwnd);
                    }
                    return LRESULT(0);
                }
                WM_SIZE => {
                    ui.painted = None;ui.painted_input = None;
                    ui.layout(hwnd, false);
                    let _ = InvalidateRect(Some(hwnd), None, true);
                    return LRESULT(0);
                }
                WM_PAINT if ui.world_view => {
                    ui.paint_world(hwnd);
                    return LRESULT(0);
                }
                WM_GETMINMAXINFO => {
                    let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                    let info = &mut *(lparam.0 as *mut MINMAXINFO);
                    info.ptMinTrackSize.x = 520 * dpi / 96;
                    info.ptMinTrackSize.y = 800 * dpi / 96;
                    return LRESULT(0);
                }
                WM_DPICHANGED => {
                    ui.painted = None;ui.painted_input = None;
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
                    ui.layout(hwnd, true);
                    return LRESULT(0);
                }
                _ => {}
            }
        }
    }
    if msg == WM_DESTROY {
        let _ = KillTimer(Some(hwnd), 1);
        PostQuitMessage(0);
        return LRESULT(0);
    }
    if msg == WM_NCDESTROY {
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}
pub fn run(peer: SocketAddr) -> Result<()> {
    run_with_resources(peer, std::path::PathBuf::from(r"D:\lineage開發\0-客戶端"))
}
pub fn run_with_resources(peer: SocketAddr, resource_root: std::path::PathBuf) -> Result<()> {
    ensure!(
        resource_root.is_absolute(),
        "resource root must be absolute"
    );
    unsafe {
        let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let result = (|| -> Result<()> {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            let class = w!("LiToNativeLoginWindow");
            let _ = RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                lpszClassName: class,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                ..Default::default()
            });
            let state = Box::new(State(RefCell::new(Ui {
                character_pages: Pages::default(),
                rental:rental::View::new(resource_root.clone())?,
                companions:companions::View::default(),
                parent: HWND::default(),
                room_tools_view:false,
                room_editor:None,
                room_member:None,
                room_choice:None,
                inventory_view: false,
                ground_view: false,
                pickup_rows: vec![],
                pickup_choice: None,
                inventory_rows: vec![],
                inventory_owner: None,
                inventory_names: super::native_inventory_names::Service::start(
                    resource_root.clone(),
                )?,
                inventory_icons: super::native_inventory_icons::Service::new(resource_root.clone()),
                inventory_icon_rows: vec![],
                inventory_choice: None,
                skill_item_target: None,
                summon_choice:None,
                teleport_rows:vec![],
                teleport_choice:None,
                item_cursor: None,
                item_selection: None,
                destroy_choice: None,
                displayed_mail: None,
                target: None,
                ground: None,
                skills: Default::default(),
                skill_book_open: false,
                skill_icons: super::native_inventory_icons::Service::spells(resource_root.clone()),
                painted: None,
                painted_input: None,
                voices: super::combat_audio::CombatVoicePool::new(4)?,
                audio_scope: None,
                audio_frame: None,
                audio_error: None,
                peer,
                resource_root,
                controller: LoginController::default(),
                controls: vec![],
                names: vec![],
                status: String::new(),
                room_labels: vec![],
                score_labels: vec![],
                result_view: false,
                world_presentation: Default::default(),
                world_view: false,
                held: Default::default(),
                held_attack: Default::default(),
                closing: false,
                font: HFONT::default(),
            })));
            let dpi = GetDpiForSystem().max(96) as i32;
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                w!("LiTo 原生登入"),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                600 * dpi / 96,
                820 * dpi / 96,
                None,
                None,
                Some(instance),
                Some((&*state as *const State).cast()),
            )?;
            let _window = WindowGuard(hwnd);
            let setup = state.0.borrow_mut().create(hwnd);
            if let Err(error) = setup {
                let _ = DestroyWindow(hwnd);
                return Err(error);
            }
            ensure!(
                SetTimer(Some(hwnd), 1, 20, None) != 0,
                "native login UI timer"
            );
            let _ = ShowWindow(hwnd, SW_SHOW);
            // A launcher using STARTF_USESHOWWINDOW/SW_HIDE overrides the first
            // ShowWindow call. This explicitly requested interactive route must
            // still present its window when launched by the hidden shell helper.
            if !IsWindowVisible(hwnd).as_bool() {
                let _ = ShowWindow(hwnd, SW_SHOW);
            }
            let _ = SetFocus(Some(state.0.borrow().control(CONNECT)));
            let mut message = MSG::default();
            loop {
                let read = GetMessageW(&mut message, None, 0, 0).0;
                if read == 0 {
                    break;
                }
                ensure!(read != -1, "native login message loop");
                // Handle child controls before their default combo-box key/wheel
                // behavior, so these inputs change pages rather than local rows.
                if message.hwnd == hwnd || IsChild(hwnd, message.hwnd).as_bool() {
                    let mut ui = state.0.borrow_mut();
                    if ui.character_paging_enabled() {
                        let turn = match (message.message, message.wParam.0) {
                            (WM_KEYDOWN, 0x21) => Some(Turn::Previous),
                            (WM_KEYDOWN, 0x22) => Some(Turn::Next),
                            _ => None,
                        };
                        if turn.is_some() || message.message == WM_MOUSEWHEEL {
                            let changed = if let Some(turn) = turn {
                                message.lParam.0 & (1 << 30) == 0 && ui.character_pages.turn(turn)
                            } else {
                                ui.character_pages.wheel((message.wParam.0 >> 16) as i16)
                            };
                            if changed {
                                send(ui.control(SOURCES), CB_SHOWDROPDOWN, WPARAM(0), LPARAM(0));
                                ui.refresh();
                            }
                            continue;
                        }
                    }
                }
                let world_focus = GetFocus() == hwnd;
                if !world_focus && matches!(message.message, WM_KEYDOWN | WM_KEYUP | WM_LBUTTONDOWN)
                {
                    let mut ui = state.0.borrow_mut();
                    ui.held.clear();
                    ui.held_attack.clear();
                }
                // Submit a received numeric NPC request from its quantity field.
                // The ordinary button path still validates owner, target and deadline.
                if message.message == WM_KEYDOWN && message.wParam.0 == 0x0d {
                    let quantity_field = {
                        let ui = state.0.borrow();
                        ui.rental.open && !ui.closing && !ui.inventory_view
                            && GetFocus() == ui.control(rental::QUANTITY)
                            && ui.controller.snapshot().merchant.input.as_ref().is_some_and(|input| input.mode == 0)
                    };
                    if quantity_field {
                        if message.lParam.0 & (1 << 30) == 0 {
                            let enabled = {
                                let ui = state.0.borrow();
                                IsWindowEnabled(ui.control(rental::ORDER)).as_bool()
                            };
                            if enabled { send(hwnd, WM_COMMAND, WPARAM(rental::ORDER as usize), LPARAM(0)); }
                        }
                        continue;
                    }
                }
                if message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {
                    let mut ui = state.0.borrow_mut();
                    if ui.inventory_view && (ui.item_selection.is_some() || ui.destroy_choice.is_some()) {
                        if let Some(selection) = ui.item_selection.as_mut() { selection.cancel(); }
                        ui.item_selection = None;
                        ui.destroy_choice = None;
                        ui.layout(hwnd, false);
                        ui.refresh_item_actions();
                        continue;
                    }
                }
                if matches!(message.message, WM_KEYDOWN | WM_SYSKEYDOWN) {
                    if let Some((slot, first)) = skill_ui::hotbar_key(message.message, message.wParam.0, message.lParam.0) {
                        let mut ui = state.0.borrow_mut();
                        if ui.skill_input_focus(GetFocus()) && ui.world_view && !ui.inventory_view && !ui.rental.open {
                            if first {
                                let result = ui.skill_slot(slot);
                                ui.refresh();
                                if let Err(e) = result { ui.text(STATUS, &format!("{e:#}")); }
                            }
                            continue;
                        }
                    }
                }
                if world_focus && message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {
                    let mut ui = state.0.borrow_mut();
                    if ui.world_view
                        && !ui.inventory_view && !(ui.rental.open || ui.companions.open)
                        && ui.controller.snapshot().phase.active_world()
                    {
                        ui.held.clear();
                        ui.held_attack.clear();
                        ui.target = None;
                        ui.ground = None;
                        ui.item_cursor = None;
                        continue;
                    }
                }
                if message.message == WM_KEYDOWN && matches!(message.wParam.0, 0x70 | 0x71 | 0x72) {
                    let mut ui = state.0.borrow_mut();
                    if ui.world_view
                        && !ui.inventory_view && !(ui.rental.open || ui.companions.open)
                        && ui.controller.snapshot().phase.active_world()
                    {
                        if message.lParam.0 & (1 << 30) == 0 {
                            ui.held.clear();
                            ui.held_attack.clear();
                            let skill = match message.wParam.0 {
                                0x70 => 1,
                                0x71 => 3,
                                _ => 4,
                            };
                            let result = ui.cast_skill(skill);
                            if let Err(e) = result {
                                ui.text(STATUS, &format!("{e:#}"));
                            }
                        }
                        continue;
                    }
                }
                if world_focus && message.message==WM_KEYDOWN && message.wParam.0==0x50 {
                    let mut ui=state.0.borrow_mut();
                    if ui.world_view&&!ui.inventory_view&&!(ui.rental.open || ui.companions.open)&&ui.controller.snapshot().phase.active_world() {
                        if message.lParam.0&(1<<30)==0 {
                            let result=ui.enter_nearby_portal();
                            if let Err(e)=result{ui.text(STATUS,&format!("{e:#}"));}
                        }
                        continue;
                    }
                }
                if world_focus && message.message==WM_KEYDOWN && message.wParam.0==0x4f {
                    let mut ui=state.0.borrow_mut();
                    if ui.world_view&&!ui.inventory_view&&!(ui.rental.open || ui.companions.open)&&ui.controller.snapshot().phase.active_world() {
                        if message.lParam.0&(1<<30)==0 {
                            if let Err(e)=ui.toggle_selected_door(){ui.text(STATUS,&format!("{e:#}"));}
                        }
                        continue;
                    }
                }
                if world_focus
                    && matches!(message.message, WM_KEYDOWN | WM_KEYUP)
                    && message.wParam.0 == 0x20
                {
                    let mut ui = state.0.borrow_mut();
                    if ui.world_view
                        && !ui.inventory_view && !(ui.rental.open || ui.companions.open)
                        && ui.controller.snapshot().phase.active_world()
                    {
                        if message.message == WM_KEYUP {
                            ui.held_attack.clear();
                        } else if message.lParam.0 & (1 << 30) == 0 {
                            if let Err(e) = ui.attack_target() {
                                ui.text(STATUS, &format!("{e:#}"));
                            }
                        }
                        continue;
                    }
                }
                if world_focus
                    && matches!(message.message, WM_KEYDOWN | WM_KEYUP)
                    && super::native_navigation::heading(message.wParam.0).is_some()
                {
                    let mut ui = state.0.borrow_mut();
                    if ui.world_view
                        && !ui.inventory_view && !(ui.rental.open || ui.companions.open)
                        && ui.controller.snapshot().phase.active_world()
                    {
                        if ui.controller.snapshot().observer {
                            ui.held.clear();ui.held_attack.clear();
                            if message.message==WM_KEYDOWN {
                                if let Some(heading)=super::native_navigation::heading(message.wParam.0) {
                                    if let Err(error)=ui.controller.pan_observer(heading){ui.text(STATUS,&format!("{error:#}"));}
                                }
                            }
                            continue;
                        }
                        if message.message == WM_KEYUP {
                            ui.held.release(message.wParam.0);
                        } else if (message.lParam.0 & (1 << 30)) == 0 {
                            ui.held_attack.clear();
                            ui.held.press(message.wParam.0);
                        }
                        continue;
                    }
                }
                if !IsDialogMessageW(hwnd, &message).as_bool() {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
            ensure!(
                state.0.borrow().controller.is_quiescent()
                    && state.0.borrow().inventory_names.quiescent()
                    && state.0.borrow().inventory_icons.quiescent()
                    && state.0.borrow().skill_icons.quiescent()
                    && state.0.borrow().rental.documents.quiescent(),
                "native login closed before worker cleanup"
            );
            Ok(())
        })();
        if !previous.is_invalid() {
            SetThreadDpiAwarenessContext(previous);
        }
        result.context("原生登入視窗")
    }
}
