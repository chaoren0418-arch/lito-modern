use lito_modern::modern_lito::native_world::World;
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn fixture(file:&[u8],name:&str)->Vec<u8>{let f:serde_json::Value=serde_json::from_slice(file).unwrap();bytes(f["packets"][name].as_str().unwrap())}
fn packet(name:&str)->Vec<u8>{fixture(include_bytes!("fixtures/native-world-metadata-current.json"),name)}
fn base(name:&str)->Vec<u8>{fixture(include_bytes!("fixtures/native-world-current.json"),name)}
fn world()->World{let mut w=World::default();for p in ["world","start","status","actor"]{w.accept(&base(p)).unwrap();}w}
#[test]
fn deployed_master_updates_only_existing_identity_and_retires_on_replacement(){
    let mut w=world();let id=0xfedcba98;let incarnation=w.actors[&id].incarnation;
    w.accept(&packet("master_chinese")).unwrap();assert_eq!(w.actors[&id].master,"測試主人");assert_eq!(w.actors[&id].incarnation,incarnation);
    let rev=w.visual_revision;w.accept(&packet("master_chinese")).unwrap();assert_eq!(w.visual_revision,rev);
    w.accept(&packet("master_empty")).unwrap();assert_eq!(w.actors[&id].master,"");
    let count=w.actors.len();w.accept(&packet("master_companion")).unwrap();assert_eq!(w.actors.len(),count);assert_eq!(w.orphan_updates,1);
    let mut a=w.actors[&id].clone();a.id=u32::MAX;w.actors.insert(a.id,a);w.accept(&packet("master_max")).unwrap();assert_eq!(w.actors[&u32::MAX].master,"a".repeat(63));
    w.accept(&packet("master_chinese")).unwrap();w.accept(&base("actor")).unwrap();assert_ne!(w.actors[&id].incarnation,incarnation);assert_eq!(w.actors[&id].master,"");
    let mut remove=vec![0x15];remove.extend(id.to_le_bytes());w.accept(&remove).unwrap();w.accept(&packet("master_chinese")).unwrap();assert!(!w.actors.contains_key(&id));
}
#[test]
fn observer_bounds_are_half_open_map_scoped_and_do_not_restrict_players(){
    let mut w=world();w.accept(&packet("box_arena")).unwrap();let b=w.observer_bounds.unwrap();
    assert!(b.contains(32704,32768));assert!(b.contains(32767,32831));assert!(!b.contains(32768,32831));assert!(!b.contains(32767,32832));assert!(!b.contains(32703,32768));
    w.accept(&packet("box_single")).unwrap();assert!(w.observer_bounds.unwrap().contains(65534,65534));assert!(w.move_request(0).is_ok());
    w.accept(&packet("box_max")).unwrap();assert_eq!(w.observer_bounds.unwrap().max_exclusive,[65535,65535]);assert!(!w.observer_bounds.unwrap().contains(65535,0));
    w.accept(&base("world")).unwrap();assert!(w.observer_bounds.is_none());assert!(w.actors.is_empty());
}
#[test]
fn malformed_master_and_bounds_are_rejected_atomically(){
    let w=world();let mut invalid=Vec::new();
    for name in ["master_chinese","master_empty","master_max","box_arena","box_max","box_single"]{
        let b=packet(name);for n in 0..b.len(){invalid.push(b[..n].to_vec());}let mut tail=b;tail.push(0);invalid.push(tail);
    }
    for raw in [vec![b'a';64],vec![0x81],vec![b'a',10,b'b'],vec![127],vec![0xff]]{
        let mut b=vec![0x4f];b.extend(0xfedcba98u32.to_le_bytes());b.extend(raw);b.push(0);invalid.push(b);
    }
    let mut zero=packet("master_chinese");zero[1..5].fill(0);invalid.push(zero);
    for coords in [[0u16,0,0,1],[0,0,1,0],[2,0,1,1],[0,2,1,1]]{let mut b=vec![0xe2];for n in coords{b.extend(n.to_le_bytes());}invalid.push(b);}
    for b in invalid{let mut actual=w.clone();let before=serde_json::to_value(&actual).unwrap();assert!(actual.accept(&b).is_err(),"accepted {b:?}");assert_eq!(before,serde_json::to_value(actual).unwrap());}
}
