//! Exact deployed Protocol.versionRequest / versionAccepted envelope.
use anyhow::{ensure, Result};

pub use super::native_opcodes::{C_VERSION, S_KEY, S_VERSION_CHECK as S_VERSION};
pub const VERSION_MAGIC: u16 = 0x8033;
pub const VERSION_PRODUCT: u32 = 950;
pub const VERSION_BUILD: u16 = 0x14f8;

/// This new client supplies a defined version envelope. No legacy registry,
/// packed pointer, or hardware serial is needed for the server's version check.
pub fn version_request(from_cd: u8) -> [u8; 10] {
    let mut body = [0; 10];
    body[0] = C_VERSION;
    body[1..3].copy_from_slice(&VERSION_MAGIC.to_le_bytes());
    body[3..7].copy_from_slice(&VERSION_PRODUCT.to_le_bytes());
    body[7] = from_cd;
    body[8..10].copy_from_slice(&VERSION_BUILD.to_le_bytes());
    body
}
pub fn seed(body: &[u8]) -> Result<u32> {
    ensure!(
        body.len() == 5 && body[0] == S_KEY,
        "expected exact native S_KEY"
    );
    Ok(u32::from_le_bytes(body[1..5].try_into().unwrap()))
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ServerVersion {
    pub revision: u32,
    pub protocol_minor: u8,
    pub compatibility_words: [u32; 4],
    pub flags: [u8; 2],
    pub country: u8,
    pub server_type: Option<u32>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VersionReply {
    Accepted(ServerVersion),
    Rejected,
}
pub fn parse_version(body: &[u8]) -> Result<VersionReply> {
    ensure!(
        body.len() >= 2 && body[0] == S_VERSION,
        "expected native S_VERSION_CHECK"
    );
    if body[1] == 1 {
        ensure!(body.len() == 2, "trailing native version rejection");
        return Ok(VersionReply::Rejected);
    }
    ensure!(
        body[1] == 0 && body.len() >= 26,
        "invalid native version success selector/body"
    );
    let word = |at: usize| u32::from_le_bytes(body[at..at + 4].try_into().unwrap());
    let revision = word(3);
    let with_type = revision >= 0x9e38;
    ensure!(
        body.len() == if with_type { 30 } else { 26 },
        "native version tail length"
    );
    Ok(VersionReply::Accepted(ServerVersion {
        revision,
        protocol_minor: body[2],
        compatibility_words: [word(7), word(11), word(15), word(19)],
        flags: [body[23], body[24]],
        country: body[25],
        server_type: with_type.then(|| word(26)),
    }))
}
