use super::*;
use aes::cipher::BlockEncrypt;
const XML: &[u8] = br#"<?xml version="1.0"?><effectlist count="2"><effect id="0" icon="5397" graphic="21" duration="-1"/><effect id="1" icon="1562" offIcon="5286"/></effectlist>"#;
fn encrypted(bytes: &[u8]) -> Vec<u8> {
    let mut out = bytes.to_vec();
    out[0] = b'X';
    let aes = Aes128::new_from_slice(&KEY).unwrap();
    let mut previous = IV;
    let mut chunks = out[4..].chunks_exact_mut(16);
    for b in &mut chunks {
        for i in 0..16 {
            b[i] ^= previous[i];
        }
        let mut block = <&[u8; 16]>::try_from(&*b).unwrap().to_owned().into();
        aes.encrypt_block(&mut block);
        b.copy_from_slice(&block);
        previous.copy_from_slice(b);
    }
    for (b, k) in chunks.into_remainder().iter_mut().zip(previous) {
        *b ^= k;
    }
    out
}
#[test]
fn effect_identity_does_not_alias_icon_or_graphic_and_encryption_preserves_tail() {
    let c = AtomicBool::new(false);
    for padding in 0..32 {
        let mut plain = XML.to_vec();
        plain.extend(vec![b' '; padding]);
        let a = EffectCatalog::parse(&plain, &c).unwrap();
        let b = EffectCatalog::parse(&encrypted(&plain), &c).unwrap();
        assert_eq!(a.entries, b.entries);
        assert_eq!(a.xml_sha256(), b.xml_sha256());
        assert_ne!(a.source_sha256(), b.source_sha256());
        assert!(!a.encrypted() && b.encrypted());
        let entry = a.get(EffectId::from_wire(1).unwrap()).unwrap();
        assert_eq!(entry.icon, 1562);
        assert_eq!(entry.off_icon, Some(5286));
        assert!(a.get(EffectId::from_wire(1562).unwrap()).is_err());
        assert!(a.get(EffectId::from_wire(21).unwrap()).is_err());
    }
}
#[test]
fn malformed_ambiguous_external_and_unbounded_catalogs_are_rejected() {
    let xml = std::str::from_utf8(XML).unwrap();
    for bad in [
        xml.replace("count=\"2\"", "count=\"1\""),
        xml.replace("id=\"1\"", "id=\"0\""),
        xml.replace("icon=\"1562\"", "icon=\"-1\""),
        xml.replace("icon=\"1562\"", "icon=\"4294967296\""),
        xml.replace("icon=\"1562\"", "icon=\"1000000\""),
        xml.replace("id=\"1\"", "id=\"2\""),
        xml.replace("icon=\"1562\"", "icon=\"1562\" icon=\"41\""),
        xml.replace("<effectlist count", "<effectlist xmlns=\"evil\" count"),
        xml.replace("count=\"2\"", "count=\"+2\""),
        xml.replace("<effect id=\"1\"", "<other id=\"1\""),
        xml.replace("</effectlist>", "<nested/></effectlist>"),
        xml.replace("</effectlist>", "x</effectlist>"),
        xml.replace("<effectlist", "<!DOCTYPE effectlist><effectlist"),
        xml.replace(
            "<effectlist",
            "<!DOCTYPE effectlist [<!ENTITY ext SYSTEM 'file:///C:/missing'>]><effectlist",
        ),
        xml.replace("icon=\"1562\"", "icon=\"&ext;\""),
        xml.replace("offIcon=\"5286\"/>", "offIcon=\"5286\"><effect/></effect>"),
    ] {
        assert!(
            EffectCatalog::parse(bad.as_bytes(), &AtomicBool::new(false)).is_err(),
            "{bad}"
        );
    }
    assert!(EffectCatalog::parse(&vec![b' '; MAX_BYTES + 1], &AtomicBool::new(false)).is_err());
    assert!(EffectCatalog::parse(XML, &AtomicBool::new(true)).is_err());
    assert!(EffectCatalog::load(Path::new("effectlist2.xml"), &AtomicBool::new(false)).is_err());
    for value in [-1, i64::MAX, i64::from(MAX_EFFECTS)] {
        assert!(EffectId::from_wire(value).is_err());
    }
    for end in 0..XML.len() - 1 {
        assert!(EffectCatalog::parse(&XML[..end], &AtomicBool::new(false)).is_err());
    }
}
#[test]
#[ignore = "explicit reference plain/encrypted Lineage XML and actual shared status TBT; no native UI or installed XML deployment"]
fn actual_lineage_effect_catalog_matches_encrypted_reference_and_shared_status_pictures() {
    let c = AtomicBool::new(false);
    let path = std::path::PathBuf::from(std::env::var_os("LITO_EFFECTLIST_REFERENCE").unwrap());
    let encrypted =
        std::path::PathBuf::from(std::env::var_os("LITO_EFFECTLIST_ENCRYPTED_REFERENCE").unwrap());
    let root = std::path::PathBuf::from(std::env::var_os("LITO_SHARED_RESOURCE_ROOT").unwrap());
    let plain = EffectCatalog::load(&path, &c).unwrap();
    let encoded = EffectCatalog::load(&encrypted, &c).unwrap();
    assert_eq!(plain.entries, encoded.entries);
    assert_eq!(plain.xml_sha256(), encoded.xml_sha256());
    assert_eq!(plain.entries.len(), 538);
    assert_eq!(
        format!("{:X}", Sha256::digest(std::fs::read(&path).unwrap())),
        "07EFD481DD89B63DA0BE8AF0F1643416A3E875B69F4B7FFFB8BE0683EC05C130"
    );
    assert_eq!(plain.get(EffectId(41)).unwrap().icon, 1562);
    assert_eq!(plain.get(EffectId(379)).unwrap().graphic, Some(747));
    assert_eq!(plain.get(EffectId(379)).unwrap().icon, 5265);
    let ids = [0, 41, 379, 499, 502, 503, 504, 505, 514, 537];
    let bank = super::super::vip_status::Assets::load(&root, encoded, &ids, &c).unwrap();
    for id in ids {
        let picture = bank.picture(EffectId::from_wire(id).unwrap()).unwrap();
        assert!(picture.rgba().chunks_exact(4).any(|p| p[3] == 255));
        assert_ne!(picture.graphic as i64, id);
        if id == 505 {
            assert_eq!((picture.width, picture.height), (32, 31));
        }
        println!(
            "EFFECT_PICTURE: id={id} picture={} source={:?}",
            picture.graphic, picture.source
        );
    }
    println!("EFFECT_CATALOG:538 identical plain/encrypted rows;10 actual mapped shared pictures; explicit reference paths,not deployed XML/full UI");
}
