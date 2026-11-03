//! Modeless, owned presentation settings. Closing this window never sends a
//! game command, closes its owner, or posts WM_QUIT. File IO belongs to one
//! PreferenceSession worker, kept alive by the mode/login message pump.
use super::preferences::{PreferenceSession, Preferences};
use anyhow::{ensure, Result};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
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
            HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow},
            Input::KeyboardAndMouse::{EnableWindow, GetFocus, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};

const INTRO: usize = 0;
const SOUND: usize = 1;
const VOLUME_LABEL: usize = 2;
const VOLUME: usize = 3;
const BACKGROUND: usize = 4;
const NOTE: usize = 5;
const STATUS: usize = 6;
const APPLY: usize = 7;
const RESET: usize = 8;
const CLOSE: usize = 9;
const COUNT: usize = 10;
const FIRST: usize = 700;
const STYLE: WINDOW_STYLE = WS_OVERLAPPEDWINDOW;
const EX: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
static NEXT: AtomicU64 = AtomicU64::new(1);
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
fn layout(width: i32, height: i32, dpi: u32) -> [BoxRect; COUNT] {
    let width = width.max(scale(640, dpi));
    let height = height.max(scale(440, dpi));
    let margin = scale(20, dpi);
    let full = width - 2 * margin;
    let mut r = [BoxRect::default(); COUNT];
    for (i, y, h) in [
        (INTRO, 20, 44),
        (SOUND, 76, 30),
        (VOLUME_LABEL, 118, 30),
        (VOLUME, 118, 30),
        (BACKGROUND, 160, 30),
        (NOTE, 206, 46),
        (STATUS, 268, 76),
    ] {
        r[i] = BoxRect {
            x: margin,
            y: scale(y, dpi),
            w: full,
            h: scale(h, dpi),
        };
    }
    r[VOLUME_LABEL].w = scale(140, dpi);
    r[VOLUME].x += scale(152, dpi);
    r[VOLUME].w = scale(180, dpi);
    let y = height - margin - scale(36, dpi);
    for (i, x, w) in [(APPLY, 0, 140), (RESET, 152, 200), (CLOSE, 364, 236)] {
        r[i] = BoxRect {
            x: margin + scale(x, dpi),
            y,
            w: scale(w, dpi),
            h: scale(36, dpi),
        };
    }
    r
}

#[derive(Default)]
pub struct SettingsController {
    session: Option<Rc<RefCell<PreferenceSession>>>,
    window: Option<SettingsWindow>,
    closing: bool,
}
impl SettingsController {
    #[cfg(test)]
    pub(crate) fn start_isolated_test(&mut self) -> Result<()> {
        ensure!(
            !self.closing && self.session.is_none() && self.window.is_none(),
            "test settings require a fresh owner"
        );
        self.session = Some(Rc::new(RefCell::new(PreferenceSession::isolated_test(
            super::preferences::shared(),
        )?)));
        Ok(())
    }
    pub fn start(&mut self) -> Result<()> {
        ensure!(!self.closing, "settings owner is closing");
        if self.session.is_none() {
            self.session = Some(Rc::new(RefCell::new(PreferenceSession::new()?)));
        }
        Ok(())
    }
    pub fn show(&mut self, owner: HWND) -> Result<()> {
        self.start()?;
        self.poll();
        if let Some(window) = &self.window {
            unsafe {
                let _ = SetForegroundWindow(window.state.hwnd.get());
            }
            return Ok(());
        }
        self.window = Some(SettingsWindow::new(
            owner,
            self.session.as_ref().unwrap().clone(),
        )?);
        Ok(())
    }
    pub fn poll(&mut self) {
        if let Some(session) = &self.session {
            session.borrow_mut().poll();
        }
        if let Some(window) = &self.window {
            if window.state.hwnd.get().0.is_null() {
                self.window = None;
            } else {
                unsafe {
                    window.refresh();
                }
            }
        }
    }
    pub fn close(&mut self) {
        self.closing = true;
        self.window = None;
        if let Some(session) = &self.session {
            session.borrow_mut().close();
        }
    }
    pub fn is_quiescent(&self) -> bool {
        self.session
            .as_ref()
            .is_none_or(|s| s.borrow().is_quiescent())
    }
    pub fn pre_translate(&self, message: &MSG) -> bool {
        let Some(window) = &self.window else {
            return false;
        };
        let hwnd = window.state.hwnd.get();
        unsafe {
            if hwnd.0.is_null() || !(message.hwnd == hwnd || IsChild(hwnd, message.hwnd).as_bool())
            {
                return false;
            }
            if message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {
                let _ = DestroyWindow(hwnd);
                return true;
            }
            // Enter operates only the focused button, never a hidden parent
            // default (login, room start, or exit) while editing preferences.
            if message.message == WM_KEYDOWN && message.wParam.0 == 0x0d {
                let focus = GetFocus();
                if let Some(c) = window.state.controls.get() {
                    for i in [APPLY, RESET, CLOSE] {
                        if focus == c[i] {
                            let _ = SendMessageW(c[i], BM_CLICK, None, None);
                            return true;
                        }
                    }
                }
                let _ = TranslateMessage(message);
                DispatchMessageW(message);
                return true;
            }
            if !IsDialogMessageW(hwnd, message).as_bool() {
                let _ = TranslateMessage(message);
                DispatchMessageW(message);
            }
            true
        }
    }
}

struct State {
    hwnd: Cell<HWND>,
    controls: Cell<Option<[HWND; COUNT]>>,
    font: Cell<HFONT>,
    session: Rc<RefCell<PreferenceSession>>,
    initialized: Cell<bool>,
    last_status: RefCell<String>,
}
struct SettingsWindow {
    state: Box<State>,
    class: Vec<u16>,
    instance: HINSTANCE,
}
impl SettingsWindow {
    fn new(owner: HWND, session: Rc<RefCell<PreferenceSession>>) -> Result<Self> {
        unsafe {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            let class = wide(&format!(
                "LiToModernSettings{}",
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
                "register settings window"
            );
            let mut window = Self {
                state: Box::new(State {
                    hwnd: Cell::new(HWND::default()),
                    controls: Cell::new(None),
                    font: Cell::new(HFONT::default()),
                    session,
                    initialized: Cell::new(false),
                    last_status: RefCell::new(String::new()),
                }),
                class,
                instance,
            };
            let hwnd = CreateWindowExW(
                EX,
                PCWSTR(window.class.as_ptr()),
                w!("LiTo — 遊戲設定"),
                STYLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                700,
                510,
                Some(owner),
                None,
                Some(instance),
                Some((&mut *window.state as *mut State).cast()),
            )?;
            window.state.hwnd.set(hwnd);
            window
                .state
                .controls
                .set(Some(create_controls(hwnd, instance)?));
            replace_font(&window.state)?;
            let dpi = GetDpiForWindow(hwnd);
            let mut rect = RECT {
                right: scale(680, dpi),
                bottom: scale(440, dpi),
                ..Default::default()
            };
            AdjustWindowRectExForDpi(&mut rect, STYLE, false, EX, dpi)?;
            SetWindowPos(
                hwnd,
                None,
                0,
                0,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE,
            )?;
            resize(&window.state);
            window.refresh();
            let _ = ShowWindow(hwnd, SW_SHOW);
            let _ = SetFocus(Some(window.state.controls.get().unwrap()[CLOSE]));
            Ok(window)
        }
    }
    unsafe fn refresh(&self) {
        let s = &self.state;
        let Some(c) = s.controls.get() else {
            return;
        };
        let session = s.session.borrow();
        let ready = session.can_apply();
        if ready && !s.initialized.replace(true) {
            write_controls(c, session.values.get());
        }
        for i in [SOUND, VOLUME, BACKGROUND, APPLY, RESET] {
            let _ = EnableWindow(c[i], ready);
        }
        let status = session.status();
        if *s.last_status.borrow() != status {
            let text = wide(status);
            let _ = SetWindowTextW(c[STATUS], PCWSTR(text.as_ptr()));
            *s.last_status.borrow_mut() = status.into();
        }
    }
}
impl Drop for SettingsWindow {
    fn drop(&mut self) {
        unsafe {
            let hwnd = self.state.hwnd.get();
            if !hwnd.0.is_null() {
                let _ = DestroyWindow(hwnd);
            }
            let font = self.state.font.get();
            if !font.0.is_null() {
                let _ = DeleteObject(font.into());
            }
            let _ = UnregisterClassW(PCWSTR(self.class.as_ptr()), Some(self.instance));
        }
    }
}
unsafe fn create_controls(parent: HWND, instance: HINSTANCE) -> Result<[HWND; COUNT]> {
    let mut controls = [HWND::default(); COUNT];
    for (i, control) in controls.iter_mut().enumerate() {
        let (class,label,style)=match i{
            SOUND|BACKGROUND=>(w!("BUTTON"),if i==SOUND{"啟用遊戲音效"}else{"程式在背景時仍播放音效"},WS_TABSTOP.0|BS_AUTOCHECKBOX as u32),
            VOLUME=>(w!("COMBOBOX"),"",WS_TABSTOP.0|WS_VSCROLL.0|CBS_DROPDOWNLIST as u32),
            APPLY|RESET|CLOSE=>(w!("BUTTON"),match i{APPLY=>"套用並保存",RESET=>"恢復預設（待套用）",_=>"關閉（捨棄未套用）"},WS_TABSTOP.0|BS_PUSHBUTTON as u32),
            _=>(w!("STATIC"),match i{INTRO=>"現代 LiTo 的個人遊戲設定\r\n只影響本程式，不修改 TW 或 Windows 裝置音量。",VOLUME_LABEL=>"音效音量",NOTE=>"設定保存在本機使用者的 LiToModern 目錄。\r\n靜音不會停止連線、戰鬥更新或取消已送出的操作。",_=>""},0x80),
        };
        let label = wide(label);
        *control = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            class,
            PCWSTR(label.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | WS_VISIBLE.0 | style),
            0,
            0,
            0,
            0,
            Some(parent),
            Some(HMENU((FIRST + i) as *mut _)),
            Some(instance),
            None,
        )?;
    }
    for percent in 0..=100 {
        let text = wide(&format!("{percent} %"));
        ensure!(
            SendMessageW(
                controls[VOLUME],
                CB_ADDSTRING,
                None,
                Some(LPARAM(text.as_ptr() as isize))
            )
            .0 >= 0,
            "add volume option"
        );
    }
    Ok(controls)
}
unsafe fn write_controls(c: [HWND; COUNT], p: Preferences) {
    let _ = SendMessageW(
        c[SOUND],
        BM_SETCHECK,
        Some(WPARAM(usize::from(p.sound_enabled))),
        None,
    );
    let _ = SendMessageW(
        c[BACKGROUND],
        BM_SETCHECK,
        Some(WPARAM(usize::from(p.background_sound))),
        None,
    );
    let _ = SendMessageW(
        c[VOLUME],
        CB_SETCURSEL,
        Some(WPARAM(usize::from(p.volume_percent))),
        None,
    );
}
unsafe fn read_controls(c: [HWND; COUNT]) -> Result<Preferences> {
    let sound = SendMessageW(c[SOUND], BM_GETCHECK, None, None).0;
    let background = SendMessageW(c[BACKGROUND], BM_GETCHECK, None, None).0;
    let volume = SendMessageW(c[VOLUME], CB_GETCURSEL, None, None).0;
    ensure!(
        (0..=1).contains(&sound) && (0..=1).contains(&background) && (0..=100).contains(&volume),
        "invalid settings selection"
    );
    Ok(Preferences {
        sound_enabled: sound == 1,
        background_sound: background == 1,
        volume_percent: volume as u8,
    })
}
unsafe fn resize(s: &State) {
    let Some(controls) = s.controls.get() else {
        return;
    };
    let hwnd = s.hwnd.get();
    let mut client = RECT::default();
    if GetClientRect(hwnd, &mut client).is_err() {
        return;
    }
    let dpi = GetDpiForWindow(hwnd);
    for (i, (control, r)) in controls
        .iter()
        .zip(layout(client.right, client.bottom, dpi))
        .enumerate()
    {
        let h = if i == VOLUME { scale(260, dpi) } else { r.h };
        let _ = MoveWindow(*control, r.x, r.y, r.w, h, true);
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
    ensure!(!font.0.is_null(), "settings font unavailable");
    if let Some(controls) = s.controls.get() {
        for control in controls {
            let _ = SendMessageW(
                control,
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
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as _);
        (*(create.lpCreateParams as *mut State)).hwnd.set(hwnd);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if ptr.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    let s = &*ptr;
    match message {
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            s.hwnd.set(HWND::default());
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        WM_SIZE => {
            resize(s);
            LRESULT(0)
        }
        WM_DPICHANGED => {
            if lparam.0 != 0 {
                let r = &*(lparam.0 as *const RECT);
                let _ = SetWindowPos(
                    hwnd,
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
            let mut r = RECT {
                right: scale(640, GetDpiForWindow(hwnd)),
                bottom: scale(440, GetDpiForWindow(hwnd)),
                ..Default::default()
            };
            if AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, GetDpiForWindow(hwnd)).is_ok() {
                let m = &mut *(lparam.0 as *mut MINMAXINFO);
                m.ptMinTrackSize.x = r.right - r.left;
                m.ptMinTrackSize.y = r.bottom - r.top;
            }
            LRESULT(0)
        }
        WM_COMMAND => {
            let Some(c) = s.controls.get() else {
                return LRESULT(0);
            };
            let index = (wparam.0 & 65535).checked_sub(FIRST);
            let notification = (wparam.0 >> 16) & 65535;
            if notification == BN_CLICKED as usize {
                match index {
                    Some(CLOSE) if lparam.0 == c[CLOSE].0 as isize => {
                        let _ = DestroyWindow(hwnd);
                    }
                    Some(RESET)
                        if lparam.0 == c[RESET].0 as isize && s.session.borrow().can_apply() =>
                    {
                        write_controls(c, Preferences::default())
                    }
                    Some(APPLY) if lparam.0 == c[APPLY].0 as isize => {
                        if let Ok(p) = read_controls(c) {
                            let _ = s.session.borrow_mut().apply(p);
                        }
                    }
                    _ => {}
                }
            }
            LRESULT(0)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn settings_controls_remain_client_relative_nonoverlapping_and_distinct_from_close() {
        for dpi in [96, 120, 144, 192, 288] {
            for extra in [0, 300] {
                let w = scale(640 + extra, dpi);
                let h = scale(440 + extra, dpi);
                let rows = layout(w, h, dpi);
                for (i, a) in rows.iter().enumerate() {
                    assert!(
                        a.x >= 0 && a.y >= 0 && a.x + a.w <= w && a.y + a.h <= h,
                        "{i} at {dpi}"
                    );
                    for b in rows.iter().skip(i + 1) {
                        assert!(
                            a.x + a.w <= b.x
                                || b.x + b.w <= a.x
                                || a.y + a.h <= b.y
                                || b.y + b.h <= a.y,
                            "overlap at {dpi}"
                        );
                    }
                }
                assert!(rows[APPLY].x < rows[RESET].x && rows[RESET].x < rows[CLOSE].x);
            }
        }
    }
}
