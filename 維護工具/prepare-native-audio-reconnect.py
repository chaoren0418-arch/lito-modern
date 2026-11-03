"""Prepare a genuine battle-audio reconnect diagnostic, separate from C72 travel."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
dst=ROOT/'維護工具/candidates/native-audio-reconnect';dst.mkdir(exist_ok=False)
src=(ROOT/'維護工具/candidates/native-teleport-audio/probe.rs').read_text(encoding='utf-8')
prefix=src.split('fn player(')[0].replace('&& self.sink.played.iter().filter(|&&id| id == 109).count() == delivered,','&& self.deliveries.len() == own_delivered,')
tail=r'''
fn enter(p: &mut Peer, endpoint: SocketAddr, role: &str) -> Result<()> {
    use m::native_login::LoginPhase as P;
    ensure!(endpoint.ip().is_loopback(), "loopback only");
    p.c.start(endpoint, m::native_connection::Limits {read_chunk:2,write_chunk:3,..Default::default()})?;
    let end=Instant::now()+Duration::from_secs(60);
    let mut submitted=false;let mut force_requested=false;
    let mut refresh=Instant::now()-Duration::from_secs(2);
    loop {
        p.tick()?;
        let s=p.c.snapshot();
        ensure!(!matches!(s.phase,P::Failed|P::Closed),"bootstrap failed {:?}",s.error);
        if s.notice.as_ref().is_some_and(|n|n.acknowledgement().is_some()){p.c.acknowledge_notice()?;}
        match p.c.snapshot().phase {
            P::ReadyForLogin if !submitted => {p.c.login(if role=="a"{"PeerFixture"}else{"PeerFixture2"},"TestOnlyPassword")?;submitted=true;},
            P::ReadyForLogin => anyhow::bail!("login rejected"),
            P::SourcesReady=>p.c.select_source(1)?,
            P::RedirectReady=>p.c.begin_game()?,
            P::CharactersReady=>p.c.select_character(0)?,
            P::CharacterSelected=>p.c.enter_channel(0)?,
            P::LobbyReady=>if p.c.snapshot().rooms.is_empty(){if refresh.elapsed()>Duration::from_secs(1){p.c.refresh_rooms()?;refresh=Instant::now();}}else{p.c.join_room(0,"")?;force_requested=false;},
            P::RoomReady=>{
                let s=p.c.snapshot();let room=s.joined_room.as_ref().context("room")?;
                let own=room.members.iter().find(|m|Some(m.name.as_str())==s.selected_character.as_deref()).context("membership")?;
                let force=if role=="a"{0}else{1};
                if own.force!=force&&!force_requested{p.c.set_force(force)?;force_requested=true;}
                else if own.force==force&&room.members.len()==2&&room.members.iter().any(|m|m.force!=force)&&own.state==0{p.c.set_ready(true)?;}
            },
            P::BattlePending|P::WorldReady=>if p.c.snapshot().world.ready()&&p.c.snapshot().world.actors.contains_key(&if role=="a"{3}else{1}){p.ready(20)?;return Ok(());},
            _=>{}
        }
        ensure!(Instant::now()<end,"existing60s peer/bootstrap bound");
        thread::sleep(Duration::from_millis(10));
    }
}
fn barrier(p:&mut Peer,root:&Path,role:&str,stage:&str)->Result<()> {
    std::fs::write(root.join(format!("{role}-{stage}")),b"ready")?;
    let other=if role=="a"{"b"}else{"a"};
    p.wait(60,|_|root.join(format!("{other}-{stage}")).exists())
}
fn player(root:&Path,source:SocketAddr,destination:SocketAddr,role:&str)->Result<()> {
    let mut p=Peer::default();
    let result=(||->Result<()> {
        enter(&mut p,source,role)?;
        barrier(&mut p,root,role,"source-ready")?;
        let before=p.c.snapshot().clone();
        let old_epoch=p.c.transport().snapshot().epoch;
        let old_peer=p.c.transport().snapshot().negotiated.as_ref().context("source peer")?.peer;
        let old_surface=p.view.frame.as_ref().context("source frame")?.input_surface(&before.world).context("surface")?;
        let source_audio=p.heal_audio(true)?;
        let source_picture=p.picture(root,role,"source")?;
        let old_cues=std::mem::take(&mut p.held);
        ensure!(old_cues.len()==1,"genuine unconsumed source cue required");
        barrier(&mut p,root,role,"captured")?;
        p.close()?;
        ensure!(p.view.frame.is_none(),"cancel retained source frame");
        let closed=json!({"frame_absent":true,"client_quiescent":p.c.is_quiescent(),"view_quiescent":p.view.quiescent()});
        p.defer_audio=false;
        enter(&mut p,destination,role)?;
        let after=p.c.snapshot().clone();let new_epoch=p.c.transport().snapshot().epoch;
        let new_peer=p.c.transport().snapshot().negotiated.as_ref().context("destination peer")?.peer;
        ensure!(old_peer!=new_peer&&old_epoch!=new_epoch,"actual endpoint/epoch unchanged");
        ensure!(!old_surface.input_ready(&after.world),"old surface accepted new scene");
        let scope=p.view.audio_scope(new_epoch,after.world.generation);
        let count=p.sink.played.len();let mut outcomes=vec![];
        for cue in &old_cues {
            let first=cue.submit(&scope,&mut p.sink)?;
            let duplicate=cue.clone().submit(&scope,&mut p.sink)?;
            ensure!(first==m::native_audio_delivery::DeliveryOutcome::Stale,"first held cue was not stale: {first:?}");
            ensure!(duplicate==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"duplicate replay");
            outcomes.push(json!({"first":format!("{first:?}"),"duplicate":format!("{duplicate:?}")}));
        }
        ensure!(p.sink.played.len()==count,"old cue delivered");
        barrier(&mut p,root,role,"destination-ready")?;
        let destination_audio=p.heal_audio(false)?;
        let destination_picture=p.picture(root,role,"destination")?;
        barrier(&mut p,root,role,"finished")?;
        std::fs::write(root.join(format!("{role}-result.json")),serde_json::to_vec_pretty(&json!({"scope":"explicit cancel/reconnect between independent C02 battle endpoints; not C72 handoff or authority continuity","role":role,"old_epoch":old_epoch,"new_epoch":new_epoch,"old_peer":old_peer.to_string(),"new_peer":new_peer.to_string(),"before":before,"after":after,"source_audio":source_audio,"destination_audio":destination_audio,"held_cue_outcomes":outcomes,"old_cue_deliveries":0,"old_surface_rejected":true,"closed":closed,"source_picture":source_picture,"destination_picture":destination_picture,"trace":p.trace,"GUI_verified":false,"physical_audio_verified":false}))?)?;
        Ok(())
    })();
    let close=p.close();
    std::fs::write(root.join(format!("{role}-shutdown.json")),serde_json::to_vec_pretty(&json!({"error":result.as_ref().err().map(|e|format!("{e:#}")),"close_error":close.as_ref().err().map(|e|format!("{e:#}")),"client_quiescent":p.c.is_quiescent(),"view_quiescent":p.view.quiescent(),"trace":p.trace}))?)?;
    result?;close
}
#[test]
#[ignore="requires two owned ordinary C02 battle servers"]
fn actual_battle_audio_cancel_and_reconnect() {
    let root=std::path::PathBuf::from(std::env::var("LITO_AUDIO_RECONNECT_ROOT").unwrap()).canonicalize().unwrap();
    assert!(root.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native-audio-reconnect").canonicalize().unwrap()));
    let ready:Value=serde_json::from_slice(&std::fs::read(root.join("endpoints.json")).unwrap()).unwrap();
    let source=ready["source"].as_str().unwrap().parse().unwrap();let destination=ready["destination"].as_str().unwrap().parse().unwrap();
    thread::scope(|s|{let a=s.spawn(||player(&root,source,destination,"a"));let b=s.spawn(||player(&root,source,destination,"b"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
'''
(dst/'probe.rs').write_text(prefix+tail,encoding='utf-8',newline='\n')
compiler=(ROOT/'維護工具/compile-native-teleport-audio.py').read_text(encoding='utf-8').replace('native-teleport-audio','native-audio-reconnect').replace('native_teleport_audio_probe','native_audio_reconnect_probe')
(ROOT/'維護工具/compile-native-audio-reconnect.py').write_text(compiler,encoding='utf-8',newline='\n')
print(dst)
