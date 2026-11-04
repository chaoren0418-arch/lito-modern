use super::*;
use serde_json::{Value,json};
use std::{fs,io::Cursor,collections::{BTreeSet,BTreeMap}};

fn hex(s:&str)->Vec<u8>{assert_eq!(s.len()%2,0);s.as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn ack()->protocol::Ack{protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f|protocol::CAP_EQUIPMENT}}
fn input()->Value{let p=Path::new(env!("CARGO_MANIFEST_DIR"));let v:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r433-catalog-snapshot.json")).unwrap()).unwrap();let b=fs::read(v["input"].as_str().unwrap()).unwrap();assert_eq!(sha(&b),v["input_sha256"].as_str().unwrap().to_ascii_lowercase());serde_json::from_slice(&b).unwrap()}
fn pinned(cfg:&Value,name:&str)->Value{let v=cfg["vectors"].as_array().unwrap().iter().find(|v|v["name"]==name).unwrap();let b=fs::read(v["path"].as_str().unwrap()).unwrap();assert_eq!(sha(&b),v["sha256"].as_str().unwrap().to_ascii_lowercase());serde_json::from_slice(&b).unwrap()}

// This is a fixed-evidence reader, not a transport decoder. Validate bounds
// before expanding hex and preserve sequence/session as exchange identity.
fn validate_capture(v:&Value)->std::result::Result<(),String>{
    let err=||Err("capture schema/bounds/identity".to_owned());
    if v["schema"]!=1||v["append_only"]!=true||v["max_records"]!=256||v["max_wire_bytes"]!=12582912{return err();}
    let Some(entries)=v["entries"].as_array()else{return err()};
    if entries.is_empty()||entries.len()>256{return err();}
    let mut keys=BTreeSet::new();let mut sessions=BTreeSet::new();let mut total=0usize;
    for (i,e) in entries.iter().enumerate(){
        let (Some(seq),Some(session),Some(source),Some(id),Some(kind))=(e["sequence"].as_u64(),e["session"].as_u64(),e["source_id"].as_u64(),e["request_id"].as_u64(),e["request_kind"].as_u64())else{return err()};
        let expected=format!("capture_{seq:04}_session_{session:03}_source_{source}_request_{id}");
        if seq!=i as u64+1||session==0||!matches!(source,7|42)||e["key"]!=expected||!keys.insert(expected)||e["object_id"]!=10275||e["epoch_u64_hex"]!="8000000000000000"||e["channel"]!=0{return err();}
        sessions.insert(session);
        let Some(q)=e["request_hex"].as_str()else{return err()};
        if q.len()<32||q.len()%2!=0||q.len()>12582912*2||!q.bytes().all(|b|b.is_ascii_hexdigit()){return err();}
        total=total.checked_add(q.len()/2).ok_or_else(||"wire overflow".to_owned())?;
        match e["outcome"].as_str(){
            Some("COMPLETE_EXCHANGE")=>{let Some(b)=e["response_hex"].as_str()else{return err()};if b.len()<32||b.len()%2!=0||b.len()>12582912*2||!b.bytes().all(|b|b.is_ascii_hexdigit()){return err();}total=total.checked_add(b.len()/2).ok_or_else(||"wire overflow".to_owned())?;},
            Some("EOF_REJECTED")=>if !e["response_hex"].is_null()||!e["response_kind"].is_null()||!e["nested89_hex"].is_null(){return err();},
            _=>return err()
        }
        if total>12582912{return err();}
        let q=hex(q);if &q[..4]!=b"LTM1"||u32_at(&q,4)as usize!=q.len()||u32_at(&q,8)as u64!=id||u16_at(&q,12)as u64!=kind{return err();}
    }
    if total as u64!=v["wire_bytes"].as_u64().unwrap_or(u64::MAX)||sessions.len()as u64>v["session_count"].as_u64().unwrap_or(0){return err();}
    Ok(())
}

#[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn append_only_reader_refuses_sequence_key_eof_and_budget_substitution(){
    let cfg=input();let v=pinned(&cfg["candidate"],"sql92/actor-sql-wire-capture.json");validate_capture(&v).unwrap();
    for change in 0..5{let mut b=v.clone();match change{
        0=>b["entries"][1]["sequence"]=b["entries"][0]["sequence"].clone(),
        1=>b["entries"][1]["key"]=b["entries"][0]["key"].clone(),
        2=>{let e=b["entries"].as_array_mut().unwrap().iter_mut().find(|e|e["outcome"]=="EOF_REJECTED").unwrap();e["response_hex"]=json!("00");},
        3=>b["wire_bytes"]=json!(12582913),
        _=>b["entries"][0]["request_hex"]=json!("nothex")
    }assert!(validate_capture(&b).is_err(),"{change}");}
}

fn observe(n:&native::Reply,peer:protocol::extensions::Selected)->battle_follow::Observation{
    use super::super::battle_read;
    let q=battle_read::Request{context:n.request.canonical.context,channel_id:n.request.canonical.channel,world:n.canonical.attachment.as_ref().unwrap().world.clone()};
    let mut f=battle_follow::Follower::new_selected(q.clone(),peer).unwrap();f.accept(battle_read::UnvalidatedReply{echo:q,state:battle_read::State::Data,data:n.canonical.data.clone()}).unwrap();f.observation().unwrap().clone()
}
fn stamp(o:&battle_follow::Observation)->Value{let s=super::super::battle_window::Stamp::of(o);json!({"source":s.context.key.source_id,"object_u64_hex":format!("{:016X}",s.context.key.object_id),"epoch_u64_hex":format!("{:016X}",s.context.epoch),"channel":s.channel,"room":s.world[0],"match":s.world[1],"generation":s.world[2],"entity":s.entity,"life":s.life,"revision":s.revision})}

#[test]
#[ignore="own fixed append-only realSQL90/91 and92/93 capture; offline only, no resource completion or socket"]
fn fixed_append_only_sql_catalog_actor_and_discovery_consumer(){
    use super::super::{battle_read,battle_action,battle_operation,battle_window,equipment};
    let cfg=input();let ac=ActorCodec::local_experiment(ack(),1792).unwrap();let cc=catalog::CatalogCodec::local_experiment(ack(),768).unwrap();let peer=protocol::extensions::Selected::new(ack(),0).unwrap();
    let mut summaries=Vec::new();let mut rows=Vec::new();let mut negatives=Vec::new();let mut proofs=Vec::new();let mut discovery=Vec::new();let mut receipts=Vec::new();let mut inventory=Vec::new();let mut worker_fences=Vec::new();
    for family in ["candidate","replay"]{for (kind,route) in [(90,"sql90/catalog"),(92,"sql92/actor")]{
        let capture=pinned(&cfg[family],&format!("{route}-sql-wire-capture.json"));validate_capture(&capture).unwrap();
        let vectors=pinned(&cfg[family],&format!("{route}-sql-wire-vectors.json"));let disc=pinned(&cfg[family],&format!("sql{kind}/discovery-original-pairs.json"));assert_eq!(disc["schema"],1);assert_eq!(disc["historical_recovery"],false);
        let entries=capture["entries"].as_array().unwrap();let mut complete=0;let mut resource=0;let mut eof=0;let mut loaded=BTreeSet::new();let mut loaded_scopes=BTreeSet::new();let mut old_keys=BTreeMap::<(u64,String,u64),Vec<u64>>::new();let mut first=BTreeSet::new();let mut identity=None;let mut source=None;let mut current_observation=None;let mut active42=None;let mut last_equipment=BTreeMap::new();let mut changed=0;let mut worker_checks=0;
        for e in entries{
            let key=e["key"].as_str().unwrap();let seq=e["sequence"].as_u64().unwrap();let session=e["session"].as_u64().unwrap();let request_kind=e["request_kind"].as_u64().unwrap();let q=hex(e["request_hex"].as_str().unwrap());
            assert_eq!(vectors[format!("{key}_request{request_kind}_hex")],e["request_hex"]);
            old_keys.entry((e["source_id"].as_u64().unwrap(),e["action"].as_str().unwrap().to_owned(),e["request_id"].as_u64().unwrap())).or_default().push(seq);
            if e["outcome"]=="EOF_REJECTED"{
                eof+=1;let validation=if request_kind==kind{
                    if kind==92{match ac.decode_request(&q){Err(_)=>"structure refused",Ok(q)=>if q.check_authority(identity.as_ref().unwrap(),source.as_ref().unwrap()).is_err(){"actor/catalog/Native authority refused"}else{if e["phase"]=="negative_cross_source_scope"{assert_ne!(q.catalog.native.canonical.context.key.source_id,e["source_id"].as_u64().unwrap()as u32);}"valid byte proof; transport owner or generation rejection is producer evidence"}}}
                    else{match cc.decode_request(&q){Err(_)=>"structure refused",Ok(q)=>if q.check_authority(&identity.as_ref().unwrap().catalog(),source.as_ref().unwrap()).is_err(){"catalog/Native authority refused"}else{if e["phase"]=="negative_cross_source_scope"{assert_ne!(q.native.canonical.context.key.source_id,e["source_id"].as_u64().unwrap()as u32);}"valid byte proof; transport owner or generation rejection is producer evidence"}}}
                }else if request_kind==88||request_kind==90{assert!(ac.decode_request(&q).is_err());"old route structurally refused by actor consumer"}else{"old generation operation EOF; no fabricated reply or Rust socket claim"};
                negatives.push(json!({"family":family,"route":kind,"sequence":seq,"session":session,"phase":e["phase"],"request_sha256":sha(&q),"response":null,"validation":validation,"Rust_transport_replayed":false}));continue;
            }
            complete+=1;let b=hex(e["response_hex"].as_str().unwrap());let response_kind=e["response_kind"].as_u64().unwrap();assert_eq!(u32_at(&b,4)as usize,b.len());assert_eq!(u32_at(&b,8),u32_at(&q,8));assert_eq!(u16_at(&b,12)as u64,response_kind);assert_eq!(vectors[format!("{key}_result{response_kind}_hex")],e["response_hex"]);
            if request_kind==kind{
                resource+=1;
                let actor=if kind==92{Some(ac.decode_reply(&b,&q).unwrap())}else{None};
                let cat=if kind==90{Some(cc.decode_reply(&b,&q).unwrap())}else{None};
                let cr=actor.as_ref().map(|r|r.catalog()).or(cat.as_ref()).unwrap();let n=cr.native();let request=&cr.request().native.canonical;
                if let Some(a)=actor.as_ref(){assert_eq!(ac.encode_request(u32_at(&q,8),a.request()).unwrap(),q);if let Some(before)=&identity{assert_eq!(a.identity().unwrap(),before);}else{identity=a.identity().cloned();}}
                else{assert_eq!(cc.encode_request(u32_at(&q,8),cr.request()).unwrap(),q);if identity.is_none(){let old:Value=serde_json::from_slice(&fs::read(Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003/r432-catalog-snapshot.json")).unwrap()).unwrap();let root=Path::new(old["candidate"]["root"].as_str().unwrap());let v:Value=serde_json::from_slice(&fs::read(root.join("evidence/actor-vectors.json")).unwrap()).unwrap();let r=ac.decode_reply(&hex(v["loading_result93_hex"].as_str().unwrap()),&hex(v["loading_request92_hex"].as_str().unwrap())).unwrap();identity=r.identity().cloned();}}
                source=n.source.clone();assert_eq!(cr.catalog().unwrap(),identity.as_ref().unwrap().catalog());
                assert_eq!(request.context.key.source_id,e["source_id"].as_u64().unwrap()as u32);assert_eq!(request.context.key.object_id,10275);assert_eq!(request.context.epoch as u64,0x8000000000000000);
                assert_eq!(request.channel,e["wire_scope"]["channel"].as_i64().unwrap()as u8);assert_eq!(request.context.key.source_id,e["wire_scope"]["source"].as_i64().unwrap()as u32);
                assert_eq!((request.room,request.match_id,request.generation),(e["requested_world"]["room"].as_i64().unwrap(),e["requested_world"]["match"].as_i64().unwrap(),e["requested_world"]["generation"].as_i64().unwrap()));
                let nested=hex(e["nested89_hex"].as_str().unwrap());let cat_at=if kind==92{92+u32_at(&b,84)as usize}else{0};let native_at=cat_at+92+u32_at(&b,cat_at+84)as usize;assert_eq!(&b[native_at..],nested);assert_eq!(vectors[format!("{key}_nested89_hex")],e["nested89_hex"]);
                if e["phase"]=="main"&&request.action==world::Action::Read&&request.match_id==0{assert!(first.insert(request.context.key.source_id));assert!(disc["entries"].as_array().unwrap().iter().any(|x|x==e));discovery.push(json!({"family":family,"route":kind,"sequence":seq,"session":session,"source":request.context.key.source_id,"full_response_sha256":sha(&b),"nested89_sha256":sha(&nested),"new_capture_not_historical_reconstruction":true}));}
                if !e["phase"].as_str().unwrap().starts_with("negative_")&&request.action==world::Action::Loaded{
                    let proof=actor.as_ref().map(|a|a.request().actor_proof).unwrap_or(cr.request().catalog_load_proof);assert!(loaded.insert(proof));assert!(loaded_scopes.insert((request.context.key.source_id,request.generation)));proofs.push(json!({"family":family,"route":kind,"sequence":seq,"session":session,"source":request.context.key.source_id,"epoch_u64_hex":format!("{:016X}",request.context.epoch),"room":request.room,"match":request.match_id,"generation":request.generation,"proof_hex":proof.iter().map(|b|format!("{b:02x}")).collect::<String>(),"Native_catalog_actor_domains_recomputed":true}));
                }
                let owner=n.canonical.data.as_ref().map(|d|{assert!(d.personal.equipment.is_none());let o=observe(n,peer);assert_eq!(o.data(),d);if request.context.key.source_id==42{active42=Some(o.clone());}
                    if kind==92&&e["phase"]=="main"&&n.canonical.attachment.as_ref().unwrap().lifecycle.phase==super::super::world_protocol::Phase::Loading{
                        let r=ac.decode_reply(&b,&q).unwrap();let s=native::Scope{connection_incarnation:session,context:request.context,channel:request.channel,joined_room:o.data().room.id,room_revision:o.data().room.revision};let mut owner=ActorOwner::local_experiment(ac).unwrap();let ticket=owner.stage(s,r).unwrap();let job=owner.begin_renderer(ticket,&o).unwrap();let mut foreign=s;foreign.connection_incarnation+=1;assert!(owner.stage(foreign,ac.decode_reply(&b,&q).unwrap()).is_err());owner.retire();assert!(job.cancellation().load(Ordering::Acquire));assert!(!owner.loaded_admission_available());worker_checks+=1;
                    }stamp(&o)});
                rows.push(json!({"family":family,"route":kind,"sequence":seq,"session":session,"key":key,"phase":e["phase"],"request_sha256":sha(&q),"response_sha256":sha(&b),"nested89_sha256":sha(&nested),"source":request.context.key.source_id,"observed_world":e["observed_world"],"state":format!("{:?}",n.canonical.state),"data_owner":owner,"exact_captured_request_verified":true,"renderer_completion":false}));
            }else{match response_kind{
                15|17=>{let r=app::decode_response_version(&b,47).unwrap();match r.response{
                    app::Response::CharacterData(r)=>{let d=r.result.unwrap();assert_eq!(d.items.len(),29);assert_eq!((d.character.class,d.character.sex),(0,0));inventory.push(json!({"family":family,"route":kind,"sequence":seq,"kind":15,"source":d.context.key.source_id,"items":29,"sha256":sha(&b)}));},
                    app::Response::Equipment(r)=>{if let Some(s)=r.snapshot{assert_eq!(s.source.items.len(),29);assert_eq!(s.source.context,r.request.context);if r.changed{assert_eq!(r.status,equipment::Status::Ok);assert_eq!(r.request.context.key.source_id,7);assert_eq!(s.revision,r.request.expected_revision+1);assert_eq!(s.equipped(10304),Some(false));changed+=1;}inventory.push(json!({"family":family,"route":kind,"sequence":seq,"kind":17,"source":r.request.context.key.source_id,"revision":s.revision,"item10304_equipped":s.equipped(10304),"changed":r.changed,"sha256":sha(&b)}));last_equipment.insert(r.request.context.key.source_id,s);}else{assert_eq!(r.status,equipment::Status::BindingMismatch);inventory.push(json!({"family":family,"route":kind,"sequence":seq,"kind":17,"status":"BindingMismatch","sha256":sha(&b)}));}},_=>unreachable!()}},
                61=>{let r=battle_read::decode_body_selected(&b[16..],peer).unwrap();let mut f=battle_follow::Follower::new_selected(r.echo.clone(),peer).unwrap();f.accept(r).unwrap();current_observation=Some(f.observation().unwrap().clone());receipts.push(json!({"family":family,"route":kind,"sequence":seq,"kind":61,"request_sha256":sha(&q),"response_sha256":sha(&b),"owner":stamp(current_observation.as_ref().unwrap())}));},
                65=>{let o=current_observation.as_ref().unwrap();let mut s=battle_window::Stamp::of(o);s.life=u64_at(&q,72)as i64;let guard=battle_action::Guard{stamp:s,target:None};let r=battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).unwrap();let borrowed=battle_action::Guard{stamp:battle_window::Stamp::of(active42.as_ref().unwrap()),target:None};assert!(battle_operation::read_reply_selected(&mut Cursor::new(&b),borrowed,&q,peer).is_err());receipts.push(json!({"family":family,"route":kind,"sequence":seq,"kind":65,"request_sha256":sha(&q),"response_sha256":sha(&b),"owner":stamp(o),"request_life":s.life,"disposition":format!("{:?}",r.disposition),"foreign_source_guard_refused":true}));},_=>panic!("Unexpected complete kind {response_kind}")}}
        }
        assert_eq!(first,BTreeSet::from([7,42]));assert_eq!(disc["entries"].as_array().unwrap().len(),2);assert_eq!(loaded.len(),4);assert_eq!(loaded_scopes.len(),4);assert_eq!(changed,1);assert_eq!(last_equipment[&7].equipped(10304),Some(false));assert_eq!(last_equipment[&42].equipped(10304),Some(true));
        let collisions:Vec<_>=old_keys.into_iter().filter(|(_,v)|v.len()>1).map(|((source,action,id),seq)|json!({"source":source,"action":action,"reused_request_id":id,"all_sequences_preserved":seq})).collect();assert!(!collisions.is_empty());
        worker_fences.push(json!({"family":family,"route":kind,"Loading_scope_jobs":worker_checks,"resources_loaded":false,"partial_completed":false,"LOADED":false}));
        summaries.push(json!({"family":family,"route":kind,"records":entries.len(),"complete_exchanges":complete,"resource_pairs":resource,"EOF_no_response":eof,"wire_bytes":capture["wire_bytes"],"proofs":4,"initial_discoveries":2,"legacy_collision_groups":collisions}));
    }}
    assert_eq!(discovery.len(),8);assert_eq!(proofs.len(),16);assert_eq!(inventory.len(),32);assert_eq!(receipts.len(),12);
    fs::write(Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003/r433-append-only-consumer-details.json"),serde_json::to_vec_pretty(&json!({"state":"FIXED_APPEND_ONLY_SQL_CAPTURE_CONSUMED","summaries":summaries,"rows":rows,"EOF_records":negatives,"new_initial_discoveries":discovery,"source_generation_proofs":proofs,"inventory_equipment":inventory,"typed_receipts":receipts,"worker_fences":worker_fences,"historical_lost91_reconstructed":false,"R432_actual_resource_partial_is_separate":true,"producer_SQL_TLS":true,"Rust_live_SQL_TLS":false,"renderer_LOADED":false,"GUI":0})).unwrap()).unwrap();
}
