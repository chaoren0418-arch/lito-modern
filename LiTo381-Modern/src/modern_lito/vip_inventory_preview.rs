//! Read-only, owner-scoped main-window backpack art. The source item instance
//! remains authoritative; missing or unsuitable shared TBT art is never guessed.
//! The decoded natural-size preview starts at the cell origin. TBT offset bytes
//! are preserved by the decoder but are not claimed as native UI placement.
use super::{
    application_protocol::SelectionContext, battle_follow::Observation,
    compositor::Canvas, inventory_icons::SharedInventoryIcons,
};
use std::{collections::BTreeMap, path::Path, sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc}, thread::{self, JoinHandle}};

pub(crate) const ICON_SIDE: u32 = 32;
const MAX_ROWS: usize = 512;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Key {
    pub context: SelectionContext,
    pub channel: u8,
    pub room: i64,
    pub match_id: i64,
    pub generation: i64,
    pub entity: i64,
    pub life: i64,
    pub graphics: Vec<i64>,
}
impl Key {
    pub(crate) fn same_reading_scope(&self, other: &Self) -> bool {
        self.context == other.context && self.channel == other.channel
            && self.room == other.room && self.match_id == other.match_id
            && self.generation == other.generation && self.entity == other.entity
            && self.life == other.life
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub object: i64,
    pub graphic: i64,
    pub label: String,
    pub quantity: i64,
    pub equipped: Option<bool>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Model {
    pub key: Key,
    pub rows: Vec<Row>,
}

impl Model {
    pub(crate) fn from_observation(
        context: SelectionContext, channel: u8, observation: &Observation,
        mut label: impl FnMut(&str) -> String,
    ) -> Option<Self> {
        let data = observation.data();
        let vip = data.vip.as_ref()?;
        let equipment = data.personal.equipment.as_ref()?;
        if i32::try_from(context.key.source_id).ok()? != vip.owner.player.source
            || i64::try_from(context.key.object_id).ok()? != vip.owner.player.object
            || i64::try_from(context.epoch).ok()? != vip.owner.player.epoch
            || vip.owner.player.channel != i32::from(channel)
            || vip.owner.entity != data.personal.entity
            || equipment.inventory.0.len() > MAX_ROWS
        { return None; }
        let rows = equipment.inventory.0.iter().map(|item| Row {
            object: item.object,
            graphic: item.inventory_graphic,
            label: label(&item.name.0).chars().filter(|c| !c.is_control()).take(180).collect(),
            quantity: item.quantity,
            equipped: observation.personal().equipped(item.object),
        }).collect::<Vec<_>>();
        let mut graphics = rows.iter().filter_map(|row| (row.graphic >= 0).then_some(row.graphic)).collect::<Vec<_>>();
        graphics.sort_unstable();
        graphics.dedup();
        Some(Self {
            key: Key { context, channel, room: data.world.identity.room,
                match_id: data.world.identity.match_id,
                generation: data.world.identity.generation,
                entity: vip.owner.entity, life: vip.owner.life, graphics },
            rows,
        })
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Art { pub bgra: Vec<u8> }
pub(crate) type Bank = BTreeMap<i64, Result<Art, &'static str>>;

fn load(root: &Path, key: &Key, cancel: &AtomicBool) -> Bank {
    let mut bank = Bank::new();
    let Ok(mut archive) = SharedInventoryIcons::open_cancellable(root, cancel) else {
        for graphic in &key.graphics { bank.insert(*graphic, Err("共用圖示資源無法開啟")); }
        return bank;
    };
    for graphic in &key.graphics {
        if cancel.load(Ordering::Acquire) { break; }
        let result = archive.load(*graphic, cancel)
            .map_err(|_| "此背包圖示無法載入")
            .and_then(|icon| {
                let image = icon.image();
                if image.width() == 0 || image.height() == 0
                    || image.width() > ICON_SIDE as usize || image.height() > ICON_SIDE as usize
                { return Err("圖示尺寸未資格化"); }
                let rgba = image.reference_rgba().map_err(|_| "背包圖示解碼失敗")?;
                let mut canvas = Canvas::new(ICON_SIDE, ICON_SIDE).map_err(|_| "圖示畫布無法建立")?;
                canvas.clear([24, 24, 24]);
                canvas.blit_rgba(&rgba, image.width() as u32, image.height() as u32, 0, 0)
                    .map_err(|_| "圖示像素無法繪製")?;
                let mut bgra = Vec::new();
                canvas.copy_bgra(&mut bgra).map_err(|_| "圖示像素無法轉換")?;
                Ok(Art { bgra })
            });
        bank.insert(*graphic, result);
    }
    bank
}

struct Job {
    key: Key,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<Bank>,
    thread: Option<JoinHandle<()>>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() { let _ = thread.join(); }
    }
}

#[derive(Default)]
pub(crate) struct Preview {
    model: Option<Model>,
    bank: Bank,
    job: Option<Job>,
    attempted: bool,
    revision: u64,
}
impl Preview {
    pub(crate) fn model(&self) -> Option<&Model> { self.model.as_ref() }
    pub(crate) fn art(&self, graphic: i64) -> Option<&Result<Art, &'static str>> { self.bank.get(&graphic) }
    pub(crate) fn revision(&self) -> u64 { self.revision }
    pub(crate) fn is_quiescent(&self) -> bool { self.job.is_none() }
    pub(crate) fn cancel(&self) {
        if let Some(job) = &self.job { job.cancel.store(true, Ordering::Release); }
    }
    pub(crate) fn reconcile(&mut self, desired: Option<Model>) {
        let old_key = self.model.as_ref().map(|m| &m.key);
        let new_key = desired.as_ref().map(|m| &m.key);
        if old_key != new_key {
            self.cancel();
            self.bank.clear();
            self.attempted = false;
        }
        if self.model != desired {
            self.model = desired;
            self.revision = self.revision.wrapping_add(1);
        }
    }
    pub(crate) fn poll(&mut self, root: &Path, closing: bool) {
        if closing { self.reconcile(None); }
        if self.job.as_ref().is_some_and(|job| job.thread.as_ref().is_some_and(JoinHandle::is_finished)) {
            let mut job = self.job.take().unwrap();
            let joined = job.thread.take().unwrap().join();
            if !closing && self.model.as_ref().is_some_and(|m| m.key == job.key)
                && !job.cancel.load(Ordering::Acquire)
            {
                self.bank = if joined.is_ok() { job.receiver.try_recv().ok() } else { None }
                    .unwrap_or_else(|| job.key.graphics.iter()
                        .map(|graphic| (*graphic, Err("圖示工作異常結束"))).collect());
                self.revision = self.revision.wrapping_add(1);
            }
        }
        if closing || self.job.is_some() || self.attempted { return; }
        let Some(model) = &self.model else { return; };
        self.attempted = true;
        if model.key.graphics.is_empty() { return; }
        let key = model.key.clone();
        let worker_key = key.clone();
        let root = root.to_path_buf();
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        match thread::Builder::new().name("lito-vip-inventory-art".into()).spawn(move || {
            let _ = sender.send(load(&root, &worker_key, &worker_cancel));
        }) {
            Ok(thread) => self.job = Some(Job { key, cancel, receiver, thread: Some(thread) }),
            Err(_) => {
                self.bank = key.graphics.iter().map(|graphic| (*graphic, Err("無法啟動圖示工作"))).collect();
                self.revision = self.revision.wrapping_add(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::application_protocol::CharacterKey;
    use std::time::{Duration, Instant};

    fn model(source: u32, graphic: i64) -> Model {
        Model {
            key: Key { context: SelectionContext {
                key: CharacterKey { source_id: source, object_id: 3333 }, epoch: 4 },
                channel: 0, room: 1, match_id: 2, generation: 3,
                entity: 44, life: 1, graphics: vec![graphic] },
            rows: vec![Row { object: 10, graphic, label: "測試".into(),
                quantity: 3, equipped: Some(false) }],
        }
    }

    #[test]
    fn owner_change_retires_old_art_and_stale_worker_without_reusing_graphic() {
        let root = Path::new("this-shared-root-does-not-exist");
        let mut preview = Preview::default();
        preview.reconcile(Some(model(7, 70)));
        preview.poll(root, false);
        preview.reconcile(Some(model(42, 70)));
        assert_eq!(preview.model().unwrap().key.context.key.source_id, 42);
        assert!(preview.art(70).is_none());
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            preview.poll(root, false);
            assert!(Instant::now() < deadline, "owned art worker did not retire");
            if preview.is_quiescent() && preview.art(70).is_some() { break; }
            std::thread::sleep(Duration::from_millis(2));
        }
        assert!(preview.art(70).unwrap().is_err());
        preview.reconcile(None);
        assert!(preview.model().is_none());
        assert!(preview.art(70).is_none());
        preview.poll(root, true);
        assert!(preview.is_quiescent());
    }

    #[test]
    fn same_owner_quantity_update_keeps_key_but_revises_row() {
        let mut preview = Preview::default();
        preview.reconcile(Some(model(7, 70)));
        let revision = preview.revision();
        let mut next = model(7, 70);
        next.rows[0].quantity = 4;
        preview.reconcile(Some(next));
        assert!(preview.revision() > revision);
        assert_eq!(preview.model().unwrap().rows[0].quantity, 4);
    }

    #[test]
    fn reading_scope_ignores_art_set_but_not_source_or_life() {
        let first = model(7, 70).key;
        let second = model(7, 71).key;
        assert!(first.same_reading_scope(&second));
        assert!(!first.same_reading_scope(&model(42, 70).key));
        let mut born_again = first.clone();
        born_again.life += 1;
        assert!(!first.same_reading_scope(&born_again));
    }
}
