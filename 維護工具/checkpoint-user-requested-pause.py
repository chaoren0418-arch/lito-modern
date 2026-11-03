"""Record the user's explicit pause after the original bounded batch has ended."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert not w['active_workloads']and not w['active_workload_sessions']and not w['source_and_corresponding_QA_frozen']and w['current_full_default_library_verified']
lib=w['current_native_catalog_library'];assert lib['source_manifest_sha256']==source and sha(lib['completion'])==lib['completion_sha256']
for key in ['current_native_installed_resource_profile_normal_qualification','current_native_library_resource_optin','current_native_library_shared_resource_optin','current_native_rental_return_reconnect_focused']:
    proof=w[key];assert proof['source_manifest_sha256']==source and sha(proof['qualification'])==proof['qualification_sha256']
inputs=read(Path(lib['run'])/'compiled-inputs.json');assert len(inputs)==1049 and all(sha(R/n)==h for n,h in inputs.items())
normal=w['current_native_installed_resource_profile_normal_qualification'];next_action='依使用者要求暫停；沒有活動建置／測試。恢復後保留A695當版正常19、完整library兩版1974／0／488及互斥81資源兩版資格，不重跑。先以'+normal['run']+'執行正式返回API16，沿原session封存核對；再用'+lib['run']+'重備當版GUI fixture，實驗旅館數量Enter及EOF清理，接續普通房間／入口兩版及其餘GUI。控制器96／廣域Release、37技能缺口與G01–G12仍待，不交人工。'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=V/'user-requested-pause'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
record=dict(status='paused_by_explicit_user_request',source_manifest_sha256=source,user_request='告一段落先暫停,先更新進度報告',paused_utc=datetime.now(timezone.utc).isoformat(),active_workloads={},active_workload_sessions={},latest_bounded_batch_session=61460,latest_bounded_batch_terminal_chunk='d30bba',latest_bounded_batch_exit_code=0,normal_qualification=normal,library=lib,resource10=w['current_native_library_resource_optin'],resource71=w['current_native_library_shared_resource_optin'],return_cancel10=w['current_native_rental_return_reconnect_focused'],resume_next_action=next_action,current_source_API16_qualified=False,current_source_GUI_runtime_qualified=False,human_acceptance_ready=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert set(z.namelist())==set(before)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);assert j['current_product_identity']['source_manifest_sha256']==source
        j.update(development_state='paused_by_user',user_requested_pause=record,next_task=next_action,next_focus=next_action,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='paused_by_user',goal_complete=False,whole_goal_complete=False)
        j.setdefault('current_user_requested_goal',{}).update(status='paused_by_user',paused_utc=record['paused_utc'],human_login_required=False,whole_goal_complete=False)
        j['current_focus']=dict(item='paused_by_user',work=next_action,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=next_action
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+next_action+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())and all(sha(R/n)==h for n,h in inputs.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),state='paused_by_user',active_sessions={},source_manifest_sha256=source,whole_goal_complete=False),ensure_ascii=False))
