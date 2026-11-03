//! Narrow, opaque qualification of native four/eight-source walk/action0 paths.
//!
//! Native evidence: the eight-frame branches in 4E1410/4E1870/4E2150/4E2E37
//! precede frame+12 bit2 (`>`). They consume every source, not skip a frame or
//! add an extra quarter-step. Initializers 4E11E0/4E1280 populate odd/even
//! 834CA8/834C68. Remote loc4E1A7E initializes source0 then immediately advances
//! to source1. Local loc4E2AFD first applies the source0 even CAMERA pulse;
//! boundaries D1..D7 pulse the current source and advance; T7 loads terminal0.
//! Local loc4E307E uses D0 before starting the next step.
//!
//! This is one unpaced reference step, not a world controller. D_i is the
//! native base integer milliseconds `(ticks * 1000) / fps`; status-dependent
//! 4E8750 adjustments and authority movement cadence are NOT inferred here.
//! Body rendering 4E0200 -> 4CD9D0 uses the cell plus the SPR's original block
//! offsets. CAMERA pulses below must never be added to the body a second time.
//! No world cell, map, server sequence, renderer, audio device or I/O is owned.
//! Generic resolve_action/frame_asset/AnimationClock remain strict about Skip.
//! Four-source proof: 4E1410/4E1870 and local 4E2150/4E2A72 select 834D68
//! before testing bit2; source1..3 then terminal0 uses the same advance. The
//! local source0 pulse and three callbacks sum to one complete projected cell.

use super::sprite_actions::{
    ActionDefinition, ActionError, ActionErrorKind, ActionFrame, ActionKey, Animation,
    Directionality, FrameAsset, FrameEvent, SpriteActionCatalog,
};
use std::{
    fmt,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

const EVEN: [(i32, i32); 8] = [
    (-3, -1),
    (0, -3),
    (3, -1),
    (6, 0),
    (3, 1),
    (0, 3),
    (-3, 1),
    (-6, 0),
];
const ODD: [(i32, i32); 8] = [
    (-3, -2),
    (0, -3),
    (3, -2),
    (6, 0),
    (3, 2),
    (0, 3),
    (-3, 2),
    (-6, 0),
];
const FOUR: [(i32, i32); 8] = [
    (-6, -3),
    (0, -6),
    (6, -3),
    (12, 0),
    (6, 3),
    (0, 6),
    (-6, 3),
    (-12, 0),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixedWalkError {
    Catalog(ActionError),
    Unsupported(&'static str),
    Bound(&'static str),
    InvalidInput(&'static str),
    TimeWentBackwards,
    Cancelled,
}
impl fmt::Display for FixedWalkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog(e) => write!(f, "fixed walk catalog: {e}"),
            Self::Unsupported(reason) => write!(f, "unsupported fixed walk: {reason}"),
            Self::Bound(reason) => write!(f, "fixed walk bound: {reason}"),
            Self::InvalidInput(reason) => write!(f, "invalid fixed walk input: {reason}"),
            Self::TimeWentBackwards => write!(f, "fixed walk elapsed time went backwards"),
            Self::Cancelled => write!(f, "fixed walk playback retired by cancellation"),
        }
    }
}
impl std::error::Error for FixedWalkError {}
type Result<T> = std::result::Result<T, FixedWalkError>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WalkProvenance {
    pub requested_sprite_id: u32,
    pub definition_sprite_id: u32,
    pub graphics_id: u32,
}

/// No arbitrary Animation constructor and no sanitized Animation accessor.
/// Inspection of the original frames is not a certificate for generic playback.
#[derive(Clone, Debug)]
pub struct QualifiedFixedWalk {
    provenance: WalkProvenance,
    animation: Animation,
    durations: Vec<Duration>,
    active_ends: Vec<Duration>,
    completion: Duration,
}

impl QualifiedFixedWalk {
    pub fn from_catalog(catalog: &SpriteActionCatalog, requested: u32) -> Result<Self> {
        Self::from_action(catalog, requested, ActionKey::new(0), false)
    }

    /// The same native four/eight-source locomotion path also reads the
    /// actor's weapon stance (actor+18h), not only action0. Keep the existing
    /// pet/NPC entry strict: only this base-player entry admits the already
    /// observed footstep audio markers. It does not play audio or enable gear.
    pub(crate) fn from_player_catalog(
        catalog: &SpriteActionCatalog,
        appearance: super::world_scene::BaseAppearance,
        weapon_type: u8,
    ) -> Result<Self> {
        let key = super::combat_animation::qualify_weapon_appearance(appearance, weapon_type)
            .map_err(|_| FixedWalkError::Unsupported("unqualified player weapon stance"))?;
        let requested = appearance
            .logical_sprite_id()
            .map_err(|_| FixedWalkError::Unsupported("unqualified player appearance"))?;
        Self::from_action(catalog, requested, ActionKey::new(key), true)
    }

    fn from_action(
        catalog: &SpriteActionCatalog,
        requested: u32,
        key: ActionKey,
        player: bool,
    ) -> Result<Self> {
        // The original resolver checks every alias/header/quarantined attribute,
        // cycle and its own private depth limit before reaching this error.
        // Only this exact terminal failure has the additional locomotion proof.
        let (definition_id, animation) = match catalog.resolve_action(requested, key) {
            Ok(resolved) => (resolved.definition_sprite_id, resolved.animation),
            Err(ActionError {
                kind: ActionErrorKind::UninterpretedEvents(id, failed_key),
                ..
            }) if failed_key == key => {
                let definition = catalog
                    .sprites
                    .get(&id)
                    .and_then(|sprite| sprite.actions.get(&key));
                match definition {
                    Some(ActionDefinition::Animation(animation)) => (id, animation),
                    _ => {
                        return Err(FixedWalkError::Unsupported(
                            "resolver endpoint is not an animation",
                        ))
                    }
                }
            }
            Err(error) => return Err(FixedWalkError::Catalog(error)),
        };
        let source = &catalog.sprites[&requested];
        let definition = &catalog.sprites[&definition_id];
        // Sprite/graphics zero is a real shared bank, not an absent identity.
        if source.id != requested
            || definition.id != definition_id
            || source.graphics_id != definition.graphics_id
        {
            return Err(FixedWalkError::Unsupported(
                "invalid or cross-bank inherited graphics identity",
            ));
        }
        if source.image_count == Some(0) || definition.image_count == Some(0) {
            return Err(FixedWalkError::Unsupported("zero image-count header"));
        }
        if animation.directionality != Directionality::EightWay
            || ![4, 8].contains(&animation.frames.len())
        {
            return Err(FixedWalkError::Unsupported(
                "only four/eight-source eight-way locomotion is proven",
            ));
        }
        if !(1..=1000).contains(&animation.frames_per_second) {
            return Err(FixedWalkError::Unsupported("FPS outside verified interval"));
        }
        if animation.label_bytes.len() > 256 * 1024 {
            return Err(FixedWalkError::Bound("animation label bytes"));
        }
        let mut durations = vec![Duration::ZERO; animation.frames.len()];
        for (index, frame) in animation.frames.iter().enumerate() {
            if frame.events.len() > 32 {
                return Err(FixedWalkError::Bound("per-source event count"));
            }
            // Player footsteps are preserved, not executed. Skip remains
            // qualified only for the original pet/NPC action0 entry; do not
            // widen the generic resolver or silently sanitize any frame.
            if frame.events.iter().any(|event| {
                !if player {
                    matches!(event, FrameEvent::Effect(478 | 479))
                } else {
                    matches!(event, FrameEvent::Skip)
                }
            }) {
                return Err(FixedWalkError::Unsupported(
                    "unproven fixed-count walk event",
                ));
            }
            let image_max = frame
                .image_id
                .checked_add(7)
                .ok_or(FixedWalkError::Bound("directional image overflow"))?;
            if frame.subframe >= 255 || format!("{}-{image_max}.spr", source.graphics_id).len() > 20
            {
                return Err(FixedWalkError::Bound("shared SPR identity"));
            }
            let millis = frame
                .duration_ticks
                .checked_mul(1000)
                .ok_or(FixedWalkError::Bound("native tick multiplication"))?
                / animation.frames_per_second;
            // Native parser loc43E8F3 stores and 4E8750 reads this as a WORD.
            // Do not emulate overflow/truncation as a usable animation.
            if millis == 0 || millis > u32::from(u16::MAX) {
                return Err(FixedWalkError::Unsupported(
                    "base duration outside nonzero native WORD",
                ));
            }
            durations[index] = Duration::from_millis(u64::from(millis));
        }
        let mut active_ends = vec![Duration::ZERO; animation.frames.len() - 1];
        let mut total = Duration::ZERO;
        for source in 1..animation.frames.len() {
            total = total
                .checked_add(durations[source])
                .ok_or(FixedWalkError::Bound("step duration overflow"))?;
            active_ends[source - 1] = total;
        }
        let completion = total
            .checked_add(durations[0])
            .ok_or(FixedWalkError::Bound("terminal duration overflow"))?;
        Ok(Self {
            provenance: WalkProvenance {
                requested_sprite_id: requested,
                definition_sprite_id: definition_id,
                graphics_id: source.graphics_id,
            },
            animation: animation.clone(),
            durations,
            active_ends,
            completion,
        })
    }

    pub fn provenance(&self) -> WalkProvenance {
        self.provenance
    }
    pub fn source_frames(&self) -> &[ActionFrame] {
        &self.animation.frames
    }
    pub fn frames_per_second(&self) -> u32 {
        self.animation.frames_per_second
    }
    pub fn total_source_ticks(&self) -> u64 {
        self.animation.total_ticks()
    }
    pub fn source_duration(&self, source: usize) -> Option<Duration> {
        self.durations.get(source).copied()
    }
    pub fn active_duration(&self) -> Duration {
        *self
            .active_ends
            .last()
            .expect("qualified nonempty active span")
    }
    pub fn step_duration(&self) -> Duration {
        self.completion
    }

    /// Asset lookup is allowed only through this certificate. It preserves all
    /// four/eight source frames and the original subframe and heading identities.
    pub fn frame_asset(&self, source: usize, heading: u8) -> Result<FrameAsset> {
        if heading > 7 {
            return Err(FixedWalkError::InvalidInput("heading"));
        }
        let frame = self
            .animation
            .frames
            .get(source)
            .ok_or(FixedWalkError::InvalidInput("source frame"))?;
        let image_id = frame
            .image_id
            .checked_add(u32::from(heading))
            .ok_or(FixedWalkError::Bound("directional image overflow"))?;
        Ok(FrameAsset {
            graphics_id: self.provenance.graphics_id,
            image_id,
            subframe: frame.subframe,
        })
    }

    /// CAMERA/attachment displacement only; body already has raw SPR offsets.
    pub fn local_camera_pulse(&self, source: usize, heading: u8) -> Result<(i32, i32)> {
        if source >= self.animation.frames.len() || heading > 7 {
            return Err(FixedWalkError::InvalidInput("pulse source/heading"));
        }
        Ok(if self.animation.frames.len() == 4 {
            FOUR[usize::from(heading)]
        } else if source & 1 == 0 {
            EVEN[usize::from(heading)]
        } else {
            ODD[usize::from(heading)]
        })
    }

    pub fn start_unpaced_step(&self, heading: u8) -> Result<WalkPlayback> {
        self.frame_asset(1, heading)?;
        Ok(WalkPlayback {
            walk: self.clone(),
            heading,
            last: None,
            next_pulse: 0,
            terminal_seen: false,
            retired: false,
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CameraPulse {
    pub source_frame: usize,
    pub elapsed: Duration,
    pub displacement: (i32, i32),
}

#[derive(Debug)]
pub struct WalkSample {
    pub source_frame: usize,
    pub asset: FrameAsset,
    /// True throughout source0 dwell and after completion. Not a server ACK.
    pub terminal: bool,
    /// One step ended; caller must explicitly create the next step.
    pub finished: bool,
    pub entered_terminal: bool,
    /// At most eight, delivered once. No audio/HP/cell mutation is performed.
    pub local_camera_pulses: Vec<CameraPulse>,
    pub next_deadline: Option<Duration>,
}

/// An owned bounded certificate clone avoids self-referential renderers. It
/// contains no SPR pixels, resource handles, world state or mutable catalog.
#[derive(Clone)]
pub struct WalkPlayback {
    walk: QualifiedFixedWalk,
    heading: u8,
    last: Option<Duration>,
    next_pulse: usize,
    terminal_seen: bool,
    retired: bool,
}

impl WalkPlayback {
    /// Cancellation permanently retires this cursor and emits nothing. A
    /// backwards call is rejected without consuming pulses; repeated elapsed
    /// calls return the same image but no duplicate pulse/terminal notification.
    /// Late calls consume at most this step's eight boundaries, never loop.
    pub fn advance(&mut self, elapsed: Duration, cancel: &AtomicBool) -> Result<WalkSample> {
        if self.retired || cancel.load(Ordering::Acquire) {
            self.retired = true;
            return Err(FixedWalkError::Cancelled);
        }
        if self.last.is_some_and(|prior| elapsed < prior) {
            return Err(FixedWalkError::TimeWentBackwards);
        }
        let source = self
            .walk
            .active_ends
            .iter()
            .position(|end| elapsed < *end)
            .map_or(0, |i| i + 1);
        let terminal = source == 0;
        let finished = elapsed >= self.walk.completion;
        let count = self.walk.animation.frames.len();
        let mut pulses = Vec::with_capacity(count - self.next_pulse);
        while self.next_pulse < count {
            let source_frame = self.next_pulse;
            let at = if source_frame == 0 {
                Duration::ZERO
            } else {
                self.walk.active_ends[source_frame - 1]
            };
            if at > elapsed {
                break;
            }
            pulses.push(CameraPulse {
                source_frame,
                elapsed: at,
                displacement: self.walk.local_camera_pulse(source_frame, self.heading)?,
            });
            self.next_pulse += 1;
        }
        let entered_terminal = terminal && !self.terminal_seen;
        self.terminal_seen |= terminal;
        self.last = Some(elapsed);
        Ok(WalkSample {
            source_frame: source,
            asset: self.walk.frame_asset(source, self.heading)?,
            terminal,
            finished,
            entered_terminal,
            local_camera_pulses: pulses,
            next_deadline: if finished {
                None
            } else if terminal {
                Some(self.walk.completion)
            } else {
                Some(self.walk.active_ends[source - 1])
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{animation::AnimationClock, sprite_actions::ActionLimits};

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; shared player locomotion qualification, no device/UI"]
    fn real_shared_player_walk_all_qualified_weapon_families_use_native_step_order() {
        use crate::modern_lito::{
            combat_animation::qualify_weapon_appearance,
            shared_action_catalog::SharedActionCatalog, world_scene::BaseAppearance,
        };
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root");
        let cancel = AtomicBool::new(false);
        let catalog = SharedActionCatalog::load(std::path::Path::new(&root), &cancel).unwrap();
        let mut families = 0;
        for class in 0..7 {
            for sex in 0..2 {
                let appearance = BaseAppearance { class, sex };
                for weapon in 0..=17 {
                    if qualify_weapon_appearance(appearance, weapon).is_err() {
                        continue;
                    }
                    let walk = QualifiedFixedWalk::from_player_catalog(
                        catalog.catalog(),
                        appearance,
                        weapon,
                    )
                    .unwrap_or_else(|e| panic!("{appearance:?} weapon{weapon}: {e}"));
                    for heading in 0..8 {
                        let mut cursor = walk.start_unpaced_step(heading).unwrap();
                        let first = cursor.advance(Duration::ZERO, &cancel).unwrap();
                        assert_eq!(first.source_frame, 1);
                        assert!(!first.terminal);
                        let last = cursor.advance(walk.active_duration(), &cancel).unwrap();
                        assert_eq!(last.source_frame, 0);
                        assert!(last.terminal && !last.finished);
                        assert!(
                            cursor
                                .advance(walk.step_duration(), &cancel)
                                .unwrap()
                                .finished
                        );
                    }
                    families += 1;
                }
            }
        }
        assert_eq!(families, 144);
        eprintln!("shared player walk: {families} class/sex/weapon families, eight headings; source1 -> terminal0; exact retained catalog");
    }

    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(
            b"#1 8 walk\n0.walk(1 8,0.0:2 0.1:2> 0.2:2 0.3:2> 0.4:2 0.5:2> 0.6:2 0.7:2>)\n",
            ActionLimits::default(),
        )
        .unwrap()
    }

    #[test]
    fn player_footsteps_are_retained_without_widening_pet_or_unknown_weapon_routes() {
        use crate::modern_lito::world_scene::BaseAppearance;
        let c = SpriteActionCatalog::parse_for_inspection(
            b"#0 64 player\n4.walk(1 8,0.0:2 0.1:2<478 0.2:2 0.3:2 0.4:2 0.5:2<479 0.6:2 0.7:2)\n",
            ActionLimits::default(),
        )
        .unwrap();
        let form = BaseAppearance { class: 0, sex: 0 };
        let walk = QualifiedFixedWalk::from_player_catalog(&c, form, 1).unwrap();
        assert_eq!(walk.source_frames()[1].events, [FrameEvent::Effect(478)]);
        assert_eq!(walk.source_frames()[5].events, [FrameEvent::Effect(479)]);
        assert!(QualifiedFixedWalk::from_catalog(&c, 0).is_err());
        for weapon in [0, 2, 8, 18, 255] {
            assert!(QualifiedFixedWalk::from_player_catalog(&c, form, weapon).is_err());
        }
        for event in [
            FrameEvent::Skip,
            FrameEvent::Effect(13),
            FrameEvent::Sound(478),
            FrameEvent::Hit,
            FrameEvent::Overlay(478),
        ] {
            let mut altered = c.clone();
            let ActionDefinition::Animation(a) = altered
                .sprites
                .get_mut(&0)
                .unwrap()
                .actions
                .get_mut(&ActionKey::new(4))
                .unwrap()
            else {
                panic!()
            };
            a.frames[1].events = vec![event];
            assert!(QualifiedFixedWalk::from_player_catalog(&altered, form, 1).is_err());
        }
    }
    fn animation(catalog: &mut SpriteActionCatalog) -> &mut Animation {
        match catalog
            .sprites
            .get_mut(&1)
            .unwrap()
            .actions
            .get_mut(&ActionKey::new(0))
            .unwrap()
        {
            ActionDefinition::Animation(animation) => animation,
            _ => panic!("fixture animation"),
        }
    }

    #[test]
    fn opaque_walk_preserves_skip_while_all_three_generic_paths_stay_strict() {
        let catalog = catalog();
        let qualified = QualifiedFixedWalk::from_catalog(&catalog, 1).unwrap();
        assert_eq!(qualified.source_frames().len(), 8);
        assert_eq!(
            qualified
                .source_frames()
                .iter()
                .filter(|f| f.events == [FrameEvent::Skip])
                .count(),
            4
        );
        assert!(matches!(
            catalog.resolve_action(1, ActionKey::new(0)),
            Err(ActionError {
                kind: ActionErrorKind::UninterpretedEvents(..),
                ..
            })
        ));
        let ActionDefinition::Animation(raw) = &catalog.sprites[&1].actions[&ActionKey::new(0)]
        else {
            panic!()
        };
        assert!(raw.frame_asset(1, 0, 0).is_err());
        assert!(AnimationClock::new(raw).is_err());
        for source in 0..8 {
            for heading in 0..8 {
                let asset = qualified.frame_asset(source, heading).unwrap();
                assert_eq!(asset.graphics_id, 1);
                assert_eq!(asset.image_id, u32::from(heading));
                assert_eq!(asset.subframe, source as u32);
            }
        }
        assert!(qualified.frame_asset(8, 0).is_err());
        assert!(qualified.frame_asset(0, 8).is_err());
        let zero = SpriteActionCatalog::parse_for_inspection(
            b"#0 8 walk\n0.walk(1 8,0.0:2 0.1:2> 0.2:2 0.3:2> 0.4:2 0.5:2> 0.6:2 0.7:2>)\n",
            ActionLimits::default(),
        )
        .unwrap();
        let zero = QualifiedFixedWalk::from_catalog(&zero, 0).unwrap();
        assert_eq!(
            zero.provenance(),
            WalkProvenance {
                requested_sprite_id: 0,
                definition_sprite_id: 0,
                graphics_id: 0
            }
        );
        assert_eq!(zero.source_frames(), qualified.source_frames());
        assert_eq!(
            zero.frame_asset(7, 7).unwrap(),
            FrameAsset {
                graphics_id: 0,
                image_id: 7,
                subframe: 7
            }
        );
    }

    #[test]
    fn source_one_first_terminal_zero_dwell_and_repeats_have_exact_boundaries() {
        let q = QualifiedFixedWalk::from_catalog(&catalog(), 1).unwrap();
        assert_eq!(q.frames_per_second(), 24);
        assert_eq!(q.total_source_ticks(), 16);
        assert_eq!(q.source_duration(0), Some(Duration::from_millis(83)));
        assert_eq!(q.active_duration(), Duration::from_millis(581));
        assert_eq!(q.step_duration(), Duration::from_millis(664));
        let cancel = AtomicBool::new(false);
        let mut cursor = q.start_unpaced_step(0).unwrap();
        let first = cursor.advance(Duration::ZERO, &cancel).unwrap();
        assert_eq!(first.source_frame, 1);
        assert!(!first.terminal && !first.finished && !first.entered_terminal);
        assert_eq!(
            first.local_camera_pulses,
            [CameraPulse {
                source_frame: 0,
                elapsed: Duration::ZERO,
                displacement: (-3, -1)
            }]
        );
        assert!(cursor
            .advance(Duration::ZERO, &cancel)
            .unwrap()
            .local_camera_pulses
            .is_empty());
        for source in 1..8 {
            let before = Duration::from_millis(source as u64 * 83) - Duration::from_nanos(1);
            assert_eq!(
                cursor.advance(before, &cancel).unwrap().source_frame,
                source
            );
            let at = cursor
                .advance(Duration::from_millis(source as u64 * 83), &cancel)
                .unwrap();
            assert_eq!(at.source_frame, if source == 7 { 0 } else { source + 1 });
            assert_eq!(at.local_camera_pulses.len(), 1);
            assert_eq!(at.local_camera_pulses[0].source_frame, source);
            assert_eq!(at.entered_terminal, source == 7);
            assert!(!at.finished);
        }
        let dwell = cursor
            .advance(
                Duration::from_millis(664) - Duration::from_nanos(1),
                &cancel,
            )
            .unwrap();
        assert!(dwell.terminal && !dwell.finished && !dwell.entered_terminal);
        let done = cursor.advance(Duration::from_millis(664), &cancel).unwrap();
        assert!(done.finished && done.terminal && done.local_camera_pulses.is_empty());
        assert_eq!(done.next_deadline, None);
        let late = cursor.advance(Duration::MAX, &cancel).unwrap();
        assert!(late.finished && !late.entered_terminal && late.local_camera_pulses.is_empty());
    }

    #[test]
    fn all_heading_camera_pulses_sum_one_projection_step_but_do_not_change_assets() {
        let q = QualifiedFixedWalk::from_catalog(&catalog(), 1).unwrap();
        let expected = [
            (-24, -12),
            (0, -24),
            (24, -12),
            (48, 0),
            (24, 12),
            (0, 24),
            (-24, 12),
            (-48, 0),
        ];
        for heading in 0..8 {
            let before = q.frame_asset(3, heading).unwrap();
            let pulses = q
                .start_unpaced_step(heading)
                .unwrap()
                .advance(Duration::MAX, &AtomicBool::new(false))
                .unwrap();
            let sum = pulses.local_camera_pulses.iter().fold((0, 0), |(x, y), p| {
                (x + p.displacement.0, y + p.displacement.1)
            });
            assert_eq!(pulses.local_camera_pulses.len(), 8);
            assert_eq!(sum, expected[usize::from(heading)]);
            assert_eq!(q.frame_asset(3, heading).unwrap(), before);
        }
    }

    #[test]
    fn four_source_walk_has_three_active_callbacks_and_independent_terminal_dwell() {
        let c = SpriteActionCatalog::parse_for_inspection(
            b"#938 48 beagle\n0.walk(1 4,0.0:5 0.1:4> 0.2:5 0.3:4>)\n",
            ActionLimits::default(),
        )
        .unwrap();
        let q = QualifiedFixedWalk::from_catalog(&c, 938).unwrap();
        assert_eq!(q.source_frames().len(), 4);
        assert_eq!(q.active_duration(), Duration::from_millis(540));
        assert_eq!(q.step_duration(), Duration::from_millis(748));
        let cancel = AtomicBool::new(false);
        let mut p = q.start_unpaced_step(0).unwrap();
        let first = p.advance(Duration::ZERO, &cancel).unwrap();
        assert_eq!(
            (
                first.source_frame,
                first.local_camera_pulses[0].displacement
            ),
            (1, (-6, -3))
        );
        for (time, source) in [(166, 2), (374, 3), (540, 0)] {
            let sample = p.advance(Duration::from_millis(time), &cancel).unwrap();
            assert_eq!(sample.source_frame, source);
            assert_eq!(sample.local_camera_pulses.len(), 1);
            assert_eq!(sample.local_camera_pulses[0].displacement, (-6, -3));
            assert!(!sample.finished);
        }
        assert!(
            !p.advance(
                Duration::from_millis(748) - Duration::from_nanos(1),
                &cancel
            )
            .unwrap()
            .finished
        );
        assert!(
            p.advance(Duration::from_millis(748), &cancel)
                .unwrap()
                .finished
        );
        let sums = [
            (-24, -12),
            (0, -24),
            (24, -12),
            (48, 0),
            (24, 12),
            (0, 24),
            (-24, 12),
            (-48, 0),
        ];
        for heading in 0..8 {
            let sample = q
                .start_unpaced_step(heading)
                .unwrap()
                .advance(Duration::MAX, &cancel)
                .unwrap();
            assert_eq!(sample.local_camera_pulses.len(), 4);
            assert_eq!(
                sample.local_camera_pulses.iter().fold((0, 0), |(x, y), p| (
                    x + p.displacement.0,
                    y + p.displacement.1
                )),
                sums[usize::from(heading)]
            );
        }
        assert!(
            c.resolve_action(938, ActionKey::new(0)).is_err(),
            "four-count Skip does not weaken generic resolver"
        );
    }

    #[test]
    fn wrong_context_event_duration_and_image_overflow_are_rejected() {
        for count in [3, 5, 6, 7, 9, 12] {
            let mut c = catalog();
            let a = animation(&mut c);
            a.frames.resize(count, a.frames[0].clone());
            assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        }
        for event in [
            FrameEvent::Hit,
            FrameEvent::Sound(13),
            FrameEvent::Effect(13),
            FrameEvent::Overlay(2),
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value: 13,
            },
        ] {
            let mut c = catalog();
            animation(&mut c).frames[1].events = vec![event];
            assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        }
        let mut c = catalog();
        animation(&mut c).directionality = Directionality::Undirected;
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        let mut c = catalog();
        animation(&mut c).frames[0].duration_ticks = u32::MAX;
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 1),
            Err(FixedWalkError::Bound(_))
        ));
        for fps in [0, 1001, u32::MAX] {
            let mut c = catalog();
            animation(&mut c).frames_per_second = fps;
            assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        }
        let mut c = catalog();
        animation(&mut c).frames[0].duration_ticks = 0;
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        let mut c = catalog();
        animation(&mut c).frames_per_second = 1000;
        animation(&mut c).frames[0].duration_ticks = u32::from(u16::MAX);
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_ok());
        animation(&mut c).frames[0].duration_ticks = u32::from(u16::MAX) + 1;
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 1),
            Err(FixedWalkError::Unsupported(_))
        ));
        let mut c = catalog();
        animation(&mut c).frames[0].image_id = u32::MAX;
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        let mut c = catalog();
        let walk = c
            .sprites
            .get_mut(&1)
            .unwrap()
            .actions
            .remove(&ActionKey::new(0))
            .unwrap();
        c.sprites
            .get_mut(&1)
            .unwrap()
            .actions
            .insert(ActionKey::new(1), walk);
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
    }

    #[test]
    fn header_quarantine_cycles_cross_graphics_and_resolver_depth_are_not_bypassed() {
        let mut c = catalog();
        c.sprites.get_mut(&1).unwrap().header_issue = Some(ActionError {
            line: 1,
            kind: ActionErrorKind::Malformed("fixture"),
        });
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 1),
            Err(FixedWalkError::Catalog(_))
        ));
        let mut c = catalog();
        c.sprites.get_mut(&1).unwrap().actions.insert(
            ActionKey::new(104),
            ActionDefinition::Quarantined {
                raw_bytes: vec![],
                reason: ActionError {
                    line: 1,
                    kind: ActionErrorKind::Malformed("fixture"),
                },
            },
        );
        assert!(QualifiedFixedWalk::from_catalog(&c, 1).is_err());
        let mut c = catalog();
        let mut alias = c.sprites[&1].clone();
        alias.id = 2;
        alias.graphics_id = 2;
        alias.actions.insert(
            ActionKey::new(0),
            ActionDefinition::Reference {
                sprite_id: 1,
                label_bytes: vec![],
            },
        );
        c.sprites.insert(2, alias);
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 2),
            Err(FixedWalkError::Unsupported(_))
        ));
        c.sprites.get_mut(&2).unwrap().graphics_id = 1;
        assert_eq!(
            QualifiedFixedWalk::from_catalog(&c, 2)
                .unwrap()
                .provenance(),
            WalkProvenance {
                requested_sprite_id: 2,
                definition_sprite_id: 1,
                graphics_id: 1,
            }
        );
        c.sprites.get_mut(&1).unwrap().actions.insert(
            ActionKey::new(0),
            ActionDefinition::Reference {
                sprite_id: 2,
                label_bytes: vec![],
            },
        );
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 2),
            Err(FixedWalkError::Catalog(ActionError {
                kind: ActionErrorKind::CyclicReference(..),
                ..
            }))
        ));
        let mut c = catalog();
        for id in 2..=67 {
            let mut alias = c.sprites[&1].clone();
            alias.id = id;
            alias.actions.insert(
                ActionKey::new(0),
                ActionDefinition::Reference {
                    sprite_id: id - 1,
                    label_bytes: vec![],
                },
            );
            c.sprites.insert(id, alias);
        }
        assert!(matches!(
            QualifiedFixedWalk::from_catalog(&c, 67),
            Err(FixedWalkError::Catalog(ActionError {
                kind: ActionErrorKind::Limit("reference depth"),
                ..
            }))
        ));
    }

    #[test]
    fn cancellation_retires_cursor_and_backwards_call_does_not_consume_future_pulses() {
        let q = QualifiedFixedWalk::from_catalog(&catalog(), 1).unwrap();
        let cancel = AtomicBool::new(false);
        let mut p = q.start_unpaced_step(2).unwrap();
        p.advance(Duration::from_millis(83), &cancel).unwrap();
        assert!(matches!(
            p.advance(Duration::ZERO, &cancel),
            Err(FixedWalkError::TimeWentBackwards)
        ));
        assert!(p
            .advance(Duration::from_millis(83), &cancel)
            .unwrap()
            .local_camera_pulses
            .is_empty());
        assert_eq!(
            p.advance(Duration::from_millis(166), &cancel)
                .unwrap()
                .local_camera_pulses[0]
                .source_frame,
            2
        );
        cancel.store(true, Ordering::Release);
        assert!(matches!(
            p.advance(Duration::MAX, &cancel),
            Err(FixedWalkError::Cancelled)
        ));
        cancel.store(false, Ordering::Release);
        assert!(matches!(
            p.advance(Duration::MAX, &cancel),
            Err(FixedWalkError::Cancelled)
        ));
        let mut fresh = q.start_unpaced_step(2).unwrap();
        assert_eq!(
            fresh
                .advance(Duration::ZERO, &cancel)
                .unwrap()
                .local_camera_pulses
                .len(),
            1
        );
    }
}
