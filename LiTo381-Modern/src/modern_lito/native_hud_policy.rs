//! Window-independent HUD rules of the native login/world window (T035-2):
//! which window messages release held world input, where the world status box
//! sits relative to the rows below it, and which status line stays on top after
//! a text update. No Win32 calls here, so the rules run under `cargo test --lib`.

/// What a window message does to held movement, held attack and item cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    Keep,
    Input,
    /// Application deactivated: also stop playing combat voices.
    InputAndAudio,
}

const WM_SIZE: u32 = 0x0005;
const WM_ACTIVATE: u32 = 0x0006;
const WM_KILLFOCUS: u32 = 0x0008;
const WM_ENABLE: u32 = 0x000A;
const WM_ACTIVATEAPP: u32 = 0x001C;
const WM_CANCELMODE: u32 = 0x001F;
const WM_ENTERMENULOOP: u32 = 0x0211;
const WM_ENTERSIZEMOVE: u32 = 0x0231;
const SIZE_MINIMIZED: usize = 1;

/// Every way the world view can stop receiving this window's key-up messages.
/// The size/move and menu loops keep dispatching the 20 ms timer while they
/// swallow key messages, so a held attack would otherwise continue unattended.
pub fn focus_release(message: u32, wparam: usize) -> Release {
    match message {
        WM_ACTIVATEAPP if wparam == 0 => Release::InputAndAudio,
        WM_KILLFOCUS | WM_CANCELMODE | WM_ENTERSIZEMOVE | WM_ENTERMENULOOP => Release::Input,
        // LOWORD: WA_INACTIVE; HIWORD: activated while minimized.
        WM_ACTIVATE if wparam & 0xffff == 0 || (wparam >> 16) & 0xffff != 0 => Release::Input,
        WM_SIZE if wparam == SIZE_MINIMIZED => Release::Input,
        WM_ENABLE if wparam == 0 => Release::Input,
        _ => Release::Keep,
    }
}

/// Vertical layout of the plain world view, in 96-DPI units above the client
/// bottom. `top` is the distance of a row's upper edge from the bottom.
pub mod strip {
    pub const STATUS_TOP: i32 = 250;
    pub const STATUS_HEIGHT: i32 = 90;
    /// Teleport destination, summon choice or skill text: one row at a time.
    pub const OPTION_TOP: i32 = 190;
    pub const OPTION_HEIGHT: i32 = 24;
    /// Rows above the option row move up by this much while it is shown.
    pub const OPTION_LIFT: i32 = 40;
    pub const HOTBAR_TOP: i32 = 155;
    pub const HOTBAR_HEIGHT: i32 = 58;
    pub const BOOK_LABEL_TOP: i32 = 453;
    pub const BOOK_LABEL_HEIGHT: i32 = 24;
    pub const BOOK_TOP: i32 = 425;
    pub const BOOK_HEIGHT: i32 = 170;
}

/// The single option row under the status box. Mirrors the visibility of the
/// three controls that share that row.
pub fn option_row(
    destination_skill: bool,
    summon_skill: bool,
    text_skill: bool,
    rental_open: bool,
    ground_view: bool,
) -> bool {
    destination_skill || (summon_skill && !rental_open) || (text_skill && !ground_view)
}

pub fn option_lift(option_row: bool) -> i32 {
    if option_row {
        strip::OPTION_LIFT
    } else {
        0
    }
}

/// Status box `(top, height)` for the world view.
pub fn world_status(inventory: bool, ground: bool, result: bool, option_row: bool) -> (i32, i32) {
    if inventory {
        (if ground { 286 } else { 326 }, 80)
    } else if result {
        (strip::STATUS_TOP, 45)
    } else {
        (strip::STATUS_TOP + option_lift(option_row), strip::STATUS_HEIGHT)
    }
}

/// `(first visible line, wrapped lines, fully visible lines)` of the status box.
pub type StatusLines = (usize, usize, usize);

/// First visible status line after the text was replaced or the box was moved
/// or resized. A reader who scrolled back keeps that line; otherwise the box
/// follows the last line, so a wrapped long hint is not left below the visible
/// rows. `previous` is the state just before the change.
pub fn status_first_line(previous: StatusLines, lines: usize, visible: usize) -> usize {
    let (previous_first, previous_lines, previous_visible) = previous;
    let last = lines.saturating_sub(visible.max(1));
    if previous_first >= previous_lines.saturating_sub(previous_visible.max(1)) {
        last
    } else {
        previous_first.min(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn focus_loss_minimize_and_modal_loops_release_held_input() {
        assert_eq!(focus_release(WM_ACTIVATEAPP, 0), Release::InputAndAudio);
        assert_eq!(focus_release(WM_ACTIVATEAPP, 1), Release::Keep);
        for message in [WM_KILLFOCUS, WM_CANCELMODE, WM_ENTERSIZEMOVE, WM_ENTERMENULOOP] {
            assert_eq!(focus_release(message, 0), Release::Input, "{message:#x}");
            assert_eq!(focus_release(message, 7), Release::Input, "{message:#x}");
        }
        // WA_INACTIVE, and WA_ACTIVE / WA_CLICKACTIVE.
        assert_eq!(focus_release(WM_ACTIVATE, 0), Release::Input);
        assert_eq!(focus_release(WM_ACTIVATE, 1), Release::Keep);
        assert_eq!(focus_release(WM_ACTIVATE, 2), Release::Keep);
        // Activated while minimized: no world view to act in.
        assert_eq!(focus_release(WM_ACTIVATE, 1 | (1 << 16)), Release::Input);
        assert_eq!(focus_release(WM_SIZE, SIZE_MINIMIZED), Release::Input);
        assert_eq!(focus_release(WM_SIZE, 0), Release::Keep);
        assert_eq!(focus_release(WM_SIZE, 2), Release::Keep);
        assert_eq!(focus_release(WM_ENABLE, 0), Release::Input);
        assert_eq!(focus_release(WM_ENABLE, 1), Release::Keep);
    }

    #[test]
    fn ordinary_messages_keep_held_input() {
        // WM_SETFOCUS, WM_PAINT, WM_KEYDOWN, WM_KEYUP, WM_TIMER, WM_LBUTTONDOWN,
        // WM_EXITSIZEMOVE, WM_EXITMENULOOP.
        for message in [0x0007, 0x000F, 0x0100, 0x0101, 0x0113, 0x0201, 0x0232, 0x0212] {
            for wparam in [0, 1, 0x20] {
                assert_eq!(focus_release(message, wparam), Release::Keep, "{message:#x}");
            }
        }
    }

    #[test]
    fn option_row_follows_the_three_controls_that_share_it() {
        assert!(!option_row(false, false, false, false, false));
        assert!(option_row(true, false, false, true, true));
        assert!(option_row(false, true, false, false, false));
        assert!(!option_row(false, true, false, true, false));
        assert!(option_row(false, false, true, false, false));
        assert!(!option_row(false, false, true, false, true));
    }

    /// `(top, height)` rows must be stacked without sharing a pixel row.
    fn stacked(rows: &[(i32, i32)]) {
        for pair in rows.windows(2) {
            let (upper, lower) = (pair[0], pair[1]);
            assert!(upper.1 > 0 && lower.1 > 0);
            assert!(upper.0 - upper.1 >= lower.0, "{upper:?} overlaps {lower:?}");
        }
    }

    #[test]
    fn world_rows_never_overlap_with_or_without_the_option_row() {
        use strip::*;
        for option in [false, true] {
            let lift = option_lift(option);
            let status = world_status(false, false, false, option);
            let mut rows = vec![
                (BOOK_LABEL_TOP + lift, BOOK_LABEL_HEIGHT),
                (BOOK_TOP + lift, BOOK_HEIGHT),
                status,
            ];
            if option {
                rows.push((OPTION_TOP, OPTION_HEIGHT));
            }
            rows.push((HOTBAR_TOP, HOTBAR_HEIGHT));
            // Book／cast row.
            rows.push((90, 32));
            // Bottom button row.
            rows.push((45, 32));
            stacked(&rows);
        }
        // Without the lift the status box would cover the option row.
        let unlifted = world_status(false, false, false, false);
        assert!(unlifted.0 - unlifted.1 < strip::OPTION_TOP);
    }

    #[test]
    fn inventory_and_result_status_boxes_keep_their_positions() {
        for option in [false, true] {
            assert_eq!(world_status(true, false, false, option), (326, 80));
            assert_eq!(world_status(true, true, false, option), (286, 80));
            assert_eq!(world_status(true, true, true, option), (286, 80));
            assert_eq!(world_status(false, false, true, option), (250, 45));
        }
        assert_eq!(world_status(false, false, false, false), (250, 90));
        assert_eq!(world_status(false, false, false, true), (290, 90));
    }

    #[test]
    fn status_follows_the_last_line_unless_the_reader_scrolled_back() {
        // Everything fits: nothing to scroll.
        assert_eq!(status_first_line((0, 4, 4), 4, 4), 0);
        assert_eq!(status_first_line((0, 0, 4), 3, 4), 0);
        // A long hint wraps to six lines in a four-line box: show the last line.
        assert_eq!(status_first_line((0, 4, 4), 6, 4), 2);
        // Still following while the wrapped text keeps changing.
        assert_eq!(status_first_line((2, 6, 4), 7, 4), 3);
        assert_eq!(status_first_line((3, 7, 4), 5, 4), 1);
        // Scrolled back to the first line: stay there.
        assert_eq!(status_first_line((0, 6, 4), 7, 4), 0);
        assert_eq!(status_first_line((1, 7, 4), 7, 4), 1);
        // A kept line is clamped when the text gets shorter.
        assert_eq!(status_first_line((1, 7, 4), 4, 4), 0);
        assert_eq!(status_first_line((2, 8, 4), 5, 4), 1);
        // Unknown metrics never divide or underflow.
        assert_eq!(status_first_line((0, 0, 0), 0, 0), 0);
        assert_eq!(status_first_line((5, 2, 0), 3, 0), 2);
    }

    #[test]
    fn status_keeps_following_when_the_box_itself_changes() {
        // A narrower window wraps the same text into more lines.
        assert_eq!(status_first_line((0, 4, 4), 6, 4), 2);
        // The box gets shorter (lobby 110 → world 90) with unchanged text.
        assert_eq!(status_first_line((0, 5, 5), 5, 4), 1);
        // It was following in the taller box and keeps following.
        assert_eq!(status_first_line((1, 6, 5), 6, 4), 2);
        // A wider or taller box shows everything again.
        assert_eq!(status_first_line((2, 6, 4), 4, 4), 0);
        assert_eq!(status_first_line((2, 6, 4), 6, 6), 0);
        // The reader's line survives a resize that still needs scrolling.
        assert_eq!(status_first_line((1, 8, 4), 9, 4), 1);
        assert_eq!(status_first_line((1, 8, 4), 8, 5), 1);
    }
}
