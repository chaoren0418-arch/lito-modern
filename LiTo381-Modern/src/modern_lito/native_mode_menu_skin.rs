//! Qualified LTsurf map-menu subset. Default routes never load this skin.
//! RGB555 reference only; no native runtime, language overlay or full UI parity.
use super::{compositor::{Canvas,Viewport},native_offline_preview_resources::Inputs};
use anyhow::{ensure,Context,Result};
use serde_json::{Value,json};
use sha2::{Digest,Sha256};
use std::{path::Path,sync::Arc};
#[path="native_background_assets.rs"]pub mod background;
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
pub struct Skin{pub normal_bgra:Arc<[u8]>,pub up_pressed_bgra:Arc<[u8]>,pub down_pressed_bgra:Arc<[u8]>,pub entry_art:Option<Arc<[u8]>>,pub character_buttons:Vec<Img>,pub channel_buttons:Vec<Img>,pub channel_background:Option<Img>,pub channel_back:Vec<Img>,thumb:Img,arrows:Vec<Img>}
impl Skin{
 pub fn load(inputs:&Inputs)->Result<Self>{
  ensure!(inputs.project.is_absolute()&&inputs.manifest.starts_with(&inputs.project),"owned skin manifest root");let v=json!({"path":inputs.manifest,"bytes":inputs.manifest_pin.bytes,"sha256":adapter::hex(&inputs.manifest_pin.sha256)});let m:Value=serde_json::from_slice(&read(&inputs.project,&v)?)?;
  let index=read(&inputs.project,&m["LTsurf_index"])?;let pak=read(&inputs.project,&m["LTsurf_pak"])?;let table=adapter::parse_index(&index,pak.len()).map_err(|e|anyhow::anyhow!("{e}"))?;
  let entry_art=if let Some(row)=m.get("entry_art"){ensure!(row["id"]==6279,"source entry image identity");let range=table.get("6279.img").context("source entry IMG absent")?;let raw=&pak[range.clone()];ensure!(raw.len()as u64==row["bytes"].as_u64().context("entry size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("entry SHA")?,"entry IMG pin drift");let img=decode_img(raw,CopyMode::Opaque)?;ensure!((img.width,img.height)==(11,11),"source entry IMG geometry");let mut c=Canvas::new(11,11)?;c.blit_rgba(&img.rgba,11,11,0,0)?;let mut bgra=Vec::new();c.copy_bgra(&mut bgra)?;Some(Arc::<[u8]>::from(bgra))}else{None};
  let mut character_buttons=Vec::new();if let Some(rows)=m.get("character_buttons"){let rows=rows.as_array().context("character button rows")?;ensure!(rows.len()==4,"four original character button images");for(row,id)in rows.iter().zip([6303,6304,6201,6202]){ensure!(row["id"]==id,"character button identity");let range=table.get(&format!("{id}.img")).context("character button absent")?;let raw=&pak[range.clone()];ensure!(raw.len()as u64==row["bytes"].as_u64().context("button size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("button SHA")?,"character button pin drift");let image=decode_img(raw,CopyMode::ColorKey)?;ensure!((image.width,image.height)==(51,23),"character button geometry");character_buttons.push(image);}}
  let load_channel=|row:&Value,id:u32,geometry:(u32,u32),mode:CopyMode|->Result<Img>{ensure!(row["id"]==id,"channel image identity");let range=table.get(&format!("{id}.img")).context("channel image absent")?;let raw=&pak[range.clone()];ensure!(raw.len()as u64==row["bytes"].as_u64().context("channel size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("channel SHA")?,"channel image pin drift");let image=decode_img(raw,mode)?;ensure!((image.width,image.height)==geometry,"channel geometry");Ok(image)};
  let channel_background=m.get("channel_background").map(|r|load_channel(r,7580,(640,480),CopyMode::Opaque)).transpose()?;
  let mut channel_buttons=Vec::new();if let Some(rows)=m.get("channel_buttons"){let rows=rows.as_array().context("channel buttons")?;ensure!(rows.len()==8,"four channel normal/pressed pairs");for(row,id)in rows.iter().zip(7581..7589){channel_buttons.push(load_channel(row,id,(72,69),CopyMode::ColorKey)?);}}
  let mut channel_back=Vec::new();if let Some(rows)=m.get("channel_back"){let rows=rows.as_array().context("channel Back")?;ensure!(rows.len()==2,"channel Back normal/pressed pair");for(row,id)in rows.iter().zip([6205,6206]){channel_back.push(load_channel(row,id,(51,23),CopyMode::ColorKey)?);}}
  ensure!(table.len()==1623,"complete pinned LTsurf index");let rows=m["members"].as_array().context("skin members absent")?;ensure!(rows.len()==6,"six selected members");let mut images=Vec::new();
  for (row,id) in rows.iter().zip([6113,6232,6233,6234,6235,6231]){ensure!(row["id"]==id,"selected member order");let name=format!("{id}.img");let range=table.get(&name).context("plain IMG absent")?;let raw=&pak[range.clone()];ensure!(raw.len() as u64==row["bytes"].as_u64().context("member size")?&&format!("{:X}",Sha256::digest(raw))==row["sha256"].as_str().context("member pin")?,"selected member bytes drift");
   for suffix in ['e','c','h','j']{ensure!(!table.contains_key(&format!("{id}{suffix}.img")),"localized variant requires separate qualification");}
   let img=decode_img(raw,if id==6113{CopyMode::Opaque}else{CopyMode::ColorKey})?;ensure!((img.width,img.height)==if id==6113{(158,82)}else if id==6231{(7,10)}else{(11,12)},"qualified IMG dimensions");images.push(img);
  }
  let mut index=read(&inputs.project,&m["LTtext_index"])?;let pak=read(&inputs.project,&m["LTtext_pak"])?;
  ensure!(index.len()==9888&&u32::from_le_bytes(index[..4].try_into()?)==353,"qualified LTtext header");super::resource_cipher::decode_payload_in_place(&mut index[4..]);
  let mut decoded=None;for row in index[4..].chunks_exact(28){let end=row[4..24].iter().position(|b|*b==0).unwrap_or(20);if row[4..4+end].eq_ignore_ascii_case(b"LTstring-k.tbl"){ensure!(decoded.is_none(),"ambiguous header member");let offset=u32::from_le_bytes(row[..4].try_into()?) as usize;let size=u32::from_le_bytes(row[24..].try_into()?) as usize;let end=offset.checked_add(size).context("header member overflow")?;ensure!(end<=pak.len()&&size<=8192,"header member bounds");let mut b=pak[offset..end].to_vec();super::resource_cipher::decode_payload_in_place(&mut b);decoded=Some(b);}}
  let bytes=decoded.context("LTstring-k header member absent")?;ensure!(bytes==read(&inputs.project,&m["header_table"])? ,"decoded header content mismatch");let lines:Vec<_>=bytes.split(|b|*b==b'\n').map(|l|l.strip_suffix(b"\r").unwrap_or(l)).collect();ensure!(lines[0]==b"296"&&lines.len()>=297&&lines[109]==b"Random"&&lines[270]==b"Hot Random","exact native header records");
  ensure!(m["headers"]==json!([{"key":-100,"index":269,"text":"Hot Random"},{"key":-1,"index":108,"text":"Random"}]),"header order/key/text");
  let compose=|up:usize,down:usize|->Result<Arc<[u8]>>{let mut c=Canvas::new(WIDTH,HEIGHT)?;c.blit_rgba(&images[0].rgba,158,82,0,0)?;c.blit_rgba(&images[up].rgba,11,12,142,3)?;c.blit_rgba(&images[down].rgba,11,12,141,65)?;let mut b=Vec::new();c.copy_bgra(&mut b)?;Ok(b.into())};
  let normal_bgra=compose(1,3)?;let up_pressed_bgra=compose(2,3)?;let down_pressed_bgra=compose(1,4)?;let thumb=images.pop().unwrap();Ok(Self{normal_bgra,up_pressed_bgra,down_pressed_bgra,entry_art,character_buttons,channel_buttons,channel_background,channel_back,thumb,arrows:images.into_iter().skip(1).collect()})
 }
 pub fn compose(&self,top:usize,count:usize,pressed:Option<super::native_mode_menu::Hit>)->Result<Arc<[u8]>>{
  let y=super::native_mode_menu::thumb_y(top,count).context("thumb top/count invalid")?;let base=match pressed{Some(super::native_mode_menu::Hit::Up)=>&self.up_pressed_bgra,Some(super::native_mode_menu::Hit::Down)=>&self.down_pressed_bgra,_=>&self.normal_bgra};let mut bgra=base.to_vec();
  for row in 0..10usize{for col in 0..7usize{let q=(row*7+col)*4;let d=((row+y as usize)*157+143+col)*4;if self.thumb.rgba[q+3]!=0{bgra[d..d+4].copy_from_slice(&[self.thumb.rgba[q+2],self.thumb.rgba[q+1],self.thumb.rgba[q],self.thumb.rgba[q+3]]);}}}
  //4744F0 draws thumb before child arrow images, including their bottom overlap.
  let (up,down)=match pressed{Some(super::native_mode_menu::Hit::Up)=>(1,2),Some(super::native_mode_menu::Hit::Down)=>(0,3),_=>(0,2)};
  for (i,x,y) in [(up,142,3),(down,141,65)]{let a=&self.arrows[i];for row in 0..12usize{for col in 0..11usize{let q=(row*11+col)*4;let d=((row+y)*157+x+col)*4;if a.rgba[q+3]!=0{bgra[d..d+4].copy_from_slice(&[a.rgba[q+2],a.rgba[q+1],a.rgba[q],a.rgba[q+3]]);}}}}
  Ok(bgra.into())
 }
}
