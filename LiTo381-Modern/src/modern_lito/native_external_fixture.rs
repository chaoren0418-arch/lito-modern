//! Explicit, default-off adapter for a pinned external fixture. No endpoint discovery.
use super::native_navigation_runner::{self as runner,Control,ExpiryChallenge,ExpiryObservation,Fault,Scope};
use serde::{Deserialize,de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest,Sha256};
use zeroize::Zeroizing;
use std::{fs,io::{Read,Write},net::{SocketAddr,TcpStream},path::{Path,PathBuf},time::{Duration,Instant,SystemTime,UNIX_EPOCH}};
type Result<T> = std::result::Result<T,Fault>;
fn err(code:&'static str)->Fault{Fault{step:"external_fixture",code}}
fn require(ok:bool,code:&'static str)->Result<()>{if ok{Ok(())}else{Err(err(code))}}
fn now()->Result<i64>{SystemTime::now().duration_since(UNIX_EPOCH).ok().and_then(|d|i64::try_from(d.as_millis()).ok()).ok_or(err("wall_clock"))}
fn decode<T:DeserializeOwned>(bytes:&[u8],limit:usize)->Result<T>{require(!bytes.is_empty()&&bytes.len()<=limit,"JSON_budget")?;serde_json::from_slice(bytes).map_err(|_|err("strict_JSON_schema"))}
fn hash(s:&str)->Result<[u8;32]>{require(s.len()==64&&s.bytes().all(|b|b.is_ascii_hexdigit()),"digest_encoding")?;let mut h=[0;32];for(i,b)in h.iter_mut().enumerate(){*b=u8::from_str_radix(&s[i*2..i*2+2],16).map_err(|_|err("digest_encoding"))?;}Ok(h)}
fn uuid(s:&str)->bool{s.len()==36&&s.bytes().enumerate().all(|(i,b)|if [8,13,18,23].contains(&i){b==b'-'}else{b.is_ascii_digit()||(b'a'..=b'f').contains(&b)})}
macro_rules! dto{($name:ident{$($field:ident:$ty:ty),*$(,)?})=>{#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct $name{$($field:$ty),*}}}
dto!(Owner{pid:u32,start_utc:String,creation_filetime_utc:String,jdk_start_utc:String,creation_time_source:String});
dto!(Mysql{pid:u32,start_utc:String,creation_filetime_utc:String,jdk_start_utc:String,creation_time_source:String,port:u16,datadir:PathBuf});
dto!(Tls{host:String,port:u16,server_name:String,root_der_path:PathBuf,root_der_sha256:String,server_der_sha256:String});
fn secret<'de,D:serde::Deserializer<'de>>(d:D)->std::result::Result<Zeroizing<String>,D::Error>{String::deserialize(d).map(Zeroizing::new)}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]struct ControlReady{host:String,port:u16,path:String,#[serde(deserialize_with="secret")]token:Zeroizing<String>}
dto!(Protocol{version:u16,capabilities:u32});
dto!(Source{id:u32,database:String});
dto!(Selection{initial_object:u64,reselect_object:u64,epoch_is_opaque_u64:bool});
dto!(Resources{map_id:u32,source_root:PathBuf,map_digest:String,sector_sha256:String});
dto!(Limits{fixture_lifetime_seconds:u32,max_hold_millis:u32,max_fresh_read_deadline_millis:u32});
dto!(Ready{schema:u32,state:String,fixture_id:String,expires_at_epoch_millis:i64,owner:Owner,candidate_manifest_sha256:String,runtime_namespace:String,runtime_namespace_root:PathBuf,navigation_baseline_manifest_sha256:String,tls:Tls,control:ControlReady,mysql:Mysql,protocol:Protocol,sources:Vec<Source>,selection:Selection,resources:Resources,limits:Limits});
pub(super) struct Adapter{ready:Ready,timer_owned:bool,challenge:Option<ExpiryChallenge>,hold:Option<Hold>,released:bool}
dto!(Hold{hold_id:String,source:u32,object:u64,entered:bool,released:bool,finished:bool,remaining_nanos_at_entry:i64,result:String,sql_query_started_epoch_millis:i64,sql_cutoff_exclusive_epoch_millis:i64,release_at_epoch_millis:i64,backend_remaining_nanos_at_release:i64});
dto!(Status{schema:u32,fixture_id:String,state:String,expires_at_epoch_millis:i64,server:Value,provider_calls:u32,hold:Option<Hold>,jobs:Value,timer_owned:bool});
dto!(Reply{schema:u32,fixture_id:String,request_id:String,status:String,result:Value});
fn confined(root:&Path,path:&Path)->Result<PathBuf>{let p=fs::canonicalize(path).map_err(|_|err("fixture_path"))?;require(p.starts_with(root)&&p!=root,"fixture_path_escape")?;Ok(p)}
fn bounded_file(p:&Path,limit:u64)->Result<Zeroizing<Vec<u8>>>{require(fs::metadata(p).map_err(|_|err("fixture_file"))?.len()<=limit,"fixture_file_budget")?;let mut b=Zeroizing::new(Vec::new());fs::File::open(p).map_err(|_|err("fixture_file"))?.take(limit+1).read_to_end(&mut b).map_err(|_|err("fixture_file"))?;require(b.len()as u64<=limit,"fixture_file_budget")?;Ok(b)}
#[cfg(windows)]fn witness(pid:u32,ft:&str)->Result<()>{
 use windows::Win32::{Foundation::{CloseHandle,FILETIME},System::Threading::{OpenProcess,GetProcessTimes,GetExitCodeProcess,PROCESS_QUERY_LIMITED_INFORMATION}};
 let expected=ft.parse::<u64>().map_err(|_|err("owner_FILETIME"))?;require(pid>0&&expected>0,"owner_identity")?;
 unsafe{let h=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,false,pid).map_err(|_|err("owner_process"))?;let mut creation=FILETIME::default();let mut exit=FILETIME::default();let mut kernel=FILETIME::default();let mut user=FILETIME::default();let mut code=0;
 let result=(||{GetProcessTimes(h,&mut creation,&mut exit,&mut kernel,&mut user).map_err(|_|err("owner_process_times"))?;GetExitCodeProcess(h,&mut code).map_err(|_|err("owner_process_exit"))?;require(code==259&&((creation.dwHighDateTime as u64)<<32|creation.dwLowDateTime as u64)==expected,"owner_PID_reused_or_retired")})();let _=CloseHandle(h);result}
}
#[cfg(not(windows))]fn witness(_:u32,_:&str)->Result<()>{Err(err("native_owner_unavailable"))}
impl Ready{
 fn validate(&self,manifest:[u8;32],time:i64)->Result<()>{
 self.validate_schema(manifest,time,1)
 }
 fn validate_schema(&self,manifest:[u8;32],time:i64,expected:u32)->Result<()>{
 require(matches!(expected,1|2)&&self.schema==expected&&self.state=="READY"&&uuid(&self.fixture_id),"ready_state")?;
 require(self.runtime_namespace=="modern"&&self.runtime_namespace_root==Path::new("D:/lito-dev-runtime/20261003-supervision/modern"),"ready_runtime_namespace")?;
 require(hash(&self.candidate_manifest_sha256)?==manifest&&hash(&self.navigation_baseline_manifest_sha256)?==hash("B20D6C58A8CC52376164FCFCC7059CB52E9C76EFFAC408CA9039534543A9055A")?,"ready_candidate_pin")?;
 require(self.expires_at_epoch_millis>time+45_000&&self.expires_at_epoch_millis<=time+600_000,"ready_expiry")?;
 require(self.tls.host=="127.0.0.1"&&self.control.host=="127.0.0.1"&&self.tls.port!=0&&self.control.port!=0&&self.tls.port!=self.control.port&&self.tls.server_name=="localhost"&&self.control.path=="/control"&&self.control.token.len()==72&&self.control.token.is_ascii()&&uuid(&self.control.token[..36])&&uuid(&self.control.token[36..]),"ready_endpoint_or_token")?;
 require(self.protocol.version==26&&self.protocol.capabilities==runner::CAPS&&self.sources.len()==2&&self.sources[0].id==7&&self.sources[0].database=="380_ac_1"&&self.sources[1].id==42&&self.sources[1].database=="380_ac_2","ready_protocol_sources")?;
 require(self.selection.initial_object==10275&&self.selection.reselect_object==10163&&self.selection.epoch_is_opaque_u64&&self.resources.map_id==564&&hash(&self.resources.map_digest)?!=[0;32]&&hash(&self.resources.sector_sha256)?==hash("B68EE758F17448316C70ED5D49AD65EB6432AF483F0591A5F8AA7987F9AF6865")?,"ready_owner_resources")?;
 require((60..=600).contains(&self.limits.fixture_lifetime_seconds)&&self.limits.max_hold_millis==8000&&self.limits.max_fresh_read_deadline_millis==10000&&self.owner.pid!=self.mysql.pid&&self.mysql.port!=0&&self.mysql.port!=self.tls.port&&self.mysql.port!=self.control.port,"ready_limits")?;
 hash(&self.tls.server_der_sha256)?;hash(&self.tls.root_der_sha256)?;Ok(())
 }
}
impl Adapter{
 /// Caller supplies the exact ready file and authorized runtime; no search or fallback.
 pub fn bind(path:&Path,runtime:&Path,manifest:[u8;32],dto_ack_only:bool)->Result<(Self,runner::Plan)>{
 let root=fs::canonicalize(runtime).map_err(|_|err("fixture_runtime"))?;let p=confined(&root,path)?;require(p==root.join("ready.json"),"ready_filename")?;
 let r:Ready=decode(&bounded_file(&p,32*1024)?,32*1024)?;Self::bind_ready(r,root,manifest,dto_ack_only,1)
 }
 fn bind_ready(r:Ready,root:PathBuf,manifest:[u8;32],dto_ack_only:bool,schema:u32)->Result<(Self,runner::Plan)>{
 r.validate_schema(manifest,now()?,schema)?;let namespace=fs::canonicalize(&r.runtime_namespace_root).map_err(|_|err("ready_runtime_namespace"))?;require(root.parent()==Some(namespace.as_path())&&root.file_name().and_then(|s|s.to_str()).is_some_and(|s|s.strip_prefix("native-external-navigation-runtime-").is_some_and(|id|id.len()==32&&id.bytes().all(|b|b.is_ascii_digit()||(b'a'..=b'f').contains(&b)))),"ready_unique_runtime_root")?;
 witness(r.owner.pid,&r.owner.creation_filetime_utc)?;witness(r.mysql.pid,&r.mysql.creation_filetime_utc)?;
 confined(&root,&r.mysql.datadir)?;confined(&root,&r.resources.source_root)?;let ca=bounded_file(&confined(&root,&r.tls.root_der_path)?,64*1024)?;
 let ca_hash=hash(&r.tls.root_der_sha256)?;require(<[u8;32]>::from(Sha256::digest(&*ca))==ca_hash,"ready_CA_pin")?;
 let server_der=bounded_file(&confined(&root,&root.join("configuration-tls/server.der"))?,64*1024)?;require(<[u8;32]>::from(Sha256::digest(&*server_der))==hash(&r.tls.server_der_sha256)?,"ready_server_certificate_pin")?;
 let plan=runner::Plan{endpoint:SocketAddr::from(([127,0,0,1],r.tls.port)),server_name:r.tls.server_name.clone(),ca_der:ca.to_vec(),ca_sha256:ca_hash,sources:[7,42],object:10275,reselected_object:10163,initial_epoch:0x8000000000000000,channel:0,weapon:10304,armor:10305,map_id:564,manifest_digest:hash(&r.resources.map_digest)?,total_timeout:Duration::from_secs(45),exchange_timeout:Duration::from_secs(2),prepare_timeout:Duration::from_secs(10),dto_ack_only};plan.validate()?;
 Ok((Self{ready:r,timer_owned:false,challenge:None,hold:None,released:false},plan))
 }
 fn active(&self,end:Instant)->Result<Duration>{require(Instant::now()<end&&now()?<self.ready.expires_at_epoch_millis,"control_deadline")?;witness(self.ready.owner.pid,&self.ready.owner.creation_filetime_utc)?;witness(self.ready.mysql.pid,&self.ready.mysql.creation_filetime_utc)?;Ok(end.saturating_duration_since(Instant::now()).min(Duration::from_secs(2)))}
 fn http(&self,body:Option<&str>,end:Instant)->Result<Vec<u8>>{self.http_expected(body,end,200)}
 fn http_expected(&self,body:Option<&str>,end:Instant,expected:u16)->Result<Vec<u8>>{
 let wait=self.active(end)?;let mut socket=TcpStream::connect_timeout(&SocketAddr::from(([127,0,0,1],self.ready.control.port)),wait).map_err(|_|err("control_connect"))?;
 let head=if let Some(b)=body{format!("POST /control HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",&*self.ready.control.token,b.len(),b)}else{format!("GET /status HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nConnection: close\r\n\r\n",&*self.ready.control.token)};let head=Zeroizing::new(head);
 socket.set_write_timeout(Some(self.active(end)?)).map_err(|_|err("control_timeout"))?;socket.write_all(head.as_bytes()).map_err(|_|err("control_write"))?;
 let mut bytes=Vec::new();loop{socket.set_read_timeout(Some(self.active(end)?)).map_err(|_|err("control_timeout"))?;let mut part=[0;4096];let n=socket.read(&mut part).map_err(|_|err("control_read"))?;if n==0{break;}require(bytes.len()+n<=128*1024,"control_response_budget")?;bytes.extend_from_slice(&part[..n]);}parse_http_expected(&bytes,expected)
 }
 pub fn command(&self,command:&str,args:&[(&str,String)],end:Instant)->Result<Value>{
 require(["PING","ARM_HOLD","RELEASE","FRESH_READ","TIMER_ARM","TIMER_CLEAR","SHUTDOWN"].contains(&command),"control_command")?;
 let request=unsafe{windows::Win32::System::Com::CoCreateGuid()}.map_err(|_|err("control_request_id"))?;let id=format!("{:08x}-{:04x}-{:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",request.data1,request.data2,request.data3,request.data4[0],request.data4[1],request.data4[2],request.data4[3],request.data4[4],request.data4[5],request.data4[6],request.data4[7]);
 let mut body=format!("fixture_id={}\nrequest_id={}\ncommand={}\n",self.ready.fixture_id,id,command);for(k,v)in args{require(!k.is_empty()&&k.bytes().all(|b|b.is_ascii_lowercase()||b==b'_')&&!v.contains(['\r','\n']),"control_argument")?;body.push_str(&format!("{k}={v}\n"));}require(body.len()<=4096,"control_request_budget")?;
 let reply:Reply=decode(&self.http(Some(&body),end)?,128*1024)?;require(reply.schema==self.ready.schema&&reply.fixture_id==self.ready.fixture_id&&reply.request_id==id&&reply.status=="OK","control_reply_owner")?;Ok(reply.result)
 }
 fn status(&self,end:Instant)->Result<Status>{let s:Status=decode(&self.http(None,end)?,128*1024)?;require(s.schema==1&&s.fixture_id==self.ready.fixture_id&&s.state=="READY"&&s.expires_at_epoch_millis==self.ready.expires_at_epoch_millis,"control_status_owner")?;Ok(s)}
 fn validate_hold(&self,h:&Hold)->Result<()>{let c=self.challenge.as_ref().ok_or(err("hold_not_armed"))?;require(h.hold_id==c.token&&h.source==c.scope.context.key.source_id&&h.object==c.scope.context.key.object_id,"hold_owner")}
}
fn parse_http(bytes:&[u8])->Result<Vec<u8>>{parse_http_expected(bytes,200)}
fn parse_http_expected(bytes:&[u8],expected:u16)->Result<Vec<u8>>{
 let split=bytes.windows(4).position(|w|w==b"\r\n\r\n").ok_or(err("control_HTTP_header"))?;require(split<=8192,"control_HTTP_header_budget")?;let h=std::str::from_utf8(&bytes[..split]).map_err(|_|err("control_HTTP_header"))?;let mut lines=h.split("\r\n");let status=match expected{200=>"HTTP/1.1 200 OK",400=>"HTTP/1.1 400 Bad Request",_=>return Err(err("control_HTTP_expected_status"))};require(lines.next()==Some(status),"control_HTTP_status")?;let mut length=None;
 for line in lines{let(k,v)=line.split_once(':').ok_or(err("control_HTTP_header"))?;if k.eq_ignore_ascii_case("content-length"){require(length.is_none(),"control_HTTP_duplicate_length")?;length=Some(v.trim().parse::<usize>().map_err(|_|err("control_HTTP_length"))?);}require(!k.eq_ignore_ascii_case("transfer-encoding"),"control_HTTP_chunked_not_allowed")?;}
 let body=&bytes[split+4..];require(length==Some(body.len())&&body.len()<=120*1024,"control_HTTP_length")?;Ok(body.to_vec())
}

#[cfg(test)]#[path="native_external_lifecycle_tests.rs"]mod lifecycle;
#[path="native_external_control_v2.rs"]pub(super) mod v2;
impl Control for Adapter{
 fn arm_expiry(&mut self,scope:Scope,item:u64,revision:i64,end:Instant)->Result<ExpiryChallenge>{
 require(scope.context.key.source_id==7&&scope.context.key.object_id==10275&&scope.channel==0&&item==10304&&!self.timer_owned&&self.challenge.is_none(),"expiry_scope")?;
 let timer=self.command("TIMER_ARM",&[("source_id","7".into()),("object_id","10275".into()),("item_object_id","10304".into()),("seconds","4".into())],end)?;self.timer_owned=true;require(timer["item"]==10304&&timer["cutoff_epoch_millis"].as_i64().is_some_and(|c|c>now().unwrap_or(i64::MAX)),"timer_result")?;
 let h:Hold=serde_json::from_value(self.command("ARM_HOLD",&[("source_id","7".into()),("object_id","10275".into())],end)?).map_err(|_|err("hold_schema"))?;require(uuid(&h.hold_id)&&h.source==7&&h.object==10275&&!h.entered&&!h.released&&!h.finished&&h.result=="ARMED","hold_arm_result")?;
 let c=ExpiryChallenge{token:h.hold_id.clone(),scope,item,gear_revision:revision};self.challenge=Some(c.clone());self.hold=Some(h);Ok(c)
 }
 fn progress_expiry(&mut self,c:&ExpiryChallenge,end:Instant)->Result<()>{
 require(self.challenge.as_ref().is_some_and(|a|a.token==c.token&&a.scope==c.scope),"expiry_challenge")?;let s=self.status(end)?;let h=s.hold.ok_or(err("hold_absent"))?;self.validate_hold(&h)?;
 if h.entered{require(h.sql_query_started_epoch_millis<h.sql_cutoff_exclusive_epoch_millis&&h.remaining_nanos_at_entry>0,"query_not_active_at_entry")?;
 if !self.released&&now()?>h.sql_cutoff_exclusive_epoch_millis{require(!h.finished&&!h.released,"hold_finished_before_release")?;let released:Hold=serde_json::from_value(self.command("RELEASE",&[("hold_id",c.token.clone())],end)?).map_err(|_|err("hold_schema"))?;self.validate_hold(&released)?;require(released.released&&released.release_at_epoch_millis>h.sql_cutoff_exclusive_epoch_millis&&released.backend_remaining_nanos_at_release>500_000_000,"release_deadline_or_cutoff")?;self.released=true;self.hold=Some(released);return Ok(());}}
 self.hold=Some(h);Ok(())
 }
 fn expiry_observation(&mut self,c:&ExpiryChallenge,end:Instant)->Result<ExpiryObservation>{require(self.challenge.as_ref().is_some_and(|a|a.token==c.token&&a.scope==c.scope&&a.item==c.item&&a.gear_revision==c.gear_revision),"expiry_challenge")?;let h=self.status(end)?.hold.ok_or(err("hold_absent"))?;self.validate_hold(&h)?;require(self.released&&h.entered&&h.released&&h.finished&&h.result=="RELEASED","actual_provider_release_required")?;Ok(ExpiryObservation{token:h.hold_id,scope:c.scope,item:c.item,gear_revision:c.gear_revision,query_started_millis:h.sql_query_started_epoch_millis,cutoff_exclusive:h.sql_cutoff_exclusive_epoch_millis,provider_finished_millis:h.release_at_epoch_millis,backend_remaining_nanos:h.backend_remaining_nanos_at_release,actual_wall_clock:true,selected_owner_registration:true})}
 fn cleanup(&mut self,end:Instant)->Result<()>{let mut failure=None;if let Some(c)=&self.challenge{if !self.released{match self.command("RELEASE",&[("hold_id",c.token.clone())],end){Ok(_)=>self.released=true,Err(e)=>failure=Some(e)}}}if self.timer_owned{match self.command("TIMER_CLEAR",&[],end){Ok(v)=>{if v["changed"]==true{self.timer_owned=false;}else{failure.get_or_insert(err("timer_not_restored"));}},Err(e)=>{failure.get_or_insert(e);}}}if let Some(e)=failure{Err(e)}else{Ok(())}}
}

#[cfg(test)]mod tests{
 use super::*;use serde_json::json;
 const MANIFEST:&str="DF46385536ECC3865087F9F6A877B1FC7F3E80EA22CA7F91A5D90C52B7143E57";
 const ID:&str="11111111-2222-4333-8444-555555555555";
 pub(super) fn model()->Value{let owner=json!({"pid":1234,"start_utc":"synthetic","creation_filetime_utc":"133000000000000000","jdk_start_utc":"synthetic","creation_time_source":"synthetic model only"});let mut mysql=owner.clone();mysql["pid"]=json!(5678);mysql["port"]=json!(12345);mysql["datadir"]=json!("model/mysql");json!({"schema":1,"state":"READY","fixture_id":ID,"runtime_namespace":"modern","runtime_namespace_root":"D:/lito-dev-runtime/20261003-supervision/modern","expires_at_epoch_millis":160000,"owner":owner,"candidate_manifest_sha256":MANIFEST,"navigation_baseline_manifest_sha256":"B20D6C58A8CC52376164FCFCC7059CB52E9C76EFFAC408CA9039534543A9055A","tls":{"host":"127.0.0.1","port":11111,"server_name":"localhost","root_der_path":"model/root.der","root_der_sha256":"11".repeat(32),"server_der_sha256":"22".repeat(32)},"control":{"host":"127.0.0.1","port":22222,"path":"/control","token":format!("{ID}{ID}")},"mysql":mysql,"protocol":{"version":26,"capabilities":65791},"sources":[{"id":7,"database":"380_ac_1"},{"id":42,"database":"380_ac_2"}],"selection":{"initial_object":10275,"reselect_object":10163,"epoch_is_opaque_u64":true},"resources":{"map_id":564,"source_root":"model/resources","map_digest":"33".repeat(32),"sector_sha256":"B68EE758F17448316C70ED5D49AD65EB6432AF483F0591A5F8AA7987F9AF6865"},"limits":{"fixture_lifetime_seconds":60,"max_hold_millis":8000,"max_fresh_read_deadline_millis":10000}})}
 fn ready(v:Value)->Ready{decode(&serde_json::to_vec(&v).unwrap(),32768).unwrap()}
 #[test]fn ready_rejects_foreign_candidate_expired_and_wrong_endpoint_protocol_source(){let r=ready(model());r.validate(hash(MANIFEST).unwrap(),100000).unwrap();assert!(r.validate([0;32],100000).is_err());assert!(r.validate(hash(MANIFEST).unwrap(),115000).is_err());for(pointer,bad)in [("/runtime_namespace",json!("server")),("/runtime_namespace_root",json!("D:/lito-dev-runtime/20261003-supervision/server")),("/state",json!("RETIRED")),("/tls/host",json!("192.0.2.1")),("/tls/port",json!(22222)),("/protocol/capabilities",json!(65790)),("/sources/1/id",json!(7)),("/selection/epoch_is_opaque_u64",json!(false)),("/resources/map_id",json!(514)),("/limits/max_hold_millis",json!(9000))]{let mut v=model();*v.pointer_mut(pointer).unwrap()=bad;assert!(ready(v).validate(hash(MANIFEST).unwrap(),100000).is_err(),"{pointer}");}}
 #[test]fn strict_ready_duplicate_unknown_truncation_budget_and_unicode_token_fail_closed(){let bytes=serde_json::to_vec(&model()).unwrap();let text=String::from_utf8(bytes.clone()).unwrap();let dup=text.replacen("\"schema\":1","\"schema\":1,\"schema\":1",1);assert!(decode::<Ready>(dup.as_bytes(),32768).is_err());let mut v=model();v["tls"]["fallback_port"]=json!(443);assert!(decode::<Ready>(&serde_json::to_vec(&v).unwrap(),32768).is_err());assert!(decode::<Ready>(&bytes[..bytes.len()-1],32768).is_err());assert!(decode::<Ready>(&bytes,bytes.len()-1).is_err());let mut v=model();v["control"]["token"]=json!("界".repeat(24));assert!(ready(v).validate(hash(MANIFEST).unwrap(),100000).is_err());}
 #[test]fn HTTP_rejects_trailing_truncated_duplicate_length_chunking_and_error_status(){let okay=b"HTTP/1.1 200 OK\r\nContent-length: 2\r\n\r\n{}";assert_eq!(parse_http(okay).unwrap(),b"{}");for b in [b"HTTP/1.1 200 OK\r\nContent-length: 3\r\n\r\n{}".as_slice(),b"HTTP/1.1 200 OK\r\nContent-length: 2\r\n\r\n{}x",b"HTTP/1.1 200 OK\r\nContent-length: 2\r\nContent-Length: 2\r\n\r\n{}",b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n0\r\n\r\n",b"HTTP/1.1 403 Forbidden\r\nContent-length: 2\r\n\r\n{}"]{assert!(parse_http(b).is_err());}}
 #[test]fn hold_source_object_token_and_complete_body_must_match_armed_operation(){let s=Scope{context:super::super::application_protocol::SelectionContext{key:super::super::application_protocol::CharacterKey{source_id:7,object_id:10275},epoch:0x8000000000000002},channel:0};let a=Adapter{ready:ready(model()),timer_owned:true,challenge:Some(ExpiryChallenge{token:ID.into(),scope:s,item:10304,gear_revision:0}),hold:None,released:false};let v=json!({"hold_id":ID,"source":7,"object":10275,"entered":true,"released":false,"finished":false,"remaining_nanos_at_entry":9000000000i64,"result":"HOLDING_AFTER_REAL_SQL","sql_query_started_epoch_millis":100000,"sql_cutoff_exclusive_epoch_millis":103000,"release_at_epoch_millis":0,"backend_remaining_nanos_at_release":0});let h:Hold=serde_json::from_value(v.clone()).unwrap();a.validate_hold(&h).unwrap();for(k,bad)in [("source",json!(42)),("object",json!(10163)),("hold_id",json!("foreign"))]{let mut mutant=v.clone();mutant[k]=bad;assert!(a.validate_hold(&serde_json::from_value(mutant).unwrap()).is_err());}let mut v=v;v["extra"]=json!(true);assert!(serde_json::from_value::<Hold>(v).is_err());}
 #[test]fn native_process_FILETIME_rejects_reused_owner_identity(){use windows::Win32::{Foundation::{FILETIME,CloseHandle},System::Threading::{OpenProcess,GetProcessTimes,PROCESS_QUERY_LIMITED_INFORMATION}};let pid=std::process::id();let mut c=FILETIME::default();let mut e=FILETIME::default();let mut k=FILETIME::default();let mut u=FILETIME::default();unsafe{let h=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,false,pid).unwrap();GetProcessTimes(h,&mut c,&mut e,&mut k,&mut u).unwrap();CloseHandle(h).unwrap();}let ft=(c.dwHighDateTime as u64)<<32|c.dwLowDateTime as u64;witness(pid,&ft.to_string()).unwrap();assert!(witness(pid,&(ft+1).to_string()).is_err());assert!(witness(0,&ft.to_string()).is_err());}
 struct CleanupFailure;
 impl Control for CleanupFailure{fn arm_expiry(&mut self,_:Scope,_:u64,_:i64,_:Instant)->Result<ExpiryChallenge>{panic!("cancel must precede control")}fn expiry_observation(&mut self,_:&ExpiryChallenge,_:Instant)->Result<ExpiryObservation>{panic!("cancel must precede control")}fn cleanup(&mut self,_:Instant)->Result<()>{Err(err("model_cleanup_failure"))}}
 #[test]fn cancellation_precedes_AUTH_and_control_cleanup_failure_is_visible(){let ca=vec![1,2,3];let p=runner::Plan{endpoint:SocketAddr::from(([127,0,0,1],1)),server_name:"localhost".into(),ca_sha256:Sha256::digest(&ca).into(),ca_der:ca,sources:[7,42],object:10275,reselected_object:10163,initial_epoch:0x8000000000000000,channel:0,weapon:10304,armor:10305,map_id:564,manifest_digest:[1;32],total_timeout:Duration::from_secs(30),exchange_timeout:Duration::from_secs(2),prepare_timeout:Duration::from_secs(10),dto_ack_only:false};let token=super::super::transport::Cancellation::new();token.cancel();let mut credentials=||->Result<super::super::application_protocol::Credentials>{panic!("AUTH must not run")};let r=runner::run_live(&p,&mut credentials,&mut CleanupFailure,token);assert_eq!(r.failure.as_ref().unwrap().code,"cancelled");assert_eq!(r.TLS_connections_opened,0);assert_eq!(r.cleanup_errors.len(),1);assert!(!r.success());let bytes=serde_json::to_vec(&r).unwrap();assert!(!String::from_utf8_lossy(&bytes).contains(ID));}
 #[test]#[ignore="Explicit dedicated modern runtime READY required; no endpoint guessing"]fn actual_Rust_SQL_TLS_external_navigation(){
 let path=PathBuf::from(std::env::var("LITO_R440_READY").expect("explicit ready path required"));let runtime=path.parent().unwrap();let allowed=fs::canonicalize("D:/lito-dev-runtime/20261003-supervision/modern").unwrap();assert!(fs::canonicalize(runtime).unwrap().starts_with(&allowed),"Modern namespace required");
 let project=Path::new(env!("CARGO_MANIFEST_DIR"));let snap:Value=serde_json::from_slice(&fs::read(project.join("logs/continuation-20261003/r441b-external-fixture-snapshot.json")).unwrap()).unwrap();let manifest=hash(snap["candidate"]["manifest_sha256"].as_str().unwrap()).unwrap();
 let(mut adapter,plan)=Adapter::bind(&path,runtime,manifest,true).expect("strict ready binding");let mut credentials=||super::super::application_protocol::Credentials::new("3333".into(),"3333".into()).map_err(|_|err("isolated_fixture_credentials"));let report=runner::run_live(&plan,&mut credentials,&mut adapter,super::super::transport::Cancellation::new());
 let shutdown=adapter.command("SHUTDOWN",&[],Instant::now()+Duration::from_secs(3));let outcome=json!({"state":if report.success()&&shutdown.is_ok(){"RUST_SQL_TLS_RECIPE_PASSED_PENDING_PROCESS_RETIREMENT"}else{"RUST_SQL_TLS_RECIPE_FAILED"},"fixture_id":adapter.ready.fixture_id,"candidate_manifest_sha256":snap["candidate"]["manifest_sha256"],"report":report,"shutdown_requested":shutdown.is_ok(),"renderer_LOADED":false,"GUI":0});let output=runtime.join("modern-rust-report.json");fs::write(&output,serde_json::to_vec_pretty(&outcome).unwrap()).unwrap();assert!(report.success()&&shutdown.is_ok(),"bounded Rust recipe failed; see redacted modern-rust-report.json");
 }
}
