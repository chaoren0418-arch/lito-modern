//! Same-EXE owned native chat panel. No network, files, optimistic messages,
//! game-key routing or saved drafts. The login controller owns all authority.
use super::{
    chat_window::InputGate,
    direct_chat_protocol::{Action, Address, Preferences},
    direct_chat_state::{Outcome, State as ChatState, Token},
    login_window::SourceChoice,
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
            CreateFontW, DeleteObject, GetMonitorInfoW, GetSysColorBrush, InvalidateRect,
            MonitorFromRect, MonitorFromWindow, CLEARTYPE_QUALITY, CLIP_DEFAULT_PRECIS,
            COLOR_BTNFACE, DEFAULT_CHARSET, HFONT, MONITORINFO, MONITOR_DEFAULTTONEAREST,
            OUT_DEFAULT_PRECIS,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::{
                SetScrollInfo, EM_EMPTYUNDOBUFFER, EM_GETFIRSTVISIBLELINE, EM_GETSEL,
                EM_LINESCROLL, EM_SCROLLCARET, EM_SETLIMITTEXT, EM_SETREADONLY, EM_SETSEL,
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
const SOURCE: usize = 7;
const NAME: usize = 8;
const RECEIVE: usize = 9;
const APPLY: usize = 10;
const BLOCK: usize = 11;
const BLOCKED: usize = 12;
const UNBLOCK: usize = 13;
const CLEAR: usize = 14;
const WORLD_RECEIVE: usize = 15;
const TRADE_RECEIVE: usize = 16;
const COUNT: usize = 17;
const CONTROL_ORDER: [usize; COUNT] = [
    SCOPE,
    SOURCE,
    NAME,
    HINT,
    HISTORY,
    INPUT,
    RECEIVE,
    WORLD_RECEIVE,
    TRADE_RECEIVE,
    APPLY,
    SEND,
    BLOCKED,
    BLOCK,
    UNBLOCK,
    CLEAR,
    STATUS,
    CLOSE,
];
const FIRST: usize = 900;
const STYLE: WINDOW_STYLE =
    WINDOW_STYLE(WS_OVERLAPPEDWINDOW.0 | WS_VSCROLL.0 | WS_HSCROLL.0 | WS_CLIPCHILDREN.0);
const EX: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
const INPUT_LIMIT: usize = 512; // UTF-16 units; wire limits are separately checked.
static NEXT: AtomicU64 = AtomicU64::new(1);

// Deliberately not Debug: targets, drafts and private text are ephemeral.
pub struct Intent {
    pub token: Token,
    pub action: Action,
    pub target: Option<Address>,
    pub preferences: Option<Preferences>,
    pub text: String,
}
#[derive(Clone)]
pub struct View {
    pub chat: ChatState,
    pub filtered_public: bool,
    pub sources: Vec<SourceChoice>,
    pub busy: bool,
    pub pending: bool,
}
#[derive(Default)]
struct Draft {
    token: Option<Token>,
    pending: Option<Action>,
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
            && !view.pending
            && !self.queued
            && self.pending.is_none()
    }
    fn intent(
        &mut self,
        view: &View,
        action: Action,
        target: Option<Address>,
        preferences: Option<Preferences>,
        text: String,
    ) -> Result<Intent, &'static str> {
        if !self.ready(view) {
            return Err("目前私訊範圍尚未確認或仍有操作待確認。");
        }
        if target
            .as_ref()
            .is_some_and(|t| !view.sources.iter().any(|s| s.id == t.source_id))
        {
            return Err("收件伺服器不在目前來源清單，未送出。");
        }
        let token = self.token.unwrap();
        view.chat
            .command(token, action, target.clone(), preferences, text.clone())?;
        self.queued = true;
        Ok(Intent {
            token,
            action,
            target,
            preferences,
            text,
        })
    }
    fn dispatched(&mut self, action: Action, accepted: bool) {
        self.queued = false;
        self.pending = accepted.then_some(action);
    }
    fn confirmed(&mut self, view: &View) -> Option<(bool, &'static str)> {
        let action = self.pending?;
        if view.pending {
            return None;
        }
        self.pending = None;
        Some(match view.chat.outcome() {
            Outcome::Confirmed(a) if a == action => (
                a == Action::Send,
                match a {
                    Action::Send => "伺服器已確認私訊送達。",
                    Action::Preferences => "接收設定已由伺服器確認。",
                    Action::Block => "已加入拒收名單。",
                    Action::Unblock => "已解除所選角色的拒收。",
                    Action::ClearBlocks => "拒收名單已清空。",
                    Action::Read => "收件狀態已確認。",
                },
            ),
            Outcome::RateLimited => (false, "發送過快，未送達；保留文字，不會自動重送。"),
            Outcome::Silenced => (false, "目前被禁言，未送達；保留文字，不會自動重送。"),
            Outcome::Unreachable => (
                false,
                "目前無法送達此角色；請確認伺服器與角色名稱，不會自動重送。",
            ),
            Outcome::BlockListFull => (false, "拒收名單已達16名，沒有新增。"),
            _ => (false, "操作未確認或私訊不可用；不會自動重送。"),
        })
    }
}
fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}
fn scale(n: i32, dpi: u32) -> i32 {
    ((i64::from(n) * i64::from(dpi.clamp(48, 768)) + 48) / 96) as i32
}
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
struct BoxRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}
fn content_size(w: i32, h: i32, dpi: u32) -> (i32, i32) {
    let w = w.max(scale(360, dpi));
    (
        w,
        h.max(scale(if w >= scale(720, dpi) { 640 } else { 980 }, dpi)),
    )
}
fn fit_outer(wanted: BoxRect, work: BoxRect) -> BoxRect {
    let w = wanted.w.max(1).min(work.w.max(1));
    let h = wanted.h.max(1).min(work.h.max(1));
    BoxRect {
        x: wanted.x.clamp(work.x, work.x + work.w.max(1) - w),
        y: wanted.y.clamp(work.y, work.y + work.h.max(1) - h),
        w,
        h,
    }
}
fn reveal_axis(position: i32, start: i32, size: i32, viewport: i32, extent: i32) -> i32 {
    let viewport = viewport.max(1);
    let next = if start < position || size > viewport {
        start
    } else if start + size > position + viewport {
        start + size - viewport
    } else {
        position
    };
    next.clamp(0, (extent - viewport).max(0))
}
fn source_label(source: &SourceChoice) -> String {
    format!("#{} {}", source.id, source.label)
}
fn layout(w: i32, h: i32, dpi: u32) -> [BoxRect; COUNT] {
    let m = scale(16, dpi);
    let (w, h) = content_size(w, h, dpi);
    let mut r = [BoxRect::default(); COUNT];
    let rect = |x, y, bw, bh| BoxRect { x, y, w: bw, h: bh };
    if w < scale(720, dpi) {
        let full = w - 2 * m;
        for (i, y, bh) in [
            (SCOPE, 16, 56),
            (SOURCE, 80, 200),
            (NAME, 124, 32),
            (HINT, 168, 84),
            (HISTORY, 264, 160),
            (INPUT, 440, 32),
            (RECEIVE, 484, 30),
            (APPLY, 526, 30),
            (SEND, 568, 30),
            (BLOCKED, 610, 102),
            (BLOCK, 724, 30),
            (UNBLOCK, 766, 30),
            (CLEAR, 808, 30),
            (STATUS, 850, 72),
            (CLOSE, 934, 30),
        ] {
            r[i] = rect(m, scale(y, dpi), full, scale(bh, dpi));
        }
        let gap = scale(8, dpi);
        let width = (full - 2 * gap) / 3;
        for (index, control) in [RECEIVE, WORLD_RECEIVE, TRADE_RECEIVE]
            .into_iter()
            .enumerate()
        {
            r[control] = rect(
                m + (width + gap) * index as i32,
                scale(484, dpi),
                width,
                scale(30, dpi),
            );
        }
        return r;
    }
    r[SCOPE] = rect(m, m, w - 2 * m, scale(28, dpi));
    // The combo's extra height is its native dropdown, not a second overlay.
    r[SOURCE] = rect(m, scale(52, dpi), scale(220, dpi), scale(200, dpi));
    r[NAME] = rect(
        scale(248, dpi),
        scale(52, dpi),
        w - scale(264, dpi),
        scale(32, dpi),
    );
    r[HINT] = rect(m, scale(92, dpi), w - 2 * m, scale(42, dpi));
    r[HISTORY] = rect(m, scale(140, dpi), w - 2 * m, h - scale(430, dpi));
    r[INPUT] = rect(m, h - scale(280, dpi), w - 2 * m, scale(32, dpi));
    r[RECEIVE] = rect(m, h - scale(238, dpi), scale(120, dpi), scale(30, dpi));
    r[WORLD_RECEIVE] = rect(
        scale(140, dpi),
        r[RECEIVE].y,
        scale(120, dpi),
        scale(30, dpi),
    );
    r[TRADE_RECEIVE] = rect(
        scale(264, dpi),
        r[RECEIVE].y,
        scale(120, dpi),
        scale(30, dpi),
    );
    r[APPLY] = rect(
        scale(392, dpi),
        r[RECEIVE].y,
        scale(152, dpi),
        scale(30, dpi),
    );
    r[SEND] = rect(
        w - m - scale(144, dpi),
        r[RECEIVE].y,
        scale(144, dpi),
        scale(30, dpi),
    );
    r[BLOCKED] = rect(m, h - scale(190, dpi), w - scale(264, dpi), scale(102, dpi));
    for (i, y) in [(BLOCK, 190), (UNBLOCK, 154), (CLEAR, 118)] {
        r[i] = rect(
            w - scale(236, dpi),
            h - scale(y, dpi),
            scale(220, dpi),
            scale(30, dpi),
        );
    }
    r[STATUS] = rect(m, h - scale(78, dpi), w - scale(160, dpi), scale(62, dpi));
    r[CLOSE] = rect(
        w - m - scale(116, dpi),
        h - m - scale(30, dpi),
        scale(116, dpi),
        scale(30, dpi),
    );
    r
}
fn transcript(chat: &ChatState) -> String {
    let mut text = if chat.history_gap() {
        "〔僅保留最近64則；部分舊訊息已略過〕\r\n".into()
    } else {
        String::new()
    };
    for m in chat.messages() {
        use std::fmt::Write;
        let _ = write!(
            text,
            "{} → {}：{}\r\n",
            m.sender.display_name(),
            m.recipient.display_name(),
            m.text
        );
    }
    text
}
fn scope_text(chat: &ChatState) -> String {
    chat.owner()
        .map(|o| format!("私訊身分：{}　｜　收件伺服器 / 角色", o.display_name()))
        .unwrap_or_else(|| "正在確認私訊身分…".into())
}
struct State {
    hwnd: Cell<HWND>,
    controls: Cell<Option<[HWND; COUNT]>>,
    font: Cell<HFONT>,
    view: RefCell<View>,
    draft: RefCell<Draft>,
    action: RefCell<Option<Intent>>,
    shown_preferences: Cell<Option<Preferences>>,
    sources: RefCell<Vec<SourceChoice>>,
    blocked: RefCell<Vec<Address>>,
    gate: Cell<InputGate>,
    history: RefCell<String>,
    scope: RefCell<String>,
    status: RefCell<String>,
    scroll_x: Cell<i32>,
    scroll_y: Cell<i32>,
    wheel: Cell<i32>,
    resizing: Cell<bool>,
}
pub struct DirectChatWindow {
    state: Box<State>,
    class: Vec<u16>,
    instance: HINSTANCE,
}
impl DirectChatWindow {
    pub fn open(owner: HWND, view: View) -> Result<Self> {
        unsafe {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            let class = wide(&format!(
                "LiToModernDirectChat{}",
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
                    hwnd: Cell::new(HWND::default()),
                    controls: Cell::new(None),
                    font: Cell::new(HFONT::default()),
                    view: RefCell::new(view.clone()),
                    draft: RefCell::new(Draft::default()),
                    action: RefCell::new(None),
                    shown_preferences: Cell::new(None),
                    sources: RefCell::new(Vec::new()),
                    blocked: RefCell::new(Vec::new()),
                    gate: Cell::new(InputGate::default()),
                    history: RefCell::new(String::new()),
                    scope: RefCell::new(String::new()),
                    status: RefCell::new(String::new()),
                    scroll_x: Cell::new(0),
                    scroll_y: Cell::new(0),
                    wheel: Cell::new(0),
                    resizing: Cell::new(false),
                }),
                class,
                instance,
            };
            let hwnd = CreateWindowExW(
                EX,
                PCWSTR(window.class.as_ptr()),
                w!("LiTo — 私訊／拒收"),
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
            for control in c {
                ensure!(
                    SetWindowSubclass(
                        control,
                        Some(control_proc),
                        2,
                        (&*window.state as *const State) as usize
                    )
                    .as_bool(),
                    "private viewport subclass"
                );
            }
            for edit in [c[INPUT], c[NAME]] {
                ensure!(
                    SetWindowSubclass(
                        edit,
                        Some(edit_proc),
                        1,
                        (&*window.state as *const State) as usize
                    )
                    .as_bool(),
                    "direct chat input subclass"
                );
            }
            replace_font(&window.state)?;
            let dpi = GetDpiForWindow(hwnd);
            let mut r = RECT {
                right: scale(760, dpi),
                bottom: scale(680, dpi),
                ..Default::default()
            };
            AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, dpi)?;
            let area = monitor_work(owner, None)
                .ok_or_else(|| anyhow::anyhow!("monitor work area unavailable"))?;
            let wanted = BoxRect {
                x: area.x + (area.w - (r.right - r.left)) / 2,
                y: area.y + (area.h - (r.bottom - r.top)) / 2,
                w: r.right - r.left,
                h: r.bottom - r.top,
            };
            place(hwnd, fit_outer(wanted, area))?;
            resize(&window.state);
            window.update(view)?;
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetFocus(Some(c[NAME]));
            reveal_focus(&window.state);
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
    pub fn take_action(&self) -> Option<Intent> {
        self.state.action.borrow_mut().take()
    }
    pub fn dispatched(&self, kind: Action, result: Result<(), String>) {
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
                self.state.action.borrow_mut().take();
                clear_draft(&self.state);
                self.state.shown_preferences.set(None);
                set_status(&self.state, "私訊入場資格已更新；舊草稿不會沿用。");
            }
            let confirmed = self.state.draft.borrow_mut().confirmed(&view);
            if let Some((clear, status)) = confirmed {
                if clear {
                    clear_input(&self.state);
                }
                set_status(&self.state, &status);
            }
            // This is a readonly EDIT so long identities can be selected and
            // scrolled. Do not reset its selection/caret on every UI refresh.
            let scope = scope_text(&view.chat);
            if *self.state.scope.borrow() != scope {
                set_text(c[SCOPE], &scope)?;
                *self.state.scope.borrow_mut() = scope;
            }
            if *self.state.sources.borrow() != view.sources {
                SendMessageW(c[SOURCE], CB_RESETCONTENT, None, None);
                for source in &view.sources {
                    let label = wide(&source_label(source));
                    let index = SendMessageW(
                        c[SOURCE],
                        CB_ADDSTRING,
                        None,
                        Some(LPARAM(label.as_ptr() as isize)),
                    )
                    .0;
                    ensure!(index >= 0, "private source list unavailable");
                }
                let selected = view
                    .chat
                    .enrollment()
                    .and_then(|e| {
                        view.sources
                            .iter()
                            .position(|s| s.id == e.context.key.source_id)
                    })
                    .unwrap_or(0);
                SendMessageW(c[SOURCE], CB_SETCURSEL, Some(WPARAM(selected)), None);
                *self.state.sources.borrow_mut() = view.sources.clone();
                clear_draft(&self.state);
            }
            if self.state.shown_preferences.get() != view.chat.preferences() {
                if let Some(p) = view.chat.preferences() {
                    for (i, enabled) in [(WORLD_RECEIVE, p.world), (TRADE_RECEIVE, p.trade)] {
                        SendMessageW(c[i], BM_SETCHECK, Some(WPARAM(enabled as usize)), None);
                    }
                    SendMessageW(
                        c[RECEIVE],
                        BM_SETCHECK,
                        Some(WPARAM(if p.whispers { 1 } else { 0 })),
                        None,
                    );
                }
                self.state.shown_preferences.set(view.chat.preferences());
            }
            if *self.state.blocked.borrow() != view.chat.blocked() {
                SendMessageW(c[BLOCKED], LB_RESETCONTENT, None, None);
                for address in view.chat.blocked() {
                    let source = view
                        .sources
                        .iter()
                        .find(|s| s.id == address.source_id)
                        .map(source_label)
                        .unwrap_or_else(|| format!("來源 #{}", address.source_id));
                    let label = wide(&format!("{}({})", address.name, source));
                    ensure!(
                        SendMessageW(
                            c[BLOCKED],
                            LB_ADDSTRING,
                            None,
                            Some(LPARAM(label.as_ptr() as isize))
                        )
                        .0 >= 0,
                        "private block list unavailable"
                    );
                }
                *self.state.blocked.borrow_mut() = view.chat.blocked().to_vec();
            }
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
            SendMessageW(
                c[INPUT],
                EM_SETREADONLY,
                Some(WPARAM((!ready) as usize)),
                None,
            );
            for i in [
                SEND, SOURCE, NAME, RECEIVE, APPLY, BLOCK, BLOCKED, UNBLOCK, CLEAR,
            ] {
                let _ = EnableWindow(c[i], ready);
            }
            for i in [WORLD_RECEIVE, TRADE_RECEIVE] {
                if (GetWindowLongW(c[i], GWL_STYLE) as u32 & WS_VISIBLE.0 != 0)
                    != view.filtered_public
                {
                    let _ = ShowWindow(
                        c[i],
                        if view.filtered_public {
                            SW_SHOWNA
                        } else {
                            SW_HIDE
                        },
                    );
                }
                let _ = EnableWindow(c[i], ready && view.filtered_public);
            }
            if view.chat.waiting() {
                set_status(&self.state, "正在取得私訊資格與收件狀態…");
            } else if view.chat.token().is_none() && !view.pending {
                set_status(&self.state, "目前私訊不可用；沒有送出操作。");
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
            if super::chat_window::readonly_tab(h, m, &[c[HISTORY], c[STATUS]]) {
                reveal_focus(&self.state);
                return true;
            }
            if m.hwnd == c[INPUT] || m.hwnd == c[NAME] {
                if m.message == WM_KEYUP {
                    let mut g = self.state.gate.get();
                    g.key_up();
                    self.state.gate.set(g);
                }
                if m.message == WM_KEYDOWN && m.wParam.0 == 0x1b && !self.state.gate.get().composing
                {
                    let _ = DestroyWindow(h);
                    return true;
                }
                if m.hwnd == c[INPUT] && m.message == WM_KEYDOWN && m.wParam.0 == 0x0d {
                    if self
                        .state
                        .gate
                        .get()
                        .can_submit(m.lParam.0 & (1 << 30) != 0, native_composing(c[INPUT]))
                    {
                        request_action(&self.state, Action::Send);
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
                if focus == c[CLOSE] {
                    let _ = DestroyWindow(h);
                }
                for (i, a) in [
                    (SEND, Action::Send),
                    (APPLY, Action::Preferences),
                    (BLOCK, Action::Block),
                    (UNBLOCK, Action::Unblock),
                    (CLEAR, Action::ClearBlocks),
                ] {
                    if focus == c[i] {
                        request_action(&self.state, a);
                    }
                }
                return true;
            }
            if !IsDialogMessageW(h, m).as_bool() {
                let _ = TranslateMessage(m);
                DispatchMessageW(m);
            }
            if m.message == WM_KEYDOWN && m.wParam.0 == 0x09 {
                reveal_focus(&self.state);
            }
            true
        }
    }
}
impl Drop for DirectChatWindow {
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
unsafe fn clear_edit(s: &State, index: usize) {
    if let Some(c) = s.controls.get() {
        let context = ImmGetContext(c[index]);
        if !context.0.is_null() {
            let active = s.gate.get().composing
                || ImmGetCompositionStringW(context, GCS_COMPSTR, None, 0) > 0;
            let cancelled =
                !active || ImmNotifyIME(context, NI_COMPOSITIONSTR, CPS_CANCEL, 0).as_bool();
            let _ = ImmReleaseContext(c[index], context);
            if !cancelled {
                let _ = DestroyWindow(s.hwnd.get());
                return;
            }
        }
        let _ = set_text(c[index], "");
        SendMessageW(c[index], EM_EMPTYUNDOBUFFER, None, None);
    }
    s.gate.set(InputGate::default());
}
unsafe fn clear_input(s: &State) {
    clear_edit(s, INPUT);
}
unsafe fn clear_draft(s: &State) {
    clear_edit(s, NAME);
    clear_edit(s, INPUT);
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
unsafe fn edit_text(h: HWND, limit: usize) -> Result<String, &'static str> {
    let len = GetWindowTextLengthW(h);
    if len < 0 || len as usize > limit {
        return Err("輸入超過上限，未送出。");
    }
    let mut buf = vec![0u16; len as usize + 1];
    if GetWindowTextW(h, &mut buf) != len {
        return Err("無法讀取完整文字，未送出。");
    }
    String::from_utf16(&buf[..len as usize]).map_err(|_| "文字編碼不完整，未送出。")
}
unsafe fn request_action(s: &State, action: Action) {
    let Some(c) = s.controls.get() else {
        return;
    };
    if !s.gate.get().can_submit(
        false,
        native_composing(c[INPUT]) || native_composing(c[NAME]),
    ) {
        set_status(s, "請先完成中文組字，再操作私訊。");
        return;
    }
    let result = (|| -> Result<Intent, &'static str> {
        let view = s.view.borrow();
        let target = match action {
            Action::Send | Action::Block => {
                let index = SendMessageW(c[SOURCE], CB_GETCURSEL, None, None).0;
                let source = usize::try_from(index)
                    .ok()
                    .and_then(|i| view.sources.get(i))
                    .ok_or("請選擇收件伺服器。")?;
                Some(Address {
                    source_id: source.id,
                    name: edit_text(c[NAME], 256)?,
                })
            }
            Action::Unblock => {
                let index = SendMessageW(c[BLOCKED], LB_GETCURSEL, None, None).0;
                Some(
                    usize::try_from(index)
                        .ok()
                        .and_then(|i| view.chat.blocked().get(i))
                        .ok_or("請先選取拒收名單中的角色。")?
                        .clone(),
                )
            }
            _ => None,
        };
        let preferences = if action == Action::Preferences {
            let mut p = view.chat.preferences().ok_or("接收設定尚未確認。")?;
            p.whispers = SendMessageW(c[RECEIVE], BM_GETCHECK, None, None).0 == 1;
            if view.filtered_public {
                p.world = SendMessageW(c[WORLD_RECEIVE], BM_GETCHECK, None, None).0 == 1;
                p.trade = SendMessageW(c[TRADE_RECEIVE], BM_GETCHECK, None, None).0 == 1;
            }
            Some(p)
        } else {
            None
        };
        let text = if action == Action::Send {
            edit_text(c[INPUT], INPUT_LIMIT)?
        } else {
            String::new()
        };
        s.draft
            .borrow_mut()
            .intent(&view, action, target, preferences, text)
    })();
    match result {
        Ok(intent) => {
            *s.action.borrow_mut() = Some(intent);
            for i in [
                SEND,
                SOURCE,
                NAME,
                RECEIVE,
                WORLD_RECEIVE,
                TRADE_RECEIVE,
                APPLY,
                BLOCK,
                BLOCKED,
                UNBLOCK,
                CLEAR,
            ] {
                let _ = EnableWindow(c[i], false);
            }
            SendMessageW(c[INPUT], EM_SETREADONLY, Some(WPARAM(1)), None);
            set_status(s, "正在提交確認；不會自動重送…");
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
            SCOPE => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32 | ES_READONLY as u32,
            ),
            INPUT | NAME => (
                w!("EDIT"),
                "",
                WS_TABSTOP.0 | WS_BORDER.0 | ES_AUTOHSCROLL as u32,
            ),
            SOURCE => (
                w!("COMBOBOX"),
                "",
                WS_TABSTOP.0 | WS_VSCROLL.0 | CBS_DROPDOWNLIST as u32,
            ),
            BLOCKED => (
                w!("LISTBOX"),
                "",
                WS_TABSTOP.0
                    | WS_BORDER.0
                    | WS_VSCROLL.0
                    | WS_HSCROLL.0
                    | LBS_NOINTEGRALHEIGHT as u32
                    | LBS_NOTIFY as u32,
            ),
            RECEIVE | WORLD_RECEIVE | TRADE_RECEIVE => (
                w!("BUTTON"),
                match i {
                    WORLD_RECEIVE => "接收世界",
                    TRADE_RECEIVE => "接收交易",
                    _ => "接收私訊",
                },
                WS_TABSTOP.0 | BS_AUTOCHECKBOX as u32,
            ),
            SEND | CLOSE | APPLY | BLOCK | UNBLOCK | CLEAR => (
                w!("BUTTON"),
                match i {
                    SEND => "私訊送出 (Enter)",
                    CLOSE => "關閉私訊",
                    APPLY => "套用接收設定",
                    BLOCK => "拒收上述角色",
                    UNBLOCK => "解除所選拒收",
                    _ => "清空拒收名單",
                },
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
            ),
            _ => (
                w!("STATIC"),
                if i == HINT {
                    "依序選伺服器、填角色名，再輸入訊息；.de 僅為文字。\r\n拒收最多16名，依伺服器區分。"
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
    SendMessageW(c[NAME], EM_SETLIMITTEXT, Some(WPARAM(256)), None);
    SendMessageW(c[INPUT], EM_SETLIMITTEXT, Some(WPARAM(INPUT_LIMIT)), None);
    SendMessageW(c[HISTORY], EM_SETLIMITTEXT, Some(WPARAM(128 * 1024)), None);
    Ok(c)
}
unsafe fn monitor_work(h: HWND, suggested: Option<&RECT>) -> Option<BoxRect> {
    let monitor = if let Some(r) = suggested {
        MonitorFromRect(r, MONITOR_DEFAULTTONEAREST)
    } else {
        MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST)
    };
    let mut info = MONITORINFO {
        cbSize: std::mem::size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    if !GetMonitorInfoW(monitor, &mut info).as_bool() {
        return None;
    }
    let r = info.rcWork;
    (r.right > r.left && r.bottom > r.top).then_some(BoxRect {
        x: r.left,
        y: r.top,
        w: r.right - r.left,
        h: r.bottom - r.top,
    })
}
unsafe fn place(h: HWND, r: BoxRect) -> Result<()> {
    SetWindowPos(h, None, r.x, r.y, r.w, r.h, SWP_NOZORDER | SWP_NOACTIVATE)?;
    Ok(())
}
unsafe fn resize(s: &State) {
    let Some(c) = s.controls.get() else {
        return;
    };
    if s.resizing.replace(true) {
        return;
    }
    (|| {
        let dpi = GetDpiForWindow(s.hwnd.get());
        let mut r = RECT::default();
        // Showing/hiding a native scrollbar changes the client area. Recompute
        // after both bars settle, rather than laying out against old dimensions.
        for _ in 0..3 {
            if GetClientRect(s.hwnd.get(), &mut r).is_err() {
                return;
            }
            let (w, h) = content_size(r.right, r.bottom, dpi);
            for (bar, position, extent, page) in [
                (SB_HORZ, &s.scroll_x, w, r.right),
                (SB_VERT, &s.scroll_y, h, r.bottom),
            ] {
                position.set(position.get().clamp(0, (extent - page).max(0)));
                SetScrollInfo(
                    s.hwnd.get(),
                    bar,
                    &SCROLLINFO {
                        cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
                        fMask: SIF_RANGE | SIF_PAGE | SIF_POS,
                        nMin: 0,
                        nMax: extent - 1,
                        nPage: page.max(1) as u32,
                        nPos: position.get(),
                        ..Default::default()
                    },
                    true,
                );
            }
        }
        if GetClientRect(s.hwnd.get(), &mut r).is_err() {
            return;
        }
        SendMessageW(
            c[BLOCKED],
            LB_SETHORIZONTALEXTENT,
            Some(WPARAM(scale(8400, dpi) as usize)),
            None,
        );
        for (h, b) in c.into_iter().zip(layout(r.right, r.bottom, dpi)) {
            let _ = MoveWindow(
                h,
                b.x - s.scroll_x.get(),
                b.y - s.scroll_y.get(),
                b.w,
                b.h,
                true,
            );
        }
        let _ = InvalidateRect(Some(s.hwnd.get()), None, true);
    })();
    s.resizing.set(false);
}
unsafe fn reveal_focus(s: &State) {
    let Some(c) = s.controls.get() else {
        return;
    };
    let focus = GetFocus();
    let Some(index) = c
        .iter()
        .position(|&h| h == focus || IsChild(h, focus).as_bool())
    else {
        return;
    };
    let mut r = RECT::default();
    if GetClientRect(s.hwnd.get(), &mut r).is_err() {
        return;
    }
    let dpi = GetDpiForWindow(s.hwnd.get());
    let (w, h) = content_size(r.right, r.bottom, dpi);
    let b = layout(r.right, r.bottom, dpi)[index];
    s.scroll_x
        .set(reveal_axis(s.scroll_x.get(), b.x, b.w, r.right, w));
    s.scroll_y.set(reveal_axis(
        s.scroll_y.get(),
        b.y,
        if index == SOURCE { scale(32, dpi) } else { b.h },
        r.bottom,
        h,
    ));
    resize(s);
}
unsafe fn scroll_bar(s: &State, vertical: bool, code: u32) {
    let mut r = RECT::default();
    if GetClientRect(s.hwnd.get(), &mut r).is_err() {
        return;
    }
    let dpi = GetDpiForWindow(s.hwnd.get());
    let (w, h) = content_size(r.right, r.bottom, dpi);
    let (bar, position, extent, page) = if vertical {
        (SB_VERT, &s.scroll_y, h, r.bottom)
    } else {
        (SB_HORZ, &s.scroll_x, w, r.right)
    };
    let mut info = SCROLLINFO {
        cbSize: std::mem::size_of::<SCROLLINFO>() as u32,
        fMask: SIF_TRACKPOS,
        ..Default::default()
    };
    let next = if code == SB_LINEUP.0 as u32 {
        position.get() - scale(24, dpi)
    } else if code == SB_LINEDOWN.0 as u32 {
        position.get() + scale(24, dpi)
    } else if code == SB_PAGEUP.0 as u32 {
        position.get() - page
    } else if code == SB_PAGEDOWN.0 as u32 {
        position.get() + page
    } else if code == SB_TOP.0 as u32 {
        0
    } else if code == SB_BOTTOM.0 as u32 {
        extent
    } else if (code == SB_THUMBTRACK.0 as u32 || code == SB_THUMBPOSITION.0 as u32)
        && GetScrollInfo(s.hwnd.get(), bar, &mut info).is_ok()
    {
        info.nTrackPos
    } else {
        return;
    };
    position.set(next.clamp(0, (extent - page).max(0)));
    resize(s);
}
unsafe fn wheel(s: &State, w: WPARAM) {
    let delta = ((w.0 >> 16) as u16 as i16) as i32 + s.wheel.get();
    let steps = delta / 120;
    s.wheel.set(delta % 120);
    if steps != 0 {
        s.scroll_y.set(
            s.scroll_y
                .get()
                .saturating_sub(steps * scale(48, GetDpiForWindow(s.hwnd.get()))),
        );
        resize(s);
    }
}
unsafe extern "system" fn control_proc(
    h: HWND,
    m: u32,
    w: WPARAM,
    l: LPARAM,
    _id: usize,
    data: usize,
) -> LRESULT {
    let s = &*(data as *const State);
    if m == WM_NCDESTROY {
        let _ = RemoveWindowSubclass(h, Some(control_proc), 2);
    }
    let result = DefSubclassProc(h, m, w, l);
    if m == WM_SETFOCUS {
        reveal_focus(s);
    }
    result
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
        WM_VSCROLL | WM_HSCROLL if l.0 == 0 => {
            scroll_bar(s, m == WM_VSCROLL, (w.0 & 65535) as u32);
            LRESULT(0)
        }
        WM_MOUSEWHEEL => {
            wheel(s, w);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            if l.0 != 0 {
                let r = &*(l.0 as *const RECT);
                if let Some(area) = monitor_work(h, Some(r)) {
                    let _ = place(
                        h,
                        fit_outer(
                            BoxRect {
                                x: r.left,
                                y: r.top,
                                w: r.right - r.left,
                                h: r.bottom - r.top,
                            },
                            area,
                        ),
                    );
                }
            }
            let _ = replace_font(s);
            resize(s);
            reveal_focus(s);
            LRESULT(0)
        }
        WM_DISPLAYCHANGE | WM_SETTINGCHANGE => {
            let mut r = RECT::default();
            if !IsIconic(h).as_bool() && !IsZoomed(h).as_bool() && GetWindowRect(h, &mut r).is_ok()
            {
                if let Some(area) = monitor_work(h, None) {
                    let _ = place(
                        h,
                        fit_outer(
                            BoxRect {
                                x: r.left,
                                y: r.top,
                                w: r.right - r.left,
                                h: r.bottom - r.top,
                            },
                            area,
                        ),
                    );
                }
            }
            resize(s);
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let dpi = GetDpiForWindow(h);
            let mut r = RECT {
                right: scale(360, dpi),
                bottom: scale(240, dpi),
                ..Default::default()
            };
            if l.0 != 0 && AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, dpi).is_ok() {
                let info = &mut *(l.0 as *mut MINMAXINFO);
                let area = monitor_work(h, None);
                info.ptMinTrackSize.x = (r.right - r.left).min(area.map_or(i32::MAX, |a| a.w));
                info.ptMinTrackSize.y = (r.bottom - r.top).min(area.map_or(i32::MAX, |a| a.h));
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            if let Some(c) = s.controls.get() {
                let id = (w.0 & 65535).checked_sub(FIRST);
                let n = w.0 >> 16;
                if n == BN_CLICKED as usize {
                    if id == Some(CLOSE) && l.0 == c[CLOSE].0 as isize {
                        let _ = DestroyWindow(h);
                    }
                    for (i, a) in [
                        (SEND, Action::Send),
                        (APPLY, Action::Preferences),
                        (BLOCK, Action::Block),
                        (UNBLOCK, Action::Unblock),
                        (CLEAR, Action::ClearBlocks),
                    ] {
                        if id == Some(i) && l.0 == c[i].0 as isize {
                            request_action(s, a);
                        }
                    }
                }
                if id == Some(SOURCE) && n == CBN_SELCHANGE as usize {
                    clear_draft(s);
                    set_status(s, "收件伺服器已變更；請重新輸入角色與訊息。");
                }
                if (id == Some(INPUT) || id == Some(NAME)) && n == EN_MAXTEXT as usize {
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
    include!("direct_chat_window_tests.rs");
}
