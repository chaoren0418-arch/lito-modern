"""Record the current source scope separately from retained 4C7 qualifications."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile

ROOT = Path(__file__).resolve().parents[1]
RUN = ROOT/'驗證紀錄/current/native-retained-geometry/validation-20261017-004225-579828'
def read(p): return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(p, value): p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

c = read(RUN/'cohort.json')
pins = read(RUN/'compiled-inputs.json')
assert len(pins) == 1025 and all(sha(ROOT/n) == h for n, h in pins.items())
assert c['source_manifest_sha256'] == '6D1891432843C753448F7CC19E3DD2849E163E516AF108CFE893BBD793B6C959'
qpath = RUN/'Debug/profile-qualified/qualified.json'
q = read(qpath)
assert q['counts'] == dict(passed=52, failed=0, ignored=3)
assert q['source_manifest_sha256'] == c['source_manifest_sha256']
completion = RUN/'completion-observation.json'
done = completion.exists()
if done:
    final = read(completion)
    assert final['original_session_id'] == 4754 and final['observed_terminal_exit_code'] == 0
    assert final['recorded_owned_processes_all_terminal']
else:
    assert c['status'] in ['compiling_retained_geometry_related_targets', 'running_retained_geometry_related_targets', 'building_unique_entry_normal']
    assert c['active_profile'] == 'Release'

files = ['工作佇列.json', '監製進度.json', '驗證紀錄/current/native-teleport-wire/progress.json',
         '驗證紀錄/current/native-terrain-residence/continuation.json', 'README.md', 'docs/HISTORY.md']
out = RUN/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip', 'x', zipfile.ZIP_DEFLATED) as z:
    for n in files: z.write(ROOT/n, n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert set(z.namelist()) == set(files)
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper() == h for n, h in before.items())

correction = dict(status='both_profiles_related_qualified' if done else 'Debug_related_qualified_Release_original_pipeline_active',
    run=str(RUN), original_session_id=4754, source_manifest_sha256=c['source_manifest_sha256'],
    source_files=1024, compiled_input_files=1025, changed_sources=c['changed_sources'],
    Debug_qualification=str(qpath), Debug_qualification_sha256=sha(qpath), Debug_counts=q['counts'],
    Debug_normal_sha256=q['normal_sha256'], source_and_runner_frozen=not done,
    actual_formal_sessions=0, GUI_verified=False, performance_accepted=False,
    full_current_regression_verified=False, whole_goal_complete=False)
if done:
    correction.update(completion_observation=str(completion), completion_observation_sha256=sha(completion),
        observed_terminal_exit_code=0, profiles=final['profiles'])
note = ('重建版與登入器獨立，登入器產物只讀。新來源6D189143已保留同代S32與導航，跨地形窗口重用；'
        '初次完整hash、來源讀鎖、sector集合變更偵測、像素與取消/64MiB契約保留。'
        '原紅測試37814為Rust101/wrapper1已封存；Debug十實際EXEs52過0敗3忽略已獨立讀回。')
note += ('Release十實際EXEs48過0敗3忽略及唯一入口兩主已獨立核對，原4754actualexit0。'
         if done else 'Release沿原4754，產品來源與該runner固定；未記為通過。')
note += ('4C7之廣域/完整library/C02三十人/G06/原資源資格各保留舊版本，不移記新來源。'
         '舊warm p95463583微秒超原250ms預算，修正後尚待安靜同條件量測；G11預算不變，完整G01–G12未完成。')
for n in files[:4]:
    value = read(ROOT/n)
    for key in ['current_default110', 'current_retained_archive_full_library', 'current_map563_retained_archive_revalidation', 'current_shared_G06_diagnostic', 'current_original_resource_optins']:
        if isinstance(value.get(key), dict):
            value[key]['applies_to_current_geometry_source'] = False
    value.update(current_retained_geometry_correction=correction,
        source_freeze_for_current_retained_geometry_validation=not done,
        source_freeze_while_Cargo=not done, active_workload_sessions={} if done else {'retained_geometry_related':4754},
        current_product_version_scope='source1024_6D189143_related_both_qualified_other_current_scopes_pending' if done else 'source1024_6D189143_Debug_related_qualified_Release_pending_old4C7_other_scopes_retained',
        current_working_tree_verified=False, working_tree_modified_since_current_normal_qualification=not done,
        next_task=note, last_goal_turn_classification='progress', last_goal_turn_evidence=[str(qpath),str(RUN/'cohort.json')],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(), goal_complete=False, whole_goal_complete=False)
    for item in value.get('items', []):
        if item['id'] == 'G03-current-regression':
            item.update(state='source6D_retained_geometry_related_both_qualified_full_regression_pending' if done else 'source6D_Debug_related_qualified_Release_pending_4C7_broad_retained', complete=False)
        if item['id'] == 'G06-room-service-branches':
            item.update(state='source4C7_controller_six_and_siege_qualified_source6D_runtime_revalidation_pending', complete=False)
    save(ROOT/n, value)
for n in files[4:]:
    path=ROOT/n
    header, _, body=path.read_text(encoding='utf-8-sig').partition('\n')
    path.write_text(header+'\n\n2026-10-17：'+note+'\n'+body, encoding='utf-8', newline='\n')
save(out/'record.json', dict(before=before, before_archive_sha256=sha(out/'before.zip'),
    after={n:sha(ROOT/n) for n in files}, correction=correction, product_source_changed=False, whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out), related_both_profiles_qualified=done, whole_goal_complete=False), ensure_ascii=False))
