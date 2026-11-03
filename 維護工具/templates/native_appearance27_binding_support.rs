#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub(crate) struct Motion {pub x:u16,pub y:u16,pub heading:u8,pub from:Option<[u16;2]>,pub revision:u64,pub predicted:bool}
impl Motion {
 pub(crate) fn capture(a:&super::native_world::Actor)->Self{Self{x:a.x,y:a.y,heading:a.heading,from:a.motion_from,revision:a.motion_revision,predicted:a.position_predicted}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub(crate) struct Snapshot {pub body:Body,pub motion:Motion}
impl Snapshot {
 fn apply(&mut self,sequence:u64,operation:&Operation){match operation{
  Operation::Appearance(p)=>{self.body=Body{graphics:p.graphics,family:p.family};self.motion.from=None;self.motion.revision=sequence;},
  Operation::Motion(m)=>self.motion=*m,
  Operation::Heading(heading)=>{self.motion.heading=*heading;self.motion.from=None;self.motion.revision=sequence;},
  Operation::Action(_)|Operation::ServerAction{..}=>{self.motion.from=None;self.motion.revision=sequence;},
 }}
}
/// Bounded wire/prediction metadata. Only a new S_PUT_OBJECT/S_RESURRECT life
/// removes it; repeated S27 and A->B->A retain their actual event ordering.
#[derive(Clone,Debug,serde::Serialize)]
pub(crate) struct Tape {pub initial:Snapshot,pub entries:VecDeque<Entry>,pub dropped_through:u64}
impl Tape {
 pub(crate) fn capture(a:&super::native_world::Actor)->Self {
  Self{initial:Snapshot{body:Body{graphics:a.graphics,family:a.walk_action},motion:Motion::capture(a)},entries:a.actions.iter().map(|c|Entry{sequence:c.sequence,operation:Operation::ServerAction{action:c.action,source:c.source}}).collect(),dropped_through:0}
 }
 pub(crate) fn record(&mut self,entry:Entry){
  if self.entries.len()==128{let old=self.entries.pop_front().unwrap();self.initial.apply(old.sequence,&old.operation);self.dropped_through=old.sequence;}
  self.entries.push_back(entry);
 }
 pub(crate) fn snapshot_at(&self,floor:u64)->Result<Snapshot>{ensure!(floor>=self.dropped_through,"body tape gap before admission floor");let mut s=self.initial;for e in self.entries.iter().filter(|e|e.sequence<=floor){s.apply(e.sequence,&e.operation);}Ok(s)}
}
impl Queue {
 pub(crate) fn last_admitted(&self)->u64{self.history.last}
 pub(crate) fn skip_through(&mut self,floor:u64){self.history.pending.retain(|e|e.sequence>floor);self.history.last=self.history.last.max(floor);}
 pub(crate) fn prune(&mut self,floor:u64){let pending:std::collections::BTreeSet<_>=self.history.pending.iter().map(|e|e.sequence).collect();self.history.seen.retain(|sequence,_|*sequence>floor||pending.contains(sequence));}
 fn same_life(a:&Owner,b:&Owner)->bool{(a.connection,a.generation,a.map,a.actor,a.incarnation)==(b.connection,b.generation,b.map,b.actor,b.incarnation)}
 pub(crate) fn rebase(&mut self,owner:Owner,body:Body,floor:u64)->Result<()>{ensure!(Self::same_life(&self.history.owner,&owner)&&owner.source_revision>self.history.owner.source_revision&&self.history.source_retired,"body source rebase identity");self.history.owner=owner;self.history.body=body;self.history.pending.clear();self.history.last=self.history.last.max(floor);self.history.source_retired=false;Ok(())}
 pub(crate) fn merge_late(&mut self,other:&Self)->Result<()>{ensure!(Self::same_life(&self.history.owner,&other.history.owner),"late body admission life");let mut facts=other.history.clone();facts.owner=self.history.owner.clone();self.merge_admission(&facts)}
}
#[derive(Clone)]
pub(crate) struct Presentation {pub owner:Owner,pub queue:Queue,pub current:Snapshot,pub ceiling:u64,pub released:VecDeque<super::native_combat::ActionCue>,pub deferred:bool,pub callback_due:Option<u64>,pub needs_rebase:bool,pub admission_floor:u64}
impl Presentation {
 pub(crate) fn new(owner:Owner,tape:&Tape,floor:u64)->Result<Self>{let current=tape.snapshot_at(floor)?;let mut queue=Queue::new(owner.clone(),current.body)?;queue.skip_through(floor);Ok(Self{owner,queue,current,ceiling:floor,released:VecDeque::new(),deferred:false,callback_due:None,needs_rebase:false,admission_floor:floor})}
 pub(crate) fn retire(&mut self){self.queue.retire_source();self.needs_rebase=true;self.released.clear();self.callback_due=None;}
 pub(crate) fn merge_late(&mut self,other:&Self)->Result<()>{self.queue.merge_late(&other.queue)}
 pub(crate) fn ingest(&mut self,tape:&Tape)->Result<()>{let floor=self.queue.last_admitted();ensure!(floor>=tape.dropped_through,"unobserved body tape evicted");self.queue.prune(tape.dropped_through);for e in tape.entries.iter().filter(|e|e.sequence>floor){self.queue.admit(&self.owner,e.clone())?;}Ok(())}
 pub(crate) fn view(&self,a:&super::native_world::Actor)->Result<super::native_world::Actor>{
  let body=self.queue.body().context("retired body graph")?;let graphics=Packet{actor:a.id,graphics:body.graphics,family:body.family}.shared_graphics()?;
  let mut view=a.clone();view.graphics=graphics;view.walk_action=body.family;view.x=self.current.motion.x;view.y=self.current.motion.y;view.heading=self.current.motion.heading;view.motion_from=self.current.motion.from;view.motion_revision=self.current.motion.revision;view.position_predicted=self.current.motion.predicted;view.action_sequence=self.ceiling;view.actions=self.released.iter().cloned().collect();view.appearance=None;Ok(view)
 }
 pub(crate) fn apply(&mut self,step:Dispatch){match step{
  Dispatch::Action{sequence,action,..}=>self.release(sequence,action,"action"),
  Dispatch::ServerAction{sequence,action,source,..}=>self.release(sequence,action,source),
  Dispatch::Appearance{sequence,current,..}=>{self.current.body=current;self.current.motion.from=None;self.current.motion.revision=sequence;self.ceiling=self.ceiling.max(sequence);},
  Dispatch::Motion{sequence,motion,..}=>{self.current.motion=motion;self.ceiling=self.ceiling.max(sequence);},
  Dispatch::Heading{sequence,heading,..}=>{self.current.motion.heading=heading;self.current.motion.from=None;self.current.motion.revision=sequence;self.ceiling=self.ceiling.max(sequence);},
 }}
 fn release(&mut self,sequence:u64,action:u8,source:&'static str){if self.released.len()==16{self.released.pop_front();}self.released.push_back(super::native_combat::ActionCue{sequence,action,source});self.ceiling=self.ceiling.max(sequence);self.current.motion.from=None;self.current.motion.revision=sequence;}
}
