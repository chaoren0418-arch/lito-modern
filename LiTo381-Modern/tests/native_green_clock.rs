use lito_modern::modern_lito::native_actor_states::{State, Style};
use lito_modern::modern_lito::native_water_clock::{Callback,Core,Owned};
use std::cell::Cell;
use sha2::{Digest,Sha256};

fn original()->serde_json::Value {
    let bytes=include_bytes!("fixtures/native-green-clock-original.json");
    assert_eq!(format!("{:X}",Sha256::digest(bytes)),"9179218BDE8536E5C6ED97FB359E55326165F124F353E604785BE01C85ED1D90");
    serde_json::from_slice(bytes).unwrap()
}
fn number(v:&serde_json::Value,name:&str)->u32 {v[name].as_str().unwrap().parse().unwrap()}
fn clock<'a>(now:&'a Cell<u32>,step:&'a Cell<u32>,reads:&'a Cell<u32>)->impl FnMut()->u32+'a {
    move||{let value=now.get();now.set(value.wrapping_add(step.get()));reads.set(reads.get()+1);value}
}
#[derive(Default)]
struct Harness {core:Core,now:Cell<u32>,step:Cell<u32>,reads:Cell<u32>}
impl Harness {
    fn start(&mut self) {self.core.green_callback(clock(&self.now,&self.step,&self.reads)).unwrap();}
    fn schedule(&mut self,delay:u32,callback:Callback,owner:u32) {
        self.core.schedule(delay,callback,owner,clock(&self.now,&self.step,&self.reads)).unwrap();
    }
    fn tick(&mut self)->bool {!self.core.dispatch(clock(&self.now,&self.step,&self.reads)).unwrap().is_empty()}
    fn check(&self,name:&str,fired:bool,fixture:&serde_json::Value) {
        let expected=fixture["states"].as_array().unwrap().iter().find(|v|v["scenario"]==name).unwrap();
        assert_eq!(self.core.green_counter(),number(expected,"counter"),"{name}");
        assert_eq!(State::from_status(1).style(false,self.core.green_counter()),Style::Green(number(expected,"step") as u8),"{name}");
        assert_eq!((self.core.events().len(),self.now.get(),self.reads.get(),u32::from(fired)),
            (number(expected,"queued") as usize,number(expected,"clock_now"),number(expected,"clock_reads"),number(expected,"fired")),"{name}");
        let rows:Vec<_>=fixture["queues"].as_array().unwrap().iter().filter(|v|v["scenario"]==name).collect();
        assert_eq!(rows.len(),self.core.events().len(),"{name}");
        for (i,(event,row)) in self.core.events().iter().zip(rows).enumerate() {
            let kind=match event.callback {Callback::Green=>"green".into(),Callback::Water=>"phase".into(),Callback::MapAux=>"map_aux".into(),
                Callback::Kept(i)=>format!("keep{i}"),Callback::User(i)=>format!("note{i}")};
            assert_eq!((event.due,event.owner,kind.as_str(),i as u32),
                (number(row,"deadline"),number(row,"owner"),row["callback"].as_str().unwrap(),number(row,"index")),"{name}");
        }
    }
}

#[test]
fn actor_green_style_consumes_delivered_native_counter_not_elapsed_nanoseconds() {
    // Original4878F0 + first-pass RGB555 selector: independently executed532
    // counter vectors and16 queue snapshots. Values are callback counts.
    let state = State::from_status(9);
    for (counter, boost) in [(0,14),(1,13),(2,13),(12,8),(20,12),(23,13),(24,14),(0xffff_ffff,9)] {
        assert_eq!(state.style(false,counter),Style::Green(boost),"native counter {counter}");
    }
}

#[test]
fn all_532_original_counter_wrap_fresh_deadline_and_shader_steps_match() {
    let f=original();let rows=f["scalars"].as_array().unwrap();assert_eq!(rows.len(),532);
    assert_eq!(f["original_archive_sha256"],"86107BA797D72D7CD380B688A5DFCE10ACE1E839372827705F948B0E0FAF46AE");
    for row in rows {
        let mut h=Harness::default();h.core=Core::with_green_counter(number(row,"initial"));h.now.set(number(row,"schedule_clock"));h.start();
        assert_eq!(h.core.green_counter(),number(row,"counter_out"));assert_eq!(h.reads.get(),number(row,"clock_reads"));
        assert_eq!(h.core.events().len(),1);let e=h.core.events()[0];
        assert_eq!((e.due,e.callback,e.owner),(number(row,"deadline"),Callback::Green,0));
        assert_eq!(State::from_status(1).style(false,h.core.green_counter()),Style::Green(number(row,"step") as u8));
    }
}

#[test]
fn all_16_original_queue_snapshots_match_late_duplicate_cancel_map_and_scene_retirement() {
    let f=original();assert_eq!(f["states"].as_array().unwrap().len(),16);assert_eq!(f["queues"].as_array().unwrap().len(),72);
    let mut h=Harness::default();h.now.set(1000);h.start();h.check("entry",false,&f);
    h.now.set(1049);let fired=h.tick();h.check("early",fired,&f);
    h.now.set(1050);let fired=h.tick();h.check("due",fired,&f);
    let mut h=Harness::default();h.start();h.now.set(1000);h.step.set(7);let fired=h.tick();h.check("late",fired,&f);
    h.step.set(0);h.now.set(1056);let fired=h.tick();h.check("late_not_due",fired,&f);
    h.now.set(1057);let fired=h.tick();h.check("late_due",fired,&f);
    let mut h=Harness::default();h.start();h.start();h.now.set(1000);h.step.set(7);let fired=h.tick();h.check("duplicate",fired,&f);
    let mut h=Harness::default();h.core=Core::with_green_counter(41);h.start();h.schedule(50,Callback::Green,77);
    h.core.cancel(Callback::Green,0);h.check("cancel_owned",false,&f);
    h.now.set(50);let fired=h.tick();h.check("cancel_other_owner_dispatched",fired,&f);
    let mut h=Harness::default();h.now.set(1000);h.core=Core::with_green_counter(41);h.start();
    for i in 0..9 {h.schedule(3000+u32::from(i),Callback::Kept(i),u32::from(i)+1);}
    h.schedule(50,Callback::Green,77);h.schedule(70,Callback::User(1),88);h.check("map_before_filter",false,&f);
    h.core.map_filter(clock(&h.now,&h.step,&h.reads)).unwrap();h.check("map_after_filter",false,&f);
    h.start();h.check("explicit_restart_after_filter",false,&f);
    let mut h=Harness::default();h.now.set(1000);h.core=Core::with_green_counter(77);h.start();
    for i in 0..9 {h.schedule(3000+u32::from(i),Callback::Kept(i),u32::from(i)+1);}
    h.schedule(50,Callback::Green,77);h.schedule(70,Callback::User(1),88);h.schedule(60,Callback::MapAux,u32::MAX);
    h.check("scene_before_cleanup",false,&f);h.core.retire_scene();h.check("scene_after_cleanup",false,&f);
    h.start();h.check("explicit_restart_after_cleanup",false,&f);
    let mut h=Harness::default();h.core=Core::with_green_counter(u32::MAX);h.start();h.core.cancel(Callback::Green,0);h.start();h.check("cancel_then_restart",false,&f);
}

#[test]
fn owner_and_scene_fences_preserve_counter_but_retire_old_callbacks() {
    let mut owned=Owned::default();owned.begin(2).unwrap();owned.loaded(2,7,||0).unwrap();
    assert_eq!(owned.green_for(2,7).unwrap(),1);
    assert!(!owned.loaded(2,7,||1000).unwrap());assert_eq!(owned.green_for(2,7).unwrap(),1);
    owned.loaded(2,8,||1000).unwrap();assert_eq!(owned.green_for(2,8).unwrap(),2);
    assert!(owned.green_for(2,7).is_err());owned.tick(2,||1000).unwrap();assert_eq!(owned.core.green_counter(),2);
    owned.retire_scene();assert!(owned.green_for(2,8).is_err());assert!(owned.core.events().is_empty());
    assert_eq!(owned.core.green_counter(),2);owned.retire();let before=owned.clone();
    assert!(owned.tick(2,||2000).is_err());assert!(owned.loaded(2,9,||0).is_err());assert_eq!(owned,before);
    owned.begin(4).unwrap();assert_eq!(owned.core.green_counter(),2);owned.loaded(4,1,||0).unwrap();
    assert_eq!(owned.green_for(4,1).unwrap(),3);assert!(owned.green_for(2,1).is_err());
}

#[test]
fn older_or_retired_generation_cannot_restart_callbacks_or_consume_a_schedule_clock() {
    let mut owned=Owned::default();owned.begin(9).unwrap();owned.loaded(9,4,||0).unwrap();
    let before=owned.clone();let reads=Cell::new(0);
    assert!(owned.loaded(9,3,||{reads.set(reads.get()+1);1000}).is_err(),"older generation restarted callbacks");
    assert_eq!(reads.get(),0);assert_eq!(owned,before);
    owned.retire_scene();let before=owned.clone();
    assert!(owned.loaded(9,4,||{reads.set(reads.get()+1);1000}).is_err(),"retired generation restarted callbacks");
    assert_eq!(reads.get(),0);assert_eq!(owned,before);
    owned.loaded(9,5,||1000).unwrap();assert_eq!(owned.green_for(9,5).unwrap(),2);
}

#[test]
fn water_and_green_share_stable_dispatch_order_and_distinct_fresh_deadlines() {
    let mut h=Harness::default();h.core.water_packet(true,clock(&h.now,&h.step,&h.reads)).unwrap();h.start();
    h.now.set(50);h.step.set(7);let fired=h.core.dispatch(clock(&h.now,&h.step,&h.reads)).unwrap();
    assert_eq!(fired.iter().map(|e|e.callback).collect::<Vec<_>>(),[Callback::Water,Callback::Green]);
    assert_eq!((h.core.counter(),h.core.green_counter()),(1,2));
    assert_eq!(h.core.events().iter().map(|e|(e.callback,e.due)).collect::<Vec<_>>(),[(Callback::Green,114),(Callback::Water,157)]);
}

#[test]
fn host_budget_refusal_and_unsigned_wrap_storm_do_not_publish_partial_counter() {
    let mut core=Core::with_green_counter(41);
    for i in 0..4096 {core.schedule(500,Callback::User(i),0,||0).unwrap();}
    let before=core.clone();assert!(core.green_callback(||0).is_err());assert_eq!(core,before);
    let mut core=Core::with_green_counter(u32::MAX);core.green_callback(||0xfffffff0).unwrap();let before=core.clone();
    assert!(core.dispatch(||0xfffffff0).is_err());assert_eq!(core,before);
}

#[test]
fn immutable_render_binding_captures_counter_without_turning_phase_into_a_resource_key() {
    use lito_modern::modern_lito::{native_environment,native_render_environment::Bound,native_world::World};
    let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut world=World::default();
    for key in ["world","start","status","actor","bot","hp","mp"] {
        let s=f["packets"][key].as_str().unwrap();let bytes:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();world.accept(&bytes).unwrap();
    }
    let mut state=native_environment::State::default();state.begin(2).unwrap();state.map_changed(2,world.map_id.unwrap(),None).unwrap();
    assert!(Bound::capture(2,&world,&state).unwrap().actor_green_counter(&world).is_err());
    state.water_clock.loaded(2,world.generation,||0).unwrap();let first=Bound::capture(2,&world,&state).unwrap();
    state.water_clock.tick(2,||50).unwrap();let second=Bound::capture(2,&world,&state).unwrap();
    assert_eq!((first.actor_green_counter(&world).unwrap(),second.actor_green_counter(&world).unwrap()),(1,2));
    assert_eq!(first.key(),second.key());
    let mut other=world.clone();other.generation+=1;assert!(first.actor_green_counter(&other).is_err());
    state.retire();assert!(!first.is_current(2,&world,&state));assert!(Bound::capture(2,&world,&state).is_err());
    state.begin(4).unwrap();state.map_changed(4,world.map_id.unwrap(),None).unwrap();state.water_clock.loaded(4,world.generation,||0).unwrap();
    assert!(!first.is_current(4,&world,&state));assert_eq!(Bound::capture(4,&world,&state).unwrap().actor_green_counter(&world).unwrap(),3);
}
