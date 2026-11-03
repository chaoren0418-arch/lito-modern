use lito_modern::modern_lito::{native_resources as nr,native_world::World,native_world_scene::Model,resources::SharedMapSectorKey,world_map::{WorldMap,SectorInput},scene::CameraRect};
use std::{path::Path,sync::atomic::AtomicBool};
const ROOT:&str=r"D:\lineage開發\0-客戶端";
fn sources(map_id:u32)->(nr::NativeSources,nr::NativeAssetPlan){
 let root=Path::new(ROOT).join("LiTo");let mut files:Vec<_>=std::fs::read_dir(root.join("LTmap").join(map_id.to_string())).unwrap().map(|e|e.unwrap().path()).filter(|p|p.extension().is_some_and(|e|e=="s32")).collect();files.sort();
 let input=files.iter().map(|p|{let n=p.file_stem().unwrap().to_str().unwrap();SectorInput{key:SharedMapSectorKey{map_id,sector_x:u16::from_str_radix(&n[..4],16).unwrap(),sector_y:u16::from_str_radix(&n[4..],16).unwrap()},bytes:std::fs::read(p).unwrap()}}).collect();
 let stop=AtomicBool::new(false);let map=WorldMap::from_native_sector_bytes(input,&stop).unwrap();let observed=nr::NativeObservedArchives::open(&root,nr::Limits::default(),&stop).unwrap();
 let plan=nr::NativeAssetPlan{namespace:nr::ResourceNamespace::NativeLitoV1,map_id,canonical_sha256:map.canonical_digest(),archives:observed.pins().to_vec(),sectors:map.bindings().iter().map(|b|nr::SectorPin{key:nr::NativeSectorKey{map_id,x:b.key.sector_x,y:b.key.sector_y},pin:nr::BytePin{bytes:b.byte_length.into(),sha256:b.sha256}}).collect()};
 (nr::NativeSources{map_root:root.join("LTmap"),archive_root:root},plan)
}
fn fixture(map:u16,position:[u16;2])->World{
 let f:serde_json::Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();let mut w=World::default();
 for key in ["world","start","status","actor","hp","mp"]{let s=f["packets"][key].as_str().unwrap();let b:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();w.accept(&b).unwrap();}
 w.map_id=Some(map);set_position(&mut w,position);w
}
fn set_position(w:&mut World,p:[u16;2]){let id=w.own_actor().unwrap().id;let a=w.actors.get_mut(&id).unwrap();a.x=p[0];a.y=p[1];}
#[test]
fn resident_native_city_pixels_match_complete_original_terrain(){
 for (map_id,center,samples) in [(500,[32768,32832],[[32767,32831],[32768,32832],[32775,32839],[32776,32840]]),(564,[32720,32792],[[32719,32791],[32720,32792],[32727,32799],[32728,32800]])] {
 let stop=AtomicBool::new(false);let(sources,plan)=sources(map_id);let adapter=nr::NativeResourceAdapter::pinned_world();
 let full=adapter.load_static(&sources,&plan,nr::Limits::default(),&stop).unwrap();
 let observed=nr::NativeObservedArchives::open(&sources.archive_root,nr::Limits::default(),&stop).unwrap();
 let local=adapter.load_static_observed_window(&sources,&plan,nr::Limits::default(),observed,center,&stop).unwrap();
 let retained=nr::NativeRetainedGeometry::open(&sources.map_root,map_id,nr::Limits::default(),&stop).unwrap();
 let mut observed=nr::NativeObservedArchives::open(&sources.archive_root,nr::Limits::default(),&stop).unwrap();
 let cached=adapter.load_static_retained_geometry_window(&sources,&plan,nr::Limits::default(),&mut observed,&retained,center,&stop).unwrap();
 assert!(std::ptr::eq(cached.geometry(),retained.geometry()),"residence assets must borrow the verified parsed geometry");
 assert_eq!(local.geometry().canonical_digest(),full.geometry().canonical_digest());assert_eq!(local.verified_plan(),full.verified_plan());assert!(local.decoded_bytes()<full.decoded_bytes());
 let ox=32704+(i32::from(plan.sectors[0].key.x)-0x7fff)*64;let oy=32704+(i32::from(plan.sectors[0].key.y)-0x7fff)*64;
 for [x,y] in samples{let(dx,dy)=(x-ox,y-oy);let camera=CameraRect{x:24+24*(dx+dy)-400,y:768+12*(-dx+dy)-300,width:800,height:600};let expected=full.render_static(camera,&stop).unwrap();let actual=local.render_static(camera,&stop).unwrap();assert_eq!(actual.rgba(),expected.rgba());assert_eq!(cached.render_static(camera,&stop).unwrap().rgba(),expected.rgba());}
 let observed=nr::NativeObservedArchives::open(&sources.archive_root,nr::Limits::default(),&stop).unwrap();let error=adapter.load_static_observed_window(&sources,&plan,nr::Limits::default(),observed,[0,0],&stop).err().unwrap();assert!(error.to_string().contains("outside source geometry bounds"));
 }
}
#[test]
fn original_siege563_residence_reloads_at_sector_boundary_and_retires_old_input(){
 let stop=AtomicBool::new(false);let mut w=fixture(563,[32813,32874]);let mut model=Model::load(Path::new(ROOT),&w,&stop).unwrap();let nav=model.navigation();
 for p in [[32714,32898],[32705,32907],[32813,32874],[32799,32910],[32729,32943],[32789,32969]]{assert!(nav.has_portal_source(p));}
 let long_routes=[[32813,32874],[32799,32910]].map(|source|nav.route_to_map_entrance_avoiding([32643,32848],source,2,|_,_|false));
 assert!(long_routes.iter().any(|r|r.is_ok()),"recorded native spawn has no original entrance approach");
 for r in long_routes.into_iter().filter_map(Result::ok){assert!(r.len()>128);}
 assert!(nav.route_to_map_entrance_avoiding([32813,32874],[32813,32874],2,|_,_|true).unwrap().is_empty());
 assert!(nav.route_to_map_entrance_avoiding([32643,32848],[1,1],2,|_,_|false).is_err());
 let first=model.draw_at(&w,&stop,0).unwrap();assert_eq!(first.receipt["map_sectors"],19);assert!(first.receipt["terrain_resident_sectors"].as_u64().unwrap()<19);assert!(first.receipt["static_resource_budget"]["decoded_bytes"].as_u64().unwrap()<=64<<20);
 assert_eq!(first.receipt["load_profile_us"]["archive_sources_reused"],false);
 assert_eq!(first.receipt["load_profile_us"]["geometry_sources_reused"],false);
 let y=(32832..32960).find(|y|nav.can_step(32767,*y,2)).unwrap() as u16;set_position(&mut w,[32767,y]);let old=model.draw_at(&w,&stop,1_000_000_000).unwrap();let surface=old.input_surface(&w).unwrap();let generation=w.generation;
 assert!(std::sync::Arc::ptr_eq(&nav,&model.navigation()),"same-map residence reload must retain verified navigation geometry");
 set_position(&mut w,[32768,y]);assert!(!surface.input_ready(&w));let authority=serde_json::to_value(&w).unwrap();let frame=model.draw_at(&w,&stop,2_000_000_000).unwrap();assert_eq!(serde_json::to_value(&w).unwrap(),authority);assert_eq!(w.generation,generation);assert_ne!(old.receipt["native_terrain_center"],frame.receipt["native_terrain_center"]);assert_eq!(old.receipt["geometry_sha256"],frame.receipt["geometry_sha256"]);assert!(frame.receipt["terrain_reload_count"].as_u64().unwrap()>old.receipt["terrain_reload_count"].as_u64().unwrap());assert!(frame.receipt["render_profile_us"]["composition_frames"].as_u64().unwrap()>old.receipt["render_profile_us"]["composition_frames"].as_u64().unwrap());assert!(frame.input_surface(&w).unwrap().input_ready(&w));
 assert!(frame.receipt["terrain_resident_sectors"].as_u64().unwrap()<19);assert!(frame.receipt["static_resource_budget"]["decoded_bytes"].as_u64().unwrap()<=64<<20);
 assert_eq!(frame.receipt["load_profile_us"]["archive_sources_reused"],true);
 assert_eq!(frame.receipt["load_profile_us"]["geometry_sources_reused"],true);
 assert!(std::sync::Arc::ptr_eq(&nav,&model.navigation()));
 let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/verification/native-terrain-residence");std::fs::create_dir_all(&output).unwrap();std::fs::write(output.join("563-static.json"),serde_json::to_vec_pretty(&serde_json::json!({"synthetic_authority_fixture":true,"first":first.receipt,"before_boundary":old.receipt,"after_boundary":frame.receipt,"all_portal_sources":nav.portal_sources().collect::<Vec<_>>()})).unwrap()).unwrap();
 for(name,f) in [("563-before.png",&old),("563-after.png",&frame)]{let mut e=png::Encoder::new(std::fs::File::create(output.join(name)).unwrap(),800,600);e.set_color(png::ColorType::Rgba);e.set_depth(png::BitDepth::Eight);e.write_header().unwrap().write_image_data(f.canvas.rgba()).unwrap();}
 // A terrain swap must keep the model's clock/caches while a real walk clip finishes.
 set_position(&mut w,[32840,32961]);model.draw_at(&w,&stop,4_000_000_000).unwrap();let id=w.own_actor().unwrap().id;
 {let a=w.actors.get_mut(&id).unwrap();a.motion_from=Some([32840,32961]);a.motion_revision+=1;a.x=32839;a.y=32960;}
 let moving=model.draw_at(&w,&stop,5_000_000_000).unwrap();let settled=model.draw_at(&w,&stop,7_000_000_000).unwrap();
 assert_eq!(moving.receipt["terrain_reload_count"],settled.receipt["terrain_reload_count"]);assert!(!moving.movement_ready(&w));assert!(settled.movement_ready(&w));
 {let a=w.actors.get_mut(&id).unwrap();a.motion_from=None;a.motion_revision+=1;}
 let cancelled=AtomicBool::new(true);let previous=model.scene().assets().draw_references();set_position(&mut w,[32832,y]);assert!(model.draw_at(&w,&cancelled,8_000_000_000).is_err());assert_eq!(model.scene().assets().draw_references(),previous);let resumed=model.draw_at(&w,&stop,8_000_000_000).unwrap();assert_eq!(resumed.receipt["native_terrain_center"][0],0x8001);
}
