//! Standalone player-facing offline CPU map menu, linked to the fixed R474 library.
#[path="native-resource-adapter.rs"] mod adapter;
use lito_modern::modern_lito::{native_offline_map_flow::{Catalog,BoundMap,OfflineMapFlow,PreviewFrame},native_resources as r};
use serde_json::{Value,json};
use std::{fs,path::{Path,PathBuf,Component},io::{self,Write},sync::atomic::AtomicBool};
fn bp(v:&Value)->r::BytePin {let h=v["sha256"].as_str().unwrap();let mut sha256=[0;32];for(i,b)in sha256.iter_mut().enumerate(){*b=u8::from_str_radix(&h[2*i..2*i+2],16).unwrap();}r::BytePin{bytes:v["bytes"].as_u64().unwrap(),sha256}}
fn read(root:&Path,p:&Value)->Vec<u8>{let mut p=p.clone();p["namespace"]=json!("MODERN_OWNED_OFFLINE_INPUT");adapter::read_pinned(root,&p,"MODERN_OWNED_OFFLINE_INPUT",256<<20).unwrap()}
fn frame(out:&Path,f:&PreviewFrame,n:u32)->PathBuf {
 let rgba=f.rgba().unwrap();let(w,h)=(f.camera.width,f.camera.height);let stride=(w*3+3)&!3;let size=54+stride*h;let mut bmp=vec![0;size as usize];bmp[..2].copy_from_slice(b"BM");bmp[2..6].copy_from_slice(&size.to_le_bytes());bmp[10..14].copy_from_slice(&54u32.to_le_bytes());bmp[14..18].copy_from_slice(&40u32.to_le_bytes());bmp[18..22].copy_from_slice(&(w as i32).to_le_bytes());bmp[22..26].copy_from_slice(&(h as i32).to_le_bytes());bmp[26..28].copy_from_slice(&1u16.to_le_bytes());bmp[28..30].copy_from_slice(&24u16.to_le_bytes());
 for y in 0..h as usize{for x in 0..w as usize{let src=(y*w as usize+x)*4;let dst=54+(h as usize-1-y)*stride as usize+x*3;bmp[dst..dst+3].copy_from_slice(&[rgba[src+2],rgba[src+1],rgba[src]]);}}
 let path=out.join(format!("view-{n:04}.bmp"));fs::OpenOptions::new().write(true).create_new(true).open(&path).unwrap().write_all(&bmp).unwrap();path
}
fn main() {
 let manifest=PathBuf::from(std::env::args().nth(1).expect("fixed binding JSON"));let out=PathBuf::from(std::env::args().nth(2).expect("new own output directory"));
 let project=Path::new(env!("R474_MODERN_ROOT"));assert!(manifest.is_absolute()&&manifest.starts_with(project.join("logs"))&&out.is_absolute()&&out.starts_with(project.join("target"))&&!out.exists());
 for path in [&manifest,&out] {assert!(!path.components().any(|c|matches!(c,Component::ParentDir|Component::CurDir)));for ancestor in path.ancestors().filter(|p|p.exists()){let md=fs::symlink_metadata(ancestor).unwrap();assert!(!md.file_type().is_symlink());#[cfg(windows)]{use std::os::windows::fs::MetadataExt;assert_eq!(md.file_attributes()&0x400,0);}}}
 fs::create_dir(&out).unwrap();
 let m:Value=serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();let index=read(project,&m["LTtext_index"]);let pak=read(project,&m["LTtext_pak"]);let c=Catalog::from_lttext(true,&index,&pak,bp(&m["LTtext_index"]),bp(&m["LTtext_pak"]),bp(&m["table_decoded"])).unwrap();
 let v:Value=serde_json::from_slice(&read(project,&m["selected_provider"])).unwrap();let a=v["archive_pins"].as_array().unwrap();let map=Path::new(v["native_map"]["path"].as_str().unwrap());let mut digest=v["native_map"].clone();digest["sha256"]=v["native_map"]["canonical_sha256"].clone();
 let b=BoundMap{mode:1,sources:r::NativeSources{map_root:map.parent().unwrap().parent().unwrap().into(),archive_root:Path::new(a[0]["path"].as_str().unwrap()).parent().unwrap().into()},plan:r::NativeAssetPlan{namespace:r::ResourceNamespace::NativeLitoV1,map_id:564,canonical_sha256:bp(&digest).sha256,sectors:vec![r::SectorPin{key:r::NativeSectorKey{map_id:564,x:0x7fff,y:0x8000},pin:bp(&v["native_map"])}],archives:a.iter().zip([r::ArchiveRole::TileIndex,r::ArchiveRole::TilePak,r::ArchiveRole::SpriteIndex,r::ArchiveRole::SpritePak]).map(|(v,role)|r::ArchivePin{role,pin:bp(v)}).collect()}};
 let mut flow=OfflineMapFlow::new(true,c,vec![b]).unwrap();let cancel=AtomicBool::new(false);let mut number=0;let mut history=Vec::new();
 println!("LiTo Modern / Offline map preview\nOriginal Native art, CPU view. Commands: list | select MODE MAP | enter | pan DX DY | back | quit");
 loop {
  print!("{:?}> ",flow.page());io::stdout().flush().unwrap();let mut line=String::new();if io::stdin().read_line(&mut line).unwrap()==0{break;}let parts:Vec<_>=line.split_whitespace().collect();if parts.is_empty(){continue;}
  if parts==["quit"] {if flow.page()!=lito_modern::modern_lito::native_offline_map_flow::Page::Menu{flow.back().unwrap();}history.push(json!({"command":"quit","page":"Menu"}));break;}
  let result:Result<Option<PreviewFrame>,String>=(||match parts.as_slice(){
   ["list"]=>{for c in flow.choices(){println!("{} {:3} {:24} {}",c.mode,c.map_id,c.name,if flow.available(c.mode,c.map_id){"[preview available]"}else{"[not bound]"});}Ok(None)},
   ["select",mode,map]=>{flow.select(mode.parse().map_err(|_|"invalid mode")?,map.parse().map_err(|_|"invalid map")?).map_err(|e|e.to_string())?;println!("Selected {}",flow.selected().unwrap().name);Ok(None)},
   ["enter"]=>{let job=flow.begin_entry().map_err(|e|e.to_string())?;match job.load(&cancel){Ok(ready)=>flow.complete_entry(ready).map(Some).map_err(|e|e.to_string()),Err(e)=>{flow.back().unwrap();Err(e.to_string())}}},
   ["pan",x,y]=>flow.pan(x.parse().map_err(|_|"invalid DX")?,y.parse().map_err(|_|"invalid DY")?,&cancel).map(Some).map_err(|e|e.to_string()),
   ["back"]=>flow.back().map(|_|None).map_err(|e|e.to_string()),_=>Err("unknown command".to_owned()),
  })();
  match result {Ok(view)=>{let image=view.map(|v|{number+=1;let path=frame(&out,&v,number);println!("CPU image: {}",path.display());path});history.push(json!({"command":line.trim(),"page":format!("{:?}",flow.page()),"image":image}));},Err(e)=>{println!("Cannot continue: {e}");history.push(json!({"command":line.trim(),"page":format!("{:?}",flow.page()),"error":e}));}}
 }
 fs::write(out.join("session.json"),serde_json::to_vec_pretty(&json!({"history":history,"offline_only":true,"renderer_LOADED":false,"GUI":0,"native_callbacks":0})).unwrap()).unwrap();
}
