//! Complete candidate scene proof. The original descriptor, public extensions
//! and private owner data are borrowed intact; no lossy legacy projection.
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
        f.write_str("CheckedArmorBattle[redacted]")
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
    /// Complete value proof against an independently selected owner/world.
    /// Shared resources, authenticated capability and temporal admission are
    /// separate; this does not construct a live battle_follow::Observation.
    pub fn check_values(&self, player: &Player, world: &WorldId) -> Result<(), ApplicationError> {
        self.check_armor_values()?;
        let d = &self.data;
        d.check_armor_context(player, world)?;
        d.check_combat_with_healing(self.healing.as_ref().map(|h| h.0.as_slice()))?;
        d.personal_facts(player, world)?;
        d.check_scene_body()?;
        crate::modern_lito::battle_follow::clocks(d)?;
        match &self.status {
            Status::Motion(m) => {
                super::super::npc_motion::MotionView { data: d, motion: m }.check_motion()?
            }
            Status::Haste(s) => super::super::haste::View {
                data: d,
                effects: s,
            }
            .check_effects()?,
            Status::Speed(s) => {
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
        for h in &self.actors.0 {
            let Some(s) = &h.state else { continue };
            require(s.sequence <= s.revision)?;
            let respawn = d
                .honor
                .lives
                .0
                .iter()
                .find(|l| l.entity == h.entity)
                .map(|l| l.respawn_revision)
                .or_else(|| {
                    d.quest
                        .as_ref()
                        .and_then(|q| q.lives.0.iter().find(|l| l.entity == h.entity))
                        .map(|l| l.respawn_revision)
                })
                .or_else(|| {
                    d.siege
                        .as_ref()
                        .and_then(|q| q.lives.0.iter().find(|l| l.entity == h.entity))
                        .map(|l| l.respawn_revision)
                });
            if let Some(revision) = respawn {
                require(if s.life > 0 && s.sequence == 0 {
                    // Quest resets in the respawn commit; Honor/Siege may
                    // reset during the subsequent expiry pass of that pulse.
                    s.revision >= revision
                } else {
                    s.revision > revision
                })?;
            }
        }
        Ok(())
    }
}
impl Candidate {
    pub fn checked(self, player: &Player, world: &WorldId) -> Result<Checked, ApplicationError> {
        self.check_values(player, world)?;
        let personal = self.data.personal_facts(player, world)?;
        Ok(Checked {
            candidate: Arc::new(self),
            personal,
        })
    }
}
