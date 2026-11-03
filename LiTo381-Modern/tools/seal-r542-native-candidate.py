"""Seal the already-built R542 candidate. No build, test, provider or launch calls."""
import difflib
import hashlib
import json
import pathlib
import re
from datetime import datetime, timezone, timedelta

P = pathlib.Path(__file__).resolve().parent.parent
L = P / 'logs/continuation-20261003'
T = P / 'target/r542-native-backend-candidate'
NOW = datetime.now(timezone(timedelta(hours=8))).isoformat()

def pin(path):
    path = pathlib.Path(path).resolve()
    h = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1 << 20), b''):
            h.update(chunk)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': h.hexdigest().upper()}

def verify(v):
    actual = pin(v['path'])
    assert actual['bytes'] == v['bytes'] and actual['sha256'] == v['sha256'], actual
    return actual

def read(name):
    return json.loads((L / name).read_text(encoding='utf-8-sig'))

def save(path, value):
    with pathlib.Path(path).open('x', encoding='utf-8', newline='\n') as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    return pin(path)

c = read('r542-candidate.json')
r540 = read('r540-native-start-clock-roster-component-handoff.json')
r532 = read('r532-player-candidate.json')
snapshot = read('r542-source-snapshot.json')
baseline = read('r540-final-production-source-snapshot.json')
assert snapshot['source_count'] == 661
unchanged = []
for v in baseline['files']:
    current = pin(P / v['path'])
    assert current['sha256'] == v['sha256'] and current['bytes'] == v['bytes'], current
    unchanged.append(current)
assert len(unchanged) == 660
current_sources = {str(p.relative_to(P)).replace('\\', '/') for p in (P / 'src').rglob('*.rs')}
assert current_sources - {v['path'] for v in baseline['files']} == {'src/bin/native_backend_485f_candidate.rs'}
new_source = (P / 'src/bin/native_backend_485f_candidate.rs').read_text(encoding='utf-8')
assert not any(x in new_source for x in ('FixedOfflineTrace', 'OfflineWireSession', 'room_start_component_fixture', 'native_pet_race_start_component'))
cargo = (P / 'Cargo.toml').read_text(encoding='utf-8')
old_cargo = (P / 'target/r541-native-room-entry-audit/source/Cargo.toml').read_text(encoding='utf-8')
addition = '[[bin]]\nname = "LiToModern-485F-Candidate"\npath = "src/bin/native_backend_485f_candidate.rs"\n\n'
assert cargo.replace(addition, '', 1) == old_cargo
assert 'default-run = "LiToModern"' in cargo
assert verify(c['linked_library'])['sha256'] == r540['production_library']['sha256']
assert c['library_fresh'] is True
preserved = [verify(r532[k]) for k in ('executable', 'production_library', 'candidate_main', 'original_main')]
assert c['default_main'] == preserved[-1]
assert pin(L/'r541-native-room-application-lifecycle-handoff.json')['sha256'] == 'E079519B4973530FFB6EBADBDE9922DB1AF8AC592ED28494C603D02FF6E04C72'
frozen_inputs = []
for v in snapshot['inputs']:
    assert verify(v['snapshot'])['sha256'] == v['sha256']
    current = pin(P / v['path'])
    assert current['sha256'] == v['sha256'] and current['bytes'] == v['bytes']
    frozen_inputs.append(v['snapshot'])
for v in c['resources']:
    verify(v)
warnings = {}
for name in ('r542-build.jsonl', 'r542-test-build.jsonl'):
    records = [json.loads(line) for line in (L/name).read_text(encoding='utf-8').splitlines() if line.strip()]
    warns = [v for v in records if v.get('reason') == 'compiler-message' and v['message']['level'] == 'warning']
    new = [v for v in warns if any('native_backend_485f_candidate.rs' in s['file_name'] for s in v['message'].get('spans', []))]
    assert not new
    warnings[name] = {'retained_library_warning_messages': len(warns), 'new_candidate_warnings': 0}
test_output = (L/'r542-tests.stdout.log').read_text(encoding='utf-8-sig')
assert re.search(r'2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out', test_output)
assert all(read(n)['exit_code'] == 0 for n in ('r542-build-accounting.json','r542-test-build-accounting.json','r542-test-accounting.json'))
diff = '\n'.join(difflib.unified_diff(old_cargo.splitlines(), cargo.splitlines(), fromfile='R541/Cargo.toml', tofile='R542/Cargo.toml')) + '\n'
with (L/'r542-cargo-source-difference.patch').open('x', encoding='utf-8', newline='\n') as stream:
    stream.write(diff)

adoption = {
    'schema':'modern.r542.repository-native-candidate-adoption.v1',
    'choice':'Named independent Cargo binary; minimal explicit Cargo registration plus one small entry source, no Runtime copy.',
    'binary':c['binary_name'], 'default_run_unchanged':'LiToModern', 'default_main_byte_equal_R532_R541':True,
    'previous_660_Rust_sources_unchanged':True, 'source_count':661,
    'new_source':c['binary_source'], 'Cargo_change':c['cargo_manifest'],
    'compiled_call_chain':'new repository CandidateRoute -> dispatch -> original Accepted::load -> original native_city_window::run_backend_485f -> original Accepted::configure -> original Runtime-owned 485F RX/TX/ROOM lifecycle',
    'launch_constraints_unchanged':['exact absolute fresh READY with trusted SHA256','same Accepted::load one-use claim','same supervisor provider freshness/identity/lease checks','same one-shot connect gate','same bounded loopback IO and original ROOM controls'],
    'pure_dispatch_boundary':'Only passes parsed path/digest and fixed resource Inputs; the production callback calls existing Accepted/load/window. Tests replace this callback, never call Accepted::load.',
    'resources':c['resources'], 'resource_root':c['resource_project_root'],
    'resource_env':'Cargo CARGO_MANIFEST_DIR only; exact R532 manifest size/SHA pins now in repository source. No custom R532_* env requirements.',
    'nested_resource_verification':'Existing adapters check nested payload pins on loading. No resource payload loader or GUI executed in this round.',
    'actual_linked_library':c['linked_library'], 'library_matches_R540_frozen_bytes':True,
    'library_recompiled':False, 'Cargo_library_fresh':True,
    'R540_components_exist_in_library_only':True,
    'R540_imported_into_485F_live_call_chain':False,
    '5350_component_promoted_to_485F_contract':False,
    'no_new_ROOM_defaults_or_fixed_trace_side_channels':True,
    'R532_artifacts_verified_preserved':preserved,
    'release_or_portable_package_published':False,
    'product_scope':'Repository candidate maintenance and reproducible Debug build; no new live ROOM/GUI/WORLD acceptance.',
}
checks = {
    'schema':'modern.r542.cli-checks-and-build-accounting.v1',
    'new_tests':[{'name':'r542_candidate_checks::exact_explicit_route_rejects_malformed_input_and_default_fallback',
                  'scope':'Exact flag/arity, absolute Unicode path, normalized digest; empty/default/foreign flag/relative path/malformed/non-ASCII digest rejection. Parsing performs no filesystem access.'},
                 {'name':'r542_candidate_checks::parsed_route_dispatches_pinned_configuration_without_reading_ready_or_claiming',
                  'scope':'Actual dispatch receives parsed configuration and exact pinned root/manifests in CPU callback; callback refusal propagates. No Accepted::load or READY access.'}],
    'passed':2, 'failed':0, 'old_tests_executed':0, 'old_PASS_replays':0, 'tests_filtered':0,
    'candidate_build_attempts':1, 'candidate_test_compile_attempts':1, 'new_test_execution_attempts':1,
    'library_recompile':0, 'compiler_warnings':warnings,
    'player_candidate_executions':0, 'Accepted_load_calls':0, 'READY_reads':0, 'claims':0,
    'socket':0,'Java':0,'SQL':0,'provider_calls':0,'GUI':0,'server_writes':0,
    'new_full_bindings':0,'complete_modes':16,'total_modes':40,'LOADED':False,
}
commands = {
    'schema':'modern.r542.reproducible-build-contract.v1',
    'requirements':'Existing Rust/MSVC toolchain and offline Cargo.lock dependencies for i686-pc-windows-msvc; two pinned manifests under build-time Modern repository root.',
    'repository_build':'cargo build --offline --locked --target i686-pc-windows-msvc --bin LiToModern-485F-Candidate',
    'finite_evidence_build':'pwsh -NoProfile -File tools/build-r542-native-candidate.ps1',
    'subsequent_evidence_name':'Pass a fresh -EvidenceName r542-<name> to preserve existing artifacts; no automatic old PASS replay.',
    'actual_build_command':c['build_command'], 'actual_test_compile_command':c['test_compile_command'],
    'actual_test_command':c['test_command'], 'script':pin(P/'tools/build-r542-native-candidate.ps1'),
    'default_product_build_unchanged':'tools/build.ps1 remains the ordinary product build tool; this round used only the dedicated candidate freeze script and never publishes or executes the player.',
    'reproducibility_claim':'Exact source/dependency/resource pins and commands supplied; not a claim of bit-identical future linker output or portable deployment.',
}
refs = [save(L/'r542-entry-and-version-adoption.json', adoption), save(L/'r542-cli-checks-and-scope.json', checks),
        save(L/'r542-reproducible-build-contract.json', commands), pin(L/'r542-cargo-source-difference.patch')]
human = P/'Native-R542_具名485F候選入口與可重現建置交接.txt'
with human.open('x', encoding='utf-8', newline='\n') as stream:
    stream.write('R542：485F候選入口正式納入repository Cargo binary；本輪僅建置與兩個新CPU CLI檢查。\n\n')
    stream.write('具名binary：LiToModern-485F-Candidate\n來源：src/bin/native_backend_485f_candidate.rs\nCargo.toml只新增一個[[bin]]，default-run=LiToModern與src/main.rs逐byte保持不變；原660個Rust檔未修改，新source合計661。\n\n')
    stream.write(adoption['compiled_call_chain']+'\n\n')
    for k in ('repository_build','finite_evidence_build'):
        stream.write(commands[k]+'\n')
    stream.write('\n新candidate：'+json.dumps(c['executable'],ensure_ascii=False)+'\n')
    stream.write('實際linked library：'+json.dumps(c['linked_library'],ensure_ascii=False)+'\n')
    stream.write('Cargo確認library fresh=true：byte-equal R540 2733EB66。沒有重編library。R540元件仍只存在庫中，不進入485F live呼叫鏈，也沒有將5350、room2/map566/三人或fixed trace clock/gen/milestone/assumedwrite轉為正式契約。\n\n')
    stream.write('資源manifest沿用R532同兩份pins；root由Cargo CARGO_MANIFEST_DIR提供，不需要R532_* env。此候選仍綁建置workspace資源，不是portable發布包。\n')
    stream.write('Accepted::load/configure與所有freshREADY/lease/identity/claim/one-connect限制原樣保留；測試替換唯一launch callback，沒有呼叫Accepted::load、讀READY或消耗claim。\n')
    stream.write('兩個新CLI checks一次PASS，舊測試0次／舊PASS replay0次；candidate／socket／Java／SQL／provider／GUI0次。Regular與testcompile各1次，新candidate warning0；各保留291個原library warning消息。\n')
    stream.write('R532 EXE/library/candidate main與R541root保留；完整WORLD/store与既存攔截仍未觸碰。剩餘fresh有限玩家/GUI與WORLD/owner/wire-clock來源仍依R541列明。整體16/40，LOADED=false，overall in_progress。\n')
refs.append(pin(human))
for name in ('r542-candidate.json','r542-source-snapshot.json','r542-build.jsonl','r542-build.stderr.log',
             'r542-test-build.jsonl','r542-test-build.stderr.log','r542-tests.stdout.log','r542-tests.stderr.log',
             'r542-build-accounting.json','r542-test-build-accounting.json','r542-test-accounting.json',
             'r532-player-candidate.json','r540-native-start-clock-roster-component-handoff.json',
             'r540-final-production-source-snapshot.json','r541-native-room-application-lifecycle-handoff.json'):
    refs.append(pin(L/name))
refs += [c[k] for k in ('executable','linked_library','test_executable','binary_source','default_main','cargo_manifest')]
refs += c['resources'] + preserved
refs += [pin(P/'README.md'),pin(P/'docs/native-backend-485f-candidate.md'),pin(P/'tools/build-r542-native-candidate.ps1'),pin(__file__)]
refs += [pin(T/'source'/n) for n in ('tools/build-r542-native-candidate.ps1','docs/native-backend-485f-candidate.md','README.md')]
root = save(L/'r542-repository-native-candidate-handoff.json', {
    'schema':'modern.r542.repository-native-candidate-handoff.v1','updated_at':NOW,
    'status':'NAMED_REPOSITORY_485F_CANDIDATE_COMPILED_TWO_NEW_CLI_CHECKS_PASSED_LIVE_UNEXECUTED',
    'executable':c['executable'],'actual_linked_library':c['linked_library'],'library_byte_equal_R540':True,
    'source_count':661,'prior_660_sources_unchanged':True,'existing_production_Rust_files_modified':[],
    'new_source':'src/bin/native_backend_485f_candidate.rs','Cargo_bin_registration_added':True,
    'default_run_unchanged':True,'default_main_unchanged':True,'build_command':commands['repository_build'],
    'new_CPU_tests_passed':2,'old_PASS_replays':0,'library_recompiled':False,'new_candidate_warnings':0,
    'player_executions':0,'socket_Java_SQL_provider_GUI':0,'READY_reads':0,'claims':0,
    '5350_contract_promoted':False,'R540_live_integration':False,'WORLD_admitted':False,
    'GUI':0,'LOADED':False,'new_full_bindings':0,'complete_modes':16,'total_modes':40,'overall_completed':False,
    'entry_adoption':refs[0],'checks_and_scope':refs[1],'build_contract':refs[2],
    'remaining':'Fresh finite supervisor-approved live/GUI acceptance and R541 owner/wire-clock/WORLD lifecycle evidence remain. No release/default-protocol transition inferred.',
    'references':refs,
})
unique = {v['path']:v for v in [root]+refs+frozen_inputs}
for v in unique.values():
    verify(v)
verification = save(L/'r542-final-pin-verification.json', {'schema':'modern.r542.final-pin-verification.v1',
    'root':root,'unique_pins_verified':len(unique),'pins':list(unique.values()),
    'frozen_build_inputs_verified':len(frozen_inputs),'prior_Rust_sources_unchanged':660,
    'new_candidate_tests_executed_only_in_initial_build':True,'seal_tests_executed':0})
progress_path = P.parent/'監製進度.json'
progress = json.loads(progress_path.read_text(encoding='utf-8-sig'))
progress['updated_at']=NOW
progress['phase']='R542_repository_explicit_485F_candidate_compiled_CLI_CPU_verified_live_unexecuted'
progress['current_task']='R542具名Cargo binary納入485F顯式入口／同freshREADY原限制；661來源／660原Rust不變、normal main/default-run不變。新EXE EA5E4591實link R540 library2733EB66，2新CLI CPU PASS／claim與所有live執行0。'
progress['overall_status']='in_progress';progress['overall_completed']=False
progress['r542']={'status':'NAMED_REPOSITORY_485F_CANDIDATE_COMPILED_CLI_CPU_ONLY','handoff':root,'verification':verification,
    'executable':c['executable'],'linked_library':c['linked_library'],'source_count':661,'prior_sources_unchanged':660,
    'new_CPU_checks_passed':2,'old_PASS_replays':0,'candidate_build_attempts':1,'test_compile_attempts':1,
    'library_recompile':0,'new_candidate_warnings':0,'default_run_unchanged':True,'default_main_unchanged':True,
    'claims':0,'READY_reads':0,'new_socket':0,'Java_SQL_provider_GUI':0,'new_full_bindings':0,
    'complete_modes':16,'total_modes':40,'GUI':0,'LOADED':False}
done='R542：repository新增具名485F Cargo入口，normal main/default-run與660原Rust來源不變；EA5E4591新candidate實link2733EB66 R540 library，2新CLI CPU PASS，READY/claim/socket/Java/SQL/provider/GUI0。'
if done not in progress['completed']:
    progress['completed'].append(done)
for path in (root['path'],verification['path']):
    if path not in progress['evidence_paths']:
        progress['evidence_paths'].append(path)
with progress_path.open('w',encoding='utf-8',newline='\n') as stream:
    json.dump(progress,stream,ensure_ascii=False,indent=2);stream.write('\n')
print(json.dumps({'root':root,'pins':len(unique),'new_candidate':c['executable'],
                  'linked_library_sha256':c['linked_library']['sha256'],'new_checks_passed':2},ensure_ascii=False))
