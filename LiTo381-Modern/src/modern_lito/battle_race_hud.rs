//! Canonical ranked race presentation. Zero finish time is NOT a finish;
//! no local timer, inferred team victory, persistence receipt or legacy Frame.
use crate::modern_lito::battle_payload::data as d;

fn remaining(deadline: i64, observed: i64) -> String {
    let seconds = deadline
        .saturating_sub(observed)
        .max(0)
        .saturating_add(999_999_999)
        / 1_000_000_000;
    format!("{:02}:{:02}", seconds / 60, seconds % 60)
}
fn finish(nanos: i64) -> String {
    let millis = nanos.max(0) / 1_000_000;
    format!(
        "{}:{:02}.{:03}",
        millis / 60_000,
        millis / 1000 % 60,
        millis % 1000
    )
}
pub(super) fn lines(data: &d::UnvalidatedData) -> Vec<String> {
    let Some(r) = &data.race else {
        return Vec::new();
    };
    let phase = match r.phase {
        d::RacePhase::Waiting => "等待起跑",
        d::RacePhase::Running => "競賽中",
        d::RacePhase::FinishWindow => "等待其他選手完賽",
        d::RacePhase::Complete => "排名已確認",
        d::RacePhase::Aborted => "本場中止，沒有排名結果",
    };
    let mut out = vec![format!(
        "寵物競速 · {phase} · 完賽 {}/{}",
        r.runners.0.iter().filter(|v| v.placement > 0).count(),
        r.runners.0.len()
    )];
    if let Some(v) = r
        .runners
        .0
        .iter()
        .find(|v| v.entity == data.personal.entity)
    {
        let text = if v.placement > 0 {
            format!(
                "你的名次：第 {} 名 · 完賽用時 {}",
                v.placement,
                finish(v.finish_elapsed_nanos)
            )
        } else if v.dnf != d::Dnf::None {
            format!(
                "你未完賽：{} · 已完成 {}/4 圈",
                match v.dnf {
                    d::Dnf::Left => "已離場",
                    d::Dnf::Disconnected => "連線中斷",
                    d::Dnf::WindowClosed => "完賽窗口已關閉",
                    d::Dnf::None => unreachable!(),
                },
                v.completed_laps
            )
        } else {
            format!(
                "已完成 {}/4 圈 · 下一檢查點 {}/{}",
                v.completed_laps,
                v.next_checkpoint + 1,
                r.gates.0.len()
            )
        };
        out.push(text);
        if v.placement == 0
            && v.dnf == d::Dnf::None
            && matches!(
                r.phase,
                d::RacePhase::Waiting | d::RacePhase::Running | d::RacePhase::FinishWindow
            )
        {
            if let Some(g) = usize::try_from(v.next_checkpoint)
                .ok()
                .and_then(|i| r.gates.0.get(i))
            {
                out.push(format!(
                    "下一檢查區域：X {}–{} · Y {}–{}",
                    g.min_x, g.max_x, g.min_y, g.max_y
                ));
            }
        }
    }
    match r.phase {
        d::RacePhase::Waiting | d::RacePhase::Running => out.push(format!(
            "{} {}",
            if r.phase == d::RacePhase::Waiting {
                "時限"
            } else {
                "伺服器剩餘"
            },
            remaining(r.active_limit_nanos, r.observed_elapsed_nanos)
        )),
        d::RacePhase::FinishWindow => out.push(format!(
            "伺服器完賽窗口剩餘 {}",
            remaining(
                r.finish_window_deadline_elapsed_nanos,
                r.observed_elapsed_nanos
            )
        )),
        d::RacePhase::Complete => {
            out.push("競速排名不計入戰鬥勝敗；保存狀態另依伺服器回覆。".into())
        }
        d::RacePhase::Aborted => {}
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    // Presentation-only values, not a substitute for canonical validation.
    fn fixture() -> d::UnvalidatedData {
        let mut x = crate::modern_lito::battle_payload::data_tests::fixture();
        x.race = Some(d::Race {
            revision: 0,
            phase: d::RacePhase::Waiting,
            course_sha256: [1; 32],
            gates: d::List(vec![d::Gate {
                min_x: 1,
                max_x: 2,
                min_y: 3,
                max_y: 4,
            }]),
            active_limit_nanos: 120_000_000_000,
            observed_elapsed_nanos: 0,
            first_finisher_entity: 0,
            finish_window_deadline_elapsed_nanos: 0,
            terminal_revision: 0,
            result_id: None,
            runners: d::List(
                (1..=2)
                    .map(|entity| d::Runner {
                        entity,
                        revision: 0,
                        progress_move_sequence: 0,
                        completed_laps: 0,
                        next_checkpoint: 0,
                        placement: 0,
                        finish_elapsed_nanos: 0,
                        dnf: d::Dnf::None,
                        base_graphics: 2145,
                    })
                    .collect(),
            ),
        });
        x
    }
    #[test]
    fn zero_finish_times_do_not_count_as_finished_and_checkpoint_is_one_based() {
        let x = fixture();
        let text = lines(&x).join("\n");
        assert!(text.contains("等待起跑 · 完賽 0/2"));
        assert!(text.contains("已完成 0/4 圈 · 下一檢查點 1/1"));
        assert!(text.contains("X 1–2 · Y 3–4") && text.contains("時限 02:00"));
        assert!(!text.contains("名次") && !text.contains("已保存"));
    }
    #[test]
    fn personal_placement_not_other_runner_or_team_drives_rank() {
        let mut x = fixture();
        let r = x.race.as_mut().unwrap();
        r.phase = d::RacePhase::FinishWindow;
        r.runners.0[1].placement = 1;
        r.runners.0[1].completed_laps = 4;
        r.runners.0[1].finish_elapsed_nanos = 61_234_567_890;
        r.observed_elapsed_nanos = 61_234_567_890;
        r.finish_window_deadline_elapsed_nanos = 71_234_567_890;
        let text = lines(&x).join("\n");
        assert!(text.contains("完賽 1/2") && text.contains("已完成 0/4 圈"));
        assert!(!text.contains("你的名次"));
        x.personal.entity = 2;
        let text = lines(&x).join("\n");
        assert!(text.contains("第 1 名 · 完賽用時 1:01.234"));
        assert!(text.contains("伺服器完賽窗口剩餘 00:10"));
        assert!(!text.contains("下一檢查"));
    }
    #[test]
    fn clock_zero_cannot_declare_dnf_or_saved_and_terminal_uses_server_reason() {
        let mut x = fixture();
        let r = x.race.as_mut().unwrap();
        r.phase = d::RacePhase::FinishWindow;
        r.observed_elapsed_nanos = 21_000_000_000;
        r.finish_window_deadline_elapsed_nanos = 20_000_000_000;
        let text = lines(&x).join("\n");
        assert!(text.contains("窗口剩餘 00:00") && !text.contains("你未完賽"));
        for (reason, label) in [
            (d::Dnf::Left, "已離場"),
            (d::Dnf::Disconnected, "連線中斷"),
            (d::Dnf::WindowClosed, "完賽窗口已關閉"),
        ] {
            let r = x.race.as_mut().unwrap();
            r.phase = d::RacePhase::Complete;
            r.runners.0[0].dnf = reason;
            let text = lines(&x).join("\n");
            assert!(text.contains(label) && text.contains("不計入戰鬥勝敗"));
            assert!(!text.contains("已保存") && !text.contains("下一檢查"));
        }
        x.race.as_mut().unwrap().phase = d::RacePhase::Aborted;
        assert!(lines(&x)[0].contains("沒有排名結果"));
        x.race = None;
        assert!(lines(&x).is_empty());
    }
}
