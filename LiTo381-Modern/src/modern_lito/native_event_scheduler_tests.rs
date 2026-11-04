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
fn owner()->(actor::ActorOwner,native::WorkTicket,actor::ActorRendererJob){let (c,q,b)=sample();let r=c.decode_reply(&b,&q).unwrap();let n=r.catalog().native();let req=battle_read::Request{context:n.request.canonical.context,channel_id:n.request.canonical.channel,world:n.canonical.attachment.as_ref().unwrap().world.clone()};let peer=protocol::extensions::Selected::new(protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f|protocol::CAP_EQUIPMENT},0).unwrap();let mut f=battle_follow::Follower::new_selected(req.clone(),peer).unwrap();f.accept(battle_read::UnvalidatedReply{echo:req,state:battle_read::State::Data,data:n.canonical.data.clone()}).unwrap();let o=f.observation().unwrap();let s=native::Scope{connection_incarnation:434,context:n.request.canonical.context,channel:n.request.canonical.channel,joined_room:o.data().room.id,room_revision:o.data().room.revision};let mut owner=actor::ActorOwner::local_experiment(c).unwrap();let t=owner.stage(s,r).unwrap();let j=owner.begin_renderer(t,o).unwrap();(owner,t,j)}
fn inputs()->ConditionalInputs{ConditionalInputs{object_id:Some(0x3fffffff),player_global:Some(10275),anchor_ms:Some(0),queue:Some(vec![]),current_action:Some(0),base_action:Some(0),frame:Some(0),modifier_1c:Some(0),modifier_21:Some(0),action21_counter:Some(0),ordinary_action_path:Some(true),idle_guards_clear:Some(true),initial:Some(InitialEvent::GeneralIdleTransition),frozen_external_state_study:Some(true)}}
fn explicit_queue(actions:Vec<u8>)->ConditionalInputs{let mut q=inputs();q.queue=Some(actions);q.initial=Some(InitialEvent::ExplicitOpcode32{delay_ms:0});q}

#[test]fn integer_order_and_literal_queue_cfg(){
    assert_eq!(nonplayer_duration(83,3,0,0,0,0).unwrap()+2,85);
    assert_eq!(nonplayer_duration(83,3,2,0,0,0).unwrap()+2,43);
    assert_eq!(nonplayer_duration(166,3,0,0,0,0).unwrap()+2,168);
    for count in [4,8,16,32]{assert_eq!(nonplayer_duration(83,3,count,0,0,0).unwrap(),41);}
    // Sequential truncation differs from a guessed combined multiplier.
    assert_eq!(nonplayer_duration(9,3,0,1,1,0).unwrap(),4); //9→6→4, not floor(9*9/16)=5
    assert_eq!(nonplayer_duration(83,21,2,1,3,1).unwrap(),8); //41→14→10→8
    assert_eq!(nonplayer_duration(83,3,0,17,4,0).unwrap(),83); //action3 not movement set
    assert_eq!(nonplayer_duration(83,0,0,17,4,0).unwrap(),46); //83→62→46
    assert_eq!(nonplayer_duration(83,3,0,2,5,0).unwrap(),62); //83→166→62
    assert_eq!(nonplayer_duration(1,3,2,0,0,0).unwrap(),0);
    assert!(nonplayer_duration(32768,3,0,0,0,0).is_err());
}
#[test]fn sampled_tick_fifo_removal_precedes_callback_and_same_tick_insert(){
    let mut q=TimerQueue::default();q.insert(100,Callback::Opcode32).unwrap();q.insert(100,Callback::FrameAdvance).unwrap();
    let a=q.take_due(100).unwrap();assert_eq!(a.callback,Callback::Opcode32);assert_eq!(q.items.len(),1);
    // Callback enqueues a due-now item; existing equal-time entry stays first.
    q.insert(100,Callback::Opcode32).unwrap();assert_eq!(q.take_due(100).unwrap().callback,Callback::FrameAdvance);assert_eq!(q.items.len(),1);
    assert_eq!(q.take_due(100).unwrap().callback,Callback::Opcode32);assert!(q.items.is_empty());assert!(q.take_due(100).is_none());
}
#[test]fn default_off_and_unknown_initial_values_refuse(){
    let mut disabled=EventModel::default();assert!(disabled.next_due().is_err());assert!(disabled.dispatch_tick(0).is_err());assert!(!disabled.loaded_admission_available());
    for field in 0..14{let mut q=inputs();match field{0=>q.object_id=None,1=>q.player_global=None,2=>q.anchor_ms=None,3=>q.queue=None,4=>q.current_action=None,5=>q.base_action=None,6=>q.frame=None,7=>q.modifier_1c=None,8=>q.modifier_21=None,9=>q.action21_counter=None,10=>q.ordinary_action_path=None,11=>q.idle_guards_clear=None,12=>q.frozen_external_state_study=None,_=>q.initial=None};assert!(q.qualify().is_err(),"{field}");}
    let mut q=inputs();q.player_global=q.object_id;assert!(q.qualify().is_err());let mut q=inputs();q.queue=Some(vec![18]);assert!(q.qualify().is_err());
}
#[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn opcode32_idle_transition_and_first_end_are_bounded(){
    let (_owner,_t,j)=owner();let mut m=EventModel::local_experiment(j.native_job(),inputs(),vec![83;11],16).unwrap();assert_eq!(m.next_due().unwrap(),Some(584));assert!(m.dispatch_tick(583).unwrap().is_empty());
    let first=m.dispatch_tick(584).unwrap();let s=first[0].step();assert_eq!((s.action,s.frame,s.actor_queue_after_pop,s.timer_queue_after_removal,s.next_due_ms),(3,1,0,0,Some(669)));
    let mut frames=vec![1];while let Some(t)=m.next_due().unwrap(){let e=m.dispatch_tick(t).unwrap();frames.push(e[0].step().frame);}
    assert_eq!(frames,vec![1,2,3,4,5,6,7,8,9,10,0]);assert!(!m.loaded_admission_available());assert!(m.dispatch_tick(2000).unwrap().is_empty());
    let mut q=inputs();q.current_action=Some(3);let m=EventModel::local_experiment(j.native_job(),q,vec![83;11],16).unwrap();assert_eq!(m.next_due().unwrap(),Some(83));
}
#[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn post_pop_queue_and_late_callback_compensation(){
    let (_owner,_t,j)=owner();let mut m=EventModel::local_experiment(j.native_job(),explicit_queue(vec![3,3,3]),vec![83;11],16).unwrap();let e=m.dispatch_tick(0).unwrap();assert_eq!((e[0].step().actor_queue_after_pop,e[0].step().next_due_ms),(2,Some(43)));
    let mut m=EventModel::local_experiment(j.native_job(),explicit_queue(vec![3]),vec![166;8],16).unwrap();let e=m.dispatch_tick(0).unwrap();assert_eq!(e[0].step().next_due_ms,Some(168));let e=m.dispatch_tick(180).unwrap();assert_eq!(e[0].step().next_due_ms,Some(336)); //166-(180-168)+2=156
    let e=m.dispatch_tick(900).unwrap();assert_eq!(e[0].step().next_due_ms,Some(903)); //clamp remaining delay1, then+2
}
#[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn replacement_retirement_and_budget_drain_bound_events(){
    let (mut owner,_t,j)=owner();let mut m=EventModel::local_experiment(j.native_job(),inputs(),vec![83;11],1).unwrap();let _event=m.dispatch_tick(584).unwrap();assert!(m.dispatch_tick(669).is_err());assert!(m.next_due().is_err());
    let mut m=EventModel::local_experiment(j.native_job(),inputs(),vec![83;11],16).unwrap();owner.retire();assert!(m.dispatch_tick(584).is_err());assert!(m.session.as_ref().unwrap().timers.items.is_empty());
    let (_owner,_t,j)=self::owner();let mut m=EventModel::local_experiment(j.native_job(),inputs(),vec![83;11],16).unwrap();m.retire();assert!(m.next_due().is_err());assert!(m.session.as_ref().unwrap().timers.items.is_empty());
    let mut q=inputs();q.anchor_ms=Some(u32::MAX-100);assert!(EventModel::local_experiment(j.native_job(),q,vec![83;11],16).is_err());
}

#[test]
#[ignore="real pinned FirePot and profile1 body conditional event CPU sequence; base Sprite stream required"]
fn real_assets_conditional_event_sequence_and_opaque_partial(){
    use super::super::{native_resources as r,native_actor_inspection::BodyAction};
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));let (mut owner,t,j)=owner();let other=self::owner();
    let ptr:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r432-catalog-snapshot.json")).unwrap()).unwrap();let provenance=Path::new(ptr["candidate"]["root"].as_str().unwrap()).join("provenance");
    let bodyroot:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r430-base-body-snapshot.json")).unwrap()).unwrap();let body=j.load_body(Path::new(bodyroot["root"].as_str().unwrap()),&provenance).unwrap();
    let text:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-catalog-snapshot.json")).unwrap()).unwrap();let textroot=Path::new(text["root"].as_str().unwrap());let fixture=p.join("target/r425-native-dm564-fixture");let sprite=fixture.join("native-archives");
    // A matching job/cancel token does not authorize another body's object or
    // substitute an effect clock for the actual body catalog.
    let mut wrong_object=EventModel::local_experiment(j.native_job(),explicit_queue(vec![3]),body.duration_ms(BodyAction::Idle).unwrap(),16).unwrap();
    assert!(wrong_object.dispatch_tick(0).unwrap()[0].render_body(j.native_job(),&body).is_err());
    let mut wrong_clock_inputs=explicit_queue(vec![3]);wrong_clock_inputs.object_id=Some(10275);wrong_clock_inputs.player_global=Some(20000);
    let mut wrong_clock=EventModel::local_experiment(j.native_job(),wrong_clock_inputs,vec![83;11],16).unwrap();
    assert!(wrong_clock.dispatch_tick(0).unwrap()[0].render_body(j.native_job(),&body).is_err());
    let out=p.join("target/r434e-conditional-event-cpu");fs::create_dir_all(&out).unwrap();let mut rows=Vec::new();let mut images=Vec::new();let mut effect_slices=Vec::new();
    for id in [38,123]{let effect=NativeActionSnapshot::load_firepot(j.source_plan(),textroot,&sprite,j.identity().catalog().action_pins(),id,j.cancellation()).unwrap();let mut m=EventModel::local_experiment(j.native_job(),inputs(),effect.duration_ms().unwrap(),16).unwrap();let mut count=0;
        while let Some(tick)=m.next_due().unwrap(){for event in m.dispatch_tick(tick).unwrap(){assert!(event.render_firepot(other.2.native_job(),&effect).is_err());let slice=event.render_firepot(j.native_job(),&effect).unwrap();let s=event.step();assert_eq!(slice.sample().3 as usize,s.frame);let hash=sha(slice.canvas().rgba());if count==0||s.frame==0{let path=out.join(format!("firepot-{id}-step-{}-frame-{}.png",s.sequence,s.frame));png(&path,slice.canvas());images.push(json!({"path":path,"sha256":sha(&fs::read(&path).unwrap()),"rgba_sha256":hash,"kind":"effect","sprite":id,"step":s}));}rows.push(json!({"asset":"FirePot","sprite":id,"step":s,"RGBA_sha256":hash,"clock_domain":"explicit conditional relative anchor0, original runtime not observed"}));count+=1;if s.frame==0{effect_slices.push(slice);}}}
        assert_eq!(count,11);m.retire();assert!(m.next_due().is_err());
    }
    let mut q=explicit_queue(vec![3]);q.object_id=Some(10275);q.player_global=Some(20000);let mut m=EventModel::local_experiment(j.native_job(),q,body.duration_ms(BodyAction::Idle).unwrap(),16).unwrap();let mut last=None;let mut last_token=None;let mut body_count=0;
    while let Some(tick)=m.next_due().unwrap(){for event in m.dispatch_tick(tick).unwrap(){assert!(event.render_body(other.2.native_job(),&body).is_err());let slice=event.render_body(j.native_job(),&body).unwrap();assert_eq!(slice.sample().2,event.step().frame);let hash=sha(slice.canvas().rgba());if body_count==0||event.step().frame==0{let path=out.join(format!("body-step-{}-frame-{}.png",event.step().sequence,event.step().frame));png(&path,slice.canvas());images.push(json!({"path":path,"sha256":sha(&fs::read(&path).unwrap()),"rgba_sha256":hash,"kind":"body","step":event.step()}));}rows.push(json!({"asset":"PrinceIdle","step":event.step(),"RGBA_sha256":hash,"events_retained_unplayed":slice.events().len(),"actual_equipment_inferred":false}));last=Some(slice);last_token=Some(event);body_count+=1;}}
    assert_eq!(body_count,8);let held=last_token.unwrap();m.retire();assert!(held.render_body(j.native_job(),&body).is_err());
    // Already rendered source slices retain their original job binding; event
    // selection never changes the established opaque partial qualification.
    let catalog=j.verify_catalog(textroot).unwrap();let sources=r::NativeSources{map_root:fixture.join("native-map"),archive_root:sprite};let assets=r::NativeResourceAdapter::pinned_inspection().load_static(&sources,j.source_plan(),Default::default(),j.cancellation()).unwrap();
    let token=j.native_job().cancellation_token();let completion=j.finish_partial(catalog,body,assets,effect_slices,last.unwrap()).unwrap();owner.complete_partial(t,completion).unwrap();assert!(owner.partial_available());assert!(!owner.loaded_admission_available());owner.retire();assert!(token.load(Ordering::Acquire));
    assert_eq!(rows.len(),30);assert_eq!(images.len(),6);fs::write(p.join("logs/continuation-20261003/r434-conditional-event-details.json"),serde_json::to_vec_pretty(&json!({"state":"REAL_ASSET_CONDITIONAL_EVENT_SEQUENCES_AND_PARTIAL_VERIFIED","rows":rows,"images":images,"rows_count":30,"FirePot_rows":22,"body_rows":8,"first_frame_is1_after_opcode32":true,"body_object_substitution_refused":true,"body_clock_table_substitution_refused":true,"body_object_id":10275,"body_conditional_player_global":20000,"source_anchor_values_runtime_observed":false,"original_loop_or_lifetime_inferred":false,"model_retire_refuses_held_event":true,"foreign_job_event_refused":true,"opaque_partial_completed":true,"owner_retire_cancels":true,"LOADED":false,"GUI":0,"original_BIN_executed":false})).unwrap()).unwrap();
}
