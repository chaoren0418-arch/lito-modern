use super::*;
/// Isolated shadow controller checks; no transport or authority is started.
fn pending_join_departure_replay() -> anyhow::Result<serde_json::Value> {
    use super::super::native_room::{EarlyRoster, Member};
    fn pending(room:u32)->LoginController {
        let mut c=LoginController::default();c.state.phase=LoginPhase::RoomJoining;
        c.state.selected_character=Some("ProbeMage".into());c.requested_room=Some(room);
        c.deadline=Some(Instant::now()+Duration::from_secs(8));c
    }
    fn departed(room:u32,name:&str)->Vec<u8>{
        let mut b=vec![0xd0];b.extend(room.to_le_bytes());b.extend(name.as_bytes());b.push(0);b
    }
    fn joined(room:u32)->Vec<u8>{
        let mut b=vec![0xce,0];b.extend(room.to_le_bytes());b.extend(b"ProbeRoom\0");
        b.extend([0,0]);b.extend(2u16.to_le_bytes());b.extend(8u16.to_le_bytes());
        b.push(1);b.extend(564u16.to_le_bytes());b.extend([0;5]);b.extend(b"ProbeMage\0");b.extend([0;5]);b
    }
    fn member(name:&str)->Member{Member{
        name:name.into(),force:0,class:0,sex:0,matches:0,wins:0,losses:0,state:0,
        affiliation:String::new(),disconnects:0,channel_selector:0,preview_state:0,
        rank:0,grade:0,certificates:[0;12],badges:[0;5],master:false,
    }}
    fn rejected(c:&mut LoginController,b:&[u8])->Result<()>{
        let before=serde_json::to_value(&c.state)?;let deadline=c.deadline;
        ensure!(c.accept_game(b).is_err(),"invalid early departure accepted");
        ensure!(serde_json::to_value(&c.state)?==before && c.deadline==deadline,"rejection changed state/deadline");Ok(())
    }
    let raw=departed(1,"ArenaBot");let mut cases=Vec::<&str>::new();
    ensure!(raw==[0xd0,1,0,0,0,0x41,0x72,0x65,0x6e,0x61,0x42,0x6f,0x74,0],"observed raw D0 changed");
    let mut c=pending(1);let before=serde_json::to_value(&c.state)?;let deadline=c.deadline;
    c.accept_game(&raw)?;
    ensure!(serde_json::to_value(&c.state)?==before && c.deadline==deadline,"early peer notification admitted membership or extended deadline");
    c.accept_game(&joined(1))?;
    ensure!(c.state.phase==LoginPhase::RoomAwaitSelf && c.state.joined_room.as_ref().unwrap().members.is_empty() && c.deadline==deadline,"D0 replaced CE/self authority");
    c.cancel();ensure!(c.is_quiescent(),"probe shutdown");cases.push("observed_D0_before_CE_admits_no_membership_and_keeps_original_deadline");

    let mut c=pending(1);let epoch=c.transport.snapshot().epoch;
    c.early_roster.push(epoch,Some(1),1,member("ArenaBot"))?;
    c.early_roster.push(epoch,Some(1),1,member("KeepPeer"))?;
    c.accept_game(&raw)?;c.accept_game(&joined(1))?;
    let members=&c.state.joined_room.as_ref().unwrap().members;
    ensure!(members.len()==1 && members[0].name=="KeepPeer","departed early snapshot survived CE or removed other peer");
    c.cancel();cases.push("early_member_then_D0_removes_only_that_snapshot");

    let mut c=pending(1);c.accept_game(&raw)?;let epoch=c.transport.snapshot().epoch;
    c.early_roster.push(epoch,Some(1),1,member("ArenaBot"))?;c.accept_game(&joined(1))?;
    ensure!(c.state.joined_room.as_ref().unwrap().members[0].name=="ArenaBot","later authoritative full member did not supersede old departure");
    c.cancel();cases.push("later_full_member_snapshot_supersedes_departure");

    let mut c=pending(1);rejected(&mut c,&departed(2,"ArenaBot"))?;c.accept_game(&joined(1))?;c.cancel();
    cases.push("wrong_requested_room_rejected_without_binding_pollution");
    let mut c=pending(1);rejected(&mut c,&departed(1,"ProbeMage"))?;c.cancel();
    cases.push("own_departure_cannot_complete_or_cancel_pending_join");
    let mut c=pending(1);c.state.phase=LoginPhase::LobbyReady;rejected(&mut c,&raw)?;c.cancel();
    cases.push("departure_outside_pending_or_active_room_rejected");

    let mut c=pending(1);c.state.phase=LoginPhase::RoomCreating;c.requested_room=None;
    rejected(&mut c,&raw)?;c.cancel();cases.push("unbound_create_does_not_accept_arbitrary_room");
    let mut c=pending(1);c.state.phase=LoginPhase::RoomCreating;c.requested_room=None;let epoch=c.transport.snapshot().epoch;
    c.early_roster.push(epoch,None,1,member("ArenaBot"))?;c.accept_game(&raw)?;c.accept_game(&joined(1))?;
    ensure!(c.state.joined_room.as_ref().unwrap().members.is_empty(),"known create roster did not consume departure");
    c.cancel();cases.push("create_requires_already_bound_same_epoch_roster");

    let mut roster=EarlyRoster::default();roster.push(8,Some(1),1,member("ArenaBot"))?;
    ensure!(roster.depart(9,Some(1),1,"ArenaBot").is_err(),"cross-epoch early departure accepted");
    ensure!(roster.take(8,1)?.len()==1,"invalid epoch removed peer");cases.push("foreign_epoch_rejected_atomically");
    let mut roster=EarlyRoster::default();
    ensure!(roster.depart(8,Some(1),1,&"x".repeat(28)).is_err(),"overlong departure accepted");
    ensure!(roster.take(9,2)?.is_empty(),"invalid name created binding");cases.push("invalid_name_cannot_create_binding");

    let mut c=pending(1);c.accept_game(&raw)?;c.accept_game(&[0xce,3])?;
    ensure!(c.state.phase==LoginPhase::LobbyReady && c.requested_room.is_none(),"rejection did not retire pending join");
    c.state.phase=LoginPhase::RoomJoining;c.requested_room=Some(2);c.accept_game(&joined(2))?;c.cancel();
    cases.push("join_refusal_clears_previous_early_departure_binding");
    let mut c=pending(1);c.accept_game(&raw)?;c.cancel();
    c.state.phase=LoginPhase::RoomJoining;c.state.selected_character=Some("ProbeMage".into());c.requested_room=Some(2);
    c.accept_game(&joined(2))?;c.cancel();ensure!(c.is_quiescent(),"cancelled probe not quiescent");
    cases.push("cancel_clears_binding_before_a_new_join");

    let mut c=pending(1);c.accept_game(&joined(1))?;c.accept_game(&raw)?;c.cancel();
    cases.push("existing_CE_before_peer_D0_order_preserved");
    for n in 0..raw.len(){let mut c=pending(1);rejected(&mut c,&raw[..n])?;c.cancel();}
    let mut malformed=raw.clone();malformed.push(0);let mut c=pending(1);rejected(&mut c,&malformed)?;c.cancel();
    cases.push("every_truncation_and_trailing_byte_rejected_atomically");
    Ok(serde_json::json!({"status":"candidate_pending_join_departure_contracts_passed",
        "cases":cases,"shadow_state_only":true,"new_network_sessions":0,"authority_mutations":0,
        "product_fix_applied":false,"GUI_verified":false,"whole_goal_complete":false}))
}

#[test]
fn observed_early_departure_and_fourteen_adversarial_join_contracts(){
 let result=pending_join_departure_replay().unwrap();
 assert_eq!(result["cases"].as_array().unwrap().len(),14);
 assert_eq!(result["new_network_sessions"],0);
}

/// Shadow-only proof of the current pending-join/retired-peer ordering.
fn retired_peer_pending_join_replay()->anyhow::Result<serde_json::Value>{
    use super::super::native_room::{self,Packet,Departure};
    fn setup(requested:u32)->anyhow::Result<LoginController>{
        let fixture:serde_json::Value=serde_json::from_slice(include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"),"/tests/fixtures/native-room-current.json")))?;
        let bytes=|key:&str|->Vec<u8>{let s=fixture["packets"][key].as_str().unwrap();(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect()};
        let Packet::Joined(mut room)=native_room::parse(&bytes("joined"))?else{anyhow::bail!("room fixture")};
        let Packet::Member{member,..}=native_room::parse(&bytes("member_0"))?else{anyhow::bail!("member fixture")};
        room.id=1;room.members=vec![member.clone(),member.clone()];room.members[0].name="ProbeMage".into();room.members[1].name="KnownOldPeer".into();
        let mut c=LoginController::default();let mut old=Some(room);
        ensure!(native_room::depart(&mut old,&mut c.retired_departures,1,"ProbeMage","ProbeMage")?==Departure::SelfLeft,"old self retirement");
        c.state.phase=LoginPhase::RoomJoining;c.state.selected_character=Some("ProbeMage".into());c.requested_room=Some(requested);c.deadline=Some(Instant::now()+Duration::from_secs(8));
        let mut next=member;next.name="KnownOldPeer".into();c.early_roster.push(c.transport.snapshot().epoch,Some(requested),requested,next)?;
        Ok(c)
    }

    let mut raw=vec![0xd0,1,0,0,0];raw.extend(b"KnownOldPeer\0");
    let mut cases=Vec::<&str>::new();
    let mut different=setup(2)?;let before=serde_json::to_value(&different.state)?;let deadline=different.deadline;
    different.accept_game(&raw)?;
    ensure!(serde_json::to_value(&different.state)?==before && different.deadline==deadline,"retired departure changed pending state/deadline");
    ensure!(different.early_roster.take(different.transport.snapshot().epoch,2)?.len()==1,"known old-room notification removed new early snapshot");
    ensure!(different.accept_game(&raw).is_err(),"retired foreign-room notification was consumed more than once");
    ensure!(serde_json::to_value(&different.state)?==before && different.deadline==deadline,"duplicate changed state/deadline");
    different.cancel();ensure!(different.is_quiescent(),"different-room shutdown");cases.push("known_retired_foreign_room_consumed_once_before_pending_join_guards");
    let mut same=setup(1)?;let deadline=same.deadline;
    same.accept_game(&raw)?;
    let peers=same.early_roster.take(same.transport.snapshot().epoch,1)?;ensure!(peers.len()==1,"retired same-room notice removed new snapshot");
    ensure!(!same.retired_departures.consume(1,"KnownOldPeer"),"retired notification left old membership unconsumed");
    for member in peers{same.early_roster.push(same.transport.snapshot().epoch,Some(1),1,member)?;}
    same.accept_game(&raw)?;
    ensure!(same.early_roster.take(same.transport.snapshot().epoch,1)?.is_empty(),"new same-room departure failed after old notification consumed");
    ensure!(same.deadline==deadline,"same-room callback extended deadline");same.cancel();ensure!(same.is_quiescent(),"same-room shutdown");cases.push("same_room_rejoin_preserves_new_snapshot_until_new_peer_departure");
    let mut unknown=setup(2)?;let before=serde_json::to_value(&unknown.state)?;let deadline=unknown.deadline;
    let mut bad=vec![0xd0,1,0,0,0];bad.extend(b"OtherOldPeer\0");
    ensure!(unknown.accept_game(&bad).is_err(),"unknown old peer used retired authorization");
    ensure!(serde_json::to_value(&unknown.state)?==before && unknown.deadline==deadline,"unknown old peer changed pending state");
    unknown.accept_game(&raw)?;ensure!(unknown.early_roster.take(unknown.transport.snapshot().epoch,2)?.len()==1,"unknown callback polluted later known callback");unknown.cancel();cases.push("unknown_old_peer_rejected_without_consuming_known_retirement");
    let mut own=setup(2)?;let before=serde_json::to_value(&own.state)?;let deadline=own.deadline;
    let mut bad=vec![0xd0,1,0,0,0];bad.extend(b"ProbeMage\0");
    ensure!(own.accept_game(&bad).is_err(),"own departure authorized before new acceptance");
    ensure!(serde_json::to_value(&own.state)?==before && own.deadline==deadline,"own invalid departure changed pending state");
    own.accept_game(&raw)?;ensure!(own.early_roster.take(own.transport.snapshot().epoch,2)?.len()==1,"own invalid callback polluted later known callback");own.cancel();cases.push("own_departure_still_rejected_before_acceptance");
    Ok(serde_json::json!({"status":"candidate_retired_peer_before_pending_join_contracts_passed","cases":cases,
        "shadow_state_only":true,"new_network_sessions":0,"authority_mutations":0,
        "product_fix_applied":false,"GUI_verified":false,"whole_goal_complete":false}))
}

#[test]
fn retired_peer_notifications_cannot_preempt_pending_join_or_erase_new_roster(){
 let result=retired_peer_pending_join_replay().unwrap();
 assert_eq!(result["cases"].as_array().unwrap().len(),4);
 assert_eq!(result["new_network_sessions"],0);
}
