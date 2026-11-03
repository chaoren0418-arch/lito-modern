"""Read compiler-only evidence independently; never launch tests or GUI."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, re, subprocess, tomllib, zipfile

R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
V = R / '驗證紀錄/current'
failed = V / 'native-development-typecheck/20261023-184100-336430'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data = lambda v: (json.dumps(v, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('--java-run',type=Path);ap.add_argument('--java-terminal-chunk');ap.add_argument('--observer-review',type=Path);args=ap.parse_args()

w = read(R / '工作佇列.json')
terminal = w['current_development_typecheck_active']
run = Path(terminal['run']).resolve(strict=True)
assert run.is_relative_to(V / 'native-development-typecheck')
session = terminal['original_session_id']
terminal_chunk = terminal['terminal_tool_chunk']
assert isinstance(session, int) and session > 0 and re.fullmatch('[0-9a-f]+', terminal_chunk)
c = read(run / 'cohort.json')
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert w['current_user_requested_goal']['test_execution_deferred_by_user']
assert terminal['observed_actual_exit_code'] == 0
assert terminal['original_identities_terminal'] and c['owned_process'] is None
assert c['status'] == 'metadata_compilation_both_profiles_complete_no_tests_executed'
assert c['tests_executed'] == c['game_or_fixture_executions'] == c['GUI_inputs'] == 0
assert not any(c[k] for k in ['normal_source_modified', 'normal_products_modified', 'candidate_applied'])
assert c['source_manifest_sha256'] == w['current_product_identity']['source_manifest_sha256']
inputs = read(run / 'base-compiled-inputs.json')
assert len(inputs) == 1049 and all(sha(R / n) == h for n, h in inputs.items())
assert all(sha(p) == h for p, h in c['normal_product_pins'].items())
assert all(sha(p) == h for p, h in c['input_pins'].items())
assert c['candidates'] == w['current_isolated_development']['candidates']

# Each generated candidate is the exact archived source with only allowed paths
# redirected. The runtime and test bodies, cfgs and assertions are unchanged.
overlays = {}
for entry in c['candidates'].values():
    cp = Path(entry['path'])
    assert sha(cp) == entry['sha256']
    ca = read(cp)
    assert ca['base_source_manifest_sha256'] == c['source_manifest_sha256']
    assert all(sha(R / n) == h for n, h in ca['base_files'].items())
    for n in ca.get('new_files', []):
        assert not (R / n).exists()
    for n, h in ca['candidate_files'].items():
        assert n not in overlays and sha(cp.parent / n) == h
        overlays[n] = cp.parent / n
assert len(overlays) == sum(entry['files'] for entry in c['candidates'].values())
generated = {n: run / 'generated' / n for n in overlays}
rebuilt = {}
for record in c['source_rewrites']:
    dest = Path(record['generated'])
    assert dest in generated.values()
    if 'origin' in record:
        origin = Path(record['origin'])
        name = next(n for n, p in overlays.items() if p == origin)
        assert dest == generated[name] and dest not in rebuilt
        text = origin.read_text('utf-8-sig')
        for change in record['rewrites']:
            before, after = change['before'], change['after']
            attr = re.fullmatch(r'#\[path\s*=\s*"([^"\n]+)"\]', before)
            inc = re.fullmatch(r'(include_bytes|include_str|include)!\("([^"\n]+)"\)', before)
            assert bool(attr) != bool(inc)
            relative = attr.group(1) if attr else inc.group(2)
            actual = ((R / name).parent / relative).resolve(strict=True)
            if attr:
                actual = overlays.get(actual.relative_to(R).as_posix(), actual)
                expected = '#[path = r#"' + str(actual) + '"#]'
            else:
                expected = inc.group(1) + '!(r#"' + str(actual) + '"#)'
            assert after == expected and before in text
            text = text.replace(before, after, 1)
        rebuilt[dest] = text
    else:
        text = rebuilt[dest]
        for change in record['generated_path_redirects']:
            name = next(n for n, p in overlays.items()
                        if change['before'] == '#[path = r#"' + str(p) + '"#]')
            assert change['after'] == '#[path = r#"' + str(generated[name]) + '"#]'
            assert change['before'] in text
            text = text.replace(change['before'], change['after'])
        rebuilt[dest] = text
assert set(rebuilt) == set(generated.values())
assert all(p.read_text('utf-8') == text for p, text in rebuilt.items())

# Independently remove only added module path attributes and compare the full
# root, preserving all Windows implementation modules and test cfg declarations.
module_map = read(c['module_map'])
assert sha(c['module_map']) == c['module_map_sha256']
normal_root = P / 'src/modern_lito/mod.rs'
normal_lines = normal_root.read_text('utf-8-sig').splitlines()
copy_lines = (run / 'generated/modern_lito.rs').read_text('utf-8').splitlines()
without_attrs = lambda lines: [s for s in lines if not re.fullmatch(r'#\[path\s*=.*\]', s)]
assert without_attrs(normal_lines) == without_attrs(copy_lines)
declarations = []
for i, line in enumerate(normal_lines):
    m = re.fullmatch(r'\s*(?:pub(?:\(crate\))?\s+)?mod\s+(\w+);\s*', line)
    if not m:
        continue
    name = m.group(1)
    prev = re.fullmatch(r'#\[path\s*=\s*"([^"\n]+)"\]', normal_lines[i-1]) if i else None
    origin = normal_root.parent / (prev.group(1) if prev else name + '.rs')
    relative = origin.relative_to(R).as_posix()
    expected = generated.get(relative, origin)
    declarations.append(dict(module=name, path=str(expected)))
assert module_map == declarations and len({m['module'] for m in module_map}) == len(module_map)
paths = re.findall(r'#\[path = r#"([^"]+)"#\]', '\n'.join(copy_lines))
assert paths == [m['path'] for m in module_map]
assert all(Path(p).is_file() for p in paths)
lib_text = (P / 'src/lib.rs').read_text('utf-8-sig')
expected_lib = lib_text.replace('pub mod modern_lito;', '#[path = r#"' + str(run / 'generated/modern_lito.rs') + '"#]\npub mod modern_lito;')
expected_lib = expected_lib.replace('#[path = "shared_v1/morph_auth.rs"]', '#[path = r#"' + str(P / 'src/shared_v1/morph_auth.rs') + '"#]')
assert (run / 'generated/lib.rs').read_text('utf-8') == expected_lib

rows = c['process_history']
assert len(rows) == sum(len(v['checks']) for v in c['profiles'].values()) and set(c['profiles']) == {'Debug', 'Release'}
normal = Path(w['current_native_installed_resource_profile_normal_qualification']['run'])
deps_declared = {d.replace('-', '_') for d in tomllib.loads((P / 'Cargo.toml').read_text('utf-8-sig'))['dependencies']}
checks = {}
for profile, result in c['profiles'].items():
    assert set(result['extern']) == deps_declared and len(deps_declared) == 13
    cargo = [json.loads(line) for line in (normal / profile / 'test-build/stdout.log').read_text('utf-8-sig').splitlines() if line.startswith('{')]
    for name, dep in result['extern'].items():
        assert sha(dep['path']) == dep['sha256']
        assert any(m.get('reason') == 'compiler-artifact' and m['target']['name'] == name and
                   Path(dep['path']).resolve(strict=True) in {Path(p).resolve(strict=True) for p in m['filenames']}
                   for m in cargo)
    task_sources = {
        'library': run / 'generated/lib.rs',
        'library-test-code': run / 'generated/lib.rs',
        'normal-entry-code': P / 'src/main.rs',
        'keyboard-test-code': generated['LiTo381-Modern/tests/native_skill_selection.rs'],
        'blind20-test-code': generated['LiTo381-Modern/tests/native_blind.rs'],
    }
    formal_test=generated.get('LiTo381-Modern/tests/native_teleport_failure.rs')
    if formal_test is not None:
        task_sources['teleport-recovery-formal-code']=formal_test
    assert set(result['checks']) == set(task_sources)
    checks[profile] = {}
    for label, source in task_sources.items():
        output = run / profile / (label + '.rmeta')
        matches = [row for row in rows if Path(row['output']) == run / profile / (label + '-compile')]
        assert len(matches) == 1
        row = matches[0]; cmd = row['command']
        assert row['exit_code'] == 0 and row['finished_utc']
        assert Path(cmd[0]).name == 'rustc.exe' and sha(cmd[0]) == c['input_pins'][cmd[0]]
        assert cmd.count('--emit=metadata') == 1 and not any(a.startswith('--emit') and a != '--emit=metadata' for a in cmd)
        assert cmd[cmd.index('-o') + 1] == str(output) and output.is_file()
        if label == 'library':
            assert output.stat().st_size > 0
        assert cmd.count(str(source)) == 1 and cmd[cmd.index('--target') + 1] == 'x86_64-pc-windows-msvc'
        assert ('--test' in cmd) == (label not in ['library','normal-entry-code'])
        expected_flags = ['opt-level=0', 'debuginfo=2'] if profile == 'Debug' else ['opt-level=z', 'lto=thin', 'codegen-units=16', 'strip=symbols']
        assert [cmd[i+1] for i, a in enumerate(cmd) if a == '-C'] == expected_flags
        extern = {cmd[i+1].split('=', 1)[0]: cmd[i+1].split('=', 1)[1] for i, a in enumerate(cmd) if a == '--extern'}
        expected_extern = {name: dep['path'] for name, dep in result['extern'].items()}
        if label not in ['library', 'library-test-code']:
            expected_extern['lito_modern'] = str(run / profile / 'library.rmeta')
        assert extern == expected_extern
        record = result['checks'][label]
        assert Path(record['metadata']) == output and sha(output) == record['metadata_sha256']
        assert record['compiler_exit_code'] == 0 and not record['test_execution']
        logs = {name: Path(row['output']) / name for name in ['stdout.log', 'stderr.log']}
        stderr = logs['stderr.log'].read_text('utf-8-sig')
        assert not re.search(r'^error(?:\[|:)', stderr, re.M)
        warning = re.findall(r'^warning: (\d+) warnings? emitted', stderr, re.M)
        assert len(warning) <= 1
        assert not list((run / profile).rglob('*.exe'))
        checks[profile][label] = dict(metadata=str(output), metadata_sha256=sha(output), metadata_bytes=output.stat().st_size, compiler_exit_code=0,
            warnings_emitted=int(warning[0]) if warning else 0, logs={n: dict(path=str(p), sha256=sha(p)) for n, p in logs.items()}, test_execution=False)

f = read(failed / 'cohort.json')
assert f['status'] == 'failed_preserved' and f['error'] == 'AssertionError()'
assert f['process_history'] == [] and f['owned_process'] is None
assert f['tests_executed'] == f['game_or_fixture_executions'] == f['GUI_inputs'] == 0
assert "target=out/'generated'/Path(name).name" in (failed / 'runner.py').read_text('utf-8')
assert "target=out/'generated'/name" in (run / 'runner.py').read_text('utf-8')

# Observe original birth identities again; a reused PID is not the old process.
java_compilation=None
java_files={}
java_identities=[]
java_source=overlays.get('LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java')
if java_source is not None:
    assert args.java_run is not None and args.java_terminal_chunk
    java_run=args.java_run.resolve(strict=True);assert java_run.is_relative_to(V/'native-recovery-java-compile')
    jc=read(java_run/'cohort.json');assert jc['status']=='javac_only_completed' and jc['compiler_exit_code']==0
    assert Path(jc['candidate'])==Path(c['candidates']['cancel-recovery']['path']) and jc['candidate_sha256']==c['candidates']['cancel-recovery']['sha256']
    assert jc['tests_executed']==jc['game_or_fixture_executions']==jc['desktop_inputs']==0
    assert all(sha(p)==h for p,h in jc['input_pins'].items()) and all(sha(p)==h for p,h in jc['classes'].items()) and jc['classes']
    assert all(sha(p)==h for p,h in jc['logs'].items())
    assert Path(jc['command'][0]).name=='javac.exe' and str(java_source) in jc['command']
    for name,path in overlays.items():
        if name.endswith('.java'):
            assert str(path) in jc['command'] and sha(path)==jc['input_pins'][str(path)]
    assert jc['command'][jc['command'].index('-d')+1]==str(java_run/'classes')
    assert not list(java_run.rglob('*.exe')) and not list(java_run.rglob('*.jar'))
    expected_formal=read(c['candidates']['cancel-recovery']['path']).get('formal_target_sha256','75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371')
    actual_formal=jc.get('formal_reference_sha256','75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371')
    assert actual_formal==expected_formal
    java_compilation=dict(run=str(java_run),cohort_sha256=sha(java_run/'cohort.json'),observed_original_tool_chunk=args.java_terminal_chunk,
        observed_actual_exit_code=0,compiler_exit_code=0,classes=jc['classes'],input_pins=jc['input_pins'],logs=jc['logs'],
        formal_reference=actual_formal,fixture_execution=False)
    if 'LiTo381-Modern/tests/oracle/NativeRecoveryReadonlySnapshot.java' in overlays:
        assert args.observer_review is not None
        op=args.observer_review.resolve(strict=True);assert op.is_relative_to(java_run)
        obs=read(op);obs_rb=read(op.with_name('readback.json'))
        assert sha(op)==obs_rb['review_sha256'] and sha(op.with_name('raw.zip'))==obs_rb['raw_archive_sha256']
        assert obs['candidate_sha256']==jc['candidate_sha256'] and obs['formal_reference_sha256']==actual_formal
        assert obs['java_compile_cohort_sha256']==sha(java_run/'cohort.json') and obs['observer_classes']==jc['classes']
        assert obs['tests_executed']==obs['classes_instantiated']==obs['GUI_inputs']==0
        assert obs['no_gameplay_method_calls_in_observer'] and obs['no_reflective_game_field_setters'] and obs['no_view_cache_refresh']
        assert sum(len(v) for v in obs['record_field_accessors_verified'].values())==18
        java_compilation.update(observer_readonly_review=str(op),observer_readonly_review_sha256=sha(op),observer_accessors_verified=18)
    java_identities=[jc['pipeline'],jc['compiler']]
    for p in java_run.rglob('*'):
        if p.is_file():java_files['java-compile/'+p.relative_to(java_run).as_posix()]=p
else:assert args.java_run is None
identities = [c['pipeline'], *rows, f['pipeline'],*java_identities]
ids = ','.join(str(r['pid']) for r in identities)
query = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @(" + ids + ")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw = subprocess.check_output(['pwsh', '-NoProfile', '-Command', query])
live = json.loads(raw.decode('utf-8-sig')) if raw.strip() else []
live = [live] if isinstance(live, dict) else live
for identity in identities:
    assert not any(x['ProcessId'] == identity['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z', '+00:00')) - datetime.fromisoformat(identity['creation_utc'])).total_seconds()) < .001 for x in live)

out = run / ('qualified-metadata-only-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'reader.py').write_bytes(Path(__file__).read_bytes())
(out / 'owned-CIM.json').write_bytes(raw)
raw_files = {}
raw_files.update(java_files)
for label, root in [('successful', run), ('preparation-failure', failed)]:
    for p in root.rglob('*'):
        if p.is_file() and (p.parent == root or p.relative_to(root).parts[0] in ['generated', 'Debug', 'Release']):
            raw_files[label + '/' + p.relative_to(root).as_posix()] = p
prior_evidence = Path(w['last_goal_turn_evidence'])
historical_reader_run = V / 'native-development-typecheck/20261023-184117-602031'
rejections = sorted(historical_reader_run.glob('reader-rejection-*'))
assert len(rejections) == 2
if run!=historical_reader_run:
    rejections+=sorted(run.glob('reader-rejection-*'))
for rejection in rejections:
    for p in rejection.iterdir():
        assert p.is_file()
        raw_files[rejection.name + '/' + p.name] = p
raw_files['terminal-checkpoint.json'] = prior_evidence
raw_files['terminal-readback.json'] = prior_evidence.with_name('readback.json')
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n, p in raw_files.items(): z.write(p, n)
with zipfile.ZipFile(out / 'raw.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == sha(p) for n, p in raw_files.items())
qualification = dict(status='metadata_typecheck_only_independently_read_back', source_manifest_sha256=c['source_manifest_sha256'],
    run=str(run), original_session_id=session, terminal_tool_chunk=terminal_chunk, observed_actual_exit_code=0,
    matches_current_candidate_registry=True, candidate_overlay_files=len(overlays),
    candidates=c['candidates'], profiles=checks, compiler_invocations=len(rows), java_compilation=java_compilation, original_process_identities=identities,
    original_identities_terminal=True, full_module_declarations=len(module_map), generated_source_paths_only_verified=True,
    direct_dependencies_per_profile=13, compiled_input_count=len(inputs), normal_product_pins=c['normal_product_pins'],
    preparation_failure=dict(run=str(failed), historical_failure_of_original_session=28991, observed_original_chunk='31baef', observed_actual_exit_code=1,
        reason='Flattened generated basenames collided between runtime and integration native_skill_selection.rs; fixed by retaining relative directories',
        compiler_children_started=0, product_failure=False),
    prior_reader_rejections=[dict(path=str(p / 'rejection.json'), sha256=sha(p / 'rejection.json'),
        original_tool_chunk=read(p / 'rejection.json')['original_tool_chunk'], original_run=str(p.parent), observed_actual_exit_code=1, product_failure=False) for p in rejections],
    raw_archive=str(out / 'raw.zip'), raw_archive_sha256=sha(out / 'raw.zip'),
    tests_executed=0, game_or_fixture_executions=0, desktop_inputs=0, normal_source_modified=False,
    normal_products_modified=False, candidate_applied=False, normal_PE_built=False,
    functional_qualification=False, GUI_full_flow_verified=False, whole_goal_complete=False,
    limitation='Compiler analysis only: library metadata is nonempty; '+str(len(rows)-2)+' binary/test metadata markers are zero bytes, so these checks rely on pinned compiler commands and diagnostics rather than serialized metadata contents. Java class compilation, if recorded, is against its explicit pinned formal reference only. No optimizer/link/PE execution, expected-red reproduction, functional or GUI qualification.',
    recorded_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_bytes(data(qualification))
record = dict(qualification, qualification=str(out / 'qualified.json'), qualification_sha256=sha(out / 'qualified.json'),
    previous_goal_turn_evidence=str(prior_evidence), test_execution_deferred_by_user=True)
(out / 'evidence.json').write_bytes(data(record))

before = read(prior_evidence.with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / n) == h for n, h in before.items())
cancel = read(c['candidates']['cancel-recovery']['path'])
work = '現行三個互斥開發候選（G06.3d取消／來源確認、T鍵目標、#20致盲）已完成Debug／Release各'+str(len(checks['Debug']))+'項metadata編譯／型別檢查，原'+str(session)+' exit0，'+str(len(overlays))+'個覆蓋檔、完整模組、每版13依賴、日誌／metadata及記錄程序終止均獨立核對。'+('G06 Java fixture只對'+java_compilation['formal_reference'][:8]+'編譯出'+str(len(java_compilation['classes']))+'class，未執行；'+('只讀observer與18個record存取器也已靜態核對。' if java_compilation.get('observer_readonly_review') else '') if java_compilation else '')+'零測試執行，未連結正常PE、未套用；各前版結果與工具錯誤保留。桌面／測試依使用者要求暫緩，下一步續原始production item application資源初始化及完整新版owner契約，不能以memory fixture或快照替代。恢復測試後驗G06原版'+str(cancel['baseline_additional_cases'])+'反例／候選'+str(cancel['new_tests_written'])+'案、正式自然33／31秒來源與完整物品接續，再最終library／Release／GUI；正常3E與兩EXE不變，整體未完成。'
staged = {}
for n in before:
    p = R / n
    if p.suffix == '.json':
        j = read(p)
        j['current_development_typecheck_qualification'] = record
        if java_compilation and java_compilation['formal_reference'].startswith('E1498871'):
            assert sha(j['current_formal_reference_identity']['formal'])==java_compilation['formal_reference']
            j['current_E149_recovery_development'].update(compile_complete=True,Rust_metadata_qualification=record['qualification'],
                Java_compile_run=java_compilation['run'],Java_reference_sha256=java_compilation['formal_reference'],
                readonly_observer_review=java_compilation.get('observer_readonly_review'),readonly_observer_review_sha256=java_compilation.get('observer_readonly_review_sha256'),
                verification_source_reference_migrated=True,full_native_owner_initialization_pending=True,runtime_qualified=False)
        j['current_development_typecheck_active'] = dict(j['current_development_typecheck_active'], status='terminal_metadata_only_independently_read_back', qualification=record['qualification'], qualification_sha256=record['qualification_sha256'])
        dc = j['current_isolated_development']
        dc.update(status='three_candidates_metadata_typechecked_tests_deferred', metadata_compilations_executed=len(rows), metadata_typecheck=record['qualification'], metadata_typecheck_sha256=record['qualification_sha256'], metadata_typecheck_matches_current_registry=True, new_revision_requires_metadata_compilation=False,java_compilation=java_compilation)
        for candidate in dc['candidates'].values():
            candidate.update(metadata_typechecked=True, metadata_qualification=record['qualification'])
            assert not any(candidate[k] for k in ['applied', 'built', 'tested'])
        for key in ['current_native_keyboard_target_candidate', 'current_native_blind20_candidate', 'current_native_cancel_recovery_candidate']:
            j[key].update(metadata_typechecked=True, metadata_qualification=record['qualification'])
        j.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        j['current_focus'] = dict(item='G06_3d_cancel_failure_recovery_development', work=work, applies_to_current_source=False, goal_complete=False)
        if n.endswith('next-validation-action.json'): j['next_action'] = work
        staged[n] = data(j)
    else:
        first, sep, rest = p.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[n] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d') + '：' + work + '\n\n' + rest).encode('utf-8')
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n in before: z.write(R / n, n)
with zipfile.ZipFile(out / 'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in before.items())
assert all(sha(R / n) == h for n, h in before.items()) and all(sha(R / n) == h for n, h in inputs.items())
for n, b in staged.items(): (R / n).write_bytes(b)
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={n: sha(R / n) for n in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(evidence=str(out / 'evidence.json'), qualification=str(out / 'qualified.json'), compiler_invocations=len(rows),java_compilation=java_compilation is not None, tests_executed=0, normal_source_unchanged=True), ensure_ascii=False))
