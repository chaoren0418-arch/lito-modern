//! Canonical F5–F12 item/skill hotbar. Bindings are session-local identities;
//! only current server-confirmed models can authorize an action.
use super::{inventory_panel::{blit, dim, Picture}, PixelPoint, Viewport};
use crate::modern_lito::{
    battle_inventory::{Key, Model},
    battle_skillbook::{Choice, Model as SkillModel},
    battle_control::{Command, State as CommandState},
    battle_window::{Intent, Stamp},
    inventory_icons::SharedInventoryIcons,
    resources::{SharedArchive, SharedArchiveKind},
};
use anyhow::{ensure, Context, Result};
use std::{path::{Path, PathBuf}, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc}, thread};

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;
const NATIVE_WIDTH: i32 = 155;
const NATIVE_HEIGHT: i32 = 110;
const NUM: i32 = 5;
const DEN: i32 = 4;
const LEFT: i32 = WIDTH - NATIVE_WIDTH * NUM / DEN;
const TOP: i32 = HEIGHT - NATIVE_HEIGHT * NUM / DEN;
const SLOT_LEFT: i32 = 16;
const SLOT_TOP: i32 = 21;
const SLOT_WIDTH: i32 = 33;
const SLOT_HEIGHT: i32 = 33;

pub(super) fn key_slot(key: usize) -> Option<usize> {
    (0x74..=0x7b).contains(&key).then(|| key - 0x74)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Binding { Supply(Key), Skill(u32) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Action { Supply(Intent), Skill(Choice) }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Graphic { Supply(i64), Skill(u32) }

/// The base Sprite archive's 336-byte SpellIcon.idx is a 1-based skill-id
/// array of little-endian TBT graphic IDs. No LT/custom icon pack is read.
pub(super) fn spell_icons(root: &Path) -> Result<[u16; 168]> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
    let bytes = archive.read_entry("SpellIcon.idx")?;
    ensure!(bytes.len() == 336, "shared SpellIcon.idx length mismatch");
    let mut entries = [0u16; 168];
    for (id, pair) in bytes.chunks_exact(2).enumerate() {
        entries[id] = u16::from_le_bytes([pair[0], pair[1]]);
    }
    Ok(entries)
}
pub(super) fn spell_graphic(entries: &[u16; 168], id: u32) -> Result<i64> {
    let index = usize::try_from(id).ok().and_then(|id| id.checked_sub(1))
        .filter(|&index| index < entries.len()).context("skill ID outside shared SpellIcon.idx")?;
    let graphic = entries[index];
    ensure!(graphic != 0, "skill has no shared SpellIcon.idx graphic");
    Ok(i64::from(graphic))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Painted {
    pub(super) viewport: Viewport,
    pub(super) stamp: Stamp,
    serial: u64,
    proof_generation: u64,
    bindings: [Option<Binding>; 8],
    actions: [Option<Action>; 8],
}
impl Painted {
    pub(super) fn valid(self, bar: &Hotbar, viewport: Viewport) -> bool {
        bar.ready.is_some()
            && self.viewport == viewport
            && bar.model.as_ref().is_some_and(|m| m.stamp == self.stamp)
            && bar.skills.as_ref().is_some_and(|m| m.stamp == self.stamp)
            && self.serial == bar.serial
            && self.proof_generation == bar.proof_generation
            && self.bindings == bar.bindings
            && self.actions == bar.actions()
    }
    pub(super) fn action(self, bar: &Hotbar, slot: usize, viewport: Viewport) -> Option<Action> {
        if !self.valid(bar, viewport) { return None; }
        self.actions.get(slot).copied().flatten()
    }
}

fn load_backdrop(root: &Path) -> Result<Picture> {
    let mut archive = SharedArchive::open(root, SharedArchiveKind::SpriteShard(0))?;
    let bytes = archive.read_entry("2300.img")?;
    ensure!(bytes.len() == 8 + (NATIVE_WIDTH * NATIVE_HEIGHT * 2) as usize,
        "shared F5-F12 2300.img length mismatch");
    let width = u16::from_le_bytes([bytes[0], bytes[1]]);
    let height = u16::from_le_bytes([bytes[2], bytes[3]]);
    let masked = u16::from_le_bytes([bytes[4], bytes[5]]);
    let key = u16::from_le_bytes([bytes[6], bytes[7]]);
    ensure!((width, height, masked) == (155, 110, 1), "shared F5-F12 2300.img header mismatch");
    let mut rgba = Vec::with_capacity(width as usize * height as usize * 4);
    for pair in bytes[8..].chunks_exact(2) {
        let pixel = u16::from_le_bytes([pair[0], pair[1]]);
        if pixel == key {
            rgba.extend_from_slice(&[0, 0, 0, 0]);
        } else {
            let color = |shift| { let v = ((pixel >> shift) & 31u16) as u8; (v << 3) | (v >> 2) };
            rgba.extend_from_slice(&[color(10), color(5), color(0), 255]);
        }
    }
    Ok(Picture { width: width.into(), height: height.into(), offset: [0, 0], rgba })
}

struct Request { serial: u64, graphics: [Option<Graphic>; 8] }
struct Reply { serial: u64, result: Result<Assets> }
struct Assets { backdrop: Arc<Picture>, icons: Vec<Option<Picture>> }
struct Consumed { stamp: Stamp, command: Command, prior_receipt: Option<u64> }

fn worker(root: PathBuf, cancel: Arc<AtomicBool>, rx: mpsc::Receiver<Request>, tx: mpsc::Sender<Reply>) {
    let backdrop = load_backdrop(&root).map(Arc::new);
    let spells = spell_icons(&root);
    for request in rx {
        if cancel.load(Ordering::Acquire) { break; }
        let result = (|| -> Result<Assets> {
            let backdrop = backdrop.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?.clone();
            let spells = spells.as_ref().map_err(|e| anyhow::anyhow!("{e:#}"))?;
            let mut reader = SharedInventoryIcons::open_cancellable(&root, &cancel)?;
            let mut icons = Vec::with_capacity(8);
            for graphic in request.graphics {
                if cancel.load(Ordering::Acquire) { anyhow::bail!("hotbar icon load cancelled"); }
                let icon = graphic.map(|graphic| {
                    let id = match graphic {
                        Graphic::Supply(id) => id,
                        Graphic::Skill(id) => spell_graphic(spells, id)?,
                    };
                    let icon = reader.load(id, &cancel)?;
                    let image = icon.image();
                    Ok::<_, anyhow::Error>(Picture {
                        width: image.width(), height: image.height(), offset: image.raw_offset(),
                        rgba: image.reference_rgba()?,
                    })
                }).transpose().with_context(|| format!("shared hotbar icon {graphic:?}"))?;
                icons.push(icon);
            }
            Ok(Assets { backdrop, icons })
        })();
        if tx.send(Reply { serial: request.serial, result }).is_err() { break; }
    }
}

pub(super) struct Hotbar {
    sender: Option<mpsc::Sender<Request>>,
    receiver: mpsc::Receiver<Reply>,
    thread: Option<thread::JoinHandle<()>>,
    cancel: Arc<AtomicBool>,
    pub(super) model: Option<Model>,
    pub(super) skills: Option<SkillModel>,
    bindings: [Option<Binding>; 8],
    consumed: Option<Consumed>,
    requested: [Option<Graphic>; 8],
    request_active: bool,
    serial: u64,
    proof_generation: u64,
    ready: Option<Assets>,
    error: Option<String>,
}
impl Hotbar {
    pub(super) fn start(root: PathBuf) -> Result<Self> {
        ensure!(root.is_absolute(), "shared hotbar resource root must be absolute");
        let (sender, rx) = mpsc::channel();
        let (tx, receiver) = mpsc::channel();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let thread = thread::Builder::new().name("lito-hotbar-art".into())
            .spawn(move || worker(root, worker_cancel, rx, tx))?;
        Ok(Self { sender: Some(sender), receiver, thread: Some(thread), cancel,
            model: None, skills: None, bindings: [None; 8], consumed: None, requested: [None; 8],
            request_active: false, serial: 0, proof_generation: 0, ready: None, error: None })
    }
    pub(super) fn error(&self) -> Option<&str> { self.error.as_deref() }
    pub(super) fn clear_proofs(&mut self) {
        if let Some(model) = self.model.as_mut() { model.clear_proofs(); }
        if let Some(skills) = self.skills.as_mut() { skills.clear_proofs(); }
    }
    pub(super) fn submitted(&mut self, stamp: Stamp, command: Command,
        prior_receipt: Option<u64>) {
        if self.model.as_ref().is_some_and(|m| m.stamp == stamp) {
            self.consumed = Some(Consumed { stamp, command, prior_receipt });
            self.clear_proofs();
        }
    }
    pub(super) fn acknowledge(&mut self, state: &CommandState) -> bool {
        let confirmed = self.consumed.as_ref().is_some_and(|c|
            state.received_after(&c.command, c.prior_receipt).is_some());
        if confirmed {
            self.consumed = None;
            // Proof revocation is separate from the asset request serial: a
            // reply already in flight must still be accepted at its serial.
            self.proof_generation = self.proof_generation.wrapping_add(1);
        }
        confirmed
    }
    pub(super) fn update(&mut self, model: Option<Model>, skills: Option<SkillModel>) -> bool {
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
        if self.model.as_ref().zip(model.as_ref()).is_some_and(|(a,b)| !a.same_owner(b))
            || self.skills.as_ref().zip(skills.as_ref()).is_some_and(|(a,b)| !a.same_owner(b))
            || model.is_none() && self.model.is_some()
            || skills.is_none() && self.skills.is_some() {
            self.bindings = [None; 8];
            changed = true;
        }
        self.model = model;
        self.skills = skills.filter(|s| self.model.as_ref().is_some_and(|m| m.stamp == s.stamp));
        if self.model.is_none() || self.skills.is_none() {
            self.consumed = None;
            let had_ready = self.ready.take().is_some();
            if self.request_active || had_ready || self.requested != [None; 8] {
                self.serial = self.serial.wrapping_add(1);
                self.requested = [None; 8];
                self.request_active = false;
                self.error = None;
                changed = true;
            }
            return changed;
        }
        let graphics = self.graphics();
        if !self.request_active || graphics != self.requested {
            self.serial = self.serial.wrapping_add(1);
            self.requested = graphics;
            self.ready = None;
            self.error = None;
            self.request_active = true;
            if self.sender.as_ref().is_none_or(|s| s.send(Request { serial: self.serial, graphics }).is_err()) {
                self.error = Some("hotbar art worker unavailable".into());
            }
            changed = true;
        }
        changed
    }
    fn graphics(&self) -> [Option<Graphic>; 8] {
        std::array::from_fn(|slot| match self.bindings[slot]? {
            Binding::Supply(key) => self.model.as_ref()?.rows.iter().find(|r| r.key == key)?
                .graphic.map(Graphic::Supply),
            Binding::Skill(id) => self.skills.as_ref()?.rows.iter().any(|r| r.id == id)
                .then_some(Graphic::Skill(id)),
        })
    }
    fn actions(&self) -> [Option<Action>; 8] {
        if self.model.as_ref().is_some_and(|m| self.consumed.as_ref().is_some_and(|c| c.stamp == m.stamp)) {
            return [None; 8];
        }
        std::array::from_fn(|slot| match self.bindings[slot]? {
            Binding::Supply(key) => self.model.as_ref()?.local(key).map(Action::Supply),
            Binding::Skill(id) => self.skills.as_ref()?.choice(id).map(Action::Skill),
        })
    }
    /// Ctrl+F5–F12 while the inventory is open. `None` explicitly clears a slot.
    pub(super) fn bind(&mut self, slot: usize, key: Option<Key>) -> bool {
        if slot >= 8 { return false; }
        if let Some(key) = key {
            if !matches!(key, Key::Supply(_)) || !self.model.as_ref().is_some_and(|m|
                m.rows.iter().any(|r| r.key == key && r.graphic.is_some())) { return false; }
        }
        self.set_binding(slot, key.map(Binding::Supply))
    }
    pub(super) fn bind_skill(&mut self, slot: usize, id: u32) -> bool {
        if slot >= 8 || !(1..=168).contains(&id) ||
            !self.skills.as_ref().is_some_and(|m| m.rows.iter().any(|r| r.id == id && r.choice.is_some())) {
            return false;
        }
        self.set_binding(slot, Some(Binding::Skill(id)))
    }
    fn set_binding(&mut self, slot: usize, binding: Option<Binding>) -> bool {
        if self.bindings[slot] == binding { return false; }
        self.bindings[slot] = binding;
        self.request_active = false;
        self.update(self.model.clone(), self.skills.clone());
        true
    }
    pub(super) fn painted(&self, viewport: Viewport) -> Option<Painted> {
        self.ready.as_ref()?;
        Some(Painted { viewport, stamp: self.model.as_ref()?.stamp, serial: self.serial,
            proof_generation: self.proof_generation,
            bindings: self.bindings, actions: self.actions() })
    }
    pub(super) fn render(&self, base: &[u8]) -> Option<Vec<u8>> {
        let assets = self.ready.as_ref()?;
        if base.len() != (WIDTH * HEIGHT * 4) as usize { return None; }
        let mut output = base.to_vec();
        blit(&mut output, &assets.backdrop, LEFT, TOP,
            NATIVE_WIDTH * NUM / DEN, NATIVE_HEIGHT * NUM / DEN, (0,0,WIDTH,HEIGHT));
        for (slot, icon) in assets.icons.iter().enumerate() {
            let Some(icon) = icon else { continue; };
            let x = LEFT + (SLOT_LEFT + (slot % 4) as i32 * SLOT_WIDTH) * NUM / DEN;
            let y = TOP + (SLOT_TOP + (slot / 4) as i32 * SLOT_HEIGHT) * NUM / DEN;
            blit(&mut output, icon,
                x + i32::from(icon.offset[0]) * NUM / DEN,
                y + i32::from(icon.offset[1]) * NUM / DEN,
                (icon.width as i32 * NUM / DEN).max(1), (icon.height as i32 * NUM / DEN).max(1),
                (x, y, x + 40, y + 40));
            if self.bindings[slot].is_some() && self.actions()[slot].is_none() {
                dim(&mut output, x, y, 40, 40);
            }
        }
        Some(output)
    }
}
impl Drop for Hotbar {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        self.sender.take();
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

pub(super) fn hit(view: Viewport, point: PixelPoint) -> Option<Option<usize>> {
    if view.width <= 0 || view.height <= 0 { return None; }
    let x = (((i64::from(point.x) - i64::from(view.x)) * i64::from(WIDTH)) / i64::from(view.width)) as i32;
    let y = (((i64::from(point.y) - i64::from(view.y)) * i64::from(HEIGHT)) / i64::from(view.height)) as i32;
    if !(LEFT..LEFT + NATIVE_WIDTH * NUM / DEN).contains(&x)
        || !(TOP..TOP + NATIVE_HEIGHT * NUM / DEN).contains(&y) { return None; }
    let x = (x - LEFT) * DEN / NUM;
    let y = (y - TOP) * DEN / NUM;
    let col = (x - SLOT_LEFT) / SLOT_WIDTH;
    let row = (y - SLOT_TOP) / SLOT_HEIGHT;
    if x >= SLOT_LEFT && y >= SLOT_TOP && (0..4).contains(&col) && (0..2).contains(&row) {
        Some(Some(row as usize * 4 + col as usize))
    } else { Some(None) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    use crate::modern_lito::{
        application_protocol::{CharacterKey, SelectionContext},
        battle_control::{Outcome, State as CommandState},
        battle_inventory::Row,
        battle_skillbook::{Choice, Model as SkillModel, Row as SkillRow},
        battle_window::{Kind, Local},
    };
    fn model(revision: i64) -> Model {
        let stamp = Stamp {
            context: SelectionContext { key: CharacterKey { source_id: 7, object_id: 3333 }, epoch: 1 },
            channel: 0, world: [1, 2, 3], entity: 1, life: 1, revision,
        };
        Model { stamp, rows: vec![Row {
            key: Key::Supply(40010), graphic: Some(80), label: "healing supply".into(),
            equip: None, unequip: None,
            local: Some(Intent { stamp, kind: Kind::Local(Local::UseItem(40010)) }),
        }] }
    }
    fn skills(stamp: Stamp) -> SkillModel {
        SkillModel { stamp, rows: vec![
            SkillRow { id: 4, label: "Light Arrow".into(), button: "選擇目標",
                choice: Some(Choice { stamp, id: 4 }) },
            SkillRow { id: 5, label: "Teleport".into(), button: "施放",
                choice: Some(Choice { stamp, id: 5 }) },
        ] }
    }
    #[test]
    fn spell_index_is_one_based_and_rejects_missing_ids() {
        let mut index = [0u16; 168];
        index[0] = 383;
        index[3] = 386;
        assert_eq!(spell_graphic(&index, 1).unwrap(), 383);
        assert_eq!(spell_graphic(&index, 4).unwrap(), 386);
        for id in [0, 2, 169, u32::MAX] { assert!(spell_graphic(&index, id).is_err()); }
    }
    #[test]
    fn skill_admission_depends_on_choice_and_both_server_stamps_not_movement() {
        let stamp = model(1).stamp;
        let choices = skills(stamp);
        let q = choices.choice(4).unwrap();
        assert!(super::super::skill_choice_eligible(q, Some(&choices), Some(stamp), Some(stamp)));
        let mut disabled = choices.clone();
        disabled.clear_proofs();
        assert!(!super::super::skill_choice_eligible(q, Some(&disabled), Some(stamp), Some(stamp)));
        let mut stale = stamp;
        stale.revision += 1;
        assert!(!super::super::skill_choice_eligible(q, Some(&choices), Some(stale), Some(stamp)));
        assert!(!super::super::skill_choice_eligible(q, Some(&choices), Some(stamp), Some(stale)));
        assert!(!super::super::skill_choice_eligible(q, None, Some(stamp), Some(stamp)));
    }
    #[test]
    fn native_eight_keys_and_scaled_hit_are_exact() {
        for slot in 0..8 {
            assert_eq!(key_slot(0x74 + slot), Some(slot));
            let view = Viewport { x: 83, y: 25, width: 1200, height: 900 };
            let x = LEFT + (SLOT_LEFT + (slot % 4) as i32 * SLOT_WIDTH + 10) * NUM / DEN;
            let y = TOP + (SLOT_TOP + (slot / 4) as i32 * SLOT_HEIGHT + 10) * NUM / DEN;
            assert_eq!(hit(view, PixelPoint { x: view.x + x * view.width / WIDTH,
                y: view.y + y * view.height / HEIGHT }), Some(Some(slot)));
        }
        assert_eq!(key_slot(0x73), None);
        assert_eq!(key_slot(0x7c), None);
    }
    #[test]
    fn keyboard_and_mouse_share_one_consumable_supply_proof() {
        let (_, receiver) = mpsc::channel();
        let mut bar = Hotbar {
            sender: None, receiver, thread: None,
            cancel: Arc::new(AtomicBool::new(false)),
            model: Some(model(1)),
            skills: Some(skills(model(1).stamp)),
            bindings: [Some(Binding::Supply(Key::Supply(40010))), None, None, None, None, None, None, None],
            consumed: None,
            requested: [Some(Graphic::Supply(80)), None, None, None, None, None, None, None],
            request_active: true, serial: 7, proof_generation: 0,
            ready: Some(Assets { backdrop: Arc::new(Picture {
                width: 155, height: 110, offset: [0,0], rgba: vec![0; 155 * 110 * 4],
            }), icons: vec![None; 8] }), error: None,
        };
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let proof = bar.painted(view).unwrap();
        let key_slot = key_slot(0x74).unwrap();
        let click_slot = hit(view, PixelPoint {
            x: LEFT + (SLOT_LEFT + 10) * NUM / DEN,
            y: TOP + (SLOT_TOP + 10) * NUM / DEN,
        }).unwrap().unwrap();
        assert_eq!(key_slot, click_slot);
        let queued = Cell::new(None);
        queued.set(proof.action(&bar, key_slot, view));
        bar.clear_proofs(); // queue clears local proof pending controller admission
        assert_eq!(queued.take(), Some(Action::Supply(Intent {
            stamp: model(1).stamp, kind: Kind::Local(Local::UseItem(40010)),
        })));
        // The owner has taken the first action. Before its next fresh model,
        // neither keyboard nor click may resurrect the old command.
        assert_eq!(proof.action(&bar, key_slot, view), None);
        assert_eq!(proof.action(&bar, click_slot, view), None);
        let mut disabled = model(1);
        disabled.rows[0].local = None;
        bar.update(Some(disabled), Some(skills(model(1).stamp)));
        let disabled_paint = bar.painted(view).unwrap();
        assert_eq!(disabled_paint.action(&bar, 0, view), None);
        bar.update(Some(model(1)), Some(skills(model(1).stamp)));
        assert!(!disabled_paint.valid(&bar, view), "newly enabled action needs a new paint");
        assert!(bar.actions()[0].is_some(), "unsubmitted/rejected command must be recoverable");
    }
    #[test]
    fn skill_binding_uses_learned_id_and_consumes_same_painted_proof() {
        let (_, receiver) = mpsc::channel();
        let inventory = model(1);
        let mut bar = Hotbar { sender: None, receiver, thread: None,
            cancel: Arc::new(AtomicBool::new(false)), model: Some(inventory.clone()),
            skills: Some(skills(inventory.stamp)), bindings: [None; 8], requested: [None; 8],
            consumed: None,
            request_active: true, serial: 7, proof_generation: 0,
            ready: Some(Assets { backdrop: Arc::new(Picture {
                width: 155, height: 110, offset: [0,0], rgba: vec![0; 155 * 110 * 4],
            }), icons: vec![None; 8] }), error: None };
        assert!(!bar.bind_skill(0, 0));
        assert!(!bar.bind_skill(0, 77), "not learned");
        assert!(bar.bind_skill(0, 4));
        // Bind invalidates the previous asset/proof; this simulates a completed
        // worker reply and successful BitBlt publication at the new serial.
        bar.ready = Some(Assets { backdrop: Arc::new(Picture {
            width: 155, height: 110, offset: [0,0], rgba: vec![0; 155 * 110 * 4],
        }), icons: vec![None; 8] });
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let proof = bar.painted(view).unwrap();
        assert_eq!(proof.action(&bar, 0, view), Some(Action::Skill(Choice {
            stamp: inventory.stamp, id: 4,
        })));
        bar.clear_proofs(); // selecting a target did not send a command
        assert_eq!(proof.action(&bar, 0, view), None);
        bar.update(Some(inventory.clone()), Some(skills(inventory.stamp)));
        assert!(bar.actions()[0].is_some(), "target selection can be repeated without a server revision");
        assert!(bar.bind_skill(1, 5));
        let command = Command::world(crate::modern_lito::battle_world::Request::read(
            inventory.stamp.context, inventory.stamp.channel, None));
        bar.submitted(inventory.stamp, command, None); // instant teleport submitted
        bar.update(Some(inventory.clone()), Some(skills(inventory.stamp)));
        assert_eq!(bar.actions(), [None; 8], "instant cast cannot replay on the same stamp");
        let mut next = skills(inventory.stamp);
        next.rows[0].choice = None;
        bar.update(Some(inventory.clone()), Some(next));
        assert_eq!(proof.action(&bar, 0, view), None);
        let mut next = inventory.clone();
        next.stamp.life += 1;
        bar.update(Some(next.clone()), Some(skills(next.stamp)));
        assert_eq!(bar.bindings, [None; 8], "life scope revokes bindings");
    }
    #[test]
    fn exact_rejection_receipt_releases_proof_without_losing_inflight_art() {
        let (request_sender, request_receiver) = mpsc::channel();
        let (reply_sender, reply_receiver) = mpsc::channel();
        let inventory = model(1);
        let skillbook = skills(inventory.stamp);
        let backdrop = Arc::new(Picture { width: 155, height: 110,
            offset: [0,0], rgba: vec![0; 155 * 110 * 4] });
        let mut bar = Hotbar { sender: Some(request_sender), receiver: reply_receiver,
            thread: None, cancel: Arc::new(AtomicBool::new(false)),
            model: Some(inventory.clone()), skills: Some(skillbook.clone()),
            bindings: [Some(Binding::Supply(Key::Supply(40010))), None, None, None, None, None, None, None],
            consumed: None,
            requested: [Some(Graphic::Supply(80)), None, None, None, None, None, None, None],
            request_active: true, serial: 7, proof_generation: 0,
            ready: Some(Assets { backdrop: backdrop.clone(), icons: vec![None; 8] }), error: None };
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let old = bar.painted(view).unwrap();
        let command = Command::world(crate::modern_lito::battle_world::Request::read(
            inventory.stamp.context, inventory.stamp.channel, None));
        let other = Command::world(crate::modern_lito::battle_world::Request::read(
            inventory.stamp.context, inventory.stamp.channel + 1, None));
        bar.submitted(inventory.stamp, command.clone(), Some(4));
        assert_eq!(old.action(&bar, 0, view), None);
        bar.update(Some(inventory.clone()), Some(skillbook.clone()));
        assert_eq!(bar.actions()[0], None, "same stamp cannot replay submitted supply");
        assert!(!bar.acknowledge(&CommandState::test_receipt(other, 5, Outcome::NotSent)));
        assert!(!bar.acknowledge(&CommandState::test_receipt(command.clone(), 4, Outcome::NotSent)));
        assert_eq!(bar.actions()[0], None, "wrong command or old receipt cannot unlock");
        assert!(bar.bind_skill(1, 4), "rebinding may request art during an in-flight command");
        let request = request_receiver.try_recv().unwrap();
        assert_eq!(request.serial, bar.serial);
        assert_eq!(request.graphics[1], Some(Graphic::Skill(4)));
        assert!(bar.ready.is_none());
        let loading_serial = bar.serial;
        assert!(bar.acknowledge(&CommandState::test_receipt(command, 5, Outcome::NotSent)));
        assert_eq!(bar.serial, loading_serial, "receipt must not invalidate the art request");
        assert_eq!(bar.proof_generation, 1);
        reply_sender.send(Reply { serial: loading_serial,
            result: Ok(Assets { backdrop, icons: vec![None; 8] }) }).unwrap();
        assert!(bar.update(Some(inventory.clone()), Some(skillbook)));
        assert!(bar.ready.is_some(), "in-flight art reply must remain accepted");
        assert_eq!(old.action(&bar, 0, view), None, "old BitBlt proof must not revive");
        let fresh = bar.painted(view).unwrap();
        assert_eq!(fresh.action(&bar, 0, view), inventory.rows[0].local.map(Action::Supply));
        assert_eq!(fresh.action(&bar, 1, view), Some(Action::Skill(Choice {
            stamp: inventory.stamp, id: 4,
        })));
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; actual Sprite00 2300.img"]
    fn real_shared_f5_f12_frame_is_qualified() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let picture = load_backdrop(&root).unwrap();
        assert_eq!((picture.width, picture.height), (155,110));
        assert!(picture.rgba.chunks_exact(4).any(|pixel| pixel[3] == 0));
        assert!(picture.rgba.chunks_exact(4).any(|pixel| pixel[3] == 255));
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; real base SpellIcon.idx + shard TBT"]
    fn real_shared_skill_ids_resolve_to_decodable_icons() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let index = spell_icons(&root).unwrap();
        let mut icons = SharedInventoryIcons::open(&root).unwrap();
        let cancel = AtomicBool::new(false);
        for (id, graphic) in [(1,383), (4,386), (5,387), (29,402),
            (43,410), (54,622), (70,653), (77,432)] {
            assert_eq!(spell_graphic(&index, id).unwrap(), graphic);
            let icon = icons.load(graphic, &cancel).unwrap();
            assert_eq!((icon.image().width(), icon.image().height()), (32, 32));
        }
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; real SpellIcon.idx + hotbar worker"]
    fn real_worker_skill_icon_and_proof_are_bound_to_current_skill_model() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut bar = Hotbar::start(root).unwrap();
        let inventory = model(1);
        let skillbook = skills(inventory.stamp);
        bar.update(Some(inventory.clone()), Some(skillbook.clone()));
        assert!(bar.bind_skill(2, 4));
        for _ in 0..1000 {
            bar.update(Some(inventory.clone()), Some(skillbook.clone()));
            if bar.ready.is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        let assets = bar.ready.as_ref().unwrap_or_else(|| panic!("real worker: {:?}", bar.error()));
        assert_eq!((assets.icons[2].as_ref().unwrap().width,
            assets.icons[2].as_ref().unwrap().height), (32,32));
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let proof = bar.painted(view).unwrap();
        assert_eq!(proof.action(&bar, 2, view), Some(Action::Skill(Choice {
            stamp: inventory.stamp, id: 4,
        })));
        bar.clear_proofs();
        assert_eq!(proof.action(&bar, 2, view), None);
        bar.update(Some(inventory.clone()), Some(skillbook.clone()));
        assert!(bar.actions()[2].is_some(), "target selection alone is not a submitted spell");
        let mut not_ready = skillbook;
        not_ready.rows[0].choice = None;
        bar.update(Some(inventory), Some(not_ready));
        assert_eq!(proof.action(&bar, 2, view), None);
    }
    #[test]
    #[ignore = "explicit LITO_SHARED_RESOURCE_ROOT; real shared 2300.img + list.ico TBT worker"]
    fn real_worker_bound_supply_is_visible_and_only_current_server_proof_acts() {
        let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut bar = Hotbar::start(root).unwrap();
        let m = model(1);
        assert!(bar.update(Some(m.clone()), Some(skills(m.stamp))));
        assert!(bar.bind(0, Some(Key::Supply(40010))));
        assert!(!bar.bind(0, Some(Key::Source(999)))); // never clears a valid slot
        for _ in 0..1000 {
            if bar.update(Some(m.clone()), Some(skills(m.stamp))) && bar.ready.is_some() { break; }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(bar.ready.is_some(), "real worker: {:?}", bar.error());
        let base = vec![0u8; (WIDTH * HEIGHT * 4) as usize];
        assert_ne!(bar.render(&base).unwrap(), base);
        let view = Viewport { x: 0, y: 0, width: 800, height: 600 };
        let proof = bar.painted(view).unwrap(); // only caller may publish after BitBlt
        assert_eq!(proof.action(&bar, 0, view), m.rows[0].local.map(Action::Supply));
        assert_eq!(proof.action(&bar, 1, view), None);
        let command = Command::world(crate::modern_lito::battle_world::Request::read(
            m.stamp.context, m.stamp.channel, None));
        bar.submitted(m.stamp, command, None);
        assert_eq!(proof.action(&bar, 0, view), None, "queued action consumes its UI proof");
        bar.update(Some(m.clone()), Some(skills(m.stamp)));
        assert_eq!(proof.action(&bar, 0, view), None, "same revision does not restore proof");
        let mut depleted = m.clone();
        depleted.rows[0].local = None;
        bar.update(Some(depleted), Some(skills(m.stamp)));
        assert_eq!(proof.action(&bar, 0, view), None);
        bar.update(Some(model(2)), Some(skills(model(2).stamp)));
        assert!(!proof.valid(&bar, view));
        bar.update(None, None);
        assert_eq!(bar.bindings, [None; 8]);
        assert!(bar.ready.is_none(), "return must release obsolete shared pixels");
        assert!(!bar.update(None, None), "closed world must not invalidate on every poll");
        assert!(!proof.valid(&bar, view));
    }
}
