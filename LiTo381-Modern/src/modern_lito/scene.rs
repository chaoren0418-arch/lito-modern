//! Reference scene composition using explicitly shared Tile archive entries.
//!
//! The draw list separates geometry/layer order from resource decoding. Only
//! visible tile sets are loaded, once per render, with an aggregate cache budget.
//! Unsupported art is an error with an asset identity, not a silent empty tile.

use std::collections::BTreeMap;

use anyhow::{ensure, Context, Result};

use super::compositor::Canvas;
use super::map::S32Sector;
use super::resources::{SharedArchive, SharedArchiveKind};
use super::tile::{decode_classic_til_with_policy, DecodedTileSet, TileLimits, TileZeroPolicy};

const MAX_DRAW_TILES: usize = 262_144;
const MAX_CACHE_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrawTile {
    pub half_x: u8,
    pub half_y: u8,
    pub archive_entry_id: u16,
    pub sub_index: u8,
    pub x: i32,
    pub y: i32,
    pub layer: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CameraRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// S32 half-tile coordinates, following the verified local MapViewer
/// ViewportRenderer formula. These are sector-local pixel coordinates.
pub fn project_half_tile(x: u8, y: u8) -> (i32, i32) {
    let half = i32::from(x) / 2;
    (
        24 * (i32::from(x) - half + i32::from(y)),
        12 * (63 - half + i32::from(y)),
    )
}

/// Ground-only inspection layer. This intentionally does not claim to render
/// objects, opacity, portals or effects, or establish game-world passability.
pub fn ground_draw_list(sector: &S32Sector) -> Result<Vec<DrawTile>> {
    ensure!(
        sector.floor.len() == 128 * 64,
        "S32 floor dimensions do not match a sector"
    );
    ensure!(
        sector.overlays.len() <= MAX_DRAW_TILES - sector.floor.len(),
        "ground overlay budget exceeded"
    );
    let mut tiles = Vec::with_capacity(sector.floor.len() + sector.overlays.len());
    for (index, reference) in sector.floor.iter().enumerate() {
        let (x, y) = project_half_tile((index % 128) as u8, (index / 128) as u8);
        tiles.push(DrawTile {
            half_x: (index % 128) as u8,
            half_y: (index / 128) as u8,
            archive_entry_id: reference.tile_id,
            sub_index: reference.subtile,
            x,
            y,
            layer: -2,
        });
    }
    for overlay in &sector.overlays {
        let (x, y) = project_half_tile(overlay.x, overlay.y);
        tiles.push(DrawTile {
            half_x: overlay.x,
            half_y: overlay.y,
            archive_entry_id: overlay.tile.tile_id,
            sub_index: overlay.tile.subtile,
            x,
            y,
            layer: -1,
        });
    }
    Ok(tiles)
}

/// Static floor, overlays and Layer4 objects, retaining group/source ordering.
/// This does not apply actor-dependent roof fading, portals or sprite effects.
/// A caller must not treat this inspection list as a gameplay map contract.
pub fn static_draw_list(sector: &S32Sector) -> Result<Vec<DrawTile>> {
    let mut tiles = ground_draw_list(sector)?;
    let count = sector.objects.iter().try_fold(tiles.len(), |count, group| {
        count.checked_add(group.tiles.len())
    });
    let count = count.context("static draw list count overflow")?;
    ensure!(count <= MAX_DRAW_TILES, "static object budget exceeded");
    tiles.try_reserve_exact(count - tiles.len())?;
    for group in &sector.objects {
        for object in &group.tiles {
            let reference = object.shared_tile_reference().with_context(|| {
                format!(
                    "unsupported negative Tile ID in S32 object group {}",
                    group.id
                )
            })?;
            let (x, y) = project_half_tile(object.x, object.y);
            tiles.push(DrawTile {
                half_x: object.x,
                half_y: object.y,
                archive_entry_id: reference.tile_id,
                sub_index: reference.subtile,
                x,
                y,
                layer: i32::from(object.layer),
            });
        }
    }
    Ok(tiles)
}

fn visible(tile: &DrawTile, camera: CameraRect) -> bool {
    let x = i64::from(tile.x) - i64::from(camera.x);
    let y = i64::from(tile.y) - i64::from(camera.y);
    x < i64::from(camera.width) && y < i64::from(camera.height) && x + 24 > 0 && y + 24 > 0
}

pub struct RenderedScene {
    pub canvas: Canvas,
    pub visible_tiles: usize,
    pub tile_sets_loaded: usize,
    pub decoded_cache_bytes: usize,
}

pub fn render_tiles(
    archive: &mut SharedArchive,
    tiles: &[DrawTile],
    camera: CameraRect,
) -> Result<RenderedScene> {
    ensure!(
        archive.kind() == SharedArchiveKind::Tile,
        "scene renderer requires the shared Tile archive"
    );
    ensure!(
        camera.width > 0 && camera.height > 0 && camera.width <= 4096 && camera.height <= 4096,
        "camera dimensions exceed reference renderer budget"
    );
    ensure!(
        tiles.len() <= MAX_DRAW_TILES,
        "draw list exceeds bounded scene budget"
    );
    let mut ordered: Vec<_> = tiles.iter().filter(|tile| visible(tile, camera)).collect();
    // Stable sorting preserves original source order within a native layer.
    ordered.sort_by_key(|tile| tile.layer);
    let mut cache: BTreeMap<u16, DecodedTileSet> = BTreeMap::new();
    let mut decoded_cache_bytes = 0usize;
    let mut canvas = Canvas::new(camera.width, camera.height)?;
    canvas.clear([18, 25, 35]);
    for tile in &ordered {
        if !cache.contains_key(&tile.archive_entry_id) {
            let name = format!("{}.til", tile.archive_entry_id);
            let remaining_pixels = (MAX_CACHE_BYTES - decoded_cache_bytes) / 4;
            ensure!(
                remaining_pixels >= 24 * 24,
                "visible tile pixel cache is full"
            );
            let limits = TileLimits {
                max_total_pixels: remaining_pixels,
                ..TileLimits::default()
            };
            // An archive may have been opened with a larger per-entry limit.
            // Reject an input this decoder cannot accept before reading it.
            let entry = archive
                .entry(&name)
                .with_context(|| format!("locate shared Tile/{name}"))?;
            ensure!(
                u64::from(entry.size) <= limits.max_input_bytes as u64,
                "shared Tile/{name} exceeds scene decoder input budget"
            );
            let bytes = archive
                .read_entry(&name)
                .with_context(|| format!("load shared Tile/{name}"))?;
            let decoded =
                decode_classic_til_with_policy(&bytes, limits, TileZeroPolicy::PreserveCoverage)
                    .with_context(|| format!("decode shared Tile/{name}"))?;
            let decoded_bytes = decoded
                .tiles
                .iter()
                .try_fold(0usize, |sum, tile| sum.checked_add(tile.rgba.len()))
                .context("tile cache size overflow")?;
            decoded_cache_bytes = decoded_cache_bytes
                .checked_add(decoded_bytes)
                .context("tile cache size overflow")?;
            ensure!(
                decoded_cache_bytes <= MAX_CACHE_BYTES,
                "visible tile cache exceeds 64 MiB"
            );
            cache.insert(tile.archive_entry_id, decoded);
        }
        let decoded = cache[&tile.archive_entry_id]
            .tiles
            .get(tile.sub_index as usize)
            .with_context(|| {
                format!(
                    "missing shared Tile/{}.til sub-index {}",
                    tile.archive_entry_id, tile.sub_index
                )
            })?;
        let x = (i64::from(tile.x) - i64::from(camera.x)).try_into()?;
        let y = (i64::from(tile.y) - i64::from(camera.y)).try_into()?;
        canvas.blit_rgba(&decoded.rgba, decoded.width, decoded.height, x, y)?;
    }
    Ok(RenderedScene {
        canvas,
        visible_tiles: ordered.len(),
        tile_sets_loaded: cache.len(),
        decoded_cache_bytes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sector_projection_has_fixed_half_tile_and_isometric_axes() {
        assert_eq!(project_half_tile(0, 0), (0, 756));
        assert_eq!(project_half_tile(1, 0), (24, 756));
        assert_eq!(project_half_tile(2, 0), (24, 744));
        assert_eq!(project_half_tile(0, 1), (24, 768));
        assert_eq!(project_half_tile(127, 63), (3048, 756));
    }
    #[test]
    fn camera_culling_uses_signed_coordinates_without_overflow() {
        let camera = CameraRect {
            x: 100,
            y: 100,
            width: 80,
            height: 60,
        };
        let mut tile = DrawTile {
            half_x: 0,
            half_y: 0,
            archive_entry_id: 0,
            sub_index: 0,
            x: 76,
            y: 100,
            layer: 0,
        };
        assert!(!visible(&tile, camera));
        tile.x = 77;
        assert!(visible(&tile, camera));
        tile.x = 180;
        assert!(!visible(&tile, camera));
        tile.x = i32::MIN;
        assert!(!visible(
            &tile,
            CameraRect {
                x: i32::MAX,
                ..camera
            }
        ));
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; writes ground-only reference PNG under target"]
    fn real_shared_sector_ground_renders_without_lt_resources() {
        use super::super::map::parse_s32;
        use super::super::resources::{read_shared_map_sector, SharedMapSectorKey};
        use sha2::{Digest, Sha256};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared resource root");
        let key = SharedMapSectorKey {
            map_id: 4,
            sector_x: 0x7ffc,
            sector_y: 0x7ff7,
        };
        let bytes = read_shared_map_sector(&root, key).unwrap();
        let sector = parse_s32(&bytes).unwrap();
        let draw_list = ground_draw_list(&sector).unwrap();
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let scene = render_tiles(
            &mut archive,
            &draw_list,
            CameraRect {
                x: 1120,
                y: 420,
                width: 800,
                height: 600,
            },
        )
        .unwrap();
        assert_eq!(scene.visible_tiles, 1734);
        assert_eq!(scene.tile_sets_loaded, 22);
        assert_eq!(scene.decoded_cache_bytes, 12_976_128);
        // This records the modern coverage-preserving reference renderer's
        // baseline, not a claim of native BIN pixel equivalence.
        let rgba_hash = format!("{:X}", Sha256::digest(scene.canvas.rgba()));
        assert_eq!(
            rgba_hash,
            "3F8A9A91252FDF6186FE0BBEB40D70262EBF7464CB0C02D2E8E55048E6E33BE1"
        );
        let output_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&output_dir).unwrap();
        let output = output_dir.join("shared-map4-ground-reference.png");
        let file = std::fs::File::create(&output).unwrap();
        let mut encoder = png::Encoder::new(file, scene.canvas.width(), scene.canvas.height());
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(scene.canvas.rgba())
            .unwrap();
        eprintln!(
            "Ground-only (not objects/world/window): {} tiles, {} sets, {} decoded pixel bytes; RGBA SHA256 {}; {}",
            scene.visible_tiles,
            scene.tile_sets_loaded,
            scene.decoded_cache_bytes,
            rgba_hash,
            output.display()
        );
    }

    #[test]
    #[ignore = "requires LITO_SHARED_RESOURCE_ROOT; writes static terrain reference PNG under target"]
    fn real_shared_static_objects_render_without_lt_resources() {
        use super::super::map::parse_s32;
        use super::super::resources::{read_shared_map_sector, SharedMapSectorKey};
        let root = std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared resource root");
        let bytes = read_shared_map_sector(
            &root,
            SharedMapSectorKey {
                map_id: 4,
                sector_x: 0x7ffc,
                sector_y: 0x7ff7,
            },
        )
        .unwrap();
        let sector = parse_s32(&bytes).unwrap();
        let draw_list = static_draw_list(&sector).unwrap();
        assert_eq!(
            draw_list.len(),
            sector.floor.len() + sector.overlays.len() + 6781
        );
        let mut archive = SharedArchive::open(&root, SharedArchiveKind::Tile).unwrap();
        let scene = render_tiles(
            &mut archive,
            &draw_list,
            CameraRect {
                x: 1120,
                y: 420,
                width: 800,
                height: 600,
            },
        )
        .unwrap();
        assert!(scene.visible_tiles > 1734);
        let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("target/modern-lito-render-tests");
        std::fs::create_dir_all(&directory).unwrap();
        let path = directory.join("shared-map4-static-reference.png");
        let mut encoder = png::Encoder::new(std::fs::File::create(&path).unwrap(), 800, 600);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(scene.canvas.rgba())
            .unwrap();
        eprintln!(
            "Static map, not actor/roof/effects/world: {} tiles, {} sets, {} bytes; {}",
            scene.visible_tiles,
            scene.tile_sets_loaded,
            scene.decoded_cache_bytes,
            path.display()
        );
    }
}
