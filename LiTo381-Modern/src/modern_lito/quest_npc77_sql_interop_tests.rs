//! v29 NPC77 through real Main/SELECT-only SQL and shared Quest rendering.
//! Deliberately tiny test HP; actual RNG/time/death confirmation, not native balance.
use super::time_attack_sql_interop_tests::{npc77_audio, npc77_request};
use super::*;

#[test]
#[ignore = "requires real ModernMain/TLS, isolated SQL and shared map554; two sparse lanes, four casts and death/revive per source, saved result/return; no HWND"]
fn java_quest_npc77_sql_sparse_waves_death_revival_saved_records_and_return() {
    sql_flow_with_npc77(false, false, false, false, true);
}

pub(super) fn assert_saved(cs: &[Client; 2]) {
    for (i, c) in cs.iter().enumerate() {
        let view = c.view();
        let f = view.combat.as_ref().unwrap();
        let entity = i as u64 + 1;
        assert_eq!(f.world.phase, world::Phase::Finished);
        assert_eq!(
            f.world.completion.as_ref().unwrap().state,
            world::SettlementState::Saved
        );
        assert_eq!(f.world.actors.len(), 1);
        assert_eq!(
            f.world.actors[0].key,
            CharacterKey {
                source_id: if i == 0 { 7 } else { 42 },
                object_id: 3333,
            }
        );
        let s = f.npc_disintegrate(entity).unwrap();
        assert_eq!(s.count, 4);
        assert_eq!(s.mp_spent, if i == 0 { 272 } else { 280 });
        assert_eq!(s.lawful, 504);
        let life = quest_life::life(&f.world, entity).unwrap();
        assert!(life.deaths >= 4 && life.respawns >= 4);
        assert!(s.hit.as_ref().unwrap().caster_life <= life.incarnation());
        assert_eq!(c.acks.len(), 1);
    }
}

pub(super) fn clear_npc77_waves(cs: &mut [Client; 2], i: usize, d: &Deadline) {
    let entity = i as u64 + 1;
    let cost = if i == 0 { 68 } else { 70 };
    let mut next = Instant::now();
    let mut targets = BTreeSet::new();
    let mut last_sound = npc77_audio(&cs[i]);
    for stage in 1..=4 {
        wait(
            cs,
            d,
            "NPC77 own real painted wave and real cooldown",
            |cs| {
                let v = cs[i].view();
                !v.combat_pending
                    && !v.spell_pending
                    && Instant::now() >= next
                    // One client's first painted frame does not imply that
                    // the other TLS connection has received its own ledger.
                    // Wait for that authoritative baseline before comparing
                    // cross-lane immutability below; never invent an empty one.
                    && cs[1 - i]
                        .view()
                        .combat
                        .as_ref()
                        .is_some_and(|f| f.npc_disintegrate((2 - i) as u64).is_some())
                    && v.npc
                        .as_ref()
                        .and_then(|r| r.frame.as_ref())
                        .is_some_and(|f| {
                            f.actors.iter().any(|n| {
                                n.alive()
                                    && n.stage == stage
                                    && cs[i].rendered.as_ref().is_some_and(|r| {
                                        r.npcs.iter().any(|p| {
                                            p.id == npc::Id {
                                                scope: f.scope,
                                                serial: n.serial,
                                            } && p.life == n.life
                                                && p.alive
                                        })
                                    })
                            })
                        })
            },
        );
        let before = cs[i].view();
        let f = before.npc.as_ref().unwrap().frame.as_ref().unwrap();
        assert_eq!(f.combat.world.audience_lane, Some(i as u8));
        assert_eq!(f.combat.world.actors.len(), 1, "no opponent bodies");
        assert_eq!(
            f.combat.world.actors[0].entity_id, entity,
            "sparse entity2 stays entity2"
        );
        assert_eq!(
            f.combat.npc_disintegrations.len(),
            1,
            "no opponent spell ledger"
        );
        assert_eq!(f.combat.npc_disintegrations[0].entity_id, entity);
        assert_eq!(f.combat.npc_disintegrations[0].count, stage as i64 - 1);
        assert!(f.actors.iter().all(|n| n.lane == i as u8));
        let own_before = f.combat.npc_disintegrate(entity).unwrap().clone();
        let opponent = cs[1 - i]
            .view()
            .combat
            .as_ref()
            .unwrap()
            .npc_disintegrate((2 - i) as u64)
            .unwrap()
            .clone();

        let mut resurrection_attempts = 0;
        loop {
            resurrection_attempts += 1;
            assert!(
                resurrection_attempts <= 8,
                "NPC77 source={i} stage={stage} cannot leave death cycle"
            );
            // Do not force HP or server time. Let this fixture's real NPC attack
            // kill the one-HP player, and require the shared renderer to paint death.
            wait(cs, d, "NPC77 real Quest death before confirmation", |cs| {
                cs[i]
                    .view()
                    .combat
                    .as_ref()
                    .is_some_and(|f| quest_life::confirmation(f, entity).is_some())
                    && cs[i]
                        .rendered
                        .as_ref()
                        .and_then(|r| r.combat.as_ref())
                        .is_some_and(|f| quest_life::confirmation(f, entity).is_some())
            });
            let dead = cs[i].view();
            let old_local_owner = crate::modern_lito::battle_window::Stamp::of(
                dead.battle
                    .observation()
                    .expect("confirmed canonical owner before resurrection"),
            );
            let f = dead.combat.as_ref().unwrap();
            let proof = quest_life::confirmation(f, entity).unwrap();
            assert_eq!(
                f.npc_disintegrate(entity),
                Some(&own_before),
                "death preserves the prior authoritative spell ledger"
            );
            let target = dead
                .npc
                .as_ref()
                .unwrap()
                .frame
                .as_ref()
                .unwrap()
                .actors
                .iter()
                .find(|n| n.alive() && n.stage == stage)
                .unwrap()
                .clone();
            let id = npc::Id {
                scope: proof.scope,
                serial: target.serial,
            };
            cs[i].command(LoginCommand::QuestRevive { proof });
            assert_eq!(
                cs[i]
                    .view()
                    .combat
                    .as_ref()
                    .unwrap()
                    .vital(entity)
                    .unwrap()
                    .hp,
                0,
                "no optimistic resurrection"
            );
            wait(
                cs,
                d,
                "NPC77 resurrection and new canonical life acknowledged",
                |cs| {
                    let v = cs[i].view();
                    !v.combat_pending
                        && v.combat.as_ref().is_some_and(|f| {
                            quest_life::life(&f.world, entity)
                                .is_some_and(|l| l.incarnation() == proof.incarnation + 1)
                                && (f.vital(entity).is_some_and(|a| a.hp == 0)
                                    || !v.action_supported
                                    || v.battle.observation().is_some_and(|o| {
                                        crate::modern_lito::battle_window::Stamp::of(o).life
                                            == (proof.incarnation + 1) as i64
                                            && o.data().actors.0.iter().any(|a| {
                                                a.entity == entity as i64
                                                    && a.vital
                                                        .as_ref()
                                                        .is_some_and(|v| v.values.hp > 0)
                                            })
                                    }))
                        })
                },
            );
            let revived = cs[i].view();
            let f = revived.combat.as_ref().unwrap();
            {
                let stamp = old_local_owner;
                use crate::modern_lito::{
                    battle_player,
                    battle_window::{Intent, Kind, Local},
                };
                assert!(stamp.life <= proof.incarnation as i64);
                // Exercise the actual controller's post-poll rejection, not
                // merely a pure UI predicate. Neither shortcut is sent.
                for (local, command) in [
                    (Local::Teleport, LoginCommand::CastTeleport),
                    (
                        Local::UseItem(40010),
                        LoginCommand::UseItem { item_id: 40010 },
                    ),
                ] {
                    let error = cs[i]
                        .controller
                        .command(battle_player::bind_local_command(
                            Some(Intent {
                                stamp,
                                kind: Kind::Local(local),
                            }),
                            command,
                        ))
                        .unwrap_err();
                    assert!(error.contains("操作的角色、生命或場次已更新"));
                    assert!(!cs[i].view().spell_pending && !cs[i].view().item_pending);
                }
            }
            assert_eq!(
                f.npc_disintegrate(entity),
                Some(&own_before),
                "reviving cannot erase prior-life skill cost or manufacture a hit"
            );
            if f.vital(entity).unwrap().hp == 0 {
                // A long-dead player may be killed again immediately at its camp.
                // The next attempt uses the new real death proof, never fake HP.
                continue;
            }
            assert_eq!(f.vital(entity).unwrap().hp, 1);
            assert!(cs[i]
                .controller
                .command(LoginCommand::QuestRevive { proof })
                .is_err());
            let q = npc77_request(&cs[i], id, target.life);
            assert_eq!(q.caster_life, proof.incarnation + 1);
            let mut stale = q;
            stale.caster_life = proof.incarnation;
            assert!(cs[i]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(stale))
                .is_err());
            stale = q;
            stale.target_life += 1;
            assert!(cs[i]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(stale))
                .is_err());
            let mut foreign = q;
            foreign.context.key.source_id = if i == 0 { 42 } else { 7 };
            assert!(cs[i]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(foreign))
                .is_err());
            if let Err(error) = cs[i]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(q))
            {
                let view = cs[i].view();
                let frame = view.combat.as_ref().unwrap();
                // The canonical stream may observe the next real death before
                // the typed command cache. Neither is synthetic; in that case
                // wait for the normal death confirmation at the loop head.
                let canonical_dead = view.battle.observation().is_some_and(|o| {
                    crate::modern_lito::battle_window::Stamp::of(o).life == q.caster_life as i64
                        && o.data().actors.0.iter().any(|a| {
                            a.entity == entity as i64
                                && a.vital.as_ref().is_some_and(|v| v.values.hp == 0)
                        })
                });
                assert!(
                    frame.vital(entity).unwrap().hp == 0 || canonical_dead,
                    "only an observed new death may retry input: {error}; canonical={:?}",
                    view.battle.observation().map(|o| (
                        crate::modern_lito::battle_window::Stamp::of(o).life,
                        o.data()
                            .actors
                            .0
                            .iter()
                            .find(|a| a.entity == entity as i64)
                            .and_then(|a| a.vital.as_ref())
                            .map(|v| v.values.hp),
                        crate::modern_lito::battle_window::target_current(
                            o,
                            crate::modern_lito::battle_window::Target::Npc {
                                serial: id.serial as i64,
                                life: target.life as i64,
                            }
                        ),
                    ))
                );
                assert_eq!(frame.npc_disintegrate(entity), Some(&own_before));
                continue;
            }
            assert!(cs[i].view().spell_pending);
            assert_eq!(
                cs[i]
                    .view()
                    .combat
                    .as_ref()
                    .unwrap()
                    .npc_disintegrate(entity),
                Some(&own_before),
                "no optimistic MP, lawful, kill or skill record"
            );
            assert!(cs[i]
                .controller
                .command(LoginCommand::CastNpcDisintegrate(q))
                .is_err());
            wait(cs, d, "NPC77 sparse Quest cast acknowledged", |cs| {
                let v = cs[i].view();
                !v.spell_pending
                    && v.combat.as_ref().is_some_and(|f| {
                        f.world.cast(entity).unwrap().request_sequence == q.sequence
                            // A v37 admission rejection refreshes READ state
                            // without consuming a spell sequence. Still require
                            // a real death and the unchanged prior spell ledger.
                            || (v.action_supported
                                && f.vital(entity).is_some_and(|a| a.hp == 0)
                                && f.world.cast(entity).unwrap().request_sequence + 1 == q.sequence
                                && f.npc_disintegrate(entity) == Some(&own_before))
                    })
            });
            let acknowledged = cs[i].view();
            let f = acknowledged.combat.as_ref().unwrap();
            let s = f.npc_disintegrate(entity).unwrap();
            if s.count == stage as i64 - 1 {
                assert_eq!(s, &own_before, "denial cannot debit mana or lawful");
                assert_eq!(
                    f.vital(entity).unwrap().hp,
                    0,
                    "unexpected denial: {}",
                    acknowledged.status
                );
                assert!(
                    acknowledged.status.contains("Dead"),
                    "unexpected denial: {}",
                    acknowledged.status
                );
                eprintln!("NPC77 source={i} stage={stage}: verified real Dead denial, re-confirming the next life");
                continue;
            }
            assert_eq!(s.count, stage as i64, "{}", acknowledged.status);
            assert_eq!(
                (s.mp_cost, s.mp_spent, s.lawful),
                (cost as u32, cost * stage as i64, 500 + stage as i64)
            );
            let hit = s.hit.as_ref().unwrap();
            assert!(
                targets.insert(id.serial),
                "every wave owns a new NPC serial"
            );
            assert_eq!(hit.target, id);
            assert_eq!(hit.target_life, target.life);
            assert_eq!(hit.caster_life, q.caster_life);
            assert!(hit.death);
            assert_eq!(hit.applied_damage, 1);
            assert_eq!(
                cs[1 - i]
                    .view()
                    .combat
                    .as_ref()
                    .unwrap()
                    .npc_disintegrate((2 - i) as u64),
                Some(&opponent),
                "one force cannot consume another source's mana or change its ledger"
            );
            break;
        }
        next = Instant::now() + Duration::from_millis(5100);
        let effect_deadline = Instant::now() + Duration::from_secs(10);
        wait(cs, d, "NPC77 real shared per-wave effect", |cs| {
            let view = cs[i].view();
            let frame = view.combat.as_ref().unwrap();
            assert!(Instant::now() < effect_deadline,
                "NPC77 source={i} stage={stage} missing effect: vital={:?} life={:?} spell={:?} painted={:?} audio={}",
                frame.vital(entity), quest_life::life(&frame.world, entity),
                frame.npc_disintegrate(entity), cs[i].rendered.as_ref().map(|r| r.revision), npc77_audio(&cs[i]));
            npc77_audio(&cs[i]) > last_sound
        });
        last_sound = npc77_audio(&cs[i]);
        assert!(
            !cs[i].light_arrow_sound,
            "this test did not enable Light Arrow"
        );
    }
    wait(cs, d, "NPC77 four real waves unlock the objective", |cs| {
        cs[i]
            .rendered
            .as_ref()
            .is_some_and(|r| r.quests.len() == 1 && r.quests[0].alive)
    });
    let v = cs[i].view();
    let f = v.combat.as_ref().unwrap();
    let life = quest_life::life(&f.world, entity).unwrap();
    assert!(life.deaths >= 4 && life.respawns >= 4);
    assert_eq!(targets.len(), 4);
    assert_eq!(f.npc_disintegrate(entity).unwrap().count, 4);
    assert_eq!(f.npc_disintegrate(entity).unwrap().mp_spent, cost * 4);
    assert_eq!(f.world.casts.len(), 1);
    assert_eq!(f.world.casts[0].entity_id, entity);
    assert_eq!(
        f.world.casts[0].mp_spent, 0,
        "NPC77 never borrows teleport's ledger"
    );
    assert!(f.npc_spells.is_empty());
    assert!(f.disintegrations.is_empty());
}
