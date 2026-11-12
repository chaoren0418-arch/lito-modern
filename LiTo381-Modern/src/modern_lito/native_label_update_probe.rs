//! Loopback-only metadata fixture. Wire packets exclusively mutate authority.
use super::{
    native_login::LoginController, native_skill_selection::Target, native_world::World,
    native_world_labels::Hover, native_world_scene::Frame,
};
use anyhow::{ensure, Context, Result};
use std::path::PathBuf;
#[derive(Default)]
pub struct Probe {
    step: usize,
    started: Option<u64>,
    hover: Option<Hover>,
    target: Option<Target>,
}
impl Probe {
    fn root() -> Result<PathBuf> {
        Ok(super::native_startup::diagnostic_project_root()?
            .join("target/current/native-label-updates/room-world-labels"))
    }
    pub fn hover(&mut self, w: &World) -> Option<Hover> {
        // Setup S0B deliberately replaces the fixture bot before step zero.
        if self.hover.and_then(|h| h.resolve(w)).is_none() {
            self.hover = Hover::capture(w, 2);
        }
        self.hover
    }
    pub fn tick(
        &mut self,
        c: &LoginController,
        frame: &Frame,
        rows: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        let root = Self::root()?;
        let w = &c.snapshot().world;
        let own = w.own_actor().context("label own")?;
        let bot = w.actors.get(&2).context("label bot")?;
        if !root.join("label-ready").exists() {
            std::fs::write(root.join("label-ready"), b"ready")?;
        }
        let Ok(step) = std::fs::read_to_string(root.join("label-step")) else {
            return Ok(false);
        };
        if step.trim() != self.step.to_string() || bot.x != own.x + 1 || bot.y != own.y + 1 {
            return Ok(false);
        }
        let (title, clan, emblem, active) = match self.step {
            0 => ("", "", 0, false),
            1 => ("稱號更新", "", 0, false),
            2 => ("盟員稱號", "測試盟", u32::MAX, false),
            3 => ("新血盟", "測試盟", 0x80000000, true),
            4 => ("", "測試盟", 0x80000000, true),
            5 | 6 => ("", "測試盟", 0, false),
            7 => ("1234567890123456", "測試盟", 0, true),
            8 => ("", "", 0, true),
            9 | 10 => ("", "", 0, false),
            _ => unreachable!(),
        };
        let own_active = (6..=9).contains(&self.step);
        if (&*bot.title, &*bot.clan, bot.emblem, bot.clan_marker_active)
            != (title, clan, emblem, active)
            || own.clan_marker_active != own_active
            || (self.step >= 6
                && (&*own.title, own.emblem)
                    != (
                        if own_active { "自身血盟" } else { "" },
                        if own_active { 7 } else { 0 },
                    ))
        {
            return Ok(false);
        }
        if self.target.is_none() {
            self.target = Target::capture(w, 2);
        }
        ensure!(
            self.target.and_then(|t| t.resolve(w)) == Some(2)
                && self.hover.and_then(|h| h.resolve(w)) == Some(2),
            "metadata retired selection"
        );
        ensure!(
            w.hp == Some([50, 50])
                && w.attack_packets == 0
                && w.own_cast_packets == 0
                && w.local_corrections == 0,
            "metadata changed combat authority"
        );
        let labels = frame.receipt["world_labels"]["lines"]
            .as_array()
            .context("metadata labels")?;
        ensure!(
            labels.iter().all(|r| r.get("error").is_none()),
            "metadata glyph failure"
        );
        ensure!(
            labels.iter().filter(|r| r["line"]["id"] == 2).count()
                == if title.is_empty() { 1 } else { 2 },
            "title line count"
        );
        if !title.is_empty() {
            ensure!(
                labels
                    .iter()
                    .any(|r| r["line"]["kind"] == "title" && r["line"]["text"] == title),
                "title text mismatch"
            );
        }
        let markers = frame.receipt["clan_markers"]["rows"]
            .as_array()
            .context("metadata marker rows")?;
        ensure!(
            markers.len() == usize::from(active),
            "own marker or missing remote marker"
        );
        ensure!(
            markers
                .iter()
                .all(|r| r["drawn"] == true && r["marker"]["id"] == 2 && r.get("error").is_none()),
            "metadata marker failure"
        );
        ensure!(frame.sounds.is_empty(), "metadata fabricated audio");
        let ns = frame.receipt["elapsed_ns"]
            .as_u64()
            .context("metadata time")?;
        let elapsed = ns.saturating_sub(*self.started.get_or_insert(ns));
        let phase = markers.first().and_then(|r| r["marker"]["phase"].as_u64());
        let mut row = frame.receipt.clone();
        row["label_step"] = serde_json::json!(self.step);
        row["world"] = serde_json::to_value(w)?;
        row["selection_retained"] = serde_json::json!(true);
        let capture = !rows
            .iter()
            .any(|r| r["label_step"] == self.step && r["captured_phase"].as_u64() == phase);
        row["captured_phase"] = serde_json::json!(phase);
        if capture {
            let path = root.join(format!(
                "label-{}-{}.png",
                self.step,
                phase.map_or("none".into(), |p| p.to_string())
            ));
            let mut enc = png::Encoder::new(std::fs::File::create(&path)?, 800, 600);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.write_header()?.write_image_data(frame.canvas.rgba())?;
            row["png"] = serde_json::json!(path);
        }
        ensure!(rows.len() < 256, "metadata frame budget");
        rows.push(row);
        if elapsed >= 750_000_000 {
            std::fs::write(root.join(format!("label-ack-{}", self.step)), b"captured")?;
            self.step += 1;
            self.started = None;
        }
        Ok(self.step == 11)
    }
}
