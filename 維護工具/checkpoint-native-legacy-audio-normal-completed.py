"""Register only the newly observed normal products and related test scope."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
run=V/'native-legacy-audio-normal/validation-20261018-104256-823814';c=read(run/'cohort.json');op=run/'completion-observation.json';o=read(op)
assert o['status']=='independently_qualified_legacy_audio_normal_both_profiles_and_terminal_processes'
assert o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal'] and sha(run/'cohort.json')==o['cohort_sha256']
assert c['status']=='executed_legacy_audio_normals_related' and c['owned_process'] is None
assert len(c['process_history'])==122 and all(j['exit_code']==0 for j in c['process_history'])
repair_dir=run/'manifest-serialization-repair';repair=read(repair_dir/'repair.json')
assert repair['observed_reader_exit_code']==1 and repair['failed_tool_chunk']=='e9e9c2' and repair['parsed_entries_identical']
assert sha(repair_dir/'source-inputs-before.json')==repair['before_sha256']
assert read(repair_dir/'source-inputs-before.json')==read(run/'source-inputs.json')==read(Path(c['application'])/'applied-source-inputs.json')
assert sha(run/'source-inputs.json')==repair['expected_sha256']==c['source_manifest_sha256']
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
profiles={}
for profile,item in o['profiles'].items():
    qp=Path(item['qualification']);q=read(qp);assert sha(qp)==item['qualification_sha256']
    assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['passed']==q['integration_passed']+q['selected_library_passed'] and q['selected_library_passed']==20
    assert len(q['targets'])==17 and q['recorded_owned_all_terminal'] and q['failed']==q['ignored']==0
    assert sha(q['normal']['path'])==q['normal']['sha256']==item['normal_sha256']
    assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
    with zipfile.ZipFile(qp.parent/'raw.zip') as z:
        for n,h in q['raw_files'].items():
            with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
    profiles[profile]=dict(**item,integration_passed=q['integration_passed'],selected_library_passed=q['selected_library_passed'])
formal=ROOT.parent/'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal)=='C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_normal_both_profiles_seventeen_integration_targets_and_two_selected_library_groups_qualified_limited',
    source_manifest_sha256=c['source_manifest_sha256'],source_files=1040,compiled_input_files=1041,
    normal_completion=str(op),normal_completion_sha256=sha(op),normal_profiles=profiles,
    normal_recorded_terminal_identities=o['recorded_original_owned_identities'],normal_related_validation_complete=True,
    normal_product_registered=True,current_source_C02_runtime_verified=False,full_library_verified=False,
    current_broad_default_verified=False,full_G06_verified=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False,
    diagnostic_C_ACTION_API_applied=False,
    ordinary_event_admission_after_new_Model_same_world_ABA_verified=False,
    preserved_manifest_serialization_reader_failure=dict(path=str(repair_dir/'repair.json'),sha256=sha(repair_dir/'repair.json'),
        original_reader_exit_code=1,original_reader_tool_chunk='e9e9c2',parsed_source_entries_identical=True,product_source_changed=False))
save(out/'evidence.json',record)
counts='／'.join(f"{p}{v['passed']}" for p,v in profiles.items())
work=f'來源8CC5A560正常兩版唯一入口建置與相關驗證已沿原78702實際終止0；每版17整合目標及2選定library群組共{counts}過，各0敗0忽略，來源／逐映像／raw與記錄程序身分終止已獨立核對。一般WAV的環境延遲只交付一次及取消退役已納入正常回歸；診斷C_ACTION入口未套入。此資格僅相關範圍；舊4E1廣域／完整library及候選跨端點結果各保留原來源，不移記。下一步當版G06控制器／Presentation／來源音效實際流程與必要廣域／library；正常GUI／實體音效、同端點及完整G01–G12仍待。8／60秒、Release／G11預算不變，登入器與重建版獨立且相鄰只讀。'
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);identity=value['current_product_identity'];assert identity['source_manifest_sha256']==c['source_manifest_sha256']
        if 'updated_date' in value:value['updated_date']='2026-10-18'
        identity.update(source_qualification_pending=False,Debug_normal_sha256=profiles['Debug']['normal_sha256'],Release_normal_sha256=profiles['Release']['normal_sha256'],
            existing_normal_source_manifest_sha256=c['source_manifest_sha256'],Debug_normal_qualified_for_current_source=True,Release_normal_qualified_for_current_source=True,
            qualification_scope='normal builds, seventeen integration targets and two selected library groups; not full-library/current TCP/GUI/full-G06')
        value['current_native_legacy_audio_application'].update(normal_validation_pending=False,qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'))
        value['current_native_legacy_audio_normal']=record
        value['current_native_legacy_audio_normal_active'].update(status='completed_qualified_related_scope',completion=str(op),completion_sha256=sha(op))
        value.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current8CC5-G06-controller-and-source-runtime',work=work,applies_to_current_source=True,goal_complete=False)
        for key in value.get('current_source_goal_validation',{}):value['current_source_goal_validation'][key]=dict(state='8CC5_related_qualified_broader_acceptance_pending',complete=False)
        for item in value.get('items',[]):
            if item['id'].startswith('G01-'):
                item['retained_pre_8CC5_current_evidence']=item.get('current_evidence')
                item.update(state='8CC5_current_normal_source_baseline_verified_limited',complete=True,
                    current_source_manifest_sha256=c['source_manifest_sha256'],
                    current_source_evidence_scope='正常兩版唯一入口建置、每版17相關整合目標與2選定library群組；非完整遊戲驗收。',
                    current_evidence=dict(qualification=str(out/'evidence.json'),qualification_sha256=sha(out/'evidence.json'),
                        source_manifest_sha256=c['source_manifest_sha256'],whole_goal_complete=False))
            if item['id'].startswith('G03-'):item.update(state='8CC5_related_qualified_broad_library_pending',complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        path.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
continuation=run.parent/'current-continuation.json';assert not continuation.exists()
save(continuation,dict(status='normal_both_profiles_related_scope_qualified_current_source_G06_pending',
    source_manifest_sha256=c['source_manifest_sha256'],normal_run=str(run),completion=str(op),completion_sha256=sha(op),
    checkpoint=str(out),checkpoint_readback_sha256=sha(out/'readback.json'),normal_profiles=profiles,
    next_driver=str(ROOT/'維護工具/run-native1040-G06-legacy-audio-normal.py'),
    next_harness=str(ROOT/'維護工具/candidates/native-teleport-fixture-arm/NativeTeleportHarness.java'),
    next='Run six existing C02 controller branches on both profiles from this qualified normal source, then current Presentation/source-audio and required broad/library scopes.',
    retained_limitation='Scope latches retire retained payloads; a newly constructed Model replaying ordinary actor/wire events in the same World or resource-root ABA remains unqualified.',
    normal_GUI_runtime_inferred=False,physical_audio_verified=False,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),passed_each={p:v['passed'] for p,v in profiles.items()},whole_goal_complete=False),ensure_ascii=False))
