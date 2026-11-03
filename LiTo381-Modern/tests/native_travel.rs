use anyhow::{ensure, Context, Result};
use lito_modern::modern_lito::{
    compositor::Canvas, native_connection::Limits,
    native_item_actions::TargetChoice, native_login::{LoginController,LoginPhase as P,Snapshot},
    native_skill_effects::{Effects,qualify_travel}, native_world::World,
    native_world_scene::Model, resources::{SharedArchive,SharedArchiveKind},
    sprite_actions::{ActionLimits,FrameEvent,SpriteActionCatalog},
    world_map::{PixelPoint,ViewProjection,WorldCell},
};
use serde_json::{json,Value};
use std::{path::Path,net::SocketAddr,sync::atomic::AtomicBool,thread,time::{Duration,Instant}};
const RESOURCES:&str=r"D:\lineage開發\0-客戶端";
fn world()->World {
    let f:Value=serde_json::from_slice(include_bytes!("fixtures/native-world-current.json")).unwrap();
    let mut w=World::default();
    for key in ["world","start","status","actor","bot"] {
        let s=f["packets"][key].as_str().unwrap();
        let b:Vec<u8>=(0..s.len()).step_by(2).map(|i|u8::from_str_radix(&s[i..i+2],16).unwrap()).collect();
        w.accept(&b).unwrap();
    }
    w
}
#[test]
fn original_169_frames_sound_and_controls_are_exact() {
    let raw=SharedArchive::open(Path::new(RESOURCES),SharedArchiveKind::Text).unwrap().read_entry("list.spr").unwrap();
    let c=SpriteActionCatalog::parse_for_inspection(&raw,ActionLimits::default()).unwrap();
    let rows=qualify_travel(&c).unwrap();
    assert_eq!(rows.len(),1);assert_eq!(rows[0].0,169);
    assert_eq!(rows[0].2.frames.iter().map(|f|f.subframe).collect::<Vec<_>>(),[6,0,1,2,3,4,5]);
    assert_eq!(rows[0].2.frames[1].events,vec![FrameEvent::Sound(119)]);
    for change in 0..3 {
        let mut altered=c.clone();let row=altered.sprites.get_mut(&169).unwrap();
        match change {0=>row.image_count=Some(2),1=>row.attributes[0].body_bytes=b"7".to_vec(),_=>row.graphics_id=170}
        assert!(qualify_travel(&altered).is_err());
    }
}
#[test]
fn original_169_bitmap_sound_once_terminal_dwell_and_portal_retirement() {
    let mut w=world();let id=w.own_actor().unwrap().id;let a=w.own_actor().unwrap();
    let projection=ViewProjection{anchor_cell:WorldCell{x:a.x.into(),y:a.y.into()},anchor_pixel:PixelPoint{x:400,y:400}};
    let mut packet=vec![0x37];packet.extend(id.to_le_bytes());packet.extend(169u16.to_le_bytes());w.accept(&packet).unwrap();
    let before=serde_json::to_value(&w).unwrap();
    let mut fx=Effects::travel();fx.prepare(Path::new(RESOURCES),&w,&AtomicBool::new(false));
    let mut canvas=Canvas::new(800,600).unwrap();let baseline=canvas.rgba().to_vec();
    let (r,e)=fx.draw(&w,&mut canvas,projection,0);
    assert_eq!(r["load_error"],Value::Null);assert_eq!(r["started"],1);assert_eq!(r["effects"][0]["subframe"],0);
    assert_eq!(e.iter().filter_map(|e|e.sound).collect::<Vec<_>>(),[119]);assert_ne!(canvas.rgba(),baseline);
    assert!(fx.draw(&w,&mut canvas,projection,0).1.is_empty());
    for step in 1..6 { assert!(fx.draw(&w,&mut canvas,projection,step*41_000_000).1.is_empty()); }
    let (r,e)=fx.draw(&w,&mut canvas,projection,246_000_000);assert!(e.is_empty());
    assert_eq!(r["effects"][0]["subframe"],6);assert_eq!(r["active"],1);
    assert_eq!(fx.draw(&w,&mut canvas,projection,287_000_000).0["active"],0);
    assert_eq!(serde_json::to_value(&w).unwrap(),before);
    let mut portal=vec![0x5a];portal.extend(w.map_id.unwrap().to_le_bytes());portal.extend([32778u16,32820u16].into_iter().flat_map(u16::to_le_bytes));
    w.accept(&portal).unwrap();let (r,e)=fx.draw(&w,&mut canvas,projection,300_000_000);
    assert_eq!(r["active"],0);assert_eq!(r["started"],0);assert!(e.is_empty());
}
fn wait(c:&mut LoginController,seconds:u64,ok:impl Fn(&Snapshot)->bool)->Result<()> {
    let end=Instant::now()+Duration::from_secs(seconds);
    loop{c.poll();ensure!(!matches!(c.snapshot().phase,P::Failed|P::Closing|P::Closed),"client failed {:?}",c.snapshot().error);
        if ok(c.snapshot()){return Ok(())}ensure!(Instant::now()<end,"timeout at {:?}",c.snapshot().phase);thread::sleep(Duration::from_millis(10));}
}
fn capture_png(path:&Path,frame:&lito_modern::modern_lito::native_world_scene::Frame)->Result<()> {
    let mut encoder=png::Encoder::new(std::fs::File::create(path)?,800,600);
    encoder.set_color(png::ColorType::Rgba);encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(frame.canvas.rgba())?;Ok(())
}
fn player(root:&Path,peer:SocketAddr,role:&str,account:&str)->Result<()> {
    let mut c=LoginController::default();let cancel=AtomicBool::new(false);let start=Instant::now();
    let result=(||->Result<()>{
        c.start(peer,Limits{read_chunk:2,write_chunk:3,..Limits::default()})?;
        wait(&mut c,15,|s|s.phase==P::ReadyForLogin)?;c.login(account,"TestOnlyPassword")?;
        wait(&mut c,15,|s|s.phase==P::SourcesReady)?;c.select_source(1)?;
        wait(&mut c,15,|s|s.phase==P::RedirectReady)?;c.begin_game()?;
        let deadline=Instant::now()+Duration::from_secs(15);
        while c.snapshot().phase!=P::CharactersReady {
            c.poll();if c.snapshot().notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){c.acknowledge_notice()?;}
            ensure!(Instant::now()<deadline&&!matches!(c.snapshot().phase,P::Failed|P::Closing|P::Closed),"GAME bootstrap {:?}",c.snapshot().error);thread::sleep(Duration::from_millis(10));
        }
        c.select_character(0)?;wait(&mut c,15,|s|s.phase==P::CharacterSelected)?;
        c.enter_channel(0)?;wait(&mut c,15,|s|s.phase==P::LobbyReady)?;
        if role=="a" {
            c.create_room(&lito_modern::modern_lito::native_room::Settings{name:"座標傳送驗證",password:"",attributes:0,capacity:2,options:None})?;
            wait(&mut c,15,|s|s.phase==P::RoomReady)?;
            std::fs::write(root.join("travel-room-id"),c.snapshot().joined_room.as_ref().unwrap().id.to_string())?;
            wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.members.len()==2))?;
        } else {
            wait(&mut c,20,|_|root.join("travel-room-id").exists())?;
            let id=std::fs::read_to_string(root.join("travel-room-id"))?.parse::<u32>()?;
            wait(&mut c,15,|s|s.rooms.iter().any(|r|r.id==id))?;
            let index=c.snapshot().rooms.iter().position(|r|r.id==id).unwrap();c.join_room(index,"")?;
            wait(&mut c,15,|s|s.phase==P::RoomReady)?;c.set_force(1)?;wait(&mut c,15,|s|s.phase==P::RoomReady)?;
        }
        wait(&mut c,20,|s|s.joined_room.as_ref().is_some_and(|r|r.members.len()==2&&r.members.iter().any(|m|m.force==0)&&r.members.iter().any(|m|m.force==1)))?;
        c.set_ready(true)?;wait(&mut c,20,|s|s.phase==P::BattlePending&&s.world.ready())?;
        let mut model=Model::load(Path::new(RESOURCES),&c.snapshot().world,&cancel)?;
        if role=="b" {
            let before=c.snapshot().clone();let epoch=c.transport().snapshot().epoch;
            let mut observations=vec![];
            let end=Instant::now()+Duration::from_secs(60);
            loop {
                c.poll();let w=&c.snapshot().world;
                let row=json!({"generation":w.generation,"effects":w.effects,"remote":w.actors.get(&1)});
                if observations.last()!=Some(&row){observations.push(row);}
                if root.join("a-travel-complete").exists(){break;}
                ensure!(Instant::now()<end&&!matches!(c.snapshot().phase,P::Failed|P::Closing|P::Closed),"peer travel observation failed {:?}",c.snapshot().error);
                thread::sleep(Duration::from_millis(10));
            }
            let after=c.snapshot().clone();
            ensure!(after.world.ready()&&after.world.generation==before.world.generation
                &&c.transport().snapshot().epoch==epoch&&after.inventory.owner==before.inventory.owner,"unrelated peer world retired");
            let frame=model.draw_at(&after.world,&cancel,start.elapsed().as_nanos() as u64)?;
            ensure!(frame.input_ready(&after.world)&&frame.movement_ready(&after.world),"peer scene unusable");
            capture_png(&root.join("b-peer-effect169.png"),&frame)?;
            std::fs::write(root.join("b-travel.json"),serde_json::to_vec_pretty(&json!({"role":role,"epoch":epoch,"before":before,"after":after,"observations":observations,"frame":frame.receipt,"original_memory_fixture_unchanged":true}))?)?;
            std::fs::write(root.join("b-travel-complete"),b"done")?;
            return Ok(());
        }
        let before=c.snapshot().clone();let epoch=c.transport().snapshot().epoch;
        let item=before.inventory.items.values().find(|i|i.use_type==5&&i.icon==828).context("SQL-derived travel wand absent")?.clone();
        let cursor=TargetChoice::capture(&before,item.id)?;
        let frame=model.draw_at(&before.world,&cancel,start.elapsed().as_nanos() as u64)?;
        capture_png(&root.join("a-before.png"),&frame)?;
        ensure!(frame.input_ready(&before.world)&&frame.movement_ready(&before.world),"initial frame not ready");
        let own=before.world.own_actor().unwrap();
        std::fs::write(root.join("a-travel-before.json"),serde_json::to_vec_pretty(&before)?)?;
        let blocked=[own.x.checked_add(1).context("fixture x")?,own.y];
        ensure!(frame.navigation.standing(blocked[0].into(),blocked[1].into()),"fixture rejection cell has no native ground");
        c.item_target_action(cursor.clone(),0,blocked,&frame)?;
        ensure!(c.item_target_action(cursor.clone(),0,blocked,&frame).is_err(),"duplicate pending request admitted");
        wait(&mut c,8,|s|s.item_feedback==lito_modern::modern_lito::native_item_actions::Feedback::TargetRejected)?;
        let rejected=c.snapshot().clone();
        ensure!(rejected.world.generation==before.world.generation&&rejected.world.own_actor().is_some_and(|a|[a.x,a.y]==[own.x,own.y])
            &&rejected.inventory.items.get(&item.id)==Some(&item),"rejected target changed authority");
        let frame=model.draw_at(&rejected.world,&cancel,start.elapsed().as_nanos() as u64)?;
        let cursor=TargetChoice::capture(&rejected,item.id)?;
        let position=[(6,2),(6,-2),(4,4),(0,6),(-4,4)].into_iter()
            .map(|(dx,dy)|(i32::from(own.x)+dx,i32::from(own.y)+dy))
            .find(|&(x,y)|frame.navigation.standing(x,y)&&rejected.world.actors.values().all(|a|a.dead||i32::from(a.x)!=x||i32::from(a.y)!=y)).context("standing destination absent")?;
        let position=[u16::try_from(position.0)?,u16::try_from(position.1)?];
        c.item_target_action(cursor.clone(),0,position,&frame)?;
        ensure!(c.item_target_action(cursor.clone(),0,position,&frame).is_err(),"duplicate pending item request admitted");
        let mut transitions=vec![];let deadline=Instant::now()+Duration::from_secs(15);
        loop {
            c.poll();let w=&c.snapshot().world;
            if transitions.last().is_none_or(|v:&Value|v["generation"]!=w.generation||v["ready"]!=w.ready()) {
                transitions.push(json!({"generation":w.generation,"ready":w.ready(),"portal":w.portal,"actors":w.actors.len(),"effects":w.effects}));
            }
            if w.generation>before.world.generation&&w.ready()&&w.own_actor().is_some_and(|a|[a.x,a.y]==position){break;}
            ensure!(Instant::now()<deadline&&!matches!(c.snapshot().phase,P::Failed|P::Closing|P::Closed),"portal did not complete {:?}",c.snapshot().error);thread::sleep(Duration::from_millis(10));
        }
        let after=c.snapshot().clone();
        ensure!(c.transport().snapshot().epoch==epoch&&after.inventory.owner==before.inventory.owner,"same-map portal changed connection/owner");
        ensure!(after.inventory.items.get(&item.id)==Some(&item),"GM wand was predicted/consumed");
        ensure!(!frame.input_ready(&after.world)&&cursor.validate(&after).is_err(),"old scene authorized target");
        model=Model::load(Path::new(RESOURCES),&after.world,&cancel)?;
        let new_frame=model.draw_at(&after.world,&cancel,start.elapsed().as_nanos() as u64)?;
        capture_png(&root.join("a-after.png"),&new_frame)?;
        ensure!(new_frame.input_ready(&after.world)&&new_frame.movement_ready(&after.world),"new scene not usable");
        // Prove actual gameplay continuation, not just a ready-looking frame.
        let actor=after.world.own_actor().unwrap();
        let heading=[0u8,4,6,2].into_iter().find(|h|new_frame.navigation.can_step(actor.x.into(),actor.y.into(),*h))
            .context("post-portal walking direction")?;
        let (x,y)=lito_modern::modern_lito::native_world::destination(actor.x,actor.y,heading)?;
        c.move_checked(heading,&new_frame.navigation)?;
        wait(&mut c,8,|s|s.world.own_actor().is_some_and(|a|[a.x,a.y]==[x,y]))?;
        let continued=c.snapshot().clone();
        ensure!(continued.world.generation==after.world.generation&&continued.inventory.owner==after.inventory.owner,"walking changed world owner");
        std::fs::write(root.join(format!("{role}-travel.json")),serde_json::to_vec_pretty(&json!({"role":role,"epoch":epoch,"item":item,"position":position,"before":before,"rejected":rejected,"blocked_position":blocked,"after":after,"continued":continued,"walk_heading":heading,"transitions":transitions,"old_frame_rejected":true,"old_cursor_rejected":true,"new_frame":new_frame.receipt}))?)?;
        std::fs::write(root.join(format!("{role}-travel-complete")),b"done")?;
        let other=if role=="a"{"b"}else{"a"};wait(&mut c,30,|_|root.join(format!("{other}-travel-complete")).exists())?;
        Ok(())
    })();
    c.cancel();let end=Instant::now()+Duration::from_secs(3);
    while !c.is_quiescent()&&Instant::now()<end{c.poll();thread::sleep(Duration::from_millis(10));}
    ensure!(c.is_quiescent(),"client worker not reclaimed");result
}
#[test]
#[ignore="requires owned SQL plus untouched formal-session travel harness"]
fn actual_sql_wand_portal_retires_owner_and_preserves_peer_scene() {
    let root=std::path::PathBuf::from(std::env::var("LITO_TRAVEL_ROOT").expect("owned root"));
    assert!(root.canonicalize().unwrap().starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current").canonicalize().unwrap()));
    let ready:Value=serde_json::from_slice(&std::fs::read(root.join("travel-ready.json")).unwrap()).unwrap();
    assert_eq!(ready["jar_sha256"],"c02aaae8d54a27e30460be2e3c47709ee64d35f007a8be27ac74a327268b9539");
    let peer:SocketAddr=format!("127.0.0.1:{}",ready["port"].as_u64().unwrap()).parse().unwrap();
    let (ar,br)=thread::scope(|scope|{let a=scope.spawn(||player(&root,peer,"a","PeerFixture"));let b=scope.spawn(||player(&root,peer,"b","PeerFixture2"));(a.join().unwrap(),b.join().unwrap())});
    std::fs::write(root.join("travel-test-done"),b"done").unwrap();
    assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");
}
