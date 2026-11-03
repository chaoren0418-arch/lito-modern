"""Reserve/release exact owned isolated candidate stages without promoting the normal product."""
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
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int);ap.add_argument('--kind',choices=['compile','resources','live'],required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);ap.add_argument('--receipt-observation-chunk');a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native44FE-extra-finite-effects')or run.is_relative_to(V/'native44FE-ability-buff-requests')
c=read(run/'receipt.json');source=c['base_source_manifest_sha256']if a.kind=='compile'else c['source_manifest_sha256'];assert source=='44FEA7384E2AC822E369256C233ACAEAA1591285FE531D915394DCD8ACEB37D0'
runner=c['original_runner']if a.kind=='compile'else c['runner'];assert not c.get('normal_product_modified')and c['actual_sessions']==0 if a.kind=='compile'else not c['normal_product_modified']
build=run if a.kind=='compile'else Path(c['compile']);inputs=read(build/'compiled-inputs.json');assert len(inputs)==1041
for n,h in inputs.items():assert sha(ROOT/n)==h,n
identity=read(build/'candidate-inputs.json');assert sha(build/'candidate-inputs.json')==c['candidate_manifest_sha256']
for n,h in identity['candidate_files'].items():assert sha(build/n)==h,n
terminal=a.terminal_chunk is not None or a.receipt_observation_chunk is not None
if a.receipt_observation_chunk:assert a.session is None and a.terminal_chunk is None and a.exit_code is None and a.receipt_observation_chunk not in ['','pending','unknown']
else:assert a.session and a.session>0
if terminal:
    assert a.receipt_observation_chunk or (a.terminal_chunk not in ['','pending','unknown']and a.exit_code is not None)
    assert c.get('owned_process')is None and not c.get('owned_processes')
    assert all('exit_code'in row for row in c['process_history'])
    assert c['status']in ['compiled_isolated_lazy_finite_effect_variant','executed_candidate_resources_pending_independent_qualification','executed_current75_ability_live_pending_independent_qualification','failed_preserved']
else:
    assert a.exit_code is None and c['status']in ['compiling_isolated_lazy_finite_effect_variant','executing_isolated_finite_resource_tests','executing_current75_ability_live','preparing_current75_ability_live']
ids=','.join(str(row['pid'])for row in [runner]+c['process_history'])
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
p=subprocess.run(['powershell.exe','-NoProfile','-Command',command],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert p.returncode==0 and not p.stderr
current=json.loads(p.stdout.decode('utf-8-sig'))if p.stdout.strip()else[];current=[current]if isinstance(current,dict)else current;current={row['ProcessId']:row for row in current}
def original_live(row):
    actual=current.get(row['pid']);return bool(actual and abs((datetime.fromisoformat(actual['created'])-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001)
if terminal:assert not any(original_live(row)for row in [runner]+c['process_history'])
else:assert original_live(runner)
queue=read(ROOT/'工作佇列.json');key='isolated_candidate_'+a.kind
assert queue['active_workload_sessions']in [{},{key:a.session}]
before=read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(ROOT/n)==h for n,h in before.items())
out=run/('checkpoint-'+('terminal-'if terminal else'active-')+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(ROOT/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
record=dict(status='original_isolated_stage_terminal_not_normal_product_qualification'if terminal else'original_isolated_stage_live',kind=a.kind,run=str(run),source_manifest_sha256=source,candidate_manifest_sha256=c['candidate_manifest_sha256'],original_session=a.session,runner=runner,
    terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,receipt_observation_chunk=a.receipt_observation_chunk,wrapper_terminal_tool_observed=bool(a.terminal_chunk),recorded_owned_all_terminal=terminal,source_overlays_used=True,normal_product_modified=False,qualification_granted=False,GUI_verified=False,physical_audio_verified=False,whole_goal_complete=False)
save(out/'evidence.json',record)
stage=('僅由receipt與原PID／出生時間核對程序已終止，原工具terminal回傳遺失，wrapper退出碼不推定；子程序退出碼及實際結果另核對。'if a.receipt_observation_chunk else('原程序已actual exit'+str(a.exit_code)+'並核對原身分終止，須保留實際結果及獨立讀回。'if terminal else'原程序仍活躍，正常來源與對應候選／QA固定，沿同一session續讀，不重啟。'))
work='當版44FEA738之廣域Debug531／Release527過、各0敗21既有忽略，以及完整library兩版各1969過0敗484忽略已獨立核對，忽略本文未執行。隔離能力強化／S6B-S6C與11特效候選 '+a.kind+' 原'+str(a.session)+'：'+stage+'候選與正常產物分開，資源／實際施法／GUI資格依實際範圍分別核對，不由編譯或metadata推廣。531灰色抽樣來源已核對原Native地板與0.til，完整正常地圖可見性仍待。先自行開發及自動端到端後才人工驗收，開發期間不請使用者登入；8秒／60秒／Release與G11預算不變，相鄰只讀且產物獨立，完整G01–G12未完成。'
date=(datetime.now(timezone.utc)+timedelta(hours=8)).strftime('%Y-%m-%d')
for n in before:
    path=ROOT/n
    if path.suffix=='.json':
        value=read(path);assert value['current_product_identity']['source_manifest_sha256']==source
        value.update(current_native44FE_isolated_work=record,active_workloads={}if terminal else{key:record},active_workload_sessions={}if terminal else{key:a.session},source_and_corresponding_QA_frozen=not terminal,candidate_and_corresponding_QA_frozen=not terminal,source_freeze_while_Cargo=not terminal,
            next_task=work,next_focus=work,last_goal_turn_classification='progress',last_goal_turn_evidence=str(out/'evidence.json'),goal_complete=False,whole_goal_complete=False)
        value['current_focus']=dict(item='current44FE-isolated-'+a.kind,work=work,applies_to_current_source=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):value['next_action']=work
        if 'updated_date'in value:value['updated_date']=date
        save(path,value)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;datetime.strptime(first[:10],'%Y-%m-%d');path.write_text(date+'：'+work+'\n\n'+rest,encoding='utf-8',newline='\n')
save(out/'readback.json',dict(before_files=before,after_files={n:sha(ROOT/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
assert all(sha(ROOT/n)==h for n,h in read(out/'readback.json')['after_files'].items())
save(run.parent/'current-continuation.json',dict(**record,latest_checkpoint=str(out),latest_readback=str(out/'readback.json'),must_poll_original_before_restart=not terminal))
print(json.dumps(dict(checkpoint=str(out),session=a.session,terminal=terminal,exit_code=a.exit_code,normal_product_modified=False),ensure_ascii=False))
