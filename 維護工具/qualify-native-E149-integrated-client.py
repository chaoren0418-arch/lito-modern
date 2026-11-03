"""Read compiler records and PE resources without executing client images."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ast, hashlib, json, struct, subprocess, zipfile
from native_E149_recovery_contract import R, P, SOURCE, FORMAL, read, sha, encoded, owned, require, check_pins

T = R / '維護工具'
V = R / '驗證紀錄/current'


def image_metadata(path):
    """Bounded raw-file PE reader; no LoadLibrary, process, or code execution."""
    data = Path(path).read_bytes()
    def u16(at):
        require(0 <= at <= len(data) - 2, 'PE u16 outside file'); return struct.unpack_from('<H', data, at)[0]
    def u32(at):
        require(0 <= at <= len(data) - 4, 'PE u32 outside file'); return struct.unpack_from('<I', data, at)[0]
    require(data[:2] == b'MZ', 'DOS signature absent')
    pe = u32(0x3c); require(data[pe:pe + 4] == b'PE\0\0', 'PE signature absent')
    machine = u16(pe + 4); count = u16(pe + 6); size = u16(pe + 20); optional = pe + 24
    require(machine == 0x8664 and 0 < count <= 96 and size >= 240 and u16(optional) == 0x20b, 'not Windows x64 PE32+')
    require(u16(optional + 68) == 2 and u16(pe + 22) & 2, 'normal Windows GUI subsystem absent')
    table = optional + size; sections = []
    for i in range(count):
        at = table + 40 * i
        sections.append((u32(at + 12), u32(at + 16), u32(at + 20)))
    def file_at(rva, length):
        require(length >= 0, 'negative mapped length')
        if rva + length <= u32(optional + 60):
            require(rva + length <= len(data), 'header RVA outside file'); return rva
        matches = [raw + rva - base for base, rawsize, raw in sections if base <= rva and rva + length <= base + rawsize]
        require(len(matches) == 1 and matches[0] + length <= len(data), 'resource RVA not uniquely backed by file bytes')
        return matches[0]
    require(u32(optional + 108) >= 3, 'resource data directory absent')
    root = u32(optional + 128); root_size = u32(optional + 132)
    require(root and 16 <= root_size <= 16 * 1024 * 1024, 'resource directory size invalid')
    base = file_at(root, root_size); leaves = {}; seen = set()
    def walk(offset, keys):
        require(len(keys) <= 2 and offset not in seen and 0 <= offset <= root_size - 16, 'resource directory invalid or cyclic')
        seen.add(offset); entries = u16(base + offset + 12) + u16(base + offset + 14)
        require(entries <= 4096 and offset + 16 + entries * 8 <= root_size, 'resource directory entries exceed section')
        for i in range(entries):
            at = base + offset + 16 + i * 8; name = u32(at); value = u32(at + 4)
            # Only integer type/name/language resources are relevant here.
            if name & 0x80000000: continue
            key = (*keys, name)
            if not keys and name not in [16, 24]: continue
            if value & 0x80000000:
                walk(value & 0x7fffffff, key)
            else:
                require(len(key) == 3 and value <= root_size - 16, 'resource data entry outside section')
                rva = u32(base + value); length = u32(base + value + 4)
                require(0 < length <= 1024 * 1024 and key not in leaves, 'resource payload invalid')
                at = file_at(rva, length); leaves[key] = data[at:at + length]
    walk(0, ())
    manifest = [(k, v) for k, v in leaves.items() if k[0] == 24 and k[1] == 1]
    require(len(manifest) == 1, 'exactly one normal app manifest required')
    version = {str(k): hashlib.sha256(v).hexdigest().upper() for k, v in leaves.items() if k[0] == 16}
    require(version, 'Windows version resource missing')
    key, xml = manifest[0]
    return dict(machine=hex(machine), optional_magic='0x20b', subsystem=2, subsystem_name='Windows GUI',
                manifest_resource_key=list(key), manifest_sha256=hashlib.sha256(xml).hexdigest().upper(),
                manifest_bytes=len(xml), version_resources=version)


def qualify(run, terminal_chunk):
    run = owned(run.resolve(strict=True), V / 'native-E149-integrated-client-build')
    c = read(run / 'cohort.json'); overlay = read(c['overlay']); w = read(R / '工作佇列.json')
    require(c['status'] == 'four_compilers_exited_no_output_executed' and c['owned_process'] is None, 'build not complete')
    require(c['source_manifest_sha256'] == SOURCE and c['formal_target_sha256'] == FORMAL, 'build identity differs')
    require(c['tests_executed'] == 0 and not c['outputs_executed'] and not c['candidate_applied'], 'development scope differs')
    require(not w['active_workloads'] and not w['active_workload_sessions'] and not w['source_and_corresponding_QA_frozen'], 'another workload active')
    require(w['current_user_requested_goal']['test_execution_deferred_by_user'], 'tests remain deferred')
    check_pins(c['input_pins']); check_pins(c['normal_product_pins'])
    require(all(sha(p) == h for p, h in c['external_toolchain_dependency_pins'].items()), 'external build input changed')
    require(sha(w['current_formal_reference_identity']['formal']) == FORMAL and sha(c['overlay']) == c['overlay_sha256'], 'formal target or overlay changed')
    require(overlay['original_registry'] == w['current_isolated_development']['candidates'] and
            overlay['original_fixture_binding'] == w['current_E149_fixture_binding'], 'canonical registry or binding changed')
    require((len(overlay['candidate_files']), len(overlay['runtime_files']), len(overlay['QA_files']), len(overlay['Java_files'])) == (19, 8, 8, 3), 'overlay roles differ')
    check_pins({overlay['candidate_paths'][n]: h for n, h in overlay['candidate_files'].items()})
    require(sha(c['original_metadata_cohort']) == c['original_metadata_cohort_sha256'], 'original metadata cohort changed')
    old = read(c['original_metadata_cohort'])
    require([(r['profile'], r['kind']) for r in c['compiler_runs']] ==
            [(pr, kind) for pr in ['Debug', 'Release'] for kind in ['library', 'normal-entry']], 'compiler order differs')
    images = {}; libraries = {}
    for row in c['compiler_runs']:
        require(row['exit_code'] == 0 and row['finished_utc'] and not row.get('original_handle_cleanup'), 'compiler did not finish normally')
        check_pins(row['logs']); output = owned(row['output'], P / 'target/current/verification/native-E149-integrated-client')
        require(sha(output) == row['output_sha256'] and output.stat().st_size == row['output_bytes'], 'build output changed')
        profile = row['profile']; kind = row['kind']
        command = list(next(r['command'] for r in old['compiler_runs'] if r['profile'] == profile and r['kind'] == 'library'))
        command[command.index('--emit=metadata')] = '--emit=link'; command[command.index('-o') + 1] = str(output)
        if kind == 'normal-entry':
            command.remove('--crate-type=rlib'); command[command.index(old['generated_root'])] = str(P / 'src/main.rs')
            command[command.index('--crate-name') + 1] = 'LiToModern'
            command += ['--extern', 'lito_modern=' + libraries[profile]['path']]
            for lib in c['Windows_resource_libraries'][profile]: command += ['-l', lib]
        for flag in c['native_link_search_paths'][profile]: command += ['-L', flag]
        require(row['command'] == command and '--test' not in command and '--cfg' not in command, 'actual normal entry compiler flags differ')
        info = dict(path=str(output), sha256=sha(output), bytes=output.stat().st_size)
        if kind == 'library':
            with output.open('rb') as f: require(f.read(8) == b'!<arch>\n', 'rlib archive missing')
            libraries[profile] = info
        else:
            pe = image_metadata(output)
            baseline_path = next(p for p in c['normal_product_pins'] if Path(p).parent.name == profile.lower())
            baseline = image_metadata(baseline_path)
            require(pe['manifest_sha256'] == sha(P / 'windows/app.manifest') == baseline['manifest_sha256'], 'embedded manifest differs')
            require(pe['version_resources'] == baseline['version_resources'], 'normal Windows version resource differs')
            info['PE'] = pe; images[profile] = info
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
    tools = [T / n for n in ['build-native-E149-integrated-client.py', 'qualify-native-E149-integrated-client.py']]
    for p in tools: compile(ast.parse(p.read_text('utf-8-sig'), filename=str(p)), str(p), 'exec')
    out = owned(V / 'native-E149-integrated-client-development' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')); out.mkdir(parents=True)
    (out / 'original-terminal-CIM.json').write_bytes(raw)
    record = dict(status='unselected_integrated_normal_client_two_profiles_linked_unrun', source_manifest_sha256=SOURCE,
                  formal_target_sha256=FORMAL, run=str(run), compile_cohort_sha256=sha(run / 'cohort.json'),
                  overlay=c['overlay'], overlay_sha256=c['overlay_sha256'], overlay_files=19, runtime_files=8, QA_files=8, Java_files=3,
                  observed_original_tool_chunk=terminal_chunk, observed_actual_exit_code=0, compiler_exit_codes=[0] * 4,
                  images=images, libraries=libraries, original_compiler_identities_terminal=True,
                  source_pins={str(p): sha(p) for p in [*tools, T / 'native-E149-integrated-client.md']},
                  canonical_registry_unchanged=overlay['original_registry'], current_fixture_binding_unchanged=w['current_E149_fixture_binding'],
                  current_formal_suite_unchanged=w['current_E149_recovery_suite_development'],
                  normal_source_unchanged=True, normal_PEs_unchanged=True, canonical_overlay_selected=False,
                  production_applied=False, tests_executed=0, outputs_executed=False, GUI_inputs=0, GUI_verified=False,
                  full_library_qualified=False, full_G06_qualified=False, whole_goal_complete=False,
                  limitations=['compiler/linker and raw PE resources only; no normal GUI or game code executed',
                               'eight QA and three Java files retained in overlay; not compiled by these four commands',
                               'new images inherit no historical runtime, G06, API, audio or GUI qualification'],
                  recorded_utc=datetime.now(timezone.utc).isoformat())
    (out / 'qualified-development.json').write_bytes(encoded(record))
    record.update(qualification=str(out / 'qualified-development.json'), qualification_sha256=sha(out / 'qualified-development.json'),
                  previous_goal_turn_evidence=w['last_goal_turn_evidence'])
    archive = {'cohort.json': run / 'cohort.json', 'overlay.json': Path(c['overlay']),
               'qualified-development.json': out / 'qualified-development.json', 'original-terminal-CIM.json': out / 'original-terminal-CIM.json'}
    archive.update({'tools/' + p.name: p for p in [*tools, T / 'native-E149-integrated-client.md']})
    archive.update({'source/' + n: Path(overlay['candidate_paths'][n]) for n in overlay['candidate_files']})
    for row in c['compiler_runs']:
        for p in row['logs']: archive[row['profile'] + '-' + row['kind'] + '/' + Path(p).name] = Path(p)
    prep = V / 'native-E149-integrated-client-preparation-failures'
    for folder in prep.iterdir():
        for p in folder.iterdir():
            if p.is_file(): archive['preparation-failures/' + folder.name + '/' + p.name] = p
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for n, p in archive.items(): z.write(owned(p), n)
    with zipfile.ZipFile(out / 'raw.zip') as z:
        require(all(hashlib.sha256(z.read(n)).hexdigest().upper() == sha(p) for n, p in archive.items()), 'raw archive differs')
    record.update(raw_archive=str(out / 'raw.zip'), raw_archive_sha256=sha(out / 'raw.zip'))
    (out / 'evidence.json').write_bytes(encoded(record))
    work = ('G06取消恢復、T鍵及#12/#21/#20/#29/#79已整理為未選入的十九檔來源清單（八Runtime、八Rust QA、三Java），逐項保留五處重疊來源關係。'
            '正常src/main.rs兩版各rlib與Windows GUI PE已完成四次編譯／連結退出0；PE只讀核對AMD64／PE32+、GUI subsystem、原manifest及版本resource。'
            '八Runtime沿完整362模組，六nested模組與原raster fixture路徑逐項核對；QA／Java只保留來源，不宣告本批編譯。'
            '新兩PE完全未執行，未選入registry／套用／資格轉移；正常1049輸入、兩正常EXE及G06原綁定／16案保持原狀。'
            '三次建置準備guard失敗已封存，均在Rust前。測試與桌面暫緩；恢復後按原元件／G06計畫，再驗合併版技能、完整library兩版、Release及正常GUI，確認後才交人工。')
    staged = {}
    for n in before:
        p = R / n
        if p.suffix == '.json':
            value = read(p); value['current_E149_integrated_client_development'] = record
            value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'),
                         last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
            value['current_focus'] = dict(item='integrated_normal_client_compiler_linker_only', work=work, applies_to_normal_EXE=False, goal_complete=False)
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
