from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r524-production-source-snapshot.json')['source_pins']};root=P/'target/r525-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if old is None or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r525-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs','native_java_room_entry.rs','native_source_connection.rs','native_java_room_map_request.rs','native_java_room_settings_request.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert len(pins)==638 and len(changes)==6
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r525-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R524=pin(L/'r524-485F-extended-compact-settings-writer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r525-player-entry-candidate';player.mkdir();entry=read(L/'r524-entry-source-contract.json');main=player/'LiToModern-R525.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r524','r525').replace('R524','R525'),'utf-8');entry['candidate_main']=pin(main);entry.pop('room_compact_settings_writer_candidate',None);entry['relative_mode_writer_candidate']='R525 explicit485F+14B6/BIN28 relative NEXT1/PREVIOUS2 D1/2 body3/frame5; trueSender intent/write only, independentD1/D5/2/3 noACK/nooptimisticState/ownerfromdisplay';save(L/'r525-entry-source-contract.json',entry)
for kind in ['player','consumer']:
 s=(P/f'tools/build-r524-{kind}.ps1').read_text('utf-8').replace('r524','r525').replace('R524','R525')
 if kind=='consumer':s=s.replace('r525-settings-writer','r525-mode-writer')
 (P/f'tools/build-r525-{kind}.ps1').write_text(s,'utf-8')
s=(P/'tools/r524-settings-writer-consumer.rs').read_text('utf-8').split('include!("r524-settings-writer-case.rs");')[0].replace('R524','R525').replace('java_room_settings_request as edit_request','java_room_mode_request as mode_request');s=s.replace('r.configure_java_room_settings_request_adapter(edit_request_adapter()?)?;','r.configure_java_room_mode_request_adapter(mode_request_adapter()?)?;');s=s.replace('(7..=112).contains(&command.len())&&command[0]==221','command.len()==3&&command.starts_with(&[209,2])&&matches!(command[2],1|2)').replace('compact edit expectation','relative mode expectation').replace('compact or extended edit expected command','relative mode expected command');(P/'tools/r525-mode-writer-consumer.rs').write_text(s+'include!("r525-mode-writer-case.rs");\n','utf-8')
testroot=P/'target/r525-new-targeted-tests';testroot.mkdir();binary=P/'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps/lito_modern-e61d19571b185afc.exe';dst=testroot/'lito_modern-relative-mode-tests.exe';shutil.copyfile(binary,dst);stdout=(L/'r525-new-targeted-tests.stdout.log').read_text('utf-8-sig');assert '2 passed; 0 failed' in stdout and '2330 filtered out' in stdout
save(L/'r525-new-targeted-tests-accounting.json',dict(binary=pin(dst),filter='modern_lito::native_city_runtime::java_room_mode_request::tests',new_tests=2,passed=2,failed=0,old_filtered=2330,old_executed=0,network_cases=0,stdout=pin(L/'r525-new-targeted-tests.stdout.log'),stderr=pin(L/'r525-new-targeted-tests.stderr.log'),source_manifest=pin(L/'r525-production-source-snapshot.json'),scope='New direction1/2 pins/invalidabsolute-likeBYTEs and realSender body3/frame5 forboth enums; five onebytepartial/full/duplicate/foreign equalserial/staleoffset/failure/zero/WouldBlock/contextretired ledger. No Runtimebook injection. Old2330 includingR523/R524 compiled but never executed.'))
print('R525 frozen638 sources/6changes;2 new focused tests only,2330 old filtered')
