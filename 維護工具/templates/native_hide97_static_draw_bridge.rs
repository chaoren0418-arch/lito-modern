// Raw static2941 has no attr104 override: constructor flags are zero after
// the catalog's zero initialization. attr101 shadow and attr112 direction6
// aliases retain -1. This bridge chooses the established normal Over path;
// it does not invent a sound, shadow, duplicate bank, or animation clock.
impl Static2941Bank {
    pub(crate) fn native_primary_flags(&self) -> u16 { 0 }

    /// The scene must independently establish a live, admitted retirement
    /// window and captured location before calling this resource-only bridge.
    /// Submitting a frame neither admits S37 nor consumes its due callback.
    pub(crate) fn submit_frame_zero<'s>(
        &'s self,
        target: &mut super::native_world_effects::Target<'_, 's>,
        position: super::world_map::WorldCell,
        view: super::world_map::ViewProjection,
        cancel: &AtomicBool,
    ) -> anyhow::Result<()> {
        active(cancel)?;
        target.submit(self.frame_zero(), position, view, super::native_world_effects::Mix::Over)
    }
}
