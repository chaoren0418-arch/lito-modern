from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r513-production-source-snapshot.json')['source_pins']}
root=P/'target/r514-production-source';root.mkdir();pins=[];changes=[];diff=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target)
 e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r514-source-difference.patch';patch.write_text('\n'.join(diff),'utf-8')
save(L/'r514-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R513=pin(L/'r513-status-family-consumer-handoff.json'),GUI=0))
s=(P/'tools/build-r513-consumer.ps1').read_text('utf-8').replace('r513','r514').replace('status-family','room-entry');(P/'tools/build-r514-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r514-player-entry-candidate';root.mkdir();main=root/'LiToModern-R514.rs'
main.write_text((P/'target/r513-player-entry-candidate/LiToModern-R513.rs').read_text('utf-8').replace('r513','r514').replace('R513','R514'),'utf-8')
entry=read(L/'r513-entry-source-contract.json');entry['candidate_main']=pin(main);entry['room_entry']='Explicit candidate API only; no Native buttons, room GUI or owner wiring';save(L/'r514-entry-source-contract.json',entry)
s=(P/'tools/build-r513-player.ps1').read_text('utf-8').replace('r513','r514').replace('R513','R514');(P/'tools/build-r514-player.ps1').write_text(s,'utf-8')
progress=read(P.parent/'監製進度.json');progress.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R514_owned_room_entry_focused_verification',current_task='R514明確AD368建立/加入room receipt候選已接線：typed變長CE/短失敗、owned control/revision、完成write後收回覆；Native guard保留。僅新Rust有限檢查，無舊PASS/fixture/原版/JavaSQL。')
progress['r514']=dict(state='candidate_verification',source_snapshot=pin(L/'r514-production-source-snapshot.json'),input_adoption=pin(L/'r514-input-adoption.json'),native_adoption=pin(L/'r514-native-evidence-adoption.json'),owner_ready=False,GUI=0)
save(P.parent/'監製進度.json',progress)
print('Frozen',len(pins),'source files;',len(changes),'changes')
