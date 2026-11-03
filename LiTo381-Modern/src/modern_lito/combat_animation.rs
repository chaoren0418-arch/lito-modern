//! Qualified shared weapon-stance one-shots, not a combat authority or player.
//!
//! Native evidence: sub_4E45F0/loc_4E490D initializes actor+13h to zero and
//! calls sub_4E5B50, which increments BEFORE loading. It plays source indices
//! 1..count-1, then loads index 0 and returns false. sub_4E34C0 stops scheduling
//! that one-shot on false; existing death action 8 is guarded against revival by
//! a later action. Index 0 is a rest/terminal image, not the initial dwell.
//!
//! Important source-name correction: FrameEvent::Effect is the existing parser's
//! name for '<'. Native sub_43DE10 stores '<', '[' and '^' in the SAME audio
//! candidate bank with modes 0/1/2. sub_4E5C00 selects one candidate; its callees
//! resolve sound/<id>.wav, NOT a same-number sprite. We preserve that original
//! enum verbatim and provide a separate, pure classification; we do not play it.
//! Neither Hit nor completion may change HP, death, rewards, or server state.

use std::{sync::Arc, time::Duration};

use anyhow::{ensure, Context, Result};

use super::{
    animation::{AnimationClock, PlaybackLimits},
    sprite_actions::{
        ActionKey, Animation, Directionality, FrameAsset, FrameEvent, SpriteActionCatalog,
    },
    world_scene::BaseAppearance,
};

pub const MAX_FRAMES: usize = 256;
pub const MAX_EVENTS: usize = 1024;
const NANOS_PER_SECOND: u128 = 1_000_000_000;

/// Retained catalog `weaponType`, NOT a native action byte. The two domains
/// have different values: dagger type2 selects action base46, for example.
/// Mapping evidence is ItemTable._weaponTypes/_weaponId and native actor+18h;
/// sub_4E0260 uses base, sub_4E02C0 uses base+3, sub_4E45F0 uses +1/+2/8.
/// This establishes presentation only, never item/class permission or damage.
pub fn weapon_stance_base(weapon_type: u8) -> Result<u32> {
    Ok(match weapon_type {
        0 => 0,
        1 => 4,
        2 => 46,
        3 => 50,
        4 | 13 => 20,
        5 | 14 => 24,
        6 | 15 => 11,
        7 | 16 => 40,
        10 => 62,
        11 | 17 => 58,
        12 => 54,
        // Throwingknife2922 is not a stance byte; arrow66 is ammunition.
        // Chain83..86 is absent from all 14 shared base characters, and is
        // beyond the original LiTo action table. No spear/unarmed fallback.
        _ => anyhow::bail!("unqualified shared weapon type {weapon_type}"),
    })
}

/// Complete idle/walk/attack/damage/death families observed in the fixed shared
/// list.spr (SHA256 523c61ad...f127). This allowlist prevents an unrelated newly
/// injected action from silently qualifying an unseen class/weapon combination.
/// Actual catalog frames, controls and assets are still validated when loaded.
pub fn qualify_weapon_appearance(appearance: BaseAppearance, weapon_type: u8) -> Result<u32> {
    appearance.logical_sprite_id()?;
    let base = weapon_stance_base(weapon_type)?;
    let allowed: &[u32] = match appearance.class {
        0 | 1 => &[0, 4, 11, 20, 24, 40, 46, 50],
        2 | 3 => &[0, 4, 11, 20, 24, 40, 46],
        4 => &[0, 4, 20, 46, 54, 58, 62],
        5 => &[0, 4, 11, 24, 50, 62],
        6 => &[0, 11, 20, 40, 58],
        _ => unreachable!(),
    };
    ensure!(
        allowed.contains(&base),
        "unqualified class/weapon shared stance"
    );
    Ok(base)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CombatAction {
    Attack,
    Damage,
    Death,
}
impl CombatAction {
    pub fn action_key(self) -> ActionKey {
        ActionKey::new(match self {
            Self::Attack => 1,
            Self::Damage => 2,
            Self::Death => 8,
        })
    }
    pub fn action_key_armed(
        self,
        appearance: BaseAppearance,
        weapon_type: u8,
    ) -> Result<ActionKey> {
        let base = qualify_weapon_appearance(appearance, weapon_type)?;
        Ok(ActionKey::new(match self {
            Self::Attack => base + 1,
            Self::Damage => base + 2,
            Self::Death => 8,
        }))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeAudioMode {
    /// '<': reuse an idle same-ID voice or allocate another if it is playing.
    AllowOverlappingVoices,
    /// '[': do not restart the already-playing same-ID voice.
    SuppressWhileSameIdPlaying,
    /// '^': native mode 2 receives distance and uses a separate voice policy.
    /// Classification is proven; its distance/admission policy is NOT ported.
    LegacyCaretDistanceMode,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeAudioGroup {
    pub mode: NativeAudioMode,
    /// Ordered, weighted alternatives, NOT simultaneous sounds. Duplicate IDs
    /// are meaningful and retained. Zero means a silent selection in the BIN.
    /// Shared-only: >=10000 is a different LT namespace and is rejected here.
    candidates: Vec<u32>,
}
impl NativeAudioGroup {
    pub fn candidates(&self) -> &[u32] {
        &self.candidates
    }
    /// Pure selection with caller-supplied random bits, no RNG/audio side effect.
    /// Native selects one candidate with an unsigned remainder by bank count.
    pub fn selected(&self, random_word: u32) -> Option<u32> {
        let id = self.candidates[(random_word as usize) % self.candidates.len()];
        (id != 0).then_some(id)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeCueClassification {
    pub audio: Option<NativeAudioGroup>,
    /// Hit/overlay/skip are preserved, never executed or treated as audio.
    pub non_audio: Vec<FrameEvent>,
}

/// Strictly classifies the observed native modifier order. Does not alter the
/// parser, discard extra candidate IDs, resolve visual assets or play anything.
pub fn classify_native_cues(events: &[FrameEvent]) -> Result<NativeCueClassification> {
    ensure!(events.len() <= MAX_EVENTS, "combat cue count exceeds bound");
    let mut mode = None;
    let mut candidates = Vec::new();
    let mut non_audio = Vec::new();
    let mut ended_audio = false;
    for event in events {
        let audio = match event {
            FrameEvent::Effect(id) => Some((NativeAudioMode::AllowOverlappingVoices, *id)),
            FrameEvent::Sound(id) => Some((NativeAudioMode::SuppressWhileSameIdPlaying, *id)),
            FrameEvent::UninterpretedNumeric {
                marker: b'^',
                value,
            } => Some((NativeAudioMode::LegacyCaretDistanceMode, *value)),
            FrameEvent::UninterpretedNumeric { .. } => anyhow::bail!("unknown native cue marker"),
            _ => None,
        };
        if let Some((next_mode, id)) = audio {
            ensure!(
                !ended_audio && mode.is_none_or(|m| m == next_mode),
                "mixed or misplaced native audio bank"
            );
            ensure!(
                id < 10_000,
                "LT or unknown audio namespace is not shared audio"
            );
            ensure!(
                candidates.len() < 3,
                "native audio bank exceeds three candidates"
            );
            mode = Some(next_mode);
            candidates.push(id);
        } else {
            ended_audio = true;
            non_audio.push(event.clone());
        }
    }
    Ok(NativeCueClassification {
        audio: mode.map(|mode| NativeAudioGroup { mode, candidates }),
        non_audio,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatProvenance {
    pub appearance: BaseAppearance,
    pub weapon_type: u8,
    pub action: CombatAction,
    pub action_key: ActionKey,
    pub logical_sprite_id: u32,
    pub definition_sprite_id: u32,
    pub graphics_id: u32,
}

struct Qualified {
    provenance: CombatProvenance,
    animation: Animation,
    /// Scheduling order is source indices 1..N-1 followed by terminal 0.
    starts: Vec<u64>,
    ends: Vec<u64>,
    total_ticks: u64,
}

#[derive(Clone)]
pub struct QualifiedCombatAnimation(Arc<Qualified>);
impl QualifiedCombatAnimation {
    /// No I/O: the caller owns shared Text/list.spr acquisition. Logical IDs
    /// select exact actions; this compatibility entry point is explicitly
    /// unarmed. Use from_catalog_armed for a confirmed retained weapon type.
    pub fn from_catalog(
        catalog: &SpriteActionCatalog,
        appearance: BaseAppearance,
        action: CombatAction,
    ) -> Result<Self> {
        Self::from_catalog_armed(catalog, appearance, 0, action)
    }

    /// Exact, qualified main weapon action. Native random extra attack variants
    /// are not guessed, and a missing armed definition never resolves unarmed.
    pub fn from_catalog_armed(
        catalog: &SpriteActionCatalog,
        appearance: BaseAppearance,
        weapon_type: u8,
        action: CombatAction,
    ) -> Result<Self> {
        let logical = appearance.logical_sprite_id()?;
        let action_key = action.action_key_armed(appearance, weapon_type)?;
        let resolved = catalog.resolve_action(logical, action_key)?;
        let gfx = catalog
            .sprites
            .get(&logical)
            .context("missing logical sprite")?
            .graphics_id;
        ensure!(
            catalog
                .sprites
                .get(&resolved.definition_sprite_id)
                .context("missing action definition")?
                .graphics_id
                == gfx,
            "unverified cross-graphics combat inheritance"
        );
        Self::qualify(
            CombatProvenance {
                appearance,
                weapon_type,
                action,
                action_key,
                logical_sprite_id: logical,
                definition_sprite_id: resolved.definition_sprite_id,
                graphics_id: gfx,
            },
            resolved.animation,
        )
    }

    fn qualify(provenance: CombatProvenance, animation: &Animation) -> Result<Self> {
        ensure!(
            animation.directionality == Directionality::EightWay,
            "unverified combat stance directionality"
        );
        ensure!(
            animation.frames.len() >= 2,
            "one-shot requires terminal and active frames"
        );
        // Reuse established validation before allocating an independent one-shot
        // schedule. This keeps '^'/Skip quarantined despite audio classification.
        let verified = AnimationClock::with_limits(
            animation,
            PlaybackLimits {
                max_frames: MAX_FRAMES,
                max_events: MAX_EVENTS,
            },
        )?;
        let animation = verified.animation();
        for frame in &animation.frames {
            ensure!(
                frame.image_id.checked_add(7).is_some(),
                "combat directional image overflow"
            );
            classify_native_cues(&frame.events)?;
        }
        let mut starts = Vec::new();
        let mut ends = Vec::new();
        starts.try_reserve_exact(animation.frames.len())?;
        ends.try_reserve_exact(animation.frames.len())?;
        let mut total_ticks = 0u64;
        for frame in &animation.frames[1..] {
            starts.push(total_ticks);
            total_ticks = total_ticks
                .checked_add(u64::from(frame.duration_ticks))
                .context("combat duration overflow")?;
            ends.push(total_ticks);
        }
        ensure!(total_ticks > 0, "zero-duration one-shot is not qualified");
        // The terminal frame's declared duration is NOT another active dwell.
        // Its cues, if supplied by a future qualified source, occur once on entry.
        starts.push(total_ticks);
        ends.push(total_ticks);
        Ok(Self(Arc::new(Qualified {
            provenance,
            animation: animation.clone(),
            starts,
            ends,
            total_ticks,
        })))
    }
    pub fn provenance(&self) -> &CombatProvenance {
        &self.0.provenance
    }
    pub fn animation(&self) -> &Animation {
        &self.0.animation
    }
    pub fn total_ticks(&self) -> u64 {
        self.0.total_ticks
    }
    pub fn duration(&self) -> Result<Duration> {
        deadline(self.0.total_ticks, self.0.animation.frames_per_second)
    }
    pub fn start(&self) -> CombatPlayback {
        CombatPlayback {
            animation: self.clone(),
            next_event_frame: 0,
            last_elapsed: None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CrossedCue {
    pub source_frame: usize,
    pub event_index: usize,
    /// Tick from active one-shot start, excluding the terminal index-0 duration.
    pub tick: u64,
    pub event: FrameEvent,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CombatSample {
    pub source_frame: usize,
    pub asset: FrameAsset,
    /// End means hold terminal index0. Authority/caller decides any later action.
    pub finished: bool,
    pub crossed_cues: Vec<CrossedCue>,
    /// Absolute playback elapsed deadline, rounded up to nanoseconds.
    pub next_deadline: Option<Duration>,
}

pub struct CombatPlayback {
    animation: QualifiedCombatAnimation,
    next_event_frame: usize,
    last_elapsed: Option<Duration>,
}
impl CombatPlayback {
    pub fn advance(&mut self, elapsed: Duration, heading: u8) -> Result<CombatSample> {
        ensure!(heading < 8, "combat heading outside 0..7");
        ensure!(
            self.last_elapsed.is_none_or(|old| elapsed >= old),
            "combat playback time went backwards"
        );
        let data = &self.animation.0;
        let scaled = elapsed
            .as_nanos()
            .checked_mul(u128::from(data.animation.frames_per_second))
            .context("combat elapsed overflow")?;
        let tick = (scaled / NANOS_PER_SECOND).min(u128::from(data.total_ticks)) as u64;
        let finished = tick == data.total_ticks;
        let position = data.ends.partition_point(|end| *end <= tick);
        let source_frame = if finished { 0 } else { position + 1 };
        let asset =
            data.animation
                .frame_asset(data.provenance.graphics_id, source_frame, heading)?;
        let pending_end = data.starts.partition_point(|start| *start <= tick);
        let source_index = |position: usize| {
            if position + 1 == data.starts.len() {
                0
            } else {
                position + 1
            }
        };
        let mut count = 0usize;
        for position in self.next_event_frame..pending_end {
            count = count
                .checked_add(data.animation.frames[source_index(position)].events.len())
                .context("combat event count overflow")?;
        }
        ensure!(count <= MAX_EVENTS, "combat event delivery exceeds bound");
        let mut crossed_cues = Vec::new();
        crossed_cues.try_reserve_exact(count)?;
        for position in self.next_event_frame..pending_end {
            let source_frame = source_index(position);
            for (event_index, event) in data.animation.frames[source_frame]
                .events
                .iter()
                .enumerate()
            {
                crossed_cues.push(CrossedCue {
                    source_frame,
                    event_index,
                    tick: data.starts[position],
                    event: event.clone(),
                });
            }
        }
        let next_deadline = if finished {
            None
        } else {
            Some(deadline(
                data.ends[position],
                data.animation.frames_per_second,
            )?)
        };
        let sample = CombatSample {
            source_frame,
            asset,
            finished,
            crossed_cues,
            next_deadline,
        };
        // Error paths do not consume cues or partially advance the one-shot.
        self.next_event_frame = pending_end;
        self.last_elapsed = Some(elapsed);
        Ok(sample)
    }
}
fn deadline(tick: u64, fps: u32) -> Result<Duration> {
    let value = u128::from(tick)
        .checked_mul(NANOS_PER_SECOND)
        .context("combat deadline overflow")?;
    let nanos = value
        .checked_add(u128::from(fps) - 1)
        .context("combat deadline overflow")?
        / u128::from(fps);
    Ok(Duration::new(
        u64::try_from(nanos / NANOS_PER_SECOND)?,
        (nanos % NANOS_PER_SECOND) as u32,
    ))
}

#[cfg(test)]
mod tests {
    use super::super::{
        audio::parse_pcm_wav,
        resources::{
            read_shared_sound_with_limit, ResourceError, SharedArchive, SharedArchiveKind,
            SharedSoundKey,
        },
        sprite::{decode_classic_spr_with_limits, SpriteLimits},
        sprite_actions::{ActionFrame, ActionLimits},
    };
    use super::*;
    use sha2::{Digest, Sha256};
    use std::{
        collections::{BTreeMap, BTreeSet},
        path::PathBuf,
    };

    fn animation() -> Animation {
        Animation {
            label_bytes: b"fixture".to_vec(),
            directionality: Directionality::EightWay,
            frames_per_second: 10,
            frames: vec![
                ActionFrame {
                    image_id: 96,
                    subframe: 9,
                    duration_ticks: 77,
                    events: vec![FrameEvent::Sound(9)],
                },
                ActionFrame {
                    image_id: 96,
                    subframe: 0,
                    duration_ticks: 2,
                    events: vec![FrameEvent::Sound(1)],
                },
                ActionFrame {
                    image_id: 96,
                    subframe: 1,
                    duration_ticks: 0,
                    events: vec![FrameEvent::Effect(2), FrameEvent::Effect(3)],
                },
                ActionFrame {
                    image_id: 96,
                    subframe: 2,
                    duration_ticks: 3,
                    events: vec![FrameEvent::Hit],
                },
            ],
        }
    }
    fn qualified(action: CombatAction, animation: &Animation) -> QualifiedCombatAnimation {
        QualifiedCombatAnimation::qualify(
            CombatProvenance {
                appearance: BaseAppearance { class: 1, sex: 0 },
                weapon_type: 0,
                action,
                action_key: action.action_key(),
                logical_sprite_id: 61,
                definition_sprite_id: 61,
                graphics_id: 3213,
            },
            animation,
        )
        .unwrap()
    }

    #[test]
    fn source_index_zero_is_terminal_not_initial_dwell_and_death_does_not_loop() {
        for action in [
            CombatAction::Attack,
            CombatAction::Damage,
            CombatAction::Death,
        ] {
            let source = qualified(action, &animation());
            assert_eq!(source.total_ticks(), 5);
            assert_eq!(source.duration().unwrap(), Duration::from_millis(500));
            let mut playback = source.start();
            let start = playback.advance(Duration::ZERO, 3).unwrap();
            assert_eq!(start.source_frame, 1);
            assert_eq!(
                start.asset,
                FrameAsset {
                    graphics_id: 3213,
                    image_id: 99,
                    subframe: 0
                }
            );
            assert_eq!(
                start.crossed_cues,
                [CrossedCue {
                    source_frame: 1,
                    event_index: 0,
                    tick: 0,
                    event: FrameEvent::Sound(1)
                }]
            );
            assert_eq!(start.next_deadline, Some(Duration::from_millis(200)));
            let middle = playback.advance(Duration::from_millis(200), 3).unwrap();
            assert_eq!(middle.source_frame, 3);
            assert_eq!(
                middle
                    .crossed_cues
                    .iter()
                    .map(|c| (&c.event, c.tick, c.source_frame))
                    .collect::<Vec<_>>(),
                vec![
                    (&FrameEvent::Effect(2), 2, 2),
                    (&FrameEvent::Effect(3), 2, 2),
                    (&FrameEvent::Hit, 2, 3)
                ]
            );
            let end = playback.advance(Duration::from_millis(500), 3).unwrap();
            assert!(end.finished);
            assert_eq!(end.source_frame, 0);
            assert_eq!(end.asset.subframe, 9);
            assert_eq!(end.next_deadline, None);
            assert_eq!(
                end.crossed_cues[0],
                CrossedCue {
                    source_frame: 0,
                    event_index: 0,
                    tick: 5,
                    event: FrameEvent::Sound(9)
                }
            );
            let held = playback.advance(Duration::MAX, 7).unwrap();
            assert!(held.finished);
            assert_eq!(held.source_frame, 0);
            assert_eq!(held.asset.image_id, 103);
            assert!(held.crossed_cues.is_empty());
        }
    }

    #[test]
    fn confirmed_pace_changes_keep_one_shot_source_phase_and_cues_exactly_once() {
        use std::time::Instant;
        let t = Instant::now();
        let mut phase = super::super::paced_animation::PacedClock::new(t);
        let mut playback = qualified(CombatAction::Attack, &animation()).start();
        assert_eq!(
            playback
                .advance(phase.elapsed(t).unwrap(), 0)
                .unwrap()
                .crossed_cues
                .len(),
            1
        );
        phase
            .retime(t + Duration::from_millis(100), 1000, 500)
            .unwrap();
        let fast = phase.elapsed(t + Duration::from_millis(150)).unwrap();
        assert_eq!(fast, Duration::from_millis(200));
        assert_eq!(playback.advance(fast, 0).unwrap().crossed_cues.len(), 3);
        phase
            .retime(t + Duration::from_millis(150), 1000, 1000)
            .unwrap();
        assert!(playback
            .advance(phase.elapsed(t + Duration::from_millis(150)).unwrap(), 0)
            .unwrap()
            .crossed_cues
            .is_empty());
        let end = playback
            .advance(phase.elapsed(t + Duration::from_millis(450)).unwrap(), 0)
            .unwrap();
        assert!(end.finished);
        assert_eq!(end.source_frame, 0);
        assert_eq!(end.crossed_cues.len(), 1);
        assert!(playback
            .advance(phase.elapsed(t + Duration::from_secs(5)).unwrap(), 0)
            .unwrap()
            .crossed_cues
            .is_empty());
    }

    #[test]
    fn initial_zero_tick_cues_cross_once_in_source_order_and_large_jump_is_bounded() {
        let mut source = animation();
        source.frames[1].duration_ticks = 0;
        let qualified = qualified(CombatAction::Damage, &source);
        let mut clock = qualified.start();
        let first = clock.advance(Duration::ZERO, 0).unwrap();
        assert_eq!(first.source_frame, 3);
        assert_eq!(first.crossed_cues.len(), 4);
        assert!(first.crossed_cues.iter().all(|c| c.tick == 0));
        assert!(clock
            .advance(Duration::ZERO, 0)
            .unwrap()
            .crossed_cues
            .is_empty());
        let mut late = qualified.start();
        let all = late.advance(Duration::MAX, 0).unwrap();
        assert_eq!(all.crossed_cues.len(), 5);
        assert_eq!(all.crossed_cues.last().unwrap().source_frame, 0);
        assert!(late
            .advance(Duration::MAX, 0)
            .unwrap()
            .crossed_cues
            .is_empty());
    }

    #[test]
    fn time_reversal_and_invalid_heading_do_not_consume_pending_cues() {
        let mut playback = qualified(CombatAction::Attack, &animation()).start();
        assert!(playback.advance(Duration::ZERO, 8).is_err());
        assert_eq!(
            playback
                .advance(Duration::ZERO, 0)
                .unwrap()
                .crossed_cues
                .len(),
            1
        );
        assert!(playback.advance(Duration::from_millis(200), 9).is_err());
        assert_eq!(
            playback
                .advance(Duration::from_millis(200), 0)
                .unwrap()
                .crossed_cues
                .len(),
            3
        );
        assert!(playback.advance(Duration::from_millis(199), 0).is_err());
        assert_eq!(
            playback
                .advance(Duration::from_millis(500), 0)
                .unwrap()
                .crossed_cues
                .len(),
            1
        );
    }

    #[test]
    fn rational_tick_deadlines_round_up_without_early_boundary() {
        let mut source = animation();
        source.frames_per_second = 3;
        source.frames[1].duration_ticks = 1;
        let mut clock = qualified(CombatAction::Death, &source).start();
        assert_eq!(
            clock.advance(Duration::ZERO, 0).unwrap().next_deadline,
            Some(Duration::from_nanos(333_333_334))
        );
        assert_eq!(
            clock
                .advance(Duration::from_nanos(333_333_333), 0)
                .unwrap()
                .source_frame,
            1
        );
        assert_eq!(
            clock
                .advance(Duration::from_nanos(333_333_334), 0)
                .unwrap()
                .source_frame,
            3
        );
    }

    #[test]
    fn native_audio_candidates_preserve_marker_modes_probability_and_other_cues() {
        let parsed = classify_native_cues(&[
            FrameEvent::Effect(1340),
            FrameEvent::Effect(1340),
            FrameEvent::Effect(4774),
            FrameEvent::Overlay(17),
            FrameEvent::Hit,
        ])
        .unwrap();
        let group = parsed.audio.unwrap();
        assert_eq!(group.mode, NativeAudioMode::AllowOverlappingVoices);
        assert_eq!(group.candidates(), [1340, 1340, 4774]);
        assert_eq!(
            [
                group.selected(0),
                group.selected(1),
                group.selected(2),
                group.selected(3)
            ],
            [Some(1340), Some(1340), Some(4774), Some(1340)]
        );
        assert_eq!(parsed.non_audio, [FrameEvent::Overlay(17), FrameEvent::Hit]);
        let group = classify_native_cues(&[FrameEvent::Sound(0), FrameEvent::Sound(1539)])
            .unwrap()
            .audio
            .unwrap();
        assert_eq!(group.mode, NativeAudioMode::SuppressWhileSameIdPlaying);
        assert_eq!(group.selected(0), None);
        let group = classify_native_cues(&[FrameEvent::UninterpretedNumeric {
            marker: b'^',
            value: 7,
        }])
        .unwrap()
        .audio
        .unwrap();
        assert_eq!(group.mode, NativeAudioMode::LegacyCaretDistanceMode);
        for invalid in [
            vec![FrameEvent::Sound(1), FrameEvent::Effect(2)],
            vec![FrameEvent::Hit, FrameEvent::Sound(1)],
            vec![FrameEvent::Effect(10_000)],
            vec![FrameEvent::Effect(1); 4],
        ] {
            assert!(classify_native_cues(&invalid).is_err());
        }
    }

    #[test]
    fn qualifying_rejects_unknown_control_flow_limits_and_directional_overflow() {
        let provenance = qualified(CombatAction::Attack, &animation())
            .provenance()
            .clone();
        for case in 0..8 {
            let mut source = animation();
            match case {
                0 => source.frames[1].events = vec![FrameEvent::Skip],
                1 => {
                    source.frames[1].events = vec![FrameEvent::UninterpretedNumeric {
                        marker: b'^',
                        value: 7,
                    }]
                }
                2 => source.frames[1].image_id = u32::MAX - 6,
                3 => source.directionality = Directionality::Undirected,
                4 => source
                    .frames
                    .resize(MAX_FRAMES + 1, source.frames[1].clone()),
                5 => source.frames[1].events = vec![FrameEvent::Hit; MAX_EVENTS + 1],
                6 => source.frames[1..]
                    .iter_mut()
                    .for_each(|f| f.duration_ticks = 0),
                7 => source.frames_per_second = 0,
                _ => unreachable!(),
            }
            assert!(
                QualifiedCombatAnimation::qualify(provenance.clone(), &source).is_err(),
                "case {case}"
            );
        }
    }

    #[test]
    fn catalog_qualification_uses_logical_mapping_and_never_substitutes_weapon_actions() {
        let catalog = SpriteActionCatalog::parse(b"#61 32=3213 knight\n1.attack(1 2,0.0:3 16.0:4!)\n2.damage(1 2,8.0:4 56.0:4)\n8.death(1 2,64.1:4 64.0:4)\n5.sword(1 2,168.0:4 184.0:4)\n", ActionLimits::default()).unwrap();
        let source = QualifiedCombatAnimation::from_catalog(
            &catalog,
            BaseAppearance { class: 1, sex: 0 },
            CombatAction::Attack,
        )
        .unwrap();
        assert_eq!(source.provenance().graphics_id, 3213);
        assert_eq!(
            source
                .start()
                .advance(Duration::ZERO, 5)
                .unwrap()
                .asset
                .file_name(),
            "3213-21.spr"
        );
        assert!(QualifiedCombatAnimation::from_catalog(
            &catalog,
            BaseAppearance { class: 0, sex: 0 },
            CombatAction::Attack
        )
        .is_err());
        let missing = SpriteActionCatalog::parse(
            b"#61 32=3213 knight\n5.sword(1 2,168.0:4 184.0:4)\n",
            ActionLimits::default(),
        )
        .unwrap();
        assert!(QualifiedCombatAnimation::from_catalog(
            &missing,
            BaseAppearance { class: 1, sex: 0 },
            CombatAction::Attack
        )
        .is_err());
    }

    #[test]
    fn retained_weapon_types_are_not_action_bytes_and_missing_families_fail_closed() {
        let knight = BaseAppearance { class: 1, sex: 0 };
        for (ty, base) in [
            (0, 0),
            (1, 4),
            (2, 46),
            (3, 50),
            (4, 20),
            (5, 24),
            (6, 11),
            (7, 40),
            (10, 62),
            (11, 58),
            (12, 54),
            (13, 20),
            (14, 24),
            (15, 11),
            (16, 40),
            (17, 58),
        ] {
            assert_eq!(weapon_stance_base(ty).unwrap(), base);
        }
        for ty in [8, 9, 18, 19, 46, 255] {
            assert!(weapon_stance_base(ty).is_err());
        }
        assert!(qualify_weapon_appearance(knight, 11).is_err());
        assert!(qualify_weapon_appearance(BaseAppearance { class: 6, sex: 0 }, 1).is_err());
        assert!(qualify_weapon_appearance(BaseAppearance { class: 5, sex: 1 }, 2).is_err());
        assert!(qualify_weapon_appearance(BaseAppearance { class: 7, sex: 0 }, 0).is_err());
        assert!(qualify_weapon_appearance(BaseAppearance { class: 0, sex: 2 }, 0).is_err());
        for ty in [0, 1, 2, 3, 5, 6, 14, 15] {
            assert_eq!(
                CombatAction::Death.action_key_armed(knight, ty).unwrap(),
                ActionKey::new(8)
            );
        }
    }

    #[test]
    fn armed_qualification_selects_exact_provenance_and_retains_one_shot_cues() {
        let catalog = SpriteActionCatalog::parse(b"#61 320=3213 knight\n1.attack(1 2,0.0:0 16.0:3!)\n5.sword(1 3,168.0:9 184.0:3[248 184.1:4!)\n47.dagger(1 2,136.0:3 152.0:4!)\n8.death(1 2,64.1:4 64.0:4)\n",ActionLimits::default()).unwrap();
        let appearance = BaseAppearance { class: 1, sex: 0 };
        let armed = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            appearance,
            1,
            CombatAction::Attack,
        )
        .unwrap();
        assert_eq!(armed.provenance().weapon_type, 1);
        assert_eq!(armed.provenance().action_key, ActionKey::new(5));
        assert_eq!(armed.total_ticks(), 7); // excludes terminal duration9
        let mut clock = armed.start();
        let start = clock.advance(Duration::ZERO, 7).unwrap();
        assert_eq!(start.asset.file_name(), "3213-191.spr");
        assert_eq!(start.crossed_cues[0].event, FrameEvent::Sound(248));
        assert_eq!(
            clock.advance(Duration::MAX, 7).unwrap().asset.file_name(),
            "3213-175.spr"
        );
        let dagger = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            appearance,
            2,
            CombatAction::Attack,
        )
        .unwrap();
        assert_eq!(
            dagger
                .start()
                .advance(Duration::ZERO, 0)
                .unwrap()
                .asset
                .file_name(),
            "3213-152.spr"
        );
        assert!(QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            appearance,
            3,
            CombatAction::Attack
        )
        .is_err());
        // A valid unarmed action exists above; it must never rescue missing51.
        assert!(
            QualifiedCombatAnimation::from_catalog(&catalog, appearance, CombatAction::Attack)
                .is_ok()
        );
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; fixed shared catalog, 144 qualified class/sex/weapon families; no output"]
    fn real_shared_weapon_families_have_exact_actions_and_native_main_attack_ticks() {
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Text).unwrap();
        assert!(archive.entry("list.spr").unwrap().size <= 8 * 1024 * 1024);
        let bytes = archive.read_entry("list.spr").unwrap();
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
        );
        let catalog = SpriteActionCatalog::parse_for_inspection(
            &bytes,
            ActionLimits {
                max_bytes: 8 * 1024 * 1024,
                ..Default::default()
            },
        )
        .unwrap();
        let mut count = 0;
        for class in 0..7 {
            for sex in 0..2 {
                for ty in 0..=18 {
                    let appearance = BaseAppearance { class, sex };
                    let Ok(base) = qualify_weapon_appearance(appearance, ty) else {
                        continue;
                    };
                    for id in [base, base + 1, base + 2, base + 3, 8] {
                        let resolved = catalog
                            .resolve_action(
                                appearance.logical_sprite_id().unwrap(),
                                ActionKey::new(id),
                            )
                            .unwrap();
                        assert_eq!(resolved.animation.directionality, Directionality::EightWay);
                        AnimationClock::with_limits(
                            resolved.animation,
                            PlaybackLimits {
                                max_frames: MAX_FRAMES,
                                max_events: MAX_EVENTS,
                            },
                        )
                        .unwrap();
                    }
                    for action in [
                        CombatAction::Attack,
                        CombatAction::Damage,
                        CombatAction::Death,
                    ] {
                        let source = QualifiedCombatAnimation::from_catalog_armed(
                            &catalog, appearance, ty, action,
                        )
                        .unwrap();
                        // Source frame1 may be an audio-only zero-tick setup.
                        // Determine the visible first dwell from the original
                        // schedule, not from the playback's own sampled index.
                        let first_dwell = source
                            .animation()
                            .frames
                            .iter()
                            .enumerate()
                            .skip(1)
                            .find(|(_, frame)| frame.duration_ticks > 0)
                            .expect("qualified action has a positive active duration")
                            .0;
                        for heading in 0..8 {
                            assert_eq!(
                                source
                                    .start()
                                    .advance(Duration::ZERO, heading)
                                    .unwrap()
                                    .source_frame,
                                first_dwell,
                                "{appearance:?} type{ty} {action:?} heading{heading}"
                            );
                            assert!(
                                source
                                    .start()
                                    .advance(Duration::MAX, heading)
                                    .unwrap()
                                    .finished
                            );
                        }
                    }
                    count += 1;
                }
            }
        }
        assert_eq!(count, 144);
        // Independently read #6658 damage2: source1 is zero ticks with three
        // candidates, source2 starts immediately with its own two-candidate bank.
        let dragon_damage = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            BaseAppearance { class: 5, sex: 0 },
            0,
            CombatAction::Damage,
        )
        .unwrap();
        assert_dragon_damage_zero_tick_setup(&dragon_damage);
        // Independent C# decrypted raw definitions #61: attack5/47/51.
        for (ty, action, ticks, hit_tick) in [(1, 5, 22, 15), (2, 47, 18, 12), (3, 51, 23, 14)] {
            let source = QualifiedCombatAnimation::from_catalog_armed(
                &catalog,
                BaseAppearance { class: 1, sex: 0 },
                ty,
                CombatAction::Attack,
            )
            .unwrap();
            assert_eq!(source.provenance().action_key, ActionKey::new(action));
            assert_eq!(source.total_ticks(), ticks);
            let end = source.start().advance(Duration::MAX, 0).unwrap();
            assert_eq!(
                end.crossed_cues
                    .iter()
                    .filter(|cue| cue.event == FrameEvent::Hit)
                    .map(|cue| cue.tick)
                    .collect::<Vec<_>>(),
                vec![hit_tick]
            );
        }
    }

    fn assert_dragon_damage_zero_tick_setup(source: &QualifiedCombatAnimation) {
        assert_eq!(source.animation().frames_per_second, 24);
        assert_eq!(
            source
                .animation()
                .frames
                .iter()
                .map(|frame| frame.duration_ticks)
                .collect::<Vec<_>>(),
            vec![4, 0, 6, 4]
        );
        assert_eq!(source.total_ticks(), 10);
        let mut playback = source.start();
        let first = playback.advance(Duration::ZERO, 3).unwrap();
        assert_eq!(first.source_frame, 2);
        assert_eq!((first.asset.image_id, first.asset.subframe), (19, 0));
        assert!(!first.finished);
        assert_eq!(first.next_deadline, Some(Duration::from_millis(250)));
        assert_eq!(
            first
                .crossed_cues
                .iter()
                .map(|cue| (cue.source_frame, cue.tick, &cue.event))
                .collect::<Vec<_>>(),
            vec![
                (1, 0, &FrameEvent::Effect(1759)),
                (1, 0, &FrameEvent::Effect(1953)),
                (1, 0, &FrameEvent::Effect(1771)),
                (2, 0, &FrameEvent::Effect(4777)),
                (2, 0, &FrameEvent::Effect(4778)),
            ]
        );
        assert!(playback
            .advance(Duration::ZERO, 3)
            .unwrap()
            .crossed_cues
            .is_empty());
        let next = playback.advance(Duration::from_millis(250), 3).unwrap();
        assert_eq!(next.source_frame, 3);
        assert!(next.crossed_cues.is_empty());
        let end = playback
            .advance(Duration::from_nanos(416_666_667), 3)
            .unwrap();
        assert_eq!(end.source_frame, 0);
        assert!(end.finished);
        assert_eq!((end.asset.image_id, end.asset.subframe), (11, 0));
        assert!(end.crossed_cues.is_empty());
        assert_eq!(end.next_deadline, None);
        let held = playback.advance(Duration::MAX, 3).unwrap();
        assert_eq!(held.source_frame, 0);
        assert!(held.finished);
        assert!(held.crossed_cues.is_empty());
        assert_eq!(held.next_deadline, None);
    }

    #[test]
    fn actual_dragon_male_damage_zero_tick_setup_emits_two_banks_once_and_holds_terminal() {
        // Literal action line independently read from shared list.spr #6658.
        let catalog=SpriteActionCatalog::parse(b"#6658 512 dragon\n2.damage(1 4,8.0:4 16.0:0<1759<1953<1771 16.0:6<4777<4778 16.1:4)\n",ActionLimits::default()).unwrap();
        let source = QualifiedCombatAnimation::from_catalog_armed(
            &catalog,
            BaseAppearance { class: 5, sex: 0 },
            0,
            CombatAction::Damage,
        )
        .unwrap();
        assert_dragon_damage_zero_tick_setup(&source);
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact shared Text/512 SPR/63 WAV read+decode, no output/playback"]
    fn real_shared_all_fourteen_unarmed_combat_actions_and_native_audio_members_are_present() {
        let root =
            PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"));
        let mut text = SharedArchive::open(&root, SharedArchiveKind::Text).unwrap();
        assert!(text.entry("list.spr").unwrap().size <= 8 * 1024 * 1024);
        let bytes = text.read_entry("list.spr").unwrap();
        assert_eq!(bytes.len(), 4_067_417);
        assert_eq!(
            format!("{:x}", Sha256::digest(&bytes)),
            "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
        );
        let catalog = SpriteActionCatalog::parse_for_inspection(
            &bytes,
            ActionLimits {
                max_bytes: 8 * 1024 * 1024,
                ..Default::default()
            },
        )
        .unwrap();
        // Independent raw-table scan: (attack ticks, damage ticks, death ticks,
        // hit tick), all ACTIVE ticks exclude source terminal entry0's duration.
        let expected = [
            (20, 10, 44, 14),
            (20, 10, 44, 15),
            (20, 8, 36, 10),
            (20, 10, 72, 8),
            (19, 8, 24, 11),
            (19, 8, 44, 7),
            (22, 8, 56, 14),
            (22, 8, 40, 14),
            (16, 8, 37, 8),
            (17, 8, 42, 7),
            (20, 10, 60, 10),
            (23, 10, 52, 13),
            (20, 10, 44, 10),
            (20, 10, 76, 13),
        ];
        let mut files: BTreeMap<String, u32> = BTreeMap::new();
        let mut sounds = BTreeSet::new();
        for class in 0..7 {
            for sex in 0..2 {
                let appearance = BaseAppearance { class, sex };
                let row = expected[(class * 2 + sex) as usize];
                for action in [
                    CombatAction::Attack,
                    CombatAction::Damage,
                    CombatAction::Death,
                ] {
                    let source =
                        QualifiedCombatAnimation::from_catalog(&catalog, appearance, action)
                            .unwrap();
                    assert_eq!(source.animation().frames_per_second, 24);
                    assert_eq!(
                        source.total_ticks(),
                        match action {
                            CombatAction::Attack => row.0,
                            CombatAction::Damage => row.1,
                            CombatAction::Death => row.2,
                        }
                    );
                    assert_eq!(
                        source.provenance().definition_sprite_id,
                        source.provenance().logical_sprite_id
                    );
                    let mut playback = source.start();
                    let all = playback.advance(Duration::MAX, 0).unwrap();
                    assert!(all.finished);
                    assert_eq!(all.source_frame, 0);
                    assert_eq!(
                        all.crossed_cues.len(),
                        source
                            .animation()
                            .frames
                            .iter()
                            .map(|f| f.events.len())
                            .sum::<usize>()
                    );
                    let hits: Vec<_> = all
                        .crossed_cues
                        .iter()
                        .filter(|cue| cue.event == FrameEvent::Hit)
                        .collect();
                    if action == CombatAction::Attack {
                        assert_eq!(hits.len(), 1);
                        assert_eq!(hits[0].tick, row.3);
                    } else {
                        assert!(hits.is_empty());
                    }
                    assert!(playback
                        .advance(Duration::MAX, 0)
                        .unwrap()
                        .crossed_cues
                        .is_empty());
                    for (frame_index, frame) in source.animation().frames.iter().enumerate() {
                        if let Some(group) = classify_native_cues(&frame.events).unwrap().audio {
                            sounds.extend(group.candidates().iter().copied().filter(|id| *id != 0));
                        }
                        for heading in 0..8 {
                            let asset = source
                                .animation()
                                .frame_asset(source.provenance().graphics_id, frame_index, heading)
                                .unwrap();
                            files
                                .entry(asset.file_name())
                                .and_modify(|n| *n = (*n).max(asset.subframe))
                                .or_insert(asset.subframe);
                        }
                    }
                }
            }
        }
        assert_eq!(files.len(), 512);
        let mut base = SharedArchive::open(&root, SharedArchiveKind::Sprite).unwrap();
        let mut shards = BTreeMap::new();
        let mut encoded = 0usize;
        let mut decoded_bytes = 0usize;
        let mut base_count = 0;
        for (name, required) in files {
            let shard = name.bytes().fold(0u32, |sum, byte| sum + u32::from(byte)) as u8 & 15;
            if let std::collections::btree_map::Entry::Vacant(slot) = shards.entry(shard) {
                slot.insert(
                    SharedArchive::open(&root, SharedArchiveKind::SpriteShard(shard)).unwrap(),
                );
            }
            let bank = shards.get_mut(&shard).unwrap();
            let archive = match bank.entry(&name) {
                Ok(_) => bank,
                Err(ResourceError::MissingEntry(_)) => {
                    base_count += 1;
                    &mut base
                }
                Err(error) => panic!("corrupt/ambiguous native shard must not fall back: {error}"),
            };
            let size = archive.entry(&name).unwrap().size as usize;
            assert!(size <= 8 * 1024 * 1024 && encoded + size <= 32 * 1024 * 1024);
            let bytes = archive.read_entry(&name).unwrap();
            encoded += bytes.len();
            let decoded = decode_classic_spr_with_limits(
                &bytes,
                SpriteLimits {
                    max_input_bytes: 8 * 1024 * 1024,
                    max_total_pixels: (128 * 1024 * 1024 - decoded_bytes) / 4,
                    ..Default::default()
                },
            )
            .unwrap_or_else(|error| panic!("{name}: {error}"));
            assert!((required as usize) < decoded.frames.len(), "{name}");
            decoded_bytes += decoded.frames.iter().map(|f| f.rgba.len()).sum::<usize>();
            let known = match name.as_str() {
                "3213-16.spr" => {
                    Some("e536cd320442ed375d84edafde9722e861b5bd294a44b918524717f3e77ed389")
                }
                "3213-64.spr" => {
                    Some("bd37645499a03cab0d75e76a5d340c8a493bcbb3a10570ac2b6cd178ec6c44d0")
                }
                "6650-64.spr" => {
                    Some("b1ee8a20c57bcbded0fe523e0972d772a9db4b8d3ffef10f0f98e6bffdb478af")
                }
                "6658-24.spr" => {
                    Some("fadf315f805b03fee0222782f7b4a9a0b22e261efa9180da84ddbc0b31f993cc")
                }
                _ => None,
            };
            if let Some(expected) = known {
                assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected);
            }
        }
        assert_eq!(base_count, 335);
        assert_eq!(encoded, 5_280_989);
        assert_eq!(decoded_bytes, 60_937_344);
        assert_eq!(sounds.len(), 63);
        let mut sound_bytes = 0usize;
        for id in sounds {
            let bytes =
                read_shared_sound_with_limit(&root, SharedSoundKey { sound_id: id }, 1024 * 1024)
                    .unwrap();
            sound_bytes += bytes.len();
            assert!(sound_bytes <= 8 * 1024 * 1024);
            parse_pcm_wav(&bytes).unwrap_or_else(|error| panic!("shared Sound/{id}.wav: {error}"));
        }
        assert_eq!(sound_bytes, 2_095_998);
        eprintln!(
            "42 combat actions / 512 SPR (335 base,177 hash-shard), encoded={encoded}, RGBA={decoded_bytes}; 63 shared WAV bytes={sound_bytes}; cues classified/preserved, NOT played, no combat authority applied"
        );
    }
}
