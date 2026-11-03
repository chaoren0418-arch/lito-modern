fn guard(r:&mut Runtime)->anyhow::Result<usize>{
 use lito_modern::modern_lito::{native_city_runtime::Action,native_mode_menu::Filter};
 let key=r.presentation_key();let mut rejected=0;
 for action in [Action::Back,Action::Enter,Action::Pan(1,1),Action::Filter(Filter::DiagnosticAll),Action::Select(0,514),Action::RowSelect(0),Action::OpenSourceMenu]{assert!(r.handle(action).is_err());assert_eq!(r.presentation_key(),key);rejected+=1;}
 assert!(r.leave_source_character_scene().is_err());assert!(r.enter_source_character_scene().is_err());assert!(!r.source_menu_invoker_enabled());assert!(r.handle_player_entry_command(Command::OpenLogin).is_err());assert_eq!(r.presentation_key(),key);Ok(rejected+3)
}
fn main()->anyhow::Result<()> {
 let at=Instant::now();let p=PathBuf::from(std::env::args_os().nth(1).unwrap());let base=p.join("logs/continuation-20261003");let o:Value=serde_json::from_slice(&fs::read(base.join("r503-source-oracle.json"))?)?;let mut trace=Vec::new();
 let mut r=ready(&p)?;let mut m=Metrics::default();let e=endpoint(&mut r,&o,1,1)?;until(&mut r,&mut m,|r|r.source_character_control(0).is_some());choose(&mut r,0)?;let t=r.source_character_submit().unwrap();e.steps.send(Step::Expect(o["normal_tx"][0].clone()))?;r.handle_player_entry_command(Command::SubmitCharacter(&t))?;
 let rejected=guard(&mut r)?;record(&base,"guard_pending_write","final Runtime and offline-gallery guard refuses old callbacks atomically while new character frame pending",json!({"rejected":rejected,"scene":"AwaitCharacterReply"}),&mut trace);
 until(&mut r,&mut m,|r|r.source_navigation_snapshot().write_complete);sent(&e);let rejected=guard(&mut r)?;assert_eq!(r.source_flow().phase(),0);record(&base,"guard_pending_reply","same guard persists after actual write completion without bypassing reply gate",json!({"rejected":rejected}),&mut trace);
 //Standalone first success must be encrypted from setupRX, no earlier refusal.
 e.steps.send(Step::Send(bytes(&o["guard_success"]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_navigation_snapshot().scene==Scene::Lobby);
 let rejected=guard(&mut r)?;assert_eq!(r.source_flow().phase(),3);assert_eq!(r.source_navigation_snapshot().return_mode,1);assert_eq!(r.source_identity().selected().unwrap().slot(),0);record(&base,"guard_lobby","actual S_C9 succeeds through final fixed library; lobby rejects every obsolete gallery action, preserves selected character and independent phase/mode",json!({"rejected":rejected}),&mut trace);
 let back=r.source_lobby_back().unwrap();closed(&mut r,&mut m)?;assert!(r.handle_player_entry_command(Command::BackLobby(&back)).is_err());let frames=finish(e);assert_eq!(frames,3);record(&base,"guard_close","Close remains admitted and joins own IO/resource owners after final guard",snapshot(&r,&m),&mut trace);
 fs::write(base.join("r503-guard-event-details.json"),serde_json::to_vec_pretty(&json!({"flows":trace,"elapsed_seconds":at.elapsed().as_secs_f64(),"actual_complete_socket_frames":frames,"new_navigation_socket_frames":1,"IO_workers_joined":1,"endpoint_threads_joined":1,"Runtime_Closed":1,"initial16_reruns":0,"historical_PASS_reruns":0,"GUI":0}))?)?;println!("R503 final affected guard increment PASS:4 flows /30 atomic action rejections /3 exact socket frames /1 endpoint+worker joined /GUI0");Ok(())
}
