"""Publish actual dual completion and retain the over-budget quiet diagnostic."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
pipeline=ROOT/'驗證紀錄/current/native-default110/20261016-231456-842850'
completion=pipeline/'completion-observation.json';c=read(completion);cohort=read(pipeline/'cohort.json')
warm=ROOT/'驗證紀錄/current/native-performance/warm-residence-20261017-003129-184958/independent-readback.json';w=read(warm)
assert c['original_session_id']==24147 and c['observed_terminal_exit_code']==0 and c['recorded_original_owned_processes_all_terminal']
assert c['recorded_original_owned_identities']==457 and c['profiles']['Debug']['counts']==dict(passed=487,failed=0,ignored=21)
assert c['profiles']['Release']['counts']==dict(passed=483,failed=0,ignored=21)
assert cohort['owned_process'] is None and not cohort['source_and_corresponding_QA_frozen']
assert w['source_manifest_sha256']==c['source_manifest_sha256'] and w['measured_reloads']==40
assert w['original_session_id']==73457 and not w['synthetic_warm_p95_budget_met'] and not w['budget_changed']
files=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
    '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out=pipeline/('tracker-completion-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={name:sha(ROOT/name) for name in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for name in files:z.write(ROOT/name,name)
with zipfile.ZipFile(out/'before.zip') as z:
    assert set(z.namelist())==set(files)
    for name,value in before.items():assert hashlib.sha256(z.read(name)).hexdigest().upper()==value
broad=dict(cohort)
broad.update(completion_observation=str(completion),completion_observation_sha256=sha(completion),original_session_id=24147,
    observed_terminal_exit_code=0,terminal_tool_chunk_id='de449d',recorded_owned_identities=457,
    recorded_owned_processes_terminal=True,source_and_corresponding_QA_frozen=False,GUI_verified=False,whole_goal_complete=False)
performance=dict(status=w['status'],qualification=str(warm),sha256=sha(warm),source_manifest_sha256=w['source_manifest_sha256'],
    original_session_id=73457,observed_terminal_exit_code=0,quiet_distribution_measured=True,
    synthetic_authority_fixture=True,actors=1,actual_formal_sessions=0,warmup_pairs=5,measured_reloads=40,wall_us=w['wall_us'],
    cold_Model_load_plus_first_draw_us=w['cold_Model_load_plus_first_draw_us'],budget_changed=False,
    original_warm_p95_budget_ms=250,synthetic_warm_p95_budget_met=False,synthetic_warm_single_budget_met=True,
    host_other_load_not_excluded=True,GUI_verified=False,performance_accepted=False,whole_goal_complete=False,
    next='Inspect and remove repeated S32 geometry parsing while retaining verified source identity and write/delete locks, original pixels, cancellation rollback and 64MiB bounds. Measure the same quiet diagnostic after related validation; do not relax the budget.')
note=('當版4C7DE771之110整合+main111實際x64映像兩版完成：Debug487/Release483過、各0敗21忽略，'
    '原24147actualexit0/de449d，雙版source/test/normal/raw及457組記錄程序身分退出已獨立核對；本輪來源/四QA固定已解除。'
    '當版完整library各1935/0/473、10原資源opt-in各10過、G06六分支96sessions及map563120sessions維持各自限定資格。'
    '無自有編譯/執行檔驗證負載之單角色合成Model重載：73457actualexit0，5暖機對/40量測，median416221/p95463583/max480341微秒，'
    '原250ms warm p95未達，預算不變；cold Model+首draw2252896微秒亦非30人GUI接受證據。'
    '測得geometry約117ms、static-assets約238ms中位數，來源確認Model與adapter各重解析S32；先保留來源鎖/像素/取消/64MiB契約改善重解析，再同條件量測。'
    '偵測術候選仍未套產品，74已學/21請求/53缺口維持；GUI/實體輸入/IME/多DPI、clean host、60分/20週期與完整G01–G12未完成。')
for name in files[:4]:
    path=ROOT/name;value=read(path)
    value.update(current_default110=broad,current_G11_warm_reload_followup=performance,
        source_freeze_for_live_default110_job=False,active_workload_sessions={},completed_current_default110_session=24147,
        current_working_tree_verified=False,current_product_version_scope='source1024_4C7DE771_broad_library_related_G06_map563_ten_optins_qualified; GUI_full_gameplay_G11_pending',
        next_task=note,last_goal_turn_classification='progress',last_goal_turn_evidence=[str(completion),str(warm)],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if 'items' in value:
        g=next(item for item in value['items'] if item['id']=='G03-current-regression')
        g.update(state='source4C7_current_broad_both_profiles_and_full_default_library_qualified_defined_scopes',
            current_broad_completion=str(completion),current_broad_completion_sha256=sha(completion),complete=False)
    save(path,value)
for name in files[4:]:
    path=ROOT/name;header,_,body=path.read_text(encoding='utf-8-sig').partition('\n')
    path.write_text(header+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={name:sha(ROOT/name) for name in files},
    completion=str(completion),completion_sha256=sha(completion),warm_qualification=str(warm),warm_qualification_sha256=sha(warm),
    product_source_changed=False,performance_accepted=False,whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out),Debug_passed=487,Release_passed=483,warm_p95_budget_met=False,whole_goal_complete=False),ensure_ascii=False))
