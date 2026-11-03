"""Register qualified Debug only while the original Release build remains live."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,subprocess,zipfile,time
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';c=read(run/'cohort.json')
qp=run/'Debug-qualified-20261018-133514-354151/qualified.json';q=read(qp)
assert(q['passed'],q['failed'],q['ignored'],q['integration_passed'],q['selected_library_passed'])==(156,0,0,110,46)
assert q['recorded_owned_all_terminal'] and q['resource_test_inventory_unchanged_from_8CC5'] and q['opted_in_original_resource_tests']==2
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and sha(q['normal']['path'])==q['normal']['sha256']
assert c['status']=='building_scoped_catalog_normal' and c['active_profile']=='Release' and c['owned_process'] is not None
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
with zipfile.ZipFile(qp.parent/'raw.zip') as z:
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    for n,h in q['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
runner=read(run/'original-runner-process.json');jobs=[runner,c['owned_process']];ids=','.join(str(j['pid']) for j in jobs)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'));live=[live] if isinstance(live,dict) else live
for j in jobs:assert any(row['ProcessId']==j['pid'] and abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())<.001 for row in live)
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('Debug-profile-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
(out/'live-process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
repair=run/'reader-repair-20261018-133412-090676';failure=read(repair/'failure.json');assert failure['observed_exit_code']==1 and failure['terminal_tool_chunk']=='b86024'
save(repair/'resolution.json',dict(status='reader_corrected_to_hash_pinned_unchanged_resource_test_in_prior_8CC5_compiled_source_archive',failed_reader_tool_chunk='b86024',source_or_product_or_test_result_changed=False,runtime_reexecuted=False,successful_reader_tool_chunk='aee67a',successful_qualification=str(qp),successful_qualification_sha256=sha(qp),superseded_scope_label_directory=str(run/'Debug-superseded-scope-label-20261018-133425-987354'),corrected_scope='unchanged lighting test inventory; six new catalog cases and one new four-spell asset/lifecycle case'))
e=dict(status='Debug_normal_and_related_156_qualified_Release_original_build_running',source_manifest_sha256=c['source_manifest_sha256'],qualification=str(qp),qualification_sha256=sha(qp),profile='Debug',normal_sha256=q['normal']['sha256'],passed=156,failed=0,ignored=0,integration_passed=110,selected_library_passed=46,integration_targets=17,selected_library_groups=4,opted_in_original_resource_tests=2,Debug_recorded_child_identities=67,Debug_recorded_children_terminal=True,original_session_id=53274,original_terminal_exit_code=None,original_runner_confirmed_live=True,Release_snapshot_owned_process=c['owned_process'],Release_qualified=False,whole_normal_validation_complete=False,reader_failure_resolution=str(repair/'resolution.json'),normal_GUI_verified=False,physical_audio_verified=False,current_source_C02_runtime_verified=False,whole_goal_complete=False)
save(out/'evidence.json',e)
work='來源33199F4E正常Debug已獨立讀回唯一入口建置、17整合目標及4組library共156過0敗0忽略，含兩個明確啟用原資源案例；67記錄子程序身分終止，讀回工具舊版封存假設錯誤與修正保留，產品及測試未重跑。Release正沿原53274建置，須續等原程序實際終止並獨立核對，尚無兩版完成資格。之後才執行當版G06控制器及Presentation／非空音效。8／60秒、Release／G11預算不變；8CC5證據按原來源保留，相鄰只讀、產物獨立。正常GUI／實體音效、完整G06及G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);identity=x['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        identity.update(Debug_normal_sha256=q['normal']['sha256'],Debug_normal_qualified_for_current_source=True,Release_normal_qualified_for_current_source=False,source_qualification_pending=True)
        x.update(current_native_scoped_catalog_Debug=e,source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='scoped-catalog-Debug-qualified-Release-original-build',work=work,applies_to_current_source=True,goal_complete=False)
        x['current_native_scoped_catalog_normal_active'].update(status=e['status'],current_profile='Release',Debug_qualification=str(qp),Debug_qualification_sha256=sha(qp),Debug_passed=156)
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
cp=run.parent/'current-continuation.json';cont=read(cp);(out/'continuation-before.json').write_bytes(cp.read_bytes());cont.update(status=e['status'],Debug_qualification=str(qp),Debug_qualification_sha256=sha(qp),latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),next_action=work,must_poll_original_session_before_restarting=True,original_terminal_exit_code=None);save(cp,cont)
print(json.dumps(dict(checkpoint=str(out),Debug_passed=156,Release_qualified=False,original_session=53274,original_runner_and_Release_child_live=True),ensure_ascii=False))
