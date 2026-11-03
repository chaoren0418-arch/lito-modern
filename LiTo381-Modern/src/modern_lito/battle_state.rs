//! Atomic v39 service + engine chronology for one authenticated TLS owner.
//! This is neither a renderer/LOAD certificate nor a command/SET permit. The
//! wire decoder separately verifies the exact operation intent and receipt.
use super::{
    application_protocol as app,
    battle_action::Guard,
    battle_follow::{Follower, Observation},
    battle_operation::{self as operation, Lifecycle},
    battle_payload::data as d,
    battle_read as read, battle_world as world, protocol, world_protocol as legacy,
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Snapshot {
    context: app::SelectionContext,
    channel: u8,
    attachment: Option<world::Attachment>,
    observation: Option<Observation>,
    full_vip: bool,
}
impl std::fmt::Debug for Snapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ConfirmedBattleState[redacted]")
    }
}
impl Snapshot {
    #[cfg(test)]
    pub(crate) fn vip_inspection_fixture(
        context: app::SelectionContext,
        channel: u8,
        observation: Observation,
    ) -> Arc<Self> {
        let data = observation.data();
        assert!(data.vip.is_some());
        assert_eq!(
            context.key.source_id,
            u32::try_from(data.personal.player.source).unwrap()
        );
        assert_eq!(
            i64::try_from(context.key.object_id).unwrap(),
            data.personal.player.object
        );
        Arc::new(Self {
            context,
            channel,
            attachment: Some(world::Attachment {
                world: data.world.identity.clone(),
                lifecycle: operation::Lifecycle {
                    revision: data.world.revision,
                    phase: legacy::Phase::Loading,
                    abort: legacy::AbortReason::None,
                    settlement: None,
                    race_settlement: None,
                    race_terminal_revision: 0,
                },
            }),
            observation: Some(observation),
            full_vip: true,
        })
    }
    pub(crate) fn scope(&self) -> (app::SelectionContext, u8) {
        (self.context, self.channel)
    }
    pub(crate) fn attachment(&self) -> Option<&world::Attachment> {
        self.attachment.as_ref()
    }
    pub(crate) fn observation(&self) -> Option<&Observation> {
        self.observation.as_ref()
    }
    pub(crate) fn full_vip(&self) -> bool {
        self.full_vip
    }
    /// An aborted service can retain an ACTIVE engine capture. Keep the real
    /// capture for diagnosis, but never offer it as an active display update.
    pub(crate) fn presentation_observation(&self) -> Option<&Observation> {
        // Full-VIP wire admission is not shared-resource presentation proof.
        if self.full_vip {
            return None;
        }
        self.attachment.as_ref().filter(|a| {
            !matches!(
                a.lifecycle.phase,
                legacy::Phase::Preparing | legacy::Phase::Aborted
            )
        })?;
        self.observation()
    }
}

#[derive(Clone, Default)]
pub(crate) struct Tracker {
    snapshot: Option<Arc<Snapshot>>,
    follower: Option<Follower>,
    // ModernRooms allocates match IDs monotonically across all channels for
    // this server lifetime, and refuses exhaustion. Keep this fence across
    // navigation, not across reconnects (which have a new server lifetime).
    high_water_match: i64,
}
impl Tracker {
    pub(crate) fn snapshot(&self) -> Option<&Arc<Snapshot>> {
        self.snapshot.as_ref()
    }
    pub(crate) fn retire(&mut self) {
        self.snapshot = None;
        self.follower = None;
    }
    fn same_scope(&self, context: app::SelectionContext, channel: u8) -> bool {
        self.snapshot
            .as_ref()
            .is_none_or(|s| s.context == context && s.channel == channel)
    }
    /// Local stale UI requests must not reach the server's fail-closed identity
    /// boundary and unnecessarily terminate a healthy authenticated stream.
    pub(crate) fn admits_world(&self, q: &world::Request) -> bool {
        self.admits_world_family(q, false)
    }
    pub(crate) fn admits_vip_world(&self, q: &world::Request) -> bool {
        self.admits_world_family(q, true)
    }
    fn admits_world_family(&self, q: &world::Request, full_vip: bool) -> bool {
        if !self.family_matches(full_vip) {
            return false;
        }
        if !self.same_scope(q.context, q.channel) {
            return false;
        }
        // Legacy PREPARE can automatically forget a terminal, non-result
        // entry. The canonical client requires an explicit confirmed LEAVE;
        // never let a new preparation silently replace its retained capture.
        if q.action == world::Action::Prepare
            && self
                .snapshot
                .as_ref()
                .is_some_and(|s| s.attachment.is_some())
        {
            return false;
        }
        if q.match_id == 0 {
            return true;
        }
        match self.snapshot.as_ref().and_then(|s| s.attachment.as_ref()) {
            Some(a) => {
                [q.room, q.match_id, q.generation]
                    == [a.world.room, a.world.match_id, a.world.generation]
            }
            None => q.action == world::Action::Read && q.match_id > self.high_water_match,
        }
    }
    pub(crate) fn admits_operation(&self, guard: Guard) -> bool {
        let s = guard.stamp;
        self.family_matches(false)
            && self.same_scope(s.context, s.channel)
            && self.snapshot.as_ref().is_some_and(|old| {
                old.attachment.as_ref().is_some_and(|a| {
                    s.world == [a.world.room, a.world.match_id, a.world.generation]
                }) && old.observation.as_ref().is_some_and(|o| {
                    o.legacy_commands_supported() && o.data().personal.entity == s.entity
                })
            })
        // A stale life is deliberately NOT rejected here: the server returns
        // an authoritative rejection. No local ACK or automatic retry.
    }
    pub(crate) fn accept_world(
        &mut self,
        reply: &world::Reply,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        reply.validate()?;
        ensure(self.family_matches(false) && world::reply_negotiated_selected(reply, peer))?;
        let q = &reply.echo;
        let leaving = q.action == world::Action::Leave && reply.status == legacy::Status::Ok;
        if leaving {
            let old = self
                .snapshot
                .as_ref()
                .and_then(|s| s.attachment.as_ref())
                .ok_or_else(invalid)?;
            ensure(
                [q.room, q.match_id, q.generation]
                    == [old.world.room, old.world.match_id, old.world.generation],
            )?;
        }
        self.accept(
            q.context,
            q.channel,
            reply.attachment.as_ref(),
            reply.data.as_ref(),
            leaving,
            peer,
        )
    }
    fn family_matches(&self, full_vip: bool) -> bool {
        self.snapshot
            .as_ref()
            .is_none_or(|s| s.attachment.is_none() || s.full_vip == full_vip)
    }
    pub(crate) fn admits_vip_equipment(&self, q: &super::battle_vip_equipment::Request) -> bool {
        let s = q.stamp;
        self.same_scope(s.context, s.channel)
            && self.snapshot.as_ref().is_some_and(|old| {
                old.full_vip
                    && old.attachment.as_ref().is_some_and(|a| {
                        s.world == [a.world.room, a.world.match_id, a.world.generation]
                    })
                    && old.observation.as_ref().is_some_and(|o| {
                        o.data().vip.is_some()
                            && o.data().personal.entity == s.entity
                            && s.revision <= o.data().world.revision
                    })
            })
    }
    pub(crate) fn accept_vip_world(
        &mut self,
        reply: &super::battle_vip_world::Reply,
        peer: protocol::extensions::Selected,
    ) -> Result<(), app::ApplicationError> {
        reply.validate(peer)?;
        let q = reply.echo();
        ensure(self.admits_vip_world(q))?;
        let leaving = q.action == world::Action::Leave && reply.status() == legacy::Status::Ok;
        self.accept_family(
            q.context,
            q.channel,
            reply.attachment(),
            reply.data(),
            leaving,
            peer,
            true,
        )
    }
    pub(crate) fn accept_vip_equipment(
        &mut self,
        reply: &super::battle_vip_equipment::Reply,
        expected: &super::battle_vip_equipment::Request,
        peer: protocol::extensions::Selected,
    ) -> Result<Option<super::battle_vip_equipment::Quote>, app::ApplicationError> {
        ensure(
            peer.allows_vip_world()
                && peer.allows_vip_equipment()
                && read::data_negotiated_selected(&reply.data, peer)
                && self.admits_vip_equipment(expected),
        )?;
        // Stage the intent/receipt and the full follower first, then also check
        // service chronology before the only publication. Failure keeps every
        // retained Arc and all three version domains unchanged.
        let mut staged = self.follower.clone().ok_or_else(invalid)?;
        let quote = reply.accept(expected, &mut staged)?;
        let s = expected.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept_family(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
            true,
        )?;
        Ok(quote)
    }
    pub(crate) fn accept_operation(
        &mut self,
        reply: &operation::Reply,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        let ack = peer.base();
        ensure(
            world::negotiated(&ack)
                && operation::reply_negotiated_selected(reply, peer)
                && self.admits_operation(reply.guard),
        )?;
        reply.lifecycle.validate(&reply.data)?;
        let s = reply.guard.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_revival(
        &mut self,
        reply: &super::battle_revival::Reply,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        reply.validate()?;
        let s = reply.echo.stamp;
        ensure(
            super::battle_revival::reply_negotiated_selected(reply, peer)
                && self.admits_operation(Guard {
                    stamp: s,
                    target: None,
                }),
        )?;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_equipment(
        &mut self,
        reply: &super::battle_equipment::Reply,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        reply.validate()?;
        let s = reply.echo.stamp;
        ensure(
            super::battle_equipment::reply_negotiated_selected(reply, peer)
                && self.admits_operation(Guard {
                    stamp: s,
                    target: None,
                }),
        )?;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_greater_haste(
        &mut self,
        reply: &super::battle_greater_haste::CandidateReply,
        expected: &super::battle_greater_haste::Intent,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        let ack = peer.base();
        ensure(
            super::battle_greater_haste::negotiated(&ack)
                && read::command_data_negotiated_selected(&reply.data, peer),
        )?;
        let before = self
            .snapshot
            .as_ref()
            .and_then(|s| s.observation.as_ref())
            .ok_or_else(invalid)?;
        reply.validate_after(expected, before.data())?;
        let s = expected.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_target_haste(
        &mut self,
        reply: &super::battle_target_haste::CandidateReply,
        expected: &super::battle_target_haste::Intent,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        let ack = peer.base();
        ensure(
            super::battle_target_haste::negotiated(&ack)
                && read::command_data_negotiated_selected(&reply.data, peer),
        )?;
        let before = self
            .snapshot
            .as_ref()
            .and_then(|s| s.observation.as_ref())
            .ok_or_else(invalid)?;
        reply.validate_after(expected, before.data())?;
        let s = expected.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_slow(
        &mut self,
        reply: &super::battle_slow::CandidateReply,
        expected: &super::battle_slow::Intent,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        let peer: protocol::extensions::Selected = peer.into();
        let ack = peer.base();
        ensure(
            super::battle_slow::negotiated(&ack)
                && read::command_data_negotiated_selected(&reply.data, peer),
        )?;
        let before = self
            .snapshot
            .as_ref()
            .and_then(|s| s.observation.as_ref())
            .ok_or_else(invalid)?;
        reply.validate_after(expected, before.data())?;
        let s = expected.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    pub(crate) fn accept_player_healing(
        &mut self,
        reply: &super::battle_player_healing::Reply,
        expected: &super::battle_player_healing::Intent,
        peer: protocol::extensions::Selected,
    ) -> Result<(), app::ApplicationError> {
        ensure(
            super::battle_player_healing::negotiated(peer)
                && self.admits_operation(Guard {
                    stamp: expected.stamp,
                    target: None,
                }),
        )?;
        let before = self
            .snapshot
            .as_ref()
            .and_then(|s| s.observation.as_ref())
            .ok_or_else(invalid)?;
        reply.check_after(expected, before.data())?;
        let s = expected.stamp;
        let attachment = world::Attachment {
            world: d::WorldId {
                room: s.world[0],
                match_id: s.world[1],
                generation: s.world[2],
            },
            lifecycle: reply.lifecycle,
        };
        self.accept(
            s.context,
            s.channel,
            Some(&attachment),
            Some(&reply.data),
            false,
            peer,
        )
    }
    fn accept(
        &mut self,
        context: app::SelectionContext,
        channel: u8,
        attachment: Option<&world::Attachment>,
        data: Option<&d::UnvalidatedData>,
        leaving: bool,
        peer: impl Into<protocol::extensions::Selected>,
    ) -> Result<(), app::ApplicationError> {
        self.accept_family(
            context,
            channel,
            attachment,
            data,
            leaving,
            peer.into(),
            false,
        )
    }
    fn accept_family(
        &mut self,
        context: app::SelectionContext,
        channel: u8,
        attachment: Option<&world::Attachment>,
        data: Option<&d::UnvalidatedData>,
        leaving: bool,
        peer: protocol::extensions::Selected,
        full_vip: bool,
    ) -> Result<(), app::ApplicationError> {
        ensure(self.family_matches(full_vip) && data.is_none_or(|d| d.vip.is_some() == full_vip))?;
        let peer: protocol::extensions::Selected = peer.into();
        let ack = peer.base();
        ensure(
            ack.version <= protocol::MAX_VERSION
                && ack.numeric_schema == protocol::NUMERIC_SCHEMA_I64_V1
                && self.same_scope(context, channel),
        )?;
        let old = self.snapshot.as_ref().and_then(|s| s.attachment.as_ref());
        match (old, attachment) {
            (Some(a), Some(b)) => {
                ensure(a.world == b.world)?;
                lifecycle_follows(a.lifecycle, b.lifecycle)?;
                let previous = self.snapshot.as_ref().unwrap().observation.as_ref();
                ensure(previous.is_none() || data.is_some())?;
                if terminal(a.lifecycle.phase) {
                    ensure(previous.map(Observation::data) == data)?;
                }
                if a.lifecycle.revision == b.lifecycle.revision {
                    ensure(
                        previous.map(|o| o.data().world.revision) == data.map(|d| d.world.revision),
                    )?;
                }
            }
            (None, Some(a)) => ensure(a.world.match_id > self.high_water_match)?,
            (Some(_), None) => ensure(leaving && data.is_none())?,
            (None, None) => ensure(!leaving && data.is_none())?,
        }
        let mut next = self.clone();
        let observation = if let Some(a) = attachment {
            let request = read::Request {
                context,
                channel_id: channel,
                world: a.world.clone(),
            };
            if next.follower.is_none() {
                next.follower = Some(Follower::new_selected(request.clone(), peer)?);
            }
            let f = next.follower.as_mut().unwrap();
            f.accept(read::UnvalidatedReply {
                echo: request,
                state: if data.is_some() {
                    read::State::Data
                } else if a.lifecycle.phase == legacy::Phase::Preparing {
                    read::State::Preparing
                } else {
                    read::State::Unavailable
                },
                data: data.cloned(),
            })?;
            next.high_water_match = a.world.match_id;
            f.observation().cloned()
        } else {
            next.follower = None;
            None
        };
        next.snapshot = Some(Arc::new(Snapshot {
            context,
            channel,
            attachment: attachment.cloned(),
            observation,
            full_vip,
        }));
        // The only publication. Every failing branch leaves snapshot, follower
        // and the retirement fence untouched, including retained Arc readers.
        *self = next;
        Ok(())
    }
}
fn terminal(p: legacy::Phase) -> bool {
    matches!(p, legacy::Phase::Aborted | legacy::Phase::Finished)
}
fn settlement_follows(
    a: Option<legacy::SettlementState>,
    b: Option<legacy::SettlementState>,
) -> bool {
    use legacy::SettlementState as S;
    a == b
        || match a {
            None => true,
            Some(S::Pending) => matches!(b, Some(S::Saved | S::Failed)),
            Some(S::Saved | S::Failed) => false,
        }
}
fn lifecycle_follows(a: Lifecycle, b: Lifecycle) -> Result<(), app::ApplicationError> {
    use legacy::Phase as P;
    ensure(b.revision >= a.revision && (b.revision != a.revision || a == b))?;
    ensure(
        a.phase == b.phase
            || !terminal(a.phase)
                && (b.phase == P::Aborted
                    || match a.phase {
                        P::Preparing => {
                            matches!(b.phase, P::Loading | P::Countdown | P::Active | P::Finished)
                        }
                        P::Loading => matches!(b.phase, P::Countdown | P::Active | P::Finished),
                        P::Countdown => matches!(b.phase, P::Active | P::Finished),
                        P::Active => b.phase == P::Finished,
                        _ => false,
                    }),
    )?;
    ensure(
        (!terminal(a.phase) || a.abort == b.abort)
            && settlement_follows(a.settlement, b.settlement)
            && settlement_follows(a.race_settlement, b.race_settlement)
            && (a.race_terminal_revision == 0
                || a.race_terminal_revision == b.race_terminal_revision),
    )
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid confirmed battle chronology",
    )
}
fn ensure(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
#[cfg(test)]
#[path = "battle_state_tests.rs"]
mod tests;
