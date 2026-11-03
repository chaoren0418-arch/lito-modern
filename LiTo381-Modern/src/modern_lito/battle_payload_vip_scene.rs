//! One complete, immutable value proof. No stripped descriptor, partial legacy
//! observation or fabricated armor state is used for an absent feature.
use super::super::personal::PersonalView;
use super::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct Checked {
    candidate: Arc<Candidate>,
    personal: PersonalView,
}
impl std::fmt::Debug for Checked {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("CheckedVipBattle[redacted]")
    }
}
impl Checked {
    pub fn candidate(&self) -> &Candidate {
        &self.candidate
    }
    pub fn personal(&self) -> &PersonalView {
        &self.personal
    }
}
impl View<'_> {
    /// Exact owner/world are supplied independently by the caller. This proves
    /// complete values, not SQL persistence, capabilities or loaded resources.
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.check_binding(player, world)?;
        let d = &self.data;
        d.check_vip_context(player, world)?;
        d.check_combat_with_healing(self.healing.as_ref().map(|h| h.0.as_slice()))?;
        super::super::personal::vip_view_facts(self, player, world)?;
        d.check_scene_body()?;
        crate::modern_lito::battle_follow::clocks(d)?;
        match &self.status {
            healing::Status::Motion(m) => {
                super::super::npc_motion::MotionView { data: d, motion: m }.check_motion()?
            }
            healing::Status::Haste(s) => super::super::haste::View {
                data: d,
                effects: s,
            }
            .check_effects()?,
            healing::Status::Speed(s) => {
                let view = super::super::speed::View {
                    data: d,
                    effects: s,
                };
                view.check_components()?;
                view.check_effects()?;
            }
        }
        if let Some(actors) = &self.healing {
            healing::View {
                data: d,
                status: &self.status,
                actors,
            }
            .check_healing_scene()?;
        }
        for a in &self.armor.0 {
            if let Some(s) = &a.state {
                require(s.sequence <= s.revision)?;
            }
        }
        Ok(())
    }
}
impl Candidate {
    pub fn checked(self, player: &Player, world: &WorldId) -> Result<Checked, ApplicationError> {
        self.check_values(player, world)?;
        let personal = super::super::personal::vip_facts(&self, player, world)?;
        Ok(Checked {
            candidate: Arc::new(self),
            personal,
        })
    }
}
