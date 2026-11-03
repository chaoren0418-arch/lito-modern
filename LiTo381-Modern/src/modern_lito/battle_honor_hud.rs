//! Presentation of already-validated tower/life/result authority. A timer or
//! zero tower HP cannot locally revive anyone or award a personal win.
use super::d;

pub(super) fn lines(data: &d::UnvalidatedData) -> Vec<String> {
    if data.world.descriptor.ruleset != d::Ruleset::HonorTowers {
        return Vec::new();
    }
    let Some(member) = usize::try_from(data.personal.entity - 1)
        .ok()
        .and_then(|i| data.room.members.0.get(i))
    else {
        return Vec::new();
    };
    let own = member.team;
    let Some(tower) = data.honor.objectives.0.iter().find(|t| t.team == own) else {
        return Vec::new();
    };
    let Some(enemy) = data.honor.objectives.0.iter().find(|t| t.team != own) else {
        return Vec::new();
    };
    let living = |team| {
        data.actors
            .0
            .iter()
            .filter(|a| {
                data.room
                    .members
                    .0
                    .get((a.entity - 1) as usize)
                    .is_some_and(|m| m.team == team)
                    && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
            })
            .count()
    };
    let total = |team| {
        data.room
            .members
            .0
            .iter()
            .filter(|m| m.team == team)
            .count()
    };
    let mut lines = vec![
        format!(
            "榮譽戰 · 己方隊{own} 存活{}/{} · 敵方存活{}/{}",
            living(own),
            total(own),
            living(enemy.team),
            total(enemy.team)
        ),
        format!(
            "己方守護塔 {}/{} · 敵方守護塔 {}/{}",
            tower.hp, tower.maximum_hp, enemy.hp, enemy.maximum_hp
        ),
        "目標：摧毀敵方守護塔；擊倒角色不會直接結束比賽。".into(),
    ];
    if let Some(result) = data.result.as_ref().and_then(|r| {
        r.participants
            .0
            .iter()
            .find(|p| p.entity == data.personal.entity)
    }) {
        lines.push(format!(
            "本場{}（非累計戰績；保存狀態另依伺服器回覆）",
            if result.verdict == d::Verdict::Win {
                "勝利"
            } else {
                "敗北"
            }
        ));
    } else if data.world.phase == d::Phase::Aborted {
        lines.push("本場已中止；不推算勝敗或復活。".into());
    } else if let Some(life) = data
        .honor
        .lives
        .0
        .iter()
        .find(|l| l.entity == data.personal.entity)
    {
        if data.world.phase == d::Phase::Active && life.death_count > life.respawn_count {
            // The wire has a due value but no equivalent current active clock.
            // Do not subtract the unrelated loading clock or a local wall clock.
            lines.push(format!(
                "已陣亡，等待伺服器自動復活（標準延遲5秒）；本場死亡{}／已復活{}",
                life.death_count, life.respawn_count
            ));
        } else {
            lines.push(format!(
                "本場死亡{}／已復活{}；生命狀態由伺服器確認。",
                life.death_count, life.respawn_count
            ));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    // Formatter fixtures only. Full payload/SQL authority is exercised by the
    // actual controller test, not established by these synthetic values.
    fn fixture() -> d::UnvalidatedData {
        let mut v = crate::modern_lito::battle_payload::data_tests::fixture();
        v.world.descriptor.ruleset = d::Ruleset::HonorTowers;
        v.world.phase = d::Phase::Active;
        v.room.members.0[0].team = 0;
        v.room.members.0[1].team = 1;
        v.honor.objectives = d::List(
            (0..2)
                .map(|team| d::HonorObjective {
                    entity: 3 + i64::from(team),
                    team,
                    action: 0,
                    graphic: 2665,
                    cell: d::Cell { x: 0, y: 0 },
                    hp: if team == 0 { 70_000 } else { 60_000 },
                    maximum_hp: 90_000,
                    revision: 0,
                    hurt_revision: 0,
                    death_revision: 0,
                    damage_loss: 0,
                    heal_gain: 0,
                    last_actor: 0,
                    last_kind: 0,
                })
                .collect(),
        );
        v.honor.lives = d::List(vec![d::HonorLife {
            entity: 1,
            death_count: 1,
            last_death_revision: 1,
            respawn_count: 0,
            respawn_revision: 0,
            respawn_hp_gain: 0,
            pending_due_millis: 5000,
            death_cell: d::Cell { x: 0, y: 0 },
        }]);
        v
    }
    #[test]
    fn perspective_tracks_owner_and_preserves_extended_hp() {
        let mut v = fixture();
        assert!(lines(&v)[0].contains("己方隊0"));
        assert!(lines(&v)[1].contains("己方守護塔 70000/90000"));
        v.personal.entity = 2;
        assert!(lines(&v)[0].contains("己方隊1"));
        assert!(lines(&v)[1].contains("己方守護塔 60000/90000"));
    }
    #[test]
    fn loading_clock_cannot_confirm_respawn_or_victory() {
        let mut v = fixture();
        v.world.loading_elapsed_nanos = 99_000_000_000;
        v.honor.objectives.0[1].hp = 0;
        let text = lines(&v).join("\n");
        assert!(text.contains("等待伺服器自動復活"));
        assert!(!text.contains("本場勝利"));
        v.honor.lives.0[0].respawn_count = 1;
        let text = lines(&v).join("\n");
        assert!(text.contains("死亡1／已復活1"));
        assert!(!text.contains("等待伺服器自動復活"));
    }
    #[test]
    fn aborted_and_other_modes_do_not_retain_pending_respawn() {
        let mut v = fixture();
        v.world.phase = d::Phase::Aborted;
        let text = lines(&v).join("\n");
        assert!(text.contains("本場已中止"));
        assert!(!text.contains("等待伺服器自動復活"));
        v.world.descriptor.ruleset = d::Ruleset::DeathmatchElimination;
        assert!(lines(&v).is_empty());
    }
}
