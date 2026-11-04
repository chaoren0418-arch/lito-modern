//! Candidate window bindings. Private Runtime controls supply all action authority.
//! Display only actual D1/E1 observations; no local countdown or owner command.
use super::{Runtime,UiPage,java_room_entry as entry,java_room_ready as ready,java_room_participants as members,java_room_settings as settings};
use anyhow::{ensure,Context,Result};
#[derive(Clone,Debug,Default,PartialEq,Eq)]pub struct Display {
 pub room_id:Option<u32>,pub room_name:String,pub members:Vec<String>,pub own_d1_state:Option<u8>,pub countdown:Option<u8>,
}
#[derive(Clone,Debug,PartialEq,Eq)]pub struct RoomRow {pub room_id:u32,pub label:String,pub password_required:bool}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Operation {Ready,Cancel,Join(usize),Refresh}
type CatalogKey=(u64,u8,u64,u64);
#[derive(Clone,Default)]pub struct Panel {
 pub display:Display,pub rooms:Vec<RoomRow>,pub catalog_key:Option<CatalogKey>,
 ready:Option<ready::Control>,cancel:Option<ready::Control>,join:Vec<Option<entry::Control>>,refresh:Option<entry::Control>,
}
fn text(raw:&[u8])->String{
 use windows::Win32::Globalization::{MultiByteToWideChar,MB_ERR_INVALID_CHARS};
 let mut wide=vec![0u16;raw.len()];let n=unsafe{MultiByteToWideChar(950,MB_ERR_INVALID_CHARS,raw,Some(&mut wide))};
 if n<=0{return "名稱無法顯示".into();}String::from_utf16_lossy(&wide[..n as usize])
}
fn state_label(state:Option<u8>)->&'static str{match state{Some(0)=>"未準備",Some(1)=>"已準備",Some(2)=>"商店中",_=>"等待狀態回包"}}
fn own_state(alias:&[u8],rows:&[members::Row])->Option<u8>{
 let mut found=rows.iter().filter(|r|r.participant.display_alias==alias);let row=found.next()?;
 if found.next().is_some(){return None;}row.last_state_receipt.as_ref().filter(|d|d.display_alias==alias).map(|d|d.state.wire())
}
fn visible_countdown(epoch:Option<u64>,room_id:u32,joined:Option<&settings::JoinedContext>,c:Option<&ready::Countdown>)->Option<u8>{
 c.filter(|c|!c.invalidated&&epoch==Some(c.epoch)&&room_id==c.room_id&&joined==Some(&c.joined)).and_then(|c|c.count)
}
impl Panel {
 pub fn capture(r:&Runtime)->Self{
  let mut p=Self::default();if r.page()!=UiPage::Menu{return p;}
  let room=r.java_room_entry_snapshot();
  if let Some(joined)=r.admit_java_room_settings_context().ok(){
   if let Some(book)=r.java_room_participants_snapshot().filter(|b|!b.joined_room_invalidated&&r.source_connection_epoch()==Some(b.epoch)&&b.room_id==room.room.as_ref().map_or(0,|r|r.id)&&b.ce_revision==Some(joined.ce_revision)){
    let current=room.room.as_ref().unwrap();p.display.room_id=Some(current.id);p.display.room_name=text(&current.name);
    p.display.members=book.rows.iter().map(|row|format!("{}：{}",text(&row.participant.display_alias),state_label(row.last_state_receipt.as_ref().map(|d|d.state.wire())))).collect();
    if let Some(alias)=r.qualified_singleton_self_alias(){p.display.own_d1_state=own_state(&alias,&book.rows);}
    let snapshot=r.java_room_ready_snapshot();p.display.countdown=visible_countdown(r.source_connection_epoch(),current.id,Some(&joined),snapshot.countdown.as_ref());
   }
  }
  p.ready=r.java_room_ready_control(ready::ParticipantIntent::Ready);p.cancel=r.java_room_ready_control(ready::ParticipantIntent::NotReady);
  p.refresh=r.java_room_entry_control(entry::Intent::RefreshList);
  if p.display.room_id.is_none(){
   if let Some(c)=r.java_room_catalog_snapshot().filter(|c|r.source_connection_epoch()==Some(c.epoch)&&c.origin==super::java_room_tail::CatalogOrigin::ChannelReceipt){
    p.catalog_key=Some((c.epoch,c.channel,c.admission_revision,c.revision));
    for row in c.rows {
     let password_required=row.attributes&4!=0;
     p.rooms.push(RoomRow{room_id:row.room_id,label:format!("{} · {} / {}{}",text(&row.name),row.display_users,row.capacity,if password_required{" · 需密碼"}else{""}),password_required});
     // This candidate joins observed public rooms only. Display never mints authority.
     p.join.push(if password_required{None}else{r.java_room_entry_control(entry::Intent::Join{room_id:row.room_id,password:vec![]})});
    }
   }
  }
  p
 }
 pub fn enabled(&self,op:Operation)->bool{match op{Operation::Ready=>self.ready.is_some(),Operation::Cancel=>self.cancel.is_some(),Operation::Join(i)=>self.join.get(i).is_some_and(Option::is_some),Operation::Refresh=>self.refresh.is_some()}}
 pub fn invoke(&self,r:&mut Runtime,op:Operation)->Result<()>{
  match op {
   Operation::Ready|Operation::Cancel=>{
    let token=match op{Operation::Ready=>&self.ready,_=>&self.cancel}.as_ref().context("目前無法操作準備狀態")?;
    r.handle_java_room_ready_control(token)
   },
   Operation::Join(i)=>{
    let c=r.java_room_catalog_snapshot().context("房間清單已失效")?;
    ensure!(self.catalog_key==Some((c.epoch,c.channel,c.admission_revision,c.revision)),"房間清單已變更，請重新選擇");
    let row=self.rooms.get(i).context("請選擇房間")?;
    ensure!(c.rows.iter().any(|current|current.room_id==row.room_id),"所選房間已移除");
    r.handle_java_room_entry_control(self.join.get(i).and_then(Option::as_ref).context("目前無法加入所選房間")?)
   },
   Operation::Refresh=>r.handle_java_room_entry_control(self.refresh.as_ref().context("目前無法重新取得清單")?),
  }
 }
 pub fn member_text(&self)->String{let mut rows=vec![format!("{} · 我的狀態：{}",self.display.room_name,state_label(self.display.own_d1_state))];rows.extend(self.display.members.iter().cloned());rows.join("\r\n")}
 pub fn countdown_text(&self)->String{self.display.countdown.map(|n|format!("倒數：{n}")).unwrap_or_default()}
}
/// Pins preserve historical AD368 layout provenance and the explicit 485F writer.
/// Called only by the explicit candidate Java-bootstrap entry, before connection.
pub fn configure(r:&mut Runtime)->Result<()>{
 use super::{login_bootstrap,java_room_departure as departure,java_room_tail as tail,java_room_catalog as catalog,java_room_self as own};
 use serde_json::json;
 let raw=|v:serde_json::Value|serde_json::to_vec(&v);
 r.configure_java_room_entry_adapter(entry::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_room_entry_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"contract_sha256":entry::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_participants_adapter(members::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_room_participants_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"contract_sha256":members::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_departure_adapter(departure::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_room_departure_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"route_sha256":departure::ROUTE_SHA,"native_sha256":departure::NATIVE_SHA,"commit_sha256":departure::COMMIT_SHA}))?)?)?;
 r.configure_java_room_tail_adapter(tail::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_room_tail_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"supplement_sha256":tail::SUPPLEMENT_SHA,"dynamic_sha256":tail::DYNAMIC_SHA,"native_sha256":tail::NATIVE_SHA}))?)?)?;
 r.configure_java_room_catalog_adapter(catalog::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_room_catalog_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"contract_sha256":catalog::CONTRACT_SHA,"lifecycle_sha256":catalog::LIFECYCLE_SHA,"comparison_sha256":catalog::COMPARISON_SHA,"publication_sha256":catalog::PUBLICATION_SHA}))?)?)?;
 r.configure_java_room_self_adapter(own::Adapter::parse(&raw(json!({"kind":"modern_synthetic_AD368_singleton_self_v1","producer_sha256":login_bootstrap::PRODUCER_SHA,"contract_sha256":own::CONTRACT_SHA}))?)?)?;
 r.configure_java_room_settings_adapter(settings::Adapter::parse(&raw(json!({"kind":"modern_synthetic_485F_room_settings_v1","producer_sha256":ready::PRODUCER_SHA,"network_sha256":super::java_room_settings_request::SETTINGS_485F_NETWORK_SHA,"layout_sha256":settings::LAYOUT_SHA,"native_sha256":settings::NATIVE_SHA}))?)?)?;
 r.configure_java_room_mode_map_adapter(settings::ModeMapAdapter::parse(&raw(json!({"kind":"modern_synthetic_485F_room_mode_map_v1","producer_sha256":ready::PRODUCER_SHA,"network_sha256":super::java_room_mode_request::NETWORK_SHA,"mode_step_sha256":super::java_room_mode_request::NATIVE_SHA,"layout_sha256":settings::LAYOUT_SHA,"native_sha256":settings::MODE_MAP_SHA,"scalar_boundary_sha256":settings::SCALAR_BOUNDARY_SHA}))?)?)?;
 r.configure_java_room_ready_adapter(ready::Adapter::parse(&raw(json!({"kind":"modern_synthetic_485F_participant_ready_v1","producer_sha256":ready::PRODUCER_SHA,"network_sha256":ready::NETWORK_SHA,"native_sha256":ready::NATIVE_SHA}))?)?)?;
 Ok(())
}

#[cfg(test)]mod r529_room_window_bindings {
 use super::*;
 fn row()->members::Row{
  let p=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003/r526-test-inputs.json");
  let v:serde_json::Value=serde_json::from_slice(&std::fs::read(p).unwrap()).unwrap();
  let s=v["host_CD"].as_str().unwrap();let bytes=s.as_bytes().chunks_exact(2).map(|h|u8::from_str_radix(std::str::from_utf8(h).unwrap(),16).unwrap()).collect::<Vec<_>>();
  let members::Packet::Participant(participant)=members::decode(&bytes).unwrap()else{panic!("participant fixture")};
  let mut body=vec![209];body.extend(&participant.display_alias);body.extend([0,0]);
  let members::Packet::State(d)=members::decode(&body).unwrap()else{panic!("D1 fixture")};
  members::Row{participant,state:members::MemberState::Ready,state_source:members::StateSource::CdArrival,last_arrival:2,last_cd_arrival:1,last_state_receipt:Some(d)}
 }
 #[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn display_uses_last_d1_not_initial_or_queued_intent(){
  let mut r=row();let alias=r.participant.display_alias.clone();
  assert_eq!(r.state,members::MemberState::Ready);assert_eq!(own_state(&alias,&[r.clone()]),Some(0));
  r.last_state_receipt.as_mut().unwrap().state=members::MemberState::Ready;
  assert_eq!(own_state(&alias,&[r.clone()]),Some(1));
  let p=Panel{display:Display{own_d1_state:Some(1),countdown:None,..Display::default()},..Panel::default()};
  assert!(p.member_text().contains("已準備"));assert_eq!(p.countdown_text(),""); // E100 does not clear D1.
  r.last_state_receipt=None;assert_eq!(own_state(&alias,&[r]),None);
 }
 #[test]#[ignore = "requires: local evidence roots (snapshot JSON holds absolute D:\\ paths + SHA pins; only runnable on the original machine)"] fn ambiguous_alias_and_absent_receipt_remain_unknown(){
  let r=row();let alias=r.participant.display_alias.clone();assert_eq!(own_state(&alias,&[r.clone(),r.clone()]),None);
  assert_eq!(own_state(b"unrelated",&[r]),None);assert_eq!(state_label(None),"等待狀態回包");
 }
 #[test]fn countdown_displays_received_byte_and_clears_for_all_retired_contexts(){
  let joined=settings::JoinedContext{ce_revision:1,source:b"fixture".to_vec(),source_revision:2,roster_revision:3,selected_slot:0,selected_alias:b"fixture".to_vec(),navigation_revision:4,channel:0};
  let mut c=ready::Countdown{epoch:1,room_id:9,joined:joined.clone(),revision:1,count:Some(3),receipts:vec![3],invalidated:false};
  for count in [3,2,1,255]{c.count=Some(count);assert_eq!(visible_countdown(Some(1),9,Some(&joined),Some(&c)),Some(count));}
  assert_eq!(visible_countdown(None,9,Some(&joined),Some(&c)),None);
  assert_eq!(visible_countdown(Some(2),9,Some(&joined),Some(&c)),None);
  assert_eq!(visible_countdown(Some(1),10,Some(&joined),Some(&c)),None);
  let mut changed=joined.clone();changed.ce_revision+=1;assert_eq!(visible_countdown(Some(1),9,Some(&changed),Some(&c)),None);
  changed=joined.clone();changed.navigation_revision+=1;assert_eq!(visible_countdown(Some(1),9,Some(&changed),Some(&c)),None);
  c.invalidated=true;assert_eq!(visible_countdown(Some(1),9,Some(&joined),Some(&c)),None);
  c.invalidated=false;c.count=None;c.receipts.push(0);assert_eq!(visible_countdown(Some(1),9,Some(&joined),Some(&c)),None);
 }
 #[test]fn unavailable_callbacks_and_close_use_actual_Runtime_guards(){
  use super::super::Action;
  use crate::modern_lito::{native_multisector_resources::Inputs,native_resources::BytePin};
  use sha2::{Digest,Sha256};
  use std::{time::{Instant,Duration},thread};
  let root=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let manifest=root.join("logs/continuation-20261003/r486-map-binding.json");let raw=std::fs::read(&manifest).unwrap();
  let mut r=Runtime::start(Inputs{project:root,manifest,manifest_pin:BytePin{bytes:raw.len()as u64,sha256:Sha256::digest(raw).into()}}).unwrap();
  let end=Instant::now()+Duration::from_secs(20);while r.page()!=UiPage::Menu{assert!(Instant::now()<end);r.poll();thread::sleep(Duration::from_millis(2));}
  configure(&mut r).unwrap();
  let p=Panel::capture(&r);let before=r.presentation_key();
  for op in [Operation::Ready,Operation::Cancel,Operation::Join(0),Operation::Refresh]{assert!(!p.enabled(op));assert!(p.invoke(&mut r,op).is_err());assert_eq!(r.presentation_key(),before);}
  let epoch=r.begin_source_connection_candidate().unwrap();r.disconnect_source_connection_candidate(&epoch).unwrap();
  assert_eq!(Panel::capture(&r).display.countdown,None);assert!(p.invoke(&mut r,Operation::Ready).is_err());
  r.handle(Action::Close).unwrap();assert_eq!(Panel::capture(&r).display,Display::default());assert!(p.invoke(&mut r,Operation::Cancel).is_err());
  let end=Instant::now()+Duration::from_secs(20);while r.page()!=UiPage::Closed{assert!(Instant::now()<end);r.poll();thread::sleep(Duration::from_millis(2));}
  assert_eq!(r.player_fixture_snapshot().workers_started,0);
 }
}
