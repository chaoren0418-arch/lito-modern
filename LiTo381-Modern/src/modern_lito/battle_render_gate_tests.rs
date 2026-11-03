//! Test-only bounded stalls around the real shared-resource renderer. They never
//! changes observations, pixels, clocks, network commands or admission rules.
use super::*;
use std::sync::atomic::{AtomicU64, AtomicU8, AtomicUsize};

pub(crate) struct Pace {
    delay: Duration,
    waits: AtomicUsize,
    frames: AtomicUsize,
    waited_millis: AtomicU64,
}
impl Pace {
    fn new(delay: Duration) -> Self {
        Self {
            delay,
            waits: AtomicUsize::new(0),
            frames: AtomicUsize::new(0),
            waited_millis: AtomicU64::new(0),
        }
    }
    fn before_render(&self, cancel: &AtomicBool) -> Result<()> {
        let started = Instant::now();
        self.waits.fetch_add(1, Ordering::AcqRel);
        loop {
            ensure!(
                !cancel.load(Ordering::Acquire),
                "test render pace cancelled"
            );
            let elapsed = started.elapsed();
            if elapsed >= self.delay {
                self.waited_millis
                    .fetch_add(elapsed.as_millis() as u64, Ordering::AcqRel);
                return Ok(());
            }
            thread::park_timeout((self.delay - elapsed).min(Duration::from_millis(5)));
        }
    }
    pub(crate) fn completed_frames(&self) -> usize {
        self.frames.load(Ordering::Acquire)
    }
    pub(crate) fn waited_millis(&self) -> u64 {
        self.waited_millis.load(Ordering::Acquire)
    }
}

#[derive(Default)]
pub(crate) struct Gate(AtomicU8);
impl Gate {
    pub(crate) fn arm(&self) {
        assert!(self
            .0
            .compare_exchange(0, 1, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
    }
    pub(crate) fn holding(&self) -> bool {
        self.0.load(Ordering::Acquire) == 2
    }
    pub(crate) fn release(&self) {
        assert!(self
            .0
            .compare_exchange(2, 3, Ordering::AcqRel, Ordering::Acquire)
            .is_ok());
    }
    pub(crate) fn completed(&self) -> bool {
        self.0.load(Ordering::Acquire) == 4
    }
    fn before_render(&self, cancel: &AtomicBool) -> Result<()> {
        if self
            .0
            .compare_exchange(1, 2, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return Ok(());
        }
        // The ordinary 180 s test deadline is unchanged. A forgotten release
        // fails this explicit gate instead of stranding renderer shutdown.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if cancel.load(Ordering::Acquire) {
                self.0.store(5, Ordering::Release);
                anyhow::bail!("test render gate cancelled");
            }
            if self.0.load(Ordering::Acquire) == 3 {
                self.0.store(4, Ordering::Release);
                return Ok(());
            }
            if Instant::now() >= deadline {
                self.0.store(5, Ordering::Release);
                anyhow::bail!("test render gate was not released");
            }
            thread::park_timeout(Duration::from_millis(5));
        }
    }
}

struct GatedScene {
    scene: SharedScene,
    gate: Arc<Gate>,
    pace: Option<Arc<Pace>>,
}
impl Renderer for GatedScene {
    fn prepared_resources(&self) -> Option<super::super::battle_scene::PreparedResources> {
        self.scene.prepared_resources()
    }
    fn requires_reload(&self, next: &Observation) -> Result<bool> {
        self.scene.requires_reload(next)
    }
    fn observe(&mut self, next: Observation, cancel: &AtomicBool) -> Result<()> {
        self.scene.observe(next, cancel)
    }
    fn render(&mut self, _: Instant, cancel: &AtomicBool) -> Result<Surface> {
        self.gate.before_render(cancel)?;
        if let Some(pace) = &self.pace {
            pace.before_render(cancel)?;
        }
        // Use real current monotonic time after the gate, not an invented or
        // rewound simulation clock. Rendering and resources stay production.
        let surface = self.scene.render(Instant::now(), cancel)?;
        if let Some(pace) = &self.pace {
            pace.frames.fetch_add(1, Ordering::AcqRel);
        }
        Ok(surface)
    }
    fn reload(
        &mut self,
        root: &Path,
        next: Observation,
        cancel: &AtomicBool,
        _: &Loader,
    ) -> Result<Box<dyn Renderer>> {
        let mut scene = SharedScene::load(root, next, cancel)?;
        scene.carry_from(&mut self.scene, Instant::now(), cancel)?;
        Ok(Box::new(Self {
            scene,
            gate: self.gate.clone(),
            pace: self.pace.clone(),
        }))
    }
}

pub(crate) fn shared(root: PathBuf) -> Result<(BattleRuntime, Arc<Gate>)> {
    controlled(root, None)
}

pub(crate) fn paced_shared(root: PathBuf) -> Result<(BattleRuntime, Arc<Pace>)> {
    let pace = Arc::new(Pace::new(Duration::from_millis(350)));
    let (runtime, _) = controlled(root, Some(pace.clone()))?;
    Ok((runtime, pace))
}

fn controlled(root: PathBuf, pace: Option<Arc<Pace>>) -> Result<(BattleRuntime, Arc<Gate>)> {
    ensure!(
        root.is_absolute(),
        "absolute test shared-resource root required"
    );
    let gate = Arc::new(Gate::default());
    let renderer_gate = gate.clone();
    let runtime = BattleRuntime::with_loader(
        root,
        Arc::new(move |root, observation, cancel| {
            Ok(Box::new(GatedScene {
                scene: SharedScene::load(root, observation, cancel)?,
                gate: renderer_gate.clone(),
                pace: pace.clone(),
            }))
        }),
        super::super::preferences::shared(),
    );
    Ok((runtime, gate))
}

#[test]
fn render_pace_waits_each_time_and_cancels_without_completing_a_frame() {
    let pace = Arc::new(Pace::new(Duration::from_millis(20)));
    let cancel = Arc::new(AtomicBool::new(false));
    let started = Instant::now();
    for _ in 0..2 {
        pace.before_render(&cancel).unwrap();
    }
    assert!(started.elapsed() >= Duration::from_millis(40));
    assert_eq!(pace.waits.load(Ordering::Acquire), 2);
    assert!(pace.waited_millis() >= 40);
    assert_eq!(pace.completed_frames(), 0, "waiting is not real painting");

    let pace = Arc::new(Pace::new(Duration::from_secs(1)));
    let child_pace = pace.clone();
    let child_cancel = cancel.clone();
    let worker = thread::spawn(move || child_pace.before_render(&child_cancel));
    let deadline = Instant::now() + Duration::from_secs(2);
    while pace.waits.load(Ordering::Acquire) == 0 && Instant::now() < deadline {
        thread::park_timeout(Duration::from_millis(1));
    }
    cancel.store(true, Ordering::Release);
    assert!(worker.join().unwrap().is_err());
    assert_eq!(pace.waits.load(Ordering::Acquire), 1);
    assert_eq!(pace.waited_millis(), 0);
    assert_eq!(pace.completed_frames(), 0);
}

#[test]
fn render_gate_is_one_shot_and_cancellable_without_resources() {
    for cancel_instead in [false, true] {
        let gate = Arc::new(Gate::default());
        let cancel = Arc::new(AtomicBool::new(false));
        gate.before_render(&cancel).unwrap();
        assert!(!gate.holding() && !gate.completed());
        gate.arm();
        let child_gate = gate.clone();
        let child_cancel = cancel.clone();
        let worker = thread::spawn(move || child_gate.before_render(&child_cancel));
        let deadline = Instant::now() + Duration::from_secs(2);
        while !gate.holding() {
            if Instant::now() >= deadline {
                cancel.store(true, Ordering::Release);
                let _ = worker.join();
                panic!("test gate worker did not enter");
            }
            thread::park_timeout(Duration::from_millis(1));
        }
        assert!(!gate.completed());
        if cancel_instead {
            cancel.store(true, Ordering::Release);
        } else {
            gate.release();
        }
        let result = worker.join().unwrap();
        assert_eq!(result.is_ok(), !cancel_instead);
        assert_eq!(gate.completed(), !cancel_instead);
        if !cancel_instead {
            gate.before_render(&cancel).unwrap();
            assert!(gate.completed(), "subsequent frames cannot block again");
        }
    }
}
