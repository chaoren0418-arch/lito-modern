use lito_modern::modern_lito::{native_city_runtime::{Runtime,UiPage,Action,login_events::FixtureProfile,login_bootstrap::{JavaFixtureAdapter,PRODUCER_SHA,CONTRACT_SHA},java_return::{Intent,Phase},player_entry_commands::Command,status_receipt},native_multisector_resources::Inputs,native_resources::BytePin};
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
fn confirm(r:&mut Runtime,intent:Intent,yes:bool)->Result<lito_modern::modern_lito::native_city_runtime::java_return::Confirmation>{let c=r.java_return_control(intent).unwrap();let token=r.request_java_return_confirmation(&c)?;r.handle_player_entry_command(Command::ConfirmJavaReturn{confirmation:&token,accepted:yes})?;Ok(token)}
struct Wire {s:TcpStream,tx:u64,rx:u64,trace:Vec<Value>}
impl Wire {
 fn new(s:TcpStream,seed:u32)->Result<Self>{s.set_read_timeout(Some(Duration::from_secs(15)))?;s.set_write_timeout(Some(Duration::from_secs(2)))?;let c=0x87546ca100000000u64|u64::from(seed);let mut w=Self{s,tx:c,rx:c,trace:vec![]};let mut b=vec![1];b.extend(seed.to_le_bytes());w.s.write_all(&[7,0])?;w.s.write_all(&b)?;Ok(w)}
 fn send(&mut self,b:&[u8])->Result<()> {let mut raw=b.to_vec();let k=self.tx.to_le_bytes();let mut prev=0;for(i,v)in raw.iter_mut().enumerate(){*v^=k[i&7]^prev;prev=*v;}self.tx=self.tx.wrapping_add(b.len()as u64);self.s.write_all(&((raw.len()+2)as u16).to_le_bytes())?;self.s.write_all(&raw)?;self.trace.push(json!({"S":b}));Ok(())}
 fn recv(&mut self)->Result<Vec<u8>> {let mut h=[0;2];self.s.read_exact(&mut h)?;let n=u16::from_le_bytes(h)as usize;ensure!((3..=157).contains(&n),"bounded mock C frame");let mut b=vec![0;n-2];self.s.read_exact(&mut b)?;let k=self.rx.to_le_bytes();let mut prev=0;for(i,v)in b.iter_mut().enumerate(){let old=*v;*v^=k[i&7]^prev;prev=old;}self.rx=self.rx.wrapping_add(b.len() as u64);self.trace.push(if b[0]==1{json!({"C_login":"MASKED_OWN_FIXTURE","body_bytes":b.len()})}else{json!({"C":b})});Ok(b)}
 fn expect(&mut self,b:&[u8])->Result<()>{let got=self.recv()?;ensure!(got==b,"new command mismatch expected opcode{:02X}, got{:02X}",b[0],got[0]);Ok(())}
 fn authenticate(&mut self,version:&[u8],game:bool,check:&[u8])->Result<()> {ensure!(self.recv()?[0]==0,"compatibility opcode");self.send(version)?;ensure!(self.recv()?[0]==1,"fresh authentication required");if game{self.send(&[230])?;}else{self.send(check)?;}Ok(())}
}
#[derive(Clone,Copy,PartialEq)]enum Case {Refresh,Resume,Expired,Unknown}
fn mock(frames:Vec<Value>,case:Case)->Result<(SocketAddr,mpsc::Sender<()>,thread::JoinHandle<Result<Vec<Value>>>)>{
 let login=TcpListener::bind("127.0.0.1:0")?;let game=TcpListener::bind("127.0.0.1:0")?;let login_addr=login.local_addr()?;let game_addr=game.local_addr()?;let (tx,rx)=mpsc::channel();
 let h=thread::spawn(move||{
  let packet=|ep:&str,op:u8|->Vec<u8>{frames.iter().filter(|v|v["epoch"]==ep&&v["direction"]=="S").filter_map(|v|v["body_hex"].as_str()).map(hex).find(|v|v[0]==op).unwrap()};
  let version=packet("LOGIN-initial",58);let check=packet("LOGIN-initial",0);let row=packet("GAME-initial",203);let status=packet("GAME-initial",12);let mut all=vec![];
  let mut l=Wire::new(login.accept()?.0,0x12345678)?;l.authenticate(&version,false,&check)?;
  let mut directory=vec![59,1];directory.extend(b"RetMoon\0");directory.extend([127,0,0,1]);directory.extend((game_addr.port()).to_le_bytes());l.send(&directory)?;l.send(&[243,1,0,0,0,0])?;l.expect(b"\xc8RetMoon\0")?;
  let mut redirect=vec![244,0,127,0,0,1];redirect.extend(game_addr.port().to_le_bytes());l.send(&redirect)?;all.extend(l.trace.clone());drop(l);
  let mut g=Wire::new(game.accept()?.0,0x10203040)?;g.authenticate(&version,true,&check)?;g.expect(&[226])?;g.send(&[3,1])?;g.send(&row)?;
  let mut select=vec![201];let end=row[1..].iter().position(|v|*v==0).unwrap()+1;select.extend_from_slice(&row[1..=end]);g.expect(&select)?;g.send(&[200,0])?;g.send(&[124,0])?;g.expect(&[202,2])?;g.send(&packet("GAME-initial",201))?;g.send(&packet("GAME-initial",202))?;g.send(&status)?;
  if case==Case::Unknown {g.send(&packet("GAME-initial",77))?;g.send(&packet("GAME-initial",80))?;all.extend(g.trace);return Ok(all);}
  if case==Case::Refresh {
   g.expect(&[203])?;g.expect(&[202,100])?;g.send(&[245])?;g.expect(b"\xc8RetMoon\0")?;g.send(&[3,1])?;g.send(&row)?;
   g.expect(&[15])?;all.extend(g.trace);return Ok(all);
  }
  g.expect(&[208])?;g.expect(&[15])?;all.extend(std::mem::take(&mut g.trace));drop(g);
  if case==Case::Expired {rx.recv().unwrap();return Ok(all);}
  let mut g=Wire::new(game.accept()?.0,0x0badcafe)?;g.authenticate(&version,true,&check)?;g.expect(&[226])?;g.send(&[3,1])?;g.send(&row)?;
  // Sealed GAME-resumed has silent restore: no client C9 or S_C8/S7C.
  g.expect(&[202,0])?;g.send(&packet("GAME-resumed",201))?;g.send(&packet("GAME-resumed",202))?;rx.recv().unwrap();g.send(&status)?;g.send(&packet("GAME-initial",77))?;g.send(&packet("GAME-initial",80))?;all.extend(g.trace);Ok(all)
 });Ok((login_addr,tx,h))
}
fn setup(p:&Path,addr:SocketAddr)->Result<Runtime>{let mut r=ready(p)?;let profile=json!({"kind":"modern_synthetic_loopback_fixture_v1","label":"SYNTHETIC:R512_NEW_RETURN_BRANCH","endpoint":addr.to_string(),"code_page":950,"from_cd_registry_dword":2,"revision_word_source":0x14f8,"supplemental":null,"read_cap":31,"write_cap":1,"connect_delay_ms":0});r.configure_synthetic_player_fixture(FixtureProfile::parse(&serde_json::to_vec(&profile)?)?)?;r.configure_login_producer_fixture(JavaFixtureAdapter::parse(&serde_json::to_vec(&json!({"kind":"modern_synthetic_AD368_bootstrap_v1","producer_sha256":PRODUCER_SHA,"contract_sha256":CONTRACT_SHA}))?)?)?;r.handle_player_entry_command(Command::OpenLogin)?;edit(&mut r)?;until(&mut r,|r|r.source_choice_control(0).is_some());let c=r.source_choice_control(0).unwrap();r.handle_player_entry_command(Command::ChooseSource(&c))?;until(&mut r,|r|r.source_news_control().is_some());let c=r.source_news_control().unwrap();r.handle_player_entry_command(Command::ContinueNews(&c))?;until(&mut r,|r|r.source_character_control(0).is_some());let c=r.source_character_control(0).unwrap();r.handle_player_entry_command(Command::ChooseCharacter{control:&c,double_click:true})?;until(&mut r,|r|r.source_channel_control(2).is_some());let c=r.source_channel_control(2).unwrap();r.handle_player_entry_command(Command::ChooseChannel(&c))?;Ok(r)}
fn status_json(s:status_receipt::Status)->Value{json!({"object_id":s.object_id,"level":s.level,"experience":s.experience,"strength":s.strength,"intelligence":s.intelligence,"wisdom":s.wisdom,"dexterity":s.dexterity,"constitution":s.constitution,"charisma":s.charisma,"current_hp":s.current_hp,"maximum_hp":s.maximum_hp,"current_mp":s.current_mp,"maximum_mp":s.maximum_mp,"armor_class":s.armor_class,"game_time":s.game_time,"food_gauge":s.food_gauge,"weight_gauge":s.weight_gauge,"lawful":s.lawful,"fire_resistance":s.fire_resistance,"water_resistance":s.water_resistance,"wind_resistance":s.wind_resistance,"earth_resistance":s.earth_resistance})}
fn main()->Result<()> {
 let p=std::env::args().nth(1).unwrap();let p=Path::new(&p);let base=p.join("logs/continuation-20261003");let data:Value=serde_json::from_slice(&fs::read(base.join("r512-test-inputs.json"))?)?;
 let frames=data["frames"].as_array().unwrap().clone();let mut results=vec![];
 for v in data["status_vectors"].as_array().unwrap(){let b=hex(v["body_hex"].as_str().unwrap());assert_eq!(status_json(status_receipt::decode(&b)?),v["expected"]);for n in 0..37 {assert!(status_receipt::decode(&b[..n]).is_err());}let mut extra=b.clone();extra.push(0);assert!(status_receipt::decode(&extra).is_err());}
 for v in data["mana_vectors"].as_array().unwrap(){let b=hex(v["body_hex"].as_str().unwrap());let s=status_receipt::decode_mana(&b)?;assert_eq!(json!([s.current_raw,s.maximum_raw,s.native_current_argument()]),v["expected"]);for n in 0..5{assert!(status_receipt::decode_mana(&b[..n]).is_err());}let mut x=b;x.push(0);assert!(status_receipt::decode_mana(&x).is_err());}
 results.push(json!({"case":"two sealed S4D + sign/high-bit raw16 and Modern exact-size policy","pass":true}));
 results.push(json!({"case":"two sealed S0C + distinct signed/LE/game-time/gauge vector, strict size","pass":true}));
 // Original encrypted bodies, independent Rust arithmetic; LOGIN stays masked.
 let mut counters=std::collections::BTreeMap::new();let mut matched=0;let mut masked=0;
 for f in &frames {let ep=f["epoch"].as_str().unwrap();if f["plain_KEY"]==true {let b=hex(f["body_hex"].as_str().unwrap());let c=0x87546ca100000000u64|u64::from(u32::from_le_bytes(b[1..5].try_into().unwrap()));counters.insert((ep,"C"),(c,c));counters.insert((ep,"S"),(c,c));continue;}if f["EOF"]==true||f["transport_event"].is_string(){continue;}let dir=f["direction"].as_str().unwrap();let c=&mut counters.get_mut(&(ep,dir)).unwrap().0;let mut b=hex(f["encrypted_body_hex"].as_str().unwrap());let key=c.to_le_bytes();let mut previous=0;for(i,v)in b.iter_mut().enumerate(){let old=*v;*v^=key[i&7]^previous;previous=old;}*c=c.wrapping_add(b.len()as u64);if f["body_hex"]=="MASKED_OWN_FIXTURE_LOGIN" {masked+=1;}else{assert_eq!(b,hex(f["body_hex"].as_str().unwrap()));matched+=1;}}
 assert_eq!((matched,masked),(54,4));results.push(json!({"case":"sealed 4-epoch wire consumption","matched_unmasked":matched,"masked_login":masked,"pass":true}));
 let mut traces=vec![];
 for case in [Case::Refresh,Case::Resume,Case::Expired,Case::Unknown] {
  let (addr,signal,worker)=mock(frames.clone(),case)?;let mut r=setup(p,addr)?;
  if case==Case::Unknown {until(&mut r,|r|r.source_connection_epoch().is_none());assert_eq!(r.last_source_rejection().unwrap().opcode,Some(80));assert!(r.last_java_character_checkpoint().unwrap().status.is_some());assert!(r.last_java_character_checkpoint().unwrap().mana.is_some());assert_eq!(r.presentation_key().phase,0);results.push(json!({"case":"S0C accepted then S4D accepted, first unsupported S50 retires epoch","pass":true}));}
  else {
   until(&mut r,|r|r.java_character_snapshot().status.is_some());
   let old_epoch=r.source_connection_candidate_token().unwrap();let old_status=old_epoch.decoded_packet(&hex(data["status_vectors"][0]["body_hex"].as_str().unwrap()))?;
   let prior_key=r.java_character_snapshot();let prior_source=r.source_bootstrap_snapshot().selected_name.clone();let old_back=r.source_lobby_back().unwrap();
   let cancelled=confirm(&mut r,Intent::Resume,false)?;assert_eq!(r.java_character_snapshot(),prior_key);assert!(r.handle_player_entry_command(Command::ConfirmJavaReturn{confirmation:&cancelled,accepted:true}).is_err());
   if case==Case::Refresh {
    r.handle_player_entry_command(Command::BackLobby(&old_back))?;until(&mut r,|r|r.java_return_snapshot().phase==Phase::Idle&&r.source_character_control(0).is_some());
    assert_eq!(r.source_connection_epoch(),Some(old_epoch.serial()));assert_eq!(r.source_bootstrap_snapshot().selected_name,prior_source);assert!(r.handle_player_entry_command(Command::BackLobby(&old_back)).is_err());assert!(r.java_character_snapshot().inventory.is_none());
    let cancel=confirm(&mut r,Intent::Logout,false)?;assert!(r.source_connection_epoch().is_some());assert!(r.resolve_java_return_confirmation(&cancel,true).is_err());confirm(&mut r,Intent::Logout,true)?;until(&mut r,|r|r.page()==UiPage::Closed);
    results.push(json!({"case":"same socket CB/CA100/F5/raw source C8/S03-CB + stale control + cancel/confirmed logout","pass":true}));
   }else{
    let accepted=confirm(&mut r,Intent::Resume,true)?;
    let old_view=r.player_fixture_write_view()?.unwrap();until(&mut r,|r|r.java_return_snapshot().phase==Phase::Retained);assert!(old_view.bytes().is_err());assert!(r.source_connection_epoch().is_none());assert_eq!(r.java_return_snapshot().source,prior_source);assert_eq!(r.java_return_snapshot().character,prior_key.selected_name);assert_eq!(r.java_return_snapshot().channel,Some(2));
    assert!(r.resolve_java_return_confirmation(&accepted,true).is_err());assert!(r.consume_java_status_candidate(&old_status).is_err());assert!(r.handle_player_entry_command(Command::BackLobby(&old_back)).is_err());
    until(&mut r,|r|r.player_fixture_snapshot().retiring_workers==0);
    if case==Case::Expired {
     let end=Instant::now()+Duration::from_secs(31);while Instant::now()<end{r.poll();thread::sleep(Duration::from_millis(10));}assert!(r.resume_java_return_fixture().is_err());signal.send(()).unwrap();results.push(json!({"case":"real 30-second retained deadline rejects fresh connection","pass":true}));
    }else{
     r.handle_player_entry_command(Command::OpenLogin)?;assert_ne!(r.source_connection_epoch(),Some(old_epoch.serial()));assert!(r.consume_java_status_candidate(&old_status).is_err());edit(&mut r)?;
     until(&mut r,|r|r.source_news_control().is_some());let c=r.source_news_control().unwrap();r.handle_player_entry_command(Command::ContinueNews(&c))?;until(&mut r,|r|r.java_return_snapshot().phase==Phase::ResumeChoice);
     assert!(r.source_character_control(0).is_none());assert!(r.java_character_snapshot().inventory.is_none());assert!(r.java_character_snapshot().character_result.is_none());let c=r.source_channel_control(0).unwrap();r.handle_player_entry_command(Command::ChooseChannel(&c))?;
     until(&mut r,|r|r.java_return_snapshot().one_shot_consumed);assert_eq!(r.java_character_snapshot().channel_info.unwrap().channel,2);assert!(r.handle_player_entry_command(Command::ChooseChannel(&c)).is_err());assert_eq!(r.presentation_key().phase,0);signal.send(()).unwrap();until(&mut r,|r|r.source_connection_epoch().is_none());assert_eq!(r.last_source_rejection().unwrap().opcode,Some(80));results.push(json!({"case":"D0/0F partial writes -> retired old epoch -> same-account fresh GAME auth/NEWS/roster -> no C9 -> CA0 overridden2 once -> S50 boundary","pass":true}));
    }
   }
  }
  close(&mut r)?;traces.push(json!({"case":match case{Case::Refresh=>"refresh",Case::Resume=>"resume",Case::Expired=>"expired",Case::Unknown=>"unknown"},"frames":worker.join().unwrap()?,"workers":format!("{:?}",r.player_fixture_snapshot())}));
 }
 fs::write(base.join("r512-new-consumer-results.json"),serde_json::to_vec_pretty(&json!({"cases":results,"traces":traces,"new_Java_SQL":0,"GUI":0,"historical_PASS_reruns":0,"owner_player_ready":false,"mock":"new return branches only; sealed receipt bodies + synthetic scheduling, no original server execution"}))?)?;println!("7 new focused groups passed; sealed wire54+masked4; owned mock workers joined; first S50 retained");Ok(())
}
