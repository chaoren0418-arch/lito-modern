
/// Class5 42F300/42F550 movement dispatch, separate from class4's rational
/// inspection timeline. This owns metadata only; callers retain actor fences.
#[derive(Clone)]
pub(crate) struct NativeAreaFireballFlight {
    path: LightArrowPath,
    animation: Arc<Animation>,
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
        anyhow::ensure!(animation.directionality==Directionality::EightWay && animation.frames_per_second==24 && animation.frames.len()==5
            && animation.frames.iter().enumerate().all(|(n,f)|f.image_id==0 && f.subframe==if n==0{4}else{n as u32-1}
                && f.duration_ticks==1 && f.events==if n==1{vec![FrameEvent::Sound(98)]}else{vec![]}), "class5 flight definition changed");
        Ok(Self {path, animation,
            last_ns: None, next_ms: Some(0), movement_ns: 0, callbacks: 0,
            frame: 0, arrived: false})
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
            // 4E5B50 increments on EVERY movement callback, wraps N to 0,
            // then enters that source via 4E5C00. It is a cyclic selector,
            // not an independent 24Hz finite timer and not terminal hold.
            self.frame = (self.frame + 1) % self.animation.frames.len();
            cues.extend(self.animation.frames[self.frame].events.iter().enumerate().map(|(event_index,event)|CrossedCue {
                source_frame:self.frame,event_index,tick:self.callbacks-1,event:event.clone()}));
        }
        self.last_ns = Some(elapsed_ns);
        Ok(NativeAreaFireballStep {position: self.path.position(Duration::from_nanos(self.movement_ns)),
            heading: self.path.heading(), source_frame: self.frame, arrived_now,
            arrived: self.arrived, next_callback_ms: self.next_ms, callbacks: self.callbacks,
            flight_cues_after_arrival: cues})
    }
}
