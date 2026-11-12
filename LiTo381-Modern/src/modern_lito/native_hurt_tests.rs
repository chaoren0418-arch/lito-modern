use super::*;

#[test]
fn lt_relative_family_is_base_plus_table_and_literal_obelisks() {
    let family: Vec<u16> = LT_RELATIVE.iter().map(|o| LT_BASE + o).collect();
    assert_eq!(family, [5902, 5946, 5947, 5948, 5949, 6004, 6005, 6167, 6171]);
    for g in family.iter().copied().chain([2665, 2667, 2669, 2671, 2674]) {
        assert!(lt_relative(g), "{g}");
        assert!(hurt_exempt(g), "{g}");
    }
    // Neighbours and the unshifted table values are ordinary bodies.
    for g in [5901, 5903, 8, 52, 277, 2666, 2673, 5894, 0, 61] {
        assert!(!lt_relative(g), "{g}");
    }
}

#[test]
fn hurt_exemption_covers_literal_bodies_and_special_form_only() {
    for g in [1245, 1590, 2001, 5620, 5920] {
        assert!(hurt_exempt(g), "{g}");
        assert!(!lt_relative(g), "{g} is not 4DF7C0");
    }
    for g in [0, 1, 61, 1244, 1246, 5919, 5921, 5894] {
        assert!(!hurt_exempt(g), "{g}");
    }
}

#[test]
fn landed_hit_effect_replacements_follow_40e5a2_order() {
    assert_eq!(landed_hit(61), LandedHit::Hurt);
    for g in [1245, 1590, 2001, 5620] {
        assert_eq!(landed_hit(g), LandedHit::Effect { graphics: 0x4e0 });
    }
    assert_eq!(landed_hit(5902), LandedHit::Effect { graphics: 5908 });
    assert_eq!(landed_hit(2674), LandedHit::Effect { graphics: 5908 });
    assert_eq!(landed_hit(5920), LandedHit::Effect { graphics: 5921 });
}

#[test]
fn queue_threshold_is_more_than_three_pending_and_consecutive_action2() {
    for pending in 0..=3 {
        assert!(!queue_suppresses(pending, Some(2), 2), "pending {pending}");
    }
    assert!(queue_suppresses(4, Some(2), 2));
    assert!(queue_suppresses(40, Some(2), 2));
    // Only an identical action-2 predecessor suppresses; other actions never do.
    assert!(!queue_suppresses(4, Some(1), 2));
    assert!(!queue_suppresses(4, Some(19), 2));
    assert!(!queue_suppresses(4, None, 2));
    assert!(!queue_suppresses(4, Some(1), 1));
    assert!(!queue_suppresses(4, Some(18), 18));
}

#[test]
fn hit_dispatch_ignores_guarded_targets_then_queues_walk_else_interrupts() {
    let t = |dead, action_44_45, graphics, walking| HurtTarget { dead, action_44_45, graphics, walking };
    assert_eq!(dispatch(&t(false, false, 61, false)), HurtDispatch::Interrupt);
    assert_eq!(dispatch(&t(false, false, 61, true)), HurtDispatch::Enqueue);
    assert_eq!(dispatch(&t(true, false, 61, true)), HurtDispatch::Ignore);
    assert_eq!(dispatch(&t(false, true, 61, false)), HurtDispatch::Ignore);
    for g in [1245, 5902, 5920, 2665] {
        assert_eq!(dispatch(&t(false, false, g, false)), HurtDispatch::Ignore, "{g}");
        assert_eq!(dispatch(&t(false, false, g, true)), HurtDispatch::Ignore, "{g}");
    }
}

#[test]
fn formal_ltlist_header_has_no_second_integer_so_action0_multiplier_is_zero() {
    // First bytes of the decoded formal member (SHA256 F55779AE…42603).
    let formal = b"285\r\n#0\t4\tgarden water up\t\t3.fire(0 4,0.0:4)\r\n";
    assert_eq!(ltlist_action0_multiplier(formal).unwrap(), FORMAL_LTLIST_ACTION0_MULTIPLIER);
    assert_eq!(ltlist_action0_multiplier(b"285 90\r\n#0").unwrap(), 90);
    assert_eq!(ltlist_action0_multiplier(b"285\t125").unwrap(), 125);
    // ' ' keeps a pending '-', any other separator clears it.
    assert_eq!(ltlist_action0_multiplier(b"285 - 80").unwrap(), -80);
    assert_eq!(ltlist_action0_multiplier(b"285 -x80").unwrap(), 80);
    // NUL and CR end the copied line; LF is an ordinary separator.
    assert_eq!(ltlist_action0_multiplier(b"285\0 90").unwrap(), 0);
    assert_eq!(ltlist_action0_multiplier(b"285\n90\r").unwrap(), 90);
    assert_eq!(ltlist_action0_multiplier(b"").unwrap(), 0);
    // 32-bit wrap of the digit accumulator, not saturation.
    assert_eq!(ltlist_action0_multiplier(b"0 4294967297").unwrap(), 1);
    assert!(ltlist_action0_multiplier(&[b'2', b' ', 0xb3, b'9']).is_err());
    let long = vec![b' '; 0x200];
    assert!(ltlist_action0_multiplier(&long).is_err());
    assert_eq!(ltlist_action0_multiplier(&long[..0x1ff]).unwrap(), 0);
}

#[test]
fn action0_multiplier_scales_only_walk_family_with_truncation() {
    for action in ACTION0_FAMILY {
        assert_eq!(scale_action0_frame_ms(167, action, 0), 167);
        assert_eq!(scale_action0_frame_ms(167, action, 90), 150);
        assert_eq!(scale_action0_frame_ms(-167, action, 90), -150);
    }
    for action in [1, 2, 3, 8, 18, 19, 83] {
        assert_eq!(scale_action0_frame_ms(167, action, 90), 167, "{action}");
    }
    assert!(!ACTION0_FAMILY.contains(&83));
}

#[test]
fn unsigned32_deadlines_wrap_sort_first_and_fire_or_stall_like_native() {
    let near = u32::MAX - 10;
    let wrapped = deadline(near, 20);
    assert_eq!(wrapped, 9);
    // Pre-wrap deadline near the top, then a wrapped one: the wrapped
    // deadline sorts first (unsigned order), equal ones stay FIFO.
    let mut q = vec![];
    for due in [deadline(near, 5), wrapped, deadline(near, 5), 9] {
        let at = insertion_index(&q, due);
        q.insert(at, due);
    }
    assert_eq!(q, [9, 9, u32::MAX - 5, u32::MAX - 5]);
    // Before the clock wraps every entry is due: the wrapped one fires early.
    let mut early = q.clone();
    assert_eq!(drain(&mut early, near), [9, 9]);
    assert_eq!(early, [u32::MAX - 5, u32::MAX - 5]);
    assert_eq!(drain(&mut early, u32::MAX - 5), [u32::MAX - 5, u32::MAX - 5]);
    // After the clock wraps, a pre-wrap deadline stalls until the dword
    // climbs back; it is not treated as overdue by a half-range heuristic.
    let mut stalled = vec![u32::MAX - 5];
    assert!(drain(&mut stalled, 3).is_empty());
    assert!(drain(&mut stalled, u32::MAX - 6).is_empty());
    assert_eq!(drain(&mut stalled, u32::MAX - 5), [u32::MAX - 5]);
    // FIFO among equal deadlines matches the 4D6625 `jbe` stop.
    assert_eq!(insertion_index(&[1, 5, 5, 7], 5), 3);
    assert_eq!(insertion_index(&[1, 5, 5, 7], 0), 0);
    assert_eq!(insertion_index(&[], 0), 0);
}
