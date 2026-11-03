use lito_modern::modern_lito::{
    native_multisector_scene::Scene,
    sprite::LayeredSprite,
    world_map::{PixelPoint, ViewProjection},
};
use std::cmp::Reverse;
struct Fragment<'a> {
    a: i64,
    b: i64,
    depth: i16,
    sequence: usize,
    pixels: &'a [u8],
    owner: u32,
    shift: [i64; 2],
    mix: u8,
}
// Independent unoptimized per-pixel compositor. It does NOT call NativeDrawKey,
// NativeDrawCell projection, Raster, Canvas blits, or the compact SPR atlas.
pub fn reference_layers(
    scene: &Scene,
    view: ViewProjection,
    bodies: &[(u32, PixelPoint, &LayeredSprite, usize, u8)],
    width: usize,
    height: usize,
    terrain_enabled: bool,
) -> (Vec<u8>, Vec<u32>) {
    reference_roofs(scene, view, bodies, width, height, terrain_enabled, None)
}
pub fn reference_roofs(
    scene: &Scene,
    view: ViewProjection,
    bodies: &[(u32, PixelPoint, &LayeredSprite, usize, u8)],
    width: usize,
    height: usize,
    terrain_enabled: bool,
    roofs: Option<&serde_json::Value>,
) -> (Vec<u8>, Vec<u32>) {
    let terrain = scene.assets().scene_terrain().unwrap();
    let mut groups = Vec::new();
    for b in scene.assets().geometry().bindings() {
        let sector = scene.assets().geometry().sector(b.key).unwrap();
        groups.extend(std::iter::repeat_n(
            None,
            sector.floor.len() + sector.overlays.len(),
        ));
        for (ordinal, group) in sector.objects.iter().enumerate() {
            groups.extend(std::iter::repeat_n(
                Some((b.key.sector_x, b.key.sector_y, ordinal)),
                group.tiles.len(),
            ));
        }
    }
    assert_eq!(groups.len(), terrain.len());
    let raw = scene
        .assets()
        .scene_roof_raw(&std::sync::atomic::AtomicBool::new(false))
        .unwrap();
    let mut draws = Vec::new();
    for ((cell, depth, key), group) in terrain.iter().zip(groups) {
        if !terrain_enabled {
            break;
        }
        let tile = &scene.assets().scene_tiles()[key];
        if depth.is_none() && (tile.tile_type as u8 & 8) != 0 {
            continue;
        }
        let mut pixels = &tile.rgba;
        let mut mix = 0;
        if let (Some(report), Some((sx, sy, ordinal))) = (roofs, group) {
            if let Some(g) = report["groups"]
                .as_array()
                .unwrap()
                .iter()
                .find(|g| g["sector"] == serde_json::json!([sx, sy]) && g["ordinal"] == ordinal)
            {
                let kind = tile.tile_type as u8;
                let shade = &g[if matches!(kind, 6 | 7) {
                    "base_half"
                } else {
                    "normal"
                }];
                if shade == "Hidden" {
                    continue;
                }
                if shade != "Native" {
                    if kind & 0x30 != 0 {
                        pixels = &raw[key].rgba;
                    }
                    if shade == "Half" {
                        mix = if kind & 0x10 != 0 { 2 } else { 3 };
                    } else {
                        let w = shade["Weight"].as_u64().unwrap() as u8;
                        mix = if w == 31 && (kind & 0x30 != 0 || !matches!(kind, 6 | 7)) {
                            0
                        } else {
                            32 + w
                        };
                    }
                }
            }
        }
        draws.push(Fragment {
            a: cell.half_x.into(),
            b: cell.y.into(),
            depth: depth.map_or(-1, i16::from),
            sequence: draws.len(),
            pixels,
            owner: 0,
            mix,
            shift: [0, 0],
        });
    }
    for (id, center, spr, sub, mix) in bodies {
        let dx = i64::from(center.x) - i64::from(view.anchor_pixel.x);
        let dy = i64::from(center.y) - i64::from(view.anchor_pixel.y);
        let x = i64::from(view.anchor_cell.x) + (dx - 2 * dy + 24).div_euclid(48);
        let y = i64::from(view.anchor_cell.y) + (dx + 2 * dy + 24).div_euclid(48);
        let shift = [
            dx - 24 * (x - i64::from(view.anchor_cell.x) + y - i64::from(view.anchor_cell.y)),
            dy - 12 * (y - i64::from(view.anchor_cell.y) - x + i64::from(view.anchor_cell.x)),
        ];
        for f in &spr.fragments.frames[*sub] {
            draws.push(Fragment {
                a: 2 * x - 32768 + i64::from(f.half_x),
                b: y + i64::from(f.y),
                depth: i16::from(f.depth),
                sequence: draws.len(),
                pixels: &spr.fragments.blocks[usize::from(f.block_id)],
                owner: *id,
                mix: *mix,
                shift,
            });
        }
    }
    let coord = |a: i64, b: i64| (2 * b - (a & !1), ((a + 1) & !1) + 2 * b);
    draws.sort_by_key(|d| {
        let (row, col) = coord(d.a, d.b);
        (row, col, d.depth, Reverse(d.sequence))
    });
    let (r0, c0) = coord(
        2 * i64::from(view.anchor_cell.x) - 32768,
        i64::from(view.anchor_cell.y),
    );
    let mut rgba = [18, 25, 35, 255].repeat(width * height);
    let mut owners = vec![0; width * height];
    for d in draws {
        let (row, col) = coord(d.a, d.b);
        let x = i64::from(view.anchor_pixel.x) - 24 + 12 * (col - c0) + d.shift[0];
        let y = i64::from(view.anchor_pixel.y) - 12 + 6 * (row - r0) + d.shift[1];
        for sy in 0..24 {
            let yy = y + sy;
            if yy < 0 || yy >= height as i64 {
                continue;
            }
            for sx in 0..24 {
                let xx = x + sx;
                if xx < 0 || xx >= width as i64 {
                    continue;
                }
                let src = ((sy * 24 + sx) * 4) as usize;
                let dst = yy as usize * width + xx as usize;
                let alpha = u32::from(d.pixels[src + 3]);
                if alpha == 255 && matches!(d.mix, 0 | 5 | 6 | 24..=30) {
                    owners[dst] = d.owner;
                }
                for c in 0..3 {
                    if matches!(d.mix, 0 | 4) {
                        rgba[dst * 4 + c] = ((u32::from(d.pixels[src + c]) * alpha
                            + u32::from(rgba[dst * 4 + c]) * (255 - alpha)
                            + 127)
                            / 255) as u8;
                    } else if alpha != 0 {
                        assert!(alpha == 255 || ((d.mix == 3 || d.mix >= 32) && alpha == 128));
                        let s = u32::from(d.pixels[src + c] >> 3);
                        let old = u32::from(rgba[dst * 4 + c] >> 3);
                        let v = match d.mix {
                            1 | 3 | 5 => (s + old) / 2,
                            6 => s / 2 + 3,
                            24..=30 => (s + if c == 1 { u32::from(d.mix - 16) } else { 0 }).min(31),
                            2 => 31 - ((31 - s) * (31 - old) + 30) / 31,
                            32..=63 => {
                                (s * u32::from(d.mix - 32) + old * u32::from(63 - d.mix)) / 31
                            }
                            _ => panic!("unsupported reference mix"),
                        } as u8;
                        rgba[dst * 4 + c] = (v << 3) | (v >> 2);
                    }
                }
            }
        }
    }
    (rgba, owners)
}

pub fn reference_inner(
    scene: &Scene,
    view: ViewProjection,
    bodies: &[(u32, PixelPoint, &LayeredSprite, usize)],
    width: usize,
    height: usize,
    terrain_enabled: bool,
) -> (Vec<u8>, Vec<u32>) {
    let layers: Vec<_> = bodies
        .iter()
        .map(|(id, p, s, sub)| (*id, *p, *s, *sub, 0))
        .collect();
    reference_layers(scene, view, &layers, width, height, terrain_enabled)
}
pub fn reference(
    scene: &Scene,
    view: ViewProjection,
    bodies: &[(u32, PixelPoint, &LayeredSprite, usize)],
    width: usize,
    height: usize,
) -> (Vec<u8>, Vec<u32>) {
    reference_inner(scene, view, bodies, width, height, true)
}


/// Static composition probes deliberately select arbitrary cameras over the
/// complete original map. They do not represent a live actor's resident view.
/// Keep this fixture explicit so the scalar reference still checks every source
/// group and pixel, while live Model tests exercise bounded residency separately.
pub struct FullNativeFixture(Scene);
impl FullNativeFixture {
    pub fn scene(&self) -> &Scene { &self.0 }
}
pub fn full_native_fixture(root: &std::path::Path, map_id:u32) -> FullNativeFixture {
    use lito_modern::modern_lito::{native_resources as nr,resources::SharedMapSectorKey,world_map::{WorldMap,SectorInput}};
    let native=root.join("LiTo");let cancel=std::sync::atomic::AtomicBool::new(false);
    let mut files:Vec<_>=std::fs::read_dir(native.join("LTmap").join(map_id.to_string())).unwrap().map(|x|x.unwrap().path()).filter(|p|p.extension().is_some_and(|x|x=="s32")).collect();files.sort();
    let inputs=files.iter().map(|p|{let n=p.file_stem().unwrap().to_str().unwrap();SectorInput{key:SharedMapSectorKey{map_id,sector_x:u16::from_str_radix(&n[..4],16).unwrap(),sector_y:u16::from_str_radix(&n[4..],16).unwrap()},bytes:std::fs::read(p).unwrap()}}).collect();
    let map=WorldMap::from_native_sector_bytes(inputs,&cancel).unwrap();
    let observed=nr::NativeObservedArchives::open(&native,nr::Limits::default(),&cancel).unwrap();
    let plan=nr::NativeAssetPlan{namespace:nr::ResourceNamespace::NativeLitoV1,map_id,canonical_sha256:map.canonical_digest(),archives:observed.pins().to_vec(),sectors:map.bindings().iter().map(|b|nr::SectorPin{key:nr::NativeSectorKey{map_id,x:b.key.sector_x,y:b.key.sector_y},pin:nr::BytePin{bytes:b.byte_length.into(),sha256:b.sha256}}).collect()};
    let assets=nr::NativeResourceAdapter::pinned_world().load_static_observed(&nr::NativeSources{map_root:native.join("LTmap"),archive_root:native},&plan,nr::Limits::default(),observed,&cancel).unwrap();
    FullNativeFixture(Scene::new_cancel(assets,&cancel).unwrap())
}
