//! Once-only direct S4A cues. Other S4A kinds remain visible as unsupported.
use super::{native_combat_animation::EnteredEvent, native_world::World};

#[derive(Clone, Default)]
pub struct Cursor {
    generation: Option<u64>,
    sequence: u64,
}
pub struct Batch {
    pub events: Vec<EnteredEvent>,
    pub dropped: usize,
    pub unsupported: usize,
}
impl Cursor {
    /// Submission consumes the captured ring even if that scene is retired
    /// before its worker can publish. No sound or authority is changed here.
    pub(crate) fn admit(&mut self,world:&World) {
        if self.generation!=Some(world.generation) {
            self.generation=Some(world.generation);self.sequence=0;
        }
        self.sequence=self.sequence.max(world.wire_sound_sequence);
    }
    pub fn take(&mut self, world: &World) -> Batch {
        if self.generation != Some(world.generation) {
            self.generation=Some(world.generation);self.sequence=0;
        }
        let mut batch=Batch{events:Vec::new(),dropped:0,unsupported:0};
        for cue in world.wire_sounds.iter().filter(|c|c.sequence>self.sequence) {
            if cue.kind!=0 {batch.unsupported+=1;continue;}
            if batch.events.len()==32 {batch.dropped+=1;continue;}
            batch.events.push(EnteredEvent{ prepared_audio:None,actor:0,sequence:cue.sequence,action:0,frame:0,
                event:format!("WireSound({})",cue.sound),sound:Some(u32::from(cue.sound))});
        }
        self.admit(world);
        batch
    }
}
