//! Production Main + isolated SQL + real shared Quest renderer and skill70.
//! No native HWND, IME, audio-device or all-map acceptance is implied.
use super::*;
use crate::modern_lito::{
    chat_state::{Delivery, Token},
    chat_window as cw, direct_chat_protocol as dc, direct_chat_state as ds,
    direct_chat_window as dw, filtered_chat_protocol as fc,
};

#[test]
#[ignore = "v33 production Main/owned SQL/shared Quest: categories, preferences, private chat, real skill70, settlement and source return; no HWND"]
fn java_filtered_chat_sql_shared_loading_cast_settlement_return_and_source_reload() {
    sql_flow_with_chat(false, false, false, false, false, true, true);
}

fn ready(cs: &mut [Client; 2], d: &Deadline) {
    wait(cs, d, "SQL filtered/private admission", |cs| {
        cs.iter().all(|c| {
            let v = c.view();
            v.chat.filtered()
                && v.chat.token().is_some()
                && v.direct_chat.token().is_some()
                && !v.conversation_pending()
                && !v.room_pending
                && !v.world_pending
                && !v.combat_pending
                && !v.spell_pending
                && !v.record_pending
        })
    });
    for c in cs.iter() {
        let v = c.view();
        let token = v.chat.token().unwrap();
        let private = v.direct_chat.token().unwrap();
        assert_eq!(private.enrollment.context, token.scope.context);
        assert_eq!(private.room_id, token.scope.room_id);
        assert_eq!(private.membership, token.membership);
        login_window::validate_snapshot(&v).unwrap();
    }
}

fn public(cs: &mut [Client; 2], d: &Deadline, kind: fc::Kind, text: &str, delivered: bool) {
    ready(cs, d);
    let before = cs[0].view();
    let intent = cw::Intent {
        token: before.chat.token().unwrap(),
        text: text.into(),
        kind: if kind == fc::Kind::Normal {
            cw::Kind::Message
        } else {
            cw::Kind::Public(kind)
        },
    };
    cs[0].command(login_window::chat_command(&before, intent).unwrap());
    assert!(cs[0].view().chat_pending);
    assert!(cs[0].view().chat.filtered_messages() == before.chat.filtered_messages());
    wait(cs, d, "SQL category exact receipt and delivery", |cs| {
        let v = cs[0].view();
        !v.chat_pending
            && v.chat.delivery() == Delivery::Confirmed
            && (!delivered
                || cs[1]
                    .view()
                    .chat
                    .filtered_messages()
                    .iter()
                    .any(|m| m.text == text))
    });
    for i in 0..if delivered { 2 } else { 1 } {
        let v = cs[i].view();
        let rows: Vec<_> = v
            .chat
            .filtered_messages()
            .iter()
            .filter(|m| m.text == text)
            .collect();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].kind, kind);
        assert_eq!(rows[0].sender.key, SUN);
        assert_eq!(rows[0].sender.character_name, "SQL Sun Knight");
        assert_eq!(rows[0].sender.source_name, "太陽神");
    }
}

fn preferences(cs: &mut [Client; 2], d: &Deadline, trade: bool) {
    ready(cs, d);
    let before = cs[1].view();
    let value = dc::Preferences {
        whispers: true,
        world: true,
        trade,
    };
    let intent = dw::Intent {
        token: before.direct_chat.token().unwrap(),
        action: dc::Action::Preferences,
        target: None,
        preferences: Some(value),
        text: String::new(),
    };
    cs[1].command(login_window::direct_chat_command(&before, intent).unwrap());
    assert!(cs[1].view().direct_chat_pending);
    assert_eq!(
        cs[1].view().direct_chat.preferences(),
        before.direct_chat.preferences()
    );
    wait(cs, d, "SQL confirmed category preference", |cs| {
        !cs[1].view().direct_chat_pending
    });
    assert_eq!(
        cs[1].view().direct_chat.outcome(),
        ds::Outcome::Confirmed(dc::Action::Preferences)
    );
    assert_eq!(cs[1].view().direct_chat.preferences(), Some(value));
}

fn private(cs: &mut [Client; 2], d: &Deadline, text: &str) {
    ready(cs, d);
    let before = cs[1].view();
    let intent = dw::Intent {
        token: before.direct_chat.token().unwrap(),
        action: dc::Action::Send,
        target: Some(dc::Address {
            source_id: 7,
            name: "SQL Sun Knight".into(),
        }),
        preferences: None,
        text: text.into(),
    };
    cs[1].command(login_window::direct_chat_command(&before, intent).unwrap());
    assert!(cs[1].view().direct_chat_pending);
    assert!(cs[1].view().direct_chat.messages() == before.direct_chat.messages());
    wait(cs, d, "SQL source-qualified private delivery", |cs| {
        !cs[1].view().direct_chat_pending
            && cs[0]
                .view()
                .direct_chat
                .messages()
                .iter()
                .any(|m| m.text == text)
    });
    assert_eq!(
        cs[1].view().direct_chat.outcome(),
        ds::Outcome::Confirmed(dc::Action::Send)
    );
    let received = cs[0].view();
    let m = received
        .direct_chat
        .messages()
        .iter()
        .find(|m| m.text == text)
        .unwrap();
    assert_eq!((m.sender.key.source_id, m.sender.key.object_id), (42, 3333));
    assert_eq!(m.sender.source_name, "月亮");
    assert_eq!(m.recipient.key, SUN);
}

pub(super) fn lobby(cs: &mut [Client; 2], d: &Deadline) -> Token {
    public(cs, d, fc::Kind::World, "SQL大廳世界", true);
    let token = cs[0].view().chat.token().unwrap();
    assert_eq!(token.scope.room_id, 0);
    token
}
pub(super) fn loading(cs: &mut [Client; 2], d: &Deadline, old: Token) {
    ready(cs, d);
    assert_ne!(
        cs[0].view().chat.token().unwrap().membership,
        old.membership
    );
    assert!(cs[0]
        .controller
        .command(LoginCommand::FilteredChatSend {
            token: old,
            kind: fc::Kind::World,
            text: "不能跨入場使用".into(),
        })
        .is_err());
    assert!(cs[0].view().chat.filtered_messages().is_empty());
    public(cs, d, fc::Kind::World, "SQL世界載入中", true);
    preferences(cs, d, false);
}
pub(super) fn active(cs: &mut [Client; 2], d: &Deadline) {
    public(cs, d, fc::Kind::Trade, "SQL拒收交易", false);
    public(cs, d, fc::Kind::Normal, "SQL拒收之後的讀取界線", true);
    assert!(!cs[1]
        .view()
        .chat
        .filtered_messages()
        .iter()
        .any(|m| m.text == "SQL拒收交易"));
    preferences(cs, d, true);
    private(cs, d, "SQL戰鬥中的跨來源私訊");
}
pub(super) fn pending_cast_rejects_chat(cs: &mut [Client; 2], i: usize) {
    let v = cs[i].view();
    assert!(v.spell_pending);
    let intent = cw::Intent {
        token: v.chat.token().unwrap(),
        kind: cw::Kind::Public(fc::Kind::Trade),
        text: "不可取代尚未確認的施法".into(),
    };
    // Exercise the actual UI adapter against the captured pending frame. A
    // second controller.command() would poll a legitimately completed cast.
    assert!(login_window::chat_command(&v, intent).is_err());
    assert!(cs[i].view().spell_pending);
    assert!(!cs[i].view().chat_pending);
}
pub(super) fn after_cast(cs: &mut [Client; 2], d: &Deadline, i: usize) {
    public(
        cs,
        d,
        fc::Kind::Trade,
        if i == 0 {
            "SQL太陽神施法完成"
        } else {
            "SQL月亮施法完成"
        },
        true,
    );
    for c in cs.iter() {
        assert!(c.view().world.frame().is_some());
        assert_eq!(
            c.acks.len(),
            1,
            "chat must not repeat the renderer admission"
        );
    }
}
pub(super) fn returned(cs: &mut [Client; 2], d: &Deadline) {
    assert!(cs
        .iter()
        .all(|c| c.view().world.frame().is_none() && c.runtime.is_quiescent()));
    public(cs, d, fc::Kind::World, "SQL結算返回房間", true);
    private(cs, d, "SQL結算後私訊");
}
pub(super) fn reselected(cs: &mut [Client; 2], d: &Deadline, old: Token, moon: SelectionContext) {
    let v = cs[0].view();
    assert!(v.chat.filtered_messages().is_empty() && v.chat.token().is_none());
    assert!(v.direct_chat.messages().is_empty() && v.direct_chat.token().is_none());
    cs[0].command(LoginCommand::EnterChannel {
        context: moon,
        channel_id: CHANNEL,
    });
    wait(cs, d, "SQL reselected source chat admission", |cs| {
        let v = cs[0].view();
        v.phase == LoginPhase::InLobby
            && v.chat.token().is_some()
            && v.direct_chat.token().is_some()
    });
    let v = cs[0].view();
    assert!(v.chat.filtered());
    assert_eq!(v.chat.token().unwrap().scope.context, moon);
    assert_eq!(v.direct_chat.owner().unwrap().key, moon.key);
    assert!(v.chat.filtered_messages().is_empty() && v.direct_chat.messages().is_empty());
    assert!(cs[0]
        .controller
        .command(LoginCommand::FilteredChatSend {
            token: old,
            kind: fc::Kind::Trade,
            text: "不可沿用舊來源".into(),
        })
        .is_err());
}
