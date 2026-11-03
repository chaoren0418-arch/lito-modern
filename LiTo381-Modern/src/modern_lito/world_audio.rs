//! Presentation-only, bounded delivery of qualified one-shot sound cues.
//! Server revisions start sounds; neither audio timing nor Hit markers can
//! change game state. A slow UI drops old cues instead of replaying a backlog.
use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

use anyhow::{ensure, Context, Result};

use super::{
    combat_animation::{
        classify_native_cues, CombatPlayback, CrossedCue, NativeAudioGroup, MAX_EVENTS,
    },
    combat_audio::{BankLimits, SoundBank},
    combat_protocol,
    world_scene::{BaseAction, SceneSnapshot, WorldSceneAssets},
};

pub(crate) const HISTORY_LIMIT: usize = 256;
const MAX_CUE_AGE: Duration = Duration::from_millis(500);

pub(crate) fn crossed_audio_groups(cues: &[CrossedCue]) -> Result<Vec<(u64, NativeAudioGroup)>> {
    ensure!(
        cues.len() <= MAX_EVENTS,
        "audio event delivery exceeds bound"
    );
    let mut groups = Vec::new();
    let mut begin = 0;
    while begin < cues.len() {
        let mut end = begin + 1;
        while end < cues.len() && cues[end].source_frame == cues[begin].source_frame {
            ensure!(
                cues[end].tick == cues[begin].tick,
                "one source frame has different audio times"
            );
            end += 1;
        }
        let events: Vec<_> = cues[begin..end]
            .iter()
            .map(|cue| cue.event.clone())
            .collect();
        if let Some(group) = classify_native_cues(&events)?.audio {
            groups.push((cues[begin].tick, group));
        }
        begin = end;
    }
    Ok(groups)
}

#[derive(Clone, Debug)]
pub struct SoundEvent {
    pub sequence: u64,
    pub created: Instant,
    pub group: NativeAudioGroup,
    /// Optional already-qualified resource bank for an additional presentation
    /// family. Shares the same bounded log and voice pool; never re-decodes it.
    pub bank: Option<Arc<SoundBank>>,
}

#[derive(Clone)]
pub struct AudioFrame {
    pub bank: Arc<SoundBank>,
    pub events: Arc<[SoundEvent]>,
}

#[derive(Clone, Default)]
pub(crate) struct EventLog {
    sequence: u64,
    events: VecDeque<SoundEvent>,
}
impl EventLog {
    pub(crate) fn last_sequence(&self) -> u64 {
        self.sequence
    }
    pub(crate) fn retain_sequences(&mut self, mut keep: impl FnMut(u64) -> bool) {
        self.events.retain(|event| keep(event.sequence));
    }
    fn push(&mut self, group: NativeAudioGroup, now: Instant) -> Result<()> {
        self.push_with_bank(group, now, None)
    }
    pub(crate) fn push_with_bank(
        &mut self,
        group: NativeAudioGroup,
        now: Instant,
        bank: Option<Arc<SoundBank>>,
    ) -> Result<()> {
        self.sequence = self
            .sequence
            .checked_add(1)
            .context("sound sequence exhausted")?;
        while self.events.len() >= HISTORY_LIMIT {
            self.events.pop_front();
        }
        self.events.push_back(SoundEvent {
            sequence: self.sequence,
            created: now,
            group,
            bank,
        });
        Ok(())
    }
    pub(crate) fn snapshot(&mut self, now: Instant) -> Arc<[SoundEvent]> {
        // Sequence is delivery order, not creation-time order: a newly
        // received, backdated effect can follow a newer body/spell cue. A
        // fresh front element must not keep stale interior cues alive.
        self.events
            .retain(|event| now.saturating_duration_since(event.created) <= MAX_CUE_AGE);
        self.events.iter().cloned().collect::<Vec<_>>().into()
    }
}

struct ActorAudio {
    revision: i64,
    action: BaseAction,
    playback: CombatPlayback,
    frames_per_second: u32,
}

/// Owned by the resource/render worker, never by an HWND callback.
pub struct AudioSequencer {
    bank: Arc<SoundBank>,
    baseline: BTreeMap<u64, i64>,
    active: BTreeMap<u64, ActorAudio>,
    log: EventLog,
}
/// Both canonical and legacy workers use the exact same qualified body banks.
/// This reads assets only during preflight, never while sampling a sound cue.
pub(crate) fn load_body_bank(
    root: &Path,
    assets: &WorldSceneAssets,
    cancel: &AtomicBool,
) -> Result<Arc<SoundBank>> {
    load_body_bank_with_walk(root, assets, cancel, false)
}
pub(crate) fn load_canonical_body_bank(
    root: &Path,
    assets: &WorldSceneAssets,
    cancel: &AtomicBool,
) -> Result<Arc<SoundBank>> {
    load_body_bank_with_walk(root, assets, cancel, true)
}
fn load_body_bank_with_walk(
    root: &Path,
    assets: &WorldSceneAssets,
    cancel: &AtomicBool,
    walk: bool,
) -> Result<Arc<SoundBank>> {
    let mut groups = Vec::new();
    for (_, _, animation) in assets.combat_animations() {
        for frame in &animation.animation().frames {
            if let Some(group) = classify_native_cues(&frame.events)?.audio {
                ensure!(groups.len() < 4096, "world sound group limit exceeded");
                groups.push(group);
            }
        }
    }
    if walk {
        for animation in assets.player_walk_animations() {
            for frame in animation.source_frames() {
                if let Some(group) = classify_native_cues(&frame.events)?.audio {
                    ensure!(groups.len() < 4096, "world sound group limit exceeded");
                    groups.push(group);
                }
            }
        }
    }
    let bank = Arc::new(SoundBank::load_shared(
        root,
        &groups,
        BankLimits::default(),
        cancel,
    )?);
    if walk {
        // Every allowed footstep alternative together fits the per-event PCM
        // budget, checked in preflight rather than discovering it mid-battle.
        let bytes: usize = [478, 479]
            .into_iter()
            .filter_map(|id| bank.get(id))
            .map(|s| s.pcm_bytes().len() * (2 / usize::from(s.format().channels)))
            .sum();
        ensure!(
            bytes <= 256 * 1024,
            "footstep PCM preflight budget exceeded"
        );
    }
    Ok(bank)
}
impl AudioSequencer {
    pub(crate) fn push_preloaded_cues(
        &mut self,
        cues: &[CrossedCue],
        bank: Arc<SoundBank>,
        now: Instant,
    ) -> Result<()> {
        for (_, group) in crossed_audio_groups(cues)? {
            ensure!(
                group
                    .candidates()
                    .iter()
                    .all(|id| *id == 0 || bank.get(*id).is_some()),
                "extra sound absent from qualified bank"
            );
            self.log.push_with_bank(group, now, Some(bank.clone()))?;
        }
        Ok(())
    }
    pub fn load_shared(
        root: &Path,
        assets: &WorldSceneAssets,
        initial: &combat_protocol::Frame,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Ok(Self {
            bank: load_body_bank(root, assets, cancel)?,
            baseline: initial
                .vitals
                .iter()
                .map(|v| {
                    (
                        v.entity_id,
                        v.attack_revision
                            .max(v.hurt_revision)
                            .max(v.death_revision)
                            .max(
                                super::siege_presentation::player_attack(
                                    &initial.world,
                                    v.entity_id,
                                )
                                .map_or(0, |e| e.revision),
                            ),
                    )
                })
                .collect(),
            active: BTreeMap::new(),
            log: EventLog::default(),
        })
    }

    pub fn advance(
        &mut self,
        assets: &WorldSceneAssets,
        snapshot: &SceneSnapshot,
        frame: &combat_protocol::Frame,
        now: Instant,
    ) -> Result<AudioFrame> {
        self.advance_npcs(assets, snapshot, frame, None, now)
    }
    pub(crate) fn pin_npc_baseline(&mut self, frame: &super::npc_protocol::Frame) {
        for (entity, baseline) in &mut self.baseline {
            *baseline = (*baseline).max(
                super::npc_presentation::player_attack(frame, *entity).map_or(0, |e| e.revision),
            );
        }
    }
    pub(crate) fn advance_npcs(
        &mut self,
        assets: &WorldSceneAssets,
        snapshot: &SceneSnapshot,
        frame: &combat_protocol::Frame,
        npc: Option<&super::npc_protocol::Frame>,
        now: Instant,
    ) -> Result<AudioFrame> {
        ensure!(
            npc.is_none_or(|n| n.combat == *frame),
            "NPC/player audio revisions differ"
        );
        for actor in snapshot.actors() {
            let vital = frame
                .vitals
                .iter()
                .find(|v| v.entity_id == actor.entity_id)
                .context("audio actor absent")?;
            let revision = match actor.action {
                BaseAction::Attack => vital
                    .attack_revision
                    .max(
                        npc.and_then(|n| {
                            super::npc_presentation::player_attack(n, actor.entity_id)
                        })
                        .map_or(0, |e| e.revision),
                    )
                    .max(
                        super::siege_presentation::player_attack(&frame.world, actor.entity_id)
                            .map_or(0, |e| e.revision),
                    ),
                BaseAction::Damage => vital.hurt_revision,
                BaseAction::Death => vital.death_revision,
                _ => {
                    self.active.remove(&actor.entity_id);
                    continue;
                }
            };
            let restart = self
                .active
                .get(&actor.entity_id)
                .is_none_or(|old| old.revision != revision || old.action != actor.action);
            if restart {
                // Initial/late-join snapshots and interrupted old events are
                // historical, not new sounds. Keep the watermark after idle.
                let baseline = self
                    .baseline
                    .get_mut(&actor.entity_id)
                    .context("audio baseline absent")?;
                if revision <= *baseline {
                    continue;
                }
                let animation = assets
                    .combat_animation_armed(actor.appearance, actor.weapon_type, actor.action)
                    .context("audio animation unqualified")?;
                self.active.insert(
                    actor.entity_id,
                    ActorAudio {
                        revision,
                        action: actor.action,
                        playback: animation.start(),
                        frames_per_second: animation.animation().frames_per_second,
                    },
                );
                *baseline = revision;
            }
            let sample = self
                .active
                .get_mut(&actor.entity_id)
                .unwrap()
                .playback
                .advance(actor.action_elapsed, actor.heading)?;
            // Preserve frame boundaries: adjacent banks in different source
            // frames must never become one randomly selected candidate group.
            for (tick, group) in crossed_audio_groups(&sample.crossed_cues)? {
                let fps = self.active[&actor.entity_id].frames_per_second;
                let nanos = u128::from(tick) * 1_000_000_000 / u128::from(fps);
                let cue_elapsed = Duration::from_nanos(u64::try_from(nanos)?);
                let age = actor.action_elapsed.saturating_sub(cue_elapsed);
                if age <= MAX_CUE_AGE {
                    self.log.push(
                        group,
                        now.checked_sub(age).context("audio timestamp overflow")?,
                    )?;
                }
            }
        }
        Ok(AudioFrame {
            bank: self.bank.clone(),
            events: self.log.snapshot(now),
        })
    }
}

#[derive(Default)]
struct Cursor {
    sequence: u64,
}
impl Cursor {
    fn consume<'a>(&mut self, events: &'a [SoundEvent], now: Instant) -> Vec<&'a SoundEvent> {
        let mut fresh = Vec::new();
        for event in events {
            if event.sequence <= self.sequence {
                continue;
            }
            self.sequence = event.sequence;
            if now
                .checked_duration_since(event.created)
                .is_some_and(|age| age <= MAX_CUE_AGE)
            {
                fresh.push(event);
            }
        }
        fresh
    }
}

#[cfg(windows)]
pub struct WorldAudioOutput {
    pool: super::combat_audio::CombatVoicePool,
    identity: Option<(u64, u64)>,
    cursor: Cursor,
    random: u32,
    failed: bool,
    gain: u8,
}
#[cfg(windows)]
impl WorldAudioOutput {
    pub fn new() -> Result<Self> {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
            | 1;
        Ok(Self {
            pool: super::combat_audio::CombatVoicePool::new(8)?,
            identity: None,
            cursor: Cursor::default(),
            random: seed,
            failed: false,
            gain: 100,
        })
    }
    pub fn poll(&mut self) {
        if self.pool.poll().is_err() {
            self.failed = true;
        }
    }
    pub fn stop(&mut self) {
        if self.pool.stop_all().is_err() {
            self.failed = true;
        }
        self.identity = None;
        self.cursor = Cursor::default();
    }
    pub fn failed(&self) -> bool {
        self.failed
    }
    /// Stops old-gain voices but does not reset the event watermark. Switching
    /// focus/mute/volume must not replay earlier sounds after recovery.
    pub fn configure(&mut self, gain: u8) {
        let gain = gain.min(100);
        if self.gain != gain {
            if self.pool.stop_all().is_err() {
                self.failed = true;
            }
            self.gain = gain;
        }
    }
    /// The UI calls `poll` once at the start of its tick, even without a new
    /// image. Submitting this batch must not query the same voices again.
    pub fn update(&mut self, identity: (u64, u64), frame: &AudioFrame, now: Instant) {
        self.update_at_gain(identity, frame, now, Some(100));
    }
    pub fn update_at_gain(
        &mut self,
        identity: (u64, u64),
        frame: &AudioFrame,
        now: Instant,
        prepared_gain: Option<u8>,
    ) {
        if self.identity != Some(identity) {
            self.stop();
            self.identity = Some(identity);
        }
        // Advance even when disabled. Never replay missed sound on recovery.
        for event in self.cursor.consume(&frame.events, now) {
            if self.failed || !gain_matches(self.gain, prepared_gain) {
                continue;
            }
            // Presentation-only selection; never shared with server combat RNG.
            self.random ^= self.random << 13;
            self.random ^= self.random >> 17;
            self.random ^= self.random << 5;
            if self
                .pool
                .play(
                    event.bank.as_deref().unwrap_or(&frame.bank),
                    &event.group,
                    self.random,
                )
                .is_err()
            {
                self.failed = true;
            }
        }
    }
}

fn gain_matches(desired: u8, prepared: Option<u8>) -> bool {
    desired > 0 && desired <= 100 && prepared == Some(desired)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::sprite_actions::FrameEvent;
    #[test]
    fn natural_recovery_keeps_playback_cursor_and_emits_no_new_sound_sequence() {
        use crate::modern_lito::{
            combat_animation::{CombatAction, QualifiedCombatAnimation},
            sprite_actions::{ActionLimits, SpriteActionCatalog},
            world_scene::{ActorState, BaseAppearance},
        };

        let (_, attacked, recovered) =
            crate::modern_lito::world_runtime::tests::natural_regen_frames();
        let assets = crate::modern_lito::world_scene::tests::synthetic_assets();
        assert_eq!(assets.combat_animations().count(), 0);
        // An empty group list never invokes SoundBank's file-read/PCM-decode
        // closure. This test delivers cues only: no resources or audio device.
        let mut audio = AudioSequencer::load_shared(
            Path::new("\0empty-bank-must-not-be-read"),
            &assets,
            &attacked,
            &AtomicBool::new(false),
        )
        .unwrap();
        assert!(audio.bank.is_empty());
        assert_eq!(audio.bank.file_bytes(), 0);
        assert_eq!(audio.bank.pcm_bytes(), 0);
        let catalog = SpriteActionCatalog::parse(
            b"#61 32=3213 synthetic knight\n1.attack(1 2,0.0:0 16.0:24[13)\n#1186 32=3221 synthetic wizard\n2.damage(1 2,0.0:0 16.0:24[13)\n",
            ActionLimits::default(),
        ).unwrap();
        for (index, action) in [(0, BaseAction::Attack), (1, BaseAction::Damage)] {
            let member = &attacked.world.room.members[index];
            let animation = QualifiedCombatAnimation::from_catalog(
                &catalog,
                BaseAppearance {
                    class: u8::try_from(member.class).unwrap(),
                    sex: member.sex,
                },
                if index == 0 {
                    CombatAction::Attack
                } else {
                    CombatAction::Damage
                },
            )
            .unwrap();
            audio.active.insert(
                index as u64 + 1,
                ActorAudio {
                    revision: 4,
                    action,
                    playback: animation.start(),
                    frames_per_second: animation.animation().frames_per_second,
                },
            );
        }
        let scene = |frame: &combat_protocol::Frame, elapsed| {
            SceneSnapshot::new(
                frame.world.match_id,
                frame.world.revision as u64,
                frame.world.manifest.digest,
                1,
                frame
                    .world
                    .actors
                    .iter()
                    .enumerate()
                    .map(|(index, actor)| {
                        let member = &frame.world.room.members[index];
                        ActorState {
                            kind: actor.kind,
                            entity_id: actor.entity_id,
                            source_id: actor.key.source_id,
                            object_id: actor.key.object_id,
                            appearance: BaseAppearance {
                                class: u8::try_from(member.class).unwrap(),
                                sex: member.sex,
                            },
                            weapon_type: frame.vitals[index].weapon_type,
                            cell: actor.position,
                            heading: actor.heading,
                            action: if index == 0 {
                                BaseAction::Attack
                            } else {
                                BaseAction::Damage
                            },
                            action_elapsed: elapsed,
                        }
                    })
                    .collect(),
            )
            .unwrap()
        };
        let now = Instant::now();
        let first = audio
            .advance(&assets, &scene(&attacked, Duration::ZERO), &attacked, now)
            .unwrap();
        assert_eq!(
            first
                .events
                .iter()
                .map(|event| event.sequence)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        assert!(first
            .events
            .iter()
            .all(|event| event.group.candidates() == [13]));
        let mut cursor = Cursor::default();
        assert_eq!(cursor.consume(&first.events, now).len(), 2);
        assert_eq!(recovered.vitals[0].hp, attacked.vitals[0].hp + 5);
        assert_eq!(recovered.vitals[0].mp, attacked.vitals[0].mp + 1);
        for elapsed in [
            Duration::from_millis(100),
            Duration::from_millis(100),
            Duration::from_millis(400),
        ] {
            // Real advance on a new natural-recovery world revision must keep
            // the two existing playback cursors. Assets intentionally have no
            // combat action to load if a mistaken restart occurs.
            let frame = audio
                .advance(
                    &assets,
                    &scene(&recovered, elapsed),
                    &recovered,
                    now + elapsed,
                )
                .unwrap();
            assert_eq!(audio.log.sequence, 2);
            assert_eq!(
                frame
                    .events
                    .iter()
                    .map(|event| event.sequence)
                    .collect::<Vec<_>>(),
                [1, 2]
            );
            assert!(cursor.consume(&frame.events, now + elapsed).is_empty());
            assert_eq!(audio.active[&1].revision, 4);
            assert_eq!(audio.active[&2].revision, 4);
        }
        let elapsed = Duration::from_secs(2);
        let retired = audio
            .advance(
                &assets,
                &scene(&recovered, elapsed),
                &recovered,
                now + elapsed,
            )
            .unwrap();
        assert!(retired.events.is_empty());
        assert_eq!(audio.log.sequence, 2);
        assert!(cursor.consume(&retired.events, now + elapsed).is_empty());
        assert!(audio.bank.is_empty());
    }
    fn group(id: u32) -> NativeAudioGroup {
        classify_native_cues(&[FrameEvent::Sound(id)])
            .unwrap()
            .audio
            .unwrap()
    }
    #[cfg(windows)]
    #[test]
    fn mute_background_gain_change_and_failed_preparation_consume_history_without_a_device() {
        let now = Instant::now();
        // Deliberately absent sound: any accidental replay would fail the pool.
        let bank = Arc::new(SoundBank::playback_subset(BTreeMap::new()).unwrap());
        let batch = |sequence| AudioFrame {
            bank: bank.clone(),
            events: vec![SoundEvent {
                sequence,
                created: now,
                group: group(97),
                bank: None,
            }]
            .into(),
        };
        let mut output = WorldAudioOutput::new().unwrap();
        let seed = output.random;
        let identity = (7, 3);
        for (sequence, desired, prepared) in [
            (1, 0, Some(100)),
            (2, 50, Some(100)),
            (3, 50, None),
            (4, 0, Some(50)),
        ] {
            output.configure(desired);
            output.update_at_gain(identity, &batch(sequence), now, prepared);
            assert_eq!(output.cursor.sequence, sequence);
            output.configure(100);
            assert_eq!(output.identity, Some(identity));
            output.update_at_gain(identity, &batch(sequence), now, Some(100));
            assert!(!output.failed());
            assert_eq!(
                output.random, seed,
                "no random choice, sound lookup or device play"
            );
        }
        assert!(gain_matches(50, Some(50)));
        assert!(!gain_matches(0, Some(0)));
        assert!(!gain_matches(101, Some(101)));
        output.stop();
        assert!(output.identity.is_none());
        assert_eq!(output.cursor.sequence, 0);
    }
    #[test]
    fn zero_tick_adjacent_frame_banks_remain_separate_and_hit_is_not_audio() {
        let cues: Vec<_> = [(1, 1759), (1, 1953), (1, 1771), (2, 4777), (2, 4778)]
            .into_iter()
            .enumerate()
            .map(|(event_index, (source_frame, id))| CrossedCue {
                source_frame,
                event_index: if source_frame == 1 {
                    event_index
                } else {
                    event_index - 3
                },
                tick: 0,
                event: FrameEvent::Effect(id),
            })
            .chain(std::iter::once(CrossedCue {
                source_frame: 2,
                event_index: 2,
                tick: 0,
                event: FrameEvent::Hit,
            }))
            .collect();
        let groups = crossed_audio_groups(&cues).unwrap();
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].0, 0);
        assert_eq!(groups[1].0, 0);
        assert_eq!(groups[0].1.candidates(), &[1759, 1953, 1771]);
        assert_eq!(groups[1].1.candidates(), &[4777, 4778]);
        let mut corrupt = cues.clone();
        corrupt[1].tick = 1;
        assert!(crossed_audio_groups(&corrupt).is_err());
        assert!(crossed_audio_groups(&vec![cues[0].clone(); MAX_EVENTS + 1]).is_err());
    }
    #[test]
    fn history_is_bounded_and_coalesced_ui_does_not_replay() {
        let now = Instant::now();
        let mut log = EventLog::default();
        for _ in 0..300 {
            log.push(group(3), now).unwrap();
        }
        let frame = log.snapshot(now);
        assert_eq!(frame.len(), HISTORY_LIMIT);
        assert_eq!(frame[0].sequence, 45);
        let mut cursor = Cursor::default();
        assert_eq!(cursor.consume(&frame, now).len(), HISTORY_LIMIT);
        assert!(cursor.consume(&frame, now).is_empty());
        log.push(group(4), now).unwrap();
        assert_eq!(cursor.consume(&log.snapshot(now), now).len(), 1);
    }
    #[test]
    fn stale_future_and_terminal_sequence_never_replay_or_wrap() {
        let now = Instant::now();
        let mut log = EventLog::default();
        log.push(group(3), now).unwrap();
        let frame = log.snapshot(now);
        let mut cursor = Cursor::default();
        assert!(cursor
            .consume(&frame, now + Duration::from_secs(1))
            .is_empty());
        assert!(cursor.consume(&frame, now).is_empty());
        assert!(log.snapshot(now + Duration::from_secs(1)).is_empty());
        log.sequence = u64::MAX;
        assert!(log.push(group(4), now).is_err());
        assert!(log.events.is_empty());
        let mut cursor = Cursor::default();
        assert!(cursor
            .consume(&frame, now - Duration::from_millis(1))
            .is_empty());
        assert!(cursor.consume(&frame, now).is_empty());
    }

    #[test]
    fn backdated_cues_expire_independently_without_reordering_delivery_sequences() {
        let now = Instant::now();
        let mut log = EventLog::default();
        log.push(group(97), now).unwrap();
        log.push(group(109), now - Duration::from_millis(400))
            .unwrap();
        let original = log.snapshot(now);
        assert_eq!(
            original.iter().map(|e| e.sequence).collect::<Vec<_>>(),
            vec![1, 2]
        );
        assert_eq!(log.snapshot(now + Duration::from_millis(100)).len(), 2);
        let next = log.snapshot(now + Duration::from_millis(100) + Duration::from_nanos(1));
        assert_eq!(next.len(), 1);
        assert_eq!(next[0].sequence, 1);
        assert_eq!(
            original.len(),
            2,
            "retained immutable frames are not rewritten"
        );
        log.push(group(121), now + Duration::from_millis(200))
            .unwrap();
        assert_eq!(
            log.snapshot(now + Duration::from_millis(200))
                .iter()
                .map(|e| e.sequence)
                .collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert_eq!(
            log.snapshot(now + Duration::from_millis(501))[0].sequence,
            3
        );
        assert!(log.snapshot(now + Duration::from_millis(701)).is_empty());
    }
}
