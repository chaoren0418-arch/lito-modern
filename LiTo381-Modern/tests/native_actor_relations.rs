use lito_modern::modern_lito::{native_actor_relations::equal_ms950,
    native_light_sources::Master, native_login::decode_text, native_world::{self,World}};
const OWN:u32=0xfedcba98;
fn put(id:u32,gfx:u16,name:&[u8],clan:&[u8],master:&[u8],status:u8)->Vec<u8> {
    let mut b=vec![0x0b];
    for v in [32720u16,32780] {b.extend(v.to_le_bytes());}
    b.extend(id.to_le_bytes()); b.extend(gfx.to_le_bytes()); b.extend([0,0,5,0]);
    b.extend(0u32.to_le_bytes()); b.extend(0u16.to_le_bytes());
    b.extend(name); b.extend([0,0,status]); b.extend(0u32.to_le_bytes());
    b.extend(clan); b.push(0); b.extend(master); b.push(0);
    b.extend([0,255,0,1,0]); b.extend(0u16.to_le_bytes()); b.push(0);
    if status&4!=0 {b.extend([0;5]);} b
}
fn notice(kind:u8,first:&[u8],second:&[u8])->Vec<u8> {
    let mut b=vec![0x36,kind]; b.extend(first); b.push(0); b.extend(second); b.push(0); b
}
fn world()->World {
    let fixture:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=World::default(); w.relations.begin(7).unwrap();
    for key in ["world","start","status"] {
        let s=fixture["packets"][key].as_str().unwrap();
        let b:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        assert!(w.accept(&b).unwrap());
    }
    w.accept(&put(OWN,10,b"Player",b"Alpha",b"",0)).unwrap(); w
}
fn flag(w:&World,id:u32)->bool {
    w.actors[&id].relations.flag_1f.resolve(|graphics|Ok(graphics as u8)).unwrap()
}

#[test]
fn ms950_trails_and_duplicate_unicode_encodings_keep_wire_identity() {
    assert!(equal_ms950(b"Player",b"pLAYER"));
    assert!(equal_ms950(&[0xa4,0x41,b'A'],&[0xa4,0x41,b'a']));
    assert!(!equal_ms950(&[0xa4,0x41],&[0xa4,0x61]));
    let first=[0xa2,0xcc]; let second=[0xa4,0x51];
    assert_eq!(decode_text(&first).unwrap(),decode_text(&second).unwrap());
    assert!(!equal_ms950(&first,&second));
    let actor=native_world::actor(&put(2,10,&first,b"Alpha",&second,0)).unwrap();
    assert_eq!(actor.name,actor.master);
    assert_eq!(actor.relations.name,first); assert_eq!(actor.relations.master,second);
    assert_eq!(actor.relations.master_kind(Some(&first)),Master::Other);
    assert_eq!(actor.relations.master_kind(Some(&second)),Master::Own);
}

#[test]
fn constructor_context_distinguishes_owned_pets_status_and_registered_declaration() {
    let mut w=world();
    w.accept(&put(2,10,b"OldPet",b"Alpha",b"Player",0)).unwrap();
    w.accept(&notice(1,b"alpha",b"Beta")).unwrap();
    assert!(!flag(&w,OWN)); // Declaration writes20 only for already registered own clan.
    assert!(!flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,1);
    for (id,clan,master,status,expected) in [
        (3,b"Alpha".as_slice(),b"pLAYER".as_slice(),0,false),
        (4,b"Alpha",b"Someone",0,true),(5,b"Alpha",b"",0,true),
        (6,b"Beta",b"Player",0,true),(7,b"Else",b"",4,true),
        (8,b"Alpha",b"Player",4,true),(9,b"Else",b"",0,false)] {
        w.accept(&put(id,10,b"Actor",clan,master,status)).unwrap();
        assert_eq!(flag(&w,id),expected,"actor{id}");
    }
    assert_eq!(w.actors[&3].relations.master_kind(Some(b"Player")),Master::Own);
    assert_eq!(w.actors[&5].relations.master_kind(Some(b"Player")),Master::None);
}

#[test]
fn duplicate_wars_remove_one_entry_but_clear_existing_actors_each_time() {
    let mut w=world(); w.accept(&put(2,6,b"Enemy",b"Beta",b"",0)).unwrap();
    w.accept(&notice(1,b"Alpha",b"Beta")).unwrap();
    w.accept(&notice(8,b"Beta",b"ALPHA")).unwrap();
    assert_eq!(w.relations.enemies.len(),2); assert!(flag(&w,2));
    w.accept(&notice(2,b"Beta",b"Alpha")).unwrap();
    assert!(w.relations.active); assert_eq!(w.relations.enemies.len(),1);
    assert!(!flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,0);
    // A new put consults the remaining duplicate, restoring the enemy flag.
    w.accept(&put(2,6,b"Enemy",b"Beta",b"",0)).unwrap(); assert!(flag(&w,2));
    w.accept(&notice(4,b"Alpha",b"Beta")).unwrap();
    assert!(!w.relations.active); assert!(w.relations.own_clan.is_empty());
    assert!(!flag(&w,2)); assert_eq!(w.actors[&OWN].relations.side_20,0);
}

#[test]
fn end_war_captures_transition_graphics_instead_of_reclassifying_after_morph() {
    let mut w=world();
    for (id,gfx) in [(2,6),(3,10),(4,11),(5,12),(6,5)] {
        w.accept(&put(id,gfx,b"Enemy",b"Beta",b"",0)).unwrap();
    }
    w.accept(&notice(1,b"Alpha",b"Beta")).unwrap();
    w.accept(&notice(3,b"Alpha",b"Beta")).unwrap();
    for id in [2,4,5] {assert!(!flag(&w,id));}
    for id in [3,6] {assert!(flag(&w,id));}
    w.actors.get_mut(&2).unwrap().graphics=10;
    w.actors.get_mut(&3).unwrap().graphics=6;
    assert!(!flag(&w,2)); assert!(flag(&w,3));
    let snapshot=serde_json::to_value(&w).unwrap();
    assert!(w.actors[&3].relations.flag_1f.resolve(|_|anyhow::bail!("metadata missing")).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(),snapshot);
}

#[test]
fn repeated_put_preserves_side_only_and_resets_prior_flag_transitions() {
    let mut w=world(); w.accept(&put(2,6,b"Enemy",b"Beta",b"",0)).unwrap();
    w.accept(&notice(1,b"Alpha",b"Beta")).unwrap(); assert!(flag(&w,2));
    w.accept(&put(2,6,b"Changed",b"Else",b"",0)).unwrap();
    assert_eq!(w.actors[&2].relations.side_20,2); assert!(!flag(&w,2));
    w.accept(&put(2,6,b"Changed",b"Else",b"",4)).unwrap(); assert!(flag(&w,2));
}

#[test]
fn map_and_scene_replacements_preserve_game_context_but_retirement_and_new_game_reset_it() {
    let mut w=world(); w.accept(&notice(1,b"Alpha",b"Beta")).unwrap();
    let context=w.relations.clone(); w.clear_preserving_generation();
    assert_eq!(w.relations,context); assert!(w.actors.is_empty());
    w.relations.begin(7).unwrap(); assert_eq!(w.relations,context);
    w.accept(&[0x28,0x34,2,1]).unwrap(); assert_eq!(w.relations,context);
    w.relations.retire(); assert_eq!(w.relations.owner(),None); assert!(!w.relations.active);
    w.relations.begin(8).unwrap(); assert_eq!(w.relations.owner(),Some(8));
    assert!(w.relations.enemies.is_empty()); assert!(w.relations.begin(0).is_err());
    assert_eq!(w.relations.owner(),Some(8));
}

#[test]
fn malformed_notice_is_atomic_and_unrelated_or_missing_own_clans_do_not_mutate_context() {
    let mut w=world(); w.accept(&notice(1,b"Alpha",b"Beta")).unwrap();
    let snapshot=serde_json::to_value(&w).unwrap();
    let valid=notice(4,b"Alpha",b"Beta");
    for n in 0..valid.len() {assert!(w.accept(&valid[..n]).is_err()); assert_eq!(serde_json::to_value(&w).unwrap(),snapshot);}
    let mut tail=valid.clone();tail.push(0);assert!(w.accept(&tail).is_err());
    assert!(w.accept(&notice(1,b"Alpha",&[0x81])).is_err());
    for b in [notice(1,b"Other",b"Else"),notice(6,b"Alpha",b"Beta"),notice(7,b"Alpha",b"Beta")] {
        assert!(w.accept(&b).unwrap()); assert_eq!(serde_json::to_value(&w).unwrap(),snapshot);
    }
    w.actors.remove(&OWN); let snapshot=serde_json::to_value(&w).unwrap();
    w.accept(&valid).unwrap(); assert_eq!(serde_json::to_value(&w).unwrap(),snapshot);
}

#[test]
fn transition_budget_rejection_does_not_partially_clear_registered_actors() {
    let mut w=world(); w.accept(&put(2,10,b"Enemy",b"Beta",b"",4)).unwrap();
    for graphics in 0..256 {
        w.actors.get_mut(&2).unwrap().graphics=graphics;
        w.accept(&notice(2,b"Alpha",b"Beta")).unwrap();
    }
    w.actors.get_mut(&2).unwrap().graphics=256;
    let snapshot=serde_json::to_value(&w).unwrap();
    assert!(w.accept(&notice(2,b"Alpha",b"Beta")).is_err());
    assert_eq!(serde_json::to_value(&w).unwrap(),snapshot);
}

#[test]
fn master_update_preserves_byte_identity_even_when_decoded_text_is_unchanged() {
    let mut w=world(); let first=[0xa2,0xcc]; let second=[0xa4,0x51];
    w.accept(&put(OWN,10,&first,b"Alpha",b"",0)).unwrap();
    w.accept(&put(2,10,b"Pet",b"Alpha",&first,4)).unwrap();
    assert_eq!(w.actors[&2].relations.master_kind(Some(&first)),Master::Own);
    let mut update=vec![0x4f]; update.extend(2u32.to_le_bytes()); update.extend(second); update.push(0);
    let old=w.actors[&2].master.clone(); let revision=w.visual_revision;
    w.accept(&update).unwrap(); assert_eq!(w.actors[&2].master,old);
    assert!(w.visual_revision>revision); assert!(flag(&w,2));
    assert_eq!(w.actors[&2].relations.master_kind(Some(&first)),Master::Other);
    let before=serde_json::to_value(&w).unwrap(); update.pop();
    assert!(w.accept(&update).is_err()); assert_eq!(serde_json::to_value(&w).unwrap(),before);
}

#[test]
fn clan_update_sets_own_pet_flag_unlike_constructor_and_empty_clan_clears_side_only() {
    fn update(w:&mut World,clan:&[u8]) {
        let mut b=vec![0x69]; b.extend(2u32.to_le_bytes()); b.extend(clan); b.extend([0,0]); b.extend(0u32.to_le_bytes());
        w.accept(&b).unwrap();
    }
    let mut w=world(); w.accept(&notice(1,b"Alpha",b"Beta")).unwrap();
    w.accept(&put(2,10,b"Pet",b"Alpha",b"Player",0)).unwrap(); assert!(!flag(&w,2));
    update(&mut w,b"ALPHA"); assert!(flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,1);
    update(&mut w,b""); assert!(flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,0);
    update(&mut w,b"Beta"); assert!(flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,2);
    update(&mut w,b"Other"); assert!(flag(&w,2)); assert_eq!(w.actors[&2].relations.side_20,2);
    // S69 must also retain distinct MS950 encodings of the same Unicode name.
    update(&mut w,&[0xa2,0xcc]); let old=w.actors[&2].clan.clone(); let revision=w.visual_revision;
    update(&mut w,&[0xa4,0x51]); assert_eq!(w.actors[&2].clan,old);
    assert_eq!(w.actors[&2].relations.clan,[0xa4,0x51]); assert!(w.visual_revision>revision);
}
