//! Original codec receipts into the complete candidate World. No server or local buff arithmetic.
use lito_modern::modern_lito::native_world::World;
fn world()->World {
    let f:serde_json::Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json"))).unwrap();let mut w=World::default();
    for key in ["world","start","status","actor","bot","hp","mp"] {let s=f["packets"][key].as_str().unwrap();let p=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect::<Vec<_>>();assert!(w.accept(&p).unwrap());}w
}
#[test]fn original_ability_packets_update_only_authoritative_fields_and_reject_partial_input_atomically(){
    let mut w=world();let hp=w.hp;let mp=w.mp;let learned=w.learned_skills.clone();let base=w.attribute_updates[0];
    for packet in [vec![0x6c,0xb0,4,15,5],vec![0x6b,0xb0,4,18,50,5],vec![0x6c,0,0,10,1],vec![0x6b,0,0,13,45,1]] {
        let before=serde_json::to_value(&w).unwrap();
        for length in 0..packet.len(){assert!(w.accept(&packet[..length]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
        let mut extra=packet.clone();extra.push(0);assert!(w.accept(&extra).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
        assert!(w.accept(&packet).unwrap());let status=w.status.unwrap();
        if packet[0]==0x6c{assert_eq!(status.dexterity,packet[3]);}else{assert_eq!(status.strength,packet[3]);assert_eq!(status.weight_gauge,packet[4]);}
        assert_eq!(w.hp,hp);assert_eq!(w.mp,mp);assert_eq!(w.learned_skills,learned);
    }
    assert_eq!(w.attribute_updates[0],base+4);assert!(w.ability_buffs.is_empty());
}
#[test]fn ability_receipts_retire_on_same_map_new_world_and_new_status_owner(){
    let mut w=world();assert!(w.accept(&[0x6c,0xb0,4,15,5]).unwrap());assert!(w.accept(&[0x6b,0xb0,4,18,50,5]).unwrap());assert_eq!(w.ability_buffs.len(),2);
    let mut status=world().status.unwrap();let original=status.object_id;status.object_id=original.wrapping_add(1);
    // Reuse the exact original status envelope; only replace its explicit object identity.
    let f:serde_json::Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json"))).unwrap();
    let s=f["packets"]["status"].as_str().unwrap();let mut p=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect::<Vec<_>>();p[1..5].copy_from_slice(&status.object_id.to_le_bytes());
    assert!(w.accept(&p).unwrap());assert!(w.ability_buffs.is_empty());
    assert!(w.accept(&[0x6c,0xb0,4,15,5]).unwrap());let map=w.map_id.unwrap();let old=w.generation;
    let mut packet=vec![0x28];packet.extend_from_slice(&map.to_le_bytes());packet.push(0);assert!(w.accept(&packet).unwrap());
    assert_eq!(w.generation,old+1);assert!(w.ability_buffs.is_empty());assert!(w.status.is_none());
    let before=serde_json::to_value(&w).unwrap();assert!(!w.accept(&[0x6c,0xb0,4,15,5]).unwrap());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    assert!(w.accept(&[0xd2]).unwrap());let orphan=w.orphan_updates;let attributes=w.attribute_updates;
    assert!(w.accept(&[0x6c,0xb0,4,15,5]).unwrap());assert!(w.ability_buffs.is_empty());assert!(w.status.is_none());
    assert_eq!(w.orphan_updates,orphan+1);assert_eq!(w.attribute_updates,attributes);
}
