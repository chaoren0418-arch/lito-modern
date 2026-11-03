"""Checkpoint the original broad pipeline, preserving per-profile scope."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', required=True, type=int)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-default110')
c = read(run/'cohort.json')
binding = read(run/'original-session-binding.json')
assert binding['original_session_id'] == a.session_id and binding['pipeline_pid'] == c['pipeline_pid']
normal_path = Path(c['normal_qualification']).resolve(strict=True)
assert normal_path.is_relative_to(ROOT/'驗證紀錄/current/native-presentation-source-owner')
n = read(normal_path)
assert sha(normal_path) == c['normal_qualification_sha256']
assert n['source_manifest_sha256'] == c['source_manifest_sha256']
inputs = read(normal_path.parent/'compiled-inputs.json')
assert len(inputs) == 1025 and all(sha(ROOT/name) == h for name,h in inputs.items())
active = c['status'] not in ['failed_preserved', 'independently_qualified_current_default110_both_profiles']
if active:
    command = "[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(c['pipeline_pid'])+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}},CommandLine|ConvertTo-Json -Compress"
    p = subprocess.run(['powershell.exe', '-NoProfile', '-Command', command], capture_output=True, creationflags=subprocess.CREATE_NO_WINDOW)
    assert p.returncode == 0 and not p.stderr
    live = json.loads(p.stdout.decode('utf-8-sig'))
    assert live['ProcessId'] == c['pipeline_pid'] and 'run-native-current-default110-both.py' in live['CommandLine']
    assert abs((datetime.fromisoformat(live['created'])-datetime.fromisoformat(c['pipeline_creation_utc'])).total_seconds()) < .001
profiles = {}
for profile, info in c['profiles'].items():
    if info['qualified']:
        q = read(info['qualification'])
        assert sha(info['qualification']) == info['qualification_sha256']
        assert q['source_manifest_sha256'] == c['source_manifest_sha256'] and q['actual_test_executables'] == 111 and q['counts']['failed'] == 0
    profiles[profile] = dict(info)
current = None
if active and c.get('active_profile'):
    matches = []
    for folder in (ROOT/'驗證紀錄/current/default-native-regression').glob('*-'+c['active_profile']):
        if not (folder/'receipt.json').exists():
            continue
        receipt = read(folder/'receipt.json')
        if receipt.get('normal_qualification') == c['normal_qualification'] and receipt.get('G06_qualification') == c['G06_qualification']:
            matches.append((folder,receipt))
    assert len(matches) == 1
    folder, receipt = matches[0]
    current = dict(run=str(folder), status=receipt['status'], completed_suites=receipt.get('completed_suites'), active_suite=receipt.get('active_suite'))
done = (run/'completion-observation.json').exists()
if done:
    terminal = read(run/'completion-observation.json')
    assert terminal['original_session_id'] == a.session_id and terminal['observed_terminal_exit_code'] == 0
    assert terminal['recorded_original_owned_processes_all_terminal']
record = dict(run=str(run), original_session_id=a.session_id, source_manifest_sha256=c['source_manifest_sha256'],
    status=c['status'], pipeline_pid=c['pipeline_pid'], pipeline_creation_utc=c['pipeline_creation_utc'],
    active_profile=c.get('active_profile'), current_profile_progress=current, profiles=profiles,
    normal_qualification=c['normal_qualification'], G06_qualification=c['G06_qualification'], qualified=done,
    source_and_corresponding_QA_frozen=active, library_unit_scope_inferred=False, ignored_bodies_executed=False,
    GUI_verified=False, whole_goal_complete=False)
if done:
    record.update(completion_observation=str(run/'completion-observation.json'), completion_sha256=sha(run/'completion-observation.json'))
app = Path(read(normal_path.parent/'cohort.json')['application'])
files = ['工作佇列.json', '監製進度.json', '驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json', str((app/'next-validation-action.json').relative_to(ROOT)).replace('\\','/')]
out = run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {name:sha(ROOT/name) for name in files}
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for name in files:
        z.write(ROOT/name, name)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(name)).hexdigest().upper() == h for name,h in before.items())
note = '來源'+c['source_manifest_sha256'][:8]+'廣域原'+str(a.session_id)+'目前'+c['status']+'；已核對版別'+','.join(p for p,v in profiles.items() if v['qualified'])+'。完整library／map563／必要原資源／正常雙玩家可見性與GUI／整體G01–G12仍待。'
for name in files[:4]:
    value = read(ROOT/name)
    assert value['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
    value.update(current_presentation_source_owner_default110=record,
        active_workload_sessions={'current_default110_both':a.session_id} if active else {},
        source_freeze_for_current_default110_both=active,
        source_freeze_while_Cargo=bool(active and current and current['status'] == 'compiling_default_targets'),
        next_task=note, progress_updated_utc=datetime.now(timezone.utc).isoformat(), goal_complete=False, whole_goal_complete=False)
    save(ROOT/name, value)
next_action = read(app/'next-validation-action.json')
next_action.update(current_broad_profile_qualifications=profiles, current_broad_profile_progress=current)
save(app/'next-validation-action.json', next_action)
save(out/'record.json', dict(before=before, before_archive_sha256=sha(out/'before.zip'), after={name:sha(ROOT/name) for name in files}, scope=record))
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tracker=str(out), status=c['status'], profiles={p:v.get('counts') for p,v in profiles.items()}, current=current, both_profiles_qualified_and_terminal=done), ensure_ascii=False))
