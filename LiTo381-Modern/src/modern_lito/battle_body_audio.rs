//! Canonical body cues, using the SAME motion clocks/samples as scene pixels.
//! No legacy Frame, local combat RNG, inferred HP, or command/LOAD authority.
use super::{
    battle_follow::Observation,
    battle_motion::{self, NpcStamp, Timeline as Motion},
    combat_animation::{NativeAudioGroup, QualifiedCombatAnimation},
    combat_audio::SoundBank,
    npc_assets::{NpcAction, NpcAppearance, NpcAssets},
    paced_animation::PacedClock,
    polymorph_assets::{PolymorphAssets, QualifiedPolymorphAction},
    world_audio::{crossed_audio_groups, AudioFrame, EventLog},
    world_scene::{BaseAction, BaseAppearance, SceneSnapshot, WorldSceneAssets},
};
use anyhow::{ensure, Context, Result};
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

const MAX_AGE: Duration = Duration::from_millis(500);
#[path = "battle_footsteps.rs"]
mod footsteps;
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "canonical body audio cancelled"
    );
    Ok(())
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct PlayerKey {
    entity: i64,
    life: i64,
    appearance: BaseAppearance,
    weapon: u8,
    logical_form: u32,
    action: BaseAction,
    revision: i64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct NpcKey {
    serial: i64,
    life: i64,
    graphic: i32,
    stamp: NpcStamp,
}
#[derive(Clone, Debug, PartialEq, Eq)]
enum Owner {
    Player(PlayerKey),
    Walk(PlayerKey),
    Npc(NpcKey),
}
#[derive(Clone)]
struct Cue {
    phase: Duration,
    group: NativeAudioGroup,
    deadline: Instant,
    consumed: bool,
}
fn schedule(
    animation: &QualifiedCombatAnimation,
    clock: &PacedClock,
    now: Instant,
) -> Result<Vec<Cue>> {
    let mut playback = animation.start();
    let sample = playback.advance(animation.duration()?, 0)?;
    let fps = u128::from(animation.animation().frames_per_second);
    crossed_audio_groups(&sample.crossed_cues)?
        .into_iter()
        .map(|(tick, group)| {
            let phase = Duration::from_nanos(u64::try_from(
                (u128::from(tick) * 1_000_000_000).div_ceil(fps),
            )?);
            Ok(Cue {
                phase,
                group,
                deadline: clock.clone().until(now, phase)?,
                consumed: false,
            })
        })
        .collect()
}
fn retime(cues: &mut [Cue], clock: &PacedClock, now: Instant) -> Result<()> {
    for cue in cues {
        // Keep a crossed deadline, even if no surface has yet delivered it.
        // Only future markers are re-timed after a confirmed pace change.
        if !cue.consumed && cue.deadline > now {
            cue.deadline = clock.clone().until(now, cue.phase)?;
        }
    }
    Ok(())
}
fn schedule_polymorph(
    animation: &QualifiedPolymorphAction,
    clock: &PacedClock,
    now: Instant,
) -> Result<Vec<Cue>> {
    animation
        .audio_phases()?
        .into_iter()
        .map(|(phase, group)| {
            Ok(Cue {
                phase,
                group,
                deadline: clock.clone().until(now, phase)?,
                consumed: false,
            })
        })
        .collect()
}
#[derive(Clone)]
struct PlayerTrack {
    key: PlayerKey,
    cues: Vec<Cue>,
    bank: Option<Arc<SoundBank>>,
}
#[derive(Clone)]
struct NpcTrack {
    key: NpcKey,
    marker: Option<(NpcAction, u128, usize)>,
    sampled: Instant,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    sampled: Instant,
    players: BTreeMap<i64, PlayerTrack>,
    walkers: BTreeMap<i64, footsteps::Track>,
    npcs: BTreeMap<i64, NpcTrack>,
    log: EventLog,
    owners: BTreeMap<u64, Owner>,
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        Self {
            observation,
            sampled: now,
            players: BTreeMap::new(),
            walkers: BTreeMap::new(),
            npcs: BTreeMap::new(),
            log: EventLog::default(),
            owners: BTreeMap::new(),
        }
    }
    pub(crate) fn advance(
        &self,
        observation: Observation,
        motion: &Motion,
        scene: &SceneSnapshot,
        assets: &WorldSceneAssets,
        npc_bank: Option<&NpcAssets>,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        self.advance_players(
            observation,
            motion,
            scene,
            assets,
            &BTreeMap::new(),
            npc_bank,
            now,
            cancel,
        )
    }
    pub(crate) fn advance_players(
        &self,
        observation: Observation,
        motion: &Motion,
        scene: &SceneSnapshot,
        assets: &WorldSceneAssets,
        polymorphs: &BTreeMap<u64, &PolymorphAssets>,
        npc_bank: Option<&NpcAssets>,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(
            now >= self.sampled && motion.matches_observation(&observation),
            "body audio owner/clock mismatch"
        );
        self.observation.check_successor(&observation)?;
        let mut candidate = self.clone();
        candidate.players.clear();
        candidate.walkers.clear();
        candidate.npcs.clear();
        let data = observation.data();
        let listener = scene
            .actors()
            .iter()
            .find(|a| a.entity_id == scene.selected_entity_id())
            .context("footstep listener absent")?
            .cell;
        for actor in scene.actors().iter().filter(|a| !scene.hidden(a.entity_id)) {
            active(cancel)?;
            let polymorph = polymorphs.get(&actor.entity_id).copied();
            let logical_form = polymorph.map_or(0, |p| p.form().logical_sprite_id());
            if scene.pet().is_none() {
                if let Some(clock) = motion.walk_audio(actor.entity_id.try_into()?, now)? {
                    let authority = data
                        .actors
                        .0
                        .iter()
                        .find(|a| a.entity == actor.entity_id as i64)
                        .context("footstep actor absent")?;
                    let key = PlayerKey {
                        entity: authority.entity,
                        life: battle_motion::life(authority).context("footstep life overflow")?,
                        appearance: actor.appearance,
                        weapon: actor.weapon_type,
                        logical_form,
                        action: BaseAction::Walk,
                        revision: authority.last_move_sequence,
                    };
                    let mut track =
                        if let Some(old) = self.walkers.get(&key.entity).filter(|p| p.key == key) {
                            old.clone()
                        } else {
                            // Newly visible/attached midway through an existing
                            // step has no owned cue history. Wait for the next
                            // confirmed step, never replay or fail the scene.
                            if !clock.peek(now)?.is_zero() {
                                continue;
                            }
                            let walk = if let Some(poly) = polymorph {
                                let action = poly.action(BaseAction::Walk)?;
                                // Reviewed5641 has no walking cues. Do not route a
                                // future polymorph WAV through base478/479 spatial PCM.
                                for source in 0..action.source_frames().len() {
                                    ensure!(
                                        action.sound_group(source)?.is_none(),
                                        "polymorph footstep audio unqualified"
                                    );
                                }
                                action.walk().context("polymorph walk absent")?
                            } else {
                                assets
                                    .player_walk_animation(key.appearance, key.weapon)
                                    .context("footstep animation unqualified")?
                            };
                            footsteps::Track {
                                key: key.clone(),
                                cues: footsteps::schedule(
                                    walk,
                                    authority.step_interval_nanos.try_into()?,
                                    clock,
                                    now,
                                )?,
                                spatial: (0, 64),
                            }
                        };
                    retime(&mut track.cues, clock, now)?;
                    track.spatial = footsteps::spatial(listener, actor.cell);
                    candidate.walkers.insert(key.entity, track);
                }
            }
            let Some((revision, action, clock)) =
                motion.player_audio(actor.entity_id.try_into()?, now)?
            else {
                continue;
            };
            ensure!(
                matches!(
                    action,
                    BaseAction::Attack | BaseAction::Damage | BaseAction::Death
                ),
                "unqualified body cue action"
            );
            let authority = data
                .actors
                .0
                .iter()
                .find(|a| a.entity == actor.entity_id as i64)
                .context("body cue actor absent")?;
            let key = PlayerKey {
                entity: authority.entity,
                life: battle_motion::life(authority).context("body cue life overflow")?,
                appearance: actor.appearance,
                weapon: actor.weapon_type,
                logical_form,
                action,
                revision,
            };
            let mut track =
                if let Some(previous) = self.players.get(&key.entity).filter(|p| p.key == key) {
                    previous.clone()
                } else {
                    // A newly attached/revealed body has no owned cue history.
                    // Do not restart or backfill an already running action.
                    if !clock.peek(now)?.is_zero() {
                        continue;
                    }
                    let (cues, bank) = if let Some(poly) = polymorph {
                        (
                            schedule_polymorph(poly.action(action)?, clock, now)?,
                            Some(poly.sound_bank()),
                        )
                    } else {
                        let animation = assets
                            .combat_animation_armed(key.appearance, key.weapon, action)
                            .context("body cue animation unqualified")?;
                        (schedule(animation, clock, now)?, None)
                    };
                    PlayerTrack {
                        key: key.clone(),
                        cues,
                        bank,
                    }
                };
            retime(&mut track.cues, clock, now)?;
            candidate.players.insert(key.entity, track);
        }
        if let Some(npcs) = data.npcs.as_ref().filter(|n| !n.actors.0.is_empty()) {
            let bank = npc_bank.context("NPC body sound bank absent")?;
            for actor in &npcs.actors.0 {
                active(cancel)?;
                let pose = motion.npc_sample(actor, bank, now)?;
                let key = NpcKey {
                    serial: actor.serial,
                    life: actor.life,
                    graphic: actor.graphic,
                    stamp: pose.stamp,
                };
                let track = self
                    .npcs
                    .get(&key.serial)
                    .filter(|p| p.key == key)
                    .cloned()
                    .unwrap_or_else(|| NpcTrack {
                        marker: key.stamp.revision.is_none().then_some((
                            pose.action,
                            pose.cycle,
                            pose.source,
                        )),
                        key: key.clone(),
                        sampled: now,
                    });
                candidate.npcs.insert(key.serial, track);
            }
        }
        candidate.observation = observation;
        candidate.sampled = now;
        candidate.prune();
        active(cancel)?;
        Ok(candidate)
    }
    fn prune(&mut self) {
        self.owners.retain(|_, owner| match owner {
            Owner::Player(key) => self.players.get(&key.entity).is_some_and(|p| p.key == *key),
            Owner::Walk(key) => self.walkers.get(&key.entity).is_some_and(|p| p.key == *key),
            Owner::Npc(key) => self.npcs.get(&key.serial).is_some_and(|n| n.key == *key),
        });
        self.log
            .retain_sequences(|id| self.owners.contains_key(&id));
    }
    pub(crate) fn sample(
        &self,
        motion: &Motion,
        player_bank: &Arc<SoundBank>,
        npc_bank: Option<&NpcAssets>,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<(Self, Option<AudioFrame>)> {
        active(cancel)?;
        ensure!(
            now >= self.sampled && motion.matches_observation(&self.observation),
            "body audio sample owner/clock mismatch"
        );
        let mut candidate = self.clone();
        let mut pending = Vec::new();
        let mut expired = Vec::new();
        for (id, track) in &mut candidate.walkers {
            active(cancel)?;
            if motion.walk_audio(*id, now)?.is_none() {
                expired.push(*id);
                continue;
            }
            // Consume all crossed markers, but emit at most the latest one.
            // A late render must not stack both feet or replay completed steps.
            let mut latest = None;
            for cue in &mut track.cues {
                if !cue.consumed && cue.deadline <= now {
                    cue.consumed = true;
                    latest = Some(cue);
                }
            }
            if let Some(cue) =
                latest.filter(|c| now.duration_since(c.deadline) <= MAX_AGE && track.spatial.0 > 0)
            {
                pending.push((
                    cue.deadline,
                    cue.group.clone(),
                    footsteps::bank(player_bank, &cue.group, track.spatial, cancel)?,
                    Owner::Walk(track.key.clone()),
                ));
            }
        }
        for id in expired {
            candidate.walkers.remove(&id);
        }
        candidate.prune();
        for player in candidate.players.values_mut() {
            active(cancel)?;
            for cue in &mut player.cues {
                if !cue.consumed && cue.deadline <= now {
                    cue.consumed = true;
                    if now.duration_since(cue.deadline) <= MAX_AGE {
                        pending.push((
                            cue.deadline,
                            cue.group.clone(),
                            player.bank.as_ref().unwrap_or(player_bank).clone(),
                            Owner::Player(player.key.clone()),
                        ));
                    }
                }
            }
        }
        if let Some(npcs) = self
            .observation
            .data()
            .npcs
            .as_ref()
            .filter(|n| !n.actors.0.is_empty())
        {
            let bank = npc_bank.context("NPC body sound bank absent")?;
            for actor in &npcs.actors.0 {
                active(cancel)?;
                let pose = motion.npc_sample(actor, bank, now)?;
                let track = candidate
                    .npcs
                    .get_mut(&actor.serial)
                    .context("NPC audio track absent")?;
                ensure!(
                    track.key.stamp == pose.stamp && now >= track.sampled,
                    "NPC cue stamp/clock mismatch"
                );
                let marker = (pose.action, pose.cycle, pose.source);
                let action =
                    bank.action(NpcAppearance::new(actor.graphic.try_into()?)?, pose.action)?;
                if !pose.stamp.historical_death
                    && track.marker != Some(marker)
                    && now.duration_since(track.sampled) <= MAX_AGE
                {
                    if let Some(group) = action.sound_group(pose.source)? {
                        let prefix = (1..action.source_frames().len())
                            .chain(std::iter::once(0))
                            .take_while(|s| *s != pose.source)
                            .try_fold(0u128, |sum, source| {
                                sum.checked_add(
                                    u128::from(
                                        action.source_frames()[source].duration_ticks * 1000 / 24,
                                    ) * 1_000_000,
                                )
                                .context("NPC cue phase overflow")
                            })?;
                        let nanos = pose
                            .cycle
                            .checked_mul(action.duration().as_nanos())
                            .and_then(|n| n.checked_add(prefix))
                            .context("NPC cue cycle overflow")?;
                        let created = pose.cue_time(Duration::from_nanos(u64::try_from(nanos)?))?;
                        ensure!(created <= now, "NPC cue precedes visible frame");
                        if now.duration_since(created) <= MAX_AGE {
                            pending.push((
                                created,
                                group.clone(),
                                bank.sound_bank(),
                                Owner::Npc(track.key.clone()),
                            ));
                        }
                    }
                }
                track.marker = Some(marker);
                track.sampled = now;
            }
        }
        pending.sort_by_key(|(created, _, _, _)| *created);
        for (created, group, bank, owner) in pending {
            ensure!(
                group
                    .candidates()
                    .iter()
                    .all(|id| *id == 0 || bank.get(*id).is_some()),
                "body cue missing qualified PCM"
            );
            if matches!(&owner, Owner::Walk(_)) {
                // The worker may run while UI delivery stalls. Withdraw the
                // previous queued foot for this step before publishing the
                // next one; already-started voices remain owned by the pool.
                candidate.owners.retain(|_, old| old != &owner);
                candidate
                    .log
                    .retain_sequences(|id| candidate.owners.contains_key(&id));
            }
            candidate.log.push_with_bank(group, created, Some(bank))?;
            candidate
                .owners
                .insert(candidate.log.last_sequence(), owner);
        }
        let events = candidate.log.snapshot(now);
        candidate
            .owners
            .retain(|id, _| events.iter().any(|event| event.sequence == *id));
        candidate.sampled = now;
        let output =
            (!player_bank.is_empty() || npc_bank.is_some() || !events.is_empty()).then(|| {
                AudioFrame {
                    bank: player_bank.clone(),
                    events,
                }
            });
        active(cancel)?;
        Ok((candidate, output))
    }
}

#[cfg(test)]
#[path = "battle_body_audio_tests.rs"]
pub(crate) mod tests;
