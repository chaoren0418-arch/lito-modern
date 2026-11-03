//! Independent x64 probe linked to the exact already-built Cargo library.
//! Its peer is the owned original C02 NativeGameTimerHarness (four sessions).
use lito_modern::modern_lito::{native_connection::Limits, native_login::{LoginController, LoginPhase}};
use std::{path::PathBuf, time::{Duration, Instant}};

fn wait(c: &mut LoginController, condition: impl Fn(&LoginController) -> bool) {
    let until = Instant::now() + Duration::from_secs(15);
    loop {
        c.poll();
        assert!(!matches!(c.snapshot().phase, LoginPhase::Failed | LoginPhase::Closed), "{:?}", c.snapshot().error);
        if condition(c) { return; }
        assert!(Instant::now() < until, "deadline {:?}: {:?}; environment {:?}; world_ready {}",
            c.snapshot().phase, c.snapshot().error, c.snapshot().environment, c.snapshot().world.ready());
        std::thread::sleep(Duration::from_millis(3));
    }
}
fn main() {
    let args: Vec<_> = std::env::args().collect();
    assert_eq!(args.len(), 4, "Expected LOGIN endpoint, owned output and workspace root");
    let peer = args[1].parse().unwrap();
    let out = PathBuf::from(&args[2]).canonicalize().unwrap();
    let workspace = PathBuf::from(&args[3]).canonicalize().unwrap();
    assert!(out.starts_with(&workspace));
    let mut c = LoginController::default();
    let mut rounds = Vec::new();
    let mut last_epoch = 0;
    for round in 0..2 {
        c.start(peer, Limits {read_chunk: 2, write_chunk: 3, ..Limits::default()}).unwrap();
        let until = Instant::now() + Duration::from_secs(15);
        let mut submitted = false;
        loop {
            c.poll();
            assert!(Instant::now() < until, "bootstrap {:?}", c.snapshot().phase);
            if c.snapshot().notice.as_ref().is_some_and(|n| n.acknowledgement().is_some()) {
                c.acknowledge_notice().unwrap();
            }
            match c.snapshot().phase {
                LoginPhase::ReadyForLogin if !submitted => { c.login("PeerFixture", "TestOnlyPassword").unwrap(); submitted=true; }
                LoginPhase::SourcesReady => c.select_source(1).unwrap(),
                LoginPhase::RedirectReady => c.begin_game().unwrap(),
                LoginPhase::CharactersReady => break,
                LoginPhase::Failed | LoginPhase::Closed => panic!("{:?}", c.snapshot().error),
                _ => {},
            }
            std::thread::sleep(Duration::from_millis(3));
        }
        let epoch = c.transport().snapshot().epoch;
        assert!(epoch > last_epoch); last_epoch=epoch;
        let before = c.snapshot().environment.clone();
        assert_eq!(before.epoch, Some(epoch));
        assert!(before.calendar.is_none() && before.map.is_none());
        assert_eq!((before.blind,before.weather,before.time_packets,before.status_time_packets),(0,[0;3],0,0));
        c.select_character(0).unwrap();
        wait(&mut c, |c| c.snapshot().phase == LoginPhase::CharacterSelected
            && c.snapshot().environment.time_packets >= 1
            && c.snapshot().services.as_ref().is_some_and(|d| !d.rooms.is_empty()));
        let admitted = c.snapshot().environment.clone();
        assert_eq!(admitted.epoch,Some(epoch));
        assert!(admitted.calendar.unwrap().raw > 0 && admitted.calendar.unwrap().raw % 3 == 0);
        c.select_rental_set(1).unwrap();wait(&mut c,|c|c.snapshot().phase==LoginPhase::RentalReady);
        c.join_rental_room(0).unwrap();
        wait(&mut c, |c| c.snapshot().phase==LoginPhase::BattlePending && c.snapshot().world.ready()
            && c.snapshot().environment.status_time_packets > 0);
        let state=c.snapshot();
        assert_eq!((state.world.map_id,state.environment.map,state.environment.map_mask_argument),(Some(500),Some(500),Some(15)));
        assert_eq!(state.environment.epoch,Some(epoch));
        assert_eq!(state.world.status.unwrap().game_time,0); // actual C02 writer
        // C02 sends world S21 before its fixed-zero S0C. The original status
        // receiver writes that zero too; the next C20/S21 later changes it.
        assert_eq!(state.environment.calendar.unwrap().raw,state.world.status.unwrap().game_time);
        let mapped=state.environment.clone();
        c.cancel();
        let retired=c.snapshot().environment.clone();
        assert!(retired.epoch.is_none()&&retired.calendar.is_none()&&retired.map.is_none());
        assert_eq!((retired.blind,retired.weather,retired.underwater),(0,[0;3],false));
        let until=Instant::now()+Duration::from_secs(10);
        while !c.is_quiescent() {c.poll();assert!(Instant::now()<until);std::thread::sleep(Duration::from_millis(3));}
        rounds.push(serde_json::json!({"round":round,"epoch":epoch,"before_admission":before,
            "after_admission":admitted,"mapped":mapped,"after_cancel":retired}));
    }
    let result=serde_json::json!({"status":"original_C02_environment_admission_world_and_retirement_verified",
        "rounds":rounds,"quiescent":c.is_quiescent(),"calendar_locally_advanced":false,
        "authority_mutations":0,"synthetic_packet_injection":false,"whole_goal_complete":false});
    std::fs::write(out.join("client.json"),serde_json::to_vec_pretty(&result).unwrap()).unwrap();
    println!("{result}");
}
