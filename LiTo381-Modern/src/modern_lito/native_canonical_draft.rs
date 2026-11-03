//! UNFROZEN Rust-local draft, not Java vectors or a negotiated live route.
//! No legacy WorldFrame conversion, no renderer/LOADED grant, no wire writes.
//! Disabled by default. Production Selected/profile/transport remain unchanged.
use super::{
    application_protocol as app, battle_payload as payload, battle_world as canonical,
    native_resources as resources, protocol, world_protocol as world,
};
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
};
type Result<T> = std::result::Result<T, app::ApplicationError>;
pub const REQUEST_KIND: u16 = 88;
pub const RESULT_KIND: u16 = 89;
pub const REQUEST_BYTES: usize = 180;
pub const MIN_RESULT_BYTES: usize = 244;
pub const MAX_SOURCE_BYTES: usize = 2860;
pub const NATIVE_BIT: u32 = 256;
// Fixed Java service selected0x1801006F: resource0x6F plus descriptor,
// canonical DATA and action-life. Healing/armor keep their own Selected gates.
const NATIVE_CANONICAL_DEPENDENCIES: u32 = 0x6f | protocol::CAP_MATCH_DESCRIPTOR
    | protocol::CAP_BATTLE_DATA | protocol::CAP_ACTION_LIFE;
const PREFIX: usize = 68;
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid local Native canonical draft",
    )
}
#[derive(Clone, Copy, Default)]
pub struct DraftCodec {
    peer: Option<protocol::extensions::Selected>,
}
impl DraftCodec {
    /// Explicit local experiment only. This is NOT verified HELLO/ACK admission.
    /// Strip only the draft routing bit for unchanged canonical DATA selection.
    pub fn local_experiment(base: protocol::Ack, extensions: u32) -> Result<Self> {
        if extensions & NATIVE_BIT == 0
            || extensions & !(NATIVE_BIT | protocol::extensions::KNOWN) != 0
            || base.version != 47
            || base.numeric_schema != protocol::NUMERIC_SCHEMA_I64_V1
            || base.capabilities & NATIVE_CANONICAL_DEPENDENCIES
                != NATIVE_CANONICAL_DEPENDENCIES
            || base.capabilities & 0x6f != 0x6f
            || !canonical::negotiated(&base)
            // Unimplemented Native full-VIP lifecycle: reject the combination.
            || extensions & (protocol::extensions::FULL_VIP_DATA
                | protocol::extensions::FULL_VIP_WORLD
                | protocol::extensions::FULL_VIP_APPEARANCE) != 0
        {
            return Err(invalid());
        }
        let peer = protocol::extensions::Selected::new(base, extensions & !NATIVE_BIT)
            .map_err(|_| invalid())?;
        Ok(Self { peer: Some(peer) })
    }
    fn peer(self) -> Result<protocol::extensions::Selected> {
        self.peer.ok_or_else(invalid)
    }
    pub fn maximum_result(self) -> Result<usize> {
        Ok(3104 + payload::Format::Selected(self.peer()?).maximum())
    }
    pub fn encode_request(self, id: u32, request: &Request) -> Result<Vec<u8>> {
        let peer = self.peer()?;
        request.validate()?;
        let inner = canonical::encode_request(id, &request.canonical, peer.base().version)?;
        let mut bytes = envelope_header(id, REQUEST_KIND, REQUEST_BYTES)?;
        bytes.extend(prefix(request));
        bytes.extend(&inner[16..]);
        Ok(bytes)
    }
    pub fn decode_request(self, bytes: &[u8]) -> Result<Request> {
        let peer = self.peer()?;
        check_header(bytes, REQUEST_KIND, REQUEST_BYTES, REQUEST_BYTES, None)?;
        check_tag(&bytes[16..20])?;
        let mut inner = app::header(
            u32_at(bytes, 8),
            canonical::REQUEST_KIND,
            canonical::REQUEST_BYTES,
        )?;
        inner.extend(&bytes[84..]);
        let q = Request {
            expected_resource_set: bytes[20..52].try_into().unwrap(),
            load_proof: bytes[52..84].try_into().unwrap(),
            canonical: canonical::expected_request(&inner, peer.base().version)?,
        };
        q.validate()?;
        Ok(q)
    }
    /// Per-kind bound checked after exactly16 bytes, before body allocation.
    /// Reads one framed message; following stream frames belong to the caller.
    pub fn read_reply(self, reader: &mut impl Read, expected: &[u8]) -> Result<Reply> {
        let peer = self.peer()?;
        let q = self.decode_request(expected)?;
        let mut header = [0; 16];
        reader.read_exact(&mut header).map_err(|_| invalid())?;
        let length = check_header(
            &header,
            RESULT_KIND,
            MIN_RESULT_BYTES,
            self.maximum_result()?,
            Some(u32_at(expected, 8)),
        )?;
        let mut body = vec![0; length - 16];
        reader.read_exact(&mut body).map_err(|_| invalid())?;
        if body[..PREFIX] != expected[16..84] || body[72..76] != [0; 4] {
            return Err(invalid());
        }
        let n = u32_at(&body, PREFIX) as usize;
        if n > MAX_SOURCE_BYTES || n > body.len() - 76 - 152 {
            return Err(invalid());
        }
        let source = if n == 0 {
            None
        } else {
            Some(SourceAttachment::decode(
                &body[76..76 + n],
                peer.base().capabilities,
            )?)
        };
        // Header substitution wraps unchanged canonical body, not a legacy DTO.
        let inner_expected = canonical::encode_request(u32_at(expected, 8), &q.canonical, 47)?;
        let canonical_body = &body[76 + n..];
        let mut inner = envelope_header(
            u32_at(expected, 8),
            canonical::RESULT_KIND,
            16 + canonical_body.len(),
        )?;
        inner.extend(canonical_body);
        let canonical =
            canonical::read_reply_selected(&mut inner.as_slice(), &inner_expected, peer)?;
        if !canonical::reply_negotiated_selected(&canonical, peer) {
            return Err(invalid());
        }
        let reply = Reply {
            request: q,
            source,
            canonical,
        };
        reply.validate()?;
        Ok(reply)
    }
    /// Exact-buffer API rejects trailing bytes; streaming API retains framing.
    pub fn decode_reply(self, bytes: &[u8], expected: &[u8]) -> Result<Reply> {
        let mut reader = bytes;
        let reply = self.read_reply(&mut reader, expected)?;
        if !reader.is_empty() {
            return Err(invalid());
        }
        Ok(reply)
    }
}
// Only16 bytes allocated; callers enforce selected per-kind bounds first.
// Application old frame whitelist/bounds are not modified.
fn envelope_header(id: u32, kind: u16, length: usize) -> Result<Vec<u8>> {
    if id == 0 || length < 16 || length > u32::MAX as usize {
        return Err(invalid());
    }
    let mut b = b"LTM1".to_vec();
    b.extend((length as u32).to_be_bytes());
    b.extend(id.to_be_bytes());
    b.extend(kind.to_be_bytes());
    b.extend([0; 2]);
    Ok(b)
}
fn check_header(bytes: &[u8], kind: u16, min: usize, max: usize, id: Option<u32>) -> Result<usize> {
    if bytes.len() < 16 {
        return Err(invalid());
    }
    let n = u32_at(bytes, 4) as usize;
    if &bytes[..4] != b"LTM1"
        || bytes[12..14] != kind.to_be_bytes()
        || bytes[14..16] != [0; 2]
        || u32_at(bytes, 8) == 0
        || id.is_some_and(|id| id != u32_at(bytes, 8))
        || !(min..=max).contains(&n)
        || kind == REQUEST_KIND && bytes.len() != n
    {
        return Err(invalid());
    }
    Ok(n)
}
fn check_tag(b: &[u8]) -> Result<()> {
    if b != [0, 1, 0, 2] {
        Err(invalid())
    } else {
        Ok(())
    }
}
#[derive(Clone, PartialEq, Eq)]
pub struct Request {
    pub expected_resource_set: [u8; 32],
    pub load_proof: [u8; 32],
    pub canonical: canonical::Request,
}
impl Request {
    fn validate(&self) -> Result<()> {
        self.canonical.validate()?;
        let first_read =
            self.canonical.action == canonical::Action::Read && self.canonical.match_id == 0;
        if (!first_read && self.expected_resource_set == [0; 32])
            || (self.canonical.action == canonical::Action::Loaded) != (self.load_proof != [0; 32])
        {
            return Err(invalid());
        }
        Ok(())
    }
}
fn prefix(q: &Request) -> Vec<u8> {
    let mut b = vec![0, 1, 0, 2];
    b.extend(q.expected_resource_set);
    b.extend(q.load_proof);
    b
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceAttachment {
    pub resource_set: [u8; 32],
    pub plan: resources::NativeAssetPlan,
}
impl SourceAttachment {
    pub fn validate(&self, capabilities: u32) -> Result<()> {
        use resources::ArchiveRole::*;
        let p = &self.plan;
        if p.namespace != resources::ResourceNamespace::NativeLitoV1
            || p.map_id > i32::MAX as u32
            || p.canonical_sha256 == [0; 32]
            || !(1..=64).contains(&p.sectors.len())
            || p.sectors.len() > 16 && capabilities & protocol::CAP_LARGE_MAPS == 0
            || p.archives.len() != 4
        {
            return Err(invalid());
        }
        let mut total = 0u64;
        let mut previous = None;
        for s in &p.sectors {
            if s.key.map_id != p.map_id
                || s.pin.bytes < 49158
                || !pin_valid(s.pin, 8 << 20)
                || previous.is_some_and(|k| k >= (s.key.x, s.key.y))
            {
                return Err(invalid());
            }
            previous = Some((s.key.x, s.key.y));
            total = total.checked_add(s.pin.bytes).ok_or_else(invalid)?;
        }
        if total > 32 << 20 {
            return Err(invalid());
        }
        for (a, role) in p
            .archives
            .iter()
            .zip([TileIndex, TilePak, SpriteIndex, SpritePak])
        {
            let maximum = if matches!(role, TileIndex | SpriteIndex) {
                8 << 20
            } else {
                256 << 20
            };
            if a.role != role || !pin_valid(a.pin, maximum) {
                return Err(invalid());
            }
        }
        if self.resource_set == [0; 32] || self.resource_set != self.computed_resource_set() {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn resource_set_preimage(&self) -> Vec<u8> {
        let mut b = b"LITO-WORLD-RESOURCES\0".to_vec();
        b.extend([0, 1, 0, 2]);
        b.extend(self.plan.map_id.to_be_bytes());
        b.extend(self.plan.canonical_sha256);
        b.extend([0, 4, 0, 0]);
        for a in &self.plan.archives {
            archive_bytes(&mut b, a);
        }
        b
    }
    pub fn computed_resource_set(&self) -> [u8; 32] {
        Sha256::digest(self.resource_set_preimage()).into()
    }
    pub fn encode(&self, capabilities: u32) -> Result<Vec<u8>> {
        self.validate(capabilities)?;
        let mut b = vec![0, 1, 0, 2, 0, 1, 0, 2];
        b.extend(self.resource_set);
        b.extend(self.plan.map_id.to_be_bytes());
        b.extend(self.plan.canonical_sha256);
        b.extend([0, 4, 0, 0]);
        for a in &self.plan.archives {
            archive_bytes(&mut b, a);
        }
        b.extend((40u32 + 40 * self.plan.sectors.len() as u32).to_be_bytes());
        b.extend([0, 1]);
        b.extend((self.plan.sectors.len() as u16).to_be_bytes());
        b.extend(self.plan.map_id.to_be_bytes());
        b.extend(self.plan.canonical_sha256);
        for s in &self.plan.sectors {
            b.extend(s.key.x.to_be_bytes());
            b.extend(s.key.y.to_be_bytes());
            b.extend((s.pin.bytes as u32).to_be_bytes());
            b.extend(s.pin.sha256);
        }
        Ok(b)
    }
    pub fn decode(b: &[u8], capabilities: u32) -> Result<Self> {
        if !(340..=MAX_SOURCE_BYTES).contains(&b.len()) {
            return Err(invalid());
        }
        check_tag(&b[..4])?;
        check_tag(&b[4..8])?;
        if b[76..80] != [0, 4, 0, 0] || b[260..262] != [0, 1] {
            return Err(invalid());
        }
        let count = u16_at(b, 262) as usize;
        let manifest_bytes = 40 + 40 * count;
        if !(1..=64).contains(&count)
            || u32_at(b, 256) as usize != manifest_bytes
            || b.len() != 260 + manifest_bytes
            || b[40..76] != b[264..300]
        {
            return Err(invalid());
        }
        let map_id = u32_at(b, 40);
        let archives = (0..4)
            .map(|i| {
                let at = 80 + 44 * i;
                if u16_at(b, at) as usize != i + 1 || b[at + 2..at + 4] != [0, 1] {
                    return Err(invalid());
                }
                Ok(resources::ArchivePin {
                    role: [
                        resources::ArchiveRole::TileIndex,
                        resources::ArchiveRole::TilePak,
                        resources::ArchiveRole::SpriteIndex,
                        resources::ArchiveRole::SpritePak,
                    ][i],
                    pin: resources::BytePin {
                        bytes: u64_at(b, at + 4),
                        sha256: b[at + 12..at + 44].try_into().unwrap(),
                    },
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let sectors = (0..count)
            .map(|i| {
                let at = 300 + 40 * i;
                resources::SectorPin {
                    key: resources::NativeSectorKey {
                        map_id,
                        x: u16_at(b, at),
                        y: u16_at(b, at + 2),
                    },
                    pin: resources::BytePin {
                        bytes: u32_at(b, at + 4).into(),
                        sha256: b[at + 8..at + 40].try_into().unwrap(),
                    },
                }
            })
            .collect();
        let source = Self {
            resource_set: b[8..40].try_into().unwrap(),
            plan: resources::NativeAssetPlan {
                namespace: resources::ResourceNamespace::NativeLitoV1,
                map_id,
                canonical_sha256: b[44..76].try_into().unwrap(),
                sectors,
                archives,
            },
        };
        source.validate(capabilities)?;
        Ok(source)
    }
    pub fn matches_data(&self, manifest: &payload::data::Manifest) -> bool {
        let p = &self.plan;
        p.map_id as i32 == manifest.map
            && p.canonical_sha256 == manifest.digest
            && p.sectors.len() == manifest.sectors.0.len()
            && p.sectors.iter().zip(&manifest.sectors.0).all(|(a, b)| {
                i32::from(a.key.x) == b.x
                    && i32::from(a.key.y) == b.y
                    && a.pin.bytes == b.byte_length as u64
                    && a.pin.sha256 == b.sha256
            })
    }
    /// Fixed-byte claim consistency only, never connection ownership, renderer
    /// support, server ledger verification or permission to submit LOADED.
    pub fn check_claim(&self, request: &Request, capabilities: u32) -> Result<()> {
        request.validate()?;
        self.validate(capabilities)?;
        if request.expected_resource_set != [0; 32]
            && request.expected_resource_set != self.resource_set
        {
            return Err(invalid());
        }
        if request.canonical.action == canonical::Action::Loaded {
            let proof: [u8; 32] =
                Sha256::digest(load_proof_preimage(self, &request.canonical)?).into();
            if request.load_proof != proof || request.canonical.digest != self.plan.canonical_sha256
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
fn pin_valid(p: resources::BytePin, maximum: u64) -> bool {
    p.bytes > 0 && p.bytes <= maximum && p.sha256 != [0; 32]
}
fn archive_bytes(b: &mut Vec<u8>, a: &resources::ArchivePin) {
    let role: u16 = match a.role {
        resources::ArchiveRole::TileIndex => 1,
        resources::ArchiveRole::TilePak => 2,
        resources::ArchiveRole::SpriteIndex => 3,
        resources::ArchiveRole::SpritePak => 4,
    };
    b.extend(role.to_be_bytes());
    b.extend([0, 1]);
    b.extend(a.pin.bytes.to_be_bytes());
    b.extend(a.pin.sha256);
}
pub struct Reply {
    pub request: Request,
    pub source: Option<SourceAttachment>,
    pub canonical: canonical::Reply,
}
impl Reply {
    fn validate(&self) -> Result<()> {
        self.request.validate()?;
        self.canonical.validate()?;
        if self.canonical.echo != self.request.canonical {
            return Err(invalid());
        }
        let q = &self.request.canonical;
        let bootstrap = q.action == canonical::Action::Read && q.match_id == 0;
        if self.source.is_none() {
            if !bootstrap
                || self.request.expected_resource_set != [0; 32]
                || self.canonical.state != canonical::State::Detached
                || !matches!(
                    self.canonical.status,
                    world::Status::Ok | world::Status::Unavailable
                )
            {
                return Err(invalid());
            }
        } else if self.canonical.state == canonical::State::Detached {
            let allowed = (bootstrap && self.canonical.status == world::Status::Ok)
                || (q.action == canonical::Action::Prepare
                    && self.canonical.status != world::Status::Ok)
                || (q.action == canonical::Action::Leave
                    && self.canonical.status == world::Status::Ok);
            if !allowed {
                return Err(invalid());
            }
        }
        if let Some(source) = &self.source {
            source.check_claim(&self.request, u32::MAX)?;
            if self
                .canonical
                .data
                .as_ref()
                .is_some_and(|d| !source.matches_data(&d.manifest))
            {
                return Err(invalid());
            }
        }
        Ok(())
    }
}
/// Diagnostic identity-bound claim bytes only. Does NOT grant LOADED or
/// renderer support; owner deliberately exposes no proof-minting API.
pub fn load_proof_preimage(source: &SourceAttachment, q: &canonical::Request) -> Result<Vec<u8>> {
    q.validate()?;
    source.validate(u32::MAX)?;
    if q.room <= 0 || q.match_id <= 0 || q.generation <= 0 {
        return Err(invalid());
    }
    let mut b = b"LITO-NATIVE-LOAD-PROOF\0".to_vec();
    b.extend(source.resource_set);
    b.extend(source.plan.canonical_sha256);
    b.extend(q.context.key.source_id.to_be_bytes());
    b.extend(q.context.key.object_id.to_be_bytes());
    // Opaque epoch, including high bit, same bytes as canonical66.
    b.extend(q.context.epoch.to_be_bytes());
    b.extend(u32::from(q.channel).to_be_bytes());
    for n in [q.room, q.match_id, q.generation] {
        b.extend(n.to_be_bytes());
    }
    Ok(b)
}
/// Candidate owner interface carries connection incarnation + current room
/// fence separately from the canonical request and SourcePlan.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Scope {
    pub connection_incarnation: u64,
    pub context: app::SelectionContext,
    pub channel: u8,
    pub joined_room: i64,
    pub room_revision: i64,
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WorkTicket {
    owner: u64,
    serial: u64,
}
static NEXT_OWNER: AtomicU64 = AtomicU64::new(1);
struct Staged {
    scope: Scope,
    source: SourceAttachment,
    canonical: canonical::Reply,
    ticket: WorkTicket,
    cancel: Arc<AtomicBool>,
    assets: Option<resources::NativeStaticAssets>,
    renderer: Option<PartialRendererCompletion>,
}
/// Immutable worker publication. Construction requires the current owner and
/// an already semantically admitted Observation, with exact canonical DATA.
pub struct RendererJob {
    ticket: WorkTicket,
    scope: Scope,
    source: SourceAttachment,
    attachment: canonical::Attachment,
    observation: super::battle_follow::Observation,
    cancel: Arc<AtomicBool>,
}
/// Opaque LOCAL PARTIAL completion, intentionally not PreparedResources and
/// incapable of minting LOADED. Owns real decoded terrain and rendered slices.
pub struct PartialRendererCompletion {
    job: RendererJob,
    assets: resources::NativeStaticAssets,
    slices: Vec<super::native_action_inspection::NativeActionSlice>,
    actors: Vec<super::native_actor_inspection::NativeActorSlice>,
}
impl RendererJob {
    pub fn admitted_observation(&self)->&super::battle_follow::Observation {&self.observation}
    pub fn cancellation_token(&self)->Arc<AtomicBool> {self.cancel.clone()}
    pub fn cancellation(&self) -> &AtomicBool { &self.cancel }
    pub fn source_plan(&self) -> &resources::NativeAssetPlan { &self.source.plan }
    pub fn finish_partial(self, assets: resources::NativeStaticAssets,
        slices: Vec<super::native_action_inspection::NativeActionSlice>) -> Result<PartialRendererCompletion> {
        if self.cancel.load(Ordering::Acquire) || assets.verified_plan()!=&self.source.plan
            || slices.is_empty() || slices.len()>10
            || slices.iter().any(|s|s.verified_plan()!=&self.source.plan)
            || slices.iter().any(|s|s.catalog_pins()!=slices[0].catalog_pins()
                || !assets.effect_records().iter().any(|e|u32::from(e.effect.sprite_id)==s.sample().0))
        {return Err(invalid());}
        Ok(PartialRendererCompletion{job:self,assets,slices,actors:Vec::new()})
    }
    pub fn finish_partial_actor(self,assets:resources::NativeStaticAssets,
        slices:Vec<super::native_action_inspection::NativeActionSlice>,actor:super::native_actor_inspection::NativeActorSlice)->Result<PartialRendererCompletion> {
        if !actor.matches_job(&self){return Err(invalid());}
        let mut result=self.finish_partial(assets,slices)?;result.actors.push(actor);Ok(result)
    }
}
/// Isolated preparation owner. Publication into live Tracker and renderer
/// support are intentionally unavailable until contract/fence integration.
#[derive(Default)]
pub struct DraftOwner {
    enabled: bool,
    codec: DraftCodec,
    owner: u64,
    serial: u64,
    staged: Option<Staged>,
}
impl DraftOwner {
    pub fn local_experiment(codec: DraftCodec) -> Result<Self> {
        codec.peer()?;
        let owner = NEXT_OWNER
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
            .map_err(|_| invalid())?;
        Ok(Self {
            enabled: true,
            codec,
            owner,
            serial: 0,
            staged: None,
        })
    }
    pub fn retire(&mut self) {
        if let Some(staged) = &self.staged {
            staged.cancel.store(true, Ordering::Release);
        }
        self.staged = None;
        self.serial = self.serial.saturating_add(1);
    }
    pub fn stage(&mut self, scope: Scope, reply: Reply) -> Result<WorkTicket> {
        if !self.enabled {
            return Err(invalid());
        }
        reply.validate()?;
        let peer = self.codec.peer()?;
        if reply.canonical.status != world::Status::Ok
            || reply
                .canonical
                .attachment
                .as_ref()
                .is_some_and(|a| a.lifecycle.phase == world::Phase::Aborted)
            || !canonical::reply_negotiated_selected(&reply.canonical, peer)
        {
            return Err(invalid());
        }
        let q = &reply.request.canonical;
        if q.action == canonical::Action::Leave
            || scope.connection_incarnation == 0
            || scope.context != q.context
            || scope.channel != q.channel
            || scope.joined_room <= 0
            || scope.room_revision <= 0
            || q.room != 0 && q.room != scope.joined_room
            || q.action == canonical::Action::Prepare && q.room_revision != scope.room_revision
        {
            return Err(invalid());
        }
        let source = reply.source.ok_or_else(invalid)?;
        source.validate(peer.base().capabilities)?;
        if let Some(current) = &self.staged {
            // Resource/binding change requires explicit retirement; never
            // reassign an old worker's assets merely from matching map ID.
            let same_attachment = match (&current.canonical.attachment, &reply.canonical.attachment)
            {
                (None, None) => true,
                (None, Some(_)) => q.action == canonical::Action::Prepare,
                (Some(a), Some(b)) => {
                    a.world == b.world
                        && b.lifecycle.revision >= a.lifecycle.revision
                        && (a.lifecycle.revision != b.lifecycle.revision || a == b)
                        && (a.lifecycle.phase != world::Phase::Aborted || a == b)
                }
                _ => false,
            };
            if current.scope != scope || current.source != source || !same_attachment {
                return Err(invalid());
            }
        }
        let serial = self.serial.checked_add(1).ok_or_else(invalid)?;
        let ticket = WorkTicket {
            owner: self.owner,
            serial,
        };
        self.serial = serial;
        if let Some(current) = &self.staged {
            current.cancel.store(true, Ordering::Release);
        }
        self.staged = Some(Staged {
            scope,
            source,
            canonical: reply.canonical,
            ticket,
            cancel: Arc::new(AtomicBool::new(false)),
            assets: None,
            renderer: None,
        });
        Ok(ticket)
    }
    pub fn asset_plan(&self, ticket: WorkTicket) -> Result<&resources::NativeAssetPlan> {
        let staged = self
            .staged
            .as_ref()
            .filter(|s| s.ticket == ticket)
            .ok_or_else(invalid)?;
        Ok(&staged.source.plan)
    }
    pub fn cancellation(&self, ticket: WorkTicket) -> Result<Arc<AtomicBool>> {
        let staged = self
            .staged
            .as_ref()
            .filter(|s| s.ticket == ticket)
            .ok_or_else(invalid)?;
        Ok(Arc::clone(&staged.cancel))
    }
    pub fn complete_resources(
        &mut self,
        ticket: WorkTicket,
        assets: resources::NativeStaticAssets,
    ) -> Result<()> {
        let staged = self
            .staged
            .as_mut()
            .filter(|s| s.ticket == ticket)
            .ok_or_else(invalid)?;
        if staged.cancel.load(Ordering::Acquire) || assets.verified_plan() != &staged.source.plan {
            return Err(invalid());
        }
        staged.assets = Some(assets);
        Ok(())
    }
    pub fn begin_renderer(&self, ticket: WorkTicket,
        observation: &super::battle_follow::Observation) -> Result<RendererJob> {
        let staged=self.staged.as_ref().filter(|s|s.ticket==ticket).ok_or_else(invalid)?;
        let data=staged.canonical.data.as_ref().ok_or_else(invalid)?;
        let attachment=staged.canonical.attachment.as_ref().ok_or_else(invalid)?;
        // Rejected updates, BeforeEngine, countdown/active or retired workers
        // cannot start a Loading renderer preparation on borrowed DATA.
        if staged.cancel.load(Ordering::Acquire) || staged.canonical.state!=canonical::State::Data
            || attachment.lifecycle.phase!=world::Phase::Loading
            || data!=observation.data() || data.world.identity!=attachment.world
            || !staged.source.matches_data(&data.manifest)
        {return Err(invalid());}
        Ok(RendererJob{ticket,scope:staged.scope,source:staged.source.clone(),attachment:attachment.clone(),
            observation:observation.clone(),cancel:Arc::clone(&staged.cancel)})
    }
    pub fn complete_partial_renderer(&mut self, ticket:WorkTicket, completion:PartialRendererCompletion) -> Result<()> {
        let staged=self.staged.as_mut().filter(|s|s.ticket==ticket).ok_or_else(invalid)?;
        let j=&completion.job;
        if j.ticket!=ticket || j.scope!=staged.scope || j.source!=staged.source
            || staged.canonical.attachment.as_ref()!=Some(&j.attachment)
            || staged.canonical.data.as_ref()!=Some(j.observation.data())
            || !Arc::ptr_eq(&j.cancel,&staged.cancel) || staged.cancel.load(Ordering::Acquire)
        {return Err(invalid());}
        staged.renderer=Some(completion);
        Ok(())
    }
    pub fn partial_renderer_available(&self)->bool {
        self.staged.as_ref().is_some_and(|s| !s.cancel.load(Ordering::Acquire) && s.renderer.is_some())
    }
    /// Even completed R426 assets cannot authorize Native LOADED yet.
    pub fn loaded_admission_available(&self) -> bool {
        false
    }
}
impl Drop for DraftOwner {
    fn drop(&mut self) {
        self.retire();
    }
}
fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(b[at..at + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
#[cfg(test)]
#[path = "native_canonical_prototype_tests.rs"]
mod prototype_tests;
#[cfg(test)]
#[path = "native_canonical_draft_tests.rs"]
mod tests;
#[cfg(test)]
#[path = "native_renderer_slice_tests.rs"]
mod renderer_slice_tests;
