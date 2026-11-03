//! A local setup form. No network owner starts before the user opens LOGIN.
use super::native_startup::{Settings, MAX_ROOT_UTF16};
use anyhow::{ensure, Context, Result};
use std::{cell::RefCell, path::PathBuf};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Controls::EM_SETLIMITTEXT,
            HiDpi::*,
            Input::KeyboardAndMouse::{GetFocus, SetFocus},
            WindowsAndMessaging::*,
        },
    },
};

const ENDPOINT: u16 = 10;
const RESOURCE: u16 = 11;
const FEEDBACK: u16 = 12;
const REMEMBER: u16 = 13;
const OPEN: u16 = 1;
const CANCEL: u16 = 2;
struct Ui {
    settings_path: PathBuf,
    initial: Option<Settings>,
    initial_error: Option<String>,
    controls: Vec<(u16, HWND)>,
    result: Option<Settings>,
    font: HFONT,
    dpi: u32,
}
struct State(RefCell<Ui>);
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}
unsafe fn send(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    SendMessageW(hwnd, msg, Some(w), Some(l))
}
impl Ui {
    fn control(&self, id: u16) -> HWND {
        self.controls
            .iter()
            .find(|(key, _)| *key == id)
            .map(|(_, h)| *h)
            .unwrap_or_default()
    }
    unsafe fn feedback(&self, value: &str) {
        let value = wide(value);
        let _ = SetWindowTextW(self.control(FEEDBACK), PCWSTR(value.as_ptr()));
    }
    unsafe fn text(&self, id: u16, limit: usize) -> Result<String> {
        let h = self.control(id);
        let len = GetWindowTextLengthW(h);
        ensure!(len >= 0 && len as usize <= limit, "輸入超出長度限制");
        let mut value = vec![0u16; len as usize + 1];
        let n = GetWindowTextW(h, &mut value);
        ensure!(n == len, "輸入已改變，請再試一次");
        Ok(String::from_utf16(&value[..n as usize])?.trim().to_owned())
    }
    unsafe fn create(&mut self, parent: HWND) -> Result<()> {
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let endpoint = self
            .initial
            .as_ref()
            .map(|s| s.endpoint().to_string())
            .unwrap_or_default();
        let root = self
            .initial
            .as_ref()
            .map(|s| s.resource_root().to_string_lossy().into_owned())
            .unwrap_or_default();
        for (id, class, label, style) in [
            (301, w!("STATIC"), "登入伺服器 IP:連接埠".to_owned(), 0),
            (ENDPOINT, w!("EDIT"), endpoint, ES_AUTOHSCROLL as u32),
            (
                302,
                w!("STATIC"),
                "遊戲資源根目錄（絕對路徑）".to_owned(),
                0,
            ),
            (RESOURCE, w!("EDIT"), root, ES_AUTOHSCROLL as u32),
            (
                REMEMBER,
                w!("BUTTON"),
                "記住伺服器與資源路徑".to_owned(),
                BS_AUTOCHECKBOX as u32,
            ),
            (
                FEEDBACK,
                w!("STATIC"),
                self.initial_error
                    .clone()
                    .unwrap_or_else(|| "設定完成後開啟帳密登入視窗。".into()),
                0,
            ),
            (
                OPEN,
                w!("BUTTON"),
                "開啟登入".to_owned(),
                BS_DEFPUSHBUTTON as u32,
            ),
            (CANCEL, w!("BUTTON"), "取消".to_owned(), 0),
        ] {
            let text = wide(&label);
            let input = matches!(id, ENDPOINT | RESOURCE);
            let tab = matches!(id, ENDPOINT | RESOURCE | REMEMBER | OPEN | CANCEL);
            let h = CreateWindowExW(
                if input {
                    WS_EX_CLIENTEDGE
                } else {
                    WINDOW_EX_STYLE(0)
                },
                class,
                PCWSTR(text.as_ptr()),
                WS_CHILD
                    | WS_VISIBLE
                    | WINDOW_STYLE(style)
                    | if tab { WS_TABSTOP } else { WINDOW_STYLE(0) },
                0,
                0,
                0,
                0,
                Some(parent),
                Some(HMENU(id as usize as *mut _)),
                Some(instance),
                None,
            )?;
            self.controls.push((id, h));
        }
        send(
            self.control(ENDPOINT),
            EM_SETLIMITTEXT,
            WPARAM(128),
            LPARAM(0),
        );
        send(
            self.control(RESOURCE),
            EM_SETLIMITTEXT,
            WPARAM(MAX_ROOT_UTF16),
            LPARAM(0),
        );
        self.layout(parent);
        Ok(())
    }
    unsafe fn layout(&mut self, parent: HWND) {
        let dpi = GetDpiForWindow(parent).max(96);
        let px = |n: i32| n * dpi as i32 / 96;
        let mut client = RECT::default();
        if GetClientRect(parent, &mut client).is_err() {
            return;
        }
        let width = (client.right - client.left) * 96 / dpi as i32;
        let rows = [
            (301, 16, 16, width - 32, 22),
            (ENDPOINT, 16, 42, width - 32, 26),
            (302, 16, 82, width - 32, 22),
            (RESOURCE, 16, 108, width - 32, 26),
            (REMEMBER, 16, 146, width - 32, 26),
            (FEEDBACK, 16, 184, width - 32, 48),
            (OPEN, width - 244, 244, 110, 32),
            (CANCEL, width - 124, 244, 108, 32),
        ];
        for (id, x, y, w, h) in rows {
            let _ = MoveWindow(self.control(id), px(x), px(y), px(w.max(1)), px(h), true);
        }
        if self.dpi != dpi || self.font.is_invalid() {
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
                for (_, h) in &self.controls {
                    send(*h, WM_SETFONT, WPARAM(font.0 as usize), LPARAM(1));
                }
                if !self.font.is_invalid() {
                    let _ = DeleteObject(self.font.into());
                }
                self.font = font;
                self.dpi = dpi;
            }
        }
    }
    unsafe fn accept(&mut self) -> Result<()> {
        let settings = Settings::new(
            &self.text(ENDPOINT, 128)?,
            &self.text(RESOURCE, MAX_ROOT_UTF16)?,
        )?;
        settings.validate_available_root()?;
        if send(self.control(REMEMBER), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1 {
            settings.save(&self.settings_path)?;
        }
        self.result = Some(settings);
        Ok(())
    }
}
unsafe extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if msg == WM_NCCREATE {
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize);
        return LRESULT(1);
    }
    let ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const State;
    if !ptr.is_null() {
        let state = &*ptr;
        match msg {
            WM_COMMAND if ((wparam.0 >> 16) & 0xffff) == BN_CLICKED as usize => {
                let id = (wparam.0 & 0xffff) as u16;
                if id == OPEN {
                    // Drop the borrow before DestroyWindow dispatches messages.
                    let result = state.0.borrow_mut().accept();
                    match result {
                        Ok(()) => {
                            let _ = DestroyWindow(hwnd);
                        }
                        Err(e) => state.0.borrow().feedback(&format!("{e:#}")),
                    }
                    return LRESULT(0);
                }
                if id == CANCEL {
                    let _ = DestroyWindow(hwnd);
                    return LRESULT(0);
                }
            }
            WM_SIZE => {
                if let Ok(mut ui) = state.0.try_borrow_mut() {
                    ui.layout(hwnd);
                }
                return LRESULT(0);
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
                if let Ok(mut ui) = state.0.try_borrow_mut() {
                    ui.layout(hwnd);
                }
                return LRESULT(0);
            }
            WM_GETMINMAXINFO => {
                let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                let info = &mut *(lparam.0 as *mut MINMAXINFO);
                info.ptMinTrackSize.x = 600 * dpi / 96;
                info.ptMinTrackSize.y = 330 * dpi / 96;
                return LRESULT(0);
            }
            WM_NCDESTROY => {
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            }
            _ => {}
        }
    }
    if msg == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, msg, wparam, lparam)
}

pub fn run(
    initial: Option<Settings>,
    settings_path: PathBuf,
    initial_error: Option<String>,
) -> Result<Option<Settings>> {
    unsafe {
        let previous = SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let result = (|| -> Result<Option<Settings>> {
            let instance = HINSTANCE(GetModuleHandleW(None)?.0);
            let class = w!("LiToNativeStartupWindow");
            let _ = RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(wndproc),
                hInstance: instance,
                lpszClassName: class,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                hbrBackground: GetSysColorBrush(COLOR_BTNFACE),
                ..Default::default()
            });
            let state = Box::new(State(RefCell::new(Ui {
                settings_path,
                initial,
                initial_error,
                controls: vec![],
                result: None,
                font: HFONT::default(),
                dpi: 0,
            })));
            let dpi = GetDpiForSystem().max(96) as i32;
            let hwnd = CreateWindowExW(
                WS_EX_CONTROLPARENT,
                class,
                w!("LiTo 連線設定"),
                WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                640 * dpi / 96,
                340 * dpi / 96,
                None,
                None,
                Some(instance),
                Some((&*state as *const State).cast()),
            )?;
            let result = (|| -> Result<Option<Settings>> {
                state.0.borrow_mut().create(hwnd)?;
                let _ = ShowWindow(hwnd, SW_SHOW);
                if !IsWindowVisible(hwnd).as_bool() {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }
                let _ = SetFocus(Some(state.0.borrow().control(ENDPOINT)));
                let mut msg = MSG::default();
                loop {
                    let read = GetMessageW(&mut msg, None, 0, 0).0;
                    if read == 0 {
                        break;
                    }
                    ensure!(read != -1, "連線設定視窗訊息讀取失敗");
                    if msg.message == WM_KEYDOWN && msg.wParam.0 == 0x1b {
                        let _ = DestroyWindow(hwnd);
                        continue;
                    }
                    if msg.message == WM_KEYDOWN && msg.wParam.0 == 0x0d {
                        let target = if GetFocus() == state.0.borrow().control(CANCEL) {
                            CANCEL
                        } else {
                            OPEN
                        };
                        send(hwnd, WM_COMMAND, WPARAM(target as usize), LPARAM(0));
                        continue;
                    }
                    if !IsDialogMessageW(hwnd, &msg).as_bool() {
                        let _ = TranslateMessage(&msg);
                        DispatchMessageW(&msg);
                    }
                }
                Ok(state.0.borrow_mut().result.take())
            })();
            if IsWindow(Some(hwnd)).as_bool() {
                let _ = DestroyWindow(hwnd);
            }
            let font = state.0.borrow().font;
            if !font.is_invalid() {
                let _ = DeleteObject(font.into());
            }
            result
        })();
        if !previous.is_invalid() {
            SetThreadDpiAwarenessContext(previous);
        }
        result.context("原生連線設定視窗")
    }
}
