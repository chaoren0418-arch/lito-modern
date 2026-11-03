//! Borrowed effect submissions. Sampling owns clocks; this queue only paints.
//! Effects are click-through, matching the existing presentation policy.
//! Equal-key order follows the modern profile submission order, not native
//! timer insertion history. Qualified Over/attr104=8 Screen and RGB555 Shadow are accepted.
use super::{
    compositor::Canvas,
    sprite::SpriteFrame,
    world_map::{ViewProjection, WorldCell},
};
use anyhow::{ensure, Context, Result};

#[derive(Clone, Copy)]
pub enum Mix {
    Over,
    Screen,
    Shadow,
    Multiply,
}
pub struct Effect<'a> {
    pub frame: &'a SpriteFrame,
    pub position: WorldCell,
    pub mix: Mix,
}
pub enum Target<'a, 's> {
    Canvas(&'a mut Canvas),
    Fragments(&'a mut Vec<Effect<'s>>),
}
impl<'s> Target<'_, 's> {
    pub fn submit_batch(&mut self, frames: &[(&'s SpriteFrame, Mix)], position: WorldCell, view: ViewProjection) -> Result<()> {
        let anchor=view.cell_to_pixel(position).context("effect projection")?;
        if let Self::Fragments(queue)=self {ensure!(queue.len().checked_add(frames.len()).is_some_and(|n|n<=512),"effect submission budget");}
        for (frame,_) in frames {
            anchor.x.checked_add(frame.x_offset).context("effect X")?;
            anchor.y.checked_add(frame.y_offset).context("effect Y")?;
            if matches!(self,Self::Fragments(_)){ensure!(frame.fragments.is_some(),"effect lacks source fragments");}
        }
        for &(frame,mix) in frames {self.submit(frame,position,view,mix)?;}
        Ok(())
    }
    pub fn submit(
        &mut self,
        frame: &'s SpriteFrame,
        position: WorldCell,
        view: ViewProjection,
        mix: Mix,
    ) -> Result<()> {
        let anchor = view.cell_to_pixel(position).context("effect projection")?;
        let x = anchor.x.checked_add(frame.x_offset).context("effect X")?;
        let y = anchor.y.checked_add(frame.y_offset).context("effect Y")?;
        match self {
            Self::Canvas(canvas) => match mix {
                Mix::Over => canvas.blit_rgba(&frame.rgba, frame.width, frame.height, x, y)?,
                Mix::Screen => {
                    canvas.blit_screen_rgb555(&frame.rgba, frame.width, frame.height, x, y)?
                }
                Mix::Shadow => canvas.blit_native_shadow_rgb555(&frame.rgba,frame.width,frame.height,x,y)?,
                Mix::Multiply => canvas.blit_multiply_rgb555(&frame.rgba,frame.width,frame.height,x,y)?,
            },
            Self::Fragments(queue) => {
                ensure!(queue.len() < 512, "effect submission budget");
                ensure!(frame.fragments.is_some(), "effect lacks source fragments");
                queue.push(Effect {
                    frame,
                    position,
                    mix,
                });
            }
        }
        Ok(())
    }
}
