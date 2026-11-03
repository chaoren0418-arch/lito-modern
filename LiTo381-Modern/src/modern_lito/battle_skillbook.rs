//! Skill-page choices are IDs bound to painted selection/life, never row indices.
//! Choosing a target spell does not cast it. Instant spells use existing inputs.
use super::{
    battle_follow::Observation,
    battle_input, battle_player,
    battle_scene::Surface,
    battle_window::{Intent, Kind, Local, Skill, Stamp, Target},
    login_window::LoginSnapshot,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Choice {
    pub stamp: Stamp,
    pub id: u32,
}
impl Choice {
    pub(crate) fn matches(self, o: &Observation) -> bool {
        self.stamp == Stamp::of(o) && o.personal().skills().iter().any(|s| s.id == self.id as i32)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Selection {
    Target(Skill),
    Cast(Intent),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Use {
    Target(Skill),
    Local(Local),
}
fn use_for(id: u32) -> Option<Use> {
    Some(match id {
        1 => Use::Target(Skill::Heal),
        4 => Use::Target(Skill::LightArrow),
        5 => Use::Local(Local::Teleport),
        54 => Use::Local(Local::GreaterHaste),
        43 => Use::Target(Skill::Haste),
        29 => Use::Target(Skill::Slow),
        70 => Use::Local(Local::FireStorm),
        77 => Use::Target(Skill::Disintegrate),
        _ => return None,
    })
}
fn kind(use_: Use, o: &Observation) -> Kind {
    // Used only by the existing readiness policy, not as a cast target.
    let target = if o.data().npcs.is_some() {
        Target::Npc { serial: 1, life: 1 }
    } else {
        Target::Player { entity: 1, life: 1 }
    };
    match use_ {
        Use::Local(local) => Kind::Local(local),
        Use::Target(Skill::LightArrow) => Kind::LightArrow(target),
        Use::Target(Skill::Haste) => Kind::Haste(target),
        Use::Target(Skill::Slow) => Kind::Slow(target),
        Use::Target(Skill::Disintegrate) => Kind::Disintegrate(target),
        Use::Target(Skill::Heal) => Kind::Heal(Target::Honor { entity: 3 }),
        Use::Target(Skill::Melee) => unreachable!("not a learned spell"),
    }
}
fn available(id: u32, surface: &Surface, view: &LoginSnapshot) -> bool {
    use_for(id).is_some_and(|u| {
        battle_player::controls_ready(surface, view)
            && battle_input::skill_available(id, view)
            && ready(u, &surface.observation, view)
    })
}
fn ready(use_: Use, o: &Observation, view: &LoginSnapshot) -> bool {
    if use_ == Use::Target(Skill::Heal) {
        battle_input::healing_ready(view)
    } else {
        battle_input::ready(kind(use_, o), view)
    }
}
fn row_label(id: u32, name: &str, status: &str) -> String {
    format!(
        "#{} {} · {}",
        id,
        name.chars()
            .filter(|c| !c.is_control())
            .take(240)
            .collect::<String>(),
        status
    )
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Row {
    pub id: u32,
    pub label: String,
    pub button: &'static str,
    pub choice: Option<Choice>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Model {
    pub stamp: Stamp,
    pub rows: Vec<Row>,
}
impl Model {
    pub(crate) fn same_owner(&self, other: &Self) -> bool {
        let mut a = self.stamp;
        let mut b = other.stamp;
        a.revision = 0;
        b.revision = 0;
        a == b
    }
    pub(crate) fn choice(&self, id: u32) -> Option<Choice> {
        self.rows.iter().find(|r| r.id == id)?.choice
    }
    pub(crate) fn clear_proofs(&mut self) {
        for row in &mut self.rows {
            row.choice = None;
        }
    }
}
pub(crate) fn model(
    surface: &Surface,
    view: &LoginSnapshot,
    label: impl Fn(&str) -> String,
) -> Option<Model> {
    if !view.canonical.supported() || !battle_player::admitted(surface, view) {
        return None;
    }
    let stamp = Stamp::of(&surface.observation);
    // Terrain/owner validation is per page, not once for every imported skill.
    let controls_ready = battle_player::controls_ready(surface, view);
    let rows = surface
        .observation
        .personal()
        .skills()
        .iter()
        .map(|s| {
            let id = s.id as u32;
            let use_ = use_for(id);
            let supported = use_.is_some() && battle_input::skill_available(id, view);
            let available = controls_ready
                && supported
                && use_.is_some_and(|u| ready(u, &surface.observation, view));
            let status = if available {
                "可操作；仍須伺服器確認"
            } else if use_.is_none() {
                "尚未實作"
            } else if !supported {
                "本場未開放或來源未資格化"
            } else {
                "等待確認或施法條件不足"
            };
            Row {
                id,
                label: row_label(id, &label(&s.name.0), status),
                button: if matches!(use_, Some(Use::Target(_))) {
                    "選擇目標"
                } else {
                    "施放"
                },
                choice: available.then_some(Choice { stamp, id }),
            }
        })
        .collect();
    Some(Model { stamp, rows })
}
/// Re-evaluate the chosen ID against the retained pixels and latest server
/// selection. Never upgrade a stale page click by substituting a new stamp.
pub(crate) fn select(choice: Choice, surface: &Surface, view: &LoginSnapshot) -> Option<Selection> {
    if !choice.matches(&surface.observation) || !available(choice.id, surface, view) {
        return None;
    }
    Some(match use_for(choice.id)? {
        Use::Target(skill) => Selection::Target(skill),
        Use::Local(local) => Selection::Cast(Intent {
            stamp: choice.stamp,
            kind: Kind::Local(local),
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_labels_are_bounded_unicode_without_control_notifications() {
        let text = row_label(77, &format!("\0\n\r\t{}", "術🪄".repeat(1000)), "尚未實作");
        assert!(!text.chars().any(char::is_control));
        assert!(text.encode_utf16().count() < 1024);
        assert!(text.starts_with("#77 術🪄"));
        assert!(text.ends_with(" · 尚未實作"));
    }
    fn fixture() -> Model {
        let stamp = Stamp::of(&crate::modern_lito::battle_scene::tests::observation(
            crate::modern_lito::battle_payload::data_tests::fixture(),
        ));
        Model {
            stamp,
            rows: [4, 5]
                .map(|id| Row {
                    id,
                    label: format!("#{id}"),
                    button: "施放",
                    choice: Some(Choice { stamp, id }),
                })
                .into(),
        }
    }
    #[test]
    fn target_choices_do_not_invent_targets_or_instant_casts() {
        for (id, skill) in [
            (1, Skill::Heal),
            (4, Skill::LightArrow),
            (77, Skill::Disintegrate),
            (43, Skill::Haste),
        ] {
            assert_eq!(use_for(id), Some(Use::Target(skill)));
        }
        for (id, local) in [
            (5, Local::Teleport),
            (54, Local::GreaterHaste),
            (70, Local::FireStorm),
        ] {
            assert_eq!(use_for(id), Some(Use::Local(local)));
        }
        assert!((0..=220)
            .filter(|&id| use_for(id).is_some())
            .eq([1, 4, 5, 29, 43, 54, 70, 77]));
    }
    #[test]
    fn reorder_and_clear_keep_skill_identity_without_reusing_row_indices() {
        let mut m = fixture();
        let q = m.choice(4).unwrap();
        m.rows.reverse();
        assert_eq!(m.choice(4), Some(q));
        assert!(m.choice(3).is_none());
        m.rows.retain(|r| r.id != 4);
        assert!(m.choice(4).is_none());
        m.clear_proofs();
        assert!(m.choice(5).is_none());
        assert_eq!(m.rows[0].label, "#5");
    }
    #[test]
    fn selection_only_survives_same_character_world_and_life() {
        let m = fixture();
        let mut next = m.clone();
        next.stamp.revision += 1;
        assert!(m.same_owner(&next));
        for field in 0..9 {
            let mut next = m.clone();
            match field {
                0 => next.stamp.context.key.source_id += 1,
                1 => next.stamp.context.key.object_id += 1,
                2 => next.stamp.context.epoch += 1,
                3 => next.stamp.channel ^= 1,
                4..=6 => next.stamp.world[field - 4] += 1,
                7 => next.stamp.entity += 1,
                _ => next.stamp.life += 1,
            }
            assert!(!m.same_owner(&next));
        }
    }
}
