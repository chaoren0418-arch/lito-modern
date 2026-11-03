// Keep private PetRace type relationships within their original module graph.
// Everything else uses the normal library API. No test body is copied or relaxed.
#[allow(unused_imports)]
pub use lito_modern::modern_lito::*;

#[path = "city_runtime.rs"]
pub mod native_city_runtime;

#[path = "../../src/modern_lito/native_backend_485f_evidence.rs"]
mod backend_evidence;
#[path = "../../src/modern_lito/native_multisector_batch_tests.rs"]
mod multisector_batch;
#[path = "../../src/modern_lito/native_pet_race_component.rs"]
pub mod native_pet_race_component;
#[path = "../../src/modern_lito/native_pet_race_terrain.rs"]
pub mod native_pet_race_terrain;
#[path = "../../src/modern_lito/native_pet_race_offline.rs"]
pub mod native_pet_race_offline;
#[path = "../../src/modern_lito/native_pet_race_session.rs"]
pub mod native_pet_race_session;
#[path = "../../src/modern_lito/native_pet_race_quit_session.rs"]
pub mod native_pet_race_quit_session;
#[path = "../../src/modern_lito/native_pet_race_return_profile.rs"]
pub mod native_pet_race_return_profile;
#[path = "../../src/modern_lito/native_pet_race_wire_boundary.rs"]
pub mod native_pet_race_wire_boundary;
#[path = "../../src/modern_lito/native_pet_race_start_component.rs"]
pub mod native_pet_race_start_component;
