//! Read-only character/equipped-item surface from the same canonical world
//! source. Item icons live in a clearly separate ledger below the figure:
//! the protocol has no authority for mapping instance IDs to native slots.
use super::{inventory_panel::{blit, Picture}, PixelPoint, Viewport};
use crate::modern_lito::{battle_follow::Observation, battle_inventory::{Key, Model as Inventory},
    battle_window::Stamp, character_data::{CharacterData, ItemKind}, inventory_icons::SharedInventoryIcons,
    resources::{SharedArchive, SharedArchiveKind}};
use anyhow::{ensure, Result};
use std::{path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc}, thread};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const FRAME_WIDTH: i32 = 193;
const FRAME_HEIGHT: i32 = 324;
const INNER_WIDTH: i32 = 181;
const INNER_HEIGHT: i32 = 309;
const NUM: i32 = 5;
const DEN: i32 = 4;
pub(super) const RIGHT: i32 = FRAME_WIDTH * NUM / DEN;
const SLOTS: usize = 8;
const LEDGER_TOP: i32 = 405;
const ICON_TOP: i32 = 444;
const ICON_LEFT: i32 = 31;
const ICON_STEP_X: i32 = 49;
const ICON_STEP_Y: i32 = 47;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Row { pub(super) id: u64, pub(super) graphic: Option<i64>, pub(super) label: String }
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Model { pub(super) stamp: Stamp, pub(super) sex: u8, pub(super) rows: Vec<Row> }
impl Model {
    pub(super) fn from_observation(o: &Observation, inventory: &Inventory) -> Option<Self> {
        let stamp = Stamp::of(o);
        if inventory.stamp != stamp { return None; }
        let source = o.personal().source()?;
        if source.context != stamp.context || source.character.sex > 1 { return None; }
        let rows = confirmed_rows(source, inventory, |id| {
            o.personal().equipped(i64::try_from(id).ok()?)
        });
        Some(Self { stamp, sex: source.character.sex, rows })
    }
    pub(super) fn same_owner(&self, other: &Self) -> bool {
        let mut a = self.stamp;
        let mut b = other.stamp;
        a.revision = 0;
        b.revision = 0;
        a == b && self.sex == other.sex
    }
    pub(super) fn contains(&self, id: u64) -> bool { self.rows.iter().any(|r| r.id == id) }
}
fn confirmed_rows(source: &CharacterData, inventory: &Inventory,
    equipped: impl Fn(u64) -> Option<bool>) -> Vec<Row> {
    source.items.iter().filter_map(|item| {
        if !matches!(item.kind, ItemKind::Weapon | ItemKind::Armor)
            || equipped(item.object_id) != Some(true) { return None; }
        let row = inventory.rows.iter().find(|row| row.key == Key::Source(item.object_id)
            && row.graphic == Some(item.inventory_graphic))?;
        Some(Row { id: item.object_id, graphic: (item.inventory_graphic >= 0)
            .then_some(item.inventory_graphic), label: row.label.clone() })
    }).collect()
}
fn page_ids(model: &Model, page: usize) -> [Option<u64>; SLOTS] {
    std::array::from_fn(|slot| model.rows.get(page * SLOTS + slot).map(|r| r.id))
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Painted {
    pub(super) viewport: Viewport,
    stamp: Stamp,
    page: usize,
    serial: u64,
    ids: [Option<u64>; SLOTS],
}
impl Painted {
    pub(super) fn valid(self, panel: &Panel, model: &Model, viewport: Viewport) -> bool {
        panel.ready.is_some() && self.viewport == viewport && self.stamp == model.stamp
            && self.page == panel.page && self.serial == panel.serial
            && self.ids == page_ids(model, panel.page)
    }
    pub(super) fn id(self, slot: usize) -> Option<u64> { self.ids.get(slot).copied().flatten() }
    pub(super) fn text_choice(self, panel: &Panel, model: &Model, viewport: Viewport,
        painted_stamp: Stamp, pending_stamp: Stamp, id: u64) -> bool {
        self.valid(panel, model, viewport) && model.stamp == painted_stamp
            && model.stamp == pending_stamp && model.contains(id)
    }
}

fn load_picture(root: &Path, shard: u8, name: &str,
    expected: (i32, i32, u16)) -> Result<Picture> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::SpriteShard(shard))?;
    let bytes = archive.read_entry(name)?;
    ensure!(bytes.len() == 8 + (expected.0 * expected.1 * 2) as usize,
        "character {name} length mismatch");
    let width = u16::from_le_bytes([bytes[0], bytes[1]]);
    let height = u16::from_le_bytes([bytes[2], bytes[3]]);
    let masked = u16::from_le_bytes([bytes[4], bytes[5]]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    ensure!((width as i32, height as i32, masked) == expected,
        "character {name} header mismatch");
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pair in bytes[8..].chunks_exact(2) {
        let pixel = u16::from_le_bytes([pair[0], pair[1]]);
        if masked == 1 && pixel == key { rgba.extend_from_slice(&[0, 0, 0, 0]); }
        else {
            let color = |shift| { let v = ((pixel >> shift) & 31u16) as u8; (v << 3) | (v >> 2) };
            rgba.extend_from_slice(&[color(10), color(5), color(0), 255]);
        }
    }
    Ok(Picture { width: width.into(), height: height.into(), offset: [0, 0], rgba })
}
struct Request { serial: u64, sex: u8, graphics: [Option<i64>; SLOTS] }
struct Reply { serial: u64, result: Result<Assets> }
struct Assets { inner: Arc<Picture>, outer: Arc<Picture>, icons: Vec<Option<Picture>> }
fn worker(root: PathBuf, cancel: Arc<AtomicBool>, rx: mpsc::Receiver<Request>, tx: mpsc::Sender<Reply>) {
    let male = load_picture(&root, 14, "2458.img", (181, 309, 0)).map(Arc::new);
    let female = load_picture(&root, 15, "2459.img", (181, 309, 0)).map(Arc::new);
    let outer = load_picture(&root, 7, "2460.img", (193, 324, 1)).map(Arc::new);
    for request in rx {
        if cancel.load(Ordering::Acquire) { break; }
        let result = (|| -> Result<Assets> {
            let inner = if request.sex == 0 { &male } else { &female }
                .as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?.clone();
            let outer = outer.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?.clone();
            let mut reader = SharedInventoryIcons::open_cancellable(&root, &cancel)?;
            let mut icons = Vec::with_capacity(SLOTS);
            for graphic in request.graphics {
                if cancel.load(Ordering::Acquire) { anyhow::bail!("equipped art cancelled"); }
                // A source may have an unknown graphic. The item stays in the
                // exact-ID ledger/text, but no guessed icon is substituted.
                let picture = graphic.and_then(|id| reader.load(id, &cancel).ok())
                    .and_then(|icon| {
                        let image = icon.image();
                        Some(Picture { width: image.width(), height: image.height(),
                            offset: image.raw_offset(), rgba: image.reference_rgba().ok()? })
                    });
                icons.push(picture);
            }
            Ok(Assets { inner, outer, icons })
        })();
        if tx.send(Reply { serial: request.serial, result }).is_err() { break; }
    }
}

pub(super) struct Panel {
    sender: Option<mpsc::Sender<Request>>,
    receiver: mpsc::Receiver<Reply>,
    thread: Option<thread::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    serial: u64,
    request_active: bool,
    requested: Option<(u8, [Option<i64>; SLOTS])>,
    page: usize,
    ready: Option<Assets>,
    error: Option<String>,
}
impl Panel {
    pub(super) fn start(root: PathBuf) -> Result<Self> {
        ensure!(root.is_absolute(), "shared character root must be absolute");
        let (sender, rx) = mpsc::channel();
        let (tx, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let thread = thread::Builder::new().name("lito-character-art".into())
            .spawn(move || worker(root, worker_cancel, rx, tx))?;
        Ok(Self { sender: Some(sender), receiver, thread: Some(thread), cancel,
            serial: 0, request_active: false, requested: None, page: 0,
            ready: None, error: None })
    }
    pub(super) fn ready(&self) -> bool { self.ready.is_some() }
    pub(super) fn error(&self) -> Option<&str> { self.error.as_deref() }
    pub(super) fn page(&self) -> usize { self.page }
    pub(super) fn pages(&self, model: &Model) -> usize {
        model.rows.len().saturating_sub(1) / SLOTS + 1
    }
    pub(super) fn visible(&self, model: &Model, id: u64) -> bool {
        page_ids(model, self.page).contains(&Some(id))
    }
    pub(super) fn first_visible(&self, model: &Model) -> Option<u64> {
        page_ids(model, self.page)[0]
    }
    pub(super) fn update(&mut self, model: Option<&Model>) -> bool {
        let mut changed = false;
        while let Ok(reply) = self.receiver.try_recv() {
            if reply.serial == self.serial {
                match reply.result {
                    Ok(assets) => { self.ready = Some(assets); self.error = None; }
                    Err(error) => { self.ready = None; self.error = Some(format!("{error:#}")); }
                }
                changed = true;
            }
        }
        let Some(model) = model else {
            if self.request_active || self.ready.is_some() {
                self.serial = self.serial.wrapping_add(1);
                self.request_active = false;
                self.requested = None;
                self.ready = None;
                self.page = 0;
                self.error = None;
                changed = true;
            }
            return changed;
        };
        self.page = self.page.min(model.rows.len().saturating_sub(1) / SLOTS);
        let graphics = std::array::from_fn(|slot| model.rows.get(self.page * SLOTS + slot)
            .and_then(|row| row.graphic));
        let requested = (model.sex, graphics);
        if !self.request_active || self.requested != Some(requested) {
            self.serial = self.serial.wrapping_add(1);
            self.request_active = true;
            self.requested = Some(requested);
            self.ready = None;
            self.error = None;
            if self.sender.as_ref().is_none_or(|tx| tx.send(Request { serial: self.serial,
                sex: model.sex, graphics }).is_err()) {
                self.error = Some("character art worker unavailable".into());
            }
            changed = true;
        }
        changed
    }
    pub(super) fn turn(&mut self, model: &Model, delta: i32) -> bool {
        let last = model.rows.len().saturating_sub(1) / SLOTS;
        let next = (self.page as i64 + i64::from(delta)).clamp(0, last as i64) as usize;
        if next == self.page { return false; }
        self.page = next;
        self.request_active = false;
        self.ready = None;
        self.update(Some(model));
        true
    }
    pub(super) fn focus(&mut self, model: &Model, id: u64) -> bool {
        let Some(index) = model.rows.iter().position(|row| row.id == id) else { return false; };
        let next = index / SLOTS;
        if next == self.page { return false; }
        self.page = next;
        self.request_active = false;
        self.ready = None;
        self.update(Some(model));
        true
    }
    pub(super) fn painted(&self, model: &Model, viewport: Viewport) -> Option<Painted> {
        self.ready.as_ref()?;
        Some(Painted { viewport, stamp: model.stamp, page: self.page,
            serial: self.serial, ids: page_ids(model, self.page) })
    }
    pub(super) fn render(&self, base: &[u8], model: &Model, selected: Option<u64>) -> Option<Vec<u8>> {
        let assets = self.ready.as_ref()?;
        if base.len() != (WIDTH * HEIGHT * 4) as usize { return None; }
        let mut output = base.to_vec();
        blit(&mut output, &assets.inner, 6 * NUM / DEN, 6 * NUM / DEN,
            INNER_WIDTH * NUM / DEN, INNER_HEIGHT * NUM / DEN, (0, 0, WIDTH, HEIGHT));
        blit(&mut output, &assets.outer, 0, 0,
            FRAME_WIDTH * NUM / DEN, FRAME_HEIGHT * NUM / DEN, (0, 0, WIDTH, HEIGHT));
        // Plain dark ledger, deliberately outside the native equipment slots.
        for y in LEDGER_TOP..580 {
            for x in 0..(FRAME_WIDTH * NUM / DEN) {
                let at = (y as usize * WIDTH as usize + x as usize) * 4;
                output[at..at + 3].copy_from_slice(&[24, 24, 24]);
            }
        }
        for (slot, icon) in assets.icons.iter().enumerate() {
            let Some(row) = model.rows.get(self.page * SLOTS + slot) else { continue; };
            let x = ICON_LEFT + (slot % 4) as i32 * ICON_STEP_X;
            let y = ICON_TOP + (slot / 4) as i32 * ICON_STEP_Y;
            if let Some(icon) = icon {
                blit(&mut output, icon, x + i32::from(icon.offset[0]) * NUM / DEN,
                    y + i32::from(icon.offset[1]) * NUM / DEN,
                    (icon.width as i32 * NUM / DEN).max(1),
                    (icon.height as i32 * NUM / DEN).max(1), (x, y, x + 42, y + 42));
            }
            if selected == Some(row.id) {
                for dy in 0..42 { for dx in 0..42 {
                    if dx != 0 && dy != 0 && dx != 41 && dy != 41 { continue; }
                    let at = ((y + dy) as usize * WIDTH as usize + (x + dx) as usize) * 4;
                    output[at..at + 3].copy_from_slice(&[0, 220, 255]);
                }}
            }
        }
        Some(output)
    }
}
impl Drop for Panel {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.sender.take();
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Hit { Slot(usize), Previous, Next, Close, Body }
pub(super) fn hit(view: Viewport, point: PixelPoint) -> Option<Hit> {
    if view.width <= 0 || view.height <= 0 { return None; }
    let x = (((i64::from(point.x) - i64::from(view.x)) * i64::from(WIDTH)) / i64::from(view.width)) as i32;
    let y = (((i64::from(point.y) - i64::from(view.y)) * i64::from(HEIGHT)) / i64::from(view.height)) as i32;
    if !(0..FRAME_WIDTH * NUM / DEN).contains(&x) || !(0..580).contains(&y) { return None; }
    let native_x = x * DEN / NUM;
    let native_y = y * DEN / NUM;
    if (174..191).contains(&native_x) && (8..30).contains(&native_y) { return Some(Hit::Close); }
    if (ICON_TOP..ICON_TOP + 2 * ICON_STEP_Y).contains(&y) {
        let col = (x - ICON_LEFT) / ICON_STEP_X;
        let row = (y - ICON_TOP) / ICON_STEP_Y;
        if x >= ICON_LEFT && (0..4).contains(&col) && (0..2).contains(&row)
            && x - ICON_LEFT - col * ICON_STEP_X < 42
            && y - ICON_TOP - row * ICON_STEP_Y < 42 {
            return Some(Hit::Slot(row as usize * 4 + col as usize));
        }
    }
    if (548..580).contains(&y) {
        if x < 75 { return Some(Hit::Previous); }
        if x >= 165 { return Some(Hit::Next); }
    }
    Some(Hit::Body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::application_protocol::{CharacterKey, SelectionContext};
    fn model() -> Model {
        let stamp = Stamp { context: SelectionContext { key: CharacterKey {
            source_id: 7, object_id: 3333 }, epoch: 1 }, channel: 0,
            world: [1, 2, 3], entity: 1, life: 1, revision: 1 };
        Model { stamp, sex: 0, rows: (0..9).map(|n| Row {
            id: 100 + n, graphic: Some(80), label: format!("equipped {n}"),
        }).collect() }
    }
    fn picture(width: usize, height: usize) -> Arc<Picture> {
        Arc::new(Picture { width, height, offset: [0, 0], rgba: vec![0; width * height * 4] })
    }
    fn panel() -> Panel {
        let (_, receiver) = mpsc::channel();
        Panel { sender: None, receiver, thread: None, cancel: Arc::new(AtomicBool::new(false)),
            serial: 7, request_active: true, requested: Some((0, [Some(80); 8])), page: 0,
            ready: Some(Assets { inner: picture(181,309), outer: picture(193,324),
                icons: vec![None; 8] }), error: None }
    }
    #[test]
    fn source_instance_and_current_equipped_flag_both_gate_ledger_rows() {
        use crate::modern_lito::{battle_inventory::Row as InventoryRow, equipment};
        let m = model();
        let mut source = (*equipment::tests::sample_snapshot(m.stamp.context).source).clone();
        source.items.first_mut().expect("source fixture item").kind = ItemKind::Weapon;
        let first = source.items.first().unwrap();
        let inventory = Inventory { stamp: m.stamp, rows: source.items.iter().map(|item| InventoryRow {
            key: Key::Source(item.object_id), graphic: Some(item.inventory_graphic),
            label: format!("source {}", item.object_id), equip: None, unequip: None, local: None,
        }).collect() };
        let rows = confirmed_rows(&source, &inventory, |id| Some(id == first.object_id));
        assert_eq!(rows.iter().map(|r| r.id).collect::<Vec<_>>(), vec![first.object_id]);
        assert_eq!(rows[0].graphic, Some(first.inventory_graphic));
        assert!(confirmed_rows(&source, &inventory, |_| None).is_empty());
        let mut wrong_graphic = inventory;
        wrong_graphic.rows[0].graphic = None;
        assert!(confirmed_rows(&source, &wrong_graphic,
            |id| Some(id == first.object_id)).is_empty(),
            "same source ID with mismatched icon cannot enter the visual ledger");
    }
    #[test]
    fn exact_source_identity_page_and_life_revoke_uncommitted_hits() {
        let mut model = model();
        let mut panel = panel();
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let first = panel.painted(&model, view).unwrap();
        assert_eq!(first.id(0), Some(100));
        assert!(first.valid(&panel, &model, view));
        model.rows.swap(0, 1);
        assert!(!first.valid(&panel, &model, view));
        let reordered = panel.painted(&model, view).unwrap();
        assert_eq!(reordered.id(0), Some(101));
        assert!(reordered.valid(&panel, &model, view));
        panel.page = 1;
        panel.serial += 1;
        assert!(!reordered.valid(&panel, &model, view));
        let next = panel.painted(&model, view).unwrap();
        assert_eq!(next.id(0), Some(108));
        assert!(!panel.visible(&model, 100));
        model.stamp.life += 1;
        assert!(!next.valid(&panel, &model, view));
        assert!(!next.valid(&panel, &model, Viewport { x: 1, ..view }));
        assert!(!next.text_choice(&panel, &model, view, next.stamp, next.stamp, 108));
    }
    #[test]
    fn separate_ledger_hits_are_not_native_equipment_slot_guesses() {
        for view in [Viewport { x: 0, y: 0, width: 800, height: 600 },
            Viewport { x: 100, y: 190, width: 400, height: 300 },
            Viewport { x: 48, y: 0, width: 1200, height: 900 }] {
            let project = |x, y| PixelPoint { x: view.x + x * view.width / WIDTH,
                y: view.y + y * view.height / HEIGHT };
            assert_eq!(hit(view, project(ICON_LEFT + ICON_STEP_X + 12, ICON_TOP + 12)),
                Some(Hit::Slot(1)));
            assert_eq!(hit(view, project(ICON_LEFT + 12, ICON_TOP + ICON_STEP_Y + 12)),
                Some(Hit::Slot(4)));
            assert_eq!(hit(view, project(160, 110)), Some(Hit::Body),
                "painted native silhouette slots are not guessed item slots");
            assert_eq!(hit(view, project(228, 24)), Some(Hit::Close));
            assert_eq!(hit(view, project(50, 560)), Some(Hit::Previous));
        assert_eq!(hit(view, project(190, 560)), Some(Hit::Next));
        }
    }
    #[test]
    fn missing_graphic_keeps_exact_id_and_visible_text_choice() {
        let mut model = model();
        model.rows[0].graphic = None;
        model.rows[0].label = "來源已裝備 · #100 · 圖示未提供".into();
        let panel = panel();
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let paint = panel.painted(&model, view).unwrap();
        assert_eq!(paint.id(0), Some(100));
        assert!(paint.valid(&panel, &model, view));
        assert!(model.rows[0].label.contains("#100"));
        assert!(model.rows[0].label.contains("圖示未提供"));
        assert!(paint.text_choice(&panel, &model, view, model.stamp, model.stamp, 100));
        assert!(!paint.text_choice(&panel, &model, view, model.stamp, model.stamp, 999));
        let mut stale = model.stamp;
        stale.life += 1;
        assert!(!paint.text_choice(&panel, &model, view, model.stamp, stale, 100));
        assert_eq!(hit(view, PixelPoint { x: ICON_LEFT + 12, y: ICON_TOP + 12 }),
            Some(Hit::Slot(0)));
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; real 2458/2459 silhouettes, 2460 frame and list.ico icon"]
    fn real_shared_character_assets_keep_close_and_equipped_icon_separate() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut panel = Panel::start(root).unwrap();
        let model = model();
        panel.update(Some(&model));
        for _ in 0..1000 {
            panel.update(Some(&model));
            if panel.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(panel.ready(), "character worker: {:?}", panel.error());
        let assets = panel.ready.as_ref().unwrap();
        assert_eq!((assets.inner.width, assets.inner.height), (181,309));
        assert_eq!((assets.outer.width, assets.outer.height), (193,324));
        let alpha = |x: usize, y: usize| assets.outer.rgba[(y * 193 + x) * 4 + 3];
        for (x,y) in [(101,260),(160,260),(130,254),(130,268)] {
            assert_eq!(alpha(x,y), 255, "real AC box has source border at ({x},{y})");
        }
        assert_eq!(alpha(130,260), 0, "AC value belongs inside transparent native box");
        assert_eq!((assets.icons[0].as_ref().unwrap().width,
            assets.icons[0].as_ref().unwrap().height), (16,24));
        let base = vec![0; (WIDTH * HEIGHT * 4) as usize];
        let output = panel.render(&base, &model, Some(100)).unwrap();
        let x_at = (24usize * WIDTH as usize + 228) * 4;
        let source_x_at = (19usize * FRAME_WIDTH as usize + 182) * 4;
        let x = &assets.outer.rgba[source_x_at..source_x_at+4];
        assert_eq!(x[3], 255);
        assert_eq!(&output[x_at..x_at+3], &[x[2], x[1], x[0]],
            "final visible close glyph is from the real outer resource");
        let selected_at = (ICON_TOP as usize * WIDTH as usize + ICON_LEFT as usize) * 4;
        assert_eq!(&output[selected_at..selected_at+3], &[0,220,255]);
        let mut unresolved = model.clone();
        unresolved.rows[0].graphic = Some(i64::MAX);
        unresolved.rows[0].label = "來源已裝備 · #100 · 共用圖示無法解碼".into();
        panel.update(Some(&unresolved));
        for _ in 0..1000 {
            panel.update(Some(&unresolved));
            if panel.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(panel.ready(), "bad icon must not hide the whole character panel: {:?}", panel.error());
        assert!(panel.ready.as_ref().unwrap().icons[0].is_none());
        assert_eq!(panel.painted(&unresolved, Viewport { x: 0, y: 0, width: 800, height: 600 })
            .unwrap().id(0), Some(100));
        assert!(unresolved.rows[0].label.contains("#100"));
        let male = panel.ready.as_ref().unwrap().inner.rgba.clone();
        let mut female = model.clone();
        female.sex = 1;
        panel.update(Some(&female));
        for _ in 0..1000 {
            panel.update(Some(&female));
            if panel.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(panel.ready(), "female shared silhouette: {:?}", panel.error());
        assert_ne!(panel.ready.as_ref().unwrap().inner.rgba, male,
            "source sex selects a distinct actual shared figure");
        panel.update(None);
        assert!(!panel.ready());
    }
}
