//! Isolated packet/selection guards. No server, resource rendering or cast result.
use lito_modern::modern_lito::{native_world,native_obstacles,world_map};
use lito_modern::modern_lito::native_skills;
use lito_modern::modern_lito::native_skill_selection;
use lito_modern::modern_lito::native_ability_buffs;
use native_world::World;
fn world()->World {
    let v:serde_json::Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-world-current.json"))).unwrap();
    let mut w=World::default();
    for key in ["world","start","status","actor","bot","hp","mp"] {
        let s=v["packets"][key].as_str().unwrap();let p=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect::<Vec<_>>();assert!(w.accept(&p).unwrap());
    }
    let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=1;own.x=32729;own.y=32797;w.actors.insert(1,own);w.status.as_mut().unwrap().object_id=1;
    let other=w.actors.get_mut(&2).unwrap();other.x=32735;other.y=32800;w.learned_skills.extend([26,42]);w
}
#[test]fn current_formal_ability_codec_vectors_and_short_or_extended_packets(){
    let data:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-ability-codecs.json")).unwrap();assert_eq!(data["level_ability_enabled"],false);
    for row in data["cases"].as_array().unwrap(){
        let s=row["hex"].as_str().unwrap();let packet=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect::<Vec<_>>();
        let update=native_ability_buffs::parse(&packet).unwrap();assert_eq!(u64::from(update.received_seconds),row["seconds"].as_u64().unwrap());assert_eq!(u64::from(update.score),row["ability"].as_u64().unwrap());assert_eq!(u64::from(update.kind),row["type"].as_u64().unwrap());
        assert_eq!(update.weight_gauge.map(u64::from),row.get("weight").and_then(|v|v.as_u64()));
        for length in 0..packet.len(){assert!(native_ability_buffs::parse(&packet[..length]).is_err());}
        let mut extra=packet.clone();extra.push(0);assert!(native_ability_buffs::parse(&extra).is_err());
    }
}
#[test]fn both_ability_buffs_use_explicit_self_or_selected_object_without_authority_writes(){
    let w=world();let before=serde_json::to_value(&w).unwrap();
    for (skill,row)in [(26,3),(42,5)] {
        assert_eq!(native_skills::request(&w,skill,None).unwrap(),[0x14,row,1,1,0,0,0]);
        assert_eq!(native_skills::request(&w,skill,Some(2)).unwrap(),[0x14,row,1,2,0,0,0]);
    }
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
}
#[test]fn ability_buffs_reject_unlearned_dead_missing_and_outside_screen_targets(){
    for skill in [26,42] {
        for change in 0..6 {
            let mut w=world();match change {
                0=>{w.learned_skills.remove(&skill);},1=>w.mp=Some([0,10]),2=>w.hp=Some([0,50]),
                3=>{w.actors.remove(&2);},4=>w.actors.get_mut(&2).unwrap().dead=true,
                _=>{let target=w.actors.get_mut(&2).unwrap();target.x=32800;target.y=32800;}
            }
            let before=serde_json::to_value(&w).unwrap();assert!(native_skills::request(&w,skill,Some(2)).is_err(),"skill {skill}, change {change}");assert_eq!(serde_json::to_value(&w).unwrap(),before);
        }
    }
}
#[test]fn ability_choice_preserves_object_and_fences_reused_actor_and_world(){
    use native_skill_selection::{Target,choice,Model};
    for skill in [26,42] {
        let mut w=world();let target=Target::capture(&w,2).unwrap();assert_eq!(choice(&w,skill,Some(target)).unwrap().target,Some(2));assert_eq!(choice(&w,skill,None).unwrap().target,None);
        let mut selection=Model::default();selection.sync(7,&w,true);selection.select(Some(skill)).unwrap();
        assert!(!selection.rows().iter().find(|r|r.id==skill).unwrap().label.contains("尚未"));
        w.actors.get_mut(&2).unwrap().incarnation+=1;assert!(choice(&w,skill,Some(target)).is_err());
        let fresh=Target::capture(&w,2).unwrap();w.generation+=1;assert!(choice(&w,skill,Some(fresh)).is_err());assert!(selection.choice(7,&w,None).is_err());
    }
}
#[test]fn ability_buffs_keep_native_screen_asymmetric_edges(){
    for skill in [26,42] {
        let mut w=world();let own=w.own_actor().unwrap();let origin=[own.x,own.y];
        for (delta,accepted)in [([9i32,9i32],true),([10,-9],true),([10,9],false),([10,10],false),([-10,-9],false)] {
            let a=w.actors.get_mut(&2).unwrap();a.x=(i32::from(origin[0])+delta[0])as u16;a.y=(i32::from(origin[1])+delta[1])as u16;
            assert_eq!(native_skills::request(&w,skill,Some(2)).is_ok(),accepted,"skill {skill}, delta {delta:?}");
        }
    }
}
