//! Explicit isolated packet fixture; never enabled by normal window input.
use super::{native_actor_states::State, native_login::LoginController, native_world_scene::Frame};
use anyhow::{ensure, Context, Result};
use std::path::PathBuf;
#[derive(Default)]
pub struct Probe {
    step: usize,
    hover: Option<super::native_world_labels::Hover>,
    hover_captured: bool,
    started: Option<u64>,
    target: Option<super::native_skill_selection::Target>,
    held: super::native_combat::HeldAttack,
}
impl Probe {
    pub fn label_hover(
        &mut self,
        world: &super::native_world::World,
    ) -> Option<super::native_world_labels::Hover> {
        if !self.hover_captured || self.step == 9 {
            self.hover = super::native_world_labels::Hover::capture(world, 2);
            self.hover_captured = true;
        }
        self.hover
    }

    fn root() -> Result<PathBuf> {
        Ok(super::native_startup::diagnostic_project_root()?
            .join("target/current/native-actor-states/room-world-states"))
    }
    pub fn tick(
        &mut self,
        c: &LoginController,
        frame: &Frame,
        rows: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        let root = Self::root()?;
        let w = &c.snapshot().world;
        let own = w.own_actor().context("state fixture own")?;
        let bot = w.actors.get(&2).context("state fixture bot")?;
        if !root.join("state-ready").exists() {
            self.target = super::native_skill_selection::Target::capture(w, 2);
            self.held.press(2, c.transport().snapshot().epoch, w);
            std::fs::write(root.join("state-ready"), b"ready")?;
        }
        let Ok(step) = std::fs::read_to_string(root.join("state-step")) else {
            return Ok(false);
        };
        if step.trim() != self.step.to_string() {
            return Ok(false);
        }
        let own_state = State {
            green: self.step == 1,
            dim: matches!(self.step, 2 | 3),
            invisible: (3..=6).contains(&self.step),
            suppressed: false,
            native_light_dead: false,
        };
        let bot_state = State {
            green: (5..=8).contains(&self.step),
            invisible: matches!(self.step, 6 | 7),
            ..Default::default()
        };
        if own.draw_state != own_state || bot.draw_state != bot_state {
            return Ok(false);
        }
        ensure!(
            w.hp == Some([50, 50]) && w.attack_packets == 0 && w.own_cast_packets == 0,
            "state fixture changed authority"
        );
        let ns = frame.receipt["elapsed_ns"]
            .as_u64()
            .context("state frame clock")?;
        let green_counter=u32::try_from(frame.receipt["actor_green_clock"]["counter"].as_u64()
            .context("state frame owned green callback counter")?).context("state frame green counter domain")?;
        for (a, local) in [(own, true), (bot, false)] {
            let row = frame.receipt["actors"]
                .as_array()
                .context("state actors")?
                .iter()
                .find(|r| r["id"] == a.id)
                .context("state actor receipt")?;
            ensure!(
                row["draw_style"] == serde_json::to_value(a.draw_state.style(local, green_counter))?,
                "state render style mismatch"
            );
            if !a.draw_state.visible(local) {
                ensure!(
                    row["hidden"] == true
                        && !frame.hits.iter().any(|(id, _)| *id == a.id)
                        && !frame.visibility.iter().any(|(id, _)| *id == a.id),
                    "invisible actor retained hit surface"
                );
            }
        }
        if bot_state.invisible {
            ensure!(
                self.held
                    .ready(
                        c.transport().snapshot().epoch,
                        w,
                        super::native_combat::Feedback::Idle,
                        true,
                        true
                    )
                    .is_none()
                    && !self.held.active(),
                "invisible held attack survived"
            );
            ensure!(
                self.target.unwrap().resolve(w).is_none() && w.attack_request(2).is_err(),
                "invisible target remains actionable"
            );
        }
        if self.step >= 8 {
            ensure!(
                self.target.unwrap().resolve(w).is_none()
                    && super::native_skill_selection::Target::capture(w, 2)
                        .and_then(|t| t.resolve(w))
                        == Some(2),
                "hidden target selection revived instead of requiring a fresh selection"
            );
        }
        let labels = frame.receipt["world_labels"]["lines"]
            .as_array()
            .context("state labels")?;
        ensure!(
            labels.iter().filter(|r| r["line"]["id"] == own.id).count() == 1,
            "own label missing"
        );
        let bot_labels = labels.iter().filter(|r| r["line"]["id"] == 2).count();
        ensure!(
            bot_labels == usize::from(self.step < 6 || self.step == 9),
            "hover label survived hidden selection retirement"
        );
        ensure!(
            labels.iter().all(|r| r.get("error").is_none()),
            "label raster failed"
        );
        ensure!(frame.sounds.is_empty(), "stationary state fabricated sound");
        let elapsed = ns.saturating_sub(*self.started.get_or_insert(ns));
        let mut row = frame.receipt.clone();
        row["state_step"] = serde_json::json!(self.step);
        row["world"] = serde_json::to_value(w)?;
        if !rows.iter().any(|r| r["state_step"] == self.step) {
            let path = root.join(format!("state-{}.png", self.step));
            let mut enc = png::Encoder::new(std::fs::File::create(&path)?, 800, 600);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()?.write_image_data(frame.canvas.rgba())?;
            row["png"] = serde_json::json!(path);
        }
        ensure!(rows.len() < 256, "state capture budget");
        rows.push(row);
        if elapsed >= 650_000_000 {
            std::fs::write(root.join(format!("state-ack-{}", self.step)), b"captured")?;
            self.step += 1;
            self.started = None;
        }
        Ok(self.step == 10)
    }
}
