"""Register the exact current controller cohort; retain GUI, Presentation and other G06 gates."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native1047-G06-controller')and a.terminal_chunk.lower()not in['','pending','unknown']
c=read(run/'cohort.json');qp=run/'qualified/qualified.json';q=read(qp);op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases'and c['owned_process']is None
assert q['status']=='independently_qualified_both_current_G06_controller_profiles_six_cases_each'and(q['actual_cases'],q['actual_formal_sessions'],q['actual_test_executables'])==(12,96,4)
assert q['original_session_id']==o['original_session_id']==a.session_id and q['observed_terminal_exit_code']==o['observed_terminal_exit_code']==0 and o['recorded_original_owned_processes_all_terminal']and o['qualification_sha256']==sha(qp)
assert c['source_manifest_sha256']==q['source_manifest_sha256']==o['source_manifest_sha256']
for n,h in read(run/'source-inputs.json').items():assert sha(ROOT/n)==h,n
normal=read(c['startup_qualification']);assert all(v['passed']==31 for v in normal['profiles'].values())and normal['source_manifest_sha256']==c['source_manifest_sha256']
skill_path=run/'current-skill-request-scope.json';skill_scope=read(skill_path)
assert skill_scope['source_manifest_sha256']==c['source_manifest_sha256']and skill_scope['request_recognized_count']==skill_scope['missing_request_count']==37
assert skill_scope['exact_request_body_delta_only_skills']==[26,42]and skill_scope['normal_ability_actual_sessions']==8 and skill_scope['recognition_is_not_successful_cast_count']
before=read(run/'active-checkpoint/readback.json')['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-completed-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='current_controller_six_cases_each_profile_independently_qualified',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),completion_observation=str(op),completion_observation_sha256=sha(op),source_manifest_sha256=c['source_manifest_sha256'],reference_jar_sha256=c['reference_jar_sha256'],normal_mains=c['normal_mains'],original_session_id=a.session_id,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,actual_cases=12,actual_sessions=96,actual_test_executables=4,recorded_owned_identities=o['recorded_original_owned_identities'],recorded_owned_all_terminal=True,qualified=True,scope='cross-endpoint,timeout,refused,wrong-service,cancel-retry,cancel-expiry controller tests in Debug and Release',current_Presentation_verified_for_source=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record);work='來源'+c['source_manifest_sha256'][:8]+'及'+c['reference_jar_sha256'][:8]+'參考之G06原'+str(a.session_id)+'已actual exit0，兩版六分支12案／96 LOGIN-GAME sessions、四測試映像及'+str(o['recorded_original_owned_identities'])+'記錄程序身分終止已逐案及整批核對。正常兩版各31相關案、11原資源特效與能力雙玩家8sessions維持各自範圍。下一步當版廣域116目標／完整library，再驗Presentation傳送非空音效及必要正常可見性。44FE較早WORLD／旅館等證據只保留原版，同端點75契約失敗保留。8秒／60秒／Release與G11預算不變，相鄰只讀且登入器產物獨立；開發期間不請使用者登入，正常GUI／實體音效及完整G01–G12未完成。'
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        j=read(p);assert j['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
        old=j.get('current_native_G06_controller')
        if old and old.get('source_manifest_sha256')!=c['source_manifest_sha256']:j.setdefault('retained_pre_ordinary_admission_G06_controller',old)
        j.update(current_native_G06_controller=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,candidate_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        j['current_native1040_G06_active'].update(status='completed_qualified_controller_scope',original_exit_code=0,terminal_tool_chunk=a.terminal_chunk)
        for item in j.get('items',[]):
            if item['id'].startswith('G05'):
                for key in ['current_skill_development_audit','current_ability_development']:
                    if key in item:item.setdefault('retained_44FE_'+key,item[key])
                item.update(state='book74_request37_missing37_two_normal_ability_scopes_qualified_remaining_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_source_evidence_scope='74 IDs from the unchanged75 reference book audit; exact request-body delta adds26/42. Recognition37 is not37 successful spells. Two normal ability scopes actually executed8sessions.',current_skill_development_audit=dict(**skill_scope,evidence=str(skill_path),evidence_sha256=sha(skill_path)),current_ability_development=dict(status='normal_two_profile_ability_self_and_rejection_scopes_qualified',source_manifest_sha256=c['source_manifest_sha256'],skills=[26,42],missing_normal_opcodes=[],normal_receiver_S6B_S6C_verified=True,actual_normal_sessions=8,actual_self_casts=4,scope_evidence=str(skill_path),scope_evidence_sha256=sha(skill_path),all_gameplay_verified=False,GUI_verified=False,whole_goal_complete=False))
            if item['id'].startswith('G06'):
                item.update(state='current_controller_qualified_Presentation_and_other_branches_pending',complete=False,current_source_manifest_sha256=c['source_manifest_sha256'],current_evidence=record)
                for step in item.get('rental_acceptance_steps',[]):
                    if step['id']=='G06.3d':step.update(state='current_controller_qualified_Presentation_and_other_branches_pending',current_source_manifest_sha256=c['source_manifest_sha256'],current_controller_evidence=record,current_controller_six_both_profiles_evidence=str(qp),current_source_revalidation_note=work,current_Presentation_verified_for_source=False,current_failure_Presentation_verified_for_source=False,current_GUI_defined_scope_verified=False,current_TW_GUI_defined_scope_verified=False,remaining=work)
        j['current_focus']=dict(item='current-Presentation-nonempty-audio-handoff',work=work,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        save(p,j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep;p.write_text((datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,status_detail='completed_controller_scope_only',checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_session_before_restarting=False,next_action=work))
print(json.dumps(dict(checkpoint=str(out),cases=12,sessions=96,whole_G06_complete=False),ensure_ascii=False))
