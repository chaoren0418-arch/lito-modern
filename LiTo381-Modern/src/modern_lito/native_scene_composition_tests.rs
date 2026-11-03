use super::*;
use super::super::{native_actor_dependency_draft as actor,native_canonical_draft as native,
    protocol,battle_world,battle_read,battle_follow};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{fs,path::Path};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn png(path:&Path,c:&super::super::compositor::Canvas){let mut e=png::Encoder::new(fs::File::create(path).unwrap(),c.width(),c.height());e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(c.rgba()).unwrap();}
fn sample()->(actor::ActorCodec,Vec<u8>,Vec<u8>){
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));let ptr:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r433-catalog-snapshot.json")).unwrap()).unwrap();
    let cfg:Value=serde_json::from_slice(&fs::read(ptr["input"].as_str().unwrap()).unwrap()).unwrap();let pin=cfg["candidate"]["vectors"].as_array().unwrap().iter().find(|v|v["name"]=="sql92/actor-sql-wire-capture.json").unwrap();
    let bytes=fs::read(pin["path"].as_str().unwrap()).unwrap();assert_eq!(sha(&bytes),pin["sha256"].as_str().unwrap().to_ascii_lowercase());let v:Value=serde_json::from_slice(&bytes).unwrap();
    let ack=protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f|protocol::CAP_EQUIPMENT};let c=actor::ActorCodec::local_experiment(ack,1792).unwrap();
    for e in v["entries"].as_array().unwrap().iter().filter(|e|e["phase"]=="main"&&e["source_id"]==7&&e["request_kind"]==92&&e["outcome"]=="COMPLETE_EXCHANGE"){
        let q=hex(e["request_hex"].as_str().unwrap());let b=hex(e["response_hex"].as_str().unwrap());let r=c.decode_reply(&b,&q).unwrap();if r.catalog().native().canonical.attachment.as_ref().is_some_and(|a|a.lifecycle.phase==super::super::world_protocol::Phase::Loading)&&r.catalog().native().canonical.data.is_some(){assert_eq!(r.request().catalog.native.canonical.context.key.object_id,10275);return(c,q,b);}
    }panic!("Fresh pinned SQL Loading actor absent")
}
fn owner()->(actor::ActorOwner,native::WorkTicket,actor::ActorRendererJob){let (c,q,b)=sample();let r=c.decode_reply(&b,&q).unwrap();let n=r.catalog().native();let req=battle_read::Request{context:n.request.canonical.context,channel_id:n.request.canonical.channel,world:n.canonical.attachment.as_ref().unwrap().world.clone()};let peer=protocol::extensions::Selected::new(protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f|protocol::CAP_EQUIPMENT},0).unwrap();let mut f=battle_follow::Follower::new_selected(req.clone(),peer).unwrap();f.accept(battle_read::UnvalidatedReply{echo:req,state:battle_read::State::Data,data:n.canonical.data.clone()}).unwrap();let o=f.observation().unwrap();let s=native::Scope{connection_incarnation:435,context:n.request.canonical.context,channel:n.request.canonical.channel,joined_room:o.data().room.id,room_revision:o.data().room.revision};let mut owner=actor::ActorOwner::local_experiment(c).unwrap();let t=owner.stage(s,r).unwrap();let j=owner.begin_renderer(t,o).unwrap();(owner,t,j)}
fn inputs(body:BodyPlacement,effects:&[usize])->Inputs{Inputs{roof:Some(RoofPolicy::StaticSourceOnly),lifetime:Some(LifetimePolicy::ExplicitSingleFrameStudy),body:Some(body),insertion:Some(std::iter::once(SceneObject::Body).chain(effects.iter().copied().map(SceneObject::Effect)).collect()),max_frames:Some(2)}}
#[test]fn disabled_unknown_roof_lifetime_and_insertion_refuse(){
    let m=SceneModel::default();assert!(!m.loaded_admission_available());assert!(m.state.is_none());
    for field in 0..5{let mut q=inputs(BodyPlacement::ActualObservation,&[0]);match field{0=>q.roof=None,1=>q.lifetime=None,2=>q.body=None,3=>q.insertion=None,_=>q.max_frames=None};assert!(q.qualify(&[0]).is_err());}
    let mut q=inputs(BodyPlacement::ActualObservation,&[0]);q.roof=Some(RoofPolicy::ActorDrivenRuntime);assert!(q.qualify(&[0]).is_err());
    let mut q=inputs(BodyPlacement::ActualObservation,&[0]);q.lifetime=Some(LifetimePolicy::NaturalRuntime);assert!(q.qualify(&[0]).is_err());
    let mut q=inputs(BodyPlacement::ActualObservation,&[0]);q.insertion=Some(vec![SceneObject::Body,SceneObject::Body]);assert!(q.qualify(&[0]).is_err());
    let mut q=inputs(BodyPlacement::ActualObservation,&[0]);q.insertion=Some(vec![SceneObject::Body,SceneObject::Effect(1)]);assert!(q.qualify(&[0]).is_err());
}
#[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn opaque_scene_binding_reselection_and_retirement_refuse(){
    let (mut o,_t,j)=owner();let other=owner();let n=j.native_job();let b=Binding{cancel:n.cancellation_token(),local:Arc::new(AtomicBool::new(false)),data:n.admitted_observation().data().clone(),plan:n.source_plan().clone()};b.check(n).unwrap();assert!(b.check(other.2.native_job()).is_err());
    let (c,q,r)=sample();let reply=c.decode_reply(&r,&q).unwrap();let data=n.admitted_observation().data();let mut scope=native::Scope{connection_incarnation:436,context:reply.request().catalog.native.canonical.context,channel:reply.request().catalog.native.canonical.channel,joined_room:data.room.id,room_revision:data.room.revision};assert!(o.stage(scope,reply).is_err());b.check(n).unwrap();scope.connection_incarnation=435;o.stage(scope,c.decode_reply(&r,&q).unwrap()).unwrap();assert!(b.check(n).is_err());
    let (mut o,_t,j)=owner();let n=j.native_job();let b=Binding{cancel:n.cancellation_token(),local:Arc::new(AtomicBool::new(false)),data:n.admitted_observation().data().clone(),plan:n.source_plan().clone()};o.retire();assert!(b.check(n).is_err());
}
#[test]fn same_cell_depth_and_newest_first_tie_control_occlusion(){
    use super::super::sprite::{SpriteFrame,SpriteFrameFragments,SpriteBlockReference,CompactSpriteBlock};
    use super::super::world_map::PixelPoint;
    let frame=|depth,rgb:[u8;4]|SpriteFrame{fragments:Some(SpriteFrameFragments{references:vec![SpriteBlockReference{half_x:0,y:0,depth,block_id:0}],atlas:Arc::new(vec![CompactSpriteBlock::test_rgba(&rgb.repeat(576))])}),x_offset:0,y_offset:0,width:24,height:24,frame_type:depth,unknown1:0,unknown2:0,rgba:rgb.repeat(576)};
    let anchor=NativeDrawCell::from_world(WorldCell{x:32768,y:32768}).unwrap();let view=ViewProjection{anchor_cell:WorldCell{x:32768,y:32768},anchor_pixel:PixelPoint{x:24,y:12}};let red=frame(9,[255,0,0,255]);let blue=frame(9,[0,0,255,255]);
    for reverse in [false,true]{let mut c=Canvas::new(24,24).unwrap();let mut r=Raster::new(view,&c);if reverse{r.sprite(anchor,&blue,0,Blend::Over).unwrap();r.sprite(anchor,&red,1,Blend::Over).unwrap();}else{r.sprite(anchor,&red,1,Blend::Over).unwrap();r.sprite(anchor,&blue,0,Blend::Over).unwrap();}let masks=r.paint(&mut c,&AtomicBool::new(false),1).unwrap();assert_eq!(&c.rgba()[..3],if reverse{&[0,0,255]}else{&[255,0,0]});assert_eq!(masks[0].contains(0,0),!reverse);}
    let high=frame(10,[0,0,255,255]);let mut c=Canvas::new(24,24).unwrap();let mut r=Raster::new(view,&c);r.sprite(anchor,&high,0,Blend::Over).unwrap();r.sprite(anchor,&red,1,Blend::Over).unwrap();let masks=r.paint(&mut c,&AtomicBool::new(false),1).unwrap();assert_eq!(&c.rgba()[..3],&[0,0,255]);assert!(!masks[0].contains(0,0));
}
#[test]
#[ignore="real pinned terrain/FirePot/profile1 scene composition; base Sprite stream required"]
fn real_native_scene_interleaving_and_owner_bound_partial(){
    use super::super::{native_resources as r,world_map::PixelPoint};
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));let (mut owner,t,j)=owner();let other=self::owner();
    let ptr:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r432-catalog-snapshot.json")).unwrap()).unwrap();let provenance=Path::new(ptr["candidate"]["root"].as_str().unwrap()).join("provenance");
    let root:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r430-base-body-snapshot.json")).unwrap()).unwrap();let body=j.load_body(Path::new(root["root"].as_str().unwrap()),&provenance).unwrap();
    let text:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-catalog-snapshot.json")).unwrap()).unwrap();let textroot=Path::new(text["root"].as_str().unwrap());let fixture=p.join("target/r425-native-dm564-fixture");let sources=r::NativeSources{map_root:fixture.join("native-map"),archive_root:fixture.join("native-archives")};
    let assets=r::NativeResourceAdapter::pinned_inspection().load_static(&sources,j.source_plan(),Default::default(),j.cancellation()).unwrap();
    let effects:Vec<_>=assets.effect_records().iter().enumerate().map(|(i,e)|(i,NativeActionSnapshot::load_firepot(j.source_plan(),textroot,&sources.archive_root,j.identity().catalog().action_pins(),u32::from(e.effect.sprite_id),j.cancellation()).unwrap())).collect();let ids:Vec<_>=effects.iter().map(|e|e.0).collect();
    assert!(SceneModel::local_experiment(&other.2,&assets,&body,&effects,inputs(BodyPlacement::ActualObservation,&ids)).is_err());
    let e=assets.effect_records()[0];let fxcell=WorldCell{x:i32::from(e.effect.x),y:i32::from(e.effect.y)};let n=j.native_job();let data=n.admitted_observation().data();let a=data.actors.0.iter().find(|a|a.entity==data.personal.entity).unwrap();let actual=WorldCell{x:a.position.x,y:a.position.y};
    let scenarios=[("actual_DATA",BodyPlacement::ActualObservation,actual),("conditional_rear",BodyPlacement::ConditionalCell(WorldCell{x:fxcell.x+1,y:fxcell.y-1}),fxcell),("conditional_same",BodyPlacement::ConditionalCell(fxcell),fxcell),("conditional_front",BodyPlacement::ConditionalCell(WorldCell{x:fxcell.x-1,y:fxcell.y+1}),fxcell)];
    let out=p.join("target/r435c-scene-cpu");fs::create_dir_all(&out).unwrap();let mut rows=Vec::new();let mut occluded=false;
    for (name,placement,camera) in scenarios{
        let view=ViewProjection{anchor_cell:camera,anchor_pixel:PixelPoint{x:400,y:330}};let mut visible_without=0;
        for with_effects in [false,true]{let fx=if with_effects{&effects[..]}else{&[]};let ids:Vec<_>=fx.iter().map(|e|e.0).collect();let mut m=SceneModel::local_experiment(&j,&assets,&body,fx,inputs(placement,&ids)).unwrap();let f=m.render(&j,view,800,600,0,0).unwrap();assert!(f.canvas(other.2.native_job()).is_err());let visible=f.body_visible_pixels(n).unwrap();let c=f.canvas(n).unwrap();let path=out.join(format!("{name}-effects-{with_effects}.png"));png(&path,c);rows.push(json!({"scenario":name,"effects":with_effects,"conditional_body":f.conditional_body,"camera":[camera.x,camera.y],"body_visible_pixels":visible,"visible_terrain":f.visible_terrain,"path":path,"sha256":sha(&fs::read(&path).unwrap()),"RGBA_sha256":sha(c.rgba()),"roof_runtime_qualified":false,"actual_equipment_inferred":false}));if with_effects{occluded|=visible<visible_without;}else{visible_without=visible;}
            assert!(m.render(&j,view,800,600,8,0).is_err());if with_effects{assert!(m.render(&j,view,800,600,0,11).is_err());}m.render(&j,view,800,600,1,1).unwrap();assert!(m.render(&j,view,800,600,0,0).is_err());assert!(f.canvas(n).is_err());m.retire();assert!(f.canvas(n).is_err());
        }
    }
    assert!(occluded,"Source FirePot must occlude body in at least one explicit relative placement");
    let mut m=SceneModel::local_experiment(&j,&assets,&body,&effects,inputs(BodyPlacement::ActualObservation,&ids)).unwrap();let f=m.render(&j,ViewProjection{anchor_cell:actual,anchor_pixel:PixelPoint{x:400,y:330}},800,600,0,0).unwrap();m.retire();assert!(f.canvas(n).is_err());drop(m);
    let catalog=j.verify_catalog(textroot).unwrap();let slices=[38,123].iter().map(|id|effects.iter().find(|(_,e)|e.scene_sprite_id()==*id).unwrap().1.render_frame_index(0,j.cancellation()).unwrap()).collect();let bs=body.render_frame_index(BodyAction::Idle,0).unwrap();let cancel=n.cancellation_token();let completion=j.finish_partial(catalog,body,assets,slices,bs).unwrap();owner.complete_partial(t,completion).unwrap();assert!(owner.partial_available());assert!(!owner.loaded_admission_available());owner.retire();assert!(cancel.load(Ordering::Acquire));
    fs::write(p.join("logs/continuation-20261003/r435-scene-consumer-details.json"),serde_json::to_vec_pretty(&json!({"state":"REAL_NATIVE_CONDITIONAL_SCENE_INTERLEAVING_AND_OPAQUE_PARTIAL_VERIFIED","rows":rows,"images":8,"real_effect_records":effects.len(),"source_effect_cell":[fxcell.x,fxcell.y],"body_actual_DATA_cell":[actual.x,actual.y],"body_occlusion_observed_in_conditional_study":occluded,"foreign_owner_refused":true,"single_interval_frame_bounds_refused":true,"budget_and_retire_revoke_output":true,"opaque_partial_completed":true,"owner_retire_cancels":true,"roof_runtime_qualified":false,"natural_lifetime_qualified":false,"actual_equipment_inferred":false,"original_BIN_executed":false,"LOADED":false,"GUI":0})).unwrap()).unwrap();
}
