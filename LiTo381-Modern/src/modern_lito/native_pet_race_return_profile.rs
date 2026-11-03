//! Separately pinned 5350B36D candidate constructor. It does not substitute the
//! historical 00FC candidate or imply a deployed 485F/live WORLD profile.
use super::native_pet_race_quit_session::{CandidateSession, RETURN_SOURCE_SHA};
use super::native_pet_race_session::Session;
use anyhow::{ensure, Result};
use sha2::{Digest, Sha256};
pub struct ReturnContract {
    _sealed: (),
}
impl ReturnContract {
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        ensure!(
            bytes.len() <= 16384
                && format!("{:X}", Sha256::digest(bytes))
                    == "6A69E16EEEF5A1177BB6826CA18D1AB62AADCFE0227E756C1D1036EF46DEC3A6",
            "unadopted RESULT return profile"
        );
        let v: serde_json::Value = serde_json::from_slice(bytes)?;
        ensure!(
            v["candidate_source"]["sha256"] == RETURN_SOURCE_SHA,
            "RESULT return source binding"
        );
        Ok(Self { _sealed: () })
    }
    pub fn compose_offline(&self, base: Session) -> Result<CandidateSession> {
        CandidateSession::from_return_predecessor(base)
    }
}
#[cfg(test)]
#[path = "native_pet_race_return_profile_tests.rs"]
mod tests;
