//! Confirmed private presentation, distinct from room-local public chat.
//! Binding is supplied only after confirmed navigation/general membership;
//! local enrollment generation rejects retired jobs even after reconnect.
use super::application_protocol::{self as app, SelectionContext};
use super::direct_chat_protocol as wire;
use std::{fmt, sync::Arc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Enrollment {
    pub context: SelectionContext,
    pub channel_id: u8,
    pub generation: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub enrollment: Enrollment,
    pub room_id: u64,
    pub membership: u64,
}
impl Token {
    fn read(self) -> wire::Request {
        wire::Request::read(
            self.enrollment.context,
            self.enrollment.channel_id,
            self.room_id,
            self.membership,
        )
    }
    fn matches(self, r: &wire::Request) -> bool {
        r.context == self.enrollment.context
            && r.channel_id == self.enrollment.channel_id
            && r.room_id == self.room_id
            && r.membership == self.membership
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Outcome {
    #[default]
    None,
    Confirmed(wire::Action),
    RateLimited,
    Unreachable,
    BlockListFull,
    Unavailable,
    Silenced,
    Unknown(wire::Action),
}
#[derive(Clone, Default, PartialEq, Eq)]
pub struct State {
    offered: bool,
    enrollment: Option<Enrollment>,
    admission: Option<Token>,
    owner: Option<wire::Peer>,
    sequence: u64,
    cursor: u64,
    available: bool,
    gap: bool,
    preferences: Option<wire::Preferences>,
    blocked: Arc<Vec<wire::Address>>,
    rows: Arc<Vec<wire::Message>>,
    outcome: Outcome,
}
impl fmt::Debug for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("DirectChatState[redacted]")
    }
}
impl State {
    pub fn supported(&self) -> bool {
        self.offered
    }
    pub fn enrollment(&self) -> Option<Enrollment> {
        self.enrollment
    }
    pub fn owner(&self) -> Option<&wire::Peer> {
        self.owner.as_ref()
    }
    pub fn messages(&self) -> &[wire::Message] {
        &self.rows
    }
    pub fn blocked(&self) -> &[wire::Address] {
        &self.blocked
    }
    pub fn preferences(&self) -> Option<wire::Preferences> {
        self.preferences
    }
    pub fn outcome(&self) -> Outcome {
        self.outcome
    }
    pub fn history_gap(&self) -> bool {
        self.gap
    }
    pub fn waiting(&self) -> bool {
        self.offered
            && self.enrollment.is_some()
            && (self.owner.is_none() || self.available && self.admission.is_none())
    }
    pub fn token(&self) -> Option<Token> {
        (self.owner.is_some() && self.available)
            .then_some(self.admission)
            .flatten()
    }

    /// None admission temporarily disables commands while a new room's public
    /// membership is being confirmed. It does not erase this enrollment's inbox.
    /// A new successful channel/navigation admission MUST use a new generation,
    /// even when returning to the same source, character and channel number.
    pub fn bind(
        &mut self,
        offered: bool,
        enrollment: Option<Enrollment>,
        admission: Option<Token>,
    ) -> Result<(), &'static str> {
        if !offered && (enrollment.is_some() || admission.is_some())
            || enrollment.is_some_and(|e| {
                !app::valid_context(e.context) || e.channel_id > 3 || e.generation == 0
            })
            || admission
                .is_some_and(|t| Some(t.enrollment) != enrollment || t.read().validate().is_err())
        {
            return Err("私訊入場資格不正確");
        }
        if self.offered != offered || self.enrollment != enrollment {
            *self = Self {
                offered,
                enrollment,
                ..Self::default()
            };
        }
        self.admission = admission;
        Ok(())
    }
    pub fn read(&self) -> Option<(Token, wire::Request)> {
        if !self.offered || self.owner.is_some() && !self.available {
            return None;
        }
        let token = self.admission?;
        let mut r = token.read();
        r.after = self.cursor;
        Some((token, r))
    }
    /// Preparing/enqueuing a mutation changes no confirmed rows or preferences.
    /// Private SEND text, including .de, is never interpreted as a GM command.
    pub fn command(
        &self,
        token: Token,
        action: wire::Action,
        target: Option<wire::Address>,
        preferences: Option<wire::Preferences>,
        text: String,
    ) -> Result<wire::Request, &'static str> {
        if self.token() != Some(token) || action == wire::Action::Read {
            return Err("私訊範圍已改變或尚未確認");
        }
        let mut r = token.read();
        r.after = self.cursor;
        r.sequence = self.sequence.checked_add(1).ok_or("私訊序號已用盡")?;
        r.action = action;
        r.target = target;
        r.preferences = preferences;
        r.text = text;
        r.validate()
            .map_err(|_| "私訊內容、收件來源或控制設定不正確")?;
        Ok(r)
    }
    pub fn receive(&mut self, token: Token, reply: wire::Reply) -> Result<(), &'static str> {
        reply.validate().map_err(|_| "私訊回覆格式不正確")?;
        if !self.offered
            || self.admission != Some(token)
            || self.enrollment != Some(token.enrollment)
            || !token.matches(&reply.echo)
        {
            return Err("私訊回覆不屬於目前入場資格");
        }
        if let Some(owner) = &self.owner {
            let sequence = if reply.echo.action == wire::Action::Read {
                Some(self.sequence)
            } else {
                self.sequence.checked_add(1)
            };
            if owner != &reply.owner
                || sequence != Some(reply.sequence)
                || reply.echo.after > self.cursor
                || reply.cursor < self.cursor
            {
                return Err("私訊身分、確認序號或收件進度已失效");
            }
        } else if reply.echo.action != wire::Action::Read
            || reply.echo.after != 0
            || reply.echo.sequence != 0
        {
            return Err("私訊必須先取得權威收件狀態");
        }
        for row in &reply.messages {
            if self
                .rows
                .iter()
                .find(|old| old.id == row.id)
                .is_some_and(|old| old != row)
            {
                return Err("私訊歷史內容被不一致回覆改寫");
            }
        }
        let mut rows = self.rows.as_ref().clone();
        rows.extend(
            reply
                .messages
                .iter()
                .filter(|m| m.id > self.cursor)
                .cloned(),
        );
        let trimmed = rows.len() > wire::MAX_MESSAGES;
        if trimmed {
            rows.drain(..rows.len() - wire::MAX_MESSAGES);
        }
        // All rejection paths above precede publication of any private state.
        self.rows = Arc::new(rows);
        self.owner = Some(reply.owner);
        self.sequence = reply.sequence;
        self.cursor = reply.cursor;
        self.available = reply.available;
        self.gap |= reply.truncated || trimmed;
        self.preferences = Some(reply.preferences);
        self.blocked = Arc::new(reply.blocked);
        if reply.echo.action != wire::Action::Read || !reply.available {
            self.outcome = if reply.echo.action == wire::Action::Read {
                Outcome::Unavailable
            } else {
                match reply.status {
                    wire::Status::Ok => Outcome::Confirmed(reply.echo.action),
                    wire::Status::RateLimited => Outcome::RateLimited,
                    wire::Status::Unreachable => Outcome::Unreachable,
                    wire::Status::BlockListFull => Outcome::BlockListFull,
                    wire::Status::Unavailable => Outcome::Unavailable,
                    wire::Status::Silenced => Outcome::Silenced,
                }
            };
        }
        Ok(())
    }
    pub fn clear(&mut self) {
        *self = Self::default();
    }
    /// Unknown is not success, denial or rollback. Discard all private identity,
    /// rows and controls when retiring; retain only the kind of uncertain action.
    pub fn retire_unknown(&mut self, action: wire::Action) -> Result<(), &'static str> {
        if action == wire::Action::Read {
            return Err("讀取不是待確認的變更");
        }
        *self = Self {
            outcome: Outcome::Unknown(action),
            ..Self::default()
        };
        Ok(())
    }
}

#[cfg(test)]
#[path = "direct_chat_state_tests.rs"]
mod tests;
