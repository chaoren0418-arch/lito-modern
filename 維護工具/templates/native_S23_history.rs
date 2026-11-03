
/// Metadata only: action admission and path/cursor state, never SPR/PCM/files.
#[derive(Clone)]
pub(crate) struct History {
    generation:Option<u64>,seen:BTreeSet<(u32,u64)>,flights:Vec<Flight>,
    launched:u64,arrived:u64,omitted:u64,
}
impl History {
    pub(crate) fn merge_admission(&mut self,other:&Self) {
        if self.generation==other.generation {self.seen.extend(other.seen.iter().copied());}
    }
}

