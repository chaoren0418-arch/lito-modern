
#[derive(Clone)]
pub(crate) struct NativeSpellCue {pub graphics:u16,pub source_frame:usize,pub occurrence:u64,pub event:FrameEvent}
#[derive(Clone)]
pub(crate) struct NativeSpellSample {
    pub position:WorldCell,pub heading:u8,pub graphics:u16,pub source_frame:usize,
    pub arrived:bool,pub arrived_now:bool,pub finished:bool,pub callbacks:u64,
    pub next_movement_ms:Option<u64>,pub arrival_ns:Option<u64>,pub cues:Vec<NativeSpellCue>,
}
/// S23 class4/ice movement callback metadata; source FPS ticks do not drive
/// the manually advanced flight selector. The secondary is a native finite clip.
#[derive(Clone)]
pub(crate) struct NativeSpellClock {
    path:LightArrowPath,graphics:u16,impact_graphics:u16,flight:Arc<Animation>,
    impact:super::native_effect_timing::Playback,secondary:bool,last_ns:Option<u64>,
    movement_ns:u64,next_movement_ms:Option<u64>,callbacks:u64,frame:usize,arrival_ns:Option<u64>,
}
impl NativeSpellClock {
    fn new(path:LightArrowPath,gfx:u16,impact_gfx:u16,flight:Arc<Animation>,impact:Arc<Animation>,secondary:bool)->anyhow::Result<Self> {
        let (expected,count,sound,direction)=match gfx {167=>(219,4,97,Directionality::EightWay),1583=>(218,4,97,Directionality::EightWay),1799=>(1800,5,719,Directionality::Undirected),1797=>(1798,4,719,Directionality::EightWay),_=>anyhow::bail!("unqualified native S23 profile")};
        anyhow::ensure!(impact_gfx==expected&&flight.frames_per_second==24&&flight.directionality==direction&&flight.frames.len()==count
            &&flight.frames.iter().enumerate().all(|(n,f)|f.events==if n==1{vec![FrameEvent::Sound(sound)]}else{vec![]}),"native S23 flight definition changed");
        Ok(Self{path,graphics:gfx,impact_graphics:impact_gfx,flight,
            impact:super::native_effect_timing::Clip::new(impact)?.start(),secondary,last_ns:None,
            movement_ns:0,next_movement_ms:Some(0),callbacks:0,frame:0,arrival_ns:None})
    }
    pub(crate) fn advance(&mut self,elapsed_ns:u64)->anyhow::Result<NativeSpellSample> {
        anyhow::ensure!(self.last_ns.is_none_or(|last|elapsed_ns>=last),"native S23 clock regressed");
        let now_ms=elapsed_ns/1_000_000;let mut flight_cues=vec![];let mut cues=vec![];let mut arrived_now=false;
        if self.next_movement_ms.is_some_and(|due|now_ms>=due) {
            if self.callbacks!=0 {self.movement_ns+=STEP_NANOS;}self.callbacks+=1;
            arrived_now=self.path.position(Duration::from_nanos(self.movement_ns))==self.path.target();
            self.next_movement_ms=if arrived_now{None}else{Some(now_ms+60)};
            if arrived_now{self.arrival_ns=Some(elapsed_ns);}
            self.frame=(self.frame+1)%self.flight.frames.len();
            flight_cues.extend(self.flight.frames[self.frame].events.iter().cloned().map(|event|NativeSpellCue{graphics:self.graphics,source_frame:self.frame,occurrence:self.callbacks,event}));
        }
        // 4246CD precedes42473A (class4); 42F4CB precedes42F500 (class5).
        if self.graphics!=1797 {cues.append(&mut flight_cues);}
        let mut graphics=self.graphics;let mut source_frame=self.frame;let mut finished=false;
        if let Some(arrival)=self.arrival_ns {
            if self.secondary {
                let sample=self.impact.advance((now_ms.saturating_sub(arrival/1_000_000))*1_000_000)?;
                graphics=self.impact_graphics;source_frame=sample.source_frame;finished=sample.finished;
                cues.extend(sample.crossed_cues.into_iter().map(|cue|NativeSpellCue{graphics:self.impact_graphics,source_frame:cue.source_frame,occurrence:1,event:cue.event}));
            }else{finished=true;}
        }
        cues.append(&mut flight_cues);self.last_ns=Some(elapsed_ns);
        Ok(NativeSpellSample{position:self.path.position(Duration::from_nanos(self.movement_ns)),heading:if self.graphics==1797{self.path.heading()}else{self.path.heading()&3},graphics,source_frame,
            arrived:self.arrival_ns.is_some(),arrived_now,finished,callbacks:self.callbacks,next_movement_ms:self.next_movement_ms,arrival_ns:self.arrival_ns,cues})
    }
}
