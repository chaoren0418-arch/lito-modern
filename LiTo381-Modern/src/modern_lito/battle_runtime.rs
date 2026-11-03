//! Owned direct-canonical resource/render worker. No legacy Frame conversion.
//! Submit/poll do no filesystem I/O and never wait on a mailbox or a live join.
//! At most two owned workers (active + retiring) and one latest pending input.
//! Reads are bounded but synchronous; cancellation is NOT a hard I/O deadline.
//! Normal exit must poll until quiescent; Drop joins as the ownership backstop.
//! Ready pixels alone are NOT permission. A LOAD request additionally requires
//! a sealed shared-resource certificate and the current loading attachment.
use super::{
    battle_follow::Observation,
    battle_payload::data::Phase,
    battle_playback::{Attachment, PreparedAudio, View as AudioView},
    battle_scene::{SharedScene, Surface},
    battle_vip_scene_preview::Preview as VipPreview,
    preferences::SharedPreferences,
};
use anyhow::{ensure, Result};
use std::{
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, TryLockError,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

#[cfg(test)]
#[path = "battle_runtime_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "battle_runtime_diagnostics_tests.rs"]
pub(crate) mod diagnostics_tests;

#[cfg(test)]
#[path = "battle_render_gate_tests.rs"]
pub(crate) mod render_gate_tests;

const INTERVAL: Duration = Duration::from_millis(50);

#[derive(Clone, Debug)]
pub enum Status {
    Idle,
    Loading,
    Ready(Arc<Surface>),
    Retired,
    Failed(&'static str),
    Stopping,
}

struct Submission {
    attachment: Arc<Attachment>,
    // Local mailbox ordinal only. It neither modifies nor represents wire data.
    ordinal: u64,
    observation: Observation,
}
struct Output {
    input: Arc<Submission>,
    surface: Arc<Surface>,
    audio: PreparedAudio,
    resources: Option<super::battle_scene::PreparedResources>,
}

trait Renderer: Send {
    fn prepared_resources(&self) -> Option<super::battle_scene::PreparedResources> {
        None
    }
    fn requires_reload(&self, next: &Observation) -> Result<bool>;
    fn observe(&mut self, next: Observation, cancel: &AtomicBool) -> Result<()>;
    fn render(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface>;
    fn reload(
        &mut self,
        root: &Path,
        next: Observation,
        cancel: &AtomicBool,
        loader: &Loader,
    ) -> Result<Box<dyn Renderer>> {
        loader(root, next, cancel)
    }
}
impl Renderer for SharedScene {
    fn prepared_resources(&self) -> Option<super::battle_scene::PreparedResources> {
        SharedScene::prepared_resources(self)
    }
    fn requires_reload(&self, next: &Observation) -> Result<bool> {
        SharedScene::requires_reload(self, next)
    }
    fn observe(&mut self, next: Observation, cancel: &AtomicBool) -> Result<()> {
        SharedScene::observe(self, next, cancel)
    }
    fn render(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface> {
        SharedScene::render(self, now, cancel)
    }
    fn reload(
        &mut self,
        root: &Path,
        next: Observation,
        cancel: &AtomicBool,
        _: &Loader,
    ) -> Result<Box<dyn Renderer>> {
        let mut replacement =
            SharedScene::load_with_effects(root, next, self.effect_catalog_path(), cancel)?;
        replacement.carry_from(self, Instant::now(), cancel)?;
        Ok(Box::new(replacement))
    }
}
impl Renderer for VipPreview {
    fn requires_reload(&self, next: &Observation) -> Result<bool> {
        VipPreview::requires_reload(self, next)
    }
    fn observe(&mut self, next: Observation, cancel: &AtomicBool) -> Result<()> {
        VipPreview::observe(self, next, cancel)
    }
    fn render(&mut self, now: Instant, cancel: &AtomicBool) -> Result<Surface> {
        VipPreview::render(self, now, cancel)
    }
    // No prepared_resources implementation: even a fully painted preview
    // cannot satisfy the full VIP LOAD barrier.
}
type Loader =
    Arc<dyn Fn(&Path, Observation, &AtomicBool) -> Result<Box<dyn Renderer>> + Send + Sync>;

struct Job {
    attachment: Arc<Attachment>,
    cancel: Arc<AtomicBool>,
    input: Arc<Mutex<Arc<Submission>>>,
    output: Arc<Mutex<Option<Arc<Output>>>>,
    handle: Option<JoinHandle<std::result::Result<(), &'static str>>>,
}
impl Job {
    fn spawn(
        root: PathBuf,
        initial: Arc<Submission>,
        loader: Loader,
        preferences: Arc<SharedPreferences>,
    ) -> Result<Self> {
        let attachment = initial.attachment.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let input = Arc::new(Mutex::new(initial.clone()));
        let output = Arc::new(Mutex::new(None));
        let stop = cancel.clone();
        let inbox = input.clone();
        let outbox = output.clone();
        let handle = thread::Builder::new()
            .name("lito-canonical-render".into())
            .spawn(move || {
                let result = run(
                    &root,
                    initial,
                    &loader,
                    &stop,
                    &inbox,
                    &outbox,
                    &preferences,
                );
                if stop.load(Ordering::Acquire) {
                    Ok(())
                } else {
                    result
                }
            })?;
        Ok(Self {
            attachment,
            cancel,
            input,
            output,
            handle: Some(handle),
        })
    }
    fn cancel(&self) {
        self.cancel.store(true, Ordering::Release);
        if let Some(handle) = &self.handle {
            handle.thread().unpark();
        }
    }
    fn finished(&self) -> bool {
        self.handle.as_ref().is_none_or(JoinHandle::is_finished)
    }
    fn join(&mut self) -> std::result::Result<(), &'static str> {
        self.handle.take().map_or(Ok(()), |h| {
            h.join().unwrap_or(Err("canonical render worker panicked"))
        })
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancel();
        let _ = self.join();
    }
}

fn run(
    root: &Path,
    initial: Arc<Submission>,
    loader: &Loader,
    cancel: &AtomicBool,
    inbox: &Mutex<Arc<Submission>>,
    outbox: &Mutex<Option<Arc<Output>>>,
    preferences: &SharedPreferences,
) -> std::result::Result<(), &'static str> {
    if cancel.load(Ordering::Acquire) {
        return Ok(());
    }
    let mut renderer = loader(root, initial.observation.clone(), cancel)
        .map_err(|_| "canonical shared resources could not be loaded")?;
    let mut current = initial;
    let mut gain_cache = super::audio_gain::GainCache::default();
    loop {
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let start = Instant::now();
        let next = inbox
            .lock()
            .map_err(|_| "canonical input mailbox failed")?
            .clone();
        if !Arc::ptr_eq(&current, &next) {
            if !Arc::ptr_eq(&current.attachment, &next.attachment)
                || next.ordinal <= current.ordinal
            {
                return Err("canonical worker input went backwards");
            }
            current
                .observation
                .check_successor(&next.observation)
                .map_err(|_| "canonical worker received incompatible observations")?;
            if renderer
                .requires_reload(&next.observation)
                .map_err(|_| "canonical resource transition is invalid")?
            {
                // Fully qualify the replacement on this worker. No empty-hand
                // fallback, no half-updated private stats over the old pixels.
                let replacement = renderer
                    .reload(root, next.observation.clone(), cancel, loader)
                    .map_err(|_| "canonical replacement resources could not be loaded")?;
                if cancel.load(Ordering::Acquire) {
                    return Ok(());
                }
                renderer = replacement;
            } else {
                renderer
                    .observe(next.observation.clone(), cancel)
                    .map_err(|_| "canonical scene update failed")?;
            }
            current = next;
        }
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let mut surface = renderer
            .render(Instant::now(), cancel)
            .map_err(|_| "canonical shared rendering failed")?;
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        let data = current.observation.data();
        if surface.width != 800
            || surface.height != 600
            || surface.bgra.len() != 800 * 600 * 4
            || surface.observation.data() != data
            || surface.geometry.world_id != data.world.identity.match_id as u64
            || surface.geometry.revision != data.world.revision as u64
            || surface
                .navigation
                .as_ref()
                .is_some_and(|n| !n.matches(&surface.observation))
        {
            return Err("canonical renderer returned a mismatched surface");
        }
        let audio = PreparedAudio::prepare(
            surface.audio.take(),
            preferences.snapshot(),
            &mut gain_cache,
            cancel,
        );
        #[cfg(test)]
        super::battle_teleport::frame_diagnostic("runtime-prepared-output", &current.observation, Instant::now(), audio.frame.as_ref());
        if cancel.load(Ordering::Acquire) {
            return Ok(());
        }
        surface.audio = audio.frame.clone();
        let output = Arc::new(Output {
            input: current.clone(),
            surface: Arc::new(surface),
            audio,
            resources: renderer.prepared_resources(),
        });
        let old = outbox
            .lock()
            .map_err(|_| "canonical output mailbox failed")?
            .replace(output);
        drop(old);
        // No catch-up burst if a render exceeded the 20Hz presentation budget.
        if let Some(wait) = INTERVAL.checked_sub(start.elapsed()) {
            thread::park_timeout(wait);
        }
    }
}

pub struct BattleRuntime {
    root: PathBuf,
    preferences: Arc<SharedPreferences>,
    loader: Loader,
    active: Option<Job>,
    retiring: Option<Job>,
    desired: Option<Arc<Submission>>,
    pending: Option<Arc<Submission>>,
    published: Option<Arc<Output>>,
    failed: Option<&'static str>,
    closing: bool,
}
impl BattleRuntime {
    /// Pure construction: no path open, resource decode or worker launch.
    pub fn new(root: PathBuf) -> Result<Self> {
        Self::with_preferences(root, super::preferences::shared())
    }
    /// Preferences are modern-only and read by the existing owned worker.
    pub fn with_preferences(root: PathBuf, preferences: Arc<SharedPreferences>) -> Result<Self> {
        Self::with_preferences_and_effects(root, None, preferences)
    }
    pub fn with_preferences_and_effects(
        root: PathBuf,
        effect_catalog_path: Option<PathBuf>,
        preferences: Arc<SharedPreferences>,
    ) -> Result<Self> {
        ensure!(
            root.is_absolute(),
            "canonical resource root must be absolute"
        );
        Ok(Self::with_loader(
            root,
            Arc::new(move |root, observation, cancel| {
                if observation.data().vip.is_some() {
                    Ok(Box::new(VipPreview::load(root, observation,
                        effect_catalog_path.as_deref(), cancel)?) as Box<dyn Renderer>)
                } else {
                    Ok(Box::new(SharedScene::load_with_effects(root, observation,
                        effect_catalog_path.as_deref(), cancel)?) as Box<dyn Renderer>)
                }
            }),
            preferences,
        ))
    }
    fn with_loader(root: PathBuf, loader: Loader, preferences: Arc<SharedPreferences>) -> Self {
        Self {
            root,
            preferences,
            loader,
            active: None,
            retiring: None,
            desired: None,
            pending: None,
            published: None,
            failed: None,
            closing: false,
        }
    }
    #[cfg(test)]
    pub(crate) fn with_vip_preview_fixture(
        root: PathBuf,
        map: Arc<super::world_map::WorldMap>,
        effect_catalog_path: PathBuf,
    ) -> Self {
        Self::with_loader(
            root,
            Arc::new(move |root, observation, cancel| {
                Ok(Box::new(VipPreview::load_map(root, observation, map.clone(),
                    Some(&effect_catalog_path), cancel)?) as Box<dyn Renderer>)
            }),
            super::preferences::shared(),
        )
    }
    /// Explicit attachment boundary. Call clear on source/world/epoch changes;
    /// update may NEVER silently rebind to a different authenticated owner.
    pub fn attach(&mut self, observation: Observation) -> Result<()> {
        ensure!(
            !self.closing && self.desired.is_none(),
            "canonical runtime already attached or closing"
        );
        ensure!(
            observation.data().world.phase != Phase::Aborted,
            "cannot attach aborted battle"
        );
        let input = Arc::new(Submission {
            attachment: Attachment::new(self.preferences.snapshot()),
            ordinal: 1,
            observation,
        });
        self.desired = Some(input.clone());
        self.pending = Some(input);
        Ok(())
    }
    /// All fallible admission precedes the only desired/pending replacement.
    /// Aborted retains its terminal observation until an explicit clear, so a
    /// later old response cannot resurrect the scene or bypass chronology.
    pub fn update(&mut self, observation: Observation) -> Result<()> {
        ensure!(!self.closing, "canonical runtime closing");
        let previous = self
            .desired
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("canonical runtime not attached"))?;
        previous.observation.check_successor(&observation)?;
        if previous.observation.data() == observation.data() {
            return Ok(());
        }
        let ordinal = previous
            .ordinal
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("canonical mailbox exhausted"))?;
        let aborted = observation.data().world.phase == Phase::Aborted;
        ensure!(
            aborted || self.failed.is_none(),
            "failed canonical runtime requires explicit retirement"
        );
        let input = Arc::new(Submission {
            attachment: previous.attachment.clone(),
            ordinal,
            observation,
        });
        self.desired = Some(input.clone());
        if aborted {
            self.pending = None;
            self.published = None;
            self.failed = None;
            self.cancel_jobs();
        } else {
            self.pending = Some(input);
        }
        Ok(())
    }
    pub fn observation(&self) -> Option<&Observation> {
        self.desired.as_ref().map(|i| &i.observation)
    }
    /// Binds a real completed surface to this still-live owned renderer.
    /// Retained pixels from a previous clear/re-attach can never qualify.
    pub(crate) fn prepared(&self, surface: &Surface) -> bool {
        if self.closing || self.failed.is_some() {
            return false;
        }
        let (Some(desired), Some(output)) = (&self.desired, &self.published) else {
            return false;
        };
        desired.observation.data().world.phase != Phase::Aborted
            && Arc::ptr_eq(&desired.attachment, &output.input.attachment)
            && std::ptr::eq(surface, output.surface.as_ref())
            && output
                .resources
                .as_ref()
                .is_some_and(|r| r.matches(&surface.observation))
            && surface
                .navigation
                .as_ref()
                .is_some_and(|n| n.matches(&surface.observation))
    }
    /// No I/O or send: only a confirmed LOADING scene can yield a request.
    /// Clock-only server reads may advance while loading, but no actor, gear,
    /// manifest, owner or scene change is allowed to reuse an earlier proof.
    pub(crate) fn loaded_request(
        &self,
        surface: &Surface,
        current: &Observation,
    ) -> Option<super::battle_world::Request> {
        let desired = self.observation()?;
        if !self.prepared(surface)
            || surface.effects_skipped
            || !super::battle_player::same_loading_picture(&surface.observation, current)
            || !super::battle_player::same_loading_picture(desired, current)
            || current
                .data()
                .actors
                .0
                .iter()
                .find(|a| a.entity == current.data().personal.entity)?
                .loaded
        {
            return None;
        }
        let stamp = super::battle_window::Stamp::of(current);
        let request = super::battle_world::Request {
            action: super::battle_world::Action::Loaded,
            digest: current.data().manifest.digest,
            ..super::battle_world::Request::read(
                stamp.context,
                stamp.channel,
                Some(&current.data().world.identity),
            )
        };
        request.validate().ok()?;
        Some(request)
    }
    /// Only the latest completed input of this still-live attachment may
    /// supply route intent. Painting may lag an update; navigation may not.
    /// No mailbox lock, geometry work, command send or LOAD ACK occurs here.
    pub fn navigation(&self) -> Option<Arc<super::battle_space::Navigation>> {
        if self.closing || self.failed.is_some() {
            return None;
        }
        let desired = self.desired.as_ref()?;
        if desired.observation.data().world.phase == Phase::Aborted {
            return None;
        }
        let output = self.published.as_ref()?;
        if output.surface.observation.data().vip.is_some() {
            return None; // preview map/occupancy is never a route permission
        }
        if output.input.ordinal != desired.ordinal
            || !Arc::ptr_eq(&output.input.attachment, &desired.attachment)
        {
            return None;
        }
        output
            .surface
            .navigation
            .as_ref()
            .filter(|n| n.matches(&desired.observation))
            .cloned()
    }
    /// Reuse STATIC terrain from this attachment while new pixels are in
    /// flight. This deliberately returns no occupancy, route, life or MOVE
    /// permission. The normal ACK-driven planner must take all of those from
    /// the latest independently confirmed command snapshot.
    pub(crate) fn routing_map(
        &self,
        current: &Observation,
    ) -> Option<Arc<super::world_map::WorldMap>> {
        if self.closing || self.failed.is_some() {
            return None;
        }
        let desired = self.desired.as_ref()?;
        if current.data().world.phase != Phase::Active
            || desired.observation.data() != current.data()
        {
            return None;
        }
        let output = self.published.as_ref()?;
        if output.surface.observation.data().vip.is_some() {
            return None;
        }
        if !Arc::ptr_eq(&output.input.attachment, &desired.attachment)
            || output.surface.observation.data().manifest != current.data().manifest
        {
            return None;
        }
        output
            .surface
            .navigation
            .as_ref()
            .filter(|n| n.matches(&output.surface.observation))
            .map(|n| n.shared_map())
    }
    /// Read-only publication view; never copies/scales PCM or locks a mailbox.
    /// The output adapter binds to the live attachment, not caller-supplied
    /// world IDs or retained Surface objects from another selection.
    pub(crate) fn audio_view(&self) -> AudioView<'_> {
        let desired = self.desired.as_ref().filter(|d| {
            !self.closing
                && self.failed.is_none()
                && d.observation.data().world.phase != Phase::Aborted
        });
        let published = self
            .published
            .as_ref()
            .filter(|p| desired.is_some_and(|d| Arc::ptr_eq(&d.attachment, &p.input.attachment)));
        AudioView {
            attachment: desired.map(|d| &d.attachment),
            publication: published.map(|p| (&p.input.attachment, &p.audio)),
            preferences: self.preferences.snapshot(),
            playable: desired.is_some_and(|d| {
                matches!(
                    d.observation.data().world.phase,
                    Phase::Active | Phase::Finished
                )
            }) && published.is_some_and(|p| {
                matches!(
                    p.surface.observation.data().world.phase,
                    Phase::Active | Phase::Finished
                )
            }),
        }
    }
    pub fn audio_notice(&self) -> Option<&'static str> {
        self.audio_view().publication.and_then(|(_, p)| p.notice)
    }
    /// Call while unfocused too. Only try_lock and already-finished joins.
    pub fn poll(&mut self) -> Status {
        self.reap();
        if self.closing {
            return if self.is_quiescent() {
                Status::Idle
            } else {
                Status::Stopping
            };
        }
        if self
            .desired
            .as_ref()
            .is_some_and(|i| i.observation.data().world.phase == Phase::Aborted)
        {
            return Status::Retired;
        }
        if self.failed.is_none() {
            self.flush();
            self.read_output();
        }
        if let Some(message) = self.failed {
            return Status::Failed(message);
        }
        if let Some(output) = &self.published {
            return Status::Ready(output.surface.clone());
        }
        if self.desired.is_some() {
            Status::Loading
        } else {
            Status::Idle
        }
    }
    fn fail(&mut self, message: &'static str) {
        self.failed = Some(message);
        self.pending = None;
        self.published = None;
        self.cancel_jobs();
    }
    fn reap(&mut self) {
        if self.retiring.as_ref().is_some_and(Job::finished) {
            if let Some(mut job) = self.retiring.take() {
                let _ = job.join();
            }
        }
        if self.active.as_ref().is_some_and(Job::finished) {
            if let Some(mut job) = self.active.take() {
                let cancelled = job.cancel.load(Ordering::Acquire);
                let result = job.join();
                let current = self
                    .desired
                    .as_ref()
                    .is_some_and(|i| Arc::ptr_eq(&i.attachment, &job.attachment));
                if !cancelled && current && !self.closing {
                    self.fail(
                        result
                            .err()
                            .unwrap_or("canonical worker exited unexpectedly"),
                    );
                }
            }
        }
    }
    fn flush(&mut self) {
        let Some(pending) = self.pending.clone() else {
            return;
        };
        if let Some(active) = &self.active {
            if !Arc::ptr_eq(&active.attachment, &pending.attachment)
                || active.cancel.load(Ordering::Acquire)
            {
                active.cancel();
                if self.retiring.is_some() {
                    return;
                }
                self.retiring = self.active.take();
            }
        }
        if let Some(active) = &self.active {
            let result = match active.input.try_lock() {
                Ok(mut mailbox) => {
                    let old = std::mem::replace(&mut *mailbox, pending);
                    drop(mailbox);
                    drop(old);
                    self.pending = None;
                    Ok(())
                }
                Err(TryLockError::WouldBlock) => Ok(()),
                Err(TryLockError::Poisoned(_)) => Err("canonical input mailbox failed"),
            };
            if let Err(message) = result {
                self.fail(message);
            }
            return;
        }
        match Job::spawn(
            self.root.clone(),
            pending,
            self.loader.clone(),
            self.preferences.clone(),
        ) {
            Ok(job) => {
                self.active = Some(job);
                self.pending = None;
            }
            Err(_) => self.fail("canonical worker could not start"),
        }
    }
    fn read_output(&mut self) {
        let (Some(active), Some(desired)) = (&self.active, &self.desired) else {
            return;
        };
        if active.cancel.load(Ordering::Acquire)
            || !Arc::ptr_eq(&active.attachment, &desired.attachment)
        {
            return;
        }
        let result = match active.output.try_lock() {
            Ok(output) => {
                if let Some(output) = output.as_ref() {
                    // Same-attachment completed observations may lag the latest
                    // input to avoid starving under continuous clock updates.
                    // Pixels/private state stay together; never rewind output.
                    if Arc::ptr_eq(&output.input.attachment, &desired.attachment)
                        && output.input.ordinal <= desired.ordinal
                        && self
                            .published
                            .as_ref()
                            .is_none_or(|old| output.input.ordinal >= old.input.ordinal)
                    {
                        self.published = Some(output.clone());
                    }
                }
                Ok(())
            }
            Err(TryLockError::WouldBlock) => Ok(()),
            Err(TryLockError::Poisoned(_)) => Err("canonical output mailbox failed"),
        };
        if let Err(message) = result {
            self.fail(message);
        }
    }
    fn cancel_jobs(&self) {
        if let Some(job) = &self.active {
            job.cancel();
        }
        if let Some(job) = &self.retiring {
            job.cancel();
        }
    }
    /// Immediate UI retirement; workers stay owned until poll joins them.
    pub fn clear(&mut self) {
        self.desired = None;
        self.pending = None;
        self.published = None;
        self.failed = None;
        self.cancel_jobs();
    }
    pub fn shutdown(&mut self) {
        self.closing = true;
        self.clear();
    }
    pub fn is_quiescent(&self) -> bool {
        self.active.is_none() && self.retiring.is_none() && self.pending.is_none()
    }
}
impl Drop for BattleRuntime {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(mut job) = self.active.take() {
            let _ = job.join();
        }
        if let Some(mut job) = self.retiring.take() {
            let _ = job.join();
        }
    }
}
