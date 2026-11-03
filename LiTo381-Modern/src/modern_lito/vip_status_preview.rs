//! Owner-only status art for the main window while full-VIP scene LOAD remains
//! guarded. Resource I/O runs on one owned worker; a stale result never paints.
use super::{
    application_protocol::SelectionContext,
    battle_follow::Observation,
    compositor::Canvas,
    effect_catalog::EffectCatalog,
    vip_status,
};
use anyhow::{ensure, Result};
use std::{
    path::Path,
    sync::{atomic::{AtomicBool, Ordering}, mpsc, Arc},
    thread::{self, JoinHandle},
};

pub(crate) const COLUMNS: usize = 16;
pub(crate) const WIDTH: u32 = 512;
pub(crate) const HEIGHT: u32 = 32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Key {
    pub context: SelectionContext,
    pub channel: u8,
    pub room: i64,
    pub match_id: i64,
    pub generation: i64,
    pub entity: i64,
    pub life: i64,
    pub icons: Vec<i64>,
}
impl Key {
    pub(crate) fn from_observation(
        context: SelectionContext, channel: u8, o: &Observation,
    ) -> Result<Option<Self>> {
        let data = o.data();
        let Some(vip) = data.vip.as_ref() else { return Ok(None) };
        let ids = vip_status::required(o)?;
        if ids.is_empty() { return Ok(None) }
        ensure!(i32::try_from(context.key.source_id).ok() == Some(vip.owner.player.source)
            && i64::try_from(context.key.object_id).ok() == Some(vip.owner.player.object)
            && vip.owner.player.channel == i32::from(channel)
            && vip.owner.entity == data.personal.entity,
            "VIP status owner/scope mismatch");
        Ok(Some(Self {
            context, channel,
            room: data.world.identity.room,
            match_id: data.world.identity.match_id,
            generation: data.world.identity.generation,
            entity: vip.owner.entity,
            life: vip.owner.life,
            icons: ids.into_iter().map(|id| i64::from(id.value())).collect(),
        }))
    }
}

pub(crate) struct Prepared {
    pub key: Key,
    pub bgra: Vec<u8>,
}
pub(crate) fn prepare(
    root: &Path, xml: &Path, key: Key, observation: &Observation, cancel: &AtomicBool,
) -> Result<Prepared> {
    ensure!(key.icons.len() <= COLUMNS,
        "VIP status exceeds the qualified main-window strip capacity");
    ensure!(Key::from_observation(key.context, key.channel, observation)?.as_ref() == Some(&key),
        "VIP status observation changed before resource load");
    let catalog = EffectCatalog::load(xml, cancel)?;
    let assets = vip_status::Assets::load(root, catalog, &key.icons, cancel)?;
    let mut canvas = Canvas::new(WIDTH, HEIGHT)?;
    canvas.clear([24, 24, 24]);
    let placements = assets.draw(observation, &mut canvas, (0, 0), COLUMNS as u32, cancel)?;
    ensure!(placements.len() == key.icons.len(), "VIP status icon count mismatch");
    let mut bgra = Vec::new();
    canvas.copy_bgra(&mut bgra)?;
    ensure!(!cancel.load(Ordering::Acquire), "VIP status cancelled");
    Ok(Prepared { key, bgra })
}

struct Job {
    key: Key,
    cancel: Arc<AtomicBool>,
    receiver: mpsc::Receiver<std::result::Result<Prepared, String>>,
    thread: Option<JoinHandle<()>>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[derive(Default)]
pub(crate) struct Preview {
    key: Option<Key>,
    job: Option<Job>,
    image: Option<Prepared>,
    attempted: bool,
    error: Option<String>,
    revision: u64,
}
impl Preview {
    pub(crate) fn image(&self) -> Option<&Prepared> { self.image.as_ref() }
    pub(crate) fn error(&self) -> Option<&str> { self.error.as_deref() }
    pub(crate) fn revision(&self) -> u64 { self.revision }
    pub(crate) fn is_quiescent(&self) -> bool { self.job.is_none() }
    pub(crate) fn cancel(&self) {
        if let Some(job) = &self.job { job.cancel.store(true, Ordering::Release); }
    }
    pub(crate) fn update(
        &mut self,
        desired: Option<(Key, &Observation)>,
        root: &Path,
        xml: Option<&Path>,
        closing: bool,
    ) {
        let next = if closing { None } else { desired.as_ref().map(|(key, _)| key.clone()) };
        if self.key != next {
            self.cancel();
            self.key = next;
            self.image = None;
            self.error = None;
            self.attempted = false;
            self.revision = self.revision.wrapping_add(1);
        }
        if self.job.as_ref().is_some_and(|job| job.thread.as_ref().is_some_and(JoinHandle::is_finished)) {
            let mut job = self.job.take().unwrap();
            let joined = job.thread.take().unwrap().join();
            if !closing && self.key.as_ref() == Some(&job.key) {
                self.error = if joined.is_err() {
                    Some("狀態圖資源工作未正常完成".into())
                } else {
                    match job.receiver.try_recv() {
                        Ok(Ok(image)) if image.key == job.key => {
                            self.image = Some(image);
                            None
                        }
                        Ok(Err(reason)) => Some(reason),
                        _ => Some("狀態圖資源工作未傳回完整影像".into()),
                    }
                };
                self.revision = self.revision.wrapping_add(1);
            }
        }
        if closing || self.job.is_some() || self.image.is_some() || self.attempted {
            return;
        }
        let Some((key, observation)) = desired else { return };
        let Some(xml) = xml else {
            self.error = Some("目前設定未指定 effectlist2.xml 絕對路徑".into());
            self.attempted = true;
            return;
        };
        let (root, xml, observation) = (root.to_path_buf(), xml.to_path_buf(), observation.clone());
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let worker_key = key.clone();
        let (sender, receiver) = mpsc::sync_channel(1);
        self.attempted = true;
        match thread::Builder::new().name("lito-vip-status-art".into()).spawn(move || {
            let result = prepare(&root, &xml, worker_key, &observation, &worker_cancel)
                .map_err(|error| error.to_string().chars().filter(|c| !c.is_control()).take(180).collect());
            let _ = sender.send(result);
        }) {
            Ok(thread) => self.job = Some(Job { key, cancel, receiver, thread: Some(thread) }),
            Err(_) => self.error = Some("無法建立狀態圖資源工作執行緒".into()),
        }
    }
}
