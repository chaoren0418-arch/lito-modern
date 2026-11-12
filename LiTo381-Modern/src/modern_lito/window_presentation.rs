//! Startup presentation recovery for the client's own top-level windows.
//!
//! A launcher can hand the process a `STARTUPINFO` show command that replaces
//! the first `ShowWindow` call: `SW_HIDE` leaves the window hidden, and
//! `SW_SHOWMINIMIZED`／`SW_SHOWMINNOACTIVE` leaves it visible but iconic. An
//! iconic window has no composed surface and no on-screen control rectangles,
//! so window capture waits for a frame that never arrives and coordinate input
//! has no geometry. A default position can also fall outside the current work
//! area. These routes are explicitly interactive, so they always end with a
//! restored window whose frame starts inside the nearest monitor's work area.
//!
//! Only the window passed in is touched; no other process, desktop or system
//! setting is changed.
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, UpdateWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    },
    UI::WindowsAndMessaging::{
        GetWindowRect, IsIconic, IsWindowVisible, IsZoomed, SetForegroundWindow, SetWindowPos,
        ShowWindow, SWP_NOACTIVATE, SWP_NOZORDER, SW_RESTORE, SW_SHOW, SW_SHOWNORMAL,
    },
};

/// Screen rectangle as `(left, top, right, bottom)`.
pub(crate) type Bounds = (i32, i32, i32, i32);

/// What the second pass has to do after the first `ShowWindow(SW_SHOW)`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ShowRecovery {
    /// Visible and not iconic.
    None,
    /// The launcher's show command hid the window.
    Show,
    /// The launcher's show command minimized the window.
    Restore,
}

pub(crate) fn show_recovery(visible: bool, iconic: bool) -> ShowRecovery {
    if !visible {
        ShowRecovery::Show
    } else if iconic {
        ShowRecovery::Restore
    } else {
        ShowRecovery::None
    }
}

/// Move (and only when larger than the work area, shrink) a window frame so it
/// lies inside `work`. Returns `None` when the frame already fits or either
/// rectangle is empty. A frame the window refuses to shrink (minimum track
/// size) still ends with its top-left corner, and so its caption, on screen.
pub(crate) fn fit_to_work_area(window: Bounds, work: Bounds) -> Option<Bounds> {
    let (left, top, right, bottom) = window;
    let (work_left, work_top, work_right, work_bottom) = work;
    let width = right.checked_sub(left)?;
    let height = bottom.checked_sub(top)?;
    let work_width = work_right.checked_sub(work_left)?;
    let work_height = work_bottom.checked_sub(work_top)?;
    if width <= 0 || height <= 0 || work_width <= 0 || work_height <= 0 {
        return None;
    }
    let width = width.min(work_width);
    let height = height.min(work_height);
    // `work_right - width >= work_left` holds after the clamp above.
    let x = left.clamp(work_left, work_right - width);
    let y = top.clamp(work_top, work_bottom - height);
    let fitted = (x, y, x + width, y + height);
    (fitted != window).then_some(fitted)
}

/// Show a just-created top-level window and undo a hidden, minimized or
/// off-screen start. Call once, on the window's own thread, before the message
/// loop. A maximized window is left as it is.
///
/// # Safety
/// `hwnd` must be a live window owned by the calling thread.
pub(crate) unsafe fn present(hwnd: HWND) {
    let _ = ShowWindow(hwnd, SW_SHOW);
    match show_recovery(
        IsWindowVisible(hwnd).as_bool(),
        IsIconic(hwnd).as_bool(),
    ) {
        ShowRecovery::None => {}
        ShowRecovery::Show => {
            // The first call consumed the launcher's SW_HIDE.
            let _ = ShowWindow(hwnd, SW_SHOWNORMAL);
        }
        ShowRecovery::Restore => {
            let _ = ShowWindow(hwnd, SW_RESTORE);
        }
    }
    // A hidden start followed by SW_SHOWNORMAL cannot be iconic, but a second
    // overridden show command is cheap to rule out.
    if IsIconic(hwnd).as_bool() {
        let _ = ShowWindow(hwnd, SW_RESTORE);
    }
    if !IsZoomed(hwnd).as_bool() {
        let mut frame = RECT::default();
        let mut monitor = MONITORINFO {
            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        if GetWindowRect(hwnd, &mut frame).is_ok()
            && GetMonitorInfoW(
                MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST),
                &mut monitor,
            )
            .as_bool()
        {
            let work = monitor.rcWork;
            if let Some((left, top, right, bottom)) = fit_to_work_area(
                (frame.left, frame.top, frame.right, frame.bottom),
                (work.left, work.top, work.right, work.bottom),
            ) {
                let _ = SetWindowPos(
                    hwnd,
                    None,
                    left,
                    top,
                    right - left,
                    bottom - top,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
    // Best effort: Windows refuses this for a process without foreground
    // rights, and the window then stays visible behind the current one.
    let _ = SetForegroundWindow(hwnd);
    // Paint now so the first composed frame does not wait for the first timer.
    let _ = UpdateWindow(hwnd);
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORK: Bounds = (0, 0, 1920, 1040);

    #[test]
    fn show_recovery_covers_hidden_minimized_and_normal_starts() {
        assert_eq!(show_recovery(true, false), ShowRecovery::None);
        assert_eq!(show_recovery(false, false), ShowRecovery::Show);
        assert_eq!(show_recovery(true, true), ShowRecovery::Restore);
        // Hidden wins: SW_RESTORE on a hidden iconic window is checked afterwards.
        assert_eq!(show_recovery(false, true), ShowRecovery::Show);
    }

    #[test]
    fn frame_inside_work_area_is_untouched() {
        assert_eq!(fit_to_work_area((100, 100, 700, 920), WORK), None);
        assert_eq!(fit_to_work_area(WORK, WORK), None);
    }

    #[test]
    fn minimized_placeholder_coordinates_are_brought_back() {
        // Windows parks iconic windows at (-32000, -32000).
        assert_eq!(
            fit_to_work_area((-32000, -32000, -31400, -31180), WORK),
            Some((0, 0, 600, 820))
        );
    }

    #[test]
    fn frame_past_right_and_bottom_edges_keeps_its_size() {
        assert_eq!(
            fit_to_work_area((1800, 900, 2400, 1720), WORK),
            Some((1320, 220, 1920, 1040))
        );
    }

    #[test]
    fn frame_taller_than_work_area_is_clamped_to_it() {
        // 600×820 DIP at 192 DPI on a 1080p work area.
        assert_eq!(
            fit_to_work_area((200, 50, 1400, 1690), WORK),
            Some((200, 0, 1400, 1040))
        );
    }

    #[test]
    fn secondary_monitor_with_negative_origin_is_respected() {
        let work = (-2560, -200, 0, 1200);
        assert_eq!(fit_to_work_area((-2000, 0, -1400, 820), work), None);
        assert_eq!(
            fit_to_work_area((-3000, -500, -2400, 320), work),
            Some((-2560, -200, -1960, 620))
        );
    }

    #[test]
    fn empty_or_inverted_rectangles_are_ignored() {
        assert_eq!(fit_to_work_area((10, 10, 10, 500), WORK), None);
        assert_eq!(fit_to_work_area((10, 10, 500, 5), WORK), None);
        assert_eq!(fit_to_work_area((10, 10, 500, 500), (0, 0, 0, 0)), None);
        assert_eq!(
            fit_to_work_area((i32::MIN, 0, i32::MAX, 10), WORK),
            None
        );
    }
}
