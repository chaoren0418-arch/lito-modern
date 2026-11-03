"""Eight-file checkpoint for compiler-only work, never functional qualification."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--mode',choices=['active','terminal'],required=True)
ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-development-typecheck')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
assert c['tests_executed']==c['game_or_fixture_executions']==c['GUI_inputs']==0
assert all(sha(R/n)==h for n,h in read(run/'base-compiled-inputs.json').items())
for p,h in c['normal_product_pins'].items():assert sha(p)==h
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(r['pid']) for r in rows)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);actual=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];actual=[actual]if isinstance(actual,dict)else actual
def live(row):return any(x['ProcessId']==row['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in actual)
if a.mode=='active':
    assert live(c['pipeline']) and c.get('finished_utc') is None and not w['active_workloads']
    assert a.terminal_chunk is None and a.exit_code is None
else:
    assert a.terminal_chunk and a.exit_code is not None and c.get('finished_utc') and not any(live(row) for row in rows)
    assert all(row['exit_code'] is not None for row in c['process_history'])
    assert w['active_workload_sessions'] in ({},{'development_typecheck':a.session})
    assert (c['status']=='metadata_compilation_both_profiles_complete_no_tests_executed')==(a.exit_code==0)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='development_metadata_compilation_running' if a.mode=='active' else 'development_metadata_compilation_terminal_independent_readback_pending',
 run=str(run),original_session_id=a.session,source_manifest_sha256=source,pipeline=c['pipeline'],process_history=c['process_history'],process_observation=actual,
 terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.exit_code,original_identities_terminal=a.mode=='terminal',
 tests_executed=0,build_scope='rustc --emit=metadata only',normal_source_modified=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record));(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'owned-CIM.json').write_bytes(raw)
work=('候選隔離編譯原'+str(a.session)+'：'+('沿原程序做兩版完整模組／正常入口／案例程式的metadata型別檢查，來源及候選固定。' if a.mode=='active' else'已終止，原exit '+str(a.exit_code)+'；編譯結果與來源／metadata仍待獨立讀回。')+'未執行測試、遊戲或GUI，正常3E及兩正式EXE不變；測試依使用者要求暫緩，目標未完成。')
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_development_typecheck_active']=record
        j.update(active_workloads={'development_typecheck':record}if a.mode=='active'else{},active_workload_sessions={'development_typecheck':a.session}if a.mode=='active'else{},
            source_and_corresponding_QA_frozen=a.mode=='active',source_freeze_while_Cargo=False,next_task=work,next_focus=work,
            last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out/'evidence.json'),mode=a.mode,tests_executed=0),ensure_ascii=False))
