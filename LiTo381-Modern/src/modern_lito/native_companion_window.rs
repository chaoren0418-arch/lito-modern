//! Native companion panel, sharing the live GAME controller.
use super::*;
use crate::modern_lito::{native_companions as pets, native_item_actions};
use std::time::Instant;
pub const TOGGLE: u16 = 340;
const ROSTER: u16 = 341;
const TALK: u16 = 342;
const ACTION: u16 = 343;
const APPLY: u16 = 344;
const INFO: u16 = 345;
const NAME: u16 = 346;
const RENAME: u16 = 347;
const TARGET: u16 = 348;
const ATTACK: u16 = 349;
const ITEM: u16 = 350;
const QUANTITY: u16 = 351;
const GIVE: u16 = 352;
const USE: u16 = 353;
const NOTICE: u16 = 354;
const IDS: [u16; 15] = [
    TOGGLE, ROSTER, TALK, ACTION, APPLY, INFO, NAME, RENAME, TARGET, ATTACK, ITEM, QUANTITY, GIVE,
    USE, NOTICE,
];
#[derive(Default)]
pub struct View {
    pub open: bool,
    pub error: Option<String>,
    message_floor: u64,
    system_floor: u64,
    roster: Vec<(pets::Choice, String)>,
    targets: Vec<(u32, u64, u64, String)>,
    items: Vec<(native_item_actions::Choice, String)>,
}
unsafe fn fill(h: HWND, labels: impl Iterator<Item = String>) {
    send(h, CB_RESETCONTENT, WPARAM(0), LPARAM(0));
    for text in labels {
        let text = wide(&text);
        send(h, CB_ADDSTRING, WPARAM(0), LPARAM(text.as_ptr() as isize));
    }
    // Require an explicit selection after every authority change.
    send(h, CB_SETCURSEL, WPARAM(usize::MAX), LPARAM(0));
}
impl Ui {
    pub(super) unsafe fn create_companion_controls(&mut self) -> Result<()> {
        let instance = HINSTANCE(GetModuleHandleW(None)?.0);
        let combo = CBS_DROPDOWNLIST as u32 | WS_VSCROLL.0;
        for (id, class, label, style) in [
            (TOGGLE, w!("BUTTON"), "寵物／召喚物", 0),
            (ROSTER, w!("COMBOBOX"), "", combo),
            (TALK, w!("BUTTON"), "開啟／更新選單", 0),
            (ACTION, w!("COMBOBOX"), "", combo),
            (APPLY, w!("BUTTON"), "送出指令", 0),
            (
                INFO,
                w!("EDIT"),
                "",
                ES_MULTILINE as u32 | ES_READONLY as u32 | WS_VSCROLL.0 | WS_BORDER.0,
            ),
            (NAME, w!("EDIT"), "", ES_AUTOHSCROLL as u32 | WS_BORDER.0),
            (RENAME, w!("BUTTON"), "送出新名稱", 0),
            (TARGET, w!("COMBOBOX"), "", combo),
            (ATTACK, w!("BUTTON"), "送出攻擊目標", 0),
            (ITEM, w!("COMBOBOX"), "", combo),
            (QUANTITY, w!("EDIT"), "1", ES_NUMBER as u32 | WS_BORDER.0),
            (GIVE, w!("BUTTON"), "交付／餵食", 0),
            (USE, w!("BUTTON"), "使用項圈／用品", 0),
            (NOTICE, w!("STATIC"), "", 0),
        ] {
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
        fill(
            self.control(ACTION),
            pets::Action::ALL.iter().map(|a| a.label().to_owned()),
        );
        send(self.control(NAME), EM_SETLIMITTEXT, WPARAM(27), LPARAM(0));
        Ok(())
    }
    pub(super) unsafe fn layout_companions(&self) {
        let show = self.world_view
            && !self.result_view
            && !self.inventory_view
            && !self.rental.open
            && !self.closing;
        let mut area = RECT::default();
        let _ = GetClientRect(self.parent, &mut area);
        let dpi = GetDpiForWindow(self.parent).max(96) as i32;
        let px = |n: i32| n * dpi / 96;
        for id in IDS {
            let _ = ShowWindow(
                self.control(id),
                if show && (id == TOGGLE || self.companions.open) {
                    SW_SHOW
                } else {
                    SW_HIDE
                },
            );
        }
        let _ = MoveWindow(
            self.control(TOGGLE),
            if self.companions.open {
                px(20)
            } else {
                (area.right - px(160)).max(px(20))
            },
            if self.companions.open {
                area.bottom - px(45)
            } else {
                px(20)
            },
            px(140),
            px(32),
            true,
        );
        self.text(
            TOGGLE,
            if self.companions.open {
                "回到場景"
            } else {
                "寵物／召喚物"
            },
        );
        let _ = EnableWindow(
            self.control(TOGGLE),
            pets::available(self.controller.snapshot()),
        );
        if !show || !self.companions.open {
            return;
        }
        for &(id, h) in &self.controls {
            if !IDS.contains(&id) {
                let _ = ShowWindow(h, SW_HIDE);
            }
        }
        let width = (area.right * 96 / dpi - 40).max(440);
        for (id, x, y, w, h) in [
            (ROSTER, 20, 20, width - 175, 200),
            (TALK, width - 140, 20, 160, 32),
            (ACTION, 20, 65, width - 175, 200),
            (APPLY, width - 140, 65, 160, 32),
            (INFO, 20, 108, width, 235),
            (NAME, 20, 355, width - 175, 28),
            (RENAME, width - 140, 355, 160, 32),
            (TARGET, 20, 400, width - 175, 200),
            (ATTACK, width - 140, 400, 160, 32),
            (ITEM, 20, 445, width - 90, 200),
            (QUANTITY, width - 60, 445, 80, 28),
            (GIVE, 20, 490, 160, 32),
            (USE, 195, 490, 190, 32),
            (NOTICE, 20, 535, width, 60),
        ] {
            let _ = MoveWindow(self.control(id), px(x), px(y), px(w), px(h), true);
        }
    }
    pub(super) unsafe fn refresh_companions(&mut self) {
        let s = self.controller.snapshot();
        let epoch = self.controller.transport().snapshot().epoch;
        if !pets::available(s) || self.closing || self.inventory_view || self.rental.open {
            if self.companions.open {
                self.companions.open = false;
                self.layout(self.parent, false);
            }
            self.companions.roster.clear();
            self.companions.targets.clear();
            self.companions.items.clear();
            self.layout_companions();
            return;
        }
        let rows: Vec<_> = s
            .world
            .actors
            .values()
            .filter_map(|a| {
                pets::Choice::capture(s, epoch, a.id)
                    .ok()
                    .map(|c| (c, a.name.clone()))
            })
            .collect();
        if rows != self.companions.roster {
            fill(
                self.control(ROSTER),
                rows.iter().map(|(c, n)| format!("{n}（{}）", c.id)),
            );
            self.companions.roster = rows;
        }
        let own = s.world.own_actor().unwrap();
        let targets: Vec<_> = s
            .world
            .actors
            .values()
            .filter(|a| {
                a.id != own.id
                    && !a.dead
                    && a.draw_state.visible(false)
                    && crate::modern_lito::native_skills::in_native_screen(
                        [own.x, own.y],
                        [a.x, a.y],
                    )
            })
            .map(|a| (a.id, a.incarnation, a.target_revision, a.name.clone()))
            .collect();
        if targets != self.companions.targets {
            fill(
                self.control(TARGET),
                targets.iter().map(|a| format!("{}（{}）", a.3, a.0)),
            );
            self.companions.targets = targets;
        }
        let items: Vec<_> = s
            .inventory
            .items
            .values()
            .filter_map(|i| {
                native_item_actions::Choice::capture(s, i.id).ok().map(|c| {
                    (
                        c,
                        crate::modern_lito::native_inventory_names::label(
                            i,
                            self.inventory_names.catalog(),
                        )
                        .text,
                    )
                })
            })
            .collect();
        if items != self.companions.items {
            fill(
                self.control(ITEM),
                items.iter().map(|(c, n)| format!("{n} ×{}", c.item.count)),
            );
            self.companions.items = items;
        }
        let session = &s.companions;
        let menu = session
            .menu
            .as_ref()
            .map(pets::Menu::text)
            .unwrap_or_else(|| "選取自己的同伴，開啟選單以取得資料。".into());
        self.text(INFO, &format!("{menu}\r\n\r\n{}", pets::summary(s)));
        let mut notice = self
            .companions
            .error
            .as_deref()
            .unwrap_or(&session.feedback)
            .to_owned();
        if s.world.message_count > self.companions.message_floor {
            if let Some((code, args)) = &s.world.last_message {
                let label = match code {
                    319 => "同伴攜帶容量不足",
                    353 => "此地圖不能召回同伴",
                    942 => "對象背包容量不足",
                    263 => "背包空間不足",
                    82 => "負重已達上限",
                    299 => "缺少所需材料",
                    _ => "請依伺服器訊息處理",
                };
                notice.push_str(&format!(
                    "\r\n伺服器訊息 {code}：{label} {}",
                    args.join(" ")
                ));
            }
        }
        if s.world.system_message_count > self.companions.system_floor {
            if let Some((_, text)) = &s.world.last_system_message {
                notice.push_str(&format!("\r\n{text}"));
            }
        }
        self.text(NOTICE, &notice);
        let _ = EnableWindow(self.control(TALK), session.can_request());
        let _ = EnableWindow(
            self.control(APPLY),
            session.menu.is_some() && session.can_request(),
        );
        let _ = EnableWindow(self.control(ATTACK), session.can_target(Instant::now()));
        let _ = EnableWindow(self.control(RENAME), session.can_rename(Instant::now()));
        self.layout_companions();
    }
    unsafe fn companion_choice(&self) -> Result<pets::Choice> {
        let i = send(self.control(ROSTER), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as usize;
        let c = self
            .companions
            .roster
            .get(i)
            .context("請先選取同伴")?
            .0
            .clone();
        c.validate(
            self.controller.snapshot(),
            self.controller.transport().snapshot().epoch,
        )?;
        Ok(c)
    }
    pub(super) unsafe fn companion_action(&mut self, id: u16) -> Result<bool> {
        if !IDS.contains(&id) {
            return Ok(false);
        }
        self.companions.error = None;
        if id == TOGGLE {
            self.companions.open = !self.companions.open;
            self.companions.message_floor = self.controller.snapshot().world.message_count;
            self.companions.system_floor = self.controller.snapshot().world.system_message_count;
            self.held.clear();
            self.held_attack.clear();
            self.item_cursor = None;
            self.layout(self.parent, false);
            self.refresh_companions();
            let _ = InvalidateRect(Some(self.parent), None, true);
            return Ok(true);
        }
        ensure!(self.companions.open && !self.closing, "請先開啟同伴控制");
        match id {
            TALK => {
                let c = self.companion_choice()?;
                self.controller.companion_talk(c)?;
            }
            APPLY => {
                let c = self.companion_choice()?;
                let i = send(self.control(ACTION), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as usize;
                let a = *pets::Action::ALL.get(i).context("請選取同伴指令")?;
                if a == pets::Action::Dismiss
                    && MessageBoxW(
                        Some(self.parent),
                        w!("確定要解放選取的同伴嗎？寵物解放可能永久移除項圈及寵物持有關係。"),
                        w!("解放同伴"),
                        MB_YESNO | MB_ICONQUESTION | MB_DEFBUTTON2,
                    ) != IDYES
                {
                    return Ok(true);
                }
                self.controller.companion_action(&c, a)?;
            }
            RENAME => {
                let name = self.read(NAME, 27)?;
                self.controller.companion_rename(&name)?;
                self.text(NAME, "");
            }
            ATTACK => {
                let i = send(self.control(TARGET), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as usize;
                let t = self.companions.targets.get(i).context("請選取攻擊對象")?;
                let a = self
                    .controller
                    .snapshot()
                    .world
                    .actors
                    .get(&t.0)
                    .context("目標已離開")?;
                ensure!(
                    (a.incarnation, a.target_revision) == (t.1, t.2),
                    "攻擊對象已變更"
                );
                self.controller.companion_target(t.0)?;
            }
            GIVE | USE => {
                let i = send(self.control(ITEM), CB_GETCURSEL, WPARAM(0), LPARAM(0)).0 as usize;
                let item = self
                    .companions
                    .items
                    .get(i)
                    .context("請選取背包物品")?
                    .0
                    .clone();
                if id == GIVE {
                    let quantity = self.read(QUANTITY, 10)?.parse()?;
                    let c = self.companion_choice()?;
                    self.controller.companion_give(&c, &item, quantity)?;
                } else {
                    self.controller
                        .item_action(item, native_item_actions::Action::Use)?;
                }
            }
            _ => {}
        }
        self.refresh_companions();
        Ok(true)
    }
}
