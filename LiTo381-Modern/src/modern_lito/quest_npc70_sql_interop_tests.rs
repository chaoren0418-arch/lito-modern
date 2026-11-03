//! v31 NPC70 through production Main, owned SQL and real shared rendering.
//! Fixture HP/balance is explicit; no HWND, audio-device or all-map claim.
use super::*;
use crate::modern_lito::npc_fire_storm_wire::{State as FireState, StructureId};

#[test]
#[ignore = "NPC70 production Main/isolated SQL, two shared Quest lanes, four waves, fire-storm objective, saved records/source reload; no HWND"]
fn java_quest_npc70_sql_shared_sparse_waves_objective_saved_records_and_source_reload() {
    sql_flow_with_npc_spells(false, false, false, false, false, true);
}

pub(super) fn assert_skill(c: &Client) {
    let v = c.view();
    let skill = v
        .skills
        .fire_storm()
        .expect("SQL learned70 and exact template");
    assert_eq!(skill.name, "$1975");
    assert_eq!((skill.mp_cost, skill.reuse_millis), (48, 2000));
    assert!(v.skills.disintegrate().is_none());
}
pub(super) fn audio(c: &Client) -> u64 {
    c.rendered
        .as_ref()
        .and_then(|r| r.audio.as_ref())
        .and_then(|a| {
            a.events
                .iter()
                .filter(|e| e.group.candidates() == [710])
                .map(|e| e.sequence)
                .max()
        })
        .unwrap_or(0)
}
pub(super) fn cast(cs: &mut [Client; 2], i: usize, d: &Deadline) -> FireState {
    cast_with_chat(cs, i, d, false)
}
fn cast_with_chat(cs: &mut [Client; 2], i: usize, d: &Deadline, filtered_chat: bool) -> FireState {
    if cs[i].canonical.is_some() {
        wait(cs, d, "canonical fire shortcut owner", |cs| {
            cs[i].canonical_surface.as_ref().is_some_and(|s| {
                use crate::modern_lito::{battle_player, battle_window, world_window::Action};
                let action =
                    battle_window::stamp_local_action(Action::FireStorm, Some(&s.observation));
                battle_player::admit_local_action(action, &cs[i].view())
                    .is_ok_and(|a| a == Action::FireStorm)
            })
        });
    }
    let v = cs[i].view();
    let f = v.combat.as_ref().unwrap();
    let id = i as u64 + 1;
    assert!(
        login_window::fire_storm_available(&v, id),
        "the actual normal-window button must expose qualified NPC70"
    );
    let owner = cs[i].canonical_surface.as_ref().map(|s| {
        use crate::modern_lito::{battle_window, world_window::Action};
        let Action::Canonical(intent) =
            battle_window::stamp_local_action(Action::FireStorm, Some(&s.observation))
        else {
            unreachable!()
        };
        intent
    });
    let before = f.npc_fire(id).unwrap().clone();
    let q = login_window::npc_fire_storm_input_request(&v).expect("actual UI NPC70 input");
    let mut invalid = q;
    invalid.context.key.source_id = if i == 0 { 42 } else { 7 };
    assert!(cs[i]
        .controller
        .command(LoginCommand::CastNpcFireStorm(invalid))
        .is_err());
    invalid = q;
    invalid.caster_life += 1;
    assert!(cs[i]
        .controller
        .command(LoginCommand::CastNpcFireStorm(invalid))
        .is_err());
    cs[i].command(crate::modern_lito::battle_player::bind_local_command(
        owner,
        LoginCommand::CastNpcFireStorm(q),
    ));
    assert!(cs[i].view().spell_pending);
    assert!(
        login_window::fire_storm_available(&cs[i].view(), id),
        "pending cast disables the button; it must not hide it"
    );
    if filtered_chat {
        quest_filtered_chat_sql_interop_tests::pending_cast_rejects_chat(cs, i);
    }
    let pending = cs[i].view();
    let confirmed = pending.combat.as_ref().unwrap();
    // command() polls completed authority before enqueueing. A clock/NPC tick
    // may legitimately advance the whole frame since v (including during the
    // two rejected commands above). It cannot acknowledge this caster request.
    // Exact full-frame non-prediction under refreshed authority is separately
    // asserted by npc70_command_polls_new_authority_before_reserving_without_predicting_a_cast.
    assert_eq!(
        confirmed.npc_fire(id),
        Some(&before),
        "no predicted cast count, MP spend, target damage or structure receipt"
    );
    assert_eq!(
        confirmed.world.cast(id).unwrap().request_sequence,
        q.sequence - 1,
        "pending request has no caster acknowledgment"
    );
    assert_eq!(
        (confirmed.world.match_id, confirmed.world.generation),
        (f.world.match_id, f.world.generation)
    );
    assert!(
        confirmed.world.revision >= f.world.revision,
        "already confirmed background authority must not roll back"
    );
    assert!(cs[i]
        .controller
        .command(LoginCommand::CastNpcFireStorm(q))
        .is_err());
    wait(cs, d, "SQL NPC70 confirmed cast", |cs| {
        let v = cs[i].view();
        !v.spell_pending
            && v.combat
                .as_ref()
                .is_some_and(|f| f.world.cast(id).unwrap().request_sequence == q.sequence)
    });
    let v = cs[i].view();
    let f = v.combat.as_ref().unwrap();
    let after = f.npc_fire(id).unwrap().clone();
    assert!(
        login_window::fire_storm_available(&v, id),
        "confirmed cast retains the same supported button"
    );
    let cost = if i == 0 { 46 } else { 48 }; // Original-INT12 knight vs mage.
    assert_eq!(after.count, before.count + 1, "{}", v.status);
    assert_eq!(after.mp_cost, cost);
    assert_eq!(after.mp_spent, before.mp_spent + i64::from(cost));
    assert_eq!(after.last_sequence, q.sequence);
    assert_eq!(after.caster_life, q.caster_life);
    assert!(f.fires.is_empty() && f.npc_spells.is_empty() && f.npc_disintegrations.is_empty());
    assert_eq!(
        f.world.cast(id).unwrap().mp_spent,
        0,
        "never borrow teleport ledger"
    );
    after
}
pub(super) fn clear_waves(cs: &mut [Client; 2], i: usize, d: &Deadline, filtered_chat: bool) {
    let id = i as u64 + 1;
    let mut next = Instant::now();
    let mut seen = BTreeSet::new();
    let mut sound = audio(&cs[i]);
    for stage in 1..=4 {
        wait(cs, d, "SQL NPC70 own painted wave and cooldown", |cs| {
            let v = cs[i].view();
            !v.spell_pending
                && !v.combat_pending
                && Instant::now() >= next
                && cs[1 - i]
                    .view()
                    .combat
                    .as_ref()
                    .is_some_and(|f| f.npc_fire((2 - i) as u64).is_some())
                && cs[i].rendered.as_ref().is_some_and(|r| {
                    r.npcs.iter().any(|n| n.alive)
                        && r.npc
                            .as_ref()
                            .and_then(|n| n.frame.as_ref())
                            .is_some_and(|f| f.actors.iter().any(|n| n.alive() && n.stage == stage))
                })
                && v.npc
                    .as_ref()
                    .and_then(|n| n.frame.as_ref())
                    .is_some_and(|f| f.actors.iter().any(|n| n.alive() && n.stage == stage))
        });
        let other = cs[1 - i]
            .view()
            .combat
            .as_ref()
            .unwrap()
            .npc_fire((2 - i) as u64)
            .unwrap()
            .clone();
        let before = cs[i].view();
        let f = before.npc.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.combat.world.audience_lane, Some(i as u8));
        assert_eq!(f.combat.world.actors.len(), 1);
        assert_eq!(f.combat.world.actors[0].entity_id, id);
        assert_eq!(f.combat.npc_fires.len(), 1);
        assert!(f.actors.iter().all(|n| n.lane == i as u8));
        let target = f
            .actors
            .iter()
            .find(|n| n.alive() && n.stage == stage)
            .unwrap();
        let target_id = npc::Id {
            scope: f.scope,
            serial: target.serial,
        };
        let result = cast_with_chat(cs, i, d, filtered_chat);
        assert_eq!(result.count, i64::from(stage));
        assert_eq!(result.targets.len(), 1);
        let hit = &result.targets[0];
        assert_eq!(hit.id, target_id);
        assert_eq!(hit.life, target.life);
        assert_eq!(hit.lane, i as u8);
        assert!(hit.outcome.death);
        assert_eq!(hit.outcome.applied_damage, 1);
        assert!(seen.insert(target_id.serial));
        assert_eq!(
            cs[1 - i]
                .view()
                .combat
                .as_ref()
                .unwrap()
                .npc_fire((2 - i) as u64),
            Some(&other),
            "private lane cannot debit another source"
        );
        next = Instant::now() + Duration::from_millis(2100);
        wait(cs, d, "SQL NPC70 real per-wave shared effect", |cs| {
            audio(&cs[i]) > sound
        });
        sound = audio(&cs[i]);
        if filtered_chat && stage == 1 {
            quest_filtered_chat_sql_interop_tests::after_cast(cs, d, i);
        }
    }
    assert_eq!(seen.len(), 4);
    wait(cs, d, "SQL NPC70 four waves unlock objective", |cs| {
        cs[i]
            .rendered
            .as_ref()
            .is_some_and(|r| r.quests.len() == 1 && r.quests[0].alive)
    });
}
pub(super) fn finish_objective(cs: &mut [Client; 2], d: &Deadline) {
    let next = Instant::now() + Duration::from_millis(2100);
    wait(cs, d, "SQL NPC70 final objective cooldown", |_| {
        Instant::now() >= next
    });
    let other = cs[1]
        .view()
        .combat
        .as_ref()
        .unwrap()
        .npc_fire(2)
        .unwrap()
        .clone();
    let sound = audio(&cs[0]);
    let result = cast(cs, 0, d);
    assert_eq!(result.count, 5);
    assert!(result.targets.is_empty());
    let hit = result
        .current_structure()
        .expect("current fire destroyed the objective");
    assert!(matches!(hit.id, StructureId::Quest { .. }));
    assert!(hit.outcome.death);
    assert_eq!(hit.outcome.applied_damage, 1);
    assert_eq!(
        cs[1].view().combat.as_ref().unwrap().npc_fire(2),
        Some(&other)
    );
    wait(cs, d, "SQL NPC70 final effect survives settlement", |cs| {
        audio(&cs[0]) > sound
    });
}
pub(super) fn assert_saved(cs: &[Client; 2]) {
    for (i, c) in cs.iter().enumerate() {
        let v = c.view();
        let f = v.combat.as_ref().unwrap();
        assert_eq!(f.world.phase, world::Phase::Finished);
        assert_eq!(
            f.world.completion.as_ref().unwrap().state,
            world::SettlementState::Saved
        );
        let state = f.npc_fire(i as u64 + 1).unwrap();
        assert_eq!(
            (state.count, state.mp_spent),
            if i == 0 { (5, 230) } else { (4, 192) }
        );
        assert_eq!(f.world.actors.len(), 1);
        assert_eq!(f.world.actors[0].key.source_id, if i == 0 { 7 } else { 42 });
        assert_eq!(f.world.actors[0].key.object_id, 3333);
    }
}
