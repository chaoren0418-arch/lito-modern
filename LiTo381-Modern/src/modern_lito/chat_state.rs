//! Bounded, confirmed chat presentation. No optimistic rows or saved drafts.
use super::application_protocol::{NavigationPhase, NavigationReply, SelectionContext};
use super::{chat_protocol as wire, filtered_chat_protocol as filtered, rooms};
use std::{fmt, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub room_id: u64,
}
impl Scope {
    pub fn current(nav: Option<&NavigationReply>, rooms: Option<&rooms::Reply>) -> Option<Self> {
        let nav = nav?;
        let rooms = rooms?;
        let channel_id = nav.snapshot.channel_id?;
        if nav.snapshot.phase != NavigationPhase::Lobby
            || rooms.common.context != nav.context
            || rooms.common.channel_id != channel_id
        {
            return None;
        }
        Some(Self {
            context: nav.context,
            channel_id,
            room_id: rooms.joined.as_ref().map_or(0, |r| r.summary.room_id),
        })
    }
    fn request(self) -> wire::Request {
        wire::Request::read(self.context, self.channel_id, self.room_id)
    }
    fn matches(self, r: &wire::Request) -> bool {
        self.context == r.context && self.channel_id == r.channel_id && self.room_id == r.room_id
    }
}

/// Captured by the input owner, not inferred later from whatever page is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub scope: Scope,
    pub membership: u64,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Delivery {
    #[default]
    None,
    Confirmed,
    RateLimited,
    CommandRequired,
    Unavailable,
    Silenced,
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct State {
    offered: bool,
    filtered: bool,
    scope: Option<Scope>,
    received: bool,
    membership: u64,
    sequence: u64,
    cursor: u64,
    available: bool,
    gap: bool,
    rows: Arc<Vec<wire::Message>>,
    filtered_rows: Arc<Vec<filtered::Message>>,
    filtered_owner: Option<super::direct_chat_protocol::Peer>,
    delivery: Delivery,
}
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ChatState[redacted]")
    }
}
impl State {
    pub fn filtered(&self) -> bool {
        self.offered && self.filtered
    }
    pub fn filtered_messages(&self) -> &[filtered::Message] {
        &self.filtered_rows
    }
    pub fn scope(&self) -> Option<Scope> {
        self.scope
    }
    pub fn token(&self) -> Option<Token> {
        (self.received && self.available).then_some(Token {
            scope: self.scope?,
            membership: self.membership,
        })
    }
    pub fn messages(&self) -> &[wire::Message] {
        &self.rows
    }
    pub fn delivery(&self) -> Delivery {
        self.delivery
    }
    pub fn history_gap(&self) -> bool {
        self.gap
    }
    pub fn supported(&self) -> bool {
        self.offered
    }
    pub fn waiting(&self) -> bool {
        self.offered && self.scope.is_some() && !self.received
    }
    pub fn align(&mut self, offered: bool, scope: Option<Scope>) {
        self.align_profile(offered, false, scope);
    }
    pub fn align_profile(&mut self, offered: bool, filtered: bool, scope: Option<Scope>) {
        let filtered = offered && filtered;
        if self.offered != offered || self.filtered != filtered || self.scope != scope {
            *self = Self {
                offered,
                filtered,
                scope,
                ..Self::default()
            };
        }
    }
    /// Every publication removes prior-audience history, including background
    /// room updates and denied navigation that reconstructs a new view.
    pub fn retain_scope(&mut self, scope: Option<Scope>) {
        self.align_profile(self.offered, self.filtered, scope);
    }
    pub fn read(&self) -> Option<wire::Request> {
        if self.filtered || !self.offered || self.received && !self.available {
            return None;
        }
        let mut r = self.scope?.request();
        if self.received {
            r.membership = self.membership;
            r.after = self.cursor;
        }
        Some(r)
    }
    pub fn send(&self, token: Token, text: String) -> Result<wire::Request, &'static str> {
        if self.filtered || self.token() != Some(token) {
            return Err("聊天範圍已改變或尚未確認，請重新確認目前頁面");
        }
        let mut r = token.scope.request();
        r.membership = token.membership;
        r.sequence = self.sequence.checked_add(1).ok_or("聊天序號已用盡")?;
        r.after = self.cursor;
        r.action = wire::Action::Send;
        r.text = text;
        r.validate()
            .map_err(|_| "聊天內容無效：最多256字且512 UTF-8 bytes，不可含控制字元")?;
        Ok(r)
    }
    pub fn receive(&mut self, reply: wire::Reply) -> Result<(), &'static str> {
        reply.validate().map_err(|_| "聊天回覆格式不正確")?;
        if self.filtered || !self.offered || !self.scope.is_some_and(|s| s.matches(&reply.echo)) {
            return Err("聊天回覆不屬於目前頁面");
        }
        if self.received {
            let sequence = if reply.echo.action == wire::Action::Send {
                self.sequence.checked_add(1)
            } else {
                Some(self.sequence)
            };
            if reply.membership != self.membership
                || reply.echo.membership != self.membership
                || reply.echo.after > self.cursor
                || reply.cursor < self.cursor
                || sequence != Some(reply.sequence)
            {
                return Err("聊天確認世代或序號已失效");
            }
        } else if reply.echo.action != wire::Action::Read
            || reply.echo.membership != 0
            || reply.echo.after != 0
        {
            return Err("聊天必須先讀取目前成員資格");
        }
        // A queued SEND may echo an older cursor than the most recent periodic
        // READ. Existing rows must agree; never append duplicates or rewrite them.
        for row in &reply.messages {
            if self
                .rows
                .iter()
                .find(|old| old.id == row.id)
                .is_some_and(|old| old != row)
            {
                return Err("聊天歷史內容被不一致回覆改寫");
            }
        }
        let new_rows: Vec<_> = reply
            .messages
            .iter()
            .filter(|m| !self.received || m.id > self.cursor)
            .cloned()
            .collect();
        if !new_rows.is_empty() {
            let mut rows = self.rows.as_ref().clone();
            rows.extend(new_rows);
            if rows.len() > wire::MAX_MESSAGES {
                rows.drain(..rows.len() - wire::MAX_MESSAGES);
                self.gap = true;
            }
            self.rows = Arc::new(rows);
        }
        self.received = true;
        self.membership = reply.membership;
        self.sequence = reply.sequence;
        self.cursor = reply.cursor;
        self.available = reply.available;
        self.gap |= reply.truncated;
        if reply.echo.action == wire::Action::Send || !reply.available {
            self.delivery = match reply.status {
                wire::Status::Ok => Delivery::Confirmed,
                wire::Status::RateLimited => Delivery::RateLimited,
                wire::Status::CommandRequired => Delivery::CommandRequired,
                wire::Status::Unavailable => Delivery::Unavailable,
            };
        }
        Ok(())
    }
    pub fn filtered_read(&self) -> Option<filtered::Request> {
        if !self.filtered() || self.received && !self.available {
            return None;
        }
        let s = self.scope?;
        let mut r = filtered::Request::read(s.context, s.channel_id, s.room_id);
        if self.received {
            r.membership = self.membership;
            r.after = self.cursor;
        }
        Some(r)
    }
    pub fn filtered_send(
        &self,
        token: Token,
        kind: filtered::Kind,
        text: String,
    ) -> Result<filtered::Request, &'static str> {
        if !self.filtered() || self.token() != Some(token) {
            return Err("新版聊天範圍尚未確認或已改變");
        }
        let mut r = self.filtered_read().ok_or("新版聊天不可用")?;
        r.action = wire::Action::Send;
        r.kind = kind;
        r.text = text;
        r.sequence = self.sequence.checked_add(1).ok_or("聊天序號已用盡")?;
        r.validate()
            .map_err(|_| "聊天內容無效：最多256字且512 UTF-8 bytes，不可含控制字元")?;
        Ok(r)
    }
    /// Separate recipient-local history: never fabricate a legacy shared-log
    /// reply. A successful receipt can confirm SEND with no self inbox row.
    pub fn receive_filtered(&mut self, reply: filtered::Reply) -> Result<(), &'static str> {
        reply.validate().map_err(|_| "新版聊天回覆格式不正確")?;
        let scope = Scope {
            context: reply.echo.context,
            channel_id: reply.echo.channel_id,
            room_id: reply.echo.room_id,
        };
        if !self.filtered() || self.scope != Some(scope) {
            return Err("新版聊天回覆不屬於目前頁面");
        }
        if self.received {
            let sequence = if reply.echo.action == wire::Action::Send {
                self.sequence.checked_add(1)
            } else {
                Some(self.sequence)
            };
            if reply.membership != self.membership
                || reply.echo.membership != self.membership
                || reply.echo.after > self.cursor
                || reply.cursor < self.cursor
                || sequence != Some(reply.sequence)
                || self.filtered_owner.as_ref() != Some(&reply.owner)
            {
                return Err("新版聊天確認世代、來源或序號已失效");
            }
        } else if reply.echo.action != wire::Action::Read
            || reply.echo.membership != 0
            || reply.echo.after != 0
        {
            return Err("新版聊天必須先取得目前入場確認");
        }
        for row in &reply.messages {
            if self
                .filtered_rows
                .iter()
                .find(|old| old.id == row.id)
                .is_some_and(|old| old != row)
            {
                return Err("新版聊天歷史被不一致回覆改寫");
            }
        }
        let incoming: Vec<_> = reply
            .messages
            .iter()
            .filter(|m| !self.received || m.id > self.cursor)
            .cloned()
            .collect();
        if !incoming.is_empty() {
            let mut rows = self.filtered_rows.as_ref().clone();
            rows.extend(incoming);
            if rows.len() > filtered::MAX_MESSAGES {
                rows.drain(..rows.len() - filtered::MAX_MESSAGES);
                self.gap = true;
            }
            self.filtered_rows = Arc::new(rows);
        }
        self.received = true;
        self.membership = reply.membership;
        self.sequence = reply.sequence;
        self.cursor = reply.cursor;
        self.available = reply.available;
        self.gap |= reply.truncated;
        self.filtered_owner = Some(reply.owner);
        if reply.echo.action == wire::Action::Send || !reply.available {
            self.delivery = match reply.status {
                filtered::Status::Ok => Delivery::Confirmed,
                filtered::Status::RateLimited => Delivery::RateLimited,
                filtered::Status::CommandRequired => Delivery::CommandRequired,
                filtered::Status::Silenced => Delivery::Silenced,
                filtered::Status::Unavailable => Delivery::Unavailable,
            };
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "chat_filtered_state_tests.rs"]
mod filtered_tests;

#[cfg(test)]
mod tests {
    use super::super::application_protocol::CharacterKey;
    use super::*;
    fn scope() -> Scope {
        Scope {
            context: SelectionContext {
                key: CharacterKey {
                    source_id: 7,
                    object_id: 3333,
                },
                epoch: 1,
            },
            channel_id: 0,
            room_id: 0,
        }
    }
    fn reply(s: &State) -> wire::Reply {
        wire::Reply {
            echo: s.read().unwrap(),
            status: wire::Status::Ok,
            membership: 1,
            sequence: s.sequence,
            cursor: s.cursor,
            available: true,
            truncated: false,
            messages: vec![],
        }
    }
    fn ready() -> State {
        let mut s = State::default();
        s.align(true, Some(scope()));
        s.receive(reply(&s)).unwrap();
        s
    }
    fn row(id: u64) -> wire::Message {
        wire::Message {
            id,
            sender: scope().context.key,
            character_name: "3333".into(),
            source_name: "太陽神".into(),
            text: format!("訊息{id}"),
        }
    }
    #[test]
    fn bootstrap_and_navigation_do_not_leak_old_history_or_reuse_membership() {
        let mut s = ready();
        let old = s.token().unwrap();
        let mut r = reply(&s);
        r.cursor = 1;
        r.messages.push(row(1));
        s.receive(r).unwrap();
        s.align(
            true,
            Some(Scope {
                room_id: 42,
                ..scope()
            }),
        );
        assert!(s.messages().is_empty() && s.waiting());
        assert!(s.send(old, "old".into()).is_err());
        s.align(true, Some(scope()));
        let mut r = reply(&s);
        r.membership = 3;
        s.receive(r).unwrap();
        assert_ne!(s.token(), Some(old));
        assert!(s.send(old, "old".into()).is_err());
        s.retain_scope(None);
        assert!(s.messages().is_empty() && s.token().is_none());
    }
    #[test]
    fn bounded_history_deduplicates_stale_send_cursor_and_empty_poll_shares_rows() {
        let mut s = ready();
        let request = s.send(s.token().unwrap(), "訊息2".into()).unwrap();
        let mut r = reply(&s);
        r.cursor = 1;
        r.messages.push(row(1));
        s.receive(r).unwrap();
        let rows = s.rows.clone();
        s.receive(reply(&s)).unwrap();
        assert!(Arc::ptr_eq(&rows, &s.rows));
        let r = wire::Reply {
            echo: request,
            status: wire::Status::Ok,
            membership: 1,
            sequence: 1,
            cursor: 2,
            available: true,
            truncated: false,
            messages: vec![row(1), row(2)],
        };
        s.receive(r).unwrap();
        assert_eq!(s.messages().len(), 2);
        let mut r = reply(&s);
        r.cursor = 66;
        r.messages = (3..=66).map(row).collect();
        s.receive(r).unwrap();
        assert_eq!(s.messages().len(), 64);
        assert_eq!(s.messages()[0].id, 3);
        assert!(s.history_gap());
    }
    #[test]
    fn denial_never_fabricates_delivery_and_next_send_uses_consumed_sequence() {
        let mut s = ready();
        for status in [
            wire::Status::RateLimited,
            wire::Status::CommandRequired,
            wire::Status::Unavailable,
        ] {
            let request = s.send(s.token().unwrap(), ".de 20".into()).unwrap();
            let seq = request.sequence;
            let r = wire::Reply {
                echo: request,
                status,
                membership: 1,
                sequence: seq,
                cursor: 0,
                available: status != wire::Status::Unavailable,
                truncated: false,
                messages: vec![],
            };
            s.receive(r).unwrap();
            assert!(s.messages().is_empty());
            assert_ne!(s.delivery(), Delivery::Confirmed);
        }
        assert!(s.token().is_none() && s.read().is_none());
    }
    #[test]
    fn stale_source_epoch_membership_cursor_and_rewritten_history_fail_without_mutation() {
        let mut s = ready();
        let mut r = reply(&s);
        r.cursor = 1;
        r.messages.push(row(1));
        s.receive(r).unwrap();
        let mut cases = vec![];
        let base = reply(&s);
        let mut r = base.clone();
        r.echo.context.key.source_id = 42;
        cases.push(r);
        let mut r = base.clone();
        r.echo.context.epoch = 2;
        cases.push(r);
        let mut r = base.clone();
        r.echo.membership = 2;
        r.membership = 2;
        cases.push(r);
        let mut r = base.clone();
        r.echo.after = 0;
        r.cursor = 0;
        cases.push(r);
        let mut r = base;
        r.echo.after = 0;
        r.messages.push(row(1));
        r.messages[0].text = "偽造".into();
        cases.push(r);
        for r in cases {
            assert!(s.receive(r).is_err());
            assert_eq!(s.messages(), [row(1)]);
            assert_eq!(s.cursor, 1);
        }
    }
    #[test]
    fn unsupported_and_unconfirmed_state_never_create_requests_or_expose_text_in_debug() {
        let mut s = State::default();
        s.align(false, Some(scope()));
        assert!(s.read().is_none());
        assert!(s.token().is_none());
        s.align(true, Some(scope()));
        assert!(s.waiting());
        assert!(s.token().is_none());
        s.receive(reply(&s)).unwrap();
        let mut r = reply(&s);
        r.cursor = 1;
        r.messages.push(row(1));
        s.receive(r).unwrap();
        assert_eq!(format!("{s:?}"), "ChatState[redacted]");
    }
}
