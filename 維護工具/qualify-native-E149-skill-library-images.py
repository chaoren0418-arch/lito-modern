"""Read the six normal/library build commands and four unexecuted PE images."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ast, hashlib, json, struct, subprocess, zipfile
from native_E149_recovery_contract import R, P, SOURCE, FORMAL, read, sha, encoded, owned, require, check_pins

from native_E149_skill_image_metadata import image_metadata
T = R / '維護工具'
V = R / '驗證紀錄/current'


def qualify(run, terminal_chunk):
    run = owned(run.resolve(strict=True), V / 'native-E149-skill-library-linked-build')
    c = read(run / 'cohort.json'); overlay = read(c['overlay']); w = read(R / '工作佇列.json')
    require(c['status'] == 'six_compilers_exited_no_output_executed' and c['owned_process'] is None, 'build not complete')
    require(c['source_manifest_sha256'] == SOURCE and c['formal_target_sha256'] == FORMAL, 'build identity differs')
    require(c['tests_executed'] == 0 and not c['outputs_executed'] and not c['candidate_applied'], 'development scope differs')
    require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'], 'another workload active')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'], 'tests remain deferred')
    check_pins(c['input_pins']); check_pins(c['normal_product_pins'])
    require(all(sha(p) == h for p, h in c['external_toolchain_dependency_pins'].items()), 'external build input changed')
    require(sha(w['current_formal_reference_identity']['formal']) == FORMAL and sha(c['overlay']) == c['overlay_sha256'], 'formal target or overlay changed')
    require(overlay['original_registry'] == w['current_isolated_development']['candidates'] and
            overlay['original_fixture_binding'] == w['current_E149_fixture_binding'], 'canonical registry or binding changed')
    require((len(overlay['candidate_files']), len(overlay['runtime_files']), len(overlay['QA_files']), len(overlay['Java_files'])) == (21, 8, 10, 3), 'overlay roles differ')
    check_pins({overlay['candidate_paths'][n]: h for n, h in overlay['candidate_files'].items()})
    require(sha(c['original_metadata_cohort']) == c['original_metadata_cohort_sha256'], 'original metadata cohort changed')
    old = read(c['original_metadata_cohort'])
    require([(r['profile'], r['kind']) for r in c['compiler_runs']] ==
            [(pr, kind) for pr in ['Debug', 'Release'] for kind in ['library', 'normal-entry', 'library-test']], 'compiler order differs')
    images = {}; test_images = {}; libraries = {}
    for row in c['compiler_runs']:
        require(row['exit_code'] == 0 and row['finished_utc'] and not row.get('original_handle_cleanup'), 'compiler did not finish normally')
        check_pins(row['logs']); output = owned(row['output'], P / 'target/current/verification/native-E149-skill-library-images')
        require(sha(output) == row['output_sha256'] and output.stat().st_size == row['output_bytes'], 'build output changed')
        profile = row['profile']; kind = row['kind']
        command = list(next(r['command'] for r in old['compiler_runs'] if r['profile'] == profile and r['kind'] == 'library'))
        command[command.index('--emit=metadata')] = '--emit=link'; command[command.index('-o') + 1] = str(output)
        if kind == 'normal-entry':
            command.remove('--crate-type=rlib'); command[command.index(old['generated_root'])] = str(P / 'src/main.rs')
            command[command.index('--crate-name') + 1] = 'LiToModern'
            command += ['--extern', 'lito_modern=' + libraries[profile]['path']]
        elif kind == 'library-test':
            command.remove('--crate-type=rlib'); command += ['--test']
        if kind != 'library':
            for lib in c['Windows_resource_libraries'][profile]: command += ['-l', lib]
        for flag in c['native_link_search_paths'][profile]: command += ['-L', flag]
        require(row['command'] == command and ('--test' in command) == (kind == 'library-test') and '--cfg' not in command, 'actual normal entry compiler flags differ')
        info = dict(path=str(output), sha256=sha(output), bytes=output.stat().st_size)
        if kind == 'library':
            with output.open('rb') as f: require(f.read(8) == b'!<arch>\n', 'rlib archive missing')
            libraries[profile] = info
        else:
            pe = image_metadata(output,3 if kind == 'library-test' else 2)
            baseline_path = next(p for p in c['normal_product_pins'] if Path(p).parent.name == profile.lower())
            baseline = image_metadata(baseline_path,2)
            require(pe['manifest_sha256'] == sha(P / 'windows/app.manifest') == baseline['manifest_sha256'], 'embedded manifest differs')
            require(pe['version_resources'] == baseline['version_resources'], 'normal Windows version resource differs')
            info['PE'] = pe
            (test_images if kind == 'library-test' else images)[profile] = info
    identities = [c['pipeline'], *[r['original_process'] for r in c['compiler_runs']]]
    ids = ','.join(str(r['pid']) for r in identities)
    query = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @(" + ids + ")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
    raw = subprocess.check_output(['pwsh', '-NoProfile', '-NonInteractive', '-Command', query], creationflags=subprocess.CREATE_NO_WINDOW)
    live = json.loads(raw.decode('utf-8-sig')) if raw.strip() else []; live = [live] if isinstance(live, dict) else live
    for row in identities:
        require(not any(x['ProcessId'] == row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z', '+00:00')) -
                    datetime.fromisoformat(row['creation_utc'])).total_seconds()) < .001 for x in live), 'original compiler process still live')
    before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
    require(len(before) == 8 and all(sha(R / n) == h for n, h in before.items()), 'checkpoint changed')
    tools = [T / n for n in ['build-native-E149-skill-library-images.py', 'qualify-native-E149-skill-library-images.py', 'native_E149_skill_image_metadata.py']]
    for p in tools: compile(ast.parse(p.read_text('utf-8-sig'), filename=str(p)), str(p), 'exec')
    out = owned(V / 'native-E149-skill-library-images-development' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')); out.mkdir(parents=True)
    (out / 'original-terminal-CIM.json').write_bytes(raw)
    base_inputs = read(c['base_compiled_inputs'])
    require(len(base_inputs) == 1049 and all(sha(R / n) == h for n, h in base_inputs.items()), 'normal source inputs changed')
    projected_inputs = dict(base_inputs) | overlay['candidate_files']
    projection = dict(schema='E149-logical-source-projection-v1', base_source_manifest_sha256=SOURCE,
                      overlay=c['overlay'], overlay_sha256=c['overlay_sha256'], files=projected_inputs,
                      physical_compiler_root=c['generated_root'], physical_module_graph=c['generated_mod'],
                      note='logical input projection; actual cfg/test selection and physical wrappers are bound by compiler commands and input pins')
    (out / 'source-projection.json').write_bytes(encoded(projection))
    record = dict(status='unselected_latest_normal_and_full_library_images_linked_unrun', source_manifest_sha256=SOURCE,
                  formal_target_sha256=FORMAL, run=str(run), compile_cohort_sha256=sha(run / 'cohort.json'),
                  overlay=c['overlay'], overlay_sha256=c['overlay_sha256'], overlay_files=21, runtime_files=8, QA_files=10, Java_files=3,
                  observed_original_tool_chunk=terminal_chunk, observed_actual_exit_code=0, compiler_exit_codes=[0] * 6,
                  images=images, library_test_images=test_images, libraries=libraries, original_compiler_identities_terminal=True,
                  projected_input_manifest=str(out / 'source-projection.json'), projected_input_manifest_sha256=sha(out / 'source-projection.json'), projected_input_count=len(projected_inputs),
                  source_pins={str(p): sha(p) for p in [*tools, T / 'native-E149-skill-library-images.md']},
                  canonical_registry_unchanged=overlay['original_registry'], current_fixture_binding_unchanged=w['current_E149_fixture_binding'],
                  current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],
                  normal_source_unchanged=True, normal_PEs_unchanged=True, canonical_overlay_selected=False,
                  production_applied=False, tests_executed=0, outputs_executed=False, GUI_inputs=0, GUI_verified=False,
                  full_library_qualified=False, full_G06_qualified=False, whole_goal_complete=False,
                  limitations=['compiler/linker and raw PE resources only; no normal GUI or game code executed',
                               'eight standalone integration files and three Java sources not compiled by these six commands; two internal QA modules are in library harness',
                               'new images inherit no historical runtime, G06, API, audio or GUI qualification'],
                  recorded_utc=datetime.now(timezone.utc).isoformat())
    (out / 'qualified-development.json').write_bytes(encoded(record))
    record.update(qualification=str(out / 'qualified-development.json'), qualification_sha256=sha(out / 'qualified-development.json'),
                  previous_goal_turn_evidence=w['last_goal_turn_evidence'])
    archive = {'cohort.json': run / 'cohort.json', 'overlay.json': Path(c['overlay']), 'source-projection.json': out / 'source-projection.json',
               'qualified-development.json': out / 'qualified-development.json', 'original-terminal-CIM.json': out / 'original-terminal-CIM.json'}
    archive.update({'tools/' + p.name: p for p in [*tools, T / 'native-E149-skill-library-images.md']})
    archive.update({'source/' + n: Path(overlay['candidate_paths'][n]) for n in overlay['candidate_files']})
    for row in c['compiler_runs']:
        for p in row['logs']: archive[row['profile'] + '-' + row['kind'] + '/' + Path(p).name] = Path(p)
    prep = V / 'native-E149-skill-library-build-preparation-failures'
    for folder in (prep.iterdir() if prep.exists() else []):
        for p in folder.iterdir():
            if p.is_file(): archive['preparation-failures/' + folder.name + '/' + p.name] = p
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for n, p in archive.items(): z.write(owned(p), n)
    with zipfile.ZipFile(out / 'raw.zip') as z:
        require(all(hashlib.sha256(z.read(n)).hexdigest().upper() == sha(p) for n, p in archive.items()), 'raw archive differs')
    record.update(raw_archive=str(out / 'raw.zip'), raw_archive_sha256=sha(out / 'raw.zip'))
    (out / 'evidence.json').write_bytes(encoded(record))
    work = ('最新21檔候選的八Runtime沿完整362模組，兩版各rlib／正常src/main.rs GUI PE／完整library harness console PE共六次編譯連結退出0。'
            '六nested模組及原raster fixture路徑逐項核對；四PE只讀核對AMD64／PE32+、subsystem2或3及原manifest／版本resource。'
            '兩份library-test映像與兩份新正常GUI映像完全未執行，連inventory／--list亦未跑；不移記歷史1975過或舊API／G06／GUI資格。'
            '八個獨立integration檔及三Java未由本批編譯，兩內部QA模組納入library harness；G06 binding／16案640筆及正常1049輸入／兩正常PE不變。'
            '候選未套用／未選入registry；恢復測試後依原元件／G06，核對實際library inventory並跑Debug／Release、整合／資源、Release與正常GUI，確認後才交人工。測試與桌面暫緩維持。')
    staged = {}
    for n in before:
        p = R / n
        if p.suffix == '.json':
            value = read(p); value['current_E149_skill_library_link_development'] = record
            value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'),
                         last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
            value['current_focus'] = dict(item='latest_skill_normal_and_full_library_harness_compiler_linker_only', work=work, applies_to_normal_EXE=False, goal_complete=False)
            if n.endswith('next-validation-action.json'): value['next_action'] = work
            staged[n] = encoded(value)
        else:
            _, sep, rest = p.read_text('utf-8-sig').partition('\n\n'); require(sep, 'checkpoint document format differs')
            staged[n] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d') + '：' + work + '\n\n' + rest).encode('utf-8')
    with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for n in before: z.write(R / n, n)
    with zipfile.ZipFile(out / 'before.zip') as z:
        require(all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in before.items()), 'checkpoint archive differs')
    require(all(sha(R / n) == h for n, h in before.items()), 'concurrent checkpoint writer')
    check_pins(c['normal_product_pins'])
    for n, blob in staged.items(): (R / n).write_bytes(blob)
    require(all((R / n).read_bytes() == blob for n, blob in staged.items()), 'checkpoint write/readback differs')
    (out / 'readback.json').write_bytes(encoded(dict(before_files=before, after_files={n: sha(R / n) for n in before},
                                                  before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
    print(json.dumps(dict(evidence=str(out / 'evidence.json'), images=images, tests_executed=0, outputs_executed=False, eight_files_read_back=True), ensure_ascii=False))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__); parser.add_argument('run', type=Path)
    parser.add_argument('--terminal-chunk', required=True); args = parser.parse_args(); qualify(args.run, args.terminal_chunk)
