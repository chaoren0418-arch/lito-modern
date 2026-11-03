"""Independent readback of the five current original-resource bodies per profile."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, re, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def verify_normal_reference(path, compiled):
    path=Path(path).resolve(strict=True)
    normal=read(path)
    assert normal['status'] in {
        'independently_qualified_retained_archive_both_profiles',
        'independently_qualified_presentation_source_owner_both_profiles',
    }
    if normal['status']=='independently_qualified_presentation_source_owner_both_profiles':
        assert path.parent.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
        assert normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal']
        assert set(normal['profiles'])=={'Debug','Release'}
        assert len(compiled)==1025 and read(path.parent/'compiled-inputs.json')==compiled
        source=path.parent/'source-inputs.json'
        assert len(read(source))==1024 and sha(source)==normal['source_manifest_sha256']
        assert sha(ROOT/'啟動現代客戶端.ps1')==normal['entry_sha256']
        cohort=read(path.parent/'cohort.json')
        for profile,info in normal['profiles'].items():
            proof=Path(info['qualification']);assert sha(proof)==info['qualification_sha256']
            q=read(proof)
            assert q['status']=='independently_qualified_presentation_source_owner_related_profile'
            assert q['source_manifest_sha256']==normal['source_manifest_sha256'] and q['counts']['failed']==0
            assert q['actual_test_executables']==10 and q['normal_sha256']==info['normal_sha256']
            assert sha(cohort['profiles'][profile]['normal']['path'])==info['normal_sha256']
    return normal

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
ap.add_argument('--terminal-exit-code', type=int, required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-resource-optin') and a.terminal_exit_code == 0 and a.session_id>0
c = read(run / 'cohort.json')
assert c['status'] == 'executed_current_both_profiles_five_original_resource_bodies' and c['owned_process'] is None
assert len(c['process_history']) == 10 and all(p['exit_code'] == 0 for p in c['process_history'])
assert len(set(c['tests'])) == 5 and set(c['profiles']) == {'Debug', 'Release'}
pins = read(run / 'source-inputs.json')
startup=c.get('library_kind')=='startup-current'
assert len(pins) == c['source_files'] == (1024 if startup else 1021) and sha(run / 'source-inputs.json') == c['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
normal_reference=c['normal_qualification'] if startup else c['normal_invariance_proof']
normal=read(normal_reference)
if startup:
    normal=verify_normal_reference(normal_reference,read(run/'compiled-inputs.json'))
    assert normal['source_manifest_sha256']==c['source_manifest_sha256']
    assert sha(normal_reference)==c['normal_qualification_sha256'] and c['normal_invariance_proof'] is None
    normal_profiles=c['normal_profiles']
    for profile,info in normal_profiles.items():assert info['sha256']==normal['profiles'][profile]['normal_sha256']
    compiled=read(run/'compiled-inputs.json');assert len(compiled)==1025
    for name,value in compiled.items():assert sha(ROOT/name)==value,name
else:normal_profiles=normal['profiles']
for info in normal_profiles.values():
    assert sha(info['path']) == info['sha256']
library = Path(c['library_qualification']).parent
assert read(library / 'source-inputs.json') == pins
assert read(c['library_qualification'])['source_manifest_sha256'] == c['source_manifest_sha256']
if startup:
    lq=read(c['library_qualification'])
    assert lq['status']=='independently_qualified_current_full_library_both_profiles_and_terminal_original_runner'
    assert lq['observed_terminal_exit_code']==0 and sha(c['library_qualification'])==c['library_qualification_sha256']
    assert read(library/'compiled-inputs.json')==compiled
inputs = read(run / 'original-input-pins.json')
assert sha(run / 'original-input-pins.json') == c['original_IDX_Text_WAV_pins_sha256']
for name, digest in inputs.items():
    assert sha(name) == digest, name
metadata = read(run / 'original-PAK-metadata.json')
assert sha(run / 'original-PAK-metadata.json') == c['original_PAK_metadata_sha256']
for name, item in metadata.items():
    p = Path(name)
    assert (p.stat().st_size, p.stat().st_mtime_ns) == (item['bytes'], item['mtime_ns']), name
historical = read(run / 'historical-report-pins.json')
assert len(historical) == c['historical_reports']
for name, digest in historical.items():
    assert sha(PROJECT / 'logs/continuation-20261003' / name) == digest, name
profiles = {}
for profile, info in c['profiles'].items():
    inventory = read(library / profile / 'inventory-audit/inventory.json')
    assert set(c['tests']) <= set(inventory['ignored_test_names']) and inventory['total_tests'] == 2408 and inventory['ignored_tests'] == 473
    exe = Path(info['immutable_test_exe']).resolve(strict=True)
    assert exe.is_relative_to(PROJECT / 'target/current/native-resource-optin')
    assert sha(exe) == info['actual_test_sha256'] == inventory['product_sha256']
    with exe.open('rb') as f:
        h = f.read(64)
        assert h[:2] == b'MZ'
        f.seek(int.from_bytes(h[60:64], 'little'))
        h = f.read(26)
        assert h[:4] == b'PE\0\0' and h[4:6] == b'\x64\x86' and h[24:26] == b'\x0b\x02'
    assert sha(info['original_product_archive']) == info['original_product_archive_sha256']
    with zipfile.ZipFile(info['original_product_archive']) as z:
        with z.open('lito_modern.exe') as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == info['actual_test_sha256']
    assert [x['name'] for x in info['results']] == c['tests']
    for index, result in enumerate(info['results']):
        d = run / profile / str(index)
        assert sha(d / 'stdout.log') == result['stdout_sha256'] and sha(d / 'stderr.log') == result['stderr_sha256']
        text = (d / 'stdout.log').read_text(encoding='utf-8-sig')
        assert re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$', text, re.M) == [(result['name'], 'ok')]
        assert re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', text) == [('ok', '1', '0', '0', '0', '2407')]
        jobs = [p for p in c['process_history'] if Path(p['log_directory']) == d]
        assert len(jobs) == 1 and jobs[0]['command'] == [str(exe), result['name'], '--exact', '--ignored', '--nocapture', '--test-threads=1']
    profiles[profile] = dict(passed=5, failed=0, actual_test_sha256=info['actual_test_sha256'],
        actual_test_archive_sha256=info['original_product_archive_sha256'])
ids = ','.join(str(p['pid']) for p in c['process_history'])
command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);$ids=@(" + ids + ");@(Get-CimInstance Win32_Process|Where-Object {$ids -contains $_.ProcessId}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
assert p.returncode == 0 and not p.stderr
live = json.loads(p.stdout.decode('utf-8-sig')) if p.stdout.strip() else []
if isinstance(live, dict):
    live = [live]
for process in live:
    for original in c['process_history']:
        if process['ProcessId'] == original['pid']:
            assert abs((datetime.fromisoformat(process['created']) - datetime.fromisoformat(original['creation_utc'])).total_seconds()) > 0.001
out = run / 'qualified'
out.mkdir()
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
(out / 'terminal-CIM.json').write_bytes(p.stdout)
files = {str(p.relative_to(run)).replace('\\', '/'): p for p in run.rglob('*') if p.is_file() and p.suffix.lower() != '.zip'}
if startup:
    files['library-reference/completion-observation.json']=Path(c['library_qualification'])
    files['library-reference/normal-related-completion.json']=Path(normal_reference)
else:
    for name in ['default-library-qualified.json', 'normal-invariance.json']:
        files['library-reference/' + name] = library / name
hashes = {name: sha(p) for name, p in files.items()}
archive = out / 'raw.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
q = dict(status='independently_qualified_current_both_profiles_five_original_resource_bodies',
    resource_scope=c.get('resource_scope', 'base'),
    test_names=c['tests'],
    source_files=len(pins), source_manifest_sha256=c['source_manifest_sha256'], profiles=profiles,
    library_kind=c.get('library_kind','legacy-corrected'),compiled_input_files=1025 if startup else None,
    unique_previously_ignored_bodies=5, actual_test_invocations=10, actual_test_executables=2,
    unchanged_default_library_ignored_count=473, other_library_ignored_bodies_not_qualified_by_this_scope=468,
    original_IDX_Text_WAV_SHA_unchanged=True, full_Sprite_PAKs_metadata_only=True,
    selected_SPR_entry_byte_pixel_references_checked_by_tests=True, historical_reports_unchanged=len(historical),
    original_session_id=a.session_id, observed_terminal_exit_code=0, original_owned_processes_terminal=True,
    raw_archive_sha256=sha(archive), raw_files=hashes, actual_formal_sessions=0,
    GUI_verified=False, audio_device_verified=False, all_resources_or_ignored_bodies_verified=False,
    whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
if c.get('resource_scope') == 'scene':
    q['sixth_scene_test_qualified'] = False
    q['unexecuted_sixth_scene_precondition'] = c['unexecuted_sixth_scene_precondition']
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'unique_previously_ignored_bodies', 'actual_test_invocations', 'actual_formal_sessions']}, ensure_ascii=False))
