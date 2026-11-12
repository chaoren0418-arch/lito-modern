//! #165 (T028-3) skill 72 Counter Detection: the exact 3934-0.spr empty-slot
//! policy. Native 4CD9D0 skips drawing for a zero-block frame; the bank keeps
//! the slot's dwell and audio but submits no pixels. Only slots 0 and 12 of
//! the pinned original file qualify; every other file or slot stays on the
//! ordinary non-empty referenced-frame rule.
use super::*;
use crate::modern_lito::sprite::{SpriteBlockReference, SpriteFrameFragments};

fn blank(left: i32) -> SpriteFrame {
    SpriteFrame {
        fragments: None,
        x_offset: left,
        y_offset: 120,
        width: 0,
        height: 0,
        frame_type: 0,
        unknown1: u16::MAX,
        unknown2: 0,
        rgba: Vec::new(),
    }
}

fn visible() -> SpriteFrame {
    SpriteFrame { x_offset: -10, y_offset: -10, width: 2, height: 1, unknown1: 0, rgba: vec![255; 8], ..blank(0) }
}

#[test]
fn counter3934_policy_pins_exact_file_identity_and_slot_left() {
    let row = REVIEWED_SELECTOR_EMPTY_SPRS.iter().find(|(n, _, _, _)| *n == "3934-0.spr").unwrap();
    assert_eq!(row.1, 1_168_552);
    assert_eq!(row.3, 352);
    let hex: String = row.2.iter().map(|b| format!("{b:02x}")).collect();
    // Independently recomputed from the PakViewer-extracted original (#165).
    assert_eq!(hex, "97183ebe2307d1a7f5d244459ae7b8434072544dc800c82d94b4b1c441f03753");
    // 3936 (holy walk) keeps its own, different reviewed left offset.
    assert_eq!(REVIEWED_SELECTOR_EMPTY_SPRS.iter().find(|(n, _, _, _)| *n == "3936-0.spr").unwrap().3, 192);
}

#[test]
fn counter3934_only_slots_0_and_12_with_exact_zero_block_metadata_are_blank() {
    let b = blank(352);
    for index in 0..13 {
        assert_eq!(is_reviewed_selector_blank("3934-0.spr", index, &b), matches!(index, 0 | 12), "slot {index}");
    }
    // Other files, including a neighbour id and the other reviewed selector
    // file with this file's offset, never inherit the policy.
    for name in ["3935-0.spr", "3934-1.spr", "3936-0.spr", "170-0.spr"] {
        assert!(!is_reviewed_selector_blank(name, 0, &b), "{name}");
    }
    assert!(is_reviewed_selector_blank("3936-0.spr", 0, &blank(192)));
    let mut changed = Vec::new();
    for edit in 0..8 {
        let mut f = blank(352);
        match edit {
            0 => f.x_offset = 351,
            1 => f.y_offset = 121,
            2 => f.width = 1,
            3 => f.height = 1,
            4 => f.unknown1 = 0,
            5 => f.unknown2 = 1,
            6 => f.rgba = vec![0; 4],
            _ => {
                f.fragments = Some(SpriteFrameFragments {
                    references: vec![SpriteBlockReference { half_x: 0, y: 0, depth: 0, block_id: 0 }],
                    atlas: Arc::new(Vec::new()),
                })
            }
        }
        changed.push(f);
    }
    for (edit, f) in changed.iter().enumerate() {
        assert!(!is_reviewed_selector_blank("3934-0.spr", 0, f), "edit {edit}");
        assert!(!is_reviewed_selector_blank("3934-0.spr", 12, f), "edit {edit}");
    }
    // An empty reference list (decoded zero-block frame) is still blank.
    let mut empty_refs = blank(352);
    empty_refs.fragments = Some(SpriteFrameFragments { references: Vec::new(), atlas: Arc::new(Vec::new()) });
    assert!(is_reviewed_selector_blank("3934-0.spr", 12, &empty_refs));
}

#[test]
fn counter3934_source_admission_rejects_forged_bytes_counts_and_slot_shapes() {
    let mut frames = vec![blank(352)];
    frames.extend((1..12).map(|_| visible()));
    frames.push(blank(352));
    let decoded = DecodedSprite { frames, has_palette: false, block_table_metadata: 0 };
    // Exact shape but not the pinned original bytes: size and SHA must fail.
    assert!(!reviewed_selector_empty_source_matches("3934-0.spr", &[0; 16], &decoded));
    assert!(!reviewed_selector_empty_source_matches("3934-0.spr", &vec![0; 1_168_552], &decoded));
    // Unknown names never qualify, whatever the bytes.
    assert!(!reviewed_selector_empty_source_matches("3935-0.spr", &[0; 16], &decoded));
    let mut wrong_count = decoded.clone();
    wrong_count.frames.pop();
    assert!(!reviewed_selector_empty_source_matches("3934-0.spr", &vec![0; 1_168_552], &wrong_count));
}

#[test]
#[ignore = "LITO_SHARED_RESOURCE_ROOT; read-only actual Text/list.spr and Sprite pak 3934-0.spr, not playback or GUI"]
fn actual_counter3934_bank_keeps_blank_dwell_slots_and_one_audio_cue() {
    let root = PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT"));
    let cancel = AtomicBool::new(false);
    let catalog = SharedActionCatalog::load(&root, &cancel).unwrap();
    let bank = SharedEffectBank::load_reviewed_selectors(&catalog, &cancel).unwrap();
    let animation = bank.animation(3934, 0).unwrap();
    assert_eq!(animation.frames_per_second, 24);
    assert_eq!(animation.directionality, Directionality::Undirected);
    assert_eq!(
        animation.frames.iter().map(|f| (f.image_id, f.subframe, f.duration_ticks)).collect::<Vec<_>>(),
        vec![(0, 12, 4), (0, 0, 4), (0, 1, 4), (0, 2, 4), (0, 3, 4), (0, 4, 4), (0, 5, 4),
             (0, 6, 4), (0, 7, 5), (0, 8, 5), (0, 9, 6), (0, 10, 6), (0, 11, 7)]
    );
    assert_eq!(animation.total_ticks(), 61);
    let cues: Vec<_> = animation.frames.iter().enumerate().filter(|(_, f)| !f.events.is_empty()).map(|(i, f)| (i, f.events.clone())).collect();
    assert_eq!(cues, vec![(2, vec![FrameEvent::Effect(1355)])]);
    // Slots 0/12 are the reviewed blanks; source0 (slot 12) and source1
    // (slot 0) dwell without pixels, every later source is visible.
    for source in 0..13 {
        let v = bank.frame(3934, 0, source, false).unwrap();
        assert_eq!(v.sprite_name, "3934-0.spr");
        assert_eq!(v.frame.rgba.is_empty(), source < 2, "source {source}");
        if source < 2 {
            assert!(is_reviewed_selector_blank("3934-0.spr", v.subframe, v.frame));
        }
    }
    let frames = bank.frames("3934-0.spr").unwrap();
    assert_eq!(frames.len(), 13);
    // Independent read of the original bytes: same pinned identity, and one
    // flipped byte is refused even with the already decoded frames.
    let name = "3934-0.spr";
    let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
    let mut selected = SharedArchive::open(&root, SharedArchiveKind::SpriteShard(shard)).unwrap();
    let bytes = match selected.entry(name) {
        Ok(_) => selected.read_entry(name).unwrap(),
        Err(ResourceError::MissingEntry(_)) => SharedArchive::open(&root, SharedArchiveKind::Sprite).unwrap().read_entry(name).unwrap(),
        Err(e) => panic!("{e:?}"),
    };
    let decoded = DecodedSprite { frames: frames.to_vec(), has_palette: false, block_table_metadata: 0 };
    assert!(reviewed_selector_empty_source_matches(name, &bytes, &decoded));
    let mut forged = bytes.clone();
    let middle = forged.len() / 2;
    forged[middle] ^= 1;
    assert!(!reviewed_selector_empty_source_matches(name, &forged, &decoded));
    let provenance = bank.sources().find(|s| s.name == name).unwrap();
    assert_eq!(provenance.sha256.as_slice(), Sha256::digest(&bytes).as_slice());
}
