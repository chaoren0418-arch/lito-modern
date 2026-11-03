from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r516-production-source-snapshot.json')['source_pins']}
root=P/'target/r517-production-source';root.mkdir(exist_ok=True);pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);
 if target.exists():assert pin(target)['sha256']==pin(p)['sha256'],'existing frozen source differs'
 else:shutil.copyfile(p,target)
 e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r517-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8')
actor=pin(P/'src/modern_lito/native_source_actor.rs');assert actor['sha256']==prior[actor['path']]['sha256']
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r517-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R516=pin(L/'r516-room-departure-consumer-handoff.json'),generic_Native_actor_unchanged=actor,Native_navigation_guard_retained=True,GUI=0))
s=(P/'tools/build-r516-consumer.ps1').read_text('utf-8').replace('r516','r517').replace('departure-consumer','owner-delete-consumer').replace('r517-one-loopback-case.rs','r517-owner-delete-case.rs')
(P/'tools/build-r517-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r517-player-entry-candidate';root.mkdir();entry=read(L/'r516-entry-source-contract.json');main=root/'LiToModern-R517.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r516','r517').replace('R516','R517'),'utf-8');entry['candidate_main']=pin(main);entry['room_departure_candidate']='Explicit D505 owner display and S_CF scoped catalog deletion/invalidation; own exit and S_CC insertion/update remain unavailable';save(L/'r517-entry-source-contract.json',entry)
s=(P/'tools/build-r516-player.ps1').read_text('utf-8').replace('r516','r517').replace('R516','R517').replace("$scratch='D:\\lito-dev-runtime\\20261003-supervision\\modern\\r495-native-send'","$scratch=Join-Path 'D:\\lito-dev-runtime\\20261003-supervision\\modern' ('r517-player-build-'+[guid]::NewGuid().ToString('N'))")
(P/'tools/build-r517-player.ps1').write_text(s,'utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R517_D505_CF_focused_integration_verification',current_task='R517 D505 owner display/CF currentchannel catalog候選：632來源7變更，229引用143檔/28新切片/54指令raw/13保存IL方法SHA相符。Rust library编譯通過，唯一新modern wire案例待驗；正常CA空catalog與CEprivate book、D0→D5、另一房CF no-op及同房invalidation。現有catalog刪除僅獨立觀察驗證，不擴CCinsert/update；0JavaSQL/舊PASS/原版GUI，自身離房與ready/start/world不解鎖。')
progress['r517']=dict(state='candidate_verification',source=pin(L/'r517-production-source-snapshot.json'),input=pin(L/'r517-input-adoption.json'),instruction_raw=pin(L/'r517-instruction-byte-adoption.json'),authorized_new_loopback_cases=1,new_Java_SQL=0,server_namespace_writes=0,CC_insert_update_expanded=False,own_exit_advanced=False,Native_guard_retained=True,GUI=0,overall_completed=False);save(P.parent/'監製進度.json',progress)
print('Frozen',len(pins),'source files,',len(changes),'changes')
