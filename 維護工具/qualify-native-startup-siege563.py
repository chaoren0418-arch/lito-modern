"""Independent retained-input, inventory, process and raw readback after exit0."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, ast, hashlib, json, re, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def archive(path, digest, hashes):
    assert sha(path) == digest
    with zipfile.ZipFile(path) as z:
        assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
        for name, h in hashes.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == h

def decode(raw):
    try:
        return raw.decode('utf-8-sig')
    except UnicodeDecodeError:
        text = raw.decode('cp950')
        assert text.encode('cp950') == raw
        return text

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', required=True, type=int)
ap.add_argument('--terminal-exit-code', required=True, type=int)
a = ap.parse_args()
assert a.terminal_exit_code == 0
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-startup-siege563')
c = read(run / 'cohort.json')
assert c['status'] == 'executed_current_selected_map563_thirty_player_TW_extended'
assert c['selected_profiles'] in [['Debug'], ['Release'], ['Debug', 'Release']]
assert c['owned_process'] is None and all(p['exit_code'] == 0 for p in c['process_history'])
pins = read(run / 'source-inputs.json')
assert len(pins) == c['source_files'] == 1024
for name, h in pins.items():
    assert sha(ROOT / name) == h
input_kind = c.get('input_kind','default110')
base = Path(c.get('input_cohort') or c['default_cohort'])
if c['default_completion_sha256'] is not None:
    assert sha(base / 'completion-observation.json') == c['default_completion_sha256']
if input_kind in ['retained-archive-related','retained-geometry-related']:
    completion = read(base / 'completion-observation.json')
    assert sha(base / 'completion-observation.json') == c['input_completion_sha256']
    assert completion['status'] == 'independently_qualified_'+('retained_geometry' if input_kind=='retained-geometry-related' else 'retained_archive')+'_both_profiles'
    assert completion['source_manifest_sha256'] == c['source_manifest_sha256']
    assert completion['recorded_owned_processes_all_terminal']
    assert c['related_ten_target_validation_only' if input_kind=='retained-geometry-related' else 'related_seven_target_validation_only'] and not c['original_default_pipeline_complete']
for profile in c['selected_profiles']:
    assert sha(c['profile_qualifications'][profile]['path']) == c['profile_qualifications'][profile]['sha256']
prior = read(base / 'cohort.json')
npath = Path(c['normal_qualification'])
assert sha(npath) == c['normal_qualification_sha256']
n = read(npath)
normal_root=(npath.parent.parent if npath.parent.name=='qualified' else npath.parent) if input_kind in ['default110','current-default110'] else base
normal = read(normal_root / 'cohort.json')
assert sha(ROOT / '啟動現代客戶端.ps1') == n['entry_sha256']
for profile, info in normal['profiles'].items():
    assert sha(info['normal']['path']) == n['profiles'][profile]['normal_sha256'] == c['normal_mains'][profile]
for name, h in c['validation_pins'].items():
    assert sha(run / name) == sha(ROOT / '維護工具' / name) == h

# Reuse only the bounded independent S7C decoder's AST, without executing its CLI.
decoder = ROOT / '維護工具/qualify-native-startup-G06-controller-case.py'
tree = ast.parse(decoder.read_text(encoding='utf-8-sig'))
fn = next(v for v in tree.body if isinstance(v, ast.FunctionDef) and v.name == 'inventory')
namespace = {}
exec(compile(ast.Module(body=[fn], type_ignores=[]), str(decoder), 'exec'), namespace)
inventory = namespace['inventory']
files = {'S7C-decoder-source.py': decoder, 'runner.py': run / 'runner.py', 'cohort.json': run / 'cohort.json',
    'source-inputs.json': run / 'source-inputs.json'}
if input_kind in ['retained-geometry-related','current-default110']:
    terminal = read(run / 'tool-terminal-observation.json')
    assert terminal['original_session_id'] == a.session_id and terminal['observed_terminal_exit_code'] == a.terminal_exit_code
    assert terminal['source'] == 'functions tools.write_stdin returned actual exit_code'
    assert terminal['terminal_tool_chunk_id']
    files['tool-terminal-observation.json'] = run / 'tool-terminal-observation.json'
elif input_kind == 'retained-archive-related':
    wrapper = read(run / 'wrapper-exit-observation.json')
    assert wrapper['status'] == 'observed_original_wrapper_terminal'
    assert wrapper['original_session_id'] == a.session_id
    if wrapper['observed_exit_code'] is None:
        terminal = read(run / 'tool-terminal-observation.json')
        assert terminal['original_session_id'] == a.session_id and terminal['observed_terminal_exit_code'] == a.terminal_exit_code
        assert terminal['source'] == 'functions tools.write_stdin returned actual exit_code'
        files['tool-terminal-observation.json'] = run / 'tool-terminal-observation.json'
    else:
        assert wrapper['observed_exit_code'] == a.terminal_exit_code
    files['wrapper-exit-observation.json'] = run / 'wrapper-exit-observation.json'
    files['wrapper-exit-observer.ps1'] = run / 'wrapper-exit-observer.ps1'
if (run / 'visual-observations.json').exists():
    visual = read(run / 'visual-observations.json')
    for row in visual['images']:
        assert sha(row['path']) == row['sha256']
    files['visual-observations.json'] = run / 'visual-observations.json'
identities = [(p['pid'], p['creation_utc']) for p in c['process_history']]
if input_kind in ['retained-geometry-related','current-default110']:
    identities.append((c['runner_identity']['pid'],c['runner_identity']['creation_utc']))
profiles = {}
for profile in c['selected_profiles']:
    item = c['profiles'][profile]
    d = Path(item['directory'])
    assert d == run / profile and item['runtime_exit_code'] == item['comparison_exit_code'] == 0
    build = read(d / 'build-inputs.json')
    assert sha(d / 'build-inputs.json') == item['retained_build_manifest_sha256']
    assert len(build['files']) == 1025 and all(sha(ROOT / name) == h for name, h in build['files'].items())
    assert all(build['files'][name] == h for name, h in pins.items())
    archive(build['compiled_source_archive'], build['compiled_source_archive_sha256'], build['files'])
    if input_kind in ['default110','current-default110']:
        products = read(Path(prior['profiles'][profile]['run']) / 'product-retention.json')
    else:
        product_info = prior['profiles'][profile]
        products = dict(products=product_info['products'])
        assert build['qualification_input_kind'] == input_kind and not build['broad_default_regression_claimed']
        assert build['test_products_archive_sha256'] == product_info['test_products_archive']['sha256']
    archive(build['test_products_archive'], build['test_products_archive_sha256'],
        {name + '.exe': v['sha256'] for name, v in products['products'].items()})
    case = Path(item['case']).resolve(strict=True)
    assert case.is_relative_to(ROOT / '驗證紀錄/current/native-arena-portal')
    r = read(case / 'receipt.json')
    runtime = Path(r['runtime']).resolve(strict=True)
    assert runtime.is_relative_to(ROOT / 'LiTo381-Modern/target/current/native-arena-portal')
    assert r['map_id'] == 563 and r['build_profile'] == profile and r['status'] == 'verified_native_siege_portal'
    assert r['test_exit_code'] == r['server_exit_code'] == 0 and r['server']['actual_client_sessions'] == 60
    assert r['server']['memberships_empty'] and r['server']['authority_mutations'] == 0 and not r['server']['sql_access']
    assert sha(case / 'build-inputs.json') == sha(d / 'build-inputs.json')
    assert sha(r['immutable_test_binary']) == r['test_binary_sha256'] == item['test_binary_sha256'] == build['products']['native_arena_portal']['sha256']
    assert r['jar_sha256'] == c['jar_sha256']
    assert 'Picked up JAVA_TOOL_OPTIONS: ' + c['JAVA_TOOL_OPTIONS_for_owned_children_only'] in decode((case / 'server.stderr.txt').read_bytes())
    assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',
        decode((case / 'test.log').read_bytes())) == [('ok', '1', '0', '0', '0', '3')]
    wire = read(case / 'arena-portal-comparison.json')
    terrain = read(case / 'terrain-residence-comparison.json')
    assert wire['receipt_sha256'] == terrain['receipt_sha256'] == sha(case / 'receipt.json')
    assert wire['actual_sessions'] == terrain['actual_sessions'] == 60 and terrain['original_source_sector_count'] == 19
    assert wire['validator_sha256'] == c['validation_pins']['verify-native-arena-portal.py']
    assert terrain['validator_sha256'] == c['validation_pins']['verify-native-terrain-arena.py']
    assert terrain['build_manifest_sha256'] == sha(d / 'build-inputs.json')
    captures = list((runtime / 'captures').glob('*/frames.jsonl'))
    assert len(captures) == 1 and sha(captures[0]) == wire['capture_sha256']
    frames = [json.loads(line) for line in captures[0].read_text(encoding='utf-8-sig').splitlines()]
    assert sum(f['plain'] == 'MASKED_C_LOGIN' for f in frames) == 60
    batches = [inventory(bytes.fromhex(f['plain'])) for f in frames if f['direction'] == 'server_to_client' and f['plain'].startswith('7C ')]
    assert batches and any(v['detail'] for b in batches for v in b.values())
    inventory_checks = []
    for role in ['a', 'b']:
        report = read(case / (role + '-arena-portal.json'))
        before = report['entered']['inventory']['items']
        matches = [i for i, batch in enumerate(batches) if batch == before]
        assert len(matches) == 1, (profile, role, 'Siege inventory must match one complete original S7C batch.')
        assert report['after_walk']['inventory']['items'] == before
        inventory_checks.append(dict(role=role, items=len(before), exact_complete_S7C_batch=matches[0],
            packet_connection_identity_inferred=False))
    for check in terrain['checks']:
        assert len(check['centers']) >= 2 and max(check['decoded_bytes']) <= 64 << 20
        for img in check['images']:
            assert sha(case / img['file']) == img['sha256']
    identities.extend((p['pid'], p['start_utc']) for p in r['owned_processes'])
    for folder, prefix in [(d, profile + '/console/'), (case, profile + '/case/'), (runtime, profile + '/runtime/')]:
        for f in folder.rglob('*'):
            if f.is_file() and f.suffix.lower() not in ['.exe', '.zip']:
                assert f.resolve().is_relative_to(folder) and not f.is_symlink()
                files[prefix + f.relative_to(folder).as_posix()] = f
    profiles[profile] = dict(case=str(case), actual_sessions=60, actual_test_sha256=item['test_binary_sha256'],
        independent_wire=wire, independent_terrain=terrain, inventory_fields_match_recorded_S7C=inventory_checks)

identities = sorted(set(identities))
query = ' OR '.join('ProcessId=' + str(pid) for pid in sorted({p for p, _ in identities}))
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false); @(Get-CimInstance Win32_Process -Filter '" + query + "'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for row in live:
    for pid, created in identities:
        if row['ProcessId'] == pid:
            assert abs((datetime.fromisoformat(row['created']) - datetime.fromisoformat(created)).total_seconds()) > 0.001
out = run / 'qualified'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
(out / 'owned-process-CIM.json').write_bytes(p.stdout)
files.update({'independent-qualifier.py': out / 'independent-qualifier.py', 'owned-process-CIM.json': out / 'owned-process-CIM.json'})
hashes = {name: sha(p) for name, p in files.items()}
raw = out / 'raw.zip'
with zipfile.ZipFile(raw, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
archive(raw, sha(raw), hashes)
q = dict(status='independently_qualified_current_selected_map563_thirty_player_TW_extended',
    validation_input_kind=input_kind, full_current_default_regression_verified=input_kind in ['default110','current-default110'],
    original_session_id=a.session_id, observed_terminal_exit_code=0, profiles=profiles,
    selected_profiles=c['selected_profiles'], actual_test_executables=len(profiles), actual_formal_sessions=60*len(profiles), source_files=1024, compiled_input_files=1025,
    source_manifest_sha256=c['source_manifest_sha256'], normal_mains=c['normal_mains'],
    resource_profile='tw13081901-v1', extended_item_details=True, jar_sha256=c['jar_sha256'],
    common_catalog_sha256=c['common_catalog_sha256'], recorded_owned_identities=len(identities),
    recorded_owned_processes_terminal=True, raw_archive_sha256=sha(raw), raw_files=hashes,
    normal_EXE_runtime_verified=False, GUI_verified=False, physical_input_verified=False,
    whole_G06_complete=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(dict(qualification=str(out / 'qualified.json'), actual_formal_sessions=q['actual_formal_sessions']), ensure_ascii=False))
