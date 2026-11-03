//! Opt-in shared-resource qualification, not a PetRace gameplay implementation.
//!
//! Root executes these tests. They never open LT archives, write shared files,
//! instantiate a renderer/audio device, or infer a physical map from a room ID.
//! Shared 5143 is an explicitly named *candidate* from retained L1PolyRace;
//! its course/checkpoints and the idle consumer remain separate qualifications.

use super::{
    animation::{AnimationClock, PlaybackLimits},
    combat_animation::{classify_native_cues, NativeAudioMode},
    combat_audio::{BankLimits, SoundBank},
    fixed_walk::QualifiedFixedWalk,
    pet_animation::{PetAction, PetAppearance, PetAssets},
    resources::{
        read_shared_map_sector, ResourceError, SharedArchive, SharedArchiveKind, SharedMapSectorKey,
    },
    scene::static_draw_list,
    shared_action_catalog::SharedActionCatalog,
    sprite::{decode_classic_spr_with_limits, DecodedSprite, SpriteFrame, SpriteLimits},
    sprite_actions::{
        ActionDefinition, ActionError, ActionErrorKind, ActionKey, Directionality, FrameAsset,
        FrameEvent, SpriteActionCatalog,
    },
    world_map::{SectorBinding, WorldCell, WorldMap, DIRECTIONS},
};
use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};

const PETS: [u32; 4] = [5065, 938, 2145, 3182];
const ABSENT_LOGICAL_MAPS: [u32; 4] = [565, 566, 569, 570];
const MAX_ENTRY: usize = 8 * 1024 * 1024;
const MAX_ENCODED: usize = 64 * 1024 * 1024;
const MAX_RGBA: usize = 128 * 1024 * 1024;
const MAX_SPRITES: usize = 256;
const MAX_ACTIONS: usize = 512;
const MAX_SOURCE_FRAMES: usize = 256;
const MAX_CUES: usize = 1024;
const MAX_RECORDS: usize = 1_000_000;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn digest(text: &str) -> [u8; 32] {
    assert_eq!(text.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&text[2 * i..2 * i + 2], 16).unwrap())
}

fn root() -> PathBuf {
    std::env::var_os("LITO_SHARED_RESOURCE_ROOT")
        .map(PathBuf::from)
        .expect("explicit LITO_SHARED_RESOURCE_ROOT; no inferred shared/LT root")
}

fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "cancelled");
    Ok(())
}

fn error_report(error: &anyhow::Error) -> Value {
    let status = if let Some(resource) = error.downcast_ref::<ResourceError>() {
        match resource {
            ResourceError::MissingEntry(_) => "missing",
            ResourceError::Io(e) if e.kind() == io::ErrorKind::NotFound => "missing",
            ResourceError::EntryTooLarge { .. }
            | ResourceError::SectorTooLarge { .. }
            | ResourceError::SoundTooLarge { .. }
            | ResourceError::TooManyEntries(_)
            | ResourceError::AllocationFailed => "bound",
            _ => "unsupported",
        }
    } else if let Some(action) = error.downcast_ref::<ActionError>() {
        match action.kind {
            ActionErrorKind::MissingSprite(_) | ActionErrorKind::MissingAction(_, _) => "missing",
            ActionErrorKind::Limit(_) => "bound",
            _ => "unsupported",
        }
    } else if error
        .downcast_ref::<io::Error>()
        .is_some_and(|e| e.kind() == io::ErrorKind::NotFound)
    {
        "missing"
    } else {
        "unsupported"
    };
    json!({"status":status,"error":format!("{error:#}")})
}

/// Only an absent *member* permits the native base-archive route. An absent
/// shard file, duplicate, bad index, or decode failure is not such a decision.
fn use_base(member: std::result::Result<(), ResourceError>) -> Result<bool, ResourceError> {
    match member {
        Ok(()) => Ok(false),
        Err(ResourceError::MissingEntry(_)) => Ok(true),
        Err(error) => Err(error),
    }
}

struct SpriteReader {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    records: usize,
}

impl SpriteReader {
    fn open(root: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        ensure!(base.len() <= MAX_RECORDS, "index record budget");
        Ok(Self {
            root: base.root().to_owned(),
            records: base.len(),
            base,
            shards: BTreeMap::new(),
        })
    }

    fn archive(&mut self, name: &str, cancel: &AtomicBool) -> Result<&mut SharedArchive> {
        active(cancel)?;
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        if !self.shards.contains_key(&shard) {
            let archive = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            self.records = self
                .records
                .checked_add(archive.len())
                .context("index overflow")?;
            ensure!(self.records <= MAX_RECORDS, "index record budget");
            self.shards.insert(shard, archive);
        }
        active(cancel)?;
        if use_base(self.shards[&shard].entry(name).map(|_| ()))? {
            self.base.entry(name)?;
            Ok(&mut self.base)
        } else {
            Ok(self.shards.get_mut(&shard).unwrap())
        }
    }
}

/// Same independent-oracle record used by the existing SPR family references:
/// six BE i32 values followed by the SHA256 of the decoded raw RGBA bytes.
fn frame_record(index: usize, frame: &SpriteFrame) -> [u8; 56] {
    let mut record = [0; 56];
    let values = [
        i32::try_from(index).unwrap(),
        frame.x_offset,
        frame.y_offset,
        i32::try_from(frame.width).unwrap(),
        i32::try_from(frame.height).unwrap(),
        i32::from(frame.frame_type),
    ];
    for (slot, value) in record[..24].chunks_exact_mut(4).zip(values) {
        slot.copy_from_slice(&value.to_be_bytes());
    }
    record[24..].copy_from_slice(&Sha256::digest(&frame.rgba));
    record
}

/// Diagnostic access to the inspection AST is not permission to resolve,
/// preload or play its quarantined action. The production resolver is untouched.
fn inspect_unresolved_definition(definition: &ActionDefinition) -> Result<Value> {
    let detail = match definition {
        ActionDefinition::Animation(animation) => {
            ensure!(
                animation.frames.len() <= MAX_SOURCE_FRAMES,
                "inspection frame budget"
            );
            ensure!(
                animation
                    .frames
                    .iter()
                    .map(|f| f.events.len())
                    .sum::<usize>()
                    <= MAX_CUES,
                "inspection event budget"
            );
            json!({
                "definition_kind":"Animation",
                "label_bytes_hex":hex(&animation.label_bytes),
                "directionality":format!("{:?}",animation.directionality),
                "fps":animation.frames_per_second,
                "has_uninterpreted_events":animation.has_uninterpreted_events(),
                "frames":animation.frames.iter().enumerate().map(|(source,f)| json!({
                    "source":source,"image":f.image_id,"subframe":f.subframe,"ticks":f.duration_ticks,
                    "events":f.events.iter().map(|event| match event {
                        FrameEvent::Hit => json!({"kind":"Hit"}),
                        FrameEvent::Sound(value) => json!({"kind":"Sound","marker_byte":b'[',"value":value}),
                        FrameEvent::Overlay(value) => json!({"kind":"Overlay","value":value}),
                        FrameEvent::Effect(value) => json!({"kind":"Effect","marker_byte":b'<',"value":value}),
                        FrameEvent::UninterpretedNumeric{marker,value} => json!({
                            "kind":"UninterpretedNumeric","marker_byte":marker,"value":value,
                        }),
                        FrameEvent::Skip => json!({"kind":"Skip","marker_byte":b'>'}),
                    }).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
            })
        }
        ActionDefinition::Quarantined { raw_bytes, reason } => {
            ensure!(raw_bytes.len() <= 256 * 1024, "inspection raw-line budget");
            json!({"definition_kind":"Quarantined","raw_bytes_hex":hex(raw_bytes),
                "raw_utf8_lossy":String::from_utf8_lossy(raw_bytes),"reason":format!("{:?}",reason.kind),"line":reason.line})
        }
        ActionDefinition::Reference {
            sprite_id,
            label_bytes,
        } => json!({
            "definition_kind":"Reference","referenced_sprite":sprite_id,"label_bytes_hex":hex(label_bytes),
        }),
    };
    Ok(json!({"status":"inspection_only_not_qualified","detail":detail}))
}

fn action_inventory(catalog: &SpriteActionCatalog, id: u32) -> Result<Value> {
    let definition = catalog.sprites.get(&id).ok_or(ActionError {
        line: 0,
        kind: ActionErrorKind::MissingSprite(id),
    })?;
    ensure!(
        definition.actions.len() <= MAX_ACTIONS,
        "action diagnostic budget"
    );
    ensure!(
        definition.attributes.len() <= MAX_ACTIONS,
        "attribute diagnostic budget"
    );
    let actions: Vec<Value> = definition
        .actions
        .keys()
        .map(|key| match catalog.resolve_action(id, *key) {
            Ok(resolved) => json!({
                "key": key.id, "variant": key.variant, "status": "resolved",
                "definition": resolved.definition_sprite_id,
                "label_utf8_lossy": String::from_utf8_lossy(&resolved.animation.label_bytes),
                "directionality": format!("{:?}", resolved.animation.directionality),
                "fps": resolved.animation.frames_per_second,
                "source_frames": resolved.animation.frames.len(),
                "ticks": resolved.animation.total_ticks(),
            }),
            Err(error) => json!({
                "key": key.id, "variant": key.variant, "status": "resolve_error",
                "error": format!("{:?}", error.kind), "line": error.line,
            }),
        })
        .collect();
    let inspection_walk0 = if id == 2145 {
        definition
            .actions
            .get(&ActionKey::new(0))
            .map(inspect_unresolved_definition)
            .transpose()?
    } else {
        None
    };
    Ok(json!({
        "requested": id, "graphics_id": definition.graphics_id,
        "image_count": definition.image_count,
        "header_issue": definition.header_issue.as_ref().map(|e| format!("{:?}", e.kind)),
        "actions": actions,
        "inspection_walk0":inspection_walk0,
        "attributes": definition.attributes.iter().map(|a| json!({
            "key": a.key.id, "variant": a.key.variant,
            "label_utf8_lossy": String::from_utf8_lossy(&a.label_bytes),
            "body_utf8_lossy": String::from_utf8_lossy(&a.body_bytes),
        })).collect::<Vec<_>>(),
    }))
}

fn qualify_walk(context: &SharedActionCatalog, id: u32, cancel: &AtomicBool) -> Result<Value> {
    active(cancel)?;
    let catalog = context.catalog();
    let fixed = if id == 2145 {
        // This actual fixture must still fail every generic entry point. The
        // new certificate consumes the eight-count locomotion context, not a
        // sanitized clone or a generic per-ID exception.
        ensure!(
            matches!(
                catalog.resolve_action(id, ActionKey::new(0)),
                Err(ActionError {
                    kind: ActionErrorKind::UninterpretedEvents(..),
                    ..
                })
            ),
            "2145 generic resolver no longer rejects the retained markers"
        );
        let raw = match &catalog.sprites[&id].actions[&ActionKey::new(0)] {
            ActionDefinition::Animation(raw) => raw,
            _ => anyhow::bail!("2145 fixture is not the original inspection animation"),
        };
        ensure!(
            raw.frame_asset(id, 0, 0).is_err() && AnimationClock::new(raw).is_err(),
            "generic frame/clock quarantine must remain strict"
        );
        Some(QualifiedFixedWalk::from_catalog(catalog, id)?)
    } else {
        None
    };
    let action = if fixed.is_none() {
        Some(catalog.resolve_action(id, ActionKey::new(0))?)
    } else {
        None
    };
    let requested = catalog
        .sprites
        .get(&id)
        .context("missing requested definition")?;
    let definition_id = fixed.as_ref().map_or_else(
        || action.as_ref().unwrap().definition_sprite_id,
        |walk| walk.provenance().definition_sprite_id,
    );
    let defining = catalog
        .sprites
        .get(&definition_id)
        .context("missing resolved definition")?;
    ensure!(
        requested.graphics_id == defining.graphics_id,
        "unverified inherited graphics identity"
    );
    if let Some(action) = &action {
        ensure!(
            action.animation.directionality == Directionality::EightWay,
            "walk is not verified eight-way"
        );
        AnimationClock::with_limits(
            action.animation,
            PlaybackLimits {
                max_frames: MAX_SOURCE_FRAMES,
                max_events: MAX_CUES,
            },
        )?;
    }
    let (frames, fps, ticks) = if let Some(walk) = &fixed {
        (
            walk.source_frames(),
            walk.frames_per_second(),
            walk.total_source_ticks(),
        )
    } else {
        let animation = action.as_ref().unwrap().animation;
        (
            animation.frames.as_slice(),
            animation.frames_per_second,
            animation.total_ticks(),
        )
    };
    let frame_asset = |source, heading| -> Result<FrameAsset> {
        if let Some(walk) = &fixed {
            Ok(walk.frame_asset(source, heading)?)
        } else {
            Ok(action.as_ref().unwrap().animation.frame_asset(
                requested.graphics_id,
                source,
                heading,
            )?)
        }
    };

    let mut plan = BTreeMap::<String, BTreeSet<usize>>::new();
    let mut samples = Vec::new();
    let mut groups = Vec::new();
    let mut non_audio = Vec::new();
    let mut cue_report = Vec::new();
    for (source, frame) in frames.iter().enumerate() {
        active(cancel)?;
        let classified = classify_native_cues(&frame.events)?;
        if let Some(group) = classified.audio {
            cue_report.push(
                json!({"source":source,"mode":format!("{:?}",group.mode),"ids":group.candidates()}),
            );
            ensure!(
                group.mode != NativeAudioMode::LegacyCaretDistanceMode,
                "unqualified distance audio consumer"
            );
            groups.push(group);
        }
        if !classified.non_audio.is_empty() {
            non_audio.push(json!({"source":source,"events":format!("{:?}",classified.non_audio)}));
        }
        for heading in 0..8 {
            let asset = frame_asset(source, heading)?;
            let name = asset.file_name();
            plan.entry(name.clone())
                .or_default()
                .insert(asset.subframe as usize);
            ensure!(plan.len() <= MAX_SPRITES, "unique SPR budget");
            samples.push((heading, source, name, asset.subframe as usize));
        }
    }
    ensure!(samples.len() <= 8 * MAX_SOURCE_FRAMES, "sample budget");
    let mut reader = SpriteReader::open(context.root(), cancel)?;
    let mut encoded_total = 0usize;
    // Validate every index record/size before payload allocation.
    for name in plan.keys() {
        let size = reader.archive(name, cancel)?.entry(name)?.size as usize;
        ensure!((1..=MAX_ENTRY).contains(&size), "per-SPR byte budget");
        encoded_total = encoded_total
            .checked_add(size)
            .context("encoded accounting overflow")?;
        ensure!(encoded_total <= MAX_ENCODED, "aggregate encoded budget");
    }
    let mut decoded = BTreeMap::<String, DecodedSprite>::new();
    let mut rgba_total = 0usize;
    let mut sprite_report = Vec::new();
    for (name, subframes) in &plan {
        active(cancel)?;
        let archive = reader.archive(name, cancel)?;
        let archive_name = format!("{:?}", archive.kind());
        let bytes = archive.read_entry(name)?;
        active(cancel)?;
        let remaining = MAX_RGBA
            .checked_sub(rgba_total)
            .context("RGBA accounting overflow")?;
        ensure!(remaining >= 4, "aggregate RGBA budget exhausted");
        let spr = decode_classic_spr_with_limits(
            &bytes,
            SpriteLimits {
                max_input_bytes: MAX_ENTRY,
                max_frames: 255,
                max_blocks: 16_384,
                max_block_references: 65_536,
                max_frame_dimension: 4096,
                max_total_pixels: (remaining / 4).min(SpriteLimits::default().max_total_pixels),
            },
        )?;
        active(cancel)?;
        for subframe in subframes {
            ensure!(*subframe < spr.frames.len(), "referenced subframe absent");
        }
        let mut aggregate = Vec::with_capacity(spr.frames.len() * 56);
        for (index, frame) in spr.frames.iter().enumerate() {
            rgba_total = rgba_total
                .checked_add(frame.rgba.len())
                .context("RGBA accounting overflow")?;
            ensure!(rgba_total <= MAX_RGBA, "aggregate RGBA budget");
            aggregate.extend_from_slice(&frame_record(index, frame));
        }
        sprite_report.push(json!({
            "name":name,"archive":archive_name,"bytes":bytes.len(),
            "sha256":hex(&Sha256::digest(&bytes)),"palette":spr.has_palette,
            "metadata":spr.block_table_metadata,"subframes":spr.frames.len(),
            "all_frames_56b_sha256":hex(&Sha256::digest(&aggregate)),
            "referenced_subframes":subframes,
        }));
        decoded.insert(name.clone(), spr);
    }
    // Preserve source order and heading identities, independently of cache order.
    let mut reference_records = Vec::with_capacity(samples.len() * 68);
    for (heading, source, name, subframe) in &samples {
        reference_records.extend_from_slice(&u32::from(*heading).to_be_bytes());
        reference_records.extend_from_slice(&u32::try_from(*source)?.to_be_bytes());
        let asset = frame_asset(*source, *heading)?;
        reference_records.extend_from_slice(&asset.image_id.to_be_bytes());
        reference_records
            .extend_from_slice(&frame_record(*subframe, &decoded[name].frames[*subframe]));
    }
    let bank = SoundBank::load_shared(
        context.root(),
        &groups,
        BankLimits {
            max_bytes: 32 * 1024 * 1024,
            max_sounds: 128,
            max_groups: MAX_CUES,
        },
        cancel,
    )?;
    let sounds: Vec<Value> = bank
        .ids()
        .map(|id| {
            let sound = bank.get(id).unwrap();
            let format = sound.format();
            json!({"id":id,"bytes":sound.file_bytes(),"sha256":hex(&sound.sha256()),
            "pcm_bytes":sound.pcm_bytes().len(),"channels":format.channels,
            "sample_rate":format.sample_rate,"bits_per_sample":format.bits_per_sample})
        })
        .collect();
    Ok(json!({
        "status":"qualified_assets", "scope":"walk0 assets only; not idle, race or audio-device playback",
        "requested":id,"definition":definition_id,"graphics_id":requested.graphics_id,
        "headings":8,"fps":fps,"ticks":ticks,
        "source_frames":frames.iter().enumerate().map(|(index,f)| json!({
            "source":index,"image":f.image_id,"subframe":f.subframe,"ticks":f.duration_ticks,
            "events":format!("{:?}",f.events),
        })).collect::<Vec<_>>(),
        "sample_count":samples.len(),"sample_order":"source then heading; BEu32 heading/source/image + 56B frame",
        "sample_68b_sha256":hex(&Sha256::digest(&reference_records)),
        "encoded_bytes":encoded_total,"rgba_bytes":rgba_total,"sprites":sprite_report,
        "audio_groups":cue_report,"sounds":sounds,"non_audio_metadata":non_audio,
        "non_audio_consumer_qualified":fixed.is_some() || non_audio.is_empty(),
        "walk_qualification":if fixed.is_some() { "opaque eight-source action0; Skip retained, generic paths still reject" } else { "original strict generic asset path" },
        "unpaced_step":fixed.as_ref().map(|q|json!({
            "source_order":[1,2,3,4,5,6,7,0],"active_ms":q.active_duration().as_millis(),
            "total_with_terminal_ms":q.step_duration().as_millis(),
            "authority_timing":"unchanged; no runtime integration", "body_extra_camera_offset":false,
        })),
        "idle":{"status":"unsupported_pending_native_consumer","assumed_action":Value::Null},
    }))
}

fn bindings_5143() -> Vec<SectorBinding> {
    [
        (
            0x7fff,
            0x8000,
            127_474,
            "72f52a1f3dbc6db65650c4c470f254a6a1d94b4283dc928b78edb4921aacdb15",
        ),
        (
            0x7fff,
            0x8001,
            141_309,
            "2712d11716d060cf220d3f33ab2c26a190bb4c333ab78e185eb24baf996effea",
        ),
        (
            0x8000,
            0x8000,
            111_184,
            "c8f0ef53c400f2558e7239c21d092be10a42613fd2b74ef0fe9270d05a3e5cb8",
        ),
        (
            0x8000,
            0x8001,
            99_133,
            "054ca18e1cb46f4084135fc5409bb692bb8e99b998c50579148f6b2059bcae0d",
        ),
    ]
    .into_iter()
    .map(|(sector_x, sector_y, byte_length, sha)| SectorBinding {
        key: SharedMapSectorKey {
            map_id: 5143,
            sector_x,
            sector_y,
        },
        byte_length,
        sha256: digest(sha),
    })
    .collect()
}

fn cell_json(cell: WorldCell) -> Value {
    json!([cell.x, cell.y])
}

fn reachable(
    map: &WorldMap,
    anchor: WorldCell,
    cancel: &AtomicBool,
) -> Result<BTreeSet<WorldCell>> {
    let mut seen = BTreeSet::new();
    if !map.standing(anchor) {
        return Ok(seen);
    }
    let mut queue = VecDeque::from([anchor]);
    seen.insert(anchor);
    while let Some(cell) = queue.pop_front() {
        active(cancel)?;
        for (dx, dy) in DIRECTIONS {
            let next = WorldCell {
                x: cell.x + dx,
                y: cell.y + dy,
            };
            if map.can_step(cell, next) && seen.insert(next) {
                queue.push_back(next);
            }
        }
        ensure!(seen.len() <= 4 * 4096, "four-sector graph bound");
    }
    Ok(seen)
}

fn square_cycle(map: &WorldMap, cells: &BTreeSet<WorldCell>) -> Option<[WorldCell; 5]> {
    for &a in cells {
        let cycle = [
            a,
            WorldCell { x: a.x + 1, y: a.y },
            WorldCell {
                x: a.x + 1,
                y: a.y + 1,
            },
            WorldCell { x: a.x, y: a.y + 1 },
            a,
        ];
        if cycle.windows(2).all(|pair| map.can_step(pair[0], pair[1])) {
            return Some(cycle);
        }
    }
    None
}

fn inspect_candidate(root: &Path, cancel: &AtomicBool) -> Result<Value> {
    let bindings = bindings_5143();
    // Require the observed complete set: never accept a convenient subset of
    // sectors or a .seg substitute. Actual reads below retain shared-root guards.
    let expected: BTreeSet<_> = bindings.iter().map(|b| b.key.file_name()).collect();
    let mut actual = BTreeSet::new();
    for entry in std::fs::read_dir(root.join("map").join("5143"))? {
        let entry = entry?;
        ensure!(actual.len() < 16, "candidate directory entry bound");
        let metadata = entry.file_type()?;
        ensure!(
            metadata.is_file() && !metadata.is_symlink(),
            "nonregular sector member"
        );
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("non-UTF8 sector name"))?;
        ensure!(actual.insert(name), "duplicate candidate filename");
    }
    ensure!(actual == expected, "candidate complete sector set changed");
    let map = WorldMap::load_shared(root, &bindings, cancel)?;
    ensure!(
        map.map_id() == 5143 && map.bindings() == bindings,
        "exact candidate manifest changed"
    );
    // Independently executed Java strict reader, centrally cross-checked with
    // the Rust first probe. These are shared5143, never LT/logical565 pins.
    ensure!(
        hex(&map.canonical_digest())
            == "f6c171c57eeae9e9223af51b12e89a1f0640086150abb41252e5e742c6feffde",
        "independent Java canonical reference mismatch"
    );
    let mut masks = Vec::with_capacity(4 * 4096);
    let mut standing = BTreeSet::new();
    let mut sectors = Vec::new();
    let mut cross_sector_edges = 0usize;
    let mut egress_bits = 0usize;
    for binding in &bindings {
        let sector = map
            .sector(binding.key)
            .context("missing qualified sector")?;
        let (ox, oy) = binding.key.world_origin();
        for y in oy..oy + 64 {
            for x in ox..ox + 64 {
                let cell = WorldCell { x, y };
                let mask = map.direction_mask(cell);
                masks.push(mask);
                if map.standing(cell) {
                    standing.insert(cell);
                }
                for (direction, (dx, dy)) in DIRECTIONS.into_iter().enumerate() {
                    let next = WorldCell {
                        x: x + dx,
                        y: y + dy,
                    };
                    let in_same_sector =
                        (ox..ox + 64).contains(&next.x) && (oy..oy + 64).contains(&next.y);
                    if !in_same_sector && map.can_step(cell, next) {
                        cross_sector_edges += 1;
                    }
                    let contained = bindings.iter().any(|b| {
                        let (nx, ny) = b.key.world_origin();
                        (nx..nx + 64).contains(&next.x) && (ny..ny + 64).contains(&next.y)
                    });
                    if !contained && mask & (1 << direction) != 0 {
                        egress_bits += 1;
                    }
                }
            }
        }
        let limitations = map.art_limitations(binding.key).unwrap();
        let draws = static_draw_list(sector)?;
        let tile_refs: BTreeSet<_> = draws
            .iter()
            .map(|d| (d.archive_entry_id, d.sub_index))
            .collect();
        sectors.push(json!({
            "name":binding.key.file_name(),"bytes":binding.byte_length,"sha256":hex(&binding.sha256),
            "origin":[ox,oy],"layer_ranges":sector.layer_ranges.iter().map(|r| r.as_ref().map(|r| [r.start,r.end])).collect::<Vec<_>>(),
            "counts":[sector.floor.len(),sector.overlays.len(),sector.attributes.len(),sector.object_tile_count(),sector.opacity.len(),sector.tile_dependencies.len(),sector.portals.len(),sector.effects.len()],
            "object_groups":sector.objects.len(),"unknown_tail":sector.has_unknown_tail(),
            "extended_effects":sector.effects_extended,"tile_ids":sector.tile_dependencies,
            "static_draws":draws.len(),"unique_static_tile_refs":tile_refs.len(),
            "limitations":{"opacity":limitations.opacity_records,"effects":limitations.sprite_effects,"portals":limitations.portals,"negative_object_tiles":limitations.negative_object_tiles,
                "all_tile_pixels":"not qualified by geometry probe","static_and_effect_render":"not qualified"},
        }));
    }
    let anchor = WorldCell { x: 32768, y: 32849 };
    ensure!(
        standing.len() == 1471 && map.direction_mask(anchor) == 255,
        "independent Java standing/anchor reference mismatch"
    );
    ensure!(
        map.standing(anchor),
        "retained candidate source anchor is blocked"
    );
    ensure!(
        egress_bits == 0,
        "candidate directions leave complete manifest"
    );
    let reached = reachable(&map, anchor, cancel)?;
    ensure!(
        reached.len() >= 10,
        "source anchor lacks ten reachable standing cells"
    );
    let cycle = square_cycle(&map, &reached).context("no verified four-edge cycle candidate")?;
    // These are geometry candidates only, not the retained ten starting grid
    // positions or the ordered 29 checkpoints. No location is adopted blindly.
    let candidates: Vec<_> = std::iter::once(anchor)
        .chain(
            reached
                .iter()
                .copied()
                .filter(|cell| *cell != anchor)
                .take(9),
        )
        .collect();
    map.validate_spawns(&candidates, cancel)?;
    let mut reached_bytes = Vec::with_capacity(reached.len() * 8);
    for cell in &reached {
        reached_bytes.extend_from_slice(&cell.x.to_be_bytes());
        reached_bytes.extend_from_slice(&cell.y.to_be_bytes());
    }
    ensure!(
        masks.len() == 16384
            && hex(&Sha256::digest(&masks))
                == "a95dddc3ea19bd1c979ce219839d990824b0ab6945256f9d01ade70e89cc7a58",
        "independent Java direction-mask reference mismatch"
    );
    Ok(json!({
        "status":"qualified_geometry_candidate","physical_map":5143,"logical_binding":"not committed",
        "sectors":sectors,"canonical_bytes":map.canonical_bytes().len(),"canonical_sha256":hex(&map.canonical_digest()),
        "mask_order":"sector_x then sector_y; each sector y then x; one u8 mask per cell",
        "mask_bytes":masks.len(),"direction_sha256":hex(&Sha256::digest(&masks)),
        "standing":standing.len(),"cross_sector_directed_edges":cross_sector_edges,"out_of_manifest_direction_bits":egress_bits,
        "source_anchor":cell_json(anchor),"anchor_mask":map.direction_mask(anchor),"anchor_standing":map.standing(anchor),
        "directed_reachable":reached.len(),"reachable_order":"x then y; BEi32 x,y",
        "reachable_sha256":hex(&Sha256::digest(&reached_bytes)),
        "required_connected_capacity":10,"independent_geometry_oracle":"Java canonical/masks/standing/anchor fixed expectations",
        "connected_spawn_candidates":candidates.into_iter().map(cell_json).collect::<Vec<_>>(),
        "small_square_cycle_candidate":cycle.into_iter().map(cell_json).collect::<Vec<_>>(),
        "race_course":"unqualified: ordered checkpoints, lap/finish rules and race-sized cycle require separate evidence",
    }))
}

#[test]
fn only_missing_selected_member_can_request_base_sprite() {
    assert!(!use_base(Ok(())).unwrap());
    assert!(use_base(Err(ResourceError::MissingEntry("5065-0.spr".into()))).unwrap());
    for error in [
        ResourceError::Io(io::Error::from(io::ErrorKind::NotFound)),
        ResourceError::MalformedIndex("fixture"),
        ResourceError::AmbiguousEntry {
            name: "5065-0.spr".into(),
            count: 2,
        },
        ResourceError::DuplicateEntry("5065-0.spr".into()),
    ] {
        assert!(use_base(Err(error)).is_err());
    }
}

#[test]
fn frame_oracle_record_preserves_signed_bounds_and_type() {
    let frame = SpriteFrame {
        fragments: None,
        x_offset: -17,
        y_offset: 9,
        width: 1,
        height: 1,
        frame_type: 41,
        unknown1: 0,
        unknown2: 0,
        rgba: vec![1, 2, 3, 255],
    };
    let record = frame_record(7, &frame);
    assert_eq!(record.len(), 56);
    assert_eq!(
        hex(&record[..24]),
        "00000007ffffffef00000009000000010000000100000029"
    );
    assert_eq!(&record[24..], &Sha256::digest(&frame.rgba)[..]);
}

#[test]
fn missing_resource_is_not_reported_as_unsupported_or_qualified() {
    let missing = anyhow::Error::new(ResourceError::MissingEntry("5065-0.spr".into()));
    let invalid = anyhow::Error::new(ResourceError::MalformedIndex("fixture"));
    let absent = anyhow::Error::new(ActionError {
        line: 0,
        kind: ActionErrorKind::MissingSprite(5065),
    });
    assert_eq!(error_report(&missing)["status"], "missing");
    assert_eq!(error_report(&invalid)["status"], "unsupported");
    assert_eq!(error_report(&absent)["status"], "missing");
}

#[test]
fn cancellation_does_not_enter_resource_acquisition() {
    assert!(active(&AtomicBool::new(true)).is_err());
    assert!(active(&AtomicBool::new(false)).is_ok());
}

#[test]
#[ignore = "requires explicit shared root; four-pet action inventory plus real shared SPR/WAV qualification; not race/idle gameplay"]
fn actual_shared_four_pet_walk_assets_and_all_action_resolutions() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for id in PETS {
        let inventory = match action_inventory(context.catalog(), id) {
            Ok(value) => value,
            Err(error) => {
                failures.push(id);
                error_report(&error)
            }
        };
        let walk = match qualify_walk(&context, id, &cancel) {
            Ok(value) => value,
            Err(error) => {
                failures.push(id);
                error_report(&error)
            }
        };
        rows.push(json!({"requested":id,"inventory":inventory,"walk":walk}));
    }
    println!(
        "{}",
        json!({"schema":1,"probe":"shared_pet_resources","list_bytes":context.byte_length(),
        "list_sha256":hex(&context.sha256()),"index_records":context.index_records(),"pets":rows,
        "idle_consumer":"unqualified; no action3 substitution","race_runtime":"not implemented by this probe","lt_fallback":false})
    );
    assert!(failures.is_empty(),"required shared walk resources unqualified: {failures:?}; diagnostic rows precede this failure");
}

#[test]
#[ignore = "requires explicit shared root; inspection of four pet shadow definitions only, not shadow rendering qualification"]
fn actual_shared_pet_shadow_definitions_inspection_only() {
    let root = root();
    let context = SharedActionCatalog::load(&root, &AtomicBool::new(false)).unwrap();
    let mut rows = Vec::new();
    let mut missing = Vec::new();
    for (body, shadow) in [(5065, 5066), (938, 939), (2145, 2146), (3182, 3183)] {
        let inspection = match action_inventory(context.catalog(), shadow) {
            Ok(value) => value,
            Err(error) => {
                missing.push(shadow);
                error_report(&error)
            }
        };
        rows.push(json!({"body":body,"shadow":shadow,"inspection":inspection}));
    }
    println!(
        "{}",
        json!({"schema":1,"probe":"pet_shadow_definitions_inspection_only",
        "list_sha256":hex(&context.sha256()),"rows":rows,"asset_or_consumer_qualification":false})
    );
    assert!(
        missing.is_empty(),
        "required shadow definitions missing/unsupported: {missing:?}"
    );
}

#[test]
#[ignore = "requires explicit shared root; all four walk/breath body+direct-shadow SPR and cues, no race/runtime/GUI claim"]
fn actual_shared_pet_walk_breath_and_direct_shadow_assets() {
    let root = root();
    let cancel = AtomicBool::new(false);
    let context = SharedActionCatalog::load(&root, &cancel).unwrap();
    let assets = PetAssets::load_with_catalog(&context, &PetAppearance::ALL, &cancel).unwrap();
    assert!(context
        .catalog()
        .resolve_action(2145, ActionKey::new(0))
        .is_err());
    let mut samples = Vec::new();
    let mut actions = Vec::new();
    let mut sources = Vec::new();
    for appearance in PetAppearance::ALL {
        for kind in [PetAction::Walk, PetAction::Breath] {
            let action = assets.action(appearance, kind).unwrap();
            actions.push(json!({"body":appearance.graphics_id(),"shadow":appearance.shadow_graphics_id(),
                "action":kind.key().id,"fps":action.frames_per_second(),"unpaced_cycle_ms":action.unpaced_cycle_duration().as_millis(),
                "frames":action.source_frames().iter().enumerate().map(|(source,f)|json!({
                    "source":source,"image":f.image_id,"subframe":f.subframe,"ticks":f.duration_ticks,"events":format!("{:?}",f.events),
                })).collect::<Vec<_>>(),
            }));
            for heading in 0..8 {
                for source in 0..action.source_frames().len() {
                    let pair = assets.frame(appearance, kind, heading, source).unwrap();
                    assert_eq!(
                        (pair.body.graphics_id, pair.shadow.graphics_id),
                        (appearance.graphics_id(), appearance.shadow_graphics_id())
                    );
                    assert_eq!(
                        (pair.body.image_id, pair.body.subframe),
                        (pair.shadow.image_id, pair.shadow.subframe)
                    );
                    for (layer, sample) in [(0, pair.body), (1, pair.shadow)] {
                        for value in [
                            appearance.graphics_id(),
                            kind.key().id,
                            u32::from(heading),
                            source as u32,
                            layer,
                            sample.image_id,
                        ] {
                            samples.extend_from_slice(&value.to_be_bytes());
                        }
                        samples.extend_from_slice(&frame_record(sample.subframe, sample.frame));
                    }
                }
            }
        }
    }
    let mut bank_reference = std::collections::BTreeMap::new();
    let mut flattened_bytes = 0usize;
    let mut fragment_bytes = 0usize;
    for provenance in assets.sprite_sources() {
        let frames = assets.sprite_frames(&provenance.name).unwrap();
        flattened_bytes += frames.iter().map(|f| f.rgba.len()).sum::<usize>();
        let atlas = &frames[0].fragments.as_ref().unwrap().atlas;
        assert!(frames
            .iter()
            .all(|f| std::sync::Arc::ptr_eq(atlas, &f.fragments.as_ref().unwrap().atlas)));
        fragment_bytes += atlas.iter().map(|b| b.retained_bytes()).sum::<usize>();
        let mut aggregate = Vec::new();
        for (subframe, frame) in frames.iter().enumerate() {
            aggregate.extend_from_slice(&frame_record(subframe, frame));
        }
        let (palette, metadata) = assets.sprite_metadata(&provenance.name).unwrap();
        assert_eq!(provenance.archive, SharedArchiveKind::Sprite);
        bank_reference.insert(
            provenance.name.clone(),
            <[u8; 32]>::from(Sha256::digest(&aggregate)),
        );
        sources.push(json!({"name":provenance.name,"archive":format!("{:?}",provenance.archive),
            "encoded":provenance.bytes,"sha256":hex(&provenance.sha256),"palette":palette,"metadata":metadata,
            "frames":frames.len(),"all_frames_56b_sha256":hex(&Sha256::digest(&aggregate))}));
    }
    let bank = assets.sound_bank();
    let sounds:Vec<_>=bank.ids().map(|id|{let sound=bank.get(id).unwrap();json!({"id":id,
        "bytes":sound.file_bytes(),"sha256":hex(&sound.sha256()),"pcm_bytes":sound.pcm_bytes().len(),
        "format":format!("{:?}",sound.format())})}).collect();
    assert_eq!(
        samples.len(),
        1088 * 80,
        "four source-qualified walk/breath rosters, all headings, both layers"
    );
    // Independent Lin.Helper.Core SprReader output, collected by the existing
    // memory-only reference tool's explicit -PetRace route. Never generated
    // from this Rust decoder: all 128 SPR/all subframes, not just source0.
    let mut bank_bytes = Vec::new();
    for (name, digest) in &bank_reference {
        bank_bytes.extend_from_slice(name.as_bytes());
        bank_bytes.push(0);
        bank_bytes.extend_from_slice(digest);
    }
    assert_eq!((bank_reference.len(), bank_bytes.len()), (128, 5524));
    assert_eq!(
        hex(&Sha256::digest(&bank_bytes)),
        "07f1fc99e7279696c2fe7d26beb1c1fb0d463fdafbf15649fc796248a2b81722"
    );
    assert_eq!(
        hex(&Sha256::digest(&samples)),
        "5e04309b41ba5fea4a7cdbb807f31fd9e51ecc0a888560aa2af59effac629a88"
    );
    assert_eq!(
        (assets.encoded_bytes(), flattened_bytes),
        (679290, 11351808)
    );
    assert!(fragment_bytes > 0);
    assert_eq!(assets.rgba_bytes(), flattened_bytes + fragment_bytes);
    assert_eq!(
        hex(&assets.catalog_sha256()),
        "523c61ad5cc362a56a36f58b262c84efa8b4435f017e7074d35b5834e8b9f127"
    );
    println!(
        "{}",
        json!({"schema":1,"probe":"shared_pet_walk_breath_direct_shadow",
        "list_sha256":hex(&assets.catalog_sha256()),"actions":actions,"sprites":sources,"sounds":sounds,
        "encoded_bytes":assets.encoded_bytes(),"rgba_bytes":assets.rgba_bytes(),"samples":samples.len()/80,
        "sample_schema":"BEu32 appearance/action/heading/source/layer/image + 56B frame; appearance input order, action0/3, heading,source,body/shadow",
        "sample_80b_sha256":hex(&Sha256::digest(&samples)),
        "ordinary_shadow_policy":"explicit actor OR5 RGB555 reference, no dynamic flags or attr8",
        "local_stop":"unqualified","race_runtime":"not exercised","independent_pixel_oracle":"C#128-SPR/all-subframes +1088 sample fixed golden",
        "bank_ordinal_name_nul_frame_sha256":hex(&Sha256::digest(&bank_bytes))})
    );
}

#[test]
#[ignore = "requires explicit shared root with observed absent 565/566/569/570; verifies refusal, not playable race qualification"]
fn actual_missing_logical_race_maps_are_rejected_without_lt_fallback() {
    let root = root();
    let mut rows = Vec::new();
    for map_id in ABSENT_LOGICAL_MAPS {
        let metadata = std::fs::symlink_metadata(root.join("map").join(map_id.to_string()));
        assert!(
            matches!(metadata,Err(ref e) if e.kind()==io::ErrorKind::NotFound),
            "observed missing-map fixture changed: {map_id}"
        );
        let key = SharedMapSectorKey {
            map_id,
            sector_x: 0x7fff,
            sector_y: 0x8000,
        };
        let result = read_shared_map_sector(&root, key);
        assert!(
            matches!(result,Err(ResourceError::Io(ref e)) if e.kind()==io::ErrorKind::NotFound),
            "shared-only reader must refuse absent map {map_id}"
        );
        rows.push(json!({"logical_map":map_id,"status":"missing_rejected","lt_fallback":false}));
    }
    println!(
        "{}",
        json!({"schema":1,"probe":"missing_shared_race_maps_negative","maps":rows})
    );
}

#[test]
#[ignore = "requires explicit shared root; complete actual5143 pins/geometry candidate, not logical565/course/render qualification"]
fn actual_shared5143_candidate_full_geometry_probe() {
    let root = root();
    let result = inspect_candidate(&root, &AtomicBool::new(false));
    match result {
        Ok(value) => println!(
            "{}",
            json!({"schema":1,"probe":"shared5143_candidate","result":value})
        ),
        Err(error) => {
            println!(
                "{}",
                json!({"schema":1,"probe":"shared5143_candidate","result":error_report(&error)})
            );
            panic!("shared5143 candidate did not qualify; diagnostic precedes failure");
        }
    }
}
