"""Register only a currently observed original compiler/pipeline; no restart."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--original-session',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-E149-recovery-linked-build');c=read(run/'cohort.json')
assert c['status'].startswith('linking_')
identities=[c['pipeline']]+([c['owned_process']] if c['owned_process'] else []);ids=','.join(str(x['pid']) for x in identities)
query="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
observed=subprocess.check_output(['pwsh','-NoProfile','-Command',query]);actual=json.loads(observed.decode('utf-8-sig')) if observed.strip() else []
actual=[actual] if isinstance(actual,dict) else actual
for row in identities:
 assert any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual),'original identity not live; inspect same original tool session'
w=read(R/'工作佇列.json');assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-active-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'owned-CIM.json').write_bytes(observed);(out/'cohort-observed.json').write_bytes((run/'cohort.json').read_bytes())
record=dict(status='original_link_pipeline_and_compiler_observed_live',run=str(run),original_session_id=a.original_session,
 profile=c['profile'],pipeline=c['pipeline'],compiler_observed=c['owned_process'],source_manifest_sha256=c['source_manifest_sha256'],
 effective_overlay_files=14,phase_observed=c['status'],source_and_corresponding_QA_frozen=True,
 original_tool_output_poll_required=True,tests_executed=0,outputs_executed=False,GUI_verified=False,whole_goal_complete=False,
 recorded_utc=datetime.now(timezone.utc).isoformat())
(out/'evidence.json').write_bytes(data(record))
work='E149隔離'+c['profile']+'連結沿原session '+str(a.original_session)+'執行；原pipeline'+('與當下rustc' if c['owned_process'] else '（兩個編譯階段之間）')+'出生身分已當場核對存活，僅建置rlib／正式案例PE，不執行輸出。固定正常1049輸入、14檔候選與依賴；來源／對應QA凍結，僅接續原handle，不因觀察逾時重啟。原child結束後逐命令／artifact／完整graph／程序退出獨立核對，才能釋放凍結。使用者暫緩測試，DB／SQL／factory／功能／GUI未執行，整體未完成。'
staged={}
for n in before:
 p=R/n
 if p.suffix=='.json':
  j=read(p);j['current_E149_recovery_link_active']=record
  j['active_workloads']={'native_E149_recovery_link':record};j['source_and_corresponding_QA_frozen']=True
  j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  j['current_focus']=dict(item='G06_3d_E149_original_isolated_link_active',work=work,goal_complete=False)
  if n.endswith('next-validation-action.json'):j['next_action']=work
  staged[n]=data(j)
 else:
  _,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[n]=(datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
assert all((R/n).read_bytes()==b for n,b in staged.items())
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),original_session_id=a.original_session,phase=c['status'],original_processes_observed_live=True),ensure_ascii=False))
