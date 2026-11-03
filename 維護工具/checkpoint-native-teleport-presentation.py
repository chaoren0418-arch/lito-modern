"""Publish actual asynchronous cross-endpoint scene scope without promoting GUI or audio gaps."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,subprocess,zipfile
ROOT=Path(__file__).resolve().parents[1]
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('qualifications',nargs='+',type=Path);ap.add_argument('--active-compile',type=Path);ap.add_argument('--active-session',type=int);a=ap.parse_args();profiles={};source=None;probe=None
for qp in a.qualifications:
    qp=qp.resolve(strict=True);assert qp.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-presentation');q=read(qp)
    assert q['status']=='independently_qualified_current_C02_Presentation_handoff_diagnostic_scope' and q['recorded_owned_all_terminal'] and q['actual_sessions']==6 and len(q['diagnostic_PNGs'])==4
    assert q['profile'] not in profiles;source=source or q['source_manifest_sha256'];assert source==q['source_manifest_sha256']
    run=Path(q['run']);r=read(run/'receipt.json');b=read(Path(r['compile'])/'receipt.json');probe=probe or b['probe_sha256'];assert probe==b['probe_sha256']
    for n,h in read(Path(r['compile'])/'compiled-inputs.json').items():assert sha(ROOT/n)==h,n
    profiles[q['profile']]=dict(qualification=str(qp),qualification_sha256=sha(qp),actual_sessions=6,actual_product_sha256=q['actual_product_sha256'],original_session_id=q['original_session_id'],terminal_tool_chunk=q['terminal_tool_chunk'],recorded_owned_identities=q['recorded_owned_identities'],recorded_owned_all_terminal=True,retained_source_cues=q['retained_source_cues'])
assert 'Debug' in profiles and set(profiles)<={'Debug','Release'}
active=None
if a.active_compile:
    assert a.active_session is not None;cp=a.active_compile.resolve(strict=True);assert cp.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-presentation');c=read(cp/'receipt.json')
    assert c['status']=='compiling_current_production_Presentation_test' and c['source_manifest_sha256']==source and c['probe_sha256']==probe and 'exit_code' not in c['compiler']
    j=c['compiler'];cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);Get-CimInstance Win32_Process -Filter 'ProcessId="+str(j['pid'])+"'|Select-Object ProcessId,CommandLine,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}}|ConvertTo-Json -Compress"
    output=subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]);live=json.loads(output.decode('utf-8-sig'));assert live['ProcessId']==j['pid']
    assert abs((datetime.fromisoformat(live['created'].replace('Z','+00:00'))-datetime.fromisoformat(j['creation_utc'])).total_seconds())<.001 and str(cp).casefold() in live['CommandLine'].casefold()
    active=dict(run=str(cp),original_session_id=a.active_session,phase='Release_diagnostic_compilation',source_manifest_sha256=source,observed_process=j,CIM=live,verified_live_utc=datetime.now(timezone.utc).isoformat(),qualification_pending=True)
failed=ROOT/'驗證紀錄/current/native-teleport-presentation/compile-20261017-202944-431538/failure-preserved.json';f=read(failed);assert f['original_exit_code']==1
record=dict(source_manifest_sha256=source,profiles=profiles,probe_sha256=probe,actual_sessions=6*len(profiles),scope='actual production-library asynchronous Presentation on successful cross-endpoint handoff only',failed_Release_diagnostic_link_attempt=dict(path=str(failed),sha256=sha(failed),original_exit_code=1,not_relabelled_as_pass=True),actual_Presentation_executed=True,normal_EXE_runtime_inferred=False,unplayed_source_audio_rejection_verified=False,GUI_verified=False,physical_input_verified=False,physical_audio_verified=False,same_endpoint_verified=False,whole_G06_complete=False,whole_goal_complete=False)
names=['工作佇列.json','監製進度.json','驗證紀錄/current/native-teleport-wire/progress.json','驗證紀錄/current/native-terrain-residence/continuation.json','驗證紀錄/current/native-presentation-source-owner/application-holy-list-20261017-060417-273807/next-validation-action.json','README.md','docs/ARCHITECTURE.zh-TW.md','docs/HISTORY.md']
for n in names[:5]:assert read(ROOT/n)['current_product_identity']['source_manifest_sha256']==source
out=ROOT/'驗證紀錄/current/native-teleport-presentation'/('tracker-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();before={n:sha(ROOT/n) for n in names}
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in names:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
scope='／'.join(profiles);work='來源'+source[:8]+' G06兩正常主及兩版六分支12案／96sessions控制器資格維持。新增'+scope+'產品library之實際非同步Presentation成功交接診斷已獨立核對，共'+str(record['actual_sessions'])+'sessions：來源500畫面、交接後frame撤下及worker quiescent、舊input surface拒絕、目的地564重新呈現，各版四張PNG與原封包／角色物品權威比對一致。'
if active:work+='Release診斷沿原'+str(a.active_session)+'以正常z／ThinLTO／codegen16／strip參數編譯中；先前缺少對應參數之連結失敗封存，產品來源未變。'
elif set(profiles)=={'Debug','Release'}:work+='兩版成功交接診斷均完成；下一步實際非同步失敗恢復及正常GUI驗收。'
else:work+='Release實際非同步場景診斷仍待。'
work+='此案來源音效事件數為0，未驗未播放音效退役或實體裝置；正常GUI／實體輸入、非同步失敗恢復、同端點、旅館／其他服務、完整G06與G01–G12／G11G12仍待。登入器及素材唯讀、產物獨立，原時限預算不變。'
for n in names[:5]:
    p=ROOT/n;j=read(p);j.update(current_native_G06_presentation=record,active_workloads={'G06-Presentation-Release-compilation':active} if active else {},active_workload_sessions={'G06-Presentation-Release-compilation':a.active_session} if active else {},source_and_corresponding_QA_frozen=active is not None,next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=record,progress_updated_utc=datetime.now(timezone.utc).isoformat(),goal_complete=False,whole_goal_complete=False)
    if n==names[4]:j['next_action']=work
    else:j['current_focus']=dict(item='G06-asynchronous-Presentation-handoff',work=work,goal_complete=False)
    if 'current_source_goal_validation' in j:j['current_source_goal_validation']['G06-room-service-branches']=dict(state='current_controller_and_scoped_Presentation_qualified_full_G06_pending',complete=False,evidence=record)
    for item in j.get('items',[]):
        if item.get('id','').startswith('G06'):
            item.update(state='current_controller_and_scoped_Presentation_qualified_full_G06_pending',complete=False,current_Presentation_evidence=record)
            for step in item.get('rental_acceptance_steps',[]):
                if step.get('id')=='G06.3d':step.update(current_Presentation_evidence=record,remaining='成功跨端點之非同步Presentation按所列profile取得診斷資格；其他profile、非同步失敗恢復、正常GUI／實體輸入、同端點、普通WORLD／其他服務及當版旅館仍待。')
    save(p,j)
for n in names[5:]:
    p=ROOT/n;first,sep,rest=p.read_text(encoding='utf-8-sig').partition('\n\n');assert first.startswith('2026-10-18') and sep;p.write_text(datetime.now().strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
for n in names[:5]:assert read(ROOT/n)['current_native_G06_presentation']==record
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());save(out/'readback.json',dict(record=record,before=before,after={n:sha(ROOT/n) for n in names},before_archive_sha256=sha(out/'before.zip'),active=active))
print(json.dumps(dict(checkpoint=str(out),qualified_profiles=list(profiles),actual_sessions=record['actual_sessions'],active=active is not None,whole_G06_complete=False,whole_goal_complete=False),ensure_ascii=False))
