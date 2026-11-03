use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,login_events::FixtureProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_room_entry::{self as room,Adapter,Intent,Settings,Phase},java_room_participants as members,java_room_departure as departure,java_room_tail as tail,java_room_catalog as catalog,java_room_self as self_flow,java_room_settings as settings,java_room_mode_request as mode_request,player_entry_commands::Command},native_multisector_resources::Inputs,native_resources::BytePin};
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
fn setup(r:&mut Runtime,addr:SocketAddr)->Result<()>{let profile=json!({"kind":"modern_synthetic_loopback_fixture_v1","label":"SYNTHETIC:R525_ONE_SELF_CORRELATION_CASE","endpoint":addr.to_string(),"code_page":950,"from_cd_registry_dword":2,"revision_word_source":0x14f8,"supplemental":null,"read_cap":31,"write_cap":1,"connect_delay_ms":0});r.configure_synthetic_player_fixture(FixtureProfile::parse(&serde_json::to_vec(&profile)?)?)?;r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;r.configure_java_room_entry_adapter(Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_entry_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":room::CONTRACT_SHA}))?)?)?;r.configure_java_room_participants_adapter(members::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_participants_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":members::CONTRACT_SHA}))?)?)?;
r.configure_java_room_departure_adapter(departure::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_departure_v1","producer_sha256":PRODUCER_SHA,"route_sha256":departure::ROUTE_SHA,"native_sha256":departure::NATIVE_SHA,"commit_sha256":departure::COMMIT_SHA}))?)?)?;
r.configure_java_room_tail_adapter(tail::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_tail_v1","producer_sha256":PRODUCER_SHA,"supplement_sha256":tail::SUPPLEMENT_SHA,"dynamic_sha256":tail::DYNAMIC_SHA,"native_sha256":tail::NATIVE_SHA}))?)?)?;r.configure_java_room_catalog_adapter(catalog::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_room_catalog_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":catalog::CONTRACT_SHA,"lifecycle_sha256":catalog::LIFECYCLE_SHA,"comparison_sha256":catalog::COMPARISON_SHA,"publication_sha256":catalog::PUBLICATION_SHA}))?)?)?;r.configure_java_room_self_adapter(self_flow::Adapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_singleton_self_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":self_flow::CONTRACT_SHA}))?)?)?;r.configure_java_room_settings_adapter(settings_adapter()?)?;r.configure_java_room_mode_map_adapter(mode_map_adapter()?)?;r.configure_java_room_mode_request_adapter(mode_request_adapter()?)?;r.handle_player_entry_command(Command::OpenLogin)?;edit(r)?;until(r,|r|r.source_choice_control(0).is_some());let c=r.source_choice_control(0).unwrap();r.handle_player_entry_command(Command::ChooseSource(&c))?;until(r,|r|r.source_news_control().is_some());let c=r.source_news_control().unwrap();r.handle_player_entry_command(Command::ContinueNews(&c))?;until(r,|r|r.source_character_control(0).is_some());let c=r.source_character_control(0).unwrap();r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:true})?;until(r,|r|r.source_channel_control(2).is_some());let c=r.source_channel_control(2).unwrap();r.handle_player_entry_command(Command::ChooseChannel(&c))?;Ok(())}
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
  let frames=data["setup_frames"].as_array().unwrap();
  let packet=|ep:&str,op:u8|->Vec<u8>{frames.iter().filter(|v|v["epoch"]==ep&&v["direction"]=="S").filter_map(|v|v["body_hex"].as_str()).map(hex).find(|v|v[0]==op).unwrap()};
  let version=packet("LOGIN-initial",58);let check=packet("LOGIN-initial",0);let row=packet("GAME-initial",203);let mut trace=vec![];
  let mut l=Wire::new(accept_bounded(&login)?,0x12345678)?;l.authenticate(&version,false,&check)?;
  let mut directory=vec![59,1];directory.extend(data["source_ascii"].as_str().unwrap().as_bytes());directory.push(0);directory.extend([127,0,0,1]);directory.extend(game_addr.port().to_le_bytes());l.send(&directory)?;l.send(&[243,1,0,0,0,0])?;let mut source=vec![200];source.extend(data["source_ascii"].as_str().unwrap().as_bytes());source.push(0);l.expect(&source)?;
  let mut redirect=vec![244,0,127,0,0,1];redirect.extend(game_addr.port().to_le_bytes());l.send(&redirect)?;trace.extend(std::mem::take(&mut l.trace));drop(l);
  let mut g=Wire::new(accept_bounded(&game)?,0x10203040)?;g.authenticate(&version,true,&check)?;g.expect(&[226])?;g.send(&[3,1])?;g.send(&row)?;
  let mut select=vec![201];let end=row[1..].iter().position(|v|*v==0).unwrap()+1;select.extend_from_slice(&row[1..=end]);g.expect(&select)?;
  g.send(&[200,0])?;g.send(&[124,0])?;g.expect(&[202,2])?;g.send(&packet("GAME-initial",201))?;g.send(&packet("GAME-initial",202))?;g.expect(&request)?;
  // Only normal wire commands establish the room. No joined-book setter.
  while let Ok(bodies)=rx.recv_timeout(Duration::from_secs(20)) {
   if bodies.is_empty(){break;}
   let mut bodies=bodies;if bodies.first().is_some_and(|b|b.is_empty()){bodies.remove(0);ensure!(!bodies.is_empty(),"relative mode expectation absent");let command=bodies.remove(0);ensure!(command.len()==3&&command.starts_with(&[209,2])&&matches!(command[2],1|2),"only new relative mode expected command");g.expect(&command)?;}g.send_many(bodies)?;done.send(())?;
  }
  trace.extend(g.trace);Ok(trace)
 });Ok((addr,tx,acks,h))
}
fn transmit(tx:&mpsc::Sender<Vec<Vec<u8>>>,acks:&mpsc::Receiver<()>,bodies:Vec<Vec<u8>>)->Result<()>{tx.send(bodies)?;acks.recv_timeout(Duration::from_secs(10))?;Ok(())}
include!("r525-mode-writer-case.rs");
