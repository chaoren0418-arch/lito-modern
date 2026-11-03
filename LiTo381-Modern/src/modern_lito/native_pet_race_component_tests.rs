use super::*;
use serde_json::{json, Value};
use std::{fs, path::PathBuf};

fn inputs(name: &str) -> Vec<u8> {
    let root = PathBuf::from(
        std::env::var_os("LITO_R533_INPUT_ROOT").expect("explicit adopted input root"),
    );
    assert!(root.is_absolute());
    fs::read(root.join(name)).unwrap()
}
fn number(v: &Value, key: &str) -> i64 {
    v[key].as_i64().unwrap()
}
fn cell(v: &Value, prefix: &str) -> Cell {
    Cell {
        x: i32::try_from(number(v, &format!("{prefix}X"))).unwrap(),
        y: i32::try_from(number(v, &format!("{prefix}Y"))).unwrap(),
    }
}

#[test]
fn r533_pet_race_consumes_96_independent_java_observations() {
    let contract = Contract::parse(&inputs("r533-pet-race-contract.json")).unwrap();
    let bytes = inputs("r533-pet-race-replay-inputs.json");
    assert_eq!(
        format!("{:X}", Sha256::digest(&bytes)),
        "4E750E53C3BF033972EE471E762713B38AA1277562E032B1D7ACA875CED2BF8A"
    );
    let replay: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(replay["namespace"], NAMESPACE);
    assert_eq!(replay["world_or_walkability_proof"], false);
    let mut cases = Vec::new();
    let mut callbacks = 0;
    for program in replay["programs"].as_array().unwrap() {
        let mut states: BTreeMap<String, Reducer> = BTreeMap::new();
        for a in program["actions"].as_array().unwrap() {
            let key = a["state"].as_str().unwrap();
            let operation = a["operation"].as_str().unwrap();
            if operation == "begin" {
                let participants: Vec<_> = a["participants"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_i64().unwrap())
                    .collect();
                let r = contract
                    .begin_component(
                        NAMESPACE,
                        number(a, "map") as u32,
                        &participants,
                        number(a, "nowNanos"),
                        number(a, "deadlineNanos"),
                    )
                    .unwrap();
                states.insert(key.to_owned(), r);
                continue;
            }
            let r = states.get_mut(key).unwrap();
            let observed = match operation {
                "moved" => {
                    callbacks += 1;
                    serde_json::to_value(r.moved(
                        number(a, "player"),
                        cell(a, "from"),
                        cell(a, "to"),
                        number(a, "nowNanos"),
                    ))
                    .unwrap()
                }
                "panelMoved" => {
                    callbacks += 1;
                    let t = r.panel_moved(
                        number(a, "player"),
                        cell(a, "from"),
                        cell(a, "to"),
                        number(a, "nowNanos"),
                    );
                    if a["java_expected"]["observed"].is_number() {
                        json!(t.duration_seconds)
                    } else {
                        serde_json::to_value(t.kind).unwrap()
                    }
                }
                "finishDeadlineNanos" => json!(r.finish_deadline_nanos()),
                "finishWindowExpired" => {
                    let now = number(a, "nowNanos");
                    assert_eq!(
                        r.finish_window_expired(now),
                        a["expected_return"].as_bool().unwrap()
                    );
                    json!(now)
                }
                _ => panic!("unadopted operation: {operation}"),
            };
            if let Some(expected) = a.get("java_expected") {
                assert_eq!(expected["passed"], true);
                assert_eq!(observed, expected["observed"], "case {}", expected["case"]);
                cases.push(expected["case"].as_str().unwrap().to_owned());
            }
        }
    }
    assert_eq!(cases.len(), 96);
    assert_eq!(callbacks, 147);
    let unique: std::collections::BTreeSet<_> = cases.iter().collect();
    assert_eq!(unique.len(), 96);
    let output = PathBuf::from(
        std::env::var_os("LITO_R533_CHECK_OUTPUT").expect("explicit new check output"),
    );
    let result = json!({"state":"R533_NEW_RUST_COMPONENT_MATCHES_96_SEALED_JAVA_OBSERVATIONS",
        "cases":cases,"movement_or_panel_callbacks":callbacks,"new_tests":1,"old_PASS_replays":0,
        "synthetic_coordinates":true,"walkability_proven":false,"runtime_started":false,
        "outer_running_deadline_guard_implemented":false,"server_effects_applied":false,
        "SQL":false,"TLS":false,"GUI":0,"LOADED":false,"complete_modes":16,"total_modes":40});
    // Keep first outcomes; no overwrite of an existing run.
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .unwrap();
    writeln!(f, "{}", serde_json::to_string_pretty(&result).unwrap()).unwrap();
}

#[test]
fn r533_pet_race_namespace_and_panel_domains() {
    let bytes = inputs("r533-pet-race-contract.json");
    let c = Contract::parse(&bytes).unwrap();
    assert!(c.course("SHARED_LINEAGE", 566).is_err());
    assert!(c.course(NAMESPACE, 565).is_err());
    let mut changed = bytes.clone();
    changed.push(b' ');
    assert!(Contract::parse(&changed).is_err());
    assert!(c.begin_component(NAMESPACE, 566, &[], 1, 2).is_err());
    assert!(c.begin_component(NAMESPACE, 566, &[7], 2, 1).is_err());
    let mut r = c
        .begin_component(NAMESPACE, 566, &[7, 42], 1, i64::MAX)
        .unwrap();
    let gate = r.course().gates[0].clone();
    assert!(
        r.moved(
            42,
            Cell {
                x: gate.min_x - 1,
                y: gate.min_y
            },
            Cell {
                x: gate.min_x,
                y: gate.min_y
            },
            2
        )
        .gate_advanced
    );
    let transform = r
        .course()
        .panels
        .iter()
        .find(|p| p.kind == PanelType::Transform)
        .unwrap()
        .point;
    let speed = r
        .course()
        .panels
        .iter()
        .find(|p| p.kind == PanelType::Speed)
        .unwrap()
        .point;
    let outside = |p: Cell| Cell { x: p.x - 1, y: p.y };
    assert_eq!(
        r.panel_moved(999, outside(transform), transform, 3).kind,
        PanelType::None
    );
    assert_eq!(
        r.panel_moved(7, transform, transform, 3).kind,
        PanelType::None
    );
    assert!(r.panel_moved(7, outside(transform), transform, 3).trailing);
    assert_eq!(
        r.panel_moved(7, outside(speed), speed, 3).kind,
        PanelType::Speed
    );
    assert_eq!(
        r.panel_moved(7, outside(transform), transform, 4_000_000_002)
            .kind,
        PanelType::None
    );
    assert_eq!(
        r.panel_moved(7, outside(transform), transform, 4_000_000_003)
            .kind,
        PanelType::Transform
    );
    // A new component owns fresh runner/cooldown state, with no old state carry.
    let mut next = c
        .begin_component(NAMESPACE, 570, &[7, 42], 5, i64::MAX)
        .unwrap();
    let p = next.course().panels[0].point;
    assert_eq!(
        next.panel_moved(7, outside(p), p, 5).kind,
        PanelType::Transform
    );
    // Saturating finish deadlines must also agree near the i64 limit.
    let mut near = c
        .begin_component(NAMESPACE, 569, &[7], i64::MAX - 100, i64::MAX)
        .unwrap();
    for _ in 0..4 {
        for gate in near.course().gates.clone() {
            near.moved(
                7,
                Cell {
                    x: gate.min_x - 1,
                    y: gate.min_y,
                },
                Cell {
                    x: gate.min_x,
                    y: gate.min_y,
                },
                i64::MAX - 1,
            );
        }
    }
    assert_eq!(near.finish_deadline_nanos(), i64::MAX);
    assert!(!near.finish_window_expired(i64::MAX - 1));
    assert!(near.finish_window_expired(i64::MAX));
}
