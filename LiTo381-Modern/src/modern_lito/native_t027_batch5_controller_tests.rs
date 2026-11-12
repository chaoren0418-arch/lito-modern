//! Issue #149 (T027 batch 5): skill 69 caster handshake (S55 → one C4E → S5A →
//! C08) and the own-object `monlist` offer of skill 67, on this module's
//! synthetic encrypted loopback. No server, game process, GUI or sound device.
//!
//! Server contract (lito-launcher-server, read-only): `useMassTeleportSpell`
//! reserves the portal and sends `Protocol.requestSummon` (`55 00 40 d`);
//! `returnSummon` accepts one `4E c` and answers S_PORTAL; every rejection is
//! `S_PARALYSE` mode 7 (`25 07`) plus an optional message, with no portal.
use super::*;
use crate::modern_lito::{native_navigation::InputStamp, native_portal::Destination, native_skills};

const UNLOCK: [u8; 2] = [0x25, 0x07];

fn caster(peer: &FaultPeer) -> LoginController {
    let mut c = seeded(peer);
    c.teleport = None;
    c.state.merchant.clear();
    c.state.joined_room = None;
    c.state.phase = LoginPhase::WorldReady;
    let epoch = c.transport.snapshot().epoch;
    c.state.environment.begin(epoch).unwrap();
    c.state.world.learned_skills.insert(69);
    c
}
/// The state `cast_teleport` leaves behind for a queued skill-69 request.
fn requested(c: &mut LoginController, since: Instant, returned: bool) -> u16 {
    let epoch = c.transport.snapshot().epoch;
    let map = c.state.world.map_id.unwrap();
    c.teleport_spell = Some((InputStamp::capture(epoch, &c.state.world), since, map));
    c.mass_return = Some(returned);
    map
}
fn own(c: &LoginController) -> (u32, u16, u16) {
    let a = c.state.world.own_actor().unwrap();
    (a.id, a.x, a.y)
}
fn s55(id: u32) -> Vec<u8> {
    let mut b = vec![0x55, 0, 0x40];
    b.extend(id.to_le_bytes());
    b
}
fn effect(id: u32) -> Vec<u8> {
    let mut b = vec![0x37];
    b.extend(id.to_le_bytes());
    b.extend(2236u16.to_le_bytes());
    b
}
fn portal(d: Destination) -> Vec<u8> {
    let mut b = vec![0x5a];
    for value in [d.map, d.x, d.y] {
        b.extend(value.to_le_bytes());
    }
    b
}
/// Emits `bodies` followed by one 2236 effect and waits until that effect was
/// accepted, so every earlier body has been dispatched in order.
fn deliver(c: &mut LoginController, peer: &FaultPeer, mut bodies: Vec<Vec<u8>>) {
    let before = c.state.world.effects.len();
    bodies.push(effect(own(c).0));
    peer.emit(bodies);
    pump(c, |c| c.state.world.effects.len() > before);
}
fn replies(peer: &FaultPeer) -> usize {
    peer.commands().iter().filter(|b| b[0] == 0x4e).count()
}

#[test]
fn mass69_answers_only_its_own_s55_once_and_then_admits_the_server_portal() {
    let mut peer = FaultPeer::new();
    let mut c = caster(&peer);
    let map = requested(&mut c, Instant::now(), false);
    let (id, x, y) = own(&c);
    // The 2236 effect is not the handshake, and another object's S55 is not ours.
    deliver(&mut c, &peer, vec![s55(id.wrapping_add(1000))]);
    assert_eq!(c.mass_return, Some(false));
    assert!(c.teleport_spell.is_some() && c.state.world.portal.is_none());
    assert_eq!(replies(&peer), 0);
    // Wrong lengths or marker bytes are not the request either.
    for bad in [vec![0x55, 0, 0x40], vec![0x55, 1, 0x40, 1, 0, 0, 0], vec![0x55, 0, 0x41, 1, 0, 0, 0]] {
        assert_eq!(native_skills::return_request_object(&bad), None);
    }
    assert_eq!(native_skills::return_request_object(&s55(0)), None);
    assert_eq!(native_skills::return_request_object(&s55(id)), Some(id));

    peer.emit(vec![s55(id)]);
    pump(&mut c, |c| c.mass_return == Some(true));
    pump(&mut c, |_| replies(&peer) == 1);
    assert!(peer.commands().contains(&native_skills::RETURN_REPLY.to_vec()));
    assert_eq!(native_skills::RETURN_REPLY, [0x4e, 0]);
    assert!(c.teleport_spell.is_some(), "the request stays owned until S5A");
    assert!(c.state.world.portal.is_none(), "C4E is not a portal");
    assert_eq!(own(&c), (id, x, y), "no local move");
    // A repeated S55 is never answered again.
    deliver(&mut c, &peer, vec![s55(id)]);
    assert_eq!(c.mass_return, Some(true));

    // The server's cell is the destination; the client acknowledges it once.
    let destination = Destination { map, x: x + 1, y };
    peer.emit(vec![portal(destination)]);
    pump(&mut c, |c| c.state.world.portal.is_some());
    assert!(c.teleport_spell.is_none() && c.mass_return.is_none());
    assert_eq!(c.state.world.portal.unwrap().destination, destination);
    pump(&mut c, |_| peer.commands().iter().any(|b| b[0] == 8));
    stop(&mut c);
    peer.close();
    assert_eq!(replies(&peer), 1);
    assert_eq!(peer.commands().iter().filter(|b| b[0] == 8).count(), 1);
}

#[test]
fn mass69_portal_before_the_return_reply_is_not_this_requests_reply() {
    let mut peer = FaultPeer::new();
    let mut c = caster(&peer);
    let map = requested(&mut c, Instant::now(), false);
    let (_, x, y) = own(&c);
    // Without this connection's C4E the request grants nothing: WORLD rejects
    // the unadmitted S_PORTAL exactly as it does without any request.
    assert!(c.accept_game(&portal(Destination { map, x: x + 1, y })).is_err());
    assert!(c.state.world.portal.is_none());
    assert!(c.teleport_spell.is_none(), "the consumed request cannot admit a later portal");
    assert!(c.accept_game(&portal(Destination { map, x: x + 1, y })).is_err());
    // A returned request still only admits the map it asked for.
    let map = requested(&mut c, Instant::now(), true);
    assert!(c.accept_game(&portal(Destination { map: map.wrapping_add(1), x, y })).is_err());
    assert!(c.state.world.portal.is_none());
    // …and requires skill 69 itself, not another destination skill.
    requested(&mut c, Instant::now(), true);
    c.state.world.learned_skills.remove(&69);
    c.state.world.learned_skills.insert(5);
    assert!(c.accept_game(&portal(Destination { map, x: x + 1, y })).is_err());
    assert!(c.state.world.portal.is_none());
    stop(&mut c);
    peer.close();
    assert_eq!(replies(&peer), 0);
    assert!(!peer.commands().iter().any(|b| b[0] == 8));
}

#[test]
fn mass69_unlock_expiry_and_stale_owner_never_send_the_return_reply() {
    for case in 0..4 {
        let mut peer = FaultPeer::new();
        let mut c = caster(&peer);
        let now = Instant::now();
        let (id, x, y) = own(&c);
        match case {
            // Rejection: mode 7 ends the wait; a later S55 is unsolicited.
            0 => {
                requested(&mut c, now, false);
                deliver(&mut c, &peer, vec![UNLOCK.to_vec(), s55(id)]);
            }
            // No S55 within the two-second reply window.
            1 => {
                requested(&mut c, now - Duration::from_secs(3), false);
                deliver(&mut c, &peer, vec![s55(id)]);
            }
            // A control interruption after the request retires its owner stamp.
            2 => {
                requested(&mut c, now, false);
                c.state.world.control_interruptions += 1;
                deliver(&mut c, &peer, vec![s55(id)]);
            }
            // After C4E the S5A wait is bounded by eight seconds, not two…
            _ => {
                let map = requested(&mut c, now - Duration::from_secs(3), true);
                c.poll();
                assert!(c.teleport_spell.is_some(), "returned request expired at two seconds");
                requested(&mut c, now - Duration::from_secs(9), true);
                c.poll();
                // …and an expired one admits no portal.
                assert!(c.accept_game(&portal(Destination { map, x: x + 1, y })).is_err());
            }
        }
        assert!(c.teleport_spell.is_none(), "case {case}");
        assert!(c.state.world.portal.is_none(), "case {case}");
        stop(&mut c);
        peer.close();
        assert_eq!(replies(&peer), 0, "case {case}");
        assert!(!peer.commands().iter().any(|b| b[0] == 8), "case {case}");
    }
}

fn hypertext(id: u32, html: &str, arguments: &[&str]) -> Vec<u8> {
    let mut b = vec![0x2a];
    b.extend(id.to_le_bytes());
    b.extend(html.as_bytes());
    b.extend([0, 0]);
    b.extend((arguments.len() as u16).to_le_bytes());
    for argument in arguments {
        b.extend(argument.as_bytes());
        b.push(0);
    }
    b
}

#[test]
fn shape67_own_monlist_offer_is_consumed_without_a_reply_or_dialog_state() {
    let mut peer = FaultPeer::new();
    let mut c = caster(&peer);
    let (id, ..) = own(&c);
    let sent = peer.commands().len();
    let before = serde_json::to_value(&c.state).unwrap();
    // The receiver needs neither skill 67 nor a pending cast.
    assert!(!c.state.world.learned_skills.contains(&67) && c.skill_pending.is_none());
    c.accept_game(&hypertext(id, "monlist", &[])).unwrap();
    c.accept_game(&hypertext(id, "monlist", &[])).unwrap();
    assert_eq!(c.shape_menu_offers(), 2);
    assert_eq!(serde_json::to_value(&c.state).unwrap(), before, "no appearance, dialog or status change");
    // Another object's monlist, or one with arguments, is not this offer and
    // keeps the existing unsolicited-dialog rejection.
    assert!(c.accept_game(&hypertext(id.wrapping_add(1000), "monlist", &[])).is_err());
    assert!(c.accept_game(&hypertext(id, "monlist", &["x"])).is_err());
    assert!(c.accept_game(&hypertext(id, "summonlist", &[])).is_err());
    assert_eq!(c.shape_menu_offers(), 2);
    stop(&mut c);
    peer.close();
    // No C_HACTION (0x27) or any other body was produced by the offer.
    assert_eq!(peer.commands().len(), sent);
}
