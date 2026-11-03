use super::*;
use crate::modern_lito::{
    battle_payload::data_tests::fixture, battle_scene::tests::observation,
    resources::SharedMapSectorKey, world_map::SectorInput,
};
use sha2::{Digest, Sha256};

pub(crate) fn trace_map(data: &d::UnvalidatedData) -> Arc<WorldMap> {
    let s = &data.manifest.sectors.0[0];
    let map = Arc::new(
        WorldMap::from_sector_bytes(
            vec![SectorInput {
                key: SharedMapSectorKey {
                    map_id: data.manifest.map as u32,
                    sector_x: s.x as u16,
                    sector_y: s.y as u16,
                },
                bytes: vec![0; 49170],
            }],
            &AtomicBool::new(false),
        )
        .unwrap(),
    );
    assert_eq!(
        map.canonical_digest(),
        data.manifest.digest,
        "test geometry must match Java authority"
    );
    map
}
pub(crate) fn sample() -> (Arc<WorldMap>, d::UnvalidatedData) {
    let mut data = fixture();
    let bytes = vec![0; 49170];
    let key = SharedMapSectorKey {
        map_id: data.manifest.map as u32,
        sector_x: data.manifest.sectors.0[0].x as u16,
        sector_y: data.manifest.sectors.0[0].y as u16,
    };
    let s = &mut data.manifest.sectors.0[0];
    s.byte_length = bytes.len() as i32;
    s.sha256 = Sha256::digest(&bytes).into();
    let map = Arc::new(
        WorldMap::from_sector_bytes(vec![SectorInput { key, bytes }], &AtomicBool::new(false))
            .unwrap(),
    );
    data.manifest.digest = map.canonical_digest();
    (map, data)
}
fn activate(data: &mut d::UnvalidatedData) {
    data.world.phase = d::Phase::Active;
    data.world.revision = 2;
    data.world.loading_elapsed_nanos = 100_000_000;
    data.world.active_elapsed_nanos = 0;
    for a in &mut data.actors.0 {
        a.loaded = true;
        a.step_interval_nanos = 200_000_000;
    }
}
#[test]
fn exact_map_and_observation_own_routes_without_command_or_load_permission() {
    let (map, mut data) = sample();
    let cancel = AtomicBool::new(false);
    let loading = observation(data.clone());
    let space = Space::qualify(map.clone(), loading.clone(), &cancel).unwrap();
    let n = space.snapshot(loading.clone(), &cancel).unwrap();
    assert!(n.matches(&loading));
    assert!(!n.matches(&observation(data.clone())));
    let goal = WorldCell { x: 32770, y: 32770 };
    assert!(n.plan(goal, PathLimits::default(), &cancel).is_err());
    activate(&mut data);
    // Step speed is server-owned; keep the initial policy for this sequence.
    for a in &mut data.actors.0 {
        a.step_interval_nanos = 1_000_000;
    }
    let active = observation(data.clone());
    let n = space.snapshot(active.clone(), &cancel).unwrap();
    assert!(n.matches(&active));
    assert!(n.plan(goal, PathLimits::default(), &cancel).is_ok());
    let other = cell(&data.actors.0[1].position);
    assert!(n.blocked().contains(&other));
    assert!(n.plan(other, PathLimits::default(), &cancel).is_err());
    assert!(!n.teleport_allowed(cell(&data.actors.0[0].position), other));
    assert!(n
        .plan(goal, PathLimits::default(), &AtomicBool::new(true))
        .is_err());
    data.personal.player.epoch += 1;
    assert!(space.snapshot(observation(data), &cancel).is_err());
}
#[test]
fn changed_raw_binding_duplicate_spawns_and_cancellation_are_rejected() {
    let (map, data) = sample();
    let cancel = AtomicBool::new(false);
    let mut changed = data.clone();
    changed.manifest.sectors.0[0].sha256[0] ^= 1;
    assert!(Space::qualify(map.clone(), observation(changed), &cancel).is_err());
    let mut changed = data.clone();
    changed.actors.0[1].position = changed.actors.0[0].position.clone();
    assert!(Space::qualify(map.clone(), observation(changed), &cancel).is_err());
    assert!(Space::qualify(map, observation(data), &AtomicBool::new(true)).is_err());
}

#[test]
fn same_owner_pose_does_not_authorize_obsolete_occupancy_but_clock_only_updates_do() {
    let (map, mut raw) = sample();
    activate(&mut raw);
    let cancel = AtomicBool::new(false);
    let first = observation(raw.clone());
    let space = Space::qualify(map, first.clone(), &cancel).unwrap();
    let navigation = space.snapshot(first, &cancel).unwrap();
    raw.world.revision += 1;
    raw.world.active_elapsed_nanos += 1_000_000;
    raw.world.loading_elapsed_nanos += 1_000_000;
    assert!(navigation.blockers_current(&observation(raw.clone())));
    let owner_position = raw.actors.0[0].position.clone();
    raw.actors.0[1].position.x += 1;
    raw.actors.0[1].heading = 2;
    raw.actors.0[1].last_move_sequence += 1;
    let changed = observation(raw.clone());
    assert_eq!(changed.data().actors.0[0].position, owner_position);
    assert!(!navigation.blockers_current(&changed));
    let refreshed = space.snapshot(changed.clone(), &cancel).unwrap();
    assert!(refreshed.blockers_current(&changed));
    raw.personal.player.epoch += 1;
    assert!(!refreshed.blockers_current(&observation(raw)));
}

/// Called with real Java Engine trace data, not a synthesized legacy Frame.
pub(crate) fn verify_traces(groups: &std::collections::BTreeMap<String, Vec<d::UnvalidatedData>>) {
    let cancel = AtomicBool::new(false);
    let mut count = 0;
    let mut modes = BTreeSet::new();
    for (name, trace) in groups {
        let first = &trace[0];
        if first.world.phase == d::Phase::Aborted {
            continue;
        }
        let map = trace_map(first);
        let space = Space::qualify(map, observation(first.clone()), &cancel)
            .unwrap_or_else(|e| panic!("{name}: mode qualification: {e:#}"));
        if name == "siege" {
            // Each candidate remains a valid canonical packet. The new local
            // mode proof must reject it because the gate no longer separates
            // the roles; packet syntax alone cannot find this problem.
            for index in [0, 1, 10, 31, 62, 63] {
                let mut gap = first.clone();
                gap.siege.as_mut().unwrap().gate.0.remove(index);
                let changed = observation(gap);
                assert!(
                    Space::qualify(space.map.clone(), changed.clone(), &cancel).is_err(),
                    "canonical gate gap {index}"
                );
                assert!(
                    space.snapshot(changed, &cancel).is_err(),
                    "old navigation cannot adopt changed gate policy"
                );
            }
        }
        for data in trace {
            let o = observation(data.clone());
            if data.world.phase == d::Phase::Aborted {
                assert!(space.snapshot(o, &cancel).is_err());
                continue;
            }
            let n = space
                .snapshot(o.clone(), &cancel)
                .unwrap_or_else(|e| panic!("{name}: navigation: {e:#}"));
            assert!(n.matches(&o));
            assert!(n.blockers_current(&o));
            assert_eq!(n.map().canonical_digest(), data.manifest.digest);
            modes.insert(data.room.mode);
            count += 1;
            let owner = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == data.personal.entity)
                .unwrap();
            if data.world.phase != d::Phase::Active
                || !occupies(data, owner.entity)
                || owner.vital.as_ref().is_some_and(|v| v.values.hp == 0)
            {
                assert!(
                    n.mover().is_err(),
                    "dead/finished/departed owner must not generate movement"
                );
            } else {
                assert!(n.mover().is_ok());
            }
            if let Some(s) = &data.siege {
                let from = cell(&s.attack_camp.0[0]);
                let to = cell(&s.defense_camp.0[0]);
                let proof = space.siege.as_ref().unwrap();
                assert!(!proof.allows(from, to, false));
                assert!(proof.allows(from, to, true));
                for g in &s.gate.0 {
                    assert_eq!(n.blocked().contains(&cell(g)), s.gate_revision == 0);
                }
                // Ruined towers remain navigation blockers, matching Java.
                for t in &s.towers.0 {
                    assert!(n.blocked().contains(&cell(&t.cell)));
                }
            }
        }
    }
    assert_eq!(modes, BTreeSet::from([0, 1, 2, 3, 4, 5]));
    eprintln!("canonical navigation: {count} Java observations across all six mode IDs; exact shared-sector identity, static siege cutset/race cycle and dynamic occupancy; no command/LOAD ACK or HWND claim");
}
