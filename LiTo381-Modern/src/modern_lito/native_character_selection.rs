//! Closed selection/callback subset:630DF8 stride60 ->415A30 vtable4F7C68.
//! Slot IMG79/80 and PNG animation members are absent from qualified assets.
//! Slot anchors are authoritative; their width/height and Native art stay open.
use anyhow::{Context,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Rect{pub x:i32,pub y:i32,pub width:i32,pub height:i32}
impl Rect{pub fn contains(self,x:i32,y:i32)->bool{let(x,y)=(i64::from(x)-i64::from(self.x),i64::from(y)-i64::from(self.y));x>=0&&y>=0&&x<i64::from(self.width)&&y<i64::from(self.height)}}
pub const SUBMIT:Rect=Rect{x:497,y:404,width:51,height:23};
pub const BACK:Rect=Rect{x:552,y:404,width:51,height:23};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Slot{pub index:usize,pub x:i32,pub y:i32,pub image_id:u32,pub callback:u32,pub width:Option<i32>,pub height:Option<i32>}
pub fn slot(index:usize)->Option<Slot>{(index<4).then(||Slot{index,x:19+157*index as i32,y:0,image_id:79,callback:0x4934A0,width:None,height:None})}
///494E00 signed class switch; variants branch on nonzero, not variant==1.
pub fn animation_table(class:u8,variant:u8)->u32{let nonzero=variant!=0;match class as i8{-1=>0x62FD70,0=>if nonzero{0x6300D0}else{0x630130},1=>if nonzero{0x62FF50}else{0x62FFB0},2=>if nonzero{0x62FE90}else{0x62FEF0},3=>if nonzero{0x630010}else{0x630070},4=>if nonzero{0x62FDD0}else{0x62FE30},_=>0x630190}}
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]pub struct Key{pub active:bool,pub scene_revision:u64,pub selection_revision:u64,pub selected:Option<usize>,pub pressed:Option<usize>}
#[derive(Clone,Default)]pub(crate) struct Scene{key:Key}
impl Scene{
 pub fn key(&self)->Key{self.key}
 pub fn enter(&mut self){self.key.active=true;}
 pub fn select(&mut self,slot:usize)->Result<()>{let revision=self.key.selection_revision.checked_add(1).context("selection revision exhausted")?;self.key.selected=Some(slot);self.key.pressed=None;self.key.selection_revision=revision;Ok(())}
 pub fn press(&mut self,slot:usize){self.key.pressed=Some(slot);}
 pub fn cancel_press(&mut self){self.key.pressed=None;}
 pub fn retire(&mut self)->Result<()>{let revision=self.key.scene_revision.checked_add(1).context("character scene revision exhausted")?;self.key=Key{active:false,scene_revision:revision,..self.key};self.key.selected=None;self.key.pressed=None;Ok(())}
}
