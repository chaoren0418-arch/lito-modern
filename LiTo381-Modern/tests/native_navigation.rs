use lito_modern::modern_lito::{
    native_navigation::{HeldInput, Navigation},
    native_speed,
    native_world::World,
    native_world_animation::Animator,
    resources::SharedMapSectorKey,
    sprite_actions::{ActionLimits, SpriteActionCatalog},
    world_map::{SectorInput, WorldMap},
};
fn bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn fixture() -> serde_json::Value {
    serde_json::from_slice(include_bytes!("fixtures/native-navigation-current.json")).unwrap()
}
fn active() -> World {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor"] {
        w.accept(&bytes(f["packets"][k].as_str().unwrap())).unwrap();
    }
    w
}
#[test]
fn honor_spawns_route_across_original_sectors_and_avoid_dynamic_cells(){
    let mut inputs=Vec::new();
    for entry in std::fs::read_dir(r"D:\lineage開發\0-客戶端\LiTo\LTmap\510").unwrap(){
        let path=entry.unwrap().path();let name=path.file_name().unwrap().to_str().unwrap();
        if !name.ends_with(".s32"){continue;}
        inputs.push(SectorInput{key:SharedMapSectorKey{map_id:510,sector_x:u16::from_str_radix(&name[..4],16).unwrap(),sector_y:u16::from_str_radix(&name[4..8],16).unwrap()},bytes:std::fs::read(&path).unwrap()});
    }
    let map=WorldMap::from_native_sector_bytes(inputs,&Default::default()).unwrap();let nav=Navigation::from_geometry(&map).unwrap();
    let start=[32766,32819];let goal=[32820,32766];let path=nav.route_to_adjacent_avoiding(start,goal,|x,y|[x,y]==[32800,32798]).unwrap();
    assert!(path.len()>=53&&path.len()<=128);let mut at=start;
    for h in path{assert!(nav.can_step(i32::from(at[0]),i32::from(at[1]),h));let next=lito_modern::modern_lito::native_world::destination(at[0],at[1],h).unwrap();at=[next.0,next.1];assert_ne!(at,[32800,32798]);}
    assert_eq!(at[0].abs_diff(goal[0]).max(at[1].abs_diff(goal[1])),1);
    assert!(nav.route_to_adjacent_avoiding(start,goal,|_,_|true).is_err());
}
#[test]
fn actual_ltmap_all_32768_direction_tests_match_deployed_server() {
    let b = std::fs::read(r"D:\lineage開發\0-客戶端\LiTo\LTmap\564\7fff8000.s32").unwrap();
    let map = WorldMap::from_native_sector_bytes(
        vec![SectorInput {
            key: SharedMapSectorKey {
                map_id: 564,
                sector_x: 0x7fff,
                sector_y: 0x8000,
            },
            bytes: b,
        }],
        &Default::default(),
    )
    .unwrap();
    let nav = Navigation::from_geometry(&map).unwrap();
    let f = fixture();
    let masks = bytes(f["map"]["masks"].as_str().unwrap());
    assert_eq!(masks.len(), 4096);
    for y in 0..64 {
        for x in 0..64 {
            assert_eq!(
                nav.mask(32704 + x, 32768 + y),
                masks[(y * 64 + x) as usize],
                "cell {x},{y}"
            );
        }
    }
    assert!(!nav.can_step(0, 0, 0));
    assert!(!nav.can_step(32729, 32797, 8));
    let (route, wall) = nav.wall_route([32729, 32797]).unwrap();
    println!("wall route {route:?}, blocked heading {wall}");
}
#[test]
fn deployed_speed_and_brave_modes_update_only_the_addressed_actor() {
    let f = fixture();
    let mut w = active();
    for key in ["speed", "brave"] {
        for row in f[key].as_array().unwrap() {
            let b = bytes(row["body"].as_str().unwrap());
            w.accept(&b).unwrap();
            let a = w.own_actor().unwrap();
            assert_eq!(
                if key == "speed" { a.speed } else { a.brave },
                row["mode"].as_u64().unwrap() as u8
            );
            assert_eq!(
                if key == "speed" {
                    a.speed_seconds
                } else {
                    a.brave_seconds
                },
                if row["mode"] == 0 {
                    0
                } else if key == "speed" {
                    30
                } else {
                    45
                }
            );
        }
    }
    let mut b = bytes(f["speed"][1]["body"].as_str().unwrap());
    b[1..5].copy_from_slice(&999u32.to_le_bytes());
    let before = w.own_actor().unwrap().clone();
    w.accept(&b).unwrap();
    assert_eq!(*w.own_actor().unwrap(), before);
    assert_eq!(w.orphan_updates, 1);
}
#[test]
fn malformed_speed_frames_do_not_mutate_and_multiplier_branches_are_explicit() {
    let f = fixture();
    for key in ["speed", "brave"] {
        let b = bytes(f[key][1]["body"].as_str().unwrap());
        let mut invalid: Vec<Vec<u8>> = (0..b.len()).map(|n| b[..n].to_vec()).collect();
        let mut tail = b.clone();
        tail.push(0);
        invalid.push(tail);
        let mut bad = b.clone();
        bad[5] = 255;
        invalid.push(bad);
        let mut bad = b;
        bad[5] = 0;
        invalid.push(bad);
        for b in invalid {
            let mut w = active();
            let before = serde_json::to_value(&w).unwrap();
            assert!(w.accept(&b).is_err());
            assert_eq!(serde_json::to_value(w).unwrap(), before);
        }
    }
    assert_eq!(
        native_speed::duration(1_000_000_000, 1, 0, 0).unwrap(),
        750_000_000
    );
    assert_eq!(
        native_speed::duration(1_000_000_000, 2, 0, 0).unwrap(),
        2_000_000_000
    );
    assert_eq!(
        native_speed::duration(1_000_000_000, 1, 1, 0).unwrap(),
        562_500_000
    );
    assert_eq!(native_speed::factor(0, 3, 3).unwrap(), (7, 8));
    assert_eq!(native_speed::factor(0, 4, 0).unwrap(), (3, 4));
    assert_eq!(native_speed::factor(0, 4, 3).unwrap(), (1, 1));
    assert_eq!(native_speed::factor(0, 5, 0).unwrap(), (3, 8));
    assert_eq!(native_speed::factor(0, 6, 0).unwrap(), (1, 1));
}
#[test]
fn speed_change_preserves_walk_progress_and_changes_remaining_cadence() {
    let cat = SpriteActionCatalog::parse(
        b"#0 16 test 0.walk(1 2,0.0:12 0.1:12) 3.idle(1 2,8.0:12 8.1:12)\n",
        ActionLimits::default(),
    )
    .unwrap();
    let mut a = active().own_actor().unwrap().clone();
    a.speed = 0;
    a.x = 101;
    a.y = 100;
    a.motion_from = Some([100, 100]);
    let mut animator = Animator::default();
    animator.begin(0, [a.id].into_iter()).unwrap();
    animator.pose(&cat, &a, 0).unwrap();
    a.speed = 1;
    let change = animator.pose(&cat, &a, 500_000_000).unwrap();
    assert_eq!(change.position, [100.5, 100.]);
    assert_eq!(animator.pose(&cat, &a, 874_999_999).unwrap().phase, "walk");
    assert_eq!(animator.pose(&cat, &a, 875_000_000).unwrap().phase, "idle");
}
#[test]
fn held_keys_ignore_repeat_preserve_tap_and_release_or_focus_cancel_stops() {
    let mut input = HeldInput::default();
    input.press(0x68);
    input.press(0x68);
    assert_eq!(input.direction(), Some(0));
    input.sent();
    assert_eq!(input.direction(), Some(0));
    input.release(0x68);
    assert_eq!(input.direction(), None);
    input.press(0x66);
    input.release(0x66);
    assert_eq!(input.direction(), Some(2));
    input.sent();
    assert_eq!(input.direction(), None);
    input.press(0x68);
    input.sent();
    input.press(0x66);
    input.sent();
    input.release(0x66);
    assert_eq!(input.direction(), Some(0));
    input.clear();
    assert_eq!(input.direction(), None);
}
#[test]
fn held_movement_survives_prediction_but_not_a_new_life_or_authority_interruption(){
    use lito_modern::modern_lito::native_navigation::InputStamp;
    let mut w=active();let mut input=HeldInput::default();
    assert!(input.synchronize(Some(InputStamp::capture(2,&w))));input.press(0x68);input.sent();
    w.predict_step(0).unwrap();
    let speed=bytes(fixture()["speed"][1]["body"].as_str().unwrap());w.accept(&speed).unwrap();
    assert!(!input.synchronize(Some(InputStamp::capture(2,&w))));assert_eq!(input.direction(),Some(0));
    let id=w.own_actor().unwrap().id;let mut revive=vec![0x11];revive.extend(id.to_le_bytes());revive.push(0);revive.extend(id.to_le_bytes());revive.extend(0u16.to_le_bytes());
    w.accept(&revive).unwrap();assert!(input.synchronize(Some(InputStamp::capture(2,&w))));assert_eq!(input.direction(),None);
    input.press(0x66);assert_eq!(input.direction(),Some(2));
    assert!(input.synchronize(Some(InputStamp::capture(4,&w))));assert_eq!(input.direction(),None);
    input.press(0x68);w.accept(&[0x63]).unwrap();assert!(input.synchronize(Some(InputStamp::capture(4,&w))));assert_eq!(input.direction(),None);
    input.press(0x68);w.local_corrections+=1;assert!(input.synchronize(Some(InputStamp::capture(4,&w))));assert_eq!(input.direction(),None);
    input.press(0x68);let mut dead=vec![0x20];dead.extend(id.to_le_bytes());dead.push(8);w.accept(&dead).unwrap();
    assert!(input.synchronize(Some(InputStamp::capture(4,&w))));assert_eq!(input.direction(),None);
    input.press(0x68);w.accept(&[0x28,0x34,2,0]).unwrap();assert!(input.synchronize(Some(InputStamp::capture(4,&w))));assert_eq!(input.direction(),None);
    input.press(0x68);assert!(input.synchronize(None));assert_eq!(input.direction(),None);
}
#[test]
fn short_paralysis_cannot_restore_an_old_press_after_release_between_samples(){
    use lito_modern::modern_lito::native_navigation::InputStamp;
    let mut w=active();let mut input=HeldInput::default();
    for mode in [2,4,6,10,12,22,24] {
        input.synchronize(Some(InputStamp::capture(2,&w)));input.press(0x68);input.sent();
        let before=w.control_interruptions;
        w.accept(&[0x25,mode]).unwrap();w.accept(&[0x25,mode]).unwrap();
        assert_eq!(w.control_interruptions,before+1);
        w.accept(&[0x25,mode+1]).unwrap();assert!(w.movement_blocks.is_empty());
        assert!(input.synchronize(Some(InputStamp::capture(2,&w))));assert_eq!(input.direction(),None);
        input.press(0x68);assert_eq!(input.direction(),Some(0));assert!(w.move_request(0).is_ok());
    }
    let stamp=InputStamp::capture(2,&w);assert!(w.accept(&[0x25,8]).is_err());assert_eq!(InputStamp::capture(2,&w),stamp);
    assert!(!input.synchronize(Some(stamp)));assert_eq!(input.direction(),Some(0));
}
