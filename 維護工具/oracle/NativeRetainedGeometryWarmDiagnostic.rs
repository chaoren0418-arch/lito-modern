//! Synthetic-authority Model diagnostic. Real load timing, no GUI acceptance.
use lito_modern::modern_lito::{native_world::World,native_world_scene::Model};
use serde_json::{json,Value};
use std::{path::PathBuf,sync::atomic::AtomicBool,time::Instant};
fn place(w:&mut World,p:[u16;2]){let id=w.own_actor().unwrap().id;let a=w.actors.get_mut(&id).unwrap();a.x=p[0];a.y=p[1];}
fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
    let root=PathBuf::from(&args[1]);let out=PathBuf::from(&args[2]);
    let fixture:Value=serde_json::from_slice(&std::fs::read(out.join("world-fixture.json"))?)?;
    let mut w=World::default();for key in ["world","start","status","actor","hp","mp"] {
        let s=fixture["packets"][key].as_str().unwrap();let b:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b)?;
    }
    w.map_id=Some(563);place(&mut w,[32813,32874]);let stop=AtomicBool::new(false);
    let cold_start=Instant::now();let mut model=Model::load(&root,&w,&stop)?;let first=model.draw_at(&w,&stop,0)?;
    let cold_us=cold_start.elapsed().as_micros() as u64;
    let nav=model.navigation();let y=(32832..32960).find(|y|nav.can_step(32767,*y,2)).unwrap() as u16;
    let positions=[[32767,y],[32768,y]];let geometry=first.receipt["geometry_sha256"].clone();drop(first);
    let mut records=Vec::new();let mut measurements=Vec::new();
    let mut old_surface:Option<lito_modern::modern_lito::native_world_scene::InputSurface>=None;let mut previous_reload=None;
    for index in 0..50usize {
        let p=positions[index%2];place(&mut w,p);
        if let Some(surface)=&old_surface {assert!(!surface.input_ready(&w));}
        let before=serde_json::to_value(&w)?;let start=Instant::now();
        let frame=model.draw_at(&w,&stop,(index as u64+1)*2_000_000_000)?;let wall_us=start.elapsed().as_micros() as u64;
        assert_eq!(serde_json::to_value(&w)?,before);assert_eq!(frame.receipt["geometry_sha256"],geometry);
        assert_eq!(frame.receipt["load_profile_us"]["archive_sources_reused"],true);
        assert_eq!(frame.receipt["load_profile_us"]["geometry_sources_reused"],true);
        assert!(std::sync::Arc::ptr_eq(&nav,&model.navigation()));
        let reload=frame.receipt["terrain_reload_count"].as_u64().unwrap();
        if let Some(old)=previous_reload {assert_eq!(reload,old+1);}previous_reload=Some(reload);
        let decoded=frame.receipt["static_resource_budget"]["decoded_bytes"].as_u64().unwrap();assert!(decoded<=64<<20);
        let surface=frame.input_surface(&w).unwrap();assert!(surface.input_ready(&w));old_surface=Some(surface);
        let phase=if index<10 {"warmup"}else{"measured"};if index>=10 {measurements.push(wall_us);}
        records.push(json!({"index":index,"phase":phase,"position":p,"wall_us":wall_us,
            "load_profile_us":frame.receipt["load_profile_us"],"render_profile_us":frame.receipt["render_profile_us"],
            "native_terrain_center":frame.receipt["native_terrain_center"],"terrain_reload_count":reload,"decoded_bytes":decoded}));
    }
    measurements.sort_unstable();assert_eq!(measurements.len(),40);
    let median=(measurements[19]+measurements[20])/2;let p95=measurements[37];let maximum=measurements[39];
    let result=json!({"status":"quiet_Release_Model_warm_residence_diagnostic_complete",
        "map":563,"synthetic_authority_fixture":true,"actual_formal_sessions":0,"actors":w.actors.len(),
        "cold_Model_load_plus_first_draw_us":cold_us,"warmup_reloads":10,"warmup_pairs":5,"measured_reloads":40,
        "wall_us":{"median":median,"p95_nearest_rank":p95,"maximum":maximum,"sorted_samples":measurements},"records":records,
        "synthetic_sampling_clock_is_not_measured_clock":true,"timing_source":"std::time::Instant around actual draw_at call",
        "correct_authority_and_old_input_retirement_checked":true,"GUI_verified":false,"thirty_player_gameplay_measured":false,
        "performance_accepted":false,"whole_goal_complete":false});
    std::fs::write(out.join("diagnostic-results.json"),serde_json::to_vec_pretty(&result)?)?;
    println!("40 warm residence reloads: median={median}us p95={p95}us max={maximum}us; synthetic Model diagnostic only");Ok(())
}
