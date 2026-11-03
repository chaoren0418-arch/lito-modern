//! Normal-player binding of the canonical worker. Pure ownership/qualification
//! coordination; neither receiving Data nor obtaining pixels alone sends LOAD.
use super::{
    battle_payload::data::Phase,
    battle_runtime::{BattleRuntime, Status},
    battle_scene::Surface,
    battle_window::Stamp,
    login_window::LoginSnapshot,
};
use anyhow::{ensure, Result};

/// Choose the negotiated owner once. A hidden/detached v39 state must never
/// fall back to a retained v37 ticket or manufacture a legacy Frame.
fn confirmed_observation(view: &LoginSnapshot) -> Option<&super::battle_follow::Observation> {
    if !view.battle_supported {
        return None;
    }
    if view.canonical.supported() {
        let scope = super::battle_control::scope_for(view)?;
        (view.canonical.snapshot()?.scope() == scope)
            .then(|| view.canonical.observation())
            .flatten()
    } else {
        let request = super::battle_session::request_for(view)?;
        (view.battle.request() == Some(&request))
            .then(|| view.battle.observation())
            .flatten()
    }
}

/// A separate, presentation-only full-VIP attachment. Never reuse this from
/// command, route or LOAD builders, which intentionally keep using the
/// ordinary confirmed_observation path until the full scene is qualified.
fn vip_preview_observation(view: &LoginSnapshot) -> Option<&super::battle_follow::Observation> {
    if !view.battle_supported || !view.canonical.supported() {
        return None;
    }
    let scope = super::battle_control::scope_for(view)?;
    let snapshot = view.canonical.snapshot()?;
    if !snapshot.full_vip() || snapshot.scope() != scope {
        return None;
    }
    let observation = view.canonical.vip_observation()?;
    let stamp = Stamp::of(observation);
    (observation.data().vip.is_some()
        && stamp.context == scope.0
        && stamp.channel == scope.1
        && observation.data().world.phase != Phase::Aborted
        && snapshot.attachment().is_some_and(|a| a.world == observation.data().world.identity))
        .then_some(observation)
}

pub fn update(runtime: &mut BattleRuntime, view: &LoginSnapshot, closing: bool) -> Result<Status> {
    if closing {
        runtime.shutdown();
        return Ok(runtime.poll());
    }
    if !view.battle_supported {
        runtime.clear();
        return Ok(runtime.poll());
    }
    let observation = vip_preview_observation(view).or_else(|| confirmed_observation(view));
    let Some(next) = observation else {
        runtime.clear();
        return Ok(runtime.poll());
    };
    if runtime.observation().is_some_and(|old| {
        let mut a = Stamp::of(old);
        let mut b = Stamp::of(next);
        a.revision = 0;
        b.revision = 0;
        // Respawn changes click authority, not the TLS/resource attachment.
        a.life = 0;
        b.life = 0;
        a != b
    }) {
        runtime.clear();
    }
    if runtime.observation().is_some() {
        // Loading's observation clock advances without changing any resource,
        // pose or occupancy. Replacing that input continuously can starve the
        // LOAD barrier on a busy machine. Do not freeze the server's timeout:
        // only identical loading pictures may retain their qualified worker.
        if !runtime
            .observation()
            .is_some_and(|old| same_loading_picture(old, next))
        {
            runtime.update(next.clone())?;
        }
    } else if next.data().world.phase != Phase::Aborted {
        runtime.attach(next.clone())?;
    }
    Ok(runtime.poll())
}
pub fn admitted(surface: &Surface, view: &LoginSnapshot) -> bool {
    let current = if surface.observation.data().vip.is_some() {
        vip_preview_observation(view)
    } else {
        confirmed_observation(view)
    };
    let Some(current) = current else { return false };
    Stamp::of(&surface.observation).current(current)
        && surface.observation.data().manifest == current.data().manifest
        && surface.observation.data().room == current.data().room
        && surface.observation.data().world.phase != Phase::Aborted
}
pub fn admitted_observation(o: &super::battle_follow::Observation, view: &LoginSnapshot) -> bool {
    confirmed_observation(view).is_some_and(|current| {
        Stamp::of(o).current(current)
            && o.data().manifest == current.data().manifest
            && o.data().room == current.data().room
            && current.data().world.phase != Phase::Aborted
    })
}
/// A current canonical picture/terrain proof replaces the *legacy hit image*
/// gate, not command authority, skill/cadence checks or LOAD authorization.
/// Dead actors also need this proof to use the explicit death confirmation UI.
pub(crate) fn controls_ready(surface: &Surface, view: &LoginSnapshot) -> bool {
    if surface.observation.data().vip.is_some() {
        return false; // full-VIP preview is not an interactive world
    }
    if !admitted(surface, view) {
        return false;
    }
    let Some(navigation) = surface.navigation.as_ref() else {
        return false;
    };
    if !confirmed_observation(view).is_some_and(|current| navigation.blockers_current(current)) {
        return false;
    }
    let o = &surface.observation;
    let Some(pose) = o
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
    else {
        return false;
    };
    if view.canonical.supported() {
        let Some(current) = confirmed_observation(view) else {
            return false;
        };
        return navigation.matches(o)
            && navigation.shared_map().map_id() as i32 == current.data().manifest.map
            && navigation.shared_map().canonical_digest() == current.data().manifest.digest
            && current.data().actors.0.iter().any(|a| {
                a.entity == pose.entity
                    && a.position == pose.position
                    && a.last_move_sequence == pose.last_move_sequence
                    && super::battle_window::life(a) == super::battle_window::life(pose)
            })
            && o.data().world.phase == current.data().world.phase
            && view.canonical.lifecycle().is_some_and(|l| {
                matches!(
                    (o.data().world.phase, l.phase),
                    (Phase::Countdown, super::world_protocol::Phase::Countdown)
                        | (Phase::Active, super::world_protocol::Phase::Active)
                        | (Phase::Finished, super::world_protocol::Phase::Finished)
                )
            });
    }
    let Some(frame) = view.world.frame() else {
        return false;
    };
    navigation.matches(o)
        && frame.actors.iter().any(|a| {
            a.entity_id == pose.entity as u64
                && a.position.x == pose.position.x
                && a.position.y == pose.position.y
                && a.sequence == pose.last_move_sequence as u64
        })
        && navigation.shared_map().map_id() == frame.manifest.map_id
        && navigation.shared_map().canonical_digest() == frame.manifest.digest
        && authority_life(view, o.data().personal.entity) == Some(Stamp::of(o).life)
        && matches!(
            (o.data().world.phase, frame.phase),
            (Phase::Countdown, super::world_protocol::Phase::Countdown)
                | (Phase::Active, super::world_protocol::Phase::Active)
                | (Phase::Finished, super::world_protocol::Phase::Finished)
        )
}

/// Same handoff in the real window and real-server tests. A canonical error or
/// unavailable picture cannot retire the mandatory full legacy preflight.
pub(crate) fn retire_legacy(
    runtime: &mut super::world_runtime::WorldRuntime,
    surface: &Surface,
    view: &LoginSnapshot,
) -> bool {
    controls_ready(surface, view)
        && view.world.frame().is_some_and(|frame| {
            runtime.finish_preflight(frame, surface.observation.data().personal.entity as u64)
        })
}
/// Full canonical resource qualification plus the current selected controller.
/// This does not send or optimistically confirm a server LOAD acknowledgement.
pub fn load_ready(runtime: &BattleRuntime, surface: &Surface, view: &LoginSnapshot) -> bool {
    if view.canonical.supported() {
        return admitted(surface, view)
            && view
                .canonical
                .lifecycle()
                .is_some_and(|l| l.phase == super::world_protocol::Phase::Loading)
            && confirmed_observation(view)
                .is_some_and(|o| runtime.loaded_request(surface, o).is_some());
    }
    admitted(surface, view)
        && view
            .battle
            .observation()
            .is_some_and(|o| runtime.loaded_request(surface, o).is_some())
        && view.world.frame().is_some_and(|w| {
            w.manifest.map_id as i32 == surface.observation.data().manifest.map
                && w.manifest.digest == surface.observation.data().manifest.digest
                && w.phase == super::world_protocol::Phase::Loading
                && surface.observation.data().world.phase == Phase::Loading
        })
}
/// v39 LOAD uses the same selected presentation owner and sealed resource
/// completion as the normal player. Pixels, a DTO, or polling alone cannot
/// authorize LOAD; this builder neither sends nor acknowledges it.
pub(crate) fn canonical_loaded_command(
    runtime: &BattleRuntime,
    surface: &Surface,
    view: &LoginSnapshot,
) -> Option<super::battle_control::Command> {
    if !view.canonical.supported() || view.world_pending || !load_ready(runtime, surface, view) {
        return None;
    }
    let request = runtime.loaded_request(surface, confirmed_observation(view)?)?;
    let command = super::battle_control::Command::world(request);
    command
        .current(&view.canonical, view.navigation.as_ref())
        .then_some(command)
}
/// Explicit death confirmation belongs to the painted, still-current life.
/// This builder neither predicts revival nor sends a retry on the user's behalf.
pub(crate) fn revival_command(
    surface: &Surface,
    view: &LoginSnapshot,
) -> Option<super::battle_control::Command> {
    if view.world_pending || !view.canonical.supported() || !controls_ready(surface, view) {
        return None;
    }
    let request = super::battle_revival::Request::from_observation(&surface.observation)?;
    let command = super::battle_control::Command::revival(request);
    command
        .current(&view.canonical, view.navigation.as_ref())
        .then_some(command)
}

/// The v37 LOAD adapter is deliberately unavailable to canonical controllers.
pub(crate) fn loaded_request(
    runtime: &BattleRuntime,
    surface: &Surface,
    view: &LoginSnapshot,
) -> Option<super::world_protocol::Request> {
    if view.canonical.supported() || !load_ready(runtime, surface, view) || view.world_pending {
        return None;
    }
    let frame = view.world.frame()?;
    let nav = view.navigation.as_ref()?;
    let actor = frame
        .actors
        .iter()
        .find(|a| a.matches_character(nav.context.key))?;
    if actor.loaded || actor.entity_id as i64 != surface.observation.data().personal.entity {
        return None;
    }
    Some(super::world_protocol::Request {
        action: super::world_protocol::Action::Loaded,
        match_id: frame.match_id,
        generation: frame.generation,
        manifest_digest: frame.manifest.digest,
        ..super::world_protocol::Request::read(nav.context, view.battle.request()?.channel_id)
    })
}
pub(crate) fn same_loading_picture(
    old: &super::battle_follow::Observation,
    next: &super::battle_follow::Observation,
) -> bool {
    if old.data().world.phase != Phase::Loading
        || next.data().world.phase != Phase::Loading
        || next.data().world.loading_elapsed_nanos < old.data().world.loading_elapsed_nanos
    {
        return false;
    }
    let mut candidate = next.data().clone();
    candidate.world.loading_elapsed_nanos = old.data().world.loading_elapsed_nanos;
    &candidate == old.data()
}
/// A click remains an intent, not an authorization to send. It must belong to
/// this selected actor's still-live attachment and target the same current life.
pub fn validate_intent(intent: super::battle_window::Intent, view: &LoginSnapshot) -> Result<()> {
    use super::battle_window::Kind;
    if view.canonical.supported() {
        return super::battle_input::validate(intent, view);
    }
    ensure!(
        !matches!(intent.kind, Kind::Haste(_)),
        "加速術需要新版指定目標協定"
    );
    let current = view
        .battle
        .observation()
        .ok_or_else(|| anyhow::anyhow!("battle observation missing"))?;
    ensure!(
        view.battle_supported
            && view.battle.request() == super::battle_session::request_for(view).as_ref()
            && intent.stamp.current(current)
            && current.data().world.phase == Phase::Active,
        "battle click belongs to another attachment or phase"
    );
    let me = current
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == intent.stamp.entity)
        .ok_or_else(|| anyhow::anyhow!("selected battle actor missing"))?;
    ensure!(
        me.vital.as_ref().is_none_or(|v| v.values.hp > 0)
            && super::battle_space::occupies(current.data(), me.entity),
        "dead or retired actor cannot act"
    );
    ensure!(
        authority_life(view, me.entity) == Some(intent.stamp.life),
        "painted caster life differs from command authority"
    );
    match intent.kind {
        Kind::Attack(t) | Kind::LightArrow(t) | Kind::Heal(t) | Kind::Disintegrate(t) => ensure!(
            super::battle_window::target_current(current, t),
            "battle target life changed"
        ),
        Kind::Portal => ensure!(
            super::battle_window::portal(current),
            "battle portal unavailable"
        ),
        _ => {}
    }
    if let Kind::Heal(target) = intent.kind {
        ensure!(
            super::battle_window::can_heal(current, target),
            "healing target is not friendly"
        );
    }
    Ok(())
}

/// Unwrap only after checking the painted owner/life. Returning an Action is
/// not authorization to send: the real window's existing typed gates run next.
pub(crate) fn admit_local_action(
    action: super::world_window::Action,
    view: &LoginSnapshot,
) -> Result<super::world_window::Action> {
    use super::{
        battle_window::{Kind, Local},
        world_window::Action,
    };
    if let Action::Canonical(intent) = action {
        if let Kind::Local(local) = intent.kind {
            ensure!(local.valid(), "invalid local gameplay intent");
            validate_intent(intent, view)?;
            return Ok(local.action());
        }
    }
    ensure!(
        !view.battle_supported || Local::from_action(action).is_none(),
        "canonical gameplay intent has no painted owner"
    );
    Ok(action)
}

pub(crate) fn bind_local_command(
    intent: Option<super::battle_window::Intent>,
    command: super::login_window::LoginCommand,
) -> super::login_window::LoginCommand {
    match intent {
        Some(intent) => super::login_window::LoginCommand::PaintedLocal {
            intent,
            command: Box::new(command),
        },
        None => command,
    }
}

/// Called AFTER SessionController::poll. Do not strip the owner at UI admission
/// and then let a later read select a different life for the typed command.
pub(crate) fn admit_local_command(
    command: super::login_window::LoginCommand,
    view: &LoginSnapshot,
) -> Result<super::login_window::LoginCommand> {
    use super::{
        battle_window::{Kind, Local},
        login_window::LoginCommand as C,
    };
    let C::PaintedLocal { intent, command } = command else {
        return Ok(command);
    };
    let Kind::Local(local) = intent.kind else {
        anyhow::bail!("not a local gameplay owner");
    };
    ensure!(local.valid(), "invalid local gameplay intent");
    validate_intent(intent, view)?;
    let matches = match (local, command.as_ref()) {
        (Local::Step(h), C::World(q)) => {
            q.action == super::world_protocol::Action::Move && q.heading == h
        }
        (Local::Teleport, C::CastTeleport) => true,
        (Local::FireStorm, C::CastFireStorm | C::CastNpcFireStorm(_)) => true,
        (Local::UseItem(a), C::UseItem { item_id: b })
        | (Local::SelectAmmo(a), C::SelectAmmo { item_id: b }) => a == *b,
        _ => false,
    };
    ensure!(matches, "painted local intent does not describe command");
    Ok(*command)
}

/// Life is semantic, unlike the two protocols' independent world revisions.
/// Check both streams so delayed canonical data cannot reuse a pre-respawn
/// click after the command cache has already accepted the new incarnation.
pub(crate) fn authority_life(view: &LoginSnapshot, entity: i64) -> Option<i64> {
    let entity = u64::try_from(entity).ok()?;
    let Some(combat) = view.combat.as_ref() else {
        return view.battle.observation().and_then(|o| {
            o.data()
                .actors
                .0
                .iter()
                .find(|a| a.entity as u64 == entity && a.vital.is_none())
                .map(|_| 1)
        });
    };
    if view.world.frame() != Some(&combat.world) {
        return None;
    }
    let actor = combat.world.actors.iter().find(|a| a.entity_id == entity)?;
    if !actor.matches_character(view.navigation.as_ref()?.context.key) {
        return None;
    }
    combat_life(combat, entity)
}
fn combat_life(combat: &super::combat_protocol::Frame, entity: u64) -> Option<i64> {
    if !combat.world.actors.iter().any(|a| a.entity_id == entity) {
        return None;
    }
    if let Some(l) = super::siege_protocol::life(&combat.world, entity) {
        return l.respawns.checked_add(1);
    }
    if let Some(l) = super::quest_life::life(&combat.world, entity) {
        return i64::try_from(l.incarnation()).ok();
    }
    if let Some(l) = combat.honor.as_ref().and_then(|h| h.life(entity)) {
        return l.respawn_count.checked_add(1);
    }
    Some(1)
}

pub fn command(
    intent: super::battle_window::Intent,
    view: &LoginSnapshot,
) -> Result<super::login_window::LoginCommand> {
    use super::{
        battle_window::{Kind, Target},
        login_window::LoginCommand as C,
    };
    if view.canonical.supported() {
        return Ok(C::Canonical(super::battle_input::target_command(
            intent, view,
        )?));
    }
    validate_intent(intent, view)?;
    let s = intent.stamp;
    let scope = super::npc_protocol::Scope {
        room_id: s.world[0] as u64,
        match_id: s.world[1] as u64,
        generation: s.world[2] as u64,
    };
    let siege_scope = super::siege_protocol::Scope {
        room_id: scope.room_id,
        match_id: scope.match_id,
        generation: scope.generation,
    };
    let npc = |serial| super::npc_protocol::Id {
        scope,
        serial: serial as u64,
    };
    let observation = view.battle.observation().unwrap();
    let actor = observation
        .data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == s.entity)
        .unwrap();
    Ok(match intent.kind {
        Kind::Attack(Target::Player { entity, .. } | Target::Honor { entity }) => C::CombatAttack {
            target_entity_id: entity as u64,
        },
        Kind::Attack(Target::Npc { serial, life }) => C::NpcAttack {
            target: npc(serial),
            life: life as u64,
        },
        Kind::Attack(Target::Quest { slot, .. }) => C::QuestAttack {
            target: super::quest_protocol::Id {
                scope,
                slot: slot.try_into()?,
            },
        },
        Kind::Attack(Target::TimeTower) => C::TimeAttackTowerAttack {
            target: super::time_attack_protocol::TowerId { scope },
        },
        Kind::Attack(Target::Siege { slot }) => C::SiegeAttack {
            target: super::siege_protocol::Target {
                scope: siege_scope,
                slot: *super::siege_protocol::Slot::ALL
                    .get(usize::from(slot))
                    .ok_or_else(|| anyhow::anyhow!("siege slot invalid"))?,
            },
        },
        Kind::Portal => C::SiegePortal { scope: siege_scope },
        Kind::LightArrow(Target::Player { entity, .. } | Target::Honor { entity }) => {
            C::CastLightArrow {
                target_entity_id: entity as u64,
            }
        }
        Kind::LightArrow(Target::Npc { serial, life }) => C::CastNpcLightArrow {
            target: npc(serial),
            life: life as u64,
        },
        Kind::Heal(Target::Honor { entity }) => C::CastBasicHeal {
            target_entity_id: entity as u64,
        },
        Kind::Disintegrate(target) => {
            let sequence = actor
                .spells
                .sequence
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("spell sequence exhausted"))?
                as u64;
            let caster_life = super::battle_window::life(actor) as u64;
            match target {
                Target::Npc { serial, life } => {
                    let r = super::npc_disintegrate_protocol::Request {
                        context: s.context,
                        channel_id: s.channel,
                        target: npc(serial),
                        sequence,
                        caster_life,
                        target_life: life as u64,
                    };
                    r.validate()?;
                    C::CastNpcDisintegrate(r)
                }
                Target::Player { entity, life } => {
                    let r = super::disintegrate_cast_protocol::Request {
                        context: s.context,
                        channel_id: s.channel,
                        match_id: scope.match_id,
                        generation: scope.generation,
                        sequence,
                        caster_life,
                        target_entity_id: entity as u64,
                        target_life: life as u64,
                    };
                    r.validate()?;
                    C::CastDisintegrate(r)
                }
                _ => anyhow::bail!("unsupported canonical disintegrate target"),
            }
        }
        _ => anyhow::bail!("intent requires local navigation, not a combat command"),
    })
}

/// Bind a painted ground intent to current command authority. The shared map
/// is static; blockers, life, gate state and every MOVE acknowledgment are not.
/// No canonical value is converted into a legacy Frame or fabricated ACK.
pub(crate) fn navigate(
    runtime: &BattleRuntime,
    view: &LoginSnapshot,
    intent: super::battle_window::Intent,
    route: &mut super::world_navigation::Navigation,
) -> Result<()> {
    validate_intent(intent, view)?;
    let super::battle_window::Kind::Navigate(goal) = intent.kind else {
        anyhow::bail!("canonical intent is not a ground target");
    };
    let current = view.battle.observation().unwrap();
    let map = runtime
        .routing_map(current)
        .ok_or_else(|| anyhow::anyhow!("qualified route terrain pending"))?;
    let frame = view
        .world
        .frame()
        .ok_or_else(|| anyhow::anyhow!("world command authority missing"))?;
    ensure!(
        frame.manifest.digest == map.canonical_digest() && frame.manifest.map_id == map.map_id(),
        "route map mismatch"
    );
    let entity = intent.stamp.entity as u64;
    if let Some(reply) = view.npc.as_ref() {
        ensure!(
            reply.frame.as_ref().is_some_and(
                |n| &n.combat.world == frame && view.combat.as_deref() == Some(&n.combat)
            ),
            "NPC command snapshots are not atomic"
        );
        route.request_npc(map, reply.clone(), entity, goal)?;
    } else if let Some(combat) = view.combat.as_ref() {
        ensure!(
            &combat.world == frame,
            "combat command snapshots are not atomic"
        );
        route.request_combat(map, combat.clone(), entity, goal)?;
    } else {
        route.request(map, std::sync::Arc::new(frame.clone()), entity, goal)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::modern_lito::{honor_protocol, quest_life, siege_protocol};
    #[test]
    fn command_lives_use_each_modes_counter_and_do_not_advance_at_death() {
        let before = honor_protocol::tests::initial();
        let dead = honor_protocol::tests::died(&before, 1);
        let revived = honor_protocol::tests::respawn(&dead, 1);
        for (reply, expected) in [(before, 1), (dead, 1), (revived, 2)] {
            let f = reply.frame.unwrap();
            assert_eq!(combat_life(&f, 1), Some(expected));
            assert_eq!(combat_life(&f, u64::MAX), None);
        }
        for (name, expected) in [("before", 1), ("dead", 1), ("revived", 2)] {
            let f = quest_life::tests::sample(name).frame.unwrap().combat;
            assert_eq!(combat_life(&f, 1), Some(expected));
        }
        for (name, expected) in [("lifeactiveplain", 1), ("respawnplain", 2)] {
            let f = siege_protocol::tests::sample(name).frame.unwrap();
            assert_eq!(combat_life(&f, 2), Some(expected));
        }
    }
}
