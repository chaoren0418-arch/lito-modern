use lito_modern::modern_lito::{native_world::World,native_skill_selection::Target,native_world_labels::Hover,native_combat_animation::Playback,sprite_actions::{ActionLimits,SpriteActionCatalog}};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn packet(name:&str)->Vec<u8>{let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();bytes(f["packets"][name].as_str().unwrap())}
fn world()->World{let mut w=World::default();for p in ["world","start","status","actor","hp","mp"]{w.accept(&packet(p)).unwrap();}w}
fn resurrection(id:u32,graphics:u16)->Vec<u8>{let mut b=vec![0x11];b.extend(id.to_le_bytes());b.push(0);b.extend(id.to_le_bytes());b.extend(graphics.to_le_bytes());b}
#[test]
fn deployed_resurrection_vectors_preserve_unsigned_identity_and_never_infer_health(){
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-lifecycle-current.json")).unwrap();
    for row in f["resurrections"].as_array().unwrap(){
        let mut w=world();let id=row["target"].as_u64().unwrap() as u32;let mut actor=w.own_actor().unwrap().clone();actor.id=id;actor.dead=true;w.actors.insert(id,actor);
        let hp=w.hp;let ratio=w.actors[&id].hp_ratio;w.accept(&bytes(row["body"].as_str().unwrap())).unwrap();let event=w.last_resurrection.unwrap();
        assert_eq!(event.target,id);assert_eq!(event.source,u32::MAX);assert_eq!(u64::from(event.action),row["action"].as_u64().unwrap());assert_eq!(u64::from(event.graphics),row["gfx"].as_u64().unwrap());
        assert!(!w.actors[&id].dead);assert_eq!(w.actors[&id].walk_action,event.action);assert_eq!(w.actors[&id].graphics,event.graphics);assert_eq!(w.hp,hp);assert_eq!(w.actors[&id].hp_ratio,ratio);
    }
}
#[test]
fn resurrected_life_retires_death_playback_selection_and_ack_without_requiring_a_new_map(){
    let mut w=world();let id=0xfedcba98;let generation=w.generation;let target=Target::capture(&w,id).unwrap();let hover=Hover::capture(&w,id).unwrap();let incarnation=w.actors[&id].incarnation;
    let mut death=vec![0x20];death.extend(id.to_le_bytes());death.push(8);w.accept(&death).unwrap();
    w.accept(&[0x0d,0,0,200,0]).unwrap();w.death_confirmation_pending=true;w.last_death_confirmed_action=Some(w.actors[&id].action_sequence);
    use lito_modern::modern_lito::resources::{SharedArchive,SharedArchiveKind};
    let raw=SharedArchive::open(std::path::Path::new(r"D:\lineage開發\0-客戶端"),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let catalog=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();let mut playback=Playback::default();
    assert!(playback.sample(&catalog,&w.actors[&id],0).unwrap().0.is_some());
    w.accept(&resurrection(id,0)).unwrap();assert!(playback.sample(&catalog,&w.actors[&id],10_000_000_000).unwrap().0.is_none());
    assert_eq!(w.generation,generation);assert_ne!(w.actors[&id].incarnation,incarnation);assert!(w.actors[&id].actions.is_empty());assert!(target.resolve(&w).is_none());assert!(hover.resolve(&w).is_none());
    assert!(!w.death_confirmation_pending);assert!(w.last_death_confirmed_action.is_none());assert!(w.move_request(0).is_err(),"HP remains zero until authoritative update");
    w.accept(&packet("hp")).unwrap();assert!(w.move_request(0).is_ok());
    w.accept(&packet("world")).unwrap();assert_eq!(w.resurrection_packets,0);assert!(w.last_resurrection.is_none());
}
#[test]
fn malformed_or_orphan_resurrection_never_creates_or_partially_mutates_an_actor(){
    let w=world();let b=resurrection(0xfedcba98,0);let mut invalid=(0..b.len()).map(|n|b[..n].to_vec()).collect::<Vec<_>>();let mut tail=b.clone();tail.push(0);invalid.push(tail);
    for range in [1..5,6..10]{let mut zero=b.clone();zero[range].fill(0);invalid.push(zero);}
    for b in invalid{let mut actual=w.clone();let before=serde_json::to_value(&actual).unwrap();assert!(actual.accept(&b).is_err());assert_eq!(serde_json::to_value(actual).unwrap(),before);}
    let mut w=w;let count=w.actors.len();w.accept(&resurrection(27,61)).unwrap();assert_eq!(w.actors.len(),count);assert_eq!(w.orphan_updates,1);assert!(!w.actors.contains_key(&27));
}
