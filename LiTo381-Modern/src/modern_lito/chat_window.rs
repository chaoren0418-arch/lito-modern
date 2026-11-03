//! Same-EXE owned native chat panel. No network, files, optimistic messages,
//! game-key routing or saved drafts. The login controller owns all authority.
use super::filtered_chat_protocol::Kind as PublicKind;
use super::{
    chat_protocol,
    chat_state::{Delivery, State as ChatState, Token},
};
use anyhow::{ensure, Result};
use std::{
    cell::{Cell, RefCell},
    sync::atomic::{AtomicU64, Ordering},
};
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
            Controls::{
                EM_EMPTYUNDOBUFFER, EM_GETFIRSTVISIBLELINE, EM_GETSEL, EM_LINESCROLL,
                EM_SCROLLCARET, EM_SETLIMITTEXT, EM_SETREADONLY, EM_SETSEL,
            },
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            Input::{
                Ime::{
                    ImmGetCompositionStringW, ImmGetContext, ImmNotifyIME, ImmReleaseContext,
                    CPS_CANCEL, GCS_COMPSTR, NI_COMPOSITIONSTR,
                },
                KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus},
            },
            Shell::{DefSubclassProc, RemoveWindowSubclass, SetWindowSubclass},
            WindowsAndMessaging::*,
        },
    },
};

const SCOPE: usize = 0;
const HISTORY: usize = 1;
const HINT: usize = 2;
const INPUT: usize = 3;
const STATUS: usize = 4;
const SEND: usize = 5;
const CLOSE: usize = 6;
const DIRECT: usize = 7;
const CATEGORY: usize = 8;
const COUNT: usize = 9;
const CONTROL_ORDER: [usize; COUNT] = [
    SCOPE, CATEGORY, HISTORY, HINT, INPUT, SEND, DIRECT, STATUS, CLOSE,
];
const FIRST: usize = 850;
const STYLE: WINDOW_STYLE = WS_OVERLAPPEDWINDOW;
const EX: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
const INPUT_LIMIT: usize = 512; // UTF-16 units; wire limits are separately checked.
static NEXT: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Message,
    Public(PublicKind),
    GmBots(u16),
}
// Deliberately not Debug: this ephemeral intent contains private text.
pub struct Intent {
    pub token: Token,
    pub text: String,
    pub kind: Kind,
}
pub fn parse_kind(text: &str) -> Result<Kind, &'static str> {
    if !chat_protocol::valid_text(text) {
        return Err("內容需為1至256字且最多512 UTF-8 bytes；不可含換行或控制字元。");
    }
    let trimmed = text.trim();
    if !trimmed.starts_with('.') {
        return Ok(Kind::Message);
    }
    let mut parts = trimmed.split_ascii_whitespace();
    if parts.next() != Some(".de") {
        return Err("尚未支援此指令；沒有廣播或執行。");
    }
    let count = parts
        .next()
        .filter(|n| !n.is_empty() && n.len() <= 3 && n.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|n| n.parse::<u16>().ok())
        .filter(|n| *n <= 100);
    if parts.next().is_some() || count.is_none() {
        return Err("用法：.de 0至100；0為清除假人，權限與容量由伺服器查驗。");
    }
    Ok(Kind::GmBots(count.unwrap()))
}
#[derive(Default, Clone, Copy)]
pub(super) struct InputGate {
    pub(super) composing: bool,
    ime_return: bool,
}
impl InputGate {
    pub(super) fn start(&mut self) {
        self.composing = true;
    }
    pub(super) fn end(&mut self) {
        self.composing = false;
        self.ime_return = true;
    }
    pub(super) fn key_up(&mut self) {
        self.ime_return = false;
    }
    pub(super) fn can_submit(&self, repeated: bool, native_composing: bool) -> bool {
        !repeated && !self.composing && !self.ime_return && !native_composing
    }
}
#[derive(Clone)]
pub struct View {
    pub chat: ChatState,
    pub busy: bool,
    pub chat_pending: bool,
    pub gm_pending: bool,
    pub gm_ready: bool,
    pub direct_supported: bool,
    pub server_status: String,
}
#[derive(Default)]
struct Draft {
    token: Option<Token>,
    pending: Option<Kind>,
    queued: bool,
}
impl Draft {
    fn align(&mut self, token: Option<Token>) -> bool {
        if self.token == token {
            return false;
        }
        *self = Self {
            token,
            ..Self::default()
        };
        true
    }
    fn ready(&self, view: &View) -> bool {
        self.token.is_some()
            && self.token == view.chat.token()
            && !view.busy
            && !self.queued
            && self.pending.is_none()
    }
    fn intent(&mut self, view: &View, text: String) -> Result<Intent, &'static str> {
        self.intent_category(view, text, PublicKind::Normal)
    }
    fn intent_category(
        &mut self,
        view: &View,
        text: String,
        category: PublicKind,
    ) -> Result<Intent, &'static str> {
        if !self.ready(view) {
            return Err("目前範圍未確認或仍有操作待確認，尚未送出。");
        }
        let mut kind = parse_kind(&text)?;
        if category != PublicKind::Normal {
            if !view.chat.filtered() {
                return Err("伺服器尚未提供分類聊天，未送出。");
            }
            if kind == Kind::Message {
                kind = Kind::Public(category);
            }
        }
        if matches!(kind, Kind::GmBots(_)) && !view.gm_ready {
            return Err("此處無法操作假人；需在尚未開局的戰鬥室，並由伺服器重新查驗GM權限。");
        }
        self.queued = true;
        Ok(Intent {
            token: self.token.unwrap(),
            text,
            kind,
        })
    }
    fn dispatched(&mut self, kind: Kind, accepted: bool) {
        self.queued = false;
        self.pending = accepted.then_some(kind);
    }
    fn confirmed(&mut self, view: &View) -> Option<(bool, String)> {
        let kind = self.pending?;
        if match kind {
            Kind::Message | Kind::Public(_) => view.chat_pending,
            Kind::GmBots(_) => view.gm_pending,
        } {
            return None;
        }
        self.pending = None;
        Some(match kind {
            Kind::GmBots(_) => (false, format!("GM操作回覆：{}", view.server_status)),
            Kind::Message | Kind::Public(_) => match view.chat.delivery() {
                Delivery::Confirmed => (true, "伺服器已確認送出。".into()),
                Delivery::RateLimited => {
                    (false, "發送過快，未送達；保留文字，不會自動重送。".into())
                }
                Delivery::CommandRequired => (false, "此訊息需走授權指令，沒有廣播。".into()),
                Delivery::Silenced => {
                    (false, "目前被禁言，未送達；保留文字，不會自動重送。".into())
                }
                _ => (false, "送出未確認或聊天不可用；不會自動重送。".into()),
            },
        })
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn scale(n: i32, dpi: u32) -> i32 {
    ((i64::from(n) * i64::from(dpi.clamp(48, 768)) + 48) / 96) as i32
}
#[derive(Clone, Copy, Default, Debug)]
struct BoxRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}
fn layout(w: i32, h: i32, dpi: u32) -> [BoxRect; COUNT] {
    let m = scale(16, dpi);
    let w = w.max(scale(540, dpi));
    let h = h.max(scale(380, dpi));
    let mut r = [BoxRect::default(); COUNT];
    r[SCOPE] = BoxRect {
        x: m,
        y: m,
        w: w - 2 * m - scale(208, dpi),
        h: scale(28, dpi),
    };
    r[HISTORY] = BoxRect {
        x: m,
        y: scale(52, dpi),
        w: w - 2 * m,
        h: h - scale(240, dpi),
    };
    r[HINT] = BoxRect {
        x: m,
        y: h - scale(180, dpi),
        w: w - 2 * m,
        h: scale(42, dpi),
    };
    r[CATEGORY] = BoxRect {
        x: w - m - scale(200, dpi),
        y: m,
        w: scale(200, dpi),
        h: scale(28, dpi),
    };
    r[INPUT] = BoxRect {
        x: m,
        y: h - scale(130, dpi),
        w: w - 2 * m,
        h: scale(32, dpi),
    };
    r[STATUS] = BoxRect {
        x: m,
        y: h - scale(90, dpi),
        w: w - 2 * m,
        h: scale(36, dpi),
    };
    r[SEND] = BoxRect {
        x: w - m - scale(244, dpi),
        y: h - m - scale(30, dpi),
        w: scale(116, dpi),
        h: scale(30, dpi),
    };
    r[CLOSE] = BoxRect {
        x: w - m - scale(116, dpi),
        y: r[SEND].y,
        w: scale(116, dpi),
        h: scale(30, dpi),
    };
    r[DIRECT] = BoxRect {
        x: m,
        y: r[SEND].y,
        w: scale(220, dpi),
        h: scale(30, dpi),
    };
    r
}
fn transcript(chat: &ChatState) -> String {
    let mut text = if chat.history_gap() {
        "〔僅保留最近64則；部分較舊訊息已略過〕\r\n".to_string()
    } else {
        String::new()
    };
    for m in chat.messages() {
        use std::fmt::Write;
        let _ = write!(text, "{}：{}\r\n", m.display_name(), m.text);
    }
    for m in chat.filtered_messages() {
        use std::fmt::Write;
        let _ = write!(
            text,
            "〔{}〕{}：{}\r\n",
            category_name(m.kind),
            m.sender.display_name(),
            m.text
        );
    }
    text
}
fn category_name(kind: PublicKind) -> &'static str {
    match kind {
        PublicKind::Normal => "一般",
        PublicKind::World => "世界",
        PublicKind::Trade => "交易",
    }
}
fn next_category(kind: PublicKind) -> PublicKind {
    match kind {
        PublicKind::Normal => PublicKind::World,
        PublicKind::World => PublicKind::Trade,
        PublicKind::Trade => PublicKind::Normal,
    }
}
struct State {
    category: Cell<PublicKind>,
    hwnd: Cell<HWND>,
    controls: Cell<Option<[HWND; COUNT]>>,
    font: Cell<HFONT>,
    view: RefCell<View>,
    draft: RefCell<Draft>,
    action: RefCell<Option<Intent>>,
    open_direct: Cell<bool>,
    gate: Cell<InputGate>,
    history: RefCell<String>,
    status: RefCell<String>,
}
pub struct ChatWindow {
    state: Box<State>,
    class: Vec<u16>,
    instance: HINSTANCE,
}
impl ChatWindow {
    pub fn open(owner: HWND, view: View) -> Result<Self> {
        unsafe {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            let class = wide(&format!(
                "LiToModernChat{}",
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            ensure!(
                RegisterClassW(&WNDCLASSW {
                    lpfnWndProc: Some(window_proc),
                    hInstance: instance,
                    lpszClassName: PCWSTR(class.as_ptr()),
                    hCursor: LoadCursorW(None, IDC_ARROW)?,
                    hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                    ..Default::default()
                }) != 0,
                "register chat window"
            );
            let mut window = Self {
                state: Box::new(State {
                    category: Cell::new(PublicKind::Normal),
                    hwnd: Cell::new(HWND::default()),
                    controls: Cell::new(None),
                    font: Cell::new(HFONT::default()),
                    view: RefCell::new(view.clone()),
                    draft: RefCell::new(Draft::default()),
                    action: RefCell::new(None),
                    open_direct: Cell::new(false),
                    gate: Cell::new(InputGate::default()),
                    history: RefCell::new(String::new()),
                    status: RefCell::new(String::new()),
                }),
                class,
                instance,
            };
            let hwnd = CreateWindowExW(
                EX,
                PCWSTR(window.class.as_ptr()),
                w!("LiTo — 大廳／房間聊天"),
                STYLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                650,
                520,
                Some(owner),
                None,
                Some(instance),
                Some((&mut *window.state as *mut State).cast()),
            )?;
            let c = create_controls(hwnd, instance)?;
            window.state.controls.set(Some(c));
            ensure!(
                SetWindowSubclass(
                    c[INPUT],
                    Some(edit_proc),
                    1,
                    (&*window.state as *const State) as usize
                )
                .as_bool(),
                "chat input subclass"
            );
            replace_font(&window.state)?;
            let dpi = GetDpiForWindow(hwnd);
            let mut r = RECT {
                right: scale(640, dpi),
                bottom: scale(460, dpi),
                ..Default::default()
            };
            AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, dpi)?;
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                r.right - r.left,
                r.bottom - r.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            resize(&window.state);
            window.update(view)?;
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetFocus(Some(c[INPUT]));
            Ok(window)
        }
    }
    pub fn handle(&self) -> Option<HWND> {
        let h = self.state.hwnd.get();
        (!h.0.is_null()).then_some(h)
    }
    pub fn has_keyboard_focus(&self) -> bool {
        self.handle().is_some_and(|h| unsafe {
            let focus = GetFocus();
            focus == h || IsChild(h, focus).as_bool()
        })
    }
    pub fn show(&self) {
        if let Some(h) = self.handle() {
            unsafe {
                let _ = ShowWindow(h, SW_RESTORE);
                let _ = SetForegroundWindow(h);
            }
        }
    }
    pub fn take_open_direct(&self) -> bool {
        self.state.open_direct.replace(false)
    }
    pub fn take_action(&self) -> Option<Intent> {
        self.state.action.borrow_mut().take()
    }
    pub fn dispatched(&self, kind: Kind, result: Result<(), String>) {
        self.state
            .draft
            .borrow_mut()
            .dispatched(kind, result.is_ok());
        unsafe {
            set_status(
                &self.state,
                &result
                    .err()
                    .unwrap_or_else(|| "等待伺服器確認；不會自動重送…".into()),
            );
        }
    }
    pub fn update(&self, view: View) -> Result<()> {
        unsafe {
            let Some(c) = self.state.controls.get() else {
                return Ok(());
            };
            if self.handle().is_none() {
                return Ok(());
            }
            let changed = self.state.draft.borrow_mut().align(view.chat.token());
            if changed {
                self.state.category.set(PublicKind::Normal);
                self.state.action.borrow_mut().take();
                clear_input(&self.state);
                set_status(
                    &self.state,
                    "聊天範圍已更新；前一範圍的草稿與紀錄不會沿用。",
                );
            }
            let confirmed = self.state.draft.borrow_mut().confirmed(&view);
            if let Some((clear, status)) = confirmed {
                if clear {
                    clear_input(&self.state);
                }
                set_status(&self.state, &status);
            }
            let scope = match view.chat.scope() {
                Some(s) if s.room_id != 0 => format!(
                    "頻道 {} · 戰鬥室 {}（含戰鬥世界）",
                    s.channel_id + 1,
                    s.room_id
                ),
                Some(s) => format!("頻道 {} · 大廳", s.channel_id + 1),
                None => "尚未進入聊天範圍".into(),
            };
            set_text(c[SCOPE], &scope)?;
            let text = transcript(&view.chat);
            if *self.state.history.borrow() != text {
                let old_end = GetWindowTextLengthW(c[HISTORY]).max(0) as u32;
                let (mut start, mut end) = (0u32, 0u32);
                SendMessageW(
                    c[HISTORY],
                    EM_GETSEL,
                    Some(WPARAM((&mut start as *mut u32) as usize)),
                    Some(LPARAM((&mut end as *mut u32) as isize)),
                );
                let first = SendMessageW(c[HISTORY], EM_GETFIRSTVISIBLELINE, None, None).0;
                let mut scroll = SCROLLINFO {
                    cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                    fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                    ..Default::default()
                };
                let at_bottom = GetScrollInfo(c[HISTORY], SB_VERT, &mut scroll).is_ok()
                    && i64::from(scroll.nPos) + i64::from(scroll.nPage) >= i64::from(scroll.nMax);
                set_text(c[HISTORY], &text)?;
                if old_end == 0 || at_bottom && start == end && end >= old_end {
                    SendMessageW(
                        c[HISTORY],
                        EM_SETSEL,
                        Some(WPARAM(usize::MAX)),
                        Some(LPARAM(-1)),
                    );
                    SendMessageW(c[HISTORY], EM_SCROLLCARET, None, None);
                } else {
                    SendMessageW(
                        c[HISTORY],
                        EM_SETSEL,
                        Some(WPARAM(start as usize)),
                        Some(LPARAM(end as isize)),
                    );
                    SendMessageW(
                        c[HISTORY],
                        EM_LINESCROLL,
                        Some(WPARAM(0)),
                        Some(LPARAM(first)),
                    );
                }
                *self.state.history.borrow_mut() = text;
            }
            let ready = self.state.draft.borrow().ready(&view);
            set_text(
                c[CATEGORY],
                &format!("類型：{}（切換）", category_name(self.state.category.get())),
            )?;
            show_control(c[CATEGORY], view.chat.filtered());
            let _ = EnableWindow(c[CATEGORY], ready && view.chat.filtered());
            SendMessageW(
                c[INPUT],
                EM_SETREADONLY,
                Some(WPARAM((!ready) as usize)),
                None,
            );
            let _ = EnableWindow(c[SEND], ready);
            show_control(c[DIRECT], view.direct_supported);
            let _ = EnableWindow(c[DIRECT], view.direct_supported);
            if view.chat.waiting() {
                set_status(&self.state, "正在取得目前聊天範圍…");
            }
            *self.state.view.borrow_mut() = view;
            Ok(())
        }
    }
    // Consume every child message before the owner's dialog/world hotkeys.
    pub fn pre_translate(&self, m: &MSG) -> bool {
        let Some(h) = self.handle() else { return false };
        unsafe {
            if m.hwnd != h && !IsChild(h, m.hwnd).as_bool() {
                return false;
            }
            let Some(c) = self.state.controls.get() else {
                return false;
            };
            if readonly_tab(h, m, &[c[HISTORY], c[STATUS]]) {
                return true;
            }
            if m.hwnd == c[INPUT] {
                if m.message == WM_KEYUP {
                    let mut g = self.state.gate.get();
                    g.key_up();
                    self.state.gate.set(g);
                }
                if m.message == WM_KEYDOWN && m.wParam.0 == 0x0d {
                    if self
                        .state
                        .gate
                        .get()
                        .can_submit(m.lParam.0 & (1 << 30) != 0, native_composing(c[INPUT]))
                    {
                        request_send(&self.state);
                        return true;
                    }
                }
                if m.message != WM_KEYDOWN || m.wParam.0 != 0x09 || self.state.gate.get().composing
                {
                    let _ = TranslateMessage(m);
                    DispatchMessageW(m);
                    return true;
                }
            }
            if m.message == WM_KEYDOWN && m.wParam.0 == 0x1b {
                let _ = DestroyWindow(h);
                return true;
            }
            if m.message == WM_KEYDOWN && m.wParam.0 == 0x0d {
                let focus = GetFocus();
                if focus == c[SEND] {
                    request_send(&self.state);
                } else if focus == c[CATEGORY] {
                    cycle_category(&self.state);
                } else if focus == c[DIRECT] && self.state.view.borrow().direct_supported {
                    self.state.open_direct.set(true);
                } else if focus == c[CLOSE] {
                    let _ = DestroyWindow(h);
                }
                return true;
            }
            if !IsDialogMessageW(h, m).as_bool() {
                let _ = TranslateMessage(m);
                DispatchMessageW(m);
            }
            true
        }
    }
}
impl Drop for ChatWindow {
    fn drop(&mut self) {
        unsafe {
            if let Some(h) = self.handle() {
                let _ = DestroyWindow(h);
            }
            let f = self.state.font.get();
            if !f.0.is_null() {
                let _ = DeleteObject(f.into());
            }
            let _ = UnregisterClassW(PCWSTR(self.class.as_ptr()), Some(self.instance));
        }
    }
}
unsafe fn set_text(h: HWND, s: &str) -> Result<()> {
    let t = wide(s);
    SetWindowTextW(h, PCWSTR(t.as_ptr()))?;
    Ok(())
}
unsafe fn show_control(h: HWND, visible: bool) {
    // A polling refresh must not activate a child or disturb keyboard focus.
    if (GetWindowLongW(h, GWL_STYLE) as u32 & WS_VISIBLE.0 != 0) != visible {
        let _ = ShowWindow(h, if visible { SW_SHOWNA } else { SW_HIDE });
    }
}
pub(super) unsafe fn readonly_tab(h: HWND, m: &MSG, controls: &[HWND]) -> bool {
    // Native multiline EDIT consumes Tab even when read-only. These panes have
    // no editable/IME input; explicitly preserve dialog traversal in both directions.
    if m.message != WM_KEYDOWN || m.wParam.0 != 0x09 || !controls.contains(&m.hwnd) {
        return false;
    }
    let previous = windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState(0x10) < 0;
    if let Ok(next) = GetNextDlgTabItem(h, Some(m.hwnd), previous) {
        let _ = SetFocus(Some(next));
    }
    true
}
unsafe fn set_status(s: &State, text: &str) {
    if *s.status.borrow() == text {
        return;
    }
    if let Some(c) = s.controls.get() {
        if set_text(c[STATUS], text).is_ok() {
            *s.status.borrow_mut() = text.into();
        }
    }
}
unsafe fn clear_input(s: &State) {
    if let Some(c) = s.controls.get() {
        // Discard an in-progress native IME composition as well as EDIT text.
        // Otherwise a delayed IME result could populate a different audience.
        let context = ImmGetContext(c[INPUT]);
        if !context.0.is_null() {
            let active = s.gate.get().composing
                || ImmGetCompositionStringW(context, GCS_COMPSTR, None, 0) > 0;
            let cancelled =
                !active || ImmNotifyIME(context, NI_COMPOSITIONSTR, CPS_CANCEL, 0).as_bool();
            let _ = ImmReleaseContext(c[INPUT], context);
            if !cancelled {
                let _ = DestroyWindow(s.hwnd.get());
                return;
            }
        }
        let _ = set_text(c[INPUT], "");
        SendMessageW(c[INPUT], EM_EMPTYUNDOBUFFER, None, None);
    }
    s.gate.set(InputGate::default());
}
unsafe fn native_composing(h: HWND) -> bool {
    let context = ImmGetContext(h);
    if context.0.is_null() {
        return false;
    }
    let n = ImmGetCompositionStringW(context, GCS_COMPSTR, None, 0);
    let _ = ImmReleaseContext(h, context);
    n > 0
}
unsafe fn request_send(s: &State) {
    let Some(c) = s.controls.get() else { return };
    if !s.gate.get().can_submit(false, native_composing(c[INPUT])) {
        set_status(s, "請先完成中文組字，再送出訊息。");
        return;
    }
    let len = GetWindowTextLengthW(c[INPUT]);
    if len < 0 || len as usize > INPUT_LIMIT {
        set_status(s, "輸入超過上限，未送出。");
        return;
    }
    let mut buf = vec![0u16; len as usize + 1];
    if GetWindowTextW(c[INPUT], &mut buf) != len {
        set_status(s, "無法讀取完整訊息，未送出。");
        return;
    }
    let Ok(text) = String::from_utf16(&buf[..len as usize]) else {
        set_status(s, "文字編碼不完整，未送出。");
        return;
    };
    let result = {
        let view = s.view.borrow();
        if s.category.get() == PublicKind::Normal {
            s.draft.borrow_mut().intent(&view, text)
        } else {
            s.draft
                .borrow_mut()
                .intent_category(&view, text, s.category.get())
        }
    };
    match result {
        Ok(intent) => {
            *s.action.borrow_mut() = Some(intent);
            let _ = EnableWindow(c[SEND], false);
            let _ = EnableWindow(c[CATEGORY], false);
            SendMessageW(c[INPUT], EM_SETREADONLY, Some(WPARAM(1)), None);
            set_status(s, "正在提交確認…");
        }
        Err(e) => set_status(s, e),
    }
}
unsafe fn create_controls(h: HWND, instance: HINSTANCE) -> Result<[HWND; COUNT]> {
    let mut c = [HWND::default(); COUNT];
    for i in CONTROL_ORDER {
        let control = &mut c[i];
        let (class, text, style) = match i {
            HISTORY | STATUS => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0
                    | WS_BORDER.0
                    | WS_VSCROLL.0
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
            ),
            INPUT => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32,
            ),
            CATEGORY => (
                w!("BUTTON"),
                "類型：一般（切換）",
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
            ),
            SEND | CLOSE | DIRECT => (
                w!("BUTTON"),
                if i == DIRECT {
                    "私訊／拒收"
                } else if i == SEND {
                    "送出 (Enter)"
                } else {
                    "關閉聊天"
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
            ),
            _ => (
                w!("STATIC"),
                if i == HINT {
                    "目前範圍限定；上限256字／512 UTF-8 bytes。\r\n.de 數量：GM授權；打字不觸發移動／技能。"
                } else {
                    ""
                },
                0x80,
            ),
        };
        let text = wide(text);
        *control = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class,
            PCWSTR(text.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
            0,
            0,
            0,
            0,
            Some(h),
            Some(HMENU((FIRST + i) as *mut _)),
            Some(instance),
            None,
        )?;
    }
    SendMessageW(c[INPUT], EM_SETLIMITTEXT, Some(WPARAM(INPUT_LIMIT)), None);
    SendMessageW(c[HISTORY], EM_SETLIMITTEXT, Some(WPARAM(128 * 1024)), None);
    Ok(c)
}
unsafe fn cycle_category(s: &State) {
    let view = s.view.borrow();
    if view.chat.filtered() && s.draft.borrow().ready(&view) {
        s.category.set(next_category(s.category.get()));
        if let Some(c) = s.controls.get() {
            let _ = set_text(
                c[CATEGORY],
                &format!("類型：{}（切換）", category_name(s.category.get())),
            );
        }
    }
}
unsafe fn resize(s: &State) {
    let Some(c) = s.controls.get() else { return };
    let mut r = RECT::default();
    if GetClientRect(s.hwnd.get(), &mut r).is_err() {
        return;
    }
    for (h, b) in c
        .into_iter()
        .zip(layout(r.right, r.bottom, GetDpiForWindow(s.hwnd.get())))
    {
        let _ = MoveWindow(h, b.x, b.y, b.w, b.h, true);
    }
}
unsafe fn replace_font(s: &State) -> Result<()> {
    let font = CreateFontW(
        -scale(16, GetDpiForWindow(s.hwnd.get())),
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
    ensure!(!font.0.is_null(), "chat font unavailable");
    if let Some(c) = s.controls.get() {
        for h in c {
            SendMessageW(
                h,
                WM_SETFONT,
                Some(WPARAM(font.0 as usize)),
                Some(LPARAM(1)),
            );
        }
    }
    let old = s.font.replace(font);
    if !old.0.is_null() {
        let _ = DeleteObject(old.into());
    }
    Ok(())
}
unsafe extern "system" fn edit_proc(
    h: HWND,
    m: u32,
    w: WPARAM,
    l: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    let s = &*(data as *const State);
    let mut g = s.gate.get();
    match m {
        WM_IME_STARTCOMPOSITION => g.start(),
        WM_IME_ENDCOMPOSITION => g.end(),
        WM_NCDESTROY => {
            let _ = RemoveWindowSubclass(h, Some(edit_proc), 1);
        }
        _ => {}
    }
    s.gate.set(g);
    DefSubclassProc(h, m, w, l)
}
unsafe extern "system" fn window_proc(h: HWND, m: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    if m == WM_NCCREATE {
        let c = &*(l.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(h, GWLP_USERDATA, c.lpCreateParams as _);
        (*(c.lpCreateParams as *mut State)).hwnd.set(h);
    }
    let ptr = GetWindowLongPtrW(h, GWLP_USERDATA) as *const State;
    if ptr.is_null() {
        return DefWindowProcW(h, m, w, l);
    }
    let s = &*ptr;
    match m {
        WM_CLOSE => {
            let _ = DestroyWindow(h);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            s.action.borrow_mut().take();
            s.hwnd.set(HWND::default());
            SetWindowLongPtrW(h, GWLP_USERDATA, 0);
            DefWindowProcW(h, m, w, l)
        }
        WM_SIZE => {
            resize(s);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            if l.0 != 0 {
                let r = &*(l.0 as *const RECT);
                let _ = SetWindowPos(
                    h,
                    None,
                    r.left,
                    r.top,
                    r.right - r.left,
                    r.bottom - r.top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
            let _ = replace_font(s);
            resize(s);
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let dpi = GetDpiForWindow(h);
            let mut r = RECT {
                right: scale(540, dpi),
                bottom: scale(380, dpi),
                ..Default::default()
            };
            if l.0 != 0 && AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, dpi).is_ok() {
                let info = &mut *(l.0 as *mut MINMAXINFO);
                info.ptMinTrackSize.x = r.right - r.left;
                info.ptMinTrackSize.y = r.bottom - r.top;
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(c) = s.controls.get() {
                let id = (w.0 & 65535).checked_sub(FIRST);
                let n = w.0 >> 16;
                if n == BN_CLICKED as usize {
                    match id {
                        Some(CATEGORY) if l.0 == c[CATEGORY].0 as isize => {
                            cycle_category(s);
                        }
                        Some(SEND) if l.0 == c[SEND].0 as isize => request_send(s),
                        Some(DIRECT)
                            if l.0 == c[DIRECT].0 as isize && s.view.borrow().direct_supported =>
                        {
                            s.open_direct.set(true);
                        }
                        Some(CLOSE) if l.0 == c[CLOSE].0 as isize => {
                            let _ = DestroyWindow(h);
                        }
                        _ => {}
                    }
                }
                if id == Some(INPUT) && n == EN_MAXTEXT as usize {
                    set_status(s, "輸入框已達上限；貼上內容可能未完整，請確認後再送出。");
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(h, m, w, l),
    }
}

#[cfg(test)]
mod tests {
    include!("chat_filtered_window_tests.rs");
    use super::*;
    fn view() -> View {
        use super::super::{
            application_protocol::{CharacterKey, SelectionContext},
            chat_state::Scope,
        };
        let mut chat = ChatState::default();
        chat.align(
            true,
            Some(Scope {
                context: SelectionContext {
                    key: CharacterKey {
                        source_id: 7,
                        object_id: 3333,
                    },
                    epoch: 1,
                },
                channel_id: 0,
                room_id: 10,
            }),
        );
        chat.receive(chat_protocol::Reply {
            echo: chat.read().unwrap(),
            status: chat_protocol::Status::Ok,
            membership: 1,
            sequence: 0,
            cursor: 0,
            available: true,
            truncated: false,
            messages: vec![],
        })
        .unwrap();
        View {
            chat,
            busy: false,
            chat_pending: false,
            gm_pending: false,
            gm_ready: true,
            direct_supported: false,
            server_status: "權威測試回覆".into(),
        }
    }
    #[test]
    fn chat_draft_is_single_flight_clears_on_audience_change_and_retains_denied_text() {
        let mut v = view();
        let mut d = Draft::default();
        assert!(d.align(v.chat.token()));
        let intent = d.intent(&v, "你好".into()).unwrap();
        assert!(!d.ready(&v));
        assert!(d.intent(&v, "duplicate".into()).is_err());
        d.dispatched(intent.kind, true);
        v.chat_pending = true;
        assert!(d.confirmed(&v).is_none());
        let request = v.chat.send(intent.token, intent.text).unwrap();
        v.chat
            .receive(chat_protocol::Reply {
                echo: request,
                status: chat_protocol::Status::RateLimited,
                membership: 1,
                sequence: 1,
                cursor: 0,
                available: true,
                truncated: false,
                messages: vec![],
            })
            .unwrap();
        v.chat_pending = false;
        let (clear, status) = d.confirmed(&v).unwrap();
        assert!(!clear && status.contains("未送達"));
        assert!(d.ready(&v));
        let _ = d.intent(&v, "queued".into()).unwrap();
        assert!(d.align(None));
        assert!(!d.queued && d.pending.is_none());
        assert!(d.intent(&v, "wrong audience".into()).is_err());
        d.align(v.chat.token());
        v.busy = true;
        assert!(d.intent(&v, "busy".into()).is_err());
        v.busy = false;
        v.gm_ready = false;
        assert!(d.intent(&v, ".de 20".into()).is_err());
        assert!(!d.queued);
    }
    #[test]
    fn chat_success_is_only_authoritative_and_gm_receipts_are_not_chat_delivery() {
        let mut v = view();
        let mut d = Draft::default();
        d.align(v.chat.token());
        let i = d.intent(&v, "世界 👩\u{200d}💻".into()).unwrap();
        d.dispatched(i.kind, true);
        assert!(transcript(&v.chat).is_empty());
        let request = v.chat.send(i.token, i.text.clone()).unwrap();
        v.chat
            .receive(chat_protocol::Reply {
                echo: request,
                status: chat_protocol::Status::Ok,
                membership: 1,
                sequence: 1,
                cursor: 1,
                available: true,
                truncated: false,
                messages: vec![chat_protocol::Message {
                    id: 1,
                    sender: i.token.scope.context.key,
                    character_name: "3333".into(),
                    source_name: "太陽神".into(),
                    text: i.text,
                }],
            })
            .unwrap();
        assert!(d.confirmed(&v).unwrap().0);
        assert!(d.confirmed(&v).is_none());
        assert!(transcript(&v.chat).contains("3333(太陽神)：世界 👩\u{200d}💻"));
        let i = d.intent(&v, ".de 20".into()).unwrap();
        d.dispatched(i.kind, true);
        v.gm_pending = true;
        assert!(d.confirmed(&v).is_none());
        v.gm_pending = false;
        let (clear, s) = d.confirmed(&v).unwrap();
        assert!(!clear && s.contains("權威測試回覆"));
        assert_eq!(v.chat.messages().len(), 1);
    }
    #[test]
    fn command_text_preserves_unicode_and_never_broadcasts_dot_commands() {
        for s in ["你好👩\u{200d}💻", "<b>plain text</b>", "  原始文字  "] {
            assert_eq!(parse_kind(s), Ok(Kind::Message));
        }
        for (n, k) in [(".de 0", 0), (".de 20", 20), (" .de 100 ", 100)] {
            assert_eq!(parse_kind(n), Ok(Kind::GmBots(k)));
        }
        for s in [
            ".de",
            ".de -1",
            ".de 101",
            ".de 1 2",
            ".de １",
            ".delete 20",
            ".de 20\n",
            ".unknown",
            " ",
        ] {
            assert!(parse_kind(s).is_err(), "{s}");
        }
    }
    #[test]
    fn ime_commit_enter_repeat_and_world_shortcuts_are_separate() {
        let mut g = InputGate::default();
        assert!(g.can_submit(false, false));
        g.start();
        assert!(!g.can_submit(false, false));
        g.end();
        assert!(!g.can_submit(false, false));
        g.key_up();
        assert!(g.can_submit(false, false));
        assert!(!g.can_submit(true, false));
        assert!(!g.can_submit(false, true));
    }
    #[test]
    fn panel_layout_is_client_relative_nonoverlapping_at_all_supported_dpi() {
        for dpi in [96, 120, 144, 192, 288, 384] {
            for (w, h) in [(540, 380), (640, 460), (1024, 700)] {
                let (w, h) = (scale(w, dpi), scale(h, dpi));
                let r = layout(w, h, dpi);
                for (i, a) in r.iter().enumerate() {
                    assert!(
                        a.x >= 0
                            && a.y >= 0
                            && a.w > 0
                            && a.h > 0
                            && a.x + a.w <= w
                            && a.y + a.h <= h,
                        "{i} {dpi}"
                    );
                    for b in r.iter().skip(i + 1) {
                        assert!(
                            a.x + a.w <= b.x
                                || b.x + b.w <= a.x
                                || a.y + a.h <= b.y
                                || b.y + b.h <= a.y,
                            "overlap {dpi}"
                        );
                    }
                }
            }
        }
    }
}
