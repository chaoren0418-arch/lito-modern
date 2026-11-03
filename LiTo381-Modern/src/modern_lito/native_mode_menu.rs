//! Native construction/filter subset; mode-rebuild trigger, keys and Back policy are Modern.
use super::{native_multi_map_resources::Entry,native_offline_menu_skin::Layout};
use anyhow::{ensure,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Filter{Mode{mode:u8,alternate:bool},DiagnosticAll}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Row{Header{key:i32,index:usize,text:&'static str},Map(usize)}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Hit{Row(usize),Up,Down,Thumb}
/// 46B7D0 compares exact mode first; alternate excludes547/548, ordinary mode3 odd only excluding549/550.
pub fn native_visible(mode:u8,map_mode:u8,map_id:u32,alternate:bool)->bool{
 if mode!=map_mode{return false;}if alternate{return !matches!(map_id,547|548);}
 mode!=3||(!matches!(map_id,549|550)&&map_id%2!=0)
}
pub fn thumb_y(top:usize,count:usize)->Option<i32>{
 if (count==0&&top!=0)||(count>0&&top>=count)||count>256{return None;}
 Some(15+if count<=1{0}else{(41*top/(count-1)) as i32})
}
#[derive(Clone,Debug)]pub struct Menu{pub top:usize,pub selected:Option<usize>,pub pressed:Option<Hit>,pub filter:Filter,rows:Vec<Row>,captured:Option<Hit>,repeat_due:Option<u64>}
impl Menu{
 pub fn for_filter(catalog:&[Entry],filter:Filter)->Result<Self>{
  ensure!(catalog.len()<=40,"bounded catalog");let mut rows=Vec::new();
  if let Filter::Mode{mode,..}=filter{ensure!(mode<6,"unknown mode");rows.extend([Row::Header{key:-100,index:269,text:"Hot Random"},Row::Header{key:-1,index:108,text:"Random"}]);}
  for(i,e)in catalog.iter().enumerate(){if match filter{Filter::DiagnosticAll=>true,Filter::Mode{mode,alternate}=>native_visible(mode,e.choice.mode,e.choice.map_id,alternate)}{rows.push(Row::Map(i));}}
  Ok(Self{top:0,selected:None,pressed:None,filter,rows,captured:None,repeat_due:None})
 }
 pub fn rows(&self)->&[Row]{&self.rows}pub fn count(&self)->usize{self.rows.len()}
 pub fn row(&self,i:usize)->Option<Row>{self.rows.get(i).copied()}
 pub fn choose(&mut self,i:usize)->Result<()>{ensure!(i<self.count(),"row outside filtered menu");self.selected=Some(i);self.cancel_press();Ok(())}
 pub fn enabled(&self,h:Hit)->bool{match h{Hit::Up=>self.top>0,Hit::Down=>self.top+1<self.count(),Hit::Row(i)=>i<self.count(),Hit::Thumb=>false}}
 pub fn hit(&self,l:&Layout,x:i32,y:i32)->Option<Hit>{let(x,y)=l.source(x,y)?;if(142..153).contains(&x)&&(3..15).contains(&y){return Some(Hit::Up);}if(141..152).contains(&x)&&(65..77).contains(&y){return Some(Hit::Down);}let ty=thumb_y(self.top,self.count())?;if(143..150).contains(&x)&&(ty..ty+10).contains(&y){return Some(Hit::Thumb);}if !(9..141).contains(&x)||!(2..74).contains(&y){return None;}let i=self.top+((y-2)/12) as usize;if i<self.count(){Some(Hit::Row(i))}else{None}}
 pub fn down(&mut self,h:Option<Hit>)->Option<usize>{self.down_at(h,0)}
 /// Native arrow construction enables repeat.414BD0 arms700ms;41AA00
 /// calls once then reschedules125ms from its current clock, without catch-up.
 pub fn down_at(&mut self,h:Option<Hit>,now_ms:u64)->Option<usize>{self.cancel_press();match h{Some(Hit::Row(i)) if i<self.count()=>{self.selected=Some(i);Some(i)},Some(h @ (Hit::Up|Hit::Down)) if self.enabled(h)=>{self.pressed=Some(h);self.captured=Some(h);self.repeat_due=now_ms.checked_add(700);None},_=>None}}
 pub fn pointer_move(&mut self,h:Option<Hit>){self.pointer_move_at(h,0);}
 pub fn pointer_move_at(&mut self,h:Option<Hit>,now_ms:u64){let before=self.pressed;self.pressed=self.captured.filter(|c|Some(*c)==h);if self.pressed.is_none(){self.repeat_due=None;}else if before.is_none(){self.repeat_due=now_ms.checked_add(700);}}
 pub fn repeat_deadline(&self)->Option<u64>{self.repeat_due}
 pub fn repeat_at(&mut self,now_ms:u64)->bool{let Some(due)=self.repeat_due else{return false;};if now_ms<due{return false;}let Some(h @ (Hit::Up|Hit::Down))=self.pressed.filter(|h|Some(*h)==self.captured)else{self.repeat_due=None;return false;};self.repeat_due=now_ms.checked_add(125);self.scroll(h)}
 fn scroll(&mut self,h:Hit)->bool{match h{Hit::Up if self.top>0=>self.top-=1,Hit::Down if self.top+1<self.count()=>self.top+=1,_=>return false}true}
 /// Native414C00 removes pending callbacks and still calls once on valid release.
 pub fn release(&mut self,h:Option<Hit>)->bool{let pressed=self.pressed;let captured=self.captured;self.cancel_press();if pressed.is_none()||pressed!=h||captured!=h{return false;}self.scroll(pressed.unwrap())}
 pub fn cancel_press(&mut self){self.pressed=None;self.captured=None;self.repeat_due=None;}
 /// Source450230 clears existing list items/count/top. Modern also discards
 /// the now-invalid selection and held repeat/capture state.
 pub fn clear_items(&mut self){self.rows.clear();self.top=0;self.selected=None;self.cancel_press();}
 pub fn key(&mut self,key:u32)->Option<usize>{if self.count()==0{return None;}let selected=self.selected.unwrap_or(0);let next=match key{0x26=>selected.saturating_sub(1),0x28=>if self.selected.is_none(){0}else{(selected+1).min(self.count()-1)},0x21=>selected.saturating_sub(6),0x22=>(selected+6).min(self.count()-1),0x24=>0,0x23=>self.count()-1,_=>return None};self.selected=Some(next);if next<self.top{self.top=next;}else if next>=self.top+6{self.top=next+1-6;}self.cancel_press();Some(next)}
 pub fn row_rect(l:&Layout,row:usize)->Option<(i32,i32,i32,i32)>{super::native_multi_map_menu::Menu::row_rect(l,row)}
 pub fn font_height(l:&Layout)->i32{super::native_multi_map_menu::Menu::font_height(l)}
 pub fn thumb_rect(&self,l:&Layout)->Option<(i32,i32,i32,i32)>{let y=thumb_y(self.top,self.count())?;let px=|n:i32|l.x+((i64::from(n)*i64::from(l.width)+156)/157) as i32;let py=|n:i32|l.y+((i64::from(n)*i64::from(l.height)+80)/81) as i32;Some((px(143),py(y),px(150),py(y+10)))}
}
#[cfg(test)]#[path="native_mode_menu_tests.rs"]mod tests;
