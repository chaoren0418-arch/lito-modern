"""Register Release-only proof and keep Debug readiness failure and parent exit1 explicit."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('parent',type=Path);a=ap.parse_args()
parent=a.parent.resolve(strict=True);assert parent.is_relative_to(V/'native8CC5-audio-cohort')
c=read(parent/'cohort.json');qp=parent/'qualified-Release-after-parent-failure/qualified.json';q=read(qp)
assert c['status']=='failed_preserved' and c['owned_process'] is None and q['status']=='independently_qualified_Release_six_completed_audio_branches_only_after_Debug_parent_failure'
assert(q['actual_cases'],q['actual_sessions'],q['retained_source_cues'],q['new_scene_sound_events_delivered_once'])==(6,48,12,12)
assert q['observed_parent_terminal_exit_code']==1 and not q['whole_parent_cohort_passed'] and not q['Debug_scope_qualified'] and not q['product_runtime_reexecuted']
assert q['normal_source_bodies_unchanged'] and not q['source_overlays_used'] and q['diagnostic_request_API_extension']
assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip') as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
fp=Path(q['parent_failure']['path']);failure=read(fp);assert sha(fp)==q['parent_failure']['sha256'] and failure['recorded_owned_all_terminal'] and failure['original_terminal_exit_code']==1
for n,h in read(parent/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(fp.parent/'readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=parent/('Release-qualified-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
cases={}
for key,pin in q['cases'].items():
    path=Path(pin['path']);item=read(path);assert sha(path)==pin['sha256'] and item['profile']=='Release' and item['recorded_owned_all_terminal']
    cases[key]=dict(qualification=str(path),qualification_sha256=sha(path),source_manifest_sha256=item['source_manifest_sha256'],
        profile='Release',scenario=item['scenario'],actual_sessions=item['actual_sessions'],actual_product_sha256=item['actual_product_sha256'],
        recorded_owned_all_terminal=True,normal_source_bodies_unchanged=True,source_overlays_used=False,diagnostic_request_API_extension=True,
        unplayed_source_audio_rejection_verified=True,new_scene_own_sound_events_delivered_once_each=True,
        original_individual_session_id=item.get('original_session_id'),original_individual_exit_code=item.get('original_exit_code'),
        observed_parent_session_id=item.get('observed_parent_session_id'),observed_parent_terminal_exit_code=item.get('observed_parent_terminal_exit_code'),
        observed_case_process_exit_code=item.get('observed_case_process_exit_code'),normal_GUI_API_permission_inferred=False,
        normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
record=dict(status='current_Release_only_normal_body_Presentation_nonempty_audio_six_branches_qualified_Debug_readiness_failed',
    qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],diagnostic_input_manifest_sha256=q['diagnostic_input_manifest_sha256'],
    qualified_profiles=['Release'],profile='Release',actual_cases=6,actual_sessions=48,actual_diagnostic_executables=2,
    observed_parent_session_id=q['observed_parent_session_id'],observed_parent_terminal_exit_code=1,parent_terminal_tool_chunk=q['parent_terminal_tool_chunk'],
    whole_parent_cohort_passed=False,parent_failure=q['parent_failure'],cases=q['cases'],actual_Presentation_executed=True,
    retained_source_cues=12,old_cue_deliveries=0,new_scene_sound_events_delivered_once=12,actual_C1F_wave_requests=22,
    recovered_ordinary_wave_deliveries=10,destination_native_heal_deliveries=2,unplayed_source_audio_rejection_verified=True,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_GUI_API_permission_inferred=False,
    product_runtime_reexecuted=False,normal_product_source_modified=False,normal_EXE_runtime_inferred=False,
    Debug_scope_qualified=False,same_World_new_Model_ABA_event_admission_verified=False,GUI_verified=False,physical_audio_verified=False,
    same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源8CC5A560之Release正常模組Presentation／非空音效六分支48sessions已按實際子程序exit0獨立核對，保留父原88600退出1與Debug失敗；沒有重跑Release。12個舊待播cue皆Stale／重複AlreadyConsumed且零交付，10個恢復一般音效與2個目的地heal109各交付一次；完整物品、原始封包與24張診斷PNG已核對。只授予Release正常本體加隔離API／假Player範圍，Debug未授予；其初始Frame就緒8秒界限失敗，未送C1F／傳送，下一步先量測工作排程、來源等待、載入與首張繪圖成本。正常各130與控制器兩版96sessions維持限定資格。正常GUI／實體音效、同端點／其他服務、同World新Model／來源ABA及完整G01–G12仍待；8／60秒、Release／G11預算不變，相鄰只讀且產物獨立。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
        x['current_native8CC5_audio_Release']=record;x['current_native8CC5_audio_cases']=cases
        x['current_native8CC5_audio_cohort_active'].update(status='parent_failed_Debug_readiness_Release_children_qualified_only',original_terminal_exit_code=1,Release_qualification=str(qp),Release_qualification_sha256=sha(qp),whole_cohort_qualified=False)
        x.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='Debug-first-Presentation-readiness-profiling',work=work,applies_to_current_source=True,goal_complete=False)
        if 'current_source_goal_validation' in x:x['current_source_goal_validation']['G06-room-service-branches']=dict(state='controller_both_profiles_and_Release_audio_qualified_Debug_readiness_failed_GUI_and_others_pending',complete=False,evidence=record)
        for item in x.get('items',[]):
            if item.get('id','').startswith('G06-'):
                item.update(state='controller_both_profiles_Release_Presentation_audio_qualified_Debug_readiness_failed',complete=False,current_evidence=record,current_source_manifest_sha256=q['source_manifest_sha256'])
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':
                        step['retained_before_Release_audio_Presentation_evidence']=step.get('current_Presentation_evidence')
                        step['retained_before_Release_audio_failure_Presentation_evidence']=step.get('current_failure_Presentation_evidence')
                        step.update(state='controller_both_profiles_Release_Presentation_audio_qualified_Debug_readiness_failed',
                            current_Presentation_evidence=record,current_failure_Presentation_evidence=record,current_source_manifest_sha256=q['source_manifest_sha256'],
                            current_Presentation_qualified_profiles=['Release'],current_failure_Presentation_qualified_profiles=['Release'],
                            current_Presentation_verified_for_source=False,current_failure_Presentation_verified_for_source=False,
                            current_source_revalidation_note=work,unplayed_source_audio_rejection_verified=False,unplayed_source_audio_qualified_profiles=['Release'],
                            current_GUI_defined_scope_verified=False,
                            remaining='Release六分支診斷與非空音效已驗；Debug初始Frame就緒失敗須定位。正常GUI／實體輸入音效、同端點、普通WORLD／其他服務、當版旅館、同World新Model／來源ABA及完整G06.3d仍待。')
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep
        p.write_text('2026-10-18：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
for folder in [parent.parent,V/'native8CC5-teleport-audio']:
    path=folder/'current-continuation.json';previous=read(path);(out/(folder.name+'-current-before.json')).write_bytes(path.read_bytes())
    previous.update(status='Release_six_branches_qualified_only_Debug_initial_readiness_failed_parent_exit1',
        Release_qualification=str(qp),Release_qualification_sha256=sha(qp),qualified_cases=cases,whole_cohort_qualified=False,
        original_terminal_exit_code=1,must_poll_original_session_before_restarting=False,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),
        next_action='Profile Debug initial scene scheduling/source wait/load/first draw under the unchanged 8-second bound. Preserve all six Release qualifications; do not rerun them or claim both profiles passed.')
    save(path,previous)
print(json.dumps(dict(checkpoint=str(out),qualified_profile='Release',cases=6,sessions=48,parent_exit_code=1,Debug_qualified=False,whole_goal_complete=False),ensure_ascii=False))
