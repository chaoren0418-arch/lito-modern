from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r512b-production-source-snapshot.json')['source_pins']};root=P/'target/r513-production-source';root.mkdir();pins=[];changes=[];diff=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);e=pin(p);e['snapshot']=str(d);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r513-source-difference.patch';patch.write_text('\n'.join(diff),'utf-8');save(L/'r513-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R512=pin(L/'r512-return-reconnect-status-handoff.json'),GUI=0))
s=(P/'tools/build-r512c-consumer.ps1').read_text('utf-8').replace('r512b-library-build','r513-library-build').replace('r512c','r513').replace('r512b-production-source-snapshot','r513-production-source-snapshot').replace('r512-return-consumer','r513-status-family-consumer');(P/'tools/build-r513-consumer.ps1').write_text(s,'utf-8')
root=P/'target/r513-player-entry-candidate';root.mkdir();main=root/'LiToModern-R513.rs';main.write_text((P/'target/r512-player-entry-candidate/LiToModern-R512.rs').read_text('utf-8').replace('r512','r513').replace('R512','R513'),'utf-8');v=read(L/'r512-entry-source-contract.json');v.pop('new_return',None);v['candidate_main']=pin(main);v['new_status_family']='Fixed receipts only; raw bits vs Java semantics, no owner/side effects';save(L/'r513-entry-source-contract.json',v)
s=(P/'tools/build-r512-player.ps1').read_text('utf-8').replace('r512b-library-build','r513-library-build').replace('r512','r513').replace('R512','R513');(P/'tools/build-r513-player.ps1').write_text(s,'utf-8')
v=read(P.parent/'監製進度.json');v.update(updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),phase='R513_status_family_focused_verification',current_task='S50/51/52/56 typed receipts與S0C重疊欄位projection已接線；8封存frame核對，僅新解析/順序/epoch與實際下一未知邊界測試，不重播舊PASS或Java/SQL/GUI。');v['r513']=dict(state='candidate_verification',source_snapshot=pin(L/'r513-production-source-snapshot.json'),input_adoption=pin(L/'r513-input-adoption.json'),GUI=0,owner_ready=False);save(P.parent/'監製進度.json',v)
print('frozen',len(pins),'source files;',len(changes),'changes')
