use super::*;

#[test]
fn preview_rejects_ordinary_observation_before_any_resource_io() {
    let data = crate::modern_lito::battle_payload::data_tests::fixture();
    let ordinary = crate::modern_lito::battle_scene::tests::observation(data);
    let cancel = AtomicBool::new(false);
    assert!(Preview::load(
        Path::new("C:\\nonexistent-lito-preview-root"),
        ordinary,
        None,
        &cancel
    )
    .err()
    .unwrap()
    .to_string()
    .contains("VIP"));
}
