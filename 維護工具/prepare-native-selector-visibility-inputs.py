"""Bind existing qualified normal builds to new visibility runtime cases.

This binder launches no compiler, test, game, or server. Its four referenced
test targets were executed by the source-matched broad regression. The normal
build and its process exit evidence remain those of the original related run.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('default_cohort', type=Path)
a = ap.parse_args()
base = a.default_cohort.resolve(strict=True)
assert base.is_relative_to(ROOT/'驗證紀錄/current/native-default111')
b = read(base/'cohort.json')
terminal = read(base/'completion-observation.json')
assert b['status'] == 'independently_qualified_current_default111_both_profiles'
assert b['normal_input_kind'] == 'selector-related'
assert terminal['observed_terminal_exit_code'] == 0 and terminal['recorded_original_owned_processes_all_terminal']
assert b['owned_process'] is None and all(p['exit_code'] == 0 for p in b['process_history'])
normal_q = Path(b['normal_qualification']).resolve(strict=True)
assert normal_q.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
n = read(normal_q)
normal_root = normal_q.parent
c = read(normal_root/'cohort.json')
assert sha(normal_q) == b['normal_qualification_sha256']
assert n['status'] == 'independently_qualified_current_normal_selector_both_profiles'
assert n['observed_terminal_exit_code'] == 0 and n['recorded_owned_processes_all_terminal']
assert n['recorded_owned_identities'] == 64
assert b['source_manifest_sha256'] == n['source_manifest_sha256'] == terminal['source_manifest_sha256']
pins = read(normal_root/'source-inputs.json')
compiled = read(normal_root/'compiled-inputs.json')
assert len(pins) == 1028 and len(compiled) == 1029
assert sha(normal_root/'source-inputs.json') == n['source_manifest_sha256']
assert all(sha(ROOT/name) == digest for name, digest in compiled.items())
assert sha(ROOT/'啟動現代客戶端.ps1') == n['entry_sha256']
reference_targets = ['native_skills', 'native_skill_selection', 'native_actor_states', 'main_unit']
profiles = {}
for profile in ['Debug', 'Release']:
    broad_path = Path(b['profiles'][profile]['qualification']).resolve(strict=True)
    broad = read(broad_path)
    assert sha(broad_path) == b['profiles'][profile]['qualification_sha256'] == terminal['profiles'][profile]['qualification_sha256']
    assert broad['status'] == 'independently_qualified_current_default111_integrations_and_main'
    assert broad['actual_test_executables'] == 112 and broad['counts']['failed'] == 0
    assert broad['source_manifest_sha256'] == n['source_manifest_sha256']
    normal = c['profiles'][profile]['normal']
    assert sha(normal['path']) == normal['sha256'] == n['profiles'][profile]['normal_sha256'] == broad['normal_mains'][profile]
    assert normal['unique_entry_exit_code'] == 0
    references = {name:broad['per_target'][name] for name in reference_targets}
    assert all(info['failed'] == 0 for info in references.values())
    profiles[profile] = dict(normal=normal, broad_qualification=str(broad_path),
        broad_qualification_sha256=sha(broad_path), referenced_broad_test_targets=references)

out = ROOT/'驗證紀錄/current/native-detection-invisibility'/('current-normal-inputs-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
(out/'binder.py').write_bytes(Path(__file__).read_bytes())
for profile, info in profiles.items():
    folder = out/profile
    folder.mkdir()
    for name in ['source-inputs.json', 'compiled-inputs.json']:
        (folder/name).write_bytes((normal_root/name).read_bytes())
    cohort = dict(status='bound_existing_qualified_current_normal_for_visibility_runtime',
        profile=profile, source_manifest_sha256=n['source_manifest_sha256'],
        normal=info['normal'], input_kind='existing-current-default111-and-presentation-normal',
        normal_qualification=str(normal_q), normal_qualification_sha256=sha(normal_q),
        default_cohort=str(base), default_completion_sha256=sha(base/'completion-observation.json'),
        **{k:v for k,v in info.items() if k != 'normal'},
        actual_processes_launched_here=0, actual_test_executables_run_here=0, actual_casts=0,
        GUI_verified=False, whole_goal_complete=False)
    save(folder/'cohort.json', cohort)
    qualified = folder/'qualified'
    qualified.mkdir()
    files = {name:sha(folder/name) for name in ['source-inputs.json', 'compiled-inputs.json', 'cohort.json']}
    with zipfile.ZipFile(qualified/'binding-raw.zip', 'x', zipfile.ZIP_DEFLATED) as z:
        for name in files:
            z.write(folder/name, name)
        z.write(out/'binder.py', 'binder.py')
    files['binder.py'] = sha(out/'binder.py')
    with zipfile.ZipFile(qualified/'binding-raw.zip') as z:
        assert set(z.namelist()) == set(files)
        for name, digest in files.items():
            with z.open(name) as f:
                assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
    save(qualified/'qualified.json', dict(status='independently_bound_existing_normal_for_visibility_runtime',
        profile=profile, source_manifest_sha256=n['source_manifest_sha256'], normal_sha256=info['normal']['sha256'],
        observed_terminal_exit_code=0, terminal_scope='existing upstream broad and related runs; this binder launches no product processes',
        actual_casts=0, actual_test_executables_run_here=0, actual_processes_launched_here=0,
        normal_qualification=str(normal_q), normal_qualification_sha256=sha(normal_q),
        broad_qualification=info['broad_qualification'], broad_qualification_sha256=info['broad_qualification_sha256'],
        referenced_broad_test_targets=info['referenced_broad_test_targets'],
        raw_archive_sha256=sha(qualified/'binding-raw.zip'), raw_files=files,
        GUI_verified=False, whole_goal_complete=False))
save(out/'inputs.json', dict(source_manifest_sha256=n['source_manifest_sha256'],
    normal_qualification=str(normal_q), normal_qualification_sha256=sha(normal_q), default_cohort=str(base),
    default_completion_sha256=sha(base/'completion-observation.json'),
    profiles={profile:dict(input=str(out/profile), qualification_sha256=sha(out/profile/'qualified/qualified.json')) for profile in profiles},
    actual_processes_launched_here=0, actual_casts=0, whole_goal_complete=False))
print(json.dumps(dict(output=str(out), profiles={profile:str(out/profile) for profile in profiles},
    actual_processes_launched_here=0, actual_casts=0), ensure_ascii=False))
