//! Debug-build-only, same-executable resource presentation gate.
//!
//! This is a raw SPR frame inspector, not an implemented game menu. It never
//! opens a network connection, writes a client setting, or starts a native BIN.

use std::{cell::RefCell, path::PathBuf};

use anyhow::{bail, ensure, Context, Result};
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, FillRect, GetStockObject, InvalidateRect, SetBkMode, SetTextColor,
    StretchDIBits, TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACK_BRUSH, DIB_RGB_COLORS,
    HBRUSH, PAINTSTRUCT, SRCCOPY, TRANSPARENT,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::HiDpi::{
    SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows::Win32::UI::WindowsAndMessaging::*;

use super::compositor::{Canvas, Viewport};
use super::resources::{SharedArchive, SharedArchiveKind};
use super::sprite::{decode_classic_spr, DecodedSprite};

const CLASS: PCWSTR = w!("ModernLiToSharedResourcePreview");
const HELP: &str = "共享 SPR 原始影格：左右鍵切換，Esc 正常關閉。尚未接入遊戲／伺服器。";

#[derive(Debug, Eq, PartialEq)]
struct Request {
    root: PathBuf,
    kind: SharedArchiveKind,
    entry: String,
}

fn parse_request(args: &[String]) -> Result<Request> {
    ensure!(
        args.len() == 5 && args[1] == "--lito-preview",
        "開發預覽用法：launcher.exe --lito-preview RESOURCE_ROOT Sprite00 NAME.spr"
    );
    let stem = args[3].to_ascii_lowercase();
    let kind = if stem == "sprite" {
        SharedArchiveKind::Sprite
    } else if stem.len() == 8
        && stem.starts_with("sprite")
        && stem.as_bytes()[6..].iter().all(u8::is_ascii_digit)
    {
        let shard = stem[6..].parse::<u8>()?;
        ensure!(shard <= 15, "Sprite 分包必須在 00..15");
        SharedArchiveKind::SpriteShard(shard)
    } else {
        bail!("SPR 預覽僅接受 Sprite 或 Sprite00..15，不搜尋其他資源包");
    };
    ensure!(
        args[4].to_ascii_lowercase().ends_with(".spr"),
        "必須指定 SPR entry"
    );
    Ok(Request {
        root: PathBuf::from(&args[2]),
        kind,
        entry: args[4].clone(),
    })
}

struct PreviewState {
    sprite: DecodedSprite,
    frame: usize,
    canvas: Canvas,
    bgra: Vec<u8>,
    anchor: (i32, i32),
    label: String,
    help: Vec<u16>,
    error: Option<String>,
}

impl PreviewState {
    fn new(sprite: DecodedSprite, label: String) -> Result<Self> {
        ensure!(!sprite.frames.is_empty(), "SPR 沒有可顯示影格");
        let min_x = sprite
            .frames
            .iter()
            .map(|frame| i64::from(frame.x_offset))
            .min()
            .unwrap();
        let min_y = sprite
            .frames
            .iter()
            .map(|frame| i64::from(frame.y_offset))
            .min()
            .unwrap();
        let max_x = sprite
            .frames
            .iter()
            .map(|frame| i64::from(frame.x_offset) + i64::from(frame.width))
            .max()
            .unwrap();
        let max_y = sprite
            .frames
            .iter()
            .map(|frame| i64::from(frame.y_offset) + i64::from(frame.height))
            .max()
            .unwrap();
        let width = (max_x - min_x + 64).max(320);
        let height = (max_y - min_y + 64).max(240);
        ensure!(width <= 2048 && height <= 2048, "SPR 範圍超出開發預覽上限");
        let anchor_x = (width - (max_x - min_x)) / 2 - min_x;
        let anchor_y = (height - (max_y - min_y)) / 2 - min_y;
        let mut state = Self {
            sprite,
            frame: 0,
            canvas: Canvas::new(width as u32, height as u32)?,
            bgra: Vec::new(),
            anchor: (anchor_x.try_into()?, anchor_y.try_into()?),
            label,
            help: HELP.encode_utf16().collect(),
            error: None,
        };
        state.rebuild()?;
        Ok(state)
    }

    fn rebuild(&mut self) -> Result<()> {
        self.canvas.clear([18, 25, 35]);
        let frame = &self.sprite.frames[self.frame];
        let x = self
            .anchor
            .0
            .checked_add(frame.x_offset)
            .context("SPR anchor overflow")?;
        let y = self
            .anchor
            .1
            .checked_add(frame.y_offset)
            .context("SPR anchor overflow")?;
        self.canvas
            .blit_rgba(&frame.rgba, frame.width, frame.height, x, y)?;
        self.canvas.copy_bgra(&mut self.bgra)
    }

    fn title(&self) -> Vec<u16> {
        let text = if let Some(error) = &self.error {
            format!("LiTo 資源預覽錯誤：{error}")
        } else {
            let frame = &self.sprite.frames[self.frame];
            format!(
                "LiTo 資源預覽（非遊戲） | {} | 影格 {}/{} | {}×{} @ {},{}",
                self.label,
                self.frame + 1,
                self.sprite.frames.len(),
                frame.width,
                frame.height,
                frame.x_offset,
                frame.y_offset
            )
        };
        text.encode_utf16().chain(Some(0)).collect()
    }
}

pub fn run_from_args(args: &[String]) -> Result<()> {
    let request = parse_request(args)?;
    let mut archive = SharedArchive::open(&request.root, request.kind)?;
    let bytes = archive.read_entry(&request.entry)?;
    let sprite = decode_classic_spr(&bytes)?;
    // All file access and decode finish before a window callback can run.
    drop(archive);
    let mut state = Box::new(RefCell::new(PreviewState::new(
        sprite,
        format!("{:?}/{}", request.kind, request.entry),
    )?));
    unsafe {
        // The existing process manifest may already establish this context.
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: instance,
            lpszClassName: CLASS,
            hCursor: LoadCursorW(None, IDC_ARROW)?,
            style: CS_HREDRAW | CS_VREDRAW,
            ..Default::default()
        };
        ensure!(
            RegisterClassW(&class) != 0,
            "register LiTo preview window: {}",
            std::io::Error::last_os_error()
        );
        let _class_guard = ClassGuard(instance);
        let title = state.borrow().title();
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            960,
            720,
            None,
            None,
            Some(instance),
            Some((&mut *state as *mut RefCell<PreviewState>).cast()),
        )?;
        let _window_guard = WindowGuard(hwnd);
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut message = MSG::default();
        loop {
            let result = GetMessageW(&mut message, None, 0, 0).0;
            if result == -1 {
                return Err(std::io::Error::last_os_error().into());
            }
            if result == 0 {
                break;
            }
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
    if let Some(error) = state.get_mut().error.take() {
        bail!(error);
    }
    Ok(())
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

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        // SAFETY: Windows supplies CREATESTRUCTW for this message. The Box is
        // owned by run_from_args until after the native window is destroyed.
        let create = &*(lparam.0 as *const CREATESTRUCTW);
        // windows-rs aliases this to SetWindowLongW(i32) on i686 and uses
        // LONG_PTR(isize) on x64; the inferred final cast keeps pointer width.
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, create.lpCreateParams as isize as _);
    }
    let state_ptr = GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *const RefCell<PreviewState>;
    if state_ptr.is_null() {
        return DefWindowProcW(hwnd, message, wparam, lparam);
    }
    match message {
        WM_PAINT => {
            // Even a synchronous native paint can dispatch nested messages.
            // Interior borrow checks prevent mutable aliasing on reentry.
            let Ok(state) = (&*state_ptr).try_borrow() else {
                return DefWindowProcW(hwnd, message, wparam, lparam);
            };
            paint(hwnd, &state);
            LRESULT(0)
        }
        WM_ERASEBKGND => LRESULT(1),
        WM_SIZE => {
            let _ = InvalidateRect(Some(hwnd), None, false);
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
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 == 0x1b => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 == 0x25 || wparam.0 == 0x27 => {
            // Do not hold a Rust mutable reference across SetWindowTextW;
            // synchronous native calls can reenter the window procedure.
            let title = {
                let Ok(mut state) = (&*state_ptr).try_borrow_mut() else {
                    return LRESULT(0);
                };
                let count = state.sprite.frames.len();
                state.frame = (state.frame + if wparam.0 == 0x25 { count - 1 } else { 1 }) % count;
                if let Err(error) = state.rebuild() {
                    state.error = Some(error.to_string());
                }
                state.title()
            };
            let _ = SetWindowTextW(hwnd, PCWSTR(title.as_ptr()));
            let _ = InvalidateRect(Some(hwnd), None, false);
            LRESULT(0)
        }
        WM_CLOSE => {
            let _ = DestroyWindow(hwnd);
            LRESULT(0)
        }
        WM_DESTROY => {
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

unsafe fn paint(hwnd: HWND, state: &PreviewState) {
    let mut paint = PAINTSTRUCT::default();
    let dc = BeginPaint(hwnd, &mut paint);
    let mut rect = RECT::default();
    if GetClientRect(hwnd, &mut rect).is_ok() {
        FillRect(dc, &rect, HBRUSH(GetStockObject(BLACK_BRUSH).0));
        if let Some(view) = Viewport::fit(
            rect.right,
            (rect.bottom - 36).max(0),
            state.canvas.width(),
            state.canvas.height(),
        ) {
            let bitmap = BITMAPINFO {
                bmiHeader: BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: state.canvas.width() as i32,
                    biHeight: -(state.canvas.height() as i32),
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                },
                ..Default::default()
            };
            StretchDIBits(
                dc,
                view.x,
                view.y,
                view.width,
                view.height,
                0,
                0,
                state.canvas.width() as i32,
                state.canvas.height() as i32,
                Some(state.bgra.as_ptr().cast()),
                &bitmap,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
        }
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, windows::Win32::Foundation::COLORREF(0x00dddddd));
        let _ = TextOutW(dc, 12, (rect.bottom - 28).max(0), &state.help);
    }
    let _ = EndPaint(hwnd, &paint);
}

#[cfg(test)]
mod tests {
    use super::*;
    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|s| s.to_string()).collect()
    }
    #[test]
    fn preview_requires_explicit_shared_root_and_archive() {
        let request = parse_request(&args(&[
            "launcher.exe",
            "--lito-preview",
            "D:/client",
            "Sprite03",
            "0-0.spr",
        ]))
        .unwrap();
        assert_eq!(request.kind, SharedArchiveKind::SpriteShard(3));
        assert_eq!(request.root, PathBuf::from("D:/client"));
        for archive in ["LTsprite", "Sprite16", "../Sprite", "Tile", "Sprite.pak"] {
            assert!(parse_request(&args(&[
                "launcher.exe",
                "--lito-preview",
                "D:/client",
                archive,
                "0-0.spr"
            ]))
            .is_err());
        }
        assert!(parse_request(&args(&["launcher.exe", "--lito-preview"])).is_err());
    }
    #[test]
    fn preview_has_explicit_not_game_label_and_normal_exit_help() {
        assert!(HELP.contains("尚未接入遊戲／伺服器"));
        assert!(HELP.contains("Esc 正常關閉"));
    }
}
