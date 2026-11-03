"""Reserve or release an exact owned compile/run without granting behavioral qualification."""
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
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--kind',choices=['compile','run'],required=True);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native33A3-same-endpoint-rental');c=read(run/'receipt.json');terminal=a.terminal_chunk is not None
assert (a.session is not None and a.session>0)or(a.session is None and a.kind=='compile'and terminal)
assert not c['source_overlays_used']and not c['normal_source_modified']and c['normal_source_bodies_unchanged']and not c['diagnostic_API_extension']
queue=read(ROOT/'工作佇列.json');source=c['source_manifest_sha256'];assert source==queue['current_product_identity']['source_manifest_sha256']
build=run if a.kind=='compile'else Path(c['compile']);inputs=read(build/'compiled-inputs.json');assert len(inputs)==1048
for n,h in inputs.items():assert sha(ROOT/n)==h,n
assert sha(build/'probe.rs')==c['probe_sha256']
if terminal:
    assert a.exit_code is not None and a.terminal_chunk not in ['','pending','unknown']and c.get('owned_process')is None and not c.get('owned_processes')
    assert all('exit_code'in row for row in c['process_history'])
    assert c['status']in ['compiled_normal_public_API_same_endpoint_rental_probe','executed_same_endpoint_advertised_rental_pending_independent_readback','failed_preserved']
else:
    assert a.exit_code is None and c['status']in ['compiling_normal_public_API_same_endpoint_rental_probe','preparing_same_endpoint_advertised_rental_probe','executing_same_endpoint_advertised_rental_probe']
identities=[c['runner']]+c['process_history'];ids=','.join(str(row['pid'])for row in identities)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);current=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];current=[current]if isinstance(current,dict)else current
def original_live(row):return any(value['ProcessId']==row['pid']and abs((datetime.fromisoformat(value['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for value in current)
if terminal:assert not any(original_live(row)for row in identities)
else:assert original_live(c['runner'])
key='same_endpoint_rental_'+a.kind;assert queue['active_workload_sessions']in [{},{key:a.session}]
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+('terminal-'if terminal else'active-')+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='original_same_endpoint_rental_stage_terminal_not_behavioral_qualification'if terminal else'original_same_endpoint_rental_stage_confirmed_live',kind=a.kind,profile=c['profile'],run=str(run),source_manifest_sha256=source,probe_sha256=c['probe_sha256'],original_session=a.session,returned_without_persistent_session=a.session is None,runner=c['runner'],terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,recorded_original_owned_processes_terminal=terminal,source_overlays_used=False,normal_source_modified=False,qualification_granted=False,normal_channel_continuation_verified=False,old_CA00_failure_preserved=True,GUI_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record);(out/'process-CIM.json').write_bytes(raw);(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes())
stage=('已actual exit'+str(a.exit_code)+'並核對原程序終止；須另讀實際編譯／封包／權威與畫面資料，不由退出碼推廣行為資格。'if terminal else'仍沿原程序執行，來源及對應QA固定，不重啟。')
origin='原'+str(a.session)if a.session is not None else'工具回傳'+a.terminal_chunk
work='來源'+source[:8]+'同端點收到租裝服務後接續之獨立'+a.kind+origin+'：'+stage+'此案依正常public API選原配裝及收到的房間，保留原CA00普通頻道失敗，不能推廣普通頻道或GUI。其前置同版廣域／library資格各依自己的紀錄；G06其他服務、正常可見性／實體輸入音效及完整G01–G12仍待。開發期間不請使用者登入，8秒／60秒／Release與G11預算不變，相鄰只讀且登入器產物獨立。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value.update(current_native33A3_same_endpoint_rental_work=record,active_workloads={}if terminal else{key:record},active_workload_sessions={}if terminal else{key:a.session},source_and_corresponding_QA_frozen=not terminal,candidate_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=not terminal and a.kind=='compile',next_task=work,next_focus=work,last_goal_turn_classification='progress'if terminal else'verified_wait',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='same-endpoint-advertised-rental-'+a.kind,work=work,applies_to_current_source=True,goal_complete=False)
        if 'updated_date'in value:value['updated_date']=date
        if n.endswith('next-validation-action.json'):value['next_action']=work
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=not terminal))
print(json.dumps(dict(checkpoint=str(out),original_session=a.session,terminal=terminal,exit_code=a.exit_code,qualification_granted=False),ensure_ascii=False))
