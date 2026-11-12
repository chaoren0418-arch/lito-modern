use lito_modern::modern_lito::{
    native_skill_selection::{Model, Target},
    native_skills,
    native_world::World,
};
fn packet(data: &[u8], key: &str) -> Vec<u8> {
    let v: serde_json::Value = serde_json::from_slice(data).unwrap();
    let s = v["packets"][key].as_str().unwrap();
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
fn world() -> World {
    let mut w = World::default();
    for key in ["world", "start", "status", "actor", "bot"] {
        w.accept(&packet(
            include_bytes!("fixtures/native-world-current.json"),
            key,
        ))
        .unwrap();
    }
    let mut a = w.actors.remove(&0xfedcba98).unwrap();
    a.id = 1;
    a.x = 32729;
    a.y = 32797;
    w.actors.insert(1, a);
    w.status.as_mut().unwrap().object_id = 1;
    let t = w.actors.get_mut(&2).unwrap();
    t.x = 32735;
    t.y = 32800;
    w.hp = Some([50, 50]);
    w.mp = Some([10, 10]);
    w.learned_skills.extend([1, 3, 4, 43, 220]);
    w
}
fn selected(w: &World, id: u16) -> Model {
    let mut m = Model::default();
    m.sync(7, w, true);
    m.select(Some(id)).unwrap();
    m
}
#[test]
fn learned_insert_remove_preserves_id_never_retargets_combo_row() {
    let mut w = world();
    let mut m = selected(&w, 4);
    w.learned_skills.remove(&1);
    m.sync(7, &w, true);
    assert_eq!(m.selected(), Some(4));
    assert_eq!(m.rows()[1].id, 4);
    w.learned_skills.insert(2);
    m.sync(7, &w, true);
    assert_eq!(m.selected(), Some(4));
    assert_eq!(m.rows()[2].id, 4);
    w.learned_skills.remove(&4);
    m.sync(7, &w, true);
    assert_eq!(m.selected(), None);
    w.learned_skills.insert(4);
    m.sync(7, &w, true);
    assert_eq!(m.selected(), None);
    assert!(m.choice(7, &w, Target::capture(&w, 2)).is_err());
}
#[test]
fn owner_change_fences_selection_before_and_after_refresh() {
    let original = world();
    for change in 0..5 {
        let mut w = original.clone();
        let mut m = selected(&w, 3);
        let mut epoch = 7;
        match change {
            0 => epoch += 1,
            1 => w.generation += 1,
            2 => w.local_corrections += 1,
            3 => w.actors.get_mut(&1).unwrap().incarnation += 1,
            _ => w.battle_result = Some(0),
        }
        assert!(m.choice(epoch, &w, None).is_err());
        m.sync(epoch, &w, true);
        assert_eq!(m.selected(), None);
    }
    let mut m = selected(&original, 3);
    m.sync(7, &original, false);
    assert!(m.rows().is_empty());
    assert_eq!(m.selected(), None);
    m.sync(7, &original, true);
    assert_eq!(m.selected(), None);
}
#[test]
fn all_learned_rows_visible_but_unsupported_and_invalid_choices_cannot_cast() {
    let mut w = world();
    w.learned_skills = (1..=220).collect();
    // 200 is not an E149 skill (196–200 have no catalog row); 220 is wired since #160.
    let mut m = selected(&w, 200);
    assert_eq!(m.rows().len(), 220);
    assert!(m.rows()[199].label.contains("尚未支援"));
    assert!(!m.rows()[219].label.contains("尚未支援"));
    assert!(m.choice(7, &w, None).is_err());
    assert!(m.select(Some(0)).is_err());
    assert_eq!(m.selected(), None);
    assert!(m.select(Some(221)).is_err());
    assert_eq!(m.selected(), None);
    m.select(None).unwrap();
    assert_eq!(m.selected(), None);
}
#[test]
fn selection_uses_formal_packets_and_preserves_authority() {
    let w = world();
    let before = serde_json::to_value(&w).unwrap();
    for (id, key) in [(1, "heal"), (3, "shield")] {
        let c = selected(&w, id).choice(7, &w, None).unwrap();
        assert_eq!(
            native_skills::request(&w, c.skill, c.target).unwrap(),
            packet(include_bytes!("fixtures/native-skills-current.json"), key)
        );
    }
    let haste = selected(&w, 43);
    assert!(haste.rows().iter().find(|row| row.id == 43).unwrap().label.contains("加速術"));
    let self_haste = haste.choice(7, &w, None).unwrap();
    assert_eq!(native_skills::request(&w, self_haste.skill, self_haste.target).unwrap(), vec![0x14,5,2,1,0,0,0]);
    let target_haste = haste.choice(7, &w, Target::capture(&w, 2)).unwrap();
    assert_eq!(target_haste.target, Some(2));
    assert_eq!(native_skills::request(&w, target_haste.skill, target_haste.target).unwrap(), vec![0x14,5,2,2,0,0,0]);
    let m = selected(&w, 4);
    assert!(m.choice(7, &w, None).is_err());
    let c = m.choice(7, &w, Target::capture(&w, 2)).unwrap();
    assert_eq!(
        native_skills::request(&w, c.skill, c.target).unwrap(),
        packet(
            include_bytes!("fixtures/native-attack-skills-current.json"),
            "request"
        )
    );
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}
#[test]
fn stale_target_never_becomes_another_actor_or_implicit_self_heal() {
    let mut w = world();
    let target = Target::capture(&w, 2);
    let heal = selected(&w, 1);
    let haste = selected(&w, 43);
    let bolt = selected(&w, 4);
    let shield = selected(&w, 3);
    w.actors.get_mut(&2).unwrap().incarnation += 1;
    assert!(heal.choice(7, &w, target).is_err());
    assert!(haste.choice(7, &w, target).is_err());
    assert!(haste.choice(7, &w, None).is_ok());
    assert!(bolt.choice(7, &w, target).is_err());
    assert_eq!(shield.choice(7, &w, target).unwrap().target, None);
    assert!(heal.choice(7, &w, None).is_ok());
    let target = Target::capture(&w, 2);
    w.actors.get_mut(&2).unwrap().dead = true;
    assert!(heal.choice(7, &w, target).is_err());
    w.actors.remove(&2);
    assert!(bolt.choice(7, &w, target).is_err());
}
#[test]
fn request_rechecks_learned_mp_hp_and_target_range_without_refresh() {
    let w = world();
    let m = selected(&w, 4);
    let t = Target::capture(&w, 2);
    for change in 0..4 {
        let mut w = w.clone();
        match change {
            0 => {
                w.learned_skills.remove(&4);
            }
            1 => w.mp = Some([0, 10]),
            2 => w.hp = Some([0, 50]),
            _ => w.actors.get_mut(&2).unwrap().x += 20,
        }
        assert!(m.choice(7, &w, t).is_err());
    }
}
#[test]
fn actual_render_camera_ground_cells_and_object_hits_reject_stale_worlds() {
    use lito_modern::modern_lito::native_world_scene::Model as Scene;
    use std::{path::Path, sync::atomic::AtomicBool};
    let mut w = world();
    let cancel = AtomicBool::new(false);
    let mut scene = Scene::load(Path::new(r"D:\lineage開發\0-客戶端"), &w, &cancel).unwrap();
    let f = scene.draw_at(&w, &cancel, 0).unwrap();
    for a in f.receipt["actors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|a| a["rendered"] == true)
    {
        let x = a["screen"][0].as_i64().unwrap() as i32;
        let y = a["screen"][1].as_i64().unwrap() as i32;
        if (0..800).contains(&x) && (0..600).contains(&y) {
            let cell = f.ground_cell(&w, x, y).unwrap();
            assert_eq!(
                [cell.x, cell.y],
                [
                    a["world"][0].as_i64().unwrap() as i32,
                    a["world"][1].as_i64().unwrap() as i32
                ]
            );
        }
    }
    for (x, y) in [(-1, 0), (800, 0), (0, -1), (0, 600), (i32::MAX, i32::MIN)] {
        assert!(f.ground_cell(&w, x, y).is_none());
    }
    let hit = f.hits.iter().find(|(id, _)| *id == 2).unwrap().1;
    let (x, y) = ((hit[0] + hit[2]) / 2, (hit[1] + hit[3]) / 2);
    assert_eq!(f.pick_current(&w, x, y, 1), Some(2));
    w.visual_revision += 1;
    assert!(f.pick_current(&w, x, y, 1).is_none());
    w.actors.get_mut(&1).unwrap().incarnation += 1;
    assert!(!f.movement_ready(&w));
    assert!(f.ground_cell(&w, 400, 300).is_none());
    w.actors.get_mut(&1).unwrap().incarnation -= 1;
    w.generation += 1;
    assert!(f.ground_cell(&w, 400, 300).is_none());
}
