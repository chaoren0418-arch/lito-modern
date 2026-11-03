use lito_modern::modern_lito::{
    native_world::World,
    native_world_animation::{Animator, Clip},
    sprite_actions::{
        ActionFrame, ActionLimits, Animation, Directionality, FrameEvent, SpriteActionCatalog,
    },
};
fn animation() -> Animation {
    Animation {
        label_bytes: b"idle".to_vec(),
        directionality: Directionality::EightWay,
        frames_per_second: 3,
        frames: vec![
            ActionFrame {
                image_id: 0,
                subframe: 9,
                duration_ticks: 0,
                events: vec![],
            },
            ActionFrame {
                image_id: 0,
                subframe: 0,
                duration_ticks: 1,
                events: vec![],
            },
            ActionFrame {
                image_id: 0,
                subframe: 1,
                duration_ticks: 2,
                events: vec![],
            },
        ],
    }
}
fn actor() -> lito_modern::modern_lito::native_world::Actor {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let s = f["packets"]["actor"].as_str().unwrap();
    let b: Vec<_> = (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect();
    lito_modern::modern_lito::native_world::actor(&b).unwrap()
}
#[test]
fn rational_repeat_skips_zero_dwell_and_has_no_per_frame_rounding_drift() {
    let c = Clip::new(0, animation()).unwrap();
    assert_eq!(c.sample(0, true), 1);
    assert_eq!(c.sample(333_333_333, true), 1);
    assert_eq!(c.sample(333_333_334, true), 2);
    assert_eq!(c.sample(1_000_000_000, true), 1);
    assert_eq!(c.sample(30_333_333_334, true), 2);
    assert_eq!(c.sample(u64::MAX, false), 2);
    let mut a = animation();
    a.frames[1].events.push(FrameEvent::Skip);
    assert!(Clip::new(0, a).is_err());
    let mut a = animation();
    a.frames.iter_mut().for_each(|f| f.duration_ticks = 0);
    assert!(Clip::new(0, a).is_err());
}
#[test]
fn walking_interpolates_then_returns_to_idle_and_correction_snaps() {
    let catalog = SpriteActionCatalog::parse(
        b"#0 16 test 0.walk(1 2,0.0:12 0.1:12) 3.idle(1 2,8.0:12 8.1:12)\n",
        ActionLimits::default(),
    )
    .unwrap();
    let mut a = actor();
    a.speed=0;
    a.x = 100;
    a.y = 100;
    a.motion_revision = 1;
    a.motion_from = None;
    let mut animator = Animator::default();
    animator.begin(0, [a.id].into_iter()).unwrap();
    assert_eq!(animator.pose(&catalog, &a, 0).unwrap().phase, "idle");
    a.motion_from = Some([100, 100]);
    a.x = 101;
    a.motion_revision = 2;
    assert_eq!(
        animator.pose(&catalog, &a, 1_000_000_000).unwrap().position,
        [100., 100.]
    );
    let mid = animator.pose(&catalog, &a, 1_500_000_000).unwrap();
    assert_eq!(mid.phase, "walk");
    assert_eq!(mid.position, [100.5, 100.]);
    assert_eq!(mid.source_frame, 1);
    assert_eq!(
        animator.pose(&catalog, &a, 2_000_000_000).unwrap().phase,
        "idle"
    );
    a.motion_revision = 3;
    a.motion_from = None;
    a.x = 100;
    assert_eq!(
        animator.pose(&catalog, &a, 2_100_000_000).unwrap().position,
        [100., 100.]
    );
    animator.begin(2_100_000_000, [a.id].into_iter()).unwrap();
    assert!(animator.begin(2_000_000_000, [a.id].into_iter()).is_err());
}
#[test]
fn paralysis_pairs_block_until_their_own_release_and_world_reset_clears() {
    let f: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w = World::default();
    for k in ["world", "start", "status", "actor"] {
        let s = f["packets"][k].as_str().unwrap();
        let b: Vec<_> = (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
            .collect();
        w.accept(&b).unwrap();
    }
    let m: serde_json::Value =
        serde_json::from_slice(include_bytes!("fixtures/native-motion-current.json")).unwrap();
    for pair in m["paralysis"].as_array().unwrap().chunks(2) {
        for (n, row) in pair.iter().enumerate() {
            let s = row["body"].as_str().unwrap();
            let b: Vec<_> = (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
                .collect();
            w.accept(&b).unwrap();
            assert_eq!(w.move_request(0).is_err(), n == 0);
        }
    }
    w.accept(&[0x25, 2]).unwrap();
    w.accept(&[0x25, 10]).unwrap();
    w.accept(&[0x25, 3]).unwrap();
    assert!(w.move_request(0).is_err());
    w.accept(&[0x25, 11]).unwrap();
    assert!(w.move_request(0).is_ok());
    let before = serde_json::to_value(&w).unwrap();
    for b in [&[0x25][..], &[0x25, 2, 0], &[0x25, 0xff]] {
        assert!(w.accept(b).is_err());
        assert_eq!(serde_json::to_value(&w).unwrap(), before);
    }
    w.accept(&[0x25, 2]).unwrap();
    w.accept(&[0x28, 0x34, 2, 0]).unwrap();
    assert!(w.movement_blocks.is_empty());
}

mod hurt_queued_after_motion {
use lito_modern::modern_lito as m;
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
fn motion_hurt_preflight()->Result<Value> {
    let fixture:Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json")))?;
    let hex=fixture["packets"]["actor"].as_str().context("retained actor fixture")?;
    let bytes=(0..hex.len()).step_by(2).map(|i|u8::from_str_radix(&hex[i..i+2],16)).collect::<std::result::Result<Vec<_>,_>>()?;
    let catalog=m::sprite_actions::SpriteActionCatalog::parse(b"#0 16 test 0.walk(1 2,0.0:12 0.1:12) 2.hurt(1 1,16.0:2) 3.idle(1 2,8.0:12 8.1:12) 8.death(1 1,24.0:2)\n",m::sprite_actions::ActionLimits::default())?;
    let mut cases=vec![];
    for heading in 0..8u8 {
        let mut actor=m::native_world::actor(&bytes)?;actor.graphics=0;actor.speed=0;actor.brave=0;actor.walk_action=0;actor.dead=false;
        actor.incarnation=1;actor.actions.clear();actor.action_sequence=0;actor.x=100;actor.y=100;actor.motion_revision=1;actor.motion_from=None;
        let mut animator=m::native_world_animation::Animator::default();animator.begin(0,[actor.id].into_iter())?;animator.pose(&catalog,&actor,0)?;
        let destination=m::native_world::destination(100,100,heading)?;actor.x=destination.0;actor.y=destination.1;actor.motion_from=Some([100,100]);actor.motion_revision=2;
        let mut history=m::native_water_clock::History::default();history.bind(7)?;
        let project=|p:[f64;2]|[(24.*(p[0]+p[1])).round() as i32,(12.*(-p[0]+p[1])).round() as i32];
        let start=project([100.,100.]);let end=project([f64::from(destination.0),f64::from(destination.1)]);
        let mut samples=vec![];
        for now in [1_000_000_000u64,1_100_000_000,1_100_000_000,1_200_000_000,1_400_000_000,1_900_000_000,2_000_000_000,2_100_000_000] {
            if samples.len()==2 {ensure!(animator.impact(&actor,999),"queued hurt rejected");}
            let pose=animator.pose(&catalog,&actor,now)?;let point=project(pose.position);
            history.sample(7,1,Some(m::native_water_clock::Motion{identity:[u64::from(actor.id),actor.incarnation,actor.motion_revision],source:[100,100],destination:[actor.x,actor.y],offset:[point[0]-start[0],point[1]-start[1]],extent:[end[0]-start[0],end[1]-start[1]]}))?;
            if now<2_000_000_000 {ensure!(pose.phase=="walk","hurt interrupted native running walk");}
            if now==2_000_000_000 {ensure!(pose.action==2 && pose.position==[f64::from(actor.x),f64::from(actor.y)],"queued hurt did not start at walk completion");}
            if now==2_100_000_000 {ensure!(pose.phase=="idle","walk replayed after completed hurt");}
            samples.push(json!({"now":now,"phase":pose.phase,"position":pose.position,"scroll":history.values()}));
        }
        actor.incarnation=2;actor.motion_revision=3;actor.motion_from=None;
        ensure!(animator.pose(&catalog,&actor,2_200_000_000)?.phase=="idle","old hurt crossed actor incarnation");
        cases.push(json!({"heading":heading,"samples":samples,"new_incarnation_retired_hurt":true}));
    }
    Ok(json!({"scope":"isolated synthetic catalog and retained actor fixture; no live World or authority writes","eight_directions":cases,"hurt_waited_for_motion_boundary":true,"camera_history_rewind_guard_preserved":true,"actual_network_sessions":0}))
}

#[test]
fn eight_directions_hurt_waits_for_walk_without_camera_rewind_and_old_life_replay(){motion_hurt_preflight().unwrap();}
}
