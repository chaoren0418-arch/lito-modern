from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r521-production-source-snapshot.json')['source_pins']};root=P/'target/r522-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if old is None or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r522-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs','native_java_room_entry.rs','native_source_connection.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert len(pins)==636 and len(changes)==6
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r522-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R521=pin(L/'r521-room-mode-map-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r522-player-entry-candidate';player.mkdir();entry=read(L/'r521-entry-source-contract.json');main=player/'LiToModern-R522.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r521','r522').replace('R521','R522'),'utf-8');entry['candidate_main']=pin(main);entry['room_map_writer_candidate']='R522 explicit CE-mode1 C_D1/3 map509/514 and namedFFFF probe, real Sender frame6; intent/write only, no ACK/optimistic map/ready/resource/GUI';save(L/'r522-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r521-{kind}.ps1').read_text('utf-8').replace('r521','r522').replace('R521','R522')
 if kind=='consumer':s=s.replace('r522-mode-map','r522-map-writer')
 (P/f'tools/build-r522-{kind}.ps1').write_text(s,'utf-8')
s=(P/'tools/r521-mode-map-consumer.rs').read_text('utf-8').split('include!("r521-mode-map-case.rs");')[0].replace('R521','R522').replace('java_room_settings as settings,','java_room_settings as settings,java_room_map_request as map_request,')
s=s.replace('r.configure_java_room_mode_map_adapter(mode_map_adapter()?)?;','r.configure_java_room_mode_map_adapter(mode_map_adapter()?)?;r.configure_java_room_map_request_adapter(map_request_adapter()?)?;')
s=s.replace('g.send_many(bodies)?;done.send(())?;','let mut bodies=bodies;if bodies.first().is_some_and(|b|b.is_empty()){bodies.remove(0);ensure!(!bodies.is_empty(),"map expectation absent");let command=bodies.remove(0);ensure!(command.len()==4&&command.starts_with(&[209,3]),"only new ordinary map expected command");g.expect(&command)?;}g.send_many(bodies)?;done.send(())?;')
(P/'tools/r522-map-writer-consumer.rs').write_text(s+'include!("r522-map-writer-case.rs");\n','utf-8')
testroot=P/'target/r522-new-targeted-tests';testroot.mkdir();binary=P/'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps/lito_modern-e61d19571b185afc.exe';dst=testroot/'lito_modern-map-request-tests.exe';shutil.copyfile(binary,dst);stdout=(L/'r522-new-targeted-tests.stdout.log').read_text('utf-8');assert '2 passed; 0 failed' in stdout and '2324 filtered out' in stdout
save(L/'r522-new-targeted-tests-accounting.json',dict(binary=pin(dst),filter='modern_lito::native_city_runtime::java_room_map_request::tests',new_tests=2,passed=2,failed=0,old_filtered=2324,old_executed=0,network_cases=0,stdout=pin(L/'r522-new-targeted-tests.stdout.log'),stderr=pin(L/'r522-new-targeted-tests.stderr.log'),source_manifest=pin(L/'r522-production-source-snapshot.json'),scope='Real Sender map frame6/decrypt/body4, one-byte partial reports/full completion/duplicate/foreign equal-serial/staleoffset/failure/zero/WouldBlock/context-retired ledger; standalone model cannot install Runtime authority.'))
print('R522 frozen636 sources/6changes;2 new targeted tests only,2324 old filtered')
