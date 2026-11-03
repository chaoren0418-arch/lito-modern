//! Minimal confirmed visual events shared by PvP70 and typed NPC70.
//! Never convert NPC targets into player targets or use retained structure
//! receipts as a new cast. Initial zero-revision entries only seed baselines.
use super::{combat_protocol::Frame, world_map::WorldCell};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Event {
    pub entity_id: u64,
    pub cast_revision: i64,
    pub from: WorldCell,
    pub caster_life: Option<u64>,
}
pub(crate) fn enabled(frame: &Frame) -> bool {
    !frame.fires.is_empty() || !frame.npc_fires.is_empty()
}
pub(crate) fn revision(frame: &Frame, entity: u64) -> i64 {
    super::fire_storm_protocol::revision(frame, entity)
        .max(frame.npc_fire(entity).map_or(0, |s| s.revision))
}
pub(crate) fn events(frame: &Frame) -> Vec<Event> {
    frame
        .fires
        .iter()
        .map(|s| Event {
            entity_id: s.entity_id,
            cast_revision: s.cast_revision,
            from: s.from,
            caster_life: None,
        })
        .chain(frame.npc_fires.iter().map(|s| Event {
            entity_id: s.entity_id,
            cast_revision: s.revision,
            from: s.origin.unwrap_or(WorldCell { x: 0, y: 0 }),
            caster_life: Some(s.caster_life),
        }))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::npc_fire_storm_protocol::tests::sample;

    #[test]
    fn npc70_visual_events_are_caster_owned_not_one_per_target_or_structure() {
        for (scenario, name) in [
            ("normal", "before"),
            ("normal", "hit"),
            ("time", "tower"),
            ("554", "cast-1-own"),
        ] {
            let (_, _, f) = sample(scenario, name);
            let c = &f.combat;
            assert!(enabled(c));
            assert!(c.fires.is_empty());
            let e = events(c);
            assert_eq!(e.len(), c.npc_fires.len());
            for (s, event) in c.npc_fires.iter().zip(e) {
                assert_eq!(event.entity_id, s.entity_id);
                assert_eq!(event.cast_revision, s.revision);
                assert_eq!(revision(c, s.entity_id), s.revision);
                assert_eq!(event.caster_life, Some(s.caster_life));
                if s.count > 0 {
                    assert_eq!(Some(event.from), s.origin);
                }
            }
        }
    }
    #[test]
    fn npc70_retained_receipts_do_not_become_new_visual_events() {
        let (_, _, hit) = sample("normal", "hit");
        for name in ["denied", "arrow", "disintegrate"] {
            let (_, _, later) = sample("normal", name);
            assert_eq!(events(&hit.combat), events(&later.combat), "{name}");
        }
        let (_, _, second) = sample("normal", "second");
        let (_, _, regen) = sample("normal", "regen");
        assert_eq!(events(&second.combat), events(&regen.combat));
        assert_ne!(events(&hit.combat), events(&second.combat));
    }
}
