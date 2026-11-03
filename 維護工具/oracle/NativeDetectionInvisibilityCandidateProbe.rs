//! Unapplied detection proposal; no product Controller or actual cast is run.
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{collections::BTreeSet,path::PathBuf,sync::atomic::AtomicBool};
#[allow(dead_code,unused_imports)]
mod port {
    pub use lito_modern::modern_lito::*;
    pub mod native_skills {
        pub use lito_modern::modern_lito::native_skills::Effect;
        use lito_modern::modern_lito::native_world::World;
        use anyhow::{ensure,Context,Result};
        include!("@REQUEST_EXTRACT@");
    }
    #[path="@SELECTION@"] pub mod native_skill_selection;
    #[path="@EFFECTS@"] pub mod native_skill_effects;
    #[path="@TOWER@"] pub mod tower_assets;
    #[path="@SHARED_CATALOG@"] pub mod shared_action_catalog;
}
use lito_modern::modern_lito::{native_world::World,resources::{SharedArchive,SharedArchiveKind},
    sprite,sprite_actions::ActionKey,native_world_effects::{Target,Mix},
    compositor::Canvas,world_map::{ViewProjection,WorldCell,PixelPoint}};
use port::shared_action_catalog::SharedActionCatalog;
fn hex(bytes:&[u8])->String {bytes.iter().map(|b|format!("{b:02x}")).collect()}
fn packet(v:&Value,key:&str)->Vec<u8>{let s=v["packets"][key].as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()}
fn world(fixture:&Value,mana:&Value)->World {
    let mut w=World::default();for key in ["world","start","status","actor","bot"]{w.accept(&packet(fixture,key)).unwrap();}
    let mut own=w.actors.remove(&0xfedcba98).unwrap();own.id=1;w.actors.insert(1,own);w.status.as_mut().unwrap().object_id=1;
    w.accept(&packet(mana,"mana")).unwrap();w.learned_skills.insert(13);w.learned_skills.insert(60);w
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let a:Vec<_>=std::env::args_os().collect();assert_eq!(a.len(),3);let root=PathBuf::from(&a[1]);let out=PathBuf::from(&a[2]);
    let fixture:Value=serde_json::from_slice(&std::fs::read(out.join("world-fixture.json"))?)?;
    let mana:Value=serde_json::from_slice(&std::fs::read(out.join("skills-fixture.json"))?)?;
    let mut w=world(&fixture,&mana);let before=serde_json::to_value(&w)?;
    assert!(lito_modern::modern_lito::native_skills::request(&w,13,None).is_err());
    let request=port::native_skills::request(&w,13,None)?;assert_eq!(hex(&request),"140104");
    let codec:Value=serde_json::from_slice(&std::fs::read(out.join("C02-selector-codec.json"))?)?;
    let row=codec["rows"].as_array().unwrap().iter().find(|row|row["skill_id"]==13).unwrap();
    assert_eq!(row["request"],hex(&request));assert_eq!(row["S37_codec_wire_gfx"],749);
    assert_eq!(serde_json::to_value(&w)?,before);assert!(port::native_skills::request(&w,13,Some(2)).is_err());
    assert_eq!(port::native_skills::reuse_millis(13),500);
    let mut menu=port::native_skill_selection::Model::default();assert!(menu.sync(11,&w,true));
    assert_eq!(menu.rows().iter().find(|row|row.id==13).unwrap().label,"#13 偵測術（自己）");
    let target=port::native_skill_selection::Target::capture(&w,2).unwrap();
    assert_eq!(port::native_skill_selection::choice(&w,13,Some(target))?.target,None);
    let mut rejected=Vec::new();
    for kind in ["unlearned","zero_mp","zero_hp","dead","not_started","movement_block"] {
        let mut bad=w.clone();match kind {
            "unlearned"=>{bad.learned_skills.remove(&13);},"zero_mp"=>bad.mp=Some([0,10]),"zero_hp"=>bad.hp=Some([0,10]),
            "dead"=>bad.actors.get_mut(&1).unwrap().dead=true,"not_started"=>bad.started=false,
            "movement_block"=>{bad.movement_blocks.insert(1);},_=>unreachable!(),
        }
        let before=serde_json::to_value(&bad)?;assert!(port::native_skills::request(&bad,13,None).is_err());
        assert_eq!(serde_json::to_value(&bad)?,before);rejected.push(kind);
    }
    w.mp=Some([1,10]);assert_eq!(port::native_skills::request(&w,13,None)?,request);w=world(&fixture,&mana);
    assert!(lito_modern::modern_lito::native_skills::request(&w,60,None).is_err());
    let tick_codec:Value=serde_json::from_slice(&std::fs::read(out.join("C02-self-tick-codec.json"))?)?;
    let before_ticks=serde_json::to_value(&w)?;let mut tick_vectors=Vec::new();
    for row in tick_codec["positive"].as_array().unwrap() {
        let tick=row["tick"].as_u64().unwrap();
        let bytes=port::native_skills::self_tick_request(&w,60,None,port::native_skills::ClientTick::from_wrapping_millis(tick))?;
        assert_eq!(hex(&bytes),row["hex"]);assert_eq!(row["skill_id"],60);
        tick_vectors.push(json!({"hex":hex(&bytes),"skill_id":60,"tick":tick}));
    }
    assert_eq!(port::native_skills::ClientTick::from_wrapping_millis(1u64<<32).as_millis(),0);
    assert_eq!(port::native_skills::ClientTick::from_wrapping_millis(u64::MAX).as_millis(),u32::MAX);
    let tick_start=port::native_skills::ClientTick::now()?;
    let timed=port::native_skills::request(&w,60,None)?;
    let tick_end=port::native_skills::ClientTick::now()?;
    assert_eq!(timed.len(),7);assert_eq!(&timed[..3],[0x14,7,3]);
    let tick_sent=u32::from_le_bytes(timed[3..].try_into().unwrap());
    assert!(tick_sent.wrapping_sub(tick_start.as_millis())<=tick_end.as_millis().wrapping_sub(tick_start.as_millis()));
    assert!(port::native_skills::request(&w,60,Some(2)).is_err());
    assert!(port::native_skills::self_tick_request(&w,60,Some(2),tick_start).is_err());
    assert!(port::native_skills::self_tick_request(&w,13,None,tick_start).is_err());
    assert_eq!(port::native_skills::reuse_millis(60),0);
    assert_eq!(menu.rows().iter().find(|row|row.id==60).unwrap().label,"#60 隱身術（自己，消耗 1 顆魔法寶石）");
    assert_eq!(port::native_skill_selection::choice(&w,60,Some(target))?.target,None);
    let mut invisibility_rejected=Vec::new();
    for kind in ["unlearned","zero_mp","zero_hp","dead","not_started","movement_block"] {
        let mut bad=w.clone();match kind {
            "unlearned"=>{bad.learned_skills.remove(&60);},"zero_mp"=>bad.mp=Some([0,10]),"zero_hp"=>bad.hp=Some([0,10]),
            "dead"=>bad.actors.get_mut(&1).unwrap().dead=true,"not_started"=>bad.started=false,
            "movement_block"=>{bad.movement_blocks.insert(1);},_=>unreachable!(),
        }
        let before=serde_json::to_value(&bad)?;
        assert!(port::native_skills::request(&bad,60,None).is_err());
        assert!(port::native_skills::self_tick_request(&bad,60,None,tick_start).is_err());
        assert_eq!(serde_json::to_value(&bad)?,before);invisibility_rejected.push(kind);
    }
    assert_eq!(serde_json::to_value(&w)?,before_ticks);
    let stale=port::native_skill_selection::Target::capture(&w,2).unwrap();
    w.accept(&[0x34,2,0,0,0,1])?;assert!(!w.actor_visible(2));assert!(stale.resolve(&w).is_none());
    assert_eq!(port::native_skill_selection::choice(&w,13,Some(stale))?.target,None);
    assert_eq!(port::native_skill_selection::choice(&w,60,Some(stale))?.target,None);
    w.accept(&[0x34,2,0,0,0,0])?;assert!(w.actor_visible(2));assert!(stale.resolve(&w).is_none());
    w.accept(&[0x34,1,0,0,0,1])?;assert!(w.actor_visible(1));
    assert_eq!(w.actors[&1].draw_state.style(true,0),lito_modern::modern_lito::native_actor_states::Style::Half);
    w=world(&fixture,&mana);
    let cancel=AtomicBool::new(false);let catalog=SharedActionCatalog::load(&root,&cancel)?;
    let animation=catalog.catalog().resolve_action(749,ActionKey::new(0))?.animation;
    let definition=&catalog.catalog().sprites[&749];
    let metadata=json!({"graphics":749,"bound_graphics":definition.graphics_id,"image_count":definition.image_count,
        "attributes":format!("{:?}",definition.attributes),"directionality":format!("{:?}",animation.directionality),
        "frames_per_second":animation.frames_per_second,"frames":animation.frames.iter().map(|f|json!({"image":f.image_id,"subframe":f.subframe,
            "ticks":f.duration_ticks,"events":format!("{:?}",f.events)})).collect::<Vec<_>>()});
    std::fs::write(out.join("original749-definition.json"),serde_json::to_vec_pretty(&metadata)?)?;
    let qualified=port::native_skill_effects::qualify_skills(catalog.catalog())?;
    assert!(qualified.iter().any(|(gfx,_,_)|*gfx==749));
    let bank=port::tower_assets::SharedEffectBank::load_with_graphics(&catalog,&cancel,port::native_skill_effects::qualify_skills,
        &[(871,192),(829,744),(830,744),(831,744),(2510,149)])?;
    let provenance=bank.sources().find(|p|p.name=="749-0.spr").unwrap();
    let mut archive=SharedArchive::open(&root,provenance.archive)?;let bytes=archive.read_entry(&provenance.name)?;
    assert_eq!(Sha256::digest(&bytes).as_slice(),provenance.sha256.as_slice());
    let source_sha=format!("{:X}",Sha256::digest(&bytes));
    let reference=sprite::decode_classic_spr_with_limits(&bytes,Default::default())?;
    w.accept(&[0x37,1,0,0,0,0xed,2])?;let authority=serde_json::to_value(&w)?;
    let position=w.effects.last().unwrap().position;
    let view=ViewProjection{anchor_cell:WorldCell{x:i32::from(position[0]),y:i32::from(position[1])},anchor_pixel:PixelPoint{x:400,y:300}};
    let mut effects=port::native_skill_effects::Effects::default();effects.prepare(&root,&w,&cancel);
    let mut reports=Vec::new();let mut sounds=BTreeSet::new();let mut due=0u64;
    for step in 0..=animation.frames.len() {
        let source=if step<animation.frames.len()-1 {step+1}else{0};
        let mut queue=Vec::new();let (report,events)=effects.draw_to(&w,&mut Target::Fragments(&mut queue),view,due*1_000_000);
        assert!(report["load_error"].is_null(),"{report}");assert_eq!(report["started"],1);assert_eq!(report["omitted"],0);
        for e in events {assert!(sounds.insert(e.event.clone()+&e.frame.to_string()));}
        let finished=step==animation.frames.len();
        if finished {assert_eq!(report["active"],0);assert!(queue.is_empty());}
        else {
            assert_eq!(report["active"],1);assert_eq!(report["effects"][0]["source_frame"],source);
            let frame=&reference.frames[animation.frames[source].subframe as usize];
            assert_eq!(queue.len(),1);assert_eq!(queue[0].frame.rgba,frame.rgba);assert!(matches!(queue[0].mix,Mix::Screen));
            if step<3 {
                let mut canvas=Canvas::new(800,600)?;canvas.clear([32,64,96]);
                Target::Canvas(&mut canvas).submit(queue[0].frame,queue[0].position,view,Mix::Screen)?;
                let file=std::fs::File::create(out.join(format!("detection-{step}.png")))?;let mut encoder=png::Encoder::new(file,800,600);
                encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);encoder.write_header()?.write_image_data(canvas.rgba())?;
            }
            due+=u64::from(animation.frames[source].duration_ticks)*1000/u64::from(animation.frames_per_second);
        }
        assert_eq!(serde_json::to_value(&w)?,authority);reports.push(report);
    }
    let result=json!({"status":"isolated_detection_invisibility_requests_menu_and_original_detection_effect_candidate_passed",
        "self_tick_vectors":tick_vectors,"invisibility_guard_rejections":invisibility_rejected,
        "windows_tick_sample":{"start":tick_start.as_millis(),"sent":tick_sent,"end":tick_end.as_millis()},
        "self_tick_codec_compared":true,"typed_tick_and_target_distinguished":true,"host_u64_wrap_guards_checked":true,
        "synthetic_visibility_packet_and_stale_target_checks":true,"native_original_clock_source_verified":false,
        "request":hex(&request),"request_guard_rejections":rejected,"request_does_not_predict_MP":true,"selector_codec_compared":true,
        "graphics":749,"SPR_name":provenance.name,"SPR_sha256":source_sha,"source_frames":animation.frames.len(),
        "samples":reports,"unique_frame_events":sounds.len(),"bank_encoded_bytes":bank.encoded(),"bank_RGBA_bytes":bank.rgba(),
        "reference_decoder":"existing classic SPR decoder; independent CSharp pixel oracle not executed here",
        "generic_skill_effect_loader_compiled":true,"unrelated_private_TW_prepare_branch_omitted":true,
        "full_candidate_effect_loader_compiled":false,"unchanged_tower_assets_compiled_as_probe_dependency":true,
        "candidate_applied_to_product":false,"actual_formal_sessions":0,"actual_casts":0,"GUI_verified":false,
        "invisible_target_reveal_or_cooldown_runtime_verified":false,"whole_goal_complete":false});
    std::fs::write(out.join("candidate-results.json"),serde_json::to_vec_pretty(&result)?)?;
    println!("detection candidate request/menu/guards and {} original source frames passed",animation.frames.len());Ok(())
}
