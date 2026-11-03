//! Same-process mode choice before any TW configuration/log/updater/hooks.
//!
//! The modern page loads only the explicitly entered or selected profile. It does not
//! connect, authenticate, launch a BIN, or guess resources. The separate game
//! settings window saves only explicitly applied modern presentation preferences.
//! Profile IO runs in one owned worker; cancellation invalidates its ticket,
//! while normal close pumps messages until that actual worker has retired.

use super::profile::{ModernProfile, ProfileError};
use super::profile_picker::{self, MAX_PATH_UTF16};
use anyhow::{ensure, Result};
use std::cell::{Cell, RefCell};
use std::ffi::OsString;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use std::path::PathBuf;
use std::thread::JoinHandle;
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
            Controls::EM_SETLIMITTEXT,
            HiDpi::{
                AdjustWindowRectExForDpi, GetDpiForWindow, SetThreadDpiAwarenessContext,
                DPI_AWARENESS_CONTEXT, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
            },
            Input::KeyboardAndMouse::{
                EnableWindow, GetFocus, GetKeyState, IsWindowEnabled, SetFocus,
            },
            WindowsAndMessaging::*,
        },
    },
};

pub enum ModeSelection {
    Tw,
    Modern(ModernProfile),
    Exit,
}

const CLASS: PCWSTR = w!("LiToModernModeSelectorV1");
const STYLE: WINDOW_STYLE = WS_OVERLAPPEDWINDOW;
const EX_STYLE: WINDOW_EX_STYLE = WS_EX_CONTROLPARENT;
const TIMER: usize = 0x4d31;
const FIRST_ID: usize = 400;
const MIN_WIDTH: i32 = 680;
const MIN_HEIGHT: i32 = 560;
const MAX_INFO_UTF16: usize = 64 * 1024;
const TITLE: usize = 0;
const INTRO: usize = 1;
const TW: usize = 2;
const TW_NOTE: usize = 3;
const MODERN: usize = 4;
const MODERN_NOTE: usize = 5;
const PATH_LABEL: usize = 6;
const PATH: usize = 7;
const LOAD: usize = 8;
const INFO: usize = 9;
const CONFIRM: usize = 10;
const BACK: usize = 11;
const STATUS: usize = 12;
const EXIT: usize = 13;
const SETTINGS: usize = 14;
const BROWSE: usize = 15;
const CONTROL_COUNT: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Page {
    Modes,
    Profile,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct BoxRect {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

fn scale(value: i32, dpi: u32) -> i32 {
    ((i64::from(value) * i64::from(dpi.clamp(48, 768)) + 48) / 96) as i32
}

fn layout(width: i32, height: i32, dpi: u32) -> [BoxRect; CONTROL_COUNT] {
    // Client dimensions only. Moving the outer window cannot alter hit regions;
    // native child BUTTONs own hit testing rather than desktop-coordinate math.
    let width = width.clamp(scale(MIN_WIDTH, dpi), 1_000_000);
    let height = height.clamp(scale(MIN_HEIGHT, dpi), 1_000_000);
    let margin = scale(24, dpi);
    let full = width - margin * 2;
    let gap = scale(12, dpi);
    let mut boxes = [BoxRect::default(); CONTROL_COUNT];
    let mut row = |index, y, h| {
        boxes[index] = BoxRect {
            x: margin,
            y: scale(y, dpi),
            width: full,
            height: scale(h, dpi),
        };
    };
    row(TITLE, 20, 36);
    row(INTRO, 64, 48);
    row(TW, 132, 48);
    row(TW_NOTE, 188, 48);
    row(MODERN, 256, 48);
    row(MODERN_NOTE, 312, 64);
    row(PATH_LABEL, 124, 24);
    row(PATH, 156, 32);
    row(LOAD, 156, 32);
    row(BROWSE, 156, 32);
    row(INFO, 204, 220);
    let load_width = scale(116, dpi);
    boxes[PATH].width = full - (load_width + gap) * 2;
    boxes[BROWSE].x = width - margin - load_width * 2 - gap;
    boxes[BROWSE].width = load_width;
    boxes[LOAD].x = width - margin - load_width;
    boxes[LOAD].width = load_width;
    let bottom = height - margin - scale(36, dpi);
    boxes[STATUS] = BoxRect {
        x: margin,
        y: bottom - scale(64, dpi),
        width: full,
        height: scale(56, dpi),
    };
    // The details EDIT grows with the window, leaving status/navigation intact.
    boxes[INFO].height = boxes[STATUS].y - scale(216, dpi);
    for (index, x, button_width) in [
        (BACK, margin, scale(116, dpi)),
        (SETTINGS, margin, scale(180, dpi)),
        (CONFIRM, margin + scale(128, dpi), scale(248, dpi)),
        (EXIT, width - margin - scale(104, dpi), scale(104, dpi)),
    ] {
        boxes[index] = BoxRect {
            x,
            y: bottom,
            width: button_width,
            height: scale(36, dpi),
        };
    }
    boxes
}

fn visible(page: Page, control: usize) -> bool {
    matches!(control, TITLE | INTRO | STATUS | EXIT)
        || match page {
            Page::Modes => matches!(control, TW | TW_NOTE | MODERN | MODERN_NOTE | SETTINGS),
            Page::Profile => matches!(
                control,
                PATH_LABEL | PATH | BROWSE | LOAD | INFO | CONFIRM | BACK
            ),
        }
}

fn actionable(page: Page, control: usize, loading: bool, ready: bool, closing: bool) -> bool {
    if closing || !visible(page, control) {
        return false;
    }
    match control {
        EXIT | BACK => true,
        TW | MODERN | PATH | BROWSE | LOAD | SETTINGS => !loading,
        CONFIRM => !loading && ready,
        _ => false,
    }
}

/// Selection only: never returns an action or triggers a default button. The
/// read-only multiline information EDIT remains reachable for selecting/copying
/// text, but cannot retain Tab instead of yielding to the confirmation button.
fn tabbable(page: Page, control: usize, loading: bool, ready: bool, closing: bool) -> bool {
    !closing
        && visible(page, control)
        && (control == INFO || actionable(page, control, loading, ready, closing))
}

fn next_tab_stop(
    page: Page,
    current: Option<usize>,
    backward: bool,
    available: &[bool; CONTROL_COUNT],
) -> Option<usize> {
    let order: &[usize] = match page {
        Page::Modes => &[TW, MODERN, SETTINGS, EXIT],
        Page::Profile => &[PATH, BROWSE, LOAD, INFO, CONFIRM, BACK, EXIT],
    };
    if let Some(start) =
        current.and_then(|current| order.iter().position(|index| *index == current))
    {
        for distance in 1..=order.len() {
            let position = if backward {
                (start + order.len() - distance) % order.len()
            } else {
                (start + distance) % order.len()
            };
            let candidate = order[position];
            if available[candidate] {
                return Some(candidate);
            }
        }
        None
    } else if backward {
        order.iter().rev().copied().find(|index| available[*index])
    } else {
        order.iter().copied().find(|index| available[*index])
    }
}

unsafe fn route_tab(hwnd: HWND, state: &WindowState, message: &MSG) -> Result<bool> {
    // Preserve Ctrl+Tab and Alt+Tab/window switching; only ordinary dialog
    // traversal (including Shift+Tab) belongs to this explicit focus order.
    if message.message != WM_KEYDOWN
        || message.wParam.0 != 0x09 // VK_TAB
        || (message.hwnd != hwnd && !IsChild(hwnd, message.hwnd).as_bool())
        || GetKeyState(0x11) < 0
        || GetKeyState(0x12) < 0
    {
        return Ok(false);
    }
    // Copy plain state before native calls, which can synchronously re-enter
    // WM_COMMAND on focus notifications. No RefCell borrow crosses SetFocus.
    let (page, loading, ready, closing) = {
        let model = state.model.borrow();
        (
            model.page,
            model.busy(),
            model.profile.is_some(),
            model.closing,
        )
    };
    let Some(controls) = state.controls.get() else {
        return Ok(true);
    };
    let focus = GetFocus();
    let current = controls.iter().position(|control| *control == focus);
    let mut available = [false; CONTROL_COUNT];
    for (index, control) in controls.iter().copied().enumerate() {
        available[index] = tabbable(page, index, loading, ready, closing)
            && IsWindow(Some(control)).as_bool()
            && IsChild(hwnd, control).as_bool()
            && IsWindowVisible(control).as_bool()
            && IsWindowEnabled(control).as_bool()
            && (GetWindowLongPtrW(control, GWL_STYLE) as u32 & WS_TABSTOP.0) != 0;
    }
    if let Some(index) = next_tab_stop(page, current, GetKeyState(0x10) < 0, &available) {
        let _ = SetFocus(Some(controls[index]));
        ensure!(
            GetFocus() == controls[index],
            "mode keyboard focus transfer failed"
        );
    }
    // Do not TranslateMessage/IsDialogMessage this Tab again: that could send
    // a tab character to the multiline EDIT or advance twice. Key-up is normal.
    Ok(true)
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct LoadTicket {
    generation: u64,
    path: PathBuf,
}

fn accepts_load(
    active: Option<&LoadTicket>,
    completed: &LoadTicket,
    page: Page,
    closing: bool,
) -> bool {
    !closing && page == Page::Profile && active == Some(completed)
}

struct LoadJob {
    ticket: LoadTicket,
    worker: JoinHandle<std::result::Result<ModernProfile, ProfileError>>,
}

struct Model {
    page: Page,
    generation: u64,
    active: Option<LoadTicket>,
    profile: Option<ModernProfile>,
    job: Option<LoadJob>,
    picking: Option<u64>,
    closing: bool,
    choice: Option<ModeSelection>,
    failure: Option<&'static str>,
    status: String,
}

impl Model {
    fn busy(&self) -> bool {
        self.job.is_some() || self.picking.is_some()
    }

    fn begin_pick(&mut self) -> Result<u64> {
        ensure!(
            self.page == Page::Profile && !self.closing && !self.busy(),
            "profile picker unavailable"
        );
        // Do not invalidate a loaded profile until the user selects a new path.
        self.picking = Some(self.generation);
        Ok(self.generation)
    }

    fn finish_pick(&mut self, generation: u64) -> bool {
        if self.picking != Some(generation) {
            return false;
        }
        self.picking = None;
        !self.closing && self.page == Page::Profile && self.generation == generation
    }

    fn retire_remaining_worker(&mut self) {
        // Exceptional teardown only. Normal WM_CLOSE reaps via is_finished
        // while the visible host continues pumping messages.
        self.active = None;
        self.profile = None;
        if let Some(job) = self.job.take() {
            let _ = job.worker.join();
        }
    }

    fn invalidate_profile(&mut self) -> Result<()> {
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("profile load generation exhausted"))?;
        self.active = None;
        self.profile = None;
        Ok(())
    }

    fn close(&mut self) {
        self.active = None;
        self.profile = None;
        self.closing = true;
        self.status = "正在退出；請先關閉選檔視窗，並等待設定檔讀取工作結束。".into();
    }

    fn reap_completed(&mut self) -> bool {
        if !self
            .job
            .as_ref()
            .is_some_and(|job| job.worker.is_finished())
        {
            return false;
        }
        let job = self.job.take().unwrap();
        // The handle is joined only once it is actually finished, never by
        // pretending that invalidating a profile ticket cancels filesystem IO.
        let result = job.worker.join();
        if accepts_load(self.active.as_ref(), &job.ticket, self.page, self.closing) {
            match result {
                Ok(Ok(profile)) => {
                    self.profile = Some(profile);
                    self.status = "設定已載入。請確認名稱、共享資源與端點，再開啟現代登入。".into();
                }
                Ok(Err(error)) => {
                    self.active = None;
                    self.status = format!("設定無法載入：{error}。可修正路徑後重試。");
                }
                Err(_) => {
                    self.active = None;
                    self.status = "設定讀取工作失敗，可重新選擇設定。".into();
                }
            }
        } else if !self.closing {
            self.status = "已捨棄取消或過期的設定讀取結果；可重新選擇模式。".into();
        }
        true
    }
}

#[derive(Clone, PartialEq, Eq)]
struct View {
    page: Page,
    loading: bool,
    ready: bool,
    closing: bool,
    status: String,
    info: String,
}

struct WindowState {
    settings: RefCell<super::settings_window::SettingsController>,
    model: RefCell<Model>,
    controls: Cell<Option<[HWND; CONTROL_COUNT]>>,
    rendering: Cell<bool>,
    rendered: RefCell<Option<View>>,
    font: Cell<HFONT>,
}

impl Drop for WindowState {
    fn drop(&mut self) {
        // Normal teardown has already reaped it. A failed/destroyed message
        // pump or unwinding must never detach owned work.
        self.model.get_mut().retire_remaining_worker();
        let font = self.font.get();
        if !font.0.is_null() {
            unsafe {
                let _ = DeleteObject(font.into());
            }
        }
    }
}

/// Returns only after this host and its owned profile worker have shut down.
/// The caller performs either the complete TW flow or same-process modern login.
pub fn run() -> Result<ModeSelection> {
    let mut state = Box::new(WindowState {
        settings: RefCell::new(super::settings_window::SettingsController::default()),
        model: RefCell::new(Model {
            page: Page::Modes,
            generation: 0,
            active: None,
            profile: None,
            job: None,
            picking: None,
            closing: false,
            choice: None,
            failure: None,
            status: "請選擇模式；目前尚未載入 TW 設定，也未連線到伺服器。".into(),
        }),
        controls: Cell::new(None),
        rendering: Cell::new(false),
        rendered: RefCell::new(None),
        font: Cell::new(HFONT::default()),
    });
    unsafe {
        // Do not change process-wide DPI policy before the user chooses TW.
        // Its existing GUI retains the manifest/runtime policy after this guard.
        let previous_dpi = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        ensure!(!previous_dpi.0.is_null(), "set mode window DPI context");
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
            "register mode selection window"
        );
        let _class = ClassGuard(instance);
        let hwnd = CreateWindowExW(
            EX_STYLE,
            CLASS,
            w!("LiTo / TW — 選擇模式"),
            STYLE,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            760,
            640,
            None,
            None,
            Some(instance),
            Some((&mut *state as *mut WindowState).cast()),
        )?;
        let _window = WindowGuard(hwnd);
        state.controls.set(Some(create_controls(hwnd, instance)?));
        replace_font(hwnd, &state)?;
        let dpi = GetDpiForWindow(hwnd);
        let mut initial = RECT {
            left: 0,
            top: 0,
            right: scale(760, dpi),
            bottom: scale(MIN_HEIGHT, dpi),
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
        resize(hwnd, &state)?;
        render(hwnd, &state)?;
        ensure!(
            SetTimer(Some(hwnd), TIMER, 30, None) != 0,
            "create profile completion timer"
        );
        let _ = ShowWindow(hwnd, SW_SHOW);
        let _ = SetFocus(Some(state.controls.get().unwrap()[TW]));
        let mut message = MSG::default();
        loop {
            match GetMessageW(&mut message, None, 0, 0).0 {
                -1 => {
                    fail(hwnd, &state, "mode selection message pump failed");
                    // There is no working UI pump on this exceptional path.
                    // Retire real IO, rather than detach or spin on OS failure.
                    state.model.borrow_mut().retire_remaining_worker();
                    break;
                }
                0 => {
                    if state.model.borrow().job.is_none() && state.settings.borrow().is_quiescent()
                    {
                        break;
                    }
                    if !IsWindow(Some(hwnd)).as_bool() {
                        // External destruction kills the HWND timer. Do not
                        // wait for a WM_TIMER that can no longer be delivered.
                        let mut model = state.model.borrow_mut();
                        model.failure = Some("mode window destroyed during profile load");
                        model.choice = None;
                        model.close();
                        model.retire_remaining_worker();
                        break;
                    } else {
                        request_close(hwnd, &state);
                    }
                }
                _ => {
                    if state.settings.borrow().pre_translate(&message) {
                        continue;
                    }
                    match route_tab(hwnd, &state, &message) {
                        Ok(true) => continue,
                        Ok(false) => {}
                        Err(_) => {
                            fail(hwnd, &state, "cannot move keyboard focus in mode selection");
                            continue;
                        }
                    }
                    if message.message == WM_KEYDOWN && message.wParam.0 == 0x1b {
                        escape(hwnd, &state);
                    } else if message.message == WM_KEYDOWN && message.wParam.0 == 0x0d {
                        let focus = GetFocus();
                        if let Some(controls) = state.controls.get() {
                            if let Some(index) =
                                controls.iter().position(|control| *control == focus)
                            {
                                dispatch(hwnd, &state, if index == PATH { LOAD } else { index });
                            }
                        }
                    } else if !IsDialogMessageW(hwnd, &message).as_bool() {
                        let _ = TranslateMessage(&message);
                        DispatchMessageW(&message);
                    }
                }
            }
        }
    }
    let model = state.model.get_mut();
    if let Some(error) = model.failure {
        anyhow::bail!(error);
    }
    Ok(model.choice.take().unwrap_or(ModeSelection::Exit))
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

struct ThreadDpiGuard(DPI_AWARENESS_CONTEXT);
impl Drop for ThreadDpiGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = SetThreadDpiAwarenessContext(self.0);
        }
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

unsafe fn set_text(hwnd: HWND, text: &str) -> Result<()> {
    ensure!(
        !text.contains('\0') && text.encode_utf16().count() <= MAX_INFO_UTF16,
        "mode UI text exceeds safe bounds"
    );
    let text = wide(text);
    SetWindowTextW(hwnd, PCWSTR(text.as_ptr()))?;
    // An EDIT may report success while a platform limit truncates it. Verify
    // actual native contents, not just the source String's expected length.
    let expected_length = text.len() - 1;
    ensure!(
        GetWindowTextLengthW(hwnd) == expected_length as i32,
        "mode UI text was truncated"
    );
    let mut actual = vec![0; expected_length + 1];
    ensure!(
        GetWindowTextW(hwnd, &mut actual) == expected_length as i32 && actual == text,
        "mode UI text readback failed"
    );
    Ok(())
}

unsafe fn create_controls(parent: HWND, instance: HINSTANCE) -> Result<[HWND; CONTROL_COUNT]> {
    let mut controls = [HWND::default(); CONTROL_COUNT];
    for index in 0..CONTROL_COUNT {
        let label = match index {
            TW => "TW13081901.bin — 原版模式",
            TW_NOTE => {
                "使用既有 TW 登入器完整流程與原版 BIN。只有選擇後才讀取其設定、更新器與 hooks。"
            }
            MODERN => "LiTo — 現代客戶端",
            MODERN_NOTE => {
                "同一 EXE 的自研模式，不啟動舊 LiTo.bin。已接登入、選角、頻道與保存資料；戰鬥尚未完成。"
            }
            PATH_LABEL => "選擇 modern-profile.json，或貼上完整路徑後按「載入設定」",
            BROWSE => "選擇檔案…",
            LOAD => "載入設定",
            CONFIRM => "確認並開啟現代登入",
            BACK => "返回模式",
            EXIT => "退出",
            SETTINGS => "現代 LiTo 遊戲設定",
            _ => "",
        };
        let (class, extra, ex_style) = match index {
            TW | MODERN | BROWSE | LOAD | CONFIRM | BACK | EXIT | SETTINGS => (
                w!("BUTTON"),
                WS_TABSTOP.0 | BS_PUSHBUTTON as u32,
                WINDOW_EX_STYLE::default(),
            ),
            PATH => (
                w!("EDIT"),
                WS_TABSTOP.0 | ES_AUTOHSCROLL as u32,
                WS_EX_CLIENTEDGE,
            ),
            INFO => (
                w!("EDIT"),
                WS_TABSTOP.0
                    | WS_VSCROLL.0
                    | ES_MULTILINE as u32
                    | ES_AUTOVSCROLL as u32
                    | ES_READONLY as u32,
                WS_EX_CLIENTEDGE,
            ),
            _ => (w!("STATIC"), 0x80, WINDOW_EX_STYLE::default()), // SS_NOPREFIX
        };
        let label = wide(label);
        controls[index] = CreateWindowExW(
            ex_style,
            class,
            PCWSTR(label.as_ptr()),
            WINDOW_STYLE(WS_CHILD.0 | extra),
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
    let _ = SendMessageW(
        controls[PATH],
        EM_SETLIMITTEXT,
        Some(WPARAM(MAX_PATH_UTF16)),
        None,
    );
    let _ = SendMessageW(
        controls[INFO],
        EM_SETLIMITTEXT,
        Some(WPARAM(MAX_INFO_UTF16)),
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
    ensure!(!font.0.is_null(), "create mode UI font");
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

unsafe fn resize(hwnd: HWND, state: &WindowState) -> Result<()> {
    let Some(controls) = state.controls.get() else {
        return Ok(());
    };
    let mut client = RECT::default();
    GetClientRect(hwnd, &mut client)?;
    if client.right <= 0 || client.bottom <= 0 {
        return Ok(());
    }
    for (control, rect) in
        controls
            .into_iter()
            .zip(layout(client.right, client.bottom, GetDpiForWindow(hwnd)))
    {
        MoveWindow(control, rect.x, rect.y, rect.width, rect.height, true)?;
    }
    Ok(())
}

unsafe fn render(hwnd: HWND, state: &WindowState) -> Result<()> {
    if state.rendering.replace(true) {
        return Ok(());
    }
    struct Rendering<'a>(&'a Cell<bool>);
    impl Drop for Rendering<'_> {
        fn drop(&mut self) {
            self.0.set(false);
        }
    }
    let _rendering = Rendering(&state.rendering);
    let Some(controls) = state.controls.get() else {
        return Ok(());
    };
    let view = {
        let model = state.model.borrow();
        View {
            page: model.page, loading: model.busy(), ready: model.profile.is_some(),
            closing: model.closing, status: model.status.clone(),
            info: model.profile.as_ref().map(|profile| format!(
                "設定名稱：{}\r\nProfile ID：{}\r\n\r\n共享資源根目錄：{}\r\n\r\n伺服器端點：{}\r\nTLS 驗證名稱：{}\r\n\r\n載入設定不代表已連線或驗證資源完整性。確認後仍需在登入頁建立並驗證 TLS 連線。",
                profile.display_name(), profile.profile_id(), profile.shared_resource_root().display(),
                profile.endpoint(), profile.tls_server_name()))
                .unwrap_or_else(|| "尚未載入可確認的設定。\r\n請提供明確的現代 profile；不使用 TW 設定、不猜測資料庫或資源路徑，也不跳過 TLS 驗證。".into()),
        }
    };
    let previous = state.rendered.borrow().clone();
    if previous.as_ref() == Some(&view) {
        return Ok(());
    }
    let page_changed = previous.as_ref().is_none_or(|old| old.page != view.page);
    if page_changed {
        set_text(
            controls[TITLE],
            if view.page == Page::Modes {
                "選擇同一 EXE 的啟動模式"
            } else {
                "LiTo 現代模式 — 確認明確的設定"
            },
        )?;
        set_text(
            controls[INTRO],
            if view.page == Page::Modes {
                "兩種模式共用此啟動 EXE，但各自使用自己的流程。此入口不修改共享遊戲資源。"
            } else {
                "此階段提供真實登入、角色、頻道與來源保存資料檢視；不是已完成的戰鬥遊戲。"
            },
        )?;
    }
    if previous.as_ref().is_none_or(|old| {
        old.status != view.status || old.page != view.page || old.loading != view.loading
    }) {
        set_text(controls[STATUS], &view.status)?;
        set_text(
            hwnd,
            &format!(
                "LiTo / TW — {}",
                if view.closing {
                    "正在退出"
                } else if view.loading {
                    "設定載入中"
                } else if view.ready {
                    "請確認設定"
                } else if view.page == Page::Modes {
                    "選擇模式"
                } else {
                    "選擇設定檔"
                }
            ),
        )?;
    }
    if previous.as_ref().is_none_or(|old| old.info != view.info) {
        set_text(controls[INFO], &view.info)?;
    }
    for (index, control) in controls.into_iter().enumerate() {
        let _ = ShowWindow(
            control,
            if visible(view.page, index) {
                SW_SHOW
            } else {
                SW_HIDE
            },
        );
        if matches!(
            index,
            TW | MODERN | PATH | BROWSE | LOAD | CONFIRM | BACK | EXIT | SETTINGS
        ) {
            let _ = EnableWindow(
                control,
                actionable(view.page, index, view.loading, view.ready, view.closing),
            );
        }
    }
    *state.rendered.borrow_mut() = Some(view.clone());
    if (page_changed
        || previous
            .as_ref()
            .is_some_and(|old| old.loading && !view.loading))
        && !view.closing
    {
        let _ = SetFocus(Some(
            controls[if view.page == Page::Modes { TW } else { PATH }],
        ));
    }
    Ok(())
}

unsafe fn read_path(state: &WindowState) -> Result<PathBuf> {
    let controls = state
        .controls
        .get()
        .ok_or_else(|| anyhow::anyhow!("profile controls unavailable"))?;
    let length = GetWindowTextLengthW(controls[PATH]);
    ensure!(
        length > 0 && length as usize <= MAX_PATH_UTF16,
        "請輸入設定檔的絕對路徑"
    );
    let mut text = vec![0; length as usize + 1];
    let copied = GetWindowTextW(controls[PATH], &mut text);
    ensure!(copied == length, "無法完整讀取設定檔路徑");
    let path = PathBuf::from(OsString::from_wide(&text[..copied as usize]));
    ensure!(
        path.is_absolute(),
        "設定檔必須使用完整絕對路徑；不以目前資料夾猜測"
    );
    Ok(path)
}

unsafe fn browse_profile(hwnd: HWND, state: &WindowState) -> Result<()> {
    let generation = state.model.borrow_mut().begin_pick()?;
    // Release all state borrows before the COM modal loop: it can dispatch
    // timers, close, or navigation messages to our owner while Show runs.
    if let Err(error) = render(hwnd, state) {
        state.model.borrow_mut().finish_pick(generation);
        return Err(error);
    }
    let selected = profile_picker::choose(hwnd);
    if !state.model.borrow_mut().finish_pick(generation) {
        // A returned path must never revive a closed/returned/edited page.
        return Ok(());
    }
    match selected {
        Ok(Some(path)) => {
            let controls = state
                .controls
                .get()
                .ok_or_else(|| anyhow::anyhow!("profile controls unavailable"))?;
            let path: Vec<_> = path.as_os_str().encode_wide().chain(Some(0)).collect();
            // SetWindowText sends EN_CHANGE synchronously. The shared LOAD
            // action invalidates exactly once after the full path is installed.
            let previous = state.rendering.replace(true);
            let updated = SetWindowTextW(controls[PATH], PCWSTR(path.as_ptr()));
            state.rendering.set(previous);
            updated?;
            dispatch(hwnd, state, LOAD);
        }
        Ok(None) => {} // Cancel preserves path, valid profile, ticket, and status.
        Err(_) => {
            state.model.borrow_mut().status =
                "無法開啟選檔視窗或讀取選擇結果。原設定未變更；可貼上完整路徑後載入。".into();
        }
    }
    Ok(())
}

unsafe fn dispatch(hwnd: HWND, state: &WindowState, control: usize) {
    if state.rendering.get() {
        return;
    }
    let enabled = {
        let model = state.model.borrow();
        actionable(
            model.page,
            control,
            model.busy(),
            model.profile.is_some(),
            model.closing,
        )
    };
    if !enabled {
        return;
    }
    let result: Result<()> = (|| {
        match control {
            SETTINGS => {
                state.settings.borrow_mut().show(hwnd)?;
                return Ok(());
            }
            EXIT => {
                request_close(hwnd, state);
                return Ok(());
            }
            BACK => {
                escape(hwnd, state);
                return Ok(());
            }
            TW => {
                let mut model = state.model.borrow_mut();
                model.choice = Some(ModeSelection::Tw);
                model.close();
            }
            MODERN => {
                state.settings.borrow_mut().start()?;
                let mut model = state.model.borrow_mut();
                model.invalidate_profile()?;
                model.page = Page::Profile;
                model.status = "請按「選擇檔案」，或輸入設定檔路徑再按「載入設定」。".into();
            }
            BROWSE => browse_profile(hwnd, state)?,
            LOAD => {
                let path = match read_path(state) {
                    Ok(path) => path,
                    Err(error) => {
                        state.model.borrow_mut().status = error.to_string();
                        return render(hwnd, state);
                    }
                };
                let mut model = state.model.borrow_mut();
                model.invalidate_profile()?;
                let ticket = LoadTicket {
                    generation: model.generation,
                    path: path.clone(),
                };
                let worker = std::thread::Builder::new()
                    .name("lito-profile-load".into())
                    .spawn(move || ModernProfile::load(&path))?;
                model.active = Some(ticket.clone());
                model.job = Some(LoadJob { ticket, worker });
                model.status = "正在背景讀取設定檔；可移動視窗、返回模式或退出。".into();
            }
            CONFIRM => {
                let path = read_path(state)?;
                let mut model = state.model.borrow_mut();
                ensure!(
                    model
                        .active
                        .as_ref()
                        .is_some_and(|ticket| ticket.path == path),
                    "profile confirmation changed"
                );
                let profile = model
                    .profile
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("profile confirmation missing"))?;
                model.choice = Some(ModeSelection::Modern(profile));
                model.close();
            }
            _ => return Ok(()),
        }
        render(hwnd, state)?;
        finish_close(hwnd, state);
        Ok(())
    })();
    if result.is_err() {
        fail(hwnd, state, "mode selection operation failed");
    }
}

unsafe fn escape(hwnd: HWND, state: &WindowState) {
    let profile_page = state.model.borrow().page == Page::Profile;
    if !profile_page {
        request_close(hwnd, state);
        return;
    }
    let result = {
        let mut model = state.model.borrow_mut();
        if model.closing {
            return;
        }
        model.page = Page::Modes;
        model.status = if model.job.is_some() {
            "已取消設定選擇；等待該次讀取結束後可選擇其他模式。"
        } else {
            "已返回模式選擇；未建立任何伺服器連線。"
        }
        .into();
        model.invalidate_profile()
    };
    if result.is_err() || render(hwnd, state).is_err() {
        fail(hwnd, state, "cannot return to mode selection");
    }
}

unsafe fn poll_worker(hwnd: HWND, state: &WindowState) {
    state.settings.borrow_mut().poll();
    {
        state.model.borrow_mut().reap_completed();
    }
    let can_render = state.model.borrow().failure.is_none();
    if can_render && render(hwnd, state).is_err() {
        fail(hwnd, state, "cannot update mode selection text");
    }
    finish_close(hwnd, state);
}

unsafe fn request_close(hwnd: HWND, state: &WindowState) {
    {
        state.model.borrow_mut().close();
    }
    if render(hwnd, state).is_err() {
        fail(hwnd, state, "cannot display mode shutdown status");
    }
    finish_close(hwnd, state);
}

unsafe fn fail(hwnd: HWND, state: &WindowState, message: &'static str) {
    {
        let mut model = state.model.borrow_mut();
        model.failure = Some(message);
        model.choice = None;
        model.close();
    }
    // Do not recursively call failing SetWindowText. No handoff can occur after
    // a partial render failure; the timer still retires any actual worker.
    if let Some(controls) = state.controls.get() {
        for control in controls {
            let _ = EnableWindow(control, false);
        }
    }
    finish_close(hwnd, state);
}

unsafe fn finish_close(hwnd: HWND, state: &WindowState) {
    if state.model.borrow().closing {
        state.settings.borrow_mut().close();
    }
    let done = {
        let model = state.model.borrow();
        model.closing && !model.busy() && state.settings.borrow().is_quiescent()
    };
    if done && IsWindow(Some(hwnd)).as_bool() {
        let _ = DestroyWindow(hwnd);
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
        WM_TIMER if wparam.0 == TIMER => {
            poll_worker(hwnd, state);
            LRESULT(0)
        }
        WM_COMMAND => {
            if !state.rendering.get() {
                let index = (wparam.0 & 0xffff).checked_sub(FIRST_ID);
                let notification = (wparam.0 >> 16) as u32;
                if let (Some(index), Some(controls)) = (index, state.controls.get()) {
                    if index < CONTROL_COUNT && controls[index].0 as isize == lparam.0 {
                        if notification == BN_CLICKED {
                            dispatch(hwnd, state, index);
                        } else if index == PATH && notification == EN_CHANGE {
                            let result = {
                                let mut model = state.model.borrow_mut();
                                model.status =
                                    "路徑已變更；請重新載入設定，前一設定已取消。".into();
                                model.invalidate_profile()
                            };
                            if result.is_err() || render(hwnd, state).is_err() {
                                fail(hwnd, state, "cannot invalidate changed profile");
                            }
                        }
                    }
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            request_close(hwnd, state);
            LRESULT(0)
        }
        WM_SIZE => {
            if resize(hwnd, state).is_err() {
                fail(hwnd, state, "cannot resize mode selection");
            }
            LRESULT(0)
        }
        WM_GETMINMAXINFO => {
            let dpi = GetDpiForWindow(hwnd).max(96);
            let mut rect = RECT {
                left: 0,
                top: 0,
                right: scale(MIN_WIDTH, dpi),
                bottom: scale(MIN_HEIGHT, dpi),
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
                fail(hwnd, state, "cannot update mode selection DPI");
            }
            LRESULT(0)
        }
        WM_DESTROY => {
            let _ = KillTimer(Some(hwnd), TIMER);
            PostQuitMessage(0);
            LRESULT(0)
        }
        WM_NCDESTROY => {
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            DefWindowProcW(hwnd, message, wparam, lparam)
        }
        _ => DefWindowProcW(hwnd, message, wparam, lparam),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available(page: Page, loading: bool, ready: bool, closing: bool) -> [bool; CONTROL_COUNT] {
        std::array::from_fn(|control| tabbable(page, control, loading, ready, closing))
    }

    fn loaded_model() -> Model {
        Model {
            page: Page::Profile,
            generation: 7,
            active: Some(LoadTicket { generation: 7, path: PathBuf::from("D:/selected/profile.json") }),
            profile: Some(ModernProfile::parse(br#"{"schema":1,"profile_id":"picker-test","display_name":"picker-test","shared_resource_root":"D:/shared/client","endpoint":"127.0.0.1:25400","tls_server_name":"localhost","trust":{"kind":"web_pki"},"connect_timeout_ms":3000,"exchange_timeout_ms":5000}"#).unwrap()),
            job: None,
            picking: None,
            closing: false,
            choice: None,
            failure: None,
            status: "loaded".into(),
        }
    }

    #[test]
    fn cancelling_picker_preserves_loaded_profile_ticket_status_and_confirmation() {
        let mut model = loaded_model();
        let active = model.active.clone();
        let generation = model.begin_pick().unwrap();
        assert!(model.busy());
        assert!(!actionable(model.page, CONFIRM, model.busy(), true, false));
        assert!(model.begin_pick().is_err(), "no nested picker");
        assert!(model.finish_pick(generation));
        assert!(!model.busy());
        assert_eq!(model.active, active);
        assert_eq!(model.profile.as_ref().unwrap().profile_id(), "picker-test");
        assert_eq!(model.generation, 7);
        assert_eq!(model.status, "loaded");
        assert!(model.choice.is_none());
        assert!(actionable(model.page, CONFIRM, model.busy(), true, false));
        assert!(!model.finish_pick(generation), "completion consumed once");
    }

    #[test]
    fn stale_picker_cannot_publish_after_navigation_edit_or_shutdown() {
        for transition in 0..3 {
            let mut model = loaded_model();
            let generation = model.begin_pick().unwrap();
            match transition {
                0 => {
                    model.page = Page::Modes;
                    model.invalidate_profile().unwrap();
                }
                1 => model.invalidate_profile().unwrap(),
                _ => model.close(),
            }
            assert!(
                model.busy(),
                "owner must survive until the modal call returns"
            );
            assert!(!model.finish_pick(generation));
            assert!(!model.busy());
            assert!(model.profile.is_none());
            assert!(model.active.is_none());
            assert!(model.choice.is_none());
        }
    }

    #[test]
    fn unrelated_completion_cannot_clear_picker_and_wrong_page_cannot_open_one() {
        let mut model = loaded_model();
        let generation = model.begin_pick().unwrap();
        assert!(!model.finish_pick(generation + 1));
        assert!(model.busy());
        assert!(model.finish_pick(generation));
        model.page = Page::Modes;
        assert!(model.begin_pick().is_err());
        model.page = Page::Profile;
        model.close();
        assert!(model.begin_pick().is_err());
    }

    #[test]
    fn readonly_info_tabs_to_confirm_and_backwards_to_load_without_an_action() {
        let controls = available(Page::Profile, false, true, false);
        assert_eq!(
            next_tab_stop(Page::Profile, Some(PATH), false, &controls),
            Some(BROWSE)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(BROWSE), false, &controls),
            Some(LOAD)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(LOAD), true, &controls),
            Some(BROWSE)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(LOAD), false, &controls),
            Some(INFO)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), false, &controls),
            Some(CONFIRM)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), true, &controls),
            Some(LOAD)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(CONFIRM), true, &controls),
            Some(INFO)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(CONFIRM), false, &controls),
            Some(BACK)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(EXIT), false, &controls),
            Some(PATH)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(PATH), true, &controls),
            Some(EXIT)
        );
        assert!(!actionable(Page::Profile, INFO, false, true, false));
    }

    #[test]
    fn incomplete_loading_disabled_and_closing_controls_cannot_enter_focus_order() {
        let no_profile = available(Page::Profile, false, false, false);
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), false, &no_profile),
            Some(BACK)
        );
        let loading = available(Page::Profile, true, false, false);
        assert_eq!(
            next_tab_stop(Page::Profile, Some(LOAD), false, &loading),
            Some(INFO)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), true, &loading),
            Some(EXIT)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(BACK), false, &loading),
            Some(EXIT)
        );
        let mut native_disabled = available(Page::Profile, false, true, false);
        native_disabled[CONFIRM] = false;
        native_disabled[BACK] = false;
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), false, &native_disabled),
            Some(EXIT)
        );
        let closed = available(Page::Profile, false, true, true);
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), false, &closed),
            None
        );
        assert_eq!(
            next_tab_stop(Page::Profile, Some(INFO), true, &[false; CONTROL_COUNT]),
            None
        );
    }

    #[test]
    fn each_mode_has_an_explicit_cycle_and_profile_never_tabs_to_tw() {
        let modes = available(Page::Modes, false, false, false);
        assert_eq!(
            next_tab_stop(Page::Modes, Some(TW), false, &modes),
            Some(MODERN)
        );
        assert_eq!(
            next_tab_stop(Page::Modes, Some(MODERN), true, &modes),
            Some(TW)
        );
        assert_eq!(
            next_tab_stop(Page::Modes, Some(EXIT), false, &modes),
            Some(TW)
        );
        let profile = available(Page::Profile, false, true, false);
        assert_eq!(
            next_tab_stop(Page::Profile, Some(TW), false, &profile),
            Some(PATH)
        );
        assert_eq!(
            next_tab_stop(Page::Profile, None, true, &profile),
            Some(EXIT)
        );
        // Even an inconsistent native eligibility mask cannot select a control
        // from another page, nor static text that never had a tab stop.
        for control in 0..CONTROL_COUNT {
            for backward in [false, true] {
                let next = next_tab_stop(
                    Page::Profile,
                    Some(control),
                    backward,
                    &[true; CONTROL_COUNT],
                )
                .unwrap();
                assert!(matches!(
                    next,
                    PATH | BROWSE | LOAD | INFO | CONFIRM | BACK | EXIT
                ));
            }
        }
    }

    #[test]
    fn native_control_layout_keeps_each_page_inside_client_at_all_dpis() {
        for dpi in [48, 96, 120, 144, 192, 288, 768] {
            for (width, height) in [(MIN_WIDTH, MIN_HEIGHT), (900, 760), (1800, 1100)] {
                let (width, height) = (scale(width, dpi), scale(height, dpi));
                let boxes = layout(width, height, dpi);
                for page in [Page::Modes, Page::Profile] {
                    let visible: Vec<_> = boxes
                        .iter()
                        .enumerate()
                        .filter(|(index, _)| visible(page, *index))
                        .collect();
                    for (i, (index, rect)) in visible.iter().enumerate() {
                        assert!(
                            rect.x >= 0 && rect.y >= 0 && rect.width > 0 && rect.height > 0,
                            "{index}"
                        );
                        assert!(
                            rect.x + rect.width <= width && rect.y + rect.height <= height,
                            "{index}"
                        );
                        for (_, other) in visible.iter().skip(i + 1) {
                            assert!(
                                rect.x + rect.width <= other.x
                                    || other.x + other.width <= rect.x
                                    || rect.y + rect.height <= other.y
                                    || other.y + other.height <= rect.y,
                                "overlap {index}"
                            );
                        }
                    }
                }
                assert!(boxes[INFO].height >= scale(220, dpi));
            }
        }
    }

    #[test]
    fn loading_and_close_gate_real_actions_without_disabling_back_or_exit() {
        assert!(actionable(Page::Modes, TW, false, false, false));
        assert!(!actionable(Page::Modes, CONFIRM, false, true, false));
        assert!(!actionable(Page::Profile, CONFIRM, false, false, false));
        assert!(actionable(Page::Profile, CONFIRM, false, true, false));
        for control in [TW, MODERN, PATH, BROWSE, LOAD, CONFIRM] {
            for page in [Page::Modes, Page::Profile] {
                assert!(!actionable(page, control, true, true, false));
            }
        }
        assert!(actionable(Page::Profile, BACK, true, false, false));
        assert!(actionable(Page::Profile, EXIT, true, false, false));
        for control in 0..CONTROL_COUNT {
            assert!(!actionable(Page::Profile, control, false, true, true));
        }
    }

    #[test]
    fn completed_load_requires_exact_path_generation_and_live_profile_page() {
        let ticket = LoadTicket {
            generation: 1,
            path: PathBuf::from("D:/selected/profile.json"),
        };
        assert!(accepts_load(Some(&ticket), &ticket, Page::Profile, false));
        assert!(!accepts_load(None, &ticket, Page::Profile, false));
        assert!(!accepts_load(Some(&ticket), &ticket, Page::Modes, false));
        assert!(!accepts_load(Some(&ticket), &ticket, Page::Profile, true));
        let mut other = ticket.clone();
        other.generation += 1;
        assert!(!accepts_load(Some(&other), &ticket, Page::Profile, false));
        other = ticket.clone();
        other.path = PathBuf::from("D:/different/profile.json");
        assert!(!accepts_load(Some(&other), &ticket, Page::Profile, false));
    }

    #[test]
    fn cancellation_does_not_detach_or_publish_a_still_running_profile_worker() {
        use std::sync::mpsc;
        use std::time::{Duration, Instant};
        for closing in [false, true] {
            let profile = ModernProfile::parse(br#"{"schema":1,"profile_id":"worker-test","display_name":"worker-test","shared_resource_root":"D:/shared/client","endpoint":"127.0.0.1:25400","tls_server_name":"localhost","trust":{"kind":"web_pki"},"connect_timeout_ms":3000,"exchange_timeout_ms":5000}"#).unwrap();
            let ticket = LoadTicket {
                generation: 1,
                path: PathBuf::from("D:/selected/profile.json"),
            };
            let (started_tx, started_rx) = mpsc::sync_channel(1);
            let (release_tx, release_rx) = mpsc::sync_channel(1);
            let worker = std::thread::spawn(move || {
                started_tx.send(()).unwrap();
                release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
                Ok(profile)
            });
            let mut model = Model {
                page: Page::Profile,
                generation: 1,
                active: Some(ticket.clone()),
                profile: None,
                job: Some(LoadJob { ticket, worker }),
                picking: None,
                closing: false,
                choice: None,
                failure: None,
                status: String::new(),
            };
            started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
            if closing {
                model.close();
            } else {
                model.page = Page::Modes;
                model.invalidate_profile().unwrap();
            }
            assert!(!model.reap_completed());
            assert!(
                model.job.is_some(),
                "cancelling the ticket is not completing the worker"
            );
            assert!(model.profile.is_none());
            release_tx.send(()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(2);
            while !model.reap_completed() {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            assert!(model.job.is_none(), "the actual owned thread was joined");
            assert!(
                model.profile.is_none(),
                "a cancelled successful result must not publish"
            );
            assert!(!model.reap_completed(), "each worker is consumed once");
        }
    }

    #[test]
    fn exceptional_state_drop_joins_actual_worker_instead_of_detaching_it() {
        use std::sync::mpsc;
        use std::time::Duration;
        let (started_tx, started_rx) = mpsc::sync_channel(1);
        let (release_tx, release_rx) = mpsc::sync_channel(1);
        let worker = std::thread::spawn(move || {
            started_tx.send(()).unwrap();
            release_rx.recv_timeout(Duration::from_secs(3)).unwrap();
            Err(ProfileError::ReadFailed)
        });
        started_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        let (dropping_tx, dropping_rx) = mpsc::sync_channel(1);
        let (done_tx, done_rx) = mpsc::sync_channel(1);
        let dropping = std::thread::spawn(move || {
            let state = WindowState {
                settings: RefCell::new(
                    crate::modern_lito::settings_window::SettingsController::default(),
                ),
                model: RefCell::new(Model {
                    page: Page::Profile,
                    generation: 1,
                    active: None,
                    profile: None,
                    job: Some(LoadJob {
                        ticket: LoadTicket {
                            generation: 1,
                            path: PathBuf::from("D:/selected/profile.json"),
                        },
                        worker,
                    }),
                    picking: None,
                    closing: true,
                    choice: None,
                    failure: Some("exceptional teardown"),
                    status: String::new(),
                }),
                controls: Cell::new(None),
                rendering: Cell::new(false),
                rendered: RefCell::new(None),
                font: Cell::new(HFONT::default()),
            };
            dropping_tx.send(()).unwrap();
            drop(state); // No native HWND/font was created by this test.
            done_tx.send(()).unwrap();
        });
        dropping_rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert!(matches!(done_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
        release_tx.send(()).unwrap();
        done_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        dropping.join().unwrap();
    }
}
