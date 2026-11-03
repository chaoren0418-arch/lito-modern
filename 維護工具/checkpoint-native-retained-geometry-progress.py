"""Publish measured warm results without promoting them to G11 acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
related=ROOT/'驗證紀錄/current/native-retained-geometry/validation-20261017-004225-579828'
warm=ROOT/'驗證紀錄/current/native-performance/retained-geometry-warm-20261017-005627-086969'
siege=ROOT/'驗證紀錄/current/native-startup-siege563/20261017-005729-442442'
g06=ROOT/'驗證紀錄/current/native-G06-controller/20261017-010712-984244'
skill=ROOT/'驗證紀錄/current/native-current-skill-support/20261017-010655-739171/qualified.json'
self_tick=ROOT/'驗證紀錄/current/native-self-tick-contract/20261017-011309-401402/codec-qualified.json'
library=ROOT/'驗證紀錄/current/native-startup-library-default/20261017-012714-666943'
c=read(related/'completion-observation.json');w=read(warm/'independent-readback.json');s=read(siege/'cohort.json')
assert c['observed_terminal_exit_code']==w['observed_terminal_exit_code']==0
assert c['source_manifest_sha256']==w['source_manifest_sha256']==s['source_manifest_sha256']
assert w['measured_reloads']==40 and w['synthetic_warm_p95_budget_met'] and not w['performance_accepted']
assert w['wall_us']['p95_nearest_rank']==167899 and not w['budget_changed']
pins=read(related/'compiled-inputs.json');assert len(pins)==1025
assert all(sha(ROOT/n)==h for n,h in pins.items())
done=(siege/'qualified/qualified.json').exists()
if done:
    sq=read(siege/'qualified/qualified.json')
    assert sq['original_session_id']==5214 and sq['observed_terminal_exit_code']==0 and sq['actual_formal_sessions']==120
else:
    assert s['status']=='running_current_map563_thirty_player_TW_extended'
g06_started=(g06/'cohort.json').exists()
g06_done=(g06/'completion-observation.json').exists()
g06_record=None
if g06_started:
    gc=read(g06/'cohort.json');assert gc['source_manifest_sha256']==c['source_manifest_sha256']
    g06_record=dict(run=str(g06),original_session_id=46631,status=gc['status'],
        source_manifest_sha256=c['source_manifest_sha256'],input_kind='retained-geometry-related',
        source_and_QA_frozen=not g06_done,current_both_profiles_qualified=g06_done,GUI_verified=False,whole_goal_complete=False)
    if g06_done:
        gq=read(g06/'completion-observation.json')
        assert gq['original_session_id']==46631 and gq['observed_terminal_exit_code']==0
        assert gq['actual_qualified_formal_sessions']==96 and gq['recorded_original_owned_processes_all_terminal']
        g06_record.update(completion_observation=str(g06/'completion-observation.json'),
            completion_observation_sha256=sha(g06/'completion-observation.json'),actual_qualified_formal_sessions=96)
skill_record=None
if skill.exists():
    sk=read(skill)
    assert sk['source_manifest_sha256']==c['source_manifest_sha256']
    assert (sk['observed_unique_learned_ids'],sk['request_recognized_learned_ids'],sk['missing_unique_learned_requests'])==(74,21,53)
    skill_record=dict(sk,qualification=str(skill),qualification_sha256=sha(skill))
self_tick_record=None
if self_tick.exists():
    st=read(self_tick)
    assert st['jar_sha256']==read(skill)['jar_sha256']
    assert st['positive_vectors']==12 and st['negative_rejections']==26
    assert st['actual_casts']==0 and not st['native_clock_source_verified']
    self_tick_record=dict(st,qualification=str(self_tick),qualification_sha256=sha(self_tick),
        helper_tick_dataflow_review=str(self_tick.parent/'tick-dataflow-review.json'),
        helper_tick_dataflow_review_sha256=sha(self_tick.parent/'tick-dataflow-review.json'))
library_started=(library/'cohort.json').exists()
library_done=(library/'completion-observation.json').exists()
library_record=None
if library_started:
    lc=read(library/'cohort.json');li=read(library/'original-runner-process.json')
    assert li['original_session_id']==80951 and lc['source_manifest_sha256']==c['source_manifest_sha256']
    library_record=dict(run=str(library),original_session_id=80951,runner_identity=li,status=lc['status'],
        source_manifest_sha256=c['source_manifest_sha256'],input_kind='retained-geometry-related',
        source_and_QA_frozen=not library_done,both_profiles_qualified=library_done,
        ignored_bodies_executed=False,GUI_verified=False,whole_goal_complete=False)
    library_record['qualified_profiles']={}
    for profile in ['Debug','Release']:
        profile_path=library/profile/'full-default-qualified/qualified.json'
        if profile_path.exists():
            pq=read(profile_path)
            assert pq['source_manifest_sha256']==c['source_manifest_sha256'] and pq['failed']==0
            library_record['qualified_profiles'][profile]=dict(qualification=str(profile_path),qualification_sha256=sha(profile_path),
                passed=pq['passed'],failed=pq['failed'],ignored=pq['ignored'],actual_test_sha256=pq['actual_test_sha256'])
    if library_done:
        lq=read(library/'completion-observation.json')
        assert lq['original_session_id']==80951 and lq['observed_terminal_exit_code']==0
        assert lq['recorded_owned_processes_all_terminal']
        library_record.update(completion_observation=str(library/'completion-observation.json'),
            completion_observation_sha256=sha(library/'completion-observation.json'),profiles=lq['profiles'])
files=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json',
       '驗證紀錄/current/native-terrain-residence/continuation.json','README.md','docs/HISTORY.md']
out=related/('progress-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
before={n:sha(ROOT/n) for n in files}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in files:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:
    assert set(z.namelist())==set(files)
    assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
performance=dict(w,qualification=str(warm/'independent-readback.json'),qualification_sha256=sha(warm/'independent-readback.json'),
    terminal_tool_chunk_id='e43812',warmup_pairs=5,baseline_source_manifest_sha256='4C7DE771A82D28CBBDCC513253376165D6E0047C27994B9521617F3880B7AA4C',
    baseline_qualification=str(ROOT/'驗證紀錄/current/native-performance/warm-residence-20261017-003129-184958/independent-readback.json'),
    baseline_p95_us=463583,current_p95_us=167899,original_warm_p95_budget_ms=250,G11_complete=False,
    next='Continue current G06 and required regression scopes; GUI/physical/60min/20cycle acceptance remains pending.' if done else 'Validate current C02 thirty-player residence/continuation, then current G06 and required regression scopes; GUI/physical/60min/20cycle acceptance remains pending.')
runtime=dict(run=str(siege),original_session_id=5214,input_kind='retained-geometry-related',source_manifest_sha256=c['source_manifest_sha256'],
    status='both_profiles_current_C02_map563_qualified' if done else s['status'],actual_formal_sessions=120 if done else 0,
    pending_actual_sessions_not_qualified=not done,source_and_QA_frozen=not done,GUI_verified=False,whole_goal_complete=False)
if done:runtime.update(qualification=str(siege/'qualified/qualified.json'),qualification_sha256=sha(siege/'qualified/qualified.json'),observed_terminal_exit_code=0)
note=('新來源6D189143地形/導航持有修正兩版十實際EXEs已52/48過、各0敗3忽略，原4754actualexit0。'
      '獨立讀回之Release單角色合成Model同條件5暖機對/40重載，原61363actualexit0：median162911/p95167899/max169777微秒；'
      '舊4C7 p95463583微秒保留，原250ms p95及1000ms單次預算不變，此診斷達門檻；不是30人GUI或完整G11接受證據。')
note+=('新來源C02三十人map563兩版120sessions已原5214actualexit0並獨立讀回。' if done else
       '新來源C02三十人map563兩版沿原5214，來源與該runner/三QA固定，未算通過。')
note+=('4C7各限定資格保留舊來源，當版廣域/必要原資源opt-in/GUI仍待；登入器產物維持只讀，完整G01–G12未完成。')
if g06_started:
    note+=('新來源G06兩版六分支96sessions原46631actualexit0與記錄owned退出已獨立核對；' if g06_done else
           '新來源G06六分支兩版沿原46631執行，來源/對應QA固定，未算完成；')
if skill_record:note+='新來源雙版S1E與請求表重核74已學/21請求辨識/53缺口，無新施法，偵測術候選仍未套產品。'
if self_tick_record:note+='C02隱身術SELF_TICK codec已兩設定實跑12有效/26拒絕並獨立讀回，140703後接unsigned LE u32；只有codec與helper tick最終log用途的靜態核對，未認定原生時鐘或施法，53缺口不減。'
if library_record:note+=('當版完整預設library雙版已原80951actualexit0及獨立讀回；忽略本文另列範圍。' if library_done else '當版完整預設library沿原80951，來源及runner/四QA固定；尚未記為通過。')
if library_record and not library_done and 'Debug' in library_record['qualified_profiles']:
    note+='Debug已1935/0/473獨立讀回，Release仍沿原工作；不推定雙版完成。'
for n in files[:4]:
    v=read(ROOT/n)
    if 'current_product_identity' in v and 'retained_product_identity_before_geometry_retention' not in v:
        v['retained_product_identity_before_geometry_retention']=v['current_product_identity']
    identity=dict(source_manifest_sha256=c['source_manifest_sha256'],source_files=1024,compiled_input_files=1025,
        normal_mains={p:i['normal_sha256'] for p,i in c['profiles'].items()},
        qualification=str(related/'completion-observation.json'),qualification_sha256=sha(related/'completion-observation.json'),
        qualified_scope='related_ten_actual_test_images_each_profile_and_unique_entry_normals',
        broad_default_current_source_pending=True,full_library_current_source_pending=not library_done,GUI_verified=False,whole_goal_complete=False)
    v.update(current_G11_warm_reload_followup=performance,current_map563_retained_geometry_revalidation=runtime,
        current_product_identity=identity,
        active_workload_sessions={'geometry_full_library':80951} if library_started and not library_done else {'geometry_G06':46631} if g06_started and not g06_done else {} if done else {'retained_geometry_map563':5214},
        source_freeze_for_live_retained_geometry_siege=not done,current_working_tree_verified=False,
        current_product_version_scope='source1024_6D189143_related_both_and_warm_Model_qualified; '+('C02_map563_both_qualified' if done else 'C02_map563_pending')+'; G06_broad_library_GUI_G11_pending',
        next_task=note,last_goal_turn_classification='progress',last_goal_turn_evidence=[str(warm/'independent-readback.json'),str(siege/'cohort.json')],
        progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    correction=v.get('current_retained_geometry_correction',{})
    correction.update(warm_diagnostic=performance,current_map563=runtime);v['current_retained_geometry_correction']=correction
    if g06_record:
        v.update(current_geometry_G06_controller_validation=g06_record,source_freeze_for_current_G06_continuation=not g06_done)
        v['current_product_version_scope']+='; G06_both_qualified' if g06_done else '; G06_active_46631'
    if library_record:
        v.update(current_geometry_full_library=library_record,current_library_run=str(library),
            current_library_phase=lc['status'],source_freeze_for_live_library_job=not library_done,
            source_freeze_while_Cargo=not library_done,pending_current_library_session=None if library_done else 80951)
        v['current_product_version_scope']+='; full_library_both_qualified' if library_done else '; full_library_active_80951'
    if skill_record:v.update(current_native_skill_support=skill_record,current_wire_learned_native_skill_gap=skill_record)
    if self_tick_record:v['current_self_tick_codec_contract']=self_tick_record
    if g06_record:v['last_goal_turn_evidence'].append(str(g06/'cohort.json'))
    if self_tick_record:v['last_goal_turn_evidence'].append(str(self_tick))
    if library_record:v['last_goal_turn_evidence'].append(str(library/'cohort.json'))
    pair=ROOT/'維護工具/candidates/native-detection-invisibility/metadata.json'
    if pair.exists():
        pair_meta=read(pair)
        assert not pair_meta['candidate_applied_to_product'] and not pair_meta['product_candidate_compiled']
        v['current_detection_invisibility_preparation']=dict(metadata=str(pair),metadata_sha256=sha(pair),
            skill_ids=[13,60],candidate_applied_to_product=False,candidate_compiled=False,
            current_missing_learned_requests=53,new_product_casts=0,whole_goal_complete=False)
    for item in v.get('items',[]):
        if item['id']=='G11-stability-performance':
            item.update(state='source6D_quiet_synthetic_Model_warm_budget_met_GUI_thirty_player_long_stability_pending',complete=False)
        if item['id']=='G06-room-service-branches' and g06_done:
            item.update(state='source6D_controller_six_and_map563_both_profiles_qualified_GUI_and_remaining_branches_pending',complete=False,
                current_controller_completion=str(g06/'completion-observation.json'))
            for step in item.get('rental_acceptance_steps',[]):
                if step['id']=='G06.3d':step['current_source6D_controller_completion']=str(g06/'completion-observation.json')
        if item['id']=='G03-current-regression' and library_record:
            item.update(state='source6D_related_and_G06_and_full_library_both_qualified_broad_pending' if library_done else 'source6D_related_and_G06_qualified_full_library_active_broad_pending',complete=False)
    save(ROOT/n,v)
for n in files[4:]:
    p=ROOT/n;header,_,body=p.read_text(encoding='utf-8-sig').partition('\n')
    p.write_text(header+'\n\n2026-10-17：'+note+'\n'+body,encoding='utf-8',newline='\n')
save(out/'record.json',dict(before=before,before_archive_sha256=sha(out/'before.zip'),after={n:sha(ROOT/n) for n in files},
    warm=performance,map563=runtime,product_source_changed=False,whole_goal_complete=False))
(out/'checkpoint-tool.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(checkpoint=str(out),warm_p95_us=167899,map563_both_qualified=done,G11_complete=False,whole_goal_complete=False),ensure_ascii=False))
