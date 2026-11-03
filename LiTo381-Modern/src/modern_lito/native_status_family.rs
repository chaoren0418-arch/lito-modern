//! Batch12 fixed S50/S51/S52/S56 receipts. No Native owner or side effects.
//! Raw storage bits and Java semantic interpretations remain distinct.
use anyhow::{ensure,Context,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Abilities {pub strength:u8,pub intelligence:u8,pub wisdom:u8,pub dexterity:u8,pub constitution:u8,pub charisma:u8,pub weight_gauge:u8}
impl Abilities {pub fn raw_scores(self)->[u8;6]{[self.strength,self.intelligence,self.wisdom,self.dexterity,self.constitution,self.charisma]}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Experience {pub level:u8,pub experience:u32}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Defense {pub armor_class_raw:u8,pub fire_raw:u8,pub water_raw:u8,pub wind_raw:u8,pub earth_raw:u8}
impl Defense {
 /// Existing Java schema0 i8 domains; not downstream Native arithmetic proof.
 pub fn java_signed(self)->[i8;5]{[self.armor_class_raw as i8,self.fire_raw as i8,self.water_raw as i8,self.wind_raw as i8,self.earth_raw as i8]}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Magic {pub spell_power_modifier_raw:u8,pub magic_resistance_modifier_raw:u8}
impl Magic {pub fn java_signed(self)->[i8;2]{[self.spell_power_modifier_raw as i8,self.magic_resistance_modifier_raw as i8]}}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Packet {Abilities(Abilities),Experience(Experience),Defense(Defense),Magic(Magic)}
pub fn decode(b:&[u8])->Result<Packet>{
 // Exact length rejection is Modern policy. Original handlers have no guard.
 match b.first(){
  Some(0x50)=>{ensure!(b.len()==8,"S50 exact eight-byte body required");Ok(Packet::Abilities(Abilities{strength:b[1],intelligence:b[2],wisdom:b[3],dexterity:b[4],constitution:b[5],charisma:b[6],weight_gauge:b[7]}))},
  Some(0x51)=>{ensure!(b.len()==6,"S51 exact six-byte body required");Ok(Packet::Experience(Experience{level:b[1],experience:u32::from_le_bytes(b[2..6].try_into().unwrap())}))},
  Some(0x52)=>{ensure!(b.len()==6,"S52 exact six-byte body required");Ok(Packet::Defense(Defense{armor_class_raw:b[1],fire_raw:b[2],water_raw:b[3],wind_raw:b[4],earth_raw:b[5]}))},
  Some(0x56)=>{ensure!(b.len()==3,"S56 exact three-byte body required");Ok(Packet::Magic(Magic{spell_power_modifier_raw:b[1],magic_resistance_modifier_raw:b[2]}))},
  _=>anyhow::bail!("not a fixed S50/S51/S52/S56 status receipt"),
 }
}
/// Inspection projection of overlapping wire fields, ordered by receipt.
/// S0C remains an immutable last full receipt in the enclosing Runtime Key.
/// No level-change/protected/time callback is implemented by this projection.
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub struct Projection {pub revision:u64,pub last_opcode:Option<u8>,pub scores:Option<[u8;6]>,pub weight_gauge:Option<u8>,pub progress:Option<Experience>,pub defense:Option<Defense>,pub magic:Option<Magic>}
impl Projection {
 fn next(self,opcode:u8)->Result<Self>{Ok(Self{revision:self.revision.checked_add(1).context("status receipt revision exhausted")?,last_opcode:Some(opcode),..self})}
 pub fn with_full_status(self,s:super::status_receipt::Status)->Result<Self>{
  Ok(Self{scores:Some([s.strength,s.intelligence,s.wisdom,s.dexterity,s.constitution,s.charisma]),weight_gauge:Some(s.weight_gauge),progress:Some(Experience{level:s.level,experience:s.experience}),defense:Some(Defense{armor_class_raw:s.armor_class as u8,fire_raw:s.fire_resistance as u8,water_raw:s.water_resistance as u8,wind_raw:s.wind_resistance as u8,earth_raw:s.earth_resistance as u8}),..self.next(12)?})
 }
 pub fn with_packet(self,p:Packet)->Result<Self>{
  Ok(match p {
   Packet::Abilities(a)=>Self{scores:Some(a.raw_scores()),weight_gauge:Some(a.weight_gauge),..self.next(80)?},
   // S51 copies level/experience only; deliberately no S0C level effect.
   Packet::Experience(e)=>Self{progress:Some(e),..self.next(81)?},
   Packet::Defense(d)=>Self{defense:Some(d),..self.next(82)?},
   Packet::Magic(m)=>Self{magic:Some(m),..self.next(86)?},
  })
 }
}
