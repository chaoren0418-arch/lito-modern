use std::{fs,path::{Path,PathBuf},sync::atomic::AtomicBool};
use lito_modern::modern_lito::{native_multisector_resources::Inputs,native_resources::{BytePin,Limits,city::{CityAssets,LoadTrace}},scene::CameraRect};
use serde_json::{Value,json};use sha2::{Digest,Sha256};
fn cam(v:&Value)->CameraRect{CameraRect{x:v["camera"][0].as_i64().unwrap()as i32,y:v["camera"][1].as_i64().unwrap()as i32,width:800,height:600}}
fn main()->anyhow::Result<()>{
 let p=PathBuf::from(std::env::args_os().nth(1).expect("project"));let manifest=p.join("logs/continuation-20261003/r486-map-binding.json");let b=fs::read(&manifest)?;let i=Inputs{project:p.clone(),manifest,manifest_pin:BytePin{bytes:b.len()as u64,sha256:Sha256::digest(&b).into()}};
 let r:Value=serde_json::from_slice(&fs::read(p.join("target/r486-paged-city/reference/reference.json"))?)?;let views=r["results"][0]["images"].as_array().unwrap();let c=AtomicBool::new(false);let trace=LoadTrace::default();
 let a=CityAssets::load_catalog_bounded(&i,3,563,Limits::default(),8<<20,&c,&trace)?;assert_eq!(a.metrics().qualified_slots,31444);let initial=a.render(cam(&views[0]),&c)?;assert_eq!(initial.rgba().len(),1920000);drop(initial);
 let before=a.cache_metrics().unwrap();let largest=views.iter().find(|v|v["name"]=="grid-2-1").unwrap();let error=format!("{:#}",a.render(cam(largest),&c).err().expect("whole view must refuse"));assert_eq!(error,"city cache visible pixel budget");let after=a.cache_metrics().unwrap();
 assert_eq!((before.resident_decoded_bytes,before.misses,before.evictions,before.completed_frames),(after.resident_decoded_bytes,after.misses,after.evictions,after.completed_frames));
 let d=json!({"state":"R486_FOCUSED_8MIB_ERROR_METADATA_AUDIT_PASS","capacity_bytes":8388608,"required_bytes":9469440,"qualified_slots":31444,"validated_slots":trace.validated_slots.load(std::sync::atomic::Ordering::Acquire),"camera":largest["camera"],"error":error,"before":before,"after":after,"no_canvas_returned":true,"residence_misses_evictions_completed_frames_unchanged":true,"executions":1,"historical_PASS_consumers_rerun":0,"GUI":0,"LOADED":false});
 let out=p.join("logs/continuation-20261003/r486-tight-budget-audit.json");fs::write(&out,serde_json::to_vec_pretty(&d)?)?;println!("R486 focused8MiB negative PASS:{error};31,444slots prequalified;whole frame refused;prior cache unchanged");Ok(())
}
