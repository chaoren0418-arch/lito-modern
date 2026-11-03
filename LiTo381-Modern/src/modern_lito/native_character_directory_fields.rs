//! The34-field S_CB envelope, separate from any actor/object/source identity.
use anyhow::{ensure,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Ranking<T> {pub overall:T,pub level:T,pub classic_source:T,pub channel:T,pub race:T}
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Abilities {pub strength:u8,pub dexterity:u8,pub constitution:u8,pub wisdom:u8,pub charisma:u8,pub intelligence:u8}
#[derive(Clone,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Fields {
 pub class:u8,pub sex:u8,pub lawful:i16,pub max_hp:u16,pub max_mp:u16,pub armor_class:i8,pub level:u8,pub abilities:Abilities,pub reserved:u8,pub native_time:u32,
 pub ranks:Ranking<u32>,pub disconnects:u32,pub win_rate_denominator:u32,pub wins:u32,pub losses:u32,pub ranking_selector:u8,pub movements:Ranking<i32>,pub check_code:u8,
}
impl Fields {
 pub(crate) fn decode(b:&[u8])->Result<Self>{
  ensure!(b.len()==79,"S_CB numeric suffix must be79 bytes");let h=|i|u16::from_le_bytes(b[i..i+2].try_into().unwrap());let d=|i|u32::from_le_bytes(b[i..i+4].try_into().unwrap());
  Ok(Self{class:b[0],sex:b[1],lawful:h(2) as i16,max_hp:h(4),max_mp:h(6),armor_class:b[8] as i8,level:b[9],abilities:Abilities{strength:b[10],dexterity:b[11],constitution:b[12],wisdom:b[13],charisma:b[14],intelligence:b[15]},reserved:b[16],native_time:d(17),ranks:Ranking{overall:d(21),level:d(25),classic_source:d(29),channel:d(33),race:d(37)},disconnects:d(41),win_rate_denominator:d(45),wins:d(49),losses:d(53),ranking_selector:b[57],movements:Ranking{overall:d(58) as i32,level:d(62) as i32,classic_source:d(66) as i32,channel:d(70) as i32,race:d(74) as i32},check_code:b[78]})
 }
 pub fn validate_java_projection(&self)->Result<()> {
  ensure!(self.class<=6&&self.sex<=1&&self.max_hp<=32767&&self.max_mp<=32767&&(1..=127).contains(&self.level)&&self.reserved==0&&self.ranking_selector<=4,"S_CB Java projected domain mismatch");
  let a=self.abilities;ensure!([a.strength,a.dexterity,a.constitution,a.wisdom,a.charisma,a.intelligence].iter().all(|v|*v<=127),"S_CB projected ability outside signed-safe domain");
  ensure!(self.check_code==self.level^a.strength^a.dexterity^a.constitution^a.wisdom^a.charisma^a.intelligence,"S_CB projected check-code mismatch");
  ensure!(self.disconnects<=self.losses&&self.wins.checked_add(self.losses)==Some(self.win_rate_denominator),"S_CB battle counters inconsistent");Ok(())
 }
}
/// Presentation conversion only; commands always echo original received bytes.
pub fn display_ms950(bytes:&[u8])->Result<String>{
 use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};
 ensure!(bytes.len()<=256&&!bytes.contains(&0),"bounded MS950 display required");if bytes.is_empty(){return Ok(String::new());}
 let mut wide=vec![0u16;bytes.len()];let n=unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,bytes,Some(&mut wide))};ensure!(n>0&&n as usize<=wide.len(),"invalid received MS950 name");Ok(String::from_utf16(&wide[..n as usize])?)
}
