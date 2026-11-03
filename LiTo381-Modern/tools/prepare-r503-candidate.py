from r491_common import *
import shutil,difflib
old=read(L/'r502-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
added=['native_entry_navigation.rs']
for name in added:
 p=P/'src/modern_lito'/name;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
diff=L/'r503-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r503-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
assert len(pins)==618;save(L/'r503-production-source-snapshot.json',dict(state='R503_ALL618_SOURCE_FROZEN',source_pins=pins,source_difference=pin(diff),changed=delta,added=added,unchanged=617-len(delta),GUI=0))
player=read(L/'r502-player-candidate.json');root=P/'target/r503-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R503.rs';s=Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r502','r503').replace('R502','R503');main.write_text(s,encoding='utf-8')
entry=read(L/'r502-entry-source-contract.json');entry.update(state='R503_SAME_PLAYER_ENTRY_NAVIGATION_SUBSET',candidate_main=pin(main),previous_R502_player_record=pin(L/'r502-player-candidate.json'),UI_assets='existing24Modern controls; distinct lobbyBack, actualreplyGate; characterqualifiedIMG onlyincharacterscene; no channel/lobby Nativeart synthesized; GUI0');save(L/'r503-entry-source-contract.json',entry)
for oldname,newname in [('build-r502-player-candidate.ps1','build-r503-player-candidate.ps1'),('build-r502-player-events-consumer.ps1','build-r503-navigation-consumer.ps1')]:
 s=(P/'tools'/oldname).read_text(encoding='utf-8').replace('r502','r503').replace('R502','R503').replace('r503b-library-build','r503c-library-build')
 if 'consumer'in newname:
  s=s.replace('player-events-consumer','navigation-consumer').replace('player_events_consumer','navigation_consumer').replace("'r501-candidate.json'","'r502-candidate.json'").replace('R501 source snapshot','R502 source snapshot').replace('211','213').replace("'src\\modern_lito\\native_login_events.rs'","'src\\modern_lito\\native_entry_navigation.rs'").replace("'src\\modern_lito\\native_player_entry_commands.rs'","'src\\modern_lito\\native_player_entry_commands.rs'")
  #R502211pairs already includes oldconsumer/entry; add this entry,consumer,newnav =214.
  s=s.replace('213','214')
 (P/'tools'/newname).write_text(s,encoding='utf-8')
print('R503 frozen',len(pins),'sources;',len(delta),'changed/1new;',diff.stat().st_size,'diffbytes')
