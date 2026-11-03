from r491_common import *
import shutil,difflib
root=P/'target/r507b-production-source';root.mkdir();pins=[];changed=[];diff=[];old={q['path']:q for q in read(L/'r506c-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q);prior=old.get(str(p))
 if not prior or q['sha256']!=prior['sha256']:
  changed.append(str(p));diff.append(''.join(difflib.unified_diff(Path(prior['snapshot']).read_text(encoding='utf-8').splitlines(True) if prior else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=prior['snapshot'] if prior else '/dev/null',tofile=str(p))))
patch=L/'r507b-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r507b-production-source-snapshot.json',dict(source_pins=pins,changed=changed,source_difference=pin(patch),sealed_R506_untouched=pin(L/'r506-native-transfer-channel-handoff.json'),GUI=0))
player=P/'target/r507b-player-entry-candidate';player.mkdir();main=player/'LiToModern-R507B.rs'
s=(P/'target/r506c-player-entry-candidate/LiToModern-R506C.rs').read_text(encoding='utf-8').replace('r506c','r507b').replace('R506C','R507B')
s=s.replace('CandidateFixture(PathBuf),','CandidateFixture(PathBuf),\n    CandidateFixtureWithStartup(PathBuf,PathBuf),')
at='        Some(x) if x == OsStr::new("--candidate-synthetic-fixture")'
s=s.replace(at,'        Some(x) if x == OsStr::new("--candidate-synthetic-startup") && args.len() == 4 => {let profile=PathBuf::from(&args[2]);let host=PathBuf::from(&args[3]);ensure!(profile.is_absolute()&&host.is_absolute(),"absolute fixture inputs required");Ok(Route::CandidateFixtureWithStartup(profile,host))}\n'+at)
s=s.replace('        Route::CandidateFixture(path) =>','        Route::CandidateFixtureWithStartup(path,host_path) => {let profile=modern_lito::native_city_runtime::login_events::FixtureProfile::load(&path)?;let host=modern_lito::native_city_runtime::startup_host::FixtureStartupHost::load(&host_path)?;candidate_offline_entry_with_startup(false,Some(profile),Some(host))?},\n        Route::CandidateFixture(path) =>')
sig='fn candidate_offline_entry(start_at_maps: bool, fixture: Option<modern_lito::native_city_runtime::login_events::FixtureProfile>) -> Result<Option<PathBuf>> {'
assert sig in s
s=s.replace(sig,sig+'candidate_offline_entry_with_startup(start_at_maps,fixture,None)}\nfn candidate_offline_entry_with_startup(start_at_maps: bool, fixture: Option<modern_lito::native_city_runtime::login_events::FixtureProfile>,host:Option<modern_lito::native_city_runtime::startup_host::FixtureStartupHost>)->Result<Option<PathBuf>> {')
old='native_city_window::run_synthetic_fixture(inputs,skin_inputs,profile)?'
s=s.replace(old,'if let Some(host)=host{native_city_window::run_synthetic_fixture_with_startup_host(inputs,skin_inputs,profile,host)?}else{'+old+'}')
main.write_text(s,encoding='utf-8');entry=read(L/'r506c-entry-source-contract.json');entry['candidate_main']=pin(main);entry['new_flag']='--candidate-synthetic-startup <absolute fixture profile> <absolute separate startup host>';save(L/'r507b-entry-source-contract.json',entry)
s=(P/'tools/build-r506c-player-candidate.ps1').read_text(encoding='utf-8').replace('r506c','r507b').replace('R506C','R507B');(P/'tools/build-r507b-player-candidate.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r507-integration-consumer.ps1').read_text(encoding='utf-8').replace("'r507-library-build.jsonl'","'r507b-library-build.jsonl'").replace('target/r507-fixed-integration-consumer','target/r507b-fixed-integration-consumer').replace('r507-consumer-build','r507b-consumer-build').replace('r507-consumer-candidate','r507b-consumer-candidate').replace("'r507-production-source-snapshot.json'","'r507b-production-source-snapshot.json'")
(P/'tools/build-r507b-integration-consumer.ps1').write_text(s,encoding='utf-8')
