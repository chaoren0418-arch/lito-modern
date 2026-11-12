use super::{shared_action_catalog::SharedActionCatalog, sprite_actions::ActionKey};
use std::{path::Path, sync::atomic::AtomicBool};

use super::{
    compositor::Canvas,
    native_actor_states::Style,
    native_combat::FrameVitals,
    native_combat_animation::{frame_sound, Playback},
    native_world::World,
    sprite_actions::{ActionLimits, FrameEvent, SpriteActionCatalog},
};

fn packet(key: &str) -> Vec<u8> {
    let fixture: serde_json::Value = serde_json::from_slice(include_bytes!(
        "../../tests/fixtures/native-world-current.json"
    ))
    .unwrap();
    let hex = fixture["packets"][key].as_str().unwrap();
    (0..hex.len())
        .step_by(2)
        .map(|index| u8::from_str_radix(&hex[index..index + 2], 16).unwrap())
        .collect()
}

fn world() -> World {
    let mut world = World::default();
    for key in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
        world.accept(&packet(key)).unwrap();
    }
    world
}

fn actor_packet(op: u8, id: u32, tail: &[u8]) -> Vec<u8> {
    let mut packet = vec![op];
    packet.extend(id.to_le_bytes());
    packet.extend(tail);
    packet
}

#[test]
fn effect_marker_selects_audio_and_hit_preserves_authoritative_hp() {
    assert_eq!(frame_sound(&[FrameEvent::Effect(119)], 0, 0), Some(119));
    assert_eq!(frame_sound(&[FrameEvent::Effect(0)], 0, 0), None);
    assert_eq!(frame_sound(&[FrameEvent::Hit], 0, 0), None);
    let weighted = [
        FrameEvent::Effect(119),
        FrameEvent::Effect(120),
        FrameEvent::Effect(119),
    ];
    assert_eq!(frame_sound(&weighted, 0, 1), Some(120));
    assert_eq!(frame_sound(&weighted, 1, 1), None);
    assert_eq!(frame_sound(&weighted, 2, 1), None);
    let catalog = SpriteActionCatalog::parse(
        b"#1 1 actor\n1.attack(1 3,0.0:1<119 0.1:1! 0.2:1)\n",
        ActionLimits::default(),
    )
    .unwrap();
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    world.actors.get_mut(&id).unwrap().graphics = 1;
    world.actors.get_mut(&id).unwrap().speed = 0;
    world.accept(&actor_packet(0x20, id, &[1])).unwrap();
    let authority = (world.hp, world.mp, world.actors[&id].hp_ratio);
    let mut playback = Playback::default();
    let (_, first) = playback.sample(&catalog, &world.actors[&id], 0).unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(first[0].sound, Some(119));
    let (_, next) = playback
        .sample(&catalog, &world.actors[&id], 50_000_000)
        .unwrap();
    assert_eq!(next.iter().filter(|event| event.event == "Hit").count(), 1);
    assert!(playback
        .sample(&catalog, &world.actors[&id], 50_000_000)
        .unwrap()
        .1
        .is_empty());
    assert_eq!((world.hp, world.mp, world.actors[&id].hp_ratio), authority);
}

#[test]
fn freeze_palette_uses_wire_flags_and_native_rgb555_pixels() {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    world.accept(&actor_packet(0x32, id, &[0, 1])).unwrap();
    let style = world.actors[&id].draw_state.style(true, 0);
    assert_eq!(style, Style::Dim);
    let mut canvas = Canvas::new(24, 24).unwrap();
    let pixels = [248, 128, 0, 255].repeat(24 * 24);
    let mut owners = vec![0; 24 * 24];
    canvas
        .blit_actor_state_fragment(&pixels, 0, 0, &mut owners, 1, style)
        .unwrap();
    assert_eq!(&canvas.rgba()[..4], &[148, 90, 24, 255]);
    assert!(owners.iter().all(|owner| *owner == 1));
    world.accept(&[0x25, 12]).unwrap();
    world.accept(&actor_packet(0x32, id, &[0, 0])).unwrap();
    assert_eq!(world.actors[&id].draw_state.style(true, 0), Style::Normal);
    assert!(world.movement_blocks.contains(&12));
    world.accept(&[0x25, 13]).unwrap();
    assert!(!world.movement_blocks.contains(&12));
}

#[test]
fn source169_receipt_survives_remove_and_same_id_replacement() {
    let mut world = world();
    let id = 2;
    let original = world.actors[&id].clone();
    world
        .accept(&actor_packet(0x37, id, &169u16.to_le_bytes()))
        .unwrap();
    world.accept(&actor_packet(0x15, id, &[])).unwrap();
    assert_eq!(world.effects.len(), 1);
    assert_eq!(world.effects[0].target, Some((id, original.incarnation)));
    assert_eq!(world.effects[0].position, [original.x, original.y]);
    world.accept(&packet("bot")).unwrap();
    assert_ne!(world.actors[&id].incarnation, original.incarnation);
    assert_eq!(world.effects.len(), 1);
}

#[test]
fn light_packet_order_and_vitals_are_bound_to_published_animation_frame() {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    world.accept(&actor_packet(0x34, id, &[1])).unwrap();
    let previous = FrameVitals::capture(&world);
    let prior_mp = world.mp;
    let mut revisions = Vec::new();
    for packet in [
        actor_packet(0x34, id, &[0]),
        actor_packet(0x20, id, &[19]),
        actor_packet(0x37, id, &2510u16.to_le_bytes()),
        vec![0x4a, 0, 119, 0],
        actor_packet(0x1b, id, &[13]),
        vec![0x4d, 7, 0, 100, 0],
    ] {
        assert!(world.accept(&packet).unwrap());
        revisions.push(world.revision);
    }
    assert!(revisions.windows(2).all(|pair| pair[1] == pair[0] + 1));
    assert!(!world.actors[&id].draw_state.invisible);
    assert_eq!(world.actors[&id].actions.last().unwrap().action, 19);
    assert!(world.actors[&id].action_sequence < world.effects[0].sequence);
    assert_eq!(world.actors[&id].light, 13);
    assert_eq!(world.mp, Some([7, 100]));
    assert_eq!(previous.mp, prior_mp);
    world.accept(&[0x0d, 1, 0, 100, 0]).unwrap();
    world.accept(&actor_packet(0x68, id, &[1])).unwrap();
    assert_ne!(previous.hp, world.hp);
    assert_ne!(
        previous.target_label(&world, id).unwrap(),
        super::native_combat::target_label(&world.actors[&id])
    );
    let published = FrameVitals::capture(&world);
    assert_eq!(published.hp, Some([1, 100]));
    assert_eq!(published.mp, Some([7, 100]));
    world.accept(&packet("actor")).unwrap();
    assert!(published.target_label(&world, id).is_none());
}

fn gate_catalog() -> SpriteActionCatalog {
    SpriteActionCatalog::parse(
        b"#1 1 actor\n0.walk(1 4,0.0:4 0.1:4 0.2:4 0.3:4) 1.attack(1 3,0.0:4 0.1:4 0.2:4) 2.damage(1 2,0.0:4 0.1:4) 3.breath(1 1,0.0:4)",
        ActionLimits::default(),
    )
    .unwrap()
}

fn gate_world() -> (World, u32) {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    let actor = world.actors.get_mut(&id).unwrap();
    actor.graphics = 1;
    actor.speed = 0;
    actor.brave = 0;
    actor.walk_action = 0;
    actor.motion_from = None;
    (world, id)
}

fn gate_frame(
    animator: &mut super::native_world_animation::Animator,
    gate: &mut super::native_combat::VitalsGate,
    catalog: &SpriteActionCatalog,
    world: &World,
    id: u32,
    now: u64,
) -> (super::native_world_animation::Pose, FrameVitals) {
    animator.begin(now, world.actors.keys().copied()).unwrap();
    let pose = animator.pose(catalog, &world.actors[&id], now).unwrap();
    let vitals = gate.present(world, |actor, barrier| animator.presented(actor, barrier));
    (pose, vitals)
}

/// Same batch: hurt action and HP arrive, then a move keeps the walk owning
/// the body. The HUD must keep the old HP until the hurt pose has started.
#[test]
fn hp_waits_for_action_held_behind_running_walk() {
    let catalog = gate_catalog();
    let (mut world, id) = gate_world();
    let mut animator = super::native_world_animation::Animator::default();
    let mut gate = super::native_combat::VitalsGate::default();
    let (_, initial) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, 0);
    let old_hp = world.hp;
    assert_eq!(initial.hp, old_hp);
    let maximum = old_hp.unwrap()[1];
    world.accept(&actor_packet(0x20, id, &[2])).unwrap();
    let mut hp = vec![0x0d, 1, 0];
    hp.extend(maximum.to_le_bytes());
    world.accept(&hp).unwrap();
    let ratio = if world.actors[&id].hp_ratio == 37 { 38 } else { 37 };
    world.accept(&actor_packet(0x68, id, &[ratio])).unwrap();
    // A later movement packet of the same batch restarts the running walk.
    let actor = world.actors.get_mut(&id).unwrap();
    actor.motion_from = Some([actor.x, actor.y]);
    actor.x += 1;
    world.visual_revision += 1;
    actor.motion_revision = world.visual_revision;
    assert_eq!(world.hp, Some([1, maximum]));
    let (walking, held) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, 10_000_000);
    assert_eq!(walking.phase, "walk");
    assert_eq!(held.hp, old_hp, "HP must not lead the held hurt action");
    assert_ne!(held.target_label(&world, id).unwrap(), super::native_combat::target_label(&world.actors[&id]));
    assert!(gate.held() >= 2);
    let walk = super::native_world_animation::resolve(&catalog, 1, 0).unwrap().duration_ns();
    let (hurt, committed) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, 10_000_000 + walk + 1_000_000);
    assert_eq!((hurt.action, hurt.phase), (2, "damage"));
    assert_eq!(committed.hp, Some([1, maximum]));
    assert_eq!(committed.target_label(&world, id).unwrap(), super::native_combat::target_label(&world.actors[&id]));
}

/// Same batch behind an unfinished action: the queued hurt and its HP/MP
/// commit together only when that hurt leaves the queue.
#[test]
fn hp_and_mp_wait_for_queued_action_behind_active_action() {
    let catalog = gate_catalog();
    let (mut world, id) = gate_world();
    let mut animator = super::native_world_animation::Animator::default();
    let mut gate = super::native_combat::VitalsGate::default();
    world.accept(&actor_packet(0x20, id, &[1])).unwrap();
    let (attack, _) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, 0);
    assert_eq!(attack.action, 1);
    let (old_hp, old_mp) = (world.hp, world.mp);
    let maximum = old_hp.unwrap()[1];
    world.accept(&actor_packet(0x20, id, &[2])).unwrap();
    let mut hp = vec![0x0d, 2, 0];
    hp.extend(maximum.to_le_bytes());
    world.accept(&hp).unwrap();
    world.accept(&[0x4d, 3, 0, 100, 0]).unwrap();
    let (still_attack, held) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, 10_000_000);
    assert_eq!(still_attack.action, 1);
    assert_eq!((held.hp, held.mp), (old_hp, old_mp));
    let attack_ns = super::native_world_animation::resolve(&catalog, 1, 1).unwrap().duration_ns();
    assert!(attack_ns > 10_000_000);
    let (hurt, committed) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, attack_ns + 1_000_000);
    assert_eq!(hurt.action, 2);
    assert_eq!((committed.hp, committed.mp), (Some([2, maximum]), Some([3, 100])));
    // No pending action: a later value commits on the next frame.
    let mut later = vec![0x0d, 5, 0];
    later.extend(maximum.to_le_bytes());
    world.accept(&later).unwrap();
    let (_, next) = gate_frame(&mut animator, &mut gate, &catalog, &world, id, attack_ns + 2_000_000);
    assert_eq!(next.hp, Some([5, maximum]));
}

#[test]
fn finite_cache_evicts_least_recent_unprotected_bank_only() {
    use super::native_skill_effects::finite_victim;
    let used: std::collections::BTreeMap<u16, u64> = [(10, 5), (11, 2), (12, 9)].into();
    let none = std::collections::BTreeSet::new();
    assert_eq!(finite_victim([10, 11, 12].into_iter(), &used, &none), Some(11));
    // A playing or not-yet-started cue protects its bank even when oldest.
    let protected: std::collections::BTreeSet<u16> = [11].into();
    assert_eq!(finite_victim([10, 11, 12].into_iter(), &used, &protected), Some(10));
    let all: std::collections::BTreeSet<u16> = [10, 11, 12].into();
    assert_eq!(finite_victim([10, 11, 12].into_iter(), &used, &all), None);
    // Unstamped entries are evicted first, ties broken by ID.
    assert_eq!(finite_victim([13, 10, 14].into_iter(), &used, &none), Some(13));
    assert!(super::native_skill_effects::FINITE_ERROR_LIMIT >= 128 + 64);
}

/// Review regression: unsupported S37 IDs must not consume the finite bank
/// capacity, so a later valid 7049 still loads and plays in the same Effects.
#[test]
#[ignore = "reads original SPR/WAV and renders offline pixels"]
fn original_unsupported_flood_does_not_block_valid_finite_effect() {
    use super::{
        native_skill_effects::Effects,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    let own = world.own_actor().unwrap();
    let projection = ViewProjection {
        anchor_cell: WorldCell { x: i32::from(own.x), y: i32::from(own.y) },
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    let mut effects = Effects::default();
    let mut canvas = Canvas::new(400, 400).unwrap();
    for gfx in 60_000u16..60_040 {
        world.accept(&actor_packet(0x37, id, &gfx.to_le_bytes())).unwrap();
    }
    effects.prepare(root, &world, &cancel);
    let (flood, _) = effects.draw(&world, &mut canvas, projection, 0);
    assert_eq!(flood["finite_banks"]["errors"].as_object().unwrap().len(), 40, "{flood}");
    assert_eq!(flood["omitted"], 40);
    world.accept(&actor_packet(0x37, id, &7049u16.to_le_bytes())).unwrap();
    effects.prepare(root, &world, &cancel);
    let (receipt, _) = effects.draw(&world, &mut canvas, projection, 1_000_000);
    assert_eq!(receipt["started"], 1, "{receipt}");
    assert_eq!(receipt["omitted"], 40, "{receipt}");
    let row = receipt["effects"].as_array().unwrap().iter()
        .find(|row| row["graphics"] == 7049).unwrap();
    assert_eq!(row["drawn"], true, "{receipt}");
    assert!(receipt["finite_banks"]["loaded"].as_array().unwrap().iter().any(|g| g == 7049));
}

#[test]
fn proc_receipts_keep_direction_and_bound_history_without_local_inference() {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    world.accept(&actor_packet(0x20, id, &[1])).unwrap();
    assert!(world.effects.is_empty());
    for index in 0..130 {
        let heading = (index % 8) as u8;
        world.accept(&actor_packet(0x1c, id, &[heading])).unwrap();
        world
            .accept(&actor_packet(0x37, id, &6983u16.to_le_bytes()))
            .unwrap();
        let sequence = world.effects.last().unwrap().sequence;
        assert_eq!(world.effect_headings[&sequence], heading);
    }
    assert_eq!(world.effects.len(), 128);
    assert_eq!(world.effect_headings.len(), 128);
    assert_eq!(world.effects_dropped, 2);
    let before = serde_json::to_value(&world).unwrap();
    let mut truncated = actor_packet(0x37, id, &7049u16.to_le_bytes());
    truncated.pop();
    assert!(world.accept(&truncated).is_err());
    assert_eq!(serde_json::to_value(&world).unwrap(), before);
    world.accept(&packet("world")).unwrap();
    assert!(world.effects.is_empty() && world.effect_headings.is_empty());
}

#[test]
fn proc_resource_qualification_rejects_unimplemented_controllers() {
    for definition in [
        "#900 1 proc\n0.effect(0 2,0.1:1 0.0:1) 102.type(1)",
        "#900 1 proc\n0.effect(0 2,0.1:1 0.0:1) 105.clothes(1 901)",
        "#900 1 proc\n0.effect(0 2,0.1:1]901 0.0:1)",
    ] {
        let catalog =
            SpriteActionCatalog::parse(definition.as_bytes(), ActionLimits::default()).unwrap();
        assert!(super::native_skill_effects::qualify_finite(&catalog, 900).is_err());
    }
    let catalog = SpriteActionCatalog::parse(
        b"#900 1 proc\n0.effect(0 2,0.1:1<119 0.0:1!) 102.type(0) 104.attr(8)",
        ActionLimits::default(),
    )
    .unwrap();
    assert_eq!(
        super::native_skill_effects::qualify_finite(&catalog, 900)
            .unwrap()
            .len(),
        1
    );
}

#[test]
#[ignore = "reads the local original client resources"]
fn original_t047_resource_definitions() {
    let catalog = SharedActionCatalog::load(
        Path::new(r"D:\lineage開發\0-客戶端"),
        &AtomicBool::new(false),
    )
    .unwrap();
    for gfx in [169, 6983, 7049, 6706, 6712, 6718, 6724] {
        let row = &catalog.catalog().sprites[&gfx];
        assert!(row.header_issue.is_none());
        if super::native_cube::is_graphics(gfx) {
            super::native_cube::qualify(catalog.catalog(), gfx).unwrap();
        } else {
            let animation = catalog
                .catalog()
                .resolve_action(gfx, ActionKey::new(0))
                .unwrap()
                .animation;
            assert!(!animation.frames.is_empty());
            if gfx == 169 {
                super::native_skill_effects::qualify_travel(catalog.catalog()).unwrap();
            } else {
                super::native_skill_effects::qualify_finite(catalog.catalog(), gfx).unwrap();
            }
        }
    }
}

fn save_canvas(canvas: &Canvas, name: &str) {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/issue-184");
    std::fs::create_dir_all(&directory).unwrap();
    let file = std::fs::File::create(directory.join(name)).unwrap();
    let mut encoder = png::Encoder::new(file, canvas.width(), canvas.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .unwrap()
        .write_image_data(canvas.rgba())
        .unwrap();
}

#[test]
#[ignore = "reads original SPR/WAV and renders offline pixels"]
fn original_kiringku_eight_directions_and_source169_pixels() {
    use super::{
        native_skill_effects::Effects,
        world_map::{PixelPoint, ViewProjection, WorldCell},
    };
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    let position = world.own_actor().unwrap();
    let projection = ViewProjection {
        anchor_cell: WorldCell {
            x: i32::from(position.x),
            y: i32::from(position.y),
        },
        anchor_pixel: PixelPoint { x: 200, y: 200 },
    };
    for gfx in [7049u16] {
        let mut signatures = std::collections::BTreeSet::new();
        for heading in 0..8 {
            world.effects.clear();
            world.effect_headings.clear();
            world.accept(&actor_packet(0x1c, id, &[heading])).unwrap();
            world
                .accept(&actor_packet(0x37, id, &gfx.to_le_bytes()))
                .unwrap();
            let mut effects = Effects::default();
            effects.prepare(root, &world, &cancel);
            // A later turn must not change the direction of an accepted effect.
            world
                .accept(&actor_packet(0x1c, id, &[(heading + 1) % 8]))
                .unwrap();
            let mut canvas = Canvas::new(400, 400).unwrap();
            let (receipt, _) = effects.draw(&world, &mut canvas, projection, 0);
            assert_eq!(receipt["omitted"], 0, "{receipt}");
            assert_eq!(receipt["effects"][0]["drawn"], true, "{receipt}");
            assert_eq!(
                receipt["effects"][0]["sprite"],
                format!("{gfx}-{heading}.spr")
            );
            assert!(canvas
                .rgba()
                .chunks_exact(4)
                .any(|pixel| pixel[..3] != [0, 0, 0]));
            signatures.insert(canvas.rgba().to_vec());
            if heading == 0 {
                save_canvas(&canvas, &format!("kiringku-{gfx}.png"));
            }
            let (_, duplicate) = effects.draw(&world, &mut canvas, projection, 0);
            assert!(duplicate.is_empty());
            let authority = (world.hp, world.mp);
            for millis in (50..=2000).step_by(50) {
                effects.draw(&world, &mut canvas, projection, millis * 1_000_000);
            }
            assert_eq!((world.hp, world.mp), authority);
        }
        assert!(signatures.len() > 1, "directional SPR must change pixels");
    }
    world.effects.clear();
    world
        .accept(&actor_packet(0x37, id, &6983u16.to_le_bytes()))
        .unwrap();
    let mut unsupported = Effects::default();
    unsupported.prepare(root, &world, &cancel);
    let mut canvas = Canvas::new(400, 400).unwrap();
    let (receipt, _) = unsupported.draw(&world, &mut canvas, projection, 0);
    assert_eq!(receipt["started"], 0);
    assert_eq!(receipt["omitted"], 1);
    let error = receipt["finite_banks"]["errors"]["6983"].as_str().unwrap();
    assert!(
        error.contains("6983-0.spr")
            && error.contains("subframe 9")
            && error.contains("count is 9"),
        "{error}"
    );
    world.effects.clear();
    world
        .accept(&actor_packet(0x37, 2, &169u16.to_le_bytes()))
        .unwrap();
    world.accept(&actor_packet(0x15, 2, &[])).unwrap();
    let mut effects = Effects::travel();
    effects.prepare(root, &world, &cancel);
    let mut canvas = Canvas::new(400, 400).unwrap();
    let (receipt, events) = effects.draw(&world, &mut canvas, projection, 0);
    assert_eq!(receipt["effects"][0]["drawn"], true, "{receipt}");
    assert!(events
        .iter()
        .any(|event| event.sound == Some(119) && event.actor == 0));
    save_canvas(&canvas, "source169-after-remove.png");
    let mut last = receipt;
    for millis in (50..=1000).step_by(50) {
        last = effects
            .draw(&world, &mut canvas, projection, millis * 1_000_000)
            .0;
    }
    assert_eq!(last["active"], 0);
    assert_eq!(last["started"], 1);
}

#[test]
#[ignore = "renders original map, cube layers, and attack frames offline"]
fn original_cube_layers_and_attack_frames_reach_world_renderer() {
    use super::native_world_scene::Model;
    let root = Path::new(r"D:\lineage開發\0-客戶端");
    let cancel = AtomicBool::new(false);
    let mut world = world();
    let own = world.own_actor().unwrap().clone();
    world.actors.get_mut(&own.id).unwrap().speed = 0;
    for (index, gfx) in [6706u16, 6712, 6718, 6724].into_iter().enumerate() {
        let mut cube = packet("bot");
        cube[1..3].copy_from_slice(&(own.x + index as u16 + 1).to_le_bytes());
        cube[3..5].copy_from_slice(&(own.y + 1).to_le_bytes());
        cube[5..9].copy_from_slice(&(100 + index as u32).to_le_bytes());
        cube[9..11].copy_from_slice(&gfx.to_le_bytes());
        world.accept(&cube).unwrap();
    }
    let mut model = Model::load(root, &world, &cancel).unwrap();
    let initial = model.draw_at(&world, &cancel, 0).unwrap();
    for id in 100..104 {
        let row = initial.receipt["actors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == id)
            .unwrap();
        assert_eq!(row["rendered"], true, "{row}");
        assert_eq!(row["phase"], "cube", "{row}");
        assert!(row["layer_error"].is_null(), "{row}");
        assert_eq!(row["layers"].as_array().unwrap().len(), 3, "{row}");
    }
    save_canvas(&initial.canvas, "cubes-initial.png");
    let mut attack = vec![0x23, 1];
    attack.extend(own.id.to_le_bytes());
    attack.extend(2u32.to_le_bytes());
    attack.extend([1, own.heading, 0, 0, 0, 0]);
    world.accept(&attack).unwrap();
    let start = model.draw_at(&world, &cancel, 100_000_000).unwrap();
    world.accept(&[0x0d, 1, 0, 200, 0]).unwrap();
    assert_eq!(start.vitals.hp, Some([100, 200]));
    let later = model.draw_at(&world, &cancel, 500_000_000).unwrap();
    assert_eq!(later.vitals.hp, Some([1, 200]));
    let actor_row = |frame: &super::native_world_scene::Frame| {
        frame.receipt["actors"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["id"] == own.id)
            .unwrap()
            .clone()
    };
    assert_ne!(
        actor_row(&start)["sprite_subframe"],
        actor_row(&later)["sprite_subframe"]
    );
    assert_ne!(start.canvas.rgba(), later.canvas.rgba());
    save_canvas(&start.canvas, "attack-start.png");
    save_canvas(&later.canvas, "attack-later.png");
    let after_loop = model.draw_at(&world, &cancel, 6_000_000_000).unwrap();
    assert!(after_loop.receipt["actors"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|row| row["phase"] == "cube")
        .all(|row| row["entered_events"]
            .as_array()
            .unwrap()
            .iter()
            .any(|event| event["event"] == "Hit")));
}
