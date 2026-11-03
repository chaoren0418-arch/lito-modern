//! Explicit supervisor-pinned Backend485F candidate. No process lifecycle authority.
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{cell::Cell,fs,io::{Read,Write},path::{Path,PathBuf},time::{Duration,Instant}};
use super::native_city_runtime::{Runtime,Action,login_events::Backend485fProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_room_ready as ready_flow,room_player::{self,Panel,Operation}};
#[path="native_backend_485f_provider.rs"]mod provider;
#[path="native_backend_485f_status_reader.rs"]mod status_reader;
#[path="native_backend_485f_evidence.rs"]pub mod evidence;
struct ConsumerSpec {fresh_run_id:String,endpoint:String,credentials_file:PathBuf,source_raw_hex:String,guest_alias_raw_hex:String,host_alias_raw_hex:String,channel:u8,room_id:u32,read_cap:usize,write_cap:usize}
struct Provider {consumer:ConsumerSpec,status_file:PathBuf,run_root:PathBuf,helper_pid:u32,port:u16,lease_end_epoch_ms:u64,status_poll_started:Cell<Option<Instant>>,identities:Vec<(u32,String)>}
fn digest(raw:&[u8])->String{Sha256::digest(raw).iter().map(|v|format!("{v:02X}")).collect()}
fn read_bound(path:&Path,bound:u64)->Result<Vec<u8>>{ensure!(path.is_absolute(),"absolute provider path required");let mut raw=vec![];fs::File::open(path)?.take(bound+1).read_to_end(&mut raw)?;ensure!(raw.len()as u64<=bound,"input exceeds bound");Ok(raw)}
fn hex_bytes(raw:&[u8])->String{raw.iter().map(|v|format!("{v:02x}")).collect()}
fn profile(p:&Provider,sha:&str)->Result<Backend485fProfile>{Backend485fProfile::parse(&serde_json::to_vec(&json!({"kind":"modern_backend_485F_loopback_v1","label":"BACKEND485F:R530_SUPERVISOR_PINNED_PLAYER_CANDIDATE","endpoint":p.consumer.endpoint,"producer_sha256":ready_flow::PRODUCER_SHA,"provider_acceptance_sha256":sha,"read_cap":p.consumer.read_cap,"write_cap":p.consumer.write_cap}))?)}
fn save_new(path:&Path,v:&Value)->Result<()>{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(path)?;f.write_all(&serde_json::to_vec_pretty(v)?)?;Ok(())}
/// Private fields prevent callers from constructing an accepted profile by relabeling a fixture.
pub struct Accepted {ready:PathBuf,sha:String,p:Provider,output:PathBuf,attempted:bool,healthy_at:Option<Instant>,armed:bool,ready_used:bool,first_failure:Option<String>,evidence:evidence::Recorder}
impl Accepted {
 pub fn load(ready:&Path,sha:&str)->Result<Self>{
  let p=provider::validate(&read_bound(ready,65536)?,sha)?;
  ensure!(ready==p.run_root.join("ready-contract.json"),"exact fresh READY path required");
  let output=Path::new(env!("CARGO_MANIFEST_DIR")).join("logs/continuation-20261003").join(format!("r532-player-{}",p.consumer.fresh_run_id));
  fs::create_dir(&output).context("run claim exists; no reuse/retry")?;
  save_new(&output.join("accepted.json"),&json!({"fresh_run_id":p.consumer.fresh_run_id,"ready_path":ready,"supervisor_sha256":sha.to_ascii_uppercase(),"no_credentials_read":true,"server_lifecycle_owner":true}))?;
  Ok(Self{ready:ready.into(),sha:sha.to_ascii_uppercase(),p,output,attempted:false,healthy_at:Some(Instant::now()),armed:false,ready_used:false,first_failure:None,evidence:evidence::Recorder::new(b"Fx485Guest(Fx485Moon)")})
 }
 pub(super) fn configure(&self,r:&mut Runtime)->Result<()>{
  r.configure_485f_backend_profile(profile(&self.p,&self.sha)?)?;
  r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;
  room_player::configure(r)
 }
 pub(super) fn begin(&mut self)->Result<()>{
  self.evidence.entry_step("initial_connect_preflight",None);
  let result=(||{
  ensure!(!self.attempted&&self.first_failure.is_none(),"one initial connection attempt only; fresh supply required");
  self.attempted=true; // Claim before fallible verification: a failed attempt cannot retry.
  let p=provider::validate(&read_bound(&self.ready,65536)?,&self.sha)?;
  ensure!(p.run_root==self.p.run_root&&p.identities==self.p.identities,"provider identity changed since entry acceptance");
  self.p=p;self.healthy_at=Some(Instant::now());Ok(())})();
  self.evidence.entry_step("initial_connect_preflight",Some(result.is_ok()));if result.is_err(){self.evidence.failure("initial_connect_preflight_refused");}result
 }
 fn cached(&self)->Result<()>{
  ensure!(self.first_failure.is_none()&&self.healthy_at.is_some_and(|t|t.elapsed()<=Duration::from_secs(1)),"provider health unavailable/stale; operation refused");
  ensure!(self.p.lease_end_epoch_ms>=provider::now_ms()?+60000,"provider lease below60seconds");Ok(())
 }
 pub(super) fn login_account(&self,account:&[u8])->Result<()>{self.cached()?;ensure!(account==b"Fx485Guest","accepted Guest account required");Ok(())}
 pub(super) fn source_allowed(&self,row:&super::native_city_runtime::login_bootstrap::SourceRow)->bool{row.name==b"Fx485Moon"&&row.address_word==u32::from_le_bytes([127,0,0,2])&&row.port_word==self.p.port}
 pub(super) fn roster_allowed(&self,r:&Runtime)->bool{r.source_identity().complete()&&r.source_identity().expected()==Some(1)&&r.source_identity().records().len()==1&&r.source_identity().records()[0].name()==b"Fx485Guest(Fx485Moon)"}
 fn runtime_qualified(&self,r:&Runtime)->Result<()>{
  ensure!(self.roster_allowed(r),"actual singleton Guest roster differs");
  ensure!(r.source_bootstrap_snapshot().selected_name.as_deref()==Some(b"Fx485Moon"),"actual source differs");
  if let Ok(j)=r.admit_java_room_settings_context(){ensure!(j.channel==self.p.consumer.channel&&j.selected_alias==b"Fx485Guest(Fx485Moon)","joined source/selection/channel differs");}
  if let Some(room)=r.java_room_entry_snapshot().room{ensure!(room.id==self.p.consumer.room_id,"accepted public room differs");}Ok(())
 }
 /// Called by the existing UI timer. All reads/queries finish before a callback.
 pub(super) fn poll_health(&mut self,r:&mut Runtime)->bool{
  if self.first_failure.is_some()||!provider::claim_status_poll(&self.p.status_poll_started,Instant::now()){return false;}
  let result=(||{
   let s=provider::fresh_status(&self.p)?;
   for (pid,creation) in &self.p.identities{provider::same_live_identity(*pid,creation)?;}
   provider::sync_common(&s["host_sync"])?;
   if !self.ready_used&&provider::pre_ready_stage(&s["host_sync"])?{provider::armed_getter(&s["getter"],&s["host_sync"],self.p.consumer.room_id)?;self.armed=true;}else if !self.ready_used{self.armed=false;}
   if r.source_identity().complete(){ensure!(self.roster_allowed(r),"provider returned a foreign/non-singleton roster");}
   if r.java_room_entry_snapshot().room.is_some(){self.runtime_qualified(r)?;}
   self.healthy_at=Some(Instant::now());Ok::<_,anyhow::Error>(())
  })();
  if let Err(e)=result{self.first_failure=Some(e.to_string());self.healthy_at=None;self.evidence.failure("provider_health_failure");self.evidence.close_intent(r);let close=r.handle(Action::Close);if close.is_err(){self.evidence.failure("health_close_refused");}self.evidence.observe(r);}true
 }
 pub(super) fn permits(&self,r:&Runtime,panel:&Panel,op:Operation)->Result<()>{
  self.cached()?;self.runtime_qualified(r)?;
  match op {
   Operation::Join(slot)=>{let c=r.java_room_catalog_snapshot().context("current catalog absent")?;ensure!(c.channel==self.p.consumer.channel&&panel.rooms.get(slot).is_some_and(|row|row.room_id==self.p.consumer.room_id&&!row.password_required),"only accepted observed public room allowed");},
   Operation::Ready=>{ensure!(self.armed&&!self.ready_used,"waiting for actual Host arming witness; Ready may be used once");let book=r.java_room_participants_snapshot().context("participants absent")?;ensure!(book.rows.iter().any(|row|row.participant.display_alias==b"Fx485Host(Fx485Moon)"&&row.last_state_receipt.as_ref().is_some_and(|s|s.state.wire()==1)),"actual Host D1one absent");},
   Operation::Cancel|Operation::Refresh=>{}
  }Ok(())
 }
 pub(super) fn invoke(&mut self,r:&mut Runtime,panel:&Panel,op:Operation)->Result<()>{
  self.evidence.observe(r);self.evidence.operation(r,op,None);
  let result=(||{self.permits(r,panel,op)?;panel.invoke(r,op)?;if op==Operation::Ready{self.ready_used=true;}Ok(())})();
  self.evidence.operation(r,op,Some(result.is_ok()));if result.is_err(){self.evidence.failure("room_operation_refused");}self.evidence.observe(r);result
 }
 pub(super) fn observe_runtime(&mut self,r:&Runtime){self.evidence.observe(r);}
 pub(super) fn record_close_intent(&mut self,r:&Runtime){self.evidence.close_intent(r);}
 pub(super) fn record_failure(&mut self,category:&'static str){self.evidence.failure(category);}
 pub(super) fn record_entry_step(&mut self,stage:&'static str,accepted:Option<bool>){self.evidence.entry_step(stage,accepted);}
 pub(super) fn finish(&mut self,r:&Runtime)->Result<()>{self.evidence.observe(r);let s=r.player_fixture_snapshot();save_new(&self.output.join("result.json"),&json!({"schema":"modern.r532.player-candidate-result.v1","fresh_run_id":self.p.consumer.fresh_run_id,"supervisor_sha256":self.sha,"connection_attempted":self.attempted,"first_provider_health_failure":self.first_failure,"evidence":self.evidence.report(),"IO_started":s.workers_started,"IO_joined":s.workers_joined,"queued_events":s.queued_events,"discarded_late_events":s.discarded_late_events,"server_stop_required":true,"credentials_contents_or_hash_recorded":false,"GUI_acceptance_proven":false,"WORLD_admitted":{"status":"unknown","reason":"no dedicated admission observation or GUI evidence"}}))}
}
