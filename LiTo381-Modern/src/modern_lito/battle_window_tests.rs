use super::*;
use crate::modern_lito::{battle_payload::data_tests, battle_scene::tests::observation};
use std::sync::Arc;

#[test]
fn canonical_result_notice_does_not_infer_verdict_before_authoritative_finish() {
    let observation = observation(data_tests::fixture());
    assert!(result_notice(&observation).is_none());
}

pub(crate) fn scene() -> Surface {
    use crate::modern_lito::{world_map::ViewProjection, world_scene as s};
    let o = observation(data_tests::fixture());
    Surface {
        geometry: s::RenderInfo {
            world_id: o.data().world.identity.match_id as u64,
            revision: o.data().world.revision as u64,
            projection: ViewProjection {
                anchor_cell: WorldCell { x: 32768, y: 32768 },
                anchor_pixel: PixelPoint { x: 400, y: 300 },
            },
            visible_tiles: 0,
            actors: vec![],
            objectives: vec![],
            npcs: vec![],
        },
        observation: o,
        width: 800,
        height: 600,
        bgra: Arc::from(vec![0; 800 * 600 * 4]),
        objectives: vec![],
        catalog_sha256: [0; 32],
        navigation: None,
        minimap: None,
        audio: None,
        effects_skipped: false,
        limitations: s::SceneLimitations {
            opacity_records: 0,
            sprite_effects: 0,
            portals: 0,
            actor_object_occlusion_unimplemented: true,
            actor_dependent_roofs_unimplemented: true,
            native_roof_timers_and_aliases_unverified: true,
            native_actor_update_order_unverified: true,
            weapon_and_transformation_art_unimplemented: true,
            native_actor_foot_pivot_unverified: true,
            unplayed_sound_ids: vec![],
            unrendered_base_effect_ids: vec![],
            unrendered_base_effects: vec![],
        },
    }
}
fn placement(entity: u64, x: i32, order: i32) -> crate::modern_lito::world_scene::ActorPlacement {
    crate::modern_lito::world_scene::ActorPlacement {
        entity_id: entity,
        cell: WorldCell { x, y: 32768 },
        anchor: PixelPoint { x: 400, y: order },
        top_left: PixelPoint { x: 390, y: 290 },
        width: 20,
        height: 20,
        visibility: None,
        graphics_id: 0,
        source_frame: 0,
        sprite_name: "synthetic-unit-only".into(),
        subframe: 0,
    }
}
const VIEW: Viewport = Viewport {
    x: 0,
    y: 0,
    width: 800,
    height: 600,
};

#[test]
fn non_target_actions_keep_the_painted_life_and_exact_catalog_intent() {
    use crate::modern_lito::{ammo_protocol, item_protocol, world_window::Action as A};
    let o = observation(data_tests::fixture());
    let mut actions: Vec<_> = (0..8).map(A::Step).collect();
    actions.extend([A::Teleport, A::FireStorm]);
    actions.extend(item_protocol::UI_ITEMS.map(A::UseItem));
    actions.extend(ammo_protocol::UI_ITEMS.map(A::SelectAmmo));
    for a in actions {
        let A::Canonical(intent) = stamp_local_action(a, Some(&o)) else {
            panic!("lost owner");
        };
        assert_eq!(intent.stamp, Stamp::of(&o));
        let Kind::Local(local) = intent.kind else {
            panic!("changed intent kind");
        };
        assert!(local.valid());
        assert_eq!(local.action(), a);
        assert_eq!(
            stamp_local_action(a, None),
            a,
            "legacy capture does not invent a canonical owner"
        );
    }
    for a in [
        A::Leave,
        A::CancelRoute,
        A::Canonical(Intent {
            stamp: Stamp::of(&o),
            kind: Kind::Cancel,
        }),
    ] {
        assert_eq!(stamp_local_action(a, Some(&o)), a);
    }
    for local in [
        Local::Step(8),
        Local::UseItem(-1),
        Local::UseItem(3333),
        Local::SelectAmmo(40010),
    ] {
        assert!(!local.valid());
    }
}

#[test]
fn stamp_binds_source_character_epoch_channel_world_and_keeps_engine_revision() {
    let d = data_tests::fixture();
    let o = observation(d.clone());
    let stamp = Stamp::of(&o);
    assert_eq!(stamp.revision, d.world.revision);
    assert!(stamp.current(&o));
    for field in 0..9 {
        let mut changed = stamp;
        match field {
            0 => changed.context.key.source_id += 1,
            1 => changed.context.key.object_id += 1,
            2 => changed.context.epoch += 1,
            3 => changed.channel ^= 1,
            4 => changed.world[0] += 1,
            5 => changed.world[1] += 1,
            6 => changed.world[2] += 1,
            7 => changed.revision += 1,
            _ => changed.life += 1,
        }
        assert!(!changed.current(&o));
    }
    assert!(!format!("{stamp:?}").contains("3333"));
}

#[test]
fn canonical_effect_and_art_limitations_are_visible_without_inventing_a_failure() {
    let mut surface = scene();
    surface.limitations.actor_object_occlusion_unimplemented = false;
    surface.limitations.actor_dependent_roofs_unimplemented = false;
    surface
        .limitations
        .weapon_and_transformation_art_unimplemented = false;
    assert!(
        presentation_notice(&surface).is_none(),
        "unverified native equivalence is not a rendering failure"
    );
    surface.effects_skipped = true;
    assert!(presentation_notice(&surface).unwrap().contains("MP"));
    surface.effects_skipped = false;
    surface
        .limitations
        .weapon_and_transformation_art_unimplemented = true;
    assert!(presentation_notice(&surface).unwrap().contains("外觀"));
    assert!(skill_status(&surface.observation, Skill::Disintegrate).is_none());
    assert!(skill_status(&surface.observation, Skill::LightArrow).is_none());
    assert!(skill_status(&surface.observation, Skill::Heal).is_none());
    assert!(skill_status(&surface.observation, Skill::Melee).is_none());
}

#[test]
fn viewport_conversion_rejects_letterbox_and_preserves_negative_coordinates() {
    let view = Viewport {
        x: 100,
        y: 50,
        width: 1600,
        height: 1200,
    };
    assert_eq!(
        pixel(view, PixelPoint { x: 100, y: 50 }),
        Some(PixelPoint { x: 0, y: 0 })
    );
    assert_eq!(
        pixel(view, PixelPoint { x: 1699, y: 1249 }),
        Some(PixelPoint { x: 799, y: 599 })
    );
    for p in [
        PixelPoint { x: 99, y: 50 },
        PixelPoint { x: 100, y: 49 },
        PixelPoint { x: 1700, y: 51 },
        PixelPoint { x: 101, y: 1250 },
        PixelPoint { x: -1, y: -1 },
    ] {
        assert!(pixel(view, p).is_none());
    }
    assert!(pixel(Viewport { width: 0, ..view }, PixelPoint { x: 100, y: 50 }).is_none());
}

#[test]
fn paint_qualification_rejects_duplicate_foreign_moved_and_unmapped_hitboxes() {
    let mut s = scene();
    s.geometry.actors.push(placement(1, 32768, 300));
    validate(&s).unwrap();
    s.geometry.actors.push(s.geometry.actors[0].clone());
    assert!(validate(&s).is_err());
    s.geometry.actors.pop();
    s.geometry.actors[0].entity_id = 10;
    assert!(validate(&s).is_err());
    s.geometry.actors[0] = placement(1, 32769, 300);
    assert!(validate(&s).is_err());
    s.geometry.actors.clear();
    s.geometry.objectives.push(placement(3, 32768, 300));
    assert!(validate(&s).is_err());
    s.objectives.push((
        ObjectiveTarget::Honor { entity: 3, team: 0 },
        s.geometry.objectives[0].clone(),
    ));
    assert!(validate(&s).is_err());
    s.geometry.objectives.clear();
    s.objectives.clear();
    s.geometry.revision += 1;
    assert!(validate(&s).is_err());
    s.geometry.revision -= 1;
    s.bgra = Arc::from(vec![0; 4]);
    assert!(validate(&s).is_err());
}

#[test]
fn hovered_names_use_global_member_source_and_painter_order_not_object_id() {
    let mut s = scene();
    let mut data = s.observation.data().clone();
    data.room.members.0[1].name.0 = "相同角色編號".into();
    data.room.members.0[1].source_name.0 = "另一伺服器".into();
    s.observation = observation(data);
    s.geometry.actors = vec![placement(2, 32770, 301), placement(1, 32768, 300)];
    validate(&s).unwrap();
    let point = PixelPoint { x: 400, y: 300 };
    let top = hit(&s, VIEW, point, Skill::Melee).unwrap();
    assert_eq!(top.name.unwrap().0, "相同角色編號(另一伺服器)");
    assert_eq!(
        top.intent.kind,
        Kind::Cancel,
        "scene-only actor is not a combat target"
    );
    s.geometry.actors[1].anchor.y = 302;
    assert_eq!(
        hit(&s, VIEW, point, Skill::Melee).unwrap().name.unwrap().0,
        "角色(太陽神)"
    );
    assert!(hit(&s, VIEW, PixelPoint { x: 420, y: 310 }, Skill::Melee)
        .unwrap()
        .name
        .is_none());
}

#[test]
fn selected_skill_never_falls_through_to_movement_or_melee_on_empty_ground() {
    let s = scene();
    let p = PixelPoint { x: 400, y: 300 };
    assert_eq!(
        hit(&s, VIEW, p, Skill::Melee).unwrap().intent.kind,
        Kind::Navigate(WorldCell { x: 32768, y: 32768 })
    );
    for skill in [Skill::LightArrow, Skill::Heal, Skill::Disintegrate] {
        assert_eq!(hit(&s, VIEW, p, skill).unwrap().intent.kind, Kind::Cancel);
    }
}

#[test]
fn absent_canonical_private_data_is_not_empty_inventory_or_zero_live_stats() {
    use crate::modern_lito::{battle_details, world_window::DetailsPage};
    let s = scene();
    let stats =
        battle_details::text(&s.observation, DetailsPage::Character, str::to_owned).unwrap();
    assert!(stats.contains("未提供來源保存資料"));
    assert!(stats.contains("未提供戰鬥 HP／MP"));
    let bag = battle_details::text(&s.observation, DetailsPage::Inventory, str::to_owned).unwrap();
    assert!(bag.contains("不是空背包"));
    assert!(!bag.contains("共 0 筆"));
    assert_eq!(health(&s.observation), "本場未提供戰鬥 HP／MP");
    assert_eq!(weight(&s.observation), "本場未提供比賽負重。");
}

#[test]
fn focused_inventory_does_not_invent_rows_or_hide_missing_source_data() {
    use crate::modern_lito::{
        battle_details,
        battle_inventory::{Key, Selection},
    };
    let s = scene();
    let text = battle_details::inventory(&s.observation, None, str::to_owned).unwrap();
    assert!(text.contains("背包總覽"));
    assert!(text.contains("不是空背包"));
    assert!(!text.contains("實例 "));
    for key in [Key::Source(1), Key::Supply(40010), Key::Ammo(40744)] {
        let focus = Selection {
            stamp: Stamp::of(&s.observation),
            key,
        };
        assert!(battle_details::inventory(&s.observation, Some(focus), str::to_owned).is_none());
    }
}

#[test]
fn loading_clock_only_does_not_starve_pixels_but_never_admits_changed_authority() {
    use crate::modern_lito::battle_player::same_loading_picture;
    let data = data_tests::fixture();
    let old = observation(data.clone());
    let mut next = data.clone();
    next.world.loading_elapsed_nanos = 100;
    let advanced = observation(next.clone());
    assert!(same_loading_picture(&old, &advanced));
    assert!(
        !same_loading_picture(&advanced, &old),
        "time must not regress"
    );
    for field in 0..6 {
        let mut changed = next.clone();
        match field {
            0 => changed.manifest.digest[0] ^= 1,
            1 => changed.actors.0[0].position.x += 1,
            2 => changed.world.revision += 1,
            3 => changed.personal.player.epoch += 1,
            4 => changed.room.members.0[0].source_name.0.push('新'),
            _ => changed.actors.0[1].loaded = true,
        }
        assert!(!same_loading_picture(&old, &observation(changed)));
    }
}
