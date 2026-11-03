//! Invalid cross-field and historical facts applied to actual Java frames.
use super::{data, haste};
fn valid(p: &haste::Candidate) {
    p.check_values(&p.data.personal.player, &p.data.world.identity)
        .unwrap();
}
fn fields(p: &haste::Candidate, name: &str, edit: impl FnOnce(&mut haste::Candidate)) {
    let mut bad = p.clone();
    edit(&mut bad);
    let parsed = haste::decode(&haste::encode(&bad).unwrap()).unwrap();
    assert!(
        parsed
            .check_values(&p.data.personal.player, &p.data.world.identity)
            .is_err(),
        "{name}"
    );
    valid(p);
    super::admission::reject(p, &parsed);
}
fn timeline(
    a: &haste::Candidate,
    b: &haste::Candidate,
    name: &str,
    edit: impl FnOnce(&mut haste::Candidate),
) {
    a.check_successor(b).unwrap();
    let mut bad = b.clone();
    edit(&mut bad);
    let parsed = haste::decode(&haste::encode(&bad).unwrap()).unwrap();
    parsed
        .check_values(&parsed.data.personal.player, &parsed.data.world.identity)
        .unwrap_or_else(|e| panic!("mutation must remain a valid single scene: {name}: {e:?}"));
    assert!(
        a.check_successor(&parsed).is_err(),
        "timeline accepted {name}"
    );
    a.check_successor(b).unwrap();
    super::admission::reject(a, &parsed);
}
pub(super) fn verify(frames: &[(String, haste::Candidate, Vec<u8>)]) {
    let frame = |name: &str| &frames.iter().find(|f| f.0 == name).unwrap().1;
    for (_, p, _) in frames {
        valid(p);
        let owner = &p.data.personal.player;
        let world = &p.data.world.identity;
        for index in 0..4 {
            let mut other = owner.clone();
            match index {
                0 => other.source += 1,
                1 => other.object += 1,
                2 => other.epoch ^= 1,
                _ => other.channel = (other.channel + 1) % 4,
            }
            assert!(p.check_values(&other, world).is_err());
        }
        for index in 0..3 {
            let mut other = world.clone();
            match index {
                0 => other.room += 1,
                1 => other.match_id += 1,
                _ => other.generation += 1,
            }
            assert!(p.check_values(owner, &other).is_err());
        }
        assert!(p.data.check_context(owner, world).is_err());
        assert!(p.data.check_combat_values(owner, world).is_err());
        assert!(p.data.personal_view(owner, world).is_err());
        assert!(p.data.check_scene_values(owner, world).is_err());
    }
    let pc = frame("pc-other-1");
    fields(
        frame("pc-target-1"),
        "latest successful cast lost player effect",
        |p| {
            p.effects.targets.0.clear();
            let b = p.data.actors.0[1].buffs.as_mut().unwrap();
            b.haste_origin = data::HasteOrigin::None;
            b.haste_until_nanos = 0;
        },
    );
    fields(
        frame("npc-haste-1"),
        "latest successful cast lost NPC effect",
        |p| {
            p.effects.targets.0.clear();
            p.effects.npcs.as_mut().unwrap().actors.0[0]
                .speed
                .haste_until_millis = 0;
        },
    );
    fields(pc, "room identity", |p| p.data.room.id += 1);
    fields(pc, "owner source view", |p| {
        p.data.personal.equipment.as_mut().unwrap().source += 1
    });
    fields(pc, "owner history", |p| p.data.personal.history = None);
    fields(pc, "source food", |p| {
        p.data.personal.equipment.as_mut().unwrap().saved.food225 = 226
    });
    fields(pc, "source/effective maximum", |p| {
        p.data
            .personal
            .equipment
            .as_mut()
            .unwrap()
            .effective
            .maximum_mp += 1
    });
    fields(pc, "live-vs-accounted HP", |p| {
        p.data.actors.0[0].vital.as_mut().unwrap().values.hp -= 1
    });
    fields(pc, "spent across lanes", |p| {
        p.data.actors.0[0].resources.as_mut().unwrap().flows.spent += 1
    });
    fields(pc, "missing spell lane", |p| {
        p.data.actors.0[0].spells.histories.0.remove(0);
    });
    fields(pc, "unknown feature", |p| {
        p.data.world.descriptor.features |= 1 << 29
    });
    fields(pc, "missing timed dependency", |p| {
        p.data.world.descriptor.features &= !(1 << 5)
    });
    fields(pc, "invalid mode", |p| p.data.room.mode = 0);
    fields(frame("pc-loading-1"), "loading active clock", |p| {
        p.data.world.active_elapsed_nanos = 0
    });
    fields(frame("quest-lanes-1"), "foreign Quest view", |p| {
        p.data.world.lane = 1
    });
    fields(frame("quest-finished-1"), "missing result", |p| {
        p.data.result = None
    });
    fields(frame("quest-finished-1"), "wrong mode result", |p| {
        p.data.result.as_mut().unwrap().mode = 1
    });
    let walked = frame("npc-walk-1");
    let refreshed = frame("npc-refresh-1");
    timeline(walked, refreshed, "frozen NPC attack pace", |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0].attack_millis += 1
    });
    timeline(walked, refreshed, "retained walk source position", |p| {
        p.effects.targets.0[0]
            .walk
            .as_mut()
            .unwrap()
            .receipt
            .impact
            .usage
            .before
            .cell
            .x += 1
    });
    timeline(
        walked,
        refreshed,
        "retained normal pace substitution",
        |p| {
            p.effects.targets.0[0].walk = None;
            let m = &mut p.effects.npcs.as_mut().unwrap().actors.0[0];
            m.walk_speed = haste::Speed {
                revision: 0,
                observed_millis: 0,
                haste_until_millis: 0,
                slow_until_millis: 0,
                slow_revision: 0,
            };
            m.last_walk.as_mut().unwrap().duration_millis = m.walk_millis;
            p.data.npcs.as_mut().unwrap().actors.0[0].next_move_millis =
                m.last_walk.as_ref().unwrap().elapsed_millis + i64::from(m.walk_millis);
        },
    );
    timeline(walked, refreshed, "retained walk geometry", |p| {
        let w = p.effects.npcs.as_mut().unwrap().actors.0[0]
            .last_walk
            .as_mut()
            .unwrap();
        w.from = data::Cell {
            x: w.to.x + 1,
            y: w.to.y + 1,
        };
        w.heading = 7;
        p.data.npcs.as_mut().unwrap().actors.0[0].heading = 7;
    });
    timeline(frame("pc-target-1"), pc, "premature player clear", |p| {
        p.effects
            .targets
            .0
            .retain(|e| !matches!(&e.target,haste::Target::Player(t) if t.entity==2));
        let b = p.data.actors.0[1].buffs.as_mut().unwrap();
        b.haste_origin = data::HasteOrigin::None;
        b.haste_until_nanos = 0;
        b.revision = p.data.world.revision;
        b.observed_nanos = p.data.world.active_elapsed_nanos;
    });
    timeline(frame("npc-haste-1"), walked, "premature NPC clear", |p| {
        p.effects.targets.0[0].active = None;
        let m = &mut p.effects.npcs.as_mut().unwrap().actors.0[0];
        m.speed.haste_until_millis = 0;
        m.speed.revision = p.data.npcs.as_ref().unwrap().revision;
        m.speed.observed_millis = p.data.npcs.as_ref().unwrap().elapsed_millis;
    });
    // These changed values satisfy the standalone motion facts but cannot be
    // substituted into a duplicate or terminal state.
    timeline(refreshed, refreshed, "same world revision", |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0].attack_millis += 1
    });
    let ended = frame("quest-finished-2");
    timeline(ended, ended, "terminal motion", |p| {
        p.effects.npcs.as_mut().unwrap().actors.0[0].attack_millis += 1
    });
}
