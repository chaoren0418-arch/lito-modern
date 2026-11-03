//! Native skill-page controls. Selection alone never casts or picks a target.
use super::*;
use crate::modern_lito::battle_skillbook::Model;
const FIRST: usize = SKILLS_FIRST;
pub(super) fn owns(id: usize) -> bool {
    (FIRST..FIRST + 2).contains(&id)
}
#[derive(Default)]
pub(super) struct Controls {
    pub handles: Cell<Option<[HWND; 2]>>,
    pub model: RefCell<Option<Model>>,
    selected: Cell<Option<u32>>,
}
impl Controls {
    pub(super) fn selection(&self) -> Option<u32> { self.selected.get() }
    pub(super) unsafe fn select_id(&self, state: &State, id: u32) {
        let Some(index) = self.model.borrow().as_ref()
            .and_then(|model| model.rows.iter().position(|row| row.id == id)) else { return; };
        let Some(handles) = self.handles.get() else { return; };
        SendMessageW(handles[0], CB_SETCURSEL, Some(WPARAM(index)), None);
        self.selected.set(Some(id));
        self.enable(state);
    }
    pub unsafe fn create(&self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        let mut handles = [HWND::default(); 2];
        for (i, label) in [w!(""), w!("施放")].into_iter().enumerate() {
            handles[i] = CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                if i == 0 { w!("COMBOBOX") } else { w!("BUTTON") },
                label,
                WS_CHILD
                    | WS_TABSTOP
                    | if i == 0 {
                        WS_VSCROLL | WINDOW_STYLE(CBS_DROPDOWNLIST as u32)
                    } else {
                        WINDOW_STYLE::default()
                    },
                0,
                0,
                180,
                if i == 0 { 280 } else { 30 },
                Some(hwnd),
                Some(HMENU((FIRST + i) as *mut _)),
                Some(instance),
                None,
            )?;
        }
        self.handles.set(Some(handles));
        Ok(())
    }
    pub fn clear_proofs(&self) {
        if let Some(m) = self.model.borrow_mut().as_mut() {
            m.clear_proofs();
        }
    }
    pub unsafe fn update(&self, state: &State, next: Option<Model>) -> Result<()> {
        let Some(handles) = self.handles.get() else {
            return Ok(());
        };
        let old = self.model.borrow_mut().take();
        let same_owner = old
            .as_ref()
            .zip(next.as_ref())
            .is_some_and(|(a, b)| a.same_owner(b));
        let selected = self.selected.get().filter(|id| {
            same_owner
                && next
                    .as_ref()
                    .is_some_and(|m| m.rows.iter().any(|r| r.id == *id))
        });
        let labels_changed = !same_owner
            || old.as_ref().zip(next.as_ref()).is_none_or(|(a, b)| {
                a.rows.len() != b.rows.len()
                    || a.rows
                        .iter()
                        .zip(&b.rows)
                        .any(|(a, b)| (a.id, &a.label) != (b.id, &b.label))
            });
        self.selected.set(None);
        if labels_changed {
            SendMessageW(handles[0], CB_RESETCONTENT, None, None);
            if let Some(m) = &next {
                for (i, row) in m.rows.iter().enumerate() {
                    let label: Vec<u16> = row.label.encode_utf16().chain(Some(0)).collect();
                    ensure!(
                        !row.label.contains('\0') && label.len() <= 1024,
                        "invalid skill label"
                    );
                    ensure!(
                        SendMessageW(
                            handles[0],
                            CB_ADDSTRING,
                            None,
                            Some(LPARAM(label.as_ptr() as isize))
                        )
                        .0 == i as isize,
                        "skill combo failed"
                    );
                }
                let index = selected.and_then(|id| m.rows.iter().position(|r| r.id == id));
                SendMessageW(
                    handles[0],
                    CB_SETCURSEL,
                    Some(WPARAM(index.unwrap_or(usize::MAX))),
                    None,
                );
            }
        }
        let visible = state.details_page.get() == Some(DetailsPage::Skills) && next.is_some();
        self.selected.set(selected);
        *self.model.borrow_mut() = next;
        for h in handles {
            let _ = ShowWindow(h, if visible { SW_SHOW } else { SW_HIDE });
        }
        self.enable(state);
        if let Some(hwnd) = state.hwnd.get() {
            if old.is_some() != self.model.borrow().is_some() {
                resize(hwnd, state);
            }
        }
        Ok(())
    }
    unsafe fn enable(&self, state: &State) {
        let Some(h) = self.handles.get() else { return };
        let (text, enabled) = {
            let m = self.model.borrow();
            let row = self
                .selected
                .get()
                .and_then(|id| m.as_ref()?.rows.iter().find(|r| r.id == id));
            (
                row.map_or("施放", |r| r.button),
                state.details_page.get() == Some(DetailsPage::Skills)
                    && row.is_some_and(|r| r.choice.is_some()),
            )
        };
        let text: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let _ = SetWindowTextW(h[1], PCWSTR(text.as_ptr()));
        let _ = EnableWindow(h[1], enabled);
    }
    pub unsafe fn command(&self, state: &State, wparam: WPARAM, lparam: LPARAM) {
        let Some(h) = self.handles.get() else { return };
        let Some(i) = (wparam.0 & 0xffff).checked_sub(FIRST).filter(|i| *i < 2) else {
            return;
        };
        if h[i].0.is_null()
            || h[i].0 as isize != lparam.0
            || state.released.get()
            || state.details_page.get() != Some(DetailsPage::Skills)
        {
            return;
        }
        let notification = wparam.0 >> 16;
        if i == 0 && notification == CBN_SELCHANGE as usize {
            let index = SendMessageW(h[0], CB_GETCURSEL, None, None).0;
            self.selected.set(
                usize::try_from(index)
                    .ok()
                    .and_then(|n| self.model.borrow().as_ref()?.rows.get(n).map(|r| r.id)),
            );
            self.enable(state);
        } else if i == 1
            && notification == BN_CLICKED as usize
            && IsWindowEnabled(h[i]).as_bool()
            && state.action.get().is_none()
        {
            let q = self
                .selected
                .get()
                .and_then(|id| self.model.borrow().as_ref()?.choice(id));
            if let Some(q) = q.filter(|q| {
                state
                    .painted_battle
                    .borrow()
                    .as_ref()
                    .is_some_and(|s| q.matches(&s.0.observation))
            }) {
                state.action.set(Some(Action::CanonicalSkill(q)));
                self.clear_proofs();
                if let Some(bar) = state.hotbar.borrow_mut().as_mut() {
                    bar.clear_proofs();
                }
                self.enable(state);
            }
        }
    }
    pub unsafe fn layout(&self, width: i32, dpi: u32) {
        if let Some(handles) = self.handles.get() {
            let columns = footer_controls(width, dpi);
            let slots = [
                (columns[0].0, columns[1].0 + columns[1].1 - columns[0].0),
                columns[2],
            ];
            for (i, (h, (x, w))) in handles.into_iter().zip(slots).enumerate() {
                let _ = SetWindowPos(
                    h,
                    None,
                    x,
                    scale(4, dpi),
                    w,
                    scale(if i == 0 { 280 } else { 28 }, dpi),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
}
