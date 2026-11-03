//! Explicit AD368 room-local departure receipts. Synthetic account/source
//! expectations qualify only this adapter's classification, never a Native
//! identity or actual backend canonicalization. Qualified singleton provenance
//! enables only a completed-write/self-notification correlation, not leave ACK.
use anyhow::{ensure, Context, Result};
use sha2::{Digest, Sha256};
use super::Runtime;
pub const ROUTE_SHA:&str="7BF90D85A297E8AAA9D90D3BD4AF11DFD13992D685FF7211CBD1A9433CA898E6";
pub const NATIVE_SHA:&str="50A26C6526BC3BE8365DA435AE50117A25265EDE1AB0DC1E2F170BAEA95006C4";
pub const COMMIT_SHA:&str="FB4EB76E965BA1C094AD7BD542A4CD7542DD5F99987CF884B041AD9C9676D44D";
#[derive(Clone)] pub struct Adapter {binding:Option<Binding>}
#[derive(Clone)] struct Binding {account_hash:[u8;32],source:Vec<u8>,alias:Vec<u8>}
impl Adapter {
 pub fn parse(bytes:&[u8])->Result<Self>{
  ensure!(bytes.len()<=1024,"departure adapter exceeds bound");
  let v:serde_json::Value=serde_json::from_slice(bytes)?;
  ensure!(v["kind"]=="modern_synthetic_AD368_room_departure_v1" && v["producer_sha256"]==super::login_bootstrap::PRODUCER_SHA
   && v["route_sha256"]==ROUTE_SHA && v["native_sha256"]==NATIVE_SHA && v["commit_sha256"]==COMMIT_SHA,"pinned AD368 departure adapter required");
  let binding=if v["synthetic_account_source_expectation"].is_null(){None}else{
   let e=&v["synthetic_account_source_expectation"];
   let account=e["account_ascii"].as_str().context("synthetic account expectation absent")?.as_bytes();
   let source=e["source_ascii"].as_str().context("synthetic source expectation absent")?.as_bytes();
   // This finite subset requires declared canonical ASCII inputs and no
   // truncation. Java non-ASCII, fallback and canonicalization are open.
   ensure!(!account.is_empty()&&!source.is_empty()&&account.iter().chain(source).all(|b|b.is_ascii_alphanumeric()||*b==b'_'),"unqualified synthetic display expectation");
   let mut alias=account.to_vec();alias.push(b'(');alias.extend(source);alias.push(b')');
   ensure!(alias.len()<=27,"synthetic display expectation would require Java truncation");
   Some(Binding{account_hash:Sha256::digest(account).into(),source:source.to_vec(),alias})
  };Ok(Self{binding})
 }
 pub(crate) fn qualify(&self,runtime:&Runtime)->Option<Vec<u8>>{
  let b=self.binding.as_ref()?;
  if !runtime.synthetic_player_fixture_configured()||runtime.java_return.account!=Some(b.account_hash)
   ||runtime.login_bootstrap.key.selected_name.as_deref()!=Some(b.source.as_slice())
   ||runtime.login_bootstrap.key.stage!=super::login_bootstrap::Stage::Roster
   ||runtime.java_character.key.stage!=super::java_character_flow::Stage::ChannelReceipts{return None;}
  Some(b.alias.clone())
 }
}
#[derive(Clone,Debug,PartialEq,Eq)] pub struct Receipt {pub room_id:u32,pub display_alias:Vec<u8>,pub raw_body:Vec<u8>}
pub fn decode(body:&[u8])->Result<Receipt>{
 ensure!((7..=33).contains(&body.len())&&body[0]==208,"AD368 S_D0 envelope mismatch");
 let end=body[5..].iter().position(|b|*b==0).context("S_D0 bounded CString absent")?+5;
 ensure!(end==body.len()-1&&end>5,"S_D0 empty alias or trailing bytes");
 let display_alias=body[5..end].to_vec();validate_cp950(&display_alias)?;
 Ok(Receipt{room_id:u32::from_le_bytes(body[1..5].try_into().unwrap()),display_alias,raw_body:body.to_vec()})
}
#[cfg(windows)] pub(crate) fn validate_cp950(raw:&[u8])->Result<()>{
 use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};
 let mut wide=[0u16;27];let count=unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,raw,Some(&mut wide))};
 ensure!(count>0,"S_D0 invalid CP950 bytes");Ok(())
}
#[cfg(not(windows))] pub(crate) fn validate_cp950(raw:&[u8])->Result<()>{ensure!(raw.is_ascii(),"CP950 validation unavailable on this host");Ok(())}
impl Runtime {
 pub fn configure_java_room_departure_adapter(&mut self,adapter:Adapter)->Result<()>{
  ensure!(self.source_connection_epoch().is_none()&&self.java_room.key.phase==super::java_room_entry::Phase::Idle
   &&self.java_participants.book.is_none(),"departure adapter must precede owned entry");
  self.java_participants.departure_adapter=Some(adapter);Ok(())
 }
 /// Reserve ordinary C_D0 only with independent singleton GAME provenance.
 /// Scheduling is not completed write, and self notification is not leave ACK.
 pub fn request_java_room_leave(&mut self)->Result<()>{self.start_qualified_self_exit_notification()}
 pub(crate) fn java_room_departure_route_enabled(&self)->bool{self.java_room_participants_route_enabled()&&self.java_participants.departure_adapter.is_some()}
}
