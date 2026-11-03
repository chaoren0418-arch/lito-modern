use lito_modern::modern_lito::{native_light_color::Remap,
    native_light_mask::{Mask, Source, SOURCE_SHA256}, native_light_sources::{self as light, Actor, Master, Policy}};
use sha2::{Digest, Sha256};
const LMP: &[u8] = include_bytes!("fixtures/native-light-source.lmp");
const VECTORS: &[u8] = include_bytes!("fixtures/native-light-sources-original.json");

fn actor(v: &serde_json::Value) -> Actor {
    let a = v.as_array().unwrap(); assert_eq!(a.len(), 14);
    let n = |i: usize| a[i].as_i64().unwrap();
    Actor { position: [n(0) as i32, n(1) as i32], raw_light: n(2) as u8,
        action: n(3) as u8, walk_action: n(4) as u8, direction: n(5) as u8,
        source_frame: n(6) as u8, frame_count: n(7) as u32,
        base_class: n(8) as u8, subtype: n(9) as u8,
        flag_1f: n(10) != 0, flag_50: n(11) != 0, mode_23: n(12) as u8,
        master: match n(13) {-1 | 0 => Master::None, 1 => Master::Own, 2 => Master::Other, _ => panic!("fixture master domain")},
    }
}

#[test]
fn all_original_graph_masks_queries_remap_cache_and_blit_order_match() {
    assert_eq!(format!("{:x}", Sha256::digest(LMP)), SOURCE_SHA256);
    assert_eq!(format!("{:X}", Sha256::digest(VECTORS)), "681200C29BAD895725F09395975868521C785D516BD3EFA00694D6333F2C9AE0");
    let source = Source::parse(LMP).unwrap();
    let root: serde_json::Value = serde_json::from_slice(VECTORS).unwrap();
    let cases = root["cases"].as_array().unwrap(); assert_eq!(cases.len(), 864);
    let mut blits = 0;
    for (index, case) in cases.iter().enumerate() {
        let h = case["header"].as_array().unwrap();
        let n = |i: usize| h[i].as_i64().unwrap();
        let actors: Vec<_> = case["actors"].as_array().unwrap().iter().map(actor).collect();
        let mut mask = Mask::new(320, 204).unwrap();
        let initial: Vec<_> = (0..65280).map(|i| ((i * 13 + n(7) as usize) & 255) as u8).collect();
        mask.compose_plane(&initial, 320, 204, 0, 0).unwrap();
        let initial_row = std::array::from_fn(|i| ((i * 7 + n(9) as usize) & 31) as u8);
        let mut remap = Remap::restore(if n(8) < 0 {None} else {Some(n(8) as u8)}, initial_row).unwrap();
        let centers: Vec<_> = actors.iter().map(|a| light::native_center(a,
            [n(3) as i32, n(4) as i32], [n(5) as i32, n(6) as i32]).unwrap()).collect();
        let policy = Policy {ambient: n(0) as u32, underwater: n(1) != 0, blind: n(2) as u8,
            own_index: (!actors.is_empty()).then_some(0),
            own_position: actors.first().map_or([1000, 1000], |a| a.position)};
        let receipt = light::compose(&source, &mut mask, &mut remap, policy, &actors, &centers).unwrap();
        let expected = &case["expected"];
        assert_eq!(format!("{:X}", Sha256::digest(mask.pixels())), expected["mask_sha256"].as_str().unwrap(), "native graph case{index}");
        let row: Vec<_> = expected["remap"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect();
        assert_eq!(remap.row().as_slice(), row, "remap case{index}");
        assert_eq!(remap.level().map_or(-1, i32::from), expected["cached"].as_i64().unwrap() as i32);
        let native_blits = expected["blits"].as_array().unwrap();
        assert_eq!(receipt.blits.len(), native_blits.len(), "blit count case{index}");
        for (actual, original) in receipt.blits.iter().zip(native_blits) {
            let offset = u32::from_le_bytes(LMP[usize::from(actual.range) * 4..usize::from(actual.range) * 4 + 4].try_into().unwrap());
            let original: Vec<_> = original.as_array().unwrap().iter().map(|v| v.as_i64().unwrap()).collect();
            assert_eq!([actual.origin[0], actual.origin[1], actual.size[0] as i64, actual.size[1] as i64, i64::from(offset)].as_slice(), original.as_slice(), "blit case{index}");
        }
        let queries: Vec<u64> = actors.iter().flat_map(|a| [u64::from(light::classify(a)),
            light::blind_two_source(a) as u64, u64::from(a.frame_count)]).collect();
        let original: Vec<_> = expected["actor_queries"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap()).collect();
        assert_eq!(queries, original, "classification queries case{index}");
        blits += receipt.blits.len();
    }
    assert_eq!(blits, 1166);
}

#[test]
fn owner_and_extent_failures_are_atomic_and_native_phase_bounds_are_explicit() {
    let source = Source::parse(LMP).unwrap();
    let mut mask = Mask::new(3, 2).unwrap(); mask.compose_plane(&[37; 6], 3, 2, 0, 0).unwrap();
    let mut remap = Remap::restore(Some(4), [7; 32]).unwrap();
    let a = Actor {position: [0, 0], raw_light: 5, action: 0, walk_action: 0, direction: 0,
        source_frame: 0, frame_count: 4, base_class: 1, subtype: 0, flag_1f: false, flag_50: false,
        mode_23: 0, master: Master::None};
    let policy = Policy {ambient: 5, underwater: false, blind: 0, own_index: Some(1), own_position: [0, 0]};
    assert!(light::compose(&source, &mut mask, &mut remap, policy, &[a], &[[0, 0]]).is_err());
    assert_eq!(mask.pixels(), &[37; 6]); assert_eq!(remap.level(), Some(4)); assert_eq!(remap.row(), &[7; 32]);
    assert!(light::compose(&source, &mut mask, &mut remap, Policy {own_index: Some(0), blind: 3, ..policy}, &[a], &[[0, 0]]).is_err());
    assert_eq!(mask.pixels(), &[37; 6]);
    assert!(light::native_center(&Actor {direction: 8, ..a}, [0, 0], [0, 0]).is_err());
    assert!(light::native_center(&Actor {frame_count: 12, source_frame: 12, ..a}, [0, 0], [0, 0]).is_err());
    assert!(light::native_center(&Actor {position: [i32::MAX, 0], ..a}, [i32::MIN, 0], [0, 0]).is_err());
}
