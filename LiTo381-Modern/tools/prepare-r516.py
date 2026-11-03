from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r515-production-source-snapshot.json')['source_pins']}
root=P/'target/r516-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r516-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8')
actor=pin(P/'src/modern_lito/native_source_actor.rs');assert actor['sha256']==prior[actor['path']]['sha256']
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r516-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R515=pin(L/'r515-room-participant-consumer-handoff.json'),generic_Native_actor_unchanged=actor,Native_navigation_guard_retained=True,GUI=0))
s=(P/'tools/build-r515-consumer.ps1').read_text('utf-8').replace('r515','r516').replace('participant-consumer','departure-consumer')
s=s.replace('$exe=Join-Path',"[IO.File]::Copy((Join-Path $PSScriptRoot 'r516-one-loopback-case.rs'),(Join-Path $root 'r516-one-loopback-case.rs'),$false)\n$exe=Join-Path")
s=s.replace("$project=Split-Path", "$scratch=Join-Path 'D:\\lito-dev-runtime\\20261003-supervision\\modern' ('r516-consumer-build-'+[guid]::NewGuid().ToString('N'));[IO.Directory]::CreateDirectory($scratch)|Out-Null;$env:TEMP=$scratch;$env:TMP=$scratch\n$project=Split-Path")
(P/'tools/build-r516-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r516-player-entry-candidate';root.mkdir();entry=read(L/'r515-entry-source-contract.json');main=root/'LiToModern-R516.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r515','r516').replace('R515','R516'),'utf-8');entry['candidate_main']=pin(main);entry['room_departure_candidate']='D0 room-local observations; independent synthetic account/source expectation for classification only, own C_D0 writer/exit remains unavailable';save(L/'r516-entry-source-contract.json',entry)
s=(P/'tools/build-r515-player.ps1').read_text('utf-8').replace('r515','r516').replace('R515','R516').replace("$scratch='D:\\lito-dev-runtime\\20261003-supervision\\modern\\r495-native-send'","$scratch=Join-Path 'D:\\lito-dev-runtime\\20261003-supervision\\modern' ('r516-player-build-'+[guid]::NewGuid().ToString('N'))")
(P/'tools/build-r516-player.ps1').write_text(s,'utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R516_modern_owned_D0_candidate_integration',current_task='R516三份D0契約已讀採用：130不同引用/77根raw/12新切片相符；Rust library編譯通過。維持全部新檔modern，唯一新synthetic loopback走正常登入/來源/GAME/NEWS/角色/channel/CE，private book與真StreamIngress；自己的C_D0 writer/exit不解鎖，D5仍下一拒收。')
progress['r516']=dict(state='candidate_verification',source=pin(L/'r516-production-source-snapshot.json'),input=pin(L/'r516-input-adoption.json'),authorized_new_loopback_cases=1,new_Java_SQL=0,server_namespace_writes=0,Native_guard_retained=True,own_exit_advanced=False,GUI=0,overall_completed=False);save(P.parent/'監製進度.json',progress)
print('Frozen',len(pins),'sources,',len(changes),'changes')
