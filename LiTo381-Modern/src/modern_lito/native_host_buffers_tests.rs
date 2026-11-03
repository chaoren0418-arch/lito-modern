use super::super::super::entry_transfer::State as TransferState;
use super::super::FixtureStartupHost;
use super::*;
use std::net::{Ipv4Addr, SocketAddrV4};

fn bounds<const N: usize>() {
    let mut buffer = NulBuffer::<N>::default();
    assert!(buffer.c_bytes().is_none());
    assert!(buffer.known_bytes().is_empty());
    for payload in 0..=N {
        let mut source = vec![0xfe; payload];
        source.push(0);
        source.extend_from_slice(&[0x81, 0x40, 0, 9]);
        let before = buffer.clone();
        if payload < N {
            buffer.copy_c_bytes(&source).unwrap();
            assert_eq!(buffer.payload().unwrap(), &source[..payload]);
            assert_eq!(buffer.c_bytes().unwrap(), &source[..payload + 1]);
        } else {
            assert!(buffer.copy_c_bytes(&source).is_err());
            assert_eq!(buffer, before);
        }
    }
    let before = buffer.clone();
    for source in [vec![], vec![0xff; N - 1], vec![0xff; N], vec![0xff; N + 1]] {
        assert!(buffer.copy_c_bytes(&source).is_err());
        assert_eq!(buffer, before);
    }
    assert!(buffer.copy_payload(&vec![1; N]).is_err());
    assert!(buffer.copy_payload(&[1, 0, 2]).is_err());
    assert_eq!(buffer, before);
    buffer.copy_payload(&vec![0x80; N - 1]).unwrap();
    assert_eq!(buffer.c_bytes().unwrap().len(), N);
}

#[test]
fn modern_host_20261007_host256_alias28_bounds_raw_and_tail() {
    bounds::<256>();
    bounds::<28>();
    let mut host = HostBuffer::default();
    let mut source = vec![0x81, 0x40, 0xff, 0xfe, 0, 0x66, 0];
    host.copy_c_bytes(&source).unwrap();
    assert_eq!(host.known_bytes(), &[0x81, 0x40, 0xff, 0xfe, 0]);
    source.fill(9);
    assert_eq!(host.payload(), Some(&[0x81, 0x40, 0xff, 0xfe][..]));
    host.copy_c_bytes(&[0x82, 0, 7, 8, 9]).unwrap();
    assert_eq!(host.c_bytes(), Some(&[0x82, 0][..]));
    assert_eq!(host.known_bytes(), &[0x82, 0, 0xff, 0xfe, 0]);
    // Even arbitrarily long input after its first NUL is not copied or scanned.
    let mut tail = vec![0; 1024];
    tail[0] = 0xab;
    host.copy_c_bytes(&tail).unwrap();
    assert_eq!(host.payload(), Some(&[0xab][..]));
}

#[test]
fn modern_host_20261007_independent_fields_atomic_rejection_and_port_bits() {
    let mut data = Buffers::default();
    data.copy_data(None, None, None).unwrap();
    assert_eq!(data, Buffers::default());
    let mut source_host = vec![0xf1, 0x80, 0];
    data.copy_data(Some(&source_host), Some(&[0xff, 0x81, 0]), Some(0xff80))
        .unwrap();
    source_host[0] = 0;
    assert_eq!(data.host.payload(), Some(&[0xf1, 0x80][..]));
    assert_eq!(data.alias.payload(), Some(&[0xff, 0x81][..]));
    let before = data.clone();
    assert!(data
        .copy_data(Some(b"new\0"), Some(&[0x61; 28]), Some(2000))
        .is_err());
    assert_eq!(data, before);
    assert!(data
        .copy_data(Some(&[0x62; 256]), Some(b"alias\0"), Some(1))
        .is_err());
    assert_eq!(data, before);
    data.copy_data(None, Some(b"\0unread"), None).unwrap();
    assert_eq!(data.host, before.host);
    assert_eq!(data.port, before.port);
    assert_eq!(data.alias.payload(), Some(&[][..]));
    assert!(Buffers::default().alias.payload().is_none());
    let host = data.host.clone();
    let alias = data.alias.clone();
    for bits in 0..=u16::MAX {
        data.port.write_bits(bits);
        assert_eq!(data.port.bits(), Some(bits));
        assert_eq!(data.port.bytes(), Some(bits.to_le_bytes()));
        assert_eq!(data.port.signed_view(), Some(bits as i16));
    }
    assert_eq!((data.host, data.alias), (host, alias));
}

#[test]
fn modern_host_20261007_existing_synthetic_startup_parser_uses_buffer() {
    let fixture = FixtureStartupHost::parse(
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"127.0.0.2"}"#,
    )
    .unwrap();
    assert_eq!(fixture.raw_host().c_bytes(), Some(&b"127.0.0.2\0"[..]));
    assert_eq!(fixture.raw_host().known_bytes(), b"127.0.0.2\0");
    assert_eq!(
        fixture.target(),
        SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 2000)
    );
    assert_eq!(fixture.clone().raw_host(), fixture.raw_host());
    for raw in [
        br#"{"kind":"native","host":"127.0.0.2"}"#.as_slice(),
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"localhost"}"#,
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"192.0.2.1"}"#,
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"127.000.0.2"}"#,
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"127.0.0.2\u0000tail"}"#,
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"127.0.0.2","alias":"127.0.0.2"}"#,
        br#"{"kind":"modern_synthetic_startup_host_v1","host":"\u00ff"}"#,
    ] {
        assert!(FixtureStartupHost::parse(raw).is_err());
    }
    assert!(FixtureStartupHost::parse(&vec![b' '; 513]).is_err());
}

#[test]
fn modern_host_20261007_existing_transfer_start_alias_return_and_atomic_error() {
    let mut state = TransferState::default();
    assert!(state.alias().is_none());
    let target = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0xff80);
    let mut name = vec![0x81; 27];
    state.start(&name, Some(target)).unwrap();
    assert_eq!(state.alias(), Some(name.as_slice()));
    assert_eq!(state.key.byte_storage.alias.c_bytes().unwrap().len(), 28);
    assert_eq!(state.key.selected_name.as_deref(), state.alias());
    assert_eq!(state.key.byte_storage.port.bits(), Some(0xff80));
    assert!(state.key.byte_storage.host.c_bytes().is_none());
    name.fill(2);
    assert_eq!(state.alias().unwrap(), &[0x81; 27]);
    let old_tail = state.key.byte_storage.alias.known_bytes().to_vec();
    state.start(&[0xfe, 0xff], None).unwrap();
    assert_eq!(state.alias(), Some(&[0xfe, 0xff][..]));
    assert_eq!(
        &state.key.byte_storage.alias.known_bytes()[3..],
        &old_tail[3..]
    );
    let before = state.key.clone();
    let reconnect = state.reconnect_after_write;
    for invalid in [&[3; 28][..], &[3, 0, 4][..]] {
        assert!(state.start(invalid, Some(target)).is_err());
        assert_eq!(state.key, before);
        assert_eq!(state.reconnect_after_write, reconnect);
    }
    let fallback = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 2), 2000);
    state.return_two(fallback, target);
    assert_eq!(state.key.byte_storage.port.bits(), Some(2000));
    assert_eq!(state.alias(), Some(&[0xfe, 0xff][..]));
    assert!(state.key.byte_storage.host.payload().is_none());
    state.key.revision = u64::MAX;
    let exhausted = state.key.clone();
    assert!(state.start(b"new", Some(target)).is_err());
    assert_eq!(state.key, exhausted);
    // Existing Modern retirement clears its owned projection, not Native globals.
    state.key.revision = 4;
    state.retire().unwrap();
    assert_eq!(state.key.byte_storage, Buffers::default());
    assert!(state.alias().is_none());
}
