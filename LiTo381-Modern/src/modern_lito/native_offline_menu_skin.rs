//! Qualified LTsurf map-menu subset. Default routes never load this skin.
//! RGB555 reference only; no native runtime, language overlay or full UI parity.
use super::{compositor::{Canvas,Viewport},native_offline_preview_resources::Inputs};
use anyhow::{ensure,Context,Result};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{path::Path,sync::Arc};
#[path="../../tools/native-resource-adapter.rs"]mod adapter;
pub const WIDTH:u32=157;pub const HEIGHT:u32=81;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum CopyMode{Opaque,ColorKey}
pub struct Img {pub width:u32,pub height:u32,pub rgba:Vec<u8>}
pub fn decode_img(bytes:&[u8],mode:CopyMode)->Result<Img>{
 ensure!(bytes.len()>=8,"IMG header truncated");let word=|n|u16::from_le_bytes([bytes[n],bytes[n+1]]);
 let(w,h)=(word(0) as i16,word(2) as i16);ensure!(w>0&&h>0,"positive signed IMG dimensions required");let(w,h)=(w as u32,h as u32);
 ensure!(w<=2048&&h<=2048&&u64::from(w)*u64::from(h)<=1_048_576,"IMG decode budget");
 ensure!(word(4)<=1,"unknown IMG flag");ensure!(bytes.len()==8+w as usize*h as usize*2,"exact IMG pixel payload required");
 let mut rgba=Vec::with_capacity(w as usize*h as usize*4);
 for p in bytes[8..].chunks_exact(2){let v=u16::from_le_bytes([p[0],p[1]]);ensure!(v&0x8000==0,"RGB555 high bit unqualified");if mode==CopyMode::ColorKey&&v==word(6){rgba.extend_from_slice(&[0;4]);}else{let c=[(v>>10)&31,(v>>5)&31,v&31];rgba.extend(c.map(|x|((x<<3)|(x>>2)) as u8));rgba.push(255);}}
 Ok(Img{width:w,height:h,rgba})
}
fn read(root:&Path,p:&Value)->Result<Vec<u8>>{let mut p=p.clone();p["namespace"]=json!("MODERN_OWNED_MENU_SKIN");adapter::read_pinned(root,&p,"MODERN_OWNED_MENU_SKIN",128<<20).map_err(|e|anyhow::anyhow!("{e}"))}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Hit{Row,Up,Down}
#[derive(Clone,Copy,Debug)]pub struct Layout{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
impl Layout{
 pub fn fit(x:i32,y:i32,w:i32,h:i32)->Option<Self>{let v=Viewport::fit(w,h,WIDTH,HEIGHT)?;let x=x.checked_add(v.x)?;let y=y.checked_add(v.y)?;x.checked_add(v.width)?;y.checked_add(v.height)?;Some(Self{x,y,width:v.width,height:v.height})}
 pub fn source(&self,x:i32,y:i32)->Option<(i32,i32)>{let(dx,dy)=(i64::from(x)-i64::from(self.x),i64::from(y)-i64::from(self.y));if dx<0||dy<0||dx>=i64::from(self.width)||dy>=i64::from(self.height){return None;}Some(((dx*157/i64::from(self.width)) as i32,(dy*81/i64::from(self.height)) as i32))}
 pub fn hit(&self,x:i32,y:i32)->Option<Hit>{let(x,y)=self.source(x,y)?;if(142..153).contains(&x)&&(3..15).contains(&y){Some(Hit::Up)}else if(141..152).contains(&x)&&(65..77).contains(&y){Some(Hit::Down)}else if(9..141).contains(&x)&&(2..14).contains(&y){Some(Hit::Row)}else{None}}
 pub fn point(&self,x:i32,y:i32)->(i32,i32){(self.x+(i64::from(x)*i64::from(self.width)/157) as i32,self.y+(i64::from(y)*i64::from(self.height)/81) as i32)}
}
pub struct Skin{pub normal_bgra:Arc<[u8]>,pub up_pressed_bgra:Arc<[u8]>,pub down_pressed_bgra:Arc<[u8]>}
impl Skin{
 pub fn load(inputs:&Inputs)->Result<Self>{
  ensure!(inputs.project.is_absolute()&&inputs.manifest.starts_with(&inputs.project),"owned skin manifest root");let v=json!({"path":inputs.manifest,"bytes":inputs.manifest_pin.bytes,"sha256":adapter::hex(&inputs.manifest_pin.sha256)});let m:Value=serde_json::from_slice(&read(&inputs.project,&v)?)?;
  let index=read(&inputs.project,&m["LTsurf_index"])?;let pak=read(&inputs.project,&m["LTsurf_pak"])?;let table=adapter::parse_index(&index,pak.len()).map_err(|e|anyhow::anyhow!("{e}"))?;
  ensure!(table.len()==1623,"complete pinned LTsurf index");let rows=m["members"].as_array().context("skin members absent")?;ensure!(rows.len()==5,"five selected members");let mut images=Vec::new();
  for (row,id) in rows.iter().zip([6113,6232,6233,6234,6235]){ensure!(row["id"]==id,"selected member order");let name=format!("{id}.img");let range=table.get(&name).context("plain IMG absent")?;let raw=&pak[range.clone()];ensure!(raw.len() as u64==row["bytes"].as_u64().context("member size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("member pin")?,"selected member bytes drift");
   for suffix in ['e','c','h','j']{ensure!(!table.contains_key(&format!("{id}{suffix}.img")),"localized variant requires separate qualification");}
   let img=decode_img(raw,if id==6113{CopyMode::Opaque}else{CopyMode::ColorKey})?;ensure!((img.width,img.height)==if id==6113{(158,82)}else{(11,12)},"qualified IMG dimensions");images.push(img);
  }
  let compose=|up:usize,down:usize|->Result<Arc<[u8]>>{let mut c=Canvas::new(WIDTH,HEIGHT)?;c.blit_rgba(&images[0].rgba,158,82,0,0)?;c.blit_rgba(&images[up].rgba,11,12,142,3)?;c.blit_rgba(&images[down].rgba,11,12,141,65)?;let mut b=Vec::new();c.copy_bgra(&mut b)?;Ok(b.into())};
  Ok(Self{normal_bgra:compose(1,3)?,up_pressed_bgra:compose(2,3)?,down_pressed_bgra:compose(1,4)?})
 }
}
// Exactly one qualified row: scrolling has no valid target. Native arrow graphics
// retain normal appearance; disabled is a Modern eligibility rule, not a new bitmap.
pub fn scroll_enabled(qualified_rows:usize)->bool{qualified_rows>6}
#[cfg(test)]#[path="native_offline_menu_skin_tests.rs"]mod tests;
