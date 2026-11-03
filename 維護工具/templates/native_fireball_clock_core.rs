
/// Class5 42F300/42F550 movement dispatch, separate from class4's rational
/// inspection timeline. This owns metadata only; callers retain actor fences.
#[derive(Clone)]
pub(crate) struct NativeAreaFireballFlight {
    path: LightArrowPath,
    animation: super::native_effect_timing::Playback,
    last_ns: Option<u64>,
    next_ms: Option<u64>,
    movement_ns: u64,
    callbacks: u64,
    frame: usize,
    arrived: bool,
}
pub(crate) struct NativeAreaFireballStep {
    pub position: WorldCell,
    pub heading: u8,
    pub source_frame: usize,
    pub arrived_now: bool,
    pub arrived: bool,
    pub next_callback_ms: Option<u64>,
    pub callbacks: u64,
    /// On arrival, dispatch the resolved recipients' 218 starts BEFORE these
    /// 171 cues (42F4CB precedes 42F500). No Hit is invented for 218.
    pub flight_cues_after_arrival: Vec<CrossedCue>,
}
impl NativeAreaFireballFlight {
    fn new(path: LightArrowPath, animation: Arc<Animation>) -> anyhow::Result<Self> {
        Ok(Self {path, animation: super::native_effect_timing::Clip::new(animation)?.start(),
            last_ns: None, next_ms: Some(0), movement_ns: 0, callbacks: 0,
            frame: 1, arrived: false})
    }
    pub(crate) fn advance(&mut self, elapsed_ns: u64) -> anyhow::Result<NativeAreaFireballStep> {
        anyhow::ensure!(self.last_ns.is_none_or(|last| elapsed_ns >= last), "class5 clock regressed");
        let now_ms = elapsed_ns / 1_000_000;
        let mut arrived_now = false;
        let mut cues = Vec::new();
        if self.next_ms.is_some_and(|due| now_ms >= due) {
            // Initial movement is immediate. A late dispatch advances ONE step
            // and schedules from NOW; elapsed wall time cannot skip cells.
            if self.callbacks != 0 {self.movement_ns += STEP_NANOS;}
            self.callbacks += 1;
            arrived_now = self.path.position(Duration::from_nanos(self.movement_ns)) == self.path.target();
            self.arrived = arrived_now;
            self.next_ms = if arrived_now {None} else {Some(now_ms + 60)};
            let sample = self.animation.advance(now_ms * 1_000_000)?;
            self.frame = sample.source_frame; // Hold terminal0 until arrival.
            cues = sample.crossed_cues;
        }
        self.last_ns = Some(elapsed_ns);
        Ok(NativeAreaFireballStep {position: self.path.position(Duration::from_nanos(self.movement_ns)),
            heading: self.path.heading(), source_frame: self.frame, arrived_now,
            arrived: self.arrived, next_callback_ms: self.next_ms, callbacks: self.callbacks,
            flight_cues_after_arrival: cues})
    }
}
