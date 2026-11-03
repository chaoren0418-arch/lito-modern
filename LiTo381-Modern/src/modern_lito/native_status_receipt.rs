//! Fixed S_STATUS/schema0 numeric receipt, Batch10 004A36F0.
//! This copies wire values only. Native protected/time/virtual side effects,
//! actor and 834888 skill owner publication remain unqualified.
use anyhow::{ensure, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ManaPoint {pub current_raw:u16,pub maximum_raw:u16}
impl ManaPoint {
    /// 004AEFB9 movsx word, unlike S0C's current-MP maskFFFF.
    pub fn native_current_argument(self)->i32 {i32::from(self.current_raw as i16)}
}
pub fn decode_mana(b:&[u8])->Result<ManaPoint> {
    ensure!(b.len()==5&&b[0]==0x4d,"S_MANA_POINT exact five-byte body required");
    Ok(ManaPoint{current_raw:u16::from_le_bytes(b[1..3].try_into().unwrap()),maximum_raw:u16::from_le_bytes(b[3..5].try_into().unwrap())})
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Status {
    pub object_id: u32, pub level: u8, pub experience: u32,
    pub strength: u8, pub intelligence: u8, pub wisdom: u8,
    pub dexterity: u8, pub constitution: u8, pub charisma: u8,
    pub current_hp: u16, pub maximum_hp: u16,
    pub current_mp: u16, pub maximum_mp: u16,
    pub armor_class: i8, pub game_time: u32,
    pub food_gauge: u8, pub weight_gauge: u8, pub lawful: i16,
    pub fire_resistance: i8, pub water_resistance: i8,
    pub wind_resistance: i8, pub earth_resistance: i8,
}
pub fn decode(b: &[u8]) -> Result<Status> {
    // Strict size is Modern policy; original numeric reader has no such guard.
    ensure!(b.len()==37 && b[0]==0x0c, "S_STATUS schema0 exact 37-byte body required");
    let d=|i|u32::from_le_bytes(b[i..i+4].try_into().unwrap());
    let h=|i|u16::from_le_bytes(b[i..i+2].try_into().unwrap());
    Ok(Status {object_id:d(1),level:b[5],experience:d(6),strength:b[10],
        intelligence:b[11],wisdom:b[12],dexterity:b[13],constitution:b[14],charisma:b[15],
        current_hp:h(16),maximum_hp:h(18),current_mp:h(20),maximum_mp:h(22),
        armor_class:b[24] as i8,game_time:d(25),food_gauge:b[29],weight_gauge:b[30],
        lawful:h(31) as i16,fire_resistance:b[33] as i8,water_resistance:b[34] as i8,
        wind_resistance:b[35] as i8,earth_resistance:b[36] as i8})
}
