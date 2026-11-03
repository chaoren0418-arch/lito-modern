"""Register current hotel scope and the concrete, visible login awaiting the human."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
run=V/'native44FE-hotel/20261018-172052-514017';qp=run/'qualified/qualified.json';q=read(qp)
assert q['status']=='independently_qualified_current_normal_all_three_hotel_rooms_both_profiles'
assert q['actual_sessions']==16 and q['actual_cases']==4 and q['maps']==[501,502,503] and q['recorded_owned_all_terminal'] and q['recorded_owned_identities']==18
assert q['original_session_id']==6109 and q['original_exit_code']==0 and q['terminal_tool_chunk']=='05797b'
assert sha(qp.parent/'raw.zip')==q['raw_archive_sha256']
with zipfile.ZipFile(qp.parent/'raw.zip')as z:
    assert set(z.namelist())==set(q['raw_files'])
    for n,h in q['raw_files'].items():
        with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
for n,h in read(run/'source-inputs.json').items():assert sha(ROOT/n)==h,n
gui=V/'native44FE-interactive-teleport/20261018-172654-846629'
g=read(gui/'receipt.json');launch=read(gui/'normal-launch-a/receipt.json')
assert g['status']=='ready_for_interactive_OS_SendInput' and g['owned_process']['pid']==56116
assert launch['status']=='normal_GUI_process_started_not_yet_accepted' and launch['normal_process']['ProcessId']==39576
assert launch['endpoint']=='127.0.0.1:56044' and launch['normal_product_sha256']==q['normal_mains']['Release']
assert g['source_manifest_sha256']==q['source_manifest_sha256'] and g['jar_sha256']==q['reference_jar_sha256']
previous=V/'native44FE-same-endpoint/20261018-171204-385221-debug/checkpoint-20261018-171851-696032/readback.json'
before=read(previous)['canonical_after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record={k:q[k]for k in ['status','source_manifest_sha256','reference_jar_sha256','normal_mains','original_session_id','original_exit_code','terminal_tool_chunk','actual_cases','actual_sessions','maps','original_world_PNGs','exact_S5A','exact_C08','recorded_owned_identities','recorded_owned_all_terminal','retained_without_rerun','initial_reader_failure','GUI_verified','physical_input_verified','physical_audio_verified','shared_keys_verified','natural_expiry_verified','manual_exit_verified']}
record.update(qualification=str(qp),qualification_sha256=sha(qp),raw_archive_sha256=q['raw_archive_sha256'],qualification_reader_original_session_id=56570,qualification_reader_exit_code=0,qualification_reader_terminal_chunk='304296',whole_goal_complete=False)
active=dict(status='current_normal_Release_login_raised_awaiting_user_authentication',controller=str(gui),controller_original_session_id=58583,
    server_identity=g['owned_process'],normal_launch_receipt=str(gui/'normal-launch-a/receipt.json'),normal_launch_receipt_sha256=sha(gui/'normal-launch-a/receipt.json'),
    normal_process=launch['normal_process'],endpoint=launch['endpoint'],title='LiTo 原生登入',returned_window_id=986460,
    sky_accessibility_readback='Endpoint, Connect button and account/password controls observed after successful activation; credentials not entered.',
    screenshot_capture_error='FrameArrived timed out: timed out waiting on channel',accessibility_recovery_succeeded=True,
    authentication_automated=False,GUI_acceptance=False,physical_input_verified=False,source_manifest_sha256=q['source_manifest_sha256'],whole_goal_complete=False)
save(gui/'visible-login-readback.json',active)
save(out/'evidence.json',dict(current_hotel_rooms=record,visible_login=active))
work='當版44FEA738／75DA69B0正常Debug與Release旅館兩分支已獨立通過：4案16sessions、501／502／503三房、8次S5A及C08、24張原PNG與18組記錄程序身分終止；鑰匙／位置／世代／權威與原S32預算核對。Debug外層CP950讀取失敗原19083exit1保留，其實際案0未重跑；續跑原6109exit0。此資格不含GUI、共享鑰匙、自然到期或手動退出。同端點當版頻道契約失敗保留，兩版23場景與控制器／音效各96sessions維持各自範圍。已由唯一入口重開當版Release原生登入並帶前景，端點127.0.0.1:56044，原58583隔離服務等待使用者自行帳密；未操作認證或授予GUI資格。登入器產物唯讀且獨立。普通WORLD／其他服務、必要廣域／完整library、有效不同資源資料集ABA、正常GUI／實體輸入音效與完整G06／G01–G12仍待；既有時限與G11預算不變。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    p=ROOT/n
    if p.suffix=='.json':
        x=read(p);assert x['current_product_identity']['source_manifest_sha256']==q['source_manifest_sha256']
        x.update(current_native44FE_hotel_rooms=record,current_visible_login=active,next_task=work,next_focus=work,
            active_workloads={'current_normal_interactive_login':active},active_workload_sessions={'current_normal_interactive_login':58583},source_and_corresponding_QA_frozen=True,
            last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        if 'updated_date'in x:x['updated_date']=date
        x['current_focus']=dict(item='current-G06-normal-visible-login',work=work,applies_to_current_source=True,goal_complete=False)
        for item in x.get('items',[]):
            if item.get('id','').startswith('G06-'):
                item['current_hotel_room_entry_evidence']=record
                for step in item.get('rental_acceptance_steps',[]):
                    if step.get('id')=='G06.3d':
                        step['current_hotel_room_entry_evidence']=record
                        step['remaining']='當版旅館三房進入已驗；共享鑰匙／自然到期／手動退出仍待當版驗證。同端點當版頻道契約失敗未通過；正常GUI現等待人工登入。普通WORLD／其他服務、實體輸入音效、有效不同資料集ABA及完整G06.3d仍待。'
        if n.endswith('next-validation-action.json'):x['next_action']=work
        save(p,x)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert datetime.strptime(first[:10],'%Y-%m-%d')and sep
        p.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(status='hotel_room_entry_qualified_visible_login_awaiting_user',run=str(run),qualification=str(qp),qualification_sha256=sha(qp),latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),current_visible_login=active,whole_goal_complete=False))
print(json.dumps(dict(checkpoint=str(out),hotel_sessions=16,visible_login_endpoint=launch['endpoint'],GUI_acceptance=False),ensure_ascii=False))
