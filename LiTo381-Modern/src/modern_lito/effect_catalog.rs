//! Explicit Lineage effect-ID -> status-picture mapping. This is not a skill,
//! graphic/GFX lookup, duration grant, or permission to read a LiTo archive.
//! Plain XML and the existing Lineage XML AES container share the same parser.
use super::resources::{open_readonly, reject_reparse_chain, verify_open_path};
use aes::{
    cipher::{BlockDecrypt, KeyInit},
    Aes128,
};
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    io::Read,
    path::{Component, Path},
    sync::atomic::{AtomicBool, Ordering},
};

pub const MAX_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_EFFECTS: u32 = 8192;
// Same resource-format constants as 客戶端xml/解密XML.py, not a server secret.
const KEY: [u8; 16] = [
    0xdc, 0x84, 1, 0x21, 0x2a, 0x40, 0x20, 0x0a, 0xdd, 0x25, 0xb9, 0xa7, 0x0d, 0xb9, 0xc9, 0x4e,
];
const IV: [u8; 16] = [
    62, 9, 120, 0xaa, 0xc4, 0xd5, 48, 99, 48, 12, 95, 0x9a, 0x80, 127, 34, 70,
];
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "effect catalog cancelled");
    Ok(())
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EffectId(u32);
impl EffectId {
    pub fn from_wire(value: i64) -> Result<Self> {
        ensure!(
            (0..i64::from(MAX_EFFECTS)).contains(&value),
            "effect ID outside catalog limit: {value}"
        );
        Ok(Self(value as u32))
    }
    pub fn value(self) -> u32 {
        self.0
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mapping {
    pub effect: EffectId,
    pub icon: u32,
    pub off_icon: Option<u32>,
    pub graphic: Option<u32>,
}
#[derive(Clone, Debug)]
pub struct EffectCatalog {
    entries: BTreeMap<EffectId, Mapping>,
    source_sha256: [u8; 32],
    xml_sha256: [u8; 32],
    encrypted: bool,
}
impl EffectCatalog {
    /// Caller supplies the exact XML path; never search parents, cwd, or TW
    /// settings, and never export decrypted data over the original resource.
    pub fn load(path: &Path, cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        ensure!(
            path.is_absolute()
                && !path
                    .components()
                    .any(|c| matches!(c, Component::ParentDir | Component::CurDir)),
            "effect catalog needs an absolute normalized path"
        );
        #[cfg(windows)]
        ensure!(
            matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), std::path::Prefix::Disk(_))),
            "effect catalog needs a local drive path"
        );
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            #[link(name = "kernel32")]
            unsafe extern "system" {
                fn GetDriveTypeW(root: *const u16) -> u32;
            }
            let root = path
                .ancestors()
                .last()
                .context("effect catalog root absent")?;
            let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
            ensure!(
                unsafe { GetDriveTypeW(wide.as_ptr()) } != 4,
                "network effect catalog is forbidden"
            );
        }
        reject_reparse_chain(path)?;
        let mut guards = Vec::new();
        for parent in path
            .ancestors()
            .skip(1)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
        {
            let handle = open_readonly(parent, true)?;
            verify_open_path(&handle, parent, true)?;
            guards.push((parent, handle));
        }
        let mut file = open_readonly(path, false)?;
        verify_open_path(&file, path, false)?;
        ensure!(
            file.metadata()?.len() <= MAX_BYTES as u64,
            "effect XML exceeds byte limit"
        );
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        active(cancel)?;
        verify_open_path(&file, path, false)?;
        for (path, handle) in &guards {
            verify_open_path(handle, path, true)?;
        }
        Self::parse(&bytes, cancel)
    }
    pub fn parse(bytes: &[u8], cancel: &AtomicBool) -> Result<Self> {
        active(cancel)?;
        ensure!(
            !bytes.is_empty() && bytes.len() <= MAX_BYTES,
            "effect XML byte limit"
        );
        let encrypted = bytes.starts_with(b"X?xm");
        let mut xml = bytes.to_vec();
        if encrypted {
            let aes = Aes128::new_from_slice(&KEY).unwrap();
            let mut previous = IV;
            let mut chunks = xml[4..].chunks_exact_mut(16);
            for block in &mut chunks {
                active(cancel)?;
                let cipher: [u8; 16] = block.try_into().unwrap();
                let mut plain = cipher.into();
                aes.decrypt_block(&mut plain);
                for i in 0..16 {
                    block[i] = plain[i] ^ previous[i];
                }
                previous = cipher;
            }
            for (byte, key) in chunks.into_remainder().iter_mut().zip(previous) {
                *byte ^= key;
            }
            xml[0] = b'<';
        }
        let text = std::str::from_utf8(&xml).context("effect XML is not UTF-8")?;
        // roxmltree also disables entity resolution. Reject even an empty DTD,
        // whose default parser policy otherwise permits it.
        ensure!(!text.contains("<!DOCTYPE"), "effect XML DTD is forbidden");
        let document = roxmltree::Document::parse_with_options(
            text,
            roxmltree::ParsingOptions {
                allow_dtd: false,
                nodes_limit: MAX_EFFECTS * 4 + 16,
                entity_resolver: None,
            },
        )
        .context("invalid effect XML")?;
        let root = document.root_element();
        ensure!(
            root.tag_name().name() == "effectlist" && root.tag_name().namespace().is_none(),
            "effectlist root required"
        );
        ensure!(
            root.attributes().len() == 1,
            "unexpected effectlist attributes"
        );
        let count = number(
            root.attribute("count").context("effect count absent")?,
            MAX_EFFECTS,
        )?;
        ensure!(count > 0, "empty effect catalog");
        let mut entries = BTreeMap::new();
        for node in root.children() {
            active(cancel)?;
            if node.is_comment() || node.is_text() && node.text().unwrap_or("").trim().is_empty() {
                continue;
            }
            ensure!(
                node.is_element()
                    && node.tag_name().name() == "effect"
                    && node.tag_name().namespace().is_none(),
                "unexpected effectlist child"
            );
            ensure!(
                node.attributes().len() <= 32 && node.attributes().all(|a| a.namespace().is_none()),
                "invalid effect attributes"
            );
            ensure!(
                node.children()
                    .all(|n| n.is_text() && n.text().unwrap_or("").trim().is_empty()),
                "nested effect content"
            );
            let id = number(node.attribute("id").context("effect ID absent")?, count - 1)?;
            let mapping = Mapping {
                effect: EffectId(id),
                icon: number(
                    node.attribute("icon").context("effect icon absent")?,
                    999_999,
                )?,
                off_icon: node
                    .attribute("offIcon")
                    .map(|s| number(s, 999_999))
                    .transpose()?,
                graphic: node
                    .attribute("graphic")
                    .map(|s| number(s, u32::MAX))
                    .transpose()?,
            };
            ensure!(
                entries.insert(mapping.effect, mapping).is_none(),
                "duplicate effect ID {id}"
            );
        }
        ensure!(
            entries.len() == count as usize,
            "effect count does not match rows"
        );
        active(cancel)?;
        Ok(Self {
            entries,
            source_sha256: Sha256::digest(bytes).into(),
            xml_sha256: Sha256::digest(&xml).into(),
            encrypted,
        })
    }
    pub fn get(&self, id: EffectId) -> Result<&Mapping> {
        self.entries
            .get(&id)
            .with_context(|| format!("effect ID {} absent", id.0))
    }
    pub fn entries(&self) -> impl Iterator<Item = &Mapping> {
        self.entries.values()
    }
    pub fn source_sha256(&self) -> [u8; 32] {
        self.source_sha256
    }
    pub fn xml_sha256(&self) -> [u8; 32] {
        self.xml_sha256
    }
    pub fn encrypted(&self) -> bool {
        self.encrypted
    }
}
fn number(text: &str, maximum: u32) -> Result<u32> {
    ensure!(
        !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()),
        "effect numeric field is not unsigned decimal"
    );
    let value: u32 = text.parse().context("effect numeric overflow")?;
    ensure!(value <= maximum, "effect numeric value exceeds limit");
    Ok(value)
}

#[cfg(test)]
#[path = "effect_catalog_tests.rs"]
mod tests;
