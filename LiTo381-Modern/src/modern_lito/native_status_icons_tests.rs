use super::*;
use crate::modern_lito::native_world::World;
use crate::modern_lito::native_world_scene::movement_tests::world;

/// Test-only port of `NativeSourceIcons1.packet` (coexist=false, no override).
fn source_icons1(active: &[(u16, i32)], epoch: u32) -> Vec<u8> {
    fn duration(seconds: i32, mode: u8) -> u8 {
        (match mode {
            0 => seconds / 4,
            1 => seconds / 16,
            2 => {
                if seconds <= 0 {
                    0
                } else if seconds < 32 {
                    (seconds / 2).max(1)
                } else {
                    seconds / 4
                }
            }
            3 => seconds.min(4080) / 16,
            4 => seconds.min(1020) / 4,
            _ => unreachable!(),
        }) as u8
    }
    let mut data = [0u8; ICONS1_LEN];
    let seconds = |skill: u16| active.iter().find(|(s, _)| *s == skill).map(|(_, v)| *v);
    for rule in ICONS1_RULES {
        if let Some(s) = seconds(rule.skill) {
            data[rule.slot] = duration(s, rule.mode);
            if rule.slot == 24 {
                data[25] = 3;
            }
        }
    }
    for skill in ICONS1_WEAPONS {
        if let Some(s) = seconds(skill) {
            data[10] = duration(s, 0);
            data[11] = (skill - 1) as u8;
            break;
        }
    }
    data[72..76].copy_from_slice(&epoch.to_le_bytes());
    let mut b = vec![S_EVENT, ICONS1];
    b.extend((ICONS1_LEN as u32).to_le_bytes());
    b.extend(data);
    b
}
fn mode_of(skill: u16) -> u8 {
    ICONS1_RULES.iter().find(|r| r.skill == skill).map(|r| r.mode).unwrap_or(0)
}
fn shield(seconds: u16, kind: u8) -> Vec<u8> {
    let mut b = vec![S_MAGE_SHIELD];
    b.extend(seconds.to_le_bytes());
    b.push(kind);
    b
}
fn event(kind: u8, seconds: u16) -> Vec<u8> {
    let mut b = vec![S_EVENT, kind];
    b.extend(seconds.to_le_bytes());
    b
}
fn speed(id: u32, mode: u8, seconds: u16) -> Vec<u8> {
    let mut b = vec![0x29];
    b.extend(id.to_le_bytes());
    b.push(mode);
    b.extend(seconds.to_le_bytes());
    b
}
fn own(w: &World) -> u32 {
    w.status.unwrap().object_id
}

#[test]
fn icons1_layout_is_the_source_36_state_table() {
    let skills = icons1_skills();
    assert_eq!(skills.len(), 36);
    assert!(skills.contains(&1017) && skills.contains(&181) && skills.contains(&155));
    let slots: BTreeSet<_> = ICONS1_RULES.iter().map(|r| r.slot).collect();
    assert_eq!(slots.len(), ICONS1_RULES.len(), "one slot per rule");
    for reserved in [10, 11, 25, 72, 73, 74, 75] {
        assert!(!slots.contains(&reserved), "slot {reserved} is shared metadata");
    }
    assert!(slots.iter().all(|s| *s < 72));
    // Only 155 is both a slot rule and a shared-weapon skill.
    let both: Vec<_> = ICONS1_WEAPONS.iter().filter(|w| ICONS1_RULES.iter().any(|r| r.skill == **w)).collect();
    assert_eq!(both, vec![&155]);
}

#[test]
fn every_icons1_state_round_trips_with_an_exact_floor() {
    for skill in icons1_skills() {
        for seconds in [1, 3, 20, 31, 32, 100, 600, 1020] {
            let packet = source_icons1(&[(skill, seconds)], 0x1234_5678);
            assert_eq!(packet.len(), 210);
            let decoded = decode_icons1(&packet).unwrap();
            assert_eq!(decoded.epoch_seconds, 0x1234_5678);
            let mode = mode_of(skill);
            if decoded.entries.is_empty() {
                // Only sub-unit durations encode as zero (mode0/4 <4 s, mode1/3 <16 s).
                assert!(matches!((mode, seconds), (0 | 4, 1..=3) | (1 | 3, 1..=15)), "{skill} {seconds}");
                continue;
            }
            assert_eq!(decoded.entries.len(), 1, "{skill}");
            let e = decoded.entries[0];
            assert_eq!(e.skill, skill);
            assert!(e.seconds as i32 <= seconds, "{skill} floor {} > {seconds}", e.seconds);
            if mode == 2 && (32..64).contains(&seconds) {
                // raw 8..15 is also a valid <32 s encoding: only the safe floor is known.
                assert_eq!(e.seconds, u32::from(e.raw) * 2);
                continue;
            }
            // Mode 2 raw 1 covers 1..=3 seconds.
            let step = match mode { 1 | 3 => 16, 2 if seconds < 32 => 3, _ => 4 };
            assert!(seconds - (e.seconds as i32) < step, "{skill} {seconds} -> {}", e.seconds);
        }
    }
}

#[test]
fn icons1_shared_weapon_marker_and_full_snapshot() {
    let all: Vec<_> = icons1_skills().into_iter().filter(|s| ![148, 149, 156, 166, 163].contains(s)).map(|s| (s, 400)).collect();
    let decoded = decode_icons1(&source_icons1(&all, 0)).unwrap();
    assert_eq!(decoded.entries.len(), 31);
    let packet = source_icons1(&[(163, 120), (133, 40)], 0);
    assert_eq!(packet[6 + 11], 162);
    assert_eq!(packet[6 + 25], 3);
    let d = decode_icons1(&packet).unwrap();
    assert_eq!(d.entries.iter().map(|e| (e.skill, e.seconds)).collect::<Vec<_>>(), vec![(133, 40), (163, 120)]);
    // 155 owns both slot 27 and the weapon slot: one state, not two.
    let d = decode_icons1(&source_icons1(&[(155, 80)], 0)).unwrap();
    assert_eq!(d.entries, vec![Icons1Entry { skill: 155, raw: 20, seconds: 80 }]);
}

#[test]
fn icons1_rejects_anything_outside_the_source_layout() {
    let good = source_icons1(&[(32, 40)], 0);
    let mut bad = good.clone();
    bad[6 + 2] = 1;
    assert!(decode_icons1(&bad).is_err(), "unassigned slot");
    let mut bad = good.clone();
    bad[6 + 25] = 2;
    assert!(decode_icons1(&bad).is_err(), "marker");
    let mut bad = source_icons1(&[(133, 40)], 0);
    bad[6 + 25] = 0;
    assert!(decode_icons1(&bad).is_err(), "slot24 without marker");
    let mut bad = good.clone();
    bad[6 + 10] = 5;
    bad[6 + 11] = 7;
    assert!(decode_icons1(&bad).is_err(), "weapon id");
    let mut bad = good.clone();
    bad[2] = 203;
    assert!(decode_icons1(&bad).is_err(), "declared length");
    assert!(decode_icons1(&good[..209]).is_err());
    let mut longer = good.clone();
    longer.push(0);
    assert!(decode_icons1(&longer).is_err());
    assert!(seconds_floor(1, 9).is_err());
}

#[test]
fn world_icons1_replaces_the_previous_snapshot_and_scene_clears() {
    let mut w = world();
    let before = w.visual_revision;
    w.accept_at(&source_icons1(&[(32, 400), (206, 160), (1017, 80)], 9), 1000, 1000).unwrap();
    assert!(w.visual_revision > before);
    let keys: Vec<_> = w.status_icons.views(1000).into_iter().map(|v| v.key).collect();
    assert_eq!(keys, vec![Key::Icons1(32), Key::Icons1(206), Key::Icons1(1017)]);
    assert_eq!(w.status_icons.icons1_epoch, Some(9));
    assert_eq!(label(Key::Icons1(1017), 0), "果實勇敢");
    w.accept_at(&source_icons1(&[(206, 160)], 10), 2000, 2000).unwrap();
    assert_eq!(w.status_icons.views(2000).into_iter().map(|v| v.key).collect::<Vec<_>>(), vec![Key::Icons1(206)]);
    // Non-status 0x7B subtypes stay with their own owners.
    assert!(w.status_icons.accept_event(&[S_EVENT, 21, 0, 0], 0).is_err());
    w.accept_at(&[0x28, 4, 0, 0], 3000, 3000).unwrap();
    assert!(w.status_icons.is_empty());
}

#[test]
fn dragon_skin_uses_shared_icon_3080_and_native_refresh_remove_refuse() {
    let mut w = world();
    w.accept_at(&shield(1200, DRAGON_SKIN), 5000, 5000).unwrap();
    let v = w.status_icons.views(5000);
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].key, Key::Shield(181));
    assert_eq!(v[0].icon, Some(3080));
    assert_ne!(v[0].icon, Some(3079));
    assert_eq!(v[0].label, "龍之護鎧");
    assert_eq!(v[0].remaining_seconds, Some(1200));
    assert_eq!(w.status_icons.views(5001)[0].remaining_seconds, Some(1200));
    assert_eq!(w.status_icons.views(6000)[0].remaining_seconds, Some(1199));
    // Refresh keeps a single entry.
    w.accept_at(&shield(1200, DRAGON_SKIN), 7000, 7000).unwrap();
    assert_eq!(w.status_icons.len(), 1);
    assert_eq!(w.status_icons.get(Key::Shield(181)).unwrap().received_ms, 7000);
    // Native refuses >32767 without touching the live lease.
    let revision = w.status_icons.revision;
    w.accept_at(&shield(32768, DRAGON_SKIN), 8000, 8000).unwrap();
    assert_eq!(w.status_icons.refused, 1);
    assert_eq!(w.status_icons.revision, revision);
    assert_eq!(w.status_icons.get(Key::Shield(181)).unwrap().seconds, Some(1200));
    w.accept_at(&shield(0, DRAGON_SKIN), 9000, 9000).unwrap();
    assert!(w.status_icons.is_empty());
    assert!(w.accept_at(&[S_MAGE_SHIELD, 1, 0], 9000, 9000).is_err());
    assert!(w.accept_at(&[S_MAGE_SHIELD, 1, 0, 181, 0], 9000, 9000).is_err());
}

#[test]
fn nine_illusion_leases_follow_server_duration_and_cancel() {
    let mut w = world();
    for (i, skill) in ILLUSION_SKILLS.iter().enumerate() {
        w.accept_at(&shield(64, *skill), 100, 100).unwrap();
        assert_eq!(shield_icon(*skill), Some(ILLUSION_ICONS[i]));
    }
    let views = w.status_icons.views(100);
    assert_eq!(views.len(), 9);
    for (v, (skill, icon)) in views.iter().zip(ILLUSION_SKILLS.iter().zip(ILLUSION_ICONS)) {
        assert_eq!(v.key, Key::Shield(*skill));
        assert_eq!(v.icon, Some(icon));
        assert_eq!(v.label, crate::modern_lito::native_skill_selection::catalog::name(u16::from(*skill)).unwrap());
        assert_eq!(v.remaining_seconds, Some(64));
    }
    // Server cancel (zero) for one, natural expiry for the rest.
    w.accept_at(&shield(0, 204), 200, 200).unwrap();
    assert_eq!(w.status_icons.len(), 8);
    assert!(!w.poll_status_icons(64_099));
    assert!(w.poll_status_icons(64_100));
    assert!(w.status_icons.is_empty());
    // Unmapped legacy types stay text-only; 5/6 keep their shared names.
    assert_eq!(shield_icon(5), None);
    assert_eq!(label(Key::Shield(5), 0), "護盾狀態（SHIELD）");
    assert_eq!(label(Key::Shield(99), 0), "狀態（類型 99）");
}

#[test]
fn icon_i2h_and_doll_are_word_leases() {
    let mut w = world();
    w.accept_at(&event(ICON_I2H, 300), 0, 0).unwrap();
    w.accept_at(&event(DOLL, 1800), 0, 0).unwrap();
    assert_eq!(w.status_icons.get(Key::ImmuneToHarm).unwrap().seconds, Some(300));
    assert_eq!(w.status_icons.get(Key::Doll).unwrap().seconds, Some(1800));
    assert_eq!(label(Key::ImmuneToHarm, 0), "聖結界");
    w.accept_at(&event(ICON_I2H, 0), 1, 1).unwrap();
    assert!(w.status_icons.get(Key::ImmuneToHarm).is_none());
    assert!(w.accept_at(&[S_EVENT, ICON_I2H, 1], 1, 1).is_err());
    assert!(w.accept_at(&[S_EVENT, DOLL, 1, 0, 0], 1, 1).is_err());
}

#[test]
fn own_speed_keeps_the_full_write_h_duration_and_expires_locally() {
    let mut w = world();
    let id = own(&w);
    // 300 s and doll29 65535 s do not fit a byte; the word is kept intact.
    w.accept_at(&speed(id, 2, 300), 1000, 1000).unwrap();
    assert_eq!(w.actors[&id].speed_seconds, 300);
    assert_eq!(w.status_icons.get(Key::Speed).unwrap().seconds, Some(300));
    assert_eq!(label(Key::Speed, 2), "緩速");
    w.accept_at(&speed(id, 2, 0xffff), 2000, 2000).unwrap();
    assert_eq!(w.actors[&id].speed_seconds, 0xffff);
    assert_eq!(w.status_icons.views(2000)[0].remaining_seconds, Some(65535));
    w.accept_at(&speed(id, 1, 300), 3000, 3000).unwrap();
    assert!(!w.poll_status_icons(302_999));
    assert_eq!(w.actors[&id].speed, 1);
    assert!(w.poll_status_icons(303_000));
    assert_eq!(w.actors[&id].speed, 0);
    assert!(w.status_icons.get(Key::Speed).is_none());
    // Zero with a mode is permanent until the server sends normal.
    w.accept_at(&speed(id, 2, 0), 4000, 4000).unwrap();
    assert!(!w.poll_status_icons(u64::MAX));
    assert_eq!(w.status_icons.views(u64::MAX)[0].remaining_seconds, None);
    w.accept_at(&speed(id, 0, 0), 5000, 5000).unwrap();
    assert!(w.status_icons.get(Key::Speed).is_none());
    // Observers' speed does not create a local timer.
    w.accept_at(&speed(id.wrapping_add(7777), 1, 30), 6000, 6000).unwrap();
    assert!(w.status_icons.get(Key::Speed).is_none());
}

#[test]
fn mp_notifies_only_on_real_change() {
    let mut w = world();
    let base = w.mp_notifications;
    let mana = |c: u16, m: u16| {
        let mut b = vec![0x4d];
        b.extend(c.to_le_bytes());
        b.extend(m.to_le_bytes());
        b
    };
    let current = w.mp.unwrap();
    w.accept(&mana(current[0], current[1])).unwrap();
    assert_eq!(w.mp_notifications, base, "same value is silent");
    w.accept(&mana(current[0].wrapping_add(1), current[1].max(current[0].wrapping_add(1)))).unwrap();
    assert_eq!(w.mp_notifications, base + 1);
    let changed = w.mp.unwrap();
    w.accept(&mana(changed[0], changed[1])).unwrap();
    w.accept(&mana(changed[0], changed[1])).unwrap();
    assert_eq!(w.mp_notifications, base + 1);
}

#[test]
fn summary_shows_ac_mp_cooldown_and_leases() {
    let mut w = world();
    w.mp = Some([30, 50]);
    w.accept_at(&shield(1200, DRAGON_SKIN), 0, 0).unwrap();
    let s = summary(&w, 0, Some(1480));
    let ac = w.status.unwrap().armor_class;
    assert!(s.starts_with(&format!("AC {ac}　MP 30/50　冷卻 1.4 秒　")), "{s}");
    assert!(s.contains("龍之護鎧〔圖示 3080〕 1200 秒"), "{s}");
    let s = summary(&World::default(), 0, None);
    assert_eq!(s, "AC —　MP 等待資料　技能就緒　狀態：無");
    for skill in ILLUSION_SKILLS {
        w.accept_at(&shield(10, skill), 0, 0).unwrap();
    }
    assert!(summary(&w, 0, None).ends_with("等 10 項"));
}
