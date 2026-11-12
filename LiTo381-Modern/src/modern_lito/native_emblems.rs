//! C30/S35 native 16x12 RGB555 emblems. Cache lifetime belongs to one World.
//! No disk cache, unsolicited insertion or automatic resend after silence.
use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

pub const WIDTH: u32 = 16;
pub const HEIGHT: u32 = 12;
pub const PIXEL_BYTES: usize = 384;
pub const CACHE_LIMIT: usize = 512;
pub const PENDING_LIMIT: usize = 32;
pub const RESPONSE_WAIT: Duration = Duration::from_secs(5);

pub fn request(id: u32) -> Result<Vec<u8>> {
    ensure!(id != 0, "zero emblem ID");
    let mut body = vec![0x30];
    body.extend_from_slice(&id.to_le_bytes());
    Ok(body)
}

#[derive(Clone, Debug)]
pub struct Image {
    pub id: u32,
    pub rgba: Vec<u8>,
    pub source_sha256: String,
}
pub fn decode(body: &[u8]) -> Result<Image> {
    ensure!(
        body.len() == 5 + PIXEL_BYTES && body[0] == 0x35,
        "emblem response envelope"
    );
    let id = u32::from_le_bytes(body[1..5].try_into()?);
    ensure!(id != 0, "zero emblem response ID");
    let mut rgba = Vec::with_capacity((WIDTH * HEIGHT * 4) as usize);
    for pair in body[5..].chunks_exact(2) {
        let word = u16::from_le_bytes([pair[0], pair[1]]);
        // The native blitter skips exactly word zero. The unused high bit
        // does not make 0x8000 transparent; it is covered black in this mode.
        let expand = |v: u16| ((v << 3) | (v >> 2)) as u8;
        rgba.extend_from_slice(&[
            expand((word >> 10) & 31),
            expand((word >> 5) & 31),
            expand(word & 31),
            if word == 0 { 0 } else { 255 },
        ]);
    }
    Ok(Image {
        id,
        rgba,
        source_sha256: format!("{:X}", Sha256::digest(&body[5..])),
    })
}

#[derive(Clone, Debug)]
struct Entry {
    requested_at: Instant,
    serial: u64,
    timed_out: bool,
    image: Option<Arc<Image>>,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Cache {
    #[serde(skip)]
    entries: BTreeMap<u32, Entry>,
    pub requests: u64,
    pub responses: u64,
    pub timeouts: u64,
    pub unsolicited: u64,
    pub evictions: u64,
}
impl Cache {
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn pending(&self) -> usize {
        self.entries
            .values()
            .filter(|e| e.image.is_none() && !e.timed_out)
            .count()
    }
    pub fn image(&self, id: u32) -> Option<&Image> {
        self.entries.get(&id)?.image.as_deref()
    }
    /// Caller sends first and commits with mark_sent only after send succeeds.
    /// Unknown IDs are not acknowledged absent by the formal server. Keep their
    /// timeout tombstones while wanted, avoiding one request per render frame.
    pub fn next_request(&mut self, wanted: &BTreeSet<u32>, now: Instant) -> Option<u32> {
        for e in self.entries.values_mut() {
            if e.image.is_none()
                && !e.timed_out
                && now.saturating_duration_since(e.requested_at) >= RESPONSE_WAIT
            {
                e.timed_out = true;
                self.timeouts = self.timeouts.saturating_add(1);
            }
        }
        if self.pending() >= PENDING_LIMIT {
            return None;
        }
        let id = wanted
            .iter()
            .copied()
            .find(|id| *id != 0 && !self.entries.contains_key(id))?;
        if self.entries.len() >= CACHE_LIMIT {
            let evict = self
                .entries
                .iter()
                .filter(|(id, e)| !wanted.contains(id) && (e.image.is_some() || e.timed_out))
                .min_by_key(|(_, e)| e.serial)
                .map(|(id, _)| *id)?;
            self.entries.remove(&evict);
            self.evictions = self.evictions.saturating_add(1);
        }
        Some(id)
    }
    pub fn mark_sent(&mut self, id: u32, now: Instant) -> Result<()> {
        ensure!(
            id != 0 && !self.entries.contains_key(&id),
            "duplicate emblem request"
        );
        ensure!(
            self.entries.len() < CACHE_LIMIT && self.pending() < PENDING_LIMIT,
            "emblem request budget"
        );
        self.requests = self.requests.saturating_add(1);
        self.entries.insert(
            id,
            Entry {
                requested_at: now,
                serial: self.requests,
                timed_out: false,
                image: None,
            },
        );
        Ok(())
    }
    /// Parse before mutation. A delayed response may fill its existing timeout
    /// tombstone, but an unknown/retired ID cannot populate this world's cache.
    pub fn accept(&mut self, body: &[u8]) -> Result<bool> {
        let image = decode(body)?;
        let Some(entry) = self.entries.get_mut(&image.id) else {
            self.unsolicited = self.unsolicited.saturating_add(1);
            return Ok(false);
        };
        if entry.image.is_some() {
            self.unsolicited = self.unsolicited.saturating_add(1);
            return Ok(false);
        }
        entry.image = Some(Arc::new(image));
        self.responses = self.responses.saturating_add(1);
        Ok(true)
    }
}

/// Native 489D8A: the hover actor's image sits 20px left of and 1px above its
/// name line (the name itself is shifted 10px right while an emblem ID is set).
pub const HOVER_NAME_OFFSET: [i32; 2] = [-20, -1];
/// Native 485F77: war-side head image relative to the actor screen position.
pub const WAR_HEAD_OFFSET: [i32; 2] = [17, -52];

fn blit(
    cache: &Cache,
    emblem: u32,
    actor: u32,
    at: [i32; 2],
    placement: &'static str,
    canvas: &mut super::compositor::Canvas,
    rows: &mut Vec<serde_json::Value>,
) -> Result<()> {
    if let Some(image) = cache.image(emblem) {
        canvas.blit_rgba(&image.rgba, WIDTH, HEIGHT, at[0], at[1])?;
        rows.push(serde_json::json!({"actor":actor,"id":image.id,"x":at[0],"y":at[1],"placement":placement,"source_sha256":image.source_sha256}));
    }
    Ok(())
}

/// Native placement. 489D15 draws only the hover actor's image beside its name
/// (the own actor included, but only while hovered). 485F30 draws a head image
/// for every other war-side actor (actor+0x20 nonzero) while the war gate
/// `byte_834F14` is set. Head anchors use the interpolated body position
/// without the native per-step sub-cell tables, like the clan markers.
pub fn draw(
    cache: &Cache,
    world: &super::native_world::World,
    lines: &[super::native_world_labels::Line],
    hover: Option<u32>,
    centers: &BTreeMap<u32, super::world_map::PixelPoint>,
    canvas: &mut super::compositor::Canvas,
    cancel: &AtomicBool,
) -> Result<serde_json::Value> {
    let mut rows = Vec::new();
    ensure!(!cancel.load(Ordering::Acquire), "emblem draw cancelled");
    let hover = hover.filter(|id| world.actor_visible(*id));
    if let Some(actor) = hover.and_then(|id| world.actors.get(&id)) {
        let line = lines
            .iter()
            .find(|l| l.id == actor.id && matches!(l.kind, "own" | "hover" | "all"));
        if let Some(line) = line.filter(|_| actor.emblem != 0) {
            let at = [
                line.x.saturating_add(HOVER_NAME_OFFSET[0]),
                line.y.saturating_add(HOVER_NAME_OFFSET[1]),
            ];
            blit(cache, actor.emblem, actor.id, at, "hover_name", canvas, &mut rows)?;
        }
    }
    if world.relations.active {
        for actor in world.actors.values() {
            ensure!(!cancel.load(Ordering::Acquire), "emblem draw cancelled");
            if actor.relations.side_20 == 0
                || actor.emblem == 0
                || hover == Some(actor.id)
                || !world.actor_visible(actor.id)
            {
                continue;
            }
            let Some(center) = centers.get(&actor.id) else {
                continue;
            };
            let at = [
                center.x.saturating_add(WAR_HEAD_OFFSET[0]),
                center.y.saturating_add(WAR_HEAD_OFFSET[1]),
            ];
            // 486032: a head row at or above the top edge is skipped.
            if at[1] <= 0 {
                continue;
            }
            blit(cache, actor.emblem, actor.id, at, "war_head", canvas, &mut rows)?;
        }
    }
    Ok(
        serde_json::json!({"rows":rows,"cache_entries":cache.len(),"pending":cache.pending(),"requests":cache.requests,"responses":cache.responses,"timeouts":cache.timeouts,"unsolicited":cache.unsolicited,"evictions":cache.evictions,"size":[WIDTH,HEIGHT],"ownership":"click_through","placement":"native_hover_name_left_20_up_1","war_gate":world.relations.active,"war_head_offset":WAR_HEAD_OFFSET,"head_anchor":"interpolated_body_position_without_step_tables","automatic_retry":false}),
    )
}

#[cfg(test)]
mod tests {
    use super::super::{
        compositor::Canvas, native_world::World, native_world_labels::Line, world_map::PixelPoint,
    };
    use super::*;
    fn world() -> World {
        let f: serde_json::Value = serde_json::from_slice(include_bytes!(
            "../../tests/fixtures/native-world-current.json"
        ))
        .unwrap();
        let mut w = World::default();
        for k in ["world", "start", "status", "actor", "bot", "hp", "mp"] {
            let h = f["packets"][k].as_str().unwrap();
            let b: Vec<u8> = (0..h.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&h[i..i + 2], 16).unwrap())
                .collect();
            w.accept(&b).unwrap();
        }
        w
    }
    fn packet(id: u32, word: u16) -> Vec<u8> {
        let mut v = vec![0x35];
        v.extend(id.to_le_bytes());
        for _ in 0..192 {
            v.extend(word.to_le_bytes());
        }
        v
    }
    fn placed(receipt: &serde_json::Value) -> Vec<(u64, i64, i64, String)> {
        let mut rows: Vec<_> = receipt["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["actor"].as_u64().unwrap(),
                    r["x"].as_i64().unwrap(),
                    r["y"].as_i64().unwrap(),
                    r["placement"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        rows.sort();
        rows
    }
    #[test]
    fn native_hover_and_war_head_placement() {
        let mut w = world();
        let own = w.own_actor().unwrap().id;
        assert!(own > 2);
        for id in [own, 2] {
            w.actors.get_mut(&id).unwrap().emblem = 7;
        }
        w.emblems.mark_sent(7, Instant::now()).unwrap();
        assert!(w.emblems.accept(&packet(7, 0x7c00)).unwrap());
        let line = |id: u32, kind: &'static str| Line {
            id,
            kind,
            text: "n".into(),
            x: 40,
            y: 30,
            color: 0x7fff,
            badge: None,
        };
        let lines = vec![line(own, "own"), line(2, "hover")];
        let mut centers = BTreeMap::from([
            (own, PixelPoint { x: 10, y: 60 }),
            (2, PixelPoint { x: 30, y: 60 }),
        ]);
        let token = AtomicBool::new(false);
        let mut canvas = Canvas::new(64, 48).unwrap();
        let own = u64::from(own);
        // The own image is not drawn beside the own label unless hovered.
        let r = draw(&w.emblems, &w, &lines, None, &centers, &mut canvas, &token).unwrap();
        assert!(placed(&r).is_empty());
        assert_eq!(canvas.rgba(), Canvas::new(64, 48).unwrap().rgba());
        let r = draw(&w.emblems, &w, &lines, Some(2), &centers, &mut canvas, &token).unwrap();
        assert_eq!(placed(&r), [(2, 20, 29, "hover_name".to_owned())]);
        let at = (29 * 64 + 20) * 4;
        assert_eq!(&canvas.rgba()[at..at + 4], &[255, 0, 0, 255]);
        // War gate: every side actor except the hover actor gets a head image.
        w.relations.active = true;
        for (id, side) in [(own as u32, 1), (2, 2)] {
            w.actors.get_mut(&id).unwrap().relations.side_20 = side;
        }
        let r = draw(&w.emblems, &w, &lines, Some(2), &centers, &mut canvas, &token).unwrap();
        assert_eq!(
            placed(&r),
            [
                (2, 20, 29, "hover_name".to_owned()),
                (own, 27, 8, "war_head".to_owned())
            ]
        );
        let r = draw(&w.emblems, &w, &lines, None, &centers, &mut canvas, &token).unwrap();
        assert_eq!(
            placed(&r),
            [
                (2, 47, 8, "war_head".to_owned()),
                (own, 27, 8, "war_head".to_owned())
            ]
        );
        // A head row at the top edge is skipped; an uncached ID draws nothing.
        centers.insert(2, PixelPoint { x: 30, y: 52 });
        w.actors.get_mut(&(own as u32)).unwrap().emblem = 8;
        let r = draw(&w.emblems, &w, &lines, None, &centers, &mut canvas, &token).unwrap();
        assert!(placed(&r).is_empty());
        token.store(true, Ordering::Release);
        assert!(draw(&w.emblems, &w, &lines, None, &centers, &mut canvas, &token).is_err());
    }
}
