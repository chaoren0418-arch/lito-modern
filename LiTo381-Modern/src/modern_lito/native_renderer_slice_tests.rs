//! Fixed service publication bytes + pinned Native CPU slices. No live TLS.
use super::*;
use super::super::{battle_follow, battle_read, battle_action, battle_operation,
    native_action_inspection::{CatalogPins, NativeActionSnapshot}, native_resources::{BytePin,NativeResourceAdapter,NativeSources}};
use serde_json::{json,Value};
use std::{fs,path::Path,io::Cursor};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn stamp_json(s:super::super::battle_window::Stamp)->Value {json!({"source":s.context.key.source_id,
    "object_u64_hex":format!("{:016X}",s.context.key.object_id),"epoch_u64_hex":format!("{:016X}",s.context.epoch),
    "channel":s.channel,"room":s.world[0],"match":s.world[1],"generation":s.world[2],"entity":s.entity,"life":s.life,"revision":s.revision})}
fn hash(s:&str)->[u8;32]{hex(s).try_into().unwrap()}
fn request_from_echo(b:&[u8])->Vec<u8>{
    let n=u32_at(b,84) as usize;
    let mut q=envelope_header(u32_at(b,8),REQUEST_KIND,REQUEST_BYTES).unwrap();
    q.extend(&b[16..84]); q.extend(&b[92+n..188+n]); q
}
fn observation(reply:&Reply,peer:protocol::extensions::Selected)->battle_follow::Observation {
    let q=battle_read::Request{context:reply.request.canonical.context,channel_id:reply.request.canonical.channel,
        world:reply.canonical.attachment.as_ref().unwrap().world.clone()};
    let mut f=battle_follow::Follower::new_selected(q.clone(),peer).unwrap();
    f.accept(battle_read::UnvalidatedReply{echo:q,state:battle_read::State::Data,data:reply.canonical.data.clone()}).unwrap();
    f.observation().unwrap().clone()
}
fn decode(c:DraftCodec,b:&[u8])->Reply {c.decode_reply(b,&request_from_echo(b)).unwrap()}
fn png(path:&Path,c:&super::super::compositor::Canvas){
    let mut e=png::Encoder::new(fs::File::create(path).unwrap(),c.width(),c.height());
    e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);
    e.write_header().unwrap().write_image_data(c.rgba()).unwrap();
}
fn demands(d:&payload::UnvalidatedData)->Value {
    json!({"world":{"room":d.world.identity.room,"match":d.world.identity.match_id,"generation":d.world.identity.generation,
        "revision":d.world.revision,"phase":d.world.phase as u8},
        "members":d.room.members.0.iter().map(|m|json!({"source":m.identity.source,"object":m.identity.id,
            "class":m.class,"sex":m.sex,"level":m.level})).collect::<Vec<_>>(),
        "actors":d.actors.0.iter().map(|a|json!({"entity":a.entity,"x":a.position.x,"y":a.position.y,"heading":a.heading,
            "loaded_by_test_peer":a.loaded,"weapon_type":a.vital.as_ref().map(|v|v.weapon_type),
            "has_vital":a.vital.is_some(),"has_resources":a.resources.is_some(),"has_buffs":a.buffs.is_some()})).collect::<Vec<_>>(),
        "personal_gear":d.personal.equipment.as_ref().map(|e|json!({"class":e.class,"sex":e.sex,"equipped_count":e.equipped.0.len(),"inventory_count":e.inventory.0.len()})),
        "public_presentation_rows":d.presentation.as_ref().map(|p|p.0.len()),"npcs":d.npcs.is_some(),
        "map":d.manifest.map,"catalog_actor_route_qualified":false})
}

#[test]
#[ignore="own frozen R430 actual SQL publication bytes; no SQL/TLS process"]
fn frozen_sql_publication_owner_and_actor_requirements() {
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r430-sql-snapshot.json")).unwrap()).unwrap();
    let input=fs::read(pointer["input"].as_str().unwrap()).unwrap();
    assert_eq!(sha(&input),pointer["input_sha256"].as_str().unwrap().to_ascii_lowercase());
    let input:Value=serde_json::from_slice(&input).unwrap();
    let base=protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f|protocol::CAP_EQUIPMENT};
    let codec=DraftCodec::local_experiment(base,256).unwrap();let peer=codec.peer().unwrap();
    let mut rows=Vec::new();
    for family in ["candidate","replay"] {
        let cfg=&input[family];let bytes=fs::read(cfg["vectors"].as_str().unwrap()).unwrap();
        assert_eq!(sha(&bytes),cfg["vectors_sha256"].as_str().unwrap().to_ascii_lowercase());
        let v:Value=serde_json::from_slice(&bytes).unwrap();
        for name in ["discovery89_hex","preparing89_hex","loading89_hex","loaded89_hex","active89_hex"] {
            let bytes=hex(v[name].as_str().unwrap());let r=decode(codec,&bytes);
            let n=u32_at(&bytes,84) as usize;assert_eq!(r.source.as_ref().unwrap().encode(base.capabilities).unwrap(),bytes[92..92+n]);
            let demand=r.canonical.data.as_ref().map(|d|{let o=observation(&r,peer);assert_eq!(o.data(),d);demands(d)});
            rows.push(json!({"family":family,"case":name,"sha256":sha(&bytes),"bytes":bytes.len(),
                "scope":{"source":r.request.canonical.context.key.source_id,"object_u64_hex":format!("{:016X}",r.request.canonical.context.key.object_id),
                    "epoch_u64_hex":format!("{:016X}",r.request.canonical.context.epoch),"channel":r.request.canonical.channel},
                "source_exact_reencode":true,"demands":demand}));
        }
        let read=hex(v["read61_source7_before_move_hex"].as_str().unwrap());
        let read_reply=battle_read::decode_body_selected(&read[16..],peer).unwrap();
        let mut f=battle_follow::Follower::new_selected(read_reply.echo.clone(),peer).unwrap();f.accept(read_reply).unwrap();
        let o=f.observation().unwrap().clone();let stamp=super::super::battle_window::Stamp::of(&o);
        let active=decode(codec,&hex(v["active89_hex"].as_str().unwrap()));let other=observation(&active,peer);
        assert_ne!(other.data().personal.player.source, o.data().personal.player.source);
        assert_eq!(stamp.life,1);
        rows.push(json!({"family":family,"case":"READ61","sha256":sha(&read),"owner":stamp_json(stamp),"demands":demands(o.data())}));
        for name in ["move65_source7_hex","stale_life65_source7_hex"] {
            let bytes=hex(v[name].as_str().unwrap());let length=116+u32_at(&bytes,120) as usize;
            let mut q=bytes[..length].to_vec();q[4..8].copy_from_slice(&(length as u32).to_be_bytes());q[12..14].copy_from_slice(&64u16.to_be_bytes());
            let mut expected=stamp;
            if name.starts_with("stale") {expected.life=u64_at(&bytes,72) as i64;}
            let guard=battle_action::Guard{stamp:expected,target:None};
            let receipt=battle_operation::read_reply_selected(&mut Cursor::new(&bytes),guard,&q,peer).unwrap();
            if name.starts_with("move") {
                assert_eq!(receipt.disposition,battle_operation::Disposition::Applied);
                let borrowed=battle_action::Guard{stamp:super::super::battle_window::Stamp::of(&other),target:None};
                assert!(battle_operation::read_reply_selected(&mut Cursor::new(&bytes),borrowed,&q,peer).is_err());
            } else {assert_ne!(receipt.disposition,battle_operation::Disposition::Applied);}
            rows.push(json!({"family":family,"case":name,"sha256":sha(&bytes),"guard":stamp_json(guard.stamp),
                "typed_receipt":true,"disposition":format!("{:?}",receipt.disposition),"demands":demands(&receipt.data),
                "expected_request_reconstructed_from_echo":true,"captured_request_verified":false}));
        }
    }
    assert_eq!(rows.len(),16);
    fs::write(p.join("logs/continuation-20261003/r430-sql-publication-details.json"),serde_json::to_vec_pretty(&json!({
        "state":"FIXED_SQL_PUBLICATION_BYTES_CONSUMED","rows":rows,"producer_SQL_TLS":true,"Rust_live_SQL_TLS":false,
        "actor_renderer_qualified":false,"LOADED":false,"GUI":0})).unwrap()).unwrap();
}

#[test]
#[ignore="own pinned R430 base archives + fixed SQL vectors; CPU study only"]
fn frozen_sql_prince_body_standing_moving_depth_native_integer_milliseconds() {
    use super::super::native_actor_inspection::{NativeActorSnapshot,NativeBaseBodyPins,BodyStudy,BodyAction};
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));
    let pointer:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r430-sql-snapshot.json")).unwrap()).unwrap();
    let input=fs::read(pointer["input"].as_str().unwrap()).unwrap();assert_eq!(sha(&input),pointer["input_sha256"].as_str().unwrap().to_ascii_lowercase());
    let input:Value=serde_json::from_slice(&input).unwrap();let cfg=&input["candidate"];
    let bytes=fs::read(cfg["vectors"].as_str().unwrap()).unwrap();assert_eq!(sha(&bytes),cfg["vectors_sha256"].as_str().unwrap().to_ascii_lowercase());let v:Value=serde_json::from_slice(&bytes).unwrap();
    let base=protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801007f};let codec=DraftCodec::local_experiment(base,256).unwrap();let peer=codec.peer().unwrap();
    let loading=decode(codec,&hex(v["loading89_hex"].as_str().unwrap()));let source=loading.source.clone().unwrap();let o=observation(&loading,peer);
    let scope=Scope{connection_incarnation:430,context:loading.request.canonical.context,channel:loading.request.canonical.channel,joined_room:o.data().room.id,room_revision:o.data().room.revision};
    let mut owner=DraftOwner::local_experiment(codec).unwrap();let ticket=owner.stage(scope,loading).unwrap();let job=owner.begin_renderer(ticket,&o).unwrap();
    let cp:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r430-base-body-snapshot.json")).unwrap()).unwrap();let root=Path::new(cp["root"].as_str().unwrap());
    let pin=|v:&Value|BytePin{bytes:v["bytes"].as_u64().unwrap(),sha256:hash(v["sha256"].as_str().unwrap())};
    let find=|name:&str|pin(cp["body_pins"].as_array().unwrap().iter().find(|v|v["name"]==name).unwrap());
    let pins=NativeBaseBodyPins{text_index:pin(&cp["text_pins"][0]),text_pak:pin(&cp["text_pins"][1]),decoded_catalog:pin(&cp["decoded_catalog"]),sprite_index:find("Sprite.idx"),sprite_pak:find("Sprite.pak"),
        shard_misses:cp["miss_shards"].as_array().unwrap().iter().map(|v|(v["shard"].as_u64().unwrap() as u8,find(v["index_name"].as_str().unwrap()),v["pak_bytes"].as_u64().unwrap())).collect()};
    let mut bad=pins.clone();bad.text_index.sha256[0]^=1;assert!(NativeActorSnapshot::load(&job,root,bad,BodyStudy::ExplicitUnarmedPrince).is_err());
    let body=NativeActorSnapshot::load(&job,root,pins.clone(),BodyStudy::ExplicitUnarmedPrince).unwrap();
    assert_eq!(body.duration_ms(BodyAction::Walk).unwrap(),vec![166;4]);assert_eq!(body.duration_ms(BodyAction::Idle).unwrap(),vec![166;8]);
    let read=hex(v["read61_source7_before_move_hex"].as_str().unwrap());let reply=battle_read::decode_body_selected(&read[16..],peer).unwrap();let read_request=reply.echo.clone();let mut f=battle_follow::Follower::new_selected(read_request.clone(),peer).unwrap();f.accept(reply).unwrap();
    let guard=battle_action::Guard{stamp:super::super::battle_window::Stamp::of(f.observation().unwrap()),target:None};let b=hex(v["move65_source7_hex"].as_str().unwrap());let length=116+u32_at(&b,120) as usize;let mut q=b[..length].to_vec();q[4..8].copy_from_slice(&(length as u32).to_be_bytes());q[12..14].copy_from_slice(&64u16.to_be_bytes());
    let receipt=battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).unwrap();assert_eq!(receipt.disposition,battle_operation::Disposition::Applied);
    f.accept(battle_read::UnvalidatedReply{echo:read_request,state:battle_read::State::Data,data:Some(receipt.data)}).unwrap();let moved=f.observation().unwrap().clone();
    let output=p.join("logs/continuation-20261003/r430-native-prince-slices");fs::create_dir_all(&output).unwrap();let mut samples=Vec::new();
    for (publication,o,action,times) in [("Loading",&o,BodyAction::Idle,&[0,166,1327][..]),("Loading",&o,BodyAction::Walk,&[0,166,663][..]),
        ("MOVE",&moved,BodyAction::Walk,&[0,166,663][..]),("MOVE",&moved,BodyAction::Idle,&[0][..])] {
        for elapsed in times {let slice=body.render_observation(o,action,*elapsed).unwrap();let sample=slice.sample();assert!(slice.depths().windows(2).all(|w|w[0].0<=w[1].0));
            let path=output.join(format!("prince-{publication}-{action:?}-{elapsed}ms.png"));png(&path,slice.canvas());
            samples.push(json!({"publication":publication,"owner":stamp_json(super::super::battle_window::Stamp::of(o)),"action":format!("{action:?}"),"elapsed_ms":elapsed,"frame":sample.2,"image":sample.3,"subframe":sample.4,"heading":sample.5,
                "png":path,"rgba_sha256":sha(slice.canvas().rgba()),"ordered_fragments":slice.depths().iter().map(|(k,r)|json!({"key":format!("{k:?}"),"half_x":r.half_x,"y":r.y,"depth":r.depth,"block":r.block_id})).collect::<Vec<_>>(),"events_retained_unplayed":format!("{:?}",slice.events())}));
        }
    }
    assert_eq!(samples.len(),10);assert!(body.render(BodyAction::Walk,664).is_err());assert!(body.render(BodyAction::Idle,1328).is_err());assert!(body.render(BodyAction::Walk,u64::MAX).is_err());
    let foreign=observation(&decode(codec,&hex(v["active89_hex"].as_str().unwrap())),peer);assert!(body.render_observation(&foreign,BodyAction::Idle,0).is_err());
    let actor_slice=body.render(BodyAction::Idle,0).unwrap();let mut other_owner=DraftOwner::local_experiment(codec).unwrap();let other_ticket=other_owner.stage(scope,decode(codec,&hex(v["loading89_hex"].as_str().unwrap()))).unwrap();let other_job=other_owner.begin_renderer(other_ticket,&o).unwrap();assert!(!actor_slice.matches_job(&other_job));assert!(actor_slice.matches_job(&job));
    let text:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-catalog-snapshot.json")).unwrap()).unwrap();let lt_pins=CatalogPins{index:pin(&text["pins"][0]),pak:pin(&text["pins"][1]),decoded_catalog:BytePin{bytes:39135,sha256:hash(text["reference_sha256"].as_str().unwrap())}};
    let fixture=p.join("target/r425-native-dm564-fixture");let sprites=fixture.join("native-archives");let mut effects=Vec::new();let mut timed=Vec::new();
    for id in [38,123] {let a=NativeActionSnapshot::load_firepot(&source.plan,Path::new(text["root"].as_str().unwrap()),&sprites,lt_pins,id,job.cancellation()).unwrap();
        for ms in [0,82,83,912] {let slice=a.render_catalog_millis(ms,job.cancellation()).unwrap();assert_eq!(slice.sample().3,if ms<83{0}else if ms==83{1}else{10});timed.push(json!({"sprite":id,"elapsed_ms":ms,"subframe":slice.sample().3,"rgba_sha256":sha(slice.canvas().rgba())}));}
        assert!(a.render_catalog_millis(913,job.cancellation()).is_err());effects.push(a.render_catalog_millis(0,job.cancellation()).unwrap());
    }
    let assets=NativeResourceAdapter::pinned_inspection().load_static(&NativeSources{map_root:fixture.join("native-map"),archive_root:sprites},&source.plan,Default::default(),job.cancellation()).unwrap();
    let completion=job.finish_partial_actor(assets,effects,actor_slice).unwrap();owner.complete_partial_renderer(ticket,completion).unwrap();assert!(owner.partial_renderer_available());assert!(!owner.loaded_admission_available());
    owner.retire();assert!(body.render(BodyAction::Idle,0).is_err());
    let report=json!({"state":"PINNED_PRINCE_UNARMED_SOURCE_STUDY_PASSED","samples":samples,"FirePot_integer_ms_samples":timed,
        "SPR_members":body.members().iter().map(|(n,h,b)|json!({"name":n,"sha256":h.iter().map(|b|format!("{b:02x}")).collect::<String>(),"bytes":b})).collect::<Vec<_>>(),
        "logical":0,"graphics_alias":3225,"actual_SQL_class":0,"actual_SQL_sex":0,"headings":[0,1],"Native_catalog_fps_origin":"43DE10 explicit native default24 and opcode110","frame_ms":166,
        "explicit_phase_zero_single_interval":true,"native_wall_start_or_loop_qualified":false,"unarmed_study_not_actual_equipment":true,
        "SourceAttachment_base_body_binding":false,"opaque_partial_actor_completion":true,"LOADED":false,"GUI":0,"Rust_live_SQL_TLS":false});
    fs::write(p.join("logs/continuation-20261003/r430-native-prince-slice-details.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
#[test]
#[ignore="own fixed R429 service + LTtext snapshots required; launches no peer"]
fn frozen_service_publications_native_firepot_ticks_depth_and_partial_completion() {
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));
    let pin:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-service-snapshot.json")).unwrap()).unwrap();
    let input=fs::read(pin["input"].as_str().unwrap()).unwrap();
    assert_eq!(sha(&input),pin["input_sha256"].as_str().unwrap().to_ascii_lowercase());
    let input:Value=serde_json::from_slice(&input).unwrap();assert_eq!(input["files_verified"],2801);
    assert_eq!(input["manifest_sha256"],"DDB28022F48B267A3F34E36E6C326784F4C1D5F3A7FDB3C7C8C318946E4B2F77");
    let root=Path::new(input["root"].as_str().unwrap());
    let s=fs::read(root.join("canonical-service-data-vectors.json")).unwrap();
    let t=fs::read(root.join("canonical-tls-vectors.json")).unwrap();
    assert_eq!(sha(&s),"22d8eacdd0faf9cadc62e1d89faade5c0835d696f8382d21b88da10658bfb6bd");
    assert_eq!(sha(&t),"c4a054bca9d5840a167ec5e33c54ce3472b52368440d5138faa43c05fafc52f8");
    let s:Value=serde_json::from_slice(&s).unwrap();let t:Value=serde_json::from_slice(&t).unwrap();
    let base=protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:s["selected_capabilities"].as_u64().unwrap() as u32};
    let codec=DraftCodec::local_experiment(base,256).unwrap();let peer=codec.peer().unwrap();
    let mut rows=Vec::new();
    for (family,v,names) in [("service",&s,&["loading89_hex","loaded89_hex","active89_hex"][..]),
        ("TLS_capture",&t,&["discovery89_hex","preparing89_hex","loading89_hex","loaded89_hex","active89_hex"][..])] {
        for name in names {
            let b=hex(v[name].as_str().unwrap());let r=decode(codec,&b);
            let source=r.source.as_ref().unwrap();let n=u32_at(&b,84) as usize;
            assert_eq!(source.encode(base.capabilities).unwrap(),b[92..92+n]);
            let demand=r.canonical.data.as_ref().map(|d|{let o=observation(&r,peer);assert!(o.data()==d);demands(d)});
            rows.push(json!({"family":family,"case":name,"bytes":b.len(),"sha256":sha(&b),
                "source_exact_reencode":true,"typed_canonical_data":r.canonical.data.is_some(),"demands":demand,
                "expected_request_reconstructed_from_echo":true,"captured_request_verified":false}));
        }
        let active=decode(codec,&hex(v["active89_hex"].as_str().unwrap()));let active_o=observation(&active,peer);
        let b=hex(v[if family=="service"{"operation65_hex"}else{"move65_hex"}].as_str().unwrap());
        let length=116+u32_at(&b,120) as usize;
        let mut q=b[..length].to_vec();q[4..8].copy_from_slice(&(length as u32).to_be_bytes());q[12..14].copy_from_slice(&64u16.to_be_bytes());
        let o=if family=="service" {active_o.clone()} else {
            // TLS active89 is participantB (source42, same object3333).
            // MOVE belongs to participantA (source7); READ61 is its actual
            // immediately preceding publication. Never borrow another owner.
            let guard=battle_action::Guard{stamp:super::super::battle_window::Stamp::of(&active_o),target:None};
            assert!(battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).is_err());
            let bytes=hex(v["read61_hex"].as_str().unwrap());
            let r=battle_read::decode_body_selected(&bytes[16..],peer).unwrap();
            let mut f=battle_follow::Follower::new_selected(r.echo.clone(),peer).unwrap();f.accept(r).unwrap();
            f.observation().unwrap().clone()
        };
        let guard=battle_action::Guard{stamp:super::super::battle_window::Stamp::of(&o),target:None};
        let operation=battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).unwrap();
        assert_eq!(operation.disposition,battle_operation::Disposition::Applied);
        rows.push(json!({"family":family,"case":"MOVE65","typed_receipt":true,"bytes":b.len(),"demands":demands(&operation.data),
            "Rust_painted_life":guard.stamp.life,"Rust_receipt_verified_against_prior_publication":true,
            "cross_source_same_object_guard_rejected":family!="service"}));
    }
    let read=hex(t["read61_hex"].as_str().unwrap());
    assert_eq!(u16_at(&read,12),61);assert_eq!(u32_at(&read,4) as usize,read.len());
    let reply=battle_read::decode_body_selected(&read[16..],peer).unwrap();
    let mut f=battle_follow::Follower::new_selected(reply.echo.clone(),peer).unwrap();f.accept(reply).unwrap();
    rows.push(json!({"family":"TLS_capture","case":"READ61","bytes":read.len(),"typed_and_semantically_admitted":true,"demands":demands(f.observation().unwrap().data())}));
    assert_eq!(rows.len(),11);
    let loading=decode(codec,&hex(s["loading89_hex"].as_str().unwrap()));
    let o=observation(&loading,peer);let source=loading.source.as_ref().unwrap().clone();
    let cp:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-catalog-snapshot.json")).unwrap()).unwrap();
    let text_root=Path::new(cp["root"].as_str().unwrap());
    let pins=CatalogPins{index:BytePin{bytes:9888,sha256:hash(cp["pins"][0]["sha256"].as_str().unwrap())},
        pak:BytePin{bytes:292553,sha256:hash(cp["pins"][1]["sha256"].as_str().unwrap())},
        decoded_catalog:BytePin{bytes:39135,sha256:hash(cp["reference_sha256"].as_str().unwrap())}};
    let fixture=p.join("target/r425-native-dm564-fixture");let sprites=fixture.join("native-archives");
    let cancel=AtomicBool::new(false);let output=p.join("logs/continuation-20261003/r429-native-action-slices");fs::create_dir_all(&output).unwrap();
    let mut action_rows=Vec::new();let mut completed_slices=Vec::new();
    for id in [38,123] {
        let a=NativeActionSnapshot::load_firepot(&source.plan,text_root,&sprites,pins,id,&cancel).unwrap();
        assert_eq!(a.durations(),vec![2;11]);
        assert_eq!(a.sprite_sha256(),hash(if id==38{"b4507d8337497aef8b3fc3a28f8fef87e344d176b8a31a7e634c08a0976aef8d"}else{"bf3e43c72610b061f312551c622cee600d61fdb7079a8f8e791026fadb50ef83"}));
        let mut samples=Vec::new();
        for tick in [0,1,2,3,10,20,21] {
            let slice=a.render_tick(tick,&cancel).unwrap();assert_eq!(slice.sample(),(id,3,tick,(tick/2) as u32));
            assert!(slice.depth_references().windows(2).all(|w|w[0].0<=w[1].0));
            let path=output.join(format!("native-{id}-action3-tick{tick}.png"));png(&path,slice.canvas());
            samples.push(json!({"tick":tick,"subframe":tick/2,"rgba_sha256":sha(slice.canvas().rgba()),"png":path,
                "ordered_fragments":slice.depth_references().iter().map(|(k,r)|json!({"key":format!("{k:?}"),"half_x":r.half_x,"y":r.y,"depth":r.depth,"block":r.block_id})).collect::<Vec<_>>()}));
            if tick==0 {completed_slices.push(slice);}
        }
        assert!(a.render_tick(22,&cancel).is_err());assert!(a.render_tick(u64::MAX,&cancel).is_err());
        assert!(a.render_tick(0,&AtomicBool::new(true)).is_err());
        action_rows.push(json!({"sprite":id,"explicit_action":3,"total_source_ticks":22,"duration_ticks":a.durations(),"samples":samples,
            "native_ms_clock":null,"Layer8_default_action_and_loop_phase_qualified":false}));
    }
    let mut bad=pins;bad.decoded_catalog.sha256[0]^=1;
    assert!(NativeActionSnapshot::load_firepot(&source.plan,text_root,&sprites,bad,38,&cancel).is_err());
    assert!(NativeActionSnapshot::load_firepot(&source.plan,text_root,&sprites,pins,39,&cancel).is_err());
    assert!(NativeActionSnapshot::load_firepot(&source.plan,text_root,&sprites,pins,38,&AtomicBool::new(true)).is_err());
    let scope=Scope{connection_incarnation:429,context:loading.request.canonical.context,channel:loading.request.canonical.channel,
        joined_room:o.data().room.id,room_revision:o.data().room.revision};
    let mut owner=DraftOwner::local_experiment(codec).unwrap();let ticket=owner.stage(scope,loading).unwrap();
    let job=owner.begin_renderer(ticket,&o).unwrap();
    let tls_loading=decode(codec,&hex(t["loading89_hex"].as_str().unwrap()));let other=observation(&tls_loading,peer);
    assert!(owner.begin_renderer(ticket,&other).is_err()); // Full publication, not just matching map564.
    assert!(DraftOwner::default().begin_renderer(ticket,&o).is_err());
    let sources=NativeSources{map_root:fixture.join("native-map"),archive_root:sprites.clone()};
    let assets=NativeResourceAdapter::pinned_inspection().load_static(&sources,&source.plan,Default::default(),job.cancellation()).unwrap();
    let completion=job.finish_partial(assets,completed_slices).unwrap();
    let mut other_owner=DraftOwner::local_experiment(codec).unwrap();
    let other_ticket=other_owner.stage(scope,decode(codec,&hex(s["loading89_hex"].as_str().unwrap()))).unwrap();
    assert!(other_owner.complete_partial_renderer(other_ticket,completion).is_err());
    let old_job=owner.begin_renderer(ticket,&o).unwrap();
    let next=owner.stage(scope,decode(codec,&hex(s["loading89_hex"].as_str().unwrap()))).unwrap();
    assert!(old_job.cancellation().load(Ordering::Acquire));assert!(owner.begin_renderer(ticket,&o).is_err());
    let job=owner.begin_renderer(next,&o).unwrap();
    let a=NativeActionSnapshot::load_firepot(&source.plan,text_root,&sprites,pins,38,job.cancellation()).unwrap();
    let slices=vec![a.render_tick(0,job.cancellation()).unwrap()];
    let assets=NativeResourceAdapter::pinned_inspection().load_static(&sources,&source.plan,Default::default(),job.cancellation()).unwrap();
    let completion=job.finish_partial(assets,slices).unwrap();owner.complete_partial_renderer(next,completion).unwrap();
    assert!(owner.partial_renderer_available());assert!(!owner.loaded_admission_available());
    let old_completion=owner.staged.as_mut().unwrap().renderer.take().unwrap();
    let new_ticket=owner.stage(scope,decode(codec,&hex(s["loading89_hex"].as_str().unwrap()))).unwrap();
    assert!(owner.complete_partial_renderer(new_ticket,old_completion).is_err());
    assert!(!owner.partial_renderer_available());
    owner.retire();assert!(!owner.partial_renderer_available());assert!(!owner.loaded_admission_available());
    let report=json!({"state":"PASSED_FIXED_PUBLICATION_AND_PARTIAL_NATIVE_CPU_SLICE","publication_frames":rows,
        "actions":action_rows,"opaque_partial_completion_verified":true,"binding_guards":["full admitted DATA","canonical attachment","SourceAttachment","connection/room fence","owner/ticket","worker cancellation"],
        "Native_clock_ms_qualified":false,"map_effect_action_binding_qualified":false,"actor_route_qualified":false,
        "SourceAttachment_LTtext_binding":false,"Rust_live_TLS":false,"SQL":false,"GUI":0,"LOADED":false,"qualified_modes":16});
    fs::write(p.join("logs/continuation-20261003/r429-native-renderer-slice-details.json"),serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}
