//! Explicit modern-only configuration. Never consults TW's list.ini, launcher.ini,
//! lineage.cfg, SQL credentials, native BIN paths or the current directory.
//!
//! Parsing a profile does not authenticate a server or validate all game assets.
//! Certificate validation belongs to TLS; resource readers still enforce their
//! handle/reparse/content checks on every load. No settings/cache files are written.

use super::resources::{open_readonly, reject_reparse_chain, verify_open_path, ResourceError};
use base64::Engine;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::fmt;
use std::io::Read;
use std::net::SocketAddr;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;

pub const MAX_PROFILE_BYTES: usize = 32 * 1024;
pub const MAX_CERTIFICATE_BYTES: usize = 16 * 1024;

/// An explicit application contract, not a server capability bypass.
/// Schema1 keeps its original v37 entry; schema2 requires Modern v47.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameplayProtocol {
    LegacyV37,
    ModernV47,
}

/// Explicit opt-in inside the existing Modern v47 profile. Absence preserves
/// the original handshake and settings namespace; present null is invalid.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GameplayExtension {
    PlayerHealingV1,
    ArmorProtectionV1,
    PlayerHealingArmorV1,
    ArmorPresentationV1,
    PlayerHealingArmorPresentationV1,
    FullVipV1,
    FullVipV2,
}

impl GameplayProtocol {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::LegacyV37 => "legacy_v37",
            Self::ModernV47 => "modern_v47",
        }
    }
}

impl GameplayExtension {
    pub const ALL: [Self; 7] = [
        Self::PlayerHealingV1,
        Self::ArmorProtectionV1,
        Self::PlayerHealingArmorV1,
        Self::ArmorPresentationV1,
        Self::PlayerHealingArmorPresentationV1,
        Self::FullVipV1,
        Self::FullVipV2,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PlayerHealingV1 => "player_healing_v1",
            Self::ArmorProtectionV1 => "armor_protection_v1",
            Self::PlayerHealingArmorV1 => "player_healing_armor_v1",
            Self::ArmorPresentationV1 => "armor_presentation_v1",
            Self::PlayerHealingArmorPresentationV1 => "player_healing_armor_presentation_v1",
            Self::FullVipV1 => "full_vip_v1",
            Self::FullVipV2 => "full_vip_v2",
        }
    }
}

fn explicit_gameplay_extension<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<GameplayExtension>, D::Error> {
    GameplayExtension::deserialize(deserializer).map(Some)
}

fn explicit_gameplay_protocol<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<GameplayProtocol>, D::Error> {
    // `default` handles an absent field. A present null must not masquerade
    // as absence and silently select the old protocol.
    GameplayProtocol::deserialize(deserializer).map(Some)
}

fn explicit_effect_catalog_path<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<PathBuf>, D::Error> {
    PathBuf::deserialize(deserializer).map(Some)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileError {
    TooLarge,
    InvalidDocument,
    UnsupportedSchema,
    InvalidIdentity,
    InvalidEndpoint,
    InvalidServerName,
    InvalidTimeout,
    InvalidTrust,
    UnsafeResourceRoot,
    UnsafeEffectCatalog,
    UnsafeProfilePath,
    ReadFailed,
}

impl fmt::Display for ProfileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Do not echo arbitrary JSON values (which might accidentally be secrets).
        write!(f, "modern profile: {self:?}")
    }
}
impl std::error::Error for ProfileError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Document {
    schema: u16,
    #[serde(default, deserialize_with = "explicit_gameplay_protocol")]
    gameplay_protocol: Option<GameplayProtocol>,
    #[serde(default, deserialize_with = "explicit_gameplay_extension")]
    gameplay_extension: Option<GameplayExtension>,
    profile_id: String,
    display_name: String,
    shared_resource_root: PathBuf,
    #[serde(default, deserialize_with = "explicit_effect_catalog_path")]
    effect_catalog_path: Option<PathBuf>,
    endpoint: SocketAddr,
    tls_server_name: String,
    trust: TrustDocument,
    connect_timeout_ms: u32,
    exchange_timeout_ms: u32,
}

#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum TrustDocument {
    WebPki {},
    /// Public DER certificate, never a private key or a validation-disable flag.
    /// This CA replaces the bundled trust set for this profile only.
    PrivateCa {
        certificate_der_base64: String,
    },
}

#[derive(Clone, PartialEq, Eq)]
pub enum Trust {
    WebPki,
    PrivateCa(Vec<u8>),
}

impl fmt::Debug for Trust {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WebPki => f.write_str("WebPki"),
            Self::PrivateCa(der) => f
                .debug_struct("PrivateCa")
                .field("certificate_bytes", &der.len())
                .finish(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ModernProfile {
    gameplay_protocol: GameplayProtocol,
    gameplay_extension: Option<GameplayExtension>,
    profile_id: String,
    display_name: String,
    shared_resource_root: PathBuf,
    effect_catalog_path: Option<PathBuf>,
    endpoint: SocketAddr,
    tls_server_name: String,
    trust: Trust,
    connect_timeout: Duration,
    exchange_timeout: Duration,
    context_key: [u8; 32],
}

impl ModernProfile {
    pub fn parse(bytes: &[u8]) -> Result<Self, ProfileError> {
        if bytes.len() > MAX_PROFILE_BYTES {
            return Err(ProfileError::TooLarge);
        }
        let doc: Document =
            serde_json::from_slice(bytes).map_err(|_| ProfileError::InvalidDocument)?;
        let gameplay_protocol = match (doc.schema, doc.gameplay_protocol) {
            (1, None) => GameplayProtocol::LegacyV37,
            (2, Some(GameplayProtocol::ModernV47)) => GameplayProtocol::ModernV47,
            (1 | 2, _) => return Err(ProfileError::InvalidDocument),
            _ => return Err(ProfileError::UnsupportedSchema),
        };
        if doc.gameplay_extension.is_some() && gameplay_protocol != GameplayProtocol::ModernV47 {
            return Err(ProfileError::InvalidDocument);
        }
        if doc.effect_catalog_path.is_some() && gameplay_protocol != GameplayProtocol::ModernV47 {
            return Err(ProfileError::InvalidDocument);
        }
        if doc.profile_id.is_empty()
            || doc.profile_id.len() > 48
            || !doc
                .profile_id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-' || c == b'_')
            || doc.display_name.is_empty()
            || doc.display_name.len() > 192
            || doc.display_name.trim() != doc.display_name
            || doc.display_name.chars().any(char::is_control)
        {
            return Err(ProfileError::InvalidIdentity);
        }
        validate_resource_root(&doc.shared_resource_root)?;
        if let Some(path) = &doc.effect_catalog_path {
            if !is_local_absolute_path(path)
                || path.components().any(|c| matches!(c, Component::CurDir))
            {
                return Err(ProfileError::UnsafeEffectCatalog);
            }
            #[cfg(windows)]
            if !matches!(path.components().next(), Some(Component::Prefix(p)) if matches!(p.kind(), std::path::Prefix::Disk(_)))
            {
                return Err(ProfileError::UnsafeEffectCatalog);
            }
        }
        let ip = doc.endpoint.ip();
        if doc.endpoint.port() == 0
            || ip.is_unspecified()
            || ip.is_multicast()
            || ip == std::net::Ipv4Addr::BROADCAST
            || matches!(doc.endpoint, SocketAddr::V6(addr) if addr.scope_id() != 0 || addr.flowinfo() != 0)
        {
            return Err(ProfileError::InvalidEndpoint);
        }
        // Explicit numeric transport address avoids an unbounded blocking DNS
        // lookup on the UI thread. DNS identity is still independently verified.
        let tls_server_name = doc.tls_server_name.to_ascii_lowercase();
        validate_server_name(&tls_server_name)?;
        let connect_timeout = bounded_timeout(doc.connect_timeout_ms)?;
        let exchange_timeout = bounded_timeout(doc.exchange_timeout_ms)?;
        let trust = match doc.trust {
            TrustDocument::WebPki {} => Trust::WebPki,
            TrustDocument::PrivateCa {
                certificate_der_base64,
            } => {
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(certificate_der_base64)
                    .map_err(|_| ProfileError::InvalidTrust)?;
                if bytes.is_empty() || bytes.len() > MAX_CERTIFICATE_BYTES {
                    return Err(ProfileError::InvalidTrust);
                }
                // DER/CA/signature validation is not reimplemented here.
                Trust::PrivateCa(bytes)
            }
        };
        let mut hash = Sha256::new();
        match gameplay_protocol {
            GameplayProtocol::LegacyV37 => hash.update(b"modern-lito-context-v1\0"),
            GameplayProtocol::ModernV47 => {
                hash.update(b"modern-lito-context-v2\0");
                hash_part(&mut hash, b"modern_v47");
            }
        }
        if let Some(extension) = doc.gameplay_extension {
            hash_part(
                &mut hash,
                match extension {
                    GameplayExtension::PlayerHealingV1 => b"player_healing_v1",
                    GameplayExtension::FullVipV1 => b"full_vip_v1",
                    GameplayExtension::FullVipV2 => b"full_vip_v2",
                    GameplayExtension::ArmorProtectionV1 => b"armor_protection_v1",
                    GameplayExtension::PlayerHealingArmorV1 => b"player_healing_armor_v1",
                    GameplayExtension::ArmorPresentationV1 => b"armor_presentation_v1",
                    GameplayExtension::PlayerHealingArmorPresentationV1 => {
                        b"player_healing_armor_presentation_v1"
                    }
                },
            );
        }
        hash_part(&mut hash, doc.profile_id.as_bytes());
        hash_part(
            &mut hash,
            doc.shared_resource_root
                .to_str()
                .ok_or(ProfileError::UnsafeResourceRoot)?
                .as_bytes(),
        );
        if let Some(path) = &doc.effect_catalog_path {
            hash_part(&mut hash, b"effect_catalog_path");
            hash_part(
                &mut hash,
                path.to_str()
                    .ok_or(ProfileError::UnsafeEffectCatalog)?
                    .as_bytes(),
            );
        }
        hash_part(&mut hash, doc.endpoint.to_string().as_bytes());
        hash_part(&mut hash, tls_server_name.as_bytes());
        match &trust {
            Trust::WebPki => hash_part(&mut hash, b"web-pki"),
            Trust::PrivateCa(der) => {
                hash_part(&mut hash, b"private-ca");
                hash_part(&mut hash, der);
            }
        }
        Ok(Self {
            gameplay_protocol,
            gameplay_extension: doc.gameplay_extension,
            profile_id: doc.profile_id,
            display_name: doc.display_name,
            shared_resource_root: doc.shared_resource_root,
            effect_catalog_path: doc.effect_catalog_path,
            endpoint: doc.endpoint,
            tls_server_name,
            trust,
            connect_timeout,
            exchange_timeout,
            context_key: hash.finalize().into(),
        })
    }

    /// One explicit, bounded, read-only file. There is no fallback search path,
    /// credential persistence, directory creation or migration of TW settings.
    pub fn load(path: &Path) -> Result<Self, ProfileError> {
        if !is_local_absolute_path(path) {
            return Err(ProfileError::UnsafeProfilePath);
        }
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
                .ok_or(ProfileError::UnsafeProfilePath)?;
            let wide: Vec<u16> = root.as_os_str().encode_wide().chain(Some(0)).collect();
            // Reject network-mapped drive letters before opening directories.
            if unsafe { GetDriveTypeW(wide.as_ptr()) } == 4 {
                return Err(ProfileError::UnsafeProfilePath);
            }
        }
        reject_reparse_chain(path).map_err(profile_io_error)?;
        let mut parents: Vec<_> = path.ancestors().skip(1).collect();
        parents.reverse();
        let mut directory_guards = Vec::with_capacity(parents.len());
        for parent in parents {
            let handle = open_readonly(parent, true).map_err(profile_io_error)?;
            verify_open_path(&handle, parent, true).map_err(profile_io_error)?;
            directory_guards.push((parent, handle));
        }
        let mut file = open_readonly(path, false).map_err(profile_io_error)?;
        verify_open_path(&file, path, false).map_err(profile_io_error)?;
        #[cfg(windows)]
        if !is_local_absolute_path(
            &super::resources::final_handle_path(&file).map_err(profile_io_error)?,
        ) {
            return Err(ProfileError::UnsafeProfilePath);
        }
        let metadata = file.metadata().map_err(|_| ProfileError::ReadFailed)?;
        if metadata.len() > MAX_PROFILE_BYTES as u64 {
            return Err(ProfileError::TooLarge);
        }
        let mut bytes = Vec::new();
        file.by_ref()
            .take(MAX_PROFILE_BYTES as u64 + 1)
            .read_to_end(&mut bytes)
            .map_err(|_| ProfileError::ReadFailed)?;
        verify_open_path(&file, path, false).map_err(profile_io_error)?;
        for (parent, handle) in &directory_guards {
            verify_open_path(handle, parent, true).map_err(profile_io_error)?;
        }
        Self::parse(&bytes)
    }

    pub fn gameplay_protocol(&self) -> GameplayProtocol {
        self.gameplay_protocol
    }
    pub fn gameplay_extension(&self) -> Option<GameplayExtension> {
        self.gameplay_extension
    }
    pub fn profile_id(&self) -> &str {
        &self.profile_id
    }
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
    pub fn shared_resource_root(&self) -> &Path {
        &self.shared_resource_root
    }
    pub fn effect_catalog_path(&self) -> Option<&Path> {
        self.effect_catalog_path.as_deref()
    }
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn tls_server_name(&self) -> &str {
        &self.tls_server_name
    }
    pub fn trust(&self) -> &Trust {
        &self.trust
    }
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }
    pub fn exchange_timeout(&self) -> Duration {
        self.exchange_timeout
    }
    /// Namespace component, NOT an asset-content manifest or an authentication
    /// token. Different spelling may deliberately cause a safe cache miss.
    pub fn context_key(&self) -> [u8; 32] {
        self.context_key
    }
}

fn profile_io_error(error: ResourceError) -> ProfileError {
    match error {
        ResourceError::UnsafePath(_) => ProfileError::UnsafeProfilePath,
        _ => ProfileError::ReadFailed,
    }
}

fn hash_part(hash: &mut Sha256, bytes: &[u8]) {
    hash.update((bytes.len() as u64).to_be_bytes());
    hash.update(bytes);
}

fn bounded_timeout(ms: u32) -> Result<Duration, ProfileError> {
    if !(100..=30_000).contains(&ms) {
        return Err(ProfileError::InvalidTimeout);
    }
    Ok(Duration::from_millis(u64::from(ms)))
}

fn validate_server_name(name: &str) -> Result<(), ProfileError> {
    if name.is_empty()
        || name.len() > 253
        || name.parse::<std::net::IpAddr>().is_ok()
        || name.split('.').any(|label| {
            label.is_empty()
                || label.len() > 63
                || label.starts_with('-')
                || label.ends_with('-')
                || !label
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-')
        })
    {
        return Err(ProfileError::InvalidServerName);
    }
    Ok(())
}

fn is_local_absolute_path(path: &Path) -> bool {
    if !path.is_absolute() || path.components().any(|c| matches!(c, Component::ParentDir)) {
        return false;
    }
    if path.components().any(|c| match c {
        Component::Normal(name) => name.to_str().is_none_or(|name| {
            name.ends_with(['.', ' ']) || name.contains(':') || name.chars().any(char::is_control)
        }),
        _ => false,
    }) {
        return false;
    }
    #[cfg(windows)]
    {
        use std::path::Prefix;
        matches!(path.components().next(), Some(Component::Prefix(p))
            if matches!(p.kind(), Prefix::Disk(_) | Prefix::VerbatimDisk(_)))
    }
    #[cfg(not(windows))]
    true
}

fn validate_resource_root(path: &Path) -> Result<(), ProfileError> {
    if !is_local_absolute_path(path) {
        return Err(ProfileError::UnsafeResourceRoot);
    }
    for component in path.components() {
        if let Component::Normal(name) = component {
            let name = name.to_str().ok_or(ProfileError::UnsafeResourceRoot)?;
            if [
                "lito", "ltmap", "lttext", "ltsound", "ltsurf", "lttile", "ltsprite", "lticon",
            ]
            .iter()
            .any(|blocked| name.eq_ignore_ascii_case(blocked))
                || name.ends_with(['.', ' '])
                || name.contains(':')
                || name.chars().any(char::is_control)
            {
                return Err(ProfileError::UnsafeResourceRoot);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};
    use std::fs::OpenOptions;

    fn document() -> Value {
        json!({
            "schema": 1, "profile_id": "lito-local", "display_name": "LiTo 測試",
            "shared_resource_root": if cfg!(windows) { "D:\\lineage開發\\0-客戶端" } else { "/shared/client" },
            "endpoint": "127.0.0.1:25400", "tls_server_name": "localhost",
            "trust": {"kind": "web_pki"},
            "connect_timeout_ms": 3000, "exchange_timeout_ms": 5000
        })
    }
    fn parse(value: &Value) -> Result<ModernProfile, ProfileError> {
        ModernProfile::parse(&serde_json::to_vec(value).unwrap())
    }
    fn modern_document() -> Value {
        let mut doc = document();
        doc["schema"] = json!(2);
        doc["gameplay_protocol"] = json!("modern_v47");
        doc
    }
    #[test]
    fn explicit_effect_catalog_is_modern_only_local_and_context_bound() {
        let old = parse(&modern_document()).unwrap();
        assert_eq!(old.effect_catalog_path(), None);
        let mut doc = modern_document();
        let path = if cfg!(windows) {
            "D:\\lineage開發\\客戶端xml\\effectlist2.xml"
        } else {
            "/client/effectlist2.xml"
        };
        doc["effect_catalog_path"] = json!(path);
        let configured = parse(&doc).unwrap();
        assert_eq!(configured.effect_catalog_path(), Some(Path::new(path)));
        assert_ne!(configured.context_key(), old.context_key());
        doc["display_name"] = json!("另一個顯示名稱");
        assert_eq!(parse(&doc).unwrap().context_key(), configured.context_key());
        for invalid in ["effectlist2.xml", "D:\\client\\..\\effectlist2.xml"] {
            doc["effect_catalog_path"] = json!(invalid);
            assert_eq!(parse(&doc).unwrap_err(), ProfileError::UnsafeEffectCatalog);
        }
        doc["effect_catalog_path"] = Value::Null;
        assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidDocument);
        let mut legacy = document();
        legacy["effect_catalog_path"] = json!(path);
        assert_eq!(parse(&legacy).unwrap_err(), ProfileError::InvalidDocument);
    }
    #[test]
    fn healing_extension_is_explicit_typed_and_context_isolated() {
        let old = parse(&modern_document()).unwrap();
        assert_eq!(old.gameplay_extension(), None);
        let mut doc = modern_document();
        doc["gameplay_extension"] = json!("player_healing_v1");
        let extended = parse(&doc).unwrap();
        assert_eq!(
            extended.gameplay_extension(),
            Some(GameplayExtension::PlayerHealingV1)
        );
        assert_eq!(extended.gameplay_protocol(), GameplayProtocol::ModernV47);
        assert_ne!(extended.context_key(), old.context_key());
        doc["display_name"] = json!("new label");
        assert_eq!(parse(&doc).unwrap().context_key(), extended.context_key());
        for value in [
            Value::Null,
            json!(true),
            json!(3),
            json!([]),
            json!(""),
            json!("player_healing_v2"),
        ] {
            doc["gameplay_extension"] = value;
            assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidDocument);
        }
        let mut legacy = document();
        legacy["gameplay_extension"] = json!("player_healing_v1");
        assert_eq!(parse(&legacy).unwrap_err(), ProfileError::InvalidDocument);
        doc["gameplay_extension"] = json!("player_healing_v1");
        let duplicate = serde_json::to_string(&doc).unwrap().replacen(
            '{',
            "{\"gameplay_extension\":\"player_healing_v1\",",
            1,
        );
        assert_eq!(
            ModernProfile::parse(duplicate.as_bytes()).unwrap_err(),
            ProfileError::InvalidDocument
        );
    }
    #[test]
    fn extended_profiles_are_explicit_and_never_alias_other_or_old_contexts() {
        let mut keys = vec![
            parse(&document()).unwrap().context_key(),
            parse(&modern_document()).unwrap().context_key(),
        ];
        for (name, extension) in [
            ("full_vip_v1", GameplayExtension::FullVipV1),
            ("full_vip_v2", GameplayExtension::FullVipV2),
            ("player_healing_v1", GameplayExtension::PlayerHealingV1),
            ("armor_protection_v1", GameplayExtension::ArmorProtectionV1),
            (
                "armor_presentation_v1",
                GameplayExtension::ArmorPresentationV1,
            ),
            (
                "player_healing_armor_presentation_v1",
                GameplayExtension::PlayerHealingArmorPresentationV1,
            ),
            (
                "player_healing_armor_v1",
                GameplayExtension::PlayerHealingArmorV1,
            ),
        ] {
            let mut doc = modern_document();
            doc["gameplay_extension"] = json!(name);
            let profile = parse(&doc).unwrap();
            assert_eq!(profile.gameplay_extension(), Some(extension));
            assert!(!keys.contains(&profile.context_key()));
            keys.push(profile.context_key());
            doc["display_name"] = json!("label only");
            assert_eq!(parse(&doc).unwrap().context_key(), profile.context_key());
            let mut legacy = document();
            legacy["gameplay_extension"] = json!(name);
            assert_eq!(parse(&legacy).unwrap_err(), ProfileError::InvalidDocument);
            let duplicate = serde_json::to_string(&doc).unwrap().replacen(
                '{',
                &format!("{{\"gameplay_extension\":\"{name}\","),
                1,
            );
            assert_eq!(
                ModernProfile::parse(duplicate.as_bytes()).unwrap_err(),
                ProfileError::InvalidDocument
            );
        }
        for value in [
            json!("armor_protection_v2"),
            json!("player_healing_armor_v2"),
            json!(["player_healing_v1", "armor_protection_v1"]),
            json!(7),
            Value::Null,
        ] {
            let mut doc = modern_document();
            doc["gameplay_extension"] = value;
            assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidDocument);
        }
    }
    #[test]
    fn explicit_modern_profile_has_its_own_context_without_changing_v1() {
        let original = parse(&document()).unwrap().context_key();
        #[cfg(windows)]
        assert_eq!(
            original
                .iter()
                .map(|b| format!("{b:02X}"))
                .collect::<String>(),
            "EC5988CEF038E25EF778A0922BA1DE280A4CE62DA4DE09D1B620CC4B1C310209"
        );
        let mut doc = document();
        doc["schema"] = json!(2);
        doc["gameplay_protocol"] = json!("modern_v47");
        let modern = parse(&doc).expect("explicit schema2 Modern v47 profile");
        assert_eq!(modern.gameplay_protocol(), GameplayProtocol::ModernV47);
        assert_eq!(
            parse(&document()).unwrap().gameplay_protocol(),
            GameplayProtocol::LegacyV37
        );
        assert_ne!(modern.context_key(), original);
        doc["display_name"] = json!("另一個顯示名稱");
        assert_eq!(parse(&doc).unwrap().context_key(), modern.context_key());
    }
    #[test]
    fn modern_profile_requires_exact_schema_and_nonnull_explicit_protocol() {
        for schema in [1, 2] {
            for value in [
                Value::Null,
                json!(37),
                json!(47),
                json!(true),
                json!("legacy_v37"),
                json!("modern_v48"),
                json!(""),
            ] {
                let mut doc = document();
                doc["schema"] = json!(schema);
                doc["gameplay_protocol"] = value;
                assert!(parse(&doc).is_err());
            }
        }
        let mut doc = document();
        doc["schema"] = json!(2);
        assert!(
            parse(&doc).is_err(),
            "schema2 cannot silently choose a protocol"
        );
        doc["schema"] = json!(1);
        doc["gameplay_protocol"] = json!("modern_v47");
        assert!(
            parse(&doc).is_err(),
            "schema1 cannot silently change gameplay"
        );
    }
    #[test]
    fn modern_profile_keeps_trust_path_timeout_and_secret_field_guards() {
        for (key, value, expected) in [
            (
                "password",
                json!("never-echo-this-value"),
                ProfileError::InvalidDocument,
            ),
            (
                "account",
                json!("never-echo-this-value"),
                ProfileError::InvalidDocument,
            ),
            (
                "bin_path",
                json!("TW13081901.bin"),
                ProfileError::InvalidDocument,
            ),
            (
                "trust",
                json!({"kind":"insecure"}),
                ProfileError::InvalidDocument,
            ),
            (
                "trust",
                json!({"kind":"private_ca","certificate_der_base64":"%%%"}),
                ProfileError::InvalidTrust,
            ),
            (
                "shared_resource_root",
                json!("LiTo"),
                ProfileError::UnsafeResourceRoot,
            ),
            (
                "endpoint",
                json!("127.0.0.1:0"),
                ProfileError::InvalidEndpoint,
            ),
            (
                "tls_server_name",
                json!("127.0.0.1"),
                ProfileError::InvalidServerName,
            ),
            (
                "exchange_timeout_ms",
                json!(0),
                ProfileError::InvalidTimeout,
            ),
        ] {
            let mut doc = modern_document();
            doc[key] = value;
            let error = parse(&doc).unwrap_err();
            assert_eq!(error, expected, "{key}");
            assert!(!error.to_string().contains("never-echo"));
        }
        let bytes = serde_json::to_string(&modern_document()).unwrap();
        let duplicate = bytes.replacen('{', "{\"gameplay_protocol\":\"modern_v47\",", 1);
        assert_eq!(
            ModernProfile::parse(duplicate.as_bytes()).unwrap_err(),
            ProfileError::InvalidDocument
        );
    }
    #[test]
    fn modern_profile_is_explicit_and_has_no_tw_or_credential_fields() {
        let profile = parse(&document()).unwrap();
        assert_eq!(profile.profile_id(), "lito-local");
        assert_eq!(profile.endpoint(), "127.0.0.1:25400".parse().unwrap());
        assert_eq!(profile.connect_timeout(), Duration::from_secs(3));
        for key in [
            "password",
            "account",
            "database",
            "bin_path",
            "native_crc",
            "disable_tls_validation",
        ] {
            let mut doc = document();
            doc[key] = json!("do-not-echo-this-secret");
            let err = parse(&doc).unwrap_err();
            assert_eq!(err, ProfileError::InvalidDocument);
            assert!(!err.to_string().contains("do-not-echo"));
        }
    }
    #[test]
    fn rejects_duplicate_unknown_missing_and_oversize_documents() {
        let bytes = serde_json::to_string(&document()).unwrap();
        let duplicate = bytes.replacen('{', "{\"schema\":1,", 1);
        assert_eq!(
            ModernProfile::parse(duplicate.as_bytes()).unwrap_err(),
            ProfileError::InvalidDocument
        );
        for key in ["schema", "endpoint", "trust", "shared_resource_root"] {
            let mut doc = document();
            doc.as_object_mut().unwrap().remove(key);
            assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidDocument);
        }
        assert_eq!(
            ModernProfile::parse(&vec![b' '; MAX_PROFILE_BYTES + 1]).unwrap_err(),
            ProfileError::TooLarge
        );
        let mut doc = document();
        doc["schema"] = json!(99);
        assert_eq!(parse(&doc).unwrap_err(), ProfileError::UnsupportedSchema);
    }
    #[test]
    fn context_separates_endpoint_resource_and_trust_but_not_display_label() {
        let original = parse(&document()).unwrap().context_key();
        for (key, value) in [
            ("profile_id", json!("another-profile")),
            ("endpoint", json!("127.0.0.1:25401")),
            ("tls_server_name", json!("lito.example.test")),
            (
                "shared_resource_root",
                json!(if cfg!(windows) {
                    "D:\\shared-client-b"
                } else {
                    "/shared/client-b"
                }),
            ),
            (
                "trust",
                json!({"kind":"private_ca", "certificate_der_base64":"AQID"}),
            ),
        ] {
            let mut doc = document();
            doc[key] = value;
            assert_ne!(parse(&doc).unwrap().context_key(), original, "{key}");
        }
        let mut doc = document();
        doc["display_name"] = json!("變更顯示名稱");
        doc["tls_server_name"] = json!("LOCALHOST");
        assert_eq!(parse(&doc).unwrap().context_key(), original);
    }
    #[test]
    fn endpoint_name_and_timeouts_cannot_disable_transport_guards() {
        for endpoint in [
            "0.0.0.0:25400",
            "127.0.0.1:0",
            "224.0.0.1:25400",
            "255.255.255.255:25400",
            "[::]:25400",
            "[ff02::1]:25400",
        ] {
            let mut doc = document();
            doc["endpoint"] = json!(endpoint);
            assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidEndpoint);
        }
        for name in [
            "",
            "127.0.0.1",
            "*.test",
            "https://localhost",
            "local host",
            "-host",
            "host-",
            "host.",
            "a..b",
            "host\0",
            "遊戲.test",
        ] {
            let mut doc = document();
            doc["tls_server_name"] = json!(name);
            assert_eq!(
                parse(&doc).unwrap_err(),
                ProfileError::InvalidServerName,
                "{name:?}"
            );
        }
        for timeout in [0, 99, 30_001, u32::MAX] {
            for key in ["connect_timeout_ms", "exchange_timeout_ms"] {
                let mut doc = document();
                doc[key] = json!(timeout);
                assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidTimeout);
            }
        }
    }
    #[test]
    fn rejects_legacy_roots_and_nonlocal_or_relative_paths() {
        for root in if cfg!(windows) {
            vec![
                "LiTo",
                "D:client",
                "D:\\client\\..\\other",
                "D:\\client\\LiTo",
                "D:\\client\\LTmap",
                "D:\\client\\LiTo.",
                "\\\\server\\share\\client",
                "\\\\.\\C:\\client",
                "D:\\client:stream",
            ]
        } else {
            vec![
                "client",
                "/client/../other",
                "/client/LiTo",
                "/client/LTmap",
            ]
        } {
            let mut doc = document();
            doc["shared_resource_root"] = json!(root);
            assert_eq!(
                parse(&doc).unwrap_err(),
                ProfileError::UnsafeResourceRoot,
                "{root}"
            );
        }
    }
    #[test]
    fn trust_requires_bounded_explicit_public_certificate_not_unsafe_options() {
        for trust in [
            json!({"kind":"insecure"}),
            json!({"kind":"web_pki", "accept_invalid_certificates":true}),
            json!({"kind":"private_ca", "certificate_der_base64":""}),
            json!({"kind":"private_ca", "certificate_der_base64":"%%%"}),
        ] {
            let mut doc = document();
            doc["trust"] = trust;
            assert!(
                parse(&doc).is_err(),
                "unexpected accepted trust configuration: {doc}"
            );
        }
        let mut doc = document();
        doc["trust"] = json!({"kind":"private_ca", "certificate_der_base64":base64::engine::general_purpose::STANDARD.encode(vec![1; MAX_CERTIFICATE_BYTES + 1])});
        assert_eq!(parse(&doc).unwrap_err(), ProfileError::InvalidTrust);
    }
    #[test]
    fn bounded_file_load_uses_explicit_path_without_writing_tw_settings() {
        use std::io::Write;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        for doc in [document(), modern_document()] {
            let path = std::env::temp_dir().join(format!(
                "modern-profile-{}-{nonce}-{}.json",
                std::process::id(),
                doc["schema"]
            ));
            // Synthetic fixture only. Production loading never writes this or any other file.
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .unwrap();
            file.write_all(&serde_json::to_vec(&doc).unwrap()).unwrap();
            drop(file);
            let result = ModernProfile::load(&path);
            let unchanged = std::fs::read(&path).unwrap();
            std::fs::remove_file(&path).unwrap();
            let loaded = result.unwrap();
            assert_eq!(loaded.profile_id(), "lito-local");
            assert_eq!(
                loaded.gameplay_protocol(),
                parse(&doc).unwrap().gameplay_protocol()
            );
            assert_eq!(unchanged, serde_json::to_vec(&doc).unwrap());
        }
        assert_eq!(
            ModernProfile::load(Path::new("launcher.ini")).unwrap_err(),
            ProfileError::UnsafeProfilePath
        );
    }

    #[cfg(windows)]
    #[test]
    fn profile_filename_rejects_ads_and_win32_aliases_before_io() {
        for path in [
            r"D:\profiles\modern.json:stream",
            r"D:\profiles\modern.json.",
            "D:\\profiles\\modern.json ",
            r"\\server\share\modern.json",
        ] {
            assert_eq!(
                ModernProfile::load(Path::new(path)).unwrap_err(),
                ProfileError::UnsafeProfilePath
            );
        }
    }

    #[cfg(windows)]
    #[test]
    #[ignore = "requires Windows developer mode or symlink privilege; synthetic fixture only"]
    fn profile_load_rejects_ancestor_and_file_symlinks() {
        use std::io::Write;
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let fixture = std::env::temp_dir().join(format!(
            "modern-profile-links-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir(&fixture).unwrap();
        let actual = fixture.join("actual");
        std::fs::create_dir(&actual).unwrap();
        let file_path = actual.join("modern.json");
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .unwrap();
        file.write_all(&serde_json::to_vec(&document()).unwrap())
            .unwrap();
        drop(file);
        let directory_link = fixture.join("linked-directory");
        let file_link = fixture.join("linked-file.json");
        let outcome = (|| -> std::io::Result<_> {
            std::os::windows::fs::symlink_dir(&actual, &directory_link)?;
            std::os::windows::fs::symlink_file(&file_path, &file_link)?;
            Ok((
                ModernProfile::load(&directory_link.join("modern.json")),
                ModernProfile::load(&file_link),
            ))
        })();
        // All targets are individual items just created by this test, no recursive delete.
        if std::fs::symlink_metadata(&file_link).is_ok() {
            std::fs::remove_file(&file_link).unwrap();
        }
        if std::fs::symlink_metadata(&directory_link).is_ok() {
            std::fs::remove_dir(&directory_link).unwrap();
        }
        std::fs::remove_file(&file_path).unwrap();
        std::fs::remove_dir(&actual).unwrap();
        std::fs::remove_dir(&fixture).unwrap();
        let (directory, file) =
            outcome.expect("explicit symlink-capable test environment required");
        assert_eq!(directory.unwrap_err(), ProfileError::UnsafeProfilePath);
        assert_eq!(file.unwrap_err(), ProfileError::UnsafeProfilePath);
    }
}
