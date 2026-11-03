//! Frozen Java prototype bytes only. No Java/server/SQL/TLS/UI is launched.
use super::*;
use serde_json::{json, Value};
use std::{collections::BTreeMap, fs, path::Path};
fn hex(s: &str) -> Vec<u8> {
    assert_eq!(s.len() % 2, 0);
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| u8::from_str_radix(std::str::from_utf8(p).unwrap(), 16).unwrap())
        .collect()
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 47,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
fn codec() -> DraftCodec {
    DraftCodec::local_experiment(ack(), NATIVE_BIT).unwrap()
}
fn sha(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn rejected(rows: &mut Vec<Value>, label: &str, category: &str, result: bool) {
    assert!(result, "Java negative equivalent accepted: {label}");
    rows.push(json!({"case":label,"category":category,"rust_rejected":true}));
}
fn replace_source(frame: &[u8], source: &SourceAttachment) -> Vec<u8> {
    let old = u32_at(frame, 84) as usize;
    let bytes = source.encode(u32::MAX).unwrap();
    let mut out = frame[..84].to_vec();
    out.extend((bytes.len() as u32).to_be_bytes());
    out.extend([0; 4]);
    out.extend(bytes);
    out.extend(&frame[92 + old..]);
    let len = out.len() as u32;
    out[4..8].copy_from_slice(&len.to_be_bytes());
    out
}
struct HeaderOnly {
    bytes: Vec<u8>,
    read: usize,
}
impl Read for HeaderOnly {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        assert!(self.read < 16, "over-limit reply body was read");
        let n = out.len().min(16 - self.read);
        out[..n].copy_from_slice(&self.bytes[self.read..self.read + n]);
        self.read += n;
        Ok(n)
    }
}
#[test]
#[ignore = "requires own pinned R428 frozen Java codec snapshot; no producer mutable sources"]
fn frozen_java_six_requests_eight_replies_proofs_and_43_negative_equivalents() {
    let input =
        std::env::var("LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT").expect("fixed input required");
    let input_bytes = fs::read(&input).unwrap();
    let expected_input =
        std::env::var("LITO_NATIVE_CANONICAL_PROTOTYPE_INPUT_SHA256").expect("input pin required");
    assert_eq!(sha(&input_bytes), expected_input.to_ascii_lowercase());
    let config: Value = serde_json::from_slice(&input_bytes).unwrap();
    assert_eq!(
        config["state"],
        "FROZEN_CODEC_PROTOTYPE_ONLY_NOT_FORMAL_WIRE"
    );
    assert_eq!(config["file_count"], 272);
    assert_eq!(
        config["manifest_sha256"],
        "B09515C8571B172AA47A06CCFE66C5E7D1E0C138E1A0242AFD179B26CD043468"
    );
    let root = Path::new(config["root"].as_str().unwrap());
    assert!(Path::new(&input).parent().unwrap() == root);
    let vector_path = Path::new(config["vectors"].as_str().unwrap());
    assert_eq!(vector_path, root.join("canonical-codec-vectors.json"));
    let bytes = fs::read(vector_path).unwrap();
    assert_eq!(
        sha(&bytes),
        "a40f157e411ca16dd71547c87d12f108d0e4d301601f7596dd2d928f94768b34"
    );
    let v: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(v["schema"], 1);
    let probe_bytes = fs::read(root.join("canonical-codec-probe.json")).unwrap();
    assert_eq!(
        sha(&probe_bytes),
        "4d36e5f1dddc3fcd9c2f5c2144915347019a9c98213b012d6ad66f58968bf5b5"
    );
    let probe: Value = serde_json::from_slice(&probe_bytes).unwrap();
    assert_eq!(
        codec().maximum_result().unwrap(),
        probe["selected_max_result_bytes"].as_u64().unwrap() as usize
    );
    let mut requests = BTreeMap::new();
    let mut request_rows = Vec::new();
    for r in v["requests"].as_array().unwrap() {
        let bytes = hex(r["hex"].as_str().unwrap());
        assert_eq!(bytes.len(), 180);
        let request = codec().decode_request(&bytes).unwrap();
        assert_eq!(
            codec().encode_request(u32_at(&bytes, 8), &request).unwrap(),
            bytes
        );
        request_rows.push(json!({"case":r["action"],"bytes":bytes.len(),"sha256":sha(&bytes),"decode_and_exact_reencode":true}));
        requests.insert(u32_at(&bytes, 8), (request, bytes));
    }
    assert_eq!(requests.len(), 6);
    let mut replies = Vec::new();
    replies.push((
        "DETACHED".to_owned(),
        hex(v["detached_hex"].as_str().unwrap()),
        false,
    ));
    replies.push((
        "PREPARING".to_owned(),
        hex(v["preparing_hex"].as_str().unwrap()),
        true,
    ));
    for r in v["presence_vectors"].as_array().unwrap() {
        replies.push((
            r["case"].as_str().unwrap().to_owned(),
            hex(r["hex"].as_str().unwrap()),
            r["source_present"].as_bool().unwrap(),
        ));
    }
    assert_eq!(replies.len(), 8);
    let mut reply_rows = Vec::new();
    for (name, bytes, present) in &replies {
        let expected = &requests[&u32_at(bytes, 8)].1;
        let parsed = codec().decode_reply(bytes, expected).unwrap();
        assert_eq!(parsed.source.is_some(), *present);
        assert!(parsed.canonical.data.is_none());
        if let Some(source) = &parsed.source {
            let n = u32_at(bytes, 84) as usize;
            assert_eq!(
                source.encode(ack().capabilities).unwrap(),
                bytes[92..92 + n]
            );
            assert_eq!(
                source.resource_set.as_slice(),
                hex(v["source_set_digest"].as_str().unwrap())
            );
            assert_eq!(source.plan.map_id, 564);
            assert_eq!(source.plan.sectors.len(), 1);
            assert_eq!(source.plan.sectors[0].pin.bytes, 128028);
        }
        reply_rows.push(json!({"case":name,"bytes":bytes.len(),"sha256":sha(bytes),"source_present":present,
            "canonical_state":parsed.canonical.state as u8,"canonical_status":parsed.canonical.status as u8,
            "canonical_body_parsed_without_legacy_frame":true,"source_attachment_exact_reencode":present}));
    }
    let preparing = hex(v["preparing_hex"].as_str().unwrap());
    let prepare_wire = &requests[&11].1;
    let source = codec()
        .decode_reply(&preparing, prepare_wire)
        .unwrap()
        .source
        .unwrap();
    let mut proof_rows = Vec::new();
    for id in [12, 20] {
        let q = &requests[&id].0;
        let preimage = load_proof_preimage(&source, &q.canonical).unwrap();
        assert_eq!(preimage.len(), 135);
        assert_eq!(sha(&preimage), format!("{:x}", Sha256::digest(&preimage)));
        assert_eq!(Sha256::digest(&preimage).as_slice(), q.load_proof);
        source.check_claim(q, ack().capabilities).unwrap();
        proof_rows.push(json!({"request_id":id,"epoch_high_bit":q.canonical.context.epoch>>63==1,
            "preimage_bytes":135,"preimage_hex":preimage.iter().map(|b|format!("{b:02x}")).collect::<String>(),
            "digest":sha(&preimage),"matches_java_proof":true}));
    }
    // Equivalent43 codec/selection/claim/metadata checks reconstructed from
    // the fixed probe source; they are not55 independently published bad HEX.
    let mut neg = Vec::new();
    rejected(
        &mut neg,
        "encode_unselected",
        "selection",
        DraftCodec::default()
            .encode_request(11, &requests[&11].0)
            .is_err(),
    );
    rejected(
        &mut neg,
        "decode_unselected",
        "selection",
        DraftCodec::default().decode_request(prepare_wire).is_err(),
    );
    rejected(
        &mut neg,
        "result_unselected",
        "selection",
        DraftCodec::default()
            .decode_reply(&preparing, prepare_wire)
            .is_err(),
    );
    for label in [
        "version_only_request_encode",
        "version_only_result_encode",
        "version_only_request_decode",
        "unselected_stream",
    ] {
        rejected(
            &mut neg,
            label,
            "draft_gate_equivalent_not_live_application_dispatch",
            DraftCodec::default().peer().is_err(),
        );
    }
    let mut huge_request = prepare_wire[..16].to_vec();
    huge_request[4..8].copy_from_slice(&(i32::MAX as u32).to_be_bytes());
    rejected(
        &mut neg,
        "request_integer_max_header",
        "wire",
        codec().decode_request(&huge_request).is_err(),
    );
    let mut huge_result = preparing[..16].to_vec();
    huge_result[4..8]
        .copy_from_slice(&((codec().maximum_result().unwrap() + 1) as u32).to_be_bytes());
    let mut header_only = HeaderOnly {
        bytes: huge_result,
        read: 0,
    };
    rejected(
        &mut neg,
        "result_max_plus_one_before_body_read",
        "wire",
        codec().read_reply(&mut header_only, prepare_wire).is_err(),
    );
    assert_eq!(header_only.read, 16);
    let mut nav = ack();
    nav.capabilities = 0x6f;
    rejected(
        &mut neg,
        "navigation_only_dependencies",
        "selection",
        DraftCodec::local_experiment(nav, NATIVE_BIT).is_err(),
    );
    for at in [0, 4, 12, 14, 16, 19, 108, 109] {
        let mut bad = prepare_wire.clone();
        bad[at] ^= 0xff;
        rejected(
            &mut neg,
            &format!("request_offset_{at}"),
            "wire",
            codec().decode_request(&bad).is_err(),
        );
    }
    let mut bad = prepare_wire.clone();
    bad[8..12].fill(0);
    rejected(
        &mut neg,
        "zero_request_id",
        "wire",
        codec().decode_request(&bad).is_err(),
    );
    let mut bad = prepare_wire.clone();
    bad[20..52].fill(0);
    rejected(
        &mut neg,
        "prepare_missing_expected_digest",
        "wire",
        codec().decode_request(&bad).is_err(),
    );
    let mut bad = requests[&12].1.clone();
    bad[52] ^= 1;
    let q = codec().decode_request(&bad).unwrap();
    rejected(
        &mut neg,
        "wrong_loaded_proof",
        "claim",
        source.check_claim(&q, ack().capabilities).is_err(),
    );
    let mut q = requests[&12].0.clone();
    q.canonical.context.key.source_id = 42;
    rejected(
        &mut neg,
        "foreign_source_same_object_proof",
        "claim",
        source.check_claim(&q, ack().capabilities).is_err(),
    );
    let mut q = requests[&12].0.clone();
    q.canonical.generation = 12;
    rejected(
        &mut neg,
        "stale_generation_proof",
        "claim",
        source.check_claim(&q, ack().capabilities).is_err(),
    );
    for at in [84, 88, 92, 95, 96, 99, 128, 136, 168, 172, 216, 348] {
        let mut bad = preparing.clone();
        bad[at] ^= 0xff;
        rejected(
            &mut neg,
            &format!("result_offset_{at}"),
            "wire",
            codec().decode_reply(&bad, prepare_wire).is_err(),
        );
    }
    let mut bad = preparing.clone();
    bad.push(0);
    rejected(
        &mut neg,
        "result_trailing_byte",
        "wire",
        codec().decode_reply(&bad, prepare_wire).is_err(),
    );
    let mut large = source.clone();
    let template = source.plan.sectors[0];
    large.plan.sectors = (0..17)
        .map(|y| {
            let mut s = template;
            s.key.y = y;
            s
        })
        .collect();
    let no_large = ack().capabilities & !protocol::CAP_LARGE_MAPS;
    rejected(
        &mut neg,
        "pre_engine17_source_encode_without_large_maps",
        "metadata",
        large.encode(no_large).is_err(),
    );
    let mut a = ack();
    a.capabilities = no_large;
    let reader = DraftCodec::local_experiment(a, NATIVE_BIT).unwrap();
    let large_wire = replace_source(&preparing, &large);
    rejected(
        &mut neg,
        "pre_engine17_result_without_large_maps",
        "wire",
        reader.decode_reply(&large_wire, prepare_wire).is_err(),
    );
    rejected(
        &mut neg,
        "pre_engine17_claim_without_large_maps",
        "claim",
        large.check_claim(&requests[&11].0, no_large).is_err(),
    );
    let mut budget = source.clone();
    budget.plan.sectors = (0..5)
        .map(|y| {
            let mut s = template;
            s.key.y = y;
            s.pin.bytes = 8 << 20;
            s
        })
        .collect();
    rejected(
        &mut neg,
        "five8MiB_aggregate",
        "metadata",
        budget.validate(ack().capabilities).is_err(),
    );
    let mut duplicate = source.clone();
    duplicate.plan.sectors.push(template);
    rejected(
        &mut neg,
        "duplicate_sector",
        "metadata",
        duplicate.validate(ack().capabilities).is_err(),
    );
    // Missing attachment for a nonzero expected request, still valid canonical
    // BUSY DTO: remove the source and correct full frame length.
    let busy = replies.iter().find(|r| r.0 == "BUSY").unwrap();
    let n = u32_at(&busy.1, 84) as usize;
    let mut absent = busy.1[..84].to_vec();
    absent.extend([0; 8]);
    absent.extend(&busy.1[92 + n..]);
    let length = absent.len() as u32;
    absent[4..8].copy_from_slice(&length.to_be_bytes());
    rejected(
        &mut neg,
        "prepare_busy_missing_source",
        "wire",
        codec().decode_reply(&absent, prepare_wire).is_err(),
    );
    let mut wrong = preparing.clone();
    wrong[20..52].fill(0x11);
    let mut expected = prepare_wire.clone();
    expected[20..52].fill(0x11);
    rejected(
        &mut neg,
        "result_wrong_expected_resource_set",
        "wire",
        codec().decode_reply(&wrong, &expected).is_err(),
    );
    assert_eq!(neg.len(), 43);
    // Same accepted bytes may not create a new resource worker on failure,
    // abort or LEAVE. A parsed proof still never opens LOADED admission.
    let scope = Scope {
        connection_incarnation: 1,
        context: requests[&11].0.canonical.context,
        channel: 0,
        joined_room: 1,
        room_revision: 10,
    };
    for name in ["DENIED", "BUSY", "UNAVAILABLE", "ABORTED", "LEAVE"] {
        let bytes = &replies.iter().find(|r| r.0 == name).unwrap().1;
        let r = codec()
            .decode_reply(bytes, &requests[&u32_at(bytes, 8)].1)
            .unwrap();
        let mut owner = DraftOwner::local_experiment(codec()).unwrap();
        assert!(owner.stage(scope, r).is_err());
        assert!(!owner.loaded_admission_available());
    }
    eprintln!(
        "NATIVE_CANONICAL_PROTOTYPE_RESULT {}",
        json!({
            "scope":"Frozen Java codec prototype only; directional Rust consumer and equivalent negative checks",
            "manifest_sha256":config["manifest_sha256"],"vectors_sha256":sha(&bytes),
            "requests":request_rows,"pre_engine_replies":reply_rows,"proofs":proof_rows,
            "negative_equivalents":neg,"negative_equivalents_passed":43,
            "producer_negative_assertions":55,"ledger_state_assertions_not_proved_by_bytes":12,
            "java_live_application_dispatch_proved":false,"java_DATA_vector_present":false,
            "server_service_generation_fence_proved":false,"TLS_SQL":false,
            "renderer_LOADED_admission":false,"qualified_modes_added":0,"GUI":0
        })
    );
}
