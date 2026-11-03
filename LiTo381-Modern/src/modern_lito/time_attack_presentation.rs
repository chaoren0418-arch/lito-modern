//! Shared activation-tower art, owned by the render worker. The draw key is
//! private: clicks retain TowerId and never become PC/NPC/Quest identities.
use super::{
    npc_protocol::Scope,
    shared_action_catalog::SharedActionCatalog,
    time_attack_protocol::{Data, TowerId},
    tower_assets::{HonorPlayback, TowerAssets},
    world_audio::AudioSequencer,
    world_map::WorldMap,
    world_protocol::Frame,
    world_scene::{ObjectiveDraw, ObjectivePlacement},
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};

pub(crate) const DRAW_ID: u64 = 200;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placement {
    pub id: TowerId,
    pub alive: bool,
    pub geometry: ObjectivePlacement,
}

pub fn target(world: &Frame, id: TowerId) -> Option<&Data> {
    (id.scope == super::quest_protocol::scope(world))
        .then_some(world.time_attack.as_ref())
        .flatten()
}

pub(crate) fn valid_placement(world: &Frame, p: &Placement) -> bool {
    target(world, p.id).is_some_and(|d| {
        p.geometry.entity_id == DRAW_ID
            && p.geometry.cell == d.tower_cell
            && p.geometry.graphics_id == d.tower_graphic
            && p.alive == (d.tower_hp > 0)
            && p.geometry.width > 0
            && p.geometry.width <= 4096
            && p.geometry.height > 0
            && p.geometry.height <= 4096
    })
}

/// Result is per frozen source/object identity, not a synthetic winning team.
pub(crate) fn result_notice(
    world: &Frame,
    key: super::application_protocol::CharacterKey,
) -> Option<String> {
    let d = world.time_attack.as_ref()?;
    let receipt = d.receipt.as_ref()?;
    let actor = world.actors.iter().find(|a| a.matches_character(key))?;
    let verdict = if d.won(actor.entity_id) {
        "勝利"
    } else {
        "敗北"
    };
    let saved = match receipt.state {
        super::world_protocol::SettlementState::Pending => "保存尚未確認",
        super::world_protocol::SettlementState::Saved => "本場結果已保存（非累計場數）",
        super::world_protocol::SettlementState::Failed => "保存回覆失敗／可能已入帳，不可重複計入",
    };
    Some(format!("本場{verdict} · {saved}；可返回原房間。"))
}

/// All times below are the last server observation. Rendering never advances
/// a wave, declares expiry or invents a successful SQL receipt.
pub(crate) fn hud(world: &Frame, entity: u64) -> Vec<String> {
    use super::time_attack_protocol::{End, Kind, Phase};
    let Some(d) = &world.time_attack else {
        return Vec::new();
    };
    let short = |s: &str| {
        if s.chars().count() > 16 {
            format!("{}…", s.chars().take(16).collect::<String>())
        } else {
            s.to_owned()
        }
    };
    let leader = world
        .member(u64::from(d.leader))
        .map(|m| format!("{}({})", short(&m.name), short(&m.source_name)))
        .unwrap_or_else(|| "未確認".into());
    let phase = match d.phase {
        Phase::WaitingForTower => "擊破啟動塔後開始",
        Phase::SpawnPending => "等待下一波",
        Phase::Fighting => "戰鬥中",
        Phase::Aborted => "本場中止，沒有勝敗結果",
        Phase::Finished => match d.end {
            End::Cleared => "全部波次完成",
            End::Timeout => "時間到",
            End::LeaderDefeated => "隊長陣亡",
            End::PartyDefeated => "隊伍已離場",
            End::None => "結果未確認",
        },
    };
    let kind = match d.waves[usize::from(d.wave)] {
        Kind::Ordinary => "普通",
        Kind::FinalBosses => "末波",
        Kind::Dragon => "龍波",
    };
    let elapsed = if d.started_millis < 0 {
        0
    } else {
        d.observed_millis - d.started_millis
    };
    let remaining = d.duration_millis.saturating_sub(elapsed).max(0);
    let seconds = (remaining + 999) / 1000;
    let mut lines = vec![
        format!(
            "合作挑戰 · {phase} · 波次 {}/{}（{kind}）· 存活怪物 {}",
            u16::from(d.wave) + 1,
            d.waves.len(),
            d.living.len()
        ),
        format!(
            "隊長：{leader} · 留場存活 {} · 離隊 {} · {} {:02}:{:02}",
            (d.alive_mask & !d.departed_mask).count_ones(),
            d.departed_mask.count_ones(),
            if d.started_millis < 0 {
                "時限"
            } else {
                "伺服器剩餘"
            },
            seconds / 60,
            seconds % 60
        ),
    ];
    if let Some(actor) = world.actor(entity) {
        if let Some(result) = result_notice(world, actor.key) {
            lines.push(result);
        } else if d.departed(entity) {
            lines.push("你已離隊，本場不會計為勝利。".into());
        } else if d.alive_mask & (1 << (entity - 1)) == 0 {
            lines.push("你已陣亡；其他存活隊員可繼續，個人結果等待伺服器判定。".into());
        }
    }
    lines
}

pub(crate) struct Presentation {
    assets: TowerAssets,
    scope: Scope,
    baseline: Data,
    action: u8,
    started: Option<Instant>,
    playback: HonorPlayback,
}
impl Presentation {
    pub(crate) fn load(
        catalog: &SharedActionCatalog,
        frame: &Frame,
        map: &WorldMap,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        super::time_attack_protocol::validate_world(frame)?;
        let d = frame
            .time_attack
            .as_ref()
            .context("cooperative tower absent")?;
        ensure!(
            map.standing(d.tower_cell),
            "cooperative tower outside shared geometry"
        );
        let assets = TowerAssets::load_with_catalog(catalog, cancel)?;
        let action = super::honor_protocol::action_for_hp(d.tower_hp, d.tower_max_hp);
        let playback = assets.start_state(d.tower_graphic, action)?;
        Ok(Self {
            assets,
            scope: super::quest_protocol::scope(frame),
            baseline: d.clone(),
            action,
            started: None,
            playback,
        })
    }
    pub(crate) fn update(&mut self, frame: &Frame, now: Instant) -> Result<()> {
        let d = target(frame, TowerId { scope: self.scope })
            .context("cooperative tower scope changed")?;
        let b = &self.baseline;
        ensure!(
            d.plan_sha256 == b.plan_sha256
                && d.waves == b.waves
                && d.duration_millis == b.duration_millis
                && d.tower_cell == b.tower_cell
                && d.tower_graphic == b.tower_graphic
                && d.tower_max_hp == b.tower_max_hp,
            "cooperative tower preflight changed"
        );
        let action = super::honor_protocol::action_for_hp(d.tower_hp, d.tower_max_hp);
        if action != self.action {
            self.playback = self.assets.start_state(d.tower_graphic, action)?;
            self.action = action;
            self.started = Some(now);
        }
        Ok(())
    }
    fn elapsed(&self, now: Instant) -> Duration {
        self.started
            .map_or(Duration::MAX, |start| now.saturating_duration_since(start))
    }
    pub(crate) fn draw<'a>(&'a self, frame: &Frame, now: Instant) -> Result<ObjectiveDraw<'a>> {
        let d = target(frame, TowerId { scope: self.scope }).context("cooperative tower absent")?;
        let elapsed = self.elapsed(now);
        let body = self
            .assets
            .state_frame(d.tower_graphic, self.action, elapsed)?;
        let shadow = self
            .assets
            .shadow_frame(d.tower_graphic, self.action, elapsed)?;
        Ok(ObjectiveDraw {
            entity_id: DRAW_ID,
            blocking: d.tower_hp > 0,
            cell: d.tower_cell,
            frame: body.frame,
            shadow: Some(shadow.frame),
            graphics_id: body.graphics_id,
            source_frame: body.source_frame,
            sprite_name: body.sprite_name,
            subframe: body.subframe,
        })
    }
    pub(crate) fn placement(
        &self,
        frame: &Frame,
        mut geometry: Vec<ObjectivePlacement>,
    ) -> Result<Placement> {
        ensure!(
            geometry.len() == 1,
            "cooperative tower paint count mismatch"
        );
        let p = Placement {
            id: TowerId { scope: self.scope },
            alive: frame
                .time_attack
                .as_ref()
                .context("cooperative tower absent")?
                .tower_hp
                > 0,
            geometry: geometry.remove(0),
        };
        ensure!(
            valid_placement(frame, &p),
            "cooperative tower paint mismatch"
        );
        Ok(p)
    }
    pub(crate) fn audio(&mut self, sequencer: &mut AudioSequencer, now: Instant) -> Result<()> {
        if let Some(started) = self.started {
            let elapsed = now.saturating_duration_since(started);
            let sample = self.playback.advance(elapsed)?;
            if elapsed <= Duration::from_millis(500) && !sample.crossed_cues.is_empty() {
                sequencer.push_preloaded_cues(
                    &sample.crossed_cues,
                    self.assets.sound_bank(),
                    started,
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        time_attack_protocol::tests::sample, world_protocol::SettlementState,
    };
    #[test]
    fn cooperative_hud_uses_observed_clock_leader_source_and_per_person_receipts() {
        let waiting = sample("waiting").frame.unwrap().combat.world;
        let lines = hud(&waiting, 1).join(" ");
        assert!(
            lines.contains("擊破啟動塔後開始")
                && lines.contains("server-7")
                && lines.contains("時限")
        );
        assert!(result_notice(&waiting, waiting.actors[0].key).is_none());
        let mut done = sample("mixed").frame.unwrap().combat.world;
        let first = done.actors[0].key;
        let second = done.actors[1].key;
        assert_eq!(first.object_id, second.object_id);
        assert_ne!(first.source_id, second.source_id);
        assert!(result_notice(&done, first).unwrap().starts_with("本場勝利"));
        assert!(result_notice(&done, second)
            .unwrap()
            .starts_with("本場敗北"));
        for (state, text) in [
            (SettlementState::Pending, "保存尚未確認"),
            (SettlementState::Saved, "本場結果已保存"),
            (SettlementState::Failed, "可能已入帳"),
        ] {
            done.time_attack
                .as_mut()
                .unwrap()
                .receipt
                .as_mut()
                .unwrap()
                .state = state;
            assert!(hud(&done, 2).last().unwrap().contains(text));
        }
        let mut foreign = first;
        foreign.source_id = 99;
        assert!(result_notice(&done, foreign).is_none());
        let timeout = sample("timeout").frame.unwrap().combat.world;
        assert!(hud(&timeout, 2).join(" ").contains("伺服器剩餘 00:00"));
        let dead = sample("memberdeath").frame.unwrap().combat.world;
        assert!(hud(&dead, 2).join(" ").contains("你已陣亡"));
        let departed = sample("departed").frame.unwrap().combat.world;
        assert!(hud(&departed, 1).join(" ").contains("你已離隊"));
    }
}
