    /// Exact source frames and native callback clips for 171/218 only. The
    /// older class4/ice inspection API keeps its existing timeline contract.
    pub(crate) fn native_area_fireball_clips(&self) -> Result<(super::native_effect_timing::Clip,super::native_effect_timing::Clip)> {
        if self.effect.gfx != 171 || self.impact.gfx != 218 {
            return Err(LightArrowError::Unsupported("native area fireball profile"));
        }
        let flight = super::native_effect_timing::Clip::new(Arc::new(self.effect.sequence.animation.clone())).map_err(LightArrowError::Render)?;
        let impact = super::native_effect_timing::Clip::new(Arc::new(self.impact.sequence.animation.clone())).map_err(LightArrowError::Render)?;
        Ok((flight,impact))
    }
    pub(crate) fn start_native_area_fireball(&self,path:LightArrowPath)->Result<NativeAreaFireballFlight> {
        self.native_area_fireball_clips()?;
        NativeAreaFireballFlight::new(path,Arc::new(self.effect.sequence.animation.clone())).map_err(LightArrowError::Render)
    }
    pub(crate) fn native_area_fireball_frame(&self,gfx:u32,heading:u8,source_frame:usize,finished:bool)->Result<VisualSample<'_>> {
        if self.effect.gfx != 171 || self.impact.gfx != 218 {
            return Err(LightArrowError::Unsupported("native area fireball profile"));
        }
        let bound=match gfx {171=>&self.effect,218=>&self.impact,_=>return Err(LightArrowError::Unsupported("native area fireball graphic"))};
        self.frame_source(bound,heading,source_frame,finished)
    }
