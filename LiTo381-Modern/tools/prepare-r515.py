from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r514b-production-source-snapshot.json')['source_pins']};root=P/'target/r515-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r515-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8')
native_actor=pin(P/'src/modern_lito/native_source_actor.rs');assert native_actor['sha256']==prior[native_actor['path']]['sha256']
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r515-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R514=pin(L/'r514-room-entry-consumer-handoff.json'),generic_Native_actor_unchanged=native_actor,Native_navigation_guard_retained=True,GUI=0))
s=(P/'tools/build-r514-consumer.ps1').read_text('utf-8').replace('r514','r515').replace('room-entry','participant');(P/'tools/build-r515-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r515-player-entry-candidate';root.mkdir();entry=read(L/'r514b-entry-source-contract.json');main=root/'LiToModern-R515.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r514b','r515').replace('R514b','R515'),'utf-8');entry['candidate_main']=pin(main);entry['room_participant_candidate']='Incremental receipt data only; Native actor/GUI/preview/ready-command authority not added';save(L/'r515-entry-source-contract.json',entry)
s=(P/'tools/build-r514b-player.ps1').read_text('utf-8').replace('r514b','r515').replace('R514b','R515');(P/'tools/build-r515-player.ps1').write_text(s,'utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R515_incremental_room_receipts_focused_verification',current_task='R515明確AD368 CD/D1增量receipt候選：630來源；same kind+rawalias重複列保留既有成員/D1state、每筆完整raw receipt另存，跨kind分離，未知/歧義D1忽略；有界可能不完整投影。僅既有vectors/新pure checks，無新fixture/舊PASS/原版/JavaSQL，Host D0不解鎖。')
progress['r515']=dict(state='candidate_verification',source=pin(L/'r515-production-source-snapshot.json'),input=pin(L/'r515-input-adoption.json'),native=pin(L/'r515-native-evidence-adoption.json'),fixture_calls=0,owner_ready=False,GUI=0);save(P.parent/'監製進度.json',progress)
print('Frozen',len(pins),'files;',len(changes),'changes;genericNativeactor unchanged, navigationguard retained')
