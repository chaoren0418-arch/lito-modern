"""Record the completed current G06 controller scope; GUI and other gates remain open."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session-id',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--audit',type=Path,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native1040-G06-controller')
c=read(run/'cohort.json');qpath=run/'qualified/qualified.json';q=read(qpath);op=run/'completion-observation.json';o=read(op)
assert c['status']=='executed_current_both_profiles_G06_controller_TW_extended_six_cases' and c['owned_process'] is None
assert q['status']=='independently_qualified_both_current_G06_controller_profiles_six_cases_each' and q['source_files']==1040 and q['actual_cases']==12 and q['actual_formal_sessions']==96 and q['actual_test_executables']==4
assert q['original_session_id']==o['original_session_id']==a.session_id and q['observed_terminal_exit_code']==o['observed_terminal_exit_code']==0
assert o['recorded_original_owned_processes_all_terminal'] and o['qualification_sha256']==sha(qpath)
assert q['source_manifest_sha256']==c['source_manifest_sha256']==o['source_manifest_sha256']
inputs=read(run/'source-inputs.json');assert len(inputs)==1040 and sha(run/'source-inputs.json')==c['source_manifest_sha256']
audit_path=a.audit.resolve(strict=True);assert audit_path.is_relative_to(ROOT/'驗證紀錄/current/native1040-learned-request-audit')
audit=read(audit_path);assert audit['source_manifest_sha256']==c['source_manifest_sha256'] and audit['observed_learned_ids']==74 and audit['recognized_learned_requests']==35 and audit['missing_learned_requests']==39
for n,h in inputs.items():assert sha(ROOT/n)==h,n
for p,h in c['normal_mains'].items():assert sha(read(Path(c['startup_prerequisite'])/'cohort.json')['profiles'][p]['normal']['path'])==h
assert a.terminal_chunk not in ['','pending','unknown']
prior_readback=read(run/'active-checkpoint/readback.json')['after_files'];assert all(sha(ROOT/n)==h for n,h in prior_readback.items())
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:assert read(ROOT/n)['current_product_identity']['source_manifest_sha256']==c['source_manifest_sha256']
out=run/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(run=str(run),qualification=str(qpath),qualification_sha256=sha(qpath),completion_observation=str(op),completion_observation_sha256=sha(op),original_session_id=a.session_id,original_exit_code=0,terminal_tool_chunk=a.terminal_chunk,source_manifest_sha256=c['source_manifest_sha256'],normal_mains=c['normal_mains'],actual_test_executables=4,actual_cases=12,actual_sessions=96,recorded_owned_identities=o['recorded_original_owned_identities'],recorded_owned_all_terminal=True,scope='cross-endpoint,timeout,refused,wrong-service,cancel-retry,cancel-expiry controller tests in both profiles only',qualified=True,GUI_verified=False,physical_input_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record)
work='來源'+c['source_manifest_sha256'][:8]+' G06原'+str(a.session_id)+'已actual exit0；正常兩主與四個實際測試映像綁定，兩版六分支12案／96 LOGIN-GAME sessions之封包、角色與物品全欄接續已獨立核對，'+str(o['recorded_original_owned_identities'])+'組記錄身分皆終止。只通過此控制器範圍，接續當版非同步Presentation跨端點與失敗退役診斷、冰錐實際C02，再廣域／完整library及正常GUI。正常兩版各130相關檢查維持各自資格；舊版本及候選結果不移記。74項既有實際技能書僅提供wire證據，新來源靜態請求辨識35／缺39，非玩法完成。8秒／60秒、G11預算及Release設定不變；正常GUI／實體輸入音效、同端點／其他服務、完整G06及G01–G12仍待。重建版與登入器產物獨立，相鄰來源只讀。'
for n in names[:5]:
    p=ROOT/n;j=read(p)
    prior=j.get('current_native_G06_controller')
    if prior and prior.get('source_manifest_sha256')!=c['source_manifest_sha256']:j.setdefault('retained_native_G06_controller_before_8CC5',prior)
    j['current_native1040_G06_active'].update(status='completed_qualified_controller_scope',completion_observation=str(op),completion_observation_sha256=sha(op),original_exit_code=0,terminal_tool_chunk=a.terminal_chunk)
    j['current_native1040_learned_request_audit']=dict(path=str(audit_path),sha256=sha(audit_path),source_manifest_sha256=c['source_manifest_sha256'],observed_learned_ids=74,recognized_learned_requests=35,missing_learned_requests=39,request_recognition_is_not_gameplay_acceptance=True,retained_E159_book_wire_only=True,new_casts=0,whole_goal_complete=False)
    if 'current_native_audio_reconnect' in j:j['current_native_audio_reconnect'].update(applies_to_current_source=False,retained_source_version_only=True)
    j.update(current_native_G06_controller=record,active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=True,source_freeze_for_current_G06_controller=dict(frozen=True,run=str(run),source_manifest_sha256=c['source_manifest_sha256']),source_freeze_while_Cargo=False,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if 'current_source_goal_validation' in j:j['current_source_goal_validation']['G06-room-service-branches']=dict(state='current1040_controller_qualified_GUI_and_other_G06_pending',complete=False,evidence=record)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-actual-Presentation-handoff-diagnostic',work=work,goal_complete=False)
    for item in j.get('items',[]):
        if item.get('id','').startswith('G06'):
            item.update(state='current1040_controller_qualified_GUI_and_other_branches_pending',complete=False,current_evidence=record,current_source_manifest_sha256=c['source_manifest_sha256'],current_source_evidence_scope=record['scope'])
            for step in item.get('rental_acceptance_steps',[]):
                if step.get('id')=='G06.3d':
                    step['retained_pre_8CC5_source_revalidation_note']=step.get('current_source_revalidation_note')
                    for key in ['current_Presentation_evidence','current_failure_Presentation_evidence','current_GUI_scope']:
                        old=step.get(key)
                        if isinstance(old,dict) and old.get('source_manifest_sha256')!=c['source_manifest_sha256']:
                            old.update(applies_to_current_source=False,retained_source_version_only=True)
                    step.update(state='current1040_controller_qualified_GUI_and_other_branches_pending',current_controller_evidence=record,
                        current_source_manifest_sha256=c['source_manifest_sha256'],current_source_revalidation_note=work,
                        current_controller_six_both_profiles_evidence=str(qpath),
                        current_Presentation_verified_for_source=False,current_failure_Presentation_verified_for_source=False,
                        current_GUI_defined_scope_verified=False,current_TW_GUI_defined_scope_verified=False,
                        remaining='當版兩版跨端點與五種恢復控制器已驗；實際非同步Presentation／正常GUI／實體輸入音效、同端點、普通WORLD／其他服務及當版旅館仍待，完整G06.3d未完成。')
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for n in names[:5]:assert read(ROOT/n)['current_native_G06_controller']==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},before_archive_sha256=sha(out/'before.zip')))
save(run.parent/'current-continuation.json',dict(**record,status='completed_qualified_controller_scope',checkpoint=str(out),must_poll_original_session_before_restarting=False,next_action='Run exact-source asynchronous Presentation six branches and ordinary-source audio in both profiles; no GUI or physical acceptance inferred.'))
print(json.dumps(dict(checkpoint=str(out),actual_cases=12,actual_sessions=96,whole_G06_complete=False,whole_goal_complete=False),ensure_ascii=False))
