//! Bounded candidate and explicitly selected live codecs. Version-only
//! Application decoders remain closed; live reads require the pending intent.
use super::*;
pub(crate) fn encode_intent_selected(
    q: &Intent,
    peer: super::super::protocol::extensions::Selected,
) -> Result<Vec<u8>, app::ApplicationError> {
    require(negotiated(peer))?;
    encode_candidate_intent(q)
}
pub fn encode_candidate_intent(q: &Intent) -> Result<Vec<u8>, app::ApplicationError> {
    q.validate()?;
    let s = q.stamp;
    let t = q.target;
    let mut b = app::header(q.request_id, REQUEST_KIND, REQUEST_BYTES)?;
    b.extend(s.context.key.source_id.to_be_bytes());
    b.extend(s.context.key.object_id.to_be_bytes());
    b.extend(s.context.epoch.to_be_bytes());
    b.extend(u32::from(s.channel).to_be_bytes());
    for n in s.world.into_iter().chain([
        s.entity,
        s.life,
        s.revision,
        q.sequence,
        q.equipment_revision,
    ]) {
        b.extend(n.to_be_bytes());
    }
    b.extend(q.x.to_be_bytes());
    b.extend(q.y.to_be_bytes());
    b.extend(t.entity.to_be_bytes());
    b.extend((t.identity.kind as u32).to_be_bytes());
    b.extend(t.identity.source_id.to_be_bytes());
    b.extend(t.identity.id.to_be_bytes());
    b.extend(t.life.to_be_bytes());
    b.extend(t.equipment_revision.to_be_bytes());
    b.extend(t.x.to_be_bytes());
    b.extend(t.y.to_be_bytes());
    Ok(b)
}
fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_be_bytes(b[at..at + 4].try_into().unwrap())
}
fn i64_at(b: &[u8], at: usize) -> i64 {
    i64::from_be_bytes(b[at..at + 8].try_into().unwrap())
}
fn header(b: &[u8], kind: u16) -> Result<(), app::ApplicationError> {
    require(
        b.len() >= 16
            && &b[..4] == b"LTM1"
            && u32_at(b, 4) as usize == b.len()
            && u32_at(b, 8) != 0
            && b[12..14] == kind.to_be_bytes()
            && b[14..16] == [0, 0],
    )
}
pub fn decode_candidate_intent(b: &[u8]) -> Result<Intent, app::ApplicationError> {
    require(b.len() == REQUEST_BYTES)?;
    header(b, REQUEST_KIND)?;
    let kind = match u32_at(b, 120) {
        0 => participant::MemberKind::Human,
        1 => participant::MemberKind::ServerBot,
        _ => return Err(invalid()),
    };
    let q = Intent {
        request_id: u32_at(b, 8),
        stamp: Stamp {
            context: app::SelectionContext {
                key: app::CharacterKey {
                    source_id: u32_at(b, 16),
                    object_id: i64_at(b, 20) as u64,
                },
                epoch: i64_at(b, 28) as u64,
            },
            channel: u32_at(b, 36).try_into().map_err(|_| invalid())?,
            world: [i64_at(b, 40), i64_at(b, 48), i64_at(b, 56)],
            entity: i64_at(b, 64),
            life: i64_at(b, 72),
            revision: i64_at(b, 80),
        },
        sequence: i64_at(b, 88),
        equipment_revision: i64_at(b, 96),
        x: u32_at(b, 104) as i32,
        y: u32_at(b, 108) as i32,
        target: Target {
            entity: i64_at(b, 112),
            identity: participant::ParticipantId {
                kind,
                source_id: u32_at(b, 124),
                id: i64_at(b, 128) as u64,
            },
            life: i64_at(b, 136),
            equipment_revision: i64_at(b, 144),
            x: u32_at(b, 152) as i32,
            y: u32_at(b, 156) as i32,
        },
    };
    q.validate()?;
    Ok(q)
}
pub fn encode_candidate_reply(r: &CandidateReply) -> Result<Vec<u8>, app::ApplicationError> {
    r.validate()?;
    let data = healing::encode(&r.battle)?;
    let size = RESULT_PREFIX_BYTES
        .checked_add(data.len())
        .ok_or_else(invalid)?;
    require(size <= MAX_RESULT_BYTES)?;
    let mut b = encode_candidate_intent(&r.echo)?;
    b[4..8].copy_from_slice(&(size as u32).to_be_bytes());
    b[12..14].copy_from_slice(&RESULT_KIND.to_be_bytes());
    b.push(r.reason as u8);
    b.extend([0u8; 3]);
    let l = r.lifecycle;
    b.extend(l.revision.to_be_bytes());
    b.push(l.phase as u8);
    b.push(l.abort as u8);
    b.push(l.settlement.map_or(0, |s| s as u8 + 1));
    b.push(l.race_settlement.map_or(0, |s| s as u8 + 1));
    b.extend(l.race_terminal_revision.to_be_bytes());
    b.extend([0u8; 4]);
    b.extend((data.len() as u32).to_be_bytes());
    b.extend(data);
    Ok(b)
}
pub fn decode_candidate_reply(
    b: &[u8],
    expected: &Intent,
) -> Result<CandidateReply, app::ApplicationError> {
    let (echo, reason, lifecycle) = decode_fields(b, expected, MAX_RESULT_BYTES)?;
    let battle = healing::decode(&b[RESULT_PREFIX_BYTES..])?;
    let r = CandidateReply {
        echo,
        reason,
        lifecycle,
        battle,
    };
    r.validate()?;
    Ok(r)
}
pub(crate) fn decode_reply_selected(
    b: &[u8],
    expected: &Intent,
    peer: super::super::protocol::extensions::Selected,
) -> Result<Reply, app::ApplicationError> {
    require(negotiated(peer))?;
    let (echo, reason, lifecycle) = decode_fields(b, expected, selected_limit(peer))?;
    let data = super::super::battle_payload::decode_selected(&b[RESULT_PREFIX_BYTES..], peer)?;
    let r = Reply {
        echo,
        reason,
        lifecycle,
        data,
    };
    r.validate()?;
    Ok(r)
}
fn decode_fields(
    b: &[u8],
    expected: &Intent,
    maximum: usize,
) -> Result<(Intent, Reason, Lifecycle), app::ApplicationError> {
    expected.validate()?;
    require((RESULT_PREFIX_BYTES..=maximum).contains(&b.len()))?;
    header(b, RESULT_KIND)?;
    let mut echo = b[..REQUEST_BYTES].to_vec();
    echo[4..8].copy_from_slice(&(REQUEST_BYTES as u32).to_be_bytes());
    echo[12..14].copy_from_slice(&REQUEST_KIND.to_be_bytes());
    let echo = decode_candidate_intent(&echo)?;
    require(echo == *expected && b[161..164] == [0u8; 3])?;
    let reason = *[
        Reason::None,
        Reason::NotActive,
        Reason::Dead,
        Reason::Unlearned,
        Reason::Unsupported,
        Reason::TooFast,
        Reason::NoMp,
        Reason::TooHeavy,
        Reason::StaleState,
        Reason::Numeric,
        Reason::TargetDead,
        Reason::StaleTarget,
        Reason::WrongInstance,
        Reason::OutOfRange,
        Reason::Blocked,
    ]
    .get(b[160] as usize)
    .ok_or_else(invalid)?;
    let lifecycle = Lifecycle::read(&b[164..188])?;
    require(u32_at(b, 188) as usize == b.len() - RESULT_PREFIX_BYTES)?;
    Ok((echo, reason, lifecycle))
}
/// Bound kind, request ID and size before allocation. Read exactly one reply.
pub fn read_candidate_reply(
    reader: &mut impl std::io::Read,
    expected: &Intent,
) -> Result<CandidateReply, app::ApplicationError> {
    decode_candidate_reply(&read_frame(reader, expected, MAX_RESULT_BYTES)?, expected)
}
pub(crate) fn read_reply_selected(
    reader: &mut impl std::io::Read,
    expected: &Intent,
    peer: super::super::protocol::extensions::Selected,
) -> Result<Reply, app::ApplicationError> {
    require(negotiated(peer))?;
    decode_reply_selected(
        &read_frame(reader, expected, selected_limit(peer))?,
        expected,
        peer,
    )
}
fn selected_limit(peer: super::super::protocol::extensions::Selected) -> usize {
    RESULT_PREFIX_BYTES + super::super::battle_payload::Format::Selected(peer).maximum()
}
fn read_frame(
    reader: &mut impl std::io::Read,
    expected: &Intent,
    maximum: usize,
) -> Result<Vec<u8>, app::ApplicationError> {
    expected.validate()?;
    let mut h = [0u8; 16];
    reader.read_exact(&mut h).map_err(|_| invalid())?;
    let size = u32_at(&h, 4) as usize;
    require(
        &h[..4] == b"LTM1"
            && u32_at(&h, 8) == expected.request_id
            && h[12..14] == RESULT_KIND.to_be_bytes()
            && h[14..16] == [0u8; 2]
            && (RESULT_PREFIX_BYTES..=maximum).contains(&size),
    )?;
    let mut b = vec![0u8; size];
    b[..16].copy_from_slice(&h);
    reader.read_exact(&mut b[16..]).map_err(|_| invalid())?;
    Ok(b)
}
