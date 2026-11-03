//! All six shared532 sectors and every static Tile reference; independent
//! expectations from verify-map532-reference.ps1 / existing C# decoders.
//! Not a proof of unimplemented L5 opacity, L8 effects or native GUI parity.
use super::{
    resources::{ResourceLimits, SharedArchive, SharedArchiveKind, SharedMapSectorKey},
    scene::{render_tiles, static_draw_list, CameraRect},
    tile::{decode_classic_til_with_limits, DecodedTile, TileLimits},
    world_map::{SectorBinding, WorldCell, WorldMap, WorldMapError, DIRECTIONS},
};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeSet, VecDeque},
    path::Path,
    sync::atomic::AtomicBool,
};
fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
fn digest(s: &str) -> [u8; 32] {
    std::array::from_fn(|i| u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).unwrap())
}
const SECTORS: [(u16, u16, u32, &str); 6] = [
    (
        32767,
        32768,
        125861,
        "d35e6f1da1c1012e68f05cdc0b6b830040fea41fdb13f24f4596a62dfeab0db3",
    ),
    (
        32767,
        32769,
        103877,
        "147e88b94f0f0f4a3277a1e9166436e534ee2f455cd8cdbe2da39778f0cca931",
    ),
    (
        32768,
        32768,
        150512,
        "56bf52953d3483f3ba9399fac9b935b2059486713ee42412626bcd5a7ecb5899",
    ),
    (
        32768,
        32769,
        126210,
        "77c86ad27d63e457c32aba1502ce95d397b00590e737b20bad243a035b617d9f",
    ),
    (
        32769,
        32768,
        149410,
        "409856e787efd3dea1cd9f7a9c450d2778d3c877cf83ec51423e91b5ba39776a",
    ),
    (
        32769,
        32769,
        136185,
        "121e8853304cc849abbdd8e8e696d32ed1b0bd82ed4d538155f8891c9445591a",
    ),
];
pub(crate) fn bindings() -> Vec<SectorBinding> {
    SECTORS
        .iter()
        .map(|(x, y, n, h)| SectorBinding {
            key: SharedMapSectorKey {
                map_id: 532,
                sector_x: *x,
                sector_y: *y,
            },
            byte_length: *n,
            sha256: digest(h),
        })
        .collect()
}
pub(crate) fn load(root: &Path) -> WorldMap {
    WorldMap::load_shared(root, &bindings(), &AtomicBool::new(false)).unwrap()
}
fn tile_entry(id: u16, sub: u16, t: &DecodedTile) -> [u8; 45] {
    let mut b = [0; 45];
    b[..2].copy_from_slice(&id.to_be_bytes());
    b[2..4].copy_from_slice(&sub.to_be_bytes());
    b[4] = t.tile_type as u8;
    b[5..9].copy_from_slice(&t.source_offset.to_be_bytes());
    b[9..13].copy_from_slice(&[t.x_offset, t.y_offset, t.content_width, t.content_height]);
    b[13..].copy_from_slice(&Sha256::digest(&t.rgba));
    b
}
#[test]
#[ignore = "explicit shared532 complete S32 and Tile qualification against independent C# oracle"]
fn actual_shared532_six_sectors_all_static_references_and_connected_geometry_match_independent_oracle(
) {
    let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("explicit shared root");
    let root = Path::new(&root);
    let cancel = AtomicBool::new(false);
    let map = load(root);
    let binds = bindings();
    assert_eq!(map.bindings(), binds);
    assert_eq!(map.canonical_bytes().len(), 24911);
    assert_eq!(
        hex(&map.canonical_digest()),
        "b483dc9c8fefefcdbd515680317fc5e44090046f6490e8d165f5f989bf6d5fdc"
    );
    let offsets: [[_; 8]; 6] = [
        [0, 32768, 32770, 49154, 118555, 125489, 125749, 125751],
        [0, 32768, 32770, 49154, 98915, 103639, 103819, 103821],
        [0, 32768, 32770, 49154, 138025, 150004, 150284, 150354],
        [0, 32768, 32770, 49154, 117958, 125942, 126146, 126148],
        [0, 32768, 32770, 49154, 137671, 149000, 149256, 149258],
        [0, 32768, 32770, 49154, 126297, 135761, 136037, 136039],
    ];
    // L4 tile records, L5, L7, L8, object groups, walkable cells.
    let counts: [[_; 6]; 6] = [
        [9735, 1386, 0, 18, 313, 1075],
        [6987, 944, 0, 9, 212, 629],
        [12497, 2395, 4, 26, 347, 920],
        [9672, 1596, 0, 10, 274, 1020],
        [12423, 2265, 0, 25, 388, 996],
        [10777, 1892, 0, 24, 425, 1119],
    ];
    let directions = [
        "1950f11d8da2a154030077b4453a512139b3dcda06486903a3028753c8724aed",
        "5324b0d83e6174dbc87cd6e08d9d8451ed2f0a94801f0008412f19a9351f970d",
        "dd69dd8fd1a760162eb989518e61d1c103182fb6897bca39034e17912e6b5a14",
        "09d66306d987789f6667aebf17a689c73a81ec5efb06bc79c7b6a1639a6ec199",
        "3001742ff70f1d816ec8bc0d43960697ff386f491d66ae4225c6219300f56795",
        "9a6ade443c502b7d2db9166421229c0b57c32ac83b68c56f6b720483d1f744c6",
    ];
    let mut draws = Vec::new();
    let mut cross_edges = 0;
    let mut standing = 0;
    let mut draw_hash = Sha256::new();
    for (i, b) in binds.iter().enumerate() {
        let s = map.sector(b.key).unwrap();
        assert_eq!(
            s.layer_ranges
                .iter()
                .map(|r| r.as_ref().unwrap().start)
                .collect::<Vec<_>>(),
            offsets[i]
        );
        assert_eq!(
            s.layer_ranges[7].as_ref().unwrap().end,
            SECTORS[i].2 as usize
        );
        assert_eq!(s.floor.len(), 8192);
        assert!(s.overlays.is_empty());
        assert_eq!(s.attributes.len(), 4096);
        assert_eq!(
            [
                s.object_tile_count(),
                s.opacity.len(),
                s.portals.len(),
                s.effects.len(),
                s.objects.len()
            ],
            [
                counts[i][0],
                counts[i][1],
                counts[i][2],
                counts[i][3],
                counts[i][4]
            ]
        );
        assert!(!s.effects_extended);
        assert!(!s.has_unknown_tail());
        let ox = 32704 + 64 * (i as i32 / 2);
        let oy = 32768 + 64 * (i as i32 % 2);
        let mut masks = Vec::new();
        for y in oy..oy + 64 {
            for x in ox..ox + 64 {
                let p = WorldCell { x, y };
                masks.push(map.direction_mask(p));
                for (dx, dy) in DIRECTIONS {
                    let q = WorldCell {
                        x: x + dx,
                        y: y + dy,
                    };
                    if map.can_step(p, q)
                        && (q.x < ox || q.x >= ox + 64 || q.y < oy || q.y >= oy + 64)
                    {
                        cross_edges += 1;
                    }
                }
            }
        }
        assert_eq!(hex(&Sha256::digest(&masks)), directions[i]);
        let n = masks.iter().filter(|m| **m & 0x55 != 0).count();
        assert_eq!(n, counts[i][5]);
        standing += n;
        let art = map.art_limitations(b.key).unwrap();
        assert_eq!(art.opacity_records, counts[i][1]);
        assert_eq!(art.sprite_effects, counts[i][3]);
        assert_eq!(art.portals, counts[i][2]);
        assert!(art.has_known_presentation_gaps());
        for mut d in static_draw_list(s).unwrap() {
            let dx = ox - 32704;
            let dy = oy - 32768;
            d.x += 24 * (dx + dy);
            d.y += 12 * (-dx + dy);
            draw_hash.update(d.archive_entry_id.to_be_bytes());
            draw_hash.update(u16::from(d.sub_index).to_be_bytes());
            draw_hash.update(d.x.to_be_bytes());
            draw_hash.update(d.y.to_be_bytes());
            draw_hash.update(d.layer.to_be_bytes());
            draws.push(d);
        }
    }
    assert_eq!(standing, 5759);
    assert_eq!(cross_edges, 976);
    assert_eq!(draws.len(), 111243);
    assert_eq!(
        hex(&draw_hash.finalize()),
        "24dcebe39845fecbef24b2f7bffc938f91871e035739908ac166ff59d51b760b"
    );
    assert_eq!(
        [
            draws.iter().map(|d| d.x).min().unwrap(),
            draws.iter().map(|d| d.y).min().unwrap(),
            draws.iter().map(|d| d.x + 24).max().unwrap(),
            draws.iter().map(|d| d.y + 24).max().unwrap()
        ],
        [0, -1536, 7680, 2304]
    );
    let anchor = WorldCell { x: 32796, y: 32839 };
    assert_eq!(map.direction_mask(anchor), 56);
    let mut seen = BTreeSet::from([anchor]);
    let mut queue = VecDeque::from([anchor]);
    let mut spawns = Vec::new();
    while let Some(p) = queue.pop_front() {
        if spawns.len() < 100 {
            spawns.push(p);
        }
        for (dx, dy) in DIRECTIONS {
            let q = WorldCell {
                x: p.x + dx,
                y: p.y + dy,
            };
            if map.can_step(p, q) && seen.insert(q) {
                queue.push_back(q);
            }
        }
        assert!(seen.len() <= 24576);
    }
    assert_eq!(seen.len(), 5759);
    map.validate_spawns(&spawns, &cancel).unwrap();
    let mut spawn_hash = Sha256::new();
    for p in &spawns {
        spawn_hash.update(p.x.to_be_bytes());
        spawn_hash.update(p.y.to_be_bytes());
    }
    assert_eq!(
        hex(&spawn_hash.finalize()),
        "0dca5584b6fd017d0c0542d0623ab1c47e9b306f326fc83bcde253659b75e69a"
    );
    let ids: BTreeSet<u16> = [
        2136, 2137, 2138, 2150, 2163, 2164, 2170, 2198, 2199, 2217, 2346, 2347, 2355, 2567, 2568,
        2570, 2572, 2573, 2574, 2778, 2779, 2780, 2781, 2782, 2783, 2784, 2785, 2786, 2787, 2788,
        2789, 2791, 2792, 2794, 2795, 2796, 2797, 2798, 2799, 2800, 2801, 2802, 2803, 2804, 2805,
        2806, 2807, 2808, 2809, 2815, 2816, 2817, 2860, 2861, 2862, 2863, 2865, 2867, 2869, 2873,
        2874, 2875, 2876, 2877, 2878, 2879, 2880, 2882, 2883, 2884, 2885, 2886, 2887, 2888, 2889,
        2891, 2892, 2898, 2953, 2954, 2955, 2956, 2957, 2958, 2959,
    ]
    .into_iter()
    .collect();
    assert_eq!(
        draws
            .iter()
            .map(|d| d.archive_entry_id)
            .collect::<BTreeSet<_>>(),
        ids
    );
    let refs = draws
        .iter()
        .map(|d| (d.archive_entry_id, d.sub_index))
        .collect::<BTreeSet<_>>();
    assert_eq!(refs.len(), 9747);
    let mut archive = SharedArchive::open_with_limits(
        root,
        SharedArchiveKind::Tile,
        ResourceLimits {
            max_entry_bytes: 16 * 1024 * 1024,
            ..Default::default()
        },
    )
    .unwrap();
    let mut all = Sha256::new();
    let mut used = Sha256::new();
    let mut encoded = 0;
    let mut blocks = 0;
    let mut referenced = 0;
    for id in &ids {
        let bytes = archive.read_entry(&format!("{id}.til")).unwrap();
        encoded += bytes.len();
        assert!(encoded <= 64 * 1024 * 1024);
        let tile = decode_classic_til_with_limits(
            &bytes,
            TileLimits {
                max_input_bytes: 16 * 1024 * 1024,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(tile.tiles.len(), 256);
        blocks += tile.tiles.len();
        assert!(blocks * 2304 <= 64 * 1024 * 1024);
        for (i, t) in tile.tiles.iter().enumerate() {
            assert_eq!((t.width, t.height, t.rgba.len()), (24, 24, 2304));
            let e = tile_entry(*id, i as u16, t);
            all.update(e);
            if refs.contains(&(*id, i as u8)) {
                used.update(e);
                referenced += 1;
            }
        }
    }
    assert_eq!((encoded, blocks, referenced), (8165981, 21760, 9747));
    assert_eq!(
        hex(&all.finalize()),
        "27211e0aee966889de20d3608b68e0efd8c71f4cf62769623f976655668d6ec9"
    );
    assert_eq!(
        hex(&used.finalize()),
        "6c70f24270b6c8951900d79bbcf16c3b55c695f88d5973d6364f92b289e4025a"
    );
    if std::env::var("LITO_WORLD_MAP_WRITE_REFERENCE").as_deref() == Ok("1") {
        let directory =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        // Diagnostic views only; these candidate cells do not configure tower camps.
        for (label, p) in [
            ("center", anchor),
            ("west", WorldCell { x: 32765, y: 32810 }),
            ("east", WorldCell { x: 32835, y: 32810 }),
        ] {
            let x = 24 * (p.x - 32704 + p.y - 32768) + 24;
            let y = 12 * (63 - (p.x - 32704) + (p.y - 32768));
            let rendered = render_tiles(
                &mut archive,
                &draws,
                CameraRect {
                    x: x - 400,
                    y: y - 300,
                    width: 800,
                    height: 600,
                },
            )
            .unwrap();
            let output = directory.join(format!("shared-map532-{label}-static-reference.png"));
            let mut encoder = png::Encoder::new(std::fs::File::create(&output).unwrap(), 800, 600);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(rendered.canvas.rgba())
                .unwrap();
            eprintln!("Static map532 diagnostic {}", output.display());
        }
    }
    let mut wrong = binds;
    wrong[2].sha256[0] ^= 1;
    assert!(matches!(
        WorldMap::load_shared(root, &wrong, &cancel),
        Err(WorldMapError::HashMismatch(_))
    ));
}
