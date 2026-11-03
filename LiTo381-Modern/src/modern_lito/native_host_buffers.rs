//! Qualified data-copy semantics, without Native initializer/epoch authority.
//! Capacities are Modern safety bounds; Native local clamps were not proved.
use anyhow::{ensure, Context, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NulBuffer<const N: usize> {
    bytes: [u8; N],
    known_prefix: usize,
    through_nul: Option<usize>,
}
impl<const N: usize> Default for NulBuffer<N> {
    fn default() -> Self {
        Self {
            bytes: [0; N],
            known_prefix: 0,
            through_nul: None,
        }
    }
}
impl<const N: usize> NulBuffer<N> {
    /// Only a bounded prefix is inspected. Source data after the first NUL is
    /// ignored; existing destination tail bytes are left untouched.
    pub fn copy_c_bytes(&mut self, source: &[u8]) -> Result<()> {
        let length = source
            .iter()
            .take(N)
            .position(|b| *b == 0)
            .context("NUL missing within byte capacity")?
            + 1;
        self.bytes[..length].copy_from_slice(&source[..length]);
        self.known_prefix = self.known_prefix.max(length);
        self.through_nul = Some(length);
        Ok(())
    }
    /// Explicit adapter for an already qualified length-delimited payload.
    /// Append its terminator; do not decode or normalize the source bytes.
    pub fn copy_payload(&mut self, source: &[u8]) -> Result<()> {
        ensure!(
            source.len() < N && !source.contains(&0),
            "payload exceeds capacity or contains NUL"
        );
        self.bytes[..source.len()].copy_from_slice(source);
        self.bytes[source.len()] = 0;
        self.known_prefix = self.known_prefix.max(source.len() + 1);
        self.through_nul = Some(source.len() + 1);
        Ok(())
    }
    /// Uninitialized tail bytes are never exposed as observed zeros.
    pub fn known_bytes(&self) -> &[u8] {
        &self.bytes[..self.known_prefix]
    }
    pub fn c_bytes(&self) -> Option<&[u8]> {
        self.through_nul.map(|n| &self.bytes[..n])
    }
    pub fn payload(&self) -> Option<&[u8]> {
        self.through_nul.map(|n| &self.bytes[..n - 1])
    }
}
pub type HostBuffer = NulBuffer<256>;
pub type AliasBuffer = NulBuffer<28>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PortWord(Option<[u8; 2]>);
impl PortWord {
    pub fn write_bits(&mut self, bits: u16) {
        self.0 = Some(bits.to_le_bytes());
    }
    pub fn bytes(self) -> Option<[u8; 2]> {
        self.0
    }
    pub fn bits(self) -> Option<u16> {
        self.0.map(u16::from_le_bytes)
    }
    /// Signed read view only; neither zero nor negative values qualify a socket.
    pub fn signed_view(self) -> Option<i16> {
        self.0.map(i16::from_le_bytes)
    }
}

/// Independent owned storage for saved host64AD74, alias64AD54 and port64AD70.
/// Writing this pure data object cannot qualify a Native startup or connection.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Buffers {
    pub host: HostBuffer,
    pub alias: AliasBuffer,
    pub port: PortWord,
}
impl Buffers {
    /// Optional inputs leave other fields untouched. Validate all supplied
    /// strings on an owned copy before publishing any field.
    pub fn copy_data(
        &mut self,
        host: Option<&[u8]>,
        alias: Option<&[u8]>,
        port: Option<u16>,
    ) -> Result<()> {
        let mut next = self.clone();
        if let Some(host) = host {
            next.host.copy_c_bytes(host)?;
        }
        if let Some(alias) = alias {
            next.alias.copy_c_bytes(alias)?;
        }
        if let Some(port) = port {
            next.port.write_bits(port);
        }
        *self = next;
        Ok(())
    }
}

#[cfg(test)]
#[path = "native_host_buffers_tests.rs"]
mod tests;
