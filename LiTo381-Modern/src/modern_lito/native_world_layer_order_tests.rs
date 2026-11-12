//! G07.3 (T058 2/2): body-linked attachments, shadow and effect fragments on
//! the current layered raster, without client archives. The scene is the
//! authored two-sector map of `native_world_occlusion_tests`; the sampled cell
//! `P[0]` is open floor.
//!
//! Native rules under test (static decode of image 4271D92F…5C2AB6E9, never
//! executed; addresses in `docs/reports/issue-339.md`):
//! - 4E0200 submits body, attachments in table order, then the shadow;
//!   483770 puts a new entry in front of entries with the same depth, and the
//!   cell list is drawn front to back. At one depth the body therefore paints
//!   last, attachment 0 after attachment 1, and the shadow first.
//! - 4E7000 gives an attachment its own sprite word, plus bit 1 only when the
//!   actor word carries 0x100; 488DAD picks the writer (8, 0x101, 0x80, plain).
//! The effect cases pin the modern equal-key policy (effects are queued after
//! the body roster); native order between two actors depends on timer history.
use super::{
    native_actor_layers::Mix,
    native_actor_states::{Presentation, Style},
    native_multisector_scene::Scene,
    native_world_effects::{self, Effect},
    native_world_layers::{self, Attachment, Body, LayeredBody},
    native_world_occlusion_tests::{cell, rgb, sample, scene, view, FLOOR, P},
    sprite::{CompactSpriteBlock, SpriteBlockReference, SpriteFrame, SpriteFrameFragments},
};
use std::sync::{atomic::AtomicBool, Arc};

/// RGB555 components.
const BODY: [u8; 3] = [30, 30, 2];
const FIRST: [u8; 3] = [2, 28, 28];
const SECOND: [u8; 3] = [28, 2, 28];
const MULTIPLY: [u8; 3] = [16, 31, 8];
const SCREEN: [u8; 3] = [8, 8, 8];
const SHADOW: [u8; 3] = [0, 0, 0];
const EFFECT: [u8; 3] = [31, 16, 0];
const BODY_DEPTH: u8 = 10;
const ABOVE: u8 = 11;

/// One full 24x24 fragment on the anchor half-cell at `depth`.
fn frame(color: [u8; 3], depth: u8) -> SpriteFrame {
    let [r, g, b] = rgb(color);
    let rgba = [r, g, b, 255].repeat(576);
    SpriteFrame {
        fragments: Some(SpriteFrameFragments {
            references: vec![SpriteBlockReference {
                half_x: 0,
                y: 0,
                depth,
                block_id: 0,
            }],
            atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&rgba)]),
        }),
        x_offset: 0,
        y_offset: 0,
        width: 24,
        height: 24,
        frame_type: 0,
        unknown1: 0,
        unknown2: 0,
        rgba,
    }
}
fn average(source: [u8; 3], under: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|c| (source[c] + under[c]) / 2)
}
fn multiply(source: [u8; 3], under: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|c| (u16::from(source[c]) * u16::from(under[c]) / 31) as u8)
}
fn screen(source: [u8; 3], under: [u8; 3]) -> [u8; 3] {
    std::array::from_fn(|c| {
        let high = u16::from(source[c].max(under[c]));
        let low = u16::from(source[c].min(under[c]));
        (high + low * (31 - high) / 31) as u8
    })
}

struct Case<'a> {
    attachments: Vec<(&'a SpriteFrame, Mix)>,
    shadow: Option<&'a SpriteFrame>,
    presentation: Presentation,
    effects: Vec<(&'a SpriteFrame, native_world_effects::Mix)>,
}
impl<'a> Case<'a> {
    fn new(attachments: Vec<(&'a SpriteFrame, Mix)>) -> Self {
        Self {
            attachments,
            shadow: None,
            presentation: Presentation::default(),
            effects: Vec::new(),
        }
    }
    fn shadow(mut self, shadow: &'a SpriteFrame) -> Self {
        self.shadow = Some(shadow);
        self
    }
    fn state(mut self, state: Style) -> Self {
        self.presentation.state = state;
        self
    }
    fn effect(mut self, frame: &'a SpriteFrame, mix: native_world_effects::Mix) -> Self {
        self.effects.push((frame, mix));
        self
    }
    /// RGB555 components of the sampled pixel and whether the body owns it.
    fn shot(self, scene: &Scene, body: &SpriteFrame) -> ([u8; 3], bool) {
        let own = cell(P[0]);
        let bodies = [LayeredBody {
            body: Body {
                id: 7,
                center: view().cell_to_pixel(own).unwrap(),
                frame: body,
            },
            attachments: self
                .attachments
                .iter()
                .map(|&(frame, mix)| Attachment { frame, mix })
                .collect(),
            shadow: self.shadow,
        }];
        let effects: Vec<_> = self
            .effects
            .iter()
            .map(|&(frame, mix)| Effect {
                frame,
                position: own,
                mix,
            })
            .collect();
        let output = native_world_layers::compose_with_presentations(
            scene,
            view(),
            320,
            240,
            &bodies,
            &Default::default(),
            &effects,
            &[self.presentation],
            &AtomicBool::new(false),
        )
        .unwrap();
        let (x, y) = sample(P[0]);
        let at = (y as usize * 320 + x as usize) * 4;
        let pixel: [u8; 3] = output.canvas.rgba()[at..at + 3].try_into().unwrap();
        // Every writer stores an expanded five-bit component.
        assert_eq!(pixel, rgb(pixel.map(|v| v >> 3)));
        (pixel.map(|v| v >> 3), output.visibility[0].1.contains(x, y))
    }
}

#[test]
fn equal_depth_layers_paint_shadow_then_attachments_in_reverse_then_the_body() {
    let scene = scene();
    let body = frame(BODY, BODY_DEPTH);
    let level = |color| frame(color, BODY_DEPTH);
    let above = |color| frame(color, ABOVE);
    let (first, second, shadow) = (level(FIRST), level(SECOND), level(SHADOW));
    let (first_above, second_above, shadow_above) = (above(FIRST), above(SECOND), above(SHADOW));

    assert_eq!(Case::new(vec![]).shot(&scene, &body), (BODY, true));
    // Same depth: the body was submitted first, so it is painted last.
    assert_eq!(
        Case::new(vec![(&first, Mix::Over), (&second, Mix::Over)])
            .shadow(&shadow)
            .shot(&scene, &body),
        (BODY, true)
    );
    // A deeper fragment paints after the body; among equal attachments the
    // first table entry is on top.
    assert_eq!(
        Case::new(vec![(&first_above, Mix::Over), (&second_above, Mix::Over)]).shot(&scene, &body),
        (FIRST, true)
    );
    assert_eq!(
        Case::new(vec![(&second_above, Mix::Over), (&first_above, Mix::Over)]).shot(&scene, &body),
        (SECOND, true)
    );
    // The shadow is submitted last, hence painted before an equal attachment:
    // an opaque attachment hides it, a multiply attachment darkens its result.
    assert_eq!(
        Case::new(vec![(&first_above, Mix::Over)])
            .shadow(&shadow_above)
            .shot(&scene, &body),
        (FIRST, true)
    );
    let multiply_above = above(MULTIPLY);
    assert_eq!(
        Case::new(vec![(&multiply_above, Mix::Multiply)])
            .shadow(&shadow_above)
            .shot(&scene, &body),
        (multiply(MULTIPLY, average(SHADOW, BODY)), true)
    );
    // The shadow alone averages with what is under it and never takes the
    // pixel away from its body; without a body pixel it owns nothing.
    assert_eq!(
        Case::new(vec![]).shadow(&shadow_above).shot(&scene, &body),
        (average(SHADOW, BODY), true)
    );
    assert_eq!(
        Case::new(vec![])
            .shadow(&shadow_above)
            .state(Style::Hidden)
            .shot(&scene, &body),
        (FLOOR, false)
    );
}

#[test]
fn attachment_writers_follow_the_sprite_word_and_only_inherit_the_half_state() {
    let scene = scene();
    let body = frame(BODY, BODY_DEPTH);
    let (over, half, multiplied, screened) = (
        frame(FIRST, ABOVE),
        frame(SECOND, ABOVE),
        frame(MULTIPLY, ABOVE),
        frame(SCREEN, ABOVE),
    );
    let state = |state| Presentation {
        state,
        body_half: false,
    };
    let one = |frame: &SpriteFrame, mix: Mix, presentation: Presentation| {
        let mut case = Case::new(vec![(frame, mix)]);
        case.presentation = presentation;
        case.shot(&scene, &body).0
    };

    // Neutral actor: each attachment keeps the writer of its own word.
    let neutral = Presentation::default();
    assert_eq!(one(&over, Mix::Over, neutral), FIRST);
    assert_eq!(one(&half, Mix::Half, neutral), average(SECOND, BODY));
    assert_eq!(one(&multiplied, Mix::Multiply, neutral), multiply(MULTIPLY, BODY));
    assert_eq!(one(&screened, Mix::Screen, neutral), screen(SCREEN, BODY));

    // Half actor (own invisibility 0x100, or the attribute-100 switch): the
    // body averages with the floor; plain and multiply attachments take the
    // half writer (0x80|1 is tested as 0x101 first), screen stays screen.
    let half_body = average(BODY, FLOOR);
    let as_half = state(Style::Half);
    assert_eq!(Case::new(vec![]).state(Style::Half).shot(&scene, &body), (half_body, true));
    assert_eq!(one(&over, Mix::Over, as_half), average(FIRST, half_body));
    assert_eq!(one(&half, Mix::Half, as_half), average(SECOND, half_body));
    assert_eq!(one(&multiplied, Mix::Multiply, as_half), average(MULTIPLY, half_body));
    assert_ne!(average(MULTIPLY, half_body), multiply(MULTIPLY, half_body));
    assert_eq!(one(&screened, Mix::Screen, as_half), screen(SCREEN, half_body));

    // Green and dim change the body only.
    let green_body = [BODY[0], (BODY[1] + 10).min(31), BODY[2]];
    let as_green = state(Style::Green(10));
    assert_eq!(Case::new(vec![]).state(Style::Green(10)).shot(&scene, &body).0, green_body);
    assert_eq!(one(&over, Mix::Over, as_green), FIRST);
    assert_eq!(one(&multiplied, Mix::Multiply, as_green), multiply(MULTIPLY, green_body));
    assert_eq!(one(&half, Mix::Half, as_green), average(SECOND, green_body));
    let dim_body = BODY.map(|v| v / 2 + 3);
    let as_dim = state(Style::Dim);
    assert_eq!(one(&over, Mix::Over, as_dim), FIRST);
    assert_eq!(one(&multiplied, Mix::Multiply, as_dim), multiply(MULTIPLY, dim_body));
    assert_eq!(one(&screened, Mix::Screen, as_dim), screen(SCREEN, dim_body));

    // A catalogue body flag 1 is a body word, not 0x100: attachments keep
    // their own writers over the half body.
    let as_flag1 = Presentation {
        state: Style::Normal,
        body_half: true,
    };
    assert_eq!(one(&over, Mix::Over, as_flag1), FIRST);
    assert_eq!(one(&multiplied, Mix::Multiply, as_flag1), multiply(MULTIPLY, half_body));

    // The shadow keeps the stable half writer in every visible state.
    let shadow = frame(SHADOW, ABOVE);
    for (state, under) in [
        (Style::Normal, BODY),
        (Style::Half, half_body),
        (Style::Green(10), green_body),
        (Style::Dim, dim_body),
    ] {
        assert_eq!(
            Case::new(vec![]).shadow(&shadow).state(state).shot(&scene, &body),
            (average(SHADOW, under), true),
            "{state:?}"
        );
    }
    // Hidden actors submit no body, attachment or shadow.
    assert_eq!(
        Case::new(vec![(&over, Mix::Over), (&multiplied, Mix::Multiply)])
            .shadow(&shadow)
            .state(Style::Hidden)
            .shot(&scene, &body),
        (FLOOR, false)
    );
}

#[test]
fn effects_share_the_depth_queue_and_stay_click_through() {
    use native_world_effects::Mix as EffectMix;
    let scene = scene();
    let body = frame(BODY, BODY_DEPTH);
    let (level, above) = (frame(EFFECT, BODY_DEPTH), frame(EFFECT, ABOVE));
    let attachment = frame(FIRST, ABOVE);
    // Modern equal-key policy: effects are queued after the body roster, so an
    // effect at the body's own depth paints under it.
    assert_eq!(
        Case::new(vec![]).effect(&level, EffectMix::Over).shot(&scene, &body),
        (BODY, true)
    );
    // The source depth byte decides otherwise; the effect never owns the pixel.
    assert_eq!(
        Case::new(vec![]).effect(&above, EffectMix::Over).shot(&scene, &body),
        (EFFECT, true)
    );
    assert_eq!(
        Case::new(vec![]).effect(&above, EffectMix::Screen).shot(&scene, &body),
        (screen(EFFECT, BODY), true)
    );
    assert_eq!(
        Case::new(vec![]).effect(&above, EffectMix::Multiply).shot(&scene, &body),
        (multiply(EFFECT, BODY), true)
    );
    // Equal depth against an attachment: the attachment was queued first and
    // paints last.
    assert_eq!(
        Case::new(vec![(&attachment, Mix::Over)])
            .effect(&above, EffectMix::Over)
            .shot(&scene, &body),
        (FIRST, true)
    );
    // Two effects at one key: the later submission paints first.
    let other = frame(SECOND, ABOVE);
    assert_eq!(
        Case::new(vec![])
            .effect(&above, EffectMix::Over)
            .effect(&other, EffectMix::Over)
            .shot(&scene, &body),
        (EFFECT, true)
    );
    // A hidden body leaves its cell to the effect, still without an owner.
    assert_eq!(
        Case::new(vec![])
            .state(Style::Hidden)
            .effect(&above, EffectMix::Over)
            .shot(&scene, &body),
        (EFFECT, false)
    );
}
