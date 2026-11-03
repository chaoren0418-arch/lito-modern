//! Pose arbitration inputs are explicit synthetic edits of frozen Java frames.
//! They are not new valid wire receipts, gameplay/RNG proofs or HWND tests.
use super::*;
use crate::modern_lito::{disintegrate_presentation::npc_tests::reply, npc_protocol as npc};

fn arrow() -> npc::Frame {
    reply("normal-arrow").frame.unwrap()
}
fn visual(f: &npc::Frame, now: Instant) -> DamageVisual {
    let event = light_arrow_events(&f.combat)
        .unwrap()
        .into_iter()
        .find(|e| e.entity_id == 1 && e.cast_count > 0)
        .unwrap();
    let m = f.combat.world.member(1).unwrap();
    DamageVisual {
        event,
        body: super::super::paced_animation::PacedClock::new(now),
        move_sequence: f.combat.world.actor(1).unwrap().sequence,
        appearance: BaseAppearance {
            class: m.class as u8,
            sex: m.sex,
        },
        launched: false,
        body_visible: true,
    }
}
fn melee(f: &npc::Frame, kind: u8, revision: i64) -> npc::Frame {
    let mut out = f.clone();
    out.exchanges.clear();
    match kind {
        0 => {
            let mut e = npc::tests::read_reply()
                .frame
                .unwrap()
                .exchanges
                .into_iter()
                .find(|e| e.direction == npc::Direction::PlayerToNpc)
                .unwrap();
            e.participant = f.combat.world.actor(1).unwrap().identity();
            e.revision = revision;
            e.npc.scope = f.scope;
            out.exchanges.push(e);
        }
        1 => {
            let mut q = reply("quest-own")
                .frame
                .unwrap()
                .combat
                .world
                .quest
                .unwrap();
            q.exchanges = vec![crate::modern_lito::quest_protocol::Exchange {
                entity_id: 1,
                slot: q.targets[0].slot,
                sequence: 1,
                revision,
                outcome: crate::modern_lito::quest_protocol::Outcome {
                    hit: true,
                    raw_damage: 1,
                    applied_damage: 1,
                    remaining_hp: 1,
                    death: false,
                    maximum: false,
                    critical: false,
                },
            }];
            out.combat.world.quest = Some(q);
        }
        2 => {
            let mut t = reply("time-attack-finished")
                .frame
                .unwrap()
                .combat
                .world
                .time_attack
                .unwrap();
            t.departed_mask = 0;
            t.exchanges = vec![crate::modern_lito::time_attack_protocol::Exchange {
                entity_id: 1,
                sequence: 1,
                revision: revision - 1,
                flags: 1,
                raw_damage: 1,
                applied_damage: 1,
                remaining_hp: 1,
            }];
            out.combat.world.time_attack = Some(t);
        }
        _ => panic!("unknown typed attack"),
    }
    assert_eq!(
        crate::modern_lito::npc_presentation::player_attack(&out, 1)
            .unwrap()
            .revision,
        revision
    );
    out
}

#[test]
fn arrow_pending_uses_ordinary_quest_and_time_attack_melee_revisions() {
    let f = arrow();
    let cast = visual(&f, Instant::now());
    for kind in 0..3 {
        for newer in [false, true] {
            let n = melee(
                &f,
                kind,
                cast.event.cast_revision as i64 + if newer { 1 } else { -1 },
            );
            assert_eq!(
                light_arrow_body_current(&n.combat, Some(&n), &cast),
                !newer,
                "kind={kind}"
            );
            assert_eq!(n.combat.vitals, f.combat.vitals);
        }
    }
    let mut other = melee(&f, 0, cast.event.cast_revision as i64 + 1);
    other.exchanges[0].participant = f.combat.world.actor(2).unwrap().identity();
    assert!(light_arrow_body_current(&other.combat, Some(&other), &cast));
    other.exchanges[0].direction = npc::Direction::NpcToParticipant;
    other.exchanges[0].participant = f.combat.world.actor(1).unwrap().identity();
    assert!(light_arrow_body_current(&other.combat, Some(&other), &cast));
}

#[test]
fn arrow_pending_hurt_is_not_movement_death_life_or_new_deliberate_action() {
    let f = arrow();
    let cast = visual(&f, Instant::now());
    for change in 0..8 {
        let mut next = f.clone();
        let c = &mut next.combat;
        let revision = cast.event.cast_revision as i64 + 1;
        match change {
            0 => {
                c.vitals[0].hurt_revision = revision;
                c.vitals[0].hp -= 1;
            }
            1 => c.world.actors[0].sequence += 1,
            2 => c.world.actors[0].position.x += 1,
            3 => c.vitals[0].hp = 0,
            4 => c.vitals[0].attack_revision = revision,
            5 => c.world.casts[0].cast_revision = revision,
            6 => c.npc_disintegrations[0].revision = revision,
            _ => {
                let mut q = reply("quest-own")
                    .frame
                    .unwrap()
                    .combat
                    .world
                    .quest
                    .unwrap();
                q.lives[0].entity_id = 1;
                q.lives[0].respawns += 1;
                c.world.quest = Some(q);
            }
        }
        assert_eq!(
            light_arrow_body_current(&next.combat, Some(&next), &cast),
            change == 0,
            "change={change}"
        );
    }
}

#[test]
fn teleport_body_yields_to_all_typed_melee_without_changing_cast_receipt() {
    let f = arrow();
    let mut event = *f.combat.world.cast(1).unwrap();
    event.cast_revision = f.combat.world.revision + 1;
    event.to = f.combat.world.actor(1).unwrap().position;
    let sequence = f.combat.world.actor(1).unwrap().sequence;
    let frozen = event;
    for kind in 0..3 {
        for newer in [false, true] {
            let n = melee(&f, kind, event.cast_revision + if newer { 1 } else { -1 });
            assert_eq!(
                teleport_body_current(&n.combat, Some(&n), &event, sequence),
                !newer,
                "kind={kind}"
            );
        }
    }
    assert_eq!(event, frozen);
    for change in 0..4 {
        let mut next = f.clone();
        match change {
            0 => next.combat.vitals[0].hurt_revision = event.cast_revision + 1,
            1 => next.combat.vitals[0].hp = 0,
            2 => next.combat.world.actors[0].sequence += 1,
            _ => next.combat.world.actors[0].position.x += 1,
        }
        assert!(!teleport_body_current(
            &next.combat,
            Some(&next),
            &event,
            sequence
        ));
    }
}

#[test]
#[ignore = "requires actual shared caster18, SPR167/219/11746 and WAV assets; synthetic pose arbitration, not live server/UI"]
fn actual_shared_pending_skills_yield_to_npc_melee_but_keep_launched_effects() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
    let cancel = AtomicBool::new(false);
    let before = reply("normal-before");
    let base = before.frame.as_ref().unwrap();
    let map = crate::modern_lito::siege_space::tests::map(&base.combat.world);
    let first = NpcInput {
        reply: Arc::new(before.clone()),
        plan: Arc::new(crate::modern_lito::npc_presentation::Plan::from_reply(&before).unwrap()),
    };
    let mut renderer = load_qualified_map_renderer(
        &root,
        &base.combat.world,
        Some(&base.combat),
        Some(&first),
        &cancel,
        map,
    )
    .unwrap();
    let light = renderer.light_arrow.as_mut().unwrap();
    let dis = renderer.disintegrate.as_mut().unwrap();
    let hit = reply("normal-hit").frame.unwrap();
    let mut reset = base.clone();
    reset.combat.world.phase = Phase::Preparing;
    let arrow = arrow();
    let mut now = Instant::now();
    for kind in 0..3 {
        for phase in 0..3 {
            let n = melee(&hit, kind, hit.combat.world.revision + 1);
            dis.update(
                &reset.combat,
                &renderer.map,
                &light.assets,
                Some(&reset),
                now,
            )
            .unwrap();
            let first = if phase == 0 { &n } else { &hit };
            dis.update(
                &first.combat,
                &renderer.map,
                &light.assets,
                Some(first),
                now,
            )
            .unwrap();
            let delay = light
                .assets
                .cast_launch_delay(visual(&arrow, now).appearance)
                .unwrap();
            assert!(delay > Duration::ZERO);
            if phase == 2 {
                dis.update(
                    &hit.combat,
                    &renderer.map,
                    &light.assets,
                    Some(&hit),
                    now + delay,
                )
                .unwrap();
            }
            let at = now + delay + Duration::from_millis(1);
            dis.update(&n.combat, &renderer.map, &light.assets, Some(&n), at)
                .unwrap();
            assert!(dis.overrides(&light.assets, at).unwrap().is_empty());
            assert_eq!(
                dis.draws(at).unwrap().len(),
                usize::from(phase == 2),
                "skill77 kind={kind} phase={phase}"
            );
            dis.update(
                &n.combat,
                &renderer.map,
                &light.assets,
                Some(&n),
                at + Duration::from_secs(3),
            )
            .unwrap();
            assert!(dis.draws(at + Duration::from_secs(3)).unwrap().is_empty());
            now += Duration::from_secs(4);

            light.baseline = light_arrow_events(&base.combat)
                .unwrap()
                .into_iter()
                .map(|e| (e.entity_id, e.cast_revision))
                .collect();
            light.active.clear();
            light.flights.clear();
            let n = melee(&arrow, kind, arrow.combat.world.revision + 1);
            let first = if phase == 0 { &n } else { &arrow };
            light
                .update(&first.combat, Some(first), &renderer.map, now)
                .unwrap();
            if phase == 2 {
                light
                    .update(&arrow.combat, Some(&arrow), &renderer.map, now + delay)
                    .unwrap();
            }
            light
                .update(
                    &n.combat,
                    Some(&n),
                    &renderer.map,
                    now + delay + Duration::from_millis(1),
                )
                .unwrap();
            assert!(light.active.is_empty());
            assert_eq!(
                light.flights.len(),
                usize::from(phase == 2),
                "arrow kind={kind} phase={phase}"
            );
            now += Duration::from_secs(4);
        }
    }
    // Hurt yields the body only; both sequential and coalesced receipts must
    // still launch once from the original committed clock.
    for coalesced in [false, true] {
        light.baseline = light_arrow_events(&base.combat)
            .unwrap()
            .into_iter()
            .map(|e| (e.entity_id, e.cast_revision))
            .collect();
        light.active.clear();
        light.flights.clear();
        let mut hurt = arrow.clone();
        hurt.combat.vitals[0].hurt_revision = hurt.combat.world.revision + 1;
        hurt.combat.vitals[0].hp -= 1;
        let first = if coalesced { &hurt } else { &arrow };
        light
            .update(&first.combat, Some(first), &renderer.map, now)
            .unwrap();
        let delay = light
            .assets
            .cast_launch_delay(visual(&arrow, now).appearance)
            .unwrap();
        light
            .update(&hurt.combat, Some(&hurt), &renderer.map, now + delay / 2)
            .unwrap();
        assert!(!light.active[&1].body_visible);
        light
            .update(&hurt.combat, Some(&hurt), &renderer.map, now + delay)
            .unwrap();
        assert_eq!(light.flights.len(), 1);
        let started = light.flights[0].started;
        light
            .update(
                &hurt.combat,
                Some(&hurt),
                &renderer.map,
                now + delay + Duration::from_millis(1),
            )
            .unwrap();
        assert_eq!(light.flights.len(), 1);
        assert_eq!(light.flights[0].started, started);
        now += Duration::from_secs(4);
    }
    assert_eq!(arrow, reply("normal-arrow").frame.unwrap());
    assert_eq!(hit, reply("normal-hit").frame.unwrap());

    // Teleport's receipt owns the origin effect immediately. Hiding its caster
    // body after melee must neither remove that effect nor replay its cues.
    let teleport = renderer.teleport.as_mut().unwrap();
    let mut teleported = base.clone();
    let actor = teleported.combat.world.actor(1).unwrap().clone();
    let event = &mut teleported.combat.world.casts[0];
    event.cast_count = 1;
    event.cast_revision = teleported.combat.world.revision + 1;
    event.from = actor.position;
    event.to = actor.position;
    teleport
        .update(&teleported.combat, &renderer.map, now)
        .unwrap();
    for kind in 0..3 {
        let n = melee(
            &teleported,
            kind,
            teleported.combat.world.casts[0].cast_revision + 1,
        );
        teleport.update(&n.combat, &renderer.map, now).unwrap();
        let cast = &teleport.active[&1];
        assert!(!teleport_body_current(
            &n.combat,
            Some(&n),
            &cast.event,
            cast.move_sequence
        ));
        assert_eq!(cast.started, now);
    }
    let playback = &mut teleport.active.get_mut(&1).unwrap().effect;
    let end = teleport.assets.effect_duration();
    assert!(playback.advance(end).unwrap().finished);
    assert!(playback.advance(end).unwrap().crossed_cues.is_empty());

    let mut mismatched = arrow.combat.clone();
    mismatched.vitals[0].hp -= 1;
    let active = light.active.len();
    let flights = light.flights.len();
    assert!(light
        .update(&mismatched, Some(&arrow), &renderer.map, now)
        .is_err());
    assert_eq!((light.active.len(), light.flights.len()), (active, flights));
    assert!(dis
        .update(&mismatched, &renderer.map, &light.assets, Some(&arrow), now)
        .is_err());
}

#[test]
fn teleport_body_yields_to_siege_tower_engine_revision_plus_one() {
    let mut frame = crate::modern_lito::siege_protocol::tests::sample("left")
        .frame
        .unwrap();
    let tower = &frame.world.siege.as_ref().unwrap().objectives[0];
    let id = tower.last_actor;
    let revision = tower.revision + 1;
    let actor = frame.world.actor(id).unwrap();
    let sequence = actor.sequence;
    let mut cast = *frame.world.cast(id).unwrap();
    cast.to = actor.position;
    // The tower snapshot is a real Java golden; only this visual receipt's
    // relative ordering is synthetic, never used for authority or transport.
    assert_eq!(frame.vital(id).unwrap().attack_revision, 0);
    cast.cast_revision = revision - 1;
    assert!(!teleport_body_current(&frame, None, &cast, sequence));
    cast.cast_revision = revision + 1;
    assert!(teleport_body_current(&frame, None, &cast, sequence));
    frame.world.siege.as_mut().unwrap().objectives[0].last_actor = if id == 1 { 2 } else { 1 };
    cast.cast_revision = revision - 1;
    assert!(teleport_body_current(&frame, None, &cast, sequence));
}
