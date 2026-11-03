from r491_common import *
import shutil,uuid,difflib
first=L/'r507-first-attempt';first.mkdir();saved=[]
for p in sorted(L.glob('r507*')):
 if p.is_file():d=first/p.name;shutil.copyfile(p,d);saved.append(pin(d))
save(first/'preservation.json',dict(files=saved,failure='0xC00000FD stack overflow before any Rust flow; global enlarged receiver was an inline array; fixed with bounded heap box',owned_retirement=read(L/'r507-integration-exit.json'),historical_PASS_reruns=0))
v=read(L/'r507-java-inputs.json');r=Path(r'D:\lito-dev-runtime\20261003-supervision\modern')/('nw-'+uuid.uuid4().hex);r.mkdir();shutil.copytree(Path(v['work'])/'db',r/'db');(r/'generation-1').mkdir();v['runtime']=str(r);save(L/'r507c-java-inputs.json',v)
root=P/'target/r507c-production-source';root.mkdir();pins=[];changed=[];diff=[];old={q['path']:q for q in read(L/'r506c-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q);prior=old.get(str(p))
 if not prior or q['sha256']!=prior['sha256']:
  changed.append(str(p));diff.append(''.join(difflib.unified_diff(Path(prior['snapshot']).read_text(encoding='utf-8').splitlines(True) if prior else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=prior['snapshot'] if prior else '/dev/null',tofile=str(p))))
patch=L/'r507c-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r507c-production-source-snapshot.json',dict(source_pins=pins,changed=changed,source_difference=pin(patch),sealed_R506_untouched=pin(L/'r506-native-transfer-channel-handoff.json'),GUI=0))
root=P/'target/r507c-player-entry-candidate';root.mkdir();main=root/'LiToModern-R507C.rs';main.write_text((P/'target/r507b-player-entry-candidate/LiToModern-R507B.rs').read_text(encoding='utf-8').replace('r507b','r507c').replace('R507B','R507C'),encoding='utf-8');entry=read(L/'r507b-entry-source-contract.json');entry['candidate_main']=pin(main);save(L/'r507c-entry-source-contract.json',entry)
(P/'tools/build-r507c-player-candidate.ps1').write_text((P/'tools/build-r507b-player-candidate.ps1').read_text(encoding='utf-8').replace('r507b','r507c').replace('R507B','R507C'),encoding='utf-8')
(P/'tools/build-r507c-integration-consumer.ps1').write_text((P/'tools/build-r507b-integration-consumer.ps1').read_text(encoding='utf-8').replace('r507b','r507c'),encoding='utf-8')
(P/'tools/run-r507c-integration.py').write_text((P/'tools/run-r507-integration.py').read_text(encoding='utf-8').replace('r507-java-inputs','r507c-java-inputs').replace('r507b','r507c').replace("L/'r507-", "L/'r507c-"),encoding='utf-8')
v=read(P.parent/'監製進度.json');v['r506']['status']='finite_modern_verified_scope_pending_withdrawn_followup_R507';v['r507'].update(state='first_failure_retired_heap_fix_building',first_attempt=str(first/'preservation.json'),runtime=str(r));v['phase']='R507_new_integration_heap_fix';v['current_task']='Java自有隔離啟動／@@datadir／四程序正常退休已證；新增Rust heap接收buffer修正stack overflow，續驗新S_FB／host Back，不重跑歷史10PASS';save(P.parent/'監製進度.json',v)
print(str(r))
