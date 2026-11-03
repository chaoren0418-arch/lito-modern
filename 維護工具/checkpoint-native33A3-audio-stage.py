"""Record one exact diagnostic audio compile/case; never infer GUI/device qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,time,zipfile
ROOT=Path(__file__).resolve().parents[1];V=ROOT/'驗證紀錄/current'
def read(p):
    for i in range(10):
        try:return json.loads(Path(p).read_text('utf-8-sig'))
        except json.JSONDecodeError:
            if i==9:raise
            time.sleep(.05)
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,x):Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--kind',choices=['compile','case'],required=True);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33A3-teleport-audio')and a.session>0;c=read(run/'receipt.json');terminal=a.terminal_chunk is not None
assert not c['source_overlays_used']and c['normal_source_bodies_unchanged']and c['diagnostic_request_API_extension']and not c['normal_product_source_modified']
build=run if a.kind=='compile'else Path(c['compile']);inputs=read(build/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
queue=read(ROOT/'工作佇列.json');source=c['source_manifest_sha256'];assert queue['current_product_identity']['source_manifest_sha256']==source
runner=c['original_runner']if a.kind=='compile'else read(run/'runner-identity.json');identities=[runner]+c['process_history']
if a.kind=='case'and c.get('case'):
    case=Path(c['case']);detail=read(case/'receipt.json')
    identities +=[dict(pid=row['pid'],creation_utc=row['start_utc'])for row in detail.get('owned_processes',[])]
if terminal:
    assert a.exit_code is not None and a.terminal_chunk not in ['','pending','unknown']and c['owned_process']is None and all('exit_code'in row for row in c['process_history'])
    assert c['status']in ['compiled_current_normal_body_audio_diagnostics','executed_current_normal_body_nonempty_audio_case','failed_preserved']
else:assert a.exit_code is None and c['status']in ['compiling_current_normal_body_audio_diagnostics','prepared_current_normal_body_nonempty_audio_case','executing_current_normal_body_nonempty_audio_case']
ids=','.join(str(row['pid'])for row in identities);ps="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',ps]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
def original_live(row):return any(actual['ProcessId']==row['pid']and abs((datetime.fromisoformat(actual['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for actual in live)
if terminal:assert not any(original_live(row)for row in identities)
else:assert original_live(runner)
key='G06_audio_'+a.kind;assert queue['active_workload_sessions']in [{},{key:a.session}]
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+('terminal-'if terminal else'active-')+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='original_current_audio_stage_terminal_not_behavioral_qualification'if terminal else'original_current_audio_stage_confirmed_live',kind=a.kind,run=str(run),profile=c['profile'],scenario=c.get('scenario'),source_manifest_sha256=source,original_session=a.session,runner=runner,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,recorded_original_owned_processes_terminal=terminal,source_overlays_used=False,normal_source_bodies_unchanged=True,diagnostic_request_API_extension=True,normal_product_source_modified=False,qualification_granted=False,normal_GUI_API_permission_inferred=False,normal_EXE_runtime_inferred=False,GUI_verified=False,physical_audio_verified=False,whole_G06_complete=False,whole_goal_complete=False)
save(out/'evidence.json',record);(out/'process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
stage=('已actual exit'+str(a.exit_code)+'並核對原程序終止，編譯／單案結果仍需自己的獨立讀回。'if terminal else'仍沿原程序執行，來源及對應QA固定，不重啟。')
work='來源'+source[:8]+'正常本體之隔離非空音效'+a.kind+'原'+str(a.session)+'：'+stage+'只附加C1F診斷請求方法，正常來源本體逐位元保留；不推廣正常EXE／GUI權限或實體音效。兩版六分支Presentation與晚到舊音效／新音效一次交付各須實際驗證，其他G06分支及完整G01–G12仍待。開發期間不請使用者登入，8秒／60秒／Release及G11預算不變，相鄰只讀且登入器產物獨立。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value.update(current_native33A3_audio_stage=record,active_workloads={}if terminal else{key:record},active_workload_sessions={}if terminal else{key:a.session},source_and_corresponding_QA_frozen=not terminal,candidate_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=not terminal and a.kind=='compile',next_task=work,next_focus=work,last_goal_turn_classification='progress'if terminal else'verified_wait',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current-normal-body-audio-'+a.kind,work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in value:value['updated_date']=date
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=not terminal))
print(json.dumps(dict(checkpoint=str(out),original_session=a.session,terminal=terminal,exit_code=a.exit_code,qualification_granted=False),ensure_ascii=False))
