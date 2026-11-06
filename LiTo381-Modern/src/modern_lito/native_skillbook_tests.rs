use super::*;
use crate::modern_lito::native_world_scene::movement_tests::world;

fn learned() -> World {
    let mut w = world();
    w.mp = Some([20, 20]);
    w.learned_skills.extend(1..=220);
    w
}

#[test]
fn complete_book_keeps_all_wire_ids_and_181_catalog_names() {
    let w = learned();
    let mut m = Model::default();
    m.sync(7, &w, true);
    assert_eq!(
        m.rows().iter().map(|r| r.id).collect::<Vec<_>>(),
        (1..=220).collect::<Vec<_>>()
    );
    assert_eq!(catalog::CATALOG.len(), 181);
    let ids: std::collections::BTreeSet<_> = catalog::CATALOG.iter().map(|r| r.0).collect();
    assert_eq!(ids.len(), 181);
    for &(id, name) in catalog::CATALOG {
        assert!(!name.is_empty());
        assert_eq!(catalog::name(id), Some(name));
        assert!(m.rows().iter().any(|r| r.id == id));
    }
    assert!(catalog::name(0).is_none());
    assert!(catalog::name(221).is_none());
    // A name or hotbar binding cannot invent an unimplemented request.
    m.bind(7, Some(220)).unwrap();
    assert_eq!(m.bound(7, &w, 7).unwrap(), 220);
    assert!(m.choice_id_at(7, &w, 220, None, None).is_err());
}

#[test]
fn all_eight_bindings_use_ids_survive_reordering_and_forget_removed_skills() {
    let mut w = learned();
    let mut m = Model::default();
    m.sync(7, &w, true);
    let ids = [2, 4, 26, 42, 58, 63, 181, 206];
    for (slot, id) in ids.into_iter().enumerate() {
        m.bind(slot, Some(id)).unwrap();
    }
    m.select(Some(206)).unwrap();
    w.learned_skills.remove(&1);
    m.sync(7, &w, true);
    for (slot, id) in ids.into_iter().enumerate() {
        assert_eq!(m.bound(7, &w, slot).unwrap(), id);
    }
    assert_eq!(m.selected(), Some(206));
    w.learned_skills.remove(&206);
    assert!(m.bound(7, &w, 7).is_err()); // Before the timer refresh.
    m.sync(7, &w, true);
    assert_eq!(m.selected(), None);
    assert_eq!(m.bindings()[7], None);
    w.learned_skills.insert(206);
    m.sync(7, &w, true);
    assert_eq!(m.bindings()[7], None);
    m.bind(0, None).unwrap();
    assert!(m.bound(7, &w, 0).is_err());
    assert!(m.bind(8, Some(2)).is_err());
    assert!(m.bind(0, Some(221)).is_err());
}

#[test]
fn stale_hotbar_and_selection_retire_on_owner_life_or_context_change() {
    for change in 0..7 {
        let mut w = learned();
        let mut m = Model::default();
        m.sync(7, &w, true);
        m.select(Some(2)).unwrap();
        m.bind(0, Some(2)).unwrap();
        let mut epoch = 7;
        let mut active = true;
        let own = w.own_actor().unwrap().id;
        match change {
            0 => epoch += 1,
            1 => w.generation += 1,
            2 => w.local_corrections += 1,
            3 => w.actors.get_mut(&own).unwrap().incarnation += 1,
            4 => w.actors.get_mut(&own).unwrap().dead = true,
            5 => w.hp = Some([0, 50]),
            _ => active = false,
        }
        if active {
            assert!(m.bound(epoch, &w, 0).is_err());
        }
        m.sync(epoch, &w, active);
        assert_eq!(m.selected(), None);
        assert_eq!(m.bindings(), &[None; 8]);
    }
}

#[test]
fn ground_hotbar_casts_its_binding_not_the_current_book_selection() {
    let w = learned();
    let mut m = Model::default();
    m.sync(7, &w, true);
    m.select(Some(2)).unwrap();
    m.bind(0, Some(58)).unwrap();
    let own = w.own_actor().unwrap();
    let position = [own.x + 1, own.y];
    let ground = Ground::capture(
        7,
        &w,
        super::super::world_map::WorldCell {
            x: i32::from(position[0]),
            y: i32::from(position[1]),
        },
    );
    let before = serde_json::to_value(&w).unwrap();
    let id = m.bound(7, &w, 0).unwrap();
    let c = m.choice_id_at(7, &w, id, None, ground).unwrap();
    assert_eq!(c.skill, 58);
    assert_eq!(c.ground, Some(position));
    assert_eq!(
        native_skills::ground_request(&w, c.skill, c.ground.unwrap()).unwrap()[..3],
        [0x14, 7, 1]
    );
    assert_eq!(m.selected(), Some(2));
    assert!(m.choice_id_at(8, &w, id, None, ground).is_err());
    assert!(m.choice_id_at(7, &w, id, None, None).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(), before);
}

#[test]
fn hotbar_target_and_authority_are_rechecked_without_ui_refresh() {
    let mut w = learned();
    let own = w.own_actor().unwrap().clone();
    let mut other = own.clone();
    other.id = 2;
    other.x += 1;
    w.actors.insert(2, other);
    let mut m = Model::default();
    m.sync(7, &w, true);
    m.bind(0, Some(4)).unwrap();
    let t = Target::capture(&w, 2);
    assert!(m.choice_id_at(7, &w, 4, t, None).is_ok());
    for change in 0..5 {
        let mut invalid = w.clone();
        match change {
            0 => invalid.mp = Some([0, 20]),
            1 => {
                invalid.learned_skills.remove(&4);
            }
            2 => invalid.actors.get_mut(&2).unwrap().incarnation += 1,
            3 => invalid.actors.get_mut(&2).unwrap().x += 20,
            _ => {
                invalid.movement_blocks.insert(2);
            }
        }
        assert!(m.choice_id_at(7, &invalid, 4, t, None).is_err());
    }
}
