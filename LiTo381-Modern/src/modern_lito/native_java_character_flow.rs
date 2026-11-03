//! Explicit AD368 directory-selection/channel interoperability receipts.
//! Native inventory/lobby owners and skills initialization are not installed.
use anyhow::{ensure,Context,Result};
use super::{Runtime,SourceDecodedPacket,entry_navigation::{self,Scene,Reply}};
#[derive(Clone,Copy,Debug,Default,PartialEq,Eq)]
pub enum Stage {#[default] Idle,Directory,AwaitCharacterResult,AwaitInventory,ChooseChannel,ResumeChooseChannel,AwaitChannelResult,AwaitChannelInfo,ChannelReceipts,Refused}
#[derive(Clone,Debug,PartialEq,Eq)]
pub struct InventoryItem {pub item_wire_id:u32,pub use_type:u8,pub charge:u8,pub icon:u16,pub blessing:u8,pub count:u32,pub identified:u8,pub name:Vec<u8>,pub detail:Vec<u8>}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct ChannelInfo {pub channel:u8,pub users:u16,pub capacity:u16}
#[derive(Clone,Debug,Default,PartialEq,Eq)]
pub struct Key {pub stage:Stage,pub selected_name:Option<Vec<u8>>,pub character_result:Option<u8>,pub inventory:Option<Vec<InventoryItem>>,pub requested_channel:Option<u8>,pub channel_result:Option<u8>,pub channel_info:Option<ChannelInfo>,pub status_fields:super::status_family::Projection,pub mana:Option<super::status_receipt::ManaPoint>,pub status:Option<super::status_receipt::Status>,pub refusal:Option<(u8,u8)>}
#[derive(Default)]pub(crate) struct State {pub(crate) key:Key}
pub enum Packet {Inventory(Vec<InventoryItem>),Channel(ChannelInfo)}
pub fn decode(body:&[u8])->Result<Packet>{
 ensure!(body.len()<=super::source_connection::MAX_DECODED_BODY,"inventory/channel receipt exceeds frame bound");
 if body.first()==Some(&202){ensure!(body.len()==6&&body[1]<4,"normal S_CA exact cchh required");return Ok(Packet::Channel(ChannelInfo{channel:body[1],users:u16::from_le_bytes(body[2..4].try_into().unwrap()),capacity:u16::from_le_bytes(body[4..6].try_into().unwrap())}));}
 ensure!(body.first()==Some(&124)&&body.len()>=2,"S7C requires unsigned c count");let mut at=2;let count=usize::from(body[1]);ensure!(count<=(body.len()-at)/16,"inventory rows cannot fit body");let mut rows=Vec::new();
 for _ in 0..count {
  let f=body.get(at..at+14).context("short dcchcdc inventory fields")?;at+=14;
  let n=body[at..].iter().take(256).position(|b|*b==0).context("inventory name outside Native256-byte CString")?;let name=body[at..at+n].to_vec();at+=n+1;
  let n=usize::from(*body.get(at).context("inventory detail count absent")?);at+=1;let detail=body.get(at..at+n).context("short inventory detail")?.to_vec();at+=n;
  let row=InventoryItem{item_wire_id:u32::from_le_bytes(f[..4].try_into().unwrap()),use_type:f[4],charge:f[5],icon:u16::from_le_bytes(f[6..8].try_into().unwrap()),blessing:f[8],count:u32::from_le_bytes(f[9..13].try_into().unwrap()),identified:f[13],name,detail};
  ensure!(row.item_wire_id!=0&&row.blessing<4&&row.count!=0,"invalid AD368 inventory row");rows.push(row);
 }
 ensure!(at==body.len(),"inventory trailing bytes");Ok(Packet::Inventory(rows))
}
#[derive(Clone,Debug)]pub enum Change {StatusFieldsReceived{opcode:u8},ManaReceived,StatusReceived,CharacterAccepted,InventoryReceived{count:usize},ChannelAccepted,ChannelInfoReceived,Refused{opcode:u8,status:u8}}
impl Runtime {
 pub fn consume_java_status_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{ensure!(event.body().first()==Some(&12),"typed status candidate requires S0C");self.consume_java_character_event(event)}
 pub fn consume_java_status_family_candidate(&mut self,event:&SourceDecodedPacket)->Result<Change>{ensure!(matches!(event.body().first(),Some(80|81|82|86)),"typed status family candidate requires S50/S51/S52/S56");self.consume_java_character_event(event)}
 pub fn java_character_snapshot(&self)->Key{self.java_character.key.clone()}
 pub fn last_java_character_checkpoint(&self)->Option<Key>{self.last_java_checkpoint.clone()}
 pub(crate) fn consume_java_character_event(&mut self,event:&SourceDecodedPacket)->Result<Change>{
  self.source_scope.admit(event)?;ensure!(self.bootstrap_adapter.is_some(),"explicit AD368 character adapter required");
  let stage=self.java_character.key.stage;
  match event.body()[0]{
   80|81|82|86=>{ensure!(stage==Stage::ChannelReceipts,"status-family receipt outside qualified channel receipts");let parsed=super::status_family::decode(event.body())?;let next=self.java_character.key.status_fields.with_packet(parsed)?;self.java_character.key.status_fields=next;Ok(Change::StatusFieldsReceived{opcode:event.body()[0]})},
   77=>{ensure!(stage==Stage::ChannelReceipts,"S_MANA_POINT outside channel receipts");let mana=super::status_receipt::decode_mana(event.body())?;self.java_character.key.mana=Some(mana);Ok(Change::ManaReceived)},
   12=>{ensure!(stage==Stage::ChannelReceipts,"S_STATUS outside qualified channel receipts");let status=super::status_receipt::decode(event.body())?;let next=self.java_character.key.status_fields.with_full_status(status)?;self.java_character.key.status_fields=next;self.java_character.key.status=Some(status);self.status="已收到狀態資料，後續狀態仍待驗證".into();Ok(Change::StatusReceived)},
   200|201=>{
    ensure!(self.entry_navigation.key.write_complete&&self.source_send.snapshot().pending.is_none(),"reply before completed current request");
    match entry_navigation::decode(event.body())?{
     Reply::Character{status}=>{
      ensure!(stage==Stage::AwaitCharacterResult,"S_C8 outside character request");self.java_character.key.character_result=Some(status);
      if status!=0{return self.refuse_java_character(200,status);}
      self.java_character.key.stage=Stage::AwaitInventory;self.status="角色已接受，等待物品清單".into();Ok(Change::CharacterAccepted)
     },
     Reply::Lobby{status,channel,fields}=>{
      ensure!(stage==Stage::AwaitChannelResult&&Some(channel)==self.java_character.key.requested_channel,"S_C9 outside matching channel request");self.java_character.key.channel_result=Some(status);
      if status!=0{return self.refuse_java_character(201,status);}
      let f=fields.context("accepted channel tail absent")?;ensure!(f.at_64ad28==0&&f.at_64ad24==0&&f.at_64ad20==0,"AD368 normal channel reserved tail must be zero");
      self.java_character.key.stage=Stage::AwaitChannelInfo;self.status="頻道已接受，等待頻道人數".into();Ok(Change::ChannelAccepted)
     },
    }
   },
   124=>{
    ensure!(stage==Stage::AwaitInventory&&self.java_character.key.character_result==Some(0),"S7C requires accepted current C9");let Packet::Inventory(rows)=decode(event.body())? else{unreachable!()};let count=rows.len();
    self.java_character.key.inventory=Some(rows);self.java_character.key.stage=Stage::ChooseChannel;
    self.entry_navigation.key.revision=self.entry_navigation.key.revision.checked_add(1).context("channel control revision exhausted")?;self.entry_navigation.key.scene=Scene::Channel;self.entry_navigation.key.pending_write=None;self.entry_navigation.key.write_complete=false;
    self.status="請選擇頻道".into();Ok(Change::InventoryReceived{count})
   },
   202=>{
    ensure!(stage==Stage::AwaitChannelInfo&&self.java_character.key.channel_result==Some(0),"S_CA requires accepted matching channel request");let Packet::Channel(info)=decode(event.body())? else{unreachable!()};ensure!(Some(info.channel)==self.java_character.key.requested_channel,"channel-info mismatch");
    let channel_revision=self.entry_navigation.key.revision.checked_add(1).context("channel revision exhausted")?;
    let catalog=self.java_tail.prepare_channel(&self.source_scope.current_epoch()?,info.channel,channel_revision)?;
    if self.java_return.key.phase==super::java_return::Phase::ResumeChannel {ensure!(self.java_return.unexpired()&&!self.java_return.key.one_shot_consumed,"one-shot channel return expired or consumed");self.java_return.key.one_shot_consumed=true;self.java_return.key.phase=super::java_return::Phase::Idle;self.java_return.expires=None;}
    self.java_tail.catalog=catalog;self.java_character.key.channel_info=Some(info);self.java_character.key.stage=Stage::ChannelReceipts;
    // Only receipts are qualified. Later status/owner packets still pass the
    // ordinary strict dispatcher and retain their first unsupported boundary.
    self.entry_navigation.key.revision=channel_revision;self.entry_navigation.key.scene=Scene::Lobby;self.entry_navigation.key.pending_write=None;self.entry_navigation.key.write_complete=false;self.entry_navigation.key.channel=Some(info.channel);
    self.status="已收到頻道資料，等待後續狀態".into();Ok(Change::ChannelInfoReceived)
   },
   _=>anyhow::bail!("not a character/channel interoperability receipt"),
  }
 }
 fn refuse_java_character(&mut self,opcode:u8,status:u8)->Result<Change>{
  ensure!(match opcode{200=>matches!(status,1|7|8),201=>matches!(status,1|10),_=>false},"unknown AD368 refusal result");
  self.java_character.key.refusal=Some((opcode,status));self.java_character.key.stage=Stage::Refused;let epoch=self.source_scope.current_epoch()?;self.disconnect_source_connection_candidate(&epoch)?;self.entry_navigation.key.scene=Scene::ReconnectRequired;self.status="選取未成功，請重新連線".into();Ok(Change::Refused{opcode,status})
 }
}
