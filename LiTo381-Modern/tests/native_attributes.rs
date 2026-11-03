use lito_modern::modern_lito::{native_attributes::{self,Update},native_world::World};
fn bytes(s:&str)->Vec<u8>{(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn fixture()->serde_json::Value{serde_json::from_slice(include_bytes!("fixtures/native-attributes-current.json")).unwrap()}
fn status()->Vec<u8>{let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();bytes(f["packets"]["status"].as_str().unwrap())}
fn world()->World{let mut w=World::default();w.accept(&[0x28,0xf4,1,0]).unwrap();w.accept(&[0xd2]).unwrap();w.accept(&status()).unwrap();w}
#[test]
fn formal_schema_zero_preserves_attribute_order_signed_modifiers_and_unsigned_experience(){
    let f=fixture();for v in f["vectors"].as_array().unwrap(){let b=bytes(v["body"].as_str().unwrap());let mut w=world();let before=w.status.unwrap();let u=native_attributes::parse(&b).unwrap();w.accept(&b).unwrap();let s=w.status.unwrap();assert_eq!(s.object_id,before.object_id);assert_eq!(w.hp,Some([before.current_hp,before.maximum_hp]));
        match u{
            Update::Ability{values,weight}=>{assert_eq!(values,[v["strength"].as_u64().unwrap() as u8,5,6,3,2,4]);assert_eq!([s.strength,s.intelligence,s.wisdom,s.dexterity,s.constitution,s.charisma],values);assert_eq!(s.weight_gauge,weight);assert_eq!(s.armor_class,before.armor_class);},
            Update::Experience{level,experience}=>{assert_eq!(u,Update::Experience{level:v["level"].as_u64().unwrap() as u8,experience:v["experience"].as_u64().unwrap() as u32});assert_eq!((s.level,s.experience),(level,experience));},
            Update::Armor(values)=>{assert_eq!(values,[v["value"].as_i64().unwrap() as i8,-128,127,-1,1]);assert_eq!([s.armor_class,s.fire_resistance,s.water_resistance,s.wind_resistance,s.earth_resistance],values);},
            Update::Magic(values)=>{assert_eq!(values,[v["value"].as_i64().unwrap() as i8,(-v["value"].as_i64().unwrap()-1) as i8]);assert_eq!(w.magic_modifiers,Some(values));assert_eq!(s,before);},
        }assert_eq!(w.attribute_updates.iter().sum::<u64>(),1);
    }
}
#[test]
fn malformed_or_extended_attributes_do_not_partially_change_current_authority(){
    let f=fixture();let w=world();let before=serde_json::to_value(&w).unwrap();let mut invalid=vec![vec![0x51,0,0,0,0,0]];
    for v in f["vectors"].as_array().unwrap(){let b=bytes(v["body"].as_str().unwrap());for n in 0..b.len(){invalid.push(b[..n].to_vec());}let mut tail=b;tail.push(0);invalid.push(tail);}
    for (op,n) in [(0x50,26),(0x51,9),(0x52,9),(0x52,21),(0x56,9)]{let mut b=vec![0;n];b[0]=op;invalid.push(b);}
    for b in invalid{let mut actual=w.clone();assert!(actual.accept(&b).is_err(),"accepted {b:?}");assert_eq!(serde_json::to_value(actual).unwrap(),before);}
}
#[test]
fn identity_and_scene_changes_retire_modifiers_and_unowned_updates_never_create_players(){
    let mut w=world();w.accept(&[0x56,255,127]).unwrap();w.accept(&status()).unwrap();assert_eq!(w.magic_modifiers,Some([-1,127]));
    let mut other=status();other[1..5].copy_from_slice(&123u32.to_le_bytes());w.accept(&other).unwrap();assert!(w.magic_modifiers.is_none());assert_eq!(w.attribute_updates,[0;4]);
    w.accept(&[0x28,0xf4,1,0]).unwrap();w.accept(&[0xd2]).unwrap();w.accept(&[0x56,255,127]).unwrap();assert!(w.status.is_none()&&w.magic_modifiers.is_none());assert_eq!(w.orphan_updates,1);assert_eq!(w.attribute_updates,[0;4]);
}
