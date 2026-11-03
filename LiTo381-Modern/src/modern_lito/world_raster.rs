//! Bounded per-half-cell composition shared by world art and its hit surface.
//! The native cell/depth contract is in world_depth. Snapshot roster order is
//! the explicit modern insertion policy between actors; native timer/update
//! history is not reconstructed by this queue. Roof state is owned by the
//! render worker and supplied as a qualified per-group shade.
#[cfg(test)]
use super::tile_pixels::RGBA_BYTES;
use super::{
    compositor::{Canvas, FragmentRows},
    native_object_dispatch::{prepared_route,select as select_object,Input as ObjectInput,Route},
    sprite::{CompactSpriteBlock, SpriteFrame},
    tile::DecodedTile,
    tile_pixels::CompactTilePixels,
    world_depth::{NativeDrawCell, NativeDrawKey},
    world_map::{PixelPoint, ViewProjection},
};
use anyhow::{ensure, Context, Result};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

const MAX_VISIBLE_FRAGMENTS: usize = 262_144;

pub(crate) struct PreparedTile {
    tile: DecodedTile,
    rows: FragmentRows,
    roof_raw: Option<(Vec<u8>, FragmentRows)>,
    compact: Option<CompactTilePixels>,
    roof_compact: Option<CompactTilePixels>,
    object_header: Option<[i8;4]>,
}
impl PreparedTile {
    #[cfg(test)]
    pub(crate) fn roof_raw_rgba(&self) -> Option<std::borrow::Cow<'_, [u8]>> {
        self.roof_raw
            .as_ref()
            .map(|(rgba, _)| match &self.roof_compact {
                Some(packed) => {
                    let mut bytes = [0; RGBA_BYTES];
                    packed.write_rgba(&mut bytes);
                    std::borrow::Cow::Owned(bytes.to_vec())
                }
                None => std::borrow::Cow::Borrowed(rgba.as_slice()),
            })
    }
    pub(crate) fn new(tile: DecodedTile) -> Result<Self> {
        ensure!(
            tile.width == 24 && tile.height == 24,
            "world tile dimensions"
        );
        let rows = FragmentRows::new(&tile.rgba)?;
        let object_header=(tile.tile_type as u8&2!=0).then_some([
            tile.x_offset as i8,tile.y_offset as i8,tile.content_width as i8,tile.content_height as i8]);
        Ok(Self {
            tile,
            rows,
            roof_raw: None,
            compact: None,
            roof_compact: None,
            object_header,
        })
    }
    pub(crate) fn with_object_header(mut self,header:[i8;4])->Result<Self> {
        ensure!(self.object_header.is_none_or(|known|known==header),"RLE object header differs from decoded source");
        self.object_header=Some(header);Ok(self)
    }
    pub(crate) fn with_roof_raw(mut self, raw: DecodedTile) -> Result<Self> {
        ensure!(
            raw.tile_type == self.tile.tile_type
                && raw.source_offset == self.tile.source_offset
                && (raw.width, raw.height) == (24, 24)
                && raw.rgba.len() == 24 * 24 * 4
                && raw.rgba.chunks_exact(4).all(|p| matches!(p[3], 0 | 255)),
            "roof raw tile identity/coverage mismatch"
        );
        let rows = FragmentRows::new(&raw.rgba)?;
        self.roof_raw = Some((raw.rgba, rows));
        self.roof_compact = None;
        Ok(self)
    }
    /// Only the large-map loader chooses compact storage. Metadata/row masks
    /// and every pixel remain identical; no compression work occurs in paint.
    pub(crate) fn compact_storage(mut self) -> Result<Self> {
        self.compact = CompactTilePixels::try_new(&self.tile.rgba)?;
        if self.compact.is_some() {
            self.tile.rgba = Vec::new();
        }
        if let Some((rgba, _)) = &mut self.roof_raw {
            self.roof_compact = CompactTilePixels::try_new(rgba)?;
            if self.roof_compact.is_some() {
                *rgba = Vec::new();
            }
        }
        Ok(self)
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        self.compact
            .as_ref()
            .map_or(self.tile.rgba.capacity(), CompactTilePixels::retained_bytes)
            + FragmentRows::BYTES
            + self.roof_raw.as_ref().map_or(0, |(rgba, _)| {
                self.roof_compact
                    .as_ref()
                    .map_or(rgba.capacity(), CompactTilePixels::retained_bytes)
                    + FragmentRows::BYTES
            })
    }
    #[cfg(test)]
    pub(crate) fn rgba_pixels<'a>(&'a self, scratch: &'a mut [u8; RGBA_BYTES]) -> &'a [u8] {
        match &self.compact {
            Some(packed) => {
                packed.write_rgba(scratch);
                scratch
            }
            None => &self.tile.rgba,
        }
    }
    fn pixels(&self) -> Pixels<'_> {
        match &self.compact {
            Some(packed) => Pixels::CompactTile(packed, &self.rows),
            None => Pixels::Rgba(&self.tile.rgba, &self.rows),
        }
    }
    fn floor_pixels(&self,parity:u8,output:&mut [u8;24*24*4],normal_scratch:&mut [u8;24*24*4],raw_scratch:&mut [u8;24*24*4])->Result<()> {
        let normal=match &self.compact {Some(p)=>{p.write_rgba(normal_scratch);&normal_scratch[..]},None=>&self.tile.rgba};
        let raw=self.roof_raw.as_ref().map(|(rgba,_)|match &self.roof_compact {Some(p)=>{p.write_rgba(raw_scratch);&raw_scratch[..]},None=>rgba.as_slice()});
        ensure!(super::native_terrain_pixels::floor_pixels(self.kind(),normal,raw,parity,output)?,"skipped floor entered raster");Ok(())
    }
}
impl PreparedTile {
    pub(crate) fn kind(&self) -> super::tile::ClassicTileType {
        self.tile.tile_type
    }
    #[cfg(test)]
    pub(crate) fn decoded_for_test(&self) -> DecodedTile {
        let mut decoded = self.tile.clone();
        let mut scratch = [0; RGBA_BYTES];
        decoded.rgba = self.rgba_pixels(&mut scratch).to_vec();
        decoded
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DrawVisibility {
    owner: u16,
    width: u32,
    height: u32,
    pixels: Arc<[u16]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        sprite::{SpriteBlockReference, SpriteFrameFragments},
        tile::ClassicTileType,
        world_map::WorldCell,
    };
    fn view(x: i32, y: i32) -> ViewProjection {
        ViewProjection {
            anchor_cell: WorldCell { x: 32768, y: 32768 },
            anchor_pixel: PixelPoint { x, y },
        }
    }
    fn origin() -> NativeDrawCell {
        NativeDrawCell {
            half_x: 32768,
            y: 32768,
        }
    }
    fn tile(rgb: [u8; 4]) -> PreparedTile {
        PreparedTile::new(DecodedTile {
            tile_type: ClassicTileType::RleEven,
            source_offset: 0,
            x_offset: 0,
            y_offset: 0,
            content_width: 24,
            content_height: 24,
            width: 24,
            height: 24,
            rgba: rgb.repeat(576),
        })
        .unwrap()
    }
    fn frame(depth: u8, rgb: [u8; 4]) -> SpriteFrame {
        SpriteFrame {
            fragments: Some(SpriteFrameFragments {
                references: vec![SpriteBlockReference {
                    half_x: 0,
                    y: 0,
                    depth,
                    block_id: 0,
                }],
                atlas: Arc::new(vec![CompactSpriteBlock::test_rgba(&rgb.repeat(576))]),
            }),
            x_offset: 0,
            y_offset: 0,
            width: 24,
            height: 24,
            frame_type: depth,
            unknown1: 0,
            unknown2: 0,
            rgba: rgb.repeat(576),
        }
    }
    fn pixel(c: &Canvas, x: usize, y: usize) -> &[u8] {
        &c.rgba()[(y * c.width() as usize + x) * 4..(y * c.width() as usize + x) * 4 + 4]
    }

    #[test]
    fn compact_tiles_preserve_blending_roofs_clipping_depth_and_hit_ownership() {
        use crate::modern_lito::world_roofs::Shade;
        for kind in [
            ClassicTileType::DiamondRight,
            ClassicTileType::DiamondLeft,
            ClassicTileType::RleEven,
            ClassicTileType::RleOdd,
            ClassicTileType::RleEvenHalfOpacity,
            ClassicTileType::RleOddHalfOpacity,
            ClassicTileType::DiamondRightFlag8,
            ClassicTileType::DiamondLeftFlag8,
            ClassicTileType::DiamondRightCloudAlpha,
            ClassicTileType::DiamondLeftCloudAlpha,
            ClassicTileType::RleEvenCloudAlpha,
            ClassicTileType::RleOddCloudAlpha,
            ClassicTileType::RleEvenInvertedAlpha,
            ClassicTileType::RleOddInvertedAlpha,
        ] {
            let mut decoded = tile([0; 4]).decoded_for_test();
            decoded.tile_type = kind;
            for i in 0..576 {
                if i % 7 < 3 {
                    decoded.rgba[i * 4..i * 4 + 4].copy_from_slice(&[
                        31,
                        91,
                        151,
                        if matches!(kind,ClassicTileType::RleEvenHalfOpacity|ClassicTileType::RleOddHalfOpacity) {128}
                        else if kind as u8 &0x30!=0 && i%2==0 {128}else{255},
                    ]);
                }
            }
            decoded.rgba[0..4].copy_from_slice(&[0, 0, 0,
                if matches!(kind,ClassicTileType::RleEvenHalfOpacity|ClassicTileType::RleOddHalfOpacity){128}else{255}]);
            let mut raw = decoded.clone();
            for pixel in raw.rgba.chunks_exact_mut(4) {
                if pixel[3] != 0 {
                    pixel.copy_from_slice(&[8, 33, 132, 255]);
                }
            }
            let prepare = || {
                let p = PreparedTile::new(decoded.clone()).unwrap();
                if kind as u8 & 0x30 != 0 {
                    p.with_roof_raw(raw.clone()).unwrap()
                } else {
                    p
                }
            };
            let reference = prepare();
            let compact = prepare().compact_storage().unwrap();
            assert_eq!(compact.decoded_for_test(), decoded);
            assert_eq!(compact.roof_raw_rgba(), reference.roof_raw_rgba());
            assert!(compact.retained_bytes() < reference.retained_bytes());
            for shade in [
                Shade::Native,
                Shade::Weight(0),
                Shade::Weight(15),
                Shade::Weight(31),
                Shade::Half,
                Shade::Hidden,
            ] {
                for (x, y) in [(24, 12), (13, 6)] {
                    for depth in [None, Some(20)] {
                        let render = |p: &PreparedTile| {
                            let mut c = Canvas::new(24, 24).unwrap();
                            let actor = frame(10, [250, 0, 0, 255]);
                            let mut r = Raster::new(view(x, y), &c);
                            r.sprite(origin(), &actor, 1, Blend::Over).unwrap();
                            r.tile_roof(origin(), depth, p, shade).unwrap();
                            let mask = r.paint(&mut c, &AtomicBool::new(false), 1).unwrap();
                            (c.rgba().to_vec(), mask)
                        };
                        assert_eq!(render(&reference), render(&compact), "{kind:?} {shade:?}");
                    }
                }
            }
        }
    }
    #[test]
    fn background_gate_includes_plain_body_and_object_pixels() {
        let mut floor=tile([0;4]).decoded_for_test();floor.rgba[..4].copy_from_slice(&[0,0,0,255]);
        let floor=PreparedTile::new(floor).unwrap();
        let mut wall=tile([0;4]).decoded_for_test();wall.rgba[(7*24+7)*4..(7*24+7)*4+4].copy_from_slice(&[0,0,255,255]);
        let wall=PreparedTile::new(wall).unwrap();let actor=frame(10,[255,0,0,255]);
        let mut c=Canvas::new(24,24).unwrap();let mut raster=Raster::new(view(24,12),&c);
        raster.tile(origin(),None,&floor).unwrap();raster.sprite(origin(),&actor,1,Blend::Over).unwrap();raster.tile(origin(),Some(20),&wall).unwrap();
        let mut gate=super::super::native_terrain_pixels::Gate::new(24,24).unwrap();
        let masks=raster.paint_with_background_gate(&mut c,&AtomicBool::new(false),1,Some(&mut gate)).unwrap();
        assert_eq!(gate.covered(),576);assert_eq!(gate.values()[0],1);assert_eq!(gate.values()[7*24+7],1);
        assert_eq!(pixel(&c,0,0),[255,0,0,255]);assert_eq!(pixel(&c,7,7),[0,0,255,255]);
        assert!(masks[0].contains(0,0));assert!(!masks[0].contains(7,7));
    }
    #[test]
    fn background_gate_preserves_black_floor_through_special_sprite_blends() {
        use crate::modern_lito::native_actor_states::Style;
        for blend in [Blend::Shadow,Blend::Screen,Blend::Multiply,Blend::Actor(Style::Half),
            Blend::Actor(Style::Dim),Blend::Actor(Style::Green(8)),Blend::Actor(Style::Hidden)] {
            let mut decoded=tile([0;4]).decoded_for_test();decoded.rgba[..4].copy_from_slice(&[0,0,0,255]);
            let floor=PreparedTile::new(decoded).unwrap();let actor=frame(10,[255,0,0,255]);
            let mut c=Canvas::new(24,24).unwrap();let mut r=Raster::new(view(24,12),&c);
            r.tile(origin(),None,&floor).unwrap();r.sprite(origin(),&actor,1,blend).unwrap();
            let mut gate=super::super::native_terrain_pixels::Gate::new(24,24).unwrap();
            r.paint_with_background_gate(&mut c,&AtomicBool::new(false),1,Some(&mut gate)).unwrap();
            assert_eq!(gate.covered(),1);assert_eq!(gate.values()[0],1);
            assert!(gate.values()[1..].iter().all(|&v|v==0));
            assert!(c.rgba().chunks_exact(4).all(|p|p[3]==255));
        }
    }
    #[test]
    fn opaque_weight31_roof_and_plain_object_have_equal_pixels_but_different_gate() {
        use crate::modern_lito::world_roofs::Shade;
        let tile=tile([0,0,255,255]);
        let paint=|shade| {
            let mut c=Canvas::new(24,24).unwrap();let mut r=Raster::new(view(24,12),&c);
            r.tile_roof(origin(),Some(20),&tile,shade).unwrap();
            let mut gate=super::super::native_terrain_pixels::Gate::new(24,24).unwrap();
            r.paint_with_background_gate(&mut c,&AtomicBool::new(false),0,Some(&mut gate)).unwrap();
            (c.rgba().to_vec(),gate)
        };
        let (normal,normal_gate)=paint(Shade::Native);let (weighted,weighted_gate)=paint(Shade::Weight(31));
        assert_eq!(normal,weighted);assert_eq!(normal_gate.covered(),576);assert_eq!(weighted_gate.covered(),0);
    }
    #[test]
    fn decoded_tile_kind_roof_and_compact_storage_preserve_original_gate_provenance() {
        use crate::modern_lito::world_roofs::Shade;
        let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("../../tests/fixtures/native-floor-gate-original.json")).unwrap();
        let sources=fixture["sources"].as_object().unwrap();assert_eq!(sources.len(),16);
        for (kind,hex) in sources {
            let kind:u8=kind.parse().unwrap();let hex=hex.as_str().unwrap();
            let block:Vec<_>=hex.as_bytes().chunks_exact(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect();
            let header=[block[1] as i8,block[2] as i8,block[3] as i8,block[4] as i8];
            let mut bytes=1u32.to_le_bytes().to_vec();bytes.extend(0u32.to_le_bytes());bytes.extend((block.len() as u32).to_le_bytes());bytes.extend(block);
            let decoded=crate::modern_lito::tile::decode_classic_til(&bytes).unwrap().tiles.remove(0);
            let raw=crate::modern_lito::tile::decode_classic_til_with_policy(&bytes,Default::default(),crate::modern_lito::tile::TileZeroPolicy::RawRgb555Coverage).unwrap().tiles.remove(0);
            let prepared=PreparedTile::new(decoded).unwrap().with_object_header(header).unwrap();
            let prepared=if kind&0x30!=0 {prepared.with_roof_raw(raw.clone()).unwrap()}else{prepared};
            for compact in [false,true] {
                let mut prepared=PreparedTile::new(prepared.decoded_for_test()).unwrap().with_object_header(header).unwrap();
                if kind&0x30!=0 {prepared=prepared.with_roof_raw(raw.clone()).unwrap();}
                let prepared=if compact {prepared.compact_storage().unwrap()}else{prepared};
                for shade in [Shade::Native,Shade::Half,Shade::Weight(0),Shade::Weight(15),Shade::Weight(31),Shade::Hidden] {
                    for [x,y] in [[0,0],[-9,-7],[28,29]] {
                        let mut c=Canvas::new(32,32).unwrap();let mut r=Raster::new(view(x+24,y+12),&c);
                        r.tile_roof(origin(),Some(20),&prepared,shade).unwrap();
                        let mut gate=super::super::native_terrain_pixels::Gate::new(32,32).unwrap();
                        r.paint_with_background_gate(&mut c,&AtomicBool::new(false),0,Some(&mut gate)).unwrap();
                        let ordinary=shade==Shade::Native && matches!(kind,2|3);
                        let unknown=usize::from(kind&2==0 && shade!=Shade::Hidden);
                        assert_eq!(gate.unqualified_fragments(),unknown);
                        assert_eq!(gate.provenance_complete(),unknown==0);
                        for dy in 0..32 {for dx in 0..32 {
                            let sx=dx as i32-x;let sy=dy as i32-y;
                            let covered=ordinary && (0..24).contains(&sx)&&(0..24).contains(&sy)
                                && raw.rgba[(sy as usize*24+sx as usize)*4+3]!=0;
                            assert_eq!(gate.values()[dy*32+dx],u8::from(covered),"kind{kind} shade{shade:?} compact{compact} at{dx},{dy}");
                        }}
                    }
                }
            }
        }
    }
    #[test]
    fn raw_til_preview_does_not_prove_original_rle_object_coverage() {
        let block=vec![0u8;577];let mut bytes=1u32.to_le_bytes().to_vec();bytes.extend(0u32.to_le_bytes());
        bytes.extend((block.len() as u32).to_le_bytes());bytes.extend(block);
        let decoded=crate::modern_lito::tile::decode_classic_til(&bytes).unwrap().tiles.remove(0);
        let prepared=PreparedTile::new(decoded).unwrap().with_object_header([0;4]).unwrap();
        let mut c=Canvas::new(24,24).unwrap();let mut r=Raster::new(view(24,12),&c);
        r.tile(origin(),Some(20),&prepared).unwrap();let mut gate=super::super::native_terrain_pixels::Gate::new(24,24).unwrap();
        r.paint_with_background_gate(&mut c,&AtomicBool::new(false),0,Some(&mut gate)).unwrap();
        assert_eq!(gate.covered(),0);assert_eq!(gate.unqualified_fragments(),1);assert!(!gate.provenance_complete());
        assert!(c.rgba().chunks_exact(4).any(|p|p==[0,0,0,255].as_slice()));
    }
    #[test]
    fn plain_click_through_effect_adds_encoded_black_gate_without_hit_owner() {
        let effect=frame(10,[0,0,0,255]);let mut c=Canvas::new(24,24).unwrap();
        let mut r=Raster::new(view(24,12),&c);r.sprite(origin(),&effect,0,Blend::EffectOver).unwrap();
        let mut gate=super::super::native_terrain_pixels::Gate::new(24,24).unwrap();
        let masks=r.paint_with_background_gate(&mut c,&AtomicBool::new(false),0,Some(&mut gate)).unwrap();
        assert!(masks.is_empty());assert_eq!(gate.covered(),576);assert_eq!(pixel(&c,0,0),[0,0,0,255]);
    }
    #[test]
    fn one_actor_is_partly_behind_wall_and_hit_mask_matches_painted_fragments() {
        let mut c = Canvas::new(96, 96).unwrap();
        let wall = tile([0, 0, 255, 255]);
        let mut actor = frame(10, [255, 0, 0, 255]);
        let f = actor.fragments.as_mut().unwrap();
        f.atlas = Arc::new(vec![
            CompactSpriteBlock::test_rgba(&[255, 0, 0, 255].repeat(576)),
            CompactSpriteBlock::test_rgba(&[0, 255, 0, 255].repeat(576)),
        ]);
        f.references.push(SpriteBlockReference {
            half_x: 1,
            y: 0,
            depth: 30,
            block_id: 1,
        });
        let mut r = Raster::new(view(48, 48), &c);
        r.tile(origin(), Some(20), &wall).unwrap();
        r.tile(
            NativeDrawCell {
                half_x: 32769,
                ..origin()
            },
            Some(20),
            &wall,
        )
        .unwrap();
        r.sprite(origin(), &actor, 1, Blend::Over).unwrap();
        let masks = r.paint(&mut c, &AtomicBool::new(false), 1).unwrap();
        assert_eq!(pixel(&c, 24, 36), [0, 0, 255, 255]);
        assert_eq!(pixel(&c, 48, 36), [0, 255, 0, 255]);
        assert!(!masks[0].contains(24, 36));
        assert!(masks[0].contains(48, 36));
        assert!(!masks[0].contains(-1, 36));
        assert!(!masks[0].contains(96, 36));
    }

    #[test]
    fn equal_depth_reverse_insertion_and_transparent_holes_do_not_steal_hits() {
        let mut c = Canvas::new(48, 48).unwrap();
        let rear = frame(18, [255, 0, 0, 255]);
        let mut front = frame(18, [0, 255, 0, 255]);
        let mut pixels = [0, 255, 0, 255].repeat(576);
        pixels[..4].copy_from_slice(&[0, 0, 0, 0]);
        front.fragments.as_mut().unwrap().atlas =
            Arc::new(vec![CompactSpriteBlock::test_rgba(&pixels)]);
        let mut r = Raster::new(view(24, 12), &c);
        // Earlier insert paints last. Its hole exposes the newer red body.
        r.sprite(origin(), &front, 1, Blend::Over).unwrap();
        r.sprite(origin(), &rear, 2, Blend::Over).unwrap();
        let m = r.paint(&mut c, &AtomicBool::new(false), 2).unwrap();
        assert_eq!(pixel(&c, 0, 0), [255, 0, 0, 255]);
        assert_eq!(pixel(&c, 1, 0), [0, 255, 0, 255]);
        assert!(!m[0].contains(0, 0));
        assert!(m[1].contains(0, 0));
        assert!(m[0].contains(1, 0));
        assert!(!m[1].contains(1, 0));
    }

    #[test]
    fn clipped_shadows_and_screen_keep_owner_while_opaque_terrain_clears_it() {
        let mut c = Canvas::new(24, 24).unwrap();
        let actor = frame(10, [255, 0, 0, 255]);
        let shadow = frame(11, [0, 0, 0, 255]);
        let light = frame(12, [0, 255, 0, 255]);
        let mut r = Raster::new(view(12, 6), &c);
        r.sprite(origin(), &actor, 1, Blend::Over).unwrap();
        r.sprite(origin(), &shadow, 0, Blend::Shadow).unwrap();
        r.sprite(origin(), &light, 0, Blend::Screen).unwrap();
        let m = r.paint(&mut c, &AtomicBool::new(false), 1).unwrap();
        assert!(m[0].contains(0, 0));
        assert!(m[0].contains(11, 17));
        assert!(!m[0].contains(12, 17));
        assert_eq!(pixel(&c, 0, 0), [123, 255, 0, 255]);
        let wall = tile([0, 0, 0, 255]);
        let mut r = Raster::new(view(12, 6), &c);
        r.sprite(origin(), &actor, 1, Blend::Over).unwrap();
        r.tile(origin(), Some(255), &wall).unwrap();
        let m = r.paint(&mut c, &AtomicBool::new(false), 1).unwrap();
        assert_eq!(pixel(&c, 0, 0), [0, 0, 0, 255]);
        assert!(!m[0].contains(0, 0));
    }

    #[test]
    fn roof_effects_use_raw_source_weight_then_native_settled_priority() {
        use crate::modern_lito::world_roofs::Shade;
        for kind in [
            ClassicTileType::RleEvenCloudAlpha,
            ClassicTileType::RleEvenInvertedAlpha,
        ] {
            let mut effect = tile([0, 0, 0, 0]);
            effect.tile.tile_type = kind;
            let mut raw = effect.tile.clone();
            raw.rgba = [0, 255, 0, 255].repeat(576);
            raw.rgba[..4].copy_from_slice(&[0; 4]);
            let effect = effect.with_roof_raw(raw).unwrap();
            for shade in [
                Shade::Native,
                Shade::Weight(31),
                Shade::Weight(15),
                Shade::Half,
                Shade::Hidden,
            ] {
                let mut c = Canvas::new(24, 24).unwrap();
                let actor = frame(10, [255, 0, 0, 255]);
                let mut r = Raster::new(view(24, 12), &c);
                r.sprite(origin(), &actor, 1, Blend::Over).unwrap();
                r.tile_roof(origin(), Some(20), &effect, shade).unwrap();
                let masks = r.paint(&mut c, &AtomicBool::new(false), 1).unwrap();
                assert_eq!(pixel(&c, 0, 0), [255, 0, 0, 255]);
                assert!(masks[0].contains(0, 0));
                let expected = match shade {
                    Shade::Native | Shade::Hidden => [255, 0, 0, 255],
                    Shade::Weight(31) => [0, 255, 0, 255],
                    Shade::Weight(15) => [132, 123, 0, 255],
                    Shade::Half if kind == ClassicTileType::RleEvenCloudAlpha => [255, 255, 0, 255],
                    Shade::Half => [123, 123, 0, 255],
                    _ => unreachable!(),
                };
                assert_eq!(pixel(&c, 1, 0), expected);
                assert_eq!(masks[0].contains(1, 0), shade != Shade::Weight(31));
            }
            let mut unqualified = tile([0; 4]);
            unqualified.tile.tile_type = kind;
            let c = Canvas::new(24, 24).unwrap();
            assert!(Raster::new(view(24, 12), &c)
                .tile_roof(origin(), Some(20), &unqualified, Shade::Half)
                .is_err());
        }
    }

    #[test]
    fn cancellation_bad_fragments_and_floor_flag8_are_explicit() {
        let mut c = Canvas::new(24, 24).unwrap();
        c.clear([7, 8, 9]);
        let before = c.rgba().to_vec();
        let mut actor = frame(18, [255, 0, 0, 255]);
        actor.fragments = None;
        let mut r = Raster::new(view(24, 12), &c);
        assert!(r.sprite(origin(), &actor, 1, Blend::Over).is_err());
        assert!(r.paint(&mut c, &AtomicBool::new(true), 1).is_err());
        assert_eq!(c.rgba(), before);
        let mut wall = tile([0, 0, 0, 255]);
        wall.tile.tile_type = ClassicTileType::DiamondRightFlag8;
        let mut r = Raster::new(view(24, 12), &c);
        assert!(!r.tile(origin(), None, &wall).unwrap());
        assert!(r.tile(origin(), Some(0), &wall).unwrap());
        r.paint(&mut c, &AtomicBool::new(false), 0).unwrap();
        assert_eq!(pixel(&c, 0, 0), [0, 0, 0, 255]);
    }
}
impl DrawVisibility {
    pub fn contains(&self, x: i64, y: i64) -> bool {
        x >= 0
            && y >= 0
            && x < i64::from(self.width)
            && y < i64::from(self.height)
            && self.pixels[y as usize * self.width as usize + x as usize] == self.owner
    }
}

#[derive(Clone, Copy)]
pub(crate) enum Blend {
    Over,
    EffectOver,
    Actor(super::native_actor_states::Style),
    Shadow,
    Screen,
    Multiply,
    Roof(super::world_roofs::Shade),
}
struct Fragment<'a> {
    key: NativeDrawKey,
    position: PixelPoint,
    source: Pixels<'a>,
    owner: u16,
    blend: Blend,
    /// Source dispatch provenance. Floors have their own writer; opaque roof
    /// weights and effect blends must not acquire ordinary-copy provenance.
    native_route: Option<Route>,
    object_coverage_qualified: bool,
}
enum Pixels<'a> {
    Rgba(&'a [u8], &'a FragmentRows),
    Compact(&'a CompactSpriteBlock),
    CompactTile(&'a CompactTilePixels, &'a FragmentRows),
    Floor(&'a PreparedTile,u8),
}
pub(crate) struct Raster<'a> {
    view: ViewProjection,
    width: u32,
    height: u32,
    sequence: u64,
    fragments: Vec<Fragment<'a>>,
}
impl<'a> Raster<'a> {
    pub fn new(view: ViewProjection, canvas: &Canvas) -> Self {
        Self {
            view,
            width: canvas.width(),
            height: canvas.height(),
            sequence: 0,
            fragments: Vec::new(),
        }
    }
    fn push(
        &mut self,
        cell: NativeDrawCell,
        depth: Option<u8>,
        source: Pixels<'a>,
        owner: u16,
        blend: Blend,
        native_route: Option<Route>,
        object_coverage_qualified: bool,
    ) -> Result<bool> {
        if let Pixels::Rgba(rgba, _) = &source {
            ensure!(
                rgba.len() == 24 * 24 * 4,
                "world fragment must be 24x24 RGBA"
            );
        }
        self.push_shifted(cell, depth, source, owner, blend, PixelPoint { x: 0, y: 0 },native_route,object_coverage_qualified)
    }
    fn push_shifted(
        &mut self,
        cell: NativeDrawCell,
        depth: Option<u8>,
        source: Pixels<'a>,
        owner: u16,
        blend: Blend,
        shift: PixelPoint,
        native_route: Option<Route>,
        object_coverage_qualified: bool,
    ) -> Result<bool> {
        let mut position = cell
            .screen_origin(self.view)
            .context("fragment projection overflow")?;
        position.x = position
            .x
            .checked_add(shift.x)
            .context("fragment display X")?;
        position.y = position
            .y
            .checked_add(shift.y)
            .context("fragment display Y")?;
        let sequence = self.sequence;
        self.sequence = sequence
            .checked_add(1)
            .context("fragment insertion overflow")?;
        if i64::from(position.x) >= i64::from(self.width)
            || i64::from(position.y) >= i64::from(self.height)
            || i64::from(position.x) + 24 <= 0
            || i64::from(position.y) + 24 <= 0
        {
            return Ok(false);
        }
        ensure!(
            self.fragments.len() < MAX_VISIBLE_FRAGMENTS,
            "visible world fragment budget exceeded"
        );
        self.fragments.try_reserve(1)?;
        self.fragments.push(Fragment {
            key: depth.map_or_else(|| cell.floor_key(), |d| cell.overlay_key(d, sequence)),
            position,
            source,
            owner,
            blend,
            native_route,
            object_coverage_qualified,
        });
        Ok(true)
    }
    #[cfg(test)]
    pub fn tile(
        &mut self,
        cell: NativeDrawCell,
        depth: Option<u8>,
        tile: &'a PreparedTile,
    ) -> Result<bool> {
        self.tile_roof(cell, depth, tile, super::world_roofs::Shade::Native)
    }
    /// A static source may already be indexed in final depth order. Preserve
    /// its insertion identity directly rather than sorting it twice per frame.
    pub fn tile_roof_source(
        &mut self,
        cell: NativeDrawCell,
        depth: Option<u8>,
        tile: &'a PreparedTile,
        shade: super::world_roofs::Shade,
        source: u64,
    ) -> Result<bool> {
        let next = self.sequence;
        self.sequence = source;
        let result = self.tile_roof(cell, depth, tile, shade);
        self.sequence = self.sequence.max(next);
        result
    }
    pub fn tile_roof(
        &mut self,
        cell: NativeDrawCell,
        depth: Option<u8>,
        tile: &'a PreparedTile,
        shade: super::world_roofs::Shade,
    ) -> Result<bool> {
        use super::world_roofs::Shade;
        // 488C34 skips bit8 only in the floor branch. The source decoder still
        // retains 8/9 pixels for inspection; never erase them in the archive.
        if depth.is_none() && (tile.kind() as u8 & 8) != 0 {
            return Ok(false);
        }
        if depth.is_none() {
            ensure!(tile.kind() as u8 &0x30==0 || tile.roof_raw.is_some(),"floor effect lacks qualified raw source");
            return self.push(cell,None,Pixels::Floor(tile,cell.half_x.rem_euclid(2) as u8),0,Blend::Over,None,true);
        }
        if shade == Shade::Hidden {
            return Ok(false);
        }
        ensure!(
            !matches!(shade, Shade::Weight(32..=255)),
            "roof weight outside native table"
        );
        // 483A80 converts TIL bits into object flags in this precedence. Roof
        // transitions then add the separate weight/half flags before dispatch.
        let kind=tile.kind() as u8;
        let flags=if kind&0x10!=0 {8} else if kind&4!=0 {0x100} else {u16::from(kind&0x20)<<2};
        let (flags,weight)=match shade {
            Shade::Weight(w)=>(flags|0x400,w as i8),
            Shade::Half if flags&8==0=>(flags|0x100,0),
            _=>(flags,0),
        };
        let native_route=Some(select_object(ObjectInput {flags,header:tile.object_header.unwrap_or([0;4]),
            visibility:15,focus:true,roof_weight:weight,counter:0}).route);
        // The original object caller interprets the payload as RLE even when
        // a TIL preview decoder accepts a raw diamond. Preserve its header,
        // but do not pretend the raw preview is qualified object coverage.
        let object_coverage_qualified=kind&2!=0;
        if shade == Shade::Native {
            return self.push(cell, depth, tile.pixels(), 0, Blend::Over,native_route,object_coverage_qualified);
        }
        let effect = (tile.kind() as u8 & 0x30) != 0;
        let pixels = if effect {
            let (rgba, rows) = tile
                .roof_raw
                .as_ref()
                .context("roof effect lacks raw coverage")?;
            match &tile.roof_compact {
                Some(packed) => Pixels::CompactTile(packed, rows),
                None => Pixels::Rgba(rgba, rows),
            }
        } else {
            tile.pixels()
        };
        // 488DAD: active weight overrides every effect. Once settled, flag8
        // (source TIL bit0x10) precedes half-opacity; bit0x20 follows it.
        let blend = match shade {
            Shade::Weight(31) if effect || !matches!(tile.kind() as u8, 6 | 7) => Blend::Over,
            Shade::Half if tile.kind() as u8 & 0x10 != 0 => Blend::Screen,
            _ => Blend::Roof(shade),
        };
        self.push(cell, depth, pixels, 0, blend,native_route,object_coverage_qualified)
    }
    pub fn sprite(
        &mut self,
        origin: NativeDrawCell,
        frame: &'a SpriteFrame,
        owner: u16,
        blend: Blend,
    ) -> Result<()> {
        self.sprite_shifted(origin, frame, owner, blend, PixelPoint { x: 0, y: 0 })
    }
    /// Preserve source depth cells while translating the sampled display pose.
    pub fn sprite_shifted(
        &mut self,
        origin: NativeDrawCell,
        frame: &'a SpriteFrame,
        owner: u16,
        blend: Blend,
        shift: PixelPoint,
    ) -> Result<()> {
        let blend=match blend {
            Blend::Actor(super::native_actor_states::Style::Hidden)=>return Ok(()),
            Blend::Actor(super::native_actor_states::Style::Normal)=>Blend::Over,
            other=>other,
        };
        let source = frame
            .fragments
            .as_ref()
            .context("world SPR lacks qualified source fragments")?;
        // 4E0200/4E7000 and qualified effect catalogs pass these source flags.
        // Ownership/click-through does not change the native write route.
        let flags=match blend {
            Blend::Over|Blend::EffectOver=>0,
            Blend::Actor(super::native_actor_states::Style::Normal)=>0,
            Blend::Actor(super::native_actor_states::Style::Half)=>1,
            Blend::Actor(super::native_actor_states::Style::Green(_))=>0x20,
            Blend::Actor(super::native_actor_states::Style::Dim)=>0x200,
            Blend::Actor(super::native_actor_states::Style::Hidden)=>2,
            Blend::Shadow=>5,
            Blend::Screen=>8,
            Blend::Multiply=>0x80,
            Blend::Roof(_)=>anyhow::bail!("SPR submission cannot infer a roof source route"),
        };
        let native_route=Some(prepared_route(flags,0));
        for &reference in &source.references {
            let cell = origin
                .fragment(reference)
                .context("SPR fragment coordinate overflow")?;
            let block = source
                .atlas
                .get(usize::from(reference.block_id))
                .context("SPR fragment block absent")?;
            self.push_shifted(
                cell,
                Some(reference.depth),
                Pixels::Compact(block),
                owner,
                blend,
                shift,
                native_route,
                true,
            )?;
        }
        Ok(())
    }
    /// All geometry and sources have been qualified before mutating canvas.
    /// Covered opaque pixels overwrite ownership; shadows/screen/partial-alpha
    /// terrain retain the visible underlying target. Alpha-zero never hits.
    pub fn paint(
        self,
        canvas: &mut Canvas,
        cancel: &AtomicBool,
        owners: usize,
    ) -> Result<Vec<DrawVisibility>> {
        self.paint_with_background_gate(canvas,cancel,owners,None)
    }
    pub fn paint_with_background_gate(mut self,canvas:&mut Canvas,cancel:&AtomicBool,owners:usize,
        mut gate:Option<&mut super::native_terrain_pixels::Gate>)->Result<Vec<DrawVisibility>> {
        ensure!(gate.as_ref().is_none_or(|g|g.dimensions()==[self.width,self.height]),"terrain gate surface mismatch");
        ensure!(
            owners < u16::MAX as usize
                && canvas.width() == self.width
                && canvas.height() == self.height,
            "raster surface/owner mismatch"
        );
        ensure!(
            self.fragments
                .iter()
                .all(|f| usize::from(f.owner) <= owners),
            "fragment owner outside roster"
        );
        let count = self.width as usize * self.height as usize;
        let mut pixels = Vec::new();
        pixels.try_reserve_exact(count)?;
        pixels.resize(count, 0u16);
        self.fragments.sort_by_key(|f| f.key);
        ensure!(!cancel.load(Ordering::Acquire), "world raster cancelled");
        canvas.clear([18, 25, 35]);
        let mut scratch = [0u8; 24 * 24 * 4];
        let mut floor_normal=[0u8;24*24*4];let mut floor_raw=[0u8;24*24*4];
        for (index, f) in self.fragments.into_iter().enumerate() {
            if index % 128 == 0 {
                ensure!(!cancel.load(Ordering::Acquire), "world raster cancelled");
            }
            let PixelPoint { x, y } = f.position;
            let is_floor=matches!(&f.source,Pixels::Floor(..));
            let (rgba, rows) = match f.source {
                Pixels::Rgba(rgba, rows) => (rgba, Some(rows)),
                Pixels::Compact(block) => {
                    block.write_rgba(&mut scratch);
                    (&scratch[..], None)
                }
                Pixels::CompactTile(block, rows) => {
                    block.write_rgba(&mut scratch);
                    (&scratch[..], Some(rows))
                }
                Pixels::Floor(tile,parity)=>{tile.floor_pixels(parity,&mut scratch,&mut floor_normal,&mut floor_raw)?;(&scratch[..],None)}
            };
            if let Some(gate)=gate.as_mut() {
                if is_floor {gate.mark(rgba,x,y)?;}
                else if !f.object_coverage_qualified {gate.note_unqualified_fragment();}
                else if f.native_route.is_some_and(|r|r.writes_background_gate(303)) {gate.mark(rgba,x,y)?;}
            }
            match f.blend {
                Blend::Over => {
                    if let Some(rows) = rows {
                        canvas.blit_prepared_world_fragment(
                            rgba,
                            rows,
                            x,
                            y,
                            &mut pixels,
                            f.owner,
                        )?;
                    } else {
                        canvas.blit_world_fragment(rgba, x, y, &mut pixels, f.owner)?;
                    }
                }
                Blend::Actor(style) => {
                    canvas.blit_actor_state_fragment(rgba, x, y, &mut pixels, f.owner, style)?
                }
                Blend::EffectOver => canvas.blit_rgba(rgba, 24, 24, x, y)?,
                Blend::Shadow => canvas.blit_native_shadow_rgb555(rgba, 24, 24, x, y)?,
                Blend::Screen => canvas.blit_screen_rgb555(rgba, 24, 24, x, y)?,
                Blend::Multiply => canvas.blit_multiply_rgb555(rgba, 24, 24, x, y)?,
                Blend::Roof(shade) => canvas.blit_roof_rgb555(rgba, x, y, shade)?,
            }
        }
        let pixels: Arc<[u16]> = pixels.into();
        Ok((1..=owners)
            .map(|owner| DrawVisibility {
                owner: owner as u16,
                width: self.width,
                height: self.height,
                pixels: Arc::clone(&pixels),
            })
            .collect())
    }
}
