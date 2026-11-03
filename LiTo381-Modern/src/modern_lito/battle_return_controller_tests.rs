//! Continue the actual v41 equipment battle, without replacing its controllers
//! or TLS sessions. This fixture does NOT negotiate SHOP: world transitions
//! must preserve that fact and reject shop input locally without disconnecting.
use super::*;
use crate::modern_lito::{
    application_protocol::{NavigationPhase, NavigationStatus},
    character_data::CharacterDataState,
    shop_protocol as shop,
};

fn unsupported_shop(cs: &mut [Client; 2], i: usize) {
    let before = cs[i].view();
    let nav = before.navigation.as_ref().unwrap();
    let request = shop::Request::read(
        nav.context,
        nav.snapshot.channel_id.unwrap(),
        before.rooms.joined(),
    );
    assert_eq!(before.shop, shop::State::Unsupported);
    assert!(!login_window::can_open_shop(&before));
    assert!(cs[i]
        .controller
        .command(LoginCommand::Shop(request))
        .is_err());
    let after = cs[i].view();
    assert_eq!(after.shop, shop::State::Unsupported);
    assert!(!after.shop_pending);
    assert_eq!(after.phase, LoginPhase::InLobby);
    assert!(
        after.shop.snapshot().is_none(),
        "unsupported is not zero balance"
    );
    assert_eq!(after.navigation, before.navigation);
    assert_eq!(after.equipment, before.equipment);
}

fn leave_lobby(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || LoginCommand::LeaveLobby(context));
    wait(cs, d, "v41 leave lobby acknowledged", |cs| {
        cs[i].view().phase == LoginPhase::ChoosingChannel
    });
    let view = cs[i].view();
    let nav = view.navigation.as_ref().unwrap();
    assert_eq!(nav.status, NavigationStatus::Ok);
    assert_eq!(nav.snapshot.phase, NavigationPhase::Channels);
    assert_eq!(nav.context, context);
    assert!(nav.snapshot.channel_id.is_none());
    assert!(matches!(view.shop, shop::State::Unsupported));
    assert!(matches!(view.rooms, rooms::State::NotRequested));
    assert!(!view.world_owns_lobby());
    assert!(!login_window::can_open_shop(&view));
}

fn roster(cs: &mut [Client; 2], i: usize, d: &Deadline, context: SelectionContext) {
    send(cs, i, d, || LoginCommand::ReturnCharacters(context));
    wait(cs, d, "v41 server refreshed roster", |cs| {
        let v = cs[i].view();
        v.phase == LoginPhase::CharactersReady && cs[i].runtime.is_quiescent()
    });
    let view = cs[i].view();
    assert!(view.roster_epoch > context.epoch);
    assert!(view.navigation.is_none());
    assert!(matches!(
        view.character_data,
        CharacterDataState::NotRequested
    ));
    assert!(matches!(view.equipment, equipment::State::NotRequested));
    assert!(matches!(view.shop, shop::State::NotRequested));
    assert!(!view.world_owns_lobby());
    assert!(cs[i].surface.is_none());
    assert!(cs[i]
        .controller
        .command(LoginCommand::SelectCharacter(context))
        .is_err());
}

pub(super) fn verify(cs: &mut [Client; 2], d: &Deadline, contexts: [SelectionContext; 2]) {
    let saved = cs.each_ref().map(gear_snapshot);
    // World LEAVE does not pretend to leave the room. Reject lobby/source
    // navigation until a separate, revision-bound room LEAVE is confirmed.
    for i in (0..2).rev() {
        unsupported_shop(cs, i);
        assert!(cs[i].view().rooms.joined().is_some());
        assert!(cs[i]
            .controller
            .command(LoginCommand::LeaveLobby(contexts[i]))
            .is_err());
        assert!(cs[i]
            .controller
            .command(LoginCommand::ReturnCharacters(contexts[i]))
            .is_err());
        send(cs, i, d, || {
            LoginCommand::Rooms(rooms::Request::read(contexts[i], CHANNEL).unwrap())
        });
        wait(cs, d, "fresh room before explicit departure", |cs| {
            !cs[i].view().room_pending
        });
        let joined = cs[i].view().rooms.joined().unwrap().clone();
        send(cs, i, d, || {
            LoginCommand::Rooms(room_command(contexts[i], &joined, rooms::Action::Leave, 0))
        });
        wait(cs, d, "room departure acknowledged", |cs| {
            !cs[i].view().room_pending
        });
        assert!(cs[i].view().rooms.joined().is_none());
        unsupported_shop(cs, i);
        leave_lobby(cs, i, d, contexts[i]);
    }
    // The same authenticated connection must re-enter every advertised lobby,
    // including the one just left. Channels are not independent TCP listeners.
    for channel in 0..4 {
        send(cs, 0, d, || LoginCommand::EnterChannel {
            context: contexts[0],
            channel_id: channel,
        });
        wait(cs, d, "v41 channel re-entry acknowledged", |cs| {
            let v = cs[0].view();
            v.phase == LoginPhase::InLobby && !v.room_pending
        });
        let view = cs[0].view();
        let nav = view.navigation.as_ref().unwrap();
        assert_eq!(nav.status, NavigationStatus::Ok);
        assert_eq!(nav.snapshot.channel_id, Some(channel));
        assert_eq!(
            nav.snapshot
                .channels
                .iter()
                .map(|c| c.id)
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(nav
            .snapshot
            .channels
            .iter()
            .all(|c| c.population == u32::from(c.id == channel)));
        assert_eq!(view.equipment, equipment::State::Ready(saved[0].clone()));
        assert!(view.canonical.equipment_supported() && !view.world_owns_lobby());
        assert!(view.rooms.joined().is_none());
        unsupported_shop(cs, 0);
        leave_lobby(cs, 0, d, contexts[0]);
    }
    for i in 0..2 {
        roster(cs, i, d, contexts[i]);
    }
    // Both leases are released before selecting the other's same-object-ID
    // character. Gear changed in source 7 must never leak into source 42.
    for source in [42, 7] {
        send(cs, 0, d, || LoginCommand::SelectSource(source));
        wait(cs, d, "v41 source reload", |cs| {
            cs[0].view().phase == LoginPhase::CharactersReady
        });
        let view = cs[0].view();
        assert_eq!(view.selected_source, Some(source));
        assert!(view.characters.iter().any(|c| c.object_id == 3333));
        let context = SelectionContext {
            key: CharacterKey {
                source_id: source,
                object_id: 3333,
            },
            epoch: view.roster_epoch,
        };
        send(cs, 0, d, || LoginCommand::SelectCharacter(context));
        wait(cs, d, "v41 source equipment reload", |cs| {
            let v = cs[0].view();
            v.phase == LoginPhase::ChoosingChannel
                && !v.record_pending
                && !v.spell_pending
                && matches!(v.equipment, equipment::State::Ready(_))
        });
        let gear = gear_snapshot(&cs[0]);
        let expected = &saved[usize::from(source == 42)];
        assert_eq!(gear.source.context, context);
        assert_eq!(gear.source.character, expected.source.character);
        assert_eq!(gear.source.stats, expected.source.stats);
        assert_eq!(gear.source.items, expected.source.items);
        assert_eq!(gear.flags, expected.flags);
        assert_eq!(gear.revision, expected.revision);
        assert!(!Arc::ptr_eq(&gear, expected));
        assert!(cs[0]
            .controller
            .command(LoginCommand::Equipment(equipment::Request::set(
                &saved[0], 1, false
            )))
            .is_err());
        assert!(cs[0]
            .controller
            .command(LoginCommand::EnterChannel {
                context: contexts[0],
                channel_id: 0
            })
            .is_err());
        roster(cs, 0, d, context);
    }
}
