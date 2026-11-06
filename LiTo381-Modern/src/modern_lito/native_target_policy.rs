//! Selected-player intent at the controller boundary, before LFC1 and queueing.
//! S_CD names/forces qualify known room players. S0B has no authoritative PC
//! type or team ID, so unlisted NPCs/companions retain server-owned admission.
use super::{
    native_forced_combat::{self, Input},
    native_login::{LoginPhase, Snapshot},
    native_room::{Member, Room},
    native_world::{Actor, World},
};
use anyhow::{ensure, Context, Result};

fn member<'a>(room: &'a Room, world: &World, actor: &Actor) -> Result<Option<&'a Member>> {
    if !actor.master.is_empty() {
        return Ok(None);
    }
    let mut matches = room
        .members
        .iter()
        .filter(|member| member.name == actor.name);
    let Some(member) = matches.next() else {
        return Ok(None);
    };
    ensure!(
        matches.next().is_none()
            && member.force <= 1
            && world
                .actors
                .values()
                .filter(|other| other.name == actor.name)
                .count()
                == 1,
        "玩家目標身分不明，請等待房間資料更新"
    );
    Ok(Some(member))
}

pub(crate) fn validate(state: &Snapshot, body: &[u8], input: Input) -> Result<()> {
    let Some(id) = native_forced_combat::object_target(body) else {
        return Ok(());
    };
    let world = &state.world;
    ensure!(
        !state.observer && !world.observer_mode && !state.battle_withdrawal_pending,
        "觀戰或退出等待中不能攻擊或施法"
    );
    let own = world.own_actor().context("施法者已離開")?;
    let target = world.actors.get(&id).context("目標已離開")?;
    if id == own.id {
        return Ok(());
    }
    if state.phase != LoginPhase::BattlePending {
        return Ok(());
    }
    let Some(room) = &state.joined_room else {
        return Ok(());
    };
    // BattleRules.Mode: 1/3/5 are team-based PvP, 2/4 cooperative.
    // Pet Race and Town retain their original policies and never gain Ctrl intent.
    if !matches!(room.mode, 1 | 3 | 5) {
        return Ok(());
    }
    let Some(target_member) = member(room, world, target)? else {
        return Ok(());
    };
    let own_member = member(room, world, own)?.context("等待自己的房間隊伍資料")?;
    if own_member.force != target_member.force {
        ensure!(
            input.foreground && input.ctrl,
            "對敵方玩家請在前景視窗按住 Ctrl 明確指定目標"
        );
    }
    // Ctrl supplies intent only. Life, visibility, range, skill kind and clan
    // validation have already run. The server still owns LOS exceptions,
    // immunity, target observer status, authoritative clans and material commit.
    Ok(())
}
