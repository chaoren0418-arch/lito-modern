use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,login_events::FixtureProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_room_entry::{self as room,Intent,Phase},java_room_ready as ready_flow,room_player::{self,Panel,Operation},player_entry_commands::Command},native_multisector_resources::Inputs,native_resources::BytePin};
use anyhow::{ensure,Result};
use serde_json::{json,Value};
use sha2::{Digest,Sha256};
use std::{fs,path::Path,net::{TcpListener,TcpStream,SocketAddr},io::{Read,Write},thread,time::{Instant,Duration},sync::mpsc};
fn hex(s:&str)->Vec<u8>{s.as_bytes().chunks_exact(2).map(|v|u8::from_str_radix(std::str::from_utf8(v).unwrap(),16).unwrap()).collect()}
fn pin_input(p:&Path,n:&str)->Inputs {let manifest=p.join("logs/continuation-20261003").join(n);let b=fs::read(&manifest).unwrap();Inputs{project:p.into(),manifest,manifest_pin:BytePin{bytes:b.len() as u64,sha256:Sha256::digest(b).into()}}}
fn ready(p:&Path)->Result<Runtime>{let mut r=Runtime::start_with_menu_skin(pin_input(p,"r486-map-binding.json"),pin_input(p,"r506-native-menu-skin-binding.json"))?;until(&mut r,|r|r.page()==UiPage::Menu);Ok(r)}
fn until(r:&mut Runtime,f:impl Fn(&Runtime)->bool){let end=Instant::now()+Duration::from_secs(15);while !f(r){assert!(Instant::now()<end,"new branch wait expired: {} {:?} {:?}",r.status(),r.java_return_snapshot(),r.last_source_rejection());r.poll();thread::sleep(Duration::from_millis(2));}}
fn close(r:&mut Runtime)->Result<()> {r.handle(Action::Close)?;until(r,|r|r.page()==UiPage::Closed);ensure!(r.player_fixture_snapshot().workers_started==r.player_fixture_snapshot().workers_joined,"worker leak");Ok(())}
fn edit(r:&mut Runtime)->Result<()> {let c=r.source_login_control().unwrap();r.handle_player_entry_command(Command::EditLogin{control:&c,account:b"RetWire",password:b"WireOnly1"})?;let c=r.source_login_control().unwrap();r.handle_player_entry_command(Command::SubmitLogin(&c))}
struct Wire {s:TcpStream,tx:u64,rx:u64,trace:Vec<Value>}
impl Wire {
 fn new(s:TcpStream,seed:u32)->Result<Self>{s.set_read_timeout(Some(Duration::from_secs(15)))?;s.set_write_timeout(Some(Duration::from_secs(2)))?;let c=0x87546ca100000000u64|u64::from(seed);let mut w=Self{s,tx:c,rx:c,trace:vec![]};let mut b=vec![1];b.extend(seed.to_le_bytes());w.s.write_all(&[7,0])?;w.s.write_all(&b)?;Ok(w)}
 fn send(&mut self,b:&[u8])->Result<()> {let mut raw=b.to_vec();let k=self.tx.to_le_bytes();let mut prev=0;for(i,v)in raw.iter_mut().enumerate(){*v^=k[i&7]^prev;prev=*v;}self.tx=self.tx.wrapping_add(b.len()as u64);self.s.write_all(&((raw.len()+2)as u16).to_le_bytes())?;self.s.write_all(&raw)?;self.trace.push(json!({"S":b}));Ok(())}
 fn recv(&mut self)->Result<Vec<u8>> {let mut h=[0;2];self.s.read_exact(&mut h)?;let n=u16::from_le_bytes(h)as usize;ensure!((3..=157).contains(&n),"bounded mock C frame");let mut b=vec![0;n-2];self.s.read_exact(&mut b)?;let before=self.rx;let encrypted=b.clone();let k=self.rx.to_le_bytes();let mut prev=0;for(i,v)in b.iter_mut().enumerate(){let old=*v;*v^=k[i&7]^prev;prev=old;}self.rx=self.rx.wrapping_add(b.len() as u64);self.trace.push(if b[0]==1{json!({"C_login":"MASKED_OWN_FIXTURE","body_bytes":b.len()})}else{json!({"C":b,"C_cipher_hex":encrypted.iter().map(|v|format!("{v:02x}")).collect::<String>(),"counter_before":before,"counter_after":self.rx,"frame_bytes":encrypted.len()+2})});Ok(b)}
 fn expect(&mut self,b:&[u8])->Result<()>{let got=self.recv()?;ensure!(got==b,"new command mismatch expected opcode{:02X}, got{:02X}",b[0],got[0]);Ok(())}
 fn authenticate(&mut self,version:&[u8],game:bool,check:&[u8])->Result<()> {ensure!(self.recv()?[0]==0,"compatibility opcode");self.send(version)?;ensure!(self.recv()?[0]==1,"fresh authentication required");if game{self.send(&[230])?;}else{self.send(check)?;}Ok(())}
}
fn setup(r:&mut Runtime,addr:SocketAddr)->Result<()>{let profile=json!({"kind":"modern_synthetic_loopback_fixture_v1","label":"SYNTHETIC:R530_NEW_PANEL_CALLBACK_ONLY","endpoint":addr.to_string(),"code_page":950,"from_cd_registry_dword":2,"revision_word_source":0x14f8,"supplemental":null,"read_cap":31,"write_cap":1,"connect_delay_ms":0});r.configure_synthetic_player_fixture(FixtureProfile::parse(&serde_json::to_vec(&profile)?)?)?;r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;room_player::configure(r)?;r.handle_player_entry_command(Command::OpenLogin)?;edit(r)?;until(r,|r|r.source_choice_control(0).is_some());let c=r.source_choice_control(0).unwrap();r.handle_player_entry_command(Command::ChooseSource(&c))?;until(r,|r|r.source_news_control().is_some());let c=r.source_news_control().unwrap();r.handle_player_entry_command(Command::ContinueNews(&c))?;until(r,|r|r.source_character_control(0).is_some());let c=r.source_character_control(0).unwrap();r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:true})?;until(r,|r|r.source_channel_control(2).is_some());let c=r.source_channel_control(2).unwrap();r.handle_player_entry_command(Command::ChooseChannel(&c))?;Ok(())}
type Producer=(SocketAddr,mpsc::Sender<Vec<Vec<u8>>>,mpsc::Receiver<()>,thread::JoinHandle<Result<Vec<Value>>>);
impl Wire {
 fn send_many(&mut self,bodies:Vec<Vec<u8>>)->Result<()>{
  let mut combined=vec![];
  for b in bodies {let before=self.tx;let mut raw=b.clone();let k=before.to_le_bytes();let mut prev=0;for(i,v)in raw.iter_mut().enumerate(){*v^=k[i&7]^prev;prev=*v;}
   self.tx=self.tx.wrapping_add(b.len() as u64);combined.extend(((raw.len()+2) as u16).to_le_bytes());combined.extend(&raw);
   self.trace.push(json!({"S":b,"cipher_hex":raw.iter().map(|v|format!("{v:02x}")).collect::<String>(),"counter_before":before,"counter_after":self.tx,"frame_bytes":raw.len()+2}));}
  self.s.write_all(&combined)?;Ok(())
 }
}
fn fixture(data:Value,request:Vec<u8>)->Result<Producer>{
 let login=TcpListener::bind("127.0.0.1:0")?;let game=TcpListener::bind("127.0.0.1:0")?;
 let addr=login.local_addr()?;let game_addr=game.local_addr()?;
 let(tx,rx)=mpsc::channel::<Vec<Vec<u8>>>();let(done,acks)=mpsc::channel();
 let h=thread::spawn(move||{
  let mut all_trace=vec![];for round in 0..1 {let frames=data["setup_frames"].as_array().unwrap();
  let packet=|ep:&str,op:u8|->Vec<u8>{frames.iter().filter(|v|v["epoch"]==ep&&v["direction"]=="S").filter_map(|v|v["body_hex"].as_str()).map(hex).find(|v|v[0]==op).unwrap()};
  let version=packet("LOGIN-initial",58);let check=packet("LOGIN-initial",0);let row=packet("GAME-initial",203);let mut trace=vec![];
  let mut l=Wire::new(accept_bounded(&login)?,0x12345678)?;l.authenticate(&version,false,&check)?;
  let mut directory=vec![59,1];directory.extend(data["source_ascii"].as_str().unwrap().as_bytes());directory.push(0);directory.extend([127,0,0,1]);directory.extend(game_addr.port().to_le_bytes());l.send(&directory)?;l.send(&[243,1,0,0,0,0])?;let mut source=vec![200];source.extend(data["source_ascii"].as_str().unwrap().as_bytes());source.push(0);l.expect(&source)?;
  let mut redirect=vec![244,0,127,0,0,1];redirect.extend(game_addr.port().to_le_bytes());l.send(&redirect)?;trace.extend(std::mem::take(&mut l.trace));drop(l);
  let mut g=Wire::new(accept_bounded(&game)?,0x10203040)?;g.authenticate(&version,true,&check)?;g.expect(&[226])?;g.send(&[3,1])?;g.send(&row)?;
  let mut select=vec![201];let end=row[1..].iter().position(|v|*v==0).unwrap()+1;select.extend_from_slice(&row[1..=end]);g.expect(&select)?;
  g.send(&[200,0])?;g.send(&[124,0])?;g.expect(&[202,2])?;g.send(&packet("GAME-initial",201))?;g.send(&packet("GAME-initial",202))?;g.send(&catalog_body())?;g.expect(&request)?;
  // Only normal wire commands establish the room. No joined-book setter.
  while let Ok(bodies)=rx.recv_timeout(Duration::from_secs(20)) {
   if bodies.is_empty(){break;}if bodies==vec![vec![254]]{done.send(())?;break;}
   let mut bodies=bodies;if bodies.first().is_some_and(|b|b.is_empty()){bodies.remove(0);ensure!(!bodies.is_empty(),"participant ready expectation absent");let command=bodies.remove(0);ensure!(command.len()==2&&command[0]==210&&command[1]<=2,"only new participant ready expected command");g.expect(&command)?;}g.send_many(bodies)?;done.send(())?;
  }
  trace.extend(g.trace);for v in &mut trace{v["round"]=json!(round);}all_trace.extend(trace);}Ok(all_trace)
 });Ok((addr,tx,acks,h))
}
fn transmit(tx:&mpsc::Sender<Vec<Vec<u8>>>,acks:&mpsc::Receiver<()>,bodies:Vec<Vec<u8>>)->Result<()>{tx.send(bodies)?;acks.recv_timeout(Duration::from_secs(10))?;Ok(())}

fn accept_bounded(listener:&TcpListener)->Result<TcpStream>{listener.set_nonblocking(true)?;let end=Instant::now()+Duration::from_secs(15);loop{match listener.accept(){Ok((s,_))=>{s.set_nonblocking(false)?;return Ok(s)},Err(e)if e.kind()==std::io::ErrorKind::WouldBlock=>{ensure!(Instant::now()<end,"owned accept deadline");thread::sleep(Duration::from_millis(2));},Err(e)=>return Err(e.into())}}}
fn catalog_body()->Vec<u8>{let mut b=vec![204,0];b.extend(1u32.to_le_bytes());b.extend(b"PanelRoom\0");b.push(8);b.extend(1u16.to_le_bytes());b.extend(4u16.to_le_bytes());b.extend([0,0,1]);b}
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
