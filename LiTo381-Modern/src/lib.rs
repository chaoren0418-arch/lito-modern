//! Independently built LiTo client. No TW launch, injection or LHZ automation.
pub mod modern_lito;
// Fixed source copy: extraction SHA and origin are recorded in provenance.
// This is not a path dependency on the active LHZ project.
#[path = "shared_v1/morph_auth.rs"]
pub mod morph_auth;
