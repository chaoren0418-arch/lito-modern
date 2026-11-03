//! Presentation from the painted, validated canonical observation only. No
//! local clock, legacy Frame, inferred winning team or invented SQL receipt.
use crate::modern_lito::battle_payload::data as d;

fn short(s: &str) -> String {
    if s.chars().count() > 16 {
        format!("{}…", s.chars().take(16).collect::<String>())
    } else {
        s.to_owned()
    }
}

pub(super) fn lines(data: &d::UnvalidatedData) -> Vec<String> {
    let Some(t) = &data.time_attack else {
        return Vec::new();
    };
    let phase = match t.phase {
        d::TimePhase::WaitingForTower => "擊破啟動塔後開始",
        d::TimePhase::SpawnPending => "等待下一波",
        d::TimePhase::Fighting => "戰鬥中",
        d::TimePhase::Aborted => "本場中止，沒有勝敗結果",
        d::TimePhase::Finished => match t.end {
            d::TimeEnd::Cleared => "全部波次完成",
            d::TimeEnd::Timeout => "時間到",
            d::TimeEnd::LeaderDefeated => "隊長陣亡",
            d::TimeEnd::PartyDefeated => "隊伍已離場",
            d::TimeEnd::None => "結果未確認",
        },
    };
    let kind = match t.waves.0.get(t.wave as usize) {
        Some(d::TimeKind::Ordinary) => "普通",
        Some(d::TimeKind::FinalBosses) => "末波",
        Some(d::TimeKind::Dragon) => "龍波",
        None => "未確認",
    };
    let leader = usize::try_from(t.leader)
        .ok()
        .and_then(|n| n.checked_sub(1))
        .and_then(|i| data.room.members.0.get(i))
        .map(|m| format!("{}({})", short(&m.name.0), short(&m.source_name.0)))
        .unwrap_or_else(|| "未確認".into());
    let elapsed = if t.started_millis < 0 {
        0
    } else {
        t.observed_millis.saturating_sub(t.started_millis).max(0)
    };
    let remaining = i64::from(t.duration_millis).saturating_sub(elapsed).max(0);
    let seconds = (remaining + 999) / 1000;
    let alive = data
        .actors
        .0
        .iter()
        .filter(|a| {
            !t.departed.0.contains(&a.entity) && a.vital.as_ref().is_some_and(|v| v.values.hp > 0)
        })
        .count();
    let mut lines = vec![
        format!(
            "合作挑戰 · {phase} · 波次 {}/{}（{kind}）· 存活怪物 {}",
            i64::from(t.wave) + 1,
            t.waves.0.len(),
            t.living.0.len()
        ),
        format!("隊長：{leader}"),
        format!(
            "留場存活 {alive} · 離隊 {} · {} {:02}:{:02}",
            t.departed.0.len(),
            if t.started_millis < 0 {
                "時限"
            } else {
                "伺服器剩餘"
            },
            seconds / 60,
            seconds % 60
        ),
    ];
    if t.phase == d::TimePhase::WaitingForTower {
        lines.push(format!("啟動塔 {}/{}", t.tower_hp, t.tower.maximum_hp));
    }
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
    } else if t.departed.0.contains(&data.personal.entity) {
        lines.push("你已離隊；個人結果等待伺服器判定。".into());
    } else if data
        .actors
        .0
        .iter()
        .find(|a| a.entity == data.personal.entity)
        .is_some_and(|a| a.vital.as_ref().is_some_and(|v| v.values.hp == 0))
    {
        lines.push("你已陣亡；其他存活隊員可繼續，個人結果等待伺服器判定。".into());
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    // Formatter fixture only; full authority/scene validation is covered by
    // the real v41 Main/SQL integration test, not claimed by these unit values.
    fn fixture() -> d::UnvalidatedData {
        let mut data = crate::modern_lito::battle_payload::data_tests::fixture();
        data.room.members.0[0].name.0 = "3333".into();
        data.room.members.0[1].name.0 = "4444".into();
        data.room.members.0[1].source_name.0 = "月亮神".into();
        data.time_attack = Some(d::TimeAttack {
            revision: 1,
            phase: d::TimePhase::WaitingForTower,
            end: d::TimeEnd::None,
            waves: d::List(vec![d::TimeKind::Ordinary, d::TimeKind::FinalBosses]),
            wave: 0,
            leader: 1,
            duration_millis: 60_000,
            started_millis: -1,
            observed_millis: 40_000,
            departed: d::List(vec![]),
            living: d::List(vec![]),
            tower: d::TimeTower {
                cell: d::Cell { x: 32768, y: 32769 },
                graphic: 2665,
                maximum_hp: 100,
            },
            tower_hp: 100,
            tower_hurt_revision: 0,
            tower_death_revision: 0,
            plan_sha256: [1; 32],
            exchanges: d::List(vec![]),
        });
        data
    }
    #[test]
    fn waiting_does_not_spend_timer_and_wave_is_one_based() {
        let mut data = fixture();
        let before = lines(&data);
        assert!(before[0].contains("擊破啟動塔後開始 · 波次 1/2（普通）"));
        assert!(before[1].contains("3333(太陽神)"));
        assert!(before[2].ends_with("時限 01:00"));
        assert_eq!(before[3], "啟動塔 100/100");
        data.world.active_elapsed_nanos = i64::MAX;
        data.world.revision += 100;
        assert_eq!(
            lines(&data),
            before,
            "unrelated world clock cannot advance mode timer"
        );
    }
    #[test]
    fn seven_minute_encounter_starts_after_tower_wait_and_uses_server_deadline() {
        let mut data = fixture();
        let t = data.time_attack.as_mut().unwrap();
        t.duration_millis = 420_000;
        t.observed_millis = 59_999;
        assert!(lines(&data)[2].ends_with("時限 07:00"));
        let t = data.time_attack.as_mut().unwrap();
        t.phase = d::TimePhase::Fighting;
        t.started_millis = 59_999;
        t.observed_millis = 60_000;
        assert!(lines(&data)[2].ends_with("伺服器剩餘 07:00"));
        data.time_attack.as_mut().unwrap().observed_millis = 479_998;
        assert!(lines(&data)[2].ends_with("伺服器剩餘 00:01"));
        let t = data.time_attack.as_mut().unwrap();
        t.phase = d::TimePhase::Finished;
        t.end = d::TimeEnd::Timeout;
        t.observed_millis = 479_999;
        let text = lines(&data).join("\n");
        assert!(text.contains("時間到") && text.contains("伺服器剩餘 00:00"));
        assert!(
            !text.contains("本場敗北"),
            "personal result still requires authority"
        );
    }
    #[test]
    fn server_clock_rounds_up_without_claiming_timeout_and_leader_can_change() {
        let mut data = fixture();
        let t = data.time_attack.as_mut().unwrap();
        t.phase = d::TimePhase::Fighting;
        t.started_millis = 40_000;
        t.observed_millis = 99_999;
        t.leader = 2;
        t.wave = 1;
        t.departed.0.push(1);
        let shown = lines(&data);
        assert!(shown[0].contains("戰鬥中 · 波次 2/2（末波）"));
        assert!(shown[1].contains("4444(月亮神)"));
        assert!(shown[2].ends_with("伺服器剩餘 00:01"));
        assert!(shown[3].contains("你已離隊"));
        data.time_attack.as_mut().unwrap().observed_millis = 100_001;
        let shown = lines(&data).join("\n");
        assert!(shown.contains("伺服器剩餘 00:00") && shown.contains("戰鬥中"));
        assert!(!shown.contains("時間到") && !shown.contains("本場敗北"));
        for (end, text) in [
            (d::TimeEnd::Timeout, "時間到"),
            (d::TimeEnd::LeaderDefeated, "隊長陣亡"),
            (d::TimeEnd::PartyDefeated, "隊伍已離場"),
            (d::TimeEnd::Cleared, "全部波次完成"),
        ] {
            let t = data.time_attack.as_mut().unwrap();
            t.phase = d::TimePhase::Finished;
            t.end = end;
            assert!(lines(&data)[0].contains(text));
        }
    }
    #[test]
    fn no_mode_is_empty_and_names_are_shortened_on_character_boundaries() {
        let mut data = fixture();
        data.room.members.0[0].name.0 = "勇".repeat(30);
        assert!(lines(&data)[1].contains(&format!("{}…(太陽神)", "勇".repeat(16))));
        data.time_attack = None;
        assert!(lines(&data).is_empty());
    }
    #[test]
    fn same_team_personal_verdict_is_not_a_team_win_or_sql_receipt() {
        let mut data = fixture();
        data.time_attack.as_mut().unwrap().phase = d::TimePhase::Finished;
        data.time_attack.as_mut().unwrap().end = d::TimeEnd::Cleared;
        data.result = Some(d::MatchResult {
            result_id: [1; 16],
            match_id: 102,
            generation: 103,
            engine_revision: 5,
            mode: 4,
            map: 540,
            finished_at_millis: 10,
            time_attack: None,
            siege: None,
            participants: d::List(vec![
                d::ResultParticipant {
                    entity: 1,
                    identity: data.room.members.0[0].identity.clone(),
                    team: 0,
                    verdict: d::Verdict::Loss,
                },
                d::ResultParticipant {
                    entity: 2,
                    identity: data.room.members.0[1].identity.clone(),
                    team: 0,
                    verdict: d::Verdict::Win,
                },
            ]),
        });
        let own = lines(&data).join("\n");
        assert!(own.contains("本場敗北") && !own.contains("本場勝利") && !own.contains("已保存"));
        data.personal.entity = 2;
        let other = lines(&data).join("\n");
        assert!(
            other.contains("本場勝利") && !other.contains("本場敗北") && !other.contains("已保存")
        );
    }
}
