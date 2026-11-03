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
  let mut l=Wire::new(login.accept()?.0,0x12345678)?;l.authenticate(&version,false,&check)?;
  let mut directory=vec![59,1];directory.extend(b"RetMoon\0");directory.extend([127,0,0,1]);directory.extend(game_addr.port().to_le_bytes());l.send(&directory)?;l.send(&[243,1,0,0,0,0])?;l.expect(b"\xc8RetMoon\0")?;
  let mut redirect=vec![244,0,127,0,0,1];redirect.extend(game_addr.port().to_le_bytes());l.send(&redirect)?;trace.extend(std::mem::take(&mut l.trace));drop(l);
  let mut g=Wire::new(game.accept()?.0,0x10203040)?;g.authenticate(&version,true,&check)?;g.expect(&[226])?;g.send(&[3,1])?;g.send(&row)?;
  let mut select=vec![201];let end=row[1..].iter().position(|v|*v==0).unwrap()+1;select.extend_from_slice(&row[1..=end]);g.expect(&select)?;
  g.send(&[200,0])?;g.send(&[124,0])?;g.expect(&[202,2])?;g.send(&packet("GAME-initial",201))?;g.send(&packet("GAME-initial",202))?;g.expect(&request)?;
  // Only normal wire commands establish the room. No joined-book setter.
  while let Ok(bodies)=rx.recv_timeout(Duration::from_secs(20)) {
   if bodies.is_empty(){break;}
   g.send_many(bodies)?;done.send(())?;
  }
  trace.extend(g.trace);Ok(trace)
 });Ok((addr,tx,acks,h))
}
fn transmit(tx:&mpsc::Sender<Vec<Vec<u8>>>,acks:&mpsc::Receiver<()>,bodies:Vec<Vec<u8>>)->Result<()>{tx.send(bodies)?;acks.recv_timeout(Duration::from_secs(10))?;Ok(())}
fn departure_body(room:u32,alias:&[u8])->Vec<u8>{let mut b=vec![208];b.extend(room.to_le_bytes());b.extend(alias);b.push(0);b}
fn rewritten_cd(template:&[u8],alias:&[u8],kind:u8)->Vec<u8>{let end=template[5..].iter().position(|b|*b==0).unwrap()+5;let mut b=template[..5].to_vec();b.extend(alias);b.push(0);b.extend_from_slice(&template[end+1..]);b[6+alias.len()]=kind;b}
fn main()->Result<()>{
 let arg=std::env::args().nth(1).unwrap();let p=Path::new(&arg);let base=p.join("logs/continuation-20261003");
 let data:Value=serde_json::from_slice(&fs::read(base.join("r516-test-inputs.json"))?)?;
 let bot=hex(data["ArenaBot"].as_str().unwrap());let maximum=departure_body(u32::MAX,&[b'A';27]);
 let mut truncations=0;for body in [&bot,&maximum]{let decoded=departure::decode(body)?;assert_eq!(decoded.raw_body,*body);for n in 0..body.len(){assert!(departure::decode(&body[..n]).is_err());truncations+=1;}let mut extra=body.clone();extra.push(0);assert!(departure::decode(&extra).is_err());}
 assert!(departure::decode(&[208,1,0,0,0,0]).is_err());assert!(departure::decode(&departure_body(1,&[0x81])).is_err());assert!(departure::decode(&departure_body(1,&[b'A';28])).is_err());
 assert_eq!(departure::decode(&departure_body(0,&[0xa4,0xa4]))?.display_alias,[0xa4,0xa4]);
 let vectors=data["participants"].as_array().unwrap();let saved=|frame:u64|hex(vectors.iter().find(|v|v["example"]==0&&v["frame_index"]==frame).unwrap()["body_hex"].as_str().unwrap());
 let host=saved(70);let guest=saved(71);let host_state=saved(78);
 let members::Packet::Participant(host_row)=members::decode(&host)? else{panic!("host CD")};let host_alias=host_row.display_alias;
 let members::Packet::Participant(guest_row)=members::decode(&guest)? else{panic!("guest CD")};let guest_alias=guest_row.display_alias;
 let mut cases=vec![json!({"case":"strict D0 ds/CP950 envelope, all new truncations, exact end, invalid bytes and maximum alias; generic Native constructor remains separate","pass":true,"truncations":truncations})];
 let mut pure=ready(p)?;let e=pure.begin_source_connection_candidate()?;assert!(e.decoded_packet(&bot).is_err());
 let mut book=members::Book::for_observation(&e,1)?;let unknown=departure_body(1,&host_alias);let event=e.decoded_java_room_participant(&unknown)?;
 book.observe(&pure,&e.decoded_java_room_participant(&host)?)?;let old=book.snapshot().rows;
 assert_eq!(book.observe(&pure,&event)?,members::Change::IgnoredUnqualifiedDeparture);assert_eq!(book.snapshot().rows,old);
 assert_eq!(book.observe(&pure,&e.decoded_java_room_participant(&bot)?)?,members::Change::DepartureAbsentNoop);
 let before=pure.presentation_key();assert!(pure.request_java_room_leave().is_err());assert_eq!(pure.presentation_key(),before);
 cases.push(json!({"case":"unqualified own/peer alias never deletes a known human row or creates a leave intent; absent ArenaBot no-op; observation book is not joined authority","pass":true}));
 // One normal-entry integration case. Two transport epochs (LOGIN/GAME),
 // one producer thread; no Java/SQL/native/player GUI execution.
 let intent=Intent::Create(Settings{name:b"BattleSkillRoom".to_vec(),attributes:8,password:vec![],capacity:2,options:None});
 let request=room::encode(&intent)?;let (addr,tx,acks,worker)=fixture(data.clone(),request)?;
 let mut r=setup(p,addr)?;until(&mut r,|r|r.java_character_snapshot().stage==lito_modern::modern_lito::native_city_runtime::java_character_flow::Stage::ChannelReceipts);
 let epoch=r.source_connection_candidate_token().unwrap();let control=r.java_room_entry_control(intent).unwrap();r.handle_java_room_entry_control(&control)?;
 until(&mut r,|r|r.java_room_entry_snapshot().phase==Phase::AwaitReply);
 transmit(&tx,&acks,vec![hex(data["CE"].as_str().unwrap())])?;until(&mut r,|r|r.java_room_participants_snapshot().is_some());
 let joined=r.java_room_participants_snapshot().unwrap();assert_eq!(joined.origin,members::Origin::JoinedCeReceipt);assert_eq!(joined.ce_revision,Some(r.java_room_entry_snapshot().revision));assert!(joined.synthetic_self_expectation_qualified);assert_eq!(joined.stream_arrivals,0);
 let room_checkpoint=r.java_room_entry_snapshot();let character=r.java_character_snapshot();let bootstrap=r.source_bootstrap_snapshot();let actor=r.source_actor_snapshot();let native=(r.source_flow().phase(),r.source_flow().dispatch_word(),r.source_flow().mode().identifier());let send=r.source_send_snapshot();
 let at=|r:&mut Runtime,n:u64|{until(r,|r|r.java_room_participants_snapshot().is_some_and(|b|b.revision==n));let b=r.java_room_participants_snapshot().unwrap();assert_eq!(b.stream_arrivals,n);assert_eq!(b.completeness,members::Completeness::PossiblyIncomplete);b};
 transmit(&tx,&acks,vec![bot.clone()])?;let b=at(&mut r,1);assert!(b.rows.is_empty());assert_eq!(b.last_stream_change,Some(members::Change::DepartureAbsentNoop));
 transmit(&tx,&acks,vec![host.clone(),guest.clone(),host_state])?;let b=at(&mut r,4);assert_eq!(b.rows.len(),2);assert_eq!(b.rows[0].state,members::MemberState::Ready);
 let own=b"RetWire(RetMoon)";let peer_other_kind=rewritten_cd(&host,&host_alias,1);let own0=rewritten_cd(&host,own,0);let own1=rewritten_cd(&host,own,1);
 transmit(&tx,&acks,vec![peer_other_kind,own0,own1])?;let b=at(&mut r,7);assert_eq!(b.rows.len(),5);
 let before=r.presentation_key();let own_error=r.request_java_room_leave().unwrap_err().to_string();assert!(own_error.contains("completed same-context C_D0"));assert_eq!(r.presentation_key(),before);assert!(r.java_return_control(java_return::Intent::Resume).is_none());
 transmit(&tx,&acks,vec![departure_body(1,own)])?;let b=at(&mut r,8);assert_eq!(b.rows.len(),5);assert_eq!(b.last_stream_change,Some(members::Change::IgnoredOwnDepartureWithoutCompletedIntent));
 transmit(&tx,&acks,vec![departure_body(2,&host_alias)])?;let b=at(&mut r,9);assert_eq!(b.rows.len(),5);assert_eq!(b.last_stream_change,Some(members::Change::IgnoredDepartureOtherRoom));
 // Equal serial belonging to another Runtime does not authorize this book.
 let foreign=pure.begin_source_connection_candidate()?;assert_eq!(foreign.serial(),epoch.serial());let before=r.presentation_key();assert!(r.consume_java_room_participants_candidate(&foreign.decoded_java_room_participant(&unknown)?).is_err());assert_eq!(r.presentation_key(),before);close(&mut pure)?;
 transmit(&tx,&acks,vec![unknown.clone()])?;let b=at(&mut r,10);assert_eq!(b.rows.len(),3);assert_eq!(b.last_stream_change,Some(members::Change::DepartureRemoved{rows:2}));assert!(b.rows.iter().all(|row|row.participant.display_alias!=host_alias));
 transmit(&tx,&acks,vec![unknown.clone()])?;let b=at(&mut r,11);assert_eq!(b.rows.len(),3);assert_eq!(b.last_stream_change,Some(members::Change::DepartureAbsentNoop));
 transmit(&tx,&acks,vec![rewritten_cd(&host,b"ArenaBot",1),bot.clone()])?;let b=at(&mut r,13);assert_eq!(b.rows.len(),3);assert_eq!(b.last_stream_change,Some(members::Change::DepartureRemoved{rows:1}));
 let mut shop=vec![209];shop.extend(&guest_alias);shop.extend([0,2]);transmit(&tx,&acks,vec![shop])?;let b=at(&mut r,14);assert_eq!(b.rows.iter().find(|row|row.participant.display_alias==guest_alias).unwrap().state,members::MemberState::ShopList);
 assert_eq!(r.java_room_entry_snapshot(),room_checkpoint);assert_eq!(r.java_character_snapshot(),character);assert_eq!(r.source_bootstrap_snapshot(),bootstrap);assert_eq!(r.source_actor_snapshot(),actor);assert_eq!((r.source_flow().phase(),r.source_flow().dispatch_word(),r.source_flow().mode().identifier()),native);assert_eq!(r.source_send_snapshot(),send);
 cases.push(json!({"case":"one normal LOGIN/source/GAME/NEWS/character/channel/owned CE integration; private JoinedCeReceipt book; real StreamIngress D0 before CD, CD/D1, cross-kind peer removal, absent no-op, Bot later removal, own-without-completed-intent and wrong-room refusal","pass":true,"actual_stream_participant_arrivals":b.stream_arrivals,"retained_room":1,"rows":b.rows.len(),"own_display_binding":"independent pre-entry synthetic account/source expectation, matched normal submitted account hash and selected source; not CE owner/physical-name derivation","real_backend_or_native_identity_qualified":false,"own_exit_advanced":false,"same_serial_foreign_epoch_refused":true}));
 let checkpoint=b.clone();let late=epoch.decoded_java_room_participant(&unknown)?;
 let dynamic:Value=serde_json::from_slice(&fs::read(base.join("r516-inputs/dynamic/frames.json"))?)?;
 let d5=dynamic.as_array().unwrap().iter().filter(|v|v["direction"]=="S").filter_map(|v|v["body_hex"].as_str()).find(|v|v.starts_with("d5")).map(hex).unwrap();
 // Coalesce the unsupported exact saved D5 and a valid late CD in one write,
 // so the rejected body is never skipped to process the later room event.
 transmit(&tx,&acks,vec![d5.clone(),host.clone()])?;until(&mut r,|r|r.source_connection_epoch().is_none());let rejection=r.last_source_rejection().unwrap();assert_eq!(rejection.opcode,Some(213));assert_eq!(rejection.body_bytes,d5.len());assert_eq!(r.last_java_room_participants_checkpoint(),Some(checkpoint.clone()));assert!(r.java_room_participants_snapshot().is_none());
 let before=r.presentation_key();assert!(r.consume_java_room_participants_candidate(&late).is_err());assert!(r.feed_source_stream_candidate(&epoch,&[]).is_err());assert_eq!(r.presentation_key(),before);
 // A fresh local epoch still cannot install a joined book from old receipts.
 let fresh=r.begin_source_connection_candidate()?;assert_ne!(fresh.serial(),epoch.serial());let before=r.presentation_key();assert!(r.consume_java_room_participants_candidate(&late).is_err());assert!(r.consume_java_room_participants_candidate(&fresh.decoded_java_room_participant(&unknown)?).is_err());assert_eq!(r.presentation_key(),before);
 close(&mut r)?;tx.send(vec![])?;let trace=worker.join().map_err(|_|anyhow::anyhow!("owned producer panicked"))??;let stats=r.player_fixture_snapshot();assert_eq!(stats.workers_started,2);assert_eq!(stats.workers_joined,2);
 assert!(trace.iter().filter_map(|v|v["C"].as_array()).all(|body|body.first().and_then(Value::as_u64)!=Some(208)),"no ordinary C_D0 intent emitted");
 cases.push(json!({"case":"actual saved D5 stays first unqualified next body; late CD not skipped into, archived 14-arrival book, stale/fresh scope refuses old room receipts, Close joins both IO workers and one wire producer","pass":true,"first_unsupported_opcode":"D5","body_bytes":d5.len(),"reason":rejection.reason,"workers_started":stats.workers_started,"workers_joined":stats.workers_joined,"discarded_late_events":stats.discarded_late_events,"queued_events":stats.queued_events,"producer_joined":true}));
 fs::write(base.join("r516-new-wire-producer-trace.json"),serde_json::to_vec_pretty(&trace)?)?;
 fs::write(base.join("r516-new-consumer-results.json"),serde_json::to_vec_pretty(&json!({"cases":cases,"new_loopback_cases":1,"socket_fixture_workers":2,"wire_producer_threads":1,"all_workers_joined":true,"normal_entry_and_private_CE_book":true,"Java_SQL":0,"Native_GUI":0,"player_runs":0,"old_PASS_replays":0,"test_only_authority_setters":0,"own_exit_advanced":false,"unqualified_self_backend_canonicalization":"Remains open: finite ASCII synthetic declaration is not actual Java account/source canonicalization authority"}))?)?;
 println!("R516 four new focused groups PASS; ONE normal-entry Modern loopback;2 IO workers+1 producer joined;0JavaSQL/NativeGUI/oldPASS");Ok(())
}
