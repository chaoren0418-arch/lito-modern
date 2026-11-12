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
/// Writer selected for one attachment. Static evidence, unpacked image SHA256
/// 4271D92F…5C2AB6E9 (never executed):
/// - 4E6EA0 copies the attachment sprite's own draw word (`647E68[sprite]`,
///   list.spr attribute 104) to actor+B0+2i next to its bank pointer.
/// - 4E7000 submits each attachment with that word, changed only by the actor
///   word at actor+1A: bit 2 skips the attachment, 0x40 ors 0x40, 0x100 ors 1.
///   Green (0x20), dim (0x200) and a catalogue body flag 1 are not inherited.
/// - 488DAD tests 8 before 0x101 before 0x80, so `8|1` stays the screen
///   writer while `0x80|1` becomes the half writer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum Mix {
    Over,
    Screen,
    /// Attachment word 1: the stable half writer (768894), same as a shadow.
    Half,
    /// Attachment word 0x80: the multiply writer (768898).
    Multiply,
}
impl Mix {
    /// Attachment words outside this set stay unqualified: 12 (bit 4 next to
    /// 8) and the option-gated 0x1000/0x2000 family have no decoded writer
    /// behaviour here.
    pub fn from_native_word(flags: u32) -> Result<Self> {
        Ok(match flags {
            0 => Self::Over,
            1 => Self::Half,
            8 => Self::Screen,
            0x80 => Self::Multiply,
            _ => anyhow::bail!("unsupported layer flags {flags}"),
        })
    }
    pub fn native_word(self) -> u16 {
        match self {
            Self::Over => 0,
            Self::Half => 1,
            Self::Screen => 8,
            Self::Multiply => 0x80,
        }
    }
    /// The writer after 4E7000 (actor word 0x100, the local invisible actor)
    /// or 4CDAF0 (`(word & !3) | 1`, the attribute-100 switch) added bit 1.
    pub fn submitted(self, actor_half: bool) -> Self {
        match self {
            Self::Screen => Self::Screen,
            _ if actor_half => Self::Half,
            other => other,
        }
    }
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
        let cube = super::native_cube::is_graphics(logical);
        if cube { super::native_cube::qualify(c, logical)?; }
        ensure!(matches!(body_flags, 0 | 1) || cube && body_flags == 128,
            "unsupported body flags {body_flags}");
        let spec = |id: u32, shadow: bool| -> Result<Spec> {
            let s = c.sprites.get(&id).context("layer catalog absent")?;
            ensure!(s.header_issue.is_none(), "unqualified layer header");
            let images = s.image_count.context("layer image count absent")?;
            ensure!(images > 0 && images <= 65536, "layer image budget");
            // 4E0233/4E7F03 submit the shadow with `(actor+1A & 0x0FFF) | 5`;
            // the shadow sprite's own attribute 104 is never read for it. A
            // shadow spec keeps the neutral `Over` value; callers always use
            // the shadow writer and never consult this field.
            let mix = if shadow {
                Mix::Over
            } else {
                Mix::from_native_word(scalar(s, 104)?.unwrap_or(0))?
            };
            Ok(Spec {
                logical: id,
                graphics: s.graphics_id,
                images,
                mix,
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
/// Sprite00～15 的分片編號：成員名稱的位元組總和取低 4 bits。
pub fn shard_of(name: &str) -> u8 {
    (name.bytes().map(u32::from).sum::<u32>() & 15) as u8
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
        let shard = shard_of(name);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{
        native_object_dispatch::{select, Input, Route},
        sprite_actions::ActionLimits,
    };

    /// Rows from the pinned client list.spr (SHA256 523C61AD…E8B9F127). The
    /// action lists of 1049, 3387, 3600 and 3908 are omitted; every 101, 102,
    /// 104 and 105 attribute is verbatim.
    const ROWS: &[u8] = b"#902  32      el                  0.walk(1 4,0.0:4 0.1:4 0.2:4 0.3:4) 101.shadow(904) 102.type(12) 105.clothes(1 903)\n\
#903  32      el skirt            102.type(0) 104.attr(1)\n\
#904  32      el s                102.type(0)\n\
#1049 24      cray                105.clothes(1 1051) 101.shadow(1050) 102.type(6) 104.attr(1)\n\
#1050 24      cray s              102.type(0) 104.attr(1)\n\
#1051 24      cray screen         102.type(0) 104.attr(8)\n\
#3387 64 super grimreaper 101.shadow(3388) 102.type(10) 105.clothes(2 3389 3390)\n\
#3388 64 super grimreaper s 102.type(0)\n\
#3389 64 super grimreaper e screen 104.attr(8)\n\
#3390 64 super grimreaper e multi 102.type(0) 104.attr(128)\n\
#3600 1 deep flower all type a 101.shadow(3602) 102.type(0) 105.clothes(1 3601)\n\
#3601 1 deep flower all type a e 102.type(0) 104.attr(8)\n\
#3602 1 deep flower all type a s 102.type(0) 104.attr(256)\n\
#3907 1 diad plane tower e 102.type(0) 104.attr(12)\n\
#3908 1 diad plane tower 102.type(11) 105.clothes(1 3907)\n";

    fn catalog() -> SpriteActionCatalog {
        SpriteActionCatalog::parse_for_inspection(ROWS, ActionLimits::default()).unwrap()
    }
    /// 488D80..488E74 first-pass selection without the see-invisible focus.
    fn route(flags: u16) -> Route {
        select(Input {
            flags,
            header: [0; 4],
            visibility: 15,
            focus: false,
            roof_weight: 0,
            counter: 0,
        })
        .route
    }
    /// Literal 4E7000: the word pushed for one attachment, `None` when the
    /// attachment is not submitted at all.
    fn submitted_word(actor: u16, own: u16) -> Option<u16> {
        if actor & 2 != 0 {
            None
        } else if actor & 0x40 != 0 {
            Some(own | 0x40)
        } else if actor & 0x100 != 0 {
            Some(own | 1)
        } else {
            Some(own)
        }
    }

    #[test]
    fn attachment_words_select_the_writer_of_the_native_dispatch() {
        for (word, mix, expected) in [
            (0u32, Mix::Over, Route::Plain),
            (1, Mix::Half, Route::Half),
            (8, Mix::Screen, Route::Flag8),
            (0x80, Mix::Multiply, Route::Flag80),
        ] {
            assert_eq!(Mix::from_native_word(word).unwrap(), mix);
            assert_eq!(u32::from(mix.native_word()), word);
            assert_eq!(route(mix.native_word()), expected);
            assert_eq!(mix.submitted(false), mix);
        }
        // Bit 4, hidden, the option-gated family and combined words have no
        // qualified attachment writer and are refused, never approximated.
        for word in [2, 4, 5, 9, 12, 0x40, 0x81, 0x100, 0x1008, 0x2001, 99, 0x1_0000] {
            assert!(Mix::from_native_word(word).is_err(), "{word}");
        }
    }

    #[test]
    fn only_the_half_state_reaches_an_attachment_as_in_4e7000_and_4cdaf0() {
        // Actor words: neutral, catalogue body flag 1, poison green, dim, the
        // local invisible actor, and the latter while poisoned.
        for actor in [0u16, 1, 0x20, 0x200, 0x100, 0x120] {
            let actor_half = actor & 0x100 != 0;
            for mix in [Mix::Over, Mix::Half, Mix::Screen, Mix::Multiply] {
                let native = submitted_word(actor, mix.native_word()).unwrap();
                assert_eq!(
                    route(native),
                    route(mix.submitted(actor_half).native_word()),
                    "{actor:#x} {mix:?}"
                );
            }
        }
        // The attribute-100 switch rewrites already submitted fragments with
        // `(word & !3) | 1` (4CDAF0), which selects the same writers.
        for mix in [Mix::Over, Mix::Half, Mix::Screen, Mix::Multiply] {
            assert_eq!(
                route((mix.native_word() & !3) | 1),
                route(mix.submitted(true).native_word()),
                "{mix:?}"
            );
        }
        assert_eq!(Mix::Multiply.submitted(true), Mix::Half);
        assert_eq!(Mix::Screen.submitted(true), Mix::Screen);
        // Hidden actors submit no attachment; another player's invisible
        // actor (0x40) is skipped by the dispatch without the focus option.
        for own in [0u16, 1, 8, 0x80] {
            assert_eq!(submitted_word(2, own), None);
            assert_eq!(submitted_word(0x102, own), None);
            assert_eq!(route(submitted_word(0x40, own).unwrap()), Route::Skip);
        }
        // The shadow word is `(actor & 0x0FFF) | 5`: every non-hidden state of
        // a body whose own word is 0 or 1 keeps the stable half writer.
        for actor in [0u16, 1, 0x20, 0x200, 0x100, 0x120] {
            assert_eq!(route((actor & 0x0fff) | 5), Route::Half, "{actor:#x}");
        }
        assert_eq!(route((2 & 0x0fff) | 5), Route::Skip);
    }

    #[test]
    fn plans_read_the_attachment_word_and_never_the_shadow_sprite_word() {
        let c = catalog();
        let el = Plan::from_catalog(&c, 902).unwrap();
        assert!(!el.body_half);
        assert_eq!(el.shadow.as_ref().unwrap().graphics, 904);
        assert_eq!(
            el.attachments.iter().map(|a| (a.graphics, a.mix)).collect::<Vec<_>>(),
            [(903, Mix::Half)]
        );
        let reaper = Plan::from_catalog(&c, 3387).unwrap();
        assert_eq!(
            reaper.attachments.iter().map(|a| (a.graphics, a.mix)).collect::<Vec<_>>(),
            [(3389, Mix::Screen), (3390, Mix::Multiply)]
        );
        let bound = reaper.bind(63).unwrap();
        assert_eq!(bound.shadow.as_deref(), Some("3388-63.spr"));
        assert_eq!(
            bound.attachments,
            [
                ("3389-63.spr".to_string(), Mix::Screen),
                ("3390-63.spr".to_string(), Mix::Multiply)
            ]
        );
        assert!(reaper.bind(64).is_err());
        // 1050 carries 104.attr(1) and 3602 carries 104.attr(256). Neither is
        // read for a shadow; the body word alone decides (4E0233).
        let cray = Plan::from_catalog(&c, 1049).unwrap();
        assert!(cray.body_half);
        assert_eq!(cray.shadow.as_ref().unwrap().graphics, 1050);
        assert_eq!(cray.attachments[0].mix, Mix::Screen);
        let flower = Plan::from_catalog(&c, 3600).unwrap();
        assert_eq!(flower.shadow.as_ref().unwrap().graphics, 3602);
        assert_eq!(flower.attachments[0].mix, Mix::Screen);
        // The same sprite used as an attachment is still qualified by its word.
        let error = Plan::from_catalog(&c, 3908).unwrap_err();
        assert!(format!("{error:#}").contains("unsupported layer flags 12"));
    }
}
