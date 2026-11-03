//! Real Java authority ticks, deliberately coalesced at the production 200ms
//! reader interval. Shared pixels/PCM with synthetic terrain, no TLS/HWND claim.
use super::body_audio_tests::{events, signature};
use super::tests::{observation, trace_scene};
use super::*;
use crate::modern_lito::npc_assets::NpcAction;
use std::collections::{BTreeMap, BTreeSet};

type CueKey = (Instant, Vec<u32>);

pub(crate) fn verify(trace: &[d::UnvalidatedData]) {
    assert_eq!(trace.len(), 13);
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let cancel = AtomicBool::new(false);
    let mut painter = trace_scene(&root, &trace[0]);
    let initial = painter.render(Instant::now(), &cancel).unwrap();
    assert!(events(&painter, &initial).is_empty());
    let start = Instant::now() + Duration::from_secs(60);
    let mut attacks = BTreeMap::<i64, Instant>::new();
    let mut expected = BTreeMap::<i64, Vec<CueKey>>::new();
    let mut heard = BTreeMap::<u64, CueKey>::new();
    let mut coalesced = 0;
    let mut new_attacks = 0;
    let mut sampled_walks = 0;
    for (index, data) in trace.iter().enumerate().skip(1) {
        let at = start + Duration::from_millis(index as u64 * 200);
        let end = at + Duration::from_millis(200);
        painter
            .observe_at(observation(data.clone()), at, &cancel)
            .unwrap();
        let npcs = &data.npcs.as_ref().unwrap().actors.0;
        assert_eq!(npcs.len(), 2);
        let state = data.npc_motion.as_ref().unwrap().state.as_ref().unwrap();
        let mut times = (0..8)
            .map(|n| at + Duration::from_millis(n * 25))
            .collect::<BTreeSet<_>>();
        for npc in npcs {
            let fact = state
                .actors
                .0
                .iter()
                .find(|f| f.serial == npc.serial)
                .unwrap();
            let form = NpcAppearance::new(npc.graphic as u32).unwrap();
            let bank = painter.npcs.as_ref().unwrap();
            // This fixture's shared idle/walk definitions contain no sounds;
            // do not infer an attack cue from a Hit marker or a guessed WAV.
            for action in [NpcAction::Walk, NpcAction::Breath] {
                let q = bank.action(form, action).unwrap();
                assert!((0..q.source_frames().len()).all(|s| q.sound_group(s).unwrap().is_none()));
            }
            let (action, begun, millis) = if npc.action == d::NpcAction::Walk {
                let w = fact.last_walk.as_ref().unwrap();
                let old = &trace[index - 1]
                    .npc_motion
                    .as_ref()
                    .unwrap()
                    .state
                    .as_ref()
                    .unwrap()
                    .actors
                    .0;
                let previous = old
                    .iter()
                    .find(|f| f.serial == npc.serial)
                    .unwrap()
                    .last_walk
                    .as_ref()
                    .map_or(0, |w| w.sequence);
                assert_eq!(w.sequence - previous, 2);
                assert_eq!(w.elapsed_millis, index as i64 * 200);
                assert_ne!(
                    w.from,
                    trace[index - 1]
                        .npcs
                        .as_ref()
                        .unwrap()
                        .actors
                        .0
                        .iter()
                        .find(|a| a.serial == npc.serial)
                        .unwrap()
                        .cell
                );
                assert_eq!(
                    painter.motion.npc_sample(npc, bank, at).unwrap().action,
                    NpcAction::Walk
                );
                coalesced += 1;
                (NpcAction::Walk, at, fact.walk_millis)
            } else {
                assert_eq!(npc.action, d::NpcAction::Attack);
                if matches!(index, 5 | 9) {
                    assert!(data
                        .npcs
                        .as_ref()
                        .unwrap()
                        .exchanges
                        .0
                        .iter()
                        .any(|e| e.serial == npc.serial
                            && e.direction == d::Direction::NpcToParticipant
                            && e.revision > trace[index - 1].world.revision));
                    attacks.insert(npc.serial, at);
                    new_attacks += 1;
                }
                (NpcAction::Attack, attacks[&npc.serial], fact.attack_millis)
            };
            let q = bank.action(form, action).unwrap();
            let mut prefix = 0u128;
            let mut cues = Vec::new();
            for source in (1..q.source_frames().len()).chain(std::iter::once(0)) {
                let offset =
                    (prefix * millis as u128 * 1_000_000).div_ceil(q.duration().as_nanos());
                let boundary = begun + Duration::from_nanos(offset as u64);
                if boundary >= at && boundary < end {
                    if boundary > at {
                        times.insert(boundary - Duration::from_nanos(1));
                    }
                    times.insert(boundary);
                }
                if let Some(group) = q.sound_group(source).unwrap() {
                    cues.push((boundary, group.candidates().to_vec()));
                }
                prefix +=
                    u128::from(q.source_frames()[source].duration_ticks * 1000 / 24) * 1_000_000;
            }
            if action == NpcAction::Attack {
                assert!(
                    !cues.is_empty(),
                    "actual shared attack must own at least one sound cue"
                );
                expected.insert(npc.serial, cues);
            }
        }
        let npc_bank = painter.npcs.as_ref().unwrap().sound_bank();
        for now in times {
            let surface = painter.render(now, &cancel).unwrap();
            assert_eq!(surface.observation.data(), data);
            for npc in npcs {
                let fact = state
                    .actors
                    .0
                    .iter()
                    .find(|f| f.serial == npc.serial)
                    .unwrap();
                let form = NpcAppearance::new(npc.graphic as u32).unwrap();
                let (mut action, begun, duration) = if npc.action == d::NpcAction::Walk {
                    (
                        NpcAction::Walk,
                        at,
                        Duration::from_millis(fact.walk_millis as u64),
                    )
                } else {
                    (
                        NpcAction::Attack,
                        attacks[&npc.serial],
                        Duration::from_millis(fact.attack_millis as u64),
                    )
                };
                let mut elapsed = now.duration_since(begun);
                if elapsed >= duration {
                    action = NpcAction::Breath;
                    elapsed -= duration;
                }
                let q = painter.npcs.as_ref().unwrap().action(form, action).unwrap();
                let phase = if action == NpcAction::Breath {
                    elapsed
                } else {
                    Duration::from_nanos(
                        (elapsed.as_nanos() * q.duration().as_nanos() / duration.as_nanos()) as u64,
                    )
                };
                let source = q.sample(phase, npc.heading as u8).unwrap().0;
                let placement = surface
                    .geometry
                    .npcs
                    .iter()
                    .find(|p| p.id.serial == npc.serial as u64)
                    .unwrap();
                assert_eq!(placement.source_frame, source);
                assert_eq!(placement.cell, cell(&npc.cell));
                let origin = if action == NpcAction::Walk && source != 0 {
                    sampled_walks += 1;
                    &fact.last_walk.as_ref().unwrap().from
                } else {
                    &npc.cell
                };
                assert_eq!(
                    placement.anchor,
                    surface
                        .geometry
                        .projection
                        .cell_to_pixel(cell(origin))
                        .unwrap()
                );
            }
            let output = events(&painter, &surface)
                .into_iter()
                .filter(|e| e.bank.as_ref().is_some_and(|b| Arc::ptr_eq(b, &npc_bank)))
                .collect::<Vec<_>>();
            let mut expected_counts = BTreeMap::<CueKey, usize>::new();
            for key in expected.values().flatten().filter(|(created, _)| {
                *created <= now && now.duration_since(*created) <= Duration::from_millis(500)
            }) {
                *expected_counts.entry(key.clone()).or_default() += 1;
            }
            let mut actual_counts = BTreeMap::<CueKey, usize>::new();
            for event in &output {
                let key = (event.created, event.group.candidates().to_vec());
                *actual_counts.entry(key.clone()).or_default() += 1;
                assert!(event
                    .group
                    .candidates()
                    .iter()
                    .all(|id| *id == 0 || event.bank.as_ref().unwrap().get(*id).is_some()));
                if let Some(old) = heard.insert(event.sequence, key.clone()) {
                    assert_eq!(old, key);
                }
            }
            assert_eq!(actual_counts, expected_counts, "NPC audio must use exact paced shared cue timestamps, independent owners and unchanged source groups");
            if now == at {
                painter
                    .observe_at(observation(data.clone()), now, &cancel)
                    .unwrap();
                let repeated = painter.render(now, &cancel).unwrap();
                assert_eq!(surface.bgra, repeated.bgra);
                assert_eq!(
                    signature(&events(&painter, &surface)),
                    signature(&events(&painter, &repeated))
                );
            }
        }
    }
    assert_eq!(coalesced, 8);
    assert_eq!(new_attacks, 4);
    assert!(sampled_walks > 8 && heard.len() >= 4);
    eprintln!("v43 100ms engine / 200ms reader: two NPCs, {coalesced} retained last steps, {new_attacks} confirmed attacks, {sampled_walks} moving layer samples, {} unique shared NPC cue events; duplicate pixels/events unchanged; no TLS/HWND/device claim", heard.len());
}
