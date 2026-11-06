//! Explicit isolated E149 runner only: no GUI or production endpoint.
use lito_modern::modern_lito::native_peer_probe;
use std::{net::SocketAddr, path::PathBuf};

#[test]
#[ignore = "requires the owned E149 memory harness; run tools/run-item-sessions.ps1"]
fn formal_item_session() {
    let peer: SocketAddr = std::env::var("LITO_ITEM_TEST_PEER").expect("owned peer").parse().unwrap();
    assert!(peer.ip().is_loopback() && peer.port()!=0);
    let root = PathBuf::from(std::env::var("LITO_NATIVE_PEER_ROOT").unwrap());
    let profile = std::env::var("LITO_NATIVE_PEER_PROFILE").unwrap();
    assert!(matches!(profile.as_str(),"item-heal-194"|"item-heal-197"|"item-stream"|"item-stream-no-reagent"|"item-stream-no-mana"));
    let handles: Vec<_> = [("a","PeerFixture"),("b","PeerFixture2")].into_iter().map(|(role,account)| {
        let root=root.clone();
        std::thread::spawn(move || {
            let result = native_peer_probe::run(peer,account,"TestOnlyPassword",1,role);
            match &result {
                Ok(value) => std::fs::write(root.join(format!("{role}-client.json")),serde_json::to_vec_pretty(value).unwrap()).unwrap(),
                Err(error) => std::fs::write(root.join(format!("{role}-error.txt")),format!("{error:#}")).unwrap(),
            }
            result.map(|v| { assert_eq!(v["worker_quiescent"],true); assert_eq!(v["shutdown"],"closed"); })
        })
    }).collect();
    let results:Vec<_>=handles.into_iter().map(|h| h.join().expect("peer thread")).collect();
    for result in results { result.expect("formal E149 item flow"); }
}
