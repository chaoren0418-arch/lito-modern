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

/// Modern own/hover-name placement: 16x12 image, two-pixel gap before the name.
/// This is explicit viewport policy, not the unqualified native all-name pass.
pub fn draw(
    cache: &Cache,
    world: &super::native_world::World,
    lines: &[super::native_world_labels::Line],
    canvas: &mut super::compositor::Canvas,
    cancel: &AtomicBool,
) -> Result<serde_json::Value> {
    let mut rows = Vec::new();
    for line in lines.iter().filter(|l| matches!(l.kind, "own" | "hover")) {
        ensure!(!cancel.load(Ordering::Acquire), "emblem draw cancelled");
        let Some(actor) = world
            .actors
            .get(&line.id)
            .filter(|_| world.actor_visible(line.id))
        else {
            continue;
        };
        if let Some(image) = cache.image(actor.emblem) {
            let x = line.x - 18;
            canvas.blit_rgba(&image.rgba, WIDTH, HEIGHT, x, line.y)?;
            rows.push(serde_json::json!({"actor":actor.id,"id":image.id,"x":x,"y":line.y,"source_sha256":image.source_sha256}));
        }
    }
    Ok(
        serde_json::json!({"rows":rows,"cache_entries":cache.len(),"pending":cache.pending(),"requests":cache.requests,"responses":cache.responses,"timeouts":cache.timeouts,"unsolicited":cache.unsolicited,"evictions":cache.evictions,"size":[WIDTH,HEIGHT],"ownership":"click_through","placement":"modern_own_hover_name_left_18","all_name_native_placement":false,"automatic_retry":false}),
    )
}
