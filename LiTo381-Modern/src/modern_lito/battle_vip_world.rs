//! Explicit full-VIP lifecycle. The ordinary envelope shape is reused, not
//! its command/data admission. A decoded reply is not resource/LOAD or UI proof.
use super::{
    application_protocol as app, battle_payload as payload, battle_world as base,
    protocol::extensions::Selected,
};
use std::io::Read;
pub const REQUEST_KIND: u16 = 84;
pub const RESULT_KIND: u16 = 85;
pub const MAX_RESULT_BYTES: usize = base::MIN_RESULT_BYTES + payload::vip::MAX_APPEARANCE_BYTES;

#[derive(Clone, PartialEq, Eq)]
pub struct Reply(base::Reply);
impl std::fmt::Debug for Reply {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("VipWorldReply[redacted]")
    }
}
impl Reply {
    pub fn echo(&self) -> &base::Request {
        &self.0.echo
    }
    pub fn state(&self) -> base::State {
        self.0.state
    }
    pub fn status(&self) -> super::world_protocol::Status {
        self.0.status
    }
    pub fn reason(&self) -> super::world_protocol::Reason {
        self.0.reason
    }
    pub fn attachment(&self) -> Option<&base::Attachment> {
        self.0.attachment.as_ref()
    }
    pub fn data(&self) -> Option<&payload::UnvalidatedData> {
        self.0.data.as_ref()
    }
    pub(crate) fn validate(&self, peer: Selected) -> Result<(), app::ApplicationError> {
        require(negotiated(peer))?;
        self.0.validate()?;
        if let Some(a) = self.attachment() {
            require(
                a.lifecycle.race_settlement.is_none() && a.lifecycle.race_terminal_revision == 0,
            )?;
        }
        if let Some(d) = self.data() {
            require(
                d.vip.is_some()
                    && d.world.descriptor.features & payload::vip::FEATURE != 0
                    && super::battle_read::data_negotiated_selected(d, peer),
            )?;
        }
        Ok(())
    }
}
pub fn negotiated(peer: Selected) -> bool {
    peer.allows_vip_world() && base::negotiated(&peer.base())
}
pub fn encode_request(
    id: u32,
    q: &base::Request,
    peer: Selected,
) -> Result<Vec<u8>, app::ApplicationError> {
    require(negotiated(peer))?;
    let mut bytes = base::encode_request(id, q, peer.base().version)?;
    bytes[12..14].copy_from_slice(&REQUEST_KIND.to_be_bytes());
    Ok(bytes)
}
pub fn read_reply(
    reader: &mut impl Read,
    expected: &[u8],
    peer: Selected,
) -> Result<Reply, app::ApplicationError> {
    require(negotiated(peer) && expected.len() == base::REQUEST_BYTES)?;
    require(expected[12..14] == REQUEST_KIND.to_be_bytes())?;
    let mut normalized = expected.to_vec();
    normalized[12..14].copy_from_slice(&base::REQUEST_KIND.to_be_bytes());
    let q = base::expected_request(&normalized, peer.base().version)?;
    let reply = Reply(base::read_reply_contract(
        reader,
        expected,
        q,
        RESULT_KIND,
        MAX_RESULT_BYTES,
        |bytes| {
            let data = payload::decode_selected(bytes, peer)?;
            require(
                data.vip.is_some() && data.world.descriptor.features & payload::vip::FEATURE != 0,
            )?;
            Ok(data)
        },
    )?);
    reply.validate(peer)?;
    Ok(reply)
}
fn require(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(app::error(
            app::ErrorKind::MalformedFrame,
            "invalid full VIP world lifecycle",
        ))
    }
}

#[cfg(test)]
#[path = "battle_vip_world_tests.rs"]
mod tests;
