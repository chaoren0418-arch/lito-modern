//! Ordinary4928C0 character-select cs(201,received record name) counterpart.
//! Raw-body preparation is crate-private; synthetic login uses the same bounded Sender.
//! Current registry/environment authority and special send queue remain OPEN.
use anyhow::{ensure,Context,Result};
use std::sync::{Arc,atomic::{AtomicBool,AtomicUsize,Ordering}};
use super::source_stream::{advance_counter,CounterStep};
#[derive(Clone,Copy,Debug)]pub enum Intent { SelectCharacter{slot:usize},ReturnLogin,CompatibilityLogin }
#[derive(Clone)]pub struct Ticket { owner:Arc<()>,serial:u64 }
impl Ticket {pub fn serial(&self)->u64{self.serial}}
pub struct Reservation {pub ticket:Ticket,pub counter_step:CounterStep,pub frame_bytes:usize}
#[derive(Clone,Copy,Debug)]pub enum WriteOutcome { Written(usize),Failed,WouldBlock }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Terminal { ZeroWrite,OverWrite,Failed,UnsupportedSpecialWrite }
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Snapshot {pub counter:Option<u64>,pub pending:Option<(u64,usize,usize)>}
#[derive(Clone,Debug,PartialEq,Eq)]pub enum Progress { Partial{offset:usize,remaining:usize},Complete,Retired{reason:Terminal,before:Snapshot} }
#[derive(Clone)]pub struct WriteView {frame:Arc<[u8]>,offset:usize,life:Arc<AtomicBool>,position:Arc<AtomicUsize>}
impl WriteView {
 pub fn bytes(&self)->Result<&[u8]>{ensure!(self.life.load(Ordering::Acquire)&&self.position.load(Ordering::Acquire)==self.offset,"retired or superseded source write view");Ok(&self.frame[self.offset..])}
 pub fn offset(&self)->usize{self.offset}
}
struct Pending {ticket:Ticket,frame:Arc<[u8]>,offset:usize,life:Arc<AtomicBool>,position:Arc<AtomicUsize>}
impl Drop for Pending {fn drop(&mut self){self.life.store(false,Ordering::Release);}}
pub(crate) struct Prepared {counter:u64,next:u64,pending:Pending,pub reservation:Reservation}
pub(crate) struct Sender {owner:Arc<()>,counter:Option<u64>,next:u64,pending:Option<Pending>}
impl Default for Sender {fn default()->Self{Self{owner:Arc::new(()),counter:None,next:1,pending:None}}}
impl Sender {
 pub(crate) fn seed_from_source(&mut self,counter:u64)->Result<()>{ensure!(self.counter.is_none()&&self.pending.is_none(),"source send seed already installed");self.counter=Some(counter);Ok(())}
 pub(crate) fn snapshot(&self)->Snapshot{Snapshot{counter:self.counter,pending:self.pending.as_ref().map(|p|(p.ticket.serial,p.frame.len(),p.offset))}}
 pub(crate) fn pending_ticket(&self)->Option<Ticket>{self.pending.as_ref().map(|p|p.ticket.clone())}
 pub(crate) fn prepare_character(&self,name:&[u8])->Result<Prepared>{
  ensure!(name.len()<28&&!name.contains(&0),"selected source name exceeds NativeNUL capacity");let mut body=Vec::with_capacity(name.len()+2);body.push(201);body.extend_from_slice(name);body.push(0);self.prepare_body(body)
 }
 pub(crate) fn prepare_room_entry(&self,body:Vec<u8>)->Result<Prepared>{ensure!(matches!(body.first(),Some(204|206|207))&&body.len()<=114,"bounded ordinary room request required");self.prepare_body(body)}
    pub(crate) fn prepare_extended_ordinary_room_settings(&self,input:&super::java_room_settings_request::Inputs,tail:&super::java_room_settings_request::Extension)->Result<Prepared> {
        self.prepare_body(super::java_room_settings_request::encode_extended(input,tail)?)
    }
 pub(crate) fn prepare_ordinary_room_settings(&self,input:&super::java_room_settings_request::Inputs)->Result<Prepared>{self.prepare_body(super::java_room_settings_request::encode(input)?)}
 pub(crate) fn prepare_participant_ready(&self,state:u8)->Result<Prepared>{ensure!(state<=2,"participant state must be0/1/2");self.prepare_body(vec![210,state])}
 pub(crate) fn prepare_ordinary_room_mode_step(&self,direction:u8)->Result<Prepared>{ensure!(matches!(direction,1|2),"relative direction must be NEXT1/PREVIOUS2");self.prepare_body(vec![209,2,direction])}
 pub(crate) fn prepare_ordinary_room_map(&self,map:u16)->Result<Prepared>{ensure!(matches!(map,509|514|65535),"map candidate outside bounded509/514/FFFF");let mut body=vec![209,3];body.extend(map.to_le_bytes());self.prepare_body(body)}
 pub(crate) fn prepare_exit_d0(&self)->Result<Prepared>{self.prepare_body(vec![208])}
 pub(crate) fn prepare_return_login(&self)->Result<Prepared>{self.prepare_body(vec![15])}
 ///48F580 cs/C8 uses the source-directory name, never the character alias.
 pub(crate) fn prepare_source_choice(&self,name:&[u8])->Result<Prepared>{ensure!(!name.is_empty()&&name.len()<64&&!name.contains(&0),"source CString outside row capacity");let mut body=vec![200];body.extend_from_slice(name);body.push(0);self.prepare_body(body)}
 ///48F500/48F550 c/E2 continuation; no skill-owner readiness assertion.
 pub(crate) fn prepare_news_continue(&self)->Result<Prepared>{self.prepare_body(vec![226])}
 ///497020 BSS64AFB8=0 ordinary cc202,100, not a page40 user selection.
 pub(crate) fn prepare_automatic_lobby(&self)->Result<Prepared>{self.prepare_body(vec![202,100])}
 pub(crate) fn prepare_notice_d9(&self)->Result<Prepared>{self.prepare_body(vec![217])}
 pub(crate) fn prepare_roster_cb(&self)->Result<Prepared>{self.prepare_body(vec![203])}
 pub(crate) fn prepare_channel(&self,channel:u8)->Result<Prepared>{ensure!(channel<4||channel==100,"source channel index unavailable");self.prepare_body(vec![202,channel])}
 ///Crateprivatefixture producers serializeknownsourceformats;notpublicrawbodyAPI.
 pub(crate) fn prepare_fixture_body(&self,body:Vec<u8>)->Result<Prepared>{ensure!((1..=155).contains(&body.len()),"fixturebodyoutsidecandidatebound");self.prepare_body(body)}
 fn prepare_body(&self,mut body:Vec<u8>)->Result<Prepared>{
  ensure!(self.pending.is_none(),"source write still pending");let before=self.counter.context("source1/d send seed absent")?;let next=self.next.checked_add(1).context("source write serial exhausted")?;
  let after=advance_counter(before,body.len())?;
  crate::modern_lito::native_wire::encrypt_in_place(before,&mut body);
  let mut frame=((body.len()+2)as u16).to_le_bytes().to_vec();frame.extend(body);
  let ticket=Ticket{owner:self.owner.clone(),serial:self.next};let reservation=Reservation{ticket:ticket.clone(),counter_step:CounterStep{before,after},frame_bytes:frame.len()};
  Ok(Prepared{counter:after,next,pending:Pending{ticket,frame:frame.into(),offset:0,life:Arc::new(AtomicBool::new(true)),position:Arc::new(AtomicUsize::new(0))},reservation})
 }
 pub(crate) fn commit(&mut self,prepared:Prepared)->Reservation{self.counter=Some(prepared.counter);self.next=prepared.next;self.pending=Some(prepared.pending);prepared.reservation}
 fn pending(&self,ticket:&Ticket)->Result<&Pending>{let p=self.pending.as_ref().context("source write is not pending")?;ensure!(Arc::ptr_eq(&self.owner,&ticket.owner)&&p.ticket.serial==ticket.serial,"retired or foreign source write ticket");Ok(p)}
 pub(crate) fn view(&self,ticket:&Ticket)->Result<WriteView>{let p=self.pending(ticket)?;Ok(WriteView{frame:p.frame.clone(),offset:p.offset,life:p.life.clone(),position:p.position.clone()})}
 pub(crate) fn report(&mut self,ticket:&Ticket,expected_offset:usize,outcome:WriteOutcome)->Result<Progress>{
  let p=self.pending(ticket)?;ensure!(p.offset==expected_offset,"stale source write offset");let remaining=p.frame.len()-p.offset;
  let terminal=match outcome {WriteOutcome::Written(0)=>Some(Terminal::ZeroWrite),WriteOutcome::Written(n)if n>remaining=>Some(Terminal::OverWrite),WriteOutcome::Failed=>Some(Terminal::Failed),WriteOutcome::WouldBlock=>Some(Terminal::UnsupportedSpecialWrite),_=>None};
  if let Some(reason)=terminal{return Ok(Progress::Retired{reason,before:self.snapshot()});}
  let WriteOutcome::Written(n)=outcome else{unreachable!()};
  let p=self.pending.as_mut().unwrap();p.offset+=n;p.position.store(p.offset,Ordering::Release);
  if p.offset==p.frame.len(){self.pending.take();Ok(Progress::Complete)}else{Ok(Progress::Partial{offset:p.offset,remaining:p.frame.len()-p.offset})}
 }
}
