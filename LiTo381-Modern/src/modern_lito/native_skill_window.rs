//! Native skill book and F5–F12 controls. Assets never authorize a request.
use super::*;
use windows::Win32::UI::Input::KeyboardAndMouse::GetKeyState;

pub(super) const BOOK: u16 = 138;
pub(super) const FIRST: u16 = 400;
pub(super) const LAST: u16 = FIRST + 7;

pub(super) fn slot_key(key: usize) -> Option<usize> {
    (0x74..=0x7b).contains(&key).then(|| key - 0x74)
}

// F10 arrives as WM_SYSKEYDOWN. Consume it without activating the system menu,
// while leaving Alt combinations and key-up messages to Windows.
pub(super) fn hotbar_key(message: u32, key: usize, flags: isize) -> Option<(usize, bool)> {
    if !matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN) || flags & (1 << 29) != 0 {
        return None;
    }
    slot_key(key).map(|slot| (slot, flags & (1 << 30) == 0))
}

#[cfg(test)]
mod skill_ui_tests {
    use super::*;
    #[test]
    fn all_eight_keys_include_f10_consume_repeats_and_leave_alt_to_windows() {
        for (slot, key) in (0x74..=0x7b).enumerate() {
            let msg = if key == 0x79 {
                WM_SYSKEYDOWN
            } else {
                WM_KEYDOWN
            };
            assert_eq!(hotbar_key(msg, key, 0), Some((slot, true)));
            assert_eq!(hotbar_key(msg, key, 1 << 30), Some((slot, false)));
            assert_eq!(hotbar_key(WM_SYSKEYDOWN, key, 1 << 29), None);
            assert_eq!(hotbar_key(WM_KEYUP, key, 0), None);
        }
        assert_eq!(hotbar_key(WM_KEYDOWN, 0x73, 0), None);
        assert_eq!(hotbar_key(WM_KEYDOWN, 0x7c, 0), None);
    }
}

impl Ui {
    pub(super) unsafe fn skill_input_focus(&self, focus: HWND) -> bool {
        focus == self.parent
            || focus == self.control(SKILLS)
            || focus == self.control(BOOK)
            || focus == self.control(CAST_SKILL)
            || (FIRST..=LAST).any(|id| focus == self.control(id))
    }
    pub(super) unsafe fn skill_slot(&mut self, slot: usize) -> Result<()> {
        ensure!(
            self.world_view
                && !self.inventory_view
                && !self.rental.open
                && !self.closing
                && self.controller.snapshot().phase.active_world()
                && !self.controller.snapshot().observer,
            "目前不能操作快捷列"
        );
        self.held.clear();
        self.held_attack.clear();
        // Refresh at activation, not just on the timer. A stale binding must
        // never authorize a different character or a removed/reordered row.
        self.refresh_skills();
        if GetKeyState(0x10) < 0 {
            self.skills.bind(slot, None)?;
        } else if GetKeyState(0x11) < 0 {
            let id = self.skills.selected().context("請先在技能書選擇技能")?;
            self.skills.bind(slot, Some(id))?;
        } else {
            let id = self.skills.bound(
                self.controller.transport().snapshot().epoch,
                &self.controller.snapshot().world,
                slot,
            )?;
            self.cast_skill(id)?;
        }
        for id in FIRST..=LAST {
            let _ = InvalidateRect(Some(self.control(id)), None, true);
        }
        Ok(())
    }
    pub(super) unsafe fn layout_skill_book(&self, area: RECT, dpi: u32) {
        let px = |v: i32| v * dpi as i32 / 96;
        let visible =
            self.world_view && !self.result_view && !self.inventory_view && !self.rental.open;
        for id in [BOOK, CAST_SKILL].into_iter().chain(FIRST..=LAST) {
            let _ = ShowWindow(self.control(id), if visible { SW_SHOW } else { SW_HIDE });
        }
        for id in [SKILLS, SKILL_LABEL] {
            let _ = ShowWindow(
                self.control(id),
                if visible && self.skill_book_open {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
        if !visible {
            return;
        }
        let width = (area.right - px(32)).max(px(80));
        let _ = MoveWindow(
            self.control(BOOK),
            px(16),
            area.bottom - px(90),
            (width - px(124)).max(px(80)),
            px(32),
            true,
        );
        let _ = MoveWindow(
            self.control(SKILL_LABEL),
            px(16),
            area.bottom - px(453),
            width,
            px(24),
            true,
        );
        let _ = MoveWindow(
            self.control(SKILLS),
            px(16),
            area.bottom - px(425),
            width,
            px(170),
            true,
        );
        send(
            self.control(SKILLS),
            LB_SETITEMHEIGHT,
            WPARAM(0),
            LPARAM(px(40) as isize),
        );
        for slot in 0..8 {
            let left = px(16) + width * slot / 8;
            let right = px(16) + width * (slot + 1) / 8;
            let _ = MoveWindow(
                self.control(FIRST + slot as u16),
                left,
                area.bottom - px(155),
                (right - left - px(2)).max(1),
                px(58),
                true,
            );
        }
    }
    pub(super) unsafe fn refresh_skill_art(&mut self) {
        let visible = self.world_view
            && !self.result_view
            && !self.inventory_view
            && !self.rental.open
            && !self.closing;
        let mut ids = Vec::new();
        if visible {
            ids.extend(self.skills.bindings().iter().flatten().copied());
            if self.skill_book_open {
                let list = self.control(SKILLS);
                let top = send(list, LB_GETTOPINDEX, WPARAM(0), LPARAM(0)).0.max(0) as usize;
                let height = send(list, LB_GETITEMHEIGHT, WPARAM(0), LPARAM(0)).0.max(1) as i32;
                let mut rect = RECT::default();
                let _ = GetClientRect(list, &mut rect);
                ids.extend(
                    self.skills
                        .rows()
                        .iter()
                        .skip(top)
                        .take(((rect.bottom.max(0) / height) + 2).min(32) as usize)
                        .map(|r| r.id),
                );
            }
        }
        if self.skill_icons.tick(&ids) {
            for id in [SKILLS].into_iter().chain(FIRST..=LAST) {
                let _ = InvalidateRect(Some(self.control(id)), None, true);
            }
        }
        let selected = self
            .skills
            .selected()
            .and_then(|id| self.skills.rows().iter().find(|r| r.id == id))
            .map(|r| r.label.as_str())
            .unwrap_or("尚未選擇");
        self.text(
            BOOK,
            &format!(
                "{}（{}）　{}",
                if self.skill_book_open {
                    "關閉技能書"
                } else {
                    "技能書"
                },
                self.skills.rows().len(),
                selected
            ),
        );
        self.text(
            SKILL_LABEL,
            "選擇技能後 Ctrl＋F5–F12 綁定；Shift＋快捷鍵清除",
        );
        for id in FIRST..=LAST {
            let _ = EnableWindow(
                self.control(id),
                visible
                    && !self.controller.snapshot().observer
                    && self
                        .controller
                        .snapshot()
                        .world
                        .own_actor()
                        .is_some_and(|a| !a.dead),
            );
        }
    }
    pub(super) unsafe fn draw_skill_control(&self, draw: &DRAWITEMSTRUCT) {
        let book = draw.CtlID == u32::from(SKILLS);
        let selected = draw.itemState.0 & ODS_SELECTED.0 != 0;
        let bg = if selected {
            COLOR_HIGHLIGHT
        } else {
            COLOR_WINDOW
        };
        FillRect(draw.hDC, &draw.rcItem, GetSysColorBrush(bg));
        let (skill, mut label) = if book {
            let Some(row) = self.skills.rows().get(draw.itemID as usize) else {
                return;
            };
            (Some(row.id), row.label.clone())
        } else {
            let slot = (draw.CtlID - u32::from(FIRST)) as usize;
            (self.skills.bindings()[slot], format!("F{}", slot + 5))
        };
        let dpi = GetDpiForWindow(self.parent).max(96) as i32;
        let px = |v: i32| v * dpi / 96;
        let mut area = draw.rcItem;
        if let Some(id) = skill {
            if let Some(icon) = self.skill_icons.icon(id) {
                let background = GetSysColor(bg);
                let pixels = icon.bgra([
                    background as u8,
                    (background >> 8) as u8,
                    (background >> 16) as u8,
                ]);
                let mut info = BITMAPINFO::default();
                info.bmiHeader = BITMAPINFOHEADER {
                    biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                    biWidth: 32,
                    biHeight: -32,
                    biPlanes: 1,
                    biBitCount: 32,
                    biCompression: BI_RGB.0,
                    ..Default::default()
                };
                let x = if book {
                    area.left + px(4)
                } else {
                    (area.left + area.right - px(32)) / 2
                };
                let mode = SetStretchBltMode(draw.hDC, COLORONCOLOR);
                StretchDIBits(
                    draw.hDC,
                    x,
                    area.top + px(4),
                    px(32),
                    px(32),
                    0,
                    0,
                    32,
                    32,
                    Some(pixels.as_ptr().cast()),
                    &info,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
                SetStretchBltMode(draw.hDC, STRETCH_BLT_MODE(mode));
            } else if book {
                label.push_str(if self.skill_icons.error(id).is_some() {
                    "〔圖示未提供〕"
                } else {
                    "〔圖示載入中〕"
                });
            } else {
                label = format!("#{id}\r\n{label}");
            }
        }
        let usable = skill.is_some_and(|id| {
            self.skills
                .choice_id_at(
                    self.controller.transport().snapshot().epoch,
                    &self.controller.snapshot().world,
                    id,
                    self.target,
                    self.ground,
                )
                .is_ok()
        });
        let color = if !usable {
            COLOR_GRAYTEXT
        } else if selected {
            COLOR_HIGHLIGHTTEXT
        } else {
            COLOR_WINDOWTEXT
        };
        let old_font = SelectObject(draw.hDC, self.font.into());
        let old_color = SetTextColor(draw.hDC, COLORREF(GetSysColor(color)));
        let old_mode = SetBkMode(draw.hDC, TRANSPARENT);
        let mut text: Vec<u16> = label.encode_utf16().collect();
        let flags = if book {
            area.left += px(44);
            DT_SINGLELINE | DT_VCENTER | DT_END_ELLIPSIS | DT_NOPREFIX
        } else {
            if skill.and_then(|id| self.skill_icons.icon(id)).is_some() {
                area.top += px(36);
            }
            DT_CENTER | DT_WORDBREAK | DT_NOPREFIX
        };
        DrawTextW(draw.hDC, &mut text, &mut area, flags);
        SetBkMode(draw.hDC, BACKGROUND_MODE(old_mode as u32));
        SetTextColor(draw.hDC, old_color);
        SelectObject(draw.hDC, old_font);
        if draw.itemState.0 & ODS_FOCUS.0 != 0 {
            let _ = DrawFocusRect(draw.hDC, &draw.rcItem);
        }
    }
}
