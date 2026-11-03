"""Keep the native running walk ahead of a queued hurt action; no product writes."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[1];dst=ROOT/'維護工具/candidates/native-class2-presentation'
pins=json.loads((ROOT/'驗證紀錄/current/native77-lawful/validation-20261017-195458-430865/compiled-inputs.json').read_text(encoding='utf-8-sig'))
def source(name):
    p=ROOT/'LiTo381-Modern/src/modern_lito'/name
    assert hashlib.sha256(p.read_bytes()).hexdigest().upper()==pins[p.relative_to(ROOT).as_posix()]
    return p.read_text(encoding='utf-8-sig')
s=source('native_combat_animation.rs');anchor='    pub fn retain(&mut self, ids: &std::collections::BTreeSet<u32>) {';assert s.count(anchor)==1
s=s.replace(anchor,'''    /// A native running walk owns the body until its boundary. Impact may
    /// enqueue hurt meanwhile, but walk wall time must not age that action.
    pub(crate) fn hold_for_motion(&mut self,actor:&Actor,now:u64)->Result<()> {
        let state=self.states.entry(actor.id).or_insert(State{
            incarnation:actor.incarnation,seen:0,last:now,speed:actor.speed,brave:actor.brave,
            active:None,queue:VecDeque::new(),
        });
        ensure!(now>=state.last,"combat clock reversed while walking");
        if state.incarnation!=actor.incarnation || actor.action_sequence<state.seen {
            state.incarnation=actor.incarnation;state.seen=0;state.active=None;state.queue.clear();
        }
        state.last=now;state.speed=actor.speed;state.brave=actor.brave;Ok(())
    }
'''+anchor)
(dst/'native_combat_animation.rs').write_text(s,encoding='utf-8',newline='\n')
s=source('native_world_animation.rs')
old='''        let (combat_pose, mut entered_events) = if guardian {
            (None, Vec::new())
        } else { self.combat.sample(catalog, actor, now)? };
''';assert s.count(old)==1;s=s.replace(old,'')
anchor='        let (action, phase, time, repeat) = if let Some(combat) = combat_pose {';assert s.count(anchor)==1
s=s.replace(anchor,'''        // Original44E6B70 recognizes a running walk and44ED2B enqueues
        // hurt through4E63C0. Do not snap to the destination for hurt and then
        // resume an older partial walk. The motion/camera clock remains live.
        let (combat_pose, mut entered_events)=if guardian {
            (None,Vec::new())
        } else if moving && elapsed<walk_duration && !actor.dead {
            self.combat.hold_for_motion(actor,now)?;(None,Vec::new())
        } else {self.combat.sample(catalog,actor,now)?};
'''+anchor)
(dst/'native_world_animation.rs').write_text(s,encoding='utf-8',newline='\n')
print('Prepared two animator overrides; water rewind guard unchanged')
