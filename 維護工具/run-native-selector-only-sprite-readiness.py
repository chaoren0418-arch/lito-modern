"""Inspect nine original candidate SPR members without changing game support."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT/'LiTo381-Modern'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('--related-run', type=Path, required=True)
ap.add_argument('--contract-review', type=Path, required=True)
a = ap.parse_args()
related = a.related_run.resolve(strict=True)
review_path = a.contract_review.resolve(strict=True)
assert related.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
assert review_path.is_relative_to(ROOT/'驗證紀錄/current/native-skill-selector-only-contract-review')
q = read(related/'completion-observation.json')
assert q['observed_terminal_exit_code'] == 0 and q['recorded_owned_processes_all_terminal']
pins = read(related/'compiled-inputs.json')
assert len(pins) == 1025
review = read(review_path)
assert review['actual_new_sessions'] == review['actual_new_casts'] == 0
assert sha(ROOT/'LiTo381-Modern/src/modern_lito/native_skills.rs') == review['evidence_pins']['LiTo381-Modern/src/modern_lito/native_skills.rs']
assert {row['skill_id'] for row in review['selector_only_missing_batch']} == {14,31,32,49,52,54,72,78,206}
for name, digest in review['evidence_pins'].items():
    assert sha(ROOT/name) == digest
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = ROOT/'驗證紀錄/current/native-selector-only-sprite-readiness'/stamp
out.mkdir(parents=True)
runtime = PROJECT/'target/current/native-selector-only-sprite-readiness'/stamp
runtime.mkdir(parents=True)
probe = ROOT/'維護工具/oracle/NativeSelectorOnlySpriteReadiness.rs'
(out/'probe.rs').write_bytes(probe.read_bytes())
(out/'runner.py').write_bytes(Path(__file__).read_bytes())
archives = [Path(r'D:\lineage開發\0-客戶端\Text.idx'), Path(r'D:\lineage開發\0-客戶端\Text.pak')]
archive_pins = {str(p):sha(p) for p in archives}
libraries, native = {}, set()
for line in (related/'Debug/build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):
        continue
    m = json.loads(line)
    if m.get('reason') == 'build-script-executed':
        native.update(m.get('linked_paths', []))
    if m.get('reason') == 'compiler-artifact' and m['target']['name'] in ['lito_modern', 'serde_json', 'sha2']:
        paths = [p for p in m['filenames'] if p.endswith('.rlib')]
        if paths:
            libraries[m['target']['name']] = paths[0]
assert set(libraries) == {'lito_modern', 'serde_json', 'sha2'}
state = dict(status='preparing_original_selector_effect_catalog_inspection', source_manifest_sha256=q['source_manifest_sha256'],
    related_qualification=str(related/'completion-observation.json'), related_qualification_sha256=sha(related/'completion-observation.json'),
    contract_review=str(review_path), contract_review_sha256=sha(review_path), probe_sha256=sha(probe),
    original_Text_archive_pins=archive_pins, libraries={n:dict(path=p,sha256=sha(p)) for n,p in libraries.items()},
    actual_new_sessions=0, actual_new_casts=0, product_source_changed=False, native_skill_support_added=False,
    pixel_equivalence_verified=False, native_caller_bank_binding_verified=False,
    process_history=[], owned_process=None, GUI_verified=False, whole_goal_complete=False)

def stable():
    assert all(sha(ROOT/name) == h for name,h in pins.items())
    assert sha(probe) == state['probe_sha256'] and sha(out/'probe.rs') == state['probe_sha256']
    assert all(sha(info['path']) == info['sha256'] for info in state['libraries'].values())

env = dict(os.environ, CARGO_MANIFEST_DIR=str(PROJECT), PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8',
    TEMP=str(PROJECT/'target/current/scratch'), TMP=str(PROJECT/'target/current/scratch'))

def execute(args, folder):
    stable()
    folder.mkdir()
    with (folder/'stdout.log').open('wb') as stdout, (folder/'stderr.log').open('wb') as stderr:
        p = subprocess.Popen(args, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        k = ctypes.WinDLL('kernel32', use_last_error=True)
        k.GetProcessTimes.argtypes = [wintypes.HANDLE]+[ctypes.POINTER(wintypes.FILETIME)]*4
        k.GetProcessTimes.restype = wintypes.BOOL
        assert k.GetProcessTimes(wintypes.HANDLE(int(p._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime|(times[0].dwHighDateTime<<32)
        item = dict(pid=p.pid, creation_utc=(datetime(1601,1,1,tzinfo=timezone.utc)+timedelta(microseconds=ticks//10)).isoformat(), arguments=args)
        state['process_history'].append(item)
        state['owned_process'] = item
        save(out/'receipt.json', state)
        code = p.wait()
        item['exit_code'] = code
        state['owned_process'] = None
        save(out/'receipt.json', state)
    stable()
    return code

exe = runtime/'selector-only-catalog.exe'
args = [str(RUST/'rustc.exe'), str(probe), '--edition=2021', '--crate-name', 'native_selector_only_catalog',
    '--target', 'x86_64-pc-windows-msvc', '-L', 'dependency='+str(PROJECT/'target/current/x86_64-pc-windows-msvc/debug/deps'),
    '-L', 'dependency='+str(PROJECT/'target/current/debug/deps'), '-o', str(exe)]
for name,p in libraries.items():
    args += ['--extern', name+'='+p]
for p in sorted(native):
    args += ['-L', p]
save(out/'receipt.json', state)
print(json.dumps(dict(output=str(out)), ensure_ascii=False), flush=True)
try:
    state['status'] = 'compiling_readonly_original_catalog_probe'
    assert execute(args, out/'build') == 0, 'Retain compile failure before changing this diagnostic'
    state.update(status='reading_original_selector_effect_catalog', executable=str(exe), executable_sha256=sha(exe))
    assert execute([str(exe)], out/'runtime') == 0
    text = (out/'runtime/stdout.log').read_text(encoding='utf-8-sig')
    matches = re.findall(r'^SELECTOR_SPR_READINESS=(.+)$', text, re.M)
    assert len(matches) == 1 and not (out/'runtime/stderr.log').read_bytes().strip()
    data = json.loads(matches[0])
    assert len(data['rows']) == 9 and {row['skill_id'] for row in data['rows']} == {14,31,32,49,52,54,72,78,206}
    for row in data['rows']:
        contract = next(v for v in review['selector_only_missing_batch'] if v['skill_id'] == row['skill_id'])
        assert row['catalog_sprite_id'] == contract['catalog_graphics']
        if 'frames' in row:
            assert row['frames'] and row['asset_pixels_decoded'] is True
    assert all(sha(name) == h for name,h in archive_pins.items())
    save(out/'sprite-observation.json', data)
    state.update(status='inspected_nine_selector_candidate_SPR_members_partial_readiness', resolved_definitions=sum('frames' in row for row in data['rows']),
        definition_errors=[row for row in data['rows'] if 'frames' not in row], original_Text_archives_unchanged=True)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    save(out/'receipt.json', state)
files = {p.relative_to(out).as_posix():p for p in out.rglob('*') if p.is_file()}
files['probe.exe'] = exe
hashes = {name:sha(p) for name,p in files.items()}
with zipfile.ZipFile(out/'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name,p in files.items():
        z.write(p, name)
with zipfile.ZipFile(out/'raw.zip') as z:
    assert set(z.namelist()) == set(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
save(out/'readback.json', dict(status=state['status'], source_manifest_sha256=state['source_manifest_sha256'],
    resolved_definitions=state['resolved_definitions'], original_Text_archives_unchanged=True,
    actual_new_sessions=0, actual_new_casts=0, product_source_changed=False, native_skill_support_added=False,
    pixel_equivalence_verified=False, native_caller_bank_binding_verified=False, GUI_verified=False,
    raw_archive_sha256=sha(out/'raw.zip'), raw_files=hashes, whole_goal_complete=False))
print(json.dumps(dict(output=str(out), resolved_definitions=state['resolved_definitions'], native_skill_support_added=False), ensure_ascii=False))
