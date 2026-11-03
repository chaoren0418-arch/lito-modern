from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r519-production-source-snapshot.json')['source_pins']};root=P/'target/r520-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r520-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_source_send.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert len(pins)==635 and len(changes)==6
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r520-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R519=pin(L/'r519-singleton-self-notification-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r520-player-entry-candidate';player.mkdir();entry=read(L/'r519-entry-source-contract.json');main=player/'LiToModern-R520.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r519','r520').replace('R519','R520'),'utf-8');entry['candidate_main']=pin(main);entry['room_settings_candidate']='R520 explicit normal owned CE D5/0 and D5/1 independent raw receipts, no ACK/pair/ready/owner inference; Native globals/map projection remain static only';save(L/'r520-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r519-{kind}.ps1').read_text('utf-8').replace('r519','r520').replace('R519','R520').replace('r520-final-library-build.jsonl','r520-library-build.jsonl')
 if kind=='consumer':s=s.replace('r520-self','r520-settings')
 (P/f'tools/build-r520-{kind}.ps1').write_text(s,'utf-8')
print('R520 frozen',len(pins),'sources;',len(changes),'changes')
