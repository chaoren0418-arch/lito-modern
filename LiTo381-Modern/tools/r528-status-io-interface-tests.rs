use super::*;
fn diagnostic()->Value{json!({"maximum_move_attempts":6,"retry_delay_ms":20,"denied_moves":0,"recovered_publications":0,"exhausted_publications":0,"first_error":null})}
fn provider_fixture(status_file:PathBuf)->Provider{
 let run_root=Path::new(SERVER_RUNS).join(format!("nw-{}","b".repeat(32)));
 Provider{consumer:ConsumerSpec{fresh_run_id:run_root.file_name().unwrap().to_str().unwrap().into(),endpoint:"127.0.0.1:12345".into(),credentials_file:run_root.join("game-player.properties"),source_raw_hex:String::new(),guest_alias_raw_hex:String::new(),host_alias_raw_hex:String::new(),channel:0,room_id:1,read_cap:31,write_cap:157},status_file,run_root,helper_pid:12345,port:12345,lease_end_epoch_ms:now_ms().unwrap()+600_000,status_poll_started:std::cell::Cell::new(None)}
}
fn status_fixture(p:&Provider)->Value{
 json!({"schema":"server.modern485.provider.status.v1","state":"READY","run_root":p.run_root,"bundle_root":BUNDLE,"candidate_sha256":ready_flow::PRODUCER_SHA,"helper_pid":p.helper_pid,"port":p.port,"login_host":"127.0.0.1","game_host":"127.0.0.2","lease_end_epoch_ms":p.lease_end_epoch_ms,"support_host_enabled":true,"background_failure":"","unexpected_WORLD_start_result":false,"status_publication":diagnostic()})
}
#[test]fn new_host_poll_cadence_has_no_sleep_cache_or_catchup(){
 let last=std::cell::Cell::new(None);let t=Instant::now();
 assert!(claim_status_poll(&last,t));
 for ms in [0,2,20,100,199]{assert!(!claim_status_poll(&last,t+Duration::from_millis(ms)));}
 assert!(claim_status_poll(&last,t+Duration::from_millis(200)));
 assert!(!claim_status_poll(&last,t+Duration::from_millis(399)));
 assert!(claim_status_poll(&last,t+Duration::from_millis(400)));
 assert!(claim_status_poll(&last,t+Duration::from_millis(1300)));
 assert!(!claim_status_poll(&last,t+Duration::from_millis(1300)));
 assert!(!claim_status_poll(&last,t+Duration::from_millis(1499)));
 assert!(claim_status_poll(&last,t+Duration::from_millis(1500)));
}
#[test]fn accepted_repair_bytes_pin_all_new_manifest_interfaces(){
 let h=include_bytes!("accepted-status-io-handoff.json");
 let l=include_bytes!("accepted-status-io-launch.json");
 let p=include_bytes!("accepted-status-io-manifest.json");
 let m=accepted_cold_bytes(h,l,p).unwrap();assert_eq!(m["files"].as_array().unwrap().len(),26);
 for which in 0..3{
  let(mut a,mut b,mut c)=(h.to_vec(),l.to_vec(),p.to_vec());
  match which{0=>a[0]^=1,1=>b[0]^=1,_=>c[0]^=1};assert!(accepted_cold_bytes(&a,&b,&c).is_err());
 }
 let handoff:Value=serde_json::from_slice(h).unwrap();let r=&handoff["precise_retry_boundary"];
 assert_eq!(r["total_move_attempts_max"],6);assert_eq!(r["total_requested_sleep_ms_max"],100);
 assert_eq!(r["strict_wallclock_bound"],false);assert_eq!(r["scope"],"status.json only, no ownership-request retry");
}
#[test]fn publication_diagnostics_do_not_grant_or_replace_status_guards(){
 let p=provider_fixture(Path::new("D:/unused-cold-status.json").into());let mut s=status_fixture(&p);
 s["status_publication"]["denied_moves"]=json!(64);s["status_publication"]["recovered_publications"]=json!(32);s["status_publication"]["exhausted_publications"]=json!(17);
 s["status_publication"]["first_error"]=json!({"utc":"cold-test-only","type":"AccessDeniedException","message":"benign diagnostic fixture","cause_confirmed":false});
 checked_status_header(&s,&p).unwrap(); // No zero-counter or no-error admission rule.
 let d=publication_diagnostic(&s).unwrap();assert_eq!(d["denied_moves"],64);
 s["background_failure"]=json!("failed");assert!(checked_status_header(&s,&p).is_err());
 s["background_failure"]=json!("");s["unexpected_WORLD_start_result"]=json!(true);assert!(checked_status_header(&s,&p).is_err());
 s["unexpected_WORLD_start_result"]=json!(false);s.as_object_mut().unwrap().remove("status_publication");assert!(checked_status_header(&s,&p).is_err());
 s["status_publication"]=diagnostic();s["status_publication"]["maximum_move_attempts"]=json!(7);assert!(checked_status_header(&s,&p).is_err());
}
#[test]fn integrated_provider_reads_benign_own_snapshot_through_same_handle(){
 let dir=PathBuf::from(std::env::var("R528_STATUS_INTEGRATION_ROOT").unwrap());
 assert_eq!(dir.parent(),Some(Path::new("D:/lito-dev-runtime/20261003-supervision/modern")));
 assert!(dir.file_name().unwrap().to_str().unwrap().starts_with("r528-status-integration-"));
 let path=dir.join("new-interface-status-only.json");let p=provider_fixture(path.clone());let body=serde_json::to_vec(&status_fixture(&p)).unwrap();
 let mut f=fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap();f.write_all(&body).unwrap();drop(f);
 let s=fresh_status(&p).unwrap();assert!(p.status_poll_started.get().is_some());
 assert_eq!(s["_modern_status_reader"]["method"],"same_opened_handle_metadata_and_bytes");
 assert_eq!(s["_modern_status_reader"]["bytes"],body.len());
 let mtime=fs::metadata(&path).unwrap().modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_millis()as u64;
 assert_eq!(s["_modern_status_reader"]["modified_epoch_ms"],mtime);
 assert_eq!(s["status_publication"],diagnostic());
 assert!(!claim_status_poll(&p.status_poll_started,p.status_poll_started.get().unwrap()+Duration::from_millis(199)));
}
#[test]fn accepted_new_bundle_and_failed_nw975_boundary_are_wired(){
 let mut run=Path::new(SERVER_RUNS).join(format!("nw-{}","c".repeat(32)));
 let sync=json!({"enabled":true,"stage":INITIAL,"requires_normal_Guest_baseline_NotReady0":true,"Host_participant_state1_sent":false,"Host_owner_opcode_only_sent":false,"failure_Host_state0_attempted":false,"baseline_D1_0_write_epochs":[],"Host_D1_1_write_epochs":[],"E103_write_epochs":[],"Guest_notready_D1_write_epochs":[],"E100_write_epochs":[]});
 let mut v=json!({"schema":"server.modern485.provider.run.v1","state":"READY","run_identity":run.file_name().unwrap().to_str().unwrap(),"run_root":run,"bundle_root":BUNDLE,"candidate":{"path":Path::new(BUNDLE).join("candidate/Ready485.jar"),"bytes":8906219,"sha256":ready_flow::PRODUCER_SHA},"ownership_file":run.join("owned-process-live.json"),"status_file":run.join("status.json"),"credentials_file":run.join("game-player.properties"),"login_endpoint":{"host":"127.0.0.1","port":12345},"game_endpoint":{"host":"127.0.0.2","port":12345},"source":{"id":72,"name":"Fx485Moon"},"roles":[{"role":"Host","account":"Fx485Host","raw_selection_alias":"Fx485Host(Fx485Moon)","physical_id":41001},{"role":"Guest","account":"Fx485Guest","raw_selection_alias":"Fx485Guest(Fx485Moon)","physical_id":41002}],"support_host_enabled":true,"host_ready_after_guest_notready":true,"host_sync_profile":sync,"room":{},"lease_end_epoch_ms":1,"server_lifecycle_owner":true,"rust_acceptance":false});
 let raw=serde_json::to_vec(&v).unwrap();static_ready(&raw,&digest(&raw)).unwrap();
 v["bundle_root"]=json!("D:/lito-dev-runtime/20261003-supervision/server/modern-485-provider-frozen-7f8b15934fc34983bca934785a4c5370");let raw=serde_json::to_vec(&v).unwrap();assert!(static_ready(&raw,&digest(&raw)).is_err());
 v["bundle_root"]=json!(BUNDLE);run=Path::new(SERVER_RUNS).join("nw-975e97ed0f74480c9c92edd7d6b7d51e");
 v["run_identity"]=json!(run.file_name().unwrap().to_str().unwrap());v["run_root"]=json!(run);
 for (key,name) in [("ownership_file","owned-process-live.json"),("status_file","status.json"),("credentials_file","game-player.properties")]{v[key]=json!(run.join(name));}
 let raw=serde_json::to_vec(&v).unwrap();let e=static_ready(&raw,&digest(&raw)).err().unwrap();assert!(e.to_string().contains("retired runs refused"));
}
