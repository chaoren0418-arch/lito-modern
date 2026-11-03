use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};
use std::{thread, time::{Duration, Instant}};
use lito_modern::modern_lito::native_city_runtime::{Runtime, java_room_ready as ready, java_room_entry};

/// Consumer observation policy, never a Runtime receipt/book/epoch writer.
#[derive(Default)]
pub struct Policy {
    pub ready_full: bool, pub cancel_queued: bool, pub cancel_full: bool,
    pub e103: bool, pub e100: bool, pub own_d1_one: bool, pub own_d1_zero: bool,
}
impl Policy {
    pub fn e1(&mut self, count: u8) -> Result<()> {
        ensure!(self.ready_full && self.own_d1_one, "E1 before local Ready fullwrite and actual own D1one observation");
        match count {
            3 => {ensure!(!self.e103 && !self.e100, "repeated E103 unqualified");self.e103=true;},
            2|1 => anyhow::bail!("actual E102/E101 observed; strict first-case deviation, retain tick and stop"),
            0 => {ensure!(self.e103 && self.cancel_queued && !self.e100, "E100 without this cancellation or repeated E100 unqualified");self.e100=true;},
            _ => anyhow::bail!("E1 value outside this finite 3/2/1/0 case"),
        }
        Ok(())
    }
    pub fn complete(&self) -> bool {self.ready_full && self.cancel_full && self.e103 && self.e100 && self.own_d1_zero}
}

pub struct Trace {
    pub stage: &'static str, pub events: Vec<Value>, pub requests: Vec<Value>,
    pub first_failure: Option<Value>, pub policy: Policy,
    start: Instant, seen_e1: usize, seen_members: u64,
    pub room_epoch: Option<u64>, pub cancel_member_revision: u64,
    pub e103_observed_us: Option<u64>, pub cancel_queued_us: Option<u64>,
    pub watch_completed: bool,
    baseline_full: bool, baseline_d1_zero: bool, baseline_member_revision: u64,
}
impl Trace {
    pub fn new() -> Self {Self{stage:"runtime_boot",events:vec![],requests:vec![],first_failure:None,policy:Policy::default(),start:Instant::now(),seen_e1:0,seen_members:0,room_epoch:None,cancel_member_revision:0,e103_observed_us:None,cancel_queued_us:None,watch_completed:false,baseline_full:false,baseline_d1_zero:false,baseline_member_revision:0}}
    fn us(&self) -> u64 {self.start.elapsed().as_micros().min(u64::MAX as u128) as u64}
    pub fn event(&mut self, kind: &str, details: Value) {self.events.push(json!({"observed_us":self.us(),"stage":self.stage,"kind":kind,"details":details}));}
    pub fn failure(&mut self, details: Value) {if self.first_failure.is_none(){self.first_failure=Some(json!({"observed_us":self.us(),"stage":self.stage,"details":details}));}}
    pub fn observe(&mut self, r: &Runtime, own: &[u8], host: &[u8]) -> Result<()> {
        // Capture partial observations before inspecting retirement/rejection. A
        // success predicate never outranks failure from the same Runtime poll.
        let snap=r.java_room_ready_snapshot();
        let snap=if snap.countdown.is_none(){r.last_java_room_ready_checkpoint().unwrap_or(snap)}else{snap};
        // Completion and E1 can be delivered together in one Runtime poll.
        // Inspect real request completion before applying the E1 policy.
        let observed_us=self.us();let mut completions=vec![];let mut retired_request=None;
        for record in &mut self.requests {
            if let Some(req)=snap.requests.iter().find(|v|Some(v.sequence)==record["sequence"].as_u64()) {
                let just_completed=record["phase"]!="CompletedWrite" && req.phase==ready::Phase::CompletedWrite;
                record["phase"]=json!(format!("{:?}",req.phase));
                record["written_bytes"]=json!(req.written_bytes);
                record["retirement"]=json!(req.retirement);
                if just_completed {record["complete_write_observed_us"]=json!(observed_us);completions.push(json!({"sequence":req.sequence,"purpose":record["purpose"],"written_bytes":req.written_bytes}));}
                if req.phase==ready::Phase::Retired && retired_request.is_none(){retired_request=Some(json!({"sequence":req.sequence,"written_bytes":req.written_bytes,"retirement":req.retirement}));}
                if req.phase==ready::Phase::CompletedWrite && req.written_bytes==4 {
                    match record["purpose"].as_str() {
                        Some("Ready")=>self.policy.ready_full=true,
                        Some("Cancel")=>self.policy.cancel_full=true,
                        Some("Baseline")=>self.baseline_full=true,
                        _=>{},
                    }
                }
            }
        }
        for e in completions {self.event("complete_write_observed",e);}
        if self.policy.ready_full && r.java_room_participants_snapshot().is_some_and(|m|m.rows.iter().any(|row|row.participant.display_alias==own && row.state.wire()==1 && row.last_state_receipt.as_ref().is_some_and(|s|s.state.wire()==1))) {self.policy.own_d1_one=true;}
        let mut policy_error=None;
        if let Some(c)=snap.countdown {
            if c.receipts.len()<self.seen_e1 {policy_error=Some(anyhow::anyhow!("countdown observation reset; no epoch reuse"));}
            for count in c.receipts.iter().skip(self.seen_e1).copied() {
                self.event("actual_E1",json!({"body_hex":format!("e1{count:02x}"),"count":count,"local_epoch_label":c.epoch,"local_room_label":c.room_id,"wire_room_generation_sequence":false,"snapshot_revision":c.revision,"count_now":c.count,"invalidated":c.invalidated}));
                if count==3 && self.e103_observed_us.is_none(){self.e103_observed_us=Some(self.us());}
                if policy_error.is_none(){if let Err(e)=self.policy.e1(count){policy_error=Some(e);}}
            }
            self.seen_e1=c.receipts.len();
            if self.policy.e100 && c.count.is_some(){policy_error=Some(anyhow::anyhow!("E100 did not clear countdown projection"));}
        }
        if let Some(m)=r.java_room_participants_snapshot() {
            if m.revision!=self.seen_members {
                let mut seen=vec![];
                for row in m.rows.iter().filter(|v|v.participant.display_alias==own || v.participant.display_alias==host) {
                    let self_row=row.participant.display_alias==own;
                    let d1=row.last_state_receipt.as_ref().map(|s|super::hex_bytes(&s.raw_body));
                    seen.push(json!({"role":if self_row{"Guest"}else{"Host"},"state":row.state.wire(),"last_D1_body_hex":d1}));
                    if self_row && self.policy.cancel_queued && m.revision>self.cancel_member_revision && row.state.wire()==0 && row.last_state_receipt.as_ref().is_some_and(|s|s.state.wire()==0) {self.policy.own_d1_zero=true;}
                    if self_row && self.baseline_member_revision>0 && m.revision>self.baseline_member_revision && row.state.wire()==0 && row.last_state_receipt.as_ref().is_some_and(|s|s.state.wire()==0) {self.baseline_d1_zero=true;}
                }
                self.event("participants_snapshot",json!({"room_id":m.room_id,"revision":m.revision,"rows":seen,"snapshot_only_not_all_intermediate_D1":true}));
                self.seen_members=m.revision;
            }
        }
        // Check writes before E1 policy using the post-poll snapshot. E1 and the
        // completion report can arrive together in this bounded poll.
        if let Some(rejection)=r.last_source_rejection() {
            self.failure(json!({"kind":"first_source_rejection","opcode":rejection.opcode,"body_bytes":rejection.body_bytes,"epoch":rejection.epoch,"reason":rejection.reason}));
            anyhow::bail!("first unsupported/WORLD/start/result/refused receipt; stop without retry");
        }
        if self.room_epoch.is_some() && (r.source_connection_epoch()!=self.room_epoch || r.java_room_entry_snapshot().phase!=java_room_entry::Phase::RoomReceipts) {
            self.failure(json!({"kind":"qualified_room_or_epoch_retired"}));
            anyhow::bail!("qualified room/source retired");
        }
        let stats=r.player_fixture_snapshot();
        if stats.workers_started>0 && !stats.active {
            self.failure(json!({"kind":"owned_socket_inactive","workers_started":stats.workers_started,"workers_joined":stats.workers_joined}));
            anyhow::bail!("owned socket ended; no automatic retry");
        }
        if let Some(e)=retired_request {self.failure(json!({"kind":"ready_write_retired","request":e}));anyhow::bail!("ready/baseline/cancel write retired; retain partial, no retry");}
        if let Some(e)=policy_error {self.failure(json!({"kind":"finite_E1_policy","error":e.to_string()}));return Err(e);}
        if self.stage=="finite_post_cancel_observation" {
            let m=r.java_room_participants_snapshot().context("postcancel private members absent")?;
            ensure!(m.rows.iter().find(|v|v.participant.display_alias==host).is_some_and(|v|v.state.wire()==1) && m.rows.iter().find(|v|v.participant.display_alias==own).is_some_and(|v|v.state.wire()==0),"postcancel Host/Guest projection changed; first failure stops");
        }
        ensure!(self.events.len()<=2048,"bounded observation event budget");
        Ok(())
    }
}

pub fn wait(r: &mut Runtime, t: &mut Trace, own: &[u8], host: &[u8], predicate: impl Fn(&Runtime,&Trace)->bool) -> Result<()> {
    let end=Instant::now()+Duration::from_secs(15);
    loop {
        t.observe(r,own,host)?;
        if Instant::now()>=end {t.failure(json!({"kind":"bounded_stage_timeout"}));anyhow::bail!("bounded stage timeout; no retry");}
        if predicate(r,t){return Ok(());}
        r.poll();
        // Immediate health/receipt inspection, with no sleep on predicate hit.
        t.observe(r,own,host)?;
        if Instant::now()>=end {t.failure(json!({"kind":"bounded_stage_timeout"}));anyhow::bail!("bounded stage timeout after poll; no retry");}
        if predicate(r,t){return Ok(());}
        thread::sleep(Duration::from_millis(2));
    }
}

#[derive(Clone,Copy,Debug)] enum Purpose {Baseline,Ready,Cancel}
fn send(r: &mut Runtime, t: &mut Trace, purpose: Purpose) -> Result<()> {
    let intent=match purpose {Purpose::Ready=>ready::ParticipantIntent::Ready,_=>ready::ParticipantIntent::NotReady};
    let before=r.java_room_participants_snapshot().context("private members absent")?;
    let c=r.java_room_ready_control(intent).context("current typed Ready/Cancel control absent")?;
    let counter=r.source_send_snapshot().counter.context("sender counter absent")?;
    r.handle_java_room_ready_control(&c)?;
    ensure!(r.java_room_participants_snapshot()==Some(before.clone()),"optimistic D1 mutation");
    let epoch=r.source_connection_candidate_token().context("owned source absent")?;
    let ticket=r.source_pending_write_ticket(&epoch)?.context("pending ready ticket absent")?;
    let frame=r.source_write_view(&epoch,&ticket)?.bytes()?.to_vec();
    let after=r.source_send_snapshot().counter.context("sender counter absent")?;
    let snap=r.java_room_ready_snapshot();let req=snap.requests.last().context("new request absent")?;
    // Save partial queued intent before waiting for transport completion/receipt.
    t.requests.push(json!({"purpose":format!("{purpose:?}"),"sequence":req.sequence,"write_serial":req.write_serial,"intent_byte":intent.byte(),"body_hex":super::hex_bytes(&req.raw_body),"frame_bytes":frame.len(),"cipher_frame_hex":super::hex_bytes(&frame),"counter_before":counter,"counter_after":after,"phase":"Queued","written_bytes":0,"retirement":null,"member_revision_before":before.revision,"queued_us":t.us(),"server_acceptance_ack":false}));
    match purpose {
        Purpose::Cancel=>{t.policy.cancel_queued=true;t.cancel_member_revision=before.revision;t.cancel_queued_us=Some(t.us());},
        Purpose::Baseline=>t.baseline_member_revision=before.revision,
        Purpose::Ready=>{},
    }
    t.event("typed_request_queued",json!({"state":intent.byte(),"sequence":req.sequence}));
    ensure!(frame.len()==4 && frame[..2]==[4,0] && after==counter+2,"D2 frame/counter boundary failed");
    Ok(())
}

pub fn cancel_case(r: &mut Runtime, t: &mut Trace, own: &[u8], host: &[u8], synchronized_host_gate: impl Fn(&Runtime)->Result<Option<Value>>) -> Result<()> {
    t.stage="qualified_room_initial_Host_notready_Guest_notready";
    wait(r,t,own,host,|r,_|r.java_room_ready_control(ready::ParticipantIntent::NotReady).is_some() && r.java_room_participants_snapshot().is_some_and(|m|m.rows.iter().any(|v|v.participant.display_alias==host && v.state.wire()==0)))?;
    let m=r.java_room_participants_snapshot().context("members absent")?;
    ensure!(m.rows.iter().filter(|v|v.participant.display_alias==own).count()==1 && m.rows.iter().find(|v|v.participant.display_alias==own).is_some_and(|v|v.state.wire()==0),"Guest initial unique notready row required");
    ensure!(r.java_room_ready_snapshot().countdown.is_none(),"unexpected countdown before GuestReady");
    t.room_epoch=r.source_connection_epoch();ensure!(t.room_epoch.is_some(),"qualified room epoch absent");
    let key=r.presentation_key();ensure!(r.request_java_room_owner_start().is_err() && r.presentation_key()==key,"owner refusal boundary changed");
    t.stage="GuestNotReady_baseline_fullwrite_and_actual_D1zero";send(r,t,Purpose::Baseline)?;
    wait(r,t,own,host,|_,t|t.baseline_full && t.baseline_d1_zero)?;
    t.stage="await_actual_Host_D1one_then_accepted_two_peer_WAITING_gate";
    wait(r,t,own,host,|r,_|r.java_room_participants_snapshot().is_some_and(|m|m.rows.iter().any(|v|v.participant.display_alias==host && v.state.wire()==1 && v.last_state_receipt.as_ref().is_some_and(|s|s.state.wire()==1))))?;
    let gate_deadline=Instant::now()+Duration::from_secs(15);
    loop {
        t.observe(r,own,host)?;
        if let Some(gate)=synchronized_host_gate(r)? {t.event("accepted_supplier_synchronized_host_gate",gate);break;}
        if Instant::now()>=gate_deadline {t.failure(json!({"kind":"Host_marker_wait_timeout"}));anyhow::bail!("Host marker timeout; hold Guest0, stop once");}
        r.poll();t.observe(r,own,host)?;thread::sleep(Duration::from_millis(2));
    }
    let m=r.java_room_participants_snapshot().context("preReady members absent")?;
    ensure!(m.rows.iter().find(|v|v.participant.display_alias==own).is_some_and(|v|v.state.wire()==0) && m.rows.iter().find(|v|v.participant.display_alias==host).is_some_and(|v|v.state.wire()==1),"Host marker raced current Host1/Guest0 projection");
    t.stage="GuestReady_queued_wait_fullwrite";send(r,t,Purpose::Ready)?;
    wait(r,t,own,host,|_,t|t.policy.ready_full)?;
    t.stage="await_actual_E103";wait(r,t,own,host,|_,t|t.policy.e103)?;
    // No D1 wait, timer, extra poll or sleep between observed E103 and typed0.
    t.stage="immediate_typed_Cancel0";send(r,t,Purpose::Cancel)?;
    t.stage="await_Cancel_fullwrite_actual_D1zero_E100";
    wait(r,t,own,host,|_,t|t.policy.complete())?;
    let m=r.java_room_participants_snapshot().context("cancelled room members absent")?;
    ensure!(m.rows.iter().find(|v|v.participant.display_alias==host).is_some_and(|v|v.state.wire()==1),"Host readiness unexpectedly changed");
    t.stage="finite_post_cancel_observation";
    let end=Instant::now()+Duration::from_secs(4);
    while Instant::now()<end {r.poll();t.observe(r,own,host)?;thread::sleep(Duration::from_millis(2));}
    t.watch_completed=true;t.event("finite_watch_completed",json!({"milliseconds":4000,"queuedtickdrain_or_sameepochfreshness_proven":false,"WORLD_impossible":false}));
    t.stage="case_complete";Ok(())
}

#[cfg(test)] mod r528_cold_sync_boundaries {
    use super::*;
    #[test] fn e103_requires_ready_fullwrite_and_independent_d1one() {
        let mut p=Policy::default();assert!(p.e1(3).is_err());assert!(!p.complete());
        p.ready_full=true;assert!(p.e1(3).is_err());p.own_d1_one=true;p.e1(3).unwrap();assert!(p.e1(0).is_err());
        p.cancel_queued=true;p.e1(0).unwrap();assert!(!p.own_d1_zero && !p.complete());
        p.cancel_full=true;assert!(!p.complete());p.own_d1_zero=true;assert!(p.complete());
    }
    #[test] fn strict_case_stops_on_positive_ticks_even_after_cancel_queued() {
        let mut p=Policy{ready_full:true,own_d1_one:true,..Default::default()};assert!(p.e1(2).is_err());
        p.e1(3).unwrap();p.cancel_queued=true;
        assert!(p.e1(2).is_err());assert!(p.e1(1).is_err());assert!(p.e103 && !p.e100 && !p.complete());
        p.e1(0).unwrap();assert!(p.e1(1).is_err());assert!(p.e1(3).is_err());assert!(p.e1(255).is_err());
    }
    #[test] fn e100_is_not_d1zero_nor_writecompletion() {
        let mut p=Policy{ready_full:true,own_d1_one:true,cancel_queued:true,..Default::default()};
        p.e1(3).unwrap();p.e1(0).unwrap();assert!(!p.own_d1_zero && !p.cancel_full && !p.complete());
        p.own_d1_zero=true;assert!(!p.complete());p.cancel_full=true;assert!(p.complete());
    }
}
