use super::*;
use super::super::{battle_read,battle_action,battle_operation,battle_window,
    native_action_inspection::NativeActionSnapshot,native_resources::{NativeResourceAdapter,NativeSources}};
use serde_json::{json,Value};
use std::{fs,io::Cursor};
fn hex(s:&str)->Vec<u8>{assert_eq!(s.len()%2,0);s.as_bytes().chunks_exact(2).map(|p|u8::from_str_radix(std::str::from_utf8(p).unwrap(),16).unwrap()).collect()}
fn sha(b:&[u8])->String{format!("{:x}",Sha256::digest(b))}
fn base()->protocol::Ack{protocol::Ack{request_id:1,version:47,numeric_schema:1,capabilities:0x1801006f}}
fn codec()->CatalogCodec{CatalogCodec::local_experiment(base(),768).unwrap()}
fn identity()->CatalogIdentity{CatalogIdentity{index:resources::BytePin{bytes:9888,sha256:hex("b8c0bb8c43affe59a90ebd2d26182269b6d60af388c80963eaaeedc8d8ae23c8").try_into().unwrap()},
    pak:resources::BytePin{bytes:292553,sha256:hex("517d853e3fe91b504953ff4b9b2eed682a3612d063f2d18e75e39fd5580f342d").try_into().unwrap()},offset:145671,
    raw_member:resources::BytePin{bytes:39135,sha256:hex("8baf964cf30f49c29e194cb5ebbef1bb6b8bb61c755975ad26b034538f660962").try_into().unwrap()},
    decoded_sha256:hex("f55779aeb5c3982d8b366068cdbe4003d802619074e54388f26ee87368f42603").try_into().unwrap()}}
fn discovery()->Request{Request{expected_catalog:[0;32],catalog_load_proof:[0;32],native:native::Request{expected_resource_set:[0;32],load_proof:[0;32],
    canonical:world::Request{context:app::SelectionContext{key:app::CharacterKey{source_id:7,object_id:10275},epoch:u64::MAX},channel:0,
        action:world::Action::Read,room:0,room_revision:0,match_id:0,generation:0,digest:[0;32]}}}}
#[test]fn selection_is_default_disabled_and_never_formal_known(){
    assert_eq!(protocol::extensions::KNOWN,255);assert!(CatalogCodec::default().maximum_result().is_err());
    for bit in [0,256,512,768|1024,768|protocol::extensions::FULL_VIP_DATA]{assert!(CatalogCodec::local_experiment(base(),bit).is_err());}
    let mut b=base();b.version=46;assert!(CatalogCodec::local_experiment(b,768).is_err());b=base();b.capabilities&=!protocol::CAP_ACTION_LIFE;assert!(CatalogCodec::local_experiment(b,768).is_err());
    assert!(protocol::extensions::Selected::new(base(),768).is_err());
}
#[test]fn identity_exact_schema_lengths_hashes_and_range(){
    let c=identity();let b=c.encode().unwrap();assert_eq!(b.len(),168);assert_eq!(CatalogIdentity::decode(&b).unwrap(),c);
    assert_eq!(sha(&[b"LITO-NATIVE-CATALOG-IDENTITY\0".as_slice(),&b].concat()),"ce1906fc061364da6fe2e3c297c178a5fae9c27bb248e6cb8ebc9f2142c095a1");
    for at in [1,3,85,87]{let mut bad=b.clone();bad[at]^=1;assert!(CatalogIdentity::decode(&bad).is_err());}
    for (at,n) in [(4,0u64),(44,268435457),(88,u64::MAX),(96,1048577)]{let mut bad=b.clone();bad[at..at+8].copy_from_slice(&n.to_be_bytes());assert!(CatalogIdentity::decode(&bad).is_err());}
    for at in [12,52,104,136]{let mut bad=b.clone();bad[at..at+32].fill(0);assert!(CatalogIdentity::decode(&bad).is_err());}
    assert!(CatalogIdentity::decode(&b[..167]).is_err());let mut extra=b;extra.push(0);assert!(CatalogIdentity::decode(&extra).is_err());
}
#[test]fn nested_ids_request_tag_and_old88_route_are_strict(){
    let c=codec();let q=discovery();let b=c.encode_request(12,&q).unwrap();assert_eq!(b.len(),264);assert!(c.decode_request(&b).unwrap()==q);
    for at in [0,7,11,13,15,17,19,95,99,101,103]{let mut bad=b.clone();bad[at]^=1;assert!(c.decode_request(&bad).is_err(),"{at}");}
    let old=c.inner().unwrap().encode_request(12,&q.native).unwrap();assert!(c.decode_request(&old).is_err());
    let mut trailing=b;trailing.push(0);assert!(c.decode_request(&trailing).is_err());
}
#[test]fn reply_bounds_are_checked_before_body_read(){
    struct HeaderOnly{bytes:Cursor<Vec<u8>>,reads:usize}
    impl Read for HeaderOnly{fn read(&mut self,b:&mut[u8])->std::io::Result<usize>{self.reads+=1;assert_eq!(self.reads,1,"Body read before bound rejection");self.bytes.read(b)}}
    let c=codec();let q=c.encode_request(12,&discovery()).unwrap();
    for n in [335,c.maximum_result().unwrap()+1]{let mut h=app::header(12,91,336).unwrap();h[4..8].copy_from_slice(&(n as u32).to_be_bytes());let mut r=HeaderOnly{bytes:Cursor::new(h),reads:0};assert!(c.read_reply(&mut r,&q).is_err());assert_eq!(r.reads,1);}
}
fn from_echo(b:&[u8])->Vec<u8>{
    let cat=u32_at(b,84) as usize;let inner=&b[92+cat..];let source=u32_at(inner,84) as usize;
    let mut old=app::header(u32_at(inner,8),88,180).unwrap();old.extend(&inner[16..84]);old.extend(&inner[92+source..188+source]);
    let mut q=app::header(u32_at(b,8),90,264).unwrap();q.extend(&b[16..84]);q.extend(old);q
}
fn observe(r:&native::Reply,peer:protocol::extensions::Selected)->battle_follow::Observation{
    let q=battle_read::Request{context:r.request.canonical.context,channel_id:r.request.canonical.channel,world:r.canonical.attachment.as_ref().unwrap().world.clone()};
    let mut f=battle_follow::Follower::new_selected(q.clone(),peer).unwrap();f.accept(battle_read::UnvalidatedReply{echo:q,state:battle_read::State::Data,data:r.canonical.data.clone()}).unwrap();f.observation().unwrap().clone()
}
fn stamp(o:&battle_follow::Observation)->Value{let s=battle_window::Stamp::of(o);json!({"source":s.context.key.source_id,"object_u64_hex":format!("{:016X}",s.context.key.object_id),
    "epoch_u64_hex":format!("{:016X}",s.context.epoch),"channel":s.channel,"room":s.world[0],"match":s.world[1],"generation":s.world[2],"entity":s.entity,"life":s.life,"revision":s.revision})}
fn scope(r:&Reply,o:&battle_follow::Observation)->native::Scope{native::Scope{connection_incarnation:431,context:r.request.native.canonical.context,
    channel:r.request.native.canonical.channel,joined_room:o.data().room.id,room_revision:o.data().room.revision}}
fn pinned_json(cfg:&Value,name:&str)->Value{let v=cfg["vectors"].as_array().unwrap().iter().find(|v|v["name"]==name).unwrap();let b=fs::read(v["path"].as_str().unwrap()).unwrap();assert_eq!(sha(&b),v["sha256"].as_str().unwrap().to_ascii_lowercase());serde_json::from_slice(&b).unwrap()}
fn row(c:CatalogCodec,family:&str,name:&str,b:&[u8],q:&[u8],peer:protocol::extensions::Selected,captured:bool)->Value{
    let r=c.decode_reply(b,q).unwrap();assert_eq!(c.encode_request(u32_at(q,8),r.request()).unwrap(),q);
    let cat=r.catalog().unwrap();assert_eq!(cat,identity());assert_eq!(cat.encode().unwrap(),b[92..260]);
    let nested=&b[260..];let n=u32_at(nested,84) as usize;assert_eq!(r.native().source.as_ref().unwrap().encode(base().capabilities).unwrap(),nested[92..92+n]);
    let data=r.native().canonical.data.as_ref().map(|d|{let o=observe(r.native(),peer);assert_eq!(o.data(),d);json!({"owner":stamp(&o),"map":d.manifest.map,"members":d.room.members.0.len(),"actors":d.actors.0.len(),"phase":d.world.phase as u8})});
    let rq=&r.request().native.canonical;
    json!({"family":family,"case":name,"kind":91,"sha256":sha(b),"bytes":b.len(),"source":rq.context.key.source_id,"object_u64_hex":format!("{:016X}",rq.context.key.object_id),
        "epoch_u64_hex":format!("{:016X}",rq.context.epoch),"channel":rq.channel,"room":rq.room,"match":rq.match_id,"generation":rq.generation,
        "catalog_digest":format!("{:x}",Sha256::digest([b"LITO-NATIVE-CATALOG-IDENTITY\0".as_slice(),&cat.encode().unwrap()].concat())),
        "source_exact_reencode":true,"data":data,"provided_request_bytes_verified":captured,
        "captured_TLS_request_verified":captured && family.contains("SQL"),"expected_request_reconstructed_from_echo":!captured})
}
#[test]
#[ignore="own frozen catalog90/91 service/TLS bytes, real LTtext + Native CPU partial; no live SQL/TLS"]
fn fixed_catalog_publications_proofs_owner_and_partial_completion(){
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));let pointer:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r431-catalog-snapshot.json")).unwrap()).unwrap();
    let bytes=fs::read(pointer["input"].as_str().unwrap()).unwrap();assert_eq!(sha(&bytes),pointer["input_sha256"].as_str().unwrap().to_ascii_lowercase());let input:Value=serde_json::from_slice(&bytes).unwrap();
    let c=codec();let peer=protocol::extensions::Selected::new(base(),0).unwrap();let mut rows=Vec::new();let mut negatives=Vec::new();
    for family in ["candidate","replay"]{
        let cfg=&input[family];let v=pinned_json(cfg,"catalog-vectors.json");let bad=pinned_json(cfg,"catalog-negative-vectors.json");let tls=pinned_json(cfg,"catalog-tls-vectors.json");
        for name in ["discovery_7","discovery_42","prepare","loading","loaded_7","active_42","leave_7","leave_42"]{
            let q=hex(v[format!("{name}_request90_hex")].as_str().unwrap());let b=hex(v[format!("{name}_result91_hex")].as_str().unwrap());rows.push(row(c,family,name,&b,&q,peer,true));
        }
        let q=hex(v["loaded_7_request90_hex"].as_str().unwrap());let reply=c.decode_reply(&hex(v["loaded_7_result91_hex"].as_str().unwrap()),&q).unwrap();let source=reply.native().source.as_ref().unwrap();
        for (name,value) in bad.as_object().unwrap(){let b=hex(value.as_str().unwrap());let stage=if name.ends_with("result91"){
            assert!(c.decode_reply(&b,&from_echo(&b)).is_err());"reply catalog presence"
        }else{match c.decode_request(&b){Err(_)=>"request structure",Ok(q)=>{assert!(q.check_authority(&identity(),source).is_err(),"{name}");"catalog/native proof authority"}}};
            negatives.push(json!({"family":family,"case":name,"sha256":sha(&b),"rejected_by":stage,"Java_service_ledger_not_replayed":true}));
        }
        for key in ["source7_full_catalog91","source42_full_catalog91"]{for (name,value) in tls[key].as_object().unwrap(){let b=hex(value.as_str().unwrap());let q=from_echo(&b);let r=c.decode_reply(&b,&q).unwrap();
            assert_eq!(r.request().native.canonical.context.key.source_id,if key.starts_with("source7_"){7}else{42});rows.push(row(c,&format!("{family}-TLS-{key}"),name,&b,&q,peer,false));
        }}
        // Old89 fields are actual captures nested in catalog91, not independent
        // authority. Match each capture byte-for-byte to a full91 publication.
        for name in ["discovery89_hex","preparing89_hex","loading89_hex","loaded89_hex","active89_hex"]{
            let b=hex(tls[name].as_str().unwrap());let mut found=None;
            for key in ["source7_full_catalog91","source42_full_catalog91"]{for value in tls[key].as_object().unwrap().values(){let full=hex(value.as_str().unwrap());if full[260..]==b {found=Some(full);break;}}}
            let full=found.unwrap();let q=from_echo(&full);let r=c.decode_reply(&full,&q).unwrap();
            let o=r.native().canonical.data.as_ref().map(|_|stamp(&observe(r.native(),peer)));
            rows.push(json!({"family":family,"case":name,"kind":89,"sha256":sha(&b),"nested_in_validated_catalog91":true,"owner":o,"source":r.request().native.canonical.context.key.source_id}));
        }
        let b=hex(tls["read61_hex"].as_str().unwrap());let reply=battle_read::decode_body_selected(&b[16..],peer).unwrap();let mut f=battle_follow::Follower::new_selected(reply.echo.clone(),peer).unwrap();f.accept(reply).unwrap();let o=f.observation().unwrap().clone();
        rows.push(json!({"family":family,"case":"READ61","kind":61,"sha256":sha(&b),"owner":stamp(&o)}));
        let b=hex(tls["move65_hex"].as_str().unwrap());let length=116+u32_at(&b,120) as usize;let mut q=b[..length].to_vec();q[4..8].copy_from_slice(&(length as u32).to_be_bytes());q[12..14].copy_from_slice(&64u16.to_be_bytes());
        let guard=battle_action::Guard{stamp:battle_window::Stamp::of(&o),target:None};let r=battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).unwrap();assert_eq!(r.disposition,battle_operation::Disposition::Applied);
        let other=hex(tls["active89_hex"].as_str().unwrap());let full=tls["source42_full_catalog91"].as_object().unwrap().values().map(|v|hex(v.as_str().unwrap())).find(|b|b[260..]==other).unwrap();let other=c.decode_reply(&full,&from_echo(&full)).unwrap();let borrowed=battle_action::Guard{stamp:battle_window::Stamp::of(&observe(other.native(),peer)),target:None};assert!(battle_operation::read_reply_selected(&mut Cursor::new(&b),borrowed,&q,peer).is_err());
        rows.push(json!({"family":family,"case":"MOVE65","kind":65,"sha256":sha(&b),"owner":stamp(&o),"disposition":format!("{:?}",r.disposition),"borrowed_source42_guard_refused":true,"captured_request_verified":false}));
    }
    assert_eq!(rows.len(),68);assert_eq!(negatives.len(),40);
    let v=pinned_json(&input["candidate"],"catalog-vectors.json");let bytes=hex(v["loading_result91_hex"].as_str().unwrap());let request=hex(v["loading_request90_hex"].as_str().unwrap());
    let reply=c.decode_reply(&bytes,&request).unwrap();let o=observe(reply.native(),peer);let scope=scope(&reply,&o);let mut owner=CatalogOwner::local_experiment(c).unwrap();let ticket=owner.stage(scope,reply).unwrap();let job=owner.begin_renderer(ticket,&o).unwrap();
    let catalog:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r429-catalog-snapshot.json")).unwrap()).unwrap();let root=Path::new(catalog["root"].as_str().unwrap());
    let verified=job.verify_catalog(root).unwrap();assert_eq!(job.catalog(),identity());
    for (name,change) in [("raw",0),("decoded",1),("offset",2)]{let mut cat=identity();if change==0{cat.raw_member.sha256[0]^=1}else if change==1{cat.decoded_sha256[0]^=1}else{cat.offset+=1;}
        assert!(resources::verify_native_catalog_member(root,cat.index,cat.pak,cat.offset,cat.raw_member,cat.decoded_sha256,job.cancellation()).is_err(),"{name}");}
    let mut other=CatalogOwner::local_experiment(c).unwrap();let other_ticket=other.stage(scope,c.decode_reply(&bytes,&request).unwrap()).unwrap();let other_job=other.begin_renderer(other_ticket,&o).unwrap();let foreign=other_job.verify_catalog(root).unwrap();
    assert!(!Arc::ptr_eq(&foreign.cancel,&verified.cancel)); // real verification is worker-bound
    let fixture=p.join("target/r425-native-dm564-fixture");let sprites=fixture.join("native-archives");let sources=NativeSources{map_root:fixture.join("native-map"),archive_root:sprites.clone()};
    let assets=NativeResourceAdapter::pinned_inspection().load_static(&sources,job.source_plan(),Default::default(),job.cancellation()).unwrap();
    let a=NativeActionSnapshot::load_firepot(job.source_plan(),root,&sprites,identity().action_pins(),38,job.cancellation()).unwrap();let slices=vec![a.render_catalog_millis(0,job.cancellation()).unwrap()];
    // Attempt mixing genuine bytes verified by another worker.
    assert!(job.finish_partial(foreign,assets,slices).is_err());
    let job=owner.begin_renderer(ticket,&o).unwrap();let assets=NativeResourceAdapter::pinned_inspection().load_static(&sources,job.source_plan(),Default::default(),job.cancellation()).unwrap();let slices=vec![a.render_catalog_millis(83,job.cancellation()).unwrap()];
    let completion=job.finish_partial(verified,assets,slices).unwrap();assert!(other.complete_partial(other_ticket,completion).is_err());
    let old_job=owner.begin_renderer(ticket,&o).unwrap();let next=owner.stage(scope,c.decode_reply(&bytes,&request).unwrap()).unwrap();assert!(old_job.verify_catalog(root).is_err());assert!(owner.begin_renderer(ticket,&o).is_err());
    let job=owner.begin_renderer(next,&o).unwrap();let verified=job.verify_catalog(root).unwrap();let assets=NativeResourceAdapter::pinned_inspection().load_static(&sources,job.source_plan(),Default::default(),job.cancellation()).unwrap();let slices=vec![a.render_catalog_millis(83,job.cancellation()).unwrap()];let completion=job.finish_partial(verified,assets,slices).unwrap();owner.complete_partial(next,completion).unwrap();assert!(owner.partial_available());assert!(!owner.loaded_admission_available());owner.retire();assert!(!owner.partial_available());
    fs::write(p.join("logs/continuation-20261003/r431-native-catalog-details.json"),serde_json::to_vec_pretty(&json!({"state":"FIXED_CATALOG_PUBLICATIONS_AND_PARTIAL_VERIFIED","rows":rows,"negatives":negatives,
        "catalog_identity_bytes":168,"request90_bytes":264,"catalog_digest":identity().digest().unwrap().iter().map(|b|format!("{b:02x}")).collect::<String>(),"raw_decoded_index_range_verified":true,
        "opaque_catalog_partial_verified":true,"Java_service_ledger_replayed":false,"base_actor_resources_covered":false,"Rust_live_SQL_TLS":false,"LOADED":false,"GUI":0})).unwrap()).unwrap();
}

#[test]
#[ignore="own frozen actual SQL catalog90/91,15/17,61/65 bytes; no live SQL/TLS"]
fn fixed_sql_catalog_owner_generation_proofs_and_equipment_publications(){
    use super::super::equipment;
    use std::collections::{BTreeMap,BTreeSet};
    let p=Path::new(env!("CARGO_MANIFEST_DIR"));let ptr:Value=serde_json::from_slice(&fs::read(p.join("logs/continuation-20261003/r431-catalog-sql-snapshot.json")).unwrap()).unwrap();
    let b=fs::read(ptr["input"].as_str().unwrap()).unwrap();assert_eq!(sha(&b),ptr["input_sha256"].as_str().unwrap().to_ascii_lowercase());let cfg:Value=serde_json::from_slice(&b).unwrap();
    let mut ack=base();ack.capabilities|=protocol::CAP_EQUIPMENT;let c=CatalogCodec::local_experiment(ack,768).unwrap();let peer=protocol::extensions::Selected::new(ack,0).unwrap();
    let mut rows=Vec::new();let mut negatives=Vec::new();let mut equipment_rows=Vec::new();let mut proof_rows=Vec::new();let mut receipts=Vec::new();
    for family in ["candidate","replay"]{
        let v=pinned_json(&cfg[family],"catalog-sql-wire90-91.json");let tls=pinned_json(&cfg[family],"catalog-sql-tls-vectors.json");
        let mut proof_set=BTreeSet::new();let mut proof_scopes=BTreeSet::new();let mut publication_count=0;let mut source=None;
        let mut source42_context=None;
        for (name,value) in v.as_object().unwrap().iter().filter(|(n,_)|n.ends_with("_request90_hex")){
            let q=hex(value.as_str().unwrap());let result_name=name.replace("_request90_hex","_result91_hex");let bytes=hex(v[&result_name].as_str().unwrap());
            let r=c.decode_reply(&bytes,&q).unwrap();source=Some(r.native().source.clone().unwrap());let request=r.request();
            let owner=request.native.canonical.context;assert_eq!(owner.key.object_id,10275);assert!(matches!(owner.key.source_id,7|42));
            if owner.key.source_id==42{source42_context=Some(owner);}
            if request.native.canonical.action==world::Action::Loaded {
                let q=&request.native.canonical;assert!(proof_set.insert(request.catalog_load_proof));assert!(proof_scopes.insert((q.context.key.source_id,q.generation)));
                proof_rows.push(json!({"family":family,"case":name,"source":q.context.key.source_id,"object_u64_hex":format!("{:016X}",q.context.key.object_id),"epoch_u64_hex":format!("{:016X}",q.context.epoch),
                    "channel":q.channel,"room":q.room,"match":q.match_id,"generation":q.generation,"proof_sha256":request.catalog_load_proof.iter().map(|b|format!("{b:02x}")).collect::<String>(),"domain_recomputed":true}));
            }
            if let Some(data)=r.native().canonical.data.as_ref(){assert!(data.personal.equipment.is_none());assert!(data.actors.0.iter().all(|a|a.vital.is_none()&&a.resources.is_none()&&a.buffs.is_none()));}
            rows.push(row(c,&format!("SQL-{family}"),name,&bytes,&q,peer,true));publication_count+=1;
        }
        // Candidate34 / replay35 differ by one normal preparation poll.
        assert_eq!(publication_count,if family=="candidate"{34}else{35});assert_eq!(proof_set.len(),4);assert_eq!(proof_scopes.len(),4);
        let source=source.unwrap();
        for (name,value) in v.as_object().unwrap().iter().filter(|(n,_)|n.starts_with("negative_")){
            let b=hex(value.as_str().unwrap());let stage=match c.decode_request(&b){Err(_)=>"old88 structural route rejected",Ok(q)=>{
                if name.contains("cross_source_scope") {
                    // The attacked recipient (source42) is producer transport
                    // evidence. Wire alone has a valid source7 proof. Show the
                    // trusted source42 context check, not a codec rejection.
                    assert!(q.check_authority(&identity(),&source).is_ok());assert_ne!(q.native.canonical.context,source42_context.unwrap());
                    "valid wire proof; differs from trusted source42 publication context; actual socket rejection remains producer evidence"
                }else{assert!(q.check_authority(&identity(),&source).is_err(),"{name}");"catalog/native proof authority rejected"}
            }};
            negatives.push(json!({"family":family,"case":name,"sha256":sha(&b),"validation":stage,"actual_TLS_rejection_replayed_by_Rust":false}));
        }
        let mut last=BTreeMap::new();let mut changed=0;let mut source42_initial=None;
        for (name,value) in v.as_object().unwrap().iter().filter(|(n,_)|n.contains("_publication15_")||n.contains("_publication17_")){
            let b=hex(value.as_str().unwrap());let r=app::decode_response_version(&b,47).unwrap();
            match r.response {
                app::Response::CharacterData(r)=>{let data=r.result.unwrap();assert_eq!(data.items.len(),29);assert_eq!(data.context.key.object_id,10275);assert_eq!(data.character.class,0);assert_eq!(data.character.sex,0);
                    equipment_rows.push(json!({"family":family,"case":name,"kind":15,"sha256":sha(&b),"source":data.context.key.source_id,"object_u64_hex":format!("{:016X}",data.context.key.object_id),"epoch_u64_hex":format!("{:016X}",data.context.epoch),"items":29}));}
                app::Response::Equipment(r)=>{assert!(matches!(r.status,equipment::Status::Ok|equipment::Status::Conflict|equipment::Status::BindingMismatch));let source=r.request.context.key.source_id;
                    let Some(s)=r.snapshot else{assert_eq!(r.status,equipment::Status::BindingMismatch);assert_eq!(source,42);assert!(!r.changed);equipment_rows.push(json!({"family":family,"case":name,"kind":17,"sha256":sha(&b),"source":source,"status":"BindingMismatch","snapshot":null,"changed":false}));continue;};
                    assert_eq!(s.source.items.len(),29);assert_eq!(s.source.context,r.request.context);
                    if source==42{let current=(s.revision,s.flags.clone());if let Some(before)=&source42_initial{assert_eq!(&current,before);}else{source42_initial=Some(current);}}
                    if r.changed{assert_eq!(r.status,equipment::Status::Ok);assert_eq!(source,7);assert_eq!(r.request.action,equipment::Action::Set);assert_eq!(r.request.item_id,10304);assert!(!r.request.desired);assert_eq!(s.revision,r.request.expected_revision+1);assert_eq!(s.equipped(10304),Some(false));changed+=1;}
                    if r.status==equipment::Status::Conflict{assert!(!r.changed);assert_eq!(r.request.action,equipment::Action::Set);assert!(s.revision!=r.request.expected_revision || s.view_token!=r.request.expected_view_token);}
                    equipment_rows.push(json!({"family":family,"case":name,"kind":17,"sha256":sha(&b),"source":source,"object_u64_hex":format!("{:016X}",r.request.context.key.object_id),"epoch_u64_hex":format!("{:016X}",r.request.context.epoch),"revision":s.revision,
                        "items":s.source.items.len(),"status":format!("{:?}",r.status),"changed":r.changed,"expected_revision":r.request.expected_revision,"item10304_equipped":s.equipped(10304),"CAS_request_view_token_preserved":true,"SCENE_ONLY_PERSONAL_is_independent":true}));last.insert(source,s);}
                _=>panic!("Unexpected inventory/equipment publication")
            }
        }
        assert_eq!(changed,1);assert_eq!(last[&7].equipped(10304),Some(false));assert_eq!(last[&42].equipped(10304),Some(true));
        assert_ne!(last[&7].source.context,last[&42].source.context);assert_ne!(last[&7].view_token,last[&42].view_token);
        // Main discovery91 was overwritten by later negative-client map key
        // reuse (source/action/requestID omits connection). Its standalone89
        // capture is still parsed but cannot claim a surviving full91 pair.
        for name in ["discovery89_hex","preparing89_hex","loading89_hex","loaded89_hex","active89_hex"]{
            let bytes=hex(tls[name].as_str().unwrap());let nested=v.as_object().unwrap().iter().filter(|(n,_)|n.ends_with("_result91_hex")).any(|(_,value)|hex(value.as_str().unwrap())[260..]==bytes);
            assert_eq!(nested,name!="discovery89_hex","{family}/{name}");
            let n=u32_at(&bytes,84) as usize;let mut q=app::header(u32_at(&bytes,8),88,180).unwrap();q.extend(&bytes[16..84]);q.extend(&bytes[92+n..188+n]);let r=c.inner().unwrap().decode_reply(&bytes,&q).unwrap();assert_eq!(r.source.as_ref().unwrap(),&source);
            if r.canonical.data.is_some(){observe(&r,peer);}
            receipts.push(json!({"family":family,"case":name,"kind":89,"sha256":sha(&bytes),"source":r.request.canonical.context.key.source_id,"object_u64_hex":format!("{:016X}",r.request.canonical.context.key.object_id),"epoch_u64_hex":format!("{:016X}",r.request.canonical.context.epoch),
                "nested_in_surviving_catalog91":nested,"producer_connection_map_key_collision":!nested,"captured_request_verified":false}));
        }
        let b=hex(tls["read61_source7_before_move_hex"].as_str().unwrap());let r=battle_read::decode_body_selected(&b[16..],peer).unwrap();let mut f=battle_follow::Follower::new_selected(r.echo.clone(),peer).unwrap();f.accept(r).unwrap();let o=f.observation().unwrap().clone();
        for name in ["move65_source7_hex","stale_life65_source7_hex"]{let b=hex(tls[name].as_str().unwrap());let length=116+u32_at(&b,120) as usize;let mut q=b[..length].to_vec();q[4..8].copy_from_slice(&(length as u32).to_be_bytes());q[12..14].copy_from_slice(&64u16.to_be_bytes());
            let mut expected=battle_window::Stamp::of(&o);if name.starts_with("stale"){expected.life=u64_at(&b,72) as i64;}let guard=battle_action::Guard{stamp:expected,target:None};let r=battle_operation::read_reply_selected(&mut Cursor::new(&b),guard,&q,peer).unwrap();
            assert_eq!(r.disposition==battle_operation::Disposition::Applied,name.starts_with("move"));receipts.push(json!({"family":family,"case":name,"sha256":sha(&b),"baseline_owner":stamp(&o),"request_life":expected.life,"disposition":format!("{:?}",r.disposition),"captured_request_verified":false}));
        }
    }
    assert_eq!(rows.len(),69);assert_eq!(negatives.len(),10);assert_eq!(equipment_rows.len(),16);assert_eq!(proof_rows.len(),8);assert_eq!(receipts.len(),14);
    fs::write(p.join("logs/continuation-20261003/r431-native-catalog-sql-details.json"),serde_json::to_vec_pretty(&json!({"state":"FIXED_SQL_CATALOG_BYTES_CONSUMED","rows":rows,"negatives":negatives,"equipment_publications":equipment_rows,"load_proofs":proof_rows,"receipts":receipts,
        "positive_pairs":69,"candidate_pairs":34,"replay_pairs":35,"distinct_source_generation_proofs_per_family":4,"SQL_inventory_items_per_source":29,"SCENE_ONLY_personal_equipment_absent_is_scope_only":true,
        "main_discovery91_overwritten_by_producer_map_key_collision":true,"discovery89_standalone_verified_only":true,"producer_SQL_TLS":true,"Rust_live_SQL_TLS":false,"base_actor_resources_covered":false,"LOADED":false,"GUI":0})).unwrap()).unwrap();
}
