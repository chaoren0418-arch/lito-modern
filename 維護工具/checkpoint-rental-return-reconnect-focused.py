"""Record/independently qualify only the exact ten Debug regression cases."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--mode',choices=['active','terminal'],required=True);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-rental-return-reconnect-focused')and a.session>0
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
for n,h in read(run/'compiled-inputs.json').items():assert sha(R/n)==h
assert sha(run/'runner.py')==sha(R/'維護工具/run-rental-return-reconnect-focused.py')
rows=[c['pipeline'],*c['process_history']]
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+' OR '.join('ProcessId='+str(r['pid'])for r in rows)+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command],creationflags=subprocess.CREATE_NO_WINDOW)
actual=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];actual=[actual]if isinstance(actual,dict)else actual
live=[r for r in rows if any(x['ProcessId']==r['pid']and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(r['creation_utc'])).total_seconds())<.001 for x in actual)]
qualified=False
if a.mode=='active':
 assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']
 assert c['pipeline']in live and c['status']in ['compiling_focused_Debug_library_fixture_regression','running_focused_Debug_library_fixture_regression']
 assert a.terminal_chunk is None and a.exit_code is None
else:
 assert a.terminal_chunk not in [None,'','pending','unknown']and a.exit_code in [0,1]and not live
 assert w['active_workload_sessions']in [{},{'rental_return_reconnect_focused':a.session}]
 if a.exit_code==0:
  assert c['status']=='executed_ten_focused_Debug_rental_return_reconnect_regressions'and c['owned_process']is None
  assert len(c['process_history'])==2 and all(x['exit_code']==0 for x in c['process_history'])
  assert c['counts']==dict(passed=10,failed=0,ignored=0,filtered_out=2452)and c['actual_Rust_exit_code']==0
  app=read(Path(c['application'])/'application.json');candidate=read(app['candidate']);assert candidate['production_source_modified']and candidate['original_C10_handshake_retained']and candidate['fallback_only_after_one_S65_and_original_8s_timeout']
  assert sorted(candidate['exact_regression_tests'])==c['expected_tests']
  text=(run/'tests/stdout.log').read_text('utf-8-sig');found=re.findall(r'^test (\S+) \.\.\. ok$',text,re.M)
  assert sorted(found)==c['expected_tests']and len(found)==10
  assert re.findall(r'test result:.*',text)==['test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 2452 filtered out; finished in '+re.search(r'finished in ([0-9.]+s)',text)[1]]
  assert c['process_history'][1]['command']==[c['product'],*c['expected_tests'],'--exact','--test-threads=1']
  assert sha(c['product'])==c['product_sha256']
  b=Path(c['product']).read_bytes();off=int.from_bytes(b[60:64],'little');assert b[:2]==b'MZ'and b[off:off+6]==b'PE\0\0\x64\x86'and b[off+24:off+26]==b'\x0b\x02'
  with zipfile.ZipFile(run/'product.zip')as z:assert len(z.namelist())==1 and hashlib.sha256(z.read(z.namelist()[0])).hexdigest().upper()==c['product_sha256']
  qualified=True
 else:assert c['status']=='failed_preserved'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'checkpoint.py').write_bytes(Path(__file__).read_bytes());(out/'owned-CIM.json').write_bytes(raw)
record=dict(status='focused_Debug_rental_return_reconnect_ten_qualified_limited'if qualified else'focused_Debug_library_fixture_running'if a.mode=='active'else'focused_Debug_library_fixture_failed_preserved',
 run=str(run),source_manifest_sha256=source,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=a.exit_code,qualified=qualified,
 profiles=dict(Debug=c.get('counts',{})),exact_tests=c['expected_tests'],recorded_original_owned_identities=len(rows),recorded_original_owned_processes_terminal=a.mode=='terminal',
 full_library_verified=False,normal_products_qualified=False,actual_network_sessions=1 if qualified else None,actual_formal_sessions=0,GUI_verified=False,whole_goal_complete=False)
if qualified:
 files={p.relative_to(run).as_posix():p for p in run.rglob('*')if p.is_file()and not p.is_relative_to(out)and p.suffix!='.zip'}
 files['reader.py']=out/'checkpoint.py';files['owned-CIM.json']=out/'owned-CIM.json';hashes={n:sha(p)for n,p in files.items()}
 with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
  for n,p in files.items():z.write(p,n)
 with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
 record.update(raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,product_sha256=c['product_sha256'],product_archive=str(run/'product.zip'),product_archive_sha256=sha(run/'product.zip'))
 (out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record));work='當版A695C127租裝返回原8秒逾時後一次性原LOGIN／原來源恢復：十項Debug返回回歸'+('已實際10過0敗並獨立核對；只含一條未認證owned-loopback連線及三種拒絕／取消條件，非正式伺服器資格。接續正式75隔離API／正常兩版19／完整library兩版及GUI。'if qualified else'沿原'+str(a.session)+'建置／執行中，未授資格。'if a.mode=='active'else'失敗已保留，先修復。')+'原603C完整library及81資源雙版、Debug GUI入口保留原版，不移記當版；不改JAR／時限，不交人工。'
staged={}
for n in before:
 p=R/n
 if p.suffix=='.json':
  v=read(p);v['current_native_rental_return_reconnect_focused']=record
  v.update(active_workloads={'rental_return_reconnect_focused':record}if a.mode=='active'else{},active_workload_sessions={'rental_return_reconnect_focused':a.session}if a.mode=='active'else{},source_and_corresponding_QA_frozen=a.mode=='active',source_freeze_while_Cargo=a.mode=='active',next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
  v['current_focus']=dict(item='focused-library-fixture-regression',work=work,applies_to_current_source=True,goal_complete=False)
  if n.endswith('next-validation-action.json'):v['next_action']=work
  staged[n]=data(v)
 else:
  first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
 for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out),qualified=qualified,original_session=a.session),ensure_ascii=False))
