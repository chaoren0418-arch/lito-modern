from r491_common import *
import shutil,difflib
old=read(L/'r501-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_login_transport.rs'},delta
added=['native_login_events.rs','native_player_entry_commands.rs']
for name in added:
 p=P/'src/modern_lito'/name;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
diff=L/'r502-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r502-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
assert len(pins)==617;save(L/'r502-production-source-snapshot.json',dict(state='R502_ALL617_SOURCE_FROZEN_BEFORE_SAME_COMMAND_EVENT_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r501-source-scope.json');scope.pop('changed_from_fixed_R500',None);scope.pop('added_from_fixed_R500',None);scope.update(state='R502_SHARED_PLAYER_CALLBACK_COMMANDS_AND_RUNTIME_SOCKET_EVENT_SCOPE',source_files=617,changed_from_fixed_R501=delta,added_from_fixed_R501=added,scope='3changed2new/612previousunchanged;Modern own outputs and existing uniqueASCII scratch;explicit synthetic ownloopback only; no JavaSQL/server/sharedBIN/provider/realcredentials/GUI')
for name in added:scope['new_source_files'].append(dict(path='src/modern_lito/'+name,sha256=pin(P/'src/modern_lito'/name)['sha256']))
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r502-source-scope.json',scope);save(L/'source-change-scope.json',scope)
player=read(L/'r501-player-candidate.json');root=P/'target/r502-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R502.rs';s=Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('r501','r502').replace('R501','R502');s=s.replace('    CandidateEntry,','    CandidateEntry,\n    CandidateFixture(PathBuf),');s=s.replace('        Some(x) if x == OsStr::new("--candidate-menu")','        Some(x) if x == OsStr::new("--candidate-synthetic-fixture") && args.len() == 3 => {\n            let path=PathBuf::from(&args[2]); ensure!(path.is_absolute(),"合成 profile 必須使用絕對路徑"); Ok(Route::CandidateFixture(path))\n        }\n        Some(x) if x == OsStr::new("--candidate-menu")');s=s.replace('candidate_offline_entry(false)?','candidate_offline_entry(false,None)?').replace('candidate_offline_entry(true)?','candidate_offline_entry(true,None)?');s=s.replace('        Route::CandidateEntry =>','        Route::CandidateFixture(path) => { let profile=modern_lito::native_city_runtime::login_events::FixtureProfile::load(&path)?; candidate_offline_entry(false,Some(profile))? },\n        Route::CandidateEntry =>');s=s.replace('fn candidate_offline_entry(start_at_maps: bool)','fn candidate_offline_entry(start_at_maps: bool, fixture: Option<modern_lito::native_city_runtime::login_events::FixtureProfile>)');s=s.replace('match native_city_window::run(inputs, skin_inputs, start_at_maps)? {','let result=if let Some(profile)=fixture {native_city_window::run_synthetic_fixture(inputs,skin_inputs,profile)?}else{native_city_window::run(inputs,skin_inputs,start_at_maps)?};\n    match result {');main.write_text(s,encoding='utf-8')
entry=read(L/'r501-entry-source-contract.json');entry.update(state='R502_EXPLICIT_SYNTHETIC_FIXTURE_PLAYER_ENTRY_SAME_WINDOW_COMMANDS_STATIC_ONLY',candidate_main=pin(main),previous_R501_player_record=pin(L/'r501-player-candidate.json'),fixture_flag='--candidate-synthetic-fixture <absolute explicit synthetic profile>',default_profile_route_unchanged=True,UI_assets='sameR498skin/modernEDIT;syntheticmode explicit INFO+button;shared EntryCommand for Window andheadless;40ms Runtime.poll boundedIOevents;normalcandidateformlocalstaging/realNativeauth notopened;GUI0');save(L/'r502-entry-source-contract.json',entry)
for oldname,newname in [('build-r501-player-candidate.ps1','build-r502-player-candidate.ps1'),('build-r501-coalesced-seed-consumer.ps1','build-r502-player-events-consumer.ps1')]:
 s=(P/'tools'/oldname).read_text(encoding='utf-8').replace('r501','r502').replace('R501','R502').replace('r502e-library-build','r502b-library-build').replace('target\\r502-final-player-entry-candidate','target\\r502-player-entry-candidate')
 if 'consumer'in newname:
  s=s.replace('coalesced-seed-consumer','player-events-consumer').replace('coalesced_seed_consumer','player_events_consumer').replace('target\\r502-final-coalesced-consumer','target\\r502-fixed-player-events-consumer').replace("'r500-candidate.json'","'r501-candidate.json'").replace('R500 source snapshot','R501 source snapshot').replace('207','211').replace("'src\\modern_lito\\native_login_wire.rs'","'src\\modern_lito\\native_login_events.rs'").replace("'src\\modern_lito\\native_login_transport.rs'","'src\\modern_lito\\native_player_entry_commands.rs'")
 (P/'tools'/newname).write_text(s,encoding='utf-8')
print('R502 frozen617sources/3changed2new/211pairs; diff',diff.stat().st_size,'bytes;independent explicitfixture candidateentry')
