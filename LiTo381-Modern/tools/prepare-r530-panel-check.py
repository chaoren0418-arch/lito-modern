from pathlib import Path
P=Path(__file__).resolve().parent.parent
s=(P/'tools/r526-ready-consumer.rs').read_text('utf-8')
s=s[:s.index('include!("r526-ready-case.rs");')]
s=s.replace('for round in 0..2','for round in 0..1')
start=s.index('r.configure_java_room_entry_adapter(');end=s.index('r.handle_player_entry_command(Command::OpenLogin)',start)
s=s[:start]+'room_player::configure(r)?;'+s[end:]
s=s.replace('java_room_ready as ready_flow,player_entry_commands', 'java_room_ready as ready_flow,room_player::{self,Panel,Operation},player_entry_commands')
s=s.replace('"SYNTHETIC:R526_ONE_SELF_CORRELATION_CASE"','"SYNTHETIC:R530_NEW_PANEL_CALLBACK_ONLY"')
s=s.replace('g.send(&packet("GAME-initial",202))?;g.expect(&request)?;', 'g.send(&packet("GAME-initial",202))?;g.send(&catalog_body())?;g.expect(&request)?;')
s+='''
fn accept_bounded(listener:&TcpListener)->Result<TcpStream>{listener.set_nonblocking(true)?;let end=Instant::now()+Duration::from_secs(15);loop{match listener.accept(){Ok((s,_))=>{s.set_nonblocking(false)?;return Ok(s)},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{ensure!(Instant::now()<end,"owned accept deadline");thread::sleep(Duration::from_millis(2));},Err(e)=>return Err(e.into())}}}
fn catalog_body()->Vec<u8>{let mut b=vec![204,0];b.extend(1u32.to_le_bytes());b.extend(b"PanelRoom\\0");b.push(8);b.extend(1u16.to_le_bytes());b.extend(4u16.to_le_bytes());b.extend([0,0,1]);b}
fn d1(alias:&[u8],state:u8)->Vec<u8>{let mut b=vec![209];b.extend(alias);b.extend([0,state]);b}
fn case(r:&mut Runtime,data:&Value,addr:SocketAddr,tx:&mpsc::Sender<Vec<Vec<u8>>>,acks:&mpsc::Receiver<()>)->Result<Value>{
 setup(r,addr)?;until(r,|r|Panel::capture(r).enabled(Operation::Join(0)));
 let join=Panel::capture(r);ensure!(join.rooms.len()==1&&join.rooms[0].room_id==1&&!join.rooms[0].password_required,"observed public catalog");
 join.invoke(r,Operation::Join(0))?;until(r,|r|r.java_room_entry_snapshot().phase==Phase::AwaitReply);
 let mut ce=hex(data["CE"].as_str().unwrap());ce[2..6].copy_from_slice(&1u32.to_le_bytes());transmit(tx,acks,vec![ce])?;
 until(r,|r|r.java_room_entry_snapshot().phase==Phase::RoomReceipts);
 transmit(tx,acks,vec![hex(data["host_CD"].as_str().unwrap()),hex(data["guest_CD"].as_str().unwrap())])?;
 until(r,|r|Panel::capture(r).enabled(Operation::Ready));
 let alias=r.java_room_self_snapshot().alias.unwrap().raw_alias;
 let initial=Panel::capture(r);assert_eq!(initial.display.own_d1_state,None);
 transmit(tx,acks,vec![d1(&alias,0)])?;until(r,|r|Panel::capture(r).display.own_d1_state==Some(0));
 let before=r.presentation_key();assert!(initial.invoke(r,Operation::Ready).is_err());assert_eq!(r.presentation_key(),before);
 let ready=Panel::capture(r);ready.invoke(r,Operation::Ready)?;
 let queued=r.java_room_ready_snapshot().requests[0].clone();assert_eq!(queued.raw_body,vec![210,1]);assert_eq!((queued.frame_bytes,queued.written_bytes),(4,0));assert_eq!(Panel::capture(r).display.own_d1_state,Some(0));
 assert!(ready.invoke(r,Operation::Ready).is_err());
 until(r,|r|r.java_room_ready_snapshot().requests[0].phase==ready_flow::Phase::CompletedWrite);
 assert_eq!(r.java_room_ready_snapshot().requests[0].written_bytes,4);
 assert_eq!(Panel::capture(r).display.own_d1_state,Some(0));
 transmit(tx,acks,vec![vec![],vec![210,1],d1(&alias,1),vec![225,3]])?;
 until(r,|r|Panel::capture(r).display.own_d1_state==Some(1)&&Panel::capture(r).display.countdown==Some(3));
 let cancel=Panel::capture(r);cancel.invoke(r,Operation::Cancel)?;
 assert_eq!(Panel::capture(r).display.own_d1_state,Some(1));assert_eq!(Panel::capture(r).display.countdown,Some(3));
 until(r,|r|r.java_room_ready_snapshot().requests[1].phase==ready_flow::Phase::CompletedWrite);
 transmit(tx,acks,vec![vec![],vec![210,0],vec![225,0]])?;
 until(r,|r|Panel::capture(r).display.countdown.is_none());assert_eq!(Panel::capture(r).display.own_d1_state,Some(1));
 transmit(tx,acks,vec![d1(&alias,0)])?;until(r,|r|Panel::capture(r).display.own_d1_state==Some(0));
 let held=Panel::capture(r);let epoch=r.source_connection_candidate_token().unwrap();
 let mut cf=vec![207];cf.extend(1u32.to_le_bytes());transmit(tx,acks,vec![vec![225,2],cf])?;
 until(r,|r|r.java_room_participants_snapshot().is_some_and(|b|b.joined_room_invalidated));
 let before=r.presentation_key();assert!(held.invoke(r,Operation::Ready).is_err());assert!(held.invoke(r,Operation::Cancel).is_err());assert_eq!(r.presentation_key(),before);assert_eq!(Panel::capture(r).display.room_id,None);assert_eq!(Panel::capture(r).display.countdown,None);
 r.disconnect_source_connection_candidate(&epoch)?;assert!(held.invoke(r,Operation::Ready).is_err());assert!(join.invoke(r,Operation::Join(0)).is_err());
 close(r)?;assert!(held.invoke(r,Operation::Cancel).is_err());assert_eq!(Panel::capture(r).display,room_player::Display::default());
 Ok(json!({"public_observed_join":true,"positive_panel_ready_cancel":true,"actual_c_d2_bodies":[[210,1],[210,0]],"queued_and_completed_write_do_not_change_d1":true,"received_e1_zero_clears_only_countdown":true,"held_panel_refused_after_d1_revision_pending_cf_source_retirement_close":true,"one_normal_login_game_epoch":true,"GUI":0,"actual_backend":false}))
}
fn save_new(p:&Path,v:&Value)->Result<()>{let mut f=fs::OpenOptions::new().write(true).create_new(true).open(p)?;f.write_all(&serde_json::to_vec_pretty(v)?)?;Ok(())}
fn main()->Result<()>{
 let arg=std::env::args().nth(1).unwrap();let p=Path::new(&arg);let base=p.join("logs/continuation-20261003");let output=base.join("r530-panel-check-attempt1");fs::create_dir(&output)?;
 let data:Value=serde_json::from_slice(&fs::read(base.join("r526-test-inputs.json"))?)?;
 let request=room::encode(&Intent::Join{room_id:1,password:vec![]})?;let(addr,tx,acks,worker)=fixture(data.clone(),request)?;let mut r=ready(p)?;
 let outcome=std::panic::catch_unwind(std::panic::AssertUnwindSafe(||case(&mut r,&data,addr,&tx,&acks)));
 let result=match outcome{Ok(v)=>v,Err(_)=>Err(anyhow::anyhow!("new Panel check assertion failure; first attempt retained"))};
 let closed=close(&mut r);let _=tx.send(vec![]);let producer=worker.join().map_err(|_|anyhow::anyhow!("own mock worker panic"));let stats=r.player_fixture_snapshot();
 save_new(&output.join("result.json"),&json!({"case":result.as_ref().ok(),"first_error":result.as_ref().err().map(|e|e.to_string()),"producer_error":producer.as_ref().ok().and_then(|r|r.as_ref().err()).map(|e|e.to_string()),"close_ok":closed.is_ok(),"IO_started":stats.workers_started,"IO_joined":stats.workers_joined,"queued_events":stats.queued_events,"discarded_late_events":stats.discarded_late_events,"GUI":0,"JavaSQL":0,"real_provider_connections":0,"old_PASS_replays":0,"execution_attempt":1}))?;
 if let Ok(Ok(trace))=&producer{save_new(&output.join("wire.json"),&json!(trace))?;}
 closed?;let trace=producer??;result?;
 let commands=trace.iter().filter(|v|v["C"][0]==210).collect::<Vec<_>>();ensure!(commands.len()==2&&commands[0]["C"]==json!([210,1])&&commands[1]["C"]==json!([210,0])&&commands.iter().all(|v|v["frame_bytes"]==4),"typed Panel callbacks produced wrong mock wire");
 ensure!(stats.workers_started==2&&stats.workers_joined==2&&stats.queued_events==0&&stats.discarded_late_events==0,"own fixture cleanup unsettled");
 println!("R530 new positive Panel Join/Ready/Cancel + D1/E1 + stale callback checks PASS; one finite Rust mock; GUI0/backend0;2IO joined");Ok(())
}
'''
# Keep only imports used by this new check; do not include or execute R526 cases.
start=s.index('use lito_modern::');end=s.index('\nuse anyhow',start)
s=s[:start]+'''use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,login_events::FixtureProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_room_entry::{self as room,Intent,Phase},java_room_ready as ready_flow,room_player::{self,Panel,Operation},player_entry_commands::Command},native_multisector_resources::Inputs,native_resources::BytePin};'''+s[end:]
with (P/'tools/r530-panel-callback-check.rs').open('x',encoding='utf-8',newline='\n')as f:f.write(s)
print('New finite Panel callback check prepared; no old case included')
