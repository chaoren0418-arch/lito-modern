use super::super::{
    application_protocol::{CharacterKey, SelectionContext},
    direct_chat_protocol as wire,
    direct_chat_state::Enrollment,
};
use super::*;
fn ready() -> View {
    let context = SelectionContext {
        key: CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: 1,
    };
    let enrollment = Enrollment {
        context,
        channel_id: 0,
        generation: 1,
    };
    let token = Token {
        enrollment,
        room_id: 0,
        membership: 3,
    };
    let mut chat = ChatState::default();
    chat.bind(true, Some(enrollment), Some(token)).unwrap();
    let r = wire::Reply {
        echo: chat.read().unwrap().1,
        status: wire::Status::Ok,
        sequence: 0,
        cursor: 0,
        available: true,
        truncated: false,
        owner: wire::Peer {
            key: context.key,
            character_name: "3333".into(),
            source_name: "太陽神".into(),
        },
        preferences: Preferences::default(),
        blocked: vec![],
        messages: vec![],
    };
    chat.receive(token, r).unwrap();
    View {
        chat,
        filtered_public: false,
        sources: vec![
            SourceChoice {
                id: 7,
                label: "太陽神".into(),
            },
            SourceChoice {
                id: 42,
                label: "月亮".into(),
            },
        ],
        busy: false,
        pending: false,
    }
}
fn target() -> Option<Address> {
    Some(Address {
        source_id: 42,
        name: "  角色3333  ".into(),
    })
}
#[test]
fn private_draft_preserves_raw_target_and_treats_gm_as_literal_without_optimism() {
    let v = ready();
    let mut d = Draft::default();
    d.align(v.chat.token());
    let before = v.chat.clone();
    let i = d
        .intent(&v, Action::Send, target(), None, ".de 20".into())
        .unwrap();
    assert_eq!(i.action, Action::Send);
    assert_eq!(i.text, ".de 20");
    assert_eq!(i.target, target());
    assert_eq!(v.chat, before);
    assert!(!d.ready(&v));
    assert!(d
        .intent(&v, Action::Send, target(), None, "重複".into())
        .is_err());
    d.dispatched(Action::Send, false);
    assert!(d.ready(&v));
}
#[test]
fn private_draft_requires_source_authority_and_valid_complete_text() {
    let v = ready();
    let mut d = Draft::default();
    d.align(v.chat.token());
    for (target, text) in [
        (None, "hello"),
        (
            Some(Address {
                source_id: 99,
                name: "X".into(),
            }),
            "hello",
        ),
        (target(), ""),
        (target(), "bad\nline"),
    ] {
        assert!(d
            .intent(&v, Action::Send, target, None, text.into())
            .is_err());
        assert!(d.ready(&v));
    }
    assert!(d
        .intent(&v, Action::Read, None, None, String::new())
        .is_err());
    assert!(d
        .intent(&v, Action::Preferences, None, None, String::new())
        .is_err());
}
#[test]
fn private_controls_and_drafts_wait_for_matching_confirmation() {
    let mut v = ready();
    let mut d = Draft::default();
    d.align(v.chat.token());
    let prefs = Preferences {
        whispers: false,
        ..Preferences::default()
    };
    let i = d
        .intent(&v, Action::Preferences, None, Some(prefs), String::new())
        .unwrap();
    let req = v
        .chat
        .command(i.token, i.action, i.target, i.preferences, i.text)
        .unwrap();
    d.dispatched(Action::Preferences, true);
    v.pending = true;
    assert!(d.confirmed(&v).is_none());
    assert!(!d.ready(&v));
    let r = wire::Reply {
        echo: req,
        status: wire::Status::Ok,
        sequence: 1,
        cursor: 0,
        available: true,
        truncated: false,
        owner: v.chat.owner().unwrap().clone(),
        preferences: prefs,
        blocked: vec![],
        messages: vec![],
    };
    v.chat.receive(i.token, r).unwrap();
    v.pending = false;
    assert_eq!(
        d.confirmed(&v),
        Some((false, "接收設定已由伺服器確認。"))
    );
    assert!(d.ready(&v));
}
#[test]
fn private_send_clears_only_after_ack_and_denial_keeps_draft() {
    for status in [
        wire::Status::Ok,
        wire::Status::Unreachable,
        wire::Status::RateLimited,
    ] {
        let mut v = ready();
        let mut d = Draft::default();
        d.align(v.chat.token());
        let i = d
            .intent(&v, Action::Send, target(), None, "保留".into())
            .unwrap();
        let req = v
            .chat
            .command(i.token, i.action, i.target, i.preferences, i.text)
            .unwrap();
        d.dispatched(Action::Send, true);
        let delivered = status == wire::Status::Ok;
        let messages = if delivered {
            vec![wire::Message {
                id: 1,
                sender_sequence: 1,
                sender: v.chat.owner().unwrap().clone(),
                recipient: wire::Peer {
                    key: CharacterKey {
                        source_id: 42,
                        object_id: 3333,
                    },
                    character_name: "角色3333".into(),
                    source_name: "月亮".into(),
                },
                text: "保留".into(),
            }]
        } else {
            vec![]
        };
        let r = wire::Reply {
            echo: req,
            status,
            sequence: 1,
            cursor: if delivered { 1 } else { 0 },
            available: true,
            truncated: false,
            owner: v.chat.owner().unwrap().clone(),
            preferences: Preferences::default(),
            blocked: vec![],
            messages,
        };
        v.chat.receive(i.token, r).unwrap();
        assert_eq!(d.confirmed(&v).unwrap().0, delivered);
    }
}
#[test]
fn private_room_or_enrollment_retirement_discards_queued_intent_and_busy_blocks_all_controls() {
    let mut v = ready();
    let mut d = Draft::default();
    d.align(v.chat.token());
    v.busy = true;
    assert!(!d.ready(&v));
    for a in [
        Action::Send,
        Action::Preferences,
        Action::Block,
        Action::Unblock,
        Action::ClearBlocks,
    ] {
        assert!(d.intent(&v, a, None, None, String::new()).is_err());
    }
    v.busy = false;
    d.intent(&v, Action::Block, target(), None, String::new())
        .unwrap();
    assert!(d.align(None));
    assert!(!d.queued && d.pending.is_none());
    assert!(!d.ready(&v));
    let mut t = v.chat.token().unwrap();
    t.enrollment.generation += 1;
    assert!(d.align(Some(t)));
    assert!(!d.ready(&v));
}
#[test]
fn private_layout_is_nonoverlapping_and_in_bounds_across_dpi_and_resize() {
    for dpi in [48, 96, 120, 144, 192, 288, 768] {
        for (w, h) in [
            (240, 180),
            (360, 240),
            (460, 340),
            (719, 480),
            (720, 480),
            (720, 640),
            (1024, 768),
            (1600, 1000),
        ] {
            let w = scale(w, dpi);
            let h = scale(h, dpi);
            let r = layout(w, h, dpi);
            let (w, h) = content_size(w, h, dpi);
            for (i, a) in r.iter().enumerate() {
                let ah = if i == SOURCE { scale(32, dpi) } else { a.h };
                assert!(
                    a.x >= 0 && a.y >= 0 && a.w > 0 && ah > 0 && a.x + a.w <= w && a.y + ah <= h,
                    "{i}"
                );
                for (j, b) in r.iter().enumerate().skip(i + 1) {
                    let bh = if j == SOURCE { scale(32, dpi) } else { b.h };
                    assert!(
                        a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + ah <= b.y || b.y + bh <= a.y,
                        "overlap {i}/{j} dpi={dpi}"
                    );
                }
            }
        }
    }
}
#[test]
fn private_transcript_uses_both_qualified_peers_and_never_public_command_labels() {
    let mut v = ready();
    let t = v.chat.token().unwrap();
    let mut r = wire::Reply {
        echo: v.chat.read().unwrap().1,
        status: wire::Status::Ok,
        sequence: 0,
        cursor: 1,
        available: true,
        truncated: false,
        owner: v.chat.owner().unwrap().clone(),
        preferences: Preferences::default(),
        blocked: vec![],
        messages: vec![],
    };
    r.messages.push(wire::Message {
        id: 1,
        sender_sequence: 1,
        sender: wire::Peer {
            key: CharacterKey {
                source_id: 42,
                object_id: 3333,
            },
            character_name: "3333".into(),
            source_name: "月亮".into(),
        },
        recipient: r.owner.clone(),
        text: ".de 20".into(),
    });
    v.chat.receive(t, r).unwrap();
    assert_eq!(transcript(&v.chat), "3333(月亮) → 3333(太陽神)：.de 20\r\n");
}

#[test]
fn outer_frame_fits_small_high_dpi_work_area_and_negative_monitor_origins() {
    for dpi in [96, 120, 144, 192, 288, 768] {
        for work in [
            BoxRect {
                x: 0,
                y: 0,
                w: 1920,
                h: 1040,
            },
            BoxRect {
                x: -1280,
                y: -120,
                w: 1280,
                h: 680,
            },
            BoxRect {
                x: 100,
                y: 20,
                w: 800,
                h: 560,
            },
            BoxRect {
                x: 0,
                y: 0,
                w: 640,
                h: 440,
            },
        ] {
            for (x, y) in [(-9999, -9999), (0, 0), (9999, 9999)] {
                let r = fit_outer(
                    BoxRect {
                        x,
                        y,
                        w: scale(760, dpi) + 32,
                        h: scale(680, dpi) + 80,
                    },
                    work,
                );
                assert!(
                    r.w > 0
                        && r.h > 0
                        && r.x >= work.x
                        && r.y >= work.y
                        && r.x + r.w <= work.x + work.w
                        && r.y + r.h <= work.y + work.h
                );
                assert_eq!(fit_outer(r, work), r);
            }
        }
    }
}
#[test]
fn tab_reveal_reaches_every_control_on_narrow_and_short_viewports() {
    let mut ids = CONTROL_ORDER;
    ids.sort_unstable();
    assert_eq!(ids, std::array::from_fn(|i| i), "every native control must be created exactly once");
    for dpi in [48, 96, 144, 192, 288, 768] {
        for (vw, vh) in [(240, 180), (360, 240), (460, 340), (720, 480), (960, 500)] {
            let vw = scale(vw, dpi);
            let vh = scale(vh, dpi);
            let (w, h) = content_size(vw, vh, dpi);
            let (mut x, mut y) = (0, 0);
            for i in CONTROL_ORDER {
                let b = layout(vw, vh, dpi)[i];
                let bh = if i == SOURCE { scale(32, dpi) } else { b.h };
                x = reveal_axis(x, b.x, b.w, vw, w);
                y = reveal_axis(y, b.y, bh, vh, h);
                assert!(x >= 0 && x <= w - vw && y >= 0 && y <= h - vh);
                assert!(
                    b.x >= x && b.y >= y && b.x < x + vw && b.y < y + vh,
                    "control origin visible {i}"
                );
                if b.w <= vw {
                    assert!(b.x + b.w <= x + vw);
                }
                if bh <= vh {
                    assert!(b.y + bh <= y + vh);
                }
            }
        }
    }
}
#[test]
fn source_numbers_disambiguate_duplicate_names_without_changing_authority() {
    assert_eq!(
        source_label(&SourceChoice {
            id: 7,
            label: "太陽神".into()
        }),
        "#7 太陽神"
    );
    assert_ne!(
        source_label(&SourceChoice {
            id: 7,
            label: "同名".into()
        }),
        source_label(&SourceChoice {
            id: 42,
            label: "同名".into()
        })
    );
}
#[test]
fn resize_clamps_old_scroll_without_changing_chat_token_or_draft() {
    let v = ready();
    let mut d = Draft::default();
    d.align(v.chat.token());
    let original = v.chat.clone();
    for (vw, vh) in [(360, 240), (1024, 1000), (480, 320), (1920, 1080)] {
        let (w, h) = content_size(vw, vh, 96);
        assert_eq!(
            reveal_axis(i32::MAX / 2, 16, 30, vw, w),
            16.min((w - vw).max(0))
        );
        assert!(reveal_axis(9999, 934, 30, vh, h) <= (h - vh).max(0));
        assert_eq!(v.chat, original);
        assert!(d.ready(&v));
    }
}
#[test]
fn identity_text_changes_only_with_confirmed_owner_or_retirement() {
    let mut v = ready();
    let identity = scope_text(&v.chat);
    assert_eq!(identity, "私訊身分：3333(太陽神)　｜　收件伺服器 / 角色");
    v.busy = true;
    v.pending = true;
    v.sources.reverse();
    assert_eq!(scope_text(&v.chat), identity);
    v.chat.clear();
    assert_eq!(scope_text(&v.chat), "正在確認私訊身分…");
}
#[test]
#[ignore = "explicit local native UI layout inspection only; synthetic private state, no connection/authentication"]
fn private_native_viewport_inspection_without_network() {
    unsafe {
        let mut view = ready();
        view.filtered_public = true;
        let window = DirectChatWindow::open(HWND::default(), view.clone()).unwrap();
        assert!(window.state.controls.get().unwrap().iter().all(|h| IsWindow(Some(*h)).as_bool()));
        let hwnd = window.handle().unwrap();
        set_text(hwnd, "LiTo — 私訊版面檢查（測試資料／無連線）").unwrap();
        let dpi = GetDpiForWindow(hwnd);
        let area = monitor_work(hwnd, None).unwrap();
        let mut r = RECT {
            right: scale(460, dpi),
            bottom: scale(340, dpi),
            ..Default::default()
        };
        AdjustWindowRectExForDpi(&mut r, STYLE, false, EX, dpi).unwrap();
        place(
            hwnd,
            fit_outer(
                BoxRect {
                    x: area.x + 32,
                    y: area.y + 32,
                    w: r.right - r.left,
                    h: r.bottom - r.top,
                },
                area,
            ),
        )
        .unwrap();
        resize(&window.state);
        reveal_focus(&window.state);
        println!("PRIVATE_UI_READY");
        let started = std::time::Instant::now();
        let mut refreshed = started;
        let mut refresh_count = 0usize;
        // Normal application message loop; no injected input or authentication.
        // The external native UI tool may inspect this one owned test window.
        while window.handle().is_some() && started.elapsed() < std::time::Duration::from_secs(180) {
            let mut m = MSG::default();
            while PeekMessageW(&mut m, None, 0, 0, PM_REMOVE).as_bool() {
                if !window.pre_translate(&m) {
                    let _ = TranslateMessage(&m);
                    DispatchMessageW(&m);
                }
            }
            if window.handle().is_some()
                && refreshed.elapsed() >= std::time::Duration::from_millis(100)
            {
                window.update(view.clone()).unwrap();
                refresh_count += 1;
                refreshed = std::time::Instant::now();
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(
            window.handle().is_none(),
            "UI inspection watchdog expired; inspection was not completed"
        );
        println!("PRIVATE_UI_CLOSED refresh_count={refresh_count}");
    }
}
