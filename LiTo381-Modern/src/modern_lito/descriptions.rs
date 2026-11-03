//! Shared Lineage `Text.pak/desc-c.tbl` names, not LTdesc or an effective-stat source.
//!
//! Evidence: the existing PakViewer OldL1Handler/PakTools decoded the selected
//! shared table to 249300 bytes (golden below), strictly readable as Windows
//! CP950. Header 17001 precedes 16945 CRLF-separated, zero-based names; empty
//! records keep their slots, and the last record has no terminator.
//! `LiTo381-開發/ida-client-oep-v2/full-disassembly.asm` sub_4D4170,
//! loc_4D4393..loc_4D43BE (lines 443697..443734), stores sequential line pointers
//! and replaces the declared count with the actual count. sub_4D44A0
//! (lines 443900..444199) consumes `$` + decimal digits + optional `;`, copying
//! surrounding literal bytes. Its runtime `%i` substitution and >=20000 LT
//! fallback are deliberately NOT implemented here. Current shared entries
//! contain no `$` references; unknown recursive/formatting semantics are errors.
//!
//! This module never uses legacy_text's lossy/automatic encoding selection.
//! Windows CP950 is selected explicitly, with invalid-byte rejection. Existing
//! archive open/read/decrypt operations are synchronous and not interruptible;
//! cancellation is cooperative around those operations and between records.
//! Call load on an owned resource worker, never a UI or heartbeat thread.

use super::resources::{ResourceError, SharedArchive, SharedArchiveKind};
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

pub const ENTRY_NAME: &str = "desc-c.tbl";
pub const MAX_TABLE_BYTES: usize = 2 * 1024 * 1024;
pub const MAX_DECLARED_ENTRIES: u32 = 100_000;
pub const MAX_LINE_BYTES: usize = 1024;
pub const MAX_INPUT_BYTES: usize = 256;
pub const MAX_RESOLVED_BYTES: usize = 512;
pub const MAX_REFERENCES: usize = 128;
const MAX_CATALOG_UTF8_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnsupportedDescription {
    RuntimeTemplate,
    InlineFormatting,
    EmbeddedReference,
}

#[derive(Debug)]
pub enum DescriptionError {
    Cancelled,
    Resource(ResourceError),
    UnsupportedPlatform,
    TableTooLarge,
    InvalidHeader,
    UnsupportedTableFormat,
    NoEntries,
    ExtraEntries,
    InvalidLineEnding {
        line: usize,
    },
    LineTooLong {
        line: usize,
    },
    InvalidEncoding {
        line: usize,
    },
    ControlCharacter {
        line: usize,
    },
    CatalogTextTooLarge,
    AllocationFailed,
    InvalidNameToken,
    NameTokenTooLong,
    InvalidReference {
        byte_offset: usize,
    },
    ReferenceOverflow {
        byte_offset: usize,
    },
    TooManyReferences,
    OutsideDeclaredRange {
        id: u32,
        declared: u32,
    },
    MissingEntry {
        id: u32,
        available: usize,
        declared: u32,
    },
    EmptyEntry {
        id: u32,
    },
    UnsupportedEntry {
        id: u32,
        reason: UnsupportedDescription,
    },
    ResolvedNameTooLong,
}

impl fmt::Display for DescriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // No original account/item token or full table content in diagnostics.
        match self {
            Self::Resource(error) => write!(f, "shared description resource: {error}"),
            Self::Cancelled => f.write_str("shared description loading cancelled"),
            other => write!(f, "shared description: {other:?}"),
        }
    }
}
impl std::error::Error for DescriptionError {}
impl From<ResourceError> for DescriptionError {
    fn from(error: ResourceError) -> Self {
        Self::Resource(error)
    }
}

/// Immutable, explicit source-token/result pair; never silently returns a raw
/// unresolved token as though it were a successfully translated name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedName {
    source_token: String,
    text: String,
    reference_ids: Vec<u32>,
}
impl ResolvedName {
    pub fn source_token(&self) -> &str {
        &self.source_token
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    /// Appearance order, including repeated references; no deduplication/reindex.
    pub fn reference_ids(&self) -> &[u32] {
        &self.reference_ids
    }
}
impl fmt::Display for ResolvedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Header and physical record counts are separate. Missing declared suffix
/// records and empty present records are NOT filled, dropped, or shifted.
pub struct DescriptionCatalog {
    declared: u32,
    entries: Vec<String>,
}
impl fmt::Debug for DescriptionCatalog {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DescriptionCatalog")
            .field("declared", &self.declared)
            .field("available", &self.entries.len())
            .finish_non_exhaustive()
    }
}
impl DescriptionCatalog {
    pub fn load_shared(root: impl AsRef<Path>) -> Result<Self, DescriptionError> {
        Self::load_shared_cancellable(root, &AtomicBool::new(false))
    }

    /// Fixed, archive-qualified member through the confined read-only handles.
    /// Cancellation never detaches this owner or promises to interrupt File I/O.
    pub fn load_shared_cancellable(
        root: impl AsRef<Path>,
        cancelled: &AtomicBool,
    ) -> Result<Self, DescriptionError> {
        check_cancelled(cancelled)?;
        let mut archive = SharedArchive::open(root, SharedArchiveKind::Text)?;
        check_cancelled(cancelled)?;
        if u64::from(archive.entry(ENTRY_NAME)?.size) > MAX_TABLE_BYTES as u64 {
            return Err(DescriptionError::TableTooLarge);
        }
        let bytes = archive.read_entry(ENTRY_NAME)?;
        check_cancelled(cancelled)?;
        Self::parse_cancellable(&bytes, cancelled)
    }

    pub fn parse_cp950(bytes: &[u8]) -> Result<Self, DescriptionError> {
        Self::parse_cancellable(bytes, &AtomicBool::new(false))
    }

    pub(crate) fn parse_cancellable(bytes: &[u8], cancelled: &AtomicBool) -> Result<Self, DescriptionError> {
        check_cancelled(cancelled)?;
        if bytes.len() > MAX_TABLE_BYTES {
            return Err(DescriptionError::TableTooLarge);
        }
        let (header, mut offset) = next_line(bytes, 0, 1)?;
        if offset == bytes.len() && !bytes.ends_with(b"\r\n") {
            return Err(DescriptionError::InvalidHeader);
        }
        if header.is_empty() || header.len() > 6 || !header.iter().all(u8::is_ascii_digit) {
            return Err(DescriptionError::InvalidHeader);
        }
        let mut declared = 0u32;
        for digit in header {
            declared = declared
                .checked_mul(10)
                .and_then(|value| value.checked_add(u32::from(digit - b'0')))
                .ok_or(DescriptionError::InvalidHeader)?;
        }
        if declared == 0 || declared > MAX_DECLARED_ENTRIES {
            return Err(DescriptionError::InvalidHeader);
        }
        let mut entries = Vec::new();
        let mut text_bytes = 0usize;
        while offset < bytes.len() {
            check_cancelled(cancelled)?;
            if entries.len() >= declared as usize {
                return Err(DescriptionError::ExtraEntries);
            }
            let line = entries.len() + 2;
            let (raw, next) = next_line(bytes, offset, line)?;
            if entries.is_empty() && raw.starts_with(b"#") {
                return Err(DescriptionError::UnsupportedTableFormat);
            }
            if raw.len() > MAX_LINE_BYTES {
                return Err(DescriptionError::LineTooLong { line });
            }
            if raw.iter().any(|byte| *byte < 32 || *byte == 127) {
                return Err(DescriptionError::ControlCharacter { line });
            }
            let decoded = decode_cp950(raw, line)?;
            check_cancelled(cancelled)?;
            if decoded.chars().any(char::is_control) {
                return Err(DescriptionError::ControlCharacter { line });
            }
            text_bytes = text_bytes
                .checked_add(decoded.len())
                .ok_or(DescriptionError::CatalogTextTooLarge)?;
            if text_bytes > MAX_CATALOG_UTF8_BYTES {
                return Err(DescriptionError::CatalogTextTooLarge);
            }
            entries
                .try_reserve(1)
                .map_err(|_| DescriptionError::AllocationFailed)?;
            entries.push(decoded);
            offset = next;
        }
        if entries.is_empty() {
            return Err(DescriptionError::NoEntries);
        }
        check_cancelled(cancelled)?;
        Ok(Self { declared, entries })
    }

    pub fn declared_count(&self) -> u32 {
        self.declared
    }
    pub fn available_count(&self) -> usize {
        self.entries.len()
    }
    pub fn is_complete(&self) -> bool {
        self.entries.len() == self.declared as usize
    }

    /// Raw CP950-decoded line at the exact ID. Call resolve_name for display:
    /// an existing empty/template/formatting entry is not a resolved item name.
    pub fn entry(&self, id: u32) -> Result<&str, DescriptionError> {
        if id >= self.declared {
            return Err(DescriptionError::OutsideDeclaredRange {
                id,
                declared: self.declared,
            });
        }
        self.entries
            .get(id as usize)
            .map(String::as_str)
            .ok_or(DescriptionError::MissingEntry {
                id,
                available: self.entries.len(),
                declared: self.declared,
            })
    }

    /// Decimal references may be embedded in literal text, have leading zeroes,
    /// and end in one optional semicolon. No hexadecimal, subtraction, lookup
    /// in LT archives, recursive expansion or runtime printf state is consulted.
    pub fn resolve_name(&self, token: &str) -> Result<ResolvedName, DescriptionError> {
        if token.len() > MAX_INPUT_BYTES {
            return Err(DescriptionError::NameTokenTooLong);
        }
        if token.trim().is_empty() || token.chars().any(char::is_control) {
            return Err(DescriptionError::InvalidNameToken);
        }
        let bytes = token.as_bytes();
        let mut text = String::new();
        let mut references = Vec::new();
        let mut offset = 0;
        let mut literal = 0;
        while offset < bytes.len() {
            if bytes[offset] != b'$' {
                offset += 1;
                continue;
            }
            append_bounded(&mut text, &token[literal..offset])?;
            let start = offset;
            offset += 1;
            if !bytes.get(offset).is_some_and(u8::is_ascii_digit) {
                return Err(DescriptionError::InvalidReference { byte_offset: start });
            }
            let mut id = 0u32;
            while let Some(digit) = bytes.get(offset).filter(|digit| digit.is_ascii_digit()) {
                id = id
                    .checked_mul(10)
                    .and_then(|value| value.checked_add(u32::from(*digit - b'0')))
                    .ok_or(DescriptionError::ReferenceOverflow { byte_offset: start })?;
                offset += 1;
            }
            if bytes.get(offset) == Some(&b';') {
                offset += 1;
            }
            if references.len() == MAX_REFERENCES {
                return Err(DescriptionError::TooManyReferences);
            }
            let entry = self.entry(id)?;
            if entry.trim().is_empty() {
                return Err(DescriptionError::EmptyEntry { id });
            }
            let unsupported = if entry.contains('$') {
                Some(UnsupportedDescription::EmbeddedReference)
            } else if entry.contains('\\') {
                Some(UnsupportedDescription::InlineFormatting)
            } else if entry
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'%' && pair[1].is_ascii_alphabetic())
            {
                Some(UnsupportedDescription::RuntimeTemplate)
            } else {
                None
            };
            if let Some(reason) = unsupported {
                return Err(DescriptionError::UnsupportedEntry { id, reason });
            }
            append_bounded(&mut text, entry)?;
            references.push(id);
            literal = offset;
        }
        append_bounded(&mut text, &token[literal..])?;
        Ok(ResolvedName {
            source_token: token.to_owned(),
            text,
            reference_ids: references,
        })
    }
}

fn append_bounded(output: &mut String, value: &str) -> Result<(), DescriptionError> {
    if output
        .len()
        .checked_add(value.len())
        .is_none_or(|length| length > MAX_RESOLVED_BYTES)
    {
        return Err(DescriptionError::ResolvedNameTooLong);
    }
    output
        .try_reserve(value.len())
        .map_err(|_| DescriptionError::AllocationFailed)?;
    output.push_str(value);
    Ok(())
}
fn check_cancelled(cancelled: &AtomicBool) -> Result<(), DescriptionError> {
    if cancelled.load(Ordering::Acquire) {
        Err(DescriptionError::Cancelled)
    } else {
        Ok(())
    }
}
fn next_line(bytes: &[u8], offset: usize, line: usize) -> Result<(&[u8], usize), DescriptionError> {
    let end = bytes[offset..]
        .iter()
        .position(|byte| *byte == b'\r' || *byte == b'\n')
        .map_or(bytes.len(), |position| offset + position);
    if end == bytes.len() {
        return Ok((&bytes[offset..end], end));
    }
    if bytes.get(end..end + 2) != Some(&b"\r\n"[..]) {
        return Err(DescriptionError::InvalidLineEnding { line });
    }
    Ok((&bytes[offset..end], end + 2))
}

#[cfg(windows)]
fn decode_cp950(bytes: &[u8], line: usize) -> Result<String, DescriptionError> {
    use windows::Win32::Globalization::{MultiByteToWideChar, MB_ERR_INVALID_CHARS};
    if bytes.is_empty() {
        return Ok(String::new());
    }
    // CP950 consumes at least one byte per UTF-16 code unit; the source line's
    // fixed bound suffices, avoiding an unbounded API-reported allocation.
    let mut wide = Vec::new();
    wide.try_reserve_exact(bytes.len())
        .map_err(|_| DescriptionError::AllocationFailed)?;
    wide.resize(bytes.len(), 0u16);
    let count = unsafe { MultiByteToWideChar(950, MB_ERR_INVALID_CHARS, bytes, Some(&mut wide)) };
    if count <= 0 || count as usize > wide.len() {
        return Err(DescriptionError::InvalidEncoding { line });
    }
    String::from_utf16(&wide[..count as usize])
        .map_err(|_| DescriptionError::InvalidEncoding { line })
}
#[cfg(not(windows))]
fn decode_cp950(_bytes: &[u8], _line: usize) -> Result<String, DescriptionError> {
    Err(DescriptionError::UnsupportedPlatform)
}

#[cfg(all(test, windows))]
mod tests {
    use super::*;

    fn fixture() -> DescriptionCatalog {
        DescriptionCatalog::parse_cp950(
            b"8\r\nSword\r\nAxe\r\n\r\n%i template\r\n\\fTname\r\n$0\r\n150% power\r\nlast",
        )
        .unwrap()
    }

    #[test]
    fn exact_line_numbers_empty_slots_and_short_declared_suffix_are_preserved() {
        let catalog = DescriptionCatalog::parse_cp950(b"5\r\nfirst\r\n\r\nthird").unwrap();
        assert_eq!(
            (
                catalog.declared_count(),
                catalog.available_count(),
                catalog.is_complete()
            ),
            (5, 3, false)
        );
        assert_eq!(catalog.entry(0).unwrap(), "first");
        assert_eq!(catalog.entry(1).unwrap(), "");
        assert_eq!(catalog.entry(2).unwrap(), "third");
        assert!(matches!(
            catalog.resolve_name("$1"),
            Err(DescriptionError::EmptyEntry { id: 1 })
        ));
        assert!(matches!(
            catalog.entry(3),
            Err(DescriptionError::MissingEntry { id: 3, .. })
        ));
        assert!(matches!(
            catalog.entry(5),
            Err(DescriptionError::OutsideDeclaredRange { id: 5, .. })
        ));
        assert!(DescriptionCatalog::parse_cp950(b"1\r\nname\r\n")
            .unwrap()
            .is_complete());
        assert_eq!(
            DescriptionCatalog::parse_cp950(b"1\r\n\r\n")
                .unwrap()
                .entry(0)
                .unwrap(),
            ""
        );
    }

    #[test]
    fn literal_decimal_references_and_native_semicolon_boundary_are_not_reindexed() {
        let catalog = fixture();
        for (token, expected, references) in [
            ("plain 名稱", "plain 名稱", vec![]),
            ("$0 $1 ($0)", "Sword Axe (Sword)", vec![0, 1, 0]),
            ("+9 $0000;$1;!", "+9 SwordAxe!", vec![0, 1]),
            ("$0x10", "Swordx10", vec![0]),
            ("$1;;", "Axe;", vec![1]),
            ("$6", "150% power", vec![6]),
        ] {
            let resolved = catalog.resolve_name(token).unwrap();
            assert_eq!(resolved.text(), expected);
            assert_eq!(resolved.to_string(), expected);
            assert_eq!(resolved.source_token(), token);
            assert_eq!(resolved.reference_ids(), references);
        }
        let repeated = catalog.resolve_name(&"$7".repeat(MAX_REFERENCES)).unwrap();
        assert_eq!(repeated.reference_ids().len(), MAX_REFERENCES);
        assert_eq!(repeated.text().len(), MAX_RESOLVED_BYTES);
    }

    #[test]
    fn unknown_references_templates_and_lt_range_have_explicit_errors() {
        let catalog = fixture();
        for token in ["$", "$-1", "$+1", "$x", "$$", "$１"] {
            assert!(matches!(
                catalog.resolve_name(token),
                Err(DescriptionError::InvalidReference { .. })
            ));
        }
        assert!(matches!(
            catalog.resolve_name("$4294967296"),
            Err(DescriptionError::ReferenceOverflow { .. })
        ));
        assert!(matches!(
            catalog.resolve_name("$20000"),
            Err(DescriptionError::OutsideDeclaredRange { id: 20000, .. })
        ));
        for (id, reason) in [
            (3, UnsupportedDescription::RuntimeTemplate),
            (4, UnsupportedDescription::InlineFormatting),
            (5, UnsupportedDescription::EmbeddedReference),
        ] {
            assert!(
                matches!(catalog.resolve_name(&format!("${id}")), Err(DescriptionError::UnsupportedEntry { id: actual, reason: actual_reason })
                if actual == id && actual_reason == reason)
            );
        }
    }

    #[test]
    fn strict_cp950_known_answer_and_invalid_bytes_never_use_lossy_or_utf8_fallback() {
        let catalog =
            DescriptionCatalog::parse_cp950(b"2\r\n\xaa\xf8\xbc\x43\r\n\xa9\xf2").unwrap();
        assert_eq!(catalog.entry(0).unwrap(), "長劍");
        assert_eq!(catalog.entry(1).unwrap(), "斧");
        for raw in [b"1\r\n\xa4".as_slice(), b"1\r\n\xa4\x20".as_slice()] {
            assert!(matches!(
                DescriptionCatalog::parse_cp950(raw),
                Err(DescriptionError::InvalidEncoding { line: 2 })
            ));
        }
    }

    #[test]
    fn byte_count_format_control_and_expansion_limits_fail_before_partial_names() {
        assert!(matches!(
            DescriptionCatalog::parse_cp950(&vec![b'x'; MAX_TABLE_BYTES + 1]),
            Err(DescriptionError::TableTooLarge)
        ));
        for raw in [
            b"0\r\nname".as_slice(),
            b"100001\r\nname",
            b"-1\r\nname",
            b"1 2\r\nname",
            b"\xef\xbb\xbf1\r\nname",
            b"1",
        ] {
            assert!(matches!(
                DescriptionCatalog::parse_cp950(raw),
                Err(DescriptionError::InvalidHeader)
            ));
        }
        assert!(matches!(
            DescriptionCatalog::parse_cp950(b"1\r\n"),
            Err(DescriptionError::NoEntries)
        ));
        assert!(matches!(
            DescriptionCatalog::parse_cp950(b"1\r\na\r\nb"),
            Err(DescriptionError::ExtraEntries)
        ));
        for raw in [
            b"1\nname".as_slice(),
            b"1\rname",
            b"1\r\nname\n",
            b"1\r\nname\r",
        ] {
            assert!(matches!(
                DescriptionCatalog::parse_cp950(raw),
                Err(DescriptionError::InvalidLineEnding { .. })
            ));
        }
        assert!(matches!(
            DescriptionCatalog::parse_cp950(b"1\r\n#0 indexed"),
            Err(DescriptionError::UnsupportedTableFormat)
        ));
        assert!(matches!(
            DescriptionCatalog::parse_cp950(b"1\r\nna\0me"),
            Err(DescriptionError::ControlCharacter { .. })
        ));
        let long = format!("1\r\n{}", "x".repeat(MAX_LINE_BYTES + 1));
        assert!(matches!(
            DescriptionCatalog::parse_cp950(long.as_bytes()),
            Err(DescriptionError::LineTooLong { .. })
        ));
        let catalog =
            DescriptionCatalog::parse_cp950(format!("1\r\n{}", "x".repeat(300)).as_bytes())
                .unwrap();
        assert!(matches!(
            catalog.resolve_name("$0$0"),
            Err(DescriptionError::ResolvedNameTooLong)
        ));
        assert!(matches!(
            catalog.resolve_name(&"x".repeat(MAX_INPUT_BYTES + 1)),
            Err(DescriptionError::NameTokenTooLong)
        ));
        for token in ["", " \u{a0}\u{2007}\u{202f}", "name\0", "name\n"] {
            assert!(matches!(
                catalog.resolve_name(token),
                Err(DescriptionError::InvalidNameToken)
            ));
        }
    }

    #[test]
    fn pre_cancelled_load_never_opens_a_resource_path_or_publishes_a_catalog() {
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            DescriptionCatalog::load_shared_cancellable("does-not-exist", &cancelled),
            Err(DescriptionError::Cancelled)
        ));
        assert!(matches!(
            DescriptionCatalog::parse_cancellable(b"1\r\nname", &cancelled),
            Err(DescriptionError::Cancelled)
        ));
    }

    #[test]
    #[ignore = "requires the selected LITO_SHARED_RESOURCE_ROOT shared Text.pak reference"]
    fn real_shared_description_matches_independent_csharp_cp950_reference() {
        use sha2::{Digest, Sha256};
        let root =
            std::env::var_os("LITO_SHARED_RESOURCE_ROOT").expect("set LITO_SHARED_RESOURCE_ROOT");
        let mut archive = SharedArchive::open(Path::new(&root), SharedArchiveKind::Text).unwrap();
        let raw = archive.read_entry(ENTRY_NAME).unwrap();
        assert_eq!(raw.len(), 249300);
        assert_eq!(
            format!("{:X}", Sha256::digest(&raw)),
            "F06C94F47271FCB01F1D5AF63725DFA9A230B5F6BE3C0DD1D53CD37628ADB7DF"
        );
        let catalog = DescriptionCatalog::parse_cp950(&raw).unwrap();
        assert_eq!(
            (
                catalog.declared_count(),
                catalog.available_count(),
                catalog.is_complete()
            ),
            (17001, 16945, false)
        );
        assert_eq!(
            catalog
                .entries
                .iter()
                .filter(|entry| entry.is_empty())
                .count(),
            19
        );
        assert!(!catalog.entries.iter().any(|entry| entry.contains('$')));
        for (id, text) in [
            (0, "長劍"),
            (9, "揮舞"),
            (10, "打開"),
            (69, "木棒"),
            (70, "匕首"),
            (225, "斗蓬"),
            (1663, "火靈戒指"),
            (1822, "精靈玉"),
            (2452, "黑暗"),
            (2468, "頭飾"),
            (5287, "勇者的南瓜袋子"),
            (16944, "日輪冠冕寶盒"),
        ] {
            assert_eq!(
                catalog.resolve_name(&format!("${id}")).unwrap().text(),
                text
            );
        }
        assert_eq!(
            catalog.resolve_name("$2452 $2468").unwrap().text(),
            "黑暗 頭飾"
        );
        assert_eq!(
            catalog.resolve_name("+9 $0 ($9)").unwrap().text(),
            "+9 長劍 (揮舞)"
        );
        assert!(matches!(
            catalog.resolve_name("$16945"),
            Err(DescriptionError::MissingEntry { .. })
        ));
        assert!(matches!(
            catalog.resolve_name("$17001"),
            Err(DescriptionError::OutsideDeclaredRange { .. })
        ));
        assert!(matches!(
            catalog.resolve_name("$20000"),
            Err(DescriptionError::OutsideDeclaredRange { .. })
        ));
        assert!(matches!(
            catalog.resolve_name("$3997"),
            Err(DescriptionError::UnsupportedEntry {
                reason: UnsupportedDescription::RuntimeTemplate,
                ..
            })
        ));
        assert!(matches!(
            catalog.resolve_name("$1929"),
            Err(DescriptionError::UnsupportedEntry {
                reason: UnsupportedDescription::InlineFormatting,
                ..
            })
        ));
        let loaded =
            DescriptionCatalog::load_shared_cancellable(Path::new(&root), &AtomicBool::new(false))
                .unwrap();
        assert_eq!(loaded.entries, catalog.entries);
        eprintln!(
            "shared desc-c.tbl: declared {}, available {}, 19 empty slots; CP950 golden verified",
            catalog.declared_count(),
            catalog.available_count()
        );
    }
}
