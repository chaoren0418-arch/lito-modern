//! Explicit schema2 selected-Work observer. Schema1 remains a distinct contract.
use super::*;
use std::collections::BTreeMap;
use serde::Serialize;
dto!(Ready2{schema:u32,control_contract:String,hold_point:String,overlap_policy:String,state:String,fixture_id:String,expires_at_epoch_millis:i64,owner:Owner,candidate_manifest_sha256:String,runtime_namespace:String,runtime_namespace_root:PathBuf,navigation_baseline_manifest_sha256:String,tls:Tls,control:ControlReady,mysql:Mysql,protocol:Protocol,sources:Vec<Source>,selection:Selection,resources:Resources,limits:Limits});
impl Ready2{
 fn common(self)->Result<Ready>{require(self.schema==2&&self.control_contract=="NATIVE_SELECTED_WORK_CONTROL_V2"&&self.hold_point=="AFTER_REAL_SQL_AND_SAME_WORK_RECONCILED_STORE_BEFORE_COMMIT"&&self.overlap_policy=="SAME_SESSION_BUSY_UNTIL_ACTUAL_WORK_RETIRES","v2_contract")?;Ok(Ready{schema:self.schema,state:self.state,fixture_id:self.fixture_id,expires_at_epoch_millis:self.expires_at_epoch_millis,owner:self.owner,candidate_manifest_sha256:self.candidate_manifest_sha256,runtime_namespace:self.runtime_namespace,runtime_namespace_root:self.runtime_namespace_root,navigation_baseline_manifest_sha256:self.navigation_baseline_manifest_sha256,tls:self.tls,control:self.control,mysql:self.mysql,protocol:self.protocol,sources:self.sources,selection:self.selection,resources:self.resources,limits:self.limits})}
}
#[derive(Clone,Deserialize,Serialize,PartialEq,Eq)]#[serde(deny_unknown_fields)]
pub(super) struct Witness{
 pub job_id:String,pub source:u32,pub object:u64,pub selected_epoch_hex:String,pub channel_id:u32,
 pub private_registration_generation:String,pub effective_gear_revision_at_entry:i64,
 pub work_deadline_monotonic_nanos:i64,pub work_operation_id:String,pub correlation_origin:String,
}
fn hex64(s:&str)->bool{s.len()==16&&s.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b))}
impl Witness{pub fn verify(&self,id:&str,s:Scope,revision:i64)->Result<()>{require(uuid(id)&&self.job_id==id&&self.source==s.context.key.source_id&&self.object==s.context.key.object_id&&self.selected_epoch_hex==format!("{:016x}",s.context.epoch)&&self.channel_id==u32::from(s.channel)&&hex64(&self.private_registration_generation)&&self.effective_gear_revision_at_entry==revision&&revision>=0&&uuid(&self.work_operation_id)&&self.correlation_origin=="ACTUAL_REGISTERED_WORK_OPERATION_SQL_AND_RECONCILED_STORE","v2_same_Work_owner_revision")}}
macro_rules! observed_dto{($name:ident{$($f:ident:$t:ty),*$(,)?})=>{
 #[derive(Deserialize,Serialize)]#[serde(deny_unknown_fields)]pub(super) struct $name{
 $(pub $f:$t,)*
 pub job_id:Option<String>,pub selected_epoch_hex:Option<String>,pub channel_id:Option<u32>,
 pub private_registration_generation:Option<String>,pub effective_gear_revision_at_entry:Option<i64>,
 pub work_deadline_monotonic_nanos:Option<i64>,pub work_operation_id:Option<String>,pub correlation_origin:Option<String>,
 }
 impl $name{pub fn witness(&self)->Result<Option<Witness>>{
 let none=self.job_id.is_none()&&self.selected_epoch_hex.is_none()&&self.channel_id.is_none()&&self.private_registration_generation.is_none()&&self.effective_gear_revision_at_entry.is_none()&&self.work_deadline_monotonic_nanos.is_none()&&self.work_operation_id.is_none()&&self.correlation_origin.is_none();if none{return Ok(None)}
 Ok(Some(Witness{job_id:self.job_id.clone().ok_or(err("v2_witness_partial"))?,source:self.source,object:self.object,selected_epoch_hex:self.selected_epoch_hex.clone().ok_or(err("v2_witness_partial"))?,channel_id:self.channel_id.ok_or(err("v2_witness_partial"))?,private_registration_generation:self.private_registration_generation.clone().ok_or(err("v2_witness_partial"))?,effective_gear_revision_at_entry:self.effective_gear_revision_at_entry.ok_or(err("v2_witness_partial"))?,work_deadline_monotonic_nanos:self.work_deadline_monotonic_nanos.ok_or(err("v2_witness_partial"))?,work_operation_id:self.work_operation_id.clone().ok_or(err("v2_witness_partial"))?,correlation_origin:self.correlation_origin.clone().ok_or(err("v2_witness_partial"))?}))
 }}
}}
observed_dto!(Hold2{hold_id:String,source:u32,object:u64,entered:bool,released:bool,finished:bool,remaining_nanos_at_entry:i64,result:String,sql_query_started_epoch_millis:i64,sql_cutoff_exclusive_epoch_millis:i64,release_at_epoch_millis:i64,backend_remaining_nanos_at_release:i64});
// Jobs always have job_id/channel_id even before observation, unlike an armed Hold.
#[derive(Deserialize,Serialize)]#[serde(deny_unknown_fields)]pub(super) struct Job2{
 pub job_id:String,pub state:String,pub source:u32,pub object:u64,pub epoch_hex:String,pub channel_id:u32,pub work_observed:bool,
 pub selected_epoch_hex:Option<String>,pub private_registration_generation:Option<String>,pub effective_gear_revision_at_entry:Option<i64>,pub work_deadline_monotonic_nanos:Option<i64>,pub work_operation_id:Option<String>,pub correlation_origin:Option<String>,
 pub revision:Option<i64>,pub weapon_object:Option<i64>,pub max_hp:Option<i32>,pub evaluated_at_millis:Option<i64>,pub valid_until_millis:Option<i64>,pub reason:Option<String>,
}
impl Job2{
 pub fn witness(&self)->Result<Option<Witness>>{
 if !self.work_observed{require(self.selected_epoch_hex.is_none()&&self.private_registration_generation.is_none()&&self.effective_gear_revision_at_entry.is_none()&&self.work_deadline_monotonic_nanos.is_none()&&self.work_operation_id.is_none()&&self.correlation_origin.is_none(),"v2_unobserved_witness_invented")?;return Ok(None)}
 Ok(Some(Witness{job_id:self.job_id.clone(),source:self.source,object:self.object,selected_epoch_hex:self.selected_epoch_hex.clone().ok_or(err("v2_witness_partial"))?,channel_id:self.channel_id,private_registration_generation:self.private_registration_generation.clone().ok_or(err("v2_witness_partial"))?,effective_gear_revision_at_entry:self.effective_gear_revision_at_entry.ok_or(err("v2_witness_partial"))?,work_deadline_monotonic_nanos:self.work_deadline_monotonic_nanos.ok_or(err("v2_witness_partial"))?,work_operation_id:self.work_operation_id.clone().ok_or(err("v2_witness_partial"))?,correlation_origin:self.correlation_origin.clone().ok_or(err("v2_witness_partial"))?}))
 }
 pub fn verify(&self,id:&str,s:Scope,revision:i64,expected:Option<&Witness>)->Result<bool>{
 require(self.job_id==id&&uuid(id)&&self.source==s.context.key.source_id&&self.object==s.context.key.object_id&&self.epoch_hex==format!("{:016x}",s.context.epoch)&&self.channel_id==u32::from(s.channel),"v2_job_request_owner")?;
 let w=self.witness()?;if let Some(w)=&w{w.verify(id,s,revision)?;if let Some(e)=expected{require(w==e,"v2_terminal_Work_changed")?;}}
 let payload=self.revision.is_some()||self.weapon_object.is_some()||self.max_hp.is_some()||self.evaluated_at_millis.is_some()||self.valid_until_millis.is_some();
 match self.state.as_str(){
 "RUNNING"=>{require(!payload&&self.reason.is_none(),"v2_running_payload")?;Ok(false)},
 "REJECTED"=>{require(!payload&&self.reason.as_deref()==Some("ACTUAL_READER_REJECTED")&&(!expected.is_some()||w.is_some()),"v2_rejected_witness")?;Ok(true)},
 "COMPLETE"=>{let observed=self.evaluated_at_millis.ok_or(err("v2_complete_payload"))?;let until=self.valid_until_millis.ok_or(err("v2_complete_payload"))?;require(w.is_some()&&self.revision==Some(revision)&&self.weapon_object.is_some()&&self.max_hp.is_some_and(|v|v>0)&&observed>0&&(until==-1||until>observed)&&self.reason.is_none(),"v2_complete_same_Work")?;Ok(true)},
 _=>Err(err("v2_job_state"))}
 }
}
fn unique_jobs<'de,D:serde::Deserializer<'de>>(d:D)->std::result::Result<BTreeMap<String,Job2>,D::Error>{
 struct Jobs;impl<'de> serde::de::Visitor<'de> for Jobs{type Value=BTreeMap<String,Job2>;fn expecting(&self,f:&mut std::fmt::Formatter)->std::fmt::Result{f.write_str("at most32 unique job UUIDs")}
 fn visit_map<M:serde::de::MapAccess<'de>>(self,mut m:M)->std::result::Result<Self::Value,M::Error>{let mut out=BTreeMap::new();while let Some((id,j))=m.next_entry::<String,Job2>()?{if out.len()>=32||!uuid(&id)||out.insert(id,j).is_some(){return Err(serde::de::Error::custom("duplicate/invalid/overbound job"))}}Ok(out)}}d.deserialize_map(Jobs)
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub(super) struct Status2{pub schema:u32,pub fixture_id:String,pub state:String,pub expires_at_epoch_millis:i64,pub server:Value,pub provider_calls:u32,pub hold:Option<Hold2>,#[serde(deserialize_with="unique_jobs")]pub jobs:BTreeMap<String,Job2>,pub timer_owned:bool}
dto!(Retirement2{schema:u32,state:String,fixture_id:String,reason:String,timer_restored:bool,TLS_port:u16,control_port:u16,world_store_quiescent:bool});
pub(super) struct Adapter2{pub transport:Adapter}
impl Adapter2{
 pub fn bind(path:&Path,runtime:&Path,manifest:[u8;32])->Result<(Self,runner::Plan)>{
 let root=fs::canonicalize(runtime).map_err(|_|err("fixture_runtime"))?;let p=confined(&root,path)?;require(p==root.join("ready.json"),"ready_filename")?;
 let r:Ready2=decode(&bounded_file(&p,32*1024)?,32*1024)?;let(a,p)=Adapter::bind_ready(r.common()?,root,manifest,false,2)?;Ok((Self{transport:a},p))
 }
 pub fn command(&self,name:&str,args:&[(&str,String)],end:Instant)->Result<Value>{self.transport.command(name,args,end)}
 pub fn status(&self,end:Instant)->Result<Status2>{let s:Status2=decode(&self.transport.http(None,end)?,128*1024)?;require(s.schema==2&&s.fixture_id==self.transport.ready.fixture_id&&s.state=="READY"&&s.expires_at_epoch_millis==self.transport.ready.expires_at_epoch_millis&&s.jobs.len()<=32,"v2_status_owner")?;Ok(s)}
}
#[cfg(test)]#[path="native_external_control_v2_tests.rs"]mod tests;
