//! Replay the same operations against the unapplied Core and compare every CSV field.
#[allow(dead_code)]
#[path="D:/lineage開發/3-lito開發/lito重建版/維護工具/candidates/native-background-clock/native_water_clock.rs"]
mod candidate;
#[path="D:/lineage開發/3-lito開發/lito重建版/維護工具/candidates/native-background-clock/ready_filter.rs"]
mod ready_filter;
use candidate::{Callback,Core};
use std::{fmt::Write,path::PathBuf};
#[derive(Default)]
struct Clock {now:u32,step:u32,reads:u32}
impl Clock {fn read(&mut self)->u32 {let value=self.now;self.now=self.now.wrapping_add(self.step);self.reads+=1;value}}
fn schedule(c:&mut Core,k:&mut Clock,delay:u32,callback:Callback,owner:u32){c.schedule(delay,callback,owner,||k.read()).unwrap();}
fn bg(c:&mut Core,k:&mut Clock){c.background_callback(||k.read()).unwrap();}
fn green(c:&mut Core,k:&mut Clock){c.green_callback(||k.read()).unwrap();}
fn initial(c:&mut Core,k:&mut Clock){c.enabled=false;bg(c,k);green(c,k);}
fn tick(c:&mut Core,k:&mut Clock)->bool {!c.dispatch(||k.read()).unwrap().is_empty()}
fn kind(callback:Callback)->String {match callback {
    Callback::Water=>"phase".into(),Callback::Green=>"green".into(),Callback::MapAux=>"map_aux".into(),
    Callback::Kept(6)=>"background".into(),Callback::Kept(i)=>format!("keep{i}"),Callback::User(1)=>"note1".into(),
    _=>panic!("unknown callback"),
}}
struct Report {states:String,queues:String,names:Vec<&'static str>,queue_rows:usize}
impl Report {
    fn new()->Self {Self{states:"scenario,x,y,green,phase,counter,enabled,queued,clock_now,clock_reads,fired\n".into(),
        queues:"scenario,index,deadline,callback,owner\n".into(),names:Vec::new(),queue_rows:0}}
    fn save(&mut self,name:&'static str,c:&Core,k:&Clock,fired:bool){
        assert!(!self.names.contains(&name));self.names.push(name);let offsets=c.background_offsets();
        writeln!(self.states,"{name},{},{},{},{},{},{},{},{},{},{}",offsets[0],offsets[1],c.green_counter(),c.phase(),c.counter(),
            u8::from(c.enabled),c.events().len(),k.now,k.reads,u8::from(fired)).unwrap();
        for (i,e) in c.events().iter().enumerate(){writeln!(self.queues,"{name},{i},{},{},{}",e.due,kind(e.callback),e.owner).unwrap();self.queue_rows+=1;}
    }
}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
    let native=PathBuf::from(&args[1]);let out=PathBuf::from(&args[2]);assert!(native.is_absolute()&&out.is_absolute());
    let native_states=std::fs::read_to_string(native.join("state-vectors.csv"))?.replace("\r\n","\n");
    let ready=native_states.contains("\nready_before,");
    let mut r=Report::new();let b=Callback::Kept(6);let w=Callback::Water;let n=Callback::User(1);
    let mut c=Core::default();let mut k=Clock{now:1000,..Default::default()};c.enabled=true;
    initial(&mut c,&mut k);r.save("initial",&c,&k,false);
    initial(&mut c,&mut k);r.save("initial_again",&c,&k,false);

    let mut c=Core::default();let mut k=Clock{now:1000,..Default::default()};
    schedule(&mut c,&mut k,1,b,0);schedule(&mut c,&mut k,2,b,0);schedule(&mut c,&mut k,3,b,9);
    schedule(&mut c,&mut k,4,n,0);schedule(&mut c,&mut k,5,w,0);green(&mut c,&mut k);
    c.background_rearm(||k.read())?;r.save("rearm",&c,&k,false);
    c.background_stop();r.save("stop",&c,&k,false);
    k.now=1003;let fired=tick(&mut c,&mut k);r.save("other_owner_due",&c,&k,fired);

    let setup=|| {
        let mut c=Core::default();let mut k=Clock{now:1000,..Default::default()};
        bg(&mut c,&mut k);green(&mut c,&mut k);schedule(&mut c,&mut k,70,w,0);schedule(&mut c,&mut k,80,n,0);
        for i in 0..9 {schedule(&mut c,&mut k,3000+u32::from(i),Callback::Kept(i),u32::from(i)+1);}
        (c,k)
    };
    let (mut c,mut k)=setup();r.save("map_before",&c,&k,false);c.map_filter(||k.read())?;r.save("map_after",&c,&k,false);
    let (mut c,mut k)=setup();schedule(&mut c,&mut k,60,Callback::MapAux,u32::MAX);r.save("scene_before",&c,&k,false);
    c.retire_scene();r.save("scene_after",&c,&k,false);
    if ready {
        let (mut c,mut k)=setup();schedule(&mut c,&mut k,60,Callback::MapAux,u32::MAX);
        r.save("ready_before",&c,&k,false);ready_filter::filter(&mut c);r.save("ready_after",&c,&k,false);
    }

    let mut c=Core::default();let mut k=Clock::default();bg(&mut c,&mut k);k.now=1000;k.step=7;
    let fired=tick(&mut c,&mut k);r.save("late",&c,&k,fired);
    k.step=0;k.now=1070;let fired=tick(&mut c,&mut k);r.save("late_early",&c,&k,fired);
    k.now=1071;let fired=tick(&mut c,&mut k);r.save("late_due",&c,&k,fired);

    let mut c=Core::default();let mut k=Clock::default();bg(&mut c,&mut k);bg(&mut c,&mut k);k.now=1000;k.step=7;
    let fired=tick(&mut c,&mut k);r.save("duplicate",&c,&k,fired);

    let mut c=Core::with_values(19,98)?;c.enabled=true;let mut k=Clock::default();
    bg(&mut c,&mut k);green(&mut c,&mut k);c.water_callback(||k.read())?;k.now=1000;k.step=7;
    let fired=tick(&mut c,&mut k);r.save("mixed",&c,&k,fired);

    let mut c=Core::with_background_offsets([u32::MAX,65535]);let mut k=Clock{now:0xfffffff0,..Default::default()};
    bg(&mut c,&mut k);r.save("wrap",&c,&k,false);
    let mut c=Core::default();let k=Clock::default();c.background_stop();r.save("stopped_idle",&c,&k,false);
    assert_eq!(r.names.len(),if ready {18}else{16});assert_eq!(r.queue_rows,if ready {90}else{74});
    let native_queues=std::fs::read_to_string(native.join("queue-vectors.csv"))?.replace("\r\n","\n");
    std::fs::write(out.join("Rust-state-vectors.csv"),&r.states)?;
    std::fs::write(out.join("Rust-queue-vectors.csv"),&r.queues)?;
    assert_eq!(r.states,native_states,"all state/counter/clock/dispatch fields");
    assert_eq!(r.queues,native_queues,"all queue ordering/deadline/callback/owner fields");
    let result=serde_json::json!({"status":"unapplied_Rust_background_queue_matches_original_selected_lifecycle",
        "snapshots":r.names.len(),"queue_rows":r.queue_rows,"scenarios":r.names,"native_scheduler_cancel_dispatch_and_filters_executed":true,
        "distinct_ready_reuse_filter_executed":ready,"ready_reuse_filter_removes_background":ready,
        "clock_source_stubbed":true,"whole_entry_cleanup_or_packet_handler_executed":false,
        "modern_scene_binding_verified":false,"candidate_applied_to_product":false,"GUI_or_real_clock_timing_verified":false,
        "actual_formal_sessions":0,"whole_goal_complete":false});
    std::fs::write(out.join("candidate-results.json"),serde_json::to_vec_pretty(&result)?)?;
    println!("background queue candidate matches all {} original snapshots and {} queue rows",r.names.len(),r.queue_rows);Ok(())
}
