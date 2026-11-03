//! Synthetic Rust-local examples only: NOT Java fixed bytes, real assets or TLS.
use super::*;
fn codec() -> DraftCodec {
    DraftCodec::local_experiment(ack(), NATIVE_BIT).unwrap()
}
#[test]
fn native_service_minimum_profile_does_not_require_unselected_healing_or_armor() {
    let mut base=ack();base.capabilities=NATIVE_CANONICAL_DEPENDENCIES;
    assert!(DraftCodec::local_experiment(base,NATIVE_BIT).is_ok());
    for bit in (0..32).map(|i|1u32<<i).filter(|b|NATIVE_CANONICAL_DEPENDENCIES & b != 0) {
        let mut bad=base;bad.capabilities &= !bit;
        assert!(DraftCodec::local_experiment(bad,NATIVE_BIT).is_err());
    }
    // Existing extension dependency closure stays stricter when selected.
    assert!(DraftCodec::local_experiment(base,NATIVE_BIT | protocol::extensions::PLAYER_HEALING_DATA).is_err());
    assert!(DraftCodec::local_experiment(base,NATIVE_BIT | protocol::extensions::ARMOR_PROTECTION_DATA).is_err());
    assert!(protocol::extensions::Selected::new(base,NATIVE_BIT).is_err());
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 9,
        version: 47,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
fn data() -> payload::UnvalidatedData {
    let mut d = payload::data_tests::fixture();
    d.presentation = Some(payload::data::List(
        d.actors
            .0
            .iter()
            .map(|a| payload::data::PublicPresentation {
                entity: a.entity,
                shot: None,
                item: None,
            })
            .collect(),
    ));
    d.npc_motion = Some(payload::npc_motion::Envelope { state: None });
    d
}
fn source() -> SourceAttachment {
    use resources::*;
    let d = data();
    let mut s = SourceAttachment {
        resource_set: [0; 32],
        plan: NativeAssetPlan {
            namespace: ResourceNamespace::NativeLitoV1,
            map_id: d.manifest.map as u32,
            canonical_sha256: d.manifest.digest,
            sectors: d
                .manifest
                .sectors
                .0
                .iter()
                .map(|s| SectorPin {
                    key: NativeSectorKey {
                        map_id: d.manifest.map as u32,
                        x: s.x as u16,
                        y: s.y as u16,
                    },
                    pin: BytePin {
                        bytes: s.byte_length as u64,
                        sha256: s.sha256,
                    },
                })
                .collect(),
            archives: [
                ArchiveRole::TileIndex,
                ArchiveRole::TilePak,
                ArchiveRole::SpriteIndex,
                ArchiveRole::SpritePak,
            ]
            .into_iter()
            .enumerate()
            .map(|(i, role)| ArchivePin {
                role,
                pin: BytePin {
                    bytes: 100 + i as u64,
                    sha256: [10 + i as u8; 32],
                },
            })
            .collect(),
        },
    };
    s.resource_set = s.computed_resource_set();
    s
}
fn q() -> Request {
    Request {
        expected_resource_set: source().resource_set,
        load_proof: [0; 32],
        canonical: canonical::Request::read(
            app::SelectionContext {
                key: app::CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1u64 << 63,
            },
            0,
            Some(&data().world.identity),
        ),
    }
}
fn first() -> Request {
    let mut q = q();
    q.expected_resource_set = [0; 32];
    q.canonical = canonical::Request::read(q.canonical.context, 0, None);
    q
}
fn prepare() -> Request {
    let mut q = q();
    q.canonical.action = canonical::Action::Prepare;
    q.canonical.room_revision = 102;
    q.canonical.match_id = 0;
    q.canonical.generation = 0;
    q
}
fn scope() -> Scope {
    Scope {
        connection_incarnation: 100,
        context: q().canonical.context,
        channel: 0,
        joined_room: 101,
        room_revision: 102,
    }
}
fn response(
    q: &Request,
    state: canonical::State,
    source: Option<&SourceAttachment>,
    d: Option<&payload::UnvalidatedData>,
) -> (Vec<u8>, Vec<u8>) {
    let expected = codec().encode_request(9, q).unwrap();
    let source = source
        .map(|s| s.encode(ack().capabilities).unwrap())
        .unwrap_or_default();
    let mut b = envelope_header(9, RESULT_KIND, MIN_RESULT_BYTES).unwrap();
    b.extend(prefix(q));
    b.extend((source.len() as u32).to_be_bytes());
    b.extend([0; 4]);
    b.extend(source);
    b.extend(&expected[84..]);
    b.extend([0, 0, state as u8, 0]);
    if state == canonical::State::Detached {
        b.extend([0; 48]);
    } else {
        for n in [101i64, 102, 103] {
            b.extend(n.to_be_bytes());
        }
        b.extend(d.map_or(1, |d| d.world.revision + 1).to_be_bytes());
        b.extend([d.map_or(0, |d| d.world.phase as u8 + 1), 0, 0, 0]);
        b.extend([0; 12]);
    }
    let payload = d
        .map(|d| {
            let mut base = d.clone();
            base.npc_motion = None;
            payload::npc_motion::encode(&payload::npc_motion::Candidate {
                data: base,
                motion: None,
            })
            .unwrap()
        })
        .unwrap_or_default();
    b.extend((payload.len() as u32).to_be_bytes());
    b.extend(payload);
    let length = b.len() as u32;
    b[4..8].copy_from_slice(&length.to_be_bytes());
    (expected, b)
}
fn reply(
    q: &Request,
    state: canonical::State,
    s: Option<&SourceAttachment>,
    d: Option<&payload::UnvalidatedData>,
) -> Reply {
    let (e, b) = response(q, state, s, d);
    codec().decode_reply(&b, &e).unwrap()
}
#[test]
fn default_disabled_and_incomplete_selection_reject_before_io() {
    assert!(DraftCodec::default().encode_request(1, &first()).is_err());
    assert!(DraftCodec::default()
        .read_reply(&mut PanicReader, &[],)
        .is_err());
    assert!(DraftOwner::default()
        .stage(
            scope(),
            reply(&first(), canonical::State::Detached, Some(&source()), None)
        )
        .is_err());
    for bits in [
        0,
        512,
        NATIVE_BIT | 512,
        NATIVE_BIT | protocol::extensions::FULL_VIP.supported,
        NATIVE_BIT | protocol::extensions::FULL_VIP_V2.supported,
    ] {
        assert!(DraftCodec::local_experiment(ack(), bits).is_err());
    }
    let mut a = ack();
    a.capabilities = 0x6f;
    assert!(DraftCodec::local_experiment(a, NATIVE_BIT).is_err());
    for missing in [
        protocol::CAP_BATTLE_DATA,
        protocol::CAP_ACTION_LIFE,
        protocol::CAP_WORLD,
        protocol::CAP_SOURCE_SCOPED_IDENTITIES,
    ] {
        let mut a = ack();
        a.capabilities &= !missing;
        assert!(DraftCodec::local_experiment(a, NATIVE_BIT).is_err());
    }
    for version in [39, 46, 48] {
        let mut a = ack();
        a.version = version;
        assert!(DraftCodec::local_experiment(a, NATIVE_BIT).is_err());
    }
    assert!(protocol::extensions::Selected::new(ack(), NATIVE_BIT).is_err());
}
struct PanicReader;
impl Read for PanicReader {
    fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
        panic!("forbidden I/O")
    }
}
#[test]
fn request_180_and_unchanged_canonical_body_with_high_bit_epoch() {
    for q in [first(), q(), prepare()] {
        let b = codec().encode_request(9, &q).unwrap();
        assert_eq!(b.len(), 180);
        assert!(codec().decode_request(&b).unwrap() == q);
        assert_eq!(
            &b[84..],
            &canonical::encode_request(9, &q.canonical, 47).unwrap()[16..]
        );
    }
    let mut loaded = q();
    loaded.canonical.action = canonical::Action::Loaded;
    loaded.canonical.digest = source().plan.canonical_sha256;
    loaded.load_proof = [9; 32];
    assert!(
        codec()
            .decode_request(&codec().encode_request(1, &loaded).unwrap())
            .unwrap()
            == loaded
    );
    let mut leave = q();
    leave.canonical.action = canonical::Action::Leave;
    assert!(codec().encode_request(1, &leave).is_ok());
}
#[test]
fn request_predicates_headers_reserved_and_exact_length_reject() {
    let b = codec().encode_request(9, &q()).unwrap();
    for at in [0, 4, 12, 14, 16, 18, 108, 109, 110, 111, 112, 113, 114, 115] {
        let mut bad = b.clone();
        bad[at] ^= 1;
        assert!(codec().decode_request(&bad).is_err(), "offset{at}");
    }
    for n in 0..b.len() {
        assert!(codec().decode_request(&b[..n]).is_err());
    }
    let mut bad = b.clone();
    bad.push(0);
    assert!(codec().decode_request(&bad).is_err());
    let mut r = q();
    r.expected_resource_set = [0; 32];
    assert!(codec().encode_request(1, &r).is_err());
    let mut r = q();
    r.load_proof = [1; 32];
    assert!(codec().encode_request(1, &r).is_err());
    let mut r = q();
    r.canonical.action = canonical::Action::Loaded;
    r.canonical.digest = [1; 32];
    assert!(codec().encode_request(1, &r).is_err());
    assert!(codec().encode_request(0, &q()).is_err());
}
#[test]
fn source_plan_manifest_and_digest_preimages_are_exact_local_examples() {
    let s = source();
    let b = s.encode(ack().capabilities).unwrap();
    assert_eq!(b.len(), 340);
    assert_eq!(SourceAttachment::decode(&b, ack().capabilities).unwrap(), s);
    assert_eq!(s.resource_set_preimage().len(), 241);
    assert!(s.matches_data(&data().manifest));
    let p = load_proof_preimage(&s, &q().canonical).unwrap();
    assert_eq!(p.len(), 135);
    assert_eq!(&p[..23], b"LITO-NATIVE-LOAD-PROOF\0");
    assert_eq!(&p[99..107], &(1u64 << 63).to_be_bytes());
    let original: [u8; 32] = Sha256::digest(&p).into();
    for field in 0..7 {
        let mut q = q().canonical;
        match field {
            0 => q.context.key.source_id = 8,
            1 => q.context.key.object_id += 1,
            2 => q.context.epoch += 1,
            3 => q.channel = 1,
            4 => q.room += 1,
            5 => q.match_id += 1,
            _ => q.generation += 1,
        }
        let hash: [u8; 32] = Sha256::digest(load_proof_preimage(&s, &q).unwrap()).into();
        assert_ne!(hash, original);
    }
    assert!(load_proof_preimage(&s, &first().canonical).is_err());
    let hex = |bytes: &[u8]| bytes.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let (request, before) = response(&prepare(), canonical::State::BeforeEngine, Some(&s), None);
    let (_, detached) = response(&first(), canonical::State::Detached, None, None);
    let (_, data) = response(&q(), canonical::State::Data, Some(&s), Some(&data()));
    eprintln!(
        "NATIVE_CANONICAL_LOCAL_EXAMPLES {}",
        serde_json::json!({
            "origin":"synthetic Rust-local draft; NOT Java fixed vectors, real assets, HELLO/ACK or live admission",
            "request88_full_bytes":request.len(),"request88_hex":hex(&request),
            "detached89_full_bytes":detached.len(),"detached89_hex":hex(&detached),
            "before_engine89_full_bytes":before.len(),"before_engine89_hex":hex(&before),
            "data89_full_bytes":data.len(),"data89_hex":hex(&data),
            "source_attachment_hex":hex(&b),"source_plan_hex":hex(&b[4..256]),
            "manifest_hex":hex(&b[260..]),"resource_set_preimage_bytes":241,
            "resource_set_preimage_hex":hex(&s.resource_set_preimage()),"resource_set_digest_hex":hex(&s.resource_set),
            "load_proof_preimage_bytes":135,"load_proof_preimage_hex":hex(&p),"load_proof_digest_hex":hex(&original),
            "qualified_modes_added":0,"GUI":0
        })
    );
}
#[test]
fn loaded_response_claim_must_match_resource_geometry_and_identity() {
    let mut request = q();
    request.canonical.action = canonical::Action::Loaded;
    request.canonical.digest = source().plan.canonical_sha256;
    request.load_proof =
        Sha256::digest(load_proof_preimage(&source(), &request.canonical).unwrap()).into();
    let mut d = data();
    d.actors.0[0].loaded = true;
    let (e, b) = response(&request, canonical::State::Data, Some(&source()), Some(&d));
    assert!(codec().decode_reply(&b, &e).is_ok());
    request.load_proof[0] ^= 1;
    let (e, b) = response(&request, canonical::State::Data, Some(&source()), Some(&d));
    assert!(codec().decode_reply(&b, &e).is_err());
    let mut other = q().canonical;
    other.context.key.source_id = 8;
    other.action = canonical::Action::Loaded;
    other.digest = source().plan.canonical_sha256;
    request.load_proof = Sha256::digest(load_proof_preimage(&source(), &other).unwrap()).into();
    let (e, b) = response(&request, canonical::State::Data, Some(&source()), Some(&d));
    assert!(codec().decode_reply(&b, &e).is_err());
}
#[test]
fn dropping_owner_cancels_worker_and_disabled_owner_cannot_be_enabled_by_version() {
    assert!(DraftOwner::local_experiment(DraftCodec::default()).is_err());
    let cancel = {
        let mut owner = DraftOwner::local_experiment(codec()).unwrap();
        let ticket = owner
            .stage(
                scope(),
                reply(
                    &prepare(),
                    canonical::State::BeforeEngine,
                    Some(&source()),
                    None,
                ),
            )
            .unwrap();
        owner.cancellation(ticket).unwrap()
    };
    assert!(cancel.load(Ordering::Acquire));
}
#[test]
fn source_schema_roles_digest_manifest_and_trailing_corruption_reject() {
    let b = source().encode(ack().capabilities).unwrap();
    for at in [
        0, 2, 4, 6, 8, 40, 44, 76, 78, 80, 82, 84, 92, 124, 126, 256, 260, 262, 264, 268,
    ] {
        let mut bad = b.clone();
        bad[at] ^= 1;
        assert!(
            SourceAttachment::decode(&bad, ack().capabilities).is_err(),
            "offset{at}"
        );
    }
    let mut bad = b.clone();
    bad.push(0);
    assert!(SourceAttachment::decode(&bad, u32::MAX).is_err());
    for n in 0..b.len() {
        assert!(SourceAttachment::decode(&b[..n], u32::MAX).is_err());
    }
    let mut s = source();
    s.plan.namespace = resources::ResourceNamespace::Shared;
    s.resource_set = s.computed_resource_set();
    assert!(s.encode(u32::MAX).is_err());
    let mut s = source();
    s.plan.archives.swap(0, 1);
    s.resource_set = s.computed_resource_set();
    assert!(s.encode(u32::MAX).is_err());
}
#[test]
fn manifest_order_sector_count_capability_and_aggregate_bounds() {
    let mut s = source();
    let template = s.plan.sectors[0];
    s.plan.sectors = (0..64)
        .map(|i| {
            let mut t = template;
            t.key.x = i;
            t.pin.bytes = 49158;
            t
        })
        .collect();
    assert_eq!(s.encode(u32::MAX).unwrap().len(), 2860);
    assert!(s
        .encode(ack().capabilities & !protocol::CAP_LARGE_MAPS)
        .is_err());
    s.plan.sectors.push(template);
    assert!(s.encode(u32::MAX).is_err());
    s.plan.sectors.pop();
    s.plan.sectors.swap(0, 1);
    assert!(s.encode(u32::MAX).is_err());
    s.plan.sectors.swap(0, 1);
    s.plan.sectors[1] = s.plan.sectors[0];
    assert!(s.encode(u32::MAX).is_err());
    let mut s = source();
    s.plan.sectors = (0..5)
        .map(|i| {
            let mut t = template;
            t.key.x = i;
            t.pin.bytes = 8 << 20;
            t
        })
        .collect();
    assert!(s.encode(u32::MAX).is_err());
    for role in 0..4 {
        let mut s = source();
        s.plan.archives[role].pin.bytes = if role % 2 == 0 {
            (8 << 20) + 1
        } else {
            (256 << 20) + 1
        };
        s.resource_set = s.computed_resource_set();
        assert!(s.encode(u32::MAX).is_err());
    }
    let mut short = source();
    short.plan.sectors[0].pin.bytes = 49157;
    assert!(short.encode(u32::MAX).is_err());
    let mut zero_map = source();
    zero_map.plan.map_id = 0;
    zero_map.plan.sectors[0].key.map_id = 0;
    zero_map.resource_set = zero_map.computed_resource_set();
    assert_eq!(
        SourceAttachment::decode(&zero_map.encode(u32::MAX).unwrap(), u32::MAX).unwrap(),
        zero_map
    );
}
#[test]
fn new_presence_matrix_keeps_error_and_leave_echo_separate_from_resource_work() {
    for (status, reason) in [(1, 4), (2, 6), (3, 5)] {
        let (e, mut b) = response(
            &prepare(),
            canonical::State::Detached,
            Some(&source()),
            None,
        );
        b[528] = status;
        b[529] = reason;
        let parsed = codec().decode_reply(&b, &e).unwrap();
        let mut owner = DraftOwner::local_experiment(codec()).unwrap();
        assert!(owner.stage(scope(), parsed).is_err());
        let (e, mut absent) = response(&prepare(), canonical::State::Detached, None, None);
        absent[188] = status;
        absent[189] = reason;
        assert!(codec().decode_reply(&absent, &e).is_err());
    }
    let mut leave = q();
    leave.canonical.action = canonical::Action::Leave;
    let parsed = reply(&leave, canonical::State::Detached, Some(&source()), None);
    let mut owner = DraftOwner::local_experiment(codec()).unwrap();
    assert!(owner.stage(scope(), parsed).is_err());
    let (e, b) = response(&leave, canonical::State::Detached, None, None);
    assert!(codec().decode_reply(&b, &e).is_err());
    let (e, mut unavailable) = response(&first(), canonical::State::Detached, None, None);
    unavailable[188] = 3;
    unavailable[189] = 5;
    assert!(codec().decode_reply(&unavailable, &e).is_ok());
}
#[test]
fn detached_before_engine_data_loaded_and_leave_reuse_canonical_dtos() {
    let empty = reply(&first(), canonical::State::Detached, None, None);
    assert!(empty.source.is_none());
    let (e, b) = response(&first(), canonical::State::Detached, None, None);
    assert_eq!(b.len(), 244);
    assert!(codec().decode_reply(&b, &e).is_ok());
    let pending = reply(&first(), canonical::State::Detached, Some(&source()), None);
    assert!(pending.canonical.attachment.is_none());
    let (e, b) = response(
        &prepare(),
        canonical::State::BeforeEngine,
        Some(&source()),
        None,
    );
    assert_eq!(b.len(), 584);
    assert!(codec()
        .decode_reply(&b, &e)
        .unwrap()
        .canonical
        .data
        .is_none());
    let loading = reply(&q(), canonical::State::Data, Some(&source()), Some(&data()));
    assert_eq!(loading.canonical.data, Some(data()));
    let mut loaded = q();
    loaded.canonical.action = canonical::Action::Loaded;
    loaded.canonical.digest = source().plan.canonical_sha256;
    loaded.load_proof = [9; 32];
    let mut d = data();
    d.actors.0[0].loaded = true;
    loaded.load_proof =
        Sha256::digest(load_proof_preimage(&source(), &loaded.canonical).unwrap()).into();
    reply(&loaded, canonical::State::Data, Some(&source()), Some(&d));
    let mut leave = q();
    leave.canonical.action = canonical::Action::Leave;
    reply(&leave, canonical::State::Detached, Some(&source()), None);
}
#[test]
fn reply_missing_source_echo_identity_and_exact_data_pins_reject() {
    let (expected, bytes) = response(&q(), canonical::State::Data, Some(&source()), Some(&data()));
    for at in [0, 4, 8, 12, 14, 16, 20, 52, 84, 88, 89, 90, 91, 92, 94, 100] {
        let mut bad = bytes.clone();
        bad[at] ^= 1;
        assert!(codec().decode_reply(&bad, &expected).is_err(), "offset{at}");
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(codec().decode_reply(&extra, &expected).is_err());
    for state in [canonical::State::BeforeEngine, canonical::State::Data] {
        let fixture = data();
        let (e, b) = response(
            &q(),
            state,
            None,
            if state == canonical::State::Data {
                Some(&fixture)
            } else {
                None
            },
        );
        assert!(codec().decode_reply(&b, &e).is_err());
    }
    let mut s = source();
    s.plan.sectors[0].pin.sha256 = [3; 32];
    let (e, b) = response(&q(), canonical::State::Data, Some(&s), Some(&data()));
    assert!(codec().decode_reply(&b, &e).is_err());
    let mut s = source();
    s.plan.sectors[0].pin.bytes += 1;
    let (e, b) = response(&q(), canonical::State::Data, Some(&s), Some(&data()));
    assert!(codec().decode_reply(&b, &e).is_err());
    let mut r = q();
    r.canonical.context.key.source_id = 8;
    assert!(codec()
        .decode_reply(&bytes, &codec().encode_request(9, &r).unwrap())
        .is_err());
}
struct HeaderOnly {
    header: Vec<u8>,
    at: usize,
}
#[test]
fn denied_busy_unavailable_and_aborted_publications_cannot_start_resource_work() {
    let mut owner = DraftOwner::local_experiment(codec()).unwrap();
    let pending = owner
        .stage(
            scope(),
            reply(&first(), canonical::State::Detached, Some(&source()), None),
        )
        .unwrap();
    let cancel = owner.cancellation(pending).unwrap();
    for status in [
        world::Status::Denied,
        world::Status::Busy,
        world::Status::Unavailable,
    ] {
        let mut denied = reply(
            &prepare(),
            canonical::State::BeforeEngine,
            Some(&source()),
            None,
        );
        denied.canonical.status = status;
        denied.canonical.reason = world::Reason::NotReady;
        denied.validate().unwrap();
        assert!(owner.stage(scope(), denied).is_err());
        assert!(owner.asset_plan(pending).is_ok());
        assert!(!cancel.load(Ordering::Acquire));
    }
    let mut aborted = reply(&q(), canonical::State::BeforeEngine, Some(&source()), None);
    let lifecycle = &mut aborted.canonical.attachment.as_mut().unwrap().lifecycle;
    lifecycle.phase = world::Phase::Aborted;
    lifecycle.revision = 2;
    lifecycle.abort = world::AbortReason::PreparationFailed;
    aborted.validate().unwrap();
    assert!(owner.stage(scope(), aborted).is_err());
    assert!(owner.asset_plan(pending).is_ok());
    assert!(!cancel.load(Ordering::Acquire));
}
impl Read for HeaderOnly {
    fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
        assert!(self.at < 16, "allocation/body read before length rejection");
        let n = b.len().min(16 - self.at);
        b[..n].copy_from_slice(&self.header[self.at..self.at + n]);
        self.at += n;
        Ok(n)
    }
}
#[test]
fn per_selected_maximum_is_enforced_before_body_read() {
    let expected = codec().encode_request(9, &q()).unwrap();
    for length in [
        243,
        codec().maximum_result().unwrap() + 1,
        u32::MAX as usize,
    ] {
        let header = envelope_header(9, RESULT_KIND, length).unwrap();
        let mut reader = HeaderOnly { header, at: 0 };
        assert!(codec().read_reply(&mut reader, &expected).is_err());
        assert_eq!(reader.at, 16);
    }
    let armor =
        DraftCodec::local_experiment(ack(), NATIVE_BIT | protocol::extensions::ARMOR.required)
            .unwrap();
    assert_eq!(
        armor.maximum_result().unwrap(),
        3104 + payload::armor::MAX_BYTES
    );
}
#[test]
fn owner_scope_retirement_stale_and_cross_owner_tickets_reject() {
    let mut owner = DraftOwner::local_experiment(codec()).unwrap();
    let first_ticket = owner
        .stage(
            scope(),
            reply(&first(), canonical::State::Detached, Some(&source()), None),
        )
        .unwrap();
    assert_eq!(owner.asset_plan(first_ticket).unwrap(), &source().plan);
    let first_cancel = owner.cancellation(first_ticket).unwrap();
    assert!(!first_cancel.load(Ordering::Acquire));
    let next = owner
        .stage(
            scope(),
            reply(
                &prepare(),
                canonical::State::BeforeEngine,
                Some(&source()),
                None,
            ),
        )
        .unwrap();
    assert!(owner.asset_plan(first_ticket).is_err());
    assert!(first_cancel.load(Ordering::Acquire));
    assert!(owner.asset_plan(next).is_ok());
    let loading = owner
        .stage(
            scope(),
            reply(&q(), canonical::State::Data, Some(&source()), Some(&data())),
        )
        .unwrap();
    assert!(owner.asset_plan(next).is_err());
    assert!(owner.asset_plan(loading).is_ok());
    for field in 0..6 {
        let mut wrong = scope();
        match field {
            0 => wrong.connection_incarnation += 1,
            1 => wrong.context.key.source_id += 1,
            2 => wrong.context.epoch += 1,
            3 => wrong.channel = 1,
            4 => wrong.joined_room += 1,
            _ => wrong.room_revision += 1,
        }
        assert!(owner
            .stage(
                wrong,
                reply(&q(), canonical::State::Data, Some(&source()), Some(&data()))
            )
            .is_err());
        assert!(owner.asset_plan(loading).is_ok());
    }
    assert!(!owner.loaded_admission_available());
    let mut other = DraftOwner::local_experiment(codec()).unwrap();
    other
        .stage(
            scope(),
            reply(&q(), canonical::State::Data, Some(&source()), Some(&data())),
        )
        .unwrap();
    assert!(other.asset_plan(loading).is_err());
    let latest_cancel = owner.cancellation(loading).unwrap();
    owner.retire();
    assert!(latest_cancel.load(Ordering::Acquire));
    assert!(owner.asset_plan(loading).is_err());
    assert!(!owner.loaded_admission_available());
}
