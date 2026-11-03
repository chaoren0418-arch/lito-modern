"""Register only the newly observed normal products and related test scope."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
run=V/'native-scoped-catalog-normal/validation-20261018-132210-585372';c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert o['status']=='independently_qualified_scoped_catalog_normal_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and sha(run/'cohort.json')==o['cohort_sha256']
assert c['status']=='executed_scoped_catalog_normals_related' and c['owned_process'] is None
assert len(c['process_history'])==134 and all(j['exit_code']==0 for j in c['process_history'])
assert (run/'source-inputs.json').read_bytes()==(Path(c['application'])/'applied-source-inputs.json').read_bytes()
assert sha(run/'source-inputs.json')==c['source_manifest_sha256']
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
profiles={}
for profile,item in o['profiles'].items():
    qp=Path(item['qualification']);q=read(qp);assert sha(qp)==item['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['passed']==q['integration_passed']+q['selected_library_passed'] and q['selected_library_passed']==46
    assert len(q['targets'])==17 and q['recorded_owned_all_terminal'] and q['failed']==q['ignored']==0
    assert sha(q['normal']['path'])==q['normal']['sha256']==item['normal_sha256']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        for n,h in q['raw_files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(**item,integration_passed=q['integration_passed'],selected_library_passed=q['selected_library_passed'])
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
drift_path=V/'formal-75DA69B0/observation-20261018-140916-340781/drift.json';drift=read(drift_path)
assert drift['status']=='stable_readonly_formal_JAR_drift_observed_and_isolated_copy_pinned'
assert sha(formal)==sha(drift['isolated_copy'])==drift['formal_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(drift['prior_copy'])==drift['prior_sha256']=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert not drift['current_formal_compatibility_qualified'] and not drift['source_modified']
wire_failure=V/'native33199-ordinary-audio-ABA/20261018-135628-012455/failure-observation.json';wf=read(wire_failure)
assert wf['actual_regression_child_exit_code']==101 and not wf['regression_passed'] and wf['source_manifest_sha256']==c['source_manifest_sha256']
latest=Path(read(run.parent/'current-continuation.json')['latest_readback']).resolve(strict=True);assert latest.is_relative_to(run)
before=read(latest)['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_normal_both_profiles_seventeen_integration_targets_and_four_selected_library_groups_qualified_limited',
    source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    normal_completion=str(op),normal_completion_sha256=sha(op),normal_profiles=profiles,
    normal_recorded_terminal_identities=o['recorded_original_owned_identities'],normal_related_validation_complete=True,
    normal_product_registered=True,current_source_C02_runtime_verified=False,full_library_verified=False,
    current_broad_default_verified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,
    diagnostic_C_ACTION_API_applied=False,
    ordinary_event_admission_after_new_Model_same_world_ABA_verified=False)
record.update(current_formal_reference_sha256=drift['formal_sha256'],formal_reference_drift_observation=str(drift_path),formal_reference_drift_observation_sha256=sha(drift_path),current_formal_compatibility_qualified=False,known_S4A_Model_recreation_failure=str(wire_failure),known_S4A_Model_recreation_failure_sha256=sha(wire_failure),ordinary_wire_audio_Model_recreation_regression_passed=False)
save(out/'evidence.json',record)
counts='／'.join(f"{p}{v['passed']}" for p,v in profiles.items())
work=f'來源33199F4E正常兩版唯一入口建置、每版17整合及4組library沿原53274實際退出0，{counts}過，各0敗0忽略與154記錄身分終止已獨立核對。只授予正常建置及相關範圍；新增隔離回歸實際101，證明同World重建Model會重新準備舊S4A音效，普通角色動作／來源ABA仍待。外部登入器JAR在建置期間改為75DA69B0，只讀差異僅一個MySQL repository類及新增SourceBuffRead；新副本已限本重建版保存，舊C02相容結果不移記新JAR。下一步驗證隔離S4A游標候選、更新75DA參考綁定及當版G06流程。來源與QA凍結，診斷API／計時未套入；8／60秒及Release／G11預算不變，相鄰只讀且產物獨立。正常GUI／實體音效、完整G06及G01–G12未完成。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        if 'updated_date' in value:value['updated_date']='2026-10-18'
        identity.update(source_qualification_pending=False,Debug_normal_sha256=profiles['Debug']['normal_sha256'],Release_normal_sha256=profiles['Release']['normal_sha256'],
            existing_normal_source_manifest_sha256=c['source_manifest_sha256'],Debug_normal_qualified_for_current_source=True,Release_normal_qualified_for_current_source=True,
            qualification_scope='normal builds, seventeen integration targets and four selected library groups; not full-library/current TCP/GUI/full-G06')
        value['current_native_scoped_catalog_application'].update(normal_validation_pending=False,qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'))
        value['current_native_scoped_catalog_normal']=record
        value['current_native_scoped_catalog_normal_active'].update(status='completed_qualified_related_scope',completion=str(op),completion_sha256=sha(op))
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current33199-G06-controller-and-source-runtime',work=work,applies_to_current_source=True,goal_complete=False)
        for key in value.get('current_source_goal_validation',{}):value['current_source_goal_validation'][key]=dict(state='33199_related_qualified_broader_acceptance_pending',complete=False)
        for item in value.get('items',[]):
            if item['id'].startswith('G01-'):
                item['retained_pre_scoped_catalog_current_evidence']=item.get('current_evidence')
                item.update(state='33199_current_normal_source_baseline_verified_limited',complete=True,
                    current_source_manifest_sha256=c['source_manifest_sha256'],
                    current_source_evidence_scope='正常兩版唯一入口建置、每版17相關整合目標與4選定library群組；非完整遊戲驗收。',
                    current_evidence=dict(qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'),
                        source_manifest_sha256=c['source_manifest_sha256'],whole_goal_complete=False))
            if item['id'].startswith('G03-'):item.update(state='33199_related_qualified_broad_library_pending',complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation=run.parent/'current-continuation.json'
if continuation.exists():(out/'continuation-before.json').write_bytes(continuation.read_bytes())
save(continuation,dict(status='normal_both_profiles_related_scope_qualified_current_source_G06_pending',
    source_manifest_sha256=c['source_manifest_sha256'],normal_run=str(run),completion=str(op),completion_sha256=sha(op),
    checkpoint=str(out),checkpoint_readback_sha256=sha(out/'readback.json'),normal_profiles=profiles,
    next_driver=str(ROOT/'維護工具/run-native1040-G06-scoped-catalog-normal.py'),
    next_harness=str(ROOT/'維護工具/candidates/native-teleport-fixture-arm/NativeTeleportHarness.java'),
    next='Run six existing C02 controller branches on both profiles from this qualified normal source, then current Presentation/source-audio and required broad/library scopes.',
    retained_limitation='Scope latches retire retained payloads; a newly constructed Model replaying ordinary actor/wire events in the same World or resource-root ABA remains unqualified.',
    normal_GUI_runtime_inferred=False,physical_audio_verified=False,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),passed_each={p:v['passed'] for p,v in profiles.items()},whole_goal_complete=False),ensure_ascii=False))
