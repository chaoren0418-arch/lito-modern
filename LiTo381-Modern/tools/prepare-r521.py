from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r520-production-source-snapshot.json')['source_pins']};root=P/'target/r521-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True),p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'],tofile=str(p))))
patch=L/'r521-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_source_send.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs','native_java_room_entry.rs','native_java_room_tail.rs','native_city_presentation.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert len(pins)==635 and len(changes)==3
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r521-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R520=pin(L/'r520-room-settings-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r521-player-entry-candidate';player.mkdir();entry=read(L/'r520-entry-source-contract.json');main=player/'LiToModern-R521.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r520','r521').replace('R520','R521'),'utf-8');entry['candidate_main']=pin(main);entry['room_mode_map_candidate']='R521 explicit structural D5/2 mode BYTE and D5/3 full mapWORD independent receipts; mode trigger unqualified, no ACK/resource/effective CE/GUI';save(L/'r521-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r520-{kind}.ps1').read_text('utf-8').replace('r520','r521').replace('R520','R521')
 if kind=='consumer':s=s.replace('r521-settings','r521-mode-map')
 (P/f'tools/build-r521-{kind}.ps1').write_text(s,'utf-8')
s=(P/'tools/r520-settings-consumer.rs').read_text('utf-8').split('include!("r520-settings-case.rs");')[0].replace('R520','R521').replace('java_return,','')
s=s.replace('r.configure_java_room_settings_adapter(settings_adapter()?)?;','r.configure_java_room_settings_adapter(settings_adapter()?)?;r.configure_java_room_mode_map_adapter(mode_map_adapter()?)?;')
(P/'tools/r521-mode-map-consumer.rs').write_text(s+'include!("r521-mode-map-case.rs");\n','utf-8')
print('R521 frozen635 sources/3changes; new independent mode/map harness only')
