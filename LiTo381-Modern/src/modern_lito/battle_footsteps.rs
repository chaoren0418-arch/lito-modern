//! Shared walk markers on the pixel clock; no invented motion or catch-up loop.
use super::*;
use crate::modern_lito::{
    combat_animation::classify_native_cues, fixed_walk::QualifiedFixedWalk, world_map::WorldCell,
};

#[derive(Clone)]
pub(super) struct Track {
    pub key: PlayerKey,
    pub cues: Vec<Cue>,
    pub spatial: (u8, u8),
}

pub(super) fn schedule(
    walk: &QualifiedFixedWalk,
    step: u64,
    clock: &PacedClock,
    now: Instant,
) -> Result<Vec<Cue>> {
    ensure!(
        step > 0 && step <= 10_000_000_000,
        "invalid footstep interval"
    );
    let mut prefix = 0u128;
    let mut cues = Vec::new();
    for source in (1..walk.source_frames().len()).chain(std::iter::once(0)) {
        if let Some(group) = classify_native_cues(&walk.source_frames()[source].events)?.audio {
            // Exact inverse of player_walk's floor(elapsed * cycle / step).
            // Native integer-ms frame lengths, not generic source tick timing.
            let phase = Duration::from_nanos(u64::try_from(
                (prefix * u128::from(step)).div_ceil(walk.step_duration().as_nanos()),
            )?);
            cues.push(Cue {
                phase,
                group,
                deadline: clock.clone().until(now, phase)?,
                consumed: false,
            });
        }
        prefix += walk
            .source_duration(source)
            .context("walk duration absent")?
            .as_nanos();
    }
    ensure!(
        prefix == walk.step_duration().as_nanos(),
        "walk schedule mismatch"
    );
    Ok(cues)
}

/// sub_4E5C00/loc_4E61D6 and sub_4C79D0: selected minus emitter, whole-cell dx
/// (native raw X is doubled), clamp dx+dy, native 0..127 volume / centred-64 pan.
/// At the audible boundary |dx|/|dy| > 13 is already silent, so reject extreme
/// coordinates before squaring. This is not `^` audio-mode emulation.
pub(super) fn spatial(listener: WorldCell, emitter: WorldCell) -> (u8, u8) {
    let mut dx = i64::from(listener.x) - i64::from(emitter.x);
    let mut dy = i64::from(listener.y) - i64::from(emitter.y);
    if dx.abs() > 13 || dy.abs() > 13 {
        return (0, 64);
    }
    if dx + dy > 14 {
        dx = 14;
        dy = 0;
    } else if dx + dy < -14 {
        dx = -14;
        dy = 0;
    }
    let pan = 64 - 63 * (dx + dy) / 14;
    let volume = (127 - 7 * (dx * dx + dy * dy) / 10).max(0);
    (volume as u8, pan as u8)
}

/// At most 256 retained events * 256 KiB playback PCM. Original WAV storage is
/// shared with preflight; no reads, path fallback or uncapped per-actor cache.
pub(super) fn bank(
    source: &Arc<SoundBank>,
    group: &NativeAudioGroup,
    spatial: (u8, u8),
    cancel: &AtomicBool,
) -> Result<Arc<SoundBank>> {
    let mut sounds = BTreeMap::new();
    let mut bytes = 0usize;
    for id in group.candidates().iter().copied().filter(|id| *id != 0) {
        active(cancel)?;
        ensure!([478, 479].contains(&id), "unqualified footstep PCM");
        if sounds.contains_key(&id) {
            continue;
        }
        let sound = source.get(id).context("footstep missing qualified PCM")?;
        ensure!(
            sound.spatial_gain().is_none(),
            "footstep bank already spatialized"
        );
        bytes = bytes
            .checked_add(sound.pcm_bytes().len() * (2 / usize::from(sound.format().channels)))
            .context("footstep PCM size overflow")?;
        ensure!(bytes <= 256 * 1024, "footstep PCM budget exceeded");
        sounds.insert(id, sound.with_spatial(spatial.0, spatial.1)?);
    }
    Ok(Arc::new(SoundBank::playback_subset(sounds)?))
}

#[cfg(test)]
#[path = "battle_footsteps_tests.rs"]
pub(crate) mod tests;
