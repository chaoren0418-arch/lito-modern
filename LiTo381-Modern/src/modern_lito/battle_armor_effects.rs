//! Owner-only selected armor proc visuals. Preserve server order/multiplicity and the
//! frozen event cell. No skill grant, body action, MP change or asset fallback.
use super::{
    battle_follow::Observation,
    battle_payload::{armor_presentation as p, data as d},
    compositor::Canvas,
    immune_to_harm_effect::ImmuneToHarmAssets,
    shared_action_catalog::SharedActionCatalog,
    sprite_actions::ActionKey,
    tower_assets::HonorPlayback,
    world_audio::{AudioFrame, EventLog, HISTORY_LIMIT},
    world_map::{ViewProjection, WorldCell, WorldMap},
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const MAX_EFFECTS: usize = 4 * p::MAX_VISUALS;
const MAX_AGE: Duration = Duration::from_millis(500);
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "armor effects cancelled");
    Ok(())
}
fn metadata(o: &Observation) -> Option<&p::Presentation> {
    o.data().armor_presentation()
}
fn owner_state(o: &Observation) -> Option<&super::battle_payload::armor::State> {
    o.data()
        .armor_actors()?
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)?
        .state
        .as_ref()
}
fn cell(c: &d::Cell) -> WorldCell {
    WorldCell { x: c.x, y: c.y }
}
pub(crate) fn check_map(o: &Observation, map: &WorldMap) -> Result<()> {
    if let Some(batch) = metadata(o).and_then(|p| p.batch.as_ref()) {
        ensure!(
            map.standing(cell(&batch.anchor.cell)),
            "armor event anchor outside shared terrain"
        );
    }
    Ok(())
}

pub(crate) struct Assets {
    required: Option<p::Requirements>,
    protection: Option<ImmuneToHarmAssets>,
}
impl Assets {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        o: &Observation,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(
            o.data().armor_actors().is_none() || metadata(o).is_some(),
            "armor presentation metadata required"
        );
        let required = metadata(o).map(|p| p.required.clone());
        let mut protection = None;
        if let Some(required) = &required {
            for &graphic in &required.graphics.0 {
                active(cancel)?;
                // Resolve the actual requested ID first, including missing11101.
                // An existing but unreviewed animation is not a qualified bank.
                catalog
                    .catalog()
                    .resolve_action(graphic.try_into()?, ActionKey::new(0))
                    .with_context(|| format!("armor graphic {graphic} has no shared action0"))?;
                ensure!(
                    graphic == p::SKILL_GRAPHIC,
                    "armor graphic {graphic} has unqualified native controls"
                );
                protection = Some(ImmuneToHarmAssets::load_with_catalog(catalog, cancel)?);
            }
        }
        active(cancel)?;
        Ok(Self {
            required,
            protection,
        })
    }
    pub(crate) fn compatible(&self, old: &Self) -> bool {
        self.required == old.required
            && match (&self.protection, &old.protection) {
                (None, None) => true,
                (Some(a), Some(b)) => {
                    a.catalog_sha256() == b.catalog_sha256()
                        && a.sprite_sources().eq(b.sprite_sources())
                        && a.sound_bank().ids().eq(b.sound_bank().ids())
                        && a.sound_bank().ids().all(|id| {
                            a.sound_bank().get(id).unwrap().sha256()
                                == b.sound_bank().get(id).unwrap().sha256()
                        })
                }
                _ => false,
            }
    }
    fn bank(&self, graphic: i32) -> Result<&ImmuneToHarmAssets> {
        ensure!(
            graphic == p::SKILL_GRAPHIC,
            "unqualified armor graphic {graphic}"
        );
        self.protection.as_ref().context("armor shared bank absent")
    }
}

#[derive(Clone)]
struct Effect {
    visual: p::Visual,
    anchor: d::Cell,
    started: Instant,
    duration: Duration,
    playback: HonorPlayback,
}
#[derive(Clone)]
pub(crate) struct Timeline {
    observation: Observation,
    observed: Instant,
    effects: Vec<Effect>,
    audio: EventLog,
    pub(crate) skipped: bool,
}
impl Timeline {
    pub(crate) fn new(observation: Observation, now: Instant) -> Self {
        // First attachment is a watermark, never a replay of historical procs.
        Self {
            observation,
            observed: now,
            effects: Vec::new(),
            audio: EventLog::default(),
            skipped: false,
        }
    }
    pub(crate) fn advance(
        &self,
        next: Observation,
        now: Instant,
        assets: &Assets,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        active(cancel)?;
        ensure!(now >= self.observed, "armor clock reversed");
        self.observation.check_successor(&next)?;
        ensure!(
            assets.required.as_ref() == metadata(&next).map(|p| &p.required),
            "armor resource requirements changed"
        );
        let mut out = self.clone();
        // Expired one-shots do not consume capacity for a fresh batch.
        out.effects
            .retain(|e| now.duration_since(e.started) < e.duration);
        let meta = metadata(&next);
        let alive = next
            .data()
            .actors
            .0
            .iter()
            .find(|a| a.entity == next.data().personal.entity)
            .and_then(|a| a.resources.as_ref())
            .is_some_and(|r| r.vitals.hp > 0);
        let new_life = metadata(&self.observation).map(|p| p.life) != meta.map(|p| p.life);
        if next.data().world.phase != d::Phase::Active || !alive || new_life {
            out.effects.clear();
            out.audio.retain_sequences(|_| false); // Keep the delivery sequence.
        }
        if next.data().world.phase == d::Phase::Active && alive {
            if let Some(m) = meta {
                let before = owner_state(&self.observation);
                let previous_sequence = before.map_or(0, |s| s.sequence);
                let sequence_delta = owner_state(&next)
                    .map_or(0, |s| s.sequence)
                    .checked_sub(previous_sequence);
                let fresh_batch = m
                    .batch
                    .as_ref()
                    .filter(|b| b.revision > self.observation.data().world.revision);
                // The armor sequence counts expiry as well as damage. A fresh
                // confirmed batch after the already-known deadline can contain
                // exactly one expiry + this hit in one publication. Do not
                // mistake that for a lost visual, but keep larger gaps visible.
                let expected_steps =
                    fresh_batch.map(|b| {
                        1 + i64::from(before.is_some_and(|s| {
                            s.until_nanos > 0 && b.observed_nanos >= s.until_nanos
                        }))
                    });
                if !new_life
                    && match expected_steps {
                        Some(expected) => sequence_delta != Some(expected),
                        // Without a fresh batch, an intervening proc may already
                        // have expired. Do not use a known expiry to conceal it.
                        None => sequence_delta.is_some_and(|n| n > 1),
                    }
                {
                    out.skipped = true;
                }
                if let Some(batch) = &m.batch {
                    if batch.revision > self.observation.data().world.revision {
                        if !batch.visuals.0.is_empty() {
                            let age = next
                                .data()
                                .world
                                .active_elapsed_nanos
                                .checked_sub(batch.observed_nanos)
                                .filter(|n| (0..=500_000_000).contains(n))
                                .map(|n| Duration::from_nanos(n as u64));
                            if let Some(age) = age {
                                if out.effects.len() + batch.visuals.0.len() > MAX_EFFECTS {
                                    out.skipped = true; // Admit an entire batch or none.
                                } else {
                                    let started = now
                                        .checked_sub(age)
                                        .context("armor timestamp underflow")?;
                                    for visual in &batch.visuals.0 {
                                        active(cancel)?;
                                        out.effects.push(Effect {
                                            visual: visual.clone(),
                                            anchor: batch.anchor.cell.clone(),
                                            started,
                                            duration: assets.bank(visual.graphic)?.duration(),
                                            playback: assets.bank(visual.graphic)?.start_effect(),
                                        });
                                    }
                                }
                            } else {
                                out.skipped = true;
                            }
                        }
                    }
                }
            }
        }
        let mut cues = out.audio.snapshot(now).len();
        for effect in &mut out.effects {
            active(cancel)?;
            let elapsed = now
                .checked_duration_since(effect.started)
                .context("armor sample reversed")?;
            let sample = effect.playback.advance(elapsed)?;
            for (tick, group) in super::world_audio::crossed_audio_groups(&sample.crossed_cues)? {
                let nanos = tick
                    .checked_mul(1_000_000_000)
                    .context("armor audio tick overflow")?
                    .div_ceil(24);
                let created = effect
                    .started
                    .checked_add(Duration::from_nanos(nanos))
                    .context("armor audio timestamp overflow")?;
                ensure!(created <= now, "future armor cue");
                if now.duration_since(created) > MAX_AGE {
                    out.skipped = true;
                    continue;
                }
                // EventLog is bounded. Never silently certify a truncated batch.
                if cues >= HISTORY_LIMIT {
                    out.skipped = true;
                }
                out.audio.push_with_bank(
                    group,
                    created,
                    Some(assets.bank(effect.visual.graphic)?.sound_bank()),
                )?;
                cues += 1;
            }
        }
        out.effects
            .retain(|e| now.duration_since(e.started) < e.duration);
        active(cancel)?;
        out.observation = next;
        out.observed = now;
        Ok(out)
    }
    pub(crate) fn draw(
        &self,
        canvas: &mut Canvas,
        projection: ViewProjection,
        assets: &Assets,
        now: Instant,
        cancel: &AtomicBool,
    ) -> Result<()> {
        active(cancel)?;
        ensure!(now >= self.observed, "armor draw clock reversed");
        for effect in &self.effects {
            active(cancel)?;
            let anchor = projection
                .cell_to_pixel(cell(&effect.anchor))
                .context("armor projection overflow")?;
            assets.bank(effect.visual.graphic)?.draw_effect(
                canvas,
                anchor,
                now.duration_since(effect.started),
            )?;
        }
        Ok(())
    }
    pub(crate) fn audio(&mut self, assets: &Assets, now: Instant) -> Option<AudioFrame> {
        assets.protection.as_ref().map(|bank| AudioFrame {
            bank: bank.sound_bank(),
            events: self.audio.snapshot(now),
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use std::{collections::BTreeMap, path::Path};
    pub(crate) fn verify_vip_missing_assets(root: &Path, frames: &[(String, Observation)]) {
        let cancel = AtomicBool::new(false);
        let catalog = SharedActionCatalog::load(root, &cancel).unwrap();
        for source in [7, 42] {
            let o = &frames
                .iter()
                .find(|(name, _)| name == &format!("armor-loading-{source}"))
                .unwrap()
                .1;
            let raw = o.data().clone();
            let v = raw.vip.as_ref().unwrap();
            assert!(raw.armor.is_none());
            assert_eq!(raw.armor_actors(), Some(&v.armor));
            assert_eq!(raw.armor_presentation(), v.presentation.as_ref());
            assert!(metadata(o).unwrap().required.graphics.0.contains(&11101));
            let error = Assets::load(&catalog, o, &cancel)
                .err()
                .expect("11101 must not disappear from VIP requirements");
            assert!(format!("{error:#}").contains("11101"));
            assert_eq!(o.data(), &raw);
        }
    }
    pub(crate) fn verify_vip(root: &Path, frames: &[(String, Observation)]) {
        use super::super::{battle_protection, immune_to_harm_icon::ImmuneToHarmIcon};
        let cancel = AtomicBool::new(false);
        let catalog = SharedActionCatalog::load(root, &cancel).unwrap();
        let icon = ImmuneToHarmIcon::load(root, &cancel).unwrap();
        let mut groups: BTreeMap<String, Vec<&(String, Observation)>> = BTreeMap::new();
        for frame in frames {
            let o = &frame.1;
            groups
                .entry(format!(
                    "{}-{}",
                    frame.0.split('-').next().unwrap(),
                    o.data().personal.player.source
                ))
                .or_default()
                .push(frame);
        }
        let mut proc_frames = 0;
        let mut icon_frames = 0;
        let mut missing_frames = 0;
        for trace in groups.values() {
            let at = Instant::now();
            let base = trace[0].1.clone();
            if metadata(&base).is_some_and(|p| p.required.graphics.0.contains(&11101)) {
                // The separate gfx0 fixture changes the armor/healing trace
                // only. Haste/slow fixtures keep their original missing rule;
                // do not silently qualify them or rewrite their requirements.
                assert!(trace[0].0.starts_with("haste") || trace[0].0.starts_with("slow"));
                for (name, o) in trace {
                    let error = Assets::load(&catalog, o, &cancel)
                        .err()
                        .expect("missing full-VIP armor resources must reject");
                    assert!(format!("{error:#}").contains("11101"), "{name}");
                    missing_frames += 1;
                }
                continue;
            }
            let assets = Assets::load(&catalog, &base, &cancel)
                .unwrap_or_else(|e| panic!("{}: {e:#}", trace[0].0));
            let icons = battle_protection::Assets::load(root, &base, &cancel).unwrap();
            let mut timeline = Timeline::new(base.clone(), at);
            let mut status = battle_protection::Timeline::new(base, at, &cancel).unwrap();
            for (name, o) in trace {
                let raw = o.data().clone();
                let now = at + Duration::from_nanos(raw.world.active_elapsed_nanos.max(0) as u64);
                timeline = timeline.advance(o.clone(), now, &assets, &cancel).unwrap();
                status = status.advance(o.clone(), now, &cancel).unwrap();
                let state = owner_state(o);
                let expected = state
                    .filter(|s| {
                        raw.world.phase == d::Phase::Active
                            && s.until_nanos > raw.world.active_elapsed_nanos
                    })
                    .map(|s| {
                        Duration::from_nanos(
                            (s.until_nanos - raw.world.active_elapsed_nanos) as u64,
                        )
                    });
                assert_eq!(
                    status.remaining(raw.personal.entity, now).unwrap(),
                    expected,
                    "{name}"
                );
                let mut status_pixels = Canvas::new(64, 64).unwrap();
                let mut expected_pixels = Canvas::new(64, 64).unwrap();
                status
                    .draw(&mut status_pixels, &icons, now, &cancel, (9, 11))
                    .unwrap();
                if expected.is_some() {
                    icon.draw(&mut expected_pixels, 9, 11).unwrap();
                    icon_frames += 1;
                }
                assert_eq!(
                    status_pixels.rgba(),
                    expected_pixels.rgba(),
                    "exact shared status pixels {name}"
                );
                let sequence = timeline.audio.last_sequence();
                let duplicate = timeline.advance(o.clone(), now, &assets, &cancel).unwrap();
                assert_eq!(
                    duplicate.audio.last_sequence(),
                    sequence,
                    "no replay {name}"
                );
                assert_eq!(duplicate.effects.len(), timeline.effects.len());
                assert_eq!(o.data(), &raw, "no authority mutation {name}");
                assert!(timeline
                    .advance(o.clone(), now, &assets, &AtomicBool::new(true))
                    .is_err());
                assert!(status
                    .advance(o.clone(), now - Duration::from_nanos(1), &cancel)
                    .is_err());
                if name == "armor-bow-42" {
                    proc_frames += 1;
                    assert_eq!(timeline.effects.len(), 1);
                    assert_eq!(expected, Some(Duration::from_secs(3)));
                    assert!(!timeline.skipped);
                    let effect = &timeline.effects[0];
                    let projection = ViewProjection {
                        anchor_cell: cell(&effect.anchor),
                        anchor_pixel: super::super::world_map::PixelPoint { x: 128, y: 128 },
                    };
                    let mut pixels = Canvas::new(256, 256).unwrap();
                    let mut reference = Canvas::new(256, 256).unwrap();
                    let blank = pixels.rgba().to_vec();
                    timeline
                        .draw(&mut pixels, projection, &assets, now, &cancel)
                        .unwrap();
                    assets
                        .bank(228)
                        .unwrap()
                        .draw_effect(&mut reference, projection.anchor_pixel, Duration::ZERO)
                        .unwrap();
                    assert_eq!(pixels.rgba(), reference.rgba());
                    assert_ne!(pixels.rgba(), blank);
                    let audio = timeline.audio(&assets, now).unwrap();
                    assert_eq!(audio.events.len(), 1);
                    assert_eq!(audio.events[0].group.candidates(), &[132]);
                    let later = now + Duration::from_secs(3);
                    let expired = status.advance(o.clone(), later, &cancel).unwrap();
                    assert_eq!(expired.remaining(raw.personal.entity, later).unwrap(), None);
                    let mut expired_pixels = Canvas::new(64, 64).unwrap();
                    let blank_status = expired_pixels.rgba().to_vec();
                    expired
                        .draw(&mut expired_pixels, &icons, later, &cancel, (9, 11))
                        .unwrap();
                    assert_eq!(expired_pixels.rgba(), blank_status);
                    if let Some(path) = std::env::var_os("LITO_VIP_ARMOR_EVIDENCE_PNG") {
                        let path = std::path::PathBuf::from(path);
                        assert!(path.is_absolute() && path.parent().unwrap().is_dir());
                        let mut canvas = Canvas::new(512, 256).unwrap();
                        canvas.clear([18, 24, 34]);
                        timeline
                            .draw(&mut canvas, projection, &assets, now, &cancel)
                            .unwrap();
                        status
                            .draw(&mut canvas, &icons, now, &cancel, (9, 11))
                            .unwrap();
                        let ended = timeline
                            .advance(o.clone(), later, &assets, &cancel)
                            .unwrap();
                        let mut right = Canvas::new(256, 256).unwrap();
                        right.clear([24, 30, 40]);
                        ended
                            .draw(&mut right, projection, &assets, later, &cancel)
                            .unwrap();
                        expired
                            .draw(&mut right, &icons, later, &cancel, (9, 11))
                            .unwrap();
                        canvas.blit_rgba(right.rgba(), 256, 256, 256, 0).unwrap();
                        let file = std::fs::OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(path)
                            .unwrap();
                        let mut encoder = png::Encoder::new(file, 512, 256);
                        encoder.set_color(png::ColorType::Rgba);
                        encoder.set_depth(png::BitDepth::Eight);
                        encoder
                            .write_header()
                            .unwrap()
                            .write_image_data(canvas.rgba())
                            .unwrap();
                    }
                    assert_eq!(o.data(), &raw, "local icon expiry is not server expiry");
                    let fresh = Timeline::new(o.clone(), now)
                        .advance(o.clone(), now, &assets, &cancel)
                        .unwrap();
                    assert!(fresh.effects.is_empty(), "late attachment does not replay");
                    let foreign = frames
                        .iter()
                        .find(|(label, _)| label == "armor-bow-7")
                        .unwrap()
                        .1
                        .clone();
                    assert!(timeline
                        .advance(foreign.clone(), now, &assets, &cancel)
                        .is_err());
                    assert!(status.advance(foreign, now, &cancel).is_err());
                    assert_eq!(timeline.audio.last_sequence(), sequence);
                }
                if name == "armor-bow-7" {
                    assert!(
                        timeline.effects.is_empty(),
                        "observer must not receive target's private proc"
                    );
                    assert!(expected.is_none(), "observer has no self protection icon");
                }
            }
        }
        assert_eq!(proc_frames, 1);
        assert_eq!(icon_frames, 2);
        assert_eq!(missing_frames, 18);
    }
    #[test]
    fn ordinary_world_never_invents_armor_assets_effects_or_cues() {
        let o = super::super::battle_scene::tests::observation(
            super::super::battle_payload::data_tests::fixture(),
        );
        let assets = Assets {
            required: None,
            protection: None,
        };
        let now = Instant::now();
        let timeline = Timeline::new(o.clone(), now);
        let mut next = timeline
            .advance(o.clone(), now, &assets, &AtomicBool::new(false))
            .unwrap();
        assert!(next.effects.is_empty() && !next.skipped && next.audio(&assets, now).is_none());
        assert!(timeline
            .advance(o.clone(), now, &assets, &AtomicBool::new(true))
            .is_err());
        assert!(timeline
            .advance(
                o,
                now - Duration::from_nanos(1),
                &assets,
                &AtomicBool::new(false)
            )
            .is_err());
    }
    pub(crate) fn verify_lifecycle(root: &Path, frames: &[(String, Observation)]) {
        use super::super::battle_protection;
        let cancel = AtomicBool::new(false);
        let catalog = SharedActionCatalog::load(root, &cancel).unwrap();
        for (prefix, owner) in [("gear-", "-0"), ("life-", "-1")] {
            let trace = frames
                .iter()
                .filter(|(name, _)| name.starts_with(prefix) && name.ends_with(owner))
                .collect::<Vec<_>>();
            let at = Instant::now() + Duration::from_secs(60);
            let base = trace[0].1.clone();
            let assets = Assets::load(&catalog, &base, &cancel).unwrap();
            let icons = battle_protection::Assets::load(root, &base, &cancel).unwrap();
            let mut timeline = Timeline::new(base.clone(), at);
            let mut status = battle_protection::Timeline::new(base, at, &cancel).unwrap();
            let mut old_sequence = 0;
            let mut now = at;
            if prefix == "life-" {
                let find = |key: &str| {
                    frames
                        .iter()
                        .find(|(name, _)| name == key)
                        .unwrap()
                        .1
                        .clone()
                };
                let first = find("life-hit-1");
                let next = find("life-damage2-1");
                // One known expiry cannot excuse TWO unobserved damage steps.
                let gap = Timeline::new(first, at)
                    .advance(
                        next.clone(),
                        at + Duration::from_nanos(next.data().world.active_elapsed_nanos as u64),
                        &assets,
                        &cancel,
                    )
                    .unwrap();
                assert!(
                    gap.skipped,
                    "coalesced hits across expiry still signal lost history"
                );
            }
            for (name, o) in trace {
                // LOADING uses -1 (not started), not an unsigned duration.
                // The local presentation clock stays monotonic across phases.
                now = now.max(
                    at + Duration::from_nanos(o.data().world.active_elapsed_nanos.max(0) as u64),
                );
                timeline = timeline
                    .advance(o.clone(), now, &assets, &cancel)
                    .unwrap_or_else(|e| panic!("{name}: {e:#}"));
                status = status.advance(o.clone(), now, &cancel).unwrap();
                assert!(!timeline.skipped, "complete life/gear sequence {name}");
                assert!(timeline.audio.last_sequence() >= old_sequence);
                old_sequence = timeline.audio.last_sequence();
                let remaining = status.remaining(o.data().personal.entity, now).unwrap();
                if [
                    "gear-hit-0",
                    "gear-off-0",
                    "gear-heal-0",
                    "gear-again-0",
                    "life-again-1",
                ]
                .contains(&name.as_str())
                {
                    assert_eq!(remaining, Some(Duration::from_secs(3)), "lease {name}");
                    assert_eq!(timeline.effects.len(), 1, "active animation {name}");
                }
                if name == "gear-offhit-0" || name == "gear-on-0" {
                    assert_eq!(
                        remaining,
                        Some(Duration::from_secs(2)),
                        "no extension {name}"
                    );
                    assert!(timeline.effects.is_empty(), "old event must age out");
                }
                if [
                    "life-dead-1",
                    "life-revived-1",
                    "life-finished-1",
                    "gear-abort-0",
                    "gear-expired-0",
                ]
                .contains(&name.as_str())
                {
                    assert!(remaining.is_none(), "no prior lease {name}");
                    assert!(timeline.effects.is_empty(), "no prior animation {name}");
                    assert!(
                        timeline.audio(&assets, now).unwrap().events.is_empty(),
                        "no prior cues {name}"
                    );
                }
                let mut canvas = Canvas::new(64, 64).unwrap();
                let blank = canvas.rgba().to_vec();
                status
                    .draw(&mut canvas, &icons, now, &cancel, (0, 0))
                    .unwrap();
                assert_eq!(
                    canvas.rgba() != blank,
                    remaining.is_some(),
                    "status pixels {name}"
                );
                let duplicate = timeline.advance(o.clone(), now, &assets, &cancel).unwrap();
                assert_eq!(duplicate.audio.last_sequence(), old_sequence);
            }
        }
    }
    pub(crate) fn verify_trace(root: &Path, frames: &BTreeMap<String, Observation>) {
        let cancel = AtomicBool::new(false);
        let catalog = SharedActionCatalog::load(root, &cancel).unwrap();
        let now = Instant::now() + Duration::from_secs(60);
        for graphic in [0, 228] {
            let get = |stage: &str| frames[&format!("gfx{graphic}-{stage}-1")].clone();
            let base = get("active");
            let assets = Assets::load(&catalog, &base, &cancel).unwrap();
            assert!(assets.compatible(&Assets::load(&catalog, &base, &cancel).unwrap()));
            let baseline = Timeline::new(base.clone(), now);
            let hit = get("hit");
            let expected = if graphic == 0 { 1 } else { 5 };
            let mut t = baseline
                .advance(hit.clone(), now, &assets, &cancel)
                .unwrap();
            assert_eq!(t.effects.len(), expected);
            assert!(!t.skipped);
            assert!(
                baseline
                    .advance(get("again"), now + Duration::from_secs(1), &assets, &cancel)
                    .unwrap()
                    .skipped
            );
            assert!(
                baseline
                    .advance(
                        get("expired"),
                        now + Duration::from_secs(4),
                        &assets,
                        &cancel
                    )
                    .unwrap()
                    .skipped,
                "expired latest batches cannot conceal coalesced damage/proc history"
            );
            assert_eq!(
                t.effects
                    .iter()
                    .map(|e| e.visual.clone())
                    .collect::<Vec<_>>(),
                metadata(&hit).unwrap().batch.as_ref().unwrap().visuals.0
            );
            let audio = t.audio(&assets, now).unwrap();
            assert_eq!(audio.events.len(), expected);
            assert!(audio.events.iter().all(|e| e.group.candidates() == [132]));
            let projection = ViewProjection {
                anchor_cell: cell(&t.effects[0].anchor),
                anchor_pixel: super::super::world_map::PixelPoint { x: 128, y: 128 },
            };
            let mut pixels = Canvas::new(256, 256).unwrap();
            let blank = pixels.rgba().to_vec();
            t.draw(&mut pixels, projection, &assets, now, &cancel)
                .unwrap();
            assert_ne!(pixels.rgba(), blank);
            let original = t.effects[0].anchor.clone();
            let moved = get("moved");
            assert_ne!(
                moved
                    .data()
                    .actors
                    .0
                    .iter()
                    .find(|a| a.entity == moved.data().personal.entity)
                    .unwrap()
                    .position,
                original
            );
            let later = now + Duration::from_millis(200);
            let moved_t = t.advance(moved.clone(), later, &assets, &cancel).unwrap();
            assert!(moved_t
                .effects
                .iter()
                .all(|e| e.anchor == original && e.started == now));
            assert_eq!(moved_t.audio.last_sequence(), expected as u64);
            let mut moved_pixels = Canvas::new(256, 256).unwrap();
            let mut reference = Canvas::new(256, 256).unwrap();
            moved_t
                .draw(&mut moved_pixels, projection, &assets, later, &cancel)
                .unwrap();
            for _ in 0..expected {
                assets
                    .bank(228)
                    .unwrap()
                    .draw_effect(
                        &mut reference,
                        projection.anchor_pixel,
                        Duration::from_millis(200),
                    )
                    .unwrap();
            }
            assert_eq!(
                moved_pixels.rgba(),
                reference.rgba(),
                "fixed event pixel position, not moving body"
            );
            let mut replay = moved_t
                .advance(moved.clone(), later, &assets, &cancel)
                .unwrap();
            assert_eq!(replay.audio.last_sequence(), expected as u64);
            assert_eq!(replay.audio(&assets, later).unwrap().events.len(), expected);
            assert!(moved_t
                .advance(moved.clone(), later, &assets, &AtomicBool::new(true))
                .is_err());
            assert!(moved_t
                .advance(base.clone(), later, &assets, &cancel)
                .is_err());
            let foreign = frames[&format!("gfx{graphic}-moved-0")].clone();
            assert!(moved_t.advance(foreign, later, &assets, &cancel).is_err());
            // Initial or replacement construction suppresses history; carry is
            // the only way to transfer a still-playing effect to a new bank.
            let mut attached = Timeline::new(hit.clone(), now)
                .advance(hit.clone(), now, &assets, &cancel)
                .unwrap();
            assert!(
                attached.effects.is_empty()
                    && attached.audio(&assets, now).unwrap().events.is_empty()
            );
            let mut delayed = baseline
                .advance(
                    get("back"),
                    now + Duration::from_millis(400),
                    &assets,
                    &cancel,
                )
                .unwrap();
            assert_eq!(delayed.effects.len(), expected);
            assert_eq!(delayed.effects[0].started, now);
            assert_eq!(
                delayed
                    .audio(&assets, now + Duration::from_millis(400))
                    .unwrap()
                    .events[0]
                    .created,
                now
            );
            let expired = moved_t
                .advance(
                    moved.clone(),
                    now + Duration::from_secs(1),
                    &assets,
                    &cancel,
                )
                .unwrap();
            assert!(expired.effects.is_empty());
            assert_eq!(expired.audio.last_sequence(), expected as u64);
            let mut aborted = t
                .advance(get("abort"), now + Duration::from_secs(4), &assets, &cancel)
                .unwrap();
            assert!(
                aborted.effects.is_empty()
                    && aborted
                        .audio(&assets, now + Duration::from_secs(4))
                        .unwrap()
                        .events
                        .is_empty()
            );
            assert_eq!(aborted.audio.last_sequence(), expected as u64);
            let mut capacity = baseline.clone();
            capacity.effects = vec![t.effects[0].clone(); MAX_EFFECTS];
            let full = capacity
                .advance(hit.clone(), now, &assets, &cancel)
                .unwrap();
            assert!(full.skipped);
            assert_eq!(
                full.effects.len(),
                MAX_EFFECTS,
                "no partially admitted batch"
            );
            let mut slow = t
                .advance(hit, now + Duration::from_secs(2), &assets, &cancel)
                .unwrap();
            assert!(
                slow.effects.is_empty()
                    && slow
                        .audio(&assets, now + Duration::from_secs(2))
                        .unwrap()
                        .events
                        .is_empty()
            );
        }
        let assets = Assets::load(&catalog, &frames["max-active-1"], &cancel).unwrap();
        let mut max = Timeline::new(frames["max-active-1"].clone(), now)
            .advance(frames["max-hit-1"].clone(), now, &assets, &cancel)
            .unwrap();
        assert_eq!(max.effects.len(), p::MAX_VISUALS);
        assert_eq!(max.audio.last_sequence(), p::MAX_VISUALS as u64);
        assert_eq!(max.audio(&assets, now).unwrap().events.len(), HISTORY_LIMIT);
        assert!(
            max.skipped,
            "2049 visuals fit but 2049 simultaneous cues must report the bounded audio loss"
        );
    }
}
