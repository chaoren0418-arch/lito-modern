//! Native chat panel (T060): history, category filter, channel and whisper
//! input. It shares the live GAME controller; every rule lives in
//! `native_chat`, so this file only moves text between controls and requests.
use super::*;
use crate::modern_lito::native_chat as chat;

pub const TOGGLE: u16 = 360;
const LOG: u16 = 361;
/// Seven check boxes in `chat::Category::ALL` order: 362..=368.
const FILTER_FIRST: u16 = 362;
const CHANNEL: u16 = 369;
const TARGET_LABEL: u16 = 370;
pub const TARGET: u16 = 371;
pub const MESSAGE: u16 = 372;
pub const SEND: u16 = 373;
const NOTICE: u16 = 374;
const FILTER_LABEL: u16 = 375;
const IDS: [u16; 16] = [
    TOGGLE,
    LOG,
    FILTER_FIRST,
    FILTER_FIRST + 1,
    FILTER_FIRST + 2,
    FILTER_FIRST + 3,
    FILTER_FIRST + 4,
    FILTER_FIRST + 5,
    FILTER_FIRST + 6,
    CHANNEL,
    TARGET_LABEL,
    TARGET,
    MESSAGE,
    SEND,
    NOTICE,
    FILTER_LABEL,
];
/// Combo row after the four `chat::Channel::ALL` rows.
const WHISPER_ROW: usize = chat::Channel::ALL.len();

#[derive(Default)]
pub struct View {
    pub open: bool,
    filter: chat::Filter,
    /// Log revision and filter last written to the history box.
    shown: Option<(u64, chat::Filter)>,
    /// Last log sequence the reader has seen with the panel open.
    seen: u64,
    notice: String,
    notice_shown: Option<String>,
    toggle_shown: Option<&'static str>,
}

impl Ui {
    pub(super) unsafe fn create_chat_controls(&mut self) -> Result<()> {
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let edit = ES_AUTOHSCROLL as u32 | WS_BORDER.0;
        let mut rows: Vec<(u16, PCWSTR, &str, u32)> = vec![
            (TOGGLE, w!("BUTTON"), "聊天", 0),
            (
                LOG,
                w!("EDIT"),
                "",
                ES_MULTILINE as u32 | ES_READONLY as u32 | WS_VSCROLL.0 | WS_BORDER.0,
            ),
            (FILTER_LABEL, w!("STATIC"), "顯示", 0),
        ];
        for (index, category) in chat::Category::ALL.iter().enumerate() {
            rows.push((
                FILTER_FIRST + index as u16,
                w!("BUTTON"),
                category.label(),
                BS_AUTOCHECKBOX as u32,
            ));
        }
        rows.extend([
            (CHANNEL, w!("COMBOBOX"), "", CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0),
            (TARGET_LABEL, w!("STATIC"), "密語對象", 0),
            (TARGET, w!("EDIT"), "", edit),
            (MESSAGE, w!("EDIT"), "", edit),
            (SEND, w!("BUTTON"), "送出", 0),
            (NOTICE, w!("STATIC"), "", 0),
        ]);
        for (id, class, label, style) in rows {
            let text = wide(label);
            let h = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                class,
                PCWSTR(text.as_ptr()),
                WINDOW_STYLE(WS_CHILD.0 | WS_TABSTOP.0 | style),
                0,
                0,
                1,
                1,
                Some(self.parent),
                Some(HMENU(id as usize as *mut _)),
                Some(instance),
                None,
            )?;
            send(h, WM_SETFONT, WPARAM(self.font.0 as usize), LPARAM(0));
            self.controls.push((id, h));
        }
        for index in 0..chat::Category::ALL.len() {
            send(
                self.control(FILTER_FIRST + index as u16),
                BM_SETCHECK,
                WPARAM(1),
                LPARAM(0),
            );
        }
        let channel = self.control(CHANNEL);
        for label in chat::Channel::ALL.iter().map(|c| c.label()).chain(["密語"]) {
            let label = wide(label);
            send(channel, CB_ADDSTRING, WPARAM(0), LPARAM(label.as_ptr() as isize));
        }
        send(channel, CB_SETCURSEL, WPARAM(0), LPARAM(0));
        // UTF-16 units; the MS950 byte limits are enforced when sending.
        send(self.control(TARGET), EM_SETLIMITTEXT, WPARAM(chat::NAME_BYTES), LPARAM(0));
        send(self.control(MESSAGE), EM_SETLIMITTEXT, WPARAM(chat::MESSAGE_BYTES), LPARAM(0));
        Ok(())
    }
    fn chat_available(&self) -> bool {
        let s = self.controller.snapshot();
        !self.closing && s.game_connected && s.selected_character.is_some()
    }
    /// Other full-page views own the window while they are shown.
    fn chat_blocked(&self) -> bool {
        self.inventory_view || self.rental.open || self.companions.open || self.room_tools_view
    }
    pub(super) unsafe fn layout_chat(&mut self) {
        let phase = self.controller.snapshot().phase;
        let show = self.chat_available()
            && !self.chat_blocked()
            && (self.chat.open || self.world_view || chat::sendable(phase));
        let mut area = RECT::default();
        let _ = GetClientRect(self.parent, &mut area);
        let dpi = GetDpiForWindow(self.parent).max(96) as i32;
        let px = |n: i32| n * dpi / 96;
        for id in IDS {
            let _ = ShowWindow(
                self.control(id),
                if show && (id == TOGGLE || self.chat.open) {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
        let (x, y) = if self.chat.open {
            (px(20), area.bottom - px(45))
        } else if self.world_view {
            // Under the companion toggle.
            ((area.right - px(160)).max(px(20)), px(58))
        } else {
            // Below every row of the lobby and room forms.
            ((area.right - px(160)).max(px(20)), area.bottom - px(45))
        };
        let _ = MoveWindow(self.control(TOGGLE), x, y, px(140), px(32), true);
        let unread = self.controller.snapshot().chat.sequence > self.chat.seen;
        let label = if self.chat.open {
            "回到畫面"
        } else if unread {
            "聊天（新訊息）"
        } else {
            "聊天"
        };
        if self.chat.toggle_shown != Some(label) {
            self.text(TOGGLE, label);
            self.chat.toggle_shown = Some(label);
        }
        if !show || !self.chat.open {
            return;
        }
        for &(id, h) in &self.controls {
            if !IDS.contains(&id) {
                let _ = ShowWindow(h, SW_HIDE);
            }
        }
        let width = (area.right * 96 / dpi - 40).max(440);
        let bottom = (area.bottom * 96 / dpi).max(480);
        let mut rows = vec![
            (LOG, 20, 20, width, bottom - 242),
            (FILTER_LABEL, 20, bottom - 208, 50, 24),
            (CHANNEL, 20, bottom - 175, 110, 200),
            (TARGET_LABEL, 140, bottom - 171, 80, 24),
            (TARGET, 225, bottom - 175, (width - 205).min(220), 28),
            (MESSAGE, 20, bottom - 135, width - 110, 28),
            (SEND, width - 80, bottom - 136, 100, 30),
            (NOTICE, 20, bottom - 97, width, 44),
        ];
        for index in 0..chat::Category::ALL.len() as i32 {
            rows.push((FILTER_FIRST + index as u16, 72 + index * 58, bottom - 212, 58, 28));
        }
        for (id, x, y, w, h) in rows {
            let _ = MoveWindow(self.control(id), px(x), px(y), px(w), px(h), true);
        }
    }
    pub(super) unsafe fn refresh_chat(&mut self) {
        if self.chat.open && (!self.chat_available() || self.chat_blocked()) {
            self.chat.open = false;
            self.chat.shown = None;
            self.painted = None;
            // A hidden field must not keep the keyboard.
            let focus = GetFocus();
            if IDS.iter().any(|id| self.control(*id) == focus) {
                let _ = SetFocus(Some(self.parent));
            }
            self.layout(self.parent, false);
            let _ = InvalidateRect(Some(self.parent), None, true);
        }
        if self.chat.open {
            let (revision, sequence, phase) = {
                let s = self.controller.snapshot();
                (s.chat.revision, s.chat.sequence, s.phase)
            };
            let key = (revision, self.chat.filter);
            if self.chat.shown != Some(key) {
                let text = self.controller.snapshot().chat.render(self.chat.filter);
                let log = self.control(LOG);
                // Keep a line the reader scrolled back to; otherwise follow the end.
                let before = self.status_lines(log);
                let text = wide(if text.is_empty() { "（尚無訊息）" } else { &text });
                let _ = SetWindowTextW(log, PCWSTR(text.as_ptr()));
                self.follow_status(log, before);
                self.chat.shown = Some(key);
            }
            self.chat.seen = sequence;
            let sendable = chat::sendable(phase);
            let notice = if !sendable {
                "目前狀態不能發言；需在大廳、房間或場景中。".to_owned()
            } else if self.chat.notice.is_empty() {
                "Enter 送出。密語請選「密語」頻道並填寫對象。".to_owned()
            } else {
                self.chat.notice.clone()
            };
            if self.chat.notice_shown.as_ref() != Some(&notice) {
                self.text(NOTICE, &notice);
                self.chat.notice_shown = Some(notice);
            }
            let _ = EnableWindow(self.control(SEND), sendable);
        }
        self.layout_chat();
    }
    /// Enter in the message or whisper-target field sends the line.
    pub(super) unsafe fn chat_input_focus(&self, focus: HWND) -> bool {
        self.chat.open
            && !self.closing
            && (focus == self.control(MESSAGE) || focus == self.control(TARGET))
    }
    pub(super) unsafe fn chat_action(&mut self, id: u16) -> Result<bool> {
        if !IDS.contains(&id) {
            return Ok(false);
        }
        if id == TOGGLE {
            ensure!(self.chat.open || (self.chat_available() && !self.chat_blocked()), "目前不能開啟聊天");
            self.chat.open = !self.chat.open;
            self.chat.notice.clear();
            self.chat.shown = None;
            self.held.clear();
            self.held_attack.clear();
            self.item_cursor = None;
            self.painted = None;
            self.layout(self.parent, false);
            self.refresh_chat();
            let _ = InvalidateRect(Some(self.parent), None, true);
            let _ = SetFocus(Some(if self.chat.open {
                self.control(MESSAGE)
            } else {
                self.parent
            }));
            return Ok(true);
        }
        ensure!(self.chat.open && !self.closing, "請先開啟聊天");
        if (FILTER_FIRST..FILTER_FIRST + chat::Category::ALL.len() as u16).contains(&id) {
            let category = chat::Category::ALL[usize::from(id - FILTER_FIRST)];
            let shown = send(self.control(id), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 == 1;
            if self.chat.filter.set(category, shown) {
                self.chat.notice = match chat::Preference::of(category) {
                    // The local filter already applies; the server preference
                    // additionally stops delivery and is best effort.
                    Some(preference) => match self.controller.chat_preference(preference, shown) {
                        Ok(()) => format!(
                            "已{}「{}」，並通知伺服器。",
                            if shown { "顯示" } else { "隱藏" },
                            category.label()
                        ),
                        Err(error) => format!(
                            "已在本機{}「{}」；伺服器設定未送出：{error:#}",
                            if shown { "顯示" } else { "隱藏" },
                            category.label()
                        ),
                    },
                    None => String::new(),
                };
            }
            return Ok(true);
        }
        if id == SEND {
            let result = (|| -> Result<&'static str> {
                let message = self.read(MESSAGE, 1024)?;
                let row = send(self.control(CHANNEL), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0;
                let row = usize::try_from(row).ok().context("請選取頻道")?;
                if row == WHISPER_ROW {
                    let target = self.read(TARGET, 64)?;
                    self.controller.chat_tell(&target, &message)?;
                    Ok("密語已送出；對象不在線或拒收時，伺服器不會通知。")
                } else {
                    let channel = *chat::Channel::ALL.get(row).context("請選取頻道")?;
                    self.controller.chat_say(channel, &message)?;
                    Ok("已送出；伺服器回傳後顯示。被禁言時伺服器不會回覆。")
                }
            })();
            match result {
                Ok(notice) => {
                    self.text(MESSAGE, "");
                    self.chat.notice = notice.to_owned();
                }
                Err(error) => self.chat.notice = format!("{error:#}"),
            }
            let _ = SetFocus(Some(self.control(MESSAGE)));
        }
        Ok(true)
    }
}
