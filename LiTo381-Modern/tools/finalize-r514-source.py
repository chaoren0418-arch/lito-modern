from r491_common import *
import shutil,difflib
initial=read(L/'r514-production-source-snapshot.json');prior={e['path']:e for e in initial['source_pins']}
baseline={e['path']:e for e in read(L/'r513-production-source-snapshot.json')['source_pins']}
root=P/'target/r514b-production-source';root.mkdir();pins=[];changes=[];delta=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target)
 e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p));base=baseline.get(str(p))
 if not base or base['sha256']!=e['sha256']:changes.append(str(p))
 if not old or old['sha256']!=e['sha256']:
  delta.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
assert delta==[str(P/'src/modern_lito/native_source_connection.rs')],delta
patch=L/'r514b-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8')
save(L/'r514b-production-source-snapshot.json',dict(source_pins=pins,changes=changes,delta_from_first_R514=delta,source_difference=pin(patch),first_R514=pin(L/'r514-production-source-snapshot.json'),GUI=0,
 unchanged_room_adapter_and_Runtime_verified=True,policy='Generic Native success retains Snapshot structural validation; ID/name admission stays in AD368 room adapter'))
entry=read(L/'r514-entry-source-contract.json');root=P/'target/r514b-player-entry-candidate';root.mkdir();main=root/'LiToModern-R514b.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r514','r514b').replace('R514','R514b'),'utf-8');entry['candidate_main']=pin(main);save(L/'r514b-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r514-{kind}.ps1').read_text('utf-8').replace('r514','r514b').replace('R514','R514b')
 if kind=='consumer':s=s.replace('room-entry','constructor-policy')
 (P/f'tools/build-r514b-{kind}.ps1').write_text(s,'utf-8')
print('Final629 sources;only generic constructor delta;room/Runtime unchanged')
