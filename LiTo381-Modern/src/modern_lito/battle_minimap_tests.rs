use super::*;
use crate::modern_lito::{
    battle_scene::tests::observation,
    battle_scene::Surface,
    battle_space::{tests::sample, Space},
    battle_window,
    compositor::Viewport,
};

fn navigation(
    data: crate::modern_lito::battle_payload::data::UnvalidatedData,
    map: Arc<WorldMap>,
) -> Arc<Navigation> {
    let o = observation(data);
    Space::qualify(map, o.clone(), &AtomicBool::new(false))
        .unwrap()
        .snapshot(o, &AtomicBool::new(false))
        .unwrap()
}
fn panel(nav: &Navigation, old: Option<&Panel>) -> Arc<Panel> {
    Panel::prepare(nav, old, &AtomicBool::new(false)).unwrap()
}
fn pixel(rgba: &[u8], x: i32, y: i32) -> [u8; 4] {
    let at = (y as usize * WIDTH as usize + x as usize) * 4;
    rgba[at..at + 4].try_into().unwrap()
}

#[test]
fn minimap_owner_life_prefers_full_vip_runtime_hp() {
    let (_, mut data) = sample();
    let actor = &mut data.actors.0[0];
    assert!(actor_alive(actor));
    actor.resources = Some(d::Resources {
        revision: 1, equipment_revision: 1, life: 1,
        birth_hp: 100, birth_mp: 50,
        vitals: d::Vitals { hp: 0, maximum_hp: 100, mp: 5, maximum_mp: 50 },
        flows: d::Flows { hp_gain: 0, hp_loss: 0, restored: 0, damaged: 0,
            hp_granted: 0, mp_gain: 0, mp_loss: 0, spent: 0,
            mp_granted: 0, hp_clipped: 0, mp_clipped: 0 },
    });
    // A legacy-only positive Vital is not allowed to recolor a dead VIP owner.
    assert!(!actor_alive(actor));
    actor.resources.as_mut().unwrap().vitals.hp = 1;
    assert!(actor_alive(actor));
}

#[test]
fn minimap_uses_qualified_map_authoritative_position_and_exact_snapshot_not_camera() {
    let (map, mut data) = sample();
    let first = navigation(data.clone(), map.clone());
    let p = panel(&first, None);
    assert!(p.matches(first.observation()));
    assert_eq!(p.key.position, WorldCell { x: 32768, y: 32768 });
    assert_eq!(p.key.map, map.map_id());
    assert_eq!(p.key.digest, map.canonical_digest());
    assert_eq!(
        pixel(&p.rgba, CENTER.x, CENTER.y),
        [SELF[0], SELF[1], SELF[2], 255]
    );
    let unrelated = navigation(data.clone(), map.clone());
    assert!(
        !p.matches(unrelated.observation()),
        "equal DTOs are not an attachment proof"
    );
    data.world.revision += 1;
    data.world.loading_elapsed_nanos += 1;
    let clock = navigation(data.clone(), map.clone());
    let cached = panel(&clock, Some(&p));
    assert!(Arc::ptr_eq(&p.rgba, &cached.rgba));
    assert!(cached.matches(clock.observation()) && !cached.matches(first.observation()));
    data.actors.0[0].position.x += 1;
    let moved = navigation(data.clone(), map.clone());
    let fresh = panel(&moved, Some(&cached));
    assert_eq!(fresh.key.position.x, 32769);
    assert_ne!(fresh.rgba, cached.rgba);
    assert!(!Arc::ptr_eq(&fresh.rgba, &cached.rgba));
    // A teleport/distant authoritative sample replaces the origin directly.
    data.actors.0[0].position.x += 20;
    data.actors.0[0].position.y += 20;
    let teleported = navigation(data, map);
    let landed = panel(&teleported, Some(&fresh));
    assert_eq!(landed.key.position, WorldCell { x: 32789, y: 32788 });
    assert_ne!(landed.rgba, fresh.rgba);
    assert!(Panel::prepare(&teleported, Some(&landed), &AtomicBool::new(true)).is_err());
    assert_eq!(
        p.key.position.x, 32768,
        "old immutable pixels are unchanged"
    );
}

#[test]
fn minimap_cannot_reveal_other_actor_positions_or_invent_unknown_terrain() {
    let (map, mut data) = sample();
    let p = panel(&navigation(data.clone(), map.clone()), None);
    data.actors.0[1].position.x += 10;
    data.actors.0[1].heading = 0;
    let changed = panel(&navigation(data, map.clone()), Some(&p));
    assert!(Arc::ptr_eq(&p.rgba, &changed.rgba));
    assert_eq!(cell_color(&map, WorldCell { x: 32780, y: 32780 }), TERRAIN);
    assert_eq!(cell_color(&map, WorldCell { x: 32767, y: 32767 }), UNKNOWN);
    let missing = project(-10, -10).unwrap();
    assert_eq!(
        pixel(&p.rgba, missing.x, missing.y),
        [UNKNOWN[0], UNKNOWN[1], UNKNOWN[2], 255]
    );
    assert_eq!(p.rgba.len(), WIDTH as usize * HEIGHT as usize * 4);
    assert!(p.rgba.chunks_exact(4).all(|p| p[3] == 255));
}

#[test]
fn minimap_projection_and_all_eight_headings_share_world_orientation() {
    assert_eq!(
        project(1, 0),
        Some(PixelPoint {
            x: CENTER.x + 2,
            y: CENTER.y - 1
        })
    );
    assert_eq!(
        project(0, 1),
        Some(PixelPoint {
            x: CENTER.x + 2,
            y: CENTER.y + 1
        })
    );
    assert_eq!(project(i32::MIN, i32::MAX), None);
    assert!(project(RADIUS, RADIUS).is_some());
    assert!(project(RADIUS + 1, 0).is_none());
    let (map, mut data) = sample();
    let mut pictures = Vec::new();
    for (heading, (dx, dy)) in DIRECTIONS.into_iter().enumerate() {
        data.actors.0[0].heading = heading as i32;
        let p = panel(&navigation(data.clone(), map.clone()), None);
        let tip = project(dx * 3, dy * 3).unwrap();
        assert_eq!(
            pixel(&p.rgba, tip.x, tip.y),
            [SELF[0], SELF[1], SELF[2], 255]
        );
        assert!(pictures.iter().all(|old| old != &p.rgba));
        pictures.push(p.rgba.clone());
    }
}

fn surface(nav: Arc<Navigation>) -> Surface {
    let mut s = battle_window::tests::scene();
    s.observation = nav.observation().clone();
    s.geometry.revision = s.observation.data().world.revision as u64;
    s.minimap = Some(panel(&nav, None));
    s.navigation = Some(nav);
    let mut canvas = Canvas::new(800, 600).unwrap();
    s.minimap.as_ref().unwrap().draw(&mut canvas).unwrap();
    let mut bgra = vec![0; 800 * 600 * 4];
    canvas.copy_bgra(&mut bgra).unwrap();
    s.bgra = bgra.into();
    s
}

#[test]
fn minimap_overlay_blocks_clicks_and_names_at_translated_scaled_and_letterboxed_viewports() {
    let (map, data) = sample();
    let mut s = surface(navigation(data.clone(), map.clone()));
    let self_actor = &s.observation.data().actors.0[0];
    s.geometry
        .actors
        .push(crate::modern_lito::world_scene::ActorPlacement {
            entity_id: self_actor.entity as u64,
            cell: WorldCell {
                x: self_actor.position.x,
                y: self_actor.position.y,
            },
            anchor: PixelPoint {
                x: X + CENTER.x,
                y: Y + CENTER.y,
            },
            top_left: PixelPoint {
                x: X + CENTER.x - 10,
                y: Y + CENTER.y - 10,
            },
            width: 20,
            height: 20,
            visibility: None,
            graphics_id: 0,
            source_frame: 0,
            sprite_name: "synthetic-unit-only".into(),
            subframe: 0,
        });
    battle_window::validate(&s).unwrap();
    let old = s.minimap.clone();
    for view in [
        Viewport {
            x: 0,
            y: 0,
            width: 800,
            height: 600,
        },
        Viewport {
            x: 31,
            y: 72,
            width: 1200,
            height: 900,
        },
        Viewport {
            x: 12,
            y: 160,
            width: 400,
            height: 300,
        },
        Viewport {
            x: 200,
            y: 100,
            width: 1600,
            height: 1200,
        },
    ] {
        let screen = |x: i32, y: i32| PixelPoint {
            x: view.x + (x * view.width + 799) / 800,
            y: view.y + (y * view.height + 599) / 600,
        };
        for skill in [
            battle_window::Skill::Melee,
            battle_window::Skill::LightArrow,
            battle_window::Skill::Heal,
            battle_window::Skill::Disintegrate,
        ] {
            for (x, y) in [
                (X, Y),
                (X + WIDTH as i32 - 2, Y + HEIGHT as i32 - 2),
                (X + CENTER.x, Y + CENTER.y),
            ] {
                assert!(battle_window::hit(&s, view, screen(x, y), skill).is_none());
            }
        }
        for (x, y) in [
            (X - 4, Y),
            (X + WIDTH as i32 + 4, Y),
            (X, Y + HEIGHT as i32 + 4),
        ] {
            assert!(
                battle_window::hit(&s, view, screen(x, y), battle_window::Skill::Melee).is_some()
            );
        }
    }
    s.minimap = None;
    assert!(battle_window::hit(
        &s,
        Viewport {
            x: 0,
            y: 0,
            width: 800,
            height: 600
        },
        PixelPoint {
            x: X + CENTER.x,
            y: Y + CENTER.y
        },
        battle_window::Skill::Melee
    )
    .unwrap()
    .name
    .is_some());
    s.minimap = old;
    let fresh = navigation(data, map);
    s.observation = fresh.observation().clone();
    s.navigation = Some(fresh.clone());
    assert_eq!(
        battle_window::validate(&s).unwrap_err().to_string(),
        "canonical minimap belongs to other pixels",
        "only the minimap is stale; navigation already owns the fresh snapshot"
    );
    s.minimap = Some(panel(&fresh, s.minimap.as_deref()));
    battle_window::validate(&s).unwrap();
    s.navigation = None;
    assert_eq!(
        battle_window::validate(&s).unwrap_err().to_string(),
        "canonical minimap belongs to other pixels",
        "a minimap without qualified navigation must be rejected"
    );
}

/// Called by the actual two-controller integration clients for every published
/// Surface. Covers loaded/active/moved/teleported/dead/finished and all tested maps.
pub(crate) fn verify_surface(s: &Surface) {
    let p = s
        .minimap
        .as_ref()
        .expect("shared renderer must publish its locator");
    assert!(p.matches(&s.observation));
    let n = s.navigation.as_ref().unwrap();
    assert!(n.matches(&s.observation));
    assert_eq!(p.key.map, n.map().map_id());
    assert_eq!(p.key.digest, n.map().canonical_digest());
    let a = s
        .observation
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == s.observation.data().personal.entity)
        .unwrap();
    assert_eq!(
        p.key.position,
        WorldCell {
            x: a.position.x,
            y: a.position.y
        }
    );
    assert_eq!(p.key.heading as i32, a.heading);
    assert_eq!(
        p.key.alive,
        a.vital.as_ref().is_none_or(|v| v.values.hp > 0)
    );
    for (x, y) in [
        (0, 0),
        (WIDTH as i32 - 1, HEIGHT as i32 - 1),
        (CENTER.x, CENTER.y),
        (CENTER.x - 16, CENTER.y - 8),
    ] {
        let rgba = pixel(&p.rgba, x, y);
        let at = (((Y + y) as u32 * s.width + (X + x) as u32) * 4) as usize;
        assert_eq!(&s.bgra[at..at + 4], &[rgba[2], rgba[1], rgba[0], rgba[3]]);
    }
}
