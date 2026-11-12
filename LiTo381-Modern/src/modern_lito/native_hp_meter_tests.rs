//! Pure HPBAR tests (T046-3): no sockets, processes, or client resources.
use super::*;
use crate::modern_lito::native_world::World;
use crate::modern_lito::native_world_scene::movement_tests::world;

const MONSTER: u32 = 0x6810_0145;

/// `Protocol.hitRatio` / candidate `NativeSourceMonsterHp`: `68 d(id) c(ratio)`.
fn hit_ratio(id: u32, ratio: u8) -> Vec<u8> {
    let mut b = vec![0x68];
    b.extend(id.to_le_bytes());
    b.push(ratio);
    b
}

/// A second actor shaped like the fixture's own actor, carrying the given
/// S_PUT_OBJECT hp byte (quest/time-attack: 0..100, ordinary monster: 0xFF).
fn with_monster(put_ratio: u8) -> World {
    let mut w = world();
    let mut a = w.own_actor().unwrap().clone();
    a.id = MONSTER;
    a.x += 2;
    a.hp_ratio = put_ratio;
    w.actors.insert(MONSTER, a);
    w
}

#[test]
fn receive_accepts_ratio_and_hide255_only() {
    for v in 0..=100u8 {
        assert_eq!(receive(v).unwrap(), v);
    }
    assert_eq!(receive(HIDDEN).unwrap(), HIDDEN);
    for v in 101..=254u8 {
        assert!(receive(v).is_err(), "{v} must be rejected");
    }
    assert_eq!(visible_ratio(0), Some(0));
    assert_eq!(visible_ratio(100), Some(100));
    assert_eq!(visible_ratio(HIDDEN), None);
    assert_eq!(visible_ratio(150), None);
}

#[test]
fn own_fixture_actor_is_hidden_by_server_value() {
    let w = world();
    let own = w.own_actor().unwrap();
    assert_eq!(own.hp_ratio, HIDDEN);
    assert!(layout(&w, &[(own.id, [300, 200, 348, 260])]).is_empty());
}

#[test]
fn quest_monster_meter_updates_and_hide255_hides() {
    let mut w = with_monster(100);
    let hits = [(MONSTER, [300, 200, 348, 260])];
    let bars = layout(&w, &hits);
    assert_eq!(bars.len(), 1);
    assert_eq!((bars[0].ratio, bars[0].fill), (100, BAR_WIDTH - 2));

    let before = w.visual_revision;
    assert!(w.accept(&hit_ratio(MONSTER, 40)).is_ok());
    assert_eq!(w.actors[&MONSTER].hp_ratio, 40);
    assert!(w.visual_revision > before);
    let bars = layout(&w, &hits);
    assert_eq!((bars[0].ratio, bars[0].fill), (40, (BAR_WIDTH - 2) * 40 / 100));

    // Candidate NativeSourceMonsterHp.hide: death sends c(255) to prior viewers.
    w.accept(&hit_ratio(MONSTER, HIDDEN)).unwrap();
    assert_eq!(w.actors[&MONSTER].hp_ratio, HIDDEN);
    assert!(layout(&w, &hits).is_empty());

    // A later server ratio shows the bar again; the client never infers.
    w.accept(&hit_ratio(MONSTER, 0)).unwrap();
    let bars = layout(&w, &hits);
    assert_eq!((bars[0].ratio, bars[0].fill), (0, 0));
}

#[test]
fn ordinary_monster_put_0xff_stays_hidden_until_server_sends_ratio() {
    let mut w = with_monster(HIDDEN);
    let hits = [(MONSTER, [300, 200, 348, 260])];
    assert!(layout(&w, &hits).is_empty());
    w.accept(&hit_ratio(MONSTER, 73)).unwrap();
    assert_eq!(layout(&w, &hits)[0].ratio, 73);
}

#[test]
fn out_of_domain_ratio_is_rejected_atomically() {
    let mut w = with_monster(55);
    let revision = w.visual_revision;
    for bad in [101u8, 200, 254] {
        assert!(w.accept(&hit_ratio(MONSTER, bad)).is_err());
    }
    assert!(w.accept(&hit_ratio(0, 50)).is_err());
    let mut long = hit_ratio(MONSTER, 50);
    long.push(0);
    assert!(w.accept(&long).is_err());
    assert_eq!(w.actors[&MONSTER].hp_ratio, 55);
    assert_eq!(w.visual_revision, revision);
}

#[test]
fn unknown_actor_meter_is_an_orphan_not_a_bar() {
    let mut w = world();
    let orphans = w.orphan_updates;
    w.accept(&hit_ratio(0x1234_5678, 50)).unwrap();
    assert_eq!(w.orphan_updates, orphans + 1);
    assert!(layout(&w, &[(0x1234_5678, [0, 50, 48, 90])]).is_empty());
}

#[test]
fn layout_places_bar_above_body_or_below_at_top_edge_and_clamps() {
    let w = with_monster(50);
    let bar = &layout(&w, &[(MONSTER, [300, 200, 348, 260])])[0];
    assert_eq!((bar.x, bar.y), (304, 200 - BAR_HEIGHT - 1));
    let bar = &layout(&w, &[(MONSTER, [300, 3, 348, 60])])[0];
    assert_eq!(bar.y, 61);
    let bar = &layout(&w, &[(MONSTER, [-30, 200, 10, 260])])[0];
    assert_eq!(bar.x, 0);
    let bar = &layout(&w, &[(MONSTER, [790, 200, 830, 260])])[0];
    assert_eq!(bar.x, 800 - BAR_WIDTH);
}

#[test]
fn fill_width_is_proportional_and_nonzero_for_any_life() {
    assert_eq!(fill_width(0), 0);
    assert_eq!(fill_width(1), 1);
    assert_eq!(fill_width(50), (BAR_WIDTH - 2) / 2);
    assert_eq!(fill_width(100), BAR_WIDTH - 2);
    let mut last = 0;
    for v in 0..=100u8 {
        let f = fill_width(v);
        assert!(f >= last && f <= BAR_WIDTH - 2);
        last = f;
    }
}

#[test]
fn draw_paints_frame_fill_and_empty_pixels() {
    let mut canvas = Canvas::new(800, 600).unwrap();
    let bar = Bar { id: MONSTER, ratio: 50, x: 100, y: 100, fill: fill_width(50) };
    let receipt = draw(&mut canvas, std::slice::from_ref(&bar)).unwrap();
    let px = |x: i32, y: i32| {
        let i = ((y * 800 + x) * 4) as usize;
        [canvas.rgba()[i], canvas.rgba()[i + 1], canvas.rgba()[i + 2]]
    };
    assert_eq!(px(100, 100), [FRAME[0], FRAME[1], FRAME[2]]);
    assert_eq!(px(101, 102), [FILL[0], FILL[1], FILL[2]]);
    assert_eq!(px(101 + bar.fill - 1, 102), [FILL[0], FILL[1], FILL[2]]);
    assert_eq!(px(101 + bar.fill, 102), [EMPTY[0], EMPTY[1], EMPTY[2]]);
    assert_eq!(px(100 + BAR_WIDTH, 102), [0, 0, 0]);
    assert_eq!(receipt["bars"][0]["ratio"], 50);
    assert_eq!(receipt["native_geometry_complete"], false);
    // Clipped at the canvas edge without error.
    draw(&mut canvas, &[Bar { x: 790, y: 597, ..bar }]).unwrap();
}
