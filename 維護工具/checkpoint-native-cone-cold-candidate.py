"""Record candidate-only cold evidence and retain the failed join gate."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
def ref(p):return dict(path=str(p),sha256=sha(p))
def archive(qp,q):
    p=qp.parent/'raw.zip';assert sha(p)==q['raw_archive_sha256']
    with zipfile.ZipFile(p)as z:
        assert set(z.namelist())==set(q['raw_files'])
        for n,h in q['raw_files'].items():
            with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
base='86E65C7B9D9D77FF73FF154B59F28465B75BA1B3C1D1C6EF315D4990584B45AE'
normal=V/'native-class2-normal/validation-20261018-014519-231950'
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
core=V/'native-cone-cold-core-runtime/20261018-050623-018724/qualified-20261018-050642-031925/qualified.json'
c=read(core);assert c['cases_each']==7 and c['profiles_exact_outcomes_equal'] and c['observed_actual_exit_code']==0
assert c['recorded_original_identities_all_terminal'] and not c['normal_product_registered'];archive(core,c)
cold_names=[
 '20261018-051131-467719/qualified-20261018-051251-009546',
 '20261018-051304-605547/qualified-20261018-051449-222869',
 '20261018-051450-866083/qualified-20261018-051617-604143',
 '20261018-051619-038716/qualified-20261018-051801-713912',
 '20261018-053244-690900/qualified-20261018-053351-615294',
 '20261018-053423-787910/qualified-20261018-053527-765372']
cold=[]
for n in cold_names:
    p=V/'native-cone-cold-live'/n/'qualified.json';q=read(p)
    assert q['kind']=='cold' and q['base_source_manifest_sha256']==base and q['original_exit_code']==0
    assert q['recorded_owned_all_terminal'] and not q['normal_product_registered'];archive(p,q)
    cold.append(dict(**ref(p),profile=q['compile_profile'],fixture=q['profile'],sessions=q['actual_sessions'],requests=q['actual_requests'],accepted=q['actual_accepted_casts']))
assert sum(x['sessions']for x in cold)==24 and sum(x['requests']for x in cold)==10 and sum(x['accepted']for x in cold)==8
moving=[]
for name,chunk in [('20261018-053244-690900/moving-qualified-20261018-053422-449724','b34654'),('20261018-053423-787910/moving-qualified-20261018-053544-969470','7d1f42')]:
    p=V/'native-cone-cold-live'/name/'qualified.json';q=read(p)
    assert q['status']=='independently_qualified_actual_C02_target_step_during_captured_cold_flight'
    assert sha(q['admission_qualification'])==q['admission_qualification_sha256'] and q['target_before']!=q['target_after']
    assert q['controlled_presentation_pause'] and not q['performance_acceptance']
    moving.append(dict(**ref(p),profile=q['compile_profile'],reader_observed_exit_code=0,reader_terminal_chunk=chunk))
class2=[]
for n in ['20261018-052040-707632/qualified-20261018-052348-058531','20261018-052349-477581/qualified-20261018-052541-946582','20261018-052543-217371/qualified-20261018-052813-838079']:
    p=V/'native-cone-cold-class2-regression'/n/'qualified.json';q=read(p)
    assert q['kind']=='class2' and q['original_exit_code']==0 and q['recorded_owned_all_terminal'];archive(p,q)
    class2.append(dict(**ref(p),profile=q['compile_profile'],fixture=q['profile']))
failure=V/'native-cone-cold-class2-regression/20261018-053532-621374/failure-boundary/boundary.json'
f=read(failure);assert f['observed_actual_exit_code']==1 and f['actual_C14_requests']==0 and f['recorded_original_identities_all_terminal'];archive(failure,f)
unobserved=V/'native-cone-cold-class2-regression/20261018-052815-740463/receipt.json'
u=read(unobserved);assert u['all_started_processes_terminal'] and all(x['exit_code']==0 for x in u['process_history'])
previous=V/'native1034-library-default/20261018-043955-238295/checkpoint-completed-20261018-050600-778232/readback.json'
before=read(previous)['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
continuation=V/'native1034-library-default/current-continuation.json'
out=V/'native-cone-cold-integration'/('candidate-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
backup={**{n:ROOT/n for n in before},'library-current-continuation.json':continuation};backup_sha={n:sha(p)for n,p in backup.items()}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in backup.items():z.write(p,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in backup_sha.items())
record=dict(status='cold_candidate_scopes_qualified_normal_unapplied_join_failure_gate_open',
    base_source_manifest_sha256=base,normal_product_modified=False,normal_product_registered=False,
    core=ref(core),core_cases_each=7,cold_C02=cold,cold_C02_sessions=24,cold_C14_requests=10,cold_accepted_casts=8,cold_refused_casts=2,
    controlled_moving_target=moving,existing34_qualified_scopes=class2,
    existing34_Release_refusal_unqualified_completed_receipt=dict(**ref(unobserved),outer_exit_observed=False,original_outer_session_id=None,qualification_granted=False),
    existing34_Release_refusal_repetition_failure=ref(failure),
    next_gate='Deterministically replay pending-join D0/CE ordering in the current controller, then prepare a bounded epoch/requested-room fix and adversarial cancellation/identity/deadline checks before normal integration. Preserve the failure; do not rerun until ordering is understood.',
    old_external_clock_mapping_qualified=False,GUI_verified=False,physical_audio_verified=False,
    full_G06_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源86E65C7B正常版本未改。冰錐隔離核心兩版各7案及C02六案24sessions已獨立核對：10請求、8接受／2拒絕；含兩版飛行中真實移動，舊路徑與新座標命中一致。既有34三案通過；Release拒絕重驗在施法前因RoomJoining離房事件失敗，原37509 exit1與raw已封存，未算通過。先確定性重播D0／CE順序並補有界修復，再整合正常技能及請求／原子性檢查；40項缺口未縮減。原正常廣域／library與G06限定資格保留；GUI／實體音效、完整G06及G01–G12未完成。登入器／正式JAR只讀、產物獨立，替換提案撤回；8／60秒、Release與G11不變。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==base
        j['current_native_cone_cold_candidate']=dict(**ref(out/'evidence.json'),**record)
        if 'current_native_cone_cold_integration_proposal'in j:
            j['current_native_cone_cold_integration_proposal'].update(candidate_runtime_executed=True,normal_product_registered=False,candidate_checkpoint=str(out/'evidence.json'))
        j.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_focus']=dict(item='pending-join-departure-ordering-before-cold-normal-integration',work=work,applies_to_current_source=True,normal_product_modified=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18')and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
j=read(continuation);j['cold_latest_candidate_checkpoint']=ref(out/'evidence.json');j['next_candidate_action']=record['next_gate'];j['active_profile']=None;j['requires_live_handle_revalidation_before_resume']=False;save(continuation,j)
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
save(out/'readback.json',dict(before_files=before,before_archive_sha256=sha(out/'before.zip'),backup_files=backup_sha,after_files={n:sha(ROOT/n)for n in before},continuation_after=ref(continuation),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
for n,h in read(normal/'compiled-inputs.json').items():assert sha(ROOT/n)==h
print(json.dumps(dict(checkpoint=str(out),cold_sessions=24,accepted=8,refused=2,normal_product_modified=False,whole_goal_complete=False),ensure_ascii=False))
