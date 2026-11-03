//! Bounded explicit AD368 CC/0 full rows. This is a possibly incomplete
//! channel view, with no reset/end/role inference or Native UI equivalence.
use anyhow::{ensure,Context,Result};
pub const CONTRACT_SHA:&str="E376B067F4E8BACD7F0DC01F2A615BE0E2B0B71C2FA3DC59E001EC0330D3F9B5";
pub const LIFECYCLE_SHA:&str="3ADB8FD6D3F48E6BB444C00BF3808A1F1A758E82C2EB2E9CEDD4F45CFAB38367";
pub const COMPARISON_SHA:&str="D7FD0DE1748ABE871A7CB3EB416146CDE2FFFF1201003D5A2187F234E3131A63";
pub const PUBLICATION_SHA:&str="1D6F4348EAB19C212459E573F2A86F648CD88A724C7B7D1DD1F8A1D24F7A405B";
#[derive(Clone)]pub struct Adapter;
impl Adapter {
 pub fn parse(bytes:&[u8])->Result<Self>{
  ensure!(bytes.len()<=1024,"catalog adapter exceeds bound");let v:serde_json::Value=serde_json::from_slice(bytes)?;
  ensure!(v["kind"]=="modern_synthetic_AD368_room_catalog_v1"&&v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA
   &&v["contract_sha256"]==CONTRACT_SHA&&v["lifecycle_sha256"]==LIFECYCLE_SHA&&v["comparison_sha256"]==COMPARISON_SHA
   &&v["publication_sha256"]==PUBLICATION_SHA,"pinned CC0 catalog adapter required");Ok(Self)
 }
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Row {
 pub room_id:u32,pub name:Vec<u8>,pub attributes:u8,pub display_users:u16,pub capacity:u16,
 /// Raw Native mapword. Java's ordinary builder writes zero. This is neither
 /// the Native lookup index/UI low BYTE nor CE/D5/WORLD map authority.
 pub native_mapword:u16,pub mode:u8,pub raw_body:Vec<u8>,
}
pub fn decode(body:&[u8])->Result<Row>{
 // Deliberate bounded adapter subset: name 1..89 bytes, mode 0..5.
 // Native CString capacity100 is separate from this admission policy.
 ensure!((16..=104).contains(&body.len())&&body[0]==204&&body[1]==0,"only bounded AD368 CC subtype0 new format qualified");
 let room_id=u32::from_le_bytes(body[2..6].try_into().unwrap());ensure!(room_id!=0,"catalog room ID absent");
 let end=body[6..].iter().position(|b|*b==0).context("CC0 bounded CString absent")?+6;
 ensure!((1..=89).contains(&(end-6))&&end+9==body.len(),"CC0 empty/oversized name or nonexact body");
 let name=body[6..end].to_vec();validate_cp950(&name)?;let f=&body[end+1..];
 ensure!(f[7]<=5,"CC0 Native invalid-mode flags unqualified");
 Ok(Row{room_id,name,attributes:f[0],display_users:u16::from_le_bytes([f[1],f[2]]),capacity:u16::from_le_bytes([f[3],f[4]]),native_mapword:u16::from_le_bytes([f[5],f[6]]),mode:f[7],raw_body:body.to_vec()})
}
#[cfg(windows)]fn validate_cp950(raw:&[u8])->Result<()>{
 use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};
 let mut wide=[0u16;89];ensure!(unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,raw,Some(&mut wide))}>0,"CC0 invalid CP950 bytes");Ok(())
}
#[cfg(not(windows))]fn validate_cp950(raw:&[u8])->Result<()>{ensure!(raw.is_ascii(),"CP950 validation unavailable on this host");Ok(())}
