from r491_common import *
import shutil,uuid,difflib
# Correction arrived after independently qualifying the current immutable payloads.
q=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-R507-login-producer-contract.json');assert pin(q)['sha256']=='BF143F967C0064BBB52C8EFB5DD2F42EBA30EAB50653C2A6448E607319D13241'
save(L/'r508-final-producer-pin-correction.json',dict(final_handoff=pin(q),previous_observation=pin(L/'r508-input-adoption.json'),final_pin_trusted_delegation_received=True,payloads_unchanged=True,old_goldens_rerun=False))
root=P/'target/r508-production-source';root.mkdir();pins=[];changed=[];diff=[];old={q['path']:q for q in read(L/'r507c-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q);prior=old.get(str(p))
 if not prior or q['sha256']!=prior['sha256']:
  changed.append(str(p));diff.append(''.join(difflib.unified_diff(Path(prior['snapshot']).read_text(encoding='utf-8').splitlines(True) if prior else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=prior['snapshot'] if prior else '/dev/null',tofile=str(p))))
patch=L/'r508-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r508-production-source-snapshot.json',dict(source_pins=pins,changed=changed,source_difference=pin(patch),sealed_R507_untouched=pin(L/'r507-native-java-host-boundary-handoff.json'),GUI=0))
player=P/'target/r508-player-entry-candidate';player.mkdir();main=player/'LiToModern-R508.rs'
s=(P/'target/r507c-player-entry-candidate/LiToModern-R507C.rs').read_text(encoding='utf-8').replace('r507c','r508').replace('R507C','R508')
s=s.replace('CandidateFixtureWithStartup(PathBuf,PathBuf),','CandidateFixtureWithStartup(PathBuf,PathBuf),\n    CandidateJavaBootstrap(PathBuf,PathBuf),')
s=s.replace('None => Ok(Route::Pick),','None => Ok(Route::Pick),\n        Some(x) if x == OsStr::new("--candidate-java-bootstrap") && args.len()==4 => {let p=PathBuf::from(&args[2]);let a=PathBuf::from(&args[3]);ensure!(p.is_absolute()&&a.is_absolute(),"absolute synthetic inputs required");Ok(Route::CandidateJavaBootstrap(p,a))},')
s=s.replace('Route::CandidateFixtureWithStartup(path,host_path) =>', '''Route::CandidateJavaBootstrap(path,adapter_path) => {let profile=modern_lito::native_city_runtime::login_events::FixtureProfile::load(&path)?;let adapter=modern_lito::native_city_runtime::login_bootstrap::JavaFixtureAdapter::load(&adapter_path)?;let inputs=candidate_inputs()?;let skin=candidate_skin_inputs()?;modern_lito::native_city_window::run_synthetic_java_bootstrap(inputs,skin,profile,adapter)?;},
        Route::CandidateFixtureWithStartup(path,host_path) =>''')
s=s.replace('let inputs=candidate_inputs()?;let skin=candidate_skin_inputs()?;modern_lito::native_city_window::run_synthetic_java_bootstrap(inputs,skin,profile,adapter)?;', 'candidate_java_entry(profile,adapter)?;')
a=s.index('    use modern_lito::{native_offline_preview_resources::Inputs',s.index('fn candidate_offline_entry_with_startup'));b=s.index('    let result=',a)
helper='fn candidate_java_entry(profile: modern_lito::native_city_runtime::login_events::FixtureProfile,adapter:modern_lito::native_city_runtime::login_bootstrap::JavaFixtureAdapter)->Result<()> {\n'+s[a:b]+'    native_city_window::run_synthetic_java_bootstrap(inputs,skin_inputs,profile,adapter)?;Ok(())\n}\n'
s=s.replace('fn version_document()',helper+'fn version_document()')
main.write_text(s,encoding='utf-8');entry=read(L/'r507c-entry-source-contract.json');entry['candidate_main']=pin(main);save(L/'r508-entry-source-contract.json',entry)
(P/'tools/build-r508-player.ps1').write_text((P/'tools/build-r507c-player-candidate.ps1').read_text(encoding='utf-8').replace('r507c','r508').replace('R507C','R508').replace('r508-library-build','r508b-library-build'),encoding='utf-8')
v=read(L/'r507n-java-inputs.json');oldwork=Path(v['work']);work=P/'target'/('r508-java-'+uuid.uuid4().hex);work.mkdir()
for name in ['candidate','ddl','db','source']:
 shutil.copytree(oldwork/name,work/name)
(work/'test-classes').mkdir()
helper=(P/'tools/R507NativeFixtureMain.java').read_text(encoding='utf-8').replace('R507NativeFixtureMain','R508NativeFixtureMain').replace('r507-','r508-')
a=helper.index('   var dirs=new ArrayList<Object>();');b=helper.index('   result.put("state"',a);helper=helper[:a]+helper[b:]
(P/'tools/R508NativeFixtureMain.java').write_text(helper,encoding='utf-8')
r=Path(r'D:/lito-dev-runtime/20261003-supervision/modern')/('nw-'+uuid.uuid4().hex);r.mkdir();shutil.copytree(work/'db',r/'db');(r/'generation-1').mkdir()
v['work']=str(work);v['runtime']=str(r);v['old_work_read_only']=str(oldwork);save(L/'r508b-java-inputs.json',v)
runner=(P/'tools/run-r507n-integration.py').read_text(encoding='utf-8').replace('r507n','r508b').replace('R507NativeFixtureMain','R508NativeFixtureMain').replace('r507-','r508-')
(P/'tools/run-r508b-integration.py').write_text(runner,encoding='utf-8')
builder=(P/'tools/build-r507g-consumer.ps1').read_text(encoding='utf-8').replace('r507c-library-build','r508b-library-build').replace('r507g','r508b').replace('r507c-production-source','r508-production-source').replace('r507-integration','r508-bootstrap').replace('r507_integration','r508_bootstrap').replace('r508b-host-consumer','r508-bootstrap-consumer').replace(";source_oracle=Pin (Join-Path $logs 'r507-source-oracle.json')",'')
(P/'tools/build-r508b-consumer.ps1').write_text(builder,encoding='utf-8')
print('Fresh owned work/runtime',work,r,'changed production files',len(changed))
