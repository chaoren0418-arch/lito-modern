from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r525-production-source-snapshot.json')['source_pins']};root=P/'target/r526-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if old is None or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True)if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot']if old else '/dev/null',tofile=str(p))))
assert len(pins)==639 and len(changes)==6,(len(pins),changes)
patch=L/'r526-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for n in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_java_character_flow.rs','native_java_room_catalog.rs','native_java_room_participants.rs','native_java_room_departure.rs','native_java_room_self.rs','native_java_room_entry.rs','native_java_room_map_request.rs','native_java_room_settings_request.rs','native_java_room_settings.rs','native_java_room_mode_request.rs']:
 e=pin(P/'src/modern_lito'/n);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r526-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R525=pin(L/'r525-relative-mode-writer-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r526-player-entry-candidate';player.mkdir();entry=read(L/'r525-entry-source-contract.json');main=player/'LiToModern-R526.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r525','r526').replace('R525','R526'),'utf-8');entry['candidate_main']=pin(main);entry['ready_countdown_candidate']='R526 typed participantC_D2 body2/frame4 intent/write;independentD1/E1 receipt;opcodeonlyownerrefused;noWORLDgrant or universalstale recognition';save(L/'r526-entry-source-contract.json',entry)
tests=P/'target/r526-new-targeted-tests';tests.mkdir();dst=tests/'lito_modern-participant-ready-tests.exe';shutil.copyfile(P/'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps/lito_modern-e61d19571b185afc.exe',dst);stdout=(L/'r526-new-targeted-tests.stdout.log').read_text('utf-8-sig');assert '2 passed; 0 failed' in stdout and '2332 filtered out' in stdout
save(L/'r526-new-targeted-tests-accounting.json',dict(binary=pin(dst),filter='modern_lito::native_city_runtime::java_room_ready::r526_tests',new_tests=2,passed=2,failed=0,old_filtered=2332,old_executed=0,network_cases=0,stdout=pin(L/'r526-new-targeted-tests.stdout.log'),stderr=pin(L/'r526-new-targeted-tests.stderr.log'),source_manifest=pin(L/'r526-production-source-snapshot.json')))
print('R526 frozen639 sources/6changes;2 new focused tests,2332 oldfiltered;13 guards unchanged')
