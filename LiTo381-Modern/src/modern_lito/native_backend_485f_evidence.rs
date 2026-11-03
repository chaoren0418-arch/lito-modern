//! Bounded sampling of existing Runtime receipt ledgers. No packet tap or authority.
//! Times are local sampling times; per-ledger receipt order is preserved, cross-ledger
//! order is unknown. Missing observations never prove wire absence or a server ACK.
use crate::modern_lito::native_city_runtime::{Runtime,UiPage,room_player::Operation,java_room_ready as ready,java_room_participants as members};
use serde_json::{json,Value};
use std::{collections::BTreeMap,time::Instant};
pub const MAX_EVENTS:usize=4096;
const MAX_SCOPES:usize=32;
type Scope=(u64,u32,Option<u64>);
#[derive(Default)]struct Cursor {participant:u64,e1:usize,requests:BTreeMap<(u64,u64),(ready::Phase,usize)>,d1_scanned:bool,e1_scanned:bool,d1_gap:bool,e1_gap:bool}
/// Records only whitelisted D1/E1/write/state facts, never authentication data.
pub struct Recorder {
 clock:Instant,events:Vec<Value>,last_us:u64,dropped:u64,cursors:BTreeMap<Scope,Cursor>,
 expected_alias:Vec<u8>,first_failure:Option<Value>,coverage_gap:bool,
 d1_zero:Option<u64>,e100:Option<u64>,world_rejected:Option<u64>,close_witness:Option<u64>,closed:bool,d1_gap:bool,e1_gap:bool,
 last_state:Option<Value>,last_rejection:Option<(u64,Option<u8>,usize)>,
}
impl Recorder {
 pub fn new(expected_alias:&[u8])->Self{Self{clock:Instant::now(),events:vec![],last_us:0,dropped:0,cursors:BTreeMap::new(),expected_alias:expected_alias.iter().copied().take(27).collect(),first_failure:None,coverage_gap:expected_alias.len()>27,d1_zero:None,e100:None,world_rejected:None,close_witness:None,closed:false,d1_gap:expected_alias.len()>27,e1_gap:false,last_state:None,last_rejection:None}}
 /// Return true only when the event is actually retained in the output prefix.
 fn push(&mut self,kind:&'static str,scope:Option<Scope>,data:Value)->bool{
  let now=u64::try_from(self.clock.elapsed().as_micros()).unwrap_or(u64::MAX).max(self.last_us);self.last_us=now;
  if self.events.len()>=MAX_EVENTS{self.dropped=self.dropped.saturating_add(1);self.coverage_gap=true;return false;}
  let (epoch,room,ce)=scope.map_or((None,None,None),|(e,r,c)|(Some(e),Some(r),c));
  self.events.push(json!({"sequence":self.events.len()+1,"observed_us":now,"kind":kind,"epoch":epoch,"room_id":room,"ce_revision":ce,"data":data}));true
 }
 fn admit_scope(&mut self,scope:Scope)->bool{if !self.cursors.contains_key(&scope){if self.cursors.len()>=MAX_SCOPES{self.coverage_gap=true;return false;}self.cursors.insert(scope,Cursor::default());}true}
 pub fn failure(&mut self,category:&'static str){
  if self.first_failure.is_none(){let v=json!({"category":category,"observed_us":u64::try_from(self.clock.elapsed().as_micros()).unwrap_or(u64::MAX),"epoch":null,"room_id":null,"details":"fixed category only; one separately retained bounded failure witness, context unknown"});self.first_failure=Some(v.clone());self.push("first_failure",None,v);}
 }
 pub fn operation(&mut self,r:&Runtime,op:Operation,accepted:Option<bool>){
  let room=r.java_room_entry_snapshot();let scope=r.source_connection_epoch().zip(room.room.as_ref()).map(|(e,v)|(e,v.id,Some(room.revision)));
  self.push(if accepted.is_none(){"operation_intent"}else{"operation_result"},scope,json!({"operation":format!("{op:?}"),"accepted":accepted,"meaning":"callback acceptance is not completed write or server ACK"}));
 }
 pub fn entry_step(&mut self,stage:&'static str,accepted:Option<bool>){self.push("entry_step",None,json!({"stage":stage,"accepted":accepted,"meaning":"local intent/result only; not receipt or ACK"}));}
 pub fn close_intent(&mut self,r:&Runtime){self.observe(r);self.push("close_intent",None,json!({"page":format!("{:?}",r.page()),"meaning":"intent only; confirmation/closed and joins are separate observations"}));}
 fn participants(&mut self,b:&members::Snapshot){
  let scope=(b.epoch,b.room_id,b.ce_revision);if !self.admit_scope(scope){self.d1_gap=true;return;}self.cursors.get_mut(&scope).unwrap().d1_scanned=true;
  let previous=self.cursors[&scope].participant;let mut cursor=previous;
  if b.receipts.len()>members::MAX_RECEIPTS||b.receipts.last().is_some_and(|v|v.sequence<previous){self.coverage_gap=true;self.cursors.get_mut(&scope).unwrap().d1_gap=true;}
  for receipt in b.receipts.iter().take(members::MAX_RECEIPTS).filter(|v|v.sequence>previous){
   if receipt.sequence!=cursor+1{self.coverage_gap=true;self.cursors.get_mut(&scope).unwrap().d1_gap=true;}cursor=receipt.sequence;
   if let members::Packet::State(d)=&receipt.packet {
    let expected=d.display_alias==self.expected_alias;
    let retained=self.push("received_d1",Some(scope),json!({"receipt_revision":receipt.sequence,"sampled_book_revision":b.revision,"state":d.state.wire(),"expected_guest_raw_alias":expected,"authenticated_singleton_context":b.authenticated_singleton_self_qualified,"meaning":"retained admitted receipt; not ACK or server membership proof"}));
    if retained&&expected&&d.state.wire()==0{self.d1_zero=Some(self.events.len()as u64);}else if !retained{self.cursors.get_mut(&scope).unwrap().d1_gap=true;}
   }
  }
  self.cursors.get_mut(&scope).unwrap().participant=cursor;
 }
 fn ready(&mut self,s:&ready::Snapshot,epoch:Option<u64>){
  if let Some(c)=&s.countdown{
   let scope=(c.epoch,c.room_id,Some(c.joined.ce_revision));if self.admit_scope(scope){self.cursors.get_mut(&scope).unwrap().e1_scanned=true;let previous=self.cursors[&scope].e1;
    if c.receipts.len()<previous||c.receipts.len()>1024||c.revision!=c.receipts.len()as u64{self.coverage_gap=true;self.cursors.get_mut(&scope).unwrap().e1_gap=true;}
    for (i,count) in c.receipts.iter().take(1024).enumerate().skip(previous){let retained=self.push("received_e1",Some(scope),json!({"receipt_revision":i+1,"count_byte":count,"sampled_invalidated":c.invalidated,"meaning":"local receipt context only; wire has no room/generation; cross-ledger order unknown"}));if retained&&*count==0{self.e100=Some(self.events.len()as u64);}else if !retained{self.cursors.get_mut(&scope).unwrap().e1_gap=true;}}
    self.cursors.get_mut(&scope).unwrap().e1=c.receipts.len().min(1024);
   }else{self.e1_gap=true;}
  }
  if s.requests.len()>ready::MAX_REQUESTS{self.coverage_gap=true;}
  for q in s.requests.iter().take(ready::MAX_REQUESTS){
   let Some(epoch)=epoch else{self.coverage_gap=true;continue;};let scope=(epoch,q.context.room_id,Some(q.context.joined.ce_revision));if !self.admit_scope(scope){continue;}
   let id=(q.sequence,q.write_serial);let next=(q.phase,q.written_bytes);
   if self.cursors[&scope].requests.get(&id)!=Some(&next){
    self.push("request_write_observation",Some(scope),json!({"request_sequence":q.sequence,"write_serial":q.write_serial,"intent_byte":q.intent.byte(),"participants_revision_at_intent":q.context.participants_revision,"phase":format!("{:?}",q.phase),"written_bytes":q.written_bytes,"frame_bytes":q.frame_bytes,"complete_write_observed":q.phase==ready::Phase::CompletedWrite&&q.written_bytes==q.frame_bytes,"retirement":q.retirement,"meaning":"existing transport progress; not server ACK"}));
    self.cursors.get_mut(&scope).unwrap().requests.insert(id,next);
   }
  }
 }
 /// Drain retained current ledgers or final checkpoints, including receipts arriving
 /// after a callback and multiple E1/D1 receipts between UI polls. Never reads a file.
 pub fn observe(&mut self,r:&Runtime){
  let current=r.java_room_participants_snapshot();let archived=if current.is_none(){r.last_java_room_participants_checkpoint()}else{None};let book=current.as_ref().or(archived.as_ref());
  if let Some(b)=book{self.participants(b);}
  let s=r.java_room_ready_snapshot();let archived=if s.requests.is_empty()&&s.countdown.is_none(){r.last_java_room_ready_checkpoint()}else{None};
  let epoch=if archived.is_some(){book.map(|b|b.epoch)}else{r.source_connection_epoch().or_else(||book.map(|b|b.epoch))};self.ready(archived.as_ref().unwrap_or(&s),epoch);
  if let Some(rejection)=r.last_source_rejection(){self.rejection(rejection.epoch,rejection.opcode,rejection.body_bytes);}
  let io=r.player_fixture_snapshot();let state=json!({"page":format!("{:?}",r.page()),"source_epoch":r.source_connection_epoch(),"IO_started":io.workers_started,"IO_joined":io.workers_joined,"queued_events":io.queued_events,"discarded_late_events":io.discarded_late_events});
  self.state(state);
 }
 fn rejection(&mut self,epoch:u64,opcode:Option<u8>,body_bytes:usize){let key=(epoch,opcode,body_bytes);if self.last_rejection!=Some(key){self.last_rejection=Some(key);let retained=self.push("source_dispatch_rejected",None,json!({"epoch":epoch,"opcode":opcode,"body_bytes":body_bytes,"reason":"withheld; opcode/length only"}));if retained&&opcode==Some(40){self.world_rejected=Some(self.events.len()as u64);}self.failure("source_dispatch_rejection");}}
 fn state(&mut self,state:Value){
  let closed=state["page"]=="Closed"&&state["IO_started"].as_u64().is_some()&&state["IO_started"]==state["IO_joined"]&&state["queued_events"]==0;
  if self.last_state.as_ref()!=Some(&state)&&self.push("runtime_close_io_state",None,state.clone()){self.last_state=Some(state.clone());if closed{self.close_witness=Some(self.events.len()as u64);}}
  self.closed=closed&&self.last_state.as_ref()==Some(&state);
 }
 fn observation(&self,witness:Option<u64>,d1:bool)->Value{
  let scopes=self.cursors.iter().filter(|(_,c)|if d1{c.d1_scanned}else{c.e1_scanned}).map(|(s,c)|json!({"epoch":s.0,"room_id":s.1,"ce_revision":s.2,"last_receipt_revision":if d1{c.participant}else{c.e1 as u64},"gap":if d1{c.d1_gap}else{c.e1_gap}})).collect::<Vec<_>>();
  let gap=if d1{self.d1_gap}else{self.e1_gap}||self.dropped>0||scopes.iter().any(|s|s["gap"]==true);
  json!({"status":if witness.is_some(){"observed"}else if self.closed&&self.close_witness.is_some()&&!scopes.is_empty()&&!gap{"not_observed"}else{"unknown"},"witness_event_sequence":witness,"scanned_scopes":scopes,"coverage_gap":gap,"scope":"this receipt domain's scanned retained ledgers only; not wire/server absence","coverage_complete_for_entire_connection":"unknown"})
 }
 pub fn report(&self)->Value{json!({"schema":"modern.r532.bounded-runtime-evidence.v1","events":self.events,"maximum_events":MAX_EVENTS,"maximum_scopes":MAX_SCOPES,"dropped_events":self.dropped,"coverage_gap":self.coverage_gap,"first_failure":self.first_failure,"expected_guest_d1_zero_receipt":self.observation(self.d1_zero,true),"e100_receipt":self.observation(self.e100,false),"WORLD_packet_rejection":{"status":if self.world_rejected.is_some(){"observed"}else{"unknown"},"witness_event_sequence":self.world_rejected,"meaning":"retained rejection opcode40 only; neither validates a WORLD payload nor proves admission/complete raw packet coverage"},"WORLD_admitted":{"status":"unknown","reason":"Runtime exposes no dedicated WORLD admission observation; no GUI/backend WORLD evidence"},"close_and_IO_join":{"status":if self.close_witness.is_some(){"observed"}else{"unknown"},"witness_event_sequence":self.close_witness,"meaning":"retained Closed/IO snapshot; no join conclusion from a dropped snapshot"},"timing":"observed_us is monotonic local sample time; batch receipts share sampling interval; event order across D1/E1/write ledgers is unknown","server_ACK":"unknown","atomic_GET":"unknown","credentials_or_authentication_packets_recorded":false})}
}

#[cfg(test)]
fn save_test_report(prefix:&str,name:&str,v:&Value){
 use std::{fs,io::{self,Write},path::PathBuf,sync::OnceLock};
 static OUTPUT:OnceLock<PathBuf>=OnceLock::new();
 let output=OUTPUT.get_or_init(||{
  let base=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/scratch/native-backend-485f-evidence");
  fs::create_dir_all(&base).unwrap();
  for sequence in 0..10_000 {
   let candidate=base.join(format!("run-{}-{sequence}",std::process::id()));
   match fs::create_dir(&candidate){Ok(())=>return candidate,Err(e)if e.kind()==io::ErrorKind::AlreadyExists=>continue,Err(e)=>panic!("create owned report directory: {e}")}
  }
  panic!("owned test report directory allocation exhausted")
 });
 let path=output.join(format!("{prefix}-check-{name}.json"));
 let mut file=fs::OpenOptions::new().write(true).create_new(true).open(path).unwrap();
 file.write_all(&serde_json::to_vec_pretty(v).unwrap()).unwrap();
}

#[cfg(test)]mod r531_evidence_checks {
 use super::*;
 use crate::modern_lito::native_city_runtime::java_room_settings::JoinedContext;
 pub(super) fn joined()->JoinedContext{JoinedContext{ce_revision:7,source:b"fixture".to_vec(),source_revision:2,roster_revision:3,selected_slot:0,selected_alias:b"ExpectedGuest".to_vec(),navigation_revision:4,channel:0}}
 pub(super) fn book()->members::Snapshot{members::Snapshot{epoch:2,room_id:1,origin:members::Origin::ExplicitObservation,ce_revision:Some(7),revision:0,completeness:members::Completeness::PossiblyIncomplete,rows:vec![],ignored_unknown_states:0,last_unknown_state:None,ignored_duplicate_rows:0,ignored_ambiguous_states:0,last_ambiguous_state:None,receipts:vec![],owner_display:None,owner_link:None,joined_room_invalidated:false,synthetic_self_expectation_qualified:false,stream_arrivals:0,last_stream_change:None,authenticated_singleton_self_qualified:false}}
 pub(super) fn state(sequence:u64,state:members::MemberState)->members::Receipt{members::Receipt{sequence,packet:members::Packet::State(members::StateReceipt{display_alias:b"ExpectedGuest".to_vec(),state,raw_body:b"R531_AUTH_SENTINEL".to_vec()})}}
 pub(super) fn request()->ready::Request{ready::Request{sequence:1,write_serial:42,intent:ready::ParticipantIntent::Ready,producer_sha256:ready::PRODUCER_SHA,context:ready::ContextKey{room_id:1,joined:joined(),participants_revision:2},phase:ready::Phase::Queued,written_bytes:0,raw_body:b"R531_AUTH_SENTINEL".to_vec(),frame_bytes:4,retirement:None}}
 fn save(name:&str,v:&Value){super::save_test_report("r531",name,v);}
 #[test]fn late_d1_e100_and_complete_write_drain_retained_ledgers_once(){
  let mut r=Recorder::new(b"ExpectedGuest");let mut b=book();let mut s=ready::Snapshot{revision:1,requests:vec![request()],countdown:None};
  r.ready(&s,Some(2));assert!(!r.events[0]["data"]["complete_write_observed"].as_bool().unwrap());
  s.requests[0].written_bytes=1;r.ready(&s,Some(2));s.requests[0].written_bytes=4;s.requests[0].phase=ready::Phase::CompletedWrite;r.ready(&s,Some(2));
  b.revision=2;b.receipts=vec![state(1,members::MemberState::Ready),state(2,members::MemberState::NotReady)];
  s.countdown=Some(ready::Countdown{epoch:2,room_id:1,joined:joined(),revision:2,count:None,receipts:vec![3,0],invalidated:true});
  r.participants(&b);r.ready(&s,Some(2));let count=r.events.len();r.participants(&b);r.ready(&s,Some(2));assert_eq!(r.events.len(),count);
  let v=r.report();assert_eq!(v["expected_guest_d1_zero_receipt"]["status"],"observed");assert_eq!(v["e100_receipt"]["status"],"observed");assert_eq!(v["WORLD_admitted"]["status"],"unknown");
  let d1=r.events.iter().filter(|e|e["kind"]=="received_d1").collect::<Vec<_>>();assert_eq!(d1.len(),2);assert_eq!(d1[1]["data"]["state"],0);
  let e1=r.events.iter().filter(|e|e["kind"]=="received_e1").collect::<Vec<_>>();assert_eq!(e1.len(),2);assert_eq!(e1[1]["data"]["count_byte"],0);
  assert!(r.events.windows(2).all(|e|e[0]["observed_us"].as_u64()<=e[1]["observed_us"].as_u64()));assert!(!v.to_string().contains("R531_AUTH_SENTINEL"));save("late-ledgers",&v);
 }
 #[test]fn unseen_is_unknown_without_coverage_and_scoped_not_observed_is_not_wire_absence(){
  let mut r=Recorder::new(b"ExpectedGuest");r.closed=true;assert_eq!(r.report()["e100_receipt"]["status"],"unknown");r.participants(&book());assert_eq!(r.report()["e100_receipt"]["status"],"unknown");
  r.coverage_gap=true;assert_eq!(r.report()["e100_receipt"]["status"],"unknown");assert_eq!(r.report()["WORLD_admitted"]["status"],"unknown");save("coverage",&r.report());
 }
 #[test]fn bounded_journal_retains_first_failure_without_auth_data_or_false_negatives(){
  let mut r=Recorder::new(b"ExpectedGuest");for _ in 0..MAX_EVENTS+4{r.push("finite_budget_probe",None,json!({}));}r.failure("first_refused_operation");r.failure("later_failure");
  let v=r.report();assert_eq!(v["events"].as_array().unwrap().len(),MAX_EVENTS);assert!(v["dropped_events"].as_u64().unwrap()>=4);assert_eq!(v["first_failure"]["category"],"first_refused_operation");assert_eq!(v["e100_receipt"]["status"],"unknown");assert_eq!(v["WORLD_admitted"]["status"],"unknown");
  save("bounded",&json!({"event_count":r.events.len(),"maximum_events":MAX_EVENTS,"dropped_events":r.dropped,"first_failure":v["first_failure"],"e100_receipt":v["e100_receipt"],"WORLD_admitted":v["WORLD_admitted"]}));
 }
 #[test]fn actual_runtime_rejection_close_and_join_remain_distinct_from_world_admission(){
  use crate::modern_lito::{native_multisector_resources::Inputs,native_resources::BytePin};use sha2::{Digest,Sha256};use std::{thread,time::Duration};
  let p=std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));let manifest=p.join("logs/continuation-20261003/r486-map-binding.json");let bytes=std::fs::read(&manifest).unwrap();let mut runtime=Runtime::start(Inputs{project:p,manifest,manifest_pin:BytePin{bytes:bytes.len()as u64,sha256:Sha256::digest(bytes).into()}}).unwrap();
  let end=Instant::now()+Duration::from_secs(20);while runtime.page()!=UiPage::Menu{assert!(Instant::now()<end);runtime.poll();thread::sleep(Duration::from_millis(2));}
  let mut r=Recorder::new(b"ExpectedGuest");r.observe(&runtime);let epoch=runtime.begin_source_connection_candidate().unwrap();runtime.feed_source_stream_candidate(&epoch,&[7,0,1,1,0,0,0]).unwrap();runtime.feed_source_stream_candidate(&epoch,&[3,0,41]).unwrap();r.observe(&runtime);
  assert_eq!(runtime.last_source_rejection().unwrap().opcode,Some(40));assert_eq!(r.report()["WORLD_packet_rejection"]["status"],"observed");assert_eq!(r.report()["WORLD_admitted"]["status"],"unknown");r.close_intent(&runtime);runtime.handle(crate::modern_lito::native_city_runtime::Action::Close).unwrap();
  let end=Instant::now()+Duration::from_secs(20);while runtime.page()!=UiPage::Closed{assert!(Instant::now()<end);runtime.poll();r.observe(&runtime);thread::sleep(Duration::from_millis(2));}r.observe(&runtime);
  assert_eq!(r.report()["close_and_IO_join"]["status"],"observed");assert_eq!(runtime.player_fixture_snapshot().workers_started,0);assert_eq!(r.report()["e100_receipt"]["status"],"unknown");save("runtime-close",&r.report());
 }
}

#[cfg(test)]mod r532_counterexample_checks {
 use super::*;use super::r531_evidence_checks::{book,joined,state,request};
 fn closed(r:&mut Recorder){r.state(json!({"page":"Closed","source_epoch":null,"IO_started":0,"IO_joined":0,"queued_events":0,"discarded_late_events":0}));}
 fn countdown(count:u8)->ready::Snapshot{ready::Snapshot{revision:0,requests:vec![],countdown:Some(ready::Countdown{epoch:2,room_id:1,joined:joined(),revision:1,count:if count==0{None}else{Some(count)},receipts:vec![count],invalidated:false})}}
 fn save(name:&str,v:&Value){super::save_test_report("r532",name,v);}
 #[test]fn d1_e1_scanned_scopes_and_gaps_are_independent(){
  let mut d1=Recorder::new(b"ExpectedGuest");d1.participants(&book());closed(&mut d1);let d1v=d1.report();assert_eq!(d1v["expected_guest_d1_zero_receipt"]["status"],"not_observed");assert_eq!(d1v["e100_receipt"]["status"],"unknown");assert!(d1v["e100_receipt"]["scanned_scopes"].as_array().unwrap().is_empty());
  let mut e1=Recorder::new(b"ExpectedGuest");e1.ready(&countdown(3),Some(2));closed(&mut e1);let e1v=e1.report();assert_eq!(e1v["e100_receipt"]["status"],"not_observed");assert_eq!(e1v["expected_guest_d1_zero_receipt"]["status"],"unknown");assert!(e1v["expected_guest_d1_zero_receipt"]["scanned_scopes"].as_array().unwrap().is_empty());
  let mut dgap=Recorder::new(b"ExpectedGuest");let mut b=book();b.revision=2;b.receipts=vec![state(2,members::MemberState::Ready)];dgap.participants(&b);dgap.ready(&countdown(3),Some(2));closed(&mut dgap);let dgv=dgap.report();assert_eq!(dgv["expected_guest_d1_zero_receipt"]["status"],"unknown");assert_eq!(dgv["e100_receipt"]["status"],"not_observed");
  let mut egap=Recorder::new(b"ExpectedGuest");egap.participants(&book());let mut s=countdown(3);s.countdown.as_mut().unwrap().revision=2;egap.ready(&s,Some(2));closed(&mut egap);let egv=egap.report();assert_eq!(egv["e100_receipt"]["status"],"unknown");assert_eq!(egv["expected_guest_d1_zero_receipt"]["status"],"not_observed");
  save("domain-coverage",&json!({"D1_only":d1v,"E1_only":e1v,"D1_gap_E1_intact":dgv,"E1_gap_D1_intact":egv,"scope":"constructed ledger classification counterexamples only; no Runtime/network/GUI"}));
 }
 fn fill(r:&mut Recorder){while r.events.len()<MAX_EVENTS{r.push("finite_budget_probe",None,json!({}));}}
 fn assert_witness(v:&Value,field:&str,kind:&str){let seq=v[field]["witness_event_sequence"].as_u64().unwrap();let e=&v["events"][(seq-1)as usize];assert_eq!(e["sequence"],seq);assert_eq!(e["kind"],kind);}
 fn brief(v:&Value)->Value{json!({"event_count":v["events"].as_array().unwrap().len(),"dropped_events":v["dropped_events"],"D1zero":v["expected_guest_d1_zero_receipt"],"E100":v["e100_receipt"],"WORLD_rejection":v["WORLD_packet_rejection"],"Close":v["close_and_IO_join"],"first_failure":v["first_failure"],"retained_nonprobe_events":v["events"].as_array().unwrap().iter().filter(|e|e["kind"]!="finite_budget_probe").collect::<Vec<_>>()})}
 #[test]fn dropped_positive_receipts_rejection_close_and_writes_never_claim_observed(){
  let mut full=Recorder::new(b"ExpectedGuest");fill(&mut full);let mut b=book();b.revision=1;b.receipts=vec![state(1,members::MemberState::NotReady)];full.participants(&b);let mut s=countdown(0);let mut q=request();q.phase=ready::Phase::CompletedWrite;q.written_bytes=4;s.requests.push(q);full.ready(&s,Some(2));full.rejection(2,Some(40),1);closed(&mut full);
  let v=full.report();for field in ["expected_guest_d1_zero_receipt","e100_receipt","WORLD_packet_rejection","close_and_IO_join"]{assert_eq!(v[field]["status"],"unknown");assert!(v[field]["witness_event_sequence"].is_null());}assert!(v["events"].as_array().unwrap().iter().all(|e|e["kind"]=="finite_budget_probe"));assert_eq!(v["first_failure"]["category"],"source_dispatch_rejection");
  let mut retained=Recorder::new(b"ExpectedGuest");retained.participants(&b);retained.ready(&countdown(0),Some(2));retained.rejection(2,Some(40),1);closed(&mut retained);fill(&mut retained);retained.entry_step("dropped_later_intent",None);let rv=retained.report();
  for (field,kind) in [("expected_guest_d1_zero_receipt","received_d1"),("e100_receipt","received_e1"),("WORLD_packet_rejection","source_dispatch_rejected"),("close_and_IO_join","runtime_close_io_state")]{assert_eq!(rv[field]["status"],"observed");assert_witness(&rv,field,kind);}
  let mut partial=Recorder::new(b"ExpectedGuest");partial.participants(&b);fill(&mut partial);partial.ready(&countdown(0),Some(2));let pv=partial.report();assert_eq!(pv["expected_guest_d1_zero_receipt"]["status"],"observed");assert_witness(&pv,"expected_guest_d1_zero_receipt","received_d1");assert_eq!(pv["e100_receipt"]["status"],"unknown");assert!(pv["e100_receipt"]["witness_event_sequence"].is_null());
  assert!(!v.to_string().contains("R531_AUTH_SENTINEL"));save("retained-evidence",&json!({"all_positives_dropped":brief(&v),"prior_positive_witnesses_retained":brief(&rv),"D1_retained_E100_dropped":brief(&pv),"scope":"finite constructed ledgers/rejection/Closed snapshots; no Runtime/socket/GUI case"}));
 }
}
