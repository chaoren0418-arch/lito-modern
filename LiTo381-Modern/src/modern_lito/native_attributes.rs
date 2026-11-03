//! Formal numeric schema 0. Extended layouts are rejected, never inferred from truncation.
use anyhow::{bail,ensure,Context,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Update{Ability{values:[u8;6],weight:u8},Experience{level:u8,experience:u32},Armor([i8;5]),Magic([i8;2])}
pub fn parse(b:&[u8])->Result<Update>{
    Ok(match *b.first().context("empty attribute update")?{
        0x50=>{ensure!(b.len()==8,"S50 schema0 exact eight bytes");Update::Ability{values:b[1..7].try_into()?,weight:b[7]}},
        0x51=>{ensure!(b.len()==6&&b[1]!=0,"S51 schema0 level and length");Update::Experience{level:b[1],experience:u32::from_le_bytes(b[2..6].try_into()?)}},
        0x52=>{ensure!(b.len()==6,"S52 schema0 exact six bytes");Update::Armor([b[1] as i8,b[2] as i8,b[3] as i8,b[4] as i8,b[5] as i8])},
        0x56=>{ensure!(b.len()==3,"S56 schema0 exact three bytes");Update::Magic([b[1] as i8,b[2] as i8])},
        _=>bail!("unsupported attribute opcode"),
    })
}
