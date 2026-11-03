//! v37 life-bound admission around existing typed commands. The proof is
//! captured BEFORE the worker queue. Rejection is a READ, never a command ACK.
use super::{
    application_protocol as app,
    battle_window::{Stamp, Target},
    protocol,
};
use std::io::Read;
pub const VERSION: u16 = 37;
pub const REQUEST_KIND: u16 = 62;
pub const RESULT_KIND: u16 = 63;
pub const MAX_REQUEST_BYTES: usize = 236;
pub const MAX_RESULT_BYTES: usize = MAX_REQUEST_BYTES + 8 + 1024 * 1024;
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Guard {
    pub stamp: Stamp,
    pub target: Option<Target>,
}
impl std::fmt::Debug for Guard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleActionGuard[redacted]")
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rejection {
    None,
    NotActive,
    StaleLife,
    Dead,
    StaleTarget,
}
pub fn negotiated(a: &protocol::Ack) -> bool {
    a.version >= VERSION && a.capabilities & protocol::CAP_ACTION_LIFE != 0
}
/// Called after the controller's poll/painted-intent validation, never at send time.
pub fn for_command(
    view: &super::login_window::LoginSnapshot,
    command: &super::login_window::LoginCommand,
) -> Result<Option<Guard>, String> {
    use super::login_window::LoginCommand as C;
    let mutates = match command {
        C::World(q) => q.action == super::world_protocol::Action::Move,
        C::CombatAttack { .. }
        | C::SiegeAttack { .. }
        | C::SiegePortal { .. }
        | C::NpcAttack { .. }
        | C::QuestAttack { .. }
        | C::TimeAttackTowerAttack { .. }
        | C::UseItem { .. }
        | C::SelectAmmo { .. }
        | C::CastTeleport
        | C::CastLightArrow { .. }
        | C::CastBasicHeal { .. }
        | C::CastFireStorm
        | C::CastDisintegrate(_)
        | C::CastNpcLightArrow { .. }
        | C::CastNpcDisintegrate(_)
        | C::CastNpcFireStorm(_) => true,
        _ => false,
    };
    if !view.action_supported || !mutates {
        return Ok(None);
    }
    let fail = || "角色或目標生命已更新；未送出操作".to_owned();
    let o = view.battle.observation().ok_or_else(fail)?;
    let d = o.data();
    let stamp = Stamp::of(o);
    if !view.battle_supported
        || view.battle.request() != super::battle_session::request_for(view).as_ref()
        || d.world.phase != super::battle_payload::data::Phase::Active
        || super::battle_player::authority_life(view, stamp.entity) != Some(stamp.life)
        || !super::battle_space::occupies(d, stamp.entity)
        || d.actors
            .0
            .iter()
            .find(|a| a.entity == stamp.entity)
            .is_none_or(|a| a.vital.as_ref().is_some_and(|v| v.values.hp == 0))
    {
        return Err(fail());
    }
    let player_or_honor = |id: u64| -> Result<Target, String> {
        if let Some(a) = d.actors.0.iter().find(|a| a.entity as u64 == id) {
            Ok(Target::Player {
                entity: a.entity,
                life: super::battle_window::life(a),
            })
        } else if d.honor.objectives.0.iter().any(|a| a.entity as u64 == id) {
            Ok(Target::Honor { entity: id as i64 })
        } else {
            Err(fail())
        }
    };
    let target = match command {
        C::CombatAttack { target_entity_id }
        | C::CastLightArrow { target_entity_id }
        | C::CastBasicHeal { target_entity_id } => Some(player_or_honor(*target_entity_id)?),
        C::NpcAttack { target, life } | C::CastNpcLightArrow { target, life } => {
            Some(Target::Npc {
                serial: target.serial as i64,
                life: *life as i64,
            })
        }
        C::QuestAttack { target } => Some(Target::Quest {
            slot: target.slot as i32,
            born: d
                .quest
                .as_ref()
                .and_then(|q| {
                    q.objectives
                        .0
                        .iter()
                        .find(|a| a.id.slot == target.slot as i32)
                })
                .ok_or_else(fail)?
                .born_revision,
        }),
        C::TimeAttackTowerAttack { .. } => Some(Target::TimeTower),
        C::SiegeAttack { target } => Some(Target::Siege {
            slot: target.slot as u8,
        }),
        C::CastDisintegrate(q) => Some(Target::Player {
            entity: q.target_entity_id as i64,
            life: q.target_life as i64,
        }),
        C::CastNpcDisintegrate(q) => Some(Target::Npc {
            serial: q.target.serial as i64,
            life: q.target_life as i64,
        }),
        _ => None,
    };
    if target.is_some_and(|t| !super::battle_window::target_current(o, t)) {
        return Err(fail());
    }
    let guard = Guard { stamp, target };
    guard.validate().map_err(|_| fail())?;
    Ok(Some(guard))
}
pub fn mutation(b: &[u8]) -> bool {
    if b.len() < 40 {
        return false;
    }
    match u16_at(b, 12) {
        20 => b[37] == 3,
        22 | 34 | 36 => b[37] == 1,
        38 => matches!(b[37], 1 | 2 | 4),
        26 | 40 | 42 | 44 | 46 | 48 => true,
        _ => false,
    }
}
fn target_parts(t: Option<Target>) -> (u8, i64, i64) {
    match t {
        None => (0, 0, 0),
        Some(Target::Player { entity, life }) => (1, entity, life),
        Some(Target::Npc { serial, life }) => (2, serial, life),
        Some(Target::Quest { slot, born }) => (3, i64::from(slot), born),
        Some(Target::Honor { entity }) => (4, entity, 0),
        Some(Target::TimeTower) => (5, 0, 0),
        Some(Target::Siege { slot }) => (6, i64::from(slot), 0),
    }
}
impl Guard {
    fn validate(self) -> Result<(), app::ApplicationError> {
        let s = self.stamp;
        let (k, id, life) = target_parts(self.target);
        if !app::valid_context(s.context)
            || s.channel > 3
            || s.world.iter().any(|v| *v < 1)
            || !(1..=100).contains(&s.entity)
            || s.life < 1
            || s.revision < 1
            || !match k {
                0 | 5 => id == 0 && life == 0,
                1 => (1..=100).contains(&id) && life > 0,
                2 => id > 0 && life > 0,
                3 => (1..=8).contains(&id) && life > 0,
                4 => (3..=102).contains(&id) && life == 0,
                6 => (0..3).contains(&id) && life == 0,
                _ => false,
            }
        {
            return Err(invalid());
        }
        Ok(())
    }
    fn bind(self, b: &[u8]) -> Result<(), app::ApplicationError> {
        self.validate()?;
        if !(72..=120).contains(&b.len())
            || !mutation(b)
            || &b[..4] != b"LTM1"
            || u32_at(b, 4) as usize != b.len()
            || u32_at(b, 8) == 0
            || b[14..16] != [0, 0]
        {
            return Err(invalid());
        }
        let s = self.stamp;
        let k = u16_at(b, 12);
        let (t, id, life) = target_parts(self.target);
        if u32_at(b, 16) != s.context.key.source_id
            || i64_at(b, 20) as u64 != s.context.key.object_id
            || i64_at(b, 28) as u64 != s.context.epoch
            || b[36] != s.channel
        {
            return Err(invalid());
        }
        let (size, match_at, room) = match k {
            20 => (120, 56, false),
            22 | 34 | 36 => (72, 40, false),
            26 => (80, 40, false),
            38 => (88, 48, true),
            40 | 46 => (96, 48, true),
            42 => (72, 48, true),
            44 => (88, 40, false),
            48 => (80, 48, true),
            _ => return Err(invalid()),
        };
        if b.len() != size
            || i64_at(b, match_at) != s.world[1]
            || i64_at(b, match_at + 8) != s.world[2]
            || room && i64_at(b, 40) != s.world[0]
        {
            return Err(invalid());
        }
        let target_ok = match k {
            20 | 34 | 36 => t == 0,
            22 => matches!(t, 1 | 4) && id == i64_at(b, 64),
            26 => match u32_at(b, 64) {
                5 | 70 => t == 0 && i64_at(b, 72) == 0,
                4 => matches!(t, 1 | 4) && id == i64_at(b, 72),
                1 => t == 4 && id == i64_at(b, 72),
                _ => false,
            },
            38 => match b[37] {
                1 => t == 2 && id == i64_at(b, 72) && life == i64_at(b, 80),
                2 => t == 3 && id == i64_at(b, 72),
                4 => t == 5,
                _ => false,
            },
            42 => {
                if b[37] == 1 {
                    t == 0
                } else {
                    b[37] == 0 && t == 6 && id == i64::from(b[38])
                }
            }
            44 => s.life == i64_at(b, 64) && t == 1 && id == i64_at(b, 72) && life == i64_at(b, 80),
            40 | 46 => {
                s.life == i64_at(b, 72) && t == 2 && id == i64_at(b, 80) && life == i64_at(b, 88)
            }
            48 => s.life == i64_at(b, 72) && t == 0,
            _ => false,
        };
        if !target_ok {
            return Err(invalid());
        }
        Ok(())
    }
}
pub fn encode(g: Guard, inner: &[u8], version: u16) -> Result<Vec<u8>, app::ApplicationError> {
    if !(VERSION..=protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    g.bind(inner)?;
    let mut b = Vec::with_capacity(116 + inner.len());
    b.extend(b"LTM1");
    b.extend(((116 + inner.len()) as u32).to_be_bytes());
    b.extend(&inner[8..12]);
    b.extend(REQUEST_KIND.to_be_bytes());
    b.extend([0; 2]);
    let s = g.stamp;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for v in [
        s.world[0], s.world[1], s.world[2], s.entity, s.life, s.revision,
    ] {
        b.extend(v.to_be_bytes());
    }
    let (k, id, life) = target_parts(g.target);
    b.push(k);
    b.extend([0; 7]);
    b.extend(id.to_be_bytes());
    b.extend(life.to_be_bytes());
    b.extend((inner.len() as u32).to_be_bytes());
    b.extend(inner);
    Ok(b)
}
/// Context/capability/expected envelope are known BEFORE reading. Result kind,
/// total bound and request ID are checked before peer-sized allocation.
pub fn read_reply(
    reader: &mut impl Read,
    g: Guard,
    expected: &[u8],
    version: u16,
) -> Result<(Rejection, app::ResponseFrame), app::ApplicationError> {
    if !(VERSION..=protocol::MAX_VERSION).contains(&version)
        || !(188..=MAX_REQUEST_BYTES).contains(&expected.len())
        || u16_at(expected, 12) != REQUEST_KIND
    {
        return Err(invalid());
    }
    if encode(g, &expected[116..], version)? != expected {
        return Err(invalid());
    }
    let mut header = [0; 16];
    reader.read_exact(&mut header).map_err(|_| invalid())?;
    let length = u32_at(&header, 4) as usize;
    if &header[..4] != b"LTM1"
        || u16_at(&header, 12) != RESULT_KIND
        || header[14..16] != [0, 0]
        || header[8..12] != expected[8..12]
        || !(212..=MAX_RESULT_BYTES).contains(&length)
    {
        return Err(invalid());
    }
    let mut body = vec![0; length - 16];
    reader.read_exact(&mut body).map_err(|_| invalid())?;
    let echo_len = expected.len() - 16;
    if body.len() < echo_len + 24
        || body[..echo_len] != expected[16..]
        || body[echo_len + 1..echo_len + 4] != [0; 3]
        || u32_at(&body, echo_len + 4) as usize != body.len() - echo_len - 8
    {
        return Err(invalid());
    }
    let rejection = match body[echo_len] {
        0 => Rejection::None,
        1 => Rejection::NotActive,
        2 => Rejection::StaleLife,
        3 => Rejection::Dead,
        4 => Rejection::StaleTarget,
        _ => return Err(invalid()),
    };
    let inner = &body[echo_len + 8..];
    if !matches!(
        u16_at(inner, 12),
        21 | 23 | 27 | 35 | 37 | 39 | 41 | 43 | 45 | 47 | 49
    ) {
        return Err(invalid());
    }
    let r = app::decode_response_version(inner, version)?;
    if r.request_id != u32_at(expected, 8) {
        return Err(invalid());
    }
    if rejection == Rejection::None {
        if echo_bytes(&r, version)? != expected[116..] {
            return Err(invalid());
        }
    } else {
        recovery(g, &r.response)?;
    }
    Ok((rejection, r))
}
fn echo_bytes(r: &app::ResponseFrame, v: u16) -> Result<Vec<u8>, app::ApplicationError> {
    use app::Response as R;
    let id = r.request_id;
    match &r.response {
        R::World(r) => super::world_protocol::encode_request(id, &r.echo),
        R::Combat(r) => super::combat_protocol::encode_request(id, &r.echo),
        R::Cast(r) => super::spell_protocol::encode_cast_version(id, &r.echo, v),
        R::Item(r) => super::item_protocol::encode_request(id, r.echo, v),
        R::Ammo(r) => super::ammo_protocol::encode_request(id, r.echo, v),
        R::Npc(r) => super::npc_protocol::encode_request(id, &r.echo),
        R::NpcSpell(r) => super::npc_spell_protocol::encode_request(id, &r.echo, v),
        R::Siege(r) => super::siege_protocol::encode_request(id, &r.echo),
        R::Disintegrate(r) => super::disintegrate_cast_protocol::encode_request(id, &r.echo, v),
        R::NpcDisintegrate(r) => super::npc_disintegrate_protocol::encode_request(id, &r.echo, v),
        R::NpcFireStorm(r) => super::npc_fire_storm_protocol::encode_request(id, &r.echo, v),
        _ => Err(invalid()),
    }
}
fn recovery(g: Guard, r: &app::Response) -> Result<(), app::ApplicationError> {
    use super::{combat_protocol as c, npc_protocol as n, world_protocol as w};
    let (context, channel, match_id, generation, frame) = match r {
        app::Response::World(r)
            if r.echo.action == w::Action::Read
                && r.status == w::Status::Ok
                && r.reason == w::Reason::None =>
        {
            (
                r.echo.context,
                r.echo.channel_id,
                r.echo.match_id,
                r.echo.generation,
                r.frame.as_ref(),
            )
        }
        app::Response::Combat(r)
            if r.echo.action == c::Action::Read
                && r.status == c::Status::Ok
                && r.reason == c::Reason::None =>
        {
            (
                r.echo.context,
                r.echo.channel_id,
                r.echo.match_id,
                r.echo.generation,
                r.frame.as_ref().map(|f| &f.world),
            )
        }
        app::Response::Npc(r)
            if r.echo.action == n::Action::Read
                && r.status == n::Status::Ok
                && r.reason == n::Reason::None =>
        {
            (
                r.echo.context,
                r.echo.channel_id,
                r.echo.world.match_id,
                r.echo.world.generation,
                r.frame.as_ref().map(|f| &f.combat.world),
            )
        }
        _ => return Err(invalid()),
    };
    let s = g.stamp;
    let f = frame.ok_or_else(invalid)?;
    if context != s.context
        || channel != s.channel
        || match_id as i64 != s.world[1]
        || generation as i64 != s.world[2]
        || f.match_id != match_id
        || f.generation != generation
        || f.room.summary.room_id as i64 != s.world[0]
        || f.revision <= s.revision
        || !f
            .actors
            .iter()
            .any(|a| a.entity_id as i64 == s.entity && a.matches_character(s.context.key))
    {
        return Err(invalid());
    }
    Ok(())
}
fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes(b[i..i + 2].try_into().unwrap())
}
fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_be_bytes(b[i..i + 4].try_into().unwrap())
}
fn i64_at(b: &[u8], i: usize) -> i64 {
    i64::from_be_bytes(b[i..i + 8].try_into().unwrap())
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid life-bound battle action",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    fn guard() -> Guard {
        Guard {
            stamp: Stamp {
                context: app::SelectionContext {
                    key: app::CharacterKey {
                        source_id: 7,
                        object_id: 3333,
                    },
                    epoch: 1u64 << 63,
                },
                channel: 3,
                world: [101, 102, 103],
                entity: 1,
                life: 2,
                revision: 4,
            },
            target: Some(Target::Player { entity: 2, life: 3 }),
        }
    }
    fn inner() -> Vec<u8> {
        super::super::combat_protocol::encode_request(
            9,
            &super::super::combat_protocol::Request {
                context: guard().stamp.context,
                channel_id: 3,
                action: super::super::combat_protocol::Action::Attack,
                match_id: 102,
                generation: 103,
                sequence: 1,
                target_entity_id: 2,
            },
        )
        .unwrap()
    }
    fn response(expected: &[u8], reason: u8) -> Vec<u8> {
        let mut result = app::header(9, 23, 76).unwrap();
        result.extend(&inner()[16..]);
        result.extend([3, 1, 0, 0]);
        let mut b = expected.to_vec();
        b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
        b.extend([reason, 0, 0, 0]);
        b.extend((result.len() as u32).to_be_bytes());
        b.extend(result);
        let len = b.len() as u32;
        b[4..8].copy_from_slice(&len.to_be_bytes());
        b
    }
    #[test]
    fn golden_matches_java_exact_width_namespace_life_and_opaque_epoch() {
        let b = encode(guard(), &inner(), 37).unwrap();
        assert_eq!(b.iter().map(|x|format!("{x:02x}")).collect::<String>(),"4c544d31000000bc00000009003e0000000000070000000000000d05800000000000000000000003000000000000006500000000000000660000000000000067000000000000000100000000000000020000000000000004010000000000000000000000000000020000000000000003000000484c544d31000000480000000900160000000000070000000000000d058000000000000000030100000000000000000066000000000000006700000000000000010000000000000002");
        assert_eq!(&b[116..], inner());
        assert_eq!(b.len(), 188);
        for version in [1, 27, 35, 36, protocol::MAX_VERSION + 1] {
            assert!(encode(guard(), &inner(), version).is_err());
        }
        assert_eq!(format!("{:?}", guard()), "BattleActionGuard[redacted]");
    }
    #[test]
    fn accepted_inner_echo_is_exact_and_outer_is_not_a_new_gameplay_ack() {
        let expected = encode(guard(), &inner(), 37).unwrap();
        let bytes = response(&expected, 0);
        let (reason, reply) = read_reply(&mut bytes.as_slice(), guard(), &expected, 37).unwrap();
        assert_eq!(reason, Rejection::None);
        assert_eq!(reply.request_id, 9);
        assert!(
            matches!(reply.response,app::Response::Combat(r) if r.echo.sequence==1&&r.status==super::super::combat_protocol::Status::Unavailable)
        );
        for n in 0..bytes.len() {
            assert!(read_reply(&mut &bytes[..n], guard(), &expected, 37).is_err());
        }
        for offset in [
            8,
            14,
            16,
            28,
            39,
            47,
            71,
            80,
            88,
            100,
            111,
            115,
            expected.len() + 1,
            bytes.len() - 12,
        ] {
            let mut wrong = bytes.clone();
            wrong[offset] ^= 1;
            assert!(
                read_reply(&mut wrong.as_slice(), guard(), &expected, 37).is_err(),
                "offset {offset}"
            );
        }
    }
    #[test]
    fn rejection_needs_real_read_not_an_ack_or_absent_frame() {
        let expected = encode(guard(), &inner(), 37).unwrap();
        for reason in 1..=5 {
            assert!(read_reply(
                &mut response(&expected, reason).as_slice(),
                guard(),
                &expected,
                37
            )
            .is_err());
        }
        let mut bytes = response(&expected, 1);
        let at = expected.len() + 8;
        bytes[at + 37] = 0;
        bytes[at + 56..at + 72].fill(0); // actual READ echo, but no valid state
        assert!(read_reply(&mut bytes.as_slice(), guard(), &expected, 37).is_err());
    }
    #[test]
    fn no_read_load_leave_credential_or_nested_frame_can_be_wrapped() {
        let b = inner();
        let g = guard();
        let mut read = b.clone();
        read[37] = 0;
        assert!(!mutation(&read));
        assert!(encode(g, &read, 37).is_err());
        for kind in [1u16, 3, 7, 18, 24, 32, 60, 62, 63] {
            let mut bad = b.clone();
            bad[12..14].copy_from_slice(&kind.to_be_bytes());
            assert!(encode(g, &bad, 37).is_err());
        }
        for n in 0..b.len() {
            assert!(encode(g, &b[..n], 37).is_err());
        }
        for target in [
            None,
            Some(Target::Npc { serial: 2, life: 3 }),
            Some(Target::Player { entity: 1, life: 3 }),
            Some(Target::Player { entity: 2, life: 0 }),
        ] {
            assert!(encode(Guard { target, ..g }, &b, 37).is_err());
        }
    }
    #[test]
    fn rejects_wrong_header_before_any_attacker_body_read_and_keeps_coalesced_frame() {
        struct HeaderOnly {
            bytes: [u8; 16],
            at: usize,
        }
        impl Read for HeaderOnly {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                assert!(self.at < 16, "read forbidden body");
                let n = b.len().min(16 - self.at);
                b[..n].copy_from_slice(&self.bytes[self.at..self.at + n]);
                self.at += n;
                Ok(n)
            }
        }
        let expected = encode(guard(), &inner(), 37).unwrap();
        for (kind, len) in [(63, 211), (63, MAX_RESULT_BYTES + 1), (61, 212), (62, 212)] {
            let mut h: [u8; 16] = expected[..16].try_into().unwrap();
            h[4..8].copy_from_slice(&(len as u32).to_be_bytes());
            h[12..14].copy_from_slice(&(kind as u16).to_be_bytes());
            assert!(
                read_reply(&mut HeaderOnly { bytes: h, at: 0 }, guard(), &expected, 37).is_err()
            );
        }
        let bytes = response(&expected, 0);
        let mut joined = bytes.clone();
        joined.extend(&bytes);
        let mut cursor = std::io::Cursor::new(joined);
        read_reply(&mut cursor, guard(), &expected, 37).unwrap();
        assert_eq!(cursor.position(), bytes.len() as u64);
        read_reply(&mut cursor, guard(), &expected, 37).unwrap();
        assert_eq!(cursor.position(), 2 * bytes.len() as u64);
    }
    #[test]
    fn capability_is_opt_in_has_dependencies_and_cannot_appear_on_v36() {
        let mut h = protocol::Hello::current(1);
        assert_eq!(h.supported_capabilities & protocol::CAP_ACTION_LIFE, 0);
        h.supported_capabilities |= protocol::CAP_BATTLE_DATA | protocol::CAP_ACTION_LIFE;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_ok());
        for bit in [
            protocol::CAP_WORLD,
            protocol::CAP_MATCH_DESCRIPTOR,
            protocol::CAP_BATTLE_DATA,
        ] {
            let mut bad = h;
            bad.supported_capabilities &= !bit;
            assert!(protocol::encode(protocol::Message::Hello(bad)).is_err());
        }
        h.max_version = 36;
        assert!(protocol::encode(protocol::Message::Hello(h)).is_err());
        let ack = protocol::Ack {
            request_id: 1,
            version: 36,
            numeric_schema: 1,
            capabilities: h.supported_capabilities,
        };
        assert!(!negotiated(&ack));
        assert!(protocol::encode(protocol::Message::Ack(ack)).is_err());
    }
}
