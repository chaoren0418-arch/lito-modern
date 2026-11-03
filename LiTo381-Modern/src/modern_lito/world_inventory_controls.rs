//! Native child controls: selection is an instance ID, never a stale row index.
use super::*;
use crate::modern_lito::battle_inventory::{Key, Model, Selection};
const FIRST: usize = INVENTORY_FIRST;
pub(super) fn owns(id: usize) -> bool {
    (FIRST..FIRST + 3).contains(&id)
}
#[derive(Default)]
pub(super) struct Controls {
    pub handles: Cell<Option<[HWND; 3]>>,
    pub model: RefCell<Option<Model>>,
    selected: Cell<Option<Key>>,
}
impl Controls {
    pub fn selection(&self) -> Option<Selection> {
        self.model
            .borrow()
            .as_ref()?
            .selection(self.selected.get()?)
    }
    unsafe fn select(&self, state: &State, key: Option<Key>) {
        if self.selected.replace(key) != key {
            // A different item starts at its heading. Ordinary stock/gear
            // updates retain the existing edit selection and scroll position.
            state.details_text.borrow_mut().clear();
            if let Some(edit) = state.details.get() {
                SendMessageW(edit, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(0)));
                let first = SendMessageW(edit, EM_GETFIRSTVISIBLELINE, None, None).0;
                SendMessageW(edit, EM_LINESCROLL, Some(WPARAM(0)), Some(LPARAM(-first)));
            }
        }
    }
    pub(super) unsafe fn select_key(&self, state: &State, key: Key) {
        let index = self.model.borrow().as_ref()
            .and_then(|m| m.rows.iter().position(|row| row.key == key));
        let Some(index) = index else { return; };
        if let Some(handles) = self.handles.get() {
            SendMessageW(handles[0], CB_SETCURSEL, Some(WPARAM(index)), None);
        }
        self.select(state, Some(key));
        self.enable(state);
    }
    pub(super) unsafe fn clear_selection(&self, state: &State) {
        if let Some(handles) = self.handles.get() {
            SendMessageW(handles[0], CB_SETCURSEL, Some(WPARAM(usize::MAX)), None);
        }
        self.select(state, None);
        if let Some(edit) = state.details.get() {
            let _ = SetWindowTextW(edit, w!("請選取目前背包頁面的物品。"));
        }
        self.enable(state);
    }
    pub unsafe fn create(&self, hwnd: HWND, instance: HINSTANCE) -> Result<()> {
        let mut handles = [HWND::default(); 3];
        for (i, label) in [w!(""), w!("裝備"), w!("卸下")].into_iter().enumerate() {
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
        let old = self.model.borrow_mut().take(); // No borrow across native re-entry.
        let same_owner = old
            .as_ref()
            .zip(next.as_ref())
            .is_some_and(|(a, b)| a.same_owner(b));
        let selected = next
            .as_ref()
            .and_then(|m| m.retain_selection(old.as_ref(), self.selected.get()));
        self.select(state, selected);
        let labels_changed = !same_owner
            || old.as_ref().zip(next.as_ref()).is_none_or(|(a, b)| {
                a.rows.len() != b.rows.len()
                    || a.rows
                        .iter()
                        .zip(&b.rows)
                        .any(|(a, b)| (a.key, &a.label) != (b.key, &b.label))
            });
        self.selected.set(None);
        if labels_changed {
            SendMessageW(handles[0], CB_RESETCONTENT, None, None);
            if let Some(m) = &next {
                for (i, row) in m.rows.iter().enumerate() {
                    let label: Vec<u16> = row.label.encode_utf16().chain(Some(0)).collect();
                    ensure!(
                        !row.label.contains('\0') && label.len() <= 1024,
                        "invalid inventory label"
                    );
                    ensure!(
                        SendMessageW(
                            handles[0],
                            CB_ADDSTRING,
                            None,
                            Some(LPARAM(label.as_ptr() as isize))
                        )
                        .0 == i as isize,
                        "inventory combo failed"
                    );
                }
                let index = selected.and_then(|id| m.rows.iter().position(|r| r.key == id));
                SendMessageW(
                    handles[0],
                    CB_SETCURSEL,
                    Some(WPARAM(index.unwrap_or(usize::MAX))),
                    None,
                );
            }
        }
        let visible = state.details_page.get() == Some(DetailsPage::Inventory) && next.is_some();
        self.selected.set(selected);
        *self.model.borrow_mut() = next;
        for h in handles {
            let _ = ShowWindow(h, if visible { SW_SHOW } else { SW_HIDE });
        }
        self.enable(state);
        if let Some(hwnd) = state.hwnd.get() {
            // Only a change in row visibility affects the details viewport.
            if old.is_some() != self.model.borrow().is_some() {
                resize(hwnd, state);
            }
        }
        Ok(())
    }
    unsafe fn enable(&self, state: &State) {
        let Some(h) = self.handles.get() else {
            return;
        };
        let visible = state.details_page.get() == Some(DetailsPage::Inventory);
        let title: Vec<u16> = self
            .selected
            .get()
            .map_or("裝備", Key::primary)
            .encode_utf16()
            .chain(Some(0))
            .collect();
        let _ = SetWindowTextW(h[1], PCWSTR(title.as_ptr()));
        let flags = {
            let model = self.model.borrow();
            [true, false].map(|desired| {
                visible
                    && self.selected.get().is_some_and(|key| {
                        model.as_ref().is_some_and(|m| match key {
                            Key::Source(id) => m.request(id, desired).is_some(),
                            _ => desired && m.local(key).is_some(),
                        })
                    })
            })
        };
        for (h, enabled) in h[1..].iter().zip(flags) {
            let _ = EnableWindow(*h, enabled);
        }
    }
    pub unsafe fn command(&self, state: &State, wparam: WPARAM, lparam: LPARAM) {
        let Some(h) = self.handles.get() else {
            return;
        };
        let Some(i) = (wparam.0 & 0xffff).checked_sub(FIRST).filter(|i| *i < 3) else {
            return;
        };
        if h[i].0.is_null()
            || h[i].0 as isize != lparam.0
            || state.released.get()
            || state.details_page.get() != Some(DetailsPage::Inventory)
        {
            return;
        }
        let notification = wparam.0 >> 16;
        if i == 0 && notification == CBN_SELCHANGE as usize {
            let index = SendMessageW(h[0], CB_GETCURSEL, None, None).0;
            let selected = usize::try_from(index)
                .ok()
                .and_then(|n| self.model.borrow().as_ref()?.rows.get(n).map(|r| r.key));
            self.select(state, selected);
            self.enable(state);
        } else if i > 0
            && notification == BN_CLICKED as usize
            && IsWindowEnabled(h[i]).as_bool()
            && state.action.get().is_none()
        {
            let action = self.selected.get().and_then(|key| {
                let model = self.model.borrow();
                let m = model.as_ref()?;
                let painted = state.painted_battle.borrow();
                let o = &painted.as_ref()?.0.observation;
                match key {
                    Key::Source(id) => m
                        .request(id, i == 1)
                        .filter(|q| q.current(o))
                        .map(Action::CanonicalEquipment),
                    _ if i == 1 => m
                        .local(key)
                        .filter(|q| q.stamp == crate::modern_lito::battle_window::Stamp::of(o))
                        .map(Action::CanonicalInventory),
                    _ => None,
                }
            });
            if let Some(action) = action {
                state.action.set(Some(action));
                self.clear_proofs();
                if let Some(bar) = state.hotbar.borrow_mut().as_mut() { bar.clear_proofs(); }
                self.enable(state);
            }
        }
    }
    pub unsafe fn layout(&self, width: i32, top: i32, dpi: u32) {
        if let Some(handles) = self.handles.get() {
            for (i, (h, (x, w))) in handles
                .into_iter()
                .zip(footer_controls(width, dpi))
                .enumerate()
            {
                let _ = SetWindowPos(
                    h,
                    None,
                    x,
                    top,
                    w,
                    scale(if i == 0 { 280 } else { 28 }, dpi),
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
            }
        }
    }
}
