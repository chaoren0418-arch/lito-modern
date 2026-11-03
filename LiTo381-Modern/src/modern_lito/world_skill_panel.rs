//! Shared-resource magic grid. Only the eight locally implemented, learned
//! skills have icon slots; the native combo retains every learned skill.
//! Worker replies supply pixels, never click authority: only a committed paint
//! can select an exact skill identity.
use super::{hotbar::{spell_graphic, spell_icons}, inventory_panel::{blit, dim, Picture}, PixelPoint, Viewport};
use crate::modern_lito::{battle_skillbook::{Choice, Model}, battle_window::Stamp,
    inventory_icons::SharedInventoryIcons, resources::{SharedArchive, SharedArchiveKind}};
use anyhow::{ensure, Context, Result};
use std::{path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc}, thread};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const FRAME_WIDTH: i32 = 193;
const FRAME_HEIGHT: i32 = 282;
const NUM: i32 = 5;
const DEN: i32 = 4;
pub(super) const LEFT: i32 = WIDTH - FRAME_WIDTH * NUM / DEN;
const TOP: i32 = 0;
const GRID_LEFT: i32 = 27;
const GRID_TOP: i32 = 34;
const CELL_WIDTH: i32 = 35;
const CELL_HEIGHT: i32 = 35;
const SUPPORTED: [u32; 8] = [1, 4, 5, 29, 43, 54, 70, 77];

fn rows(model: &Model) -> [Option<(u32, Option<Choice>)>; 8] {
    std::array::from_fn(|slot| model.rows.iter().find(|row| row.id == SUPPORTED[slot])
        .map(|row| (row.id, row.choice)))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Painted {
    pub(super) viewport: Viewport,
    stamp: Stamp,
    serial: u64,
    rows: [Option<(u32, Option<Choice>)>; 8],
}
impl Painted {
    pub(super) fn valid(self, panel: &Panel, model: &Model, view: Viewport) -> bool {
        panel.ready.is_some() && self.viewport == view && self.stamp == model.stamp
            && self.serial == panel.serial && self.rows == rows(model)
    }
    pub(super) fn id(self, slot: usize) -> Option<u32> {
        self.rows.get(slot).copied().flatten().map(|(id, _)| id)
    }
    pub(super) fn contains(self, id: u32) -> bool {
        self.rows.iter().flatten().any(|(row_id, _)| *row_id == id)
    }
}

fn load_frame(root: &Path) -> Result<Picture> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::SpriteShard(9))?;
    let bytes = archive.read_entry("2057.img")?;
    ensure!(bytes.len() == 8 + (FRAME_WIDTH * FRAME_HEIGHT * 2) as usize,
        "shared magic 2057.img length mismatch");
    let width = u16::from_le_bytes([bytes[0], bytes[1]]);
    let height = u16::from_le_bytes([bytes[2], bytes[3]]);
    let masked = u16::from_le_bytes([bytes[4], bytes[5]]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    ensure!((width, height, masked) == (193, 282, 1),
        "shared magic 2057.img header mismatch");
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pair in bytes[8..].chunks_exact(2) {
        let pixel = u16::from_le_bytes([pair[0], pair[1]]);
        if pixel == key {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let color = |shift| {
                let v = ((pixel >> shift) & 31u16) as u8;
                (v << 3) | (v >> 2)
            };
            rgba.extend_from_slice(&[color(10), color(5), color(0), 255]);
        }
    }
    Ok(Picture { width: width.into(), height: height.into(), offset: [0, 0], rgba })
}

struct Request { serial: u64, ids: [Option<u32>; 8] }
struct Reply { serial: u64, result: Result<Assets> }
struct Assets { frame: Arc<Picture>, icons: Vec<Option<Picture>> }

fn worker(root: PathBuf, cancel: Arc<AtomicBool>, rx: mpsc::Receiver<Request>, tx: mpsc::Sender<Reply>) {
    let frame = load_frame(&root).map(Arc::new);
    let index = spell_icons(&root);
    for request in rx {
        if cancel.load(Ordering::Acquire) { break; }
        let result = (|| -> Result<Assets> {
            let frame = frame.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?.clone();
            let index = index.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
            let mut reader = SharedInventoryIcons::open_cancellable(&root, &cancel)?;
            let mut icons = Vec::with_capacity(8);
            for id in request.ids {
                if cancel.load(Ordering::Acquire) { anyhow::bail!("skill art load cancelled"); }
                let icon = id.map(|id| {
                    let graphic = spell_graphic(index, id)?;
                    let icon = reader.load(graphic, &cancel)?;
                    let image = icon.image();
                    Ok::<_, anyhow::Error>(Picture { width: image.width(), height: image.height(),
                        offset: image.raw_offset(), rgba: image.reference_rgba()? })
                }).transpose().with_context(|| format!("shared magic icon {id:?}"))?;
                icons.push(icon);
            }
            Ok(Assets { frame, icons })
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
    requested: [Option<u32>; 8],
    ready: Option<Assets>,
    error: Option<String>,
}
impl Panel {
    pub(super) fn start(root: PathBuf) -> Result<Self> {
        ensure!(root.is_absolute(), "shared magic resource root must be absolute");
        let (sender, rx) = mpsc::channel();
        let (tx, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let thread = thread::Builder::new().name("lito-magic-art".into())
            .spawn(move || worker(root, worker_cancel, rx, tx))?;
        Ok(Self { sender: Some(sender), receiver, thread: Some(thread), cancel,
            serial: 0, request_active: false, requested: [None; 8], ready: None, error: None })
    }
    pub(super) fn ready(&self) -> bool { self.ready.is_some() }
    pub(super) fn error(&self) -> Option<&str> { self.error.as_deref() }
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
                self.ready = None;
                self.requested = [None; 8];
                self.error = None;
                changed = true;
            }
            return changed;
        };
        let requested = rows(model).map(|row| row.map(|(id, _)| id));
        if !self.request_active || self.requested != requested {
            self.serial = self.serial.wrapping_add(1);
            self.request_active = true;
            self.requested = requested;
            self.ready = None;
            self.error = None;
            if self.sender.as_ref().is_none_or(|tx| tx.send(Request { serial: self.serial, ids: requested }).is_err()) {
                self.error = Some("magic art worker unavailable".into());
            }
            changed = true;
        }
        changed
    }
    pub(super) fn painted(&self, model: &Model, viewport: Viewport) -> Option<Painted> {
        self.ready.as_ref()?;
        Some(Painted { viewport, stamp: model.stamp, serial: self.serial, rows: rows(model) })
    }
    pub(super) fn render(&self, base: &[u8], model: &Model, selected: Option<u32>) -> Option<Vec<u8>> {
        let assets = self.ready.as_ref()?;
        if base.len() != (WIDTH * HEIGHT * 4) as usize { return None; }
        let mut output = base.to_vec();
        blit(&mut output, &assets.frame, LEFT, TOP, FRAME_WIDTH * NUM / DEN,
            FRAME_HEIGHT * NUM / DEN, (0, 0, WIDTH, HEIGHT));
        let painted_rows = rows(model);
        for (slot, icon) in assets.icons.iter().enumerate() {
            let Some((id, choice)) = painted_rows[slot] else { continue; };
            let x = LEFT + (GRID_LEFT + (slot % 4) as i32 * CELL_WIDTH) * NUM / DEN;
            let y = TOP + (GRID_TOP + (slot / 4) as i32 * CELL_HEIGHT) * NUM / DEN;
            if let Some(icon) = icon {
                blit(&mut output, icon,
                    x + i32::from(icon.offset[0]) * NUM / DEN,
                    y + i32::from(icon.offset[1]) * NUM / DEN,
                    (icon.width as i32 * NUM / DEN).max(1),
                    (icon.height as i32 * NUM / DEN).max(1), (x, y, x + 42, y + 42));
            }
            if choice.is_none() { dim(&mut output, x, y, 42, 42); }
            if selected == Some(id) { border(&mut output, x, y, 42, 42); }
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
fn border(output: &mut [u8], x: i32, y: i32, width: i32, height: i32) {
    for dy in 0..height {
        for dx in 0..width {
            if dx != 0 && dy != 0 && dx != width - 1 && dy != height - 1 { continue; }
            let px = x + dx;
            let py = y + dy;
            if !(0..WIDTH).contains(&px) || !(0..HEIGHT).contains(&py) { continue; }
            let at = (py as usize * WIDTH as usize + px as usize) * 4;
            output[at..at + 3].copy_from_slice(&[0, 220, 255]);
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Hit { Slot(usize), Close, Body }
pub(super) fn hit(view: Viewport, point: PixelPoint) -> Option<Hit> {
    if view.width <= 0 || view.height <= 0 { return None; }
    let x = (((i64::from(point.x) - i64::from(view.x)) * i64::from(WIDTH)) / i64::from(view.width)) as i32;
    let y = (((i64::from(point.y) - i64::from(view.y)) * i64::from(HEIGHT)) / i64::from(view.height)) as i32;
    if !(LEFT..LEFT + FRAME_WIDTH * NUM / DEN).contains(&x)
        || !(TOP..TOP + FRAME_HEIGHT * NUM / DEN).contains(&y) { return None; }
    let local_x = (x - LEFT) * DEN / NUM;
    let local_y = (y - TOP) * DEN / NUM;
    if (174..191).contains(&local_x) && (8..30).contains(&local_y) { return Some(Hit::Close); }
    // Invert render's floor(native * 5/4) slot boundaries at integer pixels.
    // Keep the frame/Close mapping above: frame artwork scales by its size.
    let x = ((x - LEFT) * DEN + DEN - 1) / NUM;
    let y = ((y - TOP) * DEN + DEN - 1) / NUM;
    let col = (x - GRID_LEFT) / CELL_WIDTH;
    let row = (y - GRID_TOP) / CELL_HEIGHT;
    if x >= GRID_LEFT && y >= GRID_TOP && (0..4).contains(&col) && (0..6).contains(&row) {
        let slot = row as usize * 4 + col as usize;
        return Some(if slot < 8 { Hit::Slot(slot) } else { Hit::Body });
    }
    Some(Hit::Body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{application_protocol::{CharacterKey, SelectionContext},
        battle_skillbook::Row};

    fn model() -> Model {
        let stamp = Stamp { context: SelectionContext { key: CharacterKey {
            source_id: 7, object_id: 3333 }, epoch: 1 }, channel: 0,
            world: [1, 2, 3], entity: 1, life: 1, revision: 1 };
        Model { stamp, rows: vec![4, 77, 999, 5].into_iter().map(|id| Row {
            id, label: format!("#{id}"), button: "施放",
            choice: (id != 5).then_some(Choice { stamp, id }),
        }).collect() }
    }
    fn panel() -> Panel {
        let (_, receiver) = mpsc::channel();
        Panel { sender: None, receiver, thread: None,
            cancel: Arc::new(AtomicBool::new(false)), serial: 7,
            request_active: true, requested: [None, Some(4), Some(5), None, None, None, None, Some(77)],
            ready: Some(Assets { frame: Arc::new(Picture { width: 193, height: 282,
                offset: [0, 0], rgba: vec![0; 193 * 282 * 4] }), icons: vec![None; 8] }),
            error: None }
    }
    #[test]
    fn painted_slot_boundary_render_selects_second_skill() {
        use sha2::{Digest, Sha256};
        let model = model();
        let panel = panel();
        let view = Viewport { x:0,y:0,width:800,height:600 };
        let point = PixelPoint { x:636,y:44 };
        let output = panel.render(&vec![0;800*600*4],&model,Some(4)).unwrap();
        let at = (point.y as usize * 800 + point.x as usize) * 4;
        let painted = panel.painted(&model,view).unwrap();
        let actual = hit(view,point);
        println!("{}",serde_json::json!({"case":"skill_render_border","fixture_only":true,
            "viewport":[0,0,800,600],"point":[point.x,point.y],"selected_skill":4,
            "painted_slot":1,"painted_skill":painted.id(1),"paint_valid":painted.valid(&panel,&model,view),
            "border_bgr":&output[at..at+3],"output_sha256":format!("{:X}",Sha256::digest(&output)),
            "expected_hit":"Some(Slot(1))","actual_hit":format!("{actual:?}")}));
        assert_eq!(&output[at..at+3],&[0,220,255],"point must be on the rendered skill-4 border");
        assert!(painted.valid(&panel,&model,view));
        assert_eq!(painted.id(1),Some(4));
        assert_eq!(actual,Some(Hit::Slot(1)),"rendered border must select its own skill slot");
    }
    #[test]
    fn painted_slot_boundary_grid_preserves_control_thresholds() {
        let view = Viewport { x:0,y:0,width:800,height:600 };
        let mut cases = Vec::<(String,PixelPoint,Option<Hit>)>::new();
        for row in 0..2 {
            for cut in 0..=4 {
                let x = LEFT + (27 + cut * 35) * 5 / 4;
                let y = (34 + row * 35 + 12) * 5 / 4;
                let before = if cut == 0 { Hit::Body } else { Hit::Slot((row*4+cut-1) as usize) };
                let after = if cut == 4 { Hit::Body } else { Hit::Slot((row*4+cut) as usize) };
                cases.push((format!("grid_x_row{row}_cut{cut}_before"),PixelPoint { x:x-1,y },Some(before)));
                cases.push((format!("grid_x_row{row}_cut{cut}_at"),PixelPoint { x,y },Some(after)));
            }
        }
        for col in 0..4 {
            for cut in 0..=6 {
                let x = LEFT + (27 + col * 35 + 12) * 5 / 4;
                let y = (34 + cut * 35) * 5 / 4;
                let before = if cut == 0 || cut > 2 { Hit::Body } else { Hit::Slot(((cut-1)*4+col) as usize) };
                let after = if cut >= 2 { Hit::Body } else { Hit::Slot((cut*4+col) as usize) };
                cases.push((format!("grid_y_col{col}_cut{cut}_before"),PixelPoint { x,y:y-1 },Some(before)));
                cases.push((format!("grid_y_col{col}_cut{cut}_at"),PixelPoint { x,y },Some(after)));
            }
        }
        // Preserve existing frame/Close mapping; frame artwork scales to
        // 241x352, while independently positioned slot origins use nominal5/4.
        for (label,x,y,before,after,axis_x) in [
            ("close_left",218,22,Some(Hit::Body),Some(Hit::Close),true),
            ("close_right",239,22,Some(Hit::Close),Some(Hit::Body),true),
            ("close_top",227,10,Some(Hit::Body),Some(Hit::Close),false),
            ("close_bottom",227,38,Some(Hit::Close),Some(Hit::Body),false),
            ("frame_left",0,125,None,Some(Hit::Body),true),
            ("frame_right",241,125,Some(Hit::Body),None,true),
            ("frame_top",0,0,None,Some(Hit::Body),false),
            ("frame_bottom",0,352,Some(Hit::Body),None,false),
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
        println!("{}",serde_json::json!({"case":"skill_grid_control_thresholds","fixture_only":true,
            "viewport":[0,0,800,600],"grid_boundary_points":76,"preserved_control_points":16,
            "point_count":cases.len(),"mismatches":mismatches,"observations":observations}));
        assert_eq!(mismatches,0,"painted grid boundaries and existing control thresholds must agree");
    }
    #[test]
    fn implemented_skill_grid_uses_stable_ids_and_revokes_changed_proofs() {
        let mut model = model();
        let panel = panel();
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let paint = panel.painted(&model, view).unwrap();
        assert!(paint.valid(&panel, &model, view));
        assert_eq!(paint.id(1), Some(4));
        assert_eq!(paint.id(7), Some(77));
        assert_eq!(paint.id(2), Some(5));
        assert_eq!(paint.id(0), None);
        assert!(!paint.contains(999));
        model.rows.reverse();
        assert!(paint.valid(&panel, &model, view), "server reorder retains stable skill IDs");
        model.rows.iter_mut().find(|r| r.id == 4).unwrap().choice = None;
        assert!(!paint.valid(&panel, &model, view), "availability needs a new committed paint");
        let next = panel.painted(&model, view).unwrap();
        assert!(next.valid(&panel, &model, view));
        model.stamp.life += 1;
        assert!(!next.valid(&panel, &model, view));
        assert!(!next.valid(&panel, &model, Viewport { x: 1, ..view }));
    }
    #[test]
    fn exact_grid_hit_projects_through_letterboxed_viewport() {
        for view in [Viewport { x: 0, y: 0, width: 800, height: 600 },
            Viewport { x: 100, y: 190, width: 400, height: 300 },
            Viewport { x: 48, y: 0, width: 1200, height: 900 }] {
            let x = LEFT + (GRID_LEFT + CELL_WIDTH + 12) * NUM / DEN;
            let y = (GRID_TOP + 12) * NUM / DEN;
            let point = PixelPoint { x: view.x + x * view.width / WIDTH,
                y: view.y + y * view.height / HEIGHT };
            assert_eq!(hit(view, point), Some(Hit::Slot(1)));
            let close = PixelPoint { x: view.x + (LEFT + 182 * NUM / DEN) * view.width / WIDTH,
                y: view.y + (18 * NUM / DEN) * view.height / HEIGHT };
            assert_eq!(hit(view, close), Some(Hit::Close));
            assert_eq!(hit(view, PixelPoint { x: view.x + 1, y: view.y + 1 }), None);
        }
    }
    #[test]
    fn selected_slot_changes_same_composed_world_buffer() {
        let model = model();
        let panel = panel();
        let base = vec![0; (WIDTH * HEIGHT * 4) as usize];
        let output = panel.render(&base, &model, Some(4)).unwrap();
        let x = LEFT + (GRID_LEFT + CELL_WIDTH) * NUM / DEN;
        let y = GRID_TOP * NUM / DEN;
        let at = (y as usize * WIDTH as usize + x as usize) * 4;
        assert_eq!(&output[at..at+3], &[0, 220, 255]);
        assert_ne!(output, base);
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; real Sprite09 2057.img + SpellIcon.idx + list.ico"]
    fn actual_shared_magic_frame_and_eight_skill_icons() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let frame = load_frame(&root).unwrap();
        assert_eq!((frame.width, frame.height), (193, 282));
        assert!(frame.rgba.chunks_exact(4).any(|p| p[3] == 0));
        let mut panel = Panel::start(root).unwrap();
        let mut model = model();
        let stamp = model.stamp;
        model.rows = SUPPORTED.into_iter().map(|id| Row { id, label: format!("#{id}"),
            button: "施放", choice: Some(Choice { stamp, id }) }).collect();
        assert!(panel.update(Some(&model)));
        for _ in 0..1000 {
            panel.update(Some(&model));
            if panel.ready() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(panel.ready(), "real worker: {:?}", panel.error());
        for icon in &panel.ready.as_ref().unwrap().icons {
            assert_eq!((icon.as_ref().unwrap().width, icon.as_ref().unwrap().height), (32, 32));
        }
        panel.update(None);
        assert!(!panel.ready());
    }
}
