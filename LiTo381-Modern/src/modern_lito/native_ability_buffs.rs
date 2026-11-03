//! Current formal S6B/S6C receipts. Values and duration come only from the server.
use anyhow::{ensure,Context,Result};
#[derive(Clone,Copy,Debug,PartialEq,Eq,serde::Serialize)]
pub struct Update {
    pub opcode:u8,
    pub received_seconds:u16,
    pub score:u8,
    pub weight_gauge:Option<u8>,
    pub kind:u8,
}
pub fn parse(bytes:&[u8])->Result<Update> {
    let opcode=*bytes.first().context("empty ability buff packet")?;
    ensure!(matches!(opcode,0x6b|0x6c),"not a strength/dexterity receipt");
    ensure!(bytes.len()==if opcode==0x6b{6}else{5},"ability buff exact envelope length");
    Ok(Update{opcode,received_seconds:u16::from_le_bytes([bytes[1],bytes[2]]),score:bytes[3],
        weight_gauge:(opcode==0x6b).then(||bytes[4]),kind:bytes[bytes.len()-1]})
}
