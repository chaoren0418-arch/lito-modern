"""Real C02 arena→WORLD scene/audio retirement on the same GAME connection."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];dst=ROOT/'維護工具/candidates/native-audio-world-tree';dst.mkdir(exist_ok=False)
src=(ROOT/'維護工具/candidates/native-audio-reconnect/probe.rs').read_text(encoding='utf-8')
prefix=src.split('fn player(')[0]
tail=r'''
fn player(root:&Path,peer:SocketAddr,role:&str)->Result<()> {
    use m::native_login::LoginPhase as P;
    let mut p=Peer::default();
    let result=(||->Result<()> {
        enter(&mut p,peer,role)?;barrier(&mut p,root,role,"source-ready")?;
        let source_audio=p.heal_audio(true)?;p.ready(8)?;
        let before=p.c.snapshot().clone();let epoch=p.c.transport().snapshot().epoch;
        ensure!(before.world.learned_skills.contains(&131),"initial world tree fixture book absent");
        let old_surface=p.view.frame.as_ref().context("source frame")?.input_surface(&before.world).context("input surface")?;
        let old_cues=std::mem::take(&mut p.held);ensure!(old_cues.len()==1,"genuine unconsumed heal cue required");
        let source_picture=p.picture(root,role,"source")?;
        barrier(&mut p,root,role,"captured")?;p.ready(8)?;
        let tree_request=m::native_skills::request(&p.c.snapshot().world,131,None)?;
        p.c.cast(131,None,p.view.frame.as_ref().context("tree frame")?)?;
        let duplicate_rejected=p.c.cast(131,None,p.view.frame.as_ref().unwrap()).is_err();ensure!(duplicate_rejected,"duplicate world tree request accepted");
        p.defer_audio=false;
        p.wait(20,|p|p.c.snapshot().phase==P::WorldReady&&p.c.snapshot().world.ready()&&p.c.snapshot().world.actors.contains_key(&if role=="a"{3}else{1}))?;
        let after=p.c.snapshot().clone();let new_epoch=p.c.transport().snapshot().epoch;
        ensure!(new_epoch==epoch&&after.world.generation==before.world.generation+1&&after.world.map_id==Some(4)&&after.world.completed_portals==1,"same-connection WORLD generation mismatch");
        ensure!(after.joined_room.is_none()&&after.channel.is_none()&&after.rooms.is_empty(),"old arena membership retained");
        ensure!(after.selected_character==before.selected_character&&after.inventory.owner==before.inventory.owner&&after.inventory.items==before.inventory.items,"WORLD character/item authority changed");
        ensure!(!old_surface.input_ready(&after.world),"old arena input accepted WORLD");
        let count=p.sink.played.len();let scope=p.view.audio_scope(new_epoch,after.world.generation);let mut outcomes=vec![];
        for cue in &old_cues {
            let first=cue.submit(&scope,&mut p.sink)?;let duplicate=cue.clone().submit(&scope,&mut p.sink)?;
            ensure!(first==m::native_audio_delivery::DeliveryOutcome::Stale&&duplicate==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"old arena cue replayed");
            outcomes.push(json!({"first":format!("{first:?}"),"duplicate":format!("{duplicate:?}")}));
        }
        ensure!(p.sink.played.len()==count,"old arena cue delivered");
        p.ready(20)?;barrier(&mut p,root,role,"world-ready")?;p.ready(8)?;
        let destination_audio=p.heal_audio(false)?;p.ready(8)?;
        let f=p.view.frame.as_ref().context("WORLD frame")?;let own=p.c.snapshot().world.own_actor().context("WORLD owner")?;
        let heading=if role=="a"{2}else{6};ensure!(f.navigation.can_step(own.x.into(),own.y.into(),heading),"WORLD step blocked");
        let target=m::native_world::destination(own.x,own.y,heading)?;p.c.move_checked(heading,&f.navigation.clone())?;
        p.wait(8,|p|p.c.snapshot().world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;p.ready(8)?;
        ensure!(p.c.snapshot().world.local_corrections==after.world.local_corrections,"WORLD step corrected");
        let destination_picture=p.picture(root,role,"destination")?;
        barrier(&mut p,root,role,"walked")?;
        p.wait(8,|p|p.c.snapshot().world.actors.get(&if role=="a"{3}else{1}).is_some_and(|a|[a.x,a.y]!=[33051,32337]))?;
        std::fs::write(root.join(format!("{role}-result.json")),serde_json::to_vec_pretty(&json!({"scope":"real C02 arena→WORLD skill131, same GAME connection; not C72 endpoint handoff","role":role,"old_epoch":epoch,"new_epoch":new_epoch,"before":before,"after":after,"after_walk":p.c.snapshot(),"walk_target":target,"source_audio":source_audio,"destination_audio":destination_audio,"tree_request_hex":tree_request.iter().map(|b|format!("{b:02x}")).collect::<String>(),"duplicate_tree_rejected":duplicate_rejected,"held_cue_outcomes":outcomes,"old_cue_deliveries":0,"old_surface_rejected":true,"source_picture":source_picture,"destination_picture":destination_picture,"trace":p.trace,"GUI_verified":false,"physical_audio_verified":false}))?)?;
        barrier(&mut p,root,role,"finished")?;Ok(())
    })();
    let closed=p.close();std::fs::write(root.join(format!("{role}-shutdown.json")),serde_json::to_vec_pretty(&json!({"error":result.as_ref().err().map(|e|format!("{e:#}")),"close_error":closed.as_ref().err().map(|e|format!("{e:#}")),"client_quiescent":p.c.is_quiescent(),"view_quiescent":p.view.quiescent(),"trace":p.trace}))?)?;result?;closed
}
#[test]
#[ignore="requires owned ordinary C02 world-entry fixture"]
fn actual_battle_audio_world_tree_same_connection() {
    let root=std::path::PathBuf::from(std::env::var("LITO_AUDIO_WORLD_TREE_ROOT").unwrap()).canonicalize().unwrap();
    assert!(root.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native-audio-world-tree").canonicalize().unwrap()));
    let ready:Value=serde_json::from_slice(&std::fs::read(root.join("endpoints.json")).unwrap()).unwrap();let peer=ready["source"].as_str().unwrap().parse().unwrap();
    thread::scope(|s|{let a=s.spawn(||player(&root,peer,"a"));let b=s.spawn(||player(&root,peer,"b"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
'''
(dst/'probe.rs').write_text(prefix+tail,encoding='utf-8',newline='\n')
s=(ROOT/'維護工具/compile-native-audio-reconnect.py').read_text(encoding='utf-8').replace('native-audio-reconnect','native-audio-world-tree').replace('native_audio_reconnect_probe','native_audio_world_tree_probe')
(ROOT/'維護工具/compile-native-audio-world-tree.py').write_text(s,encoding='utf-8',newline='\n')
s=(ROOT/'維護工具/run-native-audio-reconnect.py').read_text(encoding='utf-8').replace('native-audio-reconnect','native-audio-world-tree').replace('LITO_AUDIO_RECONNECT_ROOT','LITO_AUDIO_WORLD_TREE_ROOT').replace('actual_battle_audio_cancel_and_reconnect','actual_battle_audio_world_tree_same_connection').replace("['source','destination']","['source']").replace("str(work),'heal'","str(work),'world-entry'").replace('actual_sessions=8','actual_sessions=4').replace("state['actual_sessions']=8","state['actual_sessions']=4").replace('actual_sessions=8','actual_sessions=4')
s=s.replace("scope='explicit cancel/reconnect between independent ordinary battle endpoints; no C72 handoff or authority continuity qualification'","scope='same GAME connection arena→WORLD skill131 with real deferred heal cue, character/items and movement; not C72 endpoint handoff'")
(ROOT/'維護工具/run-native-audio-world-tree.py').write_text(s,encoding='utf-8',newline='\n')
print(dst)
