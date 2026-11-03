//! Replay actual callback values; synthetic queue tests do not establish GUI time.
#[allow(dead_code)]
#[path="D:/lineage開發/3-lito開發/lito重建版/維護工具/candidates/native-background-clock/native_water_clock.rs"]
mod candidate;
use candidate::{Callback,Core,Event};
use serde_json::json;
use std::path::PathBuf;
fn main()->Result<(),Box<dyn std::error::Error>>{
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
    let input=PathBuf::from(&args[1]);let output=PathBuf::from(&args[2]);assert!(input.is_absolute()&&output.is_absolute());
    let mut cases=0;
    for line in std::fs::read_to_string(input)?.lines().skip(1){
        let v:Vec<u32>=line.split(',').map(|x|x.parse().unwrap()).collect();assert_eq!(v.len(),12);
        let mut c=Core::with_background_offsets([v[2],v[3]]);c.enabled=v[0]==1;
        let mut reads=0;c.background_callback(||{reads+=1;1000})?;
        assert_eq!(reads,1);assert_eq!(c.background_offsets(),[v[4],v[5]]);
        assert_eq!([v[6],v[7]],[v[4],v[5]]);assert_eq!((v[8],v[9],v[11]),(64,0,1));
        assert_eq!(c.events(),[Event{due:1064,callback:Callback::Kept(6),owner:0}]);cases+=1;
    }
    assert_eq!(cases,57);let mut queue_cases=Vec::new();
    let mut c=Core::with_values(7,99)?;c.enabled=true;c.background_callback(||1000)?;
    assert_eq!((c.phase(),c.counter(),c.green_counter()),(7,99,0));
    let mut green=Core::with_green_counter(u32::MAX);green.background_callback(||1000)?;assert_eq!(green.green_counter(),u32::MAX);
    queue_cases.push(json!({"name":"independent_water_and_green_values","offsets":c.background_offsets(),"events":c.events()}));

    let mut c=Core::default();c.schedule(0,Callback::Kept(6),0,||1000)?;
    let mut clocks=[1000,1107].into_iter();let fired=c.dispatch(||clocks.next().unwrap())?;
    assert!(clocks.next().is_none());assert_eq!(fired.len(),1);assert_eq!(c.background_offsets(),[65485,65535]);
    assert_eq!(c.events(),[Event{due:1171,callback:Callback::Kept(6),owner:0}]);
    queue_cases.push(json!({"name":"one_dispatch_cutoff_fresh_reschedule_no_catchup","fired":fired,"events":c.events()}));

    let mut c=Core::default();
    c.schedule(0,Callback::Kept(6),0,||50)?;c.schedule(0,Callback::Kept(6),0,||900)?;
    c.schedule(0,Callback::Kept(6),9,||1000)?;c.schedule(0,Callback::User(7),0,||888)?;
    c.background_rearm(||1000)?;
    assert_eq!(c.background_offsets(),[65485,65535]);
    assert_eq!(c.events(),[Event{due:888,callback:Callback::User(7),owner:0},Event{due:1000,callback:Callback::Kept(6),owner:9},Event{due:1064,callback:Callback::Kept(6),owner:0}]);
    queue_cases.push(json!({"name":"cancel_owner0_then_one_immediate_rearm","events":c.events()}));
    c.background_stop();assert_eq!(c.background_offsets(),[65485,65535]);
    assert_eq!(c.events(),[Event{due:888,callback:Callback::User(7),owner:0},Event{due:1000,callback:Callback::Kept(6),owner:9}]);
    queue_cases.push(json!({"name":"explicit_stop_preserves_values_and_other_owner","events":c.events()}));

    let mut c=Core::default();c.schedule(1,Callback::Kept(6),0,||1000)?;
    c.schedule(2,Callback::Kept(8),0,||1000)?;c.schedule(3,Callback::Water,0,||1000)?;
    c.map_filter(||1000)?;assert!(c.events().iter().any(|e|e.callback==Callback::Kept(6)));
    c.retire_scene();assert_eq!(c.events(),[Event{due:1001,callback:Callback::Kept(6),owner:0}]);
    assert_eq!(c.background_offsets(),[65484,65534]);
    queue_cases.push(json!({"name":"generic_source_filters_keep_slot6_without_advancing_values","events":c.events()}));

    let mut c=Core::default();for i in 0..4096 {c.schedule(0,Callback::User(i),i,||0)?;}
    let before=c.clone();assert!(c.background_callback(||0).is_err());assert_eq!(c,before);
    assert!(c.background_rearm(||0).is_err());assert_eq!(c,before);
    queue_cases.push(json!({"name":"modern_bounded_queue_rejects_before_publishing_partial_state","events":c.events().len()}));

    let mut c=Core::default();c.background_callback(||0xfffffff0)?;
    assert_eq!(c.events(),[Event{due:48,callback:Callback::Kept(6),owner:0}]);
    queue_cases.push(json!({"name":"u32_scheduled_deadline_wrap","events":c.events()}));
    assert_eq!(queue_cases.len(),7);
    let result=json!({"status":"isolated_shared_clock_background_candidate_matches_native_callback_values", "native_callback_cases":cases,
        "synthetic_queue_scenarios":queue_cases,"real_clock_or_GUI_timing_verified":false,"native_scheduler_body_executed":false,
        "actual_scene_start_stop_binding_verified":false,"candidate_applied_to_product":false,"product_source_changed":false,"actual_formal_sessions":0,"whole_goal_complete":false});
    std::fs::write(output,serde_json::to_vec_pretty(&result)?)?;println!("background queue candidate: {cases} native callback vectors and 7 synthetic queue scenarios passed");Ok(())
}
