//! Loopback fixture exercises real C30 dispatch and S35 replies, then pixels.
use super::{native_login::LoginController, native_world_scene::Frame};
use anyhow::{ensure, Context, Result};
#[derive(Default)]
pub struct Probe {
    step: usize,
}
impl Probe {
    pub fn tick(
        &mut self,
        c: &LoginController,
        frame: &Frame,
        rows: &mut Vec<serde_json::Value>,
    ) -> Result<bool> {
        let root = super::native_startup::diagnostic_project_root()?
            .join("target/current/native-emblems/room-world-emblems");
        let w = &c.snapshot().world;
        if !root.join("emblem-ready").exists() {
            std::fs::write(root.join("emblem-ready"), b"ready")?;
        }
        let Ok(step) = std::fs::read_to_string(root.join("emblem-step")) else {
            return Ok(false);
        };
        if step.trim() != self.step.to_string() {
            return Ok(false);
        }
        let own = w.own_actor().context("emblem own")?;
        let bot = w.actors.get(&2).context("emblem bot")?;
        let (own_id, bot_id, requests, responses, draws) = match self.step {
            0 => (0, u32::MAX, 1, 1, 1),
            1 => (u32::MAX, u32::MAX, 1, 1, 2),
            2 => (u32::MAX, 0x80000000, 2, 2, 2),
            3 => (u32::MAX, 7, 3, 2, 1),
            4 => (u32::MAX, 0, 3, 2, 1),
            5 => (0, 0, 3, 2, 0),
            _ => unreachable!(),
        };
        if own.emblem != own_id || bot.emblem != bot_id || w.emblems.responses < responses {
            return Ok(false);
        }
        if self.step == 3 && w.emblems.timeouts < 1 {
            return Ok(false);
        }
        let images = frame.receipt["emblems"]["rows"]
            .as_array()
            .context("emblem rows")?;
        if images.len() != draws {
            return Ok(false);
        }
        ensure!(
            w.emblems.requests == requests
                && w.emblems.responses == responses
                && w.emblems.unsolicited == 0,
            "emblem request deduplication"
        );
        ensure!(
            w.hp == Some([50, 50])
                && w.attack_packets == 0
                && w.own_cast_packets == 0
                && w.local_corrections == 0,
            "emblem changed authority"
        );
        let mut checked = 0;
        for row in images {
            let id = row["id"].as_u64().context("emblem id")? as u32;
            let offset = if id == u32::MAX { 0 } else { 1 };
            let x = row["x"].as_i64().context("emblem x")?;
            let y = row["y"].as_i64().context("emblem y")?;
            let words = [0u16, 0x8000, 0x7fff, 0xffff, 0x7c00, 0x03e0, 0x001f, 0x4210];
            for i in 0..192 {
                let word = words[(i + offset) % 8];
                if word == 0 {
                    continue;
                }
                let px = x + (i % 16) as i64;
                let py = y + (i / 16) as i64;
                ensure!(
                    (0..800).contains(&px) && (0..600).contains(&py),
                    "emblem outside capture"
                );
                let at = ((py * 800 + px) * 4) as usize;
                let expand = |n: u16| (n * 8 + n / 4) as u8;
                let expected = [
                    expand((word >> 10) & 31),
                    expand((word >> 5) & 31),
                    expand(word & 31),
                    255,
                ];
                ensure!(
                    frame.canvas.rgba()[at..at + 4] == expected,
                    "emblem pixel mismatch at {px},{py}"
                );
                checked += 1;
            }
        }
        let png = root.join(format!("emblem-{}.png", self.step));
        let mut enc = png::Encoder::new(std::fs::File::create(&png)?, 800, 600);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.write_header()?.write_image_data(frame.canvas.rgba())?;
        let mut row = frame.receipt.clone();
        row["emblem_step"] = self.step.into();
        row["png"] = serde_json::json!(png);
        row["opaque_pixels_checked"] = checked.into();
        row["world"] = serde_json::to_value(w)?;
        rows.push(row);
        std::fs::write(root.join(format!("emblem-ack-{}", self.step)), b"captured")?;
        self.step += 1;
        Ok(self.step == 6)
    }
}
