from r491_common import *
import shutil,difflib
root=P/'target/r508c-production-source';root.mkdir();pins=[];diff=[];changed=[];old={e['path']:e for e in read(L/'r507c-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q);prior=old.get(str(p))
 if not prior or prior['sha256']!=q['sha256']:
  changed.append(str(p));diff.append(''.join(difflib.unified_diff(Path(prior['snapshot']).read_text(encoding='utf-8').splitlines(True) if prior else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=prior['snapshot'] if prior else '/dev/null',tofile=str(p))))
patch=L/'r508c-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r508c-production-source-snapshot.json',dict(source_pins=pins,changed=changed,source_difference=pin(patch),GUI=0))
root=P/'target/r508q-player-entry-candidate';root.mkdir();main=root/'LiToModern-R508Q.rs';main.write_text((P/'target/r508p-player-entry-candidate/LiToModern-R508P.rs').read_text(encoding='utf-8').replace('r508p','r508q').replace('R508P','R508Q'),encoding='utf-8');v=read(L/'r508p-entry-source-contract.json');v['candidate_main']=pin(main);save(L/'r508q-entry-source-contract.json',v)
s=(P/'tools/build-r508p-player.ps1').read_text(encoding='utf-8').replace('r508p','r508q').replace('R508P','R508Q').replace('r508b-library-build','r508c-library-build');(P/'tools/build-r508q-player.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r508b-consumer.ps1').read_text(encoding='utf-8').replace('r508b','r508c').replace('r508-production-source','r508c-production-source');(P/'tools/build-r508c-consumer.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/r508-bootstrap-consumer.rs').read_text(encoding='utf-8').replace('r508b','r508c');s=s.replace('let token=r.source_choice_control(slot).unwrap();','assert!(r.handle_player_entry_command(Command::OpenLogin).is_err());let token=r.source_choice_control(slot).unwrap();');(P/'tools/r508-bootstrap-consumer.rs').write_text(s,encoding='utf-8')
v=read(L/'r508b-java-inputs.json');save(L/'r508c-java-inputs.json',v)
s=(P/'tools/run-r508b-integration.py').read_text(encoding='utf-8').replace('r508b','r508c');(P/'tools/run-r508c-integration.py').write_text(s,encoding='utf-8')
print('Final guarded R508 sources',len(pins),'changed',len(changed),'no earlier new tests executed')
