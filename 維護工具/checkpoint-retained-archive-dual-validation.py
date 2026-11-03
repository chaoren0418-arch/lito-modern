"""Preserve trackers and publish only the completed related-validation scope."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile
ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v): p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run = ROOT/'驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254'
qpath = run/'completion-observation.json'
q = read(qpath)
assert q['status'] == 'independently_qualified_retained_archive_both_profiles'
siege = ROOT/'驗證紀錄/current/native-startup-siege563/20261016-221755-210184'
sc = read(siege/'cohort.json')
assert sc['input_kind'] == 'retained-archive-related'
files = ['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out = run/('tracker-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
before = {n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files: z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:
    for name,digest in before.items(): assert hashlib.sha256(z.read(name)).hexdigest().upper() == digest
note = '新來源4C7DE771的四來源持有修正已雙版七實際EXEs限定核對：Debug38／Release34過，各0敗2忽略；唯一入口正常70F2FFEF／34397D06及source/test/normal/raw封存一致，20組記錄子程序actualexit0且退出。原91989已輸出完成且handle關閉，但工具未提供wrapper數值exit code，明記null而不推定0。靜態Release跨window總重載約0.40秒，archive_pins的2微秒僅metadata選取，不當成整體驗證時間。新版C02/TW+extended三十人map563雙版沿13803，來源及對應QA固定；A9 Debug通過與Release13.30秒失敗均原樣保留，不移記新版。新版完整library/default/必要opt-in、GUI/實體輸入/IME/多DPI、clean host及G11/G12仍待，G01–G12未完成。'
fix = dict(status=q['status'], evidence=str(qpath), sha256=sha(qpath), source_manifest_sha256=q['source_manifest_sha256'],
    source_files=1024, compiled_input_files=1025, profiles=q['profiles'], original_session_id=91989,
    observed_terminal_exit_code=None, wrapper_exit_code_unavailable=True, recorded_owned_identities=20,
    recorded_owned_processes_all_terminal=True, independently_qualified=True, actual_formal_sessions=0,
    new_G06_thirty_player_runtime_verified=False, whole_goal_complete=False)
current = dict(status='running_new_source_map563_both_profiles',run=str(siege),original_session_id=13803,
    input_kind='retained-archive-related',source_manifest_sha256=q['source_manifest_sha256'],
    active_profile=sc.get('active_profile'),runtime_qualified=False,whole_goal_complete=False)
for name in files[:4]:
    p=ROOT/name;v=read(p)
    if 'current_product_identity' in v and not v.get('retained_A9_product_identity_before_archive_retention'):
        v['retained_A9_product_identity_before_archive_retention']=v['current_product_identity']
    v.update(current_terrain_archive_retention_correction=fix,current_map563_retained_archive_revalidation=current,
        source_freeze_for_current_terrain_archive_retention_validation=False,source_freeze_for_live_retained_archive_siege=True,
        current_product_version_scope='source1024_4C7DE771_related7_dual_qualified; current_full_regression_and_live_siege_pending; A9_scopes_retained_only',
        current_working_tree_verified=False,next_task=note,last_goal_turn_classification='progress',
        last_goal_turn_evidence=[str(qpath),str(siege/'cohort.json')],progress_updated_utc=datetime.now(timezone.utc).isoformat(),
        active_workload_sessions={'retained_archive_map563_both':13803},goal_complete=False)
    v['current_product_identity']=dict(status='retained_archive_related_scope_and_unique_entry_normals_qualified',
        source_files=1024,source_manifest_sha256=q['source_manifest_sha256'],
        Debug_sha256=q['profiles']['Debug']['normal_sha256'],Release_sha256=q['profiles']['Release']['normal_sha256'],
        qualification=str(qpath),GUI_verified=False,whole_goal_complete=False)
    if 'current_focus' in v: v['current_focus']=dict(item='G06-map563-retained-archive-new-live-validation',work=note,goal_complete=False)
    save(p,v)
for name in files[4:]:
    p=ROOT/name; text=p.read_text(encoding='utf-8-sig');header,sep,body=text.partition('\n')
    p.write_text(header+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in files},
    completion_qualification=str(qpath),completion_qualification_sha256=sha(qpath),whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),status='trackers_updated_with_scoped_dual_qualification_and_live_siege'),ensure_ascii=False))
