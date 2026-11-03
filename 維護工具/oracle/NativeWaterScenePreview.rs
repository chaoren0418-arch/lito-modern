//! Diagnostic only: use the actual compiled product library and original assets.
use lito_modern::modern_lito::{native_environment::State,native_render_environment::Bound,
    native_water_clock::Callback,native_world::World,native_world_scene::{Frame,Model}};
use std::{path::Path,sync::atomic::AtomicBool};
fn retain(directory:&Path,name:&str,frame:&Frame) {
    std::fs::write(directory.join(format!("{name}.rgba")),frame.canvas.rgba()).unwrap();
    std::fs::write(directory.join(format!("{name}.json")),serde_json::to_vec_pretty(&frame.receipt).unwrap()).unwrap();
}
fn main() {
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),4);
    let root=Path::new(&args[1]);let output=Path::new(&args[2]);
    let fixture:serde_json::Value=serde_json::from_slice(&std::fs::read(&args[3]).unwrap()).unwrap();
    let mut world=World::default();world.relations.begin(2).unwrap();
    for key in ["world","start","status","actor","bot","hp","mp"] {
        let s=fixture["packets"][key].as_str().unwrap();
        let bytes:Vec<_>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        assert!(world.accept(&bytes).unwrap());
    }
    let mut environment=State::default();environment.begin(2).unwrap();environment.map_changed(2,world.map_id.unwrap(),Some(0)).unwrap();
    let cancel=AtomicBool::new(false);let mut model=Model::load(root,&world,&cancel).unwrap();
    world.predict_step(2).unwrap();
    model.draw_at_labels_environment(&world,&cancel,0,None,Some(&Bound::capture(2,&world,&environment).unwrap())).unwrap();
    let dry=model.draw_at_labels_environment(&world,&cancel,5_000_000_000,None,Some(&Bound::capture(2,&world,&environment).unwrap())).unwrap();retain(output,"dry",&dry);
    environment.map_changed(2,world.map_id.unwrap(),Some(1)).unwrap();
    let wet=model.draw_at_labels_environment(&world,&cancel,5_000_000_000,None,Some(&Bound::capture(2,&world,&environment).unwrap())).unwrap();retain(output,"water-phase0",&wet);
    let due=environment.water_clock.core.events().iter().find(|e|e.callback==Callback::Water).unwrap().due;
    environment.water_clock.tick(2,||due).unwrap();
    let wave=model.draw_at_labels_environment(&world,&cancel,5_000_000_000,None,Some(&Bound::capture(2,&world,&environment).unwrap())).unwrap();retain(output,"water-phase1",&wave);
    assert_ne!(dry.canvas.rgba(),wet.canvas.rgba());assert_ne!(wet.canvas.rgba(),wave.canvas.rgba());
    assert_eq!(wave.receipt["world_lighting"]["water_extra"]["scroll"],serde_json::json!([616,12]));
    println!("{}",serde_json::json!({"status":"compiled_product_library_real_resource_water_preview","width":800,"height":600,"phase_samples":[0,1],"scroll":[616,12],"normal_main_GUI":false,"game_sessions":0}));
}
