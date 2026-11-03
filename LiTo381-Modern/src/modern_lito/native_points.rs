//! Non-result S_DB from formal sendPointStats; trailing combat fields are all zero.
use anyhow::{ensure,Context,Result};
pub fn parse(b:&[u8])->Result<(String,u32)>{
    ensure!(b.first()==Some(&0xdb),"points opcode");
    let len=b[1..].iter().take(28).position(|x|*x==0).context("points name lacks bounded NUL")?;
    let name=super::native_login::decode_text(&b[1..1+len])?;let tail=b.get(len+2..).context("points tail")?;
    ensure!(tail.len()==18&&tail[4..].iter().all(|x|*x==0),"non-result points contain combat score fields");
    Ok((name,u32::from_le_bytes(tail[..4].try_into()?)))
}
