"""Close the original current-source controller job and retain its exact qualified scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
a=argparse.ArgumentParser();a.add_argument('run',type=Path);a.add_argument('--session-id',type=int,required=True);a.add_argument('--terminal-chunk',required=True);a=a.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-rental-service-entry-return-controller')and a.session_id>0 and a.terminal_chunk not in ['','unknown','pending']
c=read(run/'cohort.json');qp=run/'qualified/qualified.json';q=read(qp);op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases'and c['owned_process']is None
assert q['status']=='independently_qualified_both_current_G06_controller_profiles_six_cases_each'
assert q['original_session_id']==o['original_session_id']==a.session_id and q['observed_terminal_exit_code']==o['observed_terminal_exit_code']==0
assert o['recorded_original_owned_processes_all_terminal']and o['qualification_sha256']==sha(qp)
assert q['actual_formal_sessions']==o['actual_qualified_formal_sessions']==96 and q['actual_cases']==12 and q['actual_test_executables']==4
assert c['source_manifest_sha256']==q['source_manifest_sha256']==o['source_manifest_sha256']
assert sha(qp.parent/'cohort-raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'cohort-raw.zip')as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h,n
for n,h in read(run/'source-inputs.json').items():assert sha(R/n)==h,n
normal=read(c['startup_qualification']);assert normal['status']=='independently_qualified_rental_service_entry_return_normal_related_both_profiles'and normal['passed_per_profile']==19 and normal['source_manifest_sha256']==c['source_manifest_sha256']
assert sha(c['startup_qualification'])==c['startup_qualification_sha256']
w=read(R/'工作佇列.json');assert w['active_workload_sessions']=={'G06_controller':a.session_id}
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
record=dict(status='current_catalog_source_controller_six_cases_each_profile_independently_qualified',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),completion_observation=str(op),completion_observation_sha256=sha(op),source_manifest_sha256=c['source_manifest_sha256'],reference_jar_sha256=c['reference_jar_sha256'],normal_mains=c['normal_mains'],original_session_id=a.session_id,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_cases=12,actual_sessions=96,actual_test_executables=4,recorded_owned_identities=o['recorded_original_owned_identities'],recorded_owned_all_terminal=True,qualified=True,scope='cross-endpoint,timeout,refused,wrong-service,cancel-retry,cancel-expiry controller tests in Debug and Release',current_source_broad_and_library_verified=False,current_Presentation_verified_for_source=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified_by_this_cohort=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源BC2CA7E9原'+str(a.session_id)+'：兩版六分支12案／96sessions已獨立核對，保留完整物品接續與原C9拒重播斷言。初次GAME C8來源確認、自然超30秒GUI及其他廣域／服務從各自當版資格讀回；同來源已核對library不重跑或移除。4F2F Release49194到期來源錯誤保留。完整GUI／G01–G12仍待，不請使用者操作，相鄰只讀，30秒／8秒／60秒與Release／G11預算不變。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d');staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);assert v['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        v['current_native_catalog_controller_qualification']=record
        scope=dict(state='current_source_controller_qualified_same_endpoint_Presentation_audio_other_services_GUI_pending',complete=False,current_controller_evidence=record,prior_source_same_endpoint_evidence=v['current_native_catalog_handoff_qualification'],current_source_same_endpoint_qualification_granted=False)
        v['current_source_goal_validation']['G06-room-service-branches']=scope
        for item in v.get('items',[]):
            if item.get('id')=='G06-room-service-branches':
                item.update(**scope,current_evidence=record,current_evidence_applies_to_current_source=True)
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':step.update(state=scope['state'],current_source_manifest_sha256=c['source_manifest_sha256'],current_controller_evidence=record,current_controller_six_both_profiles_evidence=str(qp),current_source_revalidation_note=work,current_Presentation_verified_for_source=False,current_failure_Presentation_verified_for_source=False,remaining=work)
        v.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='current-catalog-controller-qualified-limited',work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in v:v['updated_date']=date
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_session_before_restarting=False,next_action='Current normal-body Presentation nonempty-audio scope, then remaining service/GUI scopes; Retain matching-source independently qualified library; broad and GUI have separate scope.'))
print(json.dumps(dict(checkpoint=str(out),actual_sessions=96,whole_goal_complete=False),ensure_ascii=False))
