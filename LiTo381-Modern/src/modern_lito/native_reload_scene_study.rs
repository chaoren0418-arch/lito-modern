//! Finite source48BC80 projection. Private actors retire before a separate
//! process callback table is filtered; kept globals confer no scene authority.
use super::*;
pub(crate) const WHITELIST:[u32;9]=[0x47cec0,0x47ce50,0x47cdf0,0x4159b0,0x4c32a0,0x48d700,0x4878c0,0x4875a0,0x4876e0];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) enum Owner{Zero,Actor(u64),Fire(u64),Roof(u64,usize)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub(crate) struct NativeTimer{pub serial:u64,pub at:u32,pub callback:u32,pub owner:Owner}
#[derive(Clone,Debug,PartialEq,Eq)]pub(crate) struct Source{pub private_incarnation:u64,pub binding:Binding,pub plan:NativeAssetPlan,pub appearance_manifest:[u8;32]}
#[derive(Clone,Debug)]pub(crate) struct Inputs{
 pub at:Option<u32>,pub decoded8315f0:Option<i32>,pub entry48bc80_reached:Option<bool>,pub actor_destructors_reached:Option<bool>,pub resource_transition_reached:Option<bool>,pub registry_cleared:Option<bool>,pub actor_owner_all_cancel_used:Option<bool>,pub native_table_complete_for_projection:Option<bool>,pub native_dispatch_suspended_for_reload:Option<bool>,
 pub flag64bc2b:Option<bool>,pub flag64bc2c:Option<bool>,pub flag64bc2d:Option<bool>,pub flag75a75c:Option<bool>,pub fresh_clocks:Option<Vec<u32>>,
}
struct ProcessIdentity;
pub(crate) struct GlobalTicket{identity:Arc<ProcessIdentity>,pub timer:NativeTimer}
pub(crate) struct Prepared{identity:Arc<ProcessIdentity>,revision:u64,source:Source,scene_generation:u64,scene_last:u32,scene_events:Vec<Event>,nodes:Vec<movement::queue::Node>,original:Vec<NativeTimer>,pub fact:Fact}
#[derive(Clone,Debug)]pub(crate) struct Fact{pub at:u32,pub last_fresh_clock:u32,pub destroyed_nodes:Vec<movement::queue::Node>,pub timers_after_destructor_before_filter:Vec<NativeTimer>,pub special_removed:Vec<NativeTimer>,pub saved47b720_due:u32,pub kept:Vec<NativeTimer>,pub filtered:Vec<NativeTimer>,pub rebuilt:Vec<NativeTimer>,pub order:Vec<&'static str>,pub new_incarnation:Option<u64>}
pub(crate) struct Model<'a>{pub current:super::Model<'a>,identity:Arc<ProcessIdentity>,revision:u64,pub source:Source,pub registry:Option<u64>,pub timers:Vec<NativeTimer>,serial:u64,last:u32,send_profile:Option<input::send_idle::Profile>}
fn insert(table:&mut Vec<NativeTimer>,serial:&mut u64,at:u32,callback:u32,owner:Owner)->NativeTimer{*serial+=1;let t=NativeTimer{serial:*serial,at,callback,owner};let p=table.partition_point(|old|old.at<=at);table.insert(p,t);t}
fn owned(m:&super::Model<'_>,b:&Binding,now:i64,inc:u64)->Result<Vec<(u32,u32,Owner)>>{
 let movement=m.movement.owned_timers(b,now)?;let roof=m.scene.roof.timers(b,now)?;m.scene.events.iter().map(|e|{Ok(match e.kind{
  Kind::Locomotion(s)=>{let t=movement.iter().find(|t|t.serial==s).context("Root movement timer absent")?;let(c,o)=match t.kind{movement::Kind::Step|movement::Kind::Arrival=>(0x4e2150,Owner::Zero),movement::Kind::QueuedActionPop|movement::Kind::IdlePop=>(0x4e9300,Owner::Actor(inc)),movement::Kind::AttackAdvance|movement::Kind::IdleAdvance=>(0x4e34c0,Owner::Actor(inc))};(e.at,c,o)},
  Kind::FireFirst=>(e.at,0x4e9300,Owner::Fire(inc)),Kind::FireAdvance=>(e.at,if m.scene.fire.queued_repeat{0x4e9300}else{0x4e34c0},Owner::Fire(inc)),
  Kind::Roof(s)=>{let t=roof.iter().find(|t|t.serial==s).context("Root roof timer absent")?;(e.at,match t.callback{roof::Callback::Down=>0x432f50,roof::Callback::Up=>0x432f60},Owner::Roof(inc,t.group))},
  _=>anyhow::bail!("Reload projection excludes legacy player/idlecycle roots"),
 })}).collect()
}
impl<'a> Model<'a>{
 pub fn new(current:super::Model<'a>,b:&Binding,now:i64,source:Source,extras:Option<Vec<(u32,u32,Owner)>>)->Result<Self>{
  current.scene.active(b,now)?;ensure!(source.private_incarnation>0&&source.binding==*b&&source.plan==current.scene.identity.plan&&source.appearance_manifest!=[0;32],"Private reload source/manifest identity unknown");
  let list=owned(&current,b,now,source.private_incarnation)?;let last=current.scene.last;let mut table=vec![];let mut serial=0;for(at,callback,owner)in list{insert(&mut table,&mut serial,at,callback,owner);}
  let extras=extras.context("Conditional native global timer table unknown")?;ensure!(extras.len()<=16&&extras.iter().all(|(at,c,_)|*at>last&&(0x400000..0x500000).contains(c)),"Global table clock/address/private bound");for(at,c,o)in extras{insert(&mut table,&mut serial,at,c,o);}
  let registry=Some(source.private_incarnation);Ok(Self{current,identity:Arc::new(ProcessIdentity),revision:0,source,registry,timers:table,serial,last,send_profile:None})
 }
 pub fn ticket(&self,serial:u64)->Result<GlobalTicket>{Ok(GlobalTicket{identity:self.identity.clone(),timer:*self.timers.iter().find(|t|t.serial==serial).context("Global record absent")?})}
 pub fn retained_record(&self,t:&GlobalTicket)->Result<NativeTimer>{ensure!(Arc::ptr_eq(&self.identity,&t.identity)&&self.timers.contains(&t.timer),"Foreign/removed process timer record");Ok(t.timer)}
 pub fn actor_target(&self,t:&GlobalTicket,inc:u64)->Result<NativeTimer>{let t=self.retained_record(t)?;ensure!(self.registry==Some(inc)&&matches!(t.owner,Owner::Actor(v)|Owner::Fire(v) if v==inc),"Old or global timer cannot target new actor scene");Ok(t)}
 pub fn prepare(&self,b:&Binding,now:i64,q:&Inputs)->Result<Prepared>{
  self.current.scene.active(b,now)?;ensure!(self.source.binding==*b&&self.registry==Some(self.source.private_incarnation),"Reload old source registry absent");
  let at=q.at.context("Reload clock unknown")?;ensure!(at>=self.last&&at>=self.current.scene.last&&self.current.scene.events.first().is_some_and(|t|at<t.at)&&q.native_dispatch_suspended_for_reload==Some(true),"Reload crosses owned root or source loop/dispatch order unknown");
  ensure!(q.decoded8315f0.is_some_and(|v|v!=0)&&q.entry48bc80_reached==Some(true)&&q.actor_destructors_reached==Some(true)&&q.resource_transition_reached==Some(true)&&q.registry_cleared==Some(true)&&q.actor_owner_all_cancel_used==Some(false)&&q.native_table_complete_for_projection==Some(true),"Reload decoded gate/destructor/resource/registry/table boundary unknown; actor removal is distinct");
  ensure!(q.flag64bc2b==Some(false)&&q.flag64bc2c==Some(false)&&q.flag64bc2d==Some(false)&&q.flag75a75c==Some(false),"Optional global rebuild flags unqualified");
  let nodes=self.current.movement.queued(b,now)?;let after_destructor=self.timers.clone();let special=self.timers.iter().filter(|t|t.callback==0x482f50&&t.owner==Owner::Zero).copied().collect::<Vec<_>>();let remaining=self.timers.iter().filter(|t|!(t.callback==0x482f50&&t.owner==Owner::Zero)).copied().collect::<Vec<_>>();
  let saved=remaining.iter().find(|t|t.callback==0x47b720&&t.owner==Owner::Zero).map_or(0,|t|t.at);let kept=remaining.iter().filter(|t|WHITELIST.contains(&t.callback)).copied().collect::<Vec<_>>();let filtered=remaining.iter().filter(|t|!WHITELIST.contains(&t.callback)).copied().collect::<Vec<_>>();
  let clocks=q.fresh_clocks.as_ref().context("Reload fresh clocks unknown")?;ensure!(clocks.iter().all(|v|*v>=at)&&clocks.windows(2).all(|p|p[0]<=p[1]),"Reload fresh clocks backward");
  let(delay,schedule_at,ping_at)=if saved==0{ensure!(clocks.len()==2,"No-saved branch requires two fresh schedules");(60000,clocks[0],clocks[1])}else{ensure!(clocks.len()>=3,"Saved timer probe/schedule clocks missing");let delta=i64::from(saved)-i64::from(clocks[0]);ensure!(delta.abs()<=i64::from(i32::MAX),"Signed deadline delta/wrap ambiguous");if delta<1{ensure!(clocks.len()==3,"Expired branch requires three clocks");(1,clocks[1],clocks[2])}else{ensure!(clocks.len()==4&&clocks[1]<saved,"Fresh subtract crossed saved due/signed branch unqualified");(saved-clocks[1],clocks[2],clocks[3])}};
  let mut serial=self.serial;let mut rebuild=vec![];let first=insert(&mut rebuild,&mut serial,schedule_at.checked_add(delay).context("47B720 schedule wraps")?,0x47b720,Owner::Zero);let ping=insert(&mut rebuild,&mut serial,ping_at.checked_add(150000).context("47B750 schedule wraps")?,0x47b750,Owner::Zero);
  let fact=Fact{at,last_fresh_clock:*clocks.last().unwrap(),destroyed_nodes:nodes.clone(),timers_after_destructor_before_filter:after_destructor,special_removed:special,saved47b720_due:saved,kept,filtered,rebuilt:vec![first,ping],order:vec!["OLD_ACTOR_DRAW_UNLINK","RESOURCE_TRANSITION","ACTOR_DESTRUCTORS_QUEUE_FREE_TABLE_UNCHANGED","REGISTRY_AND_GLOBAL_POINTERS_CLEAR","SPECIAL482F50_CANCEL","QUERY47B720","CALLBACK_WHITELIST_FILTER","GLOBAL_CALLBACK_REBUILD"],new_incarnation:None};
  Ok(Prepared{identity:self.identity.clone(),revision:self.revision,source:self.source.clone(),scene_generation:self.current.scene.generation,scene_last:self.current.scene.last,scene_events:self.current.scene.events.clone(),nodes,original:self.timers.clone(),fact})
 }
 pub fn commit(&mut self,b:&Binding,now:i64,mut p:Prepared,mut new:super::Model<'a>,source:Source,q:&movement::queue::Registration,draw:&DrawInputs)->Result<(Frame,Fact)>{
  self.current.scene.active(b,now)?;new.scene.active(b,now)?;draw_inputs(draw)?;
  ensure!(Arc::ptr_eq(&self.identity,&p.identity)&&self.revision==p.revision&&self.source==p.source&&self.timers==p.original&&self.current.scene.generation==p.scene_generation&&self.current.scene.last==p.scene_last&&self.current.scene.events==p.scene_events&&self.current.movement.queued(b,now)?==p.nodes,"Stale/foreign reload preparation");
  ensure!(!Arc::ptr_eq(&self.current.scene.identity,&new.scene.identity)&&source.private_incarnation>0&&source.private_incarnation!=self.source.private_incarnation&&source.binding==*b&&source.plan==new.scene.identity.plan&&source.appearance_manifest==self.source.appearance_manifest&&new.scene.generation==0&&new.scene.pose.is_none(),"New private owner/source/untouched scene not distinct");
  let registration=new.movement.prepare_registration(b,now,q)?;let s=&registration.step;let end=p.fact.rebuilt.iter().map(|t|t.at).min().context("Required new global roots absent")?;let fresh=q.at.context("New source registration clock absent")?;
  ensure!(fresh>=p.fact.at&&fresh>=p.fact.last_fresh_clock&&fresh>=new.scene.last&&new.scene.events.first().is_some_and(|t|fresh<t.at)&&p.fact.kept.iter().all(|t|fresh<t.at)&&fresh<end&&new.scene.roof.cell(b,now)?==s.cell,"New registry/frame clock/roof crosses global or actor callback");
  let pose=Pose{bank:s.bank,sub:s.sub,idle:false,cell:s.cell,redraw:1};let view=projection(s.camera)?;let canvas=paint(&new.scene.roof,None,&new.scene.layers,new.scene.banks,b,now,view,Some(pose),None,0,new.scene.effect,new.scene.fire_cell,&new.scene.identity.retired)?;
  // Preflight all source/asset/CPU output before retiring the old owner. The
  // separate process table remains intact through actor destruction.
  self.current.retire();self.registry=None;self.timers=p.fact.kept.clone();for t in &p.fact.rebuilt{let pos=self.timers.partition_point(|old|old.at<=t.at);self.timers.insert(pos,*t);}self.serial=p.fact.rebuilt.iter().map(|t|t.serial).max().unwrap();
  new.movement.commit(b,now,registration)?;new.scene.pose=Some(pose);new.scene.view=view;new.scene.player_hold_until=u32::MAX;let frame=new.scene.frame(canvas,fresh,"SOURCE_CONDITIONAL_RELOAD_NEW_REGISTRY_BASE46_FRAME0".into());
  self.registry=Some(source.private_incarnation);p.fact.new_incarnation=self.registry;p.fact.order.extend(["NEW_SOURCE_REGISTRY_INSERT_LOCAL_POINTER","NEW_SOURCE_CAMERA_ROOF_BASE46_FRAME0"]);
  for(at,c,o)in owned(&new,b,now,source.private_incarnation)?{insert(&mut self.timers,&mut self.serial,at,c,o);}self.current=new;self.source=source;self.revision+=1;self.last=fresh;Ok((frame,p.fact))
 }
}
#[cfg(test)]#[path="native_reload_scene_study_tests.rs"]mod tests;
#[path="native_input_restore_study.rs"]pub(crate) mod input;
