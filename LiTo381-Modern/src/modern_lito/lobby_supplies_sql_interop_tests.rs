//! Continue the real Main/isolated SQL supplies battle through all four
//! lobbies, then select the other source's same object ID. No native UI claim.
use super::*;

fn enter(cs: &mut [Client; 2], d: &Deadline, context: SelectionContext, channel: u8) {
    cs[0].command(LoginCommand::EnterChannel {
        context,
        channel_id: channel,
    });
    wait(cs, d, "SQL supplies lobby re-entry", |cs| {
        let v = cs[0].view();
        v.phase == LoginPhase::InLobby && !v.room_pending
    });
    let v = cs[0].view();
    assert_eq!(
        v.navigation.as_ref().unwrap().snapshot.channel_id,
        Some(channel)
    );
    assert!(v.rooms.joined().is_none());
    assert!(login_window::can_open_shop(&v));
    assert!(matches!(v.shop, shop::State::NotRequested));
    assert_source(&v, context);
}

fn leave(cs: &mut [Client; 2], d: &Deadline, context: SelectionContext) {
    cs[0].command(LoginCommand::LeaveLobby(context));
    wait(cs, d, "SQL supplies back to channels", |cs| {
        cs[0].view().phase == LoginPhase::ChoosingChannel
    });
    assert!(matches!(cs[0].view().shop, shop::State::NotRequested));
    assert!(!login_window::can_open_shop(&cs[0].view()));
}

fn buy_sell(cs: &mut [Client; 2], d: &Deadline) -> shop::Snapshot {
    let before = shop_action(cs, 0, d, shop::Action::Read, 0, 0);
    let offer = before.offer(40010).unwrap();
    let owned = before.stack(40010).map_or(0, |s| s.quantity);
    let bought = shop_action(cs, 0, d, shop::Action::Buy, 40010, 3);
    assert_eq!(bought.balance, before.balance - 3 * offer.buy_price);
    assert_eq!(bought.stack(40010).unwrap().quantity, owned + 3);
    let sold = shop_action(cs, 0, d, shop::Action::Sell, 40010, 1);
    assert_eq!(
        sold.balance,
        bought.balance + offer.buy_price.min(offer.sell_price)
    );
    assert_eq!(sold.stack(40010).unwrap().quantity, owned + 2);
    assert_eq!(sold.round_id, before.round_id);
    assert_eq!(sold.revision, bought.revision + 1);
    sold
}

pub(super) fn channels_after_battle(
    cs: &mut [Client; 2],
    d: &Deadline,
    context: SelectionContext,
) -> shop::Request {
    let carried = buy_sell(cs, d);
    assert_eq!(
        carried.stack(40010).unwrap().quantity,
        2,
        "the previous match bag was reset"
    );
    let old =
        shop::Request::read(context, CHANNEL, None).mutation(&carried, shop::Action::Buy, 40010, 1);
    let mut previous = old;
    for channel in [1, 2, 3, 0] {
        leave(cs, d, context);
        enter(cs, d, context, channel);
        assert!(
            cs[0]
                .controller
                .command(LoginCommand::Shop(previous))
                .is_err(),
            "old channel cannot spend in a new lobby"
        );
        let actual = shop_action(cs, 0, d, shop::Action::Read, 0, 0);
        assert_eq!(
            actual.round_id, carried.round_id,
            "channel changes are not new matches"
        );
        assert_eq!(actual.balance, carried.balance);
        assert_eq!(actual.stacks, carried.stacks);
        assert_source(&cs[0].view(), context);
        previous = shop::Request::read(context, channel, None).mutation(
            &actual,
            shop::Action::Buy,
            40010,
            1,
        );
    }
    let after = buy_sell(cs, d);
    assert_eq!(after.stack(40010).unwrap().quantity, 4);
    assert_source(&cs[0].view(), context);
    // A valid current request, not an already-stale revision, is retained for
    // the next source-isolation assertion.
    shop::Request::read(context, CHANNEL, None).mutation(&after, shop::Action::Buy, 40010, 1)
}

pub(super) fn changed_source(
    cs: &mut [Client; 2],
    d: &Deadline,
    context: SelectionContext,
    old: shop::Request,
) {
    assert_eq!(old.context.key.object_id, context.key.object_id);
    assert_ne!(old.context.key.source_id, context.key.source_id);
    enter(cs, d, context, CHANNEL);
    assert!(cs[0].controller.command(LoginCommand::Shop(old)).is_err());
    let empty = shop_action(cs, 0, d, shop::Action::Read, 0, 0);
    assert_eq!(empty.balance, 20000);
    assert!(
        empty.stacks.is_empty(),
        "source 7 supplies cannot leak into source 42"
    );
    assert_ne!(empty.token, old.token);
    let after = buy_sell(cs, d);
    assert_eq!(after.stack(40010).unwrap().quantity, 2);
    assert_source(&cs[0].view(), context);
    leave(cs, d, context);
}
