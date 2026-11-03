    fn native_S23_profile(&self)->Result<()> {
        if !matches!((self.effect.gfx,self.impact.gfx),(167,219)|(1583,218)|(1799,1800)|(1797,1798)) {return Err(LightArrowError::Unsupported("native S23 profile"));}Ok(())
    }
    pub(crate) fn start_native_S23(&self,path:&LightArrowPath,secondary:bool)->Result<NativeSpellClock> {
        self.native_S23_profile()?;
        NativeSpellClock::new(path.clone(),self.effect.gfx as u16,self.impact.gfx as u16,Arc::new(self.effect.sequence.animation.clone()),Arc::new(self.impact.sequence.animation.clone()),secondary).map_err(LightArrowError::Render)
    }
    pub(crate) fn native_S23_frame(&self,gfx:u16,heading:u8,source_frame:usize,finished:bool)->Result<VisualSample<'_>> {
        self.native_S23_profile()?;let bound=if u32::from(gfx)==self.effect.gfx {&self.effect}else if u32::from(gfx)==self.impact.gfx {&self.impact}else{return Err(LightArrowError::Unsupported("native S23 graphic"));};
        self.frame_source(bound,heading,source_frame,finished)
    }
    pub(crate) fn native_S23_events(&self,gfx:u16,source_frame:usize)->Result<&[FrameEvent]> {
        self.native_S23_profile()?;let bound=if u32::from(gfx)==self.effect.gfx {&self.effect}else if u32::from(gfx)==self.impact.gfx {&self.impact}else{return Err(LightArrowError::Unsupported("native S23 event graphic"));};
        Ok(&bound.sequence.animation.frames.get(source_frame).ok_or(LightArrowError::InvalidInput("native S23 event source frame"))?.events)
    }
