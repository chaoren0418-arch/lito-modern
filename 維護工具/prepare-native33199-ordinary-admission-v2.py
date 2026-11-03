"""Isolated metadata-only animation and wire admission; never edit normal sources."""
from pathlib import Path
from datetime import datetime,timezone
import json,hashlib
ROOT=Path(__file__).resolve().parents[1];P=ROOT/'LiTo381-Modern';T=ROOT/'維護工具'
out=T/'candidates/native33199-ordinary-admission-v2-ready';out.mkdir(exist_ok=False)
old=T/'candidates/native33199-wire-admission'
names=['native_wire_audio.rs','native_world_view.rs','native_world_scene.rs','native_world_animation.rs','native_combat_animation.rs']
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest().upper()
originals={};candidates={}
def replace(s,a,b):
    assert s.count(a)==1,(a,s.count(a));return s.replace(a,b,1)
for name in names:
    original=P/'src/modern_lito'/name
    originals[name]={'path':str(original),'sha256':sha(original)}
    s=(old/name if (old/name).exists() else original).read_text('utf-8-sig')
    if name=='native_combat_animation.rs':
        s=replace(s,'struct Active {','#[derive(Clone)]\nstruct Active {')
        s=replace(s,'struct State {','#[derive(Clone)]\nstruct State {')
        s=replace(s,'#[derive(Default)]\npub struct Playback {','#[derive(Clone,Default)]\npub(crate) struct History { states:BTreeMap<u32,State> }\n#[derive(Default)]\npub struct Playback {')
        s=replace(s,'impl Playback {','''impl Playback {
    pub(crate) fn history(&self)->History {History{states:self.states.clone()}}
    pub(crate) fn restore(&mut self,history:History){self.states=history.states;self.clips.clear();}
    pub(crate) fn admit(&mut self,actor:&Actor,sequence:u64,now:u64) {
        let state=self.states.entry(actor.id).or_insert(State{incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,active:None,queue:VecDeque::new()});
        if state.incarnation!=actor.incarnation {*state=State{incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,active:None,queue:VecDeque::new()};}
        state.seen=state.seen.max(sequence);
        state.queue.retain(|cue|cue.sequence>sequence);
        // An admitted death stays on its final pose without entering old events.
        if actor.dead && !state.active.as_ref().is_some_and(|active|active.cue.action==8) {
            state.active=Some(Active{cue:ActionCue{sequence,action:8,source:"action"},units:u128::MAX,emitted:usize::MAX});
        }
    }
''')
    elif name=='native_world_animation.rs':
        s=replace(s,'struct Track {','#[derive(Clone)]\nstruct Track {')
        s=replace(s,'#[derive(Default)]\npub struct Animator {','''#[derive(Clone,Default)]
pub(crate) struct Admission { actors:BTreeMap<u32,(u64,u64,u64)> }
impl Admission {
    pub(crate) fn admit(&mut self,world:&super::native_world::World) {
        self.actors.retain(|id,_|world.actors.contains_key(id));
        for actor in world.actors.values() {self.actors.insert(actor.id,(actor.incarnation,actor.action_sequence,actor.motion_revision));}
    }
}
#[derive(Clone)]
pub(crate) struct History {combat:super::native_combat_animation::History,tracks:BTreeMap<u32,Track>,last_ns:Option<u64>,retired:Admission}
#[derive(Default)]
pub struct Animator {''')
        s=replace(s,'    last_ns: Option<u64>,','    last_ns: Option<u64>,\n    retired:Admission,')
        s=replace(s,'impl Animator {','''impl Animator {
    pub(crate) fn history(&self)->History {History{combat:self.combat.history(),tracks:self.tracks.clone(),last_ns:self.last_ns,retired:self.retired.clone()}}
    pub(crate) fn restore(&mut self,history:History){self.combat.restore(history.combat);self.tracks=history.tracks;self.last_ns=history.last_ns;self.retired=history.retired;self.clips.clear();}
    pub(crate) fn retire_admitted(&mut self,admission:Admission,world:&super::native_world::World,now:u64){
        for actor in world.actors.values(){
            if let Some(&(incarnation,sequence,motion))=admission.actors.get(&actor.id).filter(|v|v.0==actor.incarnation){
                self.combat.admit(actor,sequence,now);
                if super::native_obstacles::guardian(actor) && !self.tracks.contains_key(&actor.id) && motion==actor.motion_revision {
                    self.tracks.insert(actor.id,Track{signature:(incarnation,motion,actor.graphics,actor.x,actor.y,actor.walk_action),last:now,virtual_units:0,speed:actor.speed,brave:actor.brave,from:actor.motion_from,object_emitted:usize::MAX});
                }
            }
        }
        self.retired=admission;
    }
''')
        s=replace(s,'        let (action, phase, time, repeat) =','''        if let Some(&(_,floor,_))=self.retired.actors.get(&actor.id).filter(|v|v.0==actor.incarnation) {entered_events.retain(|event|event.sequence>floor);}
        let (action, phase, time, repeat) =''')
        s=replace(s,'        let clip = self.clip(catalog, id, action)?;','        let retired_motion=self.retired.actors.get(&actor.id).filter(|v|v.0==actor.incarnation).map(|v|v.2);\n        let clip = self.clip(catalog, id, action)?;')
        s=replace(s,'        let direction = if clip.animation.directionality','        if guardian {if let Some(motion)=retired_motion {entered_events.retain(|event|event.sequence>motion);}}\n        let direction = if clip.animation.directionality')
    elif name=='native_world_scene.rs':
        s=replace(s,'water_history,None,None)','water_history,None,None,None)')
        s=replace(s,'        wire_audio:Option<super::native_wire_audio::Cursor>,','        wire_audio:Option<super::native_wire_audio::Cursor>,\n        ordinary_admission:Option<super::native_world_animation::Admission>,')
        s=replace(s,'                    if !reused {if let Some(history)=finite_history {model.restore_finite_history(history);}}','''                    if !reused {
                        if let Some(history)=finite_history {model.restore_finite_history(history);}
                        if let Some(admission)=ordinary_admission {model.animator.retire_admitted(admission,&world,elapsed_ns);}
                    }''')
        s=replace(s,'pub(crate) struct FiniteHistory {','pub(crate) struct FiniteHistory {\n    animation:super::native_world_animation::History,')
        s=replace(s,'self.cone_cold.merge(&other.cone_cold);}','self.cone_cold.merge(&other.cone_cold);self.animation=other.animation.clone();}')
        s=replace(s,'FiniteHistory{skills:','FiniteHistory{animation:self.animator.history(),skills:')
        s=replace(s,'    fn restore_finite_history(&mut self,history:FiniteHistory) {','    fn restore_finite_history(&mut self,history:FiniteHistory) {\n        self.animator.restore(history.animation);')
        # Terrain residence reload is not scene retirement: preserve clocks as is.
        needle='next.wire_audio=self.wire_audio.clone();'
        s=replace(s,needle,needle+'next.animator.restore(self.animator.history());')
    elif name=='native_world_view.rs':
        s=replace(s,'    ordinary_wire_cursor:super::native_wire_audio::Cursor,','    ordinary_wire_cursor:super::native_wire_audio::Cursor,\n    ordinary_admission:super::native_world_animation::Admission,')
        s=replace(s,'self.ordinary_wire_cursor=Default::default();','self.ordinary_wire_cursor=Default::default();self.ordinary_admission=Default::default();')
        s=replace(s,'                    Some(self.ordinary_wire_cursor.clone()),','                    Some(self.ordinary_wire_cursor.clone()),\n                    Some(self.ordinary_admission.clone()),')
        s=replace(s,'Ok(job) => {self.ordinary_wire_cursor.admit(world);','Ok(job) => {self.ordinary_wire_cursor.admit(world);self.ordinary_admission.admit(world);')
    path=out/name;path.write_text(s,encoding='utf-8',newline='\n');candidates[name]={'path':str(path),'sha256':sha(path)}
record={'source_manifest_sha256':'33199F4ED6584FD4D19FBF1EC69C3EC69AB6E62B27ABA64CC59FE0AD2C986ED4','original_files':originals,'candidate_files':candidates,'normal_sources_modified':False,'candidate_applied':False,'qualified':False,'created_utc':datetime.now(timezone.utc).isoformat()}
(out/'preparation.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(str(out))
