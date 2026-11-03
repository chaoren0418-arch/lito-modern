//! Body-linked catalog101 shadows and catalog105 attachments; no layer clocks.
use super::{
    resources::{ResourceError, SharedArchive, SharedArchiveKind},
    sprite::{self, DecodedSprite, SpriteFrame},
    sprite_actions::{ActionKey, SpriteActionCatalog, SpriteActions},
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
};
fn active(c: &AtomicBool) -> Result<()> {
    ensure!(!c.load(Ordering::Acquire), "actor layer load cancelled");
    Ok(())
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Mix {
    Over,
    Screen,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Spec {
    pub logical: u32,
    pub graphics: u32,
    pub images: u32,
    pub mix: Mix,
}
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Plan {
    pub body_half: bool,
    pub shadow: Option<Spec>,
    pub attachments: Vec<Spec>,
}
fn values(s: &SpriteActions, id: u32) -> Result<Option<Vec<u32>>> {
    let mut found = s.attributes.iter().filter(|a| a.key.id == id);
    let Some(a) = found.next() else {
        return Ok(None);
    };
    ensure!(
        a.key == ActionKey::new(id) && found.next().is_none(),
        "ambiguous layer attribute {id}"
    );
    let text = std::str::from_utf8(&a.body_bytes).context("non-ASCII layer attribute")?;
    let out: Vec<u32> = text
        .split_ascii_whitespace()
        .map(str::parse)
        .collect::<std::result::Result<_, _>>()?;
    ensure!(out.len() <= 9, "actor layer attribute budget");
    Ok(Some(out))
}
fn scalar(s: &SpriteActions, id: u32) -> Result<Option<u32>> {
    values(s, id)?
        .map(|v| {
            ensure!(v.len() == 1, "layer scalar arity");
            Ok(v[0])
        })
        .transpose()
}
impl Plan {
    pub fn from_catalog(c: &SpriteActionCatalog, logical: u32) -> Result<Self> {
        let row = c
            .sprites
            .get(&logical)
            .context("body layer catalog absent")?;
        ensure!(row.header_issue.is_none(), "unqualified body layer header");
        let body_flags = scalar(row, 104)?.unwrap_or(0);
        ensure!(matches!(body_flags, 0 | 1), "unsupported body flags {body_flags}");
        let spec = |id: u32, shadow: bool| -> Result<Spec> {
            let s = c.sprites.get(&id).context("layer catalog absent")?;
            ensure!(s.header_issue.is_none(), "unqualified layer header");
            let images = s.image_count.context("layer image count absent")?;
            ensure!(images > 0 && images <= 65536, "layer image budget");
            let flags = scalar(s, 104)?.unwrap_or(0);
            ensure!(
                (shadow && flags == 0) || (!shadow && matches!(flags, 0 | 8)),
                "unsupported layer flags {flags}"
            );
            Ok(Spec {
                logical: id,
                graphics: s.graphics_id,
                images,
                mix: if flags == 8 { Mix::Screen } else { Mix::Over },
            })
        };
        let shadow = scalar(row, 101)?.map(|id| spec(id, true)).transpose()?;
        let mut attachments = Vec::new();
        if let Some(v) = values(row, 105)? {
            ensure!(
                !v.is_empty() && v[0] <= 8 && v.len() == v[0] as usize + 1,
                "attachment count/arity"
            );
            for id in &v[1..] {
                attachments.push(spec(*id, false)?);
            }
        }
        Ok(Self {
            body_half: body_flags == 1,
            shadow,
            attachments,
        })
    }
    pub fn bind(&self, image: u32) -> Result<Bindings> {
        let member = |s: &Spec| -> Result<String> {
            ensure!(image < s.images, "body image exceeds layer bank");
            Ok(format!("{}-{image}.spr", s.graphics))
        };
        Ok(Bindings {
            body_half: self.body_half,
            shadow: self.shadow.as_ref().map(member).transpose()?,
            attachments: self
                .attachments
                .iter()
                .map(|s| Ok((member(s)?, s.mix)))
                .collect::<Result<_>>()?,
        })
    }
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Bindings {
    pub body_half: bool,
    pub shadow: Option<String>,
    pub attachments: Vec<(String, Mix)>,
}
#[derive(Clone, Debug, Default, serde::Serialize)]
pub struct Prepared {
    pub bindings: Bindings,
    pub missing_attachments: Vec<String>,
    pub missing_shadow: Option<String>,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Source {
    pub archive: String,
    pub member: String,
    pub bytes: usize,
    pub sha256: String,
}
pub struct Loaded {
    pub source: Source,
    pub sprite: DecodedSprite,
}
pub struct Bank {
    root: PathBuf,
    base: SharedArchive,
    shards: BTreeMap<u8, SharedArchive>,
    entries: BTreeMap<String, Loaded>,
    records: usize,
    bytes: usize,
}
impl Bank {
    pub fn open(root: &Path) -> Result<Self> {
        let base = SharedArchive::open(root, SharedArchiveKind::Sprite)?;
        let records = base.len();
        Ok(Self {
            root: root.to_path_buf(),
            base,
            shards: BTreeMap::new(),
            entries: BTreeMap::new(),
            records,
            bytes: 0,
        })
    }
    pub fn bytes(&self) -> usize {
        self.bytes
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn get(&self, name: &str) -> Result<&Loaded> {
        self.entries.get(name).context("actor sprite not loaded")
    }
    pub fn frame(&self, name: &str, sub: usize) -> Result<&SpriteFrame> {
        self.get(name)?
            .sprite
            .frames
            .get(sub)
            .context("body-linked subframe absent")
    }
    pub fn ensure(&mut self, name: &str, cancel: &AtomicBool) -> Result<()> {
        active(cancel)?;
        if self.entries.contains_key(name) {
            return Ok(());
        }
        ensure!(self.entries.len() < 1024, "actor member cache budget");
        let shard = (name.bytes().map(u32::from).sum::<u32>() & 15) as u8;
        if !self.shards.contains_key(&shard) {
            let a = SharedArchive::open(&self.root, SharedArchiveKind::SpriteShard(shard))?;
            let records = self
                .records
                .checked_add(a.len())
                .context("actor index overflow")?;
            ensure!(records <= 1_000_000, "actor index budget");
            self.shards.insert(shard, a);
            self.records = records;
        }
        let selected = self.shards.get_mut(&shard).unwrap();
        let (archive, kind) = match selected.entry(name) {
            Ok(_) => (selected, format!("Sprite{shard:02}")),
            Err(ResourceError::MissingEntry(_)) => (&mut self.base, "Sprite".to_string()),
            Err(e) => return Err(e.into()),
        };
        ensure!(
            archive.entry(name)?.size <= 8 << 20,
            "actor SPR byte budget"
        );
        let raw = archive.read_entry(name)?;
        active(cancel)?;
        let sprite = sprite::decode_world_spr_with_limits(
            &raw,
            sprite::SpriteLimits {
                max_input_bytes: 8 << 20,
                max_total_pixels: 4 * 1024 * 1024,
                ..Default::default()
            },
        )?;
        let bytes = self
            .bytes
            .checked_add(
                sprite
                    .retained_rgba_bytes()
                    .context("actor retained bytes overflow")?,
            )
            .context("actor cache overflow")?;
        ensure!(bytes <= 64 << 20, "actor body/layer cache budget");
        active(cancel)?;
        let source = Source {
            archive: kind,
            member: name.to_string(),
            bytes: raw.len(),
            sha256: format!("{:X}", Sha256::digest(&raw)),
        };
        self.entries
            .insert(name.to_string(), Loaded { source, sprite });
        self.bytes = bytes;
        Ok(())
    }
    pub fn prepare(&mut self, b: &Bindings, sub: usize, cancel: &AtomicBool) -> Result<Prepared> {
        let mut out = Prepared {
            bindings: b.clone(),
            missing_attachments: Vec::new(),
            missing_shadow: None,
        };
        for (name, _) in &b.attachments {
            if let Err(e) = self.ensure(name, cancel) {
                if matches!(
                    e.downcast_ref::<ResourceError>(),
                    Some(ResourceError::MissingEntry(_))
                ) {
                    out.missing_attachments.push(name.clone());
                    continue;
                }
                return Err(e);
            }
            ensure!(
                self.frame(name, sub)?.fragments.is_some(),
                "actor attachment lacks qualified fragments"
            );
        }
        // Original attachment loader disables the complete group if any bank is absent.
        if !out.missing_attachments.is_empty() {
            out.bindings.attachments.clear();
        }
        if let Some(name) = &b.shadow {
            match self.ensure(name, cancel) {
                Ok(()) => ensure!(
                    self.frame(name, sub)?.fragments.is_some(),
                    "actor shadow lacks qualified fragments"
                ),
                Err(e)
                    if matches!(
                        e.downcast_ref::<ResourceError>(),
                        Some(ResourceError::MissingEntry(_))
                    ) =>
                {
                    out.missing_shadow = Some(name.clone());
                    out.bindings.shadow = None;
                }
                Err(e) => return Err(e),
            }
        }
        active(cancel)?;
        Ok(out)
    }
}
