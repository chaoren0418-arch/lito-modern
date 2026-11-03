"""Seal original failing full-library run and release only terminal owned identities."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
from ctypes import wintypes
import argparse,ctypes,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-installed-resource-profile-library-default')
assert a.session==4535 and a.terminal_chunk=='d81b84'and a.exit_code==1
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='9F99C1203C0BB3F2CF87A873C7A81F7B20B08AFD3546A2282A84B62F6C8124C8'
assert c['status']=='failed_preserved'and c['owned_process']is None and set(c['profiles'])=={'Debug'}
assert w['active_workload_sessions']=={'full_library':4535}and w['source_and_corresponding_QA_frozen']
assert w['active_workloads']['full_library']['run']==str(run)
for n,h in read(run/'compiled-inputs.json').items():assert sha(R/n)==h
for n,h in c['validation_pins'].items():assert sha(R/'維護工具'/n)==h
assert sha(c['compiled_source_archive'])==c['compiled_source_archive_sha256']
runtime=read(run/'Debug/runtime-full-current/receipt.json');product=read(run/'Debug/product-retention.json')
assert runtime['status']=='failed_pinned_library_tests'and runtime['exit_code']==101 and runtime['filter']==''and runtime['filtered_out']==0
assert(runtime['passed'],runtime['failed'],runtime['ignored'])==(1968,2,488)
assert product['product_sha256']==runtime['product_sha256']==sha(product['product'])
restoration=read(run/'Debug/runtime-full-current/legacy-report-restoration.json');assert restoration['existing_reports_restored']
text=(run/'Debug/runtime-full-current/tests.log').read_text('utf-8-sig')
assert 'running 2458 tests'in text and re.findall(r'test result:.*',text)==['test result: FAILED. 1968 passed; 2 failed; 488 ignored; 0 measured; 0 filtered out; finished in 393.29s']
failed=re.findall(r'^test (\S+) \.\.\. FAILED$',text,re.M);assert len(failed)==2
rows=[dict(pid=c['pipeline_pid'],creation_utc=c['pipeline_creation_utc'],role='pipeline'),*c['process_history'],dict(pid=runtime['pid'],creation_utc=runtime['creation_utc'],role='actual Rust tests')]
assert len(rows)==6 and len(c['process_history'])==4
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+' OR '.join('ProcessId='+str(r['pid'])for r in rows)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW)
live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
k=ctypes.WinDLL('kernel32',use_last_error=True);k.OpenProcess.argtypes=[wintypes.DWORD,wintypes.BOOL,wintypes.DWORD];k.OpenProcess.restype=wintypes.HANDLE;k.WaitForSingleObject.argtypes=[wintypes.HANDLE,wintypes.DWORD];k.CloseHandle.argtypes=[wintypes.HANDLE]
for row in rows:
 for current in live:
  if row['pid']==current['ProcessId']and abs((datetime.fromisoformat(row['creation_utc'])-datetime.fromisoformat(current['created'].replace('Z','+00:00'))).total_seconds())<.001:
   handle=k.OpenProcess(0x100000|0x1000,False,row['pid']);assert handle
   try:assert k.WaitForSingleObject(handle,0)==0,'original owned process still active'
   finally:k.CloseHandle(handle)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('failure-checkpoint-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'sealer.py').write_bytes(Path(__file__).read_bytes());(out/'owned-CIM.json').write_bytes(raw)
files={p.relative_to(run).as_posix():p for p in run.rglob('*')if p.is_file()and not p.is_relative_to(out)and p.suffix!='.zip'}
files.update({'sealer.py':out/'sealer.py','owned-CIM.json':out/'owned-CIM.json'});hashes={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='failed_current_full_library_default_Debug_preserved_no_qualification',run=str(run),source_manifest_sha256=source,
 original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=1,actual_Rust_exit_code=101,
 profiles=dict(Debug=dict(passed=1968,failed=2,ignored=488,filtered_out=0,inventory=2458),Release=dict(executed=False)),
 actual_test_sha256=product['product_sha256'],failed_tests=failed,recorded_original_owned_identities=rows,recorded_original_owned_processes_all_terminal=True,
 raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,
 retained_compiled_source_archive=c['compiled_source_archive'],retained_compiled_source_archive_sha256=c['compiled_source_archive_sha256'],
 existing_historical_reports_restored=True,diagnosis='Roof test targets array index0 although RenderInfo now sorts by depth; new regression objective4 lies beyond 200px viewport under actual x=24*(dx+dy) projection. Preserve exact pixels/hits; resolve entity1 explicitly and make all synthetic placements demonstrably visible.',
 qualification_granted=False,GUI_verified=False,whole_goal_complete=False)
(out/'evidence.json').write_bytes(data(record))
work='來源9F99C120原4535完整library Debug1968過2敗488忽略，Rust101／wrapper1，Release未啟動；六組記錄原程序已終止，失敗封存。修正屋頂測試角色索引及新重現案視口，保留像素／命中／排序斷言，重新完整Debug／Release後再原資源及GUI，不交人工。'
staged={}
for n in before:
 p=R/n
 if p.suffix=='.json':
  v=read(p);assert v['current_product_identity']['source_manifest_sha256']==source
  v['current_native_catalog_library_failure']=record
  v['current_native_catalog_library_active'].update(status='failed_preserved_original_terminal',qualification_granted=False,terminal_tool_chunk=a.terminal_chunk,observed_actual_exit_code=1)
  v.update(active_workloads={},active_workload_sessions={},source_and_corresponding_QA_frozen=False,source_freeze_while_Cargo=False,current_full_default_library_verified=False,
   next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  v['current_focus']=dict(item='current-library-test-identity-and-viewport-repair',work=work,applies_to_current_source=True,goal_complete=False)
  v['current_source_goal_validation']['G03-current-regression']=dict(state='failed_full_library_Debug_Release_not_started',complete=False,evidence=record)
  if n.endswith('next-validation-action.json'):v['next_action']=work
  staged[n]=data(v)
 else:
  first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep
  staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),Debug=dict(passed=1968,failed=2,ignored=488),Release_executed=False,terminal_identities=6),ensure_ascii=False))
