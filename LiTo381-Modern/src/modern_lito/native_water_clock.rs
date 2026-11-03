//! Pure Rust shared water/green callbacks, deadline ordering and visual history.
//! No original instructions or function addresses are linked into the client.
use anyhow::{ensure,Result};
const BUDGET:usize=4096;
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub enum Callback {Water,Green,MapAux,Kept(u8),User(u32)}
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Event {pub due:u32,pub callback:Callback,pub owner:u32}
#[derive(Clone,Debug,Default,PartialEq,Eq,serde::Serialize)]
pub struct Core {
    pub enabled:bool,
    phase:i8,
    counter:u8,
    green_counter:u32,
    queue:Vec<Event>,
}
impl Core {
    pub fn with_values(phase:i8,counter:u8)->Result<Self> {
        ensure!((-20..=21).contains(&phase)&&counter<100,"water values outside native callback domain");
        Ok(Self{phase,counter,..Default::default()})
    }
    pub fn phase(&self)->i8 {self.phase}
    pub fn counter(&self)->u8 {self.counter}
    pub fn with_green_counter(counter:u32)->Self {Self{green_counter:counter,..Default::default()}}
    pub fn green_counter(&self)->u32 {self.green_counter}
    pub fn events(&self)->&[Event] {&self.queue}
    pub fn schedule(&mut self,delay:u32,callback:Callback,owner:u32,mut clock:impl FnMut()->u32)->Result<()> {
        ensure!(self.queue.len()<BUDGET,"water callback queue budget");
        ensure!(!matches!(callback,Callback::Kept(i) if i>=9),"water keep table domain");
        let event=Event{due:clock().wrapping_add(delay),callback,owner};
        let at=self.queue.partition_point(|old|old.due<=event.due);
        self.queue.insert(at,event);Ok(())
    }
    pub fn cancel(&mut self,callback:Callback,owner:u32) {
        self.queue.retain(|event|event.callback!=callback||event.owner!=owner);
    }
    pub fn water_callback(&mut self,clock:impl FnMut()->u32)->Result<()> {
        if !self.enabled {self.counter=0;self.phase=0;return Ok(());}
        // Stage the only fallible operation before changing the native values.
        self.schedule(100,Callback::Water,0,clock)?;
        self.counter=if self.counter==99 {0}else{self.counter+1};
        self.phase=if self.phase>20 {-20}else{self.phase+1};Ok(())
    }
    /// Original4878F0 increments before reading the fresh50ms schedule clock.
    /// Preflight the host budget so a rejected operation publishes no partial state.
    pub fn green_callback(&mut self,clock:impl FnMut()->u32)->Result<()> {
        ensure!(self.queue.len()<BUDGET,"visual callback queue budget");
        self.green_counter=self.green_counter.wrapping_add(1);
        self.schedule(50,Callback::Green,0,clock)
    }
    pub fn ensure_map_aux(&mut self,clock:impl FnMut()->u32)->Result<()> {
        if !self.queue.iter().any(|e|e.callback==Callback::MapAux&&e.owner==u32::MAX) {
            self.schedule(10_000,Callback::MapAux,u32::MAX,clock)?;
        }
        Ok(())
    }
    pub fn map_filter(&mut self,clock:impl FnMut()->u32)->Result<()> {
        let mut next=self.clone();next.queue.retain(|e|matches!(e.callback,Callback::Kept(_)));
        next.ensure_map_aux(clock)?;*self=next;Ok(())
    }
    pub fn map_loaded(&mut self,mut clock:impl FnMut()->u32)->Result<()> {
        let mut next=self.clone();next.map_filter(&mut clock)?;
        if next.enabled {next.schedule(50,Callback::Water,0,&mut clock)?;}
        *self=next;Ok(())
    }
    /// Native scene cleanup4D6470 retains the first7 callbacks, unlike the
    /// map-load filter's7+2. Visual values survive; water/aux callbacks do not.
    pub fn retire_scene(&mut self) {
        self.enabled=false;self.queue.retain(|e|matches!(e.callback,Callback::Kept(i) if i<7));
    }
    pub fn water_packet(&mut self,enabled:bool,clock:impl FnMut()->u32)->Result<()> {
        if enabled {self.schedule(50,Callback::Water,0,clock)?;}else{self.cancel(Callback::Water,0);}
        self.enabled=enabled;Ok(())
    }
    /// One dispatch cutoff, fresh clock per reschedule, no elapsed-time catchup.
    /// Bound a pathological u32-wrap callback storm instead of hanging the UI.
    pub fn dispatch(&mut self,mut clock:impl FnMut()->u32)->Result<Vec<Event>> {
        let cutoff=clock();
        if self.queue.first().is_none_or(|e|e.due>cutoff) {return Ok(Vec::new());}
        let mut next=self.clone();let mut fired=Vec::new();
        while next.queue.first().is_some_and(|e|e.due<=cutoff) {
            ensure!(fired.len()<BUDGET,"water callback dispatch budget / clock wrap");
            let event=next.queue.remove(0);fired.push(event);
            match event.callback {
                Callback::Water=>next.water_callback(&mut clock)?,
                Callback::Green=>next.green_callback(&mut clock)?,
                Callback::MapAux=>next.ensure_map_aux(&mut clock)?,
                Callback::Kept(_)|Callback::User(_)=>{},
            }
        }
        *self=next;Ok(fired)
    }
}

/// Incremental scroll is independent of the water enable flag. The native
/// helper adds one period before signed division; these bounds keep that
/// numerator nonnegative, matching the retained camera/vector domain.
pub fn scroll(values:[u16;2],delta:[i32;2])->Result<[u16;2]> {
    ensure!(values[0]<640&&values[1]<420&&(-640..=640).contains(&delta[0])
        &&(-420..=420).contains(&delta[1]),"water scroll domain");
    Ok([((i32::from(values[0])+delta[0]+640)%640) as u16,
        ((i32::from(values[1])+delta[1]+420)%420) as u16])
}

#[derive(Clone,Debug,Default,PartialEq,Eq,serde::Serialize)]
pub struct Owned {
    owner:Option<u64>,
    retired_through:u64,
    generation:Option<u64>,
    retired_generation:u64,
    pub core:Core,
}
impl Owned {
    pub fn owner(&self)->Option<u64> {self.owner}
    pub fn begin(&mut self,epoch:u64)->Result<()> {
        ensure!(epoch!=0,"zero water GAME owner");
        if self.owner==Some(epoch) {return Ok(());}
        let last=self.retired_through.max(self.owner.unwrap_or(0));
        ensure!(epoch>last,"retired water GAME owner");
        // Keep process visual values; retire authority and all old callbacks.
        self.core.enabled=false;self.core.queue.clear();self.owner=Some(epoch);
        self.retired_through=last;self.generation=None;self.retired_generation=0;Ok(())
    }
    pub fn retire(&mut self) {
        self.retired_through=self.retired_through.max(self.owner.unwrap_or(0));
        self.retired_generation=self.retired_generation.max(self.generation.unwrap_or(0));
        self.owner=None;self.generation=None;self.core.enabled=false;self.core.queue.clear();
    }
    pub fn retire_scene(&mut self) {
        self.retired_generation=self.retired_generation.max(self.generation.unwrap_or(0));
        self.core.retire_scene();self.generation=None;
    }
    fn require_owner(&self,epoch:u64)->Result<()> {
        ensure!(epoch!=0&&self.owner==Some(epoch),"stale water GAME owner");Ok(())
    }
    pub fn packet(&mut self,epoch:u64,enabled:bool,clock:impl FnMut()->u32)->Result<()> {
        self.require_owner(epoch)?;self.core.water_packet(enabled,clock)
    }
    pub fn loaded(&mut self,epoch:u64,generation:u64,mut clock:impl FnMut()->u32)->Result<bool> {
        self.require_owner(epoch)?;
        if self.generation==Some(generation) {return Ok(false);}
        ensure!(generation>self.retired_generation.max(self.generation.unwrap_or(0)),
            "retired or older visual scene generation");
        // Accepted world-ready generation: map filtering precedes the native
        // immediate green start. Same-generation asset reloads do not restart it.
        let mut next=self.core.clone();
        next.map_loaded(&mut clock)?;next.green_callback(&mut clock)?;
        self.core=next;self.generation=Some(generation);Ok(true)
    }
    pub fn green_for(&self,epoch:u64,generation:u64)->Result<u32> {
        self.require_owner(epoch)?;
        ensure!(self.generation==Some(generation),"green phase belongs to another world generation");
        Ok(self.core.green_counter())
    }
    pub fn tick(&mut self,epoch:u64,clock:impl FnMut()->u32)->Result<bool> {
        self.require_owner(epoch)?;Ok(!self.core.dispatch(clock)?.is_empty())
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Motion {
    pub identity:[u64;3], // actor ID, incarnation, motion revision
    pub source:[u16;2],
    pub destination:[u16;2],
    pub offset:[i32;2],
    pub extent:[i32;2],
}
#[derive(Clone,Debug,Default,PartialEq,Eq,serde::Serialize)]
pub struct History {
    owner:Option<u64>,retired_through:u64,generation:Option<u64>,
    values:[u16;2],motion:Option<Motion>,
}
impl History {
    pub fn owner(&self)->Option<u64> {self.owner}
    pub fn values(&self)->[u16;2] {self.values}
    pub fn bind(&mut self,epoch:u64)->Result<()> {
        ensure!(epoch!=0,"zero water scroll owner");
        if self.owner==Some(epoch) {return Ok(());}
        let last=self.retired_through.max(self.owner.unwrap_or(0));
        ensure!(epoch>last,"retired water scroll owner");
        self.owner=Some(epoch);self.retired_through=last;self.generation=None;self.motion=None;Ok(())
    }
    pub fn retire(&mut self) {
        self.retired_through=self.retired_through.max(self.owner.unwrap_or(0));
        self.owner=None;self.generation=None;self.motion=None;
    }
    pub fn detach_scene(&mut self) {self.owner=None;self.generation=None;self.motion=None;}
    /// Offset uses the same rounded projection as the camera. No additional
    /// original walking offset is applied, and map/correction jumps add none.
    pub fn sample(&mut self,epoch:u64,generation:u64,motion:Option<Motion>)->Result<[u16;2]> {
        ensure!(self.owner==Some(epoch)&&epoch!=0,"stale water scroll sample");
        let mut next=self.clone();
        if next.generation!=Some(generation) {next.generation=Some(generation);next.motion=None;}
        if let Some(m)=motion {
            let dx=i32::from(m.destination[0])-i32::from(m.source[0]);
            let dy=i32::from(m.destination[1])-i32::from(m.source[1]);
            ensure!(m.source[0].abs_diff(m.destination[0])<=1&&m.source[1].abs_diff(m.destination[1])<=1
                &&m.source!=m.destination&&m.extent==[24*(dx+dy),12*(-dx+dy)],"water motion extent");
            ensure!((0..2).all(|i|m.offset[i]>=m.extent[i].min(0)&&m.offset[i]<=m.extent[i].max(0)),"water motion offset");
            let mut applied=[0,0];
            if let Some(old)=next.motion {
                if old.identity==m.identity {
                    ensure!(old.source==m.source&&old.destination==m.destination&&old.extent==m.extent,"water motion identity reused");
                    ensure!((0..2).all(|i|m.offset[i].abs()>=old.offset[i].abs()),"water motion rewound");
                    applied=old.offset;
                } else if old.identity[..2]==m.identity[..2]&&old.identity[2]<m.identity[2]&&old.destination==m.source {
                    next.values=scroll(next.values,[old.offset[0]-old.extent[0],old.offset[1]-old.extent[1]])?;
                }
            }
            next.values=scroll(next.values,[applied[0]-m.offset[0],applied[1]-m.offset[1]])?;
            next.motion=Some(m);
        }else{next.motion=None;}
        let values=next.values;*self=next;Ok(values)
    }
    pub fn validate_owner(&self,epoch:u64)->Result<()> {
        ensure!(self.owner==Some(epoch)&&epoch!=0,"water history owner mismatch");Ok(())
    }
}
