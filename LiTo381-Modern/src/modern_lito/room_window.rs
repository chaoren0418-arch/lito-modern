//! Same-thread modeless, owned room form. No socket, worker, native BIN or world
//! start is created here. Submission is an intent, never a server acknowledgement.
use super::matchmaking_protocol as matching;
use super::rooms::{Action, Common, Draft, ModeOption, PublicDraft, Request};
use anyhow::{ensure, Result};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            CreateFontW, DeleteObject, GetSysColorBrush, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
            COLOR_BTNFACE, DEFAULT_CHARSET, HFONT, OUT_DEFAULT_PRECIS,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{EM_EMPTYUNDOBUFFER, EM_SETLIMITTEXT},
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            Input::KeyboardAndMouse::{EnableWindow, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};
use zeroize::Zeroizing;

pub enum FormOutcome {
    Submitted(Request),
    Matchmaking(matching::Request),
    Cancelled,
    Failed(&'static str),
}

const PRIVATE_FLAG: u8 = 4;
const UPPER_FLAG: u8 = 16;
const EQUAL_FLAG: u8 = 32;
const MIN_WIDTH: i32 = 580;
const MIN_HEIGHT: i32 = 620;
const JOIN_HEIGHT: i32 = 320;
const STYLE: WINDOW_STYLE = WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_CLIPCHILDREN.0);
const EX_STYLE: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
const FIRST_ID: usize = 100;
const INFO: usize = 0;
const NAME_LABEL: usize = 1;
const NAME: usize = 2;
const CAPACITY_LABEL: usize = 3;
const CAPACITY: usize = 4;
const OPTION_LABEL: usize = 5;
const OPTION: usize = 6;
const OPTION_INFO: usize = 7;
const PRIVATE: usize = 8;
const PASSWORD_LABEL: usize = 9;
const PASSWORD: usize = 10;
const UPPER: usize = 11;
const EQUAL: usize = 12;
const STATUS: usize = 13;
const SUBMIT: usize = 14;
const CANCEL: usize = 15;
const COUNT: usize = 16;
static NEXT_CLASS: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct Rect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}
fn scale(value: i32, dpi: u32) -> i32 {
    ((i64::from(value) * i64::from(dpi.max(96)) + 48) / 96) as i32
}
fn layout(width: i32, height: i32, dpi: u32) -> [Rect; COUNT] {
    let dpi = dpi.clamp(96, 768);
    let margin = scale(16, dpi);
    let content = (width - margin * 2).max(1);
    let mut boxes = [Rect::default(); COUNT];
    for (index, y, h) in [
        (INFO, 16, 48),
        (NAME_LABEL, 76, 22),
        (NAME, 100, 28),
        (CAPACITY_LABEL, 138, 22),
        (OPTION_LABEL, 202, 22),
        (OPTION, 226, 240), // Native combo dropdown extent, not its collapsed height.
        (OPTION_INFO, 264, 60),
        (UPPER, 340, 28),
        (EQUAL, 372, 28),
        (PASSWORD_LABEL, 412, 22),
        (PASSWORD, 438, 28),
        (STATUS, 484, 64),
    ] {
        boxes[index] = Rect {
            x: margin,
            y: scale(y, dpi),
            width: content,
            height: scale(h, dpi),
        };
    }
    boxes[CAPACITY] = Rect {
        x: margin,
        y: scale(162, dpi),
        width: scale(100, dpi),
        height: scale(28, dpi),
    };
    boxes[PRIVATE] = Rect {
        x: scale(144, dpi),
        y: scale(162, dpi),
        width: (width - scale(160, dpi)).max(1),
        height: scale(28, dpi),
    };
    for (index, offset) in [(SUBMIT, 248), (CANCEL, 120)] {
        boxes[index] = Rect {
            x: width - scale(offset, dpi),
            y: height - scale(46, dpi),
            width: scale(104, dpi),
            height: scale(30, dpi),
        };
    }
    boxes
}
fn visible(index: usize, join: bool) -> bool {
    !join
        || matches!(
            index,
            INFO | PASSWORD_LABEL | PASSWORD | STATUS | SUBMIT | CANCEL
        )
}
fn window_layout(width: i32, height: i32, dpi: u32, join: bool) -> [Rect; COUNT] {
    let mut result = layout(width, height, dpi);
    if join {
        let dpi = dpi.clamp(96, 768);
        let margin = scale(16, dpi);
        let content = (width - margin * 2).max(1);
        for (index, y, h) in [
            (INFO, 16, 76),
            (PASSWORD_LABEL, 108, 22),
            (PASSWORD, 134, 28),
            (STATUS, 184, 64),
        ] {
            result[index] = Rect {
                x: margin,
                y: scale(y, dpi),
                width: content,
                height: scale(h, dpi),
            };
        }
    }
    result
}

fn valid_name(value: &str) -> bool {
    !value.trim().is_empty() && value.len() <= 64 && !value.chars().any(char::is_control)
}
fn form_visible(index: usize, join: bool, matchmaking: bool) -> bool {
    if matchmaking {
        matches!(
            index,
            INFO | CAPACITY_LABEL
                | CAPACITY
                | OPTION_LABEL
                | OPTION
                | OPTION_INFO
                | STATUS
                | SUBMIT
                | CANCEL
        )
    } else {
        visible(index, join)
    }
}
fn form_layout(width: i32, height: i32, dpi: u32, join: bool, matchmaking: bool) -> [Rect; COUNT] {
    let mut boxes = window_layout(width, height, dpi, join);
    if matchmaking {
        for (index, y, h) in [
            (CAPACITY_LABEL, 84, 22),
            (CAPACITY, 110, 28),
            (OPTION_LABEL, 158, 22),
            (OPTION, 184, 240),
            (OPTION_INFO, 226, 88),
            (STATUS, 340, 96),
        ] {
            boxes[index].y = scale(y, dpi.clamp(96, 768));
            boxes[index].height = scale(h, dpi.clamp(96, 768));
        }
    }
    boxes
}
fn matching_criteria(
    options: &[ModeOption],
    selected: usize,
    capacity: &str,
) -> std::result::Result<matching::Criteria, &'static str> {
    let option = options
        .get(selected)
        .ok_or("請選擇已綁定共用資源的模式／地圖。")?;
    if capacity.is_empty() || capacity.len() > 3 || !capacity.bytes().all(|b| b.is_ascii_digit()) {
        return Err("配對人數必須為整數，並符合此模式的人數限制。");
    }
    let players = capacity.parse().map_err(|_| "配對人數無效。")?;
    let criteria = matching::Criteria {
        mode: option.mode,
        map_id: option.map_id,
        players,
    };
    if !criteria.accepts(option) {
        return Err("配對人數不符合模式／地圖限制；分隊模式需偶數人，攻城至少 30 人。");
    }
    Ok(criteria)
}
fn validate_options(options: &[ModeOption]) -> std::result::Result<(), &'static str> {
    if options.is_empty() || options.len() > 40 {
        return Err("伺服器未提供有效的模式／地圖選項。");
    }
    for (index, option) in options.iter().enumerate() {
        if option.mode > 5
            || option.map_id == 0
            || option.map_id > i32::MAX as u32
            || option.minimum_players == 0
            || option.minimum_players > 100
            || !(option.minimum_players.max(2)..=100).contains(&option.maximum_players)
            || !(1..=86400).contains(&option.seconds)
            || option.mode_name.trim().is_empty()
            || option.mode_name.len() > 64
            || option.mode_name.chars().any(char::is_control)
            || options[..index]
                .iter()
                .any(|prior| (prior.mode, prior.map_id) == (option.mode, option.map_id))
        {
            return Err("伺服器模式／地圖選項格式不一致。");
        }
    }
    Ok(())
}
fn initial_option(
    options: &[ModeOption],
    initial: Option<&PublicDraft>,
) -> std::result::Result<usize, &'static str> {
    if let Some(initial) = initial {
        // Keep the exact current pair when editing, including an unbound room.
        // The owner can select another map; never silently rewrite its settings.
        options
            .iter()
            .position(|o| (o.mode, o.map_id) == (initial.mode, initial.map_id))
            .ok_or("目前房間的模式／地圖已不在伺服器清單中，請重新整理。")
    } else {
        options
            .iter()
            .position(|o| o.world_available)
            .ok_or("伺服器尚未提供可進場的地圖，無法建立或配對房間。")
    }
}
fn default_capacity(option: &ModeOption) -> u16 {
    8.max(option.minimum_players).min(option.maximum_players)
}
fn validated_public(
    options: &[ModeOption],
    selected: usize,
    name: &str,
    capacity: &str,
    private: bool,
    upper: bool,
    equal: bool,
    password: &str,
) -> std::result::Result<PublicDraft, &'static str> {
    let option = options
        .get(selected)
        .ok_or("請選擇伺服器提供的模式／地圖組合。")?;
    if !option.world_available {
        return Err("此模式／地圖尚未開放進場，請選擇已綁定共用資源的地圖。");
    }
    if !valid_name(name) {
        return Err("房名需為 1–64 UTF-8 位元組，不可空白或包含控制字元。");
    }
    if capacity.is_empty()
        || capacity.len() > 3
        || !capacity.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err("房間人數請輸入 2–100 的整數。");
    }
    let capacity = capacity
        .parse::<u16>()
        .map_err(|_| "房間人數請輸入 2–100 的整數。")?;
    if !(2..=100).contains(&capacity) {
        return Err("房間人數請輸入 2–100 的整數。");
    }
    if capacity < option.minimum_players {
        return Err("房間人數低於此模式的最低開局人數，請依下方人數提示調整。");
    }
    if capacity > option.maximum_players {
        return Err("房間人數超出此模式／地圖的伺服器上限，請依下方人數提示調整。");
    }
    if password.len() > 64 || password.chars().any(char::is_control) {
        return Err("房間密碼最多 64 UTF-8 位元組，不可包含控制字元。");
    }
    if private && password.is_empty() {
        return Err("私人房間必須輸入密碼；修改私人房間時也需重新輸入，不會沿用未公開的密碼。");
    }
    if !private && !password.is_empty() {
        return Err("公開房間不可帶有密碼，請明確選擇私人房間或清除密碼。");
    }
    Ok(PublicDraft {
        capacity,
        mode: option.mode,
        attributes: (if private { PRIVATE_FLAG } else { 0 })
            | (if upper { UPPER_FLAG } else { 0 })
            | (if equal { EQUAL_FLAG } else { 0 }),
        map_id: option.map_id,
        name: name.to_owned(),
    })
}

struct State {
    hwnd: Cell<Option<HWND>>,
    controls: Cell<Option<[HWND; COUNT]>>,
    common: Common,
    matchmaking: bool,
    options: Vec<ModeOption>,
    outcome: RefCell<Option<FormOutcome>>,
    closing: Cell<bool>,
    rendering: Cell<bool>,
    font: Cell<HFONT>,
}
struct Class {
    name: Vec<u16>,
    instance: HINSTANCE,
}
impl Drop for Class {
    fn drop(&mut self) {
        unsafe {
            let _ = UnregisterClassW(PCWSTR(self.name.as_ptr()), Some(self.instance));
        }
    }
}

/// Owned by the parent's UI thread. No internal message loop and no WM_QUIT.
pub struct RoomForm {
    state: Box<State>,
    _class: Class,
    _ui_thread: PhantomData<Rc<()>>, // HWND/state must never be sent to another thread.
}
impl RoomForm {
    pub fn open(
        parent: HWND,
        common: Common,
        options: Vec<ModeOption>,
        initial: Option<PublicDraft>,
    ) -> Result<Self> {
        Self::open_kind(parent, common, options, initial, false)
    }
    pub fn open_matchmaking(
        parent: HWND,
        common: Common,
        options: Vec<ModeOption>,
    ) -> Result<Self> {
        ensure!(
            common.action == Action::Read,
            "matchmaking requires lobby read scope"
        );
        let options = options.into_iter().filter(|o| o.world_available).collect();
        Self::open_kind(parent, common, options, None, true)
    }
    fn open_kind(
        parent: HWND,
        common: Common,
        options: Vec<ModeOption>,
        initial: Option<PublicDraft>,
        matchmaking: bool,
    ) -> Result<Self> {
        common.validate()?;
        let join = common.action == Action::Join;
        if !join {
            validate_options(&options).map_err(anyhow::Error::msg)?;
        }
        ensure!(
            matchmaking
                || matches!(
                    common.action,
                    Action::Create | Action::Configure | Action::Join
                ),
            "room form requires create/configure/join intent"
        );
        ensure!(
            join || (common.action == Action::Configure) == initial.is_some(),
            "room configure requires current public settings"
        );
        if let Some(initial) = &initial {
            initial.validate()?;
        }
        let selected = if join {
            0
        } else {
            initial_option(&options, initial.as_ref()).map_err(anyhow::Error::msg)?
        };
        let number = NEXT_CLASS
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .map_err(|_| anyhow::anyhow!("room form class IDs exhausted"))?;
        let name = wide(&format!("ModernLiToRoomForm{number}"));
        unsafe {
            ensure!(
                IsWindow(Some(parent)).as_bool(),
                "room form owner is not a window"
            );
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            ensure!(
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    lpszClassName: PCWSTR(name.as_ptr()),
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                    ..Default::default()
                }) != 0,
                "register room form"
            );
            let mut form = Self {
                state: Box::new(State {
                    hwnd: Cell::new(None),
                    controls: Cell::new(None),
                    common,
                    matchmaking,
                    options,
                    outcome: RefCell::new(None),
                    closing: Cell::new(false),
                    rendering: Cell::new(true),
                    font: Cell::new(HFONT::default()),
                }),
                _class: Class { name, instance },
                _ui_thread: PhantomData,
            };
            let hwnd = CreateWindowExW(
                EX_STYLE,
                PCWSTR(form._class.name.as_ptr()),
                if matchmaking {
                    w!("LiTo — 配對")
                } else if join {
                    w!("LiTo — 加入戰鬥室")
                } else if common.action == Action::Create {
                    w!("LiTo — 建立戰鬥室")
                } else {
                    w!("LiTo — 修改房間設定")
                },
                STYLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                MIN_WIDTH,
                MIN_HEIGHT,
                Some(parent),
                None,
                Some(instance),
                Some((&mut *form.state as *mut State).cast()),
            )?;
            form.state.hwnd.set(Some(hwnd));
            let controls = create_controls(hwnd, instance)?;
            form.state.controls.set(Some(controls));
            if join {
                for (index, control) in controls.iter().enumerate() {
                    if !visible(index, true) {
                        let _ = ShowWindow(*control, SW_HIDE);
                    }
                }
                let room_name = initial
                    .as_ref()
                    .map(|draft| draft.name.as_str())
                    .unwrap_or("選定房間");
                set_text(
                    controls[INFO],
                    &format!(
                        "加入：{room_name}\r\n房間 ID {} · 版本 {}\r\n全員準備後，僅已綁定共用資源的地圖可啟動世界。",
                        common.room_id, common.expected_revision
                    ),
                )?;
                set_text(
                    controls[STATUS],
                    "私人房間請輸入密碼；公開房間請留空。\r\n輸入內容不會保存，加入結果由伺服器裁決。",
                )?;
                set_text(controls[SUBMIT], "提交加入")?;
            } else {
                let public = initial.unwrap_or_else(|| PublicDraft {
                    capacity: default_capacity(&form.state.options[selected]),
                    mode: form.state.options[selected].mode,
                    attributes: 0,
                    map_id: form.state.options[selected].map_id,
                    name: "新戰鬥室".into(),
                });
                set_text(
                    controls[INFO],
                    "模式與地圖來自伺服器；此表單僅提交設定。\r\n世界資源與本場可用功能由伺服器確認。",
                )?;
                set_text(controls[NAME], &public.name)?;
                set_text(controls[CAPACITY], &public.capacity.to_string())?;
                for (index, option) in form.state.options.iter().enumerate() {
                    let label = wide(&format!(
                        "{} · 地圖 {} · {}{}",
                        option.mode_name,
                        option.map_id,
                        if option.team_based {
                            "分隊"
                        } else {
                            "個人"
                        },
                        if option.world_available {
                            ""
                        } else {
                            " · 未開放進場"
                        }
                    ));
                    ensure!(
                        SendMessageW(
                            controls[OPTION],
                            CB_ADDSTRING,
                            None,
                            Some(LPARAM(label.as_ptr() as isize))
                        )
                        .0 == index as isize,
                        "room option was not inserted in server order"
                    );
                }
                ensure!(
                    SendMessageW(controls[OPTION], CB_SETCURSEL, Some(WPARAM(selected)), None).0
                        == selected as isize,
                    "select room option"
                );
                for (index, flag) in [
                    (PRIVATE, PRIVATE_FLAG),
                    (UPPER, UPPER_FLAG),
                    (EQUAL, EQUAL_FLAG),
                ] {
                    let _ = SendMessageW(
                        controls[index],
                        BM_SETCHECK,
                        Some(WPARAM(usize::from(public.attributes & flag != 0))),
                        None,
                    );
                }
                set_text(
                    controls[STATUS],
                    if public.attributes & PRIVATE_FLAG != 0 {
                        "私人房間：請重新輸入密碼；本表單不會讀取或保存舊密碼。"
                    } else {
                        "公開房間不需密碼。關閉或取消不會送出任何設定。"
                    },
                )?;
                refresh_option(&form.state)?;
                let _ = EnableWindow(controls[PASSWORD], public.attributes & PRIVATE_FLAG != 0);
            }
            if matchmaking {
                for (index, control) in controls.iter().enumerate() {
                    if !form_visible(index, false, true) {
                        let _ = ShowWindow(*control, SW_HIDE);
                    }
                }
                let option = &form.state.options[selected];
                let mut players = option
                    .minimum_players
                    .max(if option.mode == 3 { 30 } else { 2 });
                if matches!(option.mode, 1 | 2 | 3 | 5) && players % 2 != 0 {
                    players += 1;
                }
                set_text(controls[CAPACITY], &players.to_string())?;
                set_text(controls[CAPACITY_LABEL], "配對真人總人數（分隊模式需偶數）")?;
                set_text(
                    controls[INFO],
                    "配對只送出模式、地圖及人數。\r\n戰績由伺服器讀取，排隊成功不代表已入房。",
                )?;
                set_text(controls[STATUS], "配對成功後將進入房間；玩家仍須自行準備。\r\n關閉此表單不會排隊，排隊後可在大廳取消。")?;
                set_text(controls[SUBMIT], "開始配對")?;
            }
            replace_font(hwnd, &form.state)?;
            let dpi = GetDpiForWindow(hwnd).clamp(96, 768);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: scale(MIN_WIDTH, dpi),
                bottom: scale(if join { JOIN_HEIGHT } else { MIN_HEIGHT }, dpi),
            };
            AdjustWindowRectExForDpi(&mut rect, STYLE, false, EX_STYLE, dpi)?;
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            resize(hwnd, &form.state)?;
            form.state.rendering.set(false);
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetFocus(Some(
                controls[if matchmaking {
                    CAPACITY
                } else if join {
                    PASSWORD
                } else {
                    NAME
                }],
            ));
            Ok(form)
        }
    }
    /// Call without a parent RefCell borrow; IsDialogMessage may synchronously
    /// send control/owner notifications. This convenience method owns no borrow.
    pub fn pre_translate(&self, message: &MSG) -> bool {
        self.window_handle()
            .is_some_and(|hwnd| Self::pre_translate_window(hwnd, message))
    }
    pub fn pre_translate_window(hwnd: HWND, message: &MSG) -> bool {
        unsafe {
            if !IsWindow(Some(hwnd)).as_bool()
                || !(message.hwnd == hwnd || IsChild(hwnd, message.hwnd).as_bool())
            {
                return false;
            }
            if message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {
                let _ = SendMessageW(hwnd, WM_CLOSE, None, None);
                true
            } else {
                IsDialogMessageW(hwnd, message).as_bool()
            }
        }
    }
    pub fn window_handle(&self) -> Option<HWND> {
        self.state.hwnd.get()
    }
    /// Pure state poll: no native sends, mutex, worker or I/O. The request is
    /// delivered at most once, only after WM_NCDESTROY has retired the form.
    pub fn poll(&mut self) -> Option<FormOutcome> {
        if self.is_quiescent() {
            self.state.outcome.borrow_mut().take()
        } else {
            None
        }
    }
    pub fn is_quiescent(&self) -> bool {
        self.state.hwnd.get().is_none()
    }
    /// Also drops a submitted-but-unpolled request. Parent shutdown cannot turn
    /// a pending room form intent into a late network command.
    pub fn cancel(&mut self) {
        self.state.outcome.borrow_mut().take();
        self.state.closing.set(false);
        unsafe {
            finish(&self.state, FormOutcome::Cancelled);
        }
    }
}
impl Drop for RoomForm {
    fn drop(&mut self) {
        self.cancel();
        unsafe {
            if let Some(hwnd) = self.state.hwnd.get() {
                // Exceptional DestroyWindow failure must not leave a dangling
                // user-data pointer when Rust drops the state. The owner still
                // owns this HWND and will destroy it with its other owned windows.
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                let _ = DestroyWindow(hwnd);
            }
            let font = self.state.font.replace(HFONT::default());
            if !font.0.is_null() {
                let _ = DeleteObject(font.into());
            }
        }
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
unsafe fn set_text(hwnd: HWND, text: &str) -> Result<()> {
    ensure!(!text.contains('\0'), "room UI text contains NUL");
    let text = wide(text);
    ensure!(text.len() <= 4097, "room UI text exceeds limit");
    SetWindowTextW(hwnd, PCWSTR(text.as_ptr()))?;
    ensure!(
        GetWindowTextLengthW(hwnd) as usize == text.len() - 1,
        "room UI text truncated"
    );
    let mut actual = vec![0; text.len()];
    ensure!(
        GetWindowTextW(hwnd, &mut actual) as usize == text.len() - 1 && actual == text,
        "room UI text mismatch"
    );
    Ok(())
}
unsafe fn read_edit(
    hwnd: HWND,
    maximum: usize,
) -> std::result::Result<Zeroizing<String>, &'static str> {
    let count = GetWindowTextLengthW(hwnd);
    if count < 0 || count as usize > maximum {
        return Err("輸入長度超出限制。");
    }
    let mut bytes = Zeroizing::new(vec![0u16; count as usize + 1]);
    if GetWindowTextW(hwnd, &mut bytes) != count {
        return Err("無法完整讀取輸入內容。");
    }
    if char::decode_utf16(bytes[..count as usize].iter().copied()).any(|unit| unit.is_err()) {
        return Err("輸入不是有效 Unicode。");
    }
    String::from_utf16(&bytes[..count as usize])
        .map(Zeroizing::new)
        .map_err(|_| "輸入不是有效 Unicode。")
}
fn control_id(index: usize) -> usize {
    match index {
        SUBMIT => 1,
        CANCEL => 2,
        _ => FIRST_ID + index,
    }
}
unsafe fn create_controls(parent: HWND, instance: HINSTANCE) -> Result<[HWND; COUNT]> {
    let mut controls = [HWND::default(); COUNT];
    for index in 0..COUNT {
        let (class, label, style, ex) = match index {
            NAME | CAPACITY | PASSWORD => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0
                    | ES_AUTOHSCROLL as u32
                    | if index == PASSWORD {
                        ES_PASSWORD as u32
                    } else if index == CAPACITY {
                        ES_NUMBER as u32
                    } else {
                        0
                    },
                WS_EX_CLIENTEDGE,
            ),
            OPTION => (
                w!("COMBOBOX"),
                "",
                WS_TABSTOP.0 | WS_VSCROLL.0 | CBS_DROPDOWNLIST as u32,
                WINDOW_EX_STYLE::default(),
            ),
            PRIVATE | UPPER | EQUAL => (
                w!("BUTTON"),
                match index {
                    PRIVATE => "私人房間（需要密碼）",
                    UPPER => "允許高階玩家",
                    _ => "平等條件",
                },
                WS_TABSTOP.0 | BS_AUTOCHECKBOX as u32,
                WINDOW_EX_STYLE::default(),
            ),
            SUBMIT | CANCEL => (
                w!("BUTTON"),
                if index == SUBMIT {
                    "提交設定"
                } else {
                    "取消"
                },
                WS_TABSTOP.0
                    | if index == SUBMIT {
                        BS_DEFPUSHBUTTON as u32
                    } else {
                        BS_PUSHBUTTON as u32
                    },
                WINDOW_EX_STYLE::default(),
            ),
            _ => (
                w!("STATIC"),
                match index {
                    NAME_LABEL => "房間名稱",
                    CAPACITY_LABEL => "人數上限（2–100）",
                    OPTION_LABEL => "模式／地圖（伺服器提供的合法配對）",
                    PASSWORD_LABEL => "房間密碼（最多 64 UTF-8 位元組，不會保存到檔案）",
                    _ => "",
                },
                0x80,
                WINDOW_EX_STYLE::default(),
            ),
        };
        let label = wide(label);
        controls[index] = CreateWindowExW(
            ex,
            class,
            PCWSTR(label.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU(control_id(index) as *mut _)),
            Some(instance),
            None,
        )?;
    }
    for (index, limit) in [(NAME, 64), (CAPACITY, 3), (PASSWORD, 64)] {
        let _ = SendMessageW(controls[index], EM_SETLIMITTEXT, Some(WPARAM(limit)), None);
    }
    Ok(controls)
}
unsafe fn replace_font(hwnd: HWND, state: &State) -> Result<()> {
    let font = CreateFontW(
        -scale(16, GetDpiForWindow(hwnd).clamp(96, 768)),
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
    ensure!(!font.0.is_null(), "create room form font");
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
    let old = state.font.replace(font);
    if !old.0.is_null() {
        let _ = DeleteObject(old.into());
    }
    Ok(())
}
unsafe fn resize(hwnd: HWND, state: &State) -> Result<()> {
    let Some(controls) = state.controls.get() else {
        return Ok(());
    };
    let mut client = RECT::default();
    GetClientRect(hwnd, &mut client)?;
    if client.right <= 0 || client.bottom <= 0 {
        return Ok(());
    }
    let join = state.common.action == Action::Join;
    for (index, (control, rect)) in controls
        .into_iter()
        .zip(form_layout(
            client.right,
            client.bottom,
            GetDpiForWindow(hwnd),
            join,
            state.matchmaking,
        ))
        .enumerate()
    {
        if form_visible(index, join, state.matchmaking) {
            MoveWindow(control, rect.x, rect.y, rect.width, rect.height, true)?;
        }
    }
    Ok(())
}
unsafe fn checked(controls: &[HWND; COUNT], index: usize) -> bool {
    SendMessageW(controls[index], BM_GETCHECK, None, None).0 == 1
}
fn world_availability_notice(available: bool) -> &'static str {
    if available {
        "已綁定共用資源，可申請進場；可用戰鬥功能與結算依伺服器協商及本場設定。"
    } else {
        "尚未綁定共用地圖資源，不可提交；請選擇其他地圖或取消。"
    }
}
unsafe fn refresh_option(state: &State) -> Result<()> {
    let Some(controls) = state.controls.get() else {
        return Ok(());
    };
    let index = SendMessageW(controls[OPTION], CB_GETCURSEL, None, None).0;
    let option = usize::try_from(index)
        .ok()
        .and_then(|index| state.options.get(index));
    let text = option
        .map(|option| {
            format!(
                "最低開局人數 {} · 房間上限 {} 人 · 參考時間 {} 秒\r\n{}",
                option.minimum_players,
                option.maximum_players,
                option.seconds,
                world_availability_notice(option.world_available)
            )
        })
        .unwrap_or_else(|| "請選擇一組伺服器提供的模式／地圖。".into());
    set_text(controls[OPTION_INFO], &text)?;
    let _ = EnableWindow(controls[SUBMIT], option.is_some_and(|o| o.world_available));
    Ok(())
}
unsafe fn finish(state: &State, mut outcome: FormOutcome) {
    if state.closing.replace(true) {
        return;
    }
    if let Some(controls) = state.controls.get() {
        if set_text(controls[PASSWORD], "").is_err() {
            outcome = FormOutcome::Failed("無法清除房間密碼，已取消提交。");
        }
        let _ = SendMessageW(controls[PASSWORD], EM_EMPTYUNDOBUFFER, None, None);
    }
    *state.outcome.borrow_mut() = Some(outcome);
    if let Some(hwnd) = state.hwnd.get() {
        if DestroyWindow(hwnd).is_err() {
            *state.outcome.borrow_mut() =
                Some(FormOutcome::Failed("無法正常關閉房間表單，已取消提交。"));
            // Still live: never deliver Submitted or claim quiescence. Allow a
            // later owner cancellation / user close to retry instead of locking
            // this modeless form in an unresponsive closing state.
            state.closing.set(false);
        }
    }
}
unsafe fn submit(state: &State) {
    let Some(controls) = state.controls.get() else {
        return;
    };
    if state.matchmaking {
        let prepared = (|| -> std::result::Result<matching::Request, &'static str> {
            let capacity = read_edit(controls[CAPACITY], 3)?;
            let selected =
                usize::try_from(SendMessageW(controls[OPTION], CB_GETCURSEL, None, None).0)
                    .map_err(|_| "請選擇模式／地圖。")?;
            let criteria = matching_criteria(&state.options, selected, &capacity)?;
            Ok(matching::Request {
                context: state.common.context,
                channel_id: state.common.channel_id,
                action: matching::Action::Start,
                ticket_id: 0,
                criteria: Some(criteria),
            })
        })();
        match prepared {
            Ok(request) => finish(state, FormOutcome::Matchmaking(request)),
            Err(message) => {
                if set_text(controls[STATUS], message).is_err() {
                    finish(
                        state,
                        FormOutcome::Failed("無法顯示配對條件錯誤，已取消提交。"),
                    );
                }
            }
        }
        return;
    }
    let prepared = (|| -> std::result::Result<Request, &'static str> {
        if state.common.action == Action::Join {
            let mut password = read_edit(controls[PASSWORD], 64)?;
            return Request::join(state.common, std::mem::take(&mut *password))
                .map_err(|_| "加入房間的密碼／識別資料無效，尚未提交。");
        }
        let name = read_edit(controls[NAME], 64)?;
        let capacity = read_edit(controls[CAPACITY], 3)?;
        let mut password = read_edit(controls[PASSWORD], 64)?;
        let selected = usize::try_from(SendMessageW(controls[OPTION], CB_GETCURSEL, None, None).0)
            .map_err(|_| "請選擇伺服器提供的模式／地圖組合。")?;
        let public = validated_public(
            &state.options,
            selected,
            &name,
            &capacity,
            checked(&controls, PRIVATE),
            checked(&controls, UPPER),
            checked(&controls, EQUAL),
            &password,
        )?;
        let draft = Draft::new(public, std::mem::take(&mut *password))
            .map_err(|_| "房間設定格式無效，尚未提交。")?;
        Request::draft(state.common, draft)
            .map_err(|_| "房間操作已不符合目前狀態，請關閉後重新讀取房間。")
    })();
    match prepared {
        Ok(request) => finish(state, FormOutcome::Submitted(request)),
        Err(message) => {
            if set_text(controls[STATUS], message).is_err() {
                finish(
                    state,
                    FormOutcome::Failed("無法顯示房間表單錯誤，已取消提交。"),
                );
            }
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
    let pointer = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if pointer.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let state = &*pointer;
    match message {
        WM_COMMAND if !state.closing.get() && !state.rendering.get() => {
            let id = wparam.0 & 0xffff;
            let notification = (wparam.0 >> 16) as u32;
            if notification == BN_CLICKED {
                match id {
                    1 => submit(state),
                    2 => finish(state, FormOutcome::Cancelled),
                    value if value == control_id(PRIVATE) => {
                        if let Some(controls) = state.controls.get() {
                            let private = checked(&controls, PRIVATE);
                            if !private {
                                if set_text(controls[PASSWORD], "").is_err() {
                                    finish(
                                        state,
                                        FormOutcome::Failed("無法清除房間密碼，已取消提交。"),
                                    );
                                    return LRESULT(0);
                                }
                                let _ = SendMessageW(
                                    controls[PASSWORD],
                                    EM_EMPTYUNDOBUFFER,
                                    None,
                                    None,
                                );
                            }
                            let _ = EnableWindow(controls[PASSWORD], private);
                        }
                    }
                    _ => {}
                }
            } else if id == control_id(OPTION)
                && notification == CBN_SELCHANGE
                && refresh_option(state).is_err()
            {
                finish(
                    state,
                    FormOutcome::Failed("無法顯示模式／地圖選項，已取消提交。"),
                );
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            finish(state, FormOutcome::Cancelled);
            LRESULT(0)
        }
        WM_SIZE => {
            if resize(hwnd, state).is_err() {
                finish(state, FormOutcome::Failed("無法調整房間表單，已取消提交。"));
            }
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let dpi = GetDpiForWindow(hwnd).clamp(96, 768);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: scale(MIN_WIDTH, dpi),
                bottom: scale(
                    if state.common.action == Action::Join {
                        JOIN_HEIGHT
                    } else {
                        MIN_HEIGHT
                    },
                    dpi,
                ),
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
            if SetWindowPos(
                hwnd,
                None,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )
            .is_err()
                || replace_font(hwnd, state).is_err()
                || resize(hwnd, state).is_err()
            {
                finish(
                    state,
                    FormOutcome::Failed("無法調整房間表單 DPI，已取消提交。"),
                );
            }
            LRESULT(0)
        }
        WM_NCDESTROY => {
            state.hwnd.set(None);
            state.controls.set(None);
            let missing = state.outcome.borrow().is_none();
            if missing {
                *state.outcome.borrow_mut() = Some(FormOutcome::Cancelled);
            }
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn matchmaking_form_requires_qualified_map_and_server_capacity_without_room_secrets() {
        let mut options = super::super::rooms::tests::sample(
            super::super::matchmaking_protocol::tests::context(),
            false,
        )
        .options;
        let index = options
            .iter()
            .position(|o| o.mode == 1 && o.map_id == 514)
            .unwrap();
        assert!(matching_criteria(&options, index, "2").is_err());
        options[index].world_available = true;
        assert_eq!(
            matching_criteria(&options, index, "2").unwrap(),
            matching::Criteria {
                mode: 1,
                map_id: 514,
                players: 2
            }
        );
        for text in ["", " 2", "2 ", "3", "0", "101", "-2", "２", "1e2"] {
            assert!(matching_criteria(&options, index, text).is_err(), "{text}");
        }
        options[index].maximum_players = 2;
        assert!(matching_criteria(&options, index, "4").is_err());
        for i in [
            NAME,
            NAME_LABEL,
            PRIVATE,
            PASSWORD,
            PASSWORD_LABEL,
            UPPER,
            EQUAL,
        ] {
            assert!(!form_visible(i, false, true));
        }
        for dpi in [96, 120, 144, 192, 288, 768] {
            let boxes = form_layout(
                scale(MIN_WIDTH, dpi),
                scale(MIN_HEIGHT, dpi),
                dpi,
                false,
                true,
            );
            let shown = [
                INFO,
                CAPACITY_LABEL,
                CAPACITY,
                OPTION_LABEL,
                OPTION,
                OPTION_INFO,
                STATUS,
                SUBMIT,
                CANCEL,
            ];
            for (position, &i) in shown.iter().enumerate() {
                let mut a = boxes[i];
                if i == OPTION {
                    a.height = scale(28, dpi);
                }
                assert!(
                    form_visible(i, false, true)
                        && a.x >= 0
                        && a.y >= 0
                        && a.x + a.width <= scale(MIN_WIDTH, dpi)
                        && a.y + a.height <= scale(MIN_HEIGHT, dpi)
                );
                for &j in &shown[position + 1..] {
                    let mut b = boxes[j];
                    if j == OPTION {
                        b.height = scale(28, dpi);
                    }
                    assert!(
                        a.x + a.width <= b.x
                            || b.x + b.width <= a.x
                            || a.y + a.height <= b.y
                            || b.y + b.height <= a.y,
                        "overlap {i}/{j} at {dpi}"
                    );
                }
            }
        }
    }
    fn options() -> Vec<ModeOption> {
        vec![
            ModeOption {
                mode: 0,
                team_based: false,
                world_available: true,
                map_id: 565,
                minimum_players: 2,
                maximum_players: 100,
                seconds: 600,
                mode_name: "死鬥".into(),
            },
            ModeOption {
                mode: 1,
                team_based: true,
                world_available: true,
                map_id: 504,
                minimum_players: 4,
                maximum_players: 100,
                seconds: 900,
                mode_name: "搶旗".into(),
            },
        ]
    }
    #[test]
    fn unavailable_world_cannot_be_submitted_as_a_player_room() {
        let mut list = options();
        for option in &mut list {
            option.world_available = false;
        }
        assert!(validated_public(&list, 0, "房間", "8", false, false, false, "").is_err());
        list[0].world_available = true;
        assert!(validated_public(&list, 0, "房間", "8", false, false, false, "").is_ok());
        assert!(validated_public(&list, 1, "房間", "8", false, false, false, "").is_err());
    }
    #[test]
    fn capacity_below_start_minimum_cannot_create_an_unstartable_room() {
        let mut list = options();
        list[1].world_available = true;
        assert!(validated_public(&list, 1, "房間", "2", false, false, false, "").is_err());
        assert!(validated_public(&list, 1, "房間", "3", false, false, false, "").is_err());
        assert!(validated_public(&list, 1, "房間", "4", false, false, false, "").is_ok());
        list[1].mode = 3;
        list[1].map_id = 547;
        list[1].minimum_players = 30;
        assert!(validated_public(&list, 1, "攻城", "8", false, false, false, "").is_err());
        assert!(validated_public(&list, 1, "攻城", "30", false, false, false, "").is_ok());
    }
    #[test]
    fn new_room_chooses_first_bound_pair_but_edit_keeps_exact_existing_pair() {
        let mut list = options();
        list[0].world_available = false;
        assert_eq!(initial_option(&list, None).unwrap(), 1);
        let initial = PublicDraft {
            capacity: 8,
            mode: list[0].mode,
            map_id: list[0].map_id,
            attributes: 0,
            name: "既有房間".into(),
        };
        assert_eq!(initial_option(&list, Some(&initial)).unwrap(), 0);
        assert!(validated_public(&list, 0, "既有房間", "8", false, false, false, "").is_err());
        assert!(validated_public(&list, 1, "既有房間", "8", false, false, false, "").is_ok());
        list[1].world_available = false;
        assert!(initial_option(&list, None).is_err());
        assert!(initial_option(&[], None).is_err());
        assert!(initial_option(&list[1..], Some(&initial)).is_err());
        assert_eq!(initial_option(&list, Some(&initial)).unwrap(), 0);
    }
    #[test]
    fn default_capacity_obeys_both_server_start_minimum_and_map_maximum() {
        let mut list = options();
        for (minimum, maximum, expected) in [
            (1, 2, 2),
            (2, 50, 8),
            (4, 6, 6),
            (30, 100, 30),
            (50, 50, 50),
            (100, 100, 100),
        ] {
            list[0].minimum_players = minimum;
            list[0].maximum_players = maximum;
            validate_options(&list).unwrap();
            let capacity = default_capacity(&list[0]);
            assert_eq!(capacity, expected);
            assert!(validated_public(
                &list,
                0,
                "房間",
                &capacity.to_string(),
                false,
                false,
                false,
                ""
            )
            .is_ok());
        }
    }
    #[test]
    fn draft_uses_one_server_pair_and_preserves_public_flags() {
        let list = options();
        validate_options(&list).unwrap();
        let draft = validated_public(&list, 1, "測試房間", "100", true, true, true, "  ").unwrap();
        assert_eq!((draft.mode, draft.map_id), (1, 504));
        assert_eq!(draft.capacity, 100);
        assert_eq!(draft.attributes, 4 | 16 | 32);
        assert_eq!(draft.name, "測試房間");
        assert!(validated_public(&list, 2, "房間", "8", false, false, false, "").is_err());
    }
    #[test]
    fn private_edit_never_silently_reuses_or_removes_a_password() {
        let list = options();
        assert!(validated_public(&list, 0, "房间", "8", true, false, false, "").is_err());
        assert!(validated_public(&list, 0, "房间", "8", false, false, false, "secret").is_err());
        let public = validated_public(&list, 0, "房间", "8", false, false, false, "").unwrap();
        assert_eq!(public.attributes, 0);
    }
    #[test]
    fn server_capacity_is_enforced_when_creating_or_changing_map() {
        let mut list = options();
        list[0].maximum_players = 48;
        validate_options(&list).unwrap();
        assert!(validated_public(&list, 0, "房間", "48", false, false, false, "").is_ok());
        assert!(validated_public(&list, 0, "房間", "49", false, false, false, "").is_err());
        assert!(validated_public(&list, 1, "房間", "100", false, false, false, "").is_ok());
        for maximum in [0, 1, 101] {
            list[0].maximum_players = maximum;
            assert!(validate_options(&list).is_err());
        }
        list[0].maximum_players = 48;
        list[1].maximum_players = 3;
        assert!(validate_options(&list).is_err());
    }
    #[test]
    fn input_rejects_control_blank_utf8_overflow_and_noninteger_capacity() {
        let list = options();
        for name in ["", " ", "\u{a0}", "\u{2007}", "\u{202f}", "x\0y", "x\ny"] {
            assert!(validated_public(&list, 0, name, "8", false, false, false, "").is_err());
        }
        assert!(
            validated_public(&list, 0, &"界".repeat(22), "8", false, false, false, "").is_err()
        );
        assert!(validated_public(&list, 0, &"a".repeat(64), "8", false, false, false, "").is_ok());
        for capacity in ["", "1", "101", "2.0", "+2", " 8", "1000"] {
            assert!(validated_public(&list, 0, "room", capacity, false, false, false, "").is_err());
        }
        for password in ["\n".to_owned(), "界".repeat(22), "a".repeat(65)] {
            assert!(
                validated_public(&list, 0, "room", "8", true, false, false, &password).is_err()
            );
        }
    }
    #[test]
    fn world_availability_notice_reports_binding_without_claiming_all_combat_or_no_settlement() {
        assert!(world_availability_notice(true).contains("依伺服器協商及本場設定"));
        assert!(!world_availability_notice(true).contains("尚未完成"));
        assert!(!world_availability_notice(true).contains("完整戰鬥"));
        assert_eq!(
            world_availability_notice(false),
            "尚未綁定共用地圖資源，不可提交；請選擇其他地圖或取消。"
        );
    }
    #[test]
    fn malformed_options_are_not_silently_replaced_with_local_maps() {
        assert!(validate_options(&[]).is_err());
        let mut list = options();
        list.push(list[0].clone());
        assert!(validate_options(&list).is_err());
        let mut list = options();
        list[0].world_available = true;
        assert!(validate_options(&list).is_ok());
        let mut list = options();
        list[0].seconds = 0;
        assert!(validate_options(&list).is_err());
        let mut list = options();
        list[0].mode_name = "\0".into();
        assert!(validate_options(&list).is_err());
        let mut list = options();
        list[0].mode = 5;
        list[0].map_id = 510;
        list[0].team_based = true;
        assert!(validate_options(&list).is_ok());
        let mut list = options();
        list[0].map_id = 0;
        assert!(validate_options(&list).is_err());
    }
    #[test]
    fn submitted_draft_and_join_keep_the_exact_common_identity_and_revision() {
        use super::super::application_protocol::{CharacterKey, SelectionContext};
        let context = SelectionContext {
            key: CharacterKey {
                source_id: 42,
                object_id: 4_294_967_299,
            },
            epoch: u64::MAX,
        };
        let common = Common {
            context,
            channel_id: 3,
            action: Action::Configure,
            value: 0,
            room_id: 4_000_000_000,
            expected_revision: 99,
        };
        let public =
            validated_public(&options(), 0, "private", "8", true, true, false, "secret").unwrap();
        let request =
            Request::draft(common, Draft::new(public.clone(), "secret".into()).unwrap()).unwrap();
        assert_eq!(request.common, common);
        assert_eq!(request.expectation().draft, Some(public));
        let join = Common {
            action: Action::Join,
            ..common
        };
        for password in ["", "  ", "秘密"] {
            let request = Request::join(join, password.into()).unwrap();
            assert_eq!(request.common, join);
            assert!(request.expectation().draft.is_none());
        }
        assert!(Request::join(join, "x\ny".into()).is_err());
    }
    #[test]
    fn layout_is_client_relative_and_scaled_without_overlapping_collapsed_controls() {
        for dpi in [96, 120, 144, 192, 288] {
            let width = scale(MIN_WIDTH, dpi);
            let height = scale(MIN_HEIGHT, dpi);
            let rects = layout(width, height, dpi);
            for (index, rect) in rects.iter().enumerate() {
                assert!(
                    rect.x >= 0 && rect.y >= 0 && rect.width > 0 && rect.height > 0,
                    "{index}"
                );
                assert!(
                    rect.x + rect.width <= width && rect.y + rect.height <= height,
                    "{index}"
                );
            }
            assert_eq!(rects[NAME].x, scale(16, dpi));
            for left in 0..COUNT {
                for right in left + 1..COUNT {
                    let mut a = rects[left];
                    let mut b = rects[right];
                    if left == OPTION {
                        a.height = scale(28, dpi);
                    }
                    if right == OPTION {
                        b.height = scale(28, dpi);
                    }
                    assert!(
                        a.x + a.width <= b.x
                            || b.x + b.width <= a.x
                            || a.y + a.height <= b.y
                            || b.y + b.height <= a.y,
                        "{left}/{right} DPI {dpi}"
                    );
                }
            }
        }
    }
    #[test]
    fn join_layout_exposes_no_editable_room_settings() {
        for dpi in [96, 120, 144, 192, 288] {
            let width = scale(MIN_WIDTH, dpi);
            let height = scale(JOIN_HEIGHT, dpi);
            let boxes = window_layout(width, height, dpi, true);
            for index in 0..COUNT {
                assert_eq!(
                    visible(index, true),
                    matches!(
                        index,
                        INFO | PASSWORD_LABEL | PASSWORD | STATUS | SUBMIT | CANCEL
                    )
                );
                if visible(index, true) {
                    let r = boxes[index];
                    assert!(
                        r.x >= 0 && r.y >= 0 && r.x + r.width <= width && r.y + r.height <= height
                    );
                }
            }
            assert!(boxes[STATUS].y + boxes[STATUS].height <= boxes[SUBMIT].y);
        }
    }
}
