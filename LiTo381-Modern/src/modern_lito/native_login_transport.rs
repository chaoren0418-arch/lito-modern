//! Candidate-only actualloopback IO for explicitly syntheticfixture profiles.
//! No existingprovider/authservice/Java/SQL/remoteendpoint route.
use super::{Runtime,UiPage,SourceEpoch,login_form,login_wire,source_send,source_stream,StreamBatch};
use anyhow::{ensure,Context,Result};
use std::{net::{TcpStream,SocketAddr,SocketAddrV4},io::{Read,Write,ErrorKind},time::Duration};
#[derive(Clone,Debug,PartialEq,Eq)]pub struct Snapshot {pub fixture:String,pub local:SocketAddrV4,pub peer:SocketAddrV4,pub compatibility_reserved:bool,pub compatibility_complete:bool,pub queued_rx_bytes:usize}
pub struct ReadStep {pub socket_bytes:usize,pub batch:Option<StreamBatch>,pub eof:bool}
pub(crate) struct Transport {stream:Option<TcpStream>,environment:login_wire::FixtureEnvironment,local:SocketAddrV4,peer:SocketAddrV4,compatibility:Option<(u64,bool)>,pub(crate) queued:Vec<u8>}
impl Transport {
 pub(crate) fn take_stream(&mut self)->Result<TcpStream>{self.stream.take().context("socketalreadyownedbyworker")}
 pub(crate) fn connect(peer:SocketAddr,environment:login_wire::FixtureEnvironment)->Result<Self>{ensure!(peer.is_ipv4()&&peer.ip().is_loopback(),"syntheticloopbackendpoint only");let stream=TcpStream::connect_timeout(&peer,Duration::from_secs(2))?;stream.set_read_timeout(Some(Duration::from_millis(200)))?;stream.set_write_timeout(Some(Duration::from_secs(2)))?;stream.set_nodelay(true)?;let SocketAddr::V4(local)=stream.local_addr()?else{anyhow::bail!("fixturelocalIPv4required")};let SocketAddr::V4(actual)=stream.peer_addr()?else{anyhow::bail!("fixturepeerIPv4required")};ensure!(local.ip().is_loopback()&&actual.ip().is_loopback(),"actualfixture addressesmustbe loopback");Ok(Self{stream:Some(stream),environment,local,peer:actual,compatibility:None,queued:Vec::new()})}
 fn snapshot(&self)->Snapshot{Snapshot{fixture:self.environment.provenance().into(),local:self.local,peer:self.peer,compatibility_reserved:self.compatibility.is_some(),compatibility_complete:self.compatibility.is_some_and(|(_,done)|done),queued_rx_bytes:self.queued.len()}}
 pub(crate) fn write_report(&mut self,ticket:u64,p:&source_send::Progress){if let Some((serial,done))=&mut self.compatibility{if *serial==ticket&&matches!(p,source_send::Progress::Complete){*done=true;}}}
}
impl Runtime {
 pub fn source_loopback_snapshot(&self)->Option<Snapshot>{self.login_transport.as_ref().map(Transport::snapshot)}
 pub fn connect_source_loopback_fixture(&mut self,peer:SocketAddr,environment:login_wire::FixtureEnvironment)->Result<SourceEpoch>{ensure!(self.page==UiPage::Menu,"fixturetransportunavailable");let transport=Transport::connect(peer,environment)?;let epoch=self.begin_source_connection_candidate()?;self.login_transport=Some(transport);Ok(epoch)}
 ///1/dseed alreadyreceived. ValuesfromexplicitFixtureEnvironment,notPE/archived/currentOS.
 pub fn prepare_source_compatibility_fixture(&mut self,epoch:&SourceEpoch)->Result<source_send::Reservation>{self.source_scope.admit_epoch(epoch)?;let t=self.login_transport.as_ref().context("ownedfixturetransportrequired")?;ensure!(t.compatibility.is_none()&&self.source_actor.snapshot().version.is_none(),"compatibilitysendalreadyreservedor58alreadyestablished");let prepared=self.source_send.prepare_fixture_body(t.environment.compatibility_body().to_vec())?;let reservation=self.source_send.commit(prepared);self.login_transport.as_mut().unwrap().compatibility=Some((reservation.ticket.serial(),false));Ok(reservation)}
 ///Sameownedformstage ->typedcssd/cssds ->sameSender. No externalrawbody/authauthoritysetter.
 pub fn prepare_source_login_fixture(&mut self,epoch:&SourceEpoch,control:&login_form::Control)->Result<source_send::Reservation>{
  self.source_scope.admit_epoch(epoch)?;let t=self.login_transport.as_ref().context("ownedsyntheticloopbacktransportrequired")?;ensure!(t.compatibility.is_some_and(|(_,done)|done)&&self.source_actor.snapshot().version.is_some(),"sourcecompatibilitywriteandactual58versionrequired");
  let credentials=self.login_form.fixture_credentials(control)?;
  use sha2::Digest;let account:[u8;32]=sha2::Sha256::digest(&*credentials.account).into();
  let resuming=self.java_return.key.phase==super::java_return::Phase::Authenticating;
  if resuming {ensure!(self.java_return.unexpired()&&self.java_return.account==Some(account),"fresh GAME account differs or retained handoff expired");}
  let body=self.login_form.prepare_fixture_body(control,t.local.ip().octets(),&t.environment)?;let prepared=self.source_send.prepare_fixture_body(body.to_vec())?;let serial=prepared.reservation.ticket.serial();
  if self.bootstrap_adapter.is_some(){ensure!(self.login_bootstrap.key.stage==super::login_bootstrap::Stage::Idle,"bootstrap login already staged");self.java_return.account=Some(account);self.login_bootstrap.credentials=Some(credentials);self.login_bootstrap.key.credentials_pending=true;
   self.login_bootstrap.key.stage=if resuming{super::login_bootstrap::Stage::SendingGameLogin}else{super::login_bootstrap::Stage::AwaitLoginCheck};
   if resuming {self.login_bootstrap.key.write_serial=Some(serial);}}
  self.login_form.sending(control,serial)?;let reservation=self.source_send.commit(prepared);self.status="登入要求等待傳送".into();Ok(reservation)
 }
 pub(crate) fn prepare_game_bootstrap_login(&mut self)->Result<()> {
  use super::login_bootstrap::Stage;
  ensure!(self.bootstrap_adapter.is_some()&&self.login_bootstrap.key.stage==Stage::GameConnecting,"explicit GAME bootstrap adapter required");
  let t=self.login_transport.as_ref().context("owned GAME transport absent")?;ensure!(t.compatibility.is_some_and(|(_,done)|done)&&self.source_actor.snapshot().version.is_some(),"fresh GAME compatibility/version required");
  let c=self.login_bootstrap.credentials.as_ref().context("owned GAME credential lease absent")?;
  use sha2::Digest;self.java_return.account=Some(sha2::Sha256::digest(&*c.account).into());
  let body=login_wire::serialize_fixture_login(&c.account,&c.password,t.local.ip().octets(),&t.environment)?;
  let prepared=self.source_send.prepare_fixture_body(body.to_vec())?;let serial=prepared.reservation.ticket.serial();
  self.login_bootstrap.key.stage=Stage::SendingGameLogin;self.login_bootstrap.key.write_serial=Some(serial);self.source_send.commit(prepared);Ok(())
 }
 ///BoundedactualTcpStreamwrite. cap forcesnewpartialwrite tests;nativecounterreservedONCE.
 pub fn write_source_loopback_step(&mut self,epoch:&SourceEpoch,cap:usize)->Result<source_send::Progress>{
  self.source_scope.admit_epoch(epoch)?;ensure!((1..=157).contains(&cap),"fixturewritecapoutsideframebound");let ticket=self.source_send.pending_ticket().context("nosourcewritepending")?;let view=self.source_send.view(&ticket)?;let bytes=view.bytes()?;let result=self.login_transport.as_mut().context("ownedfixturetransportrequired")?.stream.as_mut().context("synchronouswriteunavailableforworkerownedsocket")?.write(&bytes[..bytes.len().min(cap)]);let outcome=match result{Ok(n)=>source_send::WriteOutcome::Written(n),Err(_)=>source_send::WriteOutcome::Failed};self.report_source_write_candidate(epoch,&ticket,view.offset(),outcome)
 }
 ///Retainunconsumed8-eventbatchtail. EOF/IOerror retireownedepoch;timeout isNoData.
 pub fn read_source_loopback_step(&mut self,epoch:&SourceEpoch,cap:usize)->Result<ReadStep>{
  self.source_scope.admit_epoch(epoch)?;ensure!((1..=1024).contains(&cap),"fixture readcapoutsidebound");let t=self.login_transport.as_mut().context("ownedfixturetransportrequired")?;let mut n=0;
  if t.queued.is_empty(){let mut raw=vec![0;cap];match t.stream.as_mut().context("synchronousreadunavailableforworkerownedsocket")?.read(&mut raw){Ok(0)=>{self.end_source_stream_candidate(epoch)?;return Ok(ReadStep{socket_bytes:0,batch:None,eof:true});},Ok(count)=>{n=count;raw.truncate(count);t.queued=raw;},Err(e)if matches!(e.kind(),ErrorKind::WouldBlock|ErrorKind::TimedOut)=>return Ok(ReadStep{socket_bytes:0,batch:None,eof:false}),Err(e)=>{self.disconnect_source_connection_candidate(epoch)?;return Err(e.into());}}}
  let data=std::mem::take(&mut t.queued);let batch=self.feed_source_stream_candidate(epoch,&data)?;if let Some(t)=&mut self.login_transport{t.queued.extend_from_slice(&data[batch.consumed..]);}
  Ok(ReadStep{socket_bytes:n,batch:Some(batch),eof:false})
 }
}
