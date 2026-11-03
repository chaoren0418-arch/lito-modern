//! GM requests always re-check authority on the server. CAP_GM_BOTS advertises
//! a wire implementation, never permission to mutate a room or read a bot in SQL.
use super::application_protocol::{self as app, ApplicationError, CharacterKey, SelectionContext};
use super::rooms;
pub const REQUEST_KIND: u16 = 30;
pub const RESULT_KIND: u16 = 31;
pub const MAX_FRAME_BYTES: usize = rooms::MAX_FRAME_BYTES + 64;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Request {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub room_id: u64,
    pub expected_revision: i64,
    /// Positive adds up to this many; zero removes this room's bots.
    pub count: u16,
}
impl Request {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        if !app::valid_context(self.context)
            || self.channel_id > 3
            || self.room_id == 0
            || self.room_id > i64::MAX as u64
            || self.expected_revision < 1
            || self.count > 100
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn read_common(self) -> rooms::Common {
        rooms::Common::read(self.context, self.channel_id)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Delivery {
    Result = 0,
    Busy = 1,
    Unavailable = 2,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub echo: Request,
    pub delivery: Delivery,
    pub added: u16,
    pub removed: u16,
    pub room: Option<rooms::Reply>,
}
impl Reply {
    pub fn validate(&self) -> Result<(), ApplicationError> {
        self.echo.validate()?;
        if self.added > 100 || self.removed > 100 {
            return Err(invalid());
        }
        if self.delivery != Delivery::Result {
            return if self.room.is_none() && self.added == 0 && self.removed == 0 {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        let room = self.room.as_ref().ok_or_else(invalid)?;
        room.validate_gm()?;
        if room.common != self.echo.read_common() {
            return Err(invalid());
        }
        if room.status != rooms::Status::Ok {
            return if self.added == 0 && self.removed == 0 {
                Ok(())
            } else {
                Err(invalid())
            };
        }
        let joined = room.joined.as_ref().ok_or_else(invalid)?;
        let changed = self.added != 0 || self.removed != 0;
        if joined.summary.room_id != self.echo.room_id
            || (if changed {
                self.echo.expected_revision.checked_add(1) != Some(joined.summary.revision)
            } else {
                self.echo.expected_revision != joined.summary.revision
            })
            || (if self.echo.count == 0 {
                self.added != 0
                    || joined
                        .members
                        .iter()
                        .any(|m| m.kind == rooms::MemberKind::ServerBot)
            } else {
                self.removed != 0 || self.added > self.echo.count
            })
        {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn confirms(&self, request: &Request, prior: Option<&rooms::Reply>) -> bool {
        if self.echo != *request || self.validate().is_err() {
            return false;
        }
        let Some(next) = &self.room else {
            return true;
        };
        if !rooms::follows(prior, next) {
            return false;
        }
        if next.status != rooms::Status::Ok {
            return true;
        }
        let (Some(old), Some(new)) = (prior.and_then(|p| p.joined.as_ref()), next.joined.as_ref())
        else {
            return false;
        };
        if old.summary.room_id != request.room_id
            || old.summary.revision != request.expected_revision
            || new.owner != old.owner
            || new.summary.settings != old.summary.settings
        {
            return false;
        }
        // Humans must neither disappear nor be forged by a bot-only mutation.
        let humans = |r: &rooms::RoomSnapshot| {
            r.members
                .iter()
                .filter(|m| m.kind == rooms::MemberKind::Human)
                .map(|m| m.identity())
                .collect::<Vec<_>>()
        };
        let before = old
            .members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::ServerBot)
            .count();
        let after = new
            .members
            .iter()
            .filter(|m| m.kind == rooms::MemberKind::ServerBot)
            .count();
        humans(old) == humans(new)
            && (request.count == 0
                || old
                    .members
                    .iter()
                    .filter(|m| m.kind == rooms::MemberKind::ServerBot)
                    .all(|m| new.members.iter().any(|n| n.identity() == m.identity())))
            && before
                .checked_add(self.added as usize)
                .and_then(|n| n.checked_sub(self.removed as usize))
                == Some(after)
    }
    pub fn notice(&self) -> String {
        match self.delivery {
            Delivery::Busy => "GM 操作忙碌，房間未更新；請稍後自行重試。".into(),
            Delivery::Unavailable => "GM 授權／來源讀取暫不可用，保留已確認的房間。".into(),
            Delivery::Result => match self.room.as_ref().map(|r| (r.status, r.reason)) {
                Some((rooms::Status::Ok, _)) => format!(
                    "伺服器確認假人操作：新增 {}、移除 {}。",
                    self.added, self.removed
                ),
                Some((_, rooms::Reason::GmNotAuthorized)) => {
                    "伺服器拒絕：目前來源帳號沒有 GM 假人權限。".into()
                }
                Some((status, reason)) => {
                    format!("伺服器拒絕假人操作：{status:?}／{reason:?}；已同步房間狀態。")
                }
                None => "GM 操作尚未確認。".into(),
            },
        }
    }
}
fn invalid() -> ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid bounded GM room frame",
    )
}
fn put_request(out: &mut Vec<u8>, r: Request) {
    out.extend(r.context.key.source_id.to_be_bytes());
    out.extend(r.context.key.object_id.to_be_bytes());
    out.extend(r.context.epoch.to_be_bytes());
    out.extend([r.channel_id, 0, 0, 0]);
    out.extend(r.room_id.to_be_bytes());
    out.extend(r.expected_revision.to_be_bytes());
    out.extend(r.count.to_be_bytes());
    out.extend([0, 0]);
}
pub fn encode_request(
    id: u32,
    request: Request,
    version: u16,
) -> Result<Vec<u8>, ApplicationError> {
    if !(7..=super::protocol::MAX_VERSION).contains(&version) {
        return Err(invalid());
    }
    request.validate()?;
    let mut out = app::header(id, REQUEST_KIND, 60)?;
    put_request(&mut out, request);
    Ok(out)
}
pub(crate) fn decode_body(bytes: &[u8], version: u16) -> Result<Reply, ApplicationError> {
    if !(7..=super::protocol::MAX_VERSION).contains(&version)
        || bytes.len() < 56
        || bytes.len() > MAX_FRAME_BYTES - 16
    {
        return Err(invalid());
    }
    let u64_at = |p| u64::from_be_bytes(bytes[p..p + 8].try_into().unwrap());
    let u32_at = |p| u32::from_be_bytes(bytes[p..p + 4].try_into().unwrap());
    let u16_at = |p| u16::from_be_bytes(bytes[p..p + 2].try_into().unwrap());
    if bytes[21..24] != [0; 3] || bytes[42..44] != [0; 2] || bytes[45..48] != [0; 3] {
        return Err(invalid());
    }
    let echo = Request {
        context: SelectionContext {
            key: CharacterKey {
                source_id: u32_at(0),
                object_id: u64_at(4),
            },
            epoch: u64_at(12),
        },
        channel_id: bytes[20],
        room_id: u64_at(24),
        expected_revision: u64_at(32) as i64,
        count: u16_at(40),
    };
    let delivery = match bytes[44] {
        0 => Delivery::Result,
        1 => Delivery::Busy,
        2 => Delivery::Unavailable,
        _ => return Err(invalid()),
    };
    let length = usize::try_from(u32_at(52)).map_err(|_| invalid())?;
    if length != bytes.len() - 56 {
        return Err(invalid());
    }
    let room = if length == 0 {
        None
    } else {
        Some(rooms::decode_gm_body(&bytes[56..], version)?)
    };
    let reply = Reply {
        echo,
        delivery,
        added: u16_at(48),
        removed: u16_at(50),
        room,
    };
    reply.validate()?;
    Ok(reply)
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(crate) fn request() -> Request {
        Request {
            context: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: u64::MAX,
            },
            channel_id: 3,
            room_id: 9,
            expected_revision: 2,
            count: 0,
        }
    }
    #[test]
    fn exact_44_byte_request_preserves_unsigned_epoch_and_old_versions_reject() {
        let bytes = encode_request(0xfedcba98, request(), 7).unwrap();
        let expected = "4c544d310000003cfedcba98001e0000000000070000000000000d05ffffffffffffffff030000000000000000000009000000000000000200000000";
        assert_eq!(
            bytes.iter().map(|x| format!("{x:02x}")).collect::<String>(),
            expected
        );
        for v in 1..=6 {
            assert!(encode_request(1, request(), v).is_err());
        }
        let mut r = request();
        r.count = 101;
        assert!(r.validate().is_err());
        r = request();
        r.context.epoch = 0;
        assert!(r.validate().is_err());
    }
    #[test]
    fn independent_java_gm_request_and_busy_literals_match_exact_bytes() {
        let decode_hex = |s: &str| {
            s.as_bytes()
                .chunks_exact(2)
                .map(|b| u8::from_str_radix(std::str::from_utf8(b).unwrap(), 16).unwrap())
                .collect::<Vec<_>>()
        };
        let request_hex = "4c544d310000003cfedcba98001e0000000000070000000000000d05ffffffffffffffff020000000000000100000002000000020000000300140000";
        let req = Request {
            channel_id: 2,
            room_id: 4_294_967_298,
            expected_revision: 8_589_934_595,
            count: 20,
            ..request()
        };
        assert_eq!(
            encode_request(0xfedcba98, req, 7).unwrap(),
            decode_hex(request_hex)
        );
        let busy_hex = concat!(
            "4c544d3100000048fedcba98001f0000",
            "000000070000000000000d05ffffffffffffffff020000000000000100000002000000020000000300140000",
            "010000000000000000000000"
        );
        let frame = app::decode_response_version(&decode_hex(busy_hex), 7).unwrap();
        assert_eq!(frame.request_id, 0xfedcba98);
        assert_eq!(
            frame.response,
            app::Response::GmBots(Reply {
                echo: req,
                delivery: Delivery::Busy,
                added: 0,
                removed: 0,
                room: None
            })
        );
    }
    #[test]
    fn busy_is_not_an_empty_room_and_padding_tail_and_truncation_reject() {
        let mut body = Vec::new();
        put_request(&mut body, request());
        body.extend([1, 0, 0, 0]);
        body.extend([0; 8]);
        let reply = decode_body(&body, 7).unwrap();
        let mut framed = app::header(1, RESULT_KIND, 16 + body.len()).unwrap();
        framed.extend(&body);
        assert_eq!(
            app::decode_response_version(&framed, 7).unwrap().response,
            app::Response::GmBots(reply.clone())
        );
        assert!(app::decode_response_version(&framed, 6).is_err());
        assert_eq!(reply.delivery, Delivery::Busy);
        assert!(reply.room.is_none());
        for end in 0..body.len() {
            assert!(decode_body(&body[..end], 7).is_err());
        }
        for p in [21, 22, 23, 42, 43, 45, 46, 47, 48, 50, 55] {
            let mut bad = body.clone();
            bad[p] = 1;
            assert!(decode_body(&bad, 7).is_err(), "{p}");
        }
        let mut bad = body.clone();
        bad.push(0);
        assert!(decode_body(&bad, 7).is_err());
        assert!(decode_body(&body, 6).is_err());
    }
    fn wire(reply: &Reply) -> Vec<u8> {
        wire_version(reply, 7)
    }
    fn wire_version(reply: &Reply, version: u16) -> Vec<u8> {
        let mut body = Vec::new();
        put_request(&mut body, reply.echo);
        body.extend([reply.delivery as u8, 0, 0, 0]);
        body.extend(reply.added.to_be_bytes());
        body.extend(reply.removed.to_be_bytes());
        let nested = reply
            .room
            .as_ref()
            .map(|r| rooms::tests::frame_version(r, version)[16..].to_vec())
            .unwrap_or_default();
        body.extend((nested.len() as u32).to_be_bytes());
        body.extend(nested);
        body
    }
    #[test]
    fn add_clear_and_denial_preserve_tagged_humans_and_exact_request_revision() {
        let mut prior = rooms::tests::sample(request().context, true);
        prior.common.channel_id = 3;
        let mut req = request();
        req.room_id = prior.joined.as_ref().unwrap().summary.room_id;
        req.expected_revision = 1;
        req.count = 1;
        let mut next = prior.clone();
        next.list_revision += 1;
        let room = next.joined.as_mut().unwrap();
        let mut bot = room.members[0].clone();
        bot.kind = rooms::MemberKind::ServerBot;
        bot.name = "BOT01".into();
        bot.team = 1;
        room.members.push(bot);
        room.summary.users += 1;
        room.summary.revision += 1;
        next.rooms[0] = room.summary.clone();
        let result = Reply {
            echo: req,
            delivery: Delivery::Result,
            added: 1,
            removed: 0,
            room: Some(next.clone()),
        };
        assert!(result.confirms(&req, Some(&prior)));
        assert_eq!(decode_body(&wire(&result), 7).unwrap(), result);
        for version in 7..=18 {
            assert_eq!(wire_version(&result, version), wire(&result));
            assert_eq!(
                decode_body(&wire_version(&result, version), version).unwrap(),
                result
            );
        }
        let mut bounded = result.clone();
        bounded.room.as_mut().unwrap().options[0].maximum_players = 48;
        let extended = wire_version(&bounded, 19);
        assert_eq!(extended.len(), wire(&result).len() + 80);
        assert_eq!(&extended[126..128], &48_u16.to_be_bytes());
        assert_eq!(decode_body(&extended, 19).unwrap(), bounded);
        assert!(decode_body(&extended, 18).is_err());
        let mut bad = extended.clone();
        bad[126..128].copy_from_slice(&101_u16.to_be_bytes());
        assert!(decode_body(&bad, 19).is_err());
        let mut wrong = result.clone();
        wrong.echo.context.epoch -= 1;
        assert!(!wrong.confirms(&req, Some(&prior)));
        let mut wrong = result.clone();
        wrong.added = 0;
        assert!(!wrong.confirms(&req, Some(&prior)));
        let mut clear_req = req;
        clear_req.count = 0;
        clear_req.expected_revision = 2;
        let mut cleared = next.clone();
        cleared.list_revision += 1;
        let r = cleared.joined.as_mut().unwrap();
        r.members.retain(|m| m.kind == rooms::MemberKind::Human);
        r.summary.users = 1;
        r.summary.revision += 1;
        cleared.rooms[0] = r.summary.clone();
        let clear = Reply {
            echo: clear_req,
            delivery: Delivery::Result,
            added: 0,
            removed: 1,
            room: Some(cleared),
        };
        assert!(clear.confirms(&clear_req, Some(&next)));
        assert_eq!(decode_body(&wire(&clear), 7).unwrap(), clear);
        prior.status = rooms::Status::Denied;
        prior.reason = rooms::Reason::GmNotAuthorized;
        let denied = Reply {
            echo: req,
            delivery: Delivery::Result,
            added: 0,
            removed: 0,
            room: Some(prior.clone()),
        };
        assert_eq!(decode_body(&wire(&denied), 7).unwrap(), denied);
        assert!(
            rooms::decode_body_version(&rooms::tests::frame_version(&prior, 7)[16..], 7).is_err(),
            "GM-only reason must not enter regular room envelope"
        );
        assert!(denied.notice().contains("沒有 GM"));
    }
}
