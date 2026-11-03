//! Opt-in observer for the SAME authenticated ammo match as the v11 regression.
//! No synthetic canonical frames/revisions, resource fallback, command or LOAD
//! authority. Legacy replies are only independent assertion oracles here.
use super::*;
use crate::modern_lito::{
    battle_follow::{Follower, Observation},
    battle_payload::data as d,
    battle_playback::tests::Recorder,
    battle_read,
    battle_runtime::{BattleRuntime, Status},
    battle_scene::Surface,
    preferences::{Preferences, SharedPreferences},
    protocol::Ack,
};

struct Owner {
    request: battle_read::Request,
    follower: Follower,
    runtime: BattleRuntime,
    last_surface: Option<Arc<Surface>>,
    audio: Recorder,
    gain: u8,
}

pub(super) struct Observer {
    owners: [Owner; 2],
    checks: usize,
}

impl Observer {
    pub(super) fn new(
        root: &std::path::Path,
        ticket: &world::Frame,
        sun: SelectionContext,
        moon: SelectionContext,
        sun_ack: Ack,
        moon_ack: Ack,
    ) -> Self {
        let owners = [(sun, sun_ack, 50), (moon, moon_ack, 25)].map(|(context, ack, gain)| {
            assert!(matches!(ack.version, 36 | 37));
            let request = battle_read::Request {
                context,
                channel_id: CHANNEL,
                world: d::WorldId {
                    room: ticket.room.summary.room_id as i64,
                    match_id: ticket.match_id as i64,
                    generation: ticket.generation as i64,
                },
            };
            let preferences = Arc::new(SharedPreferences::default());
            preferences
                .apply(Preferences {
                    volume_percent: gain,
                    ..Preferences::default()
                })
                .unwrap();
            Owner {
                follower: Follower::new(request.clone(), ack).unwrap(),
                request,
                runtime: BattleRuntime::with_preferences(root.to_path_buf(), preferences).unwrap(),
                last_surface: None,
                audio: Recorder::new(),
                gain,
            }
        });
        Self { owners, checks: 0 }
    }

    pub(super) fn check(
        &mut self,
        sun: &mut ModernTlsConnection,
        moon: &mut ModernTlsConnection,
        legacy: &combat::Frame,
        deadline: &Deadline,
    ) {
        for (owner, connection) in self.owners.iter_mut().zip([&mut *sun, &mut *moon]) {
            let old_world = connection.world_snapshot().cloned();
            let old_combat = connection.combat_snapshot().cloned();
            let reply = connection.read_battle(owner.request.clone()).unwrap();
            assert_eq!(reply.state, battle_read::State::Data);
            owner
                .follower
                .accept(reply)
                .expect("live canonical ammo follow");
            assert_eq!(connection.world_snapshot(), old_world.as_ref());
            assert_eq!(connection.combat_snapshot(), old_combat.as_ref());
            let observation = owner.follower.observation().unwrap().clone();
            compare(&observation, &owner.request, legacy);
            if owner.runtime.observation().is_none() {
                owner.runtime.attach(observation).unwrap();
            } else {
                owner.runtime.update(observation).unwrap();
            }
            owner.runtime.poll();
        }
        let a = self.owners[0].follower.observation().unwrap().clone();
        let b = self.owners[1].follower.observation().unwrap().clone();
        assert_eq!(a.data().presentation, b.data().presentation);
        assert_eq!(a.data().result, b.data().result);
        assert_ne!(a.personal().player(), b.personal().player());
        assert_eq!(a.personal().player().object, b.personal().player().object);
        // Equal object IDs from different sources must never exchange private
        // gear, stocks, skills or the connection-bound selection epoch.
        assert!(self.owners[0].runtime.update(b.clone()).is_err());
        assert_eq!(
            self.owners[0].runtime.observation().unwrap().data(),
            a.data()
        );
        let expected = [a, b];
        let mut completed = [false; 2];
        let mut keepalive = Instant::now();
        while !completed.iter().all(|done| *done) {
            deadline.check("live canonical combat surface");
            for (index, owner) in self.owners.iter_mut().enumerate() {
                match owner.runtime.poll() {
                    Status::Ready(surface)
                        if surface.observation.data() == expected[index].data() =>
                    {
                        assert_eq!((surface.width, surface.height), (800, 600));
                        assert_eq!(surface.bgra.len(), 800 * 600 * 4);
                        assert_eq!(surface.geometry.actors.len(), 2);
                        let navigation = surface
                            .navigation
                            .as_ref()
                            .expect("live canonical navigation");
                        assert!(navigation.matches(&surface.observation));
                        assert_eq!(
                            navigation.map().canonical_digest(),
                            expected[index].data().manifest.digest
                        );
                        assert_eq!(
                            surface.geometry.revision as i64,
                            expected[index].data().world.revision
                        );
                        assert_eq!(
                            surface.observation.personal().player(),
                            &owner.request.player()
                        );
                        assert!(surface.bgra.chunks_exact(4).all(|p| p[3] == 255));
                        let audio = owner.runtime.audio_view();
                        let (_, prepared) = audio
                            .publication
                            .expect("qualified canonical audio publication");
                        assert_eq!(prepared.preferences, audio.preferences);
                        if let Some(frame) = &prepared.frame {
                            assert_eq!(prepared.gain, Some(owner.gain));
                            for event in frame.events.iter() {
                                let bank = event.bank.as_deref().unwrap_or(&frame.bank);
                                for id in event.group.candidates().iter().filter(|id| **id != 0) {
                                    let sound = bank.get(*id).unwrap();
                                    assert_eq!(sound.pcm_bytes(), sound.with_gain(owner.gain).unwrap().pcm_bytes(),
                                        "actual shared PCM must have this owner's gain, not another source's settings");
                                }
                            }
                        }
                        assert!(surface
                            .bgra
                            .chunks_exact(4)
                            .any(|p| p != &surface.bgra[..4]));
                        if let Some(retained) = &owner.last_surface {
                            // Subsequent network/worker activity cannot mutate
                            // an already published immutable observation.
                            assert!(
                                retained.observation.data().world.revision
                                    <= surface.observation.data().world.revision
                            );
                        }
                        owner.last_surface = Some(surface);
                        completed[index] = true;
                    }
                    Status::Loading | Status::Ready(_) => {}
                    status => panic!("live canonical combat worker: {status:?}"),
                }
                let playback = owner.audio.tick(&owner.runtime, true, Instant::now());
                assert!(
                    !playback.device_failed,
                    "recording sink must accept prepared live PCM"
                );
            }
            if keepalive.elapsed() >= Duration::from_secs(1) {
                sun.ping().unwrap();
                moon.ping().unwrap();
                keepalive = Instant::now();
            }
            if !completed.iter().all(|done| *done) {
                deadline.pause("live canonical combat worker poll");
            }
        }
        self.checks += 1;
    }

    pub(super) fn verify_result(&self, legacy: &combat::Frame) {
        assert!(
            self.checks >= 9,
            "loading/active/item/miss/hit/denial/select/finish/saved"
        );
        let completion = legacy.world.completion.as_ref().unwrap();
        assert_eq!(completion.state, world::SettlementState::Saved);
        for owner in &self.owners {
            assert!(owner.audio.count() > 0, "each real TLS owner must deliver at least one shared PCM cue to the recording sink");
            let data = owner.follower.observation().unwrap().data();
            assert_eq!(data.world.phase, d::Phase::Finished);
            let result = data.result.as_ref().unwrap();
            assert_eq!(result.result_id, completion.event.result_id);
            assert_eq!(result.engine_revision, completion.event.engine_revision);
            assert_eq!(result.match_id, completion.event.match_id as i64);
            assert_eq!(result.generation, completion.event.generation as i64);
            assert_eq!(result.map, 514);
            assert_eq!(result.mode, 1);
            assert_eq!(result.participants.0.len(), 2);
            for (actual, previous) in result
                .participants
                .0
                .iter()
                .zip(&completion.event.participants)
            {
                assert_eq!(actual.identity.kind, d::MemberKind::Human);
                assert_eq!(actual.identity.source, previous.key.source_id as i32);
                assert_eq!(actual.identity.id, previous.key.object_id as i64);
                assert_eq!(actual.entity, previous.entity_id as i64);
                assert_eq!(actual.team, previous.team as i32);
                assert_eq!(
                    actual.verdict == d::Verdict::Win,
                    previous.verdict == world::Verdict::Win
                );
            }
        }
        eprintln!("canonical ammo TLS: {} dual-owner checkpoints; real commands, schema2 workers and one durable result; worker gain50/25 and recorded cue counts {:?} (no device claim)", self.checks, self.owners.each_ref().map(|o| o.audio.count()));
    }

    pub(super) fn shutdown(&mut self, deadline: &Deadline) {
        for owner in &mut self.owners {
            owner.runtime.shutdown();
            assert_eq!(
                owner
                    .audio
                    .tick(&owner.runtime, true, Instant::now())
                    .started,
                0
            );
        }
        while self.owners.iter().any(|o| !o.runtime.is_quiescent()) {
            for owner in &mut self.owners {
                owner.runtime.poll();
            }
            deadline.pause("canonical ammo worker retirement");
        }
    }
}

fn compare(observation: &Observation, request: &battle_read::Request, legacy: &combat::Frame) {
    let data = observation.data();
    assert_eq!(observation.personal().player(), &request.player());
    assert_eq!(data.world.identity, request.world);
    assert_eq!(data.manifest.map, legacy.world.manifest.map_id as i32);
    assert_eq!(data.manifest.digest, legacy.world.manifest.digest);
    let phase = match legacy.world.phase {
        world::Phase::Loading => d::Phase::Loading,
        world::Phase::Active => d::Phase::Active,
        world::Phase::Finished => d::Phase::Finished,
        _ => panic!("unexpected ammo checkpoint phase"),
    };
    assert_eq!(data.world.phase, phase);
    assert_eq!(data.actors.0.len(), legacy.world.actors.len());
    let public = &data.presentation.as_ref().expect("negotiated schema2").0;
    assert_eq!(public.len(), 2);
    for a in &data.actors.0 {
        let old = legacy
            .world
            .actors
            .iter()
            .find(|x| x.entity_id == a.entity as u64)
            .unwrap();
        let v = legacy.vital(old.entity_id).unwrap();
        assert_eq!(
            (a.position.x, a.position.y),
            (old.position.x, old.position.y)
        );
        assert_eq!(a.last_move_sequence, old.sequence as i64);
        assert_eq!(a.loaded, old.loaded);
        let values = &a.vital.as_ref().unwrap().values;
        assert_eq!(
            (values.hp, values.maximum_hp, values.mp, values.maximum_mp),
            (v.hp, v.max_hp, v.mp, v.max_mp)
        );
        assert_eq!(
            a.vital.as_ref().unwrap().attack_sequence,
            v.attack_sequence as i64
        );
        let receipt = public.iter().find(|p| p.entity == a.entity).unwrap();
        let arrows = legacy.ammo(old.entity_id).unwrap();
        assert_eq!(
            receipt.shot.as_ref().map_or(0, |s| s.count),
            arrows.shot_count as i64
        );
        if let Some(shot) = &receipt.shot {
            assert_eq!(shot.attack_sequence, arrows.attack_sequence as i64);
            assert_eq!((shot.from.x, shot.from.y), (arrows.from.x, arrows.from.y));
            assert_eq!((shot.to.x, shot.to.y), (arrows.to.x, arrows.to.y));
            assert_eq!(shot.sprite, 66);
            assert_eq!(shot.hit, arrows.hit);
            let d::Target::Participant { entity, identity } = &shot.target else {
                panic!("ordinary PvP bow target")
            };
            assert_eq!(*entity, arrows.last_target_entity_id as i64);
            let target = data.actors.0.iter().find(|t| t.entity == *entity).unwrap();
            assert_eq!(*identity, data.scene_member(target).unwrap().identity);
            // Public target incarnations are one-based; resource accounting
            // records the zero-based number of completed respawns. This mode
            // has no respawn, so the current incarnation must still be exact.
            assert_eq!(
                shot.target_life,
                target
                    .resources
                    .as_ref()
                    .unwrap()
                    .life
                    .checked_add(1)
                    .unwrap()
            );
            assert!(shot.revision <= data.world.revision);
        }
        let items = legacy.item(old.entity_id).unwrap();
        assert_eq!(
            receipt.item.as_ref().map_or(0, |i| i.count),
            items.use_count as i64
        );
        if let Some(item) = &receipt.item {
            assert_eq!(item.effect, items.effect);
            assert_eq!(
                item.effect, 191,
                "actual Haste use is not a synthetic effect"
            );
        }
    }
    let id = data.personal.entity as u64;
    let private_ammo = observation.personal().ammunition().unwrap();
    let old_ammo = legacy.ammo(id).unwrap();
    assert_eq!(private_ammo.entity_id, id);
    assert_eq!(private_ammo.stocks, old_ammo.stocks);
    assert_eq!(private_ammo.request_sequence, old_ammo.request_sequence);
    assert_eq!(private_ammo.selected_item_id, old_ammo.selected_item_id);
    assert_eq!(private_ammo.shot_count, old_ammo.shot_count);
    assert_eq!(observation.personal().weight(), legacy.weight(id));
    let supplies = observation.personal().supplies().unwrap();
    let old_items = legacy.item(id).unwrap();
    assert_eq!(supplies.history.uses, old_items.use_count as i64);
    assert_eq!(supplies.stocks.0.len(), old_items.stocks.len());
    for (stock, old) in supplies.stocks.0.iter().zip(&old_items.stocks) {
        assert_eq!(
            (stock.item, stock.initial, stock.remaining),
            (old.item_id, old.initial, old.remaining)
        );
    }
    let equipment = data.personal.equipment.as_ref().unwrap();
    assert_eq!(equipment.source, request.player().source);
    assert_eq!(equipment.object, request.player().object);
}
