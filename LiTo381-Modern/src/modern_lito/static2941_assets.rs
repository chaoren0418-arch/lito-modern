//! Static SPR resource proof. No Animation, sound, FPS or fabricated lifetime.
use super::*;

struct Spec {
    name: &'static str,
    size: usize,
    sha: [u8; 32],
    header: (u16, u16, u32),
    references: &'static [(i8, i8, u8, u16)],
}
const PRIMARY: Spec = Spec {
    name: "2941-0.spr",
    size: 865,
    sha: [
        0xe1, 0xa8, 0x42, 0x86, 0x5e, 0x4c, 0xc1, 0xb6, 0x1b, 0x1a, 0x6a, 0x9b, 0x43, 0xcd, 0x45,
        0x1d, 0xb9, 0x7a, 0xef, 0x51, 0x5c, 0xbe, 0x6f, 0x5f, 0xaf, 0x9a, 0x74, 0xce, 0x92, 0x2c,
        0xa0, 0x80,
    ],
    header: (0xff00, 0x0101, 920),
    references: &[(1, -1, 6, 0), (0, 0, 6, 1), (1, 0, 6, 2), (0, 1, 6, 3)],
};
const FALLBACK: Spec = Spec {
    name: "1120-0.spr",
    size: 211,
    sha: [
        0x13, 0x02, 0xf1, 0xd0, 0xc6, 0xe1, 0x63, 0x6c, 0x75, 0x21, 0x07, 0x26, 0xf3, 0x02, 0xb8,
        0xb8, 0x6f, 0x18, 0xf7, 0xd3, 0x65, 0xa1, 0xf8, 0xd1, 0xc4, 0xa5, 0x2d, 0xb3, 0xc1, 0x91,
        0x29, 0xc6,
    ],
    header: (0, 1, 178),
    references: &[(0, 0, 20, 0), (1, 0, 20, 1)],
};

/// Only this loader can create a bank. Holding an Arc proves the exact decoded
/// frame and its original depth/atlas are still available when enqueue occurs.
pub(crate) struct Static2941Bank {
    root: PathBuf,
    sprite: LoadedSprite,
    fallback: bool,
}
impl Static2941Bank {
    pub(crate) fn load(catalog: &SharedActionCatalog, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        let row = catalog
            .catalog()
            .sprites
            .get(&2941)
            .ok_or(HonorAssetError::Unsupported(
                "static2941 definition missing",
            ))?;
        if catalog.sha256() != LIST_SHA
            || row.id != 2941
            || row.graphics_id != 2941
            || row.image_count != Some(1)
            || row.header_issue.is_some()
            || !row.actions.is_empty()
            || row.attributes.len() != 1
            || row.attributes[0].key != ActionKey::new(102)
            || row.attributes[0].body_bytes.trim_ascii() != b"9"
        {
            return Err(HonorAssetError::Unsupported(
                "static2941 catalog/type9 changed",
            ));
        }
        let mut reader = SpriteReader::open(catalog.root(), catalog.index_records(), cancel)?;
        let (sprite, fallback) =
            Self::primary_or_fallback(|spec| Self::read(&mut reader, spec, cancel))?;
        active(cancel)?;
        Ok(Self {
            root: catalog.root().to_owned(),
            sprite,
            fallback,
        })
    }

    fn primary_or_fallback<T>(mut read: impl FnMut(&Spec) -> Result<T>) -> Result<(T, bool)> {
        match read(&PRIMARY) {
            Ok(sprite) => Ok((sprite, false)),
            Err(HonorAssetError::Resource(ResourceError::MissingEntry(_))) => {
                Ok((read(&FALLBACK)?, true))
            }
            Err(error) => Err(error),
        }
    }

    fn read(reader: &mut SpriteReader, spec: &Spec, cancel: &AtomicBool) -> Result<LoadedSprite> {
        active(cancel)?;
        let archive = reader.archive_for(spec.name, cancel)?;
        if archive.entry(spec.name)?.size as usize != spec.size {
            return Err(HonorAssetError::Unsupported("static2941 SPR size changed"));
        }
        let kind = archive.kind();
        let bytes = archive.read_entry(spec.name)?;
        active(cancel)?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        if bytes.len() != spec.size || digest != spec.sha {
            return Err(HonorAssetError::Unsupported(
                "static2941 SPR digest changed",
            ));
        }
        let decoded = decode_world_spr_with_limits(
            &bytes,
            SpriteLimits {
                max_input_bytes: 865,
                max_frames: 1,
                max_blocks: 4,
                max_block_references: 4,
                max_frame_dimension: 96,
                max_total_pixels: 8192,
            },
        )?;
        active(cancel)?;
        if !decoded.has_palette
            || decoded.frames.len() != 1
            || decoded.block_table_metadata != spec.header.2
        {
            return Err(HonorAssetError::Unsupported(
                "static2941 SPR frame shape changed",
            ));
        }
        let frame = &decoded.frames[0];
        let fragments = frame
            .fragments
            .as_ref()
            .ok_or(HonorAssetError::Unsupported(
                "static2941 SPR fragments missing",
            ))?;
        if (frame.unknown1, frame.unknown2) != (spec.header.0, spec.header.1)
            || fragments.atlas.len() != spec.references.len()
            || fragments.references.len() != spec.references.len()
            || fragments
                .references
                .iter()
                .zip(spec.references)
                .any(|(a, b)| (a.half_x, a.y, a.depth, a.block_id) != *b)
        {
            return Err(HonorAssetError::Unsupported(
                "static2941 SPR fragments changed",
            ));
        }
        Ok(LoadedSprite {
            provenance: SpriteProvenance {
                archive: kind,
                name: spec.name.to_owned(),
                bytes: spec.size as u32,
                sha256: digest,
            },
            decoded,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }
    pub(crate) fn frame_zero(&self) -> &SpriteFrame {
        &self.sprite.decoded.frames[0]
    }
    pub(crate) fn provenance(&self) -> &SpriteProvenance {
        &self.sprite.provenance
    }
    pub(crate) fn used_missing_spr_fallback(&self) -> bool {
        self.fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fallback_only_for_missing_entry_not_corruption_or_cancel() {
        let mut names = vec![];
        let (_, fallback) = Static2941Bank::primary_or_fallback(|spec| {
            names.push(spec.name);
            if spec.name == PRIMARY.name {
                Err(HonorAssetError::Resource(ResourceError::MissingEntry(
                    spec.name.into(),
                )))
            } else {
                Ok(())
            }
        })
        .unwrap();
        assert!(fallback);
        assert_eq!(names, [PRIMARY.name, FALLBACK.name]);
        for error in [
            HonorAssetError::Cancelled,
            HonorAssetError::Unsupported("digest changed"),
            HonorAssetError::Resource(ResourceError::Io(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "archive unreadable",
            ))),
            HonorAssetError::Resource(ResourceError::MalformedIndex("bad index")),
        ] {
            let mut error = Some(error);
            let mut calls = 0;
            let result: Result<((), bool)> = Static2941Bank::primary_or_fallback(|_| {
                calls += 1;
                Err(error.take().expect("must not try fallback"))
            });
            assert!(result.is_err());
            assert_eq!(calls, 1);
        }
    }
}
