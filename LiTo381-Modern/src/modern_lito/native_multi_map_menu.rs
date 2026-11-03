//! Native row pitch/top bounds and source arrow press/release subset.
//! Modern keyboard selection and text placement are explicit adaptations.
use super::native_offline_menu_skin::Layout;
pub const ROWS:usize=6;pub const PITCH:i32=12;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Hit{Row(usize),Up,Down}
#[derive(Clone,Debug)]pub struct Menu{pub top:usize,pub selected:usize,pub pressed:Option<Hit>,count:usize,captured:Option<Hit>}
impl Menu{
 pub fn new(count:usize,selected:usize)->Option<Self>{if count==0||count>40||selected>=count{return None;}Some(Self{top:0,selected,pressed:None,count,captured:None})}
 pub fn hit(&self,l:&Layout,x:i32,y:i32)->Option<Hit>{let(x,y)=l.source(x,y)?;if(142..153).contains(&x)&&(3..15).contains(&y){return Some(Hit::Up);}if(141..152).contains(&x)&&(65..77).contains(&y){return Some(Hit::Down);}if !(9..141).contains(&x)||!(2..74).contains(&y){return None;}let i=self.top+((y-2)/PITCH) as usize;if i<self.count{Some(Hit::Row(i))}else{None}}
 pub fn enabled(&self,h:Hit)->bool{match h{Hit::Up=>self.top>0,Hit::Down=>self.top+1<self.count,Hit::Row(i)=>i<self.count}}
 pub fn down(&mut self,h:Option<Hit>)->Option<usize>{self.cancel_press();match h{Some(Hit::Row(i)) if i<self.count=>{self.selected=i;Some(i)},Some(h @ (Hit::Up|Hit::Down)) if self.enabled(h)=>{self.pressed=Some(h);self.captured=Some(h);None},_=>None}}
 pub fn release(&mut self,h:Option<Hit>)->bool{let pressed=self.pressed.take();let captured=self.captured.take();if pressed!=h||captured!=h||pressed.is_none(){return false;}match pressed.unwrap(){Hit::Up if self.top>0=>self.top-=1,Hit::Down if self.top+1<self.count=>self.top+=1,_=>return false}true}
 pub fn pointer_move(&mut self,h:Option<Hit>){self.pressed=self.captured.filter(|c|Some(*c)==h);}
 pub fn cancel_press(&mut self){self.pressed=None;self.captured=None;}
 pub fn key(&mut self,key:u32)->Option<usize>{let next=match key{0x26=>self.selected.saturating_sub(1),0x28=>(self.selected+1).min(self.count-1),0x21=>self.selected.saturating_sub(ROWS),0x22=>(self.selected+ROWS).min(self.count-1),0x24=>0,0x23=>self.count-1,_=>return None};self.selected=next;if next<self.top{self.top=next;}else if next>=self.top+ROWS{self.top=next+1-ROWS;}self.cancel_press();Some(next)}
 pub fn row_rect(l:&Layout,row:usize)->Option<(i32,i32,i32,i32)>{if row>=ROWS{return None;}let px=|n:i32|l.x+((i64::from(n)*i64::from(l.width)+156)/157) as i32;let py=|n:i32|l.y+((i64::from(n)*i64::from(l.height)+80)/81) as i32;Some((px(9),py(2+row as i32*12),px(141),py(14+row as i32*12)))}
 pub fn font_height(l:&Layout)->i32{-(10*i64::from(l.height)/81).max(1) as i32}
}
#[cfg(test)]#[path="native_multi_map_menu_tests.rs"]mod tests;
