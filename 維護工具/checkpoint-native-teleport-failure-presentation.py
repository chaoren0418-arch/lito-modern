"""Record independent asynchronous failure cases without promoting partial profiles or GUI."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile,time
ROOT=Path(__file__).resolve().parents[1]
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text(encoding='utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('qualifications',nargs='+',type=Path);ap.add_argument('--active-run',type=Path);ap.add_argument('--active-session',type=int);a=ap.parse_args()
profiles={};source=None;probe=None;inputs=None;jobs=[];sessions=0
for qp in a.qualifications:
    qp=qp.resolve(strict=True);assert qp.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-failure-presentation');q=read(qp)
    assert q['status']=='independently_qualified_current_C02_Presentation_failure_diagnostic_scope' and q['recorded_owned_all_terminal'] and q['actual_sessions']==(10 if q['scenario']=='wrong-service' else 8)
    source=source or q['source_manifest_sha256'];assert source==q['source_manifest_sha256']
    run=Path(q['run']);r=read(run/'receipt.json');build=Path(r['compile']);b=read(build/'receipt.json');case=read(Path(r['case'])/'receipt.json')
    assert r['owned_process'] is None and all(j['exit_code']==0 for j in r['process_history']);probe=probe or b['probe_sha256'];assert probe==b['probe_sha256']
    current=read(build/'compiled-inputs.json');inputs=inputs or current;assert inputs==current
    cases=profiles.setdefault(q['profile'],{});assert q['scenario'] not in cases
    cases[q['scenario']]=dict(qualification=str(qp),qualification_sha256=sha(qp),actual_sessions=q['actual_sessions'],actual_product_sha256=q['actual_product_sha256'],original_session_id=q['original_session_id'],terminal_tool_chunk=q['terminal_tool_chunk'],retained_source_cues=q['retained_source_cues'],all_item_fields_exact=all(v['all_item_fields_exact'] for v in q['inventory_all_fields']))
    sessions+=q['actual_sessions'];jobs+=r['process_history']+[read(run/'runner-identity.json'),b['compiler']]+[dict(pid=j['pid'],creation_utc=j['start_utc']) for j in case['owned_processes']]
for n,h in inputs.items():assert sha(ROOT/n)==h,n
identities={(j['pid'],j['creation_utc']) for j in jobs};ids=','.join(str(pid) for pid in sorted({pid for pid,_ in identities}))
ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
output=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(output.decode('utf-8-sig')) if output.strip() else [];live=[live] if isinstance(live,dict) else live
for row in live:
    for pid,born in identities:
        if row['ProcessId']==pid:assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(born)).total_seconds())>.001
active=None
if a.active_run:
    assert a.active_session is not None;run=a.active_run.resolve(strict=True);assert run.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-failure-presentation');r=read(run/'receipt.json');driver=read(run/'runner-identity.json');assert r['source_manifest_sha256']==source
    ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(driver['pid'])+"'|Select-Object ProcessId,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    row=json.loads(subprocess.check_output(['pwsh','-NoProfile','-Command',ps]).decode('utf-8-sig'));assert row['ProcessId']==driver['pid'] and str(ROOT).casefold() in row['CommandLine'].casefold()
    assert abs((datetime.fromisoformat(row['created'].replace('Z','+00:00'))-datetime.fromisoformat(driver['creation_utc'])).total_seconds())<.001
    active=dict(run=str(run),original_session_id=a.active_session,profile=r['profile'],scenario=r['scenario'],driver=driver,CIM=row,source_manifest_sha256=source,verified_live_utc=datetime.now(timezone.utc).isoformat(),qualification_pending=True)
all_scenarios={'timeout','refused','wrong-service','cancel-retry','cancel-expiry'};complete=set(profiles)=={'Debug','Release'} and all(set(v)==all_scenarios for v in profiles.values())
if complete:assert sessions==84 and active is None
record=dict(source_manifest_sha256=source,probe_sha256=probe,profiles=profiles,qualified_cases=sum(map(len,profiles.values())),actual_sessions=sessions,both_profiles_all_five_verified=complete,recorded_owned_identities=len(identities),recorded_owned_all_terminal=True,scope='production-library persistent asynchronous Presentation through the listed real C02 failures, fresh LOGIN/GAME and same-map500 reentry only',unplayed_source_audio_rejection_verified=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:assert read(ROOT/n)['current_product_identity']['source_manifest_sha256']==source
out=ROOT/'驗證紀錄/current/native-teleport-failure-presentation'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='來源'+source[:8]+'原G06控制器兩版96sessions及非同步成功交接兩版12sessions維持。新增非同步失敗恢復已獨立核對'+str(record['qualified_cases'])+'案／'+str(sessions)+'sessions（'+'；'.join(p+':'+','.join(v) for p,v in profiles.items())+'）：失敗時frame撤下與worker退役、舊input surface拒絕，重新登入同map500後新場景與權威移動、S7C物品完整欄位一致。'
if active:work+='原'+str(a.active_session)+'繼續'+active['profile']+'/'+active['scenario']+'，未執行及未核對分支不列通過；來源／QA凍結。'
elif complete:work+='兩版五種失敗恢復均取得此診斷範圍資格；下一步仍需正常GUI／實體輸入及音效事件跨交接等必要範圍。'
else:work+='其餘profile／案例仍待，未宣告整批完成。'
work+='無來源音效事件的案例不作未播放音效退役或實體裝置資格。正常GUI／實體輸入音效、同端點、普通WORLD／旅館／其他服務、完整G06及G01–G12／G11G12仍待；登入器及素材唯讀、產物獨立，8秒／60秒／自然到期及全部預算維持。'
for n in names[:5]:
    p=ROOT/n;j=read(p);j.update(current_native_G06_failure_presentation=record,current_native_G06_failure_presentation_active=active,active_workloads={'G06-Presentation-failure':active} if active else {},active_workload_sessions={'G06-Presentation-failure':a.active_session} if active else {},source_and_corresponding_QA_frozen=active is not None,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-asynchronous-failure-recovery',work=work,goal_complete=False)
    if 'current_source_goal_validation' in j:j['current_source_goal_validation']['G06-room-service-branches']=dict(state='current_controller_Presentation_failure_scopes_qualified_full_G06_pending',complete=False,evidence=record)
    for item in j.get('items',[]):
        if item.get('id','').startswith('G06'):
            item.update(state='current_controller_Presentation_failure_scopes_qualified_full_G06_pending',complete=False,current_failure_Presentation_evidence=record)
            for step in item.get('rental_acceptance_steps',[]):
                if step.get('id')=='G06.3d':
                    step['current_failure_Presentation_evidence']=record
                    if complete:step['remaining']='當版兩版控制器與非同步成功／五種失敗恢復已取得各自診斷資格；未播放音效跨交接、正常GUI／實體輸入、同端點、普通WORLD／其他服務及當版旅館仍待，完整G06.3d未完成。'
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for n in names[:5]:assert read(ROOT/n)['current_native_G06_failure_presentation']==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'qualified-process-CIM.json').write_bytes(output);save(out/'readback.json',dict(record=record,active=active,before=before,after={n:sha(ROOT/n) for n in names},before_archive_sha256=sha(out/'before.zip')))
print(json.dumps(dict(checkpoint=str(out),qualified_cases=record['qualified_cases'],actual_sessions=sessions,both_profiles_complete=complete,active=active is not None,whole_G06_complete=False),ensure_ascii=False))
