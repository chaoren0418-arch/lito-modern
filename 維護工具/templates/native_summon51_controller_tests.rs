
#[cfg(test)]
mod summon51_controller_development_tests {
    use super::*;
    fn controller(summon:bool,age:Duration)->LoginController {
        let mut c=LoginController::default();let f:serde_json::Value=serde_json::from_slice(include_bytes!(r"D:\lineage開發\3-lito開發\lito重建版\LiTo381-Modern\tests\fixtures\native-world-current.json")).unwrap();for key in ["world","start","status","actor","bot"]{let h=f["packets"][key].as_str().unwrap();let b:Vec<_>=(0..h.len()).step_by(2).map(|i|u8::from_str_radix(&h[i..i+2],16).unwrap()).collect();c.state.world.accept(&b).unwrap();}c.state.phase=LoginPhase::WorldReady;c.state.game_connected=true;c.skill_summon_pending=summon;
        c.skill_pending=Some(super::super::native_skills::Pending{since:Instant::now()-age,casts:c.state.world.own_cast_packets,messages:c.state.world.message_count,generation:c.state.world.generation,incarnation:c.state.world.own_actor().unwrap().incarnation});let mut b=vec![0x20];b.extend(c.state.world.own_actor().unwrap().id.to_le_bytes());b.push(19);c.state.world.accept(&b).unwrap();c
    }
    #[test]
    fn summon_controller_keeps_late_rejection_correlation_across_two_separate_polls() {
        for code in [353u16,743,319]{let mut c=controller(true,Duration::ZERO);c.poll();assert_eq!(c.state.skill_feedback,super::super::native_skills::Feedback::CastAction);assert!(c.skill_pending.is_some());let mut b=vec![0x10];b.extend(code.to_le_bytes());b.push(0);c.state.world.accept(&b).unwrap();c.poll();assert_eq!(c.state.skill_feedback,super::super::native_skills::Feedback::Rejected(code));assert!(c.skill_pending.is_none());}
    }
    #[test]
    fn summon_controller_keeps_original_two_second_bound_and_other_skill_terminal_policy() {
        let mut ordinary=controller(false,Duration::ZERO);ordinary.poll();assert!(ordinary.skill_pending.is_none());assert_eq!(ordinary.state.skill_feedback,super::super::native_skills::Feedback::CastAction);
        let mut expired=controller(true,Duration::from_secs(3));expired.poll();assert!(expired.skill_pending.is_none());assert_eq!(expired.state.skill_feedback,super::super::native_skills::Feedback::CastAction);
        let mut retired=controller(true,Duration::ZERO);retired.state.world.generation+=1;retired.poll();assert!(retired.skill_pending.is_none());assert_eq!(retired.state.skill_feedback,super::super::native_skills::Feedback::Interrupted);
    }
}
