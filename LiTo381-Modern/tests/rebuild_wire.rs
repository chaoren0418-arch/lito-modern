#[test]
fn current_formal_jar_cipher_matches_native_x64_rx_tx_and_partial_io() {
    let vectors=include_bytes!("fixtures/formal-cipher-0.6.42.json");
    assert_eq!(lito_modern::modern_lito::rebuild_verification::verify_formal_cipher(vectors).unwrap(),24);
}
