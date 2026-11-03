use lito_modern::modern_lito::native_water_clock::{self as water,Callback,Core,History,Motion,Owned};
use std::cell::Cell;
use sha2::{Digest,Sha256};
fn fixture()->&'static[u8] {
    let b=include_bytes!("fixtures/native-water-clock-original.bin");
    assert_eq!(format!("{:X}",Sha256::digest(b)),"254A240B7E5E0EECB1399967DF50F5C3EBBDD26D953E9EE56A46259D48D51A93");
    assert_eq!(&b[..8],b"LWCT0001");b
}
fn u32_at(b:&[u8],at:usize)->u32 {u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn footer()->serde_json::Value {
    let b=fixture();let at=16+8656*11+9668*12;let len=u32_at(b,at) as usize;
    assert_eq!(at+4+len,b.len());serde_json::from_slice(&b[at+4..]).unwrap()
}
fn clock<'a>(now:&'a Cell<u32>,step:&'a Cell<u32>,reads:&'a Cell<u32>)->impl FnMut()->u32+'a {
    move||{let result=now.get();now.set(result.wrapping_add(step.get()));reads.set(reads.get()+1);result}
}
#[test]
fn all_8656_original_phase_rows_match_native_counter_wrap_and_fresh_deadline() {
    let b=fixture();assert_eq!(u32_at(b,8),8656);assert_eq!(u32_at(b,12),9668);
    for v in b[16..16+8656*11].chunks_exact(11) {
        let mut core=Core::with_values(v[2] as i8,v[1]).unwrap();core.enabled=v[0]!=0;
        let now=Cell::new(12345);let step=Cell::new(0);let reads=Cell::new(0);
        core.water_callback(clock(&now,&step,&reads)).unwrap();
        assert_eq!((core.counter(),core.phase(),core.events().len(),reads.get()),(v[3],v[4] as i8,usize::from(v[5]),u32::from(v[10])));
        assert_eq!(core.events().first().map_or(0,|e|e.due),u32_at(v,6));
        if core.enabled {assert_eq!((core.events()[0].callback,core.events()[0].owner),(Callback::Water,0));}
    }
}
#[test]
fn all_9668_original_scroll_rows_match_even_while_water_is_disabled() {
    let b=fixture();let start=16+8656*11;
    for v in b[start..start+9668*12].chunks_exact(12) {
        let h=|at|u16::from_le_bytes(v[at..at+2].try_into().unwrap());
        let delta=[i32::from(h(4) as i16),i32::from(h(6) as i16)];
        assert_eq!(water::scroll([h(0),h(2)],delta).unwrap(),[h(8),h(10)]);
    }
    assert!(water::scroll([640,0],[0,0]).is_err());assert!(water::scroll([0,0],[-641,0]).is_err());
}
#[derive(Default)]
struct Harness {core:Core,xy:[u16;2],now:Cell<u32>,step:Cell<u32>,reads:Cell<u32>,notes:Vec<(u32,u32)>}
impl Harness {
    fn schedule(&mut self,delay:u32,callback:Callback,owner:u32) {
        self.core.schedule(delay,callback,owner,clock(&self.now,&self.step,&self.reads)).unwrap();
    }
    fn tick(&mut self)->bool {
        let fired=self.core.dispatch(clock(&self.now,&self.step,&self.reads)).unwrap();
        for e in &fired {if let Callback::User(tag)=e.callback {self.notes.push((tag,e.owner));}}
        !fired.is_empty()
    }
    fn check(&self,name:&str,fired:bool,expected:&serde_json::Value) {
        let state=expected["states"].as_array().unwrap().iter().find(|v|v["scenario"]==name).unwrap();
        let actual=serde_json::json!({"scenario":name,"phase":self.core.phase().to_string(),"counter":self.core.counter().to_string(),
            "x":self.xy[0].to_string(),"y":self.xy[1].to_string(),"queued":self.core.events().len().to_string(),
            "clock_now":self.now.get().to_string(),"clock_reads":self.reads.get().to_string(),"fired":u8::from(fired).to_string()});
        assert_eq!(&actual,state,"{name}");
        let queues:Vec<_>=expected["queues"].as_array().unwrap().iter().filter(|v|v["scenario"]==name).collect();
        assert_eq!(queues.len(),self.core.events().len(),"{name}");
        for (i,(e,row)) in self.core.events().iter().zip(queues).enumerate() {
            let callback=match e.callback {Callback::Water=>"phase".into(),Callback::Green=>"green".into(),Callback::MapAux=>"map_aux".into(),Callback::Kept(n)=>format!("keep{n}"),Callback::User(n)=>format!("note{n}")};
            assert_eq!(serde_json::json!({"scenario":name,"index":i.to_string(),"deadline":e.due.to_string(),"callback":callback,"owner":e.owner.to_string()}),*row,"{name}");
        }
    }
}
#[test]
fn all_15_original_queue_scenarios_match_stable_order_cancel_map_filter_and_late_dispatch() {
    let expected=footer();assert_eq!(expected["original_archive_sha256"],"A97D38AFB45E85FE4FE13E719DA71E26D76A1791372F2D0C35D0758A627B14F0");
    let mut h=Harness::default();h.now.set(1000);
    for (d,c,o) in [(50,1,101),(50,2,102),(50,1,103),(25,1,104),(75,2,105)] {h.schedule(d,Callback::User(c),o);}
    h.check("tie",false,&expected);
    let mut h=Harness::default();h.now.set(1000);
    for (d,c,o) in [(50,1,11),(50,1,11),(50,1,22),(50,2,11),(99,1,11)] {h.schedule(d,Callback::User(c),o);}
    h.core.cancel(Callback::User(1),11);h.check("cancel",false,&expected);
    let mut h=Harness::default();h.now.set(1000);h.xy=[11,419];h.core=Core::with_values(19,98).unwrap();h.core.enabled=true;
    for i in 0..9 {h.schedule(3000+u32::from(i),Callback::Kept(i),u32::from(i)+1);}
    for d in [50,90] {h.schedule(d,Callback::Water,0);}h.schedule(95,Callback::Water,42);h.schedule(55,Callback::User(1),88);
    h.check("map_before",false,&expected);
    h.core.map_filter(clock(&h.now,&h.step,&h.reads)).unwrap();h.check("map_filtered",false,&expected);
    h.core.ensure_map_aux(clock(&h.now,&h.step,&h.reads)).unwrap();h.check("map_aux_repeat",false,&expected);
    h.now.set(2000);h.schedule(50,Callback::Water,0);h.check("map_rescheduled",false,&expected);
    h.now.set(2050);let fired=h.tick();h.check("map_tick",fired,&expected);
    let mut h=Harness::default();h.now.set(1000);h.core=Core::with_values(7,99).unwrap();h.core.enabled=true;
    h.schedule(50,Callback::Water,0);h.schedule(50,Callback::Water,0);h.core.enabled=false;h.core.cancel(Callback::Water,0);
    h.check("disabled_cancel",false,&expected);h.core.water_callback(clock(&h.now,&h.step,&h.reads)).unwrap();h.check("disabled_callback",false,&expected);
    let mut h=Harness::default();h.core.enabled=true;h.schedule(50,Callback::Water,0);h.schedule(50,Callback::Water,0);
    h.now.set(1000);h.step.set(7);let fired=h.tick();h.check("late",fired,&expected);
    h.step.set(0);h.now.set(1106);let fired=h.tick();h.check("late_not_due",fired,&expected);
    h.now.set(1107);let fired=h.tick();h.check("late_first",fired,&expected);
    h.now.set(2000);let fired=h.tick();h.check("late_again",fired,&expected);
    let mut h=Harness::default();h.now.set(1000);h.schedule(50,Callback::User(1),1);h.schedule(50,Callback::User(2),2);h.schedule(25,Callback::User(1),3);
    h.now.set(1050);let fired=h.tick();h.check("dispatch_equal",fired,&expected);assert_eq!(h.notes,[(1,3),(1,1),(2,2)]);
    let mut h=Harness::default();h.now.set(0xfffffff0);h.schedule(50,Callback::User(1),9);h.schedule(0,Callback::User(2),10);h.check("u32_wrap",false,&expected);
}
#[test]
fn game_retirement_cancels_old_callbacks_preserves_visual_values_and_rejects_old_epochs() {
    let mut owned=Owned::default();owned.begin(2).unwrap();owned.packet(2,true,||0).unwrap();
    owned.tick(2,||50).unwrap();assert_eq!(owned.core.phase(),1);
    owned.loaded(2,7,||60).unwrap();assert_eq!(owned.core.events().iter().filter(|e|e.callback==Callback::Water).count(),1);
    assert!(!owned.loaded(2,7,||65).unwrap());
    owned.packet(2,false,||70).unwrap();assert_eq!(owned.core.phase(),1);owned.retire();assert!(owned.core.events().is_empty());
    let before=owned.clone();assert!(owned.tick(2,||1000).is_err());assert_eq!(owned,before);assert!(owned.begin(2).is_err());
    owned.begin(4).unwrap();assert_eq!(owned.core.phase(),1);assert!(owned.core.events().is_empty());assert!(!owned.core.enabled);
    owned.packet(4,true,||0).unwrap();owned.tick(4,||50).unwrap();assert_eq!(owned.core.phase(),2);
}
fn motion(revision:u64,source:[u16;2],offset:[i32;2])->Motion {
    Motion{identity:[7,1,revision],source,destination:[source[0]+1,source[1]],offset,extent:[24,-12]}
}
#[test]
fn shared_camera_history_accumulates_dry_motion_once_and_ignores_map_or_correction_jumps() {
    let mut h=History::default();h.bind(2).unwrap();let from=[32760,32760];
    assert_eq!(h.sample(2,1,Some(motion(1,from,[6,-3]))).unwrap(),[634,3]);
    assert_eq!(h.sample(2,1,Some(motion(1,from,[6,-3]))).unwrap(),[634,3]);
    // A skipped terminal sample is committed only for a contiguous next step.
    assert_eq!(h.sample(2,1,Some(motion(2,[32761,32760],[6,-3]))).unwrap(),[610,15]);
    assert_eq!(h.sample(2,2,None).unwrap(),[610,15]);
    assert_eq!(h.sample(2,2,Some(motion(3,[32800,32800],[0,0]))).unwrap(),[610,15]);
    assert_eq!(h.sample(2,2,Some(motion(3,[32800,32800],[24,-12]))).unwrap(),[586,27]);
    let before=h.clone();assert!(h.sample(2,2,Some(motion(3,[32800,32800],[6,-3]))).is_err());assert_eq!(h,before);
    h.retire();let values=h.values();assert!(h.sample(2,3,None).is_err());h.bind(4).unwrap();assert_eq!(h.values(),values);
}
#[test]
fn invalid_domains_and_wrap_storm_fail_without_partially_committing_state() {
    assert!(Core::with_values(-21,0).is_err());assert!(Core::with_values(0,100).is_err());
    let mut c=Core::default();c.enabled=true;c.schedule(50,Callback::Water,0,||0xfffffff0).unwrap();
    let before=c.clone();assert!(c.dispatch(||0xfffffff0).is_err());assert_eq!(c,before);
    let mut h=History::default();h.bind(2).unwrap();let before=h.clone();let mut m=motion(1,[1,1],[0,0]);m.extent=[i32::MIN,0];
    assert!(h.sample(2,1,Some(m)).is_err());assert_eq!(h,before);
}

#[test]
fn explicit_scene_retirement_uses_first_seven_keep_entries_and_cancels_water_and_aux() {
    let mut core=Core::with_values(7,99).unwrap();core.enabled=true;
    for i in 0..9 {core.schedule(100,Callback::Kept(i),0,||0).unwrap();}
    core.schedule(50,Callback::Water,0,||0).unwrap();core.ensure_map_aux(||0).unwrap();
    core.retire_scene();assert!(!core.enabled);assert_eq!((core.phase(),core.counter()),(7,99));
    assert_eq!(core.events().len(),7);assert!(core.events().iter().all(|e|matches!(e.callback,Callback::Kept(i) if i<7)));
}
