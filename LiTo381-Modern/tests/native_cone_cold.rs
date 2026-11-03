//! Request and wire-authority regressions intended for the final normal source.
use lito_modern::modern_lito::{native_skills as skills,native_skill_selection::{Model,Target},native_world::World};
fn packet(bytes:&[u8],key:&str)->Vec<u8>{
    let v:serde_json::Value=serde_json::from_slice(bytes).unwrap();let s=v["packets"][key].as_str().unwrap();
    (0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()
}
fn cold(key:&str)->Vec<u8>{packet(include_bytes!("fixtures/native-cone-cold-current.json"),key)}
fn world()->World{
    let mut w=World::default();
    for key in ["world","start","status","actor","bot"]{w.accept(&packet(include_bytes!("fixtures/native-world-current.json"),key)).unwrap();}
    let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=3;own.x=32735;own.y=32795;
    own.actions.clear();w.actors.insert(3,own);w.status.as_mut().unwrap().object_id=3;
    let mut target=w.actors.remove(&2).unwrap();target.id=1;target.x=32729;target.y=32797;
    target.actions.clear();w.actors.insert(1,target);w.hp=Some([50,50]);w.mp=Some([5000,5000]);
    w.learned_skills.insert(38);w
}
#[test]
fn cold_request_uses_observed_C14_and_six_cell_range_without_spending_authority(){
    let w=world();let before=serde_json::to_value(&w).unwrap();
    assert_eq!(skills::request(&w,38,Some(1)).unwrap(),cold("request"));
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    assert!(skills::request(&w,38,None).is_err());assert!(skills::request(&w,38,Some(3)).is_err());
    let mut far=w.clone();far.actors.get_mut(&1).unwrap().x=32728;
    assert!(skills::request(&far,38,Some(1)).is_err());
    let mut dead=w.clone();dead.actors.get_mut(&1).unwrap().dead=true;
    assert!(skills::request(&dead,38,Some(1)).is_err());
    let mut unknown=w.clone();unknown.learned_skills.remove(&38);
    assert!(skills::request(&unknown,38,Some(1)).is_err());
    let mut low=w.clone();low.mp=Some([2,5000]);
    assert_eq!(skills::request(&low,38,Some(1)).unwrap(),cold("request"));
    assert_eq!(low.mp,Some([2,5000])); // Exact cost/refusal remains the server's authority.
    low.mp=Some([0,5000]);assert!(skills::request(&low,38,Some(1)).is_err());
    assert_eq!(skills::reuse_millis(38),10);
}
#[test]
fn cold_raw_S23_is_atomic_and_defers_hurt_until_effect_arrival(){
    let mut w=world();let raw=cold("attack");let before=serde_json::to_value(&w).unwrap();
    for n in 0..raw.len(){assert!(w.accept(&raw[..n]).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);}
    let mut bad=raw.clone();bad.push(0);assert!(w.accept(&bad).is_err());assert_eq!(serde_json::to_value(&w).unwrap(),before);
    w.accept(&raw).unwrap();assert_eq!(w.own_cast_packets,1);assert_eq!(w.own_attack_packets,0);
    assert_eq!(w.projectiles.len(),1);let cue=&w.projectiles[0];
    assert_eq!(cue.projectile.graphics,11738);assert_eq!(cue.projectile.flight_type,0);
    assert_eq!((cue.attacker,cue.target),(3,1));assert_eq!(cue.target_incarnation,Some(w.actors[&1].incarnation));
    assert!(w.actors[&1].actions.is_empty());assert_eq!(w.mp,Some([5000,5000]));assert_eq!(w.hp,Some([50,50]));
    w.accept(&cold("mana_after")).unwrap();assert_eq!(w.mp,Some([4982,5000]));assert_eq!(w.hp,Some([50,50]));
    w.accept(&[0x28,0x34,2,0]).unwrap();assert!(w.projectiles.is_empty());
}
#[test]
fn cold_selector_rechecks_target_life_generation_owner_and_book(){
    let w=world();let mut model=Model::default();model.sync(7,&w,true);model.select(Some(38)).unwrap();
    assert!(model.rows().iter().find(|r|r.id==38).unwrap().label.contains("冰錐術"));
    let target=Target::capture(&w,1);let choice=model.choice(7,&w,target).unwrap();
    assert_eq!(skills::request(&w,choice.skill,choice.target).unwrap(),cold("request"));
    for change in 0..4{
        let mut replaced=w.clone();
        match change{0=>replaced.actors.get_mut(&1).unwrap().incarnation+=1,1=>replaced.generation+=1,2=>replaced.actors.get_mut(&1).unwrap().dead=true,_=>{replaced.learned_skills.remove(&38);}}
        assert!(model.choice(7,&replaced,target).is_err());
    }
    assert!(model.choice(8,&w,target).is_err());
}
