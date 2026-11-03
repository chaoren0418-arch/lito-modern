//! A bounded, read-only battle log/HUD; never an additional chat input.
//! Native EDIT wrapping/scrolling keeps long diagnostics and mode results
//! reachable without drawing over buttons or sending actor keyboard actions.
use super::*;

const ID: usize = STATUS_TEXT;
fn style() -> WINDOW_STYLE {
    WS_CHILD
        | WS_VISIBLE
        | WS_TABSTOP
        | WS_VSCROLL
        | WINDOW_STYLE((ES_MULTILINE | ES_READONLY | ES_AUTOVSCROLL) as u32)
}
pub(super) unsafe fn create(parent: HWND, instance: HINSTANCE) -> Result<HWND> {
    let control = CreateWindowExW(
        WS_EX_CLIENTEDGE,
        w!("EDIT"),
        w!("正在準備已確認世界的共用資源…"),
        style(),
        0,
        0,
        0,
        0,
        Some(parent),
        Some(HMENU(ID as *mut _)),
        Some(instance),
        None,
    )?;
    SendMessageW(control, EM_SETLIMITTEXT, Some(WPARAM(4096)), None);
    Ok(control)
}

pub(super) fn bounds(width: i32, bottom: i32, dpi: u32) -> RECT {
    let width = width.max(0);
    let bottom = bottom.max(0);
    let margin = scale(12, dpi).min(width / 8);
    RECT {
        left: margin,
        right: width - margin,
        top: (bottom - scale(FOOTER, dpi)).max(0),
        bottom: (bottom - scale(308, dpi)).max(0),
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum KeyRoute {
    FocusWorld,
    Tab,
    Native,
}
pub(super) fn key_route(key: usize) -> KeyRoute {
    match key {
        0x1b => KeyRoute::FocusWorld,
        0x09 => KeyRoute::Tab,
        _ => KeyRoute::Native,
    }
}

/// Called only when text actually changes. Preserve inspection scroll/copy
/// selection while new server HP, phase or result messages arrive.
pub(super) unsafe fn replace(control: HWND, text: &[u16]) -> Result<()> {
    let (mut start, mut end) = (0u32, 0u32);
    SendMessageW(
        control,
        EM_GETSEL,
        Some(WPARAM((&mut start as *mut u32) as usize)),
        Some(LPARAM((&mut end as *mut u32) as isize)),
    );
    let first = SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None);
    let encoded: Vec<u16> = text.iter().copied().chain(Some(0)).collect();
    SetWindowTextW(control, PCWSTR(encoded.as_ptr()))?;
    SendMessageW(
        control,
        EM_SETSEL,
        Some(WPARAM((start as usize).min(text.len()))),
        Some(LPARAM((end as usize).min(text.len()) as isize)),
    );
    let current = SendMessageW(control, EM_GETFIRSTVISIBLELINE, None, None);
    SendMessageW(
        control,
        EM_LINESCROLL,
        None,
        Some(LPARAM(first.0 - current.0)),
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scrollable_status_is_read_only_multiline_and_wraps() {
        let s = style().0;
        for flag in [
            WS_CHILD.0,
            WS_VISIBLE.0,
            WS_TABSTOP.0,
            WS_VSCROLL.0,
            ES_MULTILINE as u32,
            ES_READONLY as u32,
            ES_AUTOVSCROLL as u32,
        ] {
            assert_ne!(s & flag, 0);
        }
        assert_eq!(s & (ES_AUTOHSCROLL as u32 | WS_HSCROLL.0), 0);
    }
    #[test]
    fn status_never_overlaps_world_or_inspection_buttons_at_dpi_or_tiny_sizes() {
        for dpi in [48, 96, 120, 144, 192, 768] {
            for width in [0, 1, 240, 800, 1600] {
                for bottom in [0, 1, 240, scale(HEIGHT + FOOTER, dpi), 2400] {
                    let r = bounds(width, bottom, dpi);
                    assert!(r.left >= 0 && r.right <= width && r.left <= r.right);
                    assert!(r.top >= 0 && r.bottom <= bottom && r.top <= r.bottom);
                    assert_eq!(r.top, (bottom - scale(FOOTER, dpi)).max(0));
                    assert!(r.bottom <= (bottom - scale(302, dpi)).max(0));
                    if bottom >= scale(HEIGHT + FOOTER, dpi) {
                        assert_eq!(r.bottom - r.top, scale(FOOTER, dpi) - scale(308, dpi));
                    }
                }
            }
        }
    }
    #[test]
    fn copy_scroll_enter_and_game_keys_stay_native_when_status_has_focus() {
        for key in [
            0x21, 0x22, 0x23, 0x24, 0x25, 0x26, 0x27, 0x28, 0x0d, 0x43, 0x46, 0x54,
        ] {
            assert_eq!(key_route(key), KeyRoute::Native);
        }
        assert_eq!(key_route(0x1b), KeyRoute::FocusWorld);
        assert_eq!(key_route(0x09), KeyRoute::Tab);
    }
}
