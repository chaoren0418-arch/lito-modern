//! Player intents -> canonical operations. No legacy Frame, optimistic state,
//! retry, resource read or network access. The controller still admits the
//! captured command after its final poll and waits for the exact TLS receipt.
use super::{
    battle_action::Guard,
    battle_control::{Command, Operation},
    battle_follow::Observation,
    battle_payload::data as d,
    battle_scene::Surface,
    battle_window::{self as window, Intent, Kind, Local, Target},
    login_window::LoginSnapshot,
    world_protocol::MatchDescriptor as Features,
};
use anyhow::{ensure, Context, Result};

fn selected(view: &LoginSnapshot) -> Result<&Observation> {
    ensure!(
        view.battle_supported
            && view.canonical.supported()
            && view
                .canonical
                .snapshot()
                .is_some_and(|s| { Some(s.scope()) == super::battle_control::scope_for(view) }),
        "新版戰鬥操作不屬於目前選角或頻道"
    );
    let observation = view
        .canonical
        .observation()
        .context("新版戰鬥狀態尚未就緒")?;
    ensure!(
        observation.legacy_commands_supported(),
        "完整 VIP 戰鬥操作尚未開放"
    );
    Ok(observation)
}

pub(crate) fn validate(intent: Intent, view: &LoginSnapshot) -> Result<()> {
    validate_observation(intent, selected(view)?)
}
/// Button availability is not command authority. Targets are still checked
/// when a painted intent is captured and again after the controller's poll.
pub(crate) fn ready(kind: Kind, view: &LoginSnapshot) -> bool {
    let Ok(o) = selected(view) else { return false };
    !view.world_pending
        && (kind != Kind::Local(Local::GreaterHaste) || view.canonical.greater_haste_supported())
        && (!matches!(kind, Kind::Haste(_)) || view.canonical.target_haste_supported())
        && (!matches!(kind, Kind::Slow(_)) || view.canonical.slow_supported())
        && (!matches!(kind, Kind::Heal(Target::Player { .. }))
            || view.canonical.player_healing_supported())
        && validate_observation(
            Intent {
                stamp: window::Stamp::of(o),
                kind: Kind::Local(Local::Step(0)),
            },
            o,
        )
        .is_ok()
        && eligibility(kind, o, &view.skills).is_ok()
}
pub(crate) fn skill_available(id: u32, view: &LoginSnapshot) -> bool {
    let Ok(o) = selected(view) else { return false };
    if id == 1 && view.canonical.player_healing_supported() && player_healing_qualified(o) {
        return true;
    }
    let npc = o.data().npcs.is_some();
    let t = if npc {
        Target::Npc { serial: 1, life: 1 }
    } else {
        Target::Player { entity: 1, life: 1 }
    };
    let kind = match id {
        5 => Kind::Local(Local::Teleport),
        70 => Kind::Local(Local::FireStorm),
        54 => Kind::Local(Local::GreaterHaste),
        43 => Kind::Haste(t),
        29 => Kind::Slow(t),
        4 => Kind::LightArrow(t),
        77 => Kind::Disintegrate(t),
        1 => Kind::Heal(Target::Honor { entity: 3 }),
        _ => return false,
    };
    let Some((_, support, lane, flag)) = skill(kind, o) else {
        return false;
    };
    feature(o, flag)
        && actor(o).is_ok_and(|a| a.spells.histories.0.iter().any(|h| h.lane == lane))
        && if id == 29 {
            view.canonical.slow_supported()
                && matches!(&view.skills, super::spell_protocol::SkillsState::Ready(r)
                if r.context == window::Stamp::of(o).context && r.slow().is_some())
        } else if id == 43 {
            view.canonical.target_haste_supported()
                && matches!(&view.skills, super::spell_protocol::SkillsState::Ready(r)
                if r.context == window::Stamp::of(o).context && r.haste().is_some())
        } else if id == 54 {
            view.canonical.greater_haste_supported()
                && matches!(&view.skills, super::spell_protocol::SkillsState::Ready(r)
                if r.context == window::Stamp::of(o).context && r.greater_haste().is_some())
        } else if id == 77 {
            matches!(&view.skills, super::spell_protocol::SkillsState::Ready(r)
            if r.context == window::Stamp::of(o).context && r.disintegrate().is_some())
        } else {
            o.personal()
                .skills()
                .iter()
                .any(|s| s.id == id as i32 && s.support == support)
        }
}

fn validate_observation(intent: Intent, o: &Observation) -> Result<()> {
    ensure!(o.legacy_commands_supported(), "完整 VIP 戰鬥操作尚未開放");
    ensure!(
        intent.stamp.current(o) && o.data().world.phase == d::Phase::Active,
        "畫面的角色生命、場次或戰鬥階段已更新"
    );
    let a = actor(o)?;
    ensure!(
        a.loaded
            && a.vital.as_ref().is_none_or(|v| v.values.hp > 0)
            && super::battle_space::occupies(o.data(), a.entity),
        "未載入、死亡或已退場的角色不能操作"
    );
    match intent.kind {
        Kind::Attack(t) | Kind::LightArrow(t) | Kind::Disintegrate(t) => {
            ensure!(window::target_current(o, t), "目標已死亡、重生或更換");
        }
        Kind::Haste(t) => ensure!(window::can_haste(o, t), "加速目標已死亡、離場或更換"),
        Kind::Slow(t) => ensure!(window::can_slow(o, t), "緩速目標已死亡、離場或更換"),
        Kind::Portal => ensure!(window::portal(o), "目前不在可使用的攻城傳送點"),
        Kind::Local(local) => ensure!(local.valid(), "不支援此按鍵操作"),
        _ => {}
    }
    if let Kind::Heal(t) = intent.kind {
        ensure!(window::can_heal(o, t), "治癒目標已死亡、離場或不屬於此分區");
    }
    Ok(())
}

/// Use the exact Surface actually painted by the HWND. A completed future
/// render or an old same-life pose does not authorize a keyboard MOVE.
pub(crate) fn command(surface: &Surface, intent: Intent, view: &LoginSnapshot) -> Result<Command> {
    ensure!(
        intent.stamp == window::Stamp::of(&surface.observation)
            && super::battle_player::controls_ready(surface, view),
        "畫面或角色位置尚未與伺服器一致"
    );
    let o = selected(view)?;
    validate_observation(intent, o)?;
    if let Kind::Local(Local::Step(heading)) = intent.kind {
        let a = actor(o)?;
        let from = super::world_map::WorldCell {
            x: a.position.x,
            y: a.position.y,
        };
        let (dx, dy) = super::world_map::DIRECTIONS[usize::from(heading)];
        let to = super::world_map::WorldCell {
            x: from.x.checked_add(dx).context("移動座標超界")?,
            y: from.y.checked_add(dy).context("移動座標超界")?,
        };
        ensure!(
            surface
                .navigation
                .as_ref()
                .is_some_and(|n| n.can_step(from, to)),
            "該位置目前無法通行"
        );
    }
    admitted_command(intent, &surface.observation, view)
}

/// Target-only entry used by SessionController after polling. Local steps and
/// ground routing must keep their painted terrain proof, not use this shortcut.
pub(crate) fn target_command(intent: Intent, view: &LoginSnapshot) -> Result<Command> {
    ensure!(
        matches!(
            intent.kind,
            Kind::Attack(_)
                | Kind::LightArrow(_)
                | Kind::Heal(_)
                | Kind::Disintegrate(_)
                | Kind::Haste(_)
                | Kind::Slow(_)
                | Kind::Portal
        ),
        "此操作需要畫面及地形證明"
    );
    // This entry has no retained painted Observation. Do not invent historical
    // caster/target facts: specialized spells still require this exact stamp.
    admitted_command(intent, selected(view)?, view)
}

fn admitted_command(
    intent: Intent,
    painted: &Observation,
    view: &LoginSnapshot,
) -> Result<Command> {
    ensure!(!view.world_pending, "上一個戰鬥操作尚未確認");
    let o = selected(view)?;
    validate_observation(intent, o)?;
    eligibility(intent.kind, o, &view.skills)?;
    let command = if let Kind::Heal(Target::Player { entity, life }) = intent.kind {
        ensure!(
            view.canonical.player_healing_supported(),
            "尚未協商角色治癒指令"
        );
        let q = super::battle_player_healing::Intent::from_observation(painted, entity)
            .context("角色治癒目標尚未就緒")?;
        ensure!(
            q.stamp == intent.stamp && q.target.life == life,
            "施法畫面或目標生命已更新，請重新確認"
        );
        Command::player_healing(q)
    } else if let Kind::Slow(target) = intent.kind {
        ensure!(view.canonical.slow_supported(), "尚未協商緩速術指令");
        let q = super::battle_slow::Intent::from_observation(painted, target)
            .context("緩速術目標尚未就緒")?;
        ensure!(q.stamp == intent.stamp, "施法畫面版本已更新，請重新確認");
        Command::slow(q)
    } else if let Kind::Haste(target) = intent.kind {
        let q = super::battle_target_haste::Intent::from_observation(painted, target)
            .context("加速術目標尚未就緒")?;
        ensure!(q.stamp == intent.stamp, "施法畫面版本已更新，請重新確認");
        Command::target_haste(q)
    } else if intent.kind == Kind::Local(Local::GreaterHaste) {
        let q = super::battle_greater_haste::Intent::from_observation(painted)
            .context("強力加速術尚未就緒")?;
        ensure!(q.stamp == intent.stamp, "施法畫面版本已更新，請重新確認");
        Command::greater_haste(q)
    } else {
        Command::operation(
            Guard {
                stamp: intent.stamp,
                target: target(intent.kind),
            },
            operation(intent, o)?,
        )
    };
    command.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        command.current(&view.canonical, view.navigation.as_ref()),
        "戰鬥操作已失效"
    );
    Ok(command)
}

fn target(kind: Kind) -> Option<Target> {
    match kind {
        Kind::Attack(t) | Kind::LightArrow(t) | Kind::Heal(t) | Kind::Disintegrate(t) => Some(t),
        _ => None,
    }
}
fn actor(o: &Observation) -> Result<&d::Actor> {
    o.data()
        .actors
        .0
        .iter()
        .find(|a| a.entity == o.data().personal.entity)
        .context("缺少目前角色的戰鬥資料")
}
fn next(sequence: i64) -> Result<u64> {
    ensure!(sequence >= 0, "操作序號不可為負值");
    Ok(sequence.checked_add(1).context("操作序號已達上限")? as u64)
}
fn feature(o: &Observation, flag: u32) -> bool {
    o.data().world.descriptor.features as u32 & flag != 0
}
fn supported(o: &Observation, flag: u32) -> Result<()> {
    ensure!(feature(o, flag), "本場未啟用此功能");
    Ok(())
}
fn skill(kind: Kind, o: &Observation) -> Option<(u32, d::Support, d::Lane, u32)> {
    Some(match kind {
        Kind::Slow(_) => (
            29,
            d::Support::Unimplemented,
            d::Lane::Slow,
            Features::SLOW_CANDIDATE,
        ),
        Kind::Haste(_) => (
            43,
            d::Support::Unimplemented,
            d::Lane::Haste,
            Features::HASTE_CANDIDATE,
        ),
        Kind::Local(Local::GreaterHaste) => (
            54,
            d::Support::Unimplemented,
            d::Lane::GreaterHaste,
            Features::GREATER_HASTE,
        ),
        Kind::Local(Local::Teleport) => {
            (5, d::Support::Teleport, d::Lane::Teleport, Features::SPELLS)
        }
        Kind::Local(Local::FireStorm) if feature(o, Features::NPC_FIRE_STORM) => (
            70,
            d::Support::FireStorm,
            d::Lane::NpcFireStorm,
            Features::NPC_FIRE_STORM,
        ),
        Kind::Local(Local::FireStorm) => (
            70,
            d::Support::FireStorm,
            d::Lane::FireStorm,
            Features::FIRE_STORM,
        ),
        Kind::LightArrow(Target::Npc { .. }) => (
            4,
            d::Support::LightArrow,
            d::Lane::NpcLightArrow,
            Features::NPC_LIGHT_ARROW,
        ),
        Kind::LightArrow(_) => (
            4,
            d::Support::LightArrow,
            d::Lane::LightArrow,
            Features::LIGHT_ARROW,
        ),
        Kind::Heal(Target::Honor { .. }) => {
            (1, d::Support::BasicHeal, d::Lane::Healing, Features::HONOR)
        }
        Kind::Disintegrate(Target::Npc { .. }) => (
            77,
            d::Support::Disintegrate,
            d::Lane::NpcDisintegrate,
            Features::NPC_DISINTEGRATE,
        ),
        Kind::Disintegrate(_) => (
            77,
            d::Support::Disintegrate,
            d::Lane::Disintegrate,
            Features::DISINTEGRATE,
        ),
        _ => return None,
    })
}
fn eligibility(
    kind: Kind,
    o: &Observation,
    skills: &super::spell_protocol::SkillsState,
) -> Result<()> {
    let a = actor(o)?;
    if matches!(kind, Kind::Attack(_) | Kind::Heal(_)) || skill(kind, o).is_some() {
        ensure!(
            o.personal().weight().is_none_or(|w| w.can_attack()),
            "負重過高，無法攻擊或施法"
        );
    }
    if matches!(kind, Kind::Heal(Target::Player { .. })) {
        ensure!(
            player_healing_qualified(o),
            "本場未開放角色治癒或來源未資格化"
        );
        let outgoing = window::player_healing_outgoing(o).context("缺少角色治癒資料")?;
        ensure!(
            a.vital
                .as_ref()
                .is_some_and(|v| v.values.mp >= i64::from(outgoing.ledger.cost)),
            "魔力不足"
        );
    } else if matches!(kind, Kind::Heal(t) if !matches!(t, Target::Honor { .. })) {
        anyhow::bail!("尚未支援此類治癒目標");
    }
    if let Some((id, support, lane, flag)) = skill(kind, o) {
        supported(o, flag)?;
        if id == 29 {
            let super::spell_protocol::SkillsState::Ready(reply) = skills else {
                anyhow::bail!("技能資格尚未載入")
            };
            ensure!(
                reply.context == window::Stamp::of(o).context && reply.slow().is_some(),
                "角色尚未取得緩速術資格"
            );
            ensure!(
                o.data().personal.materials.as_ref().is_some_and(|m| m
                    .stocks
                    .0
                    .iter()
                    .any(|s| s.item == 40318 && s.remaining > 0)),
                "魔法寶石不足"
            );
        } else if id == 43 {
            let super::spell_protocol::SkillsState::Ready(reply) = skills else {
                anyhow::bail!("技能資格尚未載入")
            };
            ensure!(
                reply.context == window::Stamp::of(o).context && reply.haste().is_some(),
                "角色尚未取得加速術資格"
            );
        } else if id == 54 {
            let super::spell_protocol::SkillsState::Ready(reply) = skills else {
                anyhow::bail!("技能資格尚未載入");
            };
            ensure!(
                reply.context == window::Stamp::of(o).context && reply.greater_haste().is_some(),
                "角色尚未取得強力加速術資格"
            );
        } else if id == 77 {
            // LTB1's raw source skill list does not include the independent
            // Disintegrate qualification catalog. Do not infer permission
            // from a history lane, old equipment, or a differently scoped list.
            let super::spell_protocol::SkillsState::Ready(reply) = skills else {
                anyhow::bail!("技能資格尚未載入");
            };
            ensure!(
                reply.context == window::Stamp::of(o).context && reply.disintegrate().is_some(),
                "角色尚未取得此技能資格"
            );
            ensure!(a.spells.lawful >= 500, "究極光裂術需要正義值 500");
        } else {
            ensure!(
                o.personal()
                    .skills()
                    .iter()
                    .any(|s| s.id == id as i32 && s.support == support),
                "角色尚未學會或不支援此技能"
            );
        }
        let lane = a
            .spells
            .histories
            .0
            .iter()
            .find(|h| h.lane == lane)
            .context("本場沒有此施法通道")?;
        ensure!(
            a.vital
                .as_ref()
                .is_some_and(|v| v.values.mp >= i64::from(lane.ledger.cost)),
            "魔力不足"
        );
    }
    match kind {
        Kind::Attack(t) => {
            supported(
                o,
                match t {
                    Target::Player { .. } | Target::Honor { .. } => Features::COMBAT,
                    Target::Npc { .. } => Features::NPC,
                    Target::Quest { .. } => Features::QUEST_OBJECTIVES,
                    Target::TimeTower => Features::TIME_ATTACK,
                    Target::Siege { .. } => Features::SIEGE,
                },
            )?;
            ensure!(a.vital.is_some(), "本場沒有角色攻擊資料");
        }
        Kind::Local(Local::UseItem(id)) => {
            supported(o, Features::ITEMS)?;
            let s = o.personal().supplies().context("缺少本場補給資料")?;
            ensure!(
                s.stocks.0.iter().any(|s| s.item == id && s.remaining > 0),
                "此補給已用完"
            );
            if let Some(k) = super::timed_protocol::Kind::from_item(id) {
                supported(o, Features::TIMED)?;
                let member = o
                    .data()
                    .room
                    .members
                    .0
                    .get((a.entity - 1) as usize)
                    .context("缺少職業資料")?;
                ensure!(
                    k.eligible(member.class.try_into()?),
                    "職業不符，無法使用此補給"
                );
            }
        }
        Kind::Local(Local::SelectAmmo(id)) => {
            supported(o, Features::AMMO)?;
            let ammo = o
                .data()
                .personal
                .ammo
                .as_ref()
                .context("缺少本場彈藥資料")?;
            ensure!(
                ammo.stocks
                    .0
                    .iter()
                    .any(|s| s.item == id && s.remaining > 0),
                "此彈藥已用完"
            );
        }
        Kind::Portal => supported(o, Features::SIEGE)?,
        _ => {}
    }
    Ok(())
}

fn player_healing_qualified(o: &Observation) -> bool {
    window::player_healing_outgoing(o).is_some()
        && o.personal()
            .skills()
            .iter()
            .any(|s| s.id == 1 && s.support == d::Support::BasicHeal)
}

/// The one skill-page/shortcut can target two explicitly different namespaces.
/// Availability for one must not disable the other or authorize its command.
pub(crate) fn healing_ready(view: &LoginSnapshot) -> bool {
    let Ok(o) = selected(view) else { return false };
    ready(
        Kind::Heal(Target::Player {
            entity: o.data().personal.entity,
            life: window::Stamp::of(o).life,
        }),
        view,
    ) || ready(Kind::Heal(Target::Honor { entity: 3 }), view)
}

/// Packet fields come only from a followed observation and the original
/// target intent. Keep each independent sequence; never use world revision as
/// attack, spell, supply, ammo, movement or life authority.
fn operation(intent: Intent, o: &Observation) -> Result<Operation> {
    use super::{
        ammo_protocol as ammo, combat_protocol as combat, item_protocol as item,
        npc_protocol as npc, siege_protocol as siege, spell_protocol as spell,
        world_protocol as world,
    };
    let s = intent.stamp;
    let a = actor(o)?;
    let scope = npc::Scope {
        room_id: s.world[0] as u64,
        match_id: s.world[1] as u64,
        generation: s.world[2] as u64,
    };
    let npc_id = |serial: i64| npc::Id {
        scope,
        serial: serial as u64,
    };
    let siege_scope = siege::Scope {
        room_id: scope.room_id,
        match_id: scope.match_id,
        generation: scope.generation,
    };
    let attack = || next(a.vital.as_ref().context("缺少攻擊序號")?.attack_sequence);
    let cast = |skill_id, target_entity_id| -> Result<Operation> {
        Ok(Operation::Spell(spell::CastRequest {
            context: s.context,
            channel_id: s.channel,
            match_id: scope.match_id,
            generation: scope.generation,
            sequence: next(a.spells.sequence)?,
            skill_id,
            target_entity_id,
        }))
    };
    Ok(match intent.kind {
        Kind::Local(Local::Step(heading)) => Operation::Move(world::Request {
            action: world::Action::Move,
            match_id: scope.match_id,
            generation: scope.generation,
            sequence: next(a.last_move_sequence)?,
            from_x: a.position.x,
            from_y: a.position.y,
            heading,
            ..world::Request::read(s.context, s.channel)
        }),
        Kind::Attack(Target::Player { entity, .. } | Target::Honor { entity }) => {
            Operation::Attack(combat::Request {
                context: s.context,
                channel_id: s.channel,
                action: combat::Action::Attack,
                match_id: scope.match_id,
                generation: scope.generation,
                sequence: attack()?,
                target_entity_id: entity as u64,
            })
        }
        Kind::Attack(t @ (Target::Npc { .. } | Target::Quest { .. } | Target::TimeTower)) => {
            let (action, serial, life) = match t {
                Target::Npc { serial, life } => (npc::Action::Attack, serial as u64, life as u64),
                Target::Quest { slot, .. } => (npc::Action::AttackObjective, slot as u64, 0),
                _ => (npc::Action::AttackTower, 0, 0),
            };
            Operation::Npc(npc::Request {
                context: s.context,
                channel_id: s.channel,
                world: scope,
                sequence: attack()?,
                action,
                serial,
                life,
            })
        }
        Kind::Attack(Target::Siege { slot }) => Operation::Siege(siege::Request {
            context: s.context,
            channel_id: s.channel,
            scope: siege_scope,
            sequence: attack()?,
            action: siege::Action::AttackTower(
                *siege::Slot::ALL
                    .get(usize::from(slot))
                    .context("攻城目標無效")?,
            ),
        }),
        Kind::Portal => Operation::Siege(siege::Request {
            context: s.context,
            channel_id: s.channel,
            scope: siege_scope,
            sequence: next(a.last_move_sequence)?,
            action: siege::Action::Portal,
        }),
        Kind::Local(Local::Teleport) => cast(spell::TELEPORT, 0)?,
        Kind::Local(Local::FireStorm) if feature(o, Features::NPC_FIRE_STORM) => {
            Operation::NpcFireStorm(super::npc_fire_storm_protocol::Request {
                context: s.context,
                channel_id: s.channel,
                scope,
                sequence: next(a.spells.sequence)?,
                caster_life: s.life as u64,
            })
        }
        Kind::Local(Local::FireStorm) => cast(spell::FIRE_STORM, 0)?,
        Kind::LightArrow(Target::Player { entity, .. } | Target::Honor { entity }) => {
            cast(spell::LIGHT_ARROW, entity as u64)?
        }
        Kind::Heal(Target::Honor { entity }) => cast(spell::BASIC_HEAL, entity as u64)?,
        Kind::LightArrow(Target::Npc { serial, life }) => {
            Operation::NpcSpell(super::npc_spell_protocol::Request {
                context: s.context,
                channel_id: s.channel,
                target: npc_id(serial),
                sequence: next(a.spells.sequence)?,
                caster_life: s.life as u64,
                target_life: life as u64,
            })
        }
        Kind::Disintegrate(Target::Player { entity, life }) => {
            Operation::Disintegrate(super::disintegrate_cast_protocol::Request {
                context: s.context,
                channel_id: s.channel,
                match_id: scope.match_id,
                generation: scope.generation,
                sequence: next(a.spells.sequence)?,
                caster_life: s.life as u64,
                target_entity_id: entity as u64,
                target_life: life as u64,
            })
        }
        Kind::Disintegrate(Target::Npc { serial, life }) => {
            Operation::NpcDisintegrate(super::npc_disintegrate_protocol::Request {
                context: s.context,
                channel_id: s.channel,
                target: npc_id(serial),
                sequence: next(a.spells.sequence)?,
                caster_life: s.life as u64,
                target_life: life as u64,
            })
        }
        Kind::Local(Local::UseItem(item_id)) => Operation::Item(item::Request {
            context: s.context,
            channel_id: s.channel,
            action: item::Action::Use,
            match_id: scope.match_id,
            generation: scope.generation,
            sequence: next(
                o.data()
                    .personal
                    .supplies
                    .as_ref()
                    .context("缺少補給序號")?
                    .request_sequence,
            )?,
            item_id,
        }),
        Kind::Local(Local::SelectAmmo(item_id)) => Operation::Ammo(ammo::Request {
            context: s.context,
            channel_id: s.channel,
            action: ammo::Action::Select,
            match_id: scope.match_id,
            generation: scope.generation,
            sequence: next(
                o.data()
                    .personal
                    .ammo
                    .as_ref()
                    .context("缺少彈藥序號")?
                    .request_sequence,
            )?,
            item_id,
        }),
        _ => anyhow::bail!("此點擊不是可直接送出的戰鬥操作"),
    })
}

#[cfg(test)]
#[path = "battle_input_tests.rs"]
pub(crate) mod tests;
