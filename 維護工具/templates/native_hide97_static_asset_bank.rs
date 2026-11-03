// #2941 has no animation. Keep its static frame outside SharedEffectBank and
// Clip: no fabricated ActionFrame, FPS, terminal dwell or status-duration loop.
// Resource acquisition is worker-only, through the existing same-root archive
// resolver. Scene/S37/History and global callback admission are separate work.
struct Static2941Spec {
    name: &'static str,
    size: usize,
    sha: [u8; 32],
    unknown1: u16,
    unknown2: u16,
    block_metadata: u32,
    references: &'static [(i8, i8, u8, u16)],
}
const STATIC2941: Static2941Spec = Static2941Spec {
    name: "2941-0.spr", size: 865,
    sha: [0xe1,0xa8,0x42,0x86,0x5e,0x4c,0xc1,0xb6,0x1b,0x1a,0x6a,0x9b,0x43,0xcd,0x45,0x1d,
          0xb9,0x7a,0xef,0x51,0x5c,0xbe,0x6f,0x5f,0xaf,0x9a,0x74,0xce,0x92,0x2c,0xa0,0x80],
    unknown1: 0xff00, unknown2: 0x0101, block_metadata: 920,
    references: &[(1,-1,6,0),(0,0,6,1),(1,0,6,2),(0,1,6,3)],
};
const STATIC2941_MISSING_SPR: Static2941Spec = Static2941Spec {
    name: "1120-0.spr", size: 211,
    sha: [0x13,0x02,0xf1,0xd0,0xc6,0xe1,0x63,0x6c,0x75,0x21,0x07,0x26,0xf3,0x02,0xb8,0xb8,
          0x6f,0x18,0xf7,0xd3,0x65,0xa1,0xf8,0xd1,0xc4,0xa5,0x2d,0xb3,0xc1,0x91,0x29,0xc6],
    unknown1: 0, unknown2: 1, block_metadata: 178,
    references: &[(0,0,20,0),(1,0,20,1)],
};

/// Immutable raw-static resource bank, not a certificate that #97 was drawn.
/// Owns neither timer nor authority; selecting a frame never advances time.
pub(crate) struct Static2941Bank {
    root: PathBuf,
    catalog_sha: [u8; 32],
    sprite: LoadedSprite,
    used_missing_spr_fallback: bool,
}
impl Static2941Bank {
    pub(crate) fn load(context: &SharedActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        if context.sha256() != LIST_SHA {
            return Err(HonorAssetError::Unsupported("static2941 original catalog changed"));
        }
        let row = context.catalog().sprites.get(&2941)
            .ok_or(HonorAssetError::Unsupported("static2941 definition absent"))?;
        if row.id != 2941 || row.graphics_id != 2941 || row.image_count != Some(1)
            || row.header_issue.is_some() || !row.actions.is_empty() || row.attributes.len() != 1
            || row.attributes[0].key != ActionKey::new(102)
            || row.attributes[0].body_bytes.trim_ascii() != b"9"
        {
            return Err(HonorAssetError::Unsupported("static2941 no-action type9 changed"));
        }
        let mut reader = SpriteReader::open(context.root(), context.index_records(), cancel)?;
        let (sprite, used_missing_spr_fallback) = match Self::read_exact(&mut reader, &STATIC2941, cancel) {
            Ok(sprite) => (sprite, false),
            // Native 4E62DD..4E6355 has an explicit missing-SPR fallback for
            // a nonplayer effect. Malformed, changed, cancelled or unreadable
            // archives are errors, never an excuse to substitute graphics1120.
            Err(HonorAssetError::Resource(ResourceError::MissingEntry(_))) =>
                (Self::read_exact(&mut reader, &STATIC2941_MISSING_SPR, cancel)?, true),
            Err(error) => return Err(error),
        };
        active(cancel)?;
        Ok(Self { root: context.root().to_owned(), catalog_sha: context.sha256(),
                  sprite, used_missing_spr_fallback })
    }

    fn read_exact(reader: &mut SpriteReader, spec: &Static2941Spec, cancel: &AtomicBool) -> Result<LoadedSprite> {
        active(cancel)?;
        let archive = reader.archive_for(spec.name, cancel)?;
        if archive.entry(spec.name)?.size as usize != spec.size {
            return Err(HonorAssetError::Unsupported("static2941 original member size changed"));
        }
        let kind = archive.kind();
        let bytes = archive.read_entry(spec.name)?;
        active(cancel)?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        if bytes.len() != spec.size || digest != spec.sha {
            return Err(HonorAssetError::Unsupported("static2941 original member digest changed"));
        }
        let decoded = decode_world_spr_with_limits(&bytes, SpriteLimits {
            max_input_bytes: 865, max_frames: 1, max_blocks: 4, max_block_references: 4,
            max_frame_dimension: 96, max_total_pixels: 8192,
        })?;
        active(cancel)?;
        if !decoded.has_palette || decoded.frames.len() != 1
            || decoded.block_table_metadata != spec.block_metadata
        {
            return Err(HonorAssetError::Unsupported("static2941 original frame shape changed"));
        }
        let frame = &decoded.frames[0];
        let fragments = frame.fragments.as_ref()
            .ok_or(HonorAssetError::Unsupported("static2941 source fragments absent"))?;
        if frame.unknown1 != spec.unknown1 || frame.unknown2 != spec.unknown2
            || fragments.references.len() != spec.references.len()
            || fragments.atlas.len() != spec.references.len()
            || fragments.references.iter().zip(spec.references).any(|(a,b)|
                (a.half_x,a.y,a.depth,a.block_id) != *b)
        {
            return Err(HonorAssetError::Unsupported("static2941 original fragments changed"));
        }
        Ok(LoadedSprite {
            provenance: SpriteProvenance { archive: kind, name: spec.name.to_owned(),
                                           bytes: spec.size as u32, sha256: digest }, decoded,
        })
    }

    pub(crate) fn frame_zero(&self) -> &SpriteFrame { &self.sprite.decoded.frames[0] }
    pub(crate) fn provenance(&self) -> &SpriteProvenance { &self.sprite.provenance }
    pub(crate) fn root(&self) -> &Path { &self.root }
    pub(crate) fn catalog_sha256(&self) -> [u8;32] { self.catalog_sha }
    pub(crate) fn used_missing_spr_fallback(&self) -> bool { self.used_missing_spr_fallback }
}
