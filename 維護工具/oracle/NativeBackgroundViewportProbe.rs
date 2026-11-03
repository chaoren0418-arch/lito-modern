//! Isolated modern-extent candidate; normal source and native plan remain pinned.
use lito_modern::modern_lito::{native_background::Bank, native_background_geometry as old,
    native_skill_graphics::{layers::DenseSource,Format as CacheFormat}, resources::{SharedArchive,SharedArchiveKind}};
use lito_modern::modern_lito::native_background_pixels::{self as pixels,Format,Image,Target,Blend};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{path::PathBuf,sync::atomic::AtomicBool};
mod native_background_pixels {pub use lito_modern::modern_lito::native_background_pixels::*;}
mod world_depth {pub use lito_modern::modern_lito::world_depth::*;}
#[path="D:/lineage開發/3-lito開發/lito重建版/維護工具/candidates/native-background-viewport/native_background_geometry.rs"]
mod candidate;

fn input(p:&[i64])->candidate::Input {
    candidate::Input {map_id:p[0].try_into().unwrap(),level:p[1].try_into().unwrap(),class:p[2].try_into().unwrap(),
        cell:world_depth::NativeDrawCell {half_x:p[3].try_into().unwrap(),y:p[4].try_into().unwrap()},
        camera:[p[5].try_into().unwrap(),p[6].try_into().unwrap()],offsets:[p[7].try_into().unwrap(),p[8].try_into().unwrap()]}
}
fn original(p:candidate::Input)->old::Input {
    old::Input {map_id:p.map_id,level:p.level,class:p.class,cell:p.cell,camera:p.camera,offsets:p.offsets}
}
fn words_sha(v:&[u16])->String {
    let bytes:Vec<_>=v.iter().flat_map(|v|v.to_le_bytes()).collect();format!("{:X}",Sha256::digest(bytes))
}
fn scalar(p:candidate::Input,x:usize,y:usize,a:&[u16],b:&[u16])->u16 {
    let half=i64::from(p.cell.half_x)/2;
    let px=(i64::from(p.cell.y)+half)*24/6+(48-i64::from(p.camera[0]))*5/6;
    let py=(i64::from(p.cell.y)-half)*12/6+(48-i64::from(p.camera[1]))*5/6;
    let rx=px.rem_euclid(384);let ry=py.rem_euclid(384);
    let column=(x as i64+rx)/384;
    let sx=(x as i64+rx)%384;
    let shift=if (px/384+column)%2==1 {(ry+192)%384}else{ry};
    let sy=(y as i64+shift)%384;
    let mut oy=i64::from(p.offsets[1]);
    if (p.offsets[0]/384)%2!=0 {oy+=192;}
    let bx=384-i64::from(p.offsets[0]%384)+sx;
    let by=384-oy%384+sy;
    let first=a[(sy*384+sx) as usize];let second=b[(by*768+bx) as usize];
    let factor=if p.level>=15||(1..=3).contains(&p.class) {None}
        else {Some(15+u16::from(p.level*9/16)*8/5)};
    let mut output=0;
    for shift in [10,5,0] {
        let aa=(first>>shift)&31;let bb=(second>>shift)&31;
        let mixed=if p.map_id==303 {31-((31-aa)*(31-bb)+30)/31}else{aa*bb/31};
        output|=factor.map_or(mixed,|n|mixed*n/31)<<shift;
    }
    output
}

fn main()->Result<(),Box<dyn std::error::Error>> {
    let args:Vec<_>=std::env::args_os().collect();assert_eq!(args.len(),3);
    let root=PathBuf::from(&args[1]);let out=PathBuf::from(&args[2]);assert!(root.is_absolute()&&out.is_absolute());
    let fixture:Value=serde_json::from_slice(&std::fs::read(out.join("geometry-fixture.json"))?)?;
    let mut legacy_calls=0;let mut partitions=0;
    for row in fixture["cases"].as_array().unwrap() {
        let v:Vec<_>=row["input"].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();let p=input(&v);
        let calls=candidate::plan(p)?;let expected=row["calls"].as_array().unwrap();assert_eq!(calls.len(),expected.len());
        for (call,e) in calls.iter().zip(expected) {
            let q=call.paint;let strength=match q.strength {pixels::Strength::Full=>32,pixels::Strength::Attenuated(n)=>i64::from(n)};
            let values=vec![call.base_id as i64,call.overlay_id as i64,q.source.left as i64,q.source.top as i64,q.source.right as i64,q.source.bottom as i64,q.destination[0] as i64,q.destination[1] as i64,q.offsets[0] as i64,q.offsets[1] as i64,strength,if q.blend==Blend::Screen{0}else{1}];
            assert_eq!(values,e.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect::<Vec<_>>());legacy_calls+=1;
        }
        for dims in [[1,1],[744,552],[800,600],[1024,1024],[383,385]] {
            let calls=candidate::plan_viewport(p,dims)?;assert!(calls.len()<=16);let mut area=0;
            for (i,call) in calls.iter().enumerate() {
                let s=call.paint.source;let d=call.paint.destination;let w=s.right-s.left;let h=s.bottom-s.top;
                assert!(w>0&&h>0&&s.right<=384&&s.bottom<=384&&d[0]+w<=dims[0] as usize&&d[1]+h<=dims[1] as usize);area+=w*h;
                for other in &calls[..i] {
                    let os=other.paint.source;let od=other.paint.destination;
                    assert!(d[0]+w<=od[0]||od[0]+os.right-os.left<=d[0]||d[1]+h<=od[1]||od[1]+os.bottom-os.top<=d[1]);
                }
            }
            assert_eq!(area,dims[0] as usize*dims[1] as usize);partitions+=1;
        }
    }
    assert_eq!(legacy_calls,17472);assert_eq!(partitions,13440);
    let bank=Bank::load_map303(&root,&AtomicBool::new(false))?;
    assert!(bank.sources["members"].as_array().unwrap().iter().all(|v|v["archive"]=="BASE/Sprite"&&v["native_index_miss"]==true));
    let mut archive=SharedArchive::open_for_names(&root,SharedArchiveKind::Sprite,&["800.img","803.img"],&AtomicBool::new(false))?;
    let aa=archive.read_entry("800.img")?;let bb=archive.read_entry("803.img")?;
    let a=DenseSource::decode_raw555(&aa)?.cache(CacheFormat::Rgb555);let b=DenseSource::decode_raw555(&bb)?.cache(CacheFormat::Rgb555);
    let base=Image::new(384,384,a.words(),Format::Rgb555)?;let overlay=Image::new(768,768,b.words(),Format::Rgb555)?;
    let frames:Value=serde_json::from_slice(&std::fs::read(out.join("frame-fixture.json"))?)?;
    let mut checks=Vec::new();let mut sampled=0;
    for (n,row) in frames["cases"].as_array().unwrap().iter().enumerate() {
        let v:Vec<_>=row["input"].as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).collect();let p=input(&v);let gate_kind=row["gate"].as_u64().unwrap();
        for [width,height] in [[744usize,552],[800,600],[1024,640]] {
            let gate:Vec<_>=(0..width*height).map(|i|if gate_kind==1{1}else if gate_kind==7&&(i/width+i%width)%2==1{7}else{0}).collect();
            let mut result=vec![0x1777u16;width*height];let calls=candidate::plan_viewport(p,[width as u32,height as u32])?;
            for call in &calls {pixels::validate_paint(base,overlay,call.paint,&Target{words:&mut result,pitch:width,height,gate:&gate,gate_pitch:width})?;}
            let mut written=0;
            for call in &calls {written+=pixels::paint(base,overlay,call.paint,Target{words:&mut result,pitch:width,height,gate:&gate,gate_pitch:width})?.written_pixels;}
            assert_eq!(written,gate.iter().filter(|&&v|v==0).count());
            for y in 0..height {for x in 0..width {
                let expected=if gate[y*width+x]==0 {scalar(p,x,y,a.words(),b.words())}else{0x1777};
                assert_eq!(result[y*width+x],expected,"case{n} extent{width}x{height} pixel{x},{y}");sampled+=1;
            }}
            if width==744 {
                let mut native=vec![0x1777u16;width*height];bank.paint(original(p),Target{words:&mut native,pitch:width,height,gate:&gate,gate_pitch:width},&AtomicBool::new(false))?;
                assert_eq!(words_sha(&native),row["frame_sha256"].as_str().unwrap());
                for call in candidate::plan(p)? {let s=call.paint.source;let d=call.paint.destination;
                    for y in 0..s.bottom-s.top {for x in 0..s.right-s.left {assert_eq!(native[(d[1]+y)*width+d[0]+x],result[(d[1]+y)*width+d[0]+x]);}}
                }
            }
            checks.push(json!({"case":n,"dimensions":[width,height],"gate":gate_kind,"calls":calls.len(),"written_pixels":written,"sha256":words_sha(&result)}));
        }
    }
    let p=input(&[303,0,0,0,0,48,48,65484,65534]);
    for dims in [[0,1],[1,0],[1025,600],[800,1025]] {assert!(candidate::plan_viewport(p,dims).is_err());}
    let result=json!({"status":"isolated_viewport_candidate_matches_native_overlap_and_scalar_full_extents","native_geometry_cases":2688,"native_ordered_calls":legacy_calls,"partition_cases":partitions,"full_word_comparisons":sampled,"frames":checks,"native_744x552_plan_unchanged":true,"modern_extent_fills_native_522_clip_omissions":true,"native_bank_paint_viewport_method_compiled":false,"normal_source_changed":false,"scene_adapter_or_camera_verified":false,"GUI_verified":false,"actual_formal_sessions":0,"whole_goal_complete":false});
    std::fs::write(out.join("candidate-results.json"),serde_json::to_vec_pretty(&result)?)?;println!("candidate: {partitions} partitions and {sampled} full pixel comparisons passed");Ok(())
}
