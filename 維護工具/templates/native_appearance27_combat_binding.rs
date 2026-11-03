impl Playback {
 pub(crate) fn seen_for(&self,a:&Actor)->u64{self.states.get(&a.id).filter(|s|s.incarnation==a.incarnation).map_or(0,|s|s.seen)}
 pub(crate) fn body_busy(&self,a:&Actor)->bool{self.states.get(&a.id).filter(|s|s.incarnation==a.incarnation).is_some_and(|s|s.active.is_some()||!s.queue.is_empty())}
 pub(crate) fn clear_body(&mut self,a:&Actor,floor:u64,now:u64){self.states.remove(&a.id);self.admit(a,floor,now);}
}
