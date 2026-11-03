use super::*;
use crate::modern_lito::{battle_state::Tracker, protocol, world_protocol as legacy};

fn context() -> app::SelectionContext {
    app::SelectionContext {
        key: app::CharacterKey {
            source_id: 7,
            object_id: 3333,
        },
        epoch: 9,
    }
}
fn ack() -> protocol::Ack {
    protocol::Ack {
        request_id: 1,
        version: 39,
        numeric_schema: 1,
        capabilities: protocol::KNOWN_CAPABILITIES,
    }
}
pub(crate) fn detached(context: app::SelectionContext, channel: u8) -> State {
    let mut tracker = Tracker::default();
    tracker
        .accept_world(
            &world::Reply {
                echo: world::Request::read(context, channel, None),
                status: legacy::Status::Ok,
                reason: legacy::Reason::None,
                state: world::State::Detached,
                attachment: None,
                data: None,
            },
            ack(),
        )
        .unwrap();
    State {
        supported: true,
        revival_supported: false,
        equipment_supported: false,
        greater_haste_supported: false,
        target_haste_supported: false,
        slow_supported: false,
        player_healing_supported: false,
        vip_world_supported: false,
        vip_equipment_supported: false,
        snapshot: tracker.snapshot().cloned(),
        receipt: None,
    }
}
pub(crate) fn received(mut state: State, command: Command, epoch: u64) -> State {
    state.receipt = Some(Arc::new(Receipt {
        epoch,
        command,
        outcome: Outcome::World(legacy::Status::Ok, legacy::Reason::None),
        equipment: None,
        vip_equipment: None,
    }));
    state
}
pub(crate) fn preparing(context: app::SelectionContext, channel: u8) -> State {
    let mut tracker = Tracker::default();
    tracker
        .accept_world(
            &world::Reply {
                echo: world::Request::read(context, channel, None),
                status: legacy::Status::Ok,
                reason: legacy::Reason::None,
                state: world::State::BeforeEngine,
                attachment: Some(world::Attachment {
                    world: crate::modern_lito::battle_payload::data::WorldId {
                        room: 12,
                        match_id: 1,
                        generation: 24,
                    },
                    lifecycle: operation::Lifecycle {
                        revision: 1,
                        phase: legacy::Phase::Preparing,
                        abort: legacy::AbortReason::None,
                        settlement: None,
                        race_settlement: None,
                        race_terminal_revision: 0,
                    },
                }),
                data: None,
            },
            ack(),
        )
        .unwrap();
    State {
        snapshot: tracker.snapshot().cloned(),
        ..detached(context, channel)
    }
}

pub(crate) fn vip_inspection(
    context: app::SelectionContext,
    channel: u8,
    observation: Observation,
) -> State {
    State {
        supported: true,
        vip_world_supported: true,
        snapshot: Some(super::super::battle_state::Snapshot::vip_inspection_fixture(
            context,
            channel,
            observation,
        )),
        ..State::default()
    }
}

#[test]
fn exact_equipment_denial_is_not_replaced_by_generic_command_notice() {
    use crate::modern_lito::equipment as e;
    let q = e::Request {
        action: e::Action::Set,
        item_id: 1,
        desired: true,
        ..e::Request::read(context())
    };
    let app::Response::Equipment(mut reply) =
        app::decode_response(&e::tests::fixture(2, q, e::Status::Denied, false))
            .unwrap()
            .response
    else {
        panic!()
    };
    reply.reason = Some(e::Reason::SlotOccupied);
    reply.blocking_item = 2;
    reply.validate().unwrap();
    let mut s = detached(context(), 2);
    s.receipt = Some(Arc::new(Receipt {
        epoch: 10,
        command: Command::world(world::Request::read(context(), 2, None)),
        outcome: Outcome::Equipment(
            super::super::battle_equipment::Reason::None,
            super::super::battle_equipment::Disposition::NotCommitted,
            Some(e::Status::Denied),
        ),
        equipment: Some(Arc::new(reply)),
        vip_equipment: None,
    }));
    assert!(s.command_status().contains("欄位已被其他物品佔用"));
    assert!(s.command_status().contains("阻擋物件 ID：2"));
    assert!(s.command_status().contains("本次未提交"));
    let shown = s.display_status(false, Some("音效載入失敗"));
    assert!(shown.contains("欄位已被其他物品佔用"));
    assert!(shown.contains("阻擋物件 ID：2"));
    assert!(shown.contains("音效載入失敗"));
    let pending = s.display_status(true, Some("正在取消路徑…"));
    assert!(pending.contains("尚未取得本次結果"));
    assert!(pending.contains("正在取消路徑"));
    assert!(!pending.contains("欄位已被其他物品佔用"));
    s.retain(None);
    assert!(!s.command_status().contains("阻擋物件"));
}
fn navigation() -> app::NavigationReply {
    app::NavigationReply {
        action: app::NavigationAction::EnterChannel,
        status: app::NavigationStatus::Ok,
        context: context(),
        snapshot: app::NavigationSnapshot {
            phase: app::NavigationPhase::Lobby,
            channel_id: Some(2),
            selected: None,
            channels: Vec::new(),
        },
    }
}
fn directory() -> rooms::Reply {
    rooms::Reply {
        common: rooms::Common::read(context(), 2),
        status: rooms::Status::Ok,
        reason: rooms::Reason::None,
        list_revision: 1,
        rooms: Vec::new(),
        options: vec![rooms::ModeOption {
            mode: 1,
            map_id: 514,
            team_based: true,
            world_available: true,
            minimum_players: 2,
            maximum_players: 100,
            seconds: 60,
            mode_name: "unit".into(),
        }],
        joined: Some(rooms::RoomSnapshot {
            owner: context().key,
            members: Vec::new(),
            summary: rooms::Summary {
                room_id: 12,
                revision: 23,
                users: 2,
                ready: 2,
                satisfied: true,
                settings: rooms::PublicDraft {
                    capacity: 2,
                    mode: 1,
                    attributes: 16,
                    map_id: 514,
                    name: "unit".into(),
                },
            },
        }),
    }
}
#[test]
fn unknown_and_hidden_state_are_not_confirmed_departures() {
    let default = State::default();
    assert!(!default.supported() && !default.attached());
    assert_eq!(default.notice(), "尚未確認世界附著。");
    let mut state = detached(context(), 2);
    assert_eq!(state.notice(), "已確認返回房間；世界附著已清理。");
    state.hide();
    assert_eq!(state.notice(), default.notice());
}

#[test]
fn equipment_and_local_expiry_notices_do_not_claim_a_live_commit() {
    use crate::modern_lito::{battle_equipment as g, equipment as e};
    for (outcome, words) in [
        (Outcome::NotSent, "沒有送出"),
        (
            Outcome::Equipment(
                g::Reason::None,
                g::Disposition::PersistedRetired,
                Some(e::Status::BackendUnavailable),
            ),
            "未套用即時戰鬥",
        ),
        (
            Outcome::Equipment(
                g::Reason::None,
                g::Disposition::NotCommitted,
                Some(e::Status::Conflict),
            ),
            "本次未提交",
        ),
        (
            Outcome::Equipment(g::Reason::Dead, g::Disposition::NotCommitted, None),
            "沒有穿脫",
        ),
    ] {
        let mut s = detached(context(), 2);
        s.receipt = Some(Arc::new(Receipt {
            epoch: 1,
            command: Command::world(world::Request::read(context(), 2, None)),
            outcome,
            equipment: None,
            vip_equipment: None,
        }));
        let notice = s.command_notice();
        assert!(notice.contains(words), "{notice}");
    }
}
#[test]
fn exact_receipt_not_later_epoch_or_other_command_is_required() {
    let q = Command::world(world::Request::read(context(), 2, None));
    let before = detached(context(), 2);
    assert!(!before.confirmed(&q, 12));
    let mut state = received(before, q.clone(), 12);
    assert!(state.confirmed(&q, 12));
    assert!(!state.confirmed(&q, 11) && !state.confirmed(&q, 13));
    assert_eq!(state.receipt_epoch(), Some(12));
    assert_eq!(state.received_after(&q, Some(11)), state.outcome());
    assert!(state.received_after(&q, Some(12)).is_none());
    assert!(state.received_after(&q, Some(13)).is_none());
    let other = Command::world(world::Request {
        action: world::Action::Prepare,
        room: 12,
        room_revision: 23,
        ..world::Request::read(context(), 2, None)
    });
    assert!(!state.confirmed(&other, 12));
    assert!(state.received_after(&other, None).is_none());
    state.snapshot = detached(context(), 2).snapshot;
    assert!(
        state.confirmed(&q, 12),
        "passive replacement keeps only the completed receipt"
    );
    state.retain(Some((context(), 1)));
    assert!(!state.confirmed(&q, 12) && state.snapshot().is_none());
}

#[test]
fn vip_family_is_explicit_and_retired_worlds_cannot_reenter_queue() {
    let old = detached(context(), 2);
    let mut vip = old.clone();
    vip.vip_world_supported = true;
    vip.vip_equipment_supported = true;
    let read = world::Request::read(context(), 2, None);
    let ordinary = Command::world(read.clone());
    let dedicated = Command::vip_world(read.clone());
    assert!(ordinary.current(&old, Some(&navigation())));
    assert!(!ordinary.current(&vip, Some(&navigation())));
    assert!(!dedicated.current(&old, Some(&navigation())));
    assert!(dedicated.current(&vip, Some(&navigation())));
    assert_ne!(old, vip);
    let retired = world::Request {
        room: 12,
        match_id: 1,
        generation: 24,
        ..read
    };
    assert!(!Command::vip_world(retired.clone()).current(&vip, Some(&navigation())));
    assert!(!Command::world(retired).current(&old, Some(&navigation())));
    vip.retain(None);
    assert!(vip.vip_observation().is_none() && vip.vip_quote().is_none());
}
#[test]
fn navigation_and_detached_state_cannot_admit_loaded_or_foreign_commands() {
    let s = detached(context(), 2);
    let nav = navigation();
    let q = Command::world(world::Request::read(context(), 2, None));
    assert!(q.current(&s, Some(&nav)) && q.validate().is_ok());
    assert!(!q.current(&State::default(), Some(&nav)));
    for i in 0..4 {
        let mut bad = nav.clone();
        match i {
            0 => bad.context.epoch += 1,
            1 => bad.context.key.source_id += 1,
            2 => bad.snapshot.channel_id = Some(1),
            _ => bad.snapshot.phase = app::NavigationPhase::Channels,
        }
        assert!(!q.current(&s, Some(&bad)));
    }
    for action in [world::Action::Loaded, world::Action::Leave] {
        let q = Command::world(world::Request {
            action,
            room: 12,
            match_id: 7,
            generation: 24,
            ..world::Request::read(context(), 2, None)
        });
        assert!(!q.current(&s, Some(&nav)));
    }
}
#[test]
fn poller_requires_current_owner_ready_room_and_available_world() {
    let s = detached(context(), 2);
    let nav = navigation();
    let room = directory();
    let poller = Poller::default();
    let now = Instant::now();
    assert_eq!(
        poller.request(&s, &nav, Some(&room), now).unwrap().action,
        world::Action::Prepare
    );
    assert!(poller.request(&s, &nav, None, now).is_none());
    for i in 0..5 {
        let mut bad = room.clone();
        match i {
            0 => bad.joined.as_mut().unwrap().owner.source_id += 1,
            1 => bad.joined.as_mut().unwrap().summary.satisfied = false,
            2 => bad.options[0].world_available = false,
            3 => bad.common.context.epoch += 1,
            _ => bad.common.channel_id = 1,
        }
        assert!(poller
            .request(&s, &nav, Some(&bad), now)
            .is_none_or(|q| q.action == world::Action::Read));
    }
}

#[test]
fn canonical_poll_eligibility_retires_outside_the_selected_joined_lobby() {
    let state = detached(context(), 2);
    let nav = navigation();
    let directory = directory();
    let now = Instant::now();
    let poller = Poller::default();
    assert!(poller
        .request(&state, &nav, Some(&directory), now)
        .is_some());
    let mut unsupported = state.clone();
    unsupported.supported = false;
    assert!(poller
        .request(&unsupported, &nav, Some(&directory), now)
        .is_none());
    for i in 0..4 {
        let mut next = nav.clone();
        match i {
            0 => next.snapshot.phase = app::NavigationPhase::Channels,
            1 => next.snapshot.channel_id = None,
            2 => next.context.epoch += 1,
            _ => next.context.key.source_id += 1,
        }
        assert!(poller
            .request(&state, &next, Some(&directory), now)
            .is_none());
    }
    let mut left = directory.clone();
    left.joined = None;
    assert!(poller.request(&state, &nav, Some(&left), now).is_none());
}
#[test]
fn only_explicit_unattached_capacity_denial_allows_timed_prepare_retry() {
    let s = detached(context(), 2);
    let nav = navigation();
    let room = directory();
    let now = Instant::now();
    let mut poller = Poller::default();
    let q = poller.request(&s, &nav, Some(&room), now).unwrap();
    let mut reply = world::Reply {
        echo: q,
        status: legacy::Status::Busy,
        reason: legacy::Reason::NotReady,
        state: world::State::Detached,
        attachment: None,
        data: None,
    };
    poller.receive(&reply, now);
    assert_eq!(
        poller
            .request(&s, &nav, Some(&room), now + Duration::from_secs(3))
            .unwrap()
            .action,
        world::Action::Read
    );
    reply.reason = legacy::Reason::Capacity;
    poller.receive(&reply, now);
    assert_eq!(
        poller
            .request(&s, &nav, Some(&room), now + Duration::from_millis(1999))
            .unwrap()
            .action,
        world::Action::Read
    );
    assert_eq!(
        poller
            .request(&s, &nav, Some(&room), now + Duration::from_secs(2))
            .unwrap()
            .action,
        world::Action::Prepare
    );
}
#[test]
fn operation_encoder_rejects_read_disguised_as_movement() {
    let guard = Guard {
        stamp: crate::modern_lito::battle_window::Stamp {
            context: context(),
            channel: 2,
            world: [12, 7, 24],
            entity: 1,
            life: 1,
            revision: 2,
        },
        target: None,
    };
    let mut q = legacy::Request {
        match_id: 7,
        generation: 24,
        ..legacy::Request::read(context(), 2)
    };
    assert!(Command::operation(guard, Operation::Move(q))
        .validate()
        .is_err());
    q.action = legacy::Action::Move;
    q.sequence = 1;
    assert!(Command::operation(guard, Operation::Move(q))
        .validate()
        .is_ok());
    q.context.epoch += 1;
    assert!(Command::operation(guard, Operation::Move(q))
        .validate()
        .is_err());
}

#[test]
fn rejected_action_notices_explain_cadence_and_geometry_without_claiming_success() {
    let q = Command::world(world::Request::read(context(), 2, None));
    for (reason, expected) in [
        (operation::Reason::TooFast, "動作間隔"),
        (operation::Reason::Wait, "動作間隔"),
        (operation::Reason::StalePosition, "位置或狀態"),
        (operation::Reason::OutOfRange, "有效距離"),
        (operation::Reason::Blocked, "視線受阻"),
        (operation::Reason::NoMp, "魔力不足"),
        (operation::Reason::Unlearned, "尚未學會"),
        (operation::Reason::TooHeavy, "負重"),
    ] {
        let mut state = detached(context(), 2);
        state.receipt = Some(Arc::new(Receipt {
            epoch: 1,
            command: q.clone(),
            equipment: None,
            vip_equipment: None,
            outcome: Outcome::Operation(
                crate::modern_lito::battle_action::Rejection::None,
                operation::Disposition::Denied,
                reason,
            ),
        }));
        assert!(state.command_notice().contains(expected));
        assert!(state.command_notice().contains("未生效"));
        assert_eq!(
            state.outcome(),
            Some(Outcome::Operation(
                crate::modern_lito::battle_action::Rejection::None,
                operation::Disposition::Denied,
                reason
            ))
        );
    }
}
