"""Resume only after the user's explicit instruction, preserving the prior pause."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert w['development_state']=='paused_by_user'and not w['active_workloads']and not w['active_workload_sessions']and not w['source_and_corresponding_QA_frozen']
pause=w['user_requested_pause'];assert pause['status']=='paused_by_explicit_user_request'and pause['source_manifest_sha256']==source
library=w['current_native_catalog_library'];assert w['current_full_default_library_verified']and library['source_manifest_sha256']==source and sha(library['completion'])==library['completion_sha256']
normal=w['current_native_installed_resource_profile_normal_qualification']
for proof in [normal,w['current_native_library_resource_optin'],w['current_native_library_shared_resource_optin'],w['current_native_rental_return_reconnect_focused']]:assert proof['source_manifest_sha256']==source and sha(proof['qualification'])==proof['qualification_sha256']
inputs=read(Path(library['run'])/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
for info in read(Path(normal['run'])/'cohort.json')['profiles'].values():assert sha(info['normal']['path'])==info['normal']['sha256']
objective='依總報告逐項完成真正Windows x64重建版剩餘開發與G01–G12：保留A695當版正常19、完整library兩版1974／0／488及額外81資源成果；先正式返回API16，再正常GUI正／負向全流程與修復，續37技能缺口、控制器96、同端點、廣域Release與可攜交付稽核；所有必要自動驗證完成後才交人工，持續更新總報告，不要求使用者登入或操作測試視窗。'
work='依使用者繼續指示恢復工作。'+objective+' 下一批run-native-rental-return-reconnect-api.py輸入正常批次：'+normal['run']+'；完成沿原session核對後，以library批次'+library['run']+'重備GUI fixture。'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'user-requested-resume'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='resumed_by_explicit_user_request',source_manifest_sha256=source,user_request='依照報告進度繼續完成工作；重新設定目標；依照報告內容逐一完成並更新報告',resumed_utc=datetime.now(timezone.utc).isoformat(),objective=objective,prior_pause=pause,system_goal_reset_attempt='create_goal rejected: unfinished goal exists; existing goal was not falsely completed',project_goal_status='active',ordered_work=['formal return API16','current normal GUI positive and negative flows','remaining skills/services/lifecycle','current-source controller and broad Release regression','portable delivery and audit','human acceptance after verified automatic tests'],current_source_normal_library_and_81_resource_qualifications_retained=True,normal_run=normal['run'],library_run=library['run'],whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert set(z.namelist())==set(before)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==source
        j.update(development_state='active_by_user_resume',pause_is_current=False,last_user_pause=pause,user_requested_resume=record,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        j['current_user_requested_goal']=dict(status='active',objective=objective,ordered_work=record['ordered_work'],resumed_utc=record['resumed_utc'],human_login_required=False,whole_goal_complete=False)
        j['current_focus']=dict(item='formal_return_API_then_normal_GUI',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),project_goal_status='active',source_manifest_sha256=source,active_sessions={},whole_goal_complete=False),ensure_ascii=False))
