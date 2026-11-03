/// Immutable binding of a validated static S37 cue to its actual worker receipt.
/// World JSON omits this local clock; neither asset load nor painting creates it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StaticEffectReceipt {
    ingress: super::native_connection::IngressReceipt,
    generation: u64,
    sequence: u64,
    graphics: u16,
    target: Option<(u32, u64)>,
    position: [u16; 2],
}
impl StaticEffectReceipt {
    pub fn ingress(&self) -> super::native_connection::IngressReceipt { self.ingress }
    pub fn matches(&self, generation: u64, connection: u64, cue: &super::native_skills::Effect) -> bool {
        self.generation == generation && self.ingress.connection_epoch() == connection
            && self.sequence == cue.sequence && self.graphics == cue.graphics
            && self.target == cue.target && self.position == cue.position
    }
}
impl World {
    /// Actual LOGIN-controller receive route. Untimed inspection callers keep
    /// using accept(); that path never invents a receipt from a new Instant.
    pub(crate) fn accept_received(
        &mut self, body: &[u8], ingress: super::native_connection::IngressReceipt,
    ) -> Result<bool> {
        ensure!(ingress.connection_epoch() != 0, "static effect without live connection epoch");
        let previous = self.visual_revision;
        let handled = self.accept(body)?;
        if handled && body.first() == Some(&0x37) && self.visual_revision != previous {
            if let Some(cue) = self.effects.last().filter(|cue|
                cue.graphics == 2941 && cue.sequence == self.visual_revision) {
                self.static_effect_receipts.insert(cue.sequence, StaticEffectReceipt {
                    ingress, generation: self.generation, sequence: cue.sequence,
                    graphics: cue.graphics, target: cue.target, position: cue.position,
                });
            }
        }
        Ok(handled)
    }
}
