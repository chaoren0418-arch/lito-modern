//! T047 second half (#185): static board, S_POLYMORPH receipt and restore,
//! invisibility recast, miss/hit markers, chain-sword -> spear fallback.
use super::{
    native_world::World,
    native_world_animation::{resolve, weapon_family_action, Animator},
    sprite_actions::{ActionLimits, SpriteActionCatalog},
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

fn polymorph(id: u32, gfx: u16, walk: u8) -> Vec<u8> {
    let mut tail = gfx.to_le_bytes().to_vec();
    tail.push(walk);
    actor_packet(0x27, id, &tail)
}

fn neutral(world: &mut World, id: u32, gfx: u16, walk: u8) {
    let actor = world.actors.get_mut(&id).unwrap();
    actor.graphics = gfx;
    actor.walk_action = walk;
    actor.speed = 0;
    actor.brave = 0;
    actor.motion_from = None;
}

/// NATIVE-WORLD:24/:34: list.spr `#1544 1 board a` has no action0. The first
/// frame must still present it as a static prop instead of dropping it.
#[test]
fn board_without_action0_is_presented_as_static_prop() {
    let catalog = SpriteActionCatalog::parse(
        b"#1544 1 board a\n102.type(14)\n",
        ActionLimits::default(),
    )
    .unwrap();
    assert!(resolve(&catalog, 1544, 0).is_err(), "catalog really lacks action0");
    let mut world = world();
    neutral(&mut world, 2, 1544, 0);
    let mut animator = Animator::default();
    animator.begin(0, world.actors.keys().copied()).unwrap();
    let pose = animator.pose(&catalog, &world.actors[&2], 0).unwrap();
    assert!(pose.static_prop);
    assert_eq!((pose.phase, pose.graphics, pose.image, pose.subframe), ("static_prop", 1544, 0, 0));
    // A board that is told to walk is not silently accepted as a prop.
    world.actors.get_mut(&2).unwrap().walk_action = 4;
    assert!(animator.pose(&catalog, &world.actors[&2], 1).is_err());
    // A missing action on a non-board sprite is not routed through the prop path.
    let animated = SpriteActionCatalog::parse(b"#1545 2 npc\n102.type(14)\n", ActionLimits::default()).unwrap();
    neutral(&mut world, 2, 1545, 0);
    assert!(animator.pose(&animated, &world.actors[&2], 2).is_err());
}

fn family_catalog(chain: bool) -> SpriteActionCatalog {
    let mut text = String::from(
        "#1 1 actor\n24.walkspear(1 4,0.0:4 0.1:4 0.2:4 0.3:4) 25.attackspear(1 3,0.0:4 0.1:4 0.2:4) 27.breathspear(1 2,0.0:4 0.1:4)",
    );
    if chain {
        text.push_str(" 83.walkchain(1 4,1.0:4 1.1:4 1.2:4 1.3:4) 84.attackchain(1 2,1.0:4 1.1:4) 86.breathchain(1 1,1.0:4)");
    }
    text.push('\n');
    SpriteActionCatalog::parse(text.as_bytes(), ActionLimits::default()).unwrap()
}

/// Server `resolveEquippedWalkAction`: chain-sword 83 falls back to spear 24
/// when the active sprite has no 83. The client must draw the spear family
/// (idle 27, attack 25) rather than failing to render the actor.
#[test]
fn chain_sword_family_falls_back_to_spear24_only_when_83_absent() {
    let spear_only = family_catalog(false);
    let chain = family_catalog(true);
    for (action, spear) in [(83, 24), (84, 25), (86, 27)] {
        assert_eq!(weapon_family_action(&spear_only, 1, action), spear);
        assert_eq!(weapon_family_action(&chain, 1, action), action);
    }
    for other in [0u8, 4, 20, 24, 25, 27, 46] {
        assert_eq!(weapon_family_action(&spear_only, 1, other), other);
    }
    // Neither family present: keep 83 so the absence is still reported.
    let empty = SpriteActionCatalog::parse(b"#1 1 actor\n0.walk(1 1,0.0:4)\n", ActionLimits::default()).unwrap();
    assert_eq!(weapon_family_action(&empty, 1, 83), 83);

    let mut world = world();
    let id = world.own_actor().unwrap().id;
    neutral(&mut world, id, 1, 83);
    let mut animator = Animator::default();
    animator.begin(0, world.actors.keys().copied()).unwrap();
    let idle = animator.pose(&spear_only, &world.actors[&id], 0).unwrap();
    assert_eq!((idle.action, idle.phase), (27, "idle"));
    // Attack frames: S_ACTION 84 plays the spear attack images, frame by frame.
    world.accept(&actor_packet(0x20, id, &[84])).unwrap();
    let per = resolve(&spear_only, 1, 25).unwrap().duration_ns() / 3;
    let start = 1_000_000;
    let first = animator.pose(&spear_only, &world.actors[&id], start).unwrap();
    assert_eq!((first.action, first.source_frame), (25, 0));
    let mut images = Vec::new();
    for step in 0..3u64 {
        let pose = animator.pose(&spear_only, &world.actors[&id], start + step * per + per / 2).unwrap();
        assert_eq!(pose.action, 25, "{step}");
        images.push(pose.source_frame);
    }
    assert_eq!(images, [0, 1, 2], "attack must advance through its frames, not only turn");

    let mut world = self::world();
    neutral(&mut world, id, 1, 83);
    let mut animator = Animator::default();
    animator.begin(0, world.actors.keys().copied()).unwrap();
    let idle = animator.pose(&chain, &world.actors[&id], 0).unwrap();
    assert_eq!((idle.action, idle.phase), (86, "idle"));
}

/// S_POLYMORPH 0x27 (`dhc`) morphs and later restores the actor. GM .polyme
/// lasts 7200 s server-side; the restore is the server's own S_POLYMORPH,
/// so local time alone never changes the appearance.
#[test]
fn polymorph_receipt_applies_and_server_restore_returns_base_body() {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    let base = (world.actors[&id].graphics, world.actors[&id].walk_action);
    let before = world.visual_revision;
    assert!(world.accept(&polymorph(id, 6080, 4)).unwrap());
    assert_eq!((world.actors[&id].graphics, world.actors[&id].walk_action), (6080, 4));
    assert!(world.visual_revision > before);
    assert_eq!(world.polymorph_packets, 1);
    assert!(!world.unhandled_opcodes.contains(&0x27));
    // Duplicate receipt is counted but is not a visual change.
    let revision = world.visual_revision;
    world.accept(&polymorph(id, 6080, 4)).unwrap();
    assert_eq!((world.visual_revision, world.polymorph_packets), (revision, 2));
    // 7200 s of local timers do not restore the body without the server packet.
    world.poll_status_icons(7_200_000 + 1);
    assert_eq!(world.actors[&id].graphics, 6080);
    world.accept(&polymorph(id, base.0, base.1)).unwrap();
    assert_eq!((world.actors[&id].graphics, world.actors[&id].walk_action), base);
    // Royal base gfx 0 is a valid restore target.
    world.accept(&polymorph(2, 0, 0)).unwrap();
    assert_eq!(world.actors[&2].graphics, 0);
    // Orphan, zero id, truncated and trailing packets.
    let orphans = world.orphan_updates;
    world.accept(&polymorph(0x7777_0000, 61, 0)).unwrap();
    assert_eq!(world.orphan_updates, orphans + 1);
    assert!(world.accept(&polymorph(0, 61, 0)).is_err());
    let mut truncated = polymorph(id, 61, 0);
    truncated.pop();
    assert!(world.accept(&truncated).is_err());
    let mut trailing = polymorph(id, 61, 0);
    trailing.push(0);
    assert!(world.accept(&trailing).is_err());
    assert_eq!(world.actors[&id].graphics, base.0);
}

/// Server recast (switching invisibility skill): own client receives
/// S_INVISIBLE 0, S_ACTION, S_INVISIBLE 1; peers receive the rebuilt actor,
/// the action and S_REMOVE_OBJECT. After the batch the hider stays hidden and
/// the own actor keeps its selection token.
#[test]
fn invisibility_recast_does_not_leave_hider_visible() {
    let mut world = world();
    let id = world.own_actor().unwrap().id;
    world.accept(&actor_packet(0x34, id, &[1])).unwrap();
    let token = world.actors[&id].target_revision;
    for p in [actor_packet(0x34, id, &[0]), actor_packet(0x20, id, &[19]), actor_packet(0x34, id, &[1])] {
        world.accept(&p).unwrap();
    }
    assert!(world.actors[&id].draw_state.invisible);
    assert_eq!(world.actors[&id].target_revision, token);
    assert_eq!(world.actors[&id].actions.last().unwrap().action, 19);
    assert_eq!(world.actors[&id].draw_state.style(true, 0), super::native_actor_states::Style::Half);
    // Peer view of a remote recast.
    let remote = 2;
    world.accept(&actor_packet(0x15, remote, &[])).unwrap();
    world.accept(&packet("bot")).unwrap();
    world.accept(&actor_packet(0x20, remote, &[19])).unwrap();
    world.accept(&actor_packet(0x15, remote, &[])).unwrap();
    assert!(!world.actors.contains_key(&remote));
    // A remote S_INVISIBLE 1 retires selection tokens exactly once.
    world.accept(&packet("bot")).unwrap();
    let first = world.actors[&remote].target_revision;
    world.accept(&actor_packet(0x34, remote, &[1])).unwrap();
    let hidden = world.actors[&remote].target_revision;
    assert_ne!(hidden, first);
    world.accept(&actor_packet(0x34, remote, &[1])).unwrap();
    assert_eq!(world.actors[&remote].target_revision, hidden);
    assert_eq!(world.actors[&remote].draw_state.style(false, 0), super::native_actor_states::Style::Hidden);
}

/// S_ATTACK hit marker: 0 is the miss animation (attacker swings, target is
/// not hurt); a positive marker queues the target's damage action. The wire
/// has no damage value or critical flag for physical attacks.
#[test]
fn attack_marker_zero_is_miss_and_positive_marker_hurts_target() {
    let mut world = world();
    let own = world.own_actor().unwrap().id;
    let attack = |marker: u8| {
        let mut b = vec![0x23, 1];
        b.extend(own.to_le_bytes());
        b.extend(2u32.to_le_bytes());
        b.extend([marker, 0, 0, 0, 0, 0]);
        b
    };
    let target_actions = world.actors[&2].actions.len();
    world.accept(&attack(0)).unwrap();
    assert_eq!(world.actors[&own].actions.last().unwrap().action, 1);
    assert_eq!(world.actors[&2].actions.len(), target_actions, "miss must not hurt");
    world.accept(&attack(10)).unwrap();
    assert_eq!(world.actors[&2].actions.last().unwrap().action, 2);
}

/// Real list.spr: every gfx of the map564 first frame resolves a pose, and
/// the chain-sword fallback holds for every base class sprite lacking 83.
#[test]
#[ignore = "reads the local original client resources"]
fn original_first_frame_twelve_objects_and_spear_fallback() {
    use super::shared_action_catalog::SharedActionCatalog;
    let root = std::path::Path::new(r"D:\lineage開發\0-客戶端");
    let shared = SharedActionCatalog::load(root, &std::sync::atomic::AtomicBool::new(false)).unwrap();
    let catalog = shared.catalog();
    let mut world = world();
    let gfx = [(61u16, 0u8), (143, 0), (2083, 29), (834, 0), (460, 0), (936, 0), (148, 0), (0, 0), (1544, 0), (237, 0), (727, 0), (98, 0)];
    let mut animator = Animator::default();
    let mut drawn = 0;
    for (index, (graphics, walk)) in gfx.into_iter().enumerate() {
        neutral(&mut world, 2, graphics, walk);
        animator.begin(index as u64, [2].into_iter()).unwrap();
        let pose = animator.pose(catalog, &world.actors[&2], index as u64).unwrap_or_else(|e| panic!("gfx {graphics}: {e:#}"));
        assert_eq!(pose.static_prop, graphics == 1544, "gfx {graphics}");
        drawn += 1;
    }
    assert_eq!(drawn, 12);
    for graphics in [0u32, 1, 37, 48, 61, 138, 734, 1186, 2786, 2796, 3225] {
        let Ok(_) = resolve(catalog, graphics, 24) else { continue };
        let fallback = weapon_family_action(catalog, graphics, 83);
        assert_eq!(fallback == 24, resolve(catalog, graphics, 83).is_err(), "gfx {graphics}");
    }
}
