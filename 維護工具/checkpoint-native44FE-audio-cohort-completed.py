"""Register only the fully read-back normal-body diagnostic audio/Presentation scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):Path(p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-audio-cohort')
c=read(run/'cohort.json');qp=run/'qualified/qualified.json';q=read(qp)
assert c['status']=='executed_current_normal_body_audio_cohort_qualification_pending' and c['owned_process'] is None
assert q['status']=='independently_qualified_current_normal_bodies_nonempty_audio_both_profiles_six_branches'
assert q['original_session_id']==a.session and q['original_exit_code']==0 and q['terminal_tool_chunk']==a.terminal_chunk
assert(q['actual_cases'],q['actual_sessions'],q['actual_diagnostic_executables'],q['retained_source_cues'],q['actual_C1F_wave_requests'])==(12,96,4,24,44)
assert q['new_scene_sound_events_delivered_once']==24 and q['recorded_original_owned_processes_all_terminal'] and q['recorded_original_owned_identities']==69
assert q['source_manifest_sha256']==c['source_manifest_sha256'] and q['normal_source_bodies_unchanged'] and not q['source_overlays_used']
assert q['diagnostic_request_API_extension'] and not q['normal_GUI_API_permission_inferred'] and not q['normal_EXE_runtime_inferred']
assert sha(qp.parent/'cohort-raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'cohort-raw.zip') as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():
        with z.open(n) as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
cases={}
for key,pin in q['cases'].items():
    path=Path(pin['path']);item=read(path);assert sha(path)==pin['sha256'] and item['recorded_owned_all_terminal'] and item['unplayed_source_audio_rejection_verified']
    cases[key]=dict(run=item['run'],qualification=str(path),qualification_sha256=sha(path),source_manifest_sha256=item['source_manifest_sha256'],
        diagnostic_input_manifest_sha256=item['diagnostic_input_manifest_sha256'],profile=item['profile'],scenario=item['scenario'],
        original_session_id=item['original_session_id'],original_exit_code=0,terminal_tool_chunk=item['terminal_tool_chunk'],actual_sessions=item['actual_sessions'],
        actual_product_sha256=item['actual_product_sha256'],recorded_owned_all_terminal=True,normal_source_bodies_unchanged=True,source_overlays_used=False,
        diagnostic_request_API_extension=True,normal_GUI_API_permission_inferred=False,normal_product_source_modified=False,
        actual_Presentation_executed=True,unplayed_source_audio_rejection_verified=True,retained_source_cues=2,
        new_scene_own_sound_events_delivered_once_each=True,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,
        whole_G06_complete=False,whole_goal_complete=False,cohort=str(run),retained_without_rerun=c['cases'][key].get('retained_without_rerun',False))
scene_proofs={}
for profile in ['Debug','Release']:
    build=read(Path(c['builds'][profile])/'receipt.json');path=Path(build['normal_qualification']);scene=read(path)
    assert sha(path)==build['normal_qualification_sha256'] and scene['source_manifest_sha256']==q['source_manifest_sha256'] and scene['passed']==23 and scene['prior_scene20_preserved_byte_prefix'] and scene['new_generation_regressions']==3 and scene['recorded_owned_all_terminal']
    scene_proofs[profile]=dict(path=str(path),sha256=sha(path),scene_cases=23,new_generation_cases=3)
for n,h in read(run/'compiled-inputs.json').items():assert sha(ROOT/n)==h
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_normal_body_Presentation_nonempty_audio_both_profiles_six_branches_qualified_limited',run=str(run),
    qualification=str(qp),qualification_sha256=sha(qp),source_manifest_sha256=q['source_manifest_sha256'],diagnostic_input_manifest_sha256=q['diagnostic_input_manifest_sha256'],
    original_session_id=a.session,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,normal_mains=q['normal_mains'],actual_cases=12,
    actual_sessions=96,actual_diagnostic_executables=4,cases=q['cases'],retained_qualified_cases_without_rerun=1,recorded_owned_identities=69,recorded_owned_all_terminal=True,
    actual_Presentation_executed=True,unplayed_source_audio_rejection_verified=True,retained_source_cues=24,old_sound_deliveries=0,
    new_scene_sound_events_delivered_once=24,actual_C1F_wave_requests=44,recovered_ordinary_wave_deliveries=20,destination_native_heal_deliveries=4,
    source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_GUI_API_permission_inferred=False,
    normal_product_source_modified=False,same_World_new_Model_event_admission_regressions_verified_in_separate_scene_suite=True,valid_distinct_resource_source_ABA_verified=False,normal_EXE_runtime_inferred=False,
    GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
record['separate_normal_scene23_qualifications']=scene_proofs
save(out/'evidence.json',record)
work=f'來源44FEA738正常模組Presentation與非空音效兩版六分支原{a.session}已實際退出0並獨立核對：12案／96sessions、44個真實C1F／S20，24個舊待播cue皆Stale／重複AlreadyConsumed且零交付，20個恢復後一般音效與4個目的地heal109各交付一次；69組記錄程序身分皆終止。保留Debug拒絕資格而不重跑。只授予正常模組本體加隔離API、診斷Player與原PNG範圍，正常EXE GUI／實體音效不推定。當版正常兩版23資源場景與控制器96sessions維持各自範圍；S4A／角色動作同World新Model、無效來源ABA與新世代Model隔離由23場景套件證明，不移記為本cohort自身的測試。下一步先核對當版同端點、旅館與普通WORLD／其他服務分支，並續做必要廣域／完整library與正常可見性。8／60秒、Release／G11界限不變，相鄰只讀且產物獨立，完整G06／G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
        x['retained_pre_cohort_audio_case_records']=x.get('current_native44FE_audio_cases',{})
        x['current_native44FE_audio_cases']=cases;x['current_native44FE_audio_cohort']=record
        x['current_native44FE_audio_cohort_active'].update(status='completed_qualified_nonempty_audio_and_Presentation_scope',original_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,qualification=str(qp),qualification_sha256=sha(qp))
        x.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,next_task=work,next_focus=work,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        x['current_focus']=dict(item='current-G06-same-endpoint-hotel-WORLD-remaining-scope',work=work,applies_to_current_source=True,goal_complete=False)
        if 'current_source_goal_validation' in x:x['current_source_goal_validation']['G06-room-service-branches']=dict(state='current_controller_Presentation_nonempty_audio_qualified_GUI_and_other_branches_pending',complete=False,evidence=record)
        for item in x.get('items',[]):
            if item.get('id','').startswith('G06-'):
                item.update(state='current_controller_Presentation_nonempty_audio_qualified_GUI_and_other_branches_pending',complete=False,current_evidence=record,current_source_manifest_sha256=q['source_manifest_sha256'])
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':
                        step['retained_before_current_audio_Presentation_evidence']=step.get('current_Presentation_evidence')
                        step['retained_before_current_audio_failure_Presentation_evidence']=step.get('current_failure_Presentation_evidence')
                        step.update(state='current_controller_Presentation_nonempty_audio_qualified_GUI_and_other_branches_pending',
                            current_Presentation_evidence=record,current_failure_Presentation_evidence=record,current_source_manifest_sha256=q['source_manifest_sha256'],
                            current_Presentation_verified_for_source=True,current_failure_Presentation_verified_for_source=True,
                            current_source_revalidation_note=work,unplayed_source_audio_rejection_verified=True,current_GUI_defined_scope_verified=False,
                            remaining='正常模組診斷六分支與非空待播音效已驗；正常GUI／實體輸入音效、同端點、普通WORLD／其他服務、當版旅館、有效不同資源資料集ABA及完整G06.3d仍待。')
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d') and sep
        p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
for folder in [run.parent,V/'native44FE-teleport-audio']:
    path=folder/'current-continuation.json';previous=read(path);(out/(folder.name+'-current-before.json')).write_bytes(path.read_bytes())
    previous.update(status='completed_qualified_current_normal_body_audio_and_Presentation_scope',qualification=str(qp),qualification_sha256=sha(qp),
        original_session_id=a.session,original_terminal_exit_code=0,terminal_tool_chunk=a.terminal_chunk,must_poll_original_session_before_restarting=False,
        latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),qualified_cases=cases,
        next_action='Current-source same-endpoint, hotel, WORLD/other services, required broad/full-library and normal visibility; physical input/audio and valid distinct resource-source ABA remain pending.')
    save(path,previous)
print(json.dumps(dict(checkpoint=str(out),actual_cases=12,actual_sessions=96,old_cues_rejected=24,new_events_delivered_once=24,whole_goal_complete=False),ensure_ascii=False))
