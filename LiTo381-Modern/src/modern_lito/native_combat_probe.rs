//! Loopback-only battle exercise through the same controller as the window.
use super::{
    native_login::{LoginController, LoginPhase},
    native_world_scene::Model,
};
use anyhow::{ensure, Context, Result};
use std::{
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
/// Probe-only bounded completion wait. A frame sampled before resource loading
/// must not be declared invalid merely because wall time advanced during draw.
/// Bad receipts remain immediate errors; an unsettled effect never passes.
pub fn reagent_completion_ready(
    fx: &serde_json::Value,
    expected_casts: u64,
    caster_ready: bool,
    elapsed: Duration,
) -> Result<bool> {
    ensure!(
        fx["started"].as_u64() == Some(expected_casts)
            && fx["omitted"].as_u64() == Some(0)
            && fx.get("load_error").is_some_and(serde_json::Value::is_null),
        "reagent effect rejected receipt: {fx}"
    );
    let active = fx["active"]
        .as_u64()
        .context("reagent effect active count absent")?;
    ensure!(
        active <= expected_casts,
        "reagent effect active count invalid: {fx}"
    );
    if active == 0 && caster_ready {
        return Ok(true);
    }
    ensure!(
        elapsed < Duration::from_secs(10),
        "reagent completion timed out after {}ms; caster_ready={caster_ready}; effects={fx}",
        elapsed.as_millis()
    );
    Ok(false)
}
/// Keep the verifier's observation floor without assuming a minimum frame rate.
/// Called only after the ranged authority, flight, arrival and settle checks.
pub fn ranged_capture_ready(frames: usize, elapsed: Duration) -> Result<bool> {
    capture_floor_ready("ranged", frames, elapsed)
}
/// Preserve the common ten-frame evidence floor after authority has settled.
/// Faster loading may complete a refusal before enough frames were sampled.
pub fn capture_floor_ready(kind: &str, frames: usize, elapsed: Duration) -> Result<bool> {
    if frames >= 10 {
        return Ok(true);
    }
    ensure!(
        elapsed < Duration::from_secs(10),
        "{kind} capture timed out: {frames}/10 frames after {}ms",
        elapsed.as_millis()
    );
    Ok(false)
}
#[derive(Default)]
pub struct Probe {
    model: Option<Model>,
    actor_states: super::native_actor_state_probe::Probe,
    label_updates: super::native_label_update_probe::Probe,
    emblems: super::native_emblem_probe::Probe,
    previous_generation: Option<u64>,
    previous_epoch: Option<u64>,
    previous_frame: Option<super::native_world_scene::Frame>,
    first_frame: Option<serde_json::Value>,
    held_attack: super::native_combat::HeldAttack,
    start: Option<Instant>,
    last_frame: Option<Instant>,
    sent: Option<Instant>,
    path: Vec<u8>,
    step: usize,
    hp: u16,
    own_count: u64,
    next_png_ms: u128,
    attacks_sent: u64,
    skills_sent: u64,
    skill_selection: super::native_skill_selection::Model,
    selection_receipts: Vec<serde_json::Value>,
    hp_before_skill: Option<u16>,
    item_probe: super::native_item_probe::WorldProbe,
    pickup_probe: super::native_ground_pickup_probe::Probe,
    reagent_pending: Option<(Instant, serde_json::Value)>,
    reagent_rows: Vec<serde_json::Value>,
    last_attack_sent: Option<Instant>,
    death_seen: Option<Instant>,
    death_ack: Option<serde_json::Value>,
    attempts: Vec<serde_json::Value>,
    pub trace: Vec<serde_json::Value>,
    pub frames: Vec<serde_json::Value>,
    pub before: Option<super::native_world::World>,
    pub result: Option<super::native_login::Snapshot>,
}
impl Probe {
    fn selected_ground_cast(
        &mut self,
        c: &mut LoginController,
        skill: u16,
        position: [u16; 2],
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        let epoch = c.transport().snapshot().epoch;
        let world = &c.snapshot().world;
        self.skill_selection.sync(
            epoch,
            world,
            c.snapshot().phase == LoginPhase::BattlePending,
        );
        self.skill_selection.select(Some(skill))?;
        let ground = super::native_skill_selection::Ground::capture(
            epoch,
            world,
            super::world_map::WorldCell {
                x: i32::from(position[0]),
                y: i32::from(position[1]),
            },
        );
        let choice = self.skill_selection.choice_at(epoch, world, None, ground)?;
        let generation = world.generation;
        c.cast_ground(choice.skill, choice.ground.unwrap(), frame)?;
        self.selection_receipts.push(serde_json::json!({"connection":epoch,"generation":generation,"skill":skill,"target":null,"ground":position}));
        Ok(())
    }
    fn reagent_tick(
        &mut self,
        c: &mut LoginController,
        frame: &super::native_world_scene::Frame,
        mode: &str,
    ) -> Result<bool> {
        use super::native_skills::Feedback;
        let success = mode == "room-world-stream";
        let count = if success { 3 } else { 2 };
        if let Some((since, before)) = &self.reagent_pending {
            if since.elapsed() < Duration::from_millis(if success { 3600 } else { 2500 }) {
                return Ok(false);
            }
            let w = &c.snapshot().world;
            let accepted = success && self.reagent_rows.len() < 2;
            let expected_casts = if success {
                (self.reagent_rows.len() + 1).min(2) as u64
            } else {
                0
            };
            let max_mp = if mode.ends_with("no-mana") { 2 } else { 80 };
            ensure!(
                w.own_attack_packets == 0
                    && w.hp == Some([50, 50])
                    && w.own_cast_packets == expected_casts
                    && w.mp == Some([max_mp - 20 * expected_casts as u16, max_mp]),
                "reagent skill authority"
            );
            ensure!(
                c.snapshot().skill_feedback
                    == if accepted {
                        Feedback::CastAction
                    } else {
                        Feedback::NoResponse
                    },
                "reagent skill feedback"
            );
            let fx = &frame.receipt["reagent_effects"];
            if !reagent_completion_ready(
                fx,
                expected_casts,
                frame.movement_ready(w),
                since.elapsed(),
            )
            .with_context(|| {
                format!(
                    "reagent sampled_frame_ns={} render_profile_us={}",
                    frame.receipt["elapsed_ns"], frame.receipt["render_profile_us"]
                )
            })? {
                return Ok(false);
            }
            self.reagent_rows.push(serde_json::json!({"before":before,"after":{"inventory":c.snapshot().inventory,"mp":w.mp,"casts":w.own_cast_packets},"feedback":c.snapshot().skill_feedback,"accepted":accepted,"effect_finished":true}));
            self.reagent_pending = None;
            return Ok(self.reagent_rows.len() == count);
        }
        let w = &c.snapshot().world;
        if !frame.movement_ready(w) {
            return Ok(false);
        }
        let own = w.own_actor().context("reagent own actor")?;
        let position = [own.x, own.y];
        let before = serde_json::json!({"inventory":c.snapshot().inventory,"mp":w.mp,"casts":w.own_cast_packets});
        self.selected_ground_cast(c, 63, position, frame)?;
        ensure!(
            c.cast_ground(63, position, frame).is_err(),
            "duplicate reagent cast admitted"
        );
        ensure!(c.attack(2).is_err(), "attack admitted during reagent wait");
        ensure!(
            before
                == serde_json::json!({"inventory":c.snapshot().inventory,"mp":c.snapshot().world.mp,"casts":c.snapshot().world.own_cast_packets}),
            "reagent predicted inventory/MP before reply"
        );
        self.skills_sent += 1;
        self.reagent_pending = Some((Instant::now(), before));
        Ok(false)
    }
    fn selected_cast(
        &mut self,
        c: &mut LoginController,
        skill: u16,
        target: Option<u32>,
        frame: &super::native_world_scene::Frame,
    ) -> Result<()> {
        let epoch = c.transport().snapshot().epoch;
        let world = &c.snapshot().world;
        self.skill_selection.sync(
            epoch,
            world,
            c.snapshot().phase == LoginPhase::BattlePending,
        );
        self.skill_selection.select(Some(skill))?;
        let target = target
            .map(|id| {
                super::native_skill_selection::Target::capture(world, id)
                    .context("probe selection target")
            })
            .transpose()?;
        let choice = self.skill_selection.choice(epoch, world, target)?;
        let generation = world.generation;
        c.cast(choice.skill, choice.target, frame)?;
        self.selection_receipts.push(
            serde_json::json!({"connection":epoch,"generation":generation,
            "skill":choice.skill,"target":choice.target}),
        );
        Ok(())
    }
    pub fn restart(&mut self) -> Result<()> {
        let world = &self
            .result
            .as_ref()
            .context("previous combat result")?
            .world;
        let generation = world.generation;
        let frame = self.model.as_mut().context("previous model")?.draw_at(
            world,
            &AtomicBool::new(false),
            self.start.unwrap().elapsed().as_nanos() as u64,
        )?;
        let epoch = self.previous_epoch;
        *self = Self {
            previous_generation: Some(generation),
            previous_epoch: epoch,
            previous_frame: Some(frame),
            ..Default::default()
        };
        Ok(())
    }
    pub fn tick(&mut self, c: &mut LoginController, mode: &str) -> Result<bool> {
        let ranged = mode == "room-world-ranged";
        let skill = match mode {
            "room-world-heal" => Some(1),
            "room-world-shield" => Some(3),
            "room-world-bolt" | "room-world-bolt-refused" => Some(4),
            "room-world-wall" | "room-world-wall-refused" => Some(58),
            "room-world-stream" | "room-world-stream-no-reagent" | "room-world-stream-no-mana" => {
                Some(63)
            }
            _ => None,
        };
        let victory = mode == "room-world-victory";
        let defeat = matches!(mode, "room-world-defeat" | "room-world-cycle");
        let refused = mode == "room-world-refused";
        let target = if refused { 1660944385 } else { 2 };
        let needs_target = !(matches!(skill, Some(3 | 63))
            || mode.starts_with("room-world-item-drop")
            || mode.starts_with("room-world-pickup"));
        if c.snapshot().phase == LoginPhase::LobbyReady && self.result.is_some() {
            return Ok(true);
        }
        if !matches!(
            c.snapshot().phase,
            LoginPhase::BattlePending | LoginPhase::BattleResult
        ) || !c.snapshot().world.ready()
        {
            return Ok(false);
        }
        if c.snapshot().world.integrity_challenge.is_none()
            || (needs_target && !c.snapshot().world.actors.contains_key(&target))
        {
            return Ok(false);
        }
        let cancel = AtomicBool::new(false);
        if self.model.is_none() {
            let w = &c.snapshot().world;
            if let Some(previous) = self.previous_generation {
                ensure!(
                    w.generation > previous
                        && self.previous_epoch == Some(c.transport().snapshot().epoch),
                    "second battle did not preserve owner/change world"
                );
                ensure!(
                    w.hp == Some([50, 50])
                        && w.own_attack_packets == 0
                        && w.attack_packets == 0
                        && w.death_confirmations_sent == 0
                        && !w.death_confirmation_pending
                        && w.scores.is_empty()
                        && w.battle_result.is_none()
                        && !w.own_actor().unwrap().dead
                        && c.snapshot().attack_feedback == super::native_combat::Feedback::Idle,
                    "second battle retained previous state"
                );
                let old = self.previous_frame.take().context("old death frame")?;
                ensure!(
                    !old.movement_ready(w) && !old.death_ready(w),
                    "old frame authorizes new world"
                );
            }
            self.previous_epoch = Some(c.transport().snapshot().epoch);
            let model = Model::load(
                std::path::Path::new(r"D:\lineage開發\0-客戶端"),
                &c.snapshot().world,
                &cancel,
            )?;
            let w = &c.snapshot().world;
            let a = w.own_actor().unwrap();
            self.path = if needs_target {
                let actor = w.actors.get(&target).context("received probe target")?;
                model.navigation().route_to_adjacent_avoiding([a.x, a.y], [actor.x, actor.y],|x,y|super::native_doors::blocks(w,x,y))?
            } else {
                Vec::new()
            };
            if ranged {
                ensure!(a.walk_action == 20, "ranged fixture stance missing");
                self.path.clear();
            }
            if matches!(skill, Some(3 | 63)) {
                self.path.clear();
            }
            if matches!(mode, "room-world-states" | "room-world-labels" | "room-world-emblems")
                || mode.starts_with("room-world-item-drop")
                || mode.starts_with("room-world-pickup")
            {
                self.path.clear();
            }
            self.hp = w.hp.context("fixture HP")?[0];
            self.own_count = w.own_attack_packets;
            self.before = Some(w.clone());
            self.start = Some(Instant::now());
            self.model = Some(model);
        }
        ensure!(
            c.snapshot().world.local_corrections == 0,
            "combat route corrected by server"
        );
        if self
            .last_frame
            .is_some_and(|t| t.elapsed() < Duration::from_millis(50))
        {
            return Ok(false);
        }
        self.last_frame = Some(Instant::now());
        let hover = if mode == "room-world-states" {
            self.actor_states.label_hover(&c.snapshot().world)
        } else if mode == "room-world-emblems" {
            super::native_world_labels::Hover::capture(&c.snapshot().world, 2)
        } else if mode == "room-world-labels" {
            self.label_updates.hover(&c.snapshot().world)
        } else {
            None
        };
        let frame = self.model.as_mut().unwrap().draw_at_labels(
            &c.snapshot().world,
            &cancel,
            self.start.unwrap().elapsed().as_nanos() as u64,
            hover,
        )?;
        if self.first_frame.is_none() {
            let own = frame.receipt["actors"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == c.snapshot().world.own_actor().unwrap().id)
                .context("first own frame")?;
            ensure!(
                own["phase"] == "idle" && frame.sounds.is_empty(),
                "fresh world retained action/sound"
            );
            self.first_frame = Some(frame.receipt.clone());
        }
        if mode == "room-world-emblems" {
            if self.emblems.tick(c, &frame, &mut self.frames)? {
                self.result = Some(c.snapshot().clone());
                return Ok(true);
            }
            return Ok(false);
        }
        if mode == "room-world-labels" {
            if self.label_updates.tick(c, &frame, &mut self.frames)? {
                self.result = Some(c.snapshot().clone());
                return Ok(true);
            }
            return Ok(false);
        }
        if mode == "room-world-states" {
            if self.actor_states.tick(c, &frame, &mut self.frames)? {
                self.result = Some(c.snapshot().clone());
                return Ok(true);
            }
            return Ok(false);
        }
        if let Some(sent) = self.sent {
            ensure!(self.frames.len() < 512, "combat capture budget");
            let mut row = frame.receipt.clone();
            if sent.elapsed().as_millis() >= self.next_png_ms {
                self.next_png_ms += if defeat { 500 } else { 250 };
                let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
                    "target/current/verification/combat-{}-g{}-{}.png",
                    mode.trim_start_matches("room-world-"),
                    c.snapshot().world.generation,
                    self.next_png_ms
                ));
                let mut enc = png::Encoder::new(std::fs::File::create(&path)?, 800, 600);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.write_header()?.write_image_data(frame.canvas.rgba())?;
                row["png"] = serde_json::json!(path);
            }
            row["world"] = serde_json::to_value(&c.snapshot().world)?;
            self.frames.push(row);
            if mode.starts_with("room-world-pickup") {
                if self.pickup_probe.tick(c, &frame, mode.ends_with("stale"))? {
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if skill == Some(63) {
                if self.reagent_tick(c, &frame, mode)? {
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if mode.starts_with("room-world-item-drop") {
                if self.item_probe.drop(c, &frame, mode.ends_with("refused"))? {
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if mode == "room-world-item-heal" {
                if self.item_probe.heal(c, &frame, self.hp)? {
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if skill == Some(58) {
                let refused = mode == "room-world-wall-refused";
                if sent.elapsed() >= Duration::from_millis(if refused { 2500 } else { 3200 }) {
                    let w = &c.snapshot().world;
                    let fx = &frame.receipt["ground_effects"];
                    ensure!(
                        w.own_attack_packets == 0 && w.hp == Some([50, 50]),
                        "ground cast invented physical action/HP"
                    );
                    if refused {
                        ensure!(
                            w.mp == Some([2, 2])
                                && w.own_cast_packets == 0
                                && w.effects.is_empty()
                                && c.snapshot().skill_feedback
                                    == super::native_skills::Feedback::NoResponse
                                && fx["started"] == 0,
                            "silent ground refusal changed authority"
                        );
                    } else {
                        let count = w.effects.iter().filter(|e| e.graphics == 168).count();
                        ensure!(
                            w.mp == Some([60, 120])
                                && w.own_cast_packets == 1
                                && (1..=8).contains(&count)
                                && c.snapshot().skill_feedback
                                    == super::native_skills::Feedback::CastAction,
                            "ground authority/cast missing"
                        );
                        ensure!(
                            w.actors[&target].hp_ratio < 100,
                            "formal fire-wall pulse never hurt bot"
                        );
                        ensure!(
                            fx["started"] == count && fx["active"] == 0 && fx["omitted"] == 0,
                            "ground visual lifetime incomplete"
                        );
                    }
                    if !capture_floor_ready("ground", self.frames.len(), sent.elapsed())? {
                        return Ok(false);
                    }
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if skill == Some(4) {
                if mode == "room-world-bolt-refused" {
                    if sent.elapsed() >= Duration::from_millis(2500) {
                        ensure!(
                            c.snapshot().skill_feedback
                                == super::native_skills::Feedback::Rejected(278)
                                && c.snapshot().world.own_cast_packets == 0
                                && c.snapshot().world.mp == Some([2, 2])
                                && frame.receipt["spell_projectiles"]["launched"] == 0,
                            "refused bolt changed authority"
                        );
                        self.result = Some(c.snapshot().clone());
                        return Ok(true);
                    }
                } else if self.skills_sent == 1 && sent.elapsed() >= Duration::from_millis(3300) {
                    ensure!(
                        c.snapshot().world.own_cast_packets == 1
                            && c.snapshot().world.mp == Some([7, 10])
                            && c.snapshot().skill_feedback
                                == super::native_skills::Feedback::CastAction,
                        "first bolt acknowledgement/MP missing"
                    );
                    self.selected_cast(c, 4, Some(target), &frame)?;
                    self.skills_sent = 2;
                    self.last_attack_sent = Some(Instant::now());
                } else if self.skills_sent == 2
                    && self.last_attack_sent.unwrap().elapsed() >= Duration::from_millis(2800)
                {
                    ensure!(
                        c.snapshot().world.own_cast_packets == 2
                            && c.snapshot().world.mp == Some([4, 10])
                            && c.snapshot().world.actors[&target].hp_ratio < 100
                            && c.snapshot().skill_feedback
                                == super::native_skills::Feedback::CastAction,
                        "second bolt authority mismatch"
                    );
                    ensure!(
                        frame.receipt["spell_projectiles"]["launched"] == 2
                            && frame.receipt["spell_projectiles"]["arrivals"] == 2
                            && frame.receipt["spell_projectiles"]["active"] == 0
                            && frame.receipt["spell_projectiles"]["omitted"] == 0,
                        "bolt effects incomplete"
                    );
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
                return Ok(false);
            }
            if let Some(skill) = skill {
                if self.skills_sent == 1 && sent.elapsed() >= Duration::from_millis(3300) {
                    ensure!(
                        c.snapshot().world.own_cast_packets == 1
                            && c.snapshot().skill_feedback
                                == super::native_skills::Feedback::CastAction,
                        "skill action absent"
                    );
                    if skill == 1 {
                        ensure!(
                            c.snapshot().world.hp.unwrap()[0] > self.hp_before_skill.unwrap(),
                            "real healing did not restore HP"
                        );
                    }
                    let expected = if skill == 1 { 6 } else { 2 };
                    ensure!(
                        c.snapshot().world.mp == Some([expected, 10]),
                        "formal spell MP mismatch"
                    );
                    ensure!(
                        self.frames.iter().any(|f| f["skill_effects"]["effects"]
                            .as_array()
                            .is_some_and(|r| r.iter().any(|e| e["drawn"] == true))),
                        "skill effect never drawn"
                    );
                    self.selected_cast(c, if skill == 1 { 3 } else { 1 }, None, &frame)?;
                    self.skills_sent = 2;
                    self.last_attack_sent = Some(Instant::now());
                } else if self.skills_sent == 2
                    && self.last_attack_sent.unwrap().elapsed() >= Duration::from_millis(2500)
                {
                    ensure!(
                        c.snapshot().skill_feedback
                            == if skill == 1 {
                                super::native_skills::Feedback::Rejected(278)
                            } else {
                                super::native_skills::Feedback::NoResponse
                            },
                        "formal insufficient-MP refusal missing"
                    );
                    ensure!(
                        c.snapshot().world.own_cast_packets == 1
                            && frame.receipt["skill_effects"]["started"] == 1
                            && frame.receipt["skill_effects"]["active"] == 0,
                        "refused skill changed presentation"
                    );
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
            } else if defeat {
                if c.snapshot().world.hp.is_some_and(|v| v[0] == 0) {
                    self.death_seen.get_or_insert(Instant::now());
                }
                if frame.death_ready(&c.snapshot().world) {
                    c.confirm_death(&frame)?;
                    self.death_ack = Some(
                        serde_json::json!({"elapsed_ns":frame.receipt["elapsed_ns"],"own":frame.receipt["actors"].as_array().unwrap().iter().find(|a|a["id"]==1),"confirmations":c.snapshot().world.death_confirmations_sent}),
                    );
                    ensure!(
                        c.confirm_death(&frame).is_err(),
                        "duplicate death confirmation sent"
                    );
                }
                if c.snapshot().phase == LoginPhase::BattleResult
                    && self
                        .death_seen
                        .is_some_and(|t| t.elapsed() >= Duration::from_millis(2500))
                {
                    ensure!(
                        c.snapshot().world.own_actor().unwrap().dead
                            && c.snapshot().world.hp.unwrap()[0] == 0
                            && !c.snapshot().world.scores.is_empty(),
                        "defeat result absent"
                    );
                    self.result = Some(c.snapshot().clone());
                    c.return_from_battle()?;
                } else if c.continue_attack(&mut self.held_attack, &frame)? {
                    ensure!(self.attacks_sent < 32, "defeat attack budget");
                    self.attacks_sent += 1;
                    self.last_attack_sent = Some(Instant::now());
                    self.attempts.push(serde_json::json!({"attempt":self.attacks_sent,"hp":c.snapshot().world.hp,"target":target}));
                }
            } else if refused {
                ensure!(
                    !c.continue_attack(&mut self.held_attack, &frame)?,
                    "silent attack automatically retried"
                );
                if self.last_attack_sent.unwrap().elapsed() >= Duration::from_millis(2500) {
                    ensure!(
                        c.snapshot().attack_feedback == super::native_combat::Feedback::NoResponse
                            && c.snapshot().world.own_attack_packets == 0
                            && c.snapshot().world.hp.unwrap()[0] == self.hp,
                        "silent refusal changed authority or feedback"
                    );
                    self.attempts.push(serde_json::json!({"attempt":self.attacks_sent,"feedback":c.snapshot().attack_feedback}));
                    ensure!(
                        !self.held_attack.active(),
                        "timeout left automatic attack armed"
                    );
                    if self.attacks_sent == 1 {
                        self.held_attack.press(
                            target,
                            c.transport().snapshot().epoch,
                            &c.snapshot().world,
                        );
                        ensure!(
                            c.continue_attack(&mut self.held_attack, &frame)?,
                            "fresh retry press did not send"
                        );
                        self.attacks_sent += 1;
                        self.last_attack_sent = Some(Instant::now());
                    } else {
                        self.result = Some(c.snapshot().clone());
                        return Ok(true);
                    }
                }
            } else if sent.elapsed() >= Duration::from_millis(2500) {
                ensure!(
                    !c.continue_attack(&mut self.held_attack, &frame)?,
                    "released attack repeated"
                );
                ensure!(
                    c.snapshot().world.own_attack_packets == self.own_count + 1,
                    "own attack acknowledgement missing"
                );
                if victory {
                    ensure!(
                        c.snapshot().phase == LoginPhase::BattleResult
                            && c.snapshot().world.actors[&2].dead
                            && !c.snapshot().world.scores.is_empty(),
                        "victory/death/scoreboard absent"
                    );
                    self.result = Some(c.snapshot().clone());
                    c.return_from_battle()?;
                } else if ranged {
                    ensure!(
                        c.snapshot().phase == LoginPhase::BattlePending
                            && c.snapshot().world.projectiles.len() == 1
                            && c.snapshot().world.projectiles[0].projectile.graphics == 66,
                        "ranged authority missing"
                    );
                    ensure!(
                        self.frames.iter().any(|f| f["projectiles"]["flights"]
                            .as_array()
                            .is_some_and(|rows| rows.iter().any(|r| r["drawn"] == true))),
                        "ranged flight never drawn"
                    );
                    let mut arrivals = 0;
                    let mut damage_frames = 0;
                    for f in &self.frames {
                        if let Some(rows) = f["projectiles"]["hurt_callbacks"].as_array() {
                            for row in rows {
                                ensure!(
                                    row["target"] == 2 && row["accepted"] == true,
                                    "ranged hurt rejected"
                                );
                                arrivals += 1;
                            }
                        }
                        for a in f["actors"].as_array().context("ranged actors")? {
                            if a["id"] == 2 && a["phase"] == "damage" {
                                ensure!(arrivals == 1, "target hurt before arrow arrival");
                                damage_frames += 1;
                            }
                        }
                    }
                    ensure!(
                        arrivals == 1 && damage_frames > 0,
                        "ranged arrival hurt missing"
                    );
                    ensure!(
                        frame.movement_ready(&c.snapshot().world),
                        "bow animation not settled"
                    );
                    if !ranged_capture_ready(self.frames.len(), sent.elapsed())? {
                        return Ok(false);
                    }
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                } else {
                    ensure!(
                        c.snapshot().phase == LoginPhase::BattlePending
                            && c.snapshot().world.hp.unwrap()[0] < self.hp,
                        "counterattack HP not authoritative"
                    );
                    ensure!(
                        frame.movement_ready(&c.snapshot().world),
                        "own action did not settle"
                    );
                    self.result = Some(c.snapshot().clone());
                    return Ok(true);
                }
            }
        } else if frame.movement_ready(&c.snapshot().world) {
            if mode.starts_with("room-world-item-drop")
                || mode.starts_with("room-world-pickup")
                || skill == Some(63)
            {
                self.sent = Some(Instant::now());
                return Ok(false);
            }
            if skill == Some(4) {
                let w = &c.snapshot().world;
                let own = w.own_actor().unwrap();
                let t = &w.actors[&target];
                if own.x.abs_diff(t.x).max(own.y.abs_diff(t.y)) <= 10 {
                    self.path.truncate(self.step);
                }
            }
            if self.step < self.path.len() {
                let heading = self.path[self.step];
                c.move_checked(heading, &frame.navigation)?;
                self.step += 1;
                self.trace.push(serde_json::json!({"step":self.step,"heading":heading,"own":c.snapshot().world.own_actor(),"elapsed_ms":self.start.unwrap().elapsed().as_millis()}));
            } else if let Some(skill) = skill {
                if skill == 1 && self.attacks_sent == 0 {
                    c.attack(target)?;
                    self.attacks_sent = 1;
                    return Ok(false);
                }
                if skill == 1
                    && (c.snapshot().world.own_attack_packets == 0
                        || c.snapshot().world.hp.unwrap()[0] >= self.hp)
                {
                    return Ok(false);
                }
                self.hp_before_skill = Some(c.snapshot().world.hp.unwrap()[0]);
                if skill == 58 {
                    let t = &c.snapshot().world.actors[&target];
                    let position = [t.x, t.y];
                    self.selected_ground_cast(c, 58, position, &frame)?;
                    ensure!(
                        c.cast_ground(58, position, &frame).is_err(),
                        "duplicate ground cast admitted"
                    );
                } else {
                    self.selected_cast(
                        c,
                        skill,
                        if skill == 4 { Some(target) } else { None },
                        &frame,
                    )?;
                }
                ensure!(
                    c.cast(skill, if skill == 4 { Some(target) } else { None }, &frame)
                        .is_err(),
                    "duplicate pending skill admitted"
                );
                ensure!(
                    c.attack(target).is_err(),
                    "attack admitted during skill wait"
                );
                self.skills_sent = 1;
                self.sent = Some(Instant::now());
                self.last_attack_sent = self.sent;
            } else {
                self.held_attack
                    .press(target, c.transport().snapshot().epoch, &c.snapshot().world);
                ensure!(
                    c.continue_attack(&mut self.held_attack, &frame)?,
                    "initial held attack did not send"
                );
                ensure!(
                    !c.continue_attack(&mut self.held_attack, &frame)?,
                    "held attack repeated before reply"
                );
                ensure!(c.attack(target).is_err(), "duplicate pending attack queued");
                if !defeat && !refused {
                    self.held_attack.clear();
                }
                self.attacks_sent = 1;
                self.sent = Some(Instant::now());
                self.last_attack_sent = self.sent;
                self.attempts.push(
                    serde_json::json!({"attempt":1,"target":target,"hp":c.snapshot().world.hp}),
                );
            }
        }
        Ok(false)
    }
    pub fn receipt(&self) -> serde_json::Value {
        serde_json::json!({"epoch":self.previous_epoch,"previous_generation":self.previous_generation,"first_frame":self.first_frame,"shared_held_attack":self.skills_sent==0,"held_active_end":self.held_attack.active(),"attacks_sent":self.attacks_sent,"skills_sent":self.skills_sent,"item_actions":self.item_probe.rows,"pickup_actions":self.pickup_probe.rows,"pickup_drop":self.pickup_probe.drop_receipt,"reagent_actions":self.reagent_rows,"skill_selection":self.selection_receipts,"hp_before_skill":self.hp_before_skill,"attempts":self.attempts,"death_ack":self.death_ack,"route":self.trace,"before":self.before,"frames":self.frames,"result":self.result,"audio_device_tested":false,"gui_tested":false})
    }
}
