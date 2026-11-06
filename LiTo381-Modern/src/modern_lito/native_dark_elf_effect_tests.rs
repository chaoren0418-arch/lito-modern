use super::*;
use crate::modern_lito::{native_effect_timing::Clip, sprite_actions::ActionLimits};
use std::sync::Arc;

const RAW: &[u8] = include_bytes!("../../tests/fixtures/native-dark-elf.spr.txt");
fn catalog(raw: &[u8]) -> SpriteActionCatalog {
    SpriteActionCatalog::parse_for_inspection(raw, ActionLimits::default()).unwrap()
}

#[test]
fn t021_exact_original_definitions_reject_timing_audio_layer_and_control_changes() {
    let c = catalog(RAW);
    let animations = qualify(&c).unwrap();
    for (id, _, a) in &animations {
        if *id == 2954 {
            assert!(a.frames.iter().all(|f| f.events.is_empty()));
        }
    }
    let text = std::str::from_utf8(RAW).unwrap();
    for (from, to) in [
        ("0.0:7", "0.0:6"),
        ("[1505", "[1506"),
        ("[1509", "<1509"),
        ("1 2954", "1 2955"),
        ("104.attr(128)", "104.attr(8)"),
        ("109.effect(0 -1)", "109.effect(1 -1)"),
        ("0.0:3[1502", "0.0:4[1502"),
    ] {
        assert!(text.contains(from));
        assert!(
            qualify(&catalog(text.replace(from, to).as_bytes())).is_err(),
            "{from}"
        );
    }
}

#[test]
fn t021_finite_callbacks_have_original_terminal_dwell_without_server_buff_deadline() {
    for (id, _, a) in qualify(&catalog(RAW)).unwrap() {
        if id == 2954 {
            continue;
        }
        let expected: u64 = a
            .frames
            .iter()
            .map(|f| u64::from(f.duration_ticks) * 1000 / 24)
            .sum();
        let clip = Clip::new(Arc::new(a)).unwrap();
        let mut p = clip.start();
        let mut sample = p.advance(0).unwrap();
        assert_eq!(sample.source_frame, 1);
        assert_eq!(sample.crossed_cues.len(), 1);
        let mut at = 0;
        while !sample.finished {
            at = p.next_callback_millis().unwrap();
            sample = p.advance(at * 1_000_000).unwrap();
            assert!(sample.crossed_cues.is_empty());
        }
        assert_eq!(at, expected);
        if id == 2942 {
            assert_eq!(at, 1496);
        }
        assert!(p.advance(320_000_000_000).unwrap().finished);
        assert!(p.next_callback_millis().is_none());
        // Late callback dispatch does not fast-forward through positive dwells.
        let mut late = clip.start();
        late.advance(0).unwrap();
        assert!(!late.advance(320_000_000_000).unwrap().finished);
    }
}
