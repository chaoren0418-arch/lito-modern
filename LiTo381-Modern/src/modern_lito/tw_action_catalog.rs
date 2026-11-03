//! Explicit shared TW action-data acquisition for the autonomous client.
//!
//! The existing TW encoder writes S + action text, optionally zlib compressed,
//! XORs every byte and AES-ECB encrypts only complete blocks, then appends the
//! existing content-binding tag. That tag is compatibility/integrity metadata,
//! NOT a trusted signature (its key is public in this client).
//! No native process, injection, variant stripping, lossy text conversion,
//! loose TXT fallback, catalog merging, LT archive or mutation is performed.
//! Loading this catalog alone does not authorize a skill or qualify playback.
use super::{
    resources,
    sprite_actions::{ActionLimits, SpriteActionCatalog},
};
use aes::{
    cipher::{BlockDecrypt, KeyInit},
    Aes128,
};
use anyhow::{ensure, Context, Result};
use flate2::{Decompress, FlushDecompress, Status};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};

const MAX_DECODED: usize = 8 * 1024 * 1024;
const XOR_TABLE: [u8; 256] = [
    0x7E, 0x89, 0xDC, 0x78, 0x7F, 0x4B, 0xB6, 0x4F, 0x7D, 0x0D, 0x08, 0x16, 0x7C, 0xCF, 0x62, 0x21,
    0x79, 0x80, 0x74, 0xA4, 0x78, 0x42, 0x1E, 0x93, 0x7A, 0x04, 0xA0, 0xCA, 0x7B, 0xC6, 0xCA, 0xFD,
    0x6C, 0xBC, 0x2E, 0xB0, 0x6D, 0x7E, 0x44, 0x87, 0x6F, 0x38, 0xFA, 0xDE, 0x6E, 0xFA, 0x90, 0xE9,
    0x6B, 0xB5, 0x86, 0x6C, 0x6A, 0x77, 0xEC, 0x5B, 0x68, 0x31, 0x52, 0x02, 0x69, 0xF3, 0x38, 0x35,
    0x62, 0xAF, 0x7F, 0x08, 0x63, 0x6D, 0x15, 0x3F, 0x61, 0x2B, 0xAB, 0x66, 0x60, 0xE9, 0xC1, 0x51,
    0x65, 0xA6, 0xD7, 0xD4, 0x64, 0x64, 0xBD, 0xE3, 0x66, 0x22, 0x03, 0xBA, 0x67, 0xE0, 0x69, 0x8D,
    0x48, 0xD7, 0xCB, 0x20, 0x49, 0x15, 0xA1, 0x17, 0x4B, 0x53, 0x1F, 0x4E, 0x4A, 0x91, 0x75, 0x79,
    0x4F, 0xDE, 0x63, 0xFC, 0x4E, 0x1C, 0x09, 0xCB, 0x4C, 0x5A, 0xB7, 0x92, 0x4D, 0x98, 0xDD, 0xA5,
    0x46, 0xC4, 0x9A, 0x98, 0x47, 0x06, 0xF0, 0xAF, 0x45, 0x40, 0x4E, 0xF6, 0x44, 0x82, 0x24, 0xC1,
    0x41, 0xCD, 0x32, 0x44, 0x40, 0x0F, 0x58, 0x73, 0x42, 0x49, 0xE6, 0x2A, 0x43, 0x8B, 0x8C, 0x1D,
    0x54, 0xF1, 0x68, 0x50, 0x55, 0x33, 0x02, 0x67, 0x57, 0x75, 0xBC, 0x3E, 0x56, 0xB7, 0xD6, 0x09,
    0x53, 0xF8, 0xC0, 0x8C, 0x52, 0x3A, 0xAA, 0xBB, 0x50, 0x7C, 0x14, 0xE2, 0x51, 0xBE, 0x7E, 0xD5,
    0x5A, 0xE2, 0x39, 0xE8, 0x5B, 0x20, 0x53, 0xDF, 0x59, 0x66, 0xED, 0x86, 0x58, 0xA4, 0x87, 0xB1,
    0x5D, 0xEB, 0x91, 0x34, 0x5C, 0x29, 0xFB, 0x03, 0x5E, 0x6F, 0x45, 0x5A, 0x5F, 0xAD, 0x2F, 0x6D,
    0xE1, 0x35, 0x1B, 0x80, 0xE0, 0xF7, 0x71, 0xB7, 0xE2, 0xB1, 0xCF, 0xEE, 0xE3, 0x73, 0xA5, 0xD9,
    0xE6, 0x3C, 0xB3, 0x5C, 0xE7, 0xFE, 0xD9, 0x6B, 0xE5, 0xB8, 0x67, 0x32, 0xE4, 0x7A, 0x0D, 0x05,
];

pub struct SharedTwActionCatalog {
    catalog: SpriteActionCatalog,
    pack_sha256: [u8; 32],
    text_sha256: [u8; 32],
    encoded_bytes: usize,
    decoded_bytes: usize,
    definitions: Vec<DefinitionSource>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionSource {
    pub sprite_id: u32,
    pub byte_offset: usize,
    pub byte_length: usize,
    pub line: usize,
    pub sha256: [u8; 32],
}
impl SharedTwActionCatalog {
    /// Explicit per-feature selection, never a replacement of the base catalog.
    /// The actual TW file has ambiguous legacy definitions (e.g. #790's two
    /// action8 entries). Those cannot confer first/last-wins semantics on new
    /// features. Selected definitions must be unique and pass the existing
    /// strict action/alias parser. Opaque control dependencies (e.g. clothes105)
    /// must be explicitly selected and validated by the feature consumer.
    pub fn load_selected(
        root: &Path,
        selected: &BTreeSet<u32>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_using(root, selected, cancel, resources::read_shared_tw_action_pack)
    }
    /// Select the current shared launcher's authenticated pack explicitly.
    pub(crate) fn load_selected_from_launcher(
        root: &Path,
        selected: &BTreeSet<u32>,
        cancel: &AtomicBool,
    ) -> Result<Self> {
        Self::load_using(root, selected, cancel, resources::read_shared_launcher_tw_action_pack)
    }
    fn load_using(
        root: &Path,
        selected: &BTreeSet<u32>,
        cancel: &AtomicBool,
        read: fn(&Path) -> std::result::Result<Vec<u8>, resources::ResourceError>,
    ) -> Result<Self> {
        ensure!(
            !selected.is_empty() && selected.len() <= 256,
            "TW action selection budget"
        );
        active(cancel)?;
        let raw = read(root)?;
        let pack_sha256 = Sha256::digest(&raw).into();
        let text = decode(&raw, cancel)?;
        active(cancel)?;
        let text_sha256 = Sha256::digest(&text).into();
        let (catalog, definitions) = parse_selected(&text, selected, cancel)?;
        active(cancel)?;
        Ok(Self {
            catalog,
            pack_sha256,
            text_sha256,
            encoded_bytes: raw.len(),
            decoded_bytes: text.len(),
            definitions,
        })
    }
    pub fn catalog(&self) -> &SpriteActionCatalog {
        &self.catalog
    }
    pub fn pack_sha256(&self) -> [u8; 32] {
        self.pack_sha256
    }
    pub fn text_sha256(&self) -> [u8; 32] {
        self.text_sha256
    }
    pub fn encoded_bytes(&self) -> usize {
        self.encoded_bytes
    }
    pub fn decoded_bytes(&self) -> usize {
        self.decoded_bytes
    }
    pub fn definitions(&self) -> &[DefinitionSource] {
        &self.definitions
    }
}

fn parse_selected(
    text: &[u8],
    selected: &BTreeSet<u32>,
    cancel: &AtomicBool,
) -> Result<(SpriteActionCatalog, Vec<DefinitionSource>)> {
    ensure!(
        !selected.is_empty() && selected.len() <= 256 && text.len() <= MAX_DECODED,
        "TW action selection budget"
    );
    let mut found = BTreeMap::<u32, DefinitionSource>::new();
    let mut current = None::<(u32, usize, usize)>;
    let mut offset = 0usize;
    for (line, raw) in text.split_inclusive(|&b| b == b'\n').enumerate() {
        if line % 1024 == 0 {
            active(cancel)?;
        }
        ensure!(raw.len() <= 256 * 1024, "TW action line budget");
        let data = raw.trim_ascii_start();
        if data.starts_with(b"#") {
            if let Some((id, start, first_line)) = current.take() {
                let source = DefinitionSource {
                    sprite_id: id,
                    byte_offset: start,
                    byte_length: offset - start,
                    line: first_line,
                    sha256: Sha256::digest(&text[start..offset]).into(),
                };
                ensure!(
                    found.insert(id, source).is_none(),
                    "duplicate selected TW sprite {id}"
                );
            }
            let n = data[1..].iter().take_while(|b| b.is_ascii_digit()).count();
            let id = std::str::from_utf8(&data[1..1 + n])
                .ok()
                .and_then(|s| s.parse::<u32>().ok());
            if let Some(id) = id.filter(|id| selected.contains(id)) {
                // A lookalike #11746garbage cannot hide a second selected ID.
                ensure!(
                    data.get(1 + n)
                        .is_none_or(|b| b.is_ascii_whitespace() || *b == b'='),
                    "malformed selected TW header"
                );
                current = Some((id, offset, line + 1));
            }
        }
        offset += raw.len();
    }
    if let Some((id, start, line)) = current {
        let source = DefinitionSource {
            sprite_id: id,
            byte_offset: start,
            byte_length: text.len() - start,
            line,
            sha256: Sha256::digest(&text[start..]).into(),
        };
        ensure!(
            found.insert(id, source).is_none(),
            "duplicate selected TW sprite {id}"
        );
    }
    ensure!(
        found.len() == selected.len(),
        "selected TW definitions absent"
    );
    let definitions: Vec<_> = found.into_values().collect();
    let total: usize = definitions.iter().map(|s| s.byte_length).sum();
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(total + definitions.len())
        .context("TW selection allocation")?;
    for source in &definitions {
        bytes.extend_from_slice(&text[source.byte_offset..source.byte_offset + source.byte_length]);
        bytes.push(b'\n');
    }
    let catalog = SpriteActionCatalog::parse_for_inspection(
        &bytes,
        ActionLimits {
            max_bytes: MAX_DECODED + 256,
            max_sprites: 256,
            ..Default::default()
        },
    )?;
    active(cancel)?;
    Ok((catalog, definitions))
}
fn active(cancel: &AtomicBool) -> Result<()> {
    ensure!(
        !cancel.load(Ordering::Acquire),
        "TW action loading cancelled"
    );
    Ok(())
}

fn decode(raw: &[u8], cancel: &AtomicBool) -> Result<Vec<u8>> {
    active(cancel)?;
    ensure!(
        (37..=resources::MAX_SHARED_TW_ACTION_PACK_BYTES as usize).contains(&raw.len()),
        "TW action pack size"
    );
    let expected = u32::from_le_bytes(raw[..4].try_into().unwrap()) as usize;
    ensure!(
        (2..=MAX_DECODED + 1).contains(&expected),
        "TW action decoded size"
    );
    let end = raw.len() - crate::morph_auth::MORPH_MAC_LEN;
    let tag = crate::morph_auth::morph_mac(&raw[..end]);
    let difference = tag
        .iter()
        .zip(&raw[end..])
        .fold(0u8, |d, (a, b)| d | (a ^ b));
    ensure!(difference == 0, "TW action content binding mismatch");
    active(cancel)?;
    let key: [u8; 16] = raw[4..20].try_into().unwrap();
    let cipher = Aes128::new(&key.into());
    let mut payload = Vec::new();
    payload
        .try_reserve_exact(end - 20)
        .context("TW action allocation")?;
    payload.extend_from_slice(&raw[20..end]);
    for (i, chunk) in payload.chunks_exact_mut(16).enumerate() {
        if i % 4096 == 0 {
            active(cancel)?;
        }
        cipher.decrypt_block(aes::Block::from_mut_slice(chunk));
    }
    for (i, byte) in payload.iter_mut().enumerate() {
        *byte ^= XOR_TABLE[i % 256] ^ key[i % 16];
    }
    let mut output = if payload.first() == Some(&b'S') {
        ensure!(payload.len() == expected, "TW raw action length mismatch");
        payload
    } else {
        let mut output = Vec::new();
        output
            .try_reserve_exact(expected + 1)
            .context("TW action decompression allocation")?;
        output.resize(expected + 1, 0);
        let mut decoder = Decompress::new(true);
        let mut input = 0usize;
        let mut written = 0usize;
        loop {
            active(cancel)?;
            let to = (written + 64 * 1024).min(output.len());
            let status = decoder
                .decompress(
                    &payload[input..],
                    &mut output[written..to],
                    FlushDecompress::None,
                )
                .context("TW action invalid zlib")?;
            let used = decoder.total_in() as usize;
            let produced = decoder.total_out() as usize;
            ensure!(
                produced <= expected,
                "TW action decompression exceeds declared length"
            );
            if status == Status::StreamEnd {
                ensure!(
                    used == payload.len() && produced == expected,
                    "TW action trailing input or length mismatch"
                );
                output.truncate(produced);
                break;
            }
            ensure!(
                used > input || produced > written,
                "TW action truncated zlib"
            );
            input = used;
            written = produced;
        }
        output
    };
    active(cancel)?;
    ensure!(output.first() == Some(&b'S'), "TW action prefix absent");
    output.remove(0);
    ensure!(!output.contains(&0), "TW action NUL in text");
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes::cipher::BlockEncrypt;
    use std::io::Write;
    fn wrap(payload: &[u8], expected: usize) -> Vec<u8> {
        let key = std::array::from_fn::<_, 16, _>(|i| i as u8);
        let mut body = payload.to_vec();
        for (i, b) in body.iter_mut().enumerate() {
            *b ^= XOR_TABLE[i % 256] ^ key[i % 16];
        }
        let cipher = Aes128::new(&key.into());
        for chunk in body.chunks_exact_mut(16) {
            cipher.encrypt_block(aes::Block::from_mut_slice(chunk));
        }
        let mut bytes = (expected as u32).to_le_bytes().to_vec();
        bytes.extend(key);
        bytes.extend(body);
        bytes.extend(crate::morph_auth::morph_mac(&bytes));
        bytes
    }
    fn compressed(bytes: &[u8]) -> Vec<u8> {
        let mut writer =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        writer.write_all(bytes).unwrap();
        writer.finish().unwrap()
    }
    #[test]
    fn raw_and_zlib_preserve_bytes_and_aes_partial_block_tails() {
        let cancel = AtomicBool::new(false);
        for n in 1..=520 {
            let mut plain = vec![b'S'];
            plain.extend((0..n).map(|i| (i % 255 + 1) as u8));
            for body in [plain.clone(), compressed(&plain)] {
                assert_eq!(
                    decode(&wrap(&body, plain.len()), &cancel).unwrap(),
                    plain[1..]
                );
            }
        }
    }
    #[test]
    fn bad_tag_header_truncation_and_unsigned_data_never_fall_back() {
        let plain = b"S#1 1 test\n0.a(0 1,0.0:1)\n";
        for body in [plain.to_vec(), compressed(plain)] {
            let bytes = wrap(&body, plain.len());
            for end in 0..bytes.len() {
                assert!(decode(&bytes[..end], &AtomicBool::new(false)).is_err());
            }
            for i in 0..bytes.len() {
                let mut bad = bytes.clone();
                bad[i] ^= 1;
                assert!(decode(&bad, &AtomicBool::new(false)).is_err());
            }
        }
        for size in [0, 1, MAX_DECODED + 2, u32::MAX as usize] {
            assert!(decode(&wrap(plain, size), &AtomicBool::new(false)).is_err());
        }
        assert!(decode(
            &vec![0; resources::MAX_SHARED_TW_ACTION_PACK_BYTES as usize + 1],
            &AtomicBool::new(false)
        )
        .is_err());
    }
    #[test]
    fn signed_invalid_zlib_length_trailing_streams_and_nuls_are_rejected() {
        let plain = b"S#1 1 test\n0.a(0 1,0.0:1)\n";
        let compressed = compressed(plain);
        let cancel = AtomicBool::new(false);
        for body in [plain.to_vec(), compressed.clone()] {
            for size in [plain.len() - 1, plain.len() + 1] {
                assert!(decode(&wrap(&body, size), &cancel).is_err());
            }
        }
        for n in 0..compressed.len() {
            assert!(decode(&wrap(&compressed[..n], plain.len()), &cancel).is_err());
        }
        for suffix in [vec![0], compressed.clone()] {
            let mut body = compressed.clone();
            body.extend(suffix);
            assert!(decode(&wrap(&body, plain.len()), &cancel).is_err());
        }
        for plain in [b"X#1".as_slice(), b"S#1\0"] {
            assert!(decode(&wrap(&self::compressed(plain), plain.len()), &cancel).is_err());
        }
    }
    #[test]
    fn selected_catalog_keeps_original_spans_and_never_merges_ambiguous_definitions() {
        let bytes=b"30000 0 0\n#9 1 ignored\n8=7\n8.d(0 1,0.0:1)\n#1 1 \xa4\xff\n0.a(0 1,0.0:2[717)\n#2 1 overlay\n104.attr(128)\n";
        let cancel = AtomicBool::new(false);
        let (catalog, spans) = parse_selected(bytes, &[1, 2].into(), &cancel).unwrap();
        assert_eq!(catalog.sprites.len(), 2);
        assert_eq!(spans[0].line, 5);
        assert_eq!(catalog.sprites[&1].label_bytes, b"\xa4\xff");
        for s in &spans {
            assert_eq!(
                s.sha256.as_slice(),
                Sha256::digest(&bytes[s.byte_offset..s.byte_offset + s.byte_length]).as_slice()
            );
        }
        assert!(parse_selected(bytes, &[9].into(), &cancel).is_err());
        assert!(parse_selected(bytes, &[3].into(), &cancel).is_err());
        assert!(parse_selected(bytes, &BTreeSet::new(), &cancel).is_err());
        assert!(parse_selected(bytes, &(0..257).collect(), &cancel).is_err());
        for source in [
            b"#1 1 a\n0.a(0 1,0.0:2)\n#1 1 b\n".as_slice(),
            b"#1 1 a\n0=2\n#2 1 b\n0.b(0 1,0.0:2)\n",
            b"#1garbage\n0.a(0 1,0.0:2)\n",
        ] {
            assert!(parse_selected(source, &[1].into(), &cancel).is_err());
        }
        assert!(parse_selected(&vec![b'a'; 256 * 1024 + 1], &[1].into(), &cancel).is_err());
    }
    #[test]
    fn cancellation_precedes_io_and_validation_never_opens_unselected_sources() {
        let root = Path::new("not-an-existing-resource-root");
        assert!(format!(
            "{:#}",
            SharedTwActionCatalog::load_selected(root, &[1].into(), &AtomicBool::new(true))
                .err()
                .unwrap()
        )
        .contains("cancelled"));
        assert!(format!(
            "{:#}",
            SharedTwActionCatalog::load_selected(root, &BTreeSet::new(), &AtomicBool::new(false))
                .err()
                .unwrap()
        )
        .contains("selection budget"));
        assert!(parse_selected(b"#1 1\n", &[1].into(), &AtomicBool::new(true)).is_err());
    }
    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact shared TW pack, no native process or writes"]
    fn shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback() {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let c = SharedTwActionCatalog::load_selected(
            &root,
            &[11746, 11747, 11748, 11749].into(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(c.encoded_bytes(), 1_402_755);
        assert_eq!(c.decoded_bytes(), 5_974_928);
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(resources::read_shared_tw_action_pack(&root).unwrap())
            ),
            "fc7b4cac77379a98e2a2edf3f645bb82ca8be9ee3f2b7fae76fcc61423ee8798"
        );
        assert_eq!(
            c.text_sha256().as_slice(),
            Sha256::digest(std::fs::read(root.join("TW13081901.txt")).unwrap()).as_slice()
        );
        assert_eq!(
            hex(&c.text_sha256()),
            "7ebe2b464c8b1ea9c58ac985dd8ff4c823784d8231894262eddcd7691ca94008"
        );
        for (s, (offset, length, hash)) in c.definitions().iter().zip([
            (
                4_637_702,
                228,
                "b95662177231c3591878d301451f6c01a5c282ecb720fda21afdf7309087c74d",
            ),
            (
                4_637_930,
                64,
                "881c96c51b21491b5b642088a7dbf8ca518a2f95cf0916e11aeab8d5313d9194",
            ),
            (
                4_637_994,
                235,
                "0f6c0a7548202b9f795722e0b58bd96a8fdc86b88b172ab8aa11c21ffe0e9698",
            ),
            (
                4_638_229,
                71,
                "a3348667a673674665d0799ed1a60333b7fa72a47c3375f7ec53e6c540818f32",
            ),
        ]) {
            assert_eq!((s.byte_offset, s.byte_length), (offset, length));
            assert_eq!(hex(&s.sha256), hash);
        }
        for id in [11746, 11748] {
            let action = c
                .catalog()
                .resolve_action(id, super::super::sprite_actions::ActionKey::new(0))
                .unwrap();
            assert_eq!(action.animation.frames.len(), 18);
            assert_eq!(action.animation.duration_ratio(), (60, 24));
        }
        for id in [11747, 11749] {
            assert!(c.catalog().sprites.contains_key(&id));
        }
    }

    #[test]
    #[ignore = "LITO_SHARED_RESOURCE_ROOT; exact current launcher TW pack and unchanged plaintext/definitions; no root-pack fallback"]
    fn shared_tw_pack_has_real_ultimate_definitions_without_base_catalog_fallback_current_launcher_tw_profile() {
        let root = std::path::PathBuf::from(
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("shared root"),
        );
        let c = SharedTwActionCatalog::load_selected_from_launcher(
            &root,
            &[11746, 11747, 11748, 11749].into(),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(c.encoded_bytes(), 1_402_755);
        assert_eq!(c.decoded_bytes(), 5_974_928);
        assert_eq!(
            format!(
                "{:x}",
                Sha256::digest(resources::read_shared_launcher_tw_action_pack(&root).unwrap())
            ),
            "b3788df4c7f077ef3ff15b070a5d020ccb224e48dec3757c4cd34528fca4b218"
        );
        assert_eq!(
            c.text_sha256().as_slice(),
            Sha256::digest(std::fs::read(root.join("TW13081901.txt")).unwrap()).as_slice()
        );
        assert_eq!(
            hex(&c.text_sha256()),
            "7ebe2b464c8b1ea9c58ac985dd8ff4c823784d8231894262eddcd7691ca94008"
        );
        for (s, (offset, length, hash)) in c.definitions().iter().zip([
            (
                4_637_702,
                228,
                "b95662177231c3591878d301451f6c01a5c282ecb720fda21afdf7309087c74d",
            ),
            (
                4_637_930,
                64,
                "881c96c51b21491b5b642088a7dbf8ca518a2f95cf0916e11aeab8d5313d9194",
            ),
            (
                4_637_994,
                235,
                "0f6c0a7548202b9f795722e0b58bd96a8fdc86b88b172ab8aa11c21ffe0e9698",
            ),
            (
                4_638_229,
                71,
                "a3348667a673674665d0799ed1a60333b7fa72a47c3375f7ec53e6c540818f32",
            ),
        ]) {
            assert_eq!((s.byte_offset, s.byte_length), (offset, length));
            assert_eq!(hex(&s.sha256), hash);
        }
        for id in [11746, 11748] {
            let action = c
                .catalog()
                .resolve_action(id, super::super::sprite_actions::ActionKey::new(0))
                .unwrap();
            assert_eq!(action.animation.frames.len(), 18);
            assert_eq!(action.animation.duration_ratio(), (60, 24));
        }
        for id in [11747, 11749] {
            assert!(c.catalog().sprites.contains_key(&id));
        }
    }
    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn owned_loose_fixture_never_uses_txt_nested_launcher_or_lt_fallback() {
        use std::{
            fs,
            path::PathBuf,
            time::{SystemTime, UNIX_EPOCH},
        };
        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                for dir in [
                    self.0.join("LTtext"),
                    self.0.join("launcher"),
                    self.0.clone(),
                ] {
                    for name in ["TW13081901.pak", "TW13081901.txt"] {
                        let _ = fs::remove_file(dir.join(name));
                    }
                    let _ = fs::remove_dir(dir);
                }
            }
        }
        let path = std::env::temp_dir().join(format!(
            "modern-tw-action-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        let fixture = Fixture(path);
        let plain = b"S#1 1 test\n0.a(0 1,0.0:1)\n";
        let valid = wrap(plain, plain.len());
        fs::write(fixture.0.join("TW13081901.txt"), &plain[1..]).unwrap();
        fs::create_dir(fixture.0.join("launcher")).unwrap();
        fs::write(fixture.0.join("launcher/TW13081901.pak"), &valid).unwrap();
        let load = |root: &Path| {
            SharedTwActionCatalog::load_selected(root, &[1].into(), &AtomicBool::new(false))
        };
        assert!(load(&fixture.0).is_err());
        let pack = fixture.0.join("TW13081901.pak");
        fs::write(&pack, b"corrupt").unwrap();
        assert!(load(&fixture.0).is_err());
        fs::write(&pack, &valid).unwrap();
        let c = load(&fixture.0).unwrap();
        assert_eq!(c.catalog().sprites.len(), 1);
        // No source handle retained after publication.
        fs::remove_file(&pack).unwrap();
        assert!(c.catalog().sprites.contains_key(&1));
        fs::File::create(&pack)
            .unwrap()
            .set_len(resources::MAX_SHARED_TW_ACTION_PACK_BYTES + 1)
            .unwrap();
        assert!(load(&fixture.0).is_err());
        fs::create_dir(fixture.0.join("LTtext")).unwrap();
        fs::write(fixture.0.join("LTtext/TW13081901.pak"), valid).unwrap();
        assert!(load(&fixture.0.join("LTtext")).is_err());
    }
}
