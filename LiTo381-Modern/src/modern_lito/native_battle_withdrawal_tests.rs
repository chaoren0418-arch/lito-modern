use super::*;

#[test]
fn withdrawal_requires_live_non_rental_battle_and_is_not_reentrant() {
    let mut c=LoginController::default();
    for phase in [LoginPhase::Disconnected,LoginPhase::RoomReady,LoginPhase::BattleResult,
        LoginPhase::ReturningCharacters,LoginPhase::RentalRestartPending] {
        c.state.phase=phase;
        assert!(c.request_battle_withdrawal().is_err());
        assert!(!c.state.battle_withdrawal_pending);
        assert!(c.deadline.is_none());
    }
    c.state.phase=LoginPhase::BattlePending;
    c.state.battle_withdrawal_pending=true;
    assert!(c.request_battle_withdrawal().is_err());
    assert!(c.state.battle_withdrawal_pending);
    c.state.battle_withdrawal_pending=false;c.state.rental_set=Some(0);
    assert!(c.request_battle_withdrawal().is_err());
}

fn system(text:&str)->Vec<u8>{
    let mut b=vec![0x0f,9];b.extend_from_slice(&encode_text(text,256).unwrap());b.push(0);b
}

#[test]
fn authoritative_combat_refusal_releases_request_without_retiring_scene() {
    let mut c=LoginController::default();c.state.phase=LoginPhase::BattlePending;
    c.state.world.map_id=Some(564);c.state.world.generation=13;c.state.world.started=true;
    c.state.battle_withdrawal_pending=true;
    c.deadline=Some(Instant::now()+Duration::from_secs(8));let deadline=c.deadline;
    c.accept_game(&system("其他系統通知")).unwrap();
    assert!(c.state.battle_withdrawal_pending);assert_eq!(c.deadline,deadline);
    c.accept_game(&system("戰鬥中無法退出，請停止攻擊並等待 10 秒。")).unwrap();
    assert!(!c.state.battle_withdrawal_pending);assert!(c.deadline.is_none());
    assert_eq!(c.state.phase,LoginPhase::BattlePending);
    assert_eq!(c.state.world.map_id,Some(564));assert_eq!(c.state.world.generation,13);
    assert_eq!(c.state.world.system_message_count,2);
}

#[test]
fn cancelled_or_failed_withdrawal_cannot_survive_connection_owner() {
    let mut c=LoginController::default();c.state.battle_withdrawal_pending=true;
    c.cancel();assert!(!c.state.battle_withdrawal_pending);
    c.state.battle_withdrawal_pending=true;c.fail("fixture failure".into());
    assert!(!c.state.battle_withdrawal_pending);
    assert_eq!(c.state.phase,LoginPhase::Failed);
}
