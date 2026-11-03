use super::*;
use crate::modern_lito::{
    native_pet_race_component::{Contract, PanelType},
    native_pet_race_offline::OfflineRace,
    native_resources::{BytePin, NativeSectorKey, SectorPin},
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::PathBuf, sync::Arc};

fn root() -> PathBuf {
    let p = PathBuf::from(
        std::env::var_os("LITO_R534_INPUT_ROOT").expect("explicit owned Native input root"),
    );
    assert!(p.is_absolute());
    p
}
fn read(name: &str) -> Vec<u8> {
    fs::read(root().join(name)).unwrap()
}
fn doc(name: &str) -> Value {
    serde_json::from_slice(&read(name)).unwrap()
}
fn cell(v: &Value) -> Cell {
    Cell {
        x: i32::try_from(v[0].as_i64().unwrap()).unwrap(),
        y: i32::try_from(v[1].as_i64().unwrap()).unwrap(),
    }
}
fn sha(s: &str) -> [u8; 32] {
    assert_eq!(s.len(), 64);
    let mut out = [0; 32];
    for (i, b) in out.iter_mut().enumerate() {
        *b = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap();
    }
    out
}
fn sector(map: u32, key: &str) -> SharedMapSectorKey {
    SharedMapSectorKey {
        map_id: map,
        sector_x: u16::from_str_radix(&key[..4], 16).unwrap(),
        sector_y: u16::from_str_radix(&key[4..], 16).unwrap(),
    }
}
fn inputs(map: u32) -> (Vec<SectorInput>, Vec<SectorPin>) {
    let d = doc("s32-layer3-to-actual-reader-byte-comparison.json");
    let mut inputs = Vec::new();
    let mut pins = Vec::new();
    for r in d["sectors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["map"] == map)
    {
        let name = r["sector"].as_str().unwrap();
        let key = sector(map, name);
        inputs.push(SectorInput {
            key,
            bytes: read(&format!("native/{map}/{name}.s32")),
        });
        pins.push(SectorPin {
            key: NativeSectorKey {
                map_id: map,
                x: key.sector_x,
                y: key.sector_y,
            },
            pin: BytePin {
                bytes: r["raw"]["bytes"].as_u64().unwrap(),
                sha256: sha(r["raw"]["sha256"].as_str().unwrap()),
            },
        });
    }
    (inputs, pins)
}
fn terrain(map: u32) -> Arc<NativeTerrain> {
    let (i, p) = inputs(map);
    Arc::new(NativeTerrain::from_sector_bytes(NAMESPACE, i, &p, &AtomicBool::new(false)).unwrap())
}
struct Bytes<'a> {
    b: &'a [u8],
    at: usize,
}
impl<'a> Bytes<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let b = &self.b[self.at..self.at + n];
        self.at += n;
        b
    }
    fn u16(&mut self, le: bool) -> u16 {
        let b = self.take(2).try_into().unwrap();
        if le {
            u16::from_le_bytes(b)
        } else {
            u16::from_be_bytes(b)
        }
    }
    fn u32(&mut self, le: bool) -> u32 {
        let b = self.take(4).try_into().unwrap();
        if le {
            u32::from_le_bytes(b)
        } else {
            u32::from_be_bytes(b)
        }
    }
}
fn binary_maps(bytes: &[u8], magic: &[u8], le: bool, size: usize) -> BTreeMap<(u32, u32), Vec<u8>> {
    let mut b = Bytes { b: bytes, at: 0 };
    assert_eq!(b.take(magic.len()), magic);
    let maps = b.u16(le);
    let mut out = BTreeMap::new();
    for _ in 0..maps {
        let map = u32::from(b.u16(le));
        let count = b.u16(le);
        for _ in 0..count {
            let key = b.u32(le);
            let bits = b.take(size).to_vec();
            assert!(out.insert((map, key), bits).is_none());
        }
    }
    assert_eq!(b.at, bytes.len());
    out
}
fn check_edge(t: &NativeTerrain, v: &Value) {
    let from = cell(&v["from"]);
    let to = cell(&v["to"]);
    assert_eq!(
        t.contains(from),
        v["from_contains"].as_bool().unwrap(),
        "{}",
        v["kind"]
    );
    assert_eq!(
        t.contains(to),
        v["to_contains"].as_bool().unwrap(),
        "{}",
        v["kind"]
    );
    assert_eq!(
        t.standing(from),
        v["from_standing"].as_bool().unwrap(),
        "{}",
        v["kind"]
    );
    assert_eq!(
        t.standing(to),
        v["to_standing"].as_bool().unwrap(),
        "{}",
        v["kind"]
    );
    assert_eq!(
        t.can_step(from, to),
        v["Arena_nativeCanStep"].as_bool().unwrap(),
        "{}",
        v["kind"]
    );
}
struct Replay {
    edges: usize,
    gates: usize,
    panel_visits: usize,
    last_finished: bool,
}
fn replay(r: &mut OfflineRace, v: &Value, now: &mut i64, connector: bool) -> Replay {
    assert_eq!(r.position(7).unwrap(), cell(&v["start"]));
    let directions = v["directions"].as_str().unwrap();
    assert_eq!(directions.len(), v["edges"].as_u64().unwrap() as usize);
    let panel = v.get("panel").filter(|p| !p.is_null()).map(cell);
    let mut visits = 0;
    let mut entries = Vec::new();
    let mut cross = 0;
    let mut last_finished = false;
    for (i, h) in directions.bytes().enumerate() {
        assert!((b'0'..=b'7').contains(&h));
        let (dx, dy) = DIRECTIONS[usize::from(h - b'0')];
        let from = r.position(7).unwrap();
        let to = Cell {
            x: from.x + dx,
            y: from.y + dy,
        };
        *now += 1_000_000;
        let step = r.step(7, to, *now).unwrap();
        assert_eq!(step.from, from);
        assert_eq!(step.to, to);
        last_finished = step.race.finished;
        if (
            (from.x - 32704).div_euclid(64),
            (from.y - 32704).div_euclid(64),
        ) != ((to.x - 32704).div_euclid(64), (to.y - 32704).div_euclid(64))
        {
            cross += 1;
        }
        if panel == Some(to) {
            visits += 1;
        }
        if step.race.gate_advanced {
            let gate = entries.len();
            entries
                .push(json!({"gate":gate,"from":[from.x,from.y],"to":[to.x,to.y],"edge_index":i}));
        }
        if connector {
            assert!(!step.race.gate_advanced, "connector advanced a gate");
        }
    }
    assert_eq!(r.position(7).unwrap(), cell(&v["end"]));
    if !connector {
        assert_eq!(json!(entries), v["gate_entries"]);
        assert_eq!(entries.len(), 5);
        assert_eq!(cross, v["cross_sector_edges"].as_u64().unwrap() as usize);
        assert_eq!(visits, v["panel_visits"].as_u64().unwrap() as usize);
    }
    Replay {
        edges: directions.len(),
        gates: entries.len(),
        panel_visits: visits,
        last_finished,
    }
}
fn race(c: &Contract, t: Arc<NativeTerrain>, start: Cell) -> OfflineRace {
    OfflineRace::begin(c, t, &[(7, start)], 0, i64::MAX).unwrap()
}

#[test]
fn r534_native_terrain_raw_reader_masks_and_composed_routes() {
    let contract = Contract::parse(&read("adopted-r533-component-contract.json")).unwrap();
    let witnesses = doc("native-race-route-witness.json");
    let extra = doc("native-spawn-connectors-and-boundary-rejections.json");
    assert_eq!(witnesses["namespace"], NAMESPACE);
    assert_eq!(extra["namespace"], NAMESPACE);
    let masks = read("native-directed-masks-v1.bin");
    assert_eq!(
        format!("{:X}", Sha256::digest(&masks)),
        "C85EA5F541F8FB1C5910B318FEBFC5D002F17638BABAB472E1B7D86BF05A6B6D"
    );
    let oracle = binary_maps(&masks, b"NRDM\x01", false, 4096);
    let catalog_bytes = read("classes/battle/arena-passable-v1.bin");
    assert_eq!(
        format!("{:X}", Sha256::digest(&catalog_bytes)),
        "665D173742317AB023BB5EFFAEE7FFEA35E044517DF21FBA868FF694C430A7E7"
    );
    let catalog = binary_maps(&catalog_bytes, b"LTAP\x02", true, 1024);
    let mut mask_cells = 0;
    let mut raw_bits = 0;
    let mut mask_differences = Vec::new();
    let mut maps = Vec::new();
    let mut totals = [0usize; 9]; // spawns,panels,connectors,composed panels,fourlaps,edges,gates,panel visits,edge witnesses
    for m in witnesses["maps"].as_array().unwrap() {
        let map = m["map"].as_u64().unwrap() as u32;
        let t = terrain(map);
        assert_eq!(
            t.geometry().canonical_digest(),
            sha(m["pure_S32_canonical_sha256"].as_str().unwrap())
        );
        for b in t.geometry().bindings() {
            let key = (u32::from(b.key.sector_x) << 16) | u32::from(b.key.sector_y);
            let bits = t.packed_layer3(b.key).unwrap();
            assert_eq!(bits.as_slice(), catalog[&(map, key)]);
            raw_bits += 8192;
            let (ox, oy) = b.key.world_origin();
            for y in 0..64 {
                for x in 0..64 {
                    let actual = t.direction_mask(Cell {
                        x: ox + x,
                        y: oy + y,
                    });
                    let expected = oracle[&(map, key)][(y * 64 + x) as usize];
                    mask_cells += 1;
                    if actual != expected {
                        mask_differences.push(json!({"map":map,"sector":format!("{key:08x}"),"x":ox+x,"y":oy+y,"actual":actual,"expected":expected}));
                    }
                }
            }
        }
        let course = contract.course(NAMESPACE, map).unwrap();
        for s in m["spawns"].as_array().unwrap() {
            let p = cell(&s["point"]);
            assert_eq!(p, course.spawns[s["slot"].as_u64().unwrap() as usize]);
            assert!(t.standing(p));
            check_edge(&t, &s["initial_heading_edge"]);
            let mut r = race(&contract, t.clone(), p);
            let q = replay(&mut r, &s["ordered_one_lap_route"], &mut 0, false);
            assert!(!q.last_finished);
            totals[0] += 1;
            totals[5] += q.edges;
            totals[6] += q.gates;
        }
        for p in m["panels"].as_array().unwrap() {
            let point = cell(&p["point"]);
            assert!(course
                .panels
                .iter()
                .any(|x| x.point == point && serde_json::to_value(x.kind).unwrap() == p["type"]));
            let mut r = race(&contract, t.clone(), course.spawns[0]);
            let q = replay(&mut r, &p["route_in_one_ordered_lap"], &mut 0, false);
            assert!(q.panel_visits > 0);
            totals[1] += 1;
            totals[5] += q.edges;
            totals[6] += q.gates;
            totals[7] += q.panel_visits;
        }
        let additional = extra["maps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["map"] == map)
            .unwrap();
        for v in m["edge_witnesses"]
            .as_array()
            .unwrap()
            .iter()
            .chain(additional["boundary_rejections"].as_array().unwrap())
        {
            check_edge(&t, v);
            totals[8] += 1;
        }
        for connector in additional["spawn_connectors"].as_array().unwrap() {
            let start = cell(&connector["start"]);
            let mut r = race(&contract, t.clone(), start);
            let mut now = 0;
            let q = replay(&mut r, connector, &mut now, true);
            totals[2] += 1;
            totals[5] += q.edges;
            assert_eq!(r.position(7).unwrap(), course.spawns[0]);
            for p in m["panels"].as_array().unwrap() {
                let mut r = race(&contract, t.clone(), start);
                let mut now = 0;
                let a = replay(&mut r, connector, &mut now, true);
                let b = replay(&mut r, &p["route_in_one_ordered_lap"], &mut now, false);
                assert!(b.panel_visits > 0);
                totals[3] += 1;
                totals[5] += a.edges + b.edges;
                totals[6] += b.gates;
                totals[7] += b.panel_visits;
            }
            let mut final_finished = false;
            for lap in m["four_lap_anchor_route"].as_array().unwrap() {
                let q = replay(&mut r, &lap["route"], &mut now, false);
                assert_eq!(q.last_finished, lap["lap"] == 4);
                final_finished = q.last_finished;
                totals[5] += q.edges;
                totals[6] += q.gates;
            }
            assert!(final_finished);
            totals[4] += 1;
            r.retire();
            assert!(r.position(7).is_err());
        }
        maps.push(json!({"map":map,"raw_sectors":t.geometry().bindings().len(),"native_masks_compared":true,"pure_parser_canonical_equal":true}));
    }
    let output = PathBuf::from(
        std::env::var_os("LITO_R534_CHECK_OUTPUT").expect("new check evidence output"),
    );
    let result = json!({"state":if mask_differences.is_empty(){"R534_NATIVE_RAW_TERRAIN_AND_OFFLINE_REDUCER_ROUTES_MATCH"}else{"R534_NATIVE_MASK_DIFFERENCES"},"maps":maps,"Layer3_u16_bit0_values_compared":raw_bits,"actual_native_mask_cells_compared":mask_cells,"heading_edge_comparisons":mask_cells*8,"mask_differences":mask_differences,"spawn_routes":totals[0],"panel_routes":totals[1],"spawn_connectors":totals[2],"composed_spawn_panel_routes":totals[3],"composed_spawn_four_lap_routes":totals[4],"offline_terrain_checked_steps":totals[5],"ordered_gate_entries":totals[6],"panel_visits":totals[7],"edge_and_boundary_witnesses":totals[8],"old96_PASS_replayed":false,"mask_bytes_used_as_terrain_inputs":false,"dynamic_occupancy_proven":false,"outer_RUNNING_roster_clock_proven":false,"server_effects_applied":false,"SQL":false,"TLS":false,"GUI":0,"LOADED":false,"new_full_bindings":0});
    let mut f = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(output)
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&result).unwrap()).unwrap();
    assert!(
        mask_differences.is_empty(),
        "Native reader masks differ from actual Java"
    );
    assert_eq!(raw_bits, 98304);
    assert_eq!(mask_cells, 49152);
    assert_eq!(&totals[..5], &[30, 26, 30, 260, 30]);
    assert_eq!(totals[8], 34);
}

#[test]
fn r534_native_terrain_rejects_unqualified_inputs_and_keeps_offline_state_atomic() {
    let (i, p) = inputs(566);
    assert!(
        NativeTerrain::from_sector_bytes("SHARED_LINEAGE", i, &p, &AtomicBool::new(false)).is_err()
    );
    let (mut i, p) = inputs(566);
    i[0].bytes[0] ^= 1;
    assert!(NativeTerrain::from_sector_bytes(NAMESPACE, i, &p, &AtomicBool::new(false)).is_err());
    let (i, p) = inputs(566);
    assert!(NativeTerrain::from_sector_bytes(NAMESPACE, i, &p, &AtomicBool::new(true)).is_err());
    let c = Contract::parse(&read("adopted-r533-component-contract.json")).unwrap();
    let t = terrain(566);
    let spawn = c.course(NAMESPACE, 566).unwrap().spawns[0];
    assert!(t.can_step(spawn, spawn));
    assert!(
        !t.geometry().can_step(world(spawn), world(spawn)),
        "pure WorldMap policy must remain unchanged"
    );
    let mut r = race(&c, t.clone(), spawn);
    assert!(r.step(999, spawn, 1).is_err());
    assert!(r
        .step(
            7,
            Cell {
                x: spawn.x + 2,
                y: spawn.y
            },
            1
        )
        .is_err());
    assert_eq!(r.position(7).unwrap(), spawn);
    assert!(r.step(7, spawn, -1).is_err());
    assert_eq!(r.position(7).unwrap(), spawn);
    let step = r.step(7, spawn, 1).unwrap();
    assert!(!step.race.gate_advanced);
    assert_eq!(step.panel.kind, PanelType::None);
    assert!(!t.contains(Cell {
        x: i32::MIN,
        y: i32::MAX
    }));
    assert!(!t.can_step(
        spawn,
        Cell {
            x: i32::MAX,
            y: spawn.y
        }
    ));
    r.retire();
    assert!(r.step(7, spawn, 2).is_err());
    assert!(OfflineRace::begin(&c, t, &[(7, Cell { x: 0, y: 0 })], 0, 1).is_err());
}
