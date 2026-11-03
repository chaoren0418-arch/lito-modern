//! Qualified S32/S34 and S0B status presentation in an ordinary world view.
//! RGB555 math follows the original lookup tables. Poison phase consumes the
//! shared delivered callback counter; animation elapsed time does not advance it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct State {
    pub green: bool,
    pub dim: bool,
    pub invisible: bool,
    pub suppressed: bool,
    /// Original actor+50: ctor/S11 clear it, S20 action8 sets it.
    /// Unlike generic dead, an existing-ID S0B refresh preserves this flag.
    pub native_light_dead: bool,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub enum Style {
    #[default]
    Normal,
    Green(u8),
    Dim,
    Half,
    Hidden,
}
/// Catalogue flag1 changes the body only. Invisibility still controls the
/// ordinary attachment blend; a body flag alone must not make attachments half.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize)]
pub struct Presentation {
    pub state: Style,
    pub body_half: bool,
}
impl Presentation {
    pub fn body_style(self) -> Style {
        // Native raster tests flag1 before the dim/green branches. Suppression
        // and visibility are resolved before any fragments are submitted.
        if self.body_half && self.state != Style::Hidden {
            Style::Half
        } else {
            self.state
        }
    }
}
impl State {
    pub fn from_status(status: u8) -> Self {
        Self {
            green: status & 1 != 0,
            dim: status & 8 != 0,
            invisible: status & 0x82 == 2,
            suppressed: status & 0x80 != 0,
            native_light_dead: false,
        }
    }
    pub fn visible(self, local: bool) -> bool {
        !self.suppressed && (local || !self.invisible)
    }
    pub fn style(self, local: bool, green_counter: u32) -> Style {
        if !self.visible(local) {
            Style::Hidden
        } else if self.invisible {
            Style::Half
        } else if self.green {
            let phase = (green_counter % 24) as u8;
            Style::Green((12u8.abs_diff(phase) + 16) / 2)
        } else if self.dim {
            Style::Dim
        } else {
            Style::Normal
        }
    }
}
