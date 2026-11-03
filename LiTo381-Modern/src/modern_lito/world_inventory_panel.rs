//! Source-backed inventory grid for the canonical world window. File I/O stays
//! on an owned worker; the HWND only composites already-qualified pixels.
use crate::modern_lito::{
    battle_inventory::{Key, Model},
    battle_window::Stamp,
    inventory_icons::SharedInventoryIcons,
    resources::{SharedArchive, SharedArchiveKind},
};
use anyhow::{ensure, Context, Result};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc, Arc,
    },
    thread,
};

const SOURCE_WIDTH: i32 = 800;
const SOURCE_HEIGHT: i32 = 600;
const NATIVE_WIDTH: i32 = 193;
const NATIVE_HEIGHT: i32 = 282;
const INNER_WIDTH: i32 = 181;
const INNER_HEIGHT: i32 = 271;
const INNER_LEFT: i32 = 6;
const INNER_TOP: i32 = 6;
const MAG_NUM: i32 = 5;
const MAG_DEN: i32 = 4;
pub(super) const LEFT: i32 = SOURCE_WIDTH - NATIVE_WIDTH * MAG_NUM / MAG_DEN;
const TOP: i32 = 0;
const SLOTS_PER_PAGE: usize = 24;

/// Input authority for the inventory pixels actually committed to the HWND.
/// A ready worker reply alone must never make its (possibly new-page) slots clickable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Painted {
    pub(super) viewport: super::Viewport,
    stamp: Stamp,
    page: usize,
    serial: u64,
    keys: [Option<Key>; SLOTS_PER_PAGE],
}
impl Painted {
    pub(super) fn valid(&self, panel: &Panel, model: &Model, viewport: super::Viewport) -> bool {
        panel.ready.is_some()
            && self.viewport == viewport
            && self.stamp == model.stamp
            && self.page == panel.page
            && self.serial == panel.serial
            && self.keys == page_keys(model, panel.page)
    }
    pub(super) fn key(&self, slot: usize) -> Option<Key> {
        self.keys.get(slot).copied().flatten()
    }
    pub(super) fn contains(&self, key: Key) -> bool {
        self.keys.contains(&Some(key))
    }
}

fn page_keys(model: &Model, page: usize) -> [Option<Key>; SLOTS_PER_PAGE] {
    std::array::from_fn(|slot| model.rows.get(page * SLOTS_PER_PAGE + slot).map(|row| row.key))
}
const GRID_LEFT: i32 = 28;
const GRID_TOP: i32 = 27;
const CELL_WIDTH: i32 = 35;
const CELL_HEIGHT: i32 = 35;

#[derive(Clone, Debug)]
pub(super) struct Picture {
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) offset: [u8; 2],
    pub(super) rgba: Vec<u8>,
}

fn load_picture(root: &Path, shard: u8, name: &str,
    expected: (i32, i32, u16)) -> Result<Picture> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::SpriteShard(shard))?;
    let bytes = archive.read_entry(name)?;
    ensure!(
        bytes.len() == 8 + (expected.0 * expected.1 * 2) as usize,
        "shared inventory {name} length mismatch"
    );
    let width = u16::from_le_bytes([bytes[0], bytes[1]]);
    let height = u16::from_le_bytes([bytes[2], bytes[3]]);
    let masked = u16::from_le_bytes([bytes[4], bytes[5]]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    ensure!(
        (width as i32, height as i32, masked) == expected,
        "shared inventory {name} header mismatch"
    );
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pair in bytes[8..].chunks_exact(2) {
        let pixel = u16::from_le_bytes([pair[0], pair[1]]);
        if masked == 1 && pixel == key {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let color = |shift| {
                let v = ((pixel >> shift) & 31u16) as u8;
                (v << 3) | (v >> 2)
            };
            rgba.extend_from_slice(&[color(10), color(5), color(0), 255]);
        }
    }
    Ok(Picture {
        width: width.into(),
        height: height.into(),
        offset: [0, 0],
        rgba,
    })
}
fn load_panel(root: &Path) -> Result<(Picture, Picture)> {
    // The previous 2056 inner image visibly says S.power/M.resist and belongs
    // to magic. The shared generic 2047 frame plus opaque 2046 item grid have
    // no magic labels; the LiTo-specific gold chrome is not in this archive.
    Ok((load_picture(root, 8, "2047.img", (193, 282, 1))?,
        load_picture(root, 7, "2046.img", (181, 271, 0))?))
}

#[derive(Debug)]
struct Request {
    serial: u64,
    graphics: Vec<Option<i64>>,
}
struct Reply {
    serial: u64,
    result: Result<Assets>,
}
struct Assets {
    panel: Arc<Picture>,
    inner: Arc<Picture>,
    icons: Vec<Option<Picture>>,
}

fn worker(
    root: PathBuf,
    cancel: Arc<AtomicBool>,
    requests: mpsc::Receiver<Request>,
    replies: mpsc::Sender<Reply>,
) {
    let loaded = load_panel(&root).map(|(outer, inner)| (Arc::new(outer), Arc::new(inner)));
    for request in requests {
        if cancel.load(Ordering::Acquire) {
            break;
        }
        let result = (|| -> Result<Assets> {
            let (panel, inner) = loaded.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
            // SharedInventoryIcons owns its lazily opened archive handles, so
            // make it mutable only inside this worker. A new reader per page is
            // bounded to 24 icons and avoids any HWND-thread file access.
            let mut reader = SharedInventoryIcons::open_cancellable(&root, &cancel)?;
            let mut pictures = Vec::with_capacity(request.graphics.len());
            for graphic in request.graphics {
                if cancel.load(Ordering::Acquire) {
                    anyhow::bail!("inventory icon load cancelled");
                }
                let picture = graphic
                    .map(|id| {
                        let icon = reader.load(id, &cancel)?;
                        let image = icon.image();
                        Ok::<_, anyhow::Error>(Picture {
                            width: image.width(),
                            height: image.height(),
                            offset: image.raw_offset(),
                            rgba: image.reference_rgba()?,
                        })
                    })
                    .transpose()
                    .with_context(|| format!("shared inventory icon {graphic:?}"))?;
                pictures.push(picture);
            }
            Ok(Assets {
                panel: panel.clone(),
                inner: inner.clone(),
                icons: pictures,
            })
        })();
        if replies
            .send(Reply {
                serial: request.serial,
                result,
            })
            .is_err()
        {
            break;
        }
    }
}

pub(super) struct Panel {
    sender: Option<mpsc::Sender<Request>>,
    receiver: mpsc::Receiver<Reply>,
    thread: Option<thread::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    serial: u64,
    request_active: bool,
    requested: Vec<Option<i64>>,
    page: usize,
    ready: Option<Assets>,
    error: Option<String>,
}
impl Panel {
    pub(super) fn start(root: PathBuf) -> Result<Self> {
        ensure!(
            root.is_absolute(),
            "shared inventory resource root must be absolute"
        );
        let (sender, requests) = mpsc::channel();
        let (replies, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let thread = thread::Builder::new()
            .name("lito-inventory-art".into())
            .spawn(move || worker(root, worker_cancel, requests, replies))?;
        Ok(Self {
            sender: Some(sender),
            receiver,
            thread: Some(thread),
            cancel,
            serial: 0,
            request_active: false,
            requested: Vec::new(),
            page: 0,
            ready: None,
            error: None,
        })
    }
    pub(super) fn page(&self) -> usize {
        self.page
    }
    pub(super) fn pages(&self, model: &Model) -> usize {
        model.rows.len().saturating_sub(1) / SLOTS_PER_PAGE + 1
    }
    pub(super) fn ready(&self) -> bool {
        self.ready.is_some()
    }
    pub(super) fn painted(&self, model: &Model, viewport: super::Viewport) -> Option<Painted> {
        self.ready.as_ref()?;
        Some(Painted {
            viewport,
            stamp: model.stamp,
            page: self.page,
            serial: self.serial,
            keys: page_keys(model, self.page),
        })
    }
    pub(super) fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }
    pub(super) fn update(&mut self, model: Option<&Model>) -> bool {
        let mut changed = false;
        while let Ok(reply) = self.receiver.try_recv() {
            if reply.serial == self.serial {
                match reply.result {
                    Ok(assets) => {
                        self.ready = Some(assets);
                        self.error = None;
                    }
                    Err(error) => {
                        self.ready = None;
                        self.error = Some(format!("{error:#}"));
                    }
                }
                changed = true;
            }
        }
        let Some(model) = model else {
            let had_ready = self.ready.take().is_some();
            if self.request_active || had_ready || !self.requested.is_empty()
                || self.error.is_some() {
                self.serial = self.serial.wrapping_add(1);
                changed = true;
            }
            self.requested.clear();
            self.request_active = false;
            self.page = 0;
            self.error = None;
            return changed;
        };
        let last = model.rows.len().saturating_sub(1) / SLOTS_PER_PAGE;
        self.page = self.page.min(last);
        let graphics: Vec<_> = model
            .rows
            .iter()
            .skip(self.page * SLOTS_PER_PAGE)
            .take(SLOTS_PER_PAGE)
            .map(|r| r.graphic)
            .collect();
        if graphics != self.requested || !self.request_active {
            self.serial = self.serial.wrapping_add(1);
            self.request_active = true;
            self.requested = graphics.clone();
            self.ready = None;
            self.error = None;
            if self.sender.as_ref().is_none_or(|tx| {
                tx.send(Request {
                    serial: self.serial,
                    graphics,
                })
                .is_err()
            }) {
                self.error = Some("inventory art worker unavailable".into());
            }
            changed = true;
        }
        changed
    }
    pub(super) fn turn(&mut self, model: &Model, delta: i32) -> bool {
        let last = model.rows.len().saturating_sub(1) / SLOTS_PER_PAGE;
        let next = (self.page as i64 + i64::from(delta)).clamp(0, last as i64) as usize;
        if next == self.page {
            return false;
        }
        self.page = next;
        self.requested.clear();
        self.request_active = false;
        self.ready = None;
        self.update(Some(model));
        true
    }
    pub(super) fn focus(
        &mut self,
        model: &Model,
        key: crate::modern_lito::battle_inventory::Key,
    ) -> bool {
        let Some(index) = model.rows.iter().position(|row| row.key == key) else {
            return false;
        };
        let next = index / SLOTS_PER_PAGE;
        if next == self.page {
            return false;
        }
        self.page = next;
        self.requested.clear();
        self.request_active = false;
        self.ready = None;
        self.update(Some(model));
        true
    }
    pub(super) fn render(
        &self,
        base: &[u8],
        model: &Model,
        selected: Option<crate::modern_lito::battle_inventory::Key>,
    ) -> Option<Vec<u8>> {
        let assets = self.ready.as_ref()?;
        if base.len() != (SOURCE_WIDTH * SOURCE_HEIGHT * 4) as usize {
            return None;
        }
        let mut output = base.to_vec();
        blit(
            &mut output,
            &assets.inner,
            LEFT + INNER_LEFT * MAG_NUM / MAG_DEN,
            TOP + INNER_TOP * MAG_NUM / MAG_DEN,
            INNER_WIDTH * MAG_NUM / MAG_DEN,
            INNER_HEIGHT * MAG_NUM / MAG_DEN,
            (0, 0, SOURCE_WIDTH, SOURCE_HEIGHT),
        );
        // The opaque inner grid may cover native (182,19), where the masked
        // outer frame draws its close ornament. Paint the transparent outer
        // last so chrome, close and fine grid lines remain visible.
        blit(
            &mut output,
            &assets.panel,
            LEFT,
            TOP,
            NATIVE_WIDTH * MAG_NUM / MAG_DEN,
            NATIVE_HEIGHT * MAG_NUM / MAG_DEN,
            (0, 0, SOURCE_WIDTH, SOURCE_HEIGHT),
        );
        for (slot, icon) in assets.icons.iter().enumerate() {
            let col = slot % 4;
            let row = slot / 4;
            let x = LEFT + ((GRID_LEFT + col as i32 * CELL_WIDTH) * MAG_NUM / MAG_DEN);
            let y = TOP + ((GRID_TOP + row as i32 * CELL_HEIGHT) * MAG_NUM / MAG_DEN);
            let source_row = model.rows.get(self.page * SLOTS_PER_PAGE + slot);
            if let Some(icon) = icon {
                let w = (icon.width as i32 * MAG_NUM / MAG_DEN).max(1);
                let h = (icon.height as i32 * MAG_NUM / MAG_DEN).max(1);
                blit(
                    &mut output,
                    icon,
                    x + i32::from(icon.offset[0]) * MAG_NUM / MAG_DEN,
                    y + i32::from(icon.offset[1]) * MAG_NUM / MAG_DEN,
                    w,
                    h,
                    (x, y, x + 43, y + 43),
                );
            }
            if source_row
                .is_some_and(|r| r.equip.is_none() && r.unequip.is_none() && r.local.is_none())
            {
                dim(&mut output, x, y, 43, 43);
            }
            if source_row.is_some_and(|r| Some(r.key) == selected) {
                border(&mut output, x, y, 43, 43, [0, 220, 255]);
            }
        }
        Some(output)
    }
}
impl Drop for Panel {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.sender.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

pub(super) fn blit(
    output: &mut [u8],
    picture: &Picture,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    clip: (i32, i32, i32, i32),
) {
    for dy in 0..height {
        let py = y + dy;
        if !(clip.1..clip.3).contains(&py) || !(0..SOURCE_HEIGHT).contains(&py) {
            continue;
        }
        let sy = (dy as usize * picture.height / height as usize).min(picture.height - 1);
        for dx in 0..width {
            let px = x + dx;
            if !(clip.0..clip.2).contains(&px) || !(0..SOURCE_WIDTH).contains(&px) {
                continue;
            }
            let sx = (dx as usize * picture.width / width as usize).min(picture.width - 1);
            let src = (sy * picture.width + sx) * 4;
            let dst = (py as usize * SOURCE_WIDTH as usize + px as usize) * 4;
            let alpha = u32::from(picture.rgba[src + 3]);
            for (target, source) in [(0, 2), (1, 1), (2, 0)] {
                output[dst + target] = ((u32::from(picture.rgba[src + source]) * alpha
                    + u32::from(output[dst + target]) * (255 - alpha)
                    + 127)
                    / 255) as u8;
            }
        }
    }
}
fn border(output: &mut [u8], x: i32, y: i32, width: i32, height: i32, bgr: [u8; 3]) {
    for dy in 0..height {
        for dx in 0..width {
            if dx != 0 && dy != 0 && dx != width - 1 && dy != height - 1 {
                continue;
            }
            let px = x + dx;
            let py = y + dy;
            if !(0..SOURCE_WIDTH).contains(&px) || !(0..SOURCE_HEIGHT).contains(&py) {
                continue;
            }
            let at = (py as usize * SOURCE_WIDTH as usize + px as usize) * 4;
            output[at..at + 3].copy_from_slice(&bgr);
        }
    }
}
pub(super) fn dim(output: &mut [u8], x: i32, y: i32, width: i32, height: i32) {
    for dy in 1..height - 1 {
        for dx in 1..width - 1 {
            let px = x + dx;
            let py = y + dy;
            if !(0..SOURCE_WIDTH).contains(&px) || !(0..SOURCE_HEIGHT).contains(&py) {
                continue;
            }
            let at = (py as usize * SOURCE_WIDTH as usize + px as usize) * 4;
            for component in &mut output[at..at + 3] {
                *component = (*component as u16 * 2 / 5) as u8;
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Hit {
    Slot(usize),
    Previous,
    Next,
    Close,
    Body,
}
pub(super) fn hit(view: super::Viewport, point: super::PixelPoint) -> Option<Hit> {
    if view.width <= 0 || view.height <= 0 {
        return None;
    }
    let sx = ((i64::from(point.x) - i64::from(view.x)) * i64::from(SOURCE_WIDTH)
        / i64::from(view.width)) as i32;
    let sy = ((i64::from(point.y) - i64::from(view.y)) * i64::from(SOURCE_HEIGHT)
        / i64::from(view.height)) as i32;
    if sx < LEFT || sy < TOP {
        return None;
    }
    let local_x = (sx - LEFT) * MAG_DEN / MAG_NUM;
    let local_y = (sy - TOP) * MAG_DEN / MAG_NUM;
    if !(0..NATIVE_WIDTH).contains(&local_x) || !(0..NATIVE_HEIGHT).contains(&local_y) {
        return None;
    }
    if (174..191).contains(&local_x) && (8..30).contains(&local_y) {
        return Some(Hit::Close);
    }
    // Slots start at floor(native * 5/4) in render. For integer source pixels,
    // invert those boundaries with floor((source * 4 + 3)/5). Frame/Close/Pager
    // keep their existing mapping; their artwork uses a different size ratio.
    let grid_x = ((sx - LEFT) * MAG_DEN + MAG_DEN - 1) / MAG_NUM;
    let grid_y = ((sy - TOP) * MAG_DEN + MAG_DEN - 1) / MAG_NUM;
    let col = (grid_x - GRID_LEFT) / CELL_WIDTH;
    let row = (grid_y - GRID_TOP) / CELL_HEIGHT;
    if grid_x >= GRID_LEFT && grid_y >= GRID_TOP && (0..4).contains(&col) && (0..6).contains(&row)
    {
        return Some(Hit::Slot(row as usize * 4 + col as usize));
    }
    if local_y >= 243 {
        if local_x < 80 {
            return Some(Hit::Previous);
        }
        if local_x >= 100 {
            return Some(Hit::Next);
        }
    }
    Some(Hit::Body)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn painted_slot_boundary_render_selects_second_item() {
        use crate::modern_lito::{application_protocol::{CharacterKey, SelectionContext},
            battle_inventory::Row};
        use sha2::{Digest, Sha256};
        let stamp = Stamp { context: SelectionContext { key: CharacterKey {
            source_id: 7, object_id: 3333 }, epoch: 1 }, channel: 0,
            world: [1, 2, 3], entity: 1, life: 1, revision: 1 };
        let model = Model { stamp, rows: (0..24).map(|slot| Row {
            key: Key::Source(100 + slot), graphic: None, label: format!("item {slot}"),
            equip: None, unequip: None, local: None,
        }).collect() };
        let (_, receiver) = mpsc::channel();
        let panel = Panel { sender: None, receiver, thread: None,
            cancel: Arc::new(AtomicBool::new(false)), serial: 7,
            request_active: true, requested: vec![None; 24], page: 0,
            ready: Some(Assets {
                panel: Arc::new(Picture { width: 193, height: 282, offset: [0, 0],
                    rgba: vec![0; 193 * 282 * 4] }),
                inner: Arc::new(Picture { width: 181, height: 271, offset: [0, 0],
                    rgba: vec![0; 181 * 271 * 4] }), icons: vec![None; 24],
            }), error: None };
        let view = super::super::Viewport { x: 0, y: 0, width: 800, height: 600 };
        let point = super::super::PixelPoint { x: 637, y: 34 };
        let output = panel.render(&vec![0; 800 * 600 * 4], &model, Some(Key::Source(101))).unwrap();
        let at = (point.y as usize * 800 + point.x as usize) * 4;
        let painted = panel.painted(&model, view).unwrap();
        let actual = hit(view, point);
        println!("{}", serde_json::json!({"case":"inventory_render_border", "fixture_only":true,
            "viewport":[0,0,800,600], "point":[point.x,point.y], "selected_key":101,
            "painted_slot":1, "painted_key":format!("{:?}",painted.key(1)),
            "paint_valid":painted.valid(&panel,&model,view), "border_bgr":&output[at..at+3],
            "output_sha256":format!("{:X}",Sha256::digest(&output)),
            "expected_hit":"Some(Slot(1))", "actual_hit":format!("{actual:?}")}));
        assert_eq!(&output[at..at+3], &[0,220,255], "point must be on the rendered slot-1 border");
        assert!(painted.valid(&panel, &model, view));
        assert_eq!(painted.key(1), Some(Key::Source(101)));
        assert_eq!(actual, Some(Hit::Slot(1)), "rendered border must select its own item slot");
    }
    #[test]
    fn painted_slot_boundary_grid_preserves_control_thresholds() {
        use super::super::{PixelPoint, Viewport};
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let mut cases = Vec::<(String, PixelPoint, Option<Hit>)>::new();
        // Grid boundaries are the source-pixel starts used by render, including
        // the full logical cell gutters beyond the 43px icon/border clipping.
        for row in 0..6 {
            for cut in 0..=4 {
                let x = LEFT + (28 + cut * 35) * 5 / 4;
                let y = (27 + row * 35 + 12) * 5 / 4;
                let before = if cut == 0 { Hit::Body } else { Hit::Slot((row * 4 + cut - 1) as usize) };
                let after = if cut == 4 { Hit::Body } else { Hit::Slot((row * 4 + cut) as usize) };
                cases.push((format!("grid_x_row{row}_cut{cut}_before"), PixelPoint { x:x-1,y }, Some(before)));
                cases.push((format!("grid_x_row{row}_cut{cut}_at"), PixelPoint { x,y }, Some(after)));
            }
        }
        for col in 0..4 {
            for cut in 0..=6 {
                let x = LEFT + (28 + col * 35 + 12) * 5 / 4;
                let y = (27 + cut * 35) * 5 / 4;
                let before = if cut == 0 { Hit::Body } else { Hit::Slot(((cut - 1) * 4 + col) as usize) };
                let after = if cut == 6 { Hit::Body } else { Hit::Slot((cut * 4 + col) as usize) };
                cases.push((format!("grid_y_col{col}_cut{cut}_before"), PixelPoint { x,y:y-1 }, Some(before)));
                cases.push((format!("grid_y_col{col}_cut{cut}_at"), PixelPoint { x,y }, Some(after)));
            }
        }
        // Frame/Close/Pager retain their existing inverse and logical extent.
        // These literal source-pixel cuts deliberately differ from grid cuts.
        for (label,x,y,before,after,axis_x) in [
            ("close_left",218,22,Some(Hit::Body),Some(Hit::Close),true),
            ("close_right",239,22,Some(Hit::Close),Some(Hit::Body),true),
            ("close_top",227,10,Some(Hit::Body),Some(Hit::Close),false),
            ("close_bottom",227,38,Some(Hit::Close),Some(Hit::Body),false),
            ("pager_previous_end",100,317,Some(Hit::Previous),Some(Hit::Body),true),
            ("pager_next_start",125,317,Some(Hit::Body),Some(Hit::Next),true),
            ("pager_top",75,304,Some(Hit::Body),Some(Hit::Previous),false),
            ("frame_left",0,125,None,Some(Hit::Body),true),
            ("frame_right",242,125,Some(Hit::Body),None,true),
            ("frame_top",0,0,None,Some(Hit::Body),false),
            ("frame_bottom",194,353,Some(Hit::Next),None,false),
        ] {
            cases.push((format!("{label}_before"),PixelPoint { x:LEFT+x-i32::from(axis_x),y:y-i32::from(!axis_x) },before));
            cases.push((format!("{label}_at"),PixelPoint { x:LEFT+x,y },after));
        }
        let mut mismatches = 0;
        let observations: Vec<_> = cases.iter().map(|(label,point,expected)| {
            let actual = hit(view,*point);
            mismatches += usize::from(actual != *expected);
            serde_json::json!({"label":label,"point":[point.x,point.y],
                "expected_hit":format!("{expected:?}"),"actual_hit":format!("{actual:?}")})
        }).collect();
        println!("{}",serde_json::json!({"case":"inventory_grid_control_thresholds","fixture_only":true,
            "viewport":[0,0,800,600],"grid_boundary_points":116,"preserved_control_points":22,
            "point_count":cases.len(),"mismatches":mismatches,"observations":observations}));
        assert_eq!(mismatches,0,"painted grid boundaries and existing control thresholds must agree");
    }
    #[test]
    fn closed_inventory_retires_one_request_without_idle_serial_churn_or_stale_reply() {
        use crate::modern_lito::{application_protocol::{CharacterKey, SelectionContext},
            battle_inventory::Row};
        let stamp = Stamp { context: SelectionContext { key: CharacterKey {
            source_id: 7, object_id: 3333 }, epoch: 1 }, channel: 0,
            world: [1, 2, 3], entity: 1, life: 1, revision: 1 };
        let model = Model { stamp, rows: vec![Row { key: Key::Source(100),
            graphic: Some(80), label: "source #100".into(),
            equip: None, unequip: None, local: None }] };
        let (request_tx, request_rx) = mpsc::channel();
        let (reply_tx, reply_rx) = mpsc::channel();
        let mut panel = Panel { sender: Some(request_tx), receiver: reply_rx,
            thread: None, cancel: Arc::new(AtomicBool::new(false)), serial: 0,
            request_active: false, requested: Vec::new(), page: 0,
            ready: None, error: None };
        assert!(panel.update(Some(&model)));
        let obsolete = request_rx.try_recv().unwrap();
        assert_eq!(obsolete.serial, panel.serial);
        assert!(panel.update(None));
        let closed_serial = panel.serial;
        for _ in 0..100 {
            assert!(!panel.update(None));
            assert_eq!(panel.serial, closed_serial,
                "a closed unchanged page must not churn request serial or redraw");
        }
        reply_tx.send(Reply { serial: obsolete.serial, result: Ok(Assets {
            panel: Arc::new(Picture { width: 193, height: 282, offset: [0,0],
                rgba: vec![0; 193 * 282 * 4] }),
            inner: Arc::new(Picture { width: 181, height: 271, offset: [0,0],
                rgba: vec![0; 181 * 271 * 4] }), icons: vec![] }) }).unwrap();
        assert!(!panel.update(None), "closed page must ignore an obsolete worker reply");
        assert!(!panel.ready());
        assert!(panel.update(Some(&model)));
        let reopened = request_rx.try_recv().unwrap();
        assert_ne!(reopened.serial, obsolete.serial);
        assert!(!panel.update(Some(&model)));
        assert!(request_rx.try_recv().is_err(), "unchanged open page must not reread art");
    }
    #[test]
    fn page_worker_ready_is_not_click_authority_until_new_pixels_commit() {
        use crate::modern_lito::application_protocol::{CharacterKey, SelectionContext};
        use crate::modern_lito::battle_inventory::Row;
        let stamp = Stamp {
            context: SelectionContext { key: CharacterKey { source_id: 7, object_id: 3333 }, epoch: 1 },
            channel: 0,
            world: [1, 2, 3],
            entity: 1,
            life: 1,
            revision: 1,
        };
        let mut model = Model {
            stamp,
            rows: (0..25).map(|n| Row {
                key: Key::Source(n + 100),
                graphic: Some(80),
                label: format!("item {n}"),
                equip: None,
                unequip: None,
                local: None,
            }).collect(),
        };
        let (_, receiver) = mpsc::channel();
        let mut panel = Panel {
            sender: None,
            receiver,
            thread: None,
            cancel: Arc::new(AtomicBool::new(false)),
            serial: 7,
            request_active: true,
            requested: vec![Some(80); 24],
            page: 0,
            ready: Some(Assets {
                panel: Arc::new(Picture { width: 193, height: 282, offset: [0, 0], rgba: vec![0; 193 * 282 * 4] }),
                inner: Arc::new(Picture { width: 181, height: 271, offset: [0, 0], rgba: vec![0; 181 * 271 * 4] }),
                icons: vec![],
            }),
            error: None,
        };
        let viewport = super::super::Viewport { x: 0, y: 0, width: 800, height: 600 };
        assert!(panel.render(&vec![0; 800 * 600 * 4], &model, None).is_some());
        let old_paint = panel.painted(&model, viewport).unwrap(); // successful paint/BitBlt publication
        assert_eq!(old_paint.key(0), Some(Key::Source(100)));
        assert!(old_paint.valid(&panel, &model, viewport));

        // A worker may qualify new-page assets before WM_PAINT. The screen is
        // still page zero, and neither old nor speculative new hit can pass.
        panel.page = 1;
        panel.serial = 8;
        panel.ready = Some(Assets {
            panel: Arc::new(Picture { width: 193, height: 282, offset: [0, 0], rgba: vec![0; 193 * 282 * 4] }),
            inner: Arc::new(Picture { width: 181, height: 271, offset: [0, 0], rgba: vec![0; 181 * 271 * 4] }),
            icons: vec![],
        });
        assert!(!old_paint.valid(&panel, &model, viewport));
        let new_paint = panel.painted(&model, viewport).unwrap(); // only publish after BitBlt
        assert!(new_paint.valid(&panel, &model, viewport));
        assert_eq!(new_paint.key(0), Some(Key::Source(124)));

        // Same page and icon ID, but different server item identity.
        model.rows[24].key = Key::Source(999);
        assert!(!new_paint.valid(&panel, &model, viewport));
        let repainted = panel.painted(&model, viewport).unwrap();
        assert_eq!(repainted.key(0), Some(Key::Source(999)));
        assert!(repainted.valid(&panel, &model, viewport));
        assert!(!repainted.valid(&panel, &model, super::super::Viewport { x: 1, ..viewport }));
    }
    #[test]
    fn page_slots_project_through_the_same_world_viewport() {
        for view in [
            super::super::Viewport {
                x: 0,
                y: 0,
                width: 800,
                height: 600,
            },
            super::super::Viewport {
                x: 100,
                y: 190,
                width: 400,
                height: 300,
            },
            super::super::Viewport {
                x: 48,
                y: 0,
                width: 1200,
                height: 900,
            },
        ] {
            let source_x = LEFT + (GRID_LEFT + 2 * CELL_WIDTH + 15) * MAG_NUM / MAG_DEN;
            let source_y = (GRID_TOP + 3 * CELL_HEIGHT + 14) * MAG_NUM / MAG_DEN;
            let point = super::super::PixelPoint {
                x: view.x + source_x * view.width / SOURCE_WIDTH,
                y: view.y + source_y * view.height / SOURCE_HEIGHT,
            };
            assert_eq!(hit(view, point), Some(Hit::Slot(14)));
            let close = super::super::PixelPoint {
                x: view.x + (LEFT + 182 * MAG_NUM / MAG_DEN) * view.width / SOURCE_WIDTH,
                y: view.y + (18 * MAG_NUM / MAG_DEN) * view.height / SOURCE_HEIGHT,
            };
            assert_eq!(hit(view, close), Some(Hit::Close));
            for (native_x, expected) in [(60, Hit::Previous), (155, Hit::Next)] {
                let page = super::super::PixelPoint {
                    x: view.x + (LEFT + native_x * MAG_NUM / MAG_DEN) * view.width / SOURCE_WIDTH,
                    y: view.y + (253 * MAG_NUM / MAG_DEN) * view.height / SOURCE_HEIGHT,
                };
                assert_eq!(hit(view, page), Some(expected));
            }
            let last_row = super::super::PixelPoint {
                x: view.x + (LEFT + 60 * MAG_NUM / MAG_DEN) * view.width / SOURCE_WIDTH,
                y: view.y + (232 * MAG_NUM / MAG_DEN) * view.height / SOURCE_HEIGHT,
            };
            assert_eq!(hit(view, last_row), Some(Hit::Slot(20)),
                "last item row must not become a pager hit");
            assert_eq!(
                hit(
                    view,
                    super::super::PixelPoint {
                        x: view.x + 1,
                        y: view.y + 1
                    }
                ),
                None
            );
        }
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; actual Sprite08 2047.img, Sprite07 2046.img and list.ico icons"]
    fn actual_shared_inventory_frame_and_icons_are_qualified() {
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let (outer, inner) = load_panel(&root).unwrap();
        assert_eq!((outer.width, outer.height), (193, 282));
        assert_eq!((inner.width, inner.height), (181, 271));
        let mut icons = SharedInventoryIcons::open(&root).unwrap();
        for graphic in [7, 77, 78, 80, 81, 130, 1529] {
            let icon = icons.load(graphic, &AtomicBool::new(false)).unwrap();
            assert!(icon.image().width() > 0 && icon.image().height() > 0);
        }
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; background dual-frame/list.ico icon worker, CPU overlay and exact grid selection"]
    fn real_worker_composes_confirmed_inventory_slot_without_ui_thread_file_io() {
        use crate::modern_lito::{
            application_protocol::{CharacterKey, SelectionContext},
            battle_inventory::{Key, Row},
            battle_window::Stamp,
        };
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let stamp = Stamp {
            context: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1,
            },
            channel: 0,
            world: [1, 2, 3],
            entity: 1,
            life: 1,
            revision: 1,
        };
        let key = Key::Source(1001);
        let model = Model {
            stamp,
            rows: vec![Row {
                key,
                graphic: Some(80),
                label: "server-confirmed potion".into(),
                equip: None,
                unequip: None,
                local: None,
            }],
        };
        let mut panel = Panel::start(root).unwrap();
        assert!(panel.update(Some(&model)));
        for _ in 0..200 {
            if panel.update(Some(&model)) && panel.ready() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            panel.ready(),
            "worker did not qualify actual shared panel + icon: {:?}",
            panel.error()
        );
        let base = vec![0u8; (SOURCE_WIDTH * SOURCE_HEIGHT * 4) as usize];
        let output = panel.render(&base, &model, Some(key)).unwrap();
        let x = LEFT + GRID_LEFT * MAG_NUM / MAG_DEN;
        let y = GRID_TOP * MAG_NUM / MAG_DEN;
        let at = (y as usize * SOURCE_WIDTH as usize + x as usize) * 4;
        assert_eq!(
            &output[at..at + 3],
            &[0, 220, 255],
            "selected exact slot border"
        );
        // Real shared chrome must remain visible over the opaque inner panel.
        // The exact white close glyph pixel belongs to outer 2047, not to
        // inner 2046; this fails if composition order is reversed.
        let assets = panel.ready.as_ref().unwrap();
        let outer_at = (19 * NATIVE_WIDTH as usize + 182) * 4;
        let close = &assets.panel.rgba[outer_at..outer_at + 4];
        assert_eq!(close[3], 255);
        let close_x = LEFT + 228; // 228*193/241 = native x182
        let close_y = 24;         // 24*282/352 = native y19
        assert_eq!(228 * NATIVE_WIDTH / (NATIVE_WIDTH * MAG_NUM / MAG_DEN), 182);
        assert_eq!(24 * NATIVE_HEIGHT / (NATIVE_HEIGHT * MAG_NUM / MAG_DEN), 19);
        let close_at = (close_y as usize * SOURCE_WIDTH as usize + close_x as usize) * 4;
        assert_eq!(&output[close_at..close_at + 3], &[close[2], close[1], close[0]],
            "source X must survive final CPU composition");
        assert_eq!(hit(super::super::Viewport { x: 0, y: 0, width: 800, height: 600 },
            super::super::PixelPoint { x: close_x, y: close_y }), Some(Hit::Close));
        assert_ne!(
            output, base,
            "actual shared panel and icon must affect the same battle buffer"
        );
        panel.update(None);
        assert!(
            !panel.ready(),
            "return must retire obsolete panel pixels and icon ownership"
        );
    }
}
