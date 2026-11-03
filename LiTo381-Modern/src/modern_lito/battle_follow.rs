//! One authenticated attachment's canonical observation stream. This is a
//! consistency follower, not a simulation, command ACK or persistence receipt.
//! Immutable snapshots still require shared-resource preflight before display.
use super::{
    application_protocol as app,
    battle_payload::{data::*, personal::PersonalView},
    battle_read as read, protocol,
};
use std::sync::Arc;
#[path = "battle_follow_actor.rs"]
mod actor;
#[path = "battle_follow_modes.rs"]
mod modes;
#[cfg(test)]
#[path = "battle_follow_tests.rs"]
mod tests;

#[derive(Clone)]
pub struct Observation {
    data: Arc<UnvalidatedData>,
    personal: PersonalView,
}
impl std::fmt::Debug for Observation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FollowedBattleObservation[redacted]")
    }
}
impl Observation {
    pub fn data(&self) -> &UnvalidatedData {
        &self.data
    }
    pub fn personal(&self) -> &PersonalView {
        &self.personal
    }
    pub(crate) fn legacy_commands_supported(&self) -> bool {
        self.data.vip.is_none()
            && self.data.world.descriptor.features & super::battle_payload::vip::FEATURE == 0
    }
    /// Consumers may coalesce already accepted observations, but a different
    /// follower/attachment must not be mistaken for this consumer's successor.
    pub(crate) fn check_successor(&self, next: &Self) -> Result<(), app::ApplicationError> {
        follows(self.data(), next.data())
    }
}
#[derive(Clone)]
pub struct Follower {
    request: read::Request,
    peer: protocol::extensions::Selected,
    state: read::State,
    observation: Option<Observation>,
}
impl std::fmt::Debug for Follower {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BattleFollower[redacted]")
    }
}
impl Follower {
    pub fn new(request: read::Request, ack: protocol::Ack) -> Result<Self, app::ApplicationError> {
        Self::new_selected(request, protocol::extensions::Selected::legacy(ack))
    }
    pub fn new_selected(
        request: read::Request,
        peer: protocol::extensions::Selected,
    ) -> Result<Self, app::ApplicationError> {
        let ack = peer.base();
        request.validate()?;
        ensure(
            read::negotiated(&ack)
                && ack.version <= protocol::MAX_VERSION
                && ack.numeric_schema == protocol::NUMERIC_SCHEMA_I64_V1,
        )?;
        Ok(Self {
            request,
            peer,
            state: read::State::Preparing,
            observation: None,
        })
    }
    pub fn state(&self) -> read::State {
        self.state
    }
    pub fn observation(&self) -> Option<&Observation> {
        self.observation.as_ref()
    }
    /// Validate everything before the only assignment. A rejected response
    /// cannot partially change world, inventory, status or retained snapshots.
    pub fn accept(&mut self, reply: read::UnvalidatedReply) -> Result<(), app::ApplicationError> {
        ensure(reply.echo == self.request && read::reply_negotiated_selected(&reply, self.peer))?;
        ensure(self.state != read::State::Unavailable || reply.state == read::State::Unavailable)?;
        let next = match (reply.state, reply.data) {
            (read::State::Preparing, None) => {
                ensure(self.observation.is_none())?;
                None
            }
            (read::State::Unavailable, None) => {
                ensure(self.observation.is_none())?;
                None
            }
            (read::State::Data, Some(d)) => {
                d.check_scene_values(&self.request.player(), &self.request.world)?;
                clocks(&d)?;
                if let Some(old) = &self.observation {
                    follows(old.data(), &d)?;
                }
                let personal = d.personal_view(&self.request.player(), &self.request.world)?;
                Some(Observation {
                    data: Arc::new(d),
                    personal,
                })
            }
            _ => return Err(invalid()),
        };
        self.observation = next;
        self.state = reply.state;
        Ok(())
    }
}
fn invalid() -> app::ApplicationError {
    app::error(
        app::ErrorKind::MalformedFrame,
        "invalid canonical battle sequence",
    )
}
fn ensure(ok: bool) -> Result<(), app::ApplicationError> {
    if ok {
        Ok(())
    } else {
        Err(invalid())
    }
}
fn terminal(p: Phase) -> bool {
    matches!(p, Phase::Finished | Phase::Aborted)
}
fn phase(a: Phase, b: Phase) -> bool {
    a == b
        || !terminal(a)
            && (b == Phase::Aborted
                || match a {
                    Phase::Loading => {
                        matches!(b, Phase::Countdown | Phase::Active | Phase::Finished)
                    }
                    Phase::Countdown => matches!(b, Phase::Active | Phase::Finished),
                    Phase::Active => b == Phase::Finished,
                    _ => false,
                })
}
/// Event domains must be supplied explicitly. NPC resident revisions are NOT
/// engine revisions; Quest progression is yet another independent counter.
fn event<T: PartialEq>(a: &T, b: &T, ar: i64, br: i64, observed: i64) -> bool {
    br >= ar && if br == ar { a == b } else { br > observed }
}
fn advances(a: i64, b: i64, budget: i64) -> bool {
    b >= a && i128::from(b) - i128::from(a) <= i128::from(budget)
}
fn optional<T>(a: &Option<T>, b: &Option<T>, f: impl FnOnce(&T, &T) -> bool) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => f(a, b),
        _ => false,
    }
}
fn clock(a: &RegenClock, b: &RegenClock, elapsed: i64, same_world: bool) -> bool {
    if b.next_second_elapsed_nanos < a.next_second_elapsed_nanos {
        return false;
    }
    if b.next_second_elapsed_nanos == a.next_second_elapsed_nanos {
        return !same_world || a == b;
    }
    elapsed >= a.next_second_elapsed_nanos
        && b.next_second_elapsed_nanos / 1_000_000_000 <= elapsed / 1_000_000_000 + 1
}
pub(crate) fn clocks(d: &UnvalidatedData) -> Result<(), app::ApplicationError> {
    let w = &d.world;
    ensure(match w.phase {
        Phase::Loading | Phase::Countdown => w.active_elapsed_nanos == -1,
        Phase::Active | Phase::Finished => w.active_elapsed_nanos >= 0,
        Phase::Aborted => true,
    })
}
/// Checks raw candidate history without constructing an admitted Observation.
/// Session/capability and pending-command ownership remain separate gates.
pub(crate) fn check_candidate_sequence(
    a: &UnvalidatedData,
    b: &UnvalidatedData,
) -> Result<(), app::ApplicationError> {
    a.check_scene_values(&a.personal.player, &a.world.identity)?;
    b.check_scene_values(&a.personal.player, &a.world.identity)?;
    clocks(a)?;
    clocks(b)?;
    follows(a, b)
}
pub(crate) fn check_haste_candidate_sequence(
    a: &super::battle_payload::haste::Candidate,
    b: &super::battle_payload::haste::Candidate,
) -> Result<(), app::ApplicationError> {
    a.check_values(&a.data.personal.player, &a.data.world.identity)?;
    b.check_values(&a.data.personal.player, &a.data.world.identity)?;
    follows(&a.data, &b.data)
}
pub(crate) fn check_speed_candidate_sequence(
    a: &super::battle_payload::speed::Candidate,
    b: &super::battle_payload::speed::Candidate,
) -> Result<(), app::ApplicationError> {
    a.check_values(&a.data.personal.player, &a.data.world.identity)?;
    b.check_values(&a.data.personal.player, &a.data.world.identity)?;
    follows(&a.data, &b.data)
}
/// Internal reuse after the complete schema-specific value proof. This does
/// not construct an Observation or waive negotiation for a candidate schema.
pub(crate) fn check_healing_view_sequence(
    a: &super::battle_payload::healing::View<'_>,
    b: &super::battle_payload::healing::View<'_>,
) -> Result<(), app::ApplicationError> {
    a.check_values(&a.data.personal.player, &a.data.world.identity)?;
    b.check_values(&a.data.personal.player, &a.data.world.identity)?;
    follows_base(a.data, b.data)
}
fn follows(a: &UnvalidatedData, b: &UnvalidatedData) -> Result<(), app::ApplicationError> {
    if a.vip.is_some() || b.vip.is_some() {
        return super::battle_payload::vip::follows_attached(a, b);
    }
    if a.armor.is_some() || b.armor.is_some() {
        return super::battle_payload::armor::follows_attached(a, b);
    }
    match (&a.healing, &b.healing) {
        (Some(_), Some(_)) => super::battle_payload::healing::follows_attached(a, b),
        (None, None) => follows_base(a, b),
        _ => Err(invalid()),
    }
}
/// Candidate-only proof: keep armor, status and healing intact, and never
/// admit the new descriptor via an older negotiated Observation.
pub(crate) fn check_armor_view_sequence(
    a: &super::battle_payload::armor::View<'_>,
    b: &super::battle_payload::armor::View<'_>,
) -> Result<(), app::ApplicationError> {
    a.check_values(&a.data.personal.player, &a.data.world.identity)?;
    b.check_values(&a.data.personal.player, &a.data.world.identity)?;
    follows_base(&a.data, &b.data)
}
pub(crate) fn check_vip_view_sequence(
    a: &super::battle_payload::vip::View<'_>,
    b: &super::battle_payload::vip::View<'_>,
) -> Result<(), app::ApplicationError> {
    a.check_values(&a.data.personal.player, &a.data.world.identity)?;
    b.check_values(&a.data.personal.player, &a.data.world.identity)?;
    follows_base(&a.data, &b.data)
}
fn follows_base(a: &UnvalidatedData, b: &UnvalidatedData) -> Result<(), app::ApplicationError> {
    let x = &a.world;
    let y = &b.world;
    ensure(
        x.identity == y.identity
            && a.room == b.room
            && a.manifest == b.manifest
            && x.descriptor == y.descriptor
            && x.lane == y.lane
            && a.personal.player == b.personal.player
            && a.personal.entity == b.personal.entity
            && a.actors.0.len() == b.actors.0.len()
            && y.revision >= x.revision
            && phase(x.phase, y.phase)
            && y.loading_elapsed_nanos >= x.loading_elapsed_nanos
            && y.active_elapsed_nanos >= x.active_elapsed_nanos,
    )?;
    if x.active_elapsed_nanos >= 0 {
        ensure(
            x.loading_elapsed_nanos - x.active_elapsed_nanos
                == y.loading_elapsed_nanos - y.active_elapsed_nanos,
        )?;
    }
    if terminal(x.phase) {
        return ensure(a == b);
    }
    if x.revision == y.revision {
        let mut allowed = a.clone();
        allowed.world.loading_elapsed_nanos = y.loading_elapsed_nanos;
        allowed.world.active_elapsed_nanos = y.active_elapsed_nanos;
        if let (Some(r), Some(n)) = (&mut allowed.race, &b.race) {
            ensure(n.observed_elapsed_nanos >= r.observed_elapsed_nanos)?;
            r.observed_elapsed_nanos = n.observed_elapsed_nanos;
        }
        if let (Some(t), Some(n)) = (&mut allowed.time_attack, &b.time_attack) {
            ensure(n.observed_millis >= t.observed_millis)?;
            t.observed_millis = n.observed_millis;
        }
        if let (Some(c), Some(n)) = (&mut allowed.personal.regen_clock, &b.personal.regen_clock) {
            ensure(clock(c, n, y.active_elapsed_nanos, true))?;
            *c = n.clone();
        }
        return ensure(allowed == *b);
    }
    actor::follows(a, b)?;
    super::battle_payload::speed::follows_attached(a, b)?;
    super::battle_payload::haste::follows_attached(a, b)?;
    super::battle_payload::npc_motion::follows_attached(a, b)?;
    super::battle_payload::presentation::follows(a, b)?;
    modes::follows(a, b)?;
    Ok(())
}
