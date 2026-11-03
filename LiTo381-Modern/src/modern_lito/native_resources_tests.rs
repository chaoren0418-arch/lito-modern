use super::*;
use serde_json::{json, Value};
use std::time::Duration;
fn index(records: &[(u32, &[u8], u32)]) -> Vec<u8> {
    let mut bytes = (records.len() as u32).to_le_bytes().to_vec();
    for (offset, name, size) in records {
        let mut r = [0; 28];
        r[..4].copy_from_slice(&offset.to_le_bytes());
        r[4..4 + name.len()].copy_from_slice(name);
        r[24..].copy_from_slice(&size.to_le_bytes());
        bytes.extend(r);
    }
    bytes
}
#[test]
fn old28_names_duplicates_and_all_unused_bounds_are_closed() {
    let cancel = AtomicBool::new(false);
    assert_eq!(
        parse_index(&index(&[(3, b"1026.TIL", 7)]), 10, 100, &cancel).unwrap()["1026.til"],
        (3, 7)
    );
    for bytes in [
        index(&[(0, b"a.til", 1), (1, b"A.TIL", 1)]),
        index(&[(0, b"../a.til", 1)]),
        index(&[(0, b"a\0b", 1)]),
        index(&[(0, b"", 1)]),
        index(&[(10, b"unused.til", 1)]),
        index(&[(u32::MAX, b"unused.til", 1)]),
        index(&[(0, b"a.til", 101)]),
    ] {
        assert!(parse_index(&bytes, 10, 100, &cancel).is_err());
    }
    let mut huge = index(&[(0, b"a.til", 1)]);
    huge[..4].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(parse_index(&huge, 10, 100, &cancel).is_err());
    let mut trailing = index(&[(0, b"a.til", 1)]);
    trailing.push(0);
    assert!(parse_index(&trailing, 10, 100, &cancel).is_err());
    assert!(parse_index(&index(&[(0, b"a.til", 1)]), 10, 100, &AtomicBool::new(true)).is_err());
}
#[test]
fn chunked_hash_cancel_during_read_stops_before_next_chunk_and_never_returns_proof() {
    struct Interrupt<'a> {
        cancel: &'a AtomicBool,
        calls: usize,
    }
    impl Read for Interrupt<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            self.calls += 1;
            out.fill(1);
            self.cancel.store(true, Ordering::Release);
            Ok(out.len())
        }
    }
    let cancel = AtomicBool::new(false);
    let mut read = Interrupt {
        cancel: &cancel,
        calls: 0,
    };
    assert!(hash_reader(&mut read, 128 * 1024, &cancel).is_err());
    assert_eq!(read.calls, 1);
    assert!(hash_reader(&mut [1u8; 4].as_slice(), 5, &AtomicBool::new(false)).is_err());
    assert!(hash_reader(&mut [1u8; 4].as_slice(), 3, &AtomicBool::new(false)).is_err());
}
fn dummy_plan() -> NativeAssetPlan {
    NativeAssetPlan {
        namespace: ResourceNamespace::NativeLitoV1,
        map_id: 564,
        canonical_sha256: [1; 32],
        sectors: vec![SectorPin {
            key: NativeSectorKey {
                map_id: 564,
                x: 0x7fff,
                y: 0x8000,
            },
            pin: BytePin {
                bytes: 1,
                sha256: [1; 32],
            },
        }],
        archives: [
            ArchiveRole::TileIndex,
            ArchiveRole::TilePak,
            ArchiveRole::SpriteIndex,
            ArchiveRole::SpritePak,
        ]
        .into_iter()
        .map(|role| ArchivePin {
            role,
            pin: BytePin {
                bytes: 1,
                sha256: [1; 32],
            },
        })
        .collect(),
    }
}
#[test]
fn disabled_cancelled_namespace_and_roles_reject_before_any_source_io() {
    let sources = NativeSources {
        map_root: "missing-native-root".into(),
        archive_root: "missing-archives".into(),
    };
    let mut plan = dummy_plan();
    let cancel = AtomicBool::new(false);
    let default = NativeResourceAdapter::default();
    assert!(default
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap_err_text()
        .contains("disabled"));
    let adapter = NativeResourceAdapter::pinned_inspection();
    assert!(adapter
        .load_static(&sources, &plan, Limits::default(), &AtomicBool::new(true))
        .unwrap_err_text()
        .contains("cancelled"));
    plan.namespace = ResourceNamespace::Shared;
    assert!(adapter
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap_err_text()
        .contains("identity"));
    plan.namespace = ResourceNamespace::NativeLitoV1;
    plan.archives.swap(0, 1);
    assert!(adapter
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap_err_text()
        .contains("roles"));
    plan.archives.swap(0, 1);
    plan.sectors[0].key.map_id = 514;
    assert!(adapter
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap_err_text()
        .contains("identity"));
    plan.sectors[0].key.map_id = 564;
    plan.sectors.push(plan.sectors[0]);
    assert!(adapter
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap_err_text()
        .contains("duplicate"));
}
trait Failure {
    fn unwrap_err_text(self) -> String;
}
impl<T> Failure for Result<T> {
    fn unwrap_err_text(self) -> String {
        match self {
            Err(e) => e.to_string(),
            Ok(_) => panic!("unexpected Native admission"),
        }
    }
}
fn digest(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    std::array::from_fn(|i| u8::from_str_radix(&value[i * 2..i * 2 + 2], 16).unwrap())
}
fn actual_plan(input: &Value) -> (NativeSources, NativeAssetPlan) {
    let pin = |p: &Value| BytePin {
        bytes: p["bytes"].as_u64().unwrap(),
        sha256: digest(p["sha256"].as_str().unwrap()),
    };
    let map_path = PathBuf::from(input["native_map"]["path"].as_str().unwrap());
    let archives = input["archive_pins"].as_array().unwrap();
    let sources = NativeSources {
        map_root: map_path.parent().unwrap().parent().unwrap().to_owned(),
        archive_root: PathBuf::from(archives[0]["path"].as_str().unwrap())
            .parent()
            .unwrap()
            .to_owned(),
    };
    let roles = [
        ArchiveRole::TileIndex,
        ArchiveRole::TilePak,
        ArchiveRole::SpriteIndex,
        ArchiveRole::SpritePak,
    ];
    let plan = NativeAssetPlan {
        namespace: ResourceNamespace::NativeLitoV1,
        map_id: 564,
        canonical_sha256: digest(input["native_map"]["canonical_sha256"].as_str().unwrap()),
        sectors: vec![SectorPin {
            key: NativeSectorKey {
                map_id: 564,
                x: 0x7fff,
                y: 0x8000,
            },
            pin: pin(&input["native_map"]),
        }],
        archives: archives
            .iter()
            .zip(roles)
            .map(|(p, role)| ArchivePin { role, pin: pin(p) })
            .collect(),
    };
    (sources, plan)
}
#[test]
#[ignore = "explicit immutable Native564 input and owned output directory required"]
fn real_native564_adapter_static_cancel_drift_and_roof_compositor() {
    let input_path = std::env::var_os("LITO_MODERN_NATIVE_PINNED_INPUT").expect("explicit input");
    let input: Value = serde_json::from_slice(&fs::read(input_path).unwrap()).unwrap();
    assert_eq!(input["candidate_id"], "native-dm564-geometry.20261004.1");
    let (sources, plan) = actual_plan(&input);
    let cancel = AtomicBool::new(false);
    let adapter = NativeResourceAdapter::pinned_inspection();
    let assets = adapter
        .load_static(&sources, &plan, Limits::default(), &cancel)
        .unwrap();
    assert_eq!(
        (
            assets.tile_entries(),
            assets.decoded_slots(),
            assets.draw_references()
        ),
        (32, 3434, 18994)
    );
    assert_eq!(assets.decoded_bytes(), 7911936);
    assert_eq!(
        (assets.opacity_records(), assets.effect_records().len()),
        (390, 5)
    );
    assert_eq!(
        assets
            .effect_records()
            .iter()
            .map(|r| r.effect.sprite_id)
            .collect::<BTreeSet<_>>(),
        [38, 123].into()
    );
    let camera = CameraRect {
        x: 1136,
        y: 484,
        width: 800,
        height: 600,
    };
    let canvas = assets.render_static(camera, &cancel).unwrap();
    let hash = |c: &Canvas| format!("{:x}", Sha256::digest(c.rgba()));
    assert_eq!(
        hash(&canvas),
        "8462eabd10e5d7930222d6b1438681703485c451fe964c9d5a03c442e69d8358"
    );
    for failure in ["raw", "canonical", "archive", "budget"] {
        let mut bad = plan.clone();
        let mut limits = Limits::default();
        match failure {
            "raw" => bad.sectors[0].pin.sha256[0] ^= 1,
            "canonical" => bad.canonical_sha256[0] ^= 1,
            "archive" => bad.archives[0].pin.sha256[0] ^= 1,
            _ => limits.decoded_bytes = 1024,
        };
        assert!(
            adapter
                .load_static(&sources, &bad, limits, &cancel)
                .is_err(),
            "{failure}"
        );
    }
    assert!(assets
        .render_static(camera, &AtomicBool::new(true))
        .is_err());
    let raw = &assets.encoded[&1026];
    for slot in [106, 255] {
        assert!(
            ValidatedTileInput::new_with_empty_tail(raw, TileLimits::default())
                .unwrap()
                .decode_selected(&BTreeSet::from([slot]), false)
                .is_err()
        );
    }
    assert!(super::super::tile::decode_classic_til_selected(
        raw,
        TileLimits::default(),
        &BTreeSet::from([105])
    )
    .is_err());
    let mut dynamic =
        json!({"status":"not_qualified","reason":"No qualified roof trigger in sample"});
    match assets.prepare_roofs(&cancel) {
        Err(error) => dynamic = json!({"status":"not_qualified","reason":error.to_string()}),
        Ok(mut roofs) => {
            let key = plan.sectors[0].key.geometry_key();
            let sector = assets.map.sector(key).unwrap();
            let (ox, oy) = key.world_origin();
            if let Some(record) = sector.opacity.iter().find(|r| {
                r.x < 128
                    && r.y < 64
                    && r.x % 2 == 0
                    && !sector.objects[usize::from(r.object_index)].tiles.is_empty()
                    && assets.map.standing(WorldCell {
                        x: ox + i32::from(r.x / 2),
                        y: oy + i32::from(r.y),
                    })
            }) {
                let inside = WorldCell {
                    x: ox + i32::from(record.x / 2),
                    y: oy + i32::from(record.y),
                };
                let group = Group(key.sector_x, key.sector_y, record.object_index);
                let outside = (0..64)
                    .flat_map(|y| {
                        (0..64).map(move |x| WorldCell {
                            x: ox + x,
                            y: oy + y,
                        })
                    })
                    .find(|&c| {
                        assets.map.standing(c)
                            && roofs.roofs.settled(c).unwrap().shade(Some(group), false)
                                == Shade::Native
                    })
                    .unwrap();
                let left = assets.draw.iter().map(|d| d.tile.x).min().unwrap();
                let top = assets.draw.iter().map(|d| d.tile.y).min().unwrap();
                let right = assets.draw.iter().map(|d| d.tile.x + 24).max().unwrap();
                let bottom = assets.draw.iter().map(|d| d.tile.y + 24).max().unwrap();
                let roof_camera = CameraRect {
                    x: left,
                    y: top,
                    width: (right - left) as u32,
                    height: (bottom - top) as u32,
                };
                let now = Instant::now();
                let mut hashes = Vec::new();
                let output = PathBuf::from(
                    std::env::var_os("LITO_MODERN_NATIVE_OUTPUT").expect("owned output directory"),
                );
                assert!(output.is_absolute() && output.is_dir());
                for (cell, ms) in [
                    (outside, 0),
                    (inside, 1),
                    (inside, 451),
                    (outside, 452),
                    (outside, 902),
                ] {
                    let canvas = roofs
                        .render(
                            &assets,
                            roof_camera,
                            1,
                            1,
                            cell,
                            now + Duration::from_millis(ms),
                            &cancel,
                        )
                        .unwrap();
                    hashes.push(hash(&canvas));
                    let file = fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(output.join(format!("native-roof-{ms}ms.png")))
                        .unwrap();
                    let mut png = png::Encoder::new(file, canvas.width(), canvas.height());
                    png.set_color(png::ColorType::Rgba);
                    png.set_depth(png::BitDepth::Eight);
                    png.write_header()
                        .unwrap()
                        .write_image_data(canvas.rgba())
                        .unwrap();
                }
                assert_ne!(hashes[0], hashes[2]);
                assert_eq!(hashes[0], hashes[4]);
                dynamic = json!({"status":"passed_cpu_group_clock_fade_restore","opacity_records":390,"sample_group":record.object_index,"sample_kind":record.kind,"rgba_hashes":hashes,"original_timer_alias_parity":false,"GUI":false});
            }
        }
    }
    eprintln!(
        "NATIVE_ADAPTER_RESULT {}",
        json!({"static_slots":assets.decoded_slots(),"camera_rgba":hash(&canvas),"opacity_dynamic":dynamic,"effect_records":assets.effect_records().len(),"effect_playback_qualified":false,"wire":false,"GUI":false})
    );
}
