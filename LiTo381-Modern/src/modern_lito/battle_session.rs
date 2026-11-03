//! Read-only canonical observations owned by the normal login TLS worker.
//! These never confirm a UI command, LOAD, equipment SET, or saved settlement.
use super::{
    application_protocol as app,
    battle_follow::{Follower, Observation},
    battle_payload::data::WorldId,
    battle_read as read,
    login_window::{LoginPhase, LoginSnapshot},
    protocol, world_protocol as world,
};
use std::sync::Arc;

#[derive(Clone, Debug)]
struct Snapshot {
    request: read::Request,
    status: read::State,
    observation: Option<Observation>,
}

/// Cheap immutable publication, separate from the legacy command ACK caches.
/// Detached means no currently admitted attachment, not an empty inventory.
#[derive(Clone, Default, Debug)]
pub struct State(Option<Arc<Snapshot>>);
impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (None, None) => true,
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for State {}
impl State {
    pub fn request(&self) -> Option<&read::Request> {
        self.0.as_ref().map(|s| &s.request)
    }
    pub fn status(&self) -> Option<read::State> {
        self.0.as_ref().map(|s| s.status)
    }
    pub fn observation(&self) -> Option<&Observation> {
        self.0.as_ref()?.observation.as_ref()
    }
    pub(crate) fn clear(&mut self) {
        self.0 = None;
    }
    pub(crate) fn retain(&mut self, request: Option<&read::Request>) {
        if self.request() != request {
            self.clear();
        }
    }
}

/// Only identity comes from the acknowledged legacy ticket. No legacy Frame
/// is fabricated from canonical data, and its revision is never rewritten.
pub(crate) fn request_for(view: &LoginSnapshot) -> Option<read::Request> {
    if view.phase != LoginPhase::InLobby {
        return None;
    }
    let nav = view.navigation.as_ref()?;
    if nav.snapshot.phase != app::NavigationPhase::Lobby {
        return None;
    }
    let world::State::Ready(reply) = &view.world else {
        return None;
    };
    let frame = reply.frame.as_ref()?;
    if frame.phase == world::Phase::Preparing
        || nav.context != reply.echo.context
        || nav.snapshot.channel_id != Some(reply.echo.channel_id)
        || view.selected_source != Some(nav.context.key.source_id)
    {
        return None;
    }
    let request = read::Request {
        context: nav.context,
        channel_id: reply.echo.channel_id,
        world: WorldId {
            room: frame.room.summary.room_id.try_into().ok()?,
            match_id: frame.match_id.try_into().ok()?,
            generation: frame.generation.try_into().ok()?,
        },
    };
    request.validate().ok()?;
    Some(request)
}

pub(crate) struct Tracker {
    peer: protocol::extensions::Selected,
    follower: Option<Follower>,
    state: State,
}
impl Tracker {
    pub(crate) fn new(peer: impl Into<protocol::extensions::Selected>) -> Self {
        Self {
            peer: peer.into(),
            follower: None,
            state: State::default(),
        }
    }
    /// An unavailable publication is terminal for this attachment; do not spin
    /// on an absent server engine. A new selection/match gets a fresh follower.
    pub(crate) fn read_request(&self) -> Option<read::Request> {
        (self.state.status() != Some(read::State::Unavailable))
            .then(|| self.state.request().cloned())
            .flatten()
    }
    pub(crate) fn state(&self) -> State {
        self.state.clone()
    }
    pub(crate) fn align(
        &mut self,
        request: Option<read::Request>,
    ) -> Result<bool, app::ApplicationError> {
        let request = request.filter(|_| read::negotiated(&self.peer.base()));
        if request.as_ref() == self.state.request() {
            return Ok(false);
        }
        let next = request
            .as_ref()
            .map(|request| Follower::new_selected(request.clone(), self.peer))
            .transpose()?;
        self.follower = next;
        self.state = State(request.map(|request| {
            Arc::new(Snapshot {
                request,
                status: read::State::Preparing,
                observation: None,
            })
        }));
        Ok(true)
    }
    pub(crate) fn accept(
        &mut self,
        reply: read::UnvalidatedReply,
    ) -> Result<(), app::ApplicationError> {
        let follower = self.follower.as_mut().ok_or_else(|| {
            app::error(
                app::ErrorKind::IdentityMismatch,
                "detached battle observation",
            )
        })?;
        let request = reply.echo.clone();
        follower.accept(reply)?;
        self.state = State(Some(Arc::new(Snapshot {
            request,
            status: follower.state(),
            observation: follower.observation().cloned(),
        })));
        Ok(())
    }
}

#[cfg(test)]
#[path = "battle_session_tests.rs"]
mod tests;
