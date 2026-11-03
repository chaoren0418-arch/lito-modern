"""Extend the sealed three-peer flow with a real WORLD map4→map4 cycle."""
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1];T=ROOT/'維護工具';C=T/'candidates/native-class2-presentation'
s=(C/'live_probe_world.rs').read_text(encoding='utf-8-sig')
s=s.replace('target/current/native-class2-world','target/current/native-class2-same-map')
anchor='        let destination_picture=p.picture(&root,role,"destination")?;';assert s.count(anchor)==1
s=s.replace(anchor,anchor+'\n        let after_walk=p.c.snapshot().clone();\n        let same_map=same_map_cycle(&mut p,&root,role)?;')
s=s.replace('"after_walk":p.c.snapshot(),','"after_walk":after_walk,"same_map":same_map,')
s+=r'''
fn same_map_cycle(p:&mut Peer,root:&Path,role:&str)->Result<Value> {
    use m::native_login::LoginPhase as P;
    p.ready(8)?;barrier(p,root,role,"same-source-ready")?;
    let source_audio=p.heal_audio(true)?;p.ready(8)?;
    let before=p.c.snapshot().clone();let epoch=p.c.transport().snapshot().epoch;
    ensure!(before.phase==P::WorldReady&&before.world.map_id==Some(4),"same-map WORLD source");
    let old_surface=p.view.frame.as_ref().context("same-map source frame")?.input_surface(&before.world).context("same-map input")?;
    let old_cues=std::mem::take(&mut p.held);ensure!(old_cues.len()==1,"one deferred real109 required");
    let source_picture=p.picture(root,role,"same-source")?;
    let request=m::native_skills::request(&before.world,131,None)?;
    p.c.cast(131,None,p.view.frame.as_ref().unwrap())?;
    let immediate=p.c.snapshot().clone();
    ensure!(immediate.world.hp==before.world.hp&&immediate.world.mp==before.world.mp&&immediate.inventory.items==before.inventory.items,"same-map queue predicted authority");
    p.wait(20,|p|p.c.snapshot().phase==P::WorldReady&&p.c.snapshot().world.ready()
        &&p.c.snapshot().world.generation==before.world.generation+1)?;
    let after=p.c.snapshot().clone();
    ensure!(p.c.transport().snapshot().epoch==epoch&&after.world.map_id==Some(4)
        &&after.world.completed_portals==before.world.completed_portals+1,"same-map generation/epoch/portal");
    ensure!(after.inventory.items==before.inventory.items&&after.inventory.owner==before.inventory.owner&&after.selected_character==before.selected_character,"same-map authority continuity");
    ensure!(!old_surface.input_ready(&after.world),"same-map old input accepted");
    ensure!(after.environment.ambient_override_counter==0,"same-map ambient lease retained");
    let count=p.sink.played.len();let scope=p.view.audio_scope(epoch,after.world.generation);let mut outcomes=vec![];
    for cue in old_cues {
        let first=cue.submit(&scope,&mut p.sink)?;let duplicate=cue.submit(&scope,&mut p.sink)?;
        ensure!(first==m::native_audio_delivery::DeliveryOutcome::Stale&&duplicate==m::native_audio_delivery::DeliveryOutcome::AlreadyConsumed,"same-map old109 outcomes {first:?}/{duplicate:?}");
        outcomes.push(json!({"first":format!("{first:?}"),"duplicate":format!("{duplicate:?}")}));
    }
    ensure!(p.sink.played.len()==count,"same-map old109 delivered");
    p.defer_audio=false;p.ready(20)?;barrier(p,root,role,"same-destination-ready")?;
    let destination_audio=p.heal_audio(false)?;p.ready(8)?;
    let frame=p.view.frame.as_ref().unwrap();let own=p.c.snapshot().world.own_actor().context("same-map destination owner")?;
    let heading=if role=="a"{2}else{6};ensure!(frame.navigation.can_step(own.x.into(),own.y.into(),heading),"same-map step blocked");
    let target=m::native_world::destination(own.x,own.y,heading)?;p.c.move_checked(heading,&frame.navigation)?;
    p.wait(8,|p|p.c.snapshot().world.own_actor().is_some_and(|a|(a.x,a.y)==target))?;p.ready(8)?;
    ensure!(p.c.snapshot().world.local_corrections==after.world.local_corrections,"same-map movement corrected");
    Ok(json!({"scope":"actual C02 WORLD map4→map4 S5A, same GAME connection; source109 genuinely deferred","before":before,"immediate_tree_queue":immediate,"after":after,"after_walk":p.c.snapshot(),"old_epoch":epoch,"new_epoch":p.c.transport().snapshot().epoch,"tree_request_hex":request.iter().map(|b|format!("{b:02x}")).collect::<String>(),"held_cue_outcomes":outcomes,"old_surface_rejected":true,"old_cue_deliveries":0,"source_audio":source_audio,"destination_audio":destination_audio,"source_picture":source_picture,"destination_picture":p.picture(root,role,"same-destination")?,"walk_target":target,"actual_active_class2_flash_on_same_map":false,"GUI_verified":false,"physical_audio_verified":false}))
}
'''
path=C/'live_probe_same_map.rs';assert not path.exists();path.write_text(s,encoding='utf-8',newline='\n')
s=(T/'run-native-class2-world.py').read_text(encoding='utf-8-sig')
s=s.replace("ROOT/'驗證紀錄/current/native-class2-world'/stamp","ROOT/'驗證紀錄/current/native-class2-same-map'/stamp")
s=s.replace("PROJECT/'target/current/native-class2-world'/stamp","PROJECT/'target/current/native-class2-same-map'/stamp")
s=s.replace("entry.name=='live_probe_world.rs'","entry.name=='live_probe_same_map.rs'")
s=s.replace('actual_requests=5,actual_casts=1,actual_world_tree_transitions=2','actual_requests=11,actual_casts=1,actual_world_tree_transitions=4')
anchor="        assert report['source34_sink_calls']==0";assert anchor in s
s=s.replace(anchor,anchor+"\n        same=report['same_map'];assert same['before']['world']['map_id']==same['after']['world']['map_id']==4\n        assert same['after']['world']['generation']==same['before']['world']['generation']+1\n        assert same['held_cue_outcomes']==[dict(first='Stale',duplicate='AlreadyConsumed')]")
(T/'run-native-class2-same-map.py').write_text(s,encoding='utf-8',newline='\n')
print(path)
