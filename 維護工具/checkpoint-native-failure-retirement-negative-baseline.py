"""Checkpoint a temporary test-only counterexample; never qualify normal products."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',required=True,type=int);ap.add_argument('--mode',required=True,choices=['active','terminal']);ap.add_argument('--terminal-chunk');ap.add_argument('--terminal-exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-failure-retirement-negative-baseline')and a.session>0
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert sha(run/'runner.py')==sha(R/'維護工具/run-native-failure-retirement-negative-baseline.py')
rows=[c['runner'],*c['process_history']];filters=' OR '.join('ProcessId='+str(x['pid'])for x in rows)
cmd="@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',cmd]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
utc=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'))
present=lambda x:any(y['ProcessId']==x['pid']and abs((utc(y['created'])-utc(x['creation_utc'])).total_seconds())<.001 for y in live)
qualified=False
if a.mode=='active':
    assert not w['active_workloads'] and present(c['runner']) and not c['restored'] and a.terminal_chunk is None and a.terminal_exit_code is None
    pins=read(run/'overlay-compiled-inputs.json');assert all(sha(R/n)==h for n,h in pins.items())
else:
    assert w['active_workload_sessions']=={'negative_retirement_baseline':a.session}and not any(present(x)for x in rows)
    assert a.terminal_chunk not in [None,'','pending','unknown'] and a.terminal_exit_code in [0,1]and c['restored']and c['original_library_product_restored']
    pins=read(run/'before-compiled-inputs.json');assert all(sha(R/n)==h for n,h in pins.items())
    assert sha(c['original_product_retention']['product'])==c['original_product_retention']['product_sha256']
    if a.terminal_exit_code==0:
        assert c['status']=='observed_expected_two_retirement_counterexamples_pending_independent_readback'and c['actual_Rust_exit_code']==101
        assert c['counts']==dict(passed=0,failed=2,ignored=0,filtered_out=2460)and [x['exit_code']for x in c['process_history']]==[0,101]
        log=(run/'tests/stdout.log').read_text('utf-8-sig');assert sorted(re.findall(r'^test (\S+) \.\.\. FAILED$',log,re.M))==sorted(c['expected_tests'])
        assert log.count('assertion failed: c.snapshot().joined_room.is_none() && c.snapshot().rooms.is_empty()')==2
        assert sha(c['original_compiled_source_archive'])==c['original_compiled_source_archive_sha256']
        overlay=read(run/'overlay-compiled-inputs.json');assert {n for n in pins if pins[n]!=overlay[n]}=={c['changed_test_source']}
        assert c['process_history'][1]['command']==[c['product'],*c['expected_tests'],'--exact','--test-threads=1']
        with zipfile.ZipFile(c['original_compiled_source_archive'])as z:
            assert set(z.namelist())==set(pins)
            for n,h in pins.items():
                with z.open(n)as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==h
        with zipfile.ZipFile(run/'test-product.zip')as z:
            assert len(z.namelist())==1
            with z.open(z.namelist()[0])as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==c['product_sha256']
        qualified=True
    else:assert c['status']=='failed_preserved'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
record=dict(status='test_only_negative_baseline_running'if a.mode=='active'else'expected_two_retirement_counterexamples_independently_verified'if qualified else'negative_baseline_execution_failed_preserved',run=str(run),source_manifest_sha256=source,original_session_id=a.session,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=a.terminal_exit_code,baseline_counterexamples_verified=qualified,runtime_product_sources_unchanged=True,test_only_overlay=True,temporary_test_overlay_active=a.mode=='active',original_source_and_library_PE_restored=c['restored'],recorded_original_identities_terminal=a.mode=='terminal',expected_Rust_exit_code=101,normal_product_qualification_granted=False,GUI_verified=False,whole_goal_complete=False)
(out/'owned-CIM.json').write_bytes(raw);(out/'reader.py').write_bytes(Path(__file__).read_bytes())
if qualified:
    files={x.relative_to(run).as_posix():x for x in run.rglob('*')if x.is_file()and not x.is_relative_to(out)and x.suffix!='.zip'}
    files['reader.py']=out/'reader.py';files['owned-CIM.json']=out/'owned-CIM.json';hashes={n:sha(p)for n,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
    record.update(raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_archive=str(run/'test-product.zip'),product_archive_sha256=sha(run/'test-product.zip'))
    (out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版失敗退役反例：'+('沿原'+str(a.session)+'只覆加兩個測試斷言建置／執行，Runtime未改；所有對應QA與來源凍結，完成後原始碼及原library PE回復。'if a.mode=='active'else'兩個原時限失敗分支確實保留joined_room，預期Rust101／2敗已核對，原測試來源與原library PE均回復；接續候選修復，非正常產品測試失敗或資格。'if qualified else'執行失敗封存，先修驗證框架。')+'不交人工；原正常／library／資源資格各留其來源。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_failure_retirement_negative_baseline']=record;j.update(active_workloads={'negative_retirement_baseline':record}if a.mode=='active'else{},active_workload_sessions={'negative_retirement_baseline':a.session}if a.mode=='active'else{},source_and_corresponding_QA_frozen=a.mode=='active',source_freeze_while_Cargo=a.mode=='active',temporary_test_overlay_active=a.mode=='active',next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),baseline_counterexamples_verified=qualified,original_session_id=a.session),ensure_ascii=False))
