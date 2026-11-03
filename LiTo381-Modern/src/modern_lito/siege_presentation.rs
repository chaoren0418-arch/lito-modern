//! Three shared Lineage towers and explicit modern geometry markers. No LT art,
//! invented server clock, synthetic player target, or optimistic gate opening.
use super::{
    compositor::Canvas,
    shared_action_catalog::SharedActionCatalog,
    siege_protocol::{self as siege, Scope, Slot, Target},
    siege_space::Space,
    tower_assets::{HonorPlayback, TowerAssets},
    world_audio::AudioSequencer,
    world_map::{ViewProjection, WorldMap},
    world_protocol::{Frame, SettlementState},
    world_scene::{ObjectiveDraw, ObjectivePlacement},
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::{atomic::AtomicBool, Arc},
    time::{Duration, Instant},
};

pub(crate) fn draw_id(slot: Slot) -> u64 {
    300 + slot as u64
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub target: Target,
    pub alive: bool,
    pub geometry: ObjectivePlacement,
}
pub(crate) fn valid_placement(world: &Frame, p: &Placement) -> bool {
    world.siege.as_ref().is_some_and(|d| {
        let t = &d.objectives[p.target.slot as usize];
        p.target.scope == Scope::of(world)
            && p.geometry.entity_id == draw_id(p.target.slot)
            && p.geometry.cell == t.cell
            && p.geometry.graphics_id == t.graphic
            && p.alive == (t.hp > 0)
            && (1..=4096).contains(&p.geometry.width)
            && (1..=4096).contains(&p.geometry.height)
    })
}
pub(crate) fn valid_placements(world: &Frame, placements: &[Placement]) -> bool {
    placements.len() == if world.siege.is_some() { 3 } else { 0 }
        && placements
            .iter()
            .enumerate()
            .all(|(i, p)| p.target.slot as usize == i && valid_placement(world, p))
}
pub(crate) fn portal_ready(frame: &super::combat_protocol::Frame, entity: u64) -> bool {
    let w = &frame.world;
    w.phase == super::world_protocol::Phase::Active
        && frame.vital(entity).is_some_and(|v| v.hp > 0)
        && w.siege.as_ref().is_some_and(|d| {
            w.member(entity).is_some_and(|m| m.team != d.attacker)
                && w.actor(entity)
                    .is_some_and(|a| d.portal_destination(a.position).is_some())
        })
}
pub(crate) fn player_attack(
    world: &Frame,
    entity: u64,
) -> Option<super::npc_presentation::ConfirmedAttack> {
    world
        .siege
        .as_ref()?
        .objectives
        .iter()
        .filter(|t| t.last_actor == entity && t.hit_count > 0)
        .max_by_key(|t| t.revision)
        .and_then(|t| {
            Some(super::npc_presentation::ConfirmedAttack {
                revision: t.revision.checked_add(1)?,
                cell: Some(t.cell),
            })
        })
}
pub(crate) fn portalled(before: &Frame, after: &Frame, entity: u64) -> bool {
    let Some((a, b)) = before.actor(entity).zip(after.actor(entity)) else {
        return false;
    };
    Scope::of(before) == Scope::of(after)
        && b.sequence > a.sequence
        && after.siege.as_ref().is_some_and(|d| {
            before.siege.as_ref().is_some_and(|old| old.same_policy(d))
                && after.member(entity).is_some_and(|m| m.team != d.attacker)
                && d.portal_destination(a.position) == Some(b.position)
        })
}
pub(crate) fn result_notice(
    world: &Frame,
    key: super::application_protocol::CharacterKey,
) -> Option<String> {
    let d = world.siege.as_ref()?;
    let receipt = d.receipt.as_ref()?;
    let actor = world.actors.iter().find(|a| a.matches_character(key))?;
    let member = world.member(actor.entity_id)?;
    let verdict = if d.winner == Some(member.team) {
        "勝利"
    } else {
        "敗北"
    };
    let saved = match receipt.state {
        SettlementState::Pending => "保存尚未確認",
        SettlementState::Saved => "本場結果已保存（非累計場數）",
        SettlementState::Failed => "保存回覆失敗／可能已入帳，不可重複計入",
    };
    Some(format!("本場{verdict} · {saved}；可返回原房間。"))
}
pub(crate) fn hud(world: &Frame, entity: u64) -> Vec<String> {
    let Some(d) = &world.siege else {
        return Vec::new();
    };
    let role = if world.member(entity).is_some_and(|m| m.team == d.attacker) {
        "攻方"
    } else {
        "守方"
    };
    let phase = match d.phase {
        siege::Phase::Waiting => "等待開戰",
        siege::Phase::Outer => "摧毀兩座外塔",
        siege::Phase::Inner => "城門已開，內塔可攻擊",
        siege::Phase::Finished => "本場結束",
        siege::Phase::Aborted => "本場中止，沒有勝敗結果",
    };
    let seconds = (d.duration_millis + 999) / 1000;
    let mut lines = vec![
        format!(
            "攻城 · {role} · {phase} · 設定時限 {:02}:{:02}（非剩餘倒數）",
            seconds / 60,
            seconds % 60
        ),
        format!(
            "左外塔 {}/{} · 右外塔 {}/{} · 內塔 {}/{}",
            d.objectives[0].hp,
            d.objectives[0].max_hp,
            d.objectives[1].hp,
            d.objectives[1].max_hp,
            d.objectives[2].hp,
            d.objectives[2].max_hp
        ),
        "藍色菱形為傳送門：守方站上後點擊自身；紅色菱形為尚未開啟的城門。".into(),
    ];
    if let Some(a) = world.actor(entity) {
        if let Some(result) = result_notice(world, a.key) {
            lines.push(result);
        } else if d.lives.iter().any(|l| l.entity_id == entity && l.dead()) {
            lines.push("已陣亡，等待伺服器確認於陣營出生點復活。".into());
        }
    }
    lines
}

struct Visual {
    action: u8,
    started: Option<Instant>,
    playback: HonorPlayback,
}
pub(crate) struct Presentation {
    assets: TowerAssets,
    space: Space,
    visuals: Vec<Visual>,
}
impl Presentation {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        world: &Frame,
        map: Arc<WorldMap>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        let space = Space::qualify(map, world, cancel)?;
        let d = world.siege.as_ref().context("siege absent")?;
        let assets = TowerAssets::load_with_catalog(catalog, cancel)?;
        let visuals = d
            .objectives
            .iter()
            .map(|t| {
                let action = super::honor_protocol::action_for_hp(t.hp, t.max_hp);
                Ok(Visual {
                    action,
                    started: None,
                    playback: assets.start_state(t.graphic, action)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            assets,
            space,
            visuals,
        })
    }
    pub(crate) fn update(&mut self, w: &Frame, now: Instant) -> Result<()> {
        ensure!(self.space.matches(w), "siege presentation geometry changed");
        for (t, v) in w
            .siege
            .as_ref()
            .context("siege absent")?
            .objectives
            .iter()
            .zip(&mut self.visuals)
        {
            let action = super::honor_protocol::action_for_hp(t.hp, t.max_hp);
            if action != v.action {
                v.playback = self.assets.start_state(t.graphic, action)?;
                v.action = action;
                v.started = Some(now);
            }
        }
        Ok(())
    }
    pub(crate) fn draws<'a>(&'a self, w: &Frame, now: Instant) -> Result<Vec<ObjectiveDraw<'a>>> {
        ensure!(self.space.matches(w), "siege draw scope changed");
        let d = w.siege.as_ref().context("siege absent")?;
        Slot::ALL
            .into_iter()
            .map(|slot| {
                let t = &d.objectives[slot as usize];
                let v = &self.visuals[slot as usize];
                let elapsed = v
                    .started
                    .map_or(Duration::MAX, |s| now.saturating_duration_since(s));
                let body = self.assets.state_frame(t.graphic, v.action, elapsed)?;
                let shadow = self.assets.shadow_frame(t.graphic, v.action, elapsed)?;
                Ok(ObjectiveDraw {
                    entity_id: draw_id(slot),
                    blocking: true,
                    cell: t.cell,
                    frame: body.frame,
                    shadow: Some(shadow.frame),
                    graphics_id: body.graphics_id,
                    source_frame: body.source_frame,
                    sprite_name: body.sprite_name,
                    subframe: body.subframe,
                })
            })
            .collect()
    }
    pub(crate) fn placements(
        &self,
        w: &Frame,
        mut geometry: Vec<ObjectivePlacement>,
    ) -> Result<Vec<Placement>> {
        ensure!(geometry.len() == 3, "siege tower paint count mismatch");
        geometry.sort_by_key(|g| g.entity_id);
        let d = w.siege.as_ref().context("siege absent")?;
        let result: Vec<_> = Slot::ALL
            .into_iter()
            .zip(geometry)
            .map(|(slot, geometry)| Placement {
                target: Target {
                    scope: Scope::of(w),
                    slot,
                },
                alive: d.objectives[slot as usize].hp > 0,
                geometry,
            })
            .collect();
        ensure!(
            valid_placements(w, &result),
            "siege tower paint identity mismatch"
        );
        Ok(result)
    }
    pub(crate) fn audio(&mut self, sequencer: &mut AudioSequencer, now: Instant) -> Result<()> {
        for v in &mut self.visuals {
            if let Some(start) = v.started {
                let elapsed = now.saturating_duration_since(start);
                let sample = v.playback.advance(elapsed)?;
                if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                    sequencer.push_preloaded_cues(
                        &sample.crossed_cues,
                        self.assets.sound_bank(),
                        start,
                    )?;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn markers(
        &self,
        canvas: &mut Canvas,
        w: &Frame,
        projection: ViewProjection,
    ) -> Result<()> {
        ensure!(self.space.matches(w), "siege marker scope changed");
        let d = w.siege.as_ref().context("siege absent")?;
        for (cell, colour) in d
            .gate
            .iter()
            .copied()
            .map(|c| {
                (
                    c,
                    if d.open() {
                        [96, 192, 96]
                    } else {
                        [255, 72, 48]
                    },
                )
            })
            .chain([d.portal_outside, d.portal_inside].map(|c| (c, [64, 192, 255])))
        {
            let at = projection
                .cell_to_pixel(cell)
                .context("siege marker projection overflow")?;
            let rgba = marker(colour);
            canvas.blit_rgba(
                &rgba,
                49,
                25,
                at.x.checked_sub(24).context("siege marker x overflow")?,
                at.y.checked_sub(12).context("siege marker y overflow")?,
            )?;
        }
        Ok(())
    }
}
fn marker(colour: [u8; 3]) -> [u8; 49 * 25 * 4] {
    let mut rgba = [0; 49 * 25 * 4];
    for y in 0..25i32 {
        for x in 0..49i32 {
            let distance = (x - 24).abs() + 2 * (y - 12).abs();
            if (22..=24).contains(&distance) {
                let at = (y as usize * 49 + x as usize) * 4;
                rgba[at..at + 3].copy_from_slice(&colour);
                rgba[at + 3] = 230;
            }
        }
    }
    rgba
}
