"""Real formal configuration refusal: retain arena, authority and unplayed cue."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];dst=ROOT/'維護工具/candidates/native-audio-world-tree-refused';dst.mkdir(exist_ok=False)
source=(ROOT/'維護工具/candidates/native-audio-world-tree/probe.rs').read_text(encoding='utf-8')
prefix=source.split('fn player(')[0]
tail=r'''
fn player(root:&Path,peer:SocketAddr,role:&str)->Result<()> {
    use m::native_login::LoginPhase as P;
    let mut p=Peer::default();
    let result=(||->Result<()> {
        enter(&mut p,peer,role)?;barrier(&mut p,root,role,"source-ready")?;
        let source_audio=p.heal_audio(true)?;p.ready(8)?;
        let before=p.c.snapshot().clone();let epoch=p.c.transport().snapshot().epoch;
        ensure!(before.world.learned_skills.contains(&131),"initial fixture skill131 absent");
        let cues=std::mem::take(&mut p.held);ensure!(cues.len()==1,"real unconsumed heal cue absent");
        let source_picture=p.picture(root,role,"source")?;
        barrier(&mut p,root,role,"captured")?;
        let mut refusals=vec![];
        for _ in 0..2 {
            p.ready(8)?;let prior=p.c.snapshot().clone();
            let request=m::native_skills::request(&prior.world,131,None)?;
            p.c.cast(131,None,p.view.frame.as_ref().context("tree frame")?)?;
            let immediate=p.c.snapshot().clone();
            let duplicate_rejected=p.c.cast(131,None,p.view.frame.as_ref().unwrap()).is_err();ensure!(duplicate_rejected,"duplicate pending tree request accepted");
            p.wait(8,|p|p.c.snapshot().world.message_count>prior.world.message_count&&p.c.snapshot().skill_feedback==m::native_skills::Feedback::Rejected(276))?;
            let after=p.c.snapshot().clone();
            ensure!(after.phase==P::BattlePending&&after.world.generation==before.world.generation&&p.c.transport().snapshot().epoch==epoch&&after.world.map_id==Some(564)&&after.world.portal.is_none()&&after.world.completed_portals==before.world.completed_portals,"refusal retired arena");
            ensure!(after.inventory==prior.inventory&&after.world.mp==prior.world.mp&&after.world.hp==prior.world.hp&&after.world.own_cast_packets==prior.world.own_cast_packets&&after.selected_character==prior.selected_character,"refused request changed authority");
            refusals.push(json!({"before":prior,"immediate":immediate,"after":after,"duplicate_rejected":duplicate_rejected,"request_hex":request.iter().map(|b|format!("{b:02x}")).collect::<String>()}));
        }
        p.ready(8)?;let after=p.c.snapshot().clone();
        let scope=p.view.audio_scope(epoch,after.world.generation);let count=p.sink.played.len();let mut outcomes=vec![];
        for cue in &cues {
            let first=cue.submit(&scope,&mut p.sink)?;let duplicate=cue.clone().submit(&scope,&mut p.sink)?;
            ensure!(matches!(first,m::native_audio_delivery::DeliveryOutcome::Played(m::combat_audio::PlayOutcome::Started{sound_id:109,..}))&&duplicate==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"refusal discarded or replayed original cue");
            outcomes.push(json!({"first":format!("{first:?}"),"duplicate":format!("{duplicate:?}")}));
        }
        ensure!(p.sink.played.len()==count+1,"original cue did not play exactly once");
        p.defer_audio=false;
        let f=p.view.frame.as_ref().context("retained arena frame")?;let own=p.c.snapshot().world.own_actor().context("arena owner")?;
        let heading=(0..8).find(|h|f.navigation.can_step(own.x.into(),own.y.into(),*h)&&m::native_world::destination(own.x,own.y,*h).is_ok_and(|(x,y)|!p.c.snapshot().world.actors.values().any(|a|a.id!=own.id&&!a.dead&&a.x==x&&a.y==y))).context("unoccupied native step")?;
        let target=m::native_world::destination(own.x,own.y,heading)?;p.c.move_checked(heading,&f.navigation.clone())?;
        p.wait(8,|p|p.c.snapshot().world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;p.ready(8)?;
        ensure!(p.c.snapshot().world.local_corrections==after.world.local_corrections,"post-refusal move corrected");
        let followup_audio=p.heal_audio(false)?;p.ready(8)?;
        let retained_picture=p.picture(root,role,"retained")?;
        std::fs::write(root.join(format!("{role}-result.json")),serde_json::to_vec_pretty(&json!({"scope":"C02 classic world compatibility disabled: two explicit skill131 refusals276 retain original arena and unconsumed sound109","role":role,"epoch":epoch,"before":before,"after_refusal":after,"after_walk":p.c.snapshot(),"walk_target":target,"source_audio":source_audio,"followup_audio":followup_audio,"refusals":refusals,"held_cue_outcomes":outcomes,"retained_cue_deliveries":1,"source_picture":source_picture,"retained_picture":retained_picture,"trace":p.trace,"GUI_verified":false,"physical_audio_verified":false}))?)?;
        barrier(&mut p,root,role,"finished")?;Ok(())
    })();
    let closed=p.close();std::fs::write(root.join(format!("{role}-shutdown.json")),serde_json::to_vec_pretty(&json!({"error":result.as_ref().err().map(|e|format!("{e:#}")),"close_error":closed.as_ref().err().map(|e|format!("{e:#}")),"client_quiescent":p.c.is_quiescent(),"view_quiescent":p.view.quiescent(),"trace":p.trace}))?)?;result?;closed
}
#[test]
#[ignore="requires owned C02 world-tree-disabled fixture"]
fn actual_battle_audio_world_tree_refused() {
    let root=std::path::PathBuf::from(std::env::var("LITO_AUDIO_WORLD_TREE_REFUSED_ROOT").unwrap()).canonicalize().unwrap();
    assert!(root.starts_with(Path::new(env!("CARGO_MANIFEST_DIR")).join("target/current/native-audio-world-tree-refused").canonicalize().unwrap()));
    let ready:Value=serde_json::from_slice(&std::fs::read(root.join("endpoints.json")).unwrap()).unwrap();let peer=ready["source"].as_str().unwrap().parse().unwrap();
    thread::scope(|s|{let a=s.spawn(||player(&root,peer,"a"));let b=s.spawn(||player(&root,peer,"b"));let ar=a.join().unwrap();let br=b.join().unwrap();assert!(ar.is_ok()&&br.is_ok(),"a={ar:?}; b={br:?}");});
}
'''
(dst/'probe.rs').write_text(prefix+tail,encoding='utf-8',newline='\n')
s=(ROOT/'維護工具/compile-native-audio-world-tree.py').read_text(encoding='utf-8').replace('native-audio-world-tree','native-audio-world-tree-refused').replace('native_audio_world_tree_probe','native_audio_world_tree_refused_probe')
(ROOT/'維護工具/compile-native-audio-world-tree-refused.py').write_text(s,encoding='utf-8',newline='\n')
s=(ROOT/'維護工具/run-native-audio-world-tree.py').read_text(encoding='utf-8').replace('native-audio-world-tree','native-audio-world-tree-refused').replace('LITO_AUDIO_WORLD_TREE_ROOT','LITO_AUDIO_WORLD_TREE_REFUSED_ROOT').replace('actual_battle_audio_world_tree_same_connection','actual_battle_audio_world_tree_refused').replace("str(work),'world-entry'","str(work),'world-tree-disabled'")
s=s.replace("assert j['old_cue_deliveries']==0 and j['old_surface_rejected']","assert j['retained_cue_deliveries']==1 and len(j['refusals'])==2").replace("assert j['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]","assert len(j['held_cue_outcomes'])==1 and j['held_cue_outcomes'][0]['first'].startswith('Played(Started { sound_id: 109,') and j['held_cue_outcomes'][0]['duplicate']=='AlreadyConsumed'")
s=s.replace("scope='same GAME connection arena→WORLD skill131 with real deferred heal cue, character/items and movement; not C72 endpoint handoff'","scope='two real world-tree-disabled refusals276 retain arena and one original unconsumed cue; movement and fresh heal succeed'")
(ROOT/'維護工具/run-native-audio-world-tree-refused.py').write_text(s,encoding='utf-8',newline='\n')
print(dst)
