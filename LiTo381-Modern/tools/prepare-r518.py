from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r517-production-source-snapshot.json')['source_pins']}
root=P/'target/r518-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target)
 e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r518-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8')
guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_java_room_departure.rs','native_java_room_participants.rs','native_java_character_flow.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r518-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R517=pin(L/'r517-owner-catalog-delete-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r518-player-entry-candidate';player.mkdir();entry=read(L/'r517-entry-source-contract.json');main=player/'LiToModern-R518.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r517','r518').replace('R517','R518'),'utf-8');entry['candidate_main']=pin(main);entry['room_catalog_candidate']='CC0 bounded add-if-absent/preserve-first, scoped CF delete and joined invalidation; no reset/upsert/ownexit/Native GUI qualification';save(L/'r518-entry-source-contract.json',entry)
s=(P/'tools/build-r517-player.ps1').read_text('utf-8').replace('r517','r518').replace('R517','R518');(P/'tools/build-r518-player.ps1').write_text(s,'utf-8')
print('Frozen',len(pins),'sources;',len(changes),'changes')
