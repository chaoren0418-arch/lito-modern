//! Same-EXE development scene window with asynchronous bounded loading.
//! This is a presentation integration gate, not a connected or playable game.
//! It deliberately owns no native BIN, game socket, gameplay inputs or SQL.

use super::{
    audio::parse_pcm_wav,
    audio_output::{PcmVoice, VoiceStatus},
    compositor::Viewport,
    scene_model::SceneModel,
};
use anyhow::{ensure, Context, Result};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, TryRecvError},
        Arc, Mutex,
    },
    thread::JoinHandle,
    time::Instant,
};
use windows::{
    core::{w, PCWSTR},
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, WPARAM},
        Graphics::Gdi::{
            BeginPaint, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject,
            EndPaint, FillRect, GetStockObject, InvalidateRect, SelectObject, SetBkMode,
            SetTextColor, StretchDIBits, TextOutW, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
            BLACK_BRUSH, DIB_RGB_COLORS, HBRUSH, PAINTSTRUCT, SRCCOPY, TRANSPARENT,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            HiDpi::{SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            WindowsAndMessaging::*,
        },
    },
};

const CLASS: PCWSTR = w!("ModernLiToSharedScenePreview");
const TIMER: usize = 1;
const HELP: &str =
    "離線資源驗證：點擊放置角色；Space 動作／音效；Esc 返回。尚無伺服器、移動或戰鬥。";

fn parse_request(args: &[String]) -> Result<(PathBuf, u8)> {
    ensure!(
        args.len() == 4 && args[1] == "--lito-scene",
        "用法：launcher.exe --lito-scene RESOURCE_ROOT HEADING(0..7)"
    );
    let heading: u8 = args[3].parse().context("heading must be 0..7")?;
    ensure!(heading < 8, "heading must be 0..7");
    Ok((PathBuf::from(&args[2]), heading))
}

struct LoadingJob {
    receiver: Receiver<Result<SceneModel>>,
    progress: Arc<Mutex<String>>,
    cancel: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,
}
impl LoadingJob {
    fn start(root: PathBuf, heading: u8) -> Result<Self> {
        let (sender, receiver) = mpsc::sync_channel(1);
        let progress = Arc::new(Mutex::new("初始化共享資源".to_string()));
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_progress = progress.clone();
        let worker_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("lito-scene-load".into())
            .spawn(move || {
                let result = SceneModel::load(&root, heading, &worker_cancel, |stage| {
                    if let Ok(mut progress) = worker_progress.lock() {
                        *progress = stage.to_string();
                    }
                });
                // Exactly one result in a one-slot channel: closing the window
                // cannot deadlock a producer behind stale progress messages.
                let _ = sender.send(result);
            })?;
        Ok(Self {
            receiver,
            progress,
            cancel,
            worker: Some(worker),
        })
    }
}
impl Drop for LoadingJob {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        // No detached helpers: finish the current bounded local read/decode,
        // then cancellation checkpoints release all owned archive handles.
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct Ui {
    job: LoadingJob,
    model: Option<SceneModel>,
    model_start: Option<Instant>,
    loading_status: String,
    error: Option<String>,
}
struct WindowState {
    ui: RefCell<Ui>,
    voice: RefCell<PcmVoice>,
    submitted: Cell<u64>,
    completed: Cell<u64>,
}
struct Tick {
    changed: bool,
    sounds: Vec<Vec<u8>>,
}

impl Ui {
    fn tick(&mut self) -> Result<Tick> {
        if self.error.is_some() {
            return Ok(Tick {
                changed: false,
                sounds: vec![],
            });
        }
        let mut changed = false;
        if self.model.is_none() {
            if let Ok(progress) = self.job.progress.try_lock() {
                if self.loading_status != *progress {
                    self.loading_status = progress.clone();
                    changed = true;
                }
            }
            match self.job.receiver.try_recv() {
                Ok(result) => {
                    self.model = Some(result?);
                    self.model_start = Some(Instant::now());
                    changed = true;
                }
                Err(TryRecvError::Empty) => {
                    return Ok(Tick {
                        changed,
                        sounds: vec![],
                    });
                }
                Err(TryRecvError::Disconnected) => {
                    anyhow::bail!("scene loader ended without a result")
                }
            }
        }
        let model = self.model.as_mut().context("missing prepared model")?;
        let update = model.advance(
            self.model_start
                .context("missing presentation start")?
                .elapsed(),
        )?;
        // This gate has one voice and an inspected action with one sound cue.
        // Other actions need a mixer policy, not silently replaced voices.
        ensure!(
            update.sounds.len() <= 1,
            "scene cue batch requires a multi-voice mixer"
        );
        let mut sounds = Vec::new();
        for id in update.sounds {
            sounds.push(model.sound(id)?.to_vec());
        }
        Ok(Tick {
            changed: changed || update.changed,
            sounds,
        })
    }
    fn trigger(&mut self) -> Result<()> {
        if self.error.is_some() {
            return Ok(());
        }
        if let (Some(model), Some(start)) = (&mut self.model, self.model_start) {
            model.trigger_action(start.elapsed())?;
        }
        Ok(())
    }
    fn title(&self) -> String {
        if let Some(error) = &self.error {
            format!("LiTo 場景驗證錯誤：{error}")
        } else if let Some(model) = &self.model {
            model.status()
        } else {
            format!(
                "LiTo 載入中（非連線遊戲）：{} | Esc 取消",
                self.loading_status
            )
        }
    }
}

pub fn run_from_args(args: &[String]) -> Result<()> {
    let (root, heading) = parse_request(args)?;
    let mut state = Box::new(WindowState {
        ui: RefCell::new(Ui {
            job: LoadingJob::start(root, heading)?,
            model: None,
            model_start: None,
            loading_status: "初始化共享資源".into(),
            error: None,
        }),
        voice: RefCell::new(PcmVoice::new()),
        submitted: Cell::new(0),
        completed: Cell::new(0),
    });
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        ensure!(
            RegisterClassW(&WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: instance,
                lpszClassName: CLASS,
                hCursor: LoadCursorW(None, IDC_ARROW)?,
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            }) != 0,
            "register scene window: {}",
            std::io::Error::last_os_error()
        );
        let _class = ClassGuard(instance);
        let title = wide(&state.ui.borrow().title());
        let hwnd = CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            CLASS,
            PCWSTR(title.as_ptr()),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            1040,
            820,
            None,
            None,
            Some(instance),
            Some((&mut *state as *mut WindowState).cast()),
        )?;
        let _window = WindowGuard(hwnd);
        ensure!(
            SetTimer(Some(hwnd), TIMER, 20, None) != 0,
            "create scene update timer"
        );
        let _ = ShowWindow(hwnd, SW_SHOW);
        let mut message = MSG::default();
        loop {
            match GetMessageW(&mut message, None, 0, 0).0 {
                -1 => return Err(std::io::Error::last_os_error().into()),
                0 => break,
                _ => {
                    let _ = TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }
    state.voice.get_mut().stop()?;
    if let Some(error) = state.ui.get_mut().error.take() {
        anyhow::bail!(error);
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
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

unsafe fn refresh_title(hwnd: HWND, state: &WindowState) {
    let title = {
        let Ok(ui) = state.ui.try_borrow() else {
            return;
        };
        wide(&format!(
            "{} | audio={}/{}",
            ui.title(),
            state.submitted.get(),
            state.completed.get()
        ))
    };
    // No UI borrow across a synchronous, potentially reentrant window API.
    let _ = SetWindowTextW(hwnd, PCWSTR(title.as_ptr()));
    let _ = InvalidateRect(Some(hwnd), None, false);
}
unsafe fn fail(hwnd: HWND, state: &WindowState, error: anyhow::Error) {
    let _ = KillTimer(Some(hwnd), TIMER);
    if let Ok(mut ui) = state.ui.try_borrow_mut() {
        if ui.error.is_none() {
            ui.error = Some(format!("{error:#}"));
        }
    }
    let _ = state.voice.try_borrow_mut().map(|mut voice| voice.stop());
    refresh_title(hwnd, state);
}
unsafe fn update(hwnd: HWND, state: &WindowState) {
    let tick = {
        let Ok(mut ui) = state.ui.try_borrow_mut() else {
            return;
        };
        ui.tick()
    };
    let outcome = (|| -> Result<bool> {
        let tick = tick?;
        let mut changed = tick.changed;
        {
            // Cues own their WAV bytes; model borrows have ended before the
            // driver can cause any synchronous callback or window reentry.
            let mut voice = state
                .voice
                .try_borrow_mut()
                .context("reentrant audio update")?;
            if voice.poll()? == VoiceStatus::Finished {
                state.completed.set(
                    state
                        .completed
                        .get()
                        .checked_add(1)
                        .context("audio counter overflow")?,
                );
                changed = true;
            }
            for bytes in tick.sounds {
                voice.start(&parse_pcm_wav(&bytes)?)?;
                state.submitted.set(
                    state
                        .submitted
                        .get()
                        .checked_add(1)
                        .context("audio counter overflow")?,
                );
                changed = true;
            }
        }
        Ok(changed)
    })();
    match outcome {
        Ok(true) => refresh_title(hwnd, state),
        Ok(false) => {}
        Err(error) => fail(hwnd, state, error),
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
        WM_LBUTTONDOWN => {
            let mut rect = RECT::default();
            if GetClientRect(hwnd, &mut rect).is_ok() {
                // LPARAM packs signed 16-bit client coordinates on both x86/x64.
                let x = lparam.0 as u16 as i16 as i32;
                let y = ((lparam.0 as usize >> 16) as u16) as i16 as i32;
                let point = Viewport::fit(rect.right, (rect.bottom - 44).max(0), 800, 600)
                    .and_then(|view| view.canvas_point(x, y, 800, 600));
                if let Some((x,y)) = point {
                    let result = state.ui.try_borrow_mut().map_err(anyhow::Error::from)
                        .and_then(|mut ui| {
                            if let Some(model) = ui.model.as_mut() { model.place_actor(x,y)?; }
                            Ok(())
                        });
                    if let Err(error) = result { fail(hwnd, state, error); }
                    else { update(hwnd, state); }
                }
            }
            LRESULT(0)
        }
        WM_TIMER if wparam.0 == TIMER => {
            update(hwnd, state);
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 == 0x20 && (lparam.0 & (1 << 30)) == 0 => {
            let result = {
                state
                    .ui
                    .try_borrow_mut()
                    .map_err(anyhow::Error::from)
                    .and_then(|mut ui| ui.trigger())
            };
            match result {
                Ok(()) => update(hwnd, state),
                Err(error) => fail(hwnd, state, error),
            }
            LRESULT(0)
        }
        WM_KEYDOWN if wparam.0 == 0x1b => {
            close(hwnd, state);
            LRESULT(0)
        }
        WM_CLOSE => {
            close(hwnd, state);
            LRESULT(0)
        }
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
        WM_ERASEBKGND => LRESULT(1),
        WM_PAINT => {
            if let Ok(ui) = state.ui.try_borrow() {
                paint(hwnd, &ui);
                LRESULT(0)
            } else {
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
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
unsafe fn close(hwnd: HWND, state: &WindowState) {
    if let Ok(ui) = state.ui.try_borrow() {
        ui.job.cancel.store(true, Ordering::Release);
    }
    let result = state
        .voice
        .try_borrow_mut()
        .context("reentrant audio close")
        .and_then(|mut voice| voice.stop().map_err(Into::into));
    if let Err(error) = result {
        if let Ok(mut ui) = state.ui.try_borrow_mut() {
            ui.error = Some(error.to_string());
        }
    }
    let _ = DestroyWindow(hwnd);
}
unsafe fn paint(hwnd: HWND, ui: &Ui) {
    let mut paint = PAINTSTRUCT::default();
    let paint_dc = BeginPaint(hwnd, &mut paint);
    let mut rect = RECT::default();
    if GetClientRect(hwnd, &mut rect).is_ok() {
        let width = rect.right - rect.left;
        let height = rect.bottom - rect.top;
        if width <= 0 || height <= 0 {
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let dc = CreateCompatibleDC(Some(paint_dc));
        if dc.0.is_null() {
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let bitmap = CreateCompatibleBitmap(paint_dc, width, height);
        if bitmap.0.is_null() {
            let _ = DeleteDC(dc);
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        let previous = SelectObject(dc, bitmap.into());
        if previous.0.is_null() {
            let _ = DeleteObject(bitmap.into());
            let _ = DeleteDC(dc);
            let _ = EndPaint(hwnd, &paint);
            return;
        }
        FillRect(dc, &rect, HBRUSH(GetStockObject(BLACK_BRUSH).0));
        if let Some(model) = &ui.model {
            if let Some(view) = Viewport::fit(rect.right, (rect.bottom - 44).max(0), 800, 600) {
                let bitmap = BITMAPINFO {
                    bmiHeader: BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: 800,
                        biHeight: -600,
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
                    800,
                    600,
                    Some(model.bgra().as_ptr().cast()),
                    &bitmap,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            }
        } else {
            SetTextColor(dc, COLORREF(0x00dddddd));
            SetBkMode(dc, TRANSPARENT);
            let _ = TextOutW(dc, 24, 48, &ui.title().encode_utf16().collect::<Vec<_>>());
        }
        SetTextColor(dc, COLORREF(0x00dddddd));
        SetBkMode(dc, TRANSPARENT);
        let _ = TextOutW(
            dc,
            12,
            (rect.bottom - 30).max(0),
            &HELP.encode_utf16().collect::<Vec<_>>(),
        );
        let _ = BitBlt(paint_dc, 0, 0, width, height, Some(dc), 0, 0, SRCCOPY);
        SelectObject(dc, previous);
        let _ = DeleteObject(bitmap.into());
        let _ = DeleteDC(dc);
    }
    let _ = EndPaint(hwnd, &paint);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn failed_loading_is_terminal_and_preserves_the_original_diagnostic() {
        let (sender, receiver) = mpsc::sync_channel(1);
        drop(sender);
        let mut ui = Ui {
            job: LoadingJob {
                receiver,
                progress: Arc::new(Mutex::new(String::new())),
                cancel: Arc::new(AtomicBool::new(false)),
                worker: None,
            },
            model: None,
            model_start: None,
            loading_status: String::new(),
            error: Some("original shared asset failure".into()),
        };
        for _ in 0..3 {
            ui.trigger().unwrap();
            let tick = ui.tick().unwrap();
            assert!(!tick.changed);
            assert!(tick.sounds.is_empty());
            assert_eq!(ui.error.as_deref(), Some("original shared asset failure"));
        }
    }
    #[test]
    fn requires_explicit_root_and_valid_direction_before_starting_loader() {
        let args = |heading: &str| {
            ["launcher.exe", "--lito-scene", "D:/client", heading]
                .map(str::to_owned)
                .to_vec()
        };
        assert_eq!(parse_request(&args("7")).unwrap().1, 7);
        for bad in ["8", "-1", "abc", "256"] {
            assert!(parse_request(&args(bad)).is_err());
        }
        assert!(parse_request(&[]).is_err());
        assert!(HELP.contains("尚無伺服器"));
    }
}
