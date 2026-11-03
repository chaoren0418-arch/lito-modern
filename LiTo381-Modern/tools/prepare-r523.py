from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r522-production-source-snapshot.json')['source_pins']}
root=P/'target/r523-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if old is None or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r523-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs','native_java_room_entry.rs','native_source_connection.rs','native_java_room_settings.rs','native_java_room_map_request.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert len(pins)==637 and len(changes)==5
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r523-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R522=pin(L/'r522-ordinary-map-writer-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r523-player-entry-candidate';player.mkdir();entry=read(L/'r522-entry-source-contract.json');main=player/'LiToModern-R523.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r522','r523').replace('R522','R523'),'utf-8');entry['candidate_main']=pin(main);entry.pop('room_map_writer_candidate',None);entry['room_compact_settings_writer_candidate']='R523 AD368 compact C_DD finite ASCII subset; true Sender, intent/write only, independent D1/D5, no ACK/optimistic state;485F queued and not adopted';save(L/'r523-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r522-{kind}.ps1').read_text('utf-8').replace('r522','r523').replace('R522','R523')
 if kind=='consumer':s=s.replace('r523-map-writer','r523-settings-writer')
 (P/f'tools/build-r523-{kind}.ps1').write_text(s,'utf-8')
s=(P/'tools/r522-map-writer-consumer.rs').read_text('utf-8').split('include!("r522-map-writer-case.rs");')[0].replace('R522','R523').replace('java_room_map_request as map_request','java_room_settings_request as edit_request')
s=s.replace('r.configure_java_room_map_request_adapter(map_request_adapter()?)?;','r.configure_java_room_settings_request_adapter(edit_request_adapter()?)?;')
s=s.replace('"map expectation absent"','"compact edit expectation absent"').replace('command.len()==4&&command.starts_with(&[209,3])','(7..=99).contains(&command.len())&&command[0]==221').replace('"only new ordinary map expected command"','"only new compact edit expected command"')
(P/'tools/r523-settings-writer-consumer.rs').write_text(s+'include!("r523-settings-writer-case.rs");\n','utf-8')
testroot=P/'target/r523-new-targeted-tests';testroot.mkdir();binary=P/'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps/lito_modern-e61d19571b185afc.exe';dst=testroot/'lito_modern-compact-settings-request-tests.exe';shutil.copyfile(binary,dst)
stdout=(L/'r523-new-targeted-tests.stdout.log').read_text('utf-8');assert '2 passed; 0 failed' in stdout and '2326 filtered out' in stdout
save(L/'r523-new-targeted-tests-accounting.json',dict(binary=pin(dst),filter='modern_lito::native_city_runtime::java_room_settings_request::tests',new_tests=2,passed=2,failed=0,old_filtered=2326,old_executed=0,network_cases=0,stdout=pin(L/'r523-new-targeted-tests.stdout.log'),stderr=pin(L/'r523-new-targeted-tests.stderr.log'),source_manifest=pin(L/'r523-production-source-snapshot.json'),scope='Finite ASCII/compact encode and real Sender frame21/decrypt/body19, one-byte partial/full completion/duplicate/foreign equal-serial/staleoffset/failure/zero/WouldBlock/context-retired ledger. No standalone observation installs Runtime authority.'))
print('R523 frozen637 sources/5changes;2 new targeted tests only,2326 old filtered')
