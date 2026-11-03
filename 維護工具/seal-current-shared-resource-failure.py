"""Preserve original failed opt-in run and publish exact limited disposition."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import json,hashlib,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
run=V/'native-library-shared-resource-optin-execution/20261019-195340-685986'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
data=lambda v:(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
w=read(R/'工作佇列.json');c=read(run/'cohort.json');record=w['current_native_library_shared_resource_optin']
assert record['observed_exit_code']==1 and record['terminal_tool_chunk']=='639258'and not record['qualified']
assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
assert c['status']=='failed_preserved' and c['resource_metadata_unchanged']and c['diagnostic_historical_outputs_restored']
assert set(c['profiles'])=={'Debug'} and c['profiles']['Debug']['exit_code']==101
s=(run/'Debug/tests.log').read_text('utf-8-sig')
assert re.findall(r'test result: (.*)',s)==['FAILED. 65 passed; 7 failed; 0 ignored; 0 measured; 2381 filtered out; finished in 207.21s']
failed=re.findall(r'^test (\S+) \.\.\. FAILED$',s,re.M);assert len(failed)==7
summaries=[]
for section in s.split('\n---- ')[1:]:
    name=section.split(' stdout ----',1)[0]
    if name not in failed:continue
    head=section.splitlines()[:9]
    summaries.append(dict(test=name,error_lines=[line[:300]for line in head],kind=
        'missing_explicit_legacy_TW_resource'if'os error 2'in section[:1400]else
        'installation_Text_index_reference_delta'if'left: 37497'in section[:1000]else
        'objective_render_placement_order_or_visibility_identity_mismatch'))
assert len(summaries)==7
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('failure-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
(out/'sealer.py').write_bytes(Path(__file__).read_bytes())
files={p.relative_to(run).as_posix():p for p in run.rglob('*')if p.is_file()and not p.is_relative_to(out)}
# Retain separate target-owned optional diagnostic captures even on a failed test.
captures=R/'LiTo381-Modern/target/current/diagnostic-output/shared-resource-20261019-195340-685986'
for p in captures.rglob('*.png'):files['optional-diagnostic/'+p.relative_to(captures).as_posix()]=p
hashes={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(record,profiles=dict(Debug=dict(passed=65,failed=7,ignored=0,filtered_out=2381,Rust_exit_code=101),Release=dict(executed=False)),
    original_failure_preserved=True,analysis=str(out/'evidence.json'),raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),
    raw_files=hashes,failures=summaries,resource_metadata_unchanged=True,diagnostic_historical_outputs_restored=True,
    next_action='修復原資源條件與原生RenderInfo排序／遮罩身分；保留所有原斷言及失敗，不把65項成功移記為整批通過。Release尚未執行。')
(out/'evidence.json').write_bytes(data(record))
work='當版預設library兩版各1969過0敗484忽略、native77兩版各10過；額外72原資源Debug65過7敗、Rust101／wrapper1，Release未啟動。先修復資源條件與原生排序／遮罩後重驗，再跑正常GUI；不交人工、不要求使用者登入。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        v=read(p);v['current_native_library_shared_resource_optin']=record
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='current_native_library_shared_resource_failure',work=work,applies_to_current_source=True,goal_complete=False)
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
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},evidence_sha256=sha(out/'evidence.json'),before_archive_sha256=sha(out/'before.zip'))))
print(json.dumps(dict(failure=str(out),Debug=dict(passed=65,failed=7),Release_executed=False,raw_files=len(files)),ensure_ascii=False))
