//! T023-4: reachable G06 item-continuation rejection/expiry, T key entry,
//! cancel-late and malformed/cross-generation continuation replies.
//!
//! Synthetic encrypted loopback only. The server-side StateCopy denials
//! (active ScheduledFuture / EquipmentTimerLease / pending magic-stone expiry)
//! are not observable on the wire; the client can only see the destination
//! refusing the continuation (S_LITO_SELECT_CHAR_RESULT != 0) or never
//! completing it. Both must fail boundedly without C8/C9 and keep exactly one
//! unused correlation for an explicit authenticated LOGIN.
use super::*;

fn handoff() -> (LoginController, FaultPeer, FaultPeer) {
    let old = FaultPeer::new(); let target = FaultPeer::new();
    let mut c = seeded(&old);
    c.teleport.as_mut().unwrap().character = "重建角色".into();
    old.emit(vec![bytes("availability"), endpoint(target.peer)]);
    pump(&mut c, |c| c.transport.snapshot().epoch == 2 && c.state.phase == LoginPhase::GameLoginPending);
    (c, old, target)
}

/// Destination GAME published the character; the formal bootstrap now owns
/// the one-shot continuation, so the client waits without sending C9.
fn continuation_pending() -> (LoginController, FaultPeer, FaultPeer) {
    let (mut c, old, target) = handoff();
    target.emit(vec![game_packet("ready"), vec![3, 1], game_packet("row")]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharacterPending);
    assert_eq!(c.state.selected_character.as_deref(), Some("重建角色"));
    assert!(c.teleport.is_some(), "continuation consumed before the destination answered");
    (c, old, target)
}

fn no_directory_commands(target: &FaultPeer, name: &str) {
    for op in [0xc8, 0xc9] { assert!(!target.opcodes().contains(&op), "{name}: destination sent {op:02X}"); }
}

fn finish_with_recovery(c: LoginController, old: SocketAddr) {
    let (c, login, game) = explicit_recovery_redirect(c, old, Duration::from_secs(5));
    complete_explicit_recovery(c, login, game);
}

/// Rewrites every S7C row ID (IDs are server object IDs, not catalog IDs).
/// `keep` rows keep their source ID, the next `drop` rows are omitted (the
/// lease ended naturally) and the rest are re-rented under fresh IDs.
/// Returns (destination batch, source IDs in wire order, destination IDs).
fn rerented_batch(source: &[u8], keep: usize, drop: usize) -> (Vec<u8>, Vec<u32>, Vec<u32>) {
    assert_eq!(source[0], 0x7c);
    let mut rows = Vec::new(); let mut at = 2;
    for _ in 0..source[1] {
        let start = at; at += 14;
        at += source[at..].iter().position(|&b| b == 0).unwrap() + 1;
        at += 1 + usize::from(source[at]);
        rows.push(source[start..at].to_vec());
    }
    assert_eq!(at, source.len(), "fixture S7C layout");
    let wire: Vec<u32> = rows.iter().map(|row| u32::from_le_bytes(row[..4].try_into().unwrap())).collect();
    let mut out = vec![0x7c, 0]; let mut ids = Vec::new();
    for (index, mut row) in rows.into_iter().enumerate() {
        if (keep..keep + drop).contains(&index) { continue; }
        if index >= keep {
            let fresh = u32::from_le_bytes(row[..4].try_into().unwrap()) ^ 0x0040_0000;
            row[..4].copy_from_slice(&fresh.to_le_bytes());
        }
        ids.push(u32::from_le_bytes(row[..4].try_into().unwrap()));
        out[1] += 1; out.extend(row);
    }
    (out, wire, ids)
}

/// StateCopy refusing a live lease/expiry is reported to the client as a
/// rejected continuation. Every formal non-zero result fails closed.
#[cfg_attr(test, test)]
pub fn destination_refused_continuation_keeps_one_recovery_without_c9() {
    for (index, result) in ["SILENT_REJECTED", "PARTICIPATION_RESTRICTED", "INVALID_CHARACTER_NAME"].into_iter().enumerate() {
        let (mut c, mut old, mut target) = continuation_pending();
        target.emit(vec![game_packet(result)]);
        pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
        assert!(c.state.error.as_deref().unwrap().contains("destination rejected teleport continuation"), "{result}");
        retired(&c); assert_one_correlation(&c, old.peer);
        assert_eq!(c.transport.snapshot().epoch, 2, "{result}: refused continuation reopened");
        old.close(); target.close();
        no_directory_commands(&target, result);
        assert_eq!(old.opcodes().iter().filter(|&&op| op == 0xd9).count(), 1);
        if index == 0 { let peer = old.peer; finish_with_recovery(c, peer); }
    }
}

/// A destination that never answers the staged continuation (for example a
/// server-side copy exception) is bounded by the 8s deadline. A direct
/// teleport never falls back to C8 source confirmation or a second C9.
#[cfg_attr(test, test)]
pub fn destination_silent_continuation_is_bounded_and_keeps_one_recovery() {
    let (mut c, mut old, mut target) = continuation_pending();
    let deadline = c.deadline.expect("continuation deadline");
    assert!(deadline <= Instant::now() + Duration::from_secs(8));
    c.deadline = Some(Instant::now() - Duration::from_millis(1));
    pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
    assert!(c.state.error.as_deref().unwrap().contains("deadline"));
    retired(&c); assert_one_correlation(&c, old.peer);
    old.close(); target.close();
    no_directory_commands(&target, "silent");
    let peer = old.peer; finish_with_recovery(c, peer);
}

/// Natural rental expiry between source and destination: unexpired rows keep
/// their ID, expired rows are absent and re-rented rows use new IDs. The
/// client publishes exactly the destination authority; no source row or old
/// ID is carried over, and late deltas for retired IDs never re-grant them.
#[cfg_attr(test, test)]
pub fn expired_and_rerented_items_use_only_destination_ids() {
    let source = bytes("inventory");
    let (mut c, mut old, mut target) = continuation_pending();
    let (batch, old_ids, new_ids) = rerented_batch(&source, 10, 2);
    assert_eq!(old_ids.len(), 17);
    assert_eq!(old_ids.iter().collect::<std::collections::BTreeSet<_>>().len(), 17, "fixture IDs unique");
    assert_eq!(new_ids.len(), 15);
    assert!(new_ids[10..].iter().all(|id| *id != 0 && !old_ids.contains(id)), "fresh IDs collide");
    target.emit(vec![game_packet("ACCEPTED"), batch]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharacterSelected && c.state.inventory.batches == 1);
    assert!(c.teleport.is_none() && c.recovery_character.is_none());
    let published: Vec<u32> = c.state.inventory.items.keys().copied().collect();
    let mut expected = new_ids.clone(); expected.sort_unstable();
    assert_eq!(published, expected);
    for id in &old_ids[10..12] { assert!(!c.state.inventory.items.contains_key(id), "expired {id:08X} re-granted"); }
    for id in &old_ids[12..] { assert!(!c.state.inventory.items.contains_key(id), "old lease ID {id:08X} kept"); }
    let owner = c.state.inventory.owner.clone().unwrap();
    assert_eq!((owner.connection, owner.character.as_str()), (2, "重建角色"));
    let orphans = c.state.inventory.orphan_updates;
    let expired = old_ids[10];
    let mut remove = vec![0x17]; remove.extend(expired.to_le_bytes());
    let mut count = vec![0x1a]; count.extend(old_ids[12].to_le_bytes()); count.extend(5u32.to_le_bytes());
    target.emit(vec![remove, count]);
    pump(&mut c, |c| c.state.inventory.orphan_updates == orphans + 2);
    assert_eq!(c.state.inventory.items.keys().copied().collect::<Vec<_>>(), expected);
    assert_eq!(c.state.phase, LoginPhase::CharacterSelected);
    stop(&mut c); old.close(); target.close();
    no_directory_commands(&target, "expiry");
    assert!(c.teleport_recovery.is_none(), "completed continuation left a recovery");
}

/// Malformed or misordered destination continuation replies. Before the
/// selection result the staged continuation is unconsumed: keep one
/// correlation. After an accepted result the server consumed it: none.
#[cfg_attr(test, test)]
pub fn malformed_continuation_replies_fail_closed_on_the_destination() {
    let inventory = bytes("inventory");
    let cases: Vec<(&str, bool, Vec<Vec<u8>>)> = vec![
        ("selected-truncated", true, vec![vec![0xc8]]),
        ("selected-trailing", true, vec![vec![0xc8, 0, 0]]),
        ("selected-unknown-result", true, vec![vec![0xc8, 5]]),
        ("selected-repeated", false, vec![game_packet("ACCEPTED"), game_packet("ACCEPTED")]),
        ("inventory-truncated", false, vec![game_packet("ACCEPTED"), inventory[..40].to_vec()]),
        ("inventory-trailing", false, vec![game_packet("ACCEPTED"), [inventory.as_slice(), &[0]].concat()]),
    ];
    for (name, kept, bodies) in cases {
        let (mut c, mut old, mut target) = continuation_pending();
        target.emit(bodies);
        pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
        retired(&c);
        assert_eq!(c.transport.snapshot().epoch, 2, "{name}: another epoch");
        if kept { assert_one_correlation(&c, old.peer); }
        else { assert!(c.teleport_recovery.is_none(), "{name}: consumed continuation revived"); }
        old.close(); target.close();
        no_directory_commands(&target, name);
        assert_eq!(old.opcodes().iter().filter(|&&op| op == 0xd9).count(), 1, "{name}");
    }
}

/// Cross-generation: the destination replays the selection result before it
/// has published a directory (an earlier epoch's reply shape), and the
/// destination replays the source's F3/F4 teleport pair. Neither may start a
/// handoff, select a character or reach C9.
#[cfg_attr(test, test)]
pub fn cross_generation_continuation_replies_do_not_select_or_retarget() {
    for name in ["selected-before-directory", "source-teleport-pair"] {
        let (mut c, mut old, mut target) = handoff();
        let bodies = if name == "selected-before-directory" { vec![game_packet("ready"), game_packet("ACCEPTED")] }
            else { vec![game_packet("ready"), bytes("availability"), endpoint(old.peer)] };
        target.emit(bodies);
        pump(&mut c, |c| c.state.phase == LoginPhase::Failed && c.is_quiescent());
        retired(&c);
        assert_eq!(c.transport.snapshot().epoch, 2, "{name}: another epoch");
        assert_ne!(c.state.redirect, Some(old.peer), "{name}: retargeted to the source");
        assert_one_correlation(&c, old.peer);
        old.close(); target.close();
        no_directory_commands(&target, name);
        assert!(!target.opcodes().contains(&0xd9), "{name}: destination quit");
    }
}

fn wait_naturally(c: &mut LoginController, millis: u64, phase: LoginPhase) {
    let start = Instant::now();
    while start.elapsed() < Duration::from_millis(millis) {
        c.poll();
        assert_eq!(c.state.phase, phase, "late-cancel wait left {phase:?}: {:?}", c.state.error);
        thread::sleep(Duration::from_millis(5));
    }
}

/// G06 cancel-late (HISTORY L479: natural 3500ms wait, then cancel) while
/// the destination GAME is connected but silent, and again while the staged
/// continuation is pending. Cancel completes boundedly, quits the source once
/// and leaves one correlation for one explicit LOGIN.
#[cfg_attr(test, test)]
pub fn late_cancel_keeps_one_recovery_for_an_explicit_login() {
    for stage in ["game-login-pending", "continuation-pending"] {
        let (mut c, mut old, mut target) = if stage == "game-login-pending" { handoff() } else { continuation_pending() };
        let phase = c.state.phase;
        wait_naturally(&mut c, 3500, phase);
        let cancelled = Instant::now();
        c.cancel();
        pump(&mut c, |c| c.is_quiescent());
        assert!(cancelled.elapsed() < Duration::from_secs(3), "{stage}: cancel not bounded");
        assert_eq!(c.state.phase, LoginPhase::Closed, "{stage}");
        retired(&c); assert_one_correlation(&c, old.peer);
        for _ in 0..8 { c.poll(); assert_eq!(c.transport.snapshot().epoch, 2); thread::sleep(Duration::from_millis(2)); }
        old.close(); target.close();
        no_directory_commands(&target, stage);
        assert_eq!(old.opcodes().iter().filter(|&&op| op == 0xd9).count(), 1, "{stage}");
        assert_eq!(target.accepted.load(Ordering::Acquire), 1, "{stage}");
        let peer = old.peer; finish_with_recovery(c, peer);
    }
}

/// Cancel after the continuation completed is an ordinary exit: no recovery
/// correlation is created and a later explicit LOGIN shows the plain source
/// directory instead of auto-continuing a consumed loadout.
#[cfg_attr(test, test)]
pub fn cancel_after_completed_continuation_creates_no_recovery() {
    let (mut c, mut old, mut target) = continuation_pending();
    target.emit(vec![game_packet("ACCEPTED"), bytes("inventory")]);
    pump(&mut c, |c| c.state.phase == LoginPhase::CharacterSelected && c.state.inventory.batches == 1);
    wait_naturally(&mut c, 500, LoginPhase::CharacterSelected);
    c.cancel(); c.cancel();
    pump(&mut c, |c| c.is_quiescent());
    retired(&c);
    assert!(c.teleport_recovery.is_none() && c.recovery_character.is_none());
    old.close(); target.close();
    no_directory_commands(&target, "completed");
}

/// T key entry. Cycling follows the presented draw order, skips the own
/// actor, absent ids and dead bodies (unless #41 corpse mode), wraps, and is
/// scoped to the live world: after the teleport scene retires nothing is
/// selectable and a selection captured before the handoff no longer resolves.
#[cfg_attr(test, test)]
pub fn t_key_cycle_is_scoped_to_the_presented_live_world() {
    use crate::modern_lito::native_skill_selection::{cycle_target, Target};
    let mut peer = FaultPeer::new(); let mut c = seeded_rental(&peer);
    let other = NPC + 1;
    let mut clone = c.state.world.actors.get(&NPC).unwrap().clone(); clone.id = other;
    c.state.world.actors.insert(other, clone);
    let world = &c.state.world; let own = world.own_actor().unwrap().id;
    let presented = [own, NPC, 0xdead_beef, other, NPC];
    let first = cycle_target(world, presented, own, None, false).unwrap();
    assert_eq!(first.id, NPC);
    let second = cycle_target(world, presented, own, Some(first), false).unwrap();
    assert_eq!(second.id, other);
    assert_eq!(cycle_target(world, presented, own, Some(second), false).unwrap().id, NPC, "wrap");
    assert!(cycle_target(world, [own], own, None, false).is_none(), "own actor selected");
    assert!(cycle_target(world, presented, NPC, None, false).is_none(), "foreign own id accepted");
    c.state.world.actors.get_mut(&other).unwrap().dead = true;
    let world = &c.state.world;
    assert_eq!(cycle_target(world, presented, own, Some(first), false).unwrap().id, NPC, "dead body cycled");
    assert_eq!(cycle_target(world, presented, own, None, true).unwrap().id, other, "corpse mode");
    let before: Target = cycle_target(world, presented, own, None, false).unwrap();
    assert_eq!(before.resolve(world), Some(NPC));
    stop(&mut c); peer.close();
    retired(&c);
    let world = &c.state.world;
    assert!(cycle_target(world, presented, own, Some(before), false).is_none(), "retired world selectable");
    assert!(before.resolve(world).is_none(), "pre-handoff selection survived");
}
