//! Issue #147: offline packets and original-resource rendering; no game/service.
use lito_modern::modern_lito::{
    native_attack_many,
    native_login::{LoginController, LoginPhase, Snapshot},
    native_skill_selection::{Model, Target},
    native_skills as skills, native_world_effects,
    world_map::{PixelPoint, ViewProjection, WorldCell},
};
use std::time::{Duration, Instant};
const ITEM: u32 = 0xfedcba98;
const IDS: [u16; 8] = [44,45,46,47,48,50,51,53];
fn fixture(data: &[u8], key: &str) -> Vec<u8> {
    let value: serde_json::Value = serde_json::from_slice(data).unwrap();
    let text = value["packets"][key].as_str().unwrap();
    (0..text.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&text[i..i + 2], 16).unwrap())
        .collect()
}
fn state() -> Snapshot {
    let mut s = LoginController::default().snapshot().clone();
    for key in ["world", "start", "status", "actor", "bot"] {
        s.world
            .accept(&fixture(
                include_bytes!("fixtures/native-world-current.json"),
                key,
            ))
            .unwrap();
    }
    let mut own = s.world.actors.remove(&ITEM).unwrap();
    own.id = 1;
    own.x = 32770;
    own.y = 32818;
    s.world.actors.insert(1, own);
    s.world.status.as_mut().unwrap().object_id = 1;
    let actor = s.world.actors.get_mut(&2).unwrap();
    actor.x = 32771;
    actor.y = 32818;
    s.phase = LoginPhase::WorldReady;
    s.game_connected = true;
    s.selected_character = Some("Batch3".into());
    s.selected_source = Some("fixture".into());
    s.world.hp = Some([50, 50]);
    s.world.mp = Some([100, 100]);
    // Formal S1E learned-book bits, all eight IDs and no fabricated learned state.
    s.world.accept(&[0x1e,7,0,0,0,0,0,0xf8,0x16]).unwrap();
    s.inventory.begin(7,"Batch3").unwrap();
    s
}
fn effect(id: u32, gfx: u16) -> Vec<u8> {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b
}
fn many(gfx: u16) -> Vec<u8> {
    let mut b = vec![0x39, 18];
    b.extend(1u32.to_le_bytes());
    b.extend(32770u16.to_le_bytes());
    b.extend(32818u16.to_le_bytes());
    b.push(2);
    b.extend(9u32.to_le_bytes());
    b.extend(gfx.to_le_bytes());
    b.extend([0, 0, 0]);
    b.extend(2u16.to_le_bytes());
    b.extend(2u32.to_le_bytes());
    b.push(0x20);
    b.extend(3u32.to_le_bytes());
    b.push(0);
    b
}

fn cast(s:&Snapshot,id:u16)->anyhow::Result<Vec<u8>> {
    match id {
        51=>skills::summon_request(s,7,&skills::SummonTarget::capture(s,7,7)?),
        53=>skills::request(&s.world,id,None),
        _=>skills::request(&s.world,id,Some(2)),
    }
}
fn projectile(gfx:u16)->Vec<u8> {
    let mut b=vec![0x23,18];b.extend(1u32.to_le_bytes());b.extend(2u32.to_le_bytes());
    b.extend([1,2]);b.extend(17u32.to_le_bytes());b.extend(gfx.to_le_bytes());b.push(0);
    for v in [32770u16,32818,32773,32818]{b.extend(v.to_le_bytes());}b.extend([0,0]);b
}
fn hit(s:&Snapshot)->lito_modern::modern_lito::native_combat_animation::EnteredEvent {
    lito_modern::modern_lito::native_combat_animation::EnteredEvent{prepared_audio:None,actor:1,sequence:s.world.actors[&1].action_sequence,action:18,frame:1,event:"Hit".into(),sound:None}
}
fn view()->ViewProjection {ViewProjection{anchor_cell:WorldCell{x:32770,y:32818},anchor_pixel:PixelPoint{x:400,y:300}}}
fn message(s:&mut Snapshot,code:u16){let mut b=vec![0x10];b.extend(code.to_le_bytes());b.push(0);s.world.accept(&b).unwrap();}
fn pending(s:&Snapshot,now:Instant)->skills::Pending {skills::Pending{since:now,casts:s.world.own_cast_packets,messages:s.world.message_count,generation:s.world.generation,incarnation:s.world.own_actor().unwrap().incarnation}}

#[test]
fn eight_wire_shapes_are_exact_and_costs_are_not_predicted() {
    let mut s=state();s.world.hp=Some([1,50]);s.world.mp=Some([1,100]);
    assert_eq!(s.world.learned_skills.iter().copied().collect::<Vec<_>>(),IDS);
    let before=serde_json::to_value(&s.world).unwrap();
    for (id,prefix) in [(44,[0x14,5,3]),(45,[0x14,5,4]),(46,[0x14,5,5]),(47,[0x14,5,6]),(48,[0x14,5,7]),(50,[0x14,6,1]),(51,[0x14,6,2]),(53,[0x14,6,4])] {
        let mut expected=prefix.to_vec();
        match id {51=>expected.extend(7u32.to_le_bytes()),53=>{},_=>expected.extend(2u32.to_le_bytes())}
        if matches!(id,45|46|47|50){expected.extend([0x03,0x80,0x32,0x80]);}
        assert_eq!(cast(&s,id).unwrap(),expected,"{id}");
    }
    assert_eq!(serde_json::to_value(&s.world).unwrap(),before);
    assert!(skills::request(&s.world,51,None).is_err());
    assert!(skills::request(&s.world,51,Some(7)).is_err());
    assert!(skills::request(&s.world,53,Some(2)).is_err());
    for id in [44,48] {let b=skills::request(&s.world,id,None).unwrap();assert_eq!(&b[3..],&1u32.to_le_bytes());}
    assert_eq!(skills::reuse_millis(44),500);assert_eq!(skills::reuse_millis(45),100);
    assert_eq!(skills::reuse_millis(51),1000);assert_eq!(skills::reuse_millis(53),1500);
    assert_eq!(skills::client_clock_millis(50),1000);
}
#[test]
fn caster_and_target_gates_preserve_authority() {
    for id in IDS {for reason in 0..7 {
        let mut s=state();match reason {0=>{s.world.learned_skills.remove(&id);},1=>s.world.hp=Some([0,50]),2=>s.world.mp=None,3=>s.world.mp=Some([0,100]),4=>s.world.actors.get_mut(&1).unwrap().dead=true,5=>{s.world.movement_blocks.insert(1);},_=>{s.world.actors.remove(&1);}}
        let before=serde_json::to_value(&s.world).unwrap();assert!(cast(&s,id).is_err(),"{id}/{reason}");assert_eq!(serde_json::to_value(&s.world).unwrap(),before);
    }}
    for id in [44,45,46,47,48,50] {for reason in 0..3 {
        let mut s=state();match reason {0=>s.world.actors.get_mut(&2).unwrap().dead=true,1=>{s.world.actors.remove(&2);},_=>s.world.actors.get_mut(&2).unwrap().x+=30}
        assert!(cast(&s,id).is_err(),"{id}/{reason}");
    }}
}
#[test]
fn ranges_menu_and_stale_target_intent_are_enforced() {
    for (id,range) in [(44,3),(45,8),(46,3),(47,3),(50,6)] {
        let mut s=state();let a=s.world.actors.get_mut(&2).unwrap();a.x=32770+range;a.y=32818+range;
        assert!(cast(&s,id).is_ok());s.world.actors.get_mut(&2).unwrap().x+=1;assert!(cast(&s,id).is_err());
    }
    let mut s=state();let mut menu=Model::default();menu.sync(7,&s.world,true);
    assert_eq!(menu.rows().len(),8);assert!(menu.rows().iter().all(|r|!r.label.contains("尚未支援")));
    menu.select(Some(46)).unwrap();let target=Target::capture(&s.world,2).unwrap();
    assert!(menu.choice_at(7,&s.world,Some(target),None).is_ok());
    s.world.actors.get_mut(&2).unwrap().incarnation+=1;
    assert!(menu.choice_at(7,&s.world,Some(target),None).is_err());
    menu.select(Some(53)).unwrap();assert!(menu.choice_at(7,&s.world,Some(target),None).is_ok());
    assert!(menu.choice_at(8,&s.world,None,None).is_err());
}
#[test]
fn all_summon_choices_require_explicit_current_inventory_and_world() {
    let s=state();let mut unique=std::collections::BTreeSet::new();
    for row in skills::SUMMON_SELECTIONS {
        assert!(unique.insert(row.selection));let t=skills::SummonTarget::capture(&s,7,row.selection).unwrap();
        let bytes=skills::summon_request(&s,7,&t).unwrap();assert_eq!(&bytes[3..],&row.selection.to_le_bytes());
    }
    assert_eq!(unique.len(),31);assert!(unique.contains(&274));
    for invalid in [0,1,2,6,19,u32::MAX]{assert!(skills::SummonTarget::capture(&s,7,invalid).is_err());}
    let t=skills::SummonTarget::capture(&s,7,7).unwrap();assert!(skills::summon_request(&s,8,&t).is_err());
    for reason in 0..10 {let mut changed=s.clone();match reason {
        0=>changed.inventory.revision+=1,1=>changed.inventory.owner.as_mut().unwrap().selection+=1,
        2=>changed.world.generation+=1,3=>changed.world.local_corrections+=1,
        4=>changed.world.actors.get_mut(&1).unwrap().incarnation+=1,5=>changed.world.map_id=Some(9),
        6=>changed.game_connected=false,7=>changed.observer=true,8=>changed.phase=LoginPhase::LobbyReady,
        _=>{changed.world.movement_blocks.insert(1);}}
        assert!(skills::summon_request(&changed,7,&t).is_err(),"{reason}");
    }
}
#[test]
fn summon_late_rejection_survives_cast_action_and_unrelated_messages() {
    for code in [299,319,353,743] {
        let mut s=state();let now=Instant::now();let p=pending(&s,now);
        s.world.accept(&[0x20,1,0,0,0,18]).unwrap();
        assert_eq!(p.observe_summon(&s.world,now),skills::Feedback::CastAction);
        assert!(p.hold_summon(skills::Feedback::CastAction,now+Duration::from_millis(1999)));
        message(&mut s,code);message(&mut s,161);
        assert_eq!(p.observe_summon(&s.world,now),skills::Feedback::Rejected(code));
        s.world.generation+=1;assert_eq!(p.observe_summon(&s.world,now),skills::Feedback::Interrupted);
    }
    let s=state();let now=Instant::now();let p=pending(&s,now);
    assert_eq!(p.observe_summon(&s.world,now+Duration::from_secs(2)),skills::Feedback::NoResponse);
    assert!(!p.hold_summon(skills::Feedback::CastAction,now+Duration::from_secs(2)));
}
#[test]
fn all_receive_profiles_are_atomic_and_do_not_grant_status_or_costs() {
    for b in [effect(2,870),effect(2,2228),effect(2,2176),effect(1,2510),effect(1,755),projectile(129),projectile(1811),projectile(756),many(758)] {
        let original=state();
        for n in 0..b.len(){let mut s=original.clone();let before=serde_json::to_value(&s.world).unwrap();assert!(s.world.accept(&b[..n]).is_err());assert_eq!(serde_json::to_value(&s.world).unwrap(),before);}
        let mut s=original.clone();s.world.accept(&b).unwrap();
        assert_eq!(s.world.hp,original.world.hp);assert_eq!(s.world.mp,original.world.mp);
        assert_eq!(s.world.movement_blocks,original.world.movement_blocks);assert_eq!(s.world.actors.len(),2);
        if b[0]==0x23 {assert_eq!(s.world.projectiles.len(),1);}
        if b[0]==0x39 {assert_eq!(s.world.many_attacks.len(),1);}
    }
    let mut s=state();let p=pending(&s,Instant::now());message(&mut s,79);
    assert_eq!(p.observe_item(&s.world,Instant::now()),skills::Feedback::Rejected(79));
    assert_eq!(p.observe(&s.world,Instant::now()),skills::Feedback::Pending);
    s.world.accept(&[0x25,2]).unwrap();
    s.world.accept(&effect(1,870)).unwrap();
    assert!(s.world.movement_blocks.contains(&2)); // Cancellation graphics cannot clear a lock.
    s.world.accept(&[0x25,3]).unwrap();assert!(s.world.movement_blocks.is_empty());
    s.world.accept(&[0x0d,1,0,50,0]).unwrap();s.world.accept(&[0x4d,3,0,100,0]).unwrap();
    assert_eq!(s.world.hp,Some([1,50]));assert_eq!(s.world.mp,Some([3,100]));
}
#[test]
#[ignore="original resource memory rendering; no GUI or sound device"]
fn original_object_effects_use_exact_banks_and_blends() {
    use lito_modern::modern_lito::native_skill_effects::Effects;
    let root=std::path::Path::new(r"D:\lineage開發\0-客戶端");let cancel=std::sync::atomic::AtomicBool::new(false);
    for (gfx,sound) in [(870,Some(140)),(2228,Some(843)),(2176,Some(800)),(2510,None),(755,None)] {
        let mut s=state();s.world.accept(&effect(2,gfx)).unwrap();let mut fx=Effects::default();fx.prepare(root,&s.world,&cancel);
        let mut audio=Vec::new();let mut drawn=false;
        for step in 0..401 {
            let mut fragments=Vec::new();let (r,e)=fx.draw_to(&s.world,&mut native_world_effects::Target::Fragments(&mut fragments),view(),step*10_000_000);
            assert!(r["load_error"].is_null(),"{gfx}: {r}");assert_eq!(r["omitted"],0,"{r}");
            for f in &fragments {assert!(match gfx{2228=>matches!(f.mix,native_world_effects::Mix::Multiply),2510=>matches!(f.mix,native_world_effects::Mix::Over),_=>matches!(f.mix,native_world_effects::Mix::Screen)});}
            drawn|=!fragments.is_empty();audio.extend(e.into_iter().filter_map(|e|e.sound));
            if step==400 {assert_eq!(r["active"],0);}
        }
        assert!(drawn);assert_eq!(audio,sound.into_iter().collect::<Vec<_>>());
    }
}
#[test]
#[ignore="original resource memory rendering; no GUI or sound device"]
fn original_tornado_is_one_four_image_group_and_hit_list() {
    let mut s=state();s.world.accept(&many(758)).unwrap();let e=hit(&s);
    let mut fx=native_attack_many::Effects::default();fx.prepare(std::path::Path::new(r"D:\lineage開發\0-客戶端"),&s.world,&std::sync::atomic::AtomicBool::new(false));
    let mut hits=Vec::new();let mut audio=Vec::new();let mut drawn=false;
    for step in 0..501 {
        hits.extend(fx.update(&s.world,std::iter::once(&e),step*10_000_000));
        let mut fragments=Vec::new();let (r,sounds)=fx.draw_to(&mut native_world_effects::Target::Fragments(&mut fragments),view(),step*10_000_000);
        assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["omitted"],0,"{r}");assert_eq!(r["started"],1);
        if !fragments.is_empty(){assert_eq!(fragments.len(),4);drawn=true;
            assert_eq!(fragments.iter().map(|f|[f.position.x,f.position.y]).collect::<Vec<_>>(),vec![[32772,32820],[32768,32816],[32768,32820],[32772,32816]]);}
        audio.extend(sounds.into_iter().filter_map(|e|e.sound));if step==500 {assert_eq!(r["active"],0);}
    }
    assert!(drawn);assert_eq!(hits.len(),1);assert_eq!(hits[0].target,2);assert_eq!(audio,vec![125]);
}
#[test]
#[ignore="original resource memory rendering; no GUI or sound device"]
fn original_sunburst_uses_wire_destination_and_original_hit_timing() {
    use lito_modern::modern_lito::native_class1::Effects;
    let mut s=state();s.world.accept(&projectile(1811)).unwrap();let e=hit(&s);let mut fx=Effects::default();
    fx.prepare(std::path::Path::new(r"D:\lineage開發\0-客戶端"),&s.world,&std::sync::atomic::AtomicBool::new(false));
    let mut audio=Vec::new();let mut hits=Vec::new();let mut drawn=false;let mut diagnostic_hits=0;
    for step in 0..401 {hits.extend(fx.update(&s.world,std::iter::once(&e),step*10_000_000));
        let mut fragments=Vec::new();let (r,sounds)=fx.draw_to(&mut native_world_effects::Target::Fragments(&mut fragments),view(),step*10_000_000);
        assert!(r["load_error"].is_null(),"{r}");assert_eq!(r["omitted"],0,"{r}");assert_eq!(r["started"],1);
        diagnostic_hits=r["hits"].as_u64().unwrap();
        for f in &fragments {assert_eq!(f.position,WorldCell{x:32773,y:32818});}drawn|=!fragments.is_empty();audio.extend(sounds.into_iter().filter_map(|e|e.sound));
        if step==400{assert_eq!(r["active"],0);}
    }assert!(drawn);assert_eq!(audio,vec![709]);assert!(hits.is_empty());assert_eq!(diagnostic_hits,1);
}
#[test]
#[ignore="original resource memory rendering; no GUI or sound device"]
fn original_lance_is_silent_and_eruption_propagates_with_paired_shadows() {
    use lito_modern::modern_lito::native_spell_projectiles::Projectiles;
    let root=std::path::Path::new(r"D:\lineage開發\0-客戶端");let cancel=std::sync::atomic::AtomicBool::new(false);
    for gfx in [756,129] {
        let mut s=state();s.world.actors.get_mut(&2).unwrap().x=32773;s.world.accept(&projectile(gfx)).unwrap();let e=hit(&s);
        let mut fx=Projectiles::default();fx.prepare(root,&s.world,&cancel);
        let mut audio=Vec::new();let mut drawn=false;let mut final_draw=false;
        for step in 0..1501 {
            assert!(fx.update(&s.world,std::iter::once(&e),step*10_000_000).is_empty());
            let mut fragments=Vec::new();let (r,sounds)=fx.draw_to(&mut native_world_effects::Target::Fragments(&mut fragments),view(),step*10_000_000);
            assert!(r["load_error"].is_null(),"{r}");assert!(r["eruption"]["load_error"].is_null(),"{r}");
            assert_eq!(r["omitted"],0,"{r}");assert_eq!(r["eruption"]["omitted"],0,"{r}");
            drawn|=!fragments.is_empty();audio.extend(sounds.into_iter().filter_map(|e|e.sound));
            if gfx==129 {
                assert_eq!(fragments.len()%2,0);for pair in fragments.chunks(2){assert!(matches!(pair[0].mix,native_world_effects::Mix::Over));assert!(matches!(pair[1].mix,native_world_effects::Mix::Shadow));assert_eq!(pair[0].position,pair[1].position);}
                if let Some(casts)=r["eruption"]["casts"].as_array(){for cast in casts {for n in cast["nodes"].as_array().unwrap(){final_draw|=n["final_draw"]==true;}}}
            }
            if step==1500 {assert_eq!(r["active"],0);assert_eq!(r["eruption"]["active"],0);}
        }
        assert!(drawn);if gfx==756{assert!(audio.is_empty());}else{assert_eq!(audio.iter().filter(|&&v|v==128).count(),3);assert_eq!(audio.iter().filter(|&&v|v==129).count(),3);assert_eq!(audio.iter().filter(|&&v|v==3080).count(),1);assert!(final_draw);}
    }
}
