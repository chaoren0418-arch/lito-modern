"""Checkpoint a source-bound G06 run without promoting partial case results."""
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
ap.add_argument('run', type=Path)
ap.add_argument('--session-id', type=int, required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT/'驗證紀錄/current/native-G06-controller')
c = read(run/'cohort.json')
binding = read(run/'original-session-binding.json')
assert binding['original_session_id'] == a.session_id
assert binding['runner_identity'] == read(run/'runner-identity.json')
assert sha(run/'source-inputs.json') == c['source_manifest_sha256']
pins = read(run/'source-inputs.json')
assert len(pins) == 1024 and all(sha(ROOT/n) == h for n, h in pins.items())
extra = c['additional_resource_input']
assert sha(ROOT/extra['path']) == extra['sha256']
for name, digest in c['validation_pins'].items():
    assert sha(ROOT/'維護工具'/name) == digest
assert sha(c['startup_qualification']) == c['startup_qualification_sha256']
profiles = {}
for profile, info in c['profiles'].items():
    cases = {}
    for scenario in info['cases']:
        p = run/profile/('qualified-'+scenario)/'qualified.json'
        if p.exists():
            q = read(p)
            assert q['source_manifest_sha256'] == c['source_manifest_sha256']
            assert q['status'] == 'independently_readback_qualified_current_G06_TW_extended_controller_case'
            cases[scenario] = dict(qualification=str(p), qualification_sha256=sha(p), sessions=q['actual_formal_sessions'])
    profiles[profile] = dict(qualified_cases=cases, qualified_sessions=sum(v['sessions'] for v in cases.values()))
done = (run/'completion-observation.json').exists()
if done:
    completion = read(run/'completion-observation.json')
    assert completion['original_session_id'] == a.session_id and completion['observed_terminal_exit_code'] == 0
    assert completion['recorded_original_owned_processes_all_terminal'] and completion['actual_qualified_formal_sessions'] == 96
    assert sum(v['qualified_sessions'] for v in profiles.values()) == 96
active = c['status'] not in ['failed_preserved', 'executed_current_both_profiles_G06_controller_TW_extended_six_cases']
record = dict(run=str(run), source_manifest_sha256=c['source_manifest_sha256'], status=c['status'],
    original_session_id=a.session_id, runner_identity=binding['runner_identity'],
    active_profile=c.get('active_profile'), active_scenario=c.get('active_scenario'), profiles=profiles,
    qualified=done, source_and_corresponding_QA_frozen=active, GUI_verified=False, physical_input_verified=False,
    same_endpoint_verified=False, whole_G06_complete=False, whole_goal_complete=False)
if done:
    record.update(completion_observation=str(run/'completion-observation.json'), completion_sha256=sha(run/'completion-observation.json'),
        qualified_sessions=96, recorded_owned_identities=completion['recorded_original_owned_identities'])
files = ['工作佇列.json', '監製進度.json', '驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json']
if done:
    files += ['README.md', 'docs/HISTORY.md']
out = run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n in files:
        z.write(ROOT/n, n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n,h in before.items())
note = ('來源'+c['source_manifest_sha256'][:8]+' G06六分支兩版96sessions已獨立核對，下一步同來源廣域111目標／完整library／map563；GUI與全G06仍未完成。'
    if done else '來源'+c['source_manifest_sha256'][:8]+' G06原'+str(a.session_id)+'之'+str(c.get('active_profile'))+'/'+str(c.get('active_scenario'))+'執行中；部分案例不等於整批通過，保留原程序等待實際終止。')
if c['status'] == 'failed_preserved':
    note = '來源'+c['source_manifest_sha256'][:8]+' G06失敗；先封存原來源／映像／日誌，再定位，未算整批通過。'
for n in files[:4]:
    v = read(ROOT/n)
    assert v['current_product_identity']['source_manifest_sha256'] == c['source_manifest_sha256']
    v.update(current_G06_presentation_source_owner=record,
        active_workload_sessions={'current_G06_controller':a.session_id} if active else {},
        source_freeze_for_current_G06_controller=active,
        source_freeze_while_Cargo=active and c['status'] == 'compiling_current_G06_controller_targets',
        next_task=note+' 登入器／伺服器來源只讀且產物獨立；8秒／60秒／G11預算維持原值。',
        progress_updated_utc=datetime.now(timezone.utc).isoformat(), goal_complete=False, whole_goal_complete=False)
    save(ROOT/n, v)
if done:
    document_note = ('2026-10-17：來源'+c['source_manifest_sha256'][:8]+'之G06原'+str(a.session_id)
        +'已actual exit0；兩版六分支12案／96 LOGIN-GAME sessions經獨立封包、角色／物品全欄接續及來源／映像核對，'
        +str(completion['recorded_original_owned_identities'])+'組記錄程序身分皆已終止。只通過此控制器範圍，'
        'GUI／實體輸入／同端點及完整G06未完成。相關正常兩主與Debug58／Release54過維持各自資格；'
        '下一步同來源廣域111目標、完整library、map563與正常雙玩家可見性回歸。原12C3兩批G06失敗及00C0發布競態失敗封存不變。'
        '8秒／60秒／G11預算不變，登入器與伺服器來源只讀且產物獨立，完整G01–G12未完成。')
    for n in files[4:]:
        p = ROOT/n
        head, _, body = p.read_text(encoding='utf-8-sig').partition('\n')
        p.write_text(head+'\n\n'+document_note+'\n'+body, encoding='utf-8', newline='\n')
save(out/'record.json', dict(before=before, before_archive_sha256=sha(out/'before.zip'),
    after={n:sha(ROOT/n) for n in files}, scope=record))
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tracker=str(out), status=c['status'], qualified=done,
    qualified_sessions={p:v['qualified_sessions'] for p,v in profiles.items()}), ensure_ascii=False))
