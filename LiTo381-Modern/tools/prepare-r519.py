from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r518-production-source-snapshot.json')['source_pins']};root=P/'target/r519-production-source';root.mkdir();pins=[];changes=[];patches=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 target=root/p.relative_to(P);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,target);e=pin(p);e['snapshot']=str(target);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:
  changes.append(str(p));patches.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text('utf-8').splitlines(True) if old else [],p.read_text('utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r519-source-difference.patch';patch.write_text('\n'.join(patches),'utf-8');guards=[]
for name in ['native_source_actor.rs','rooms.rs','native_source_identity.rs','native_source_send.rs','native_java_character_flow.rs','native_java_room_catalog.rs']:
 e=pin(P/'src/modern_lito'/name);assert e['sha256']==prior[e['path']]['sha256'];guards.append(e)
assert 'navigation mode side effects unqualified; retire source epoch' in (P/'src/modern_lito/native_city_runtime.rs').read_text('utf-8')
save(L/'r519-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R518=pin(L/'r518-catalog-ingress-consumer-handoff.json'),unchanged_guards=guards,Native_navigation_guard_retained=True,GUI=0))
player=P/'target/r519-player-entry-candidate';player.mkdir();entry=read(L/'r518-entry-source-contract-final.json');entry.pop('inherited_compilation_contract',None);main=player/'LiToModern-R519.rs';main.write_text(Path(entry['candidate_main']['path']).read_text('utf-8').replace('r518','r519').replace('R518','R519'),'utf-8');entry['candidate_main']=pin(main);entry['room_departure_candidate']='R519 singleton raw GAME roster provenance plus completed ordinary C_D0/self notification correlation; scene/channel transition and committed leave ACK remain unqualified';entry['room_self_candidate']='Normal one-row S03/fullCB/ownedC9+acceptedC8/currentsource+selection+ordinaryCE, raw alias preserved; no synthetic declared expectation for own writer';save(L/'r519-entry-source-contract.json',entry)
s=(P/'tools/build-r518-player.ps1').read_text('utf-8').replace('r518','r519').replace('R518','R519').replace('r519-library-build.jsonl','r519-final-library-build.jsonl');(P/'tools/build-r519-player.ps1').write_text(s,'utf-8')
testroot=P/'target/r519-new-targeted-tests';testroot.mkdir();testexe=P/'target/r409-dev-ready/i686-pc-windows-msvc/debug/deps/lito_modern-e61d19571b185afc.exe';dst=testroot/'lito_modern-self-tests.exe';shutil.copyfile(testexe,dst)
stdout=(L/'r519-new-targeted-tests.stdout.log').read_text('utf-8');assert '3 passed; 0 failed' in stdout and '2321 filtered out' in stdout
save(L/'r519-new-targeted-tests-accounting.json',dict(binary=pin(dst),filter='modern_lito::native_city_runtime::java_room_self::tests',new_tests=3,passed=3,failed=0,old_tests_filtered=2321,old_tests_executed=0,network_cases=0,stdout=pin(L/'r519-new-targeted-tests.stdout.log'),stderr=pin(L/'r519-new-targeted-tests.stderr.log'),source_manifest=pin(L/'r519-production-source-snapshot.json'),scope='Real bounded Sender full3-byte/partial/failure reports and once-only context model; singleton byte-provenance shape only. Pure tests do not install Runtime authority or qualify new actual backend auth. Entire lib-test build carries existing88 compiler warnings; only these3 selected tests ran.'))
print('Frozen',len(pins),'sources;',len(changes),'changes;3 new targeted tests archived,2321 old filtered')
