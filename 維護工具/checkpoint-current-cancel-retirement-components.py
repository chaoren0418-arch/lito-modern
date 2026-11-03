"""Reserve or qualify original return tests and the new cancel regression."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return(json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--mode',choices=['active','terminal'],required=True);ap.add_argument('--session',type=int);ap.add_argument('--terminal-chunk');ap.add_argument('--exit-code',type=int);a=ap.parse_args();run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-cancel-retirement-components')
c=read(run/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']
assert not c['normal_product_modified'] and not c['candidate_applied'] and sha(c['candidate'])==c['candidate_sha256']
assert all(sha(R/n)==h for n,h in c['compiled_inputs'].items()) and all(sha(p)==h for p,h in c['generated_and_referenced_pins'].items())
proposal=read(c['candidate']);assert all(sha(R/n)==h for n,h in proposal['before'].items()) and all(sha(p)==proposal['after'][n] for n,p in proposal['candidate_files'].items())
original=(R/'LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs').read_text('utf-8-sig');extended=Path(proposal['candidate_files']['LiTo381-Modern/src/modern_lito/native_rental_return_tests.rs']).read_text('utf-8-sig');assert extended.startswith(original)
assert len(c['test_names'])==11 and len(re.findall(r'(?m)^#\[test\]\nfn (\w+)\(',original))==10
rows=[c['pipeline'],*c['process_history']];ids=','.join(str(x['pid']) for x in rows)
cmd="@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
obs=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert obs.returncode==0 and not obs.stderr
live=json.loads(obs.stdout.decode('utf-8-sig')) if obs.stdout.strip() else [];live=[live] if isinstance(live,dict) else live
same=lambda x,y:x['ProcessId']==y['pid'] and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(y['creation_utc'])).total_seconds())<.001
terminal=a.mode=='terminal'
if terminal:
    assert a.terminal_chunk not in [None,'','unknown','pending'] and a.exit_code==0 and c['status']=='executed_cancel_retirement_components_both_profiles'
    assert c['owned_process'] is None and len(rows)==9 and not any(same(x,y) for x in live for y in rows)
    assert [x['exit_code'] for x in c['process_history']]==[0,101,0,0]*2 and not any(x.get('exact_owned_timeout_cleanup') for x in c['process_history'])
    assert not w['active_workloads'] or w['active_workload_sessions']=={'cancel_retirement_components':a.session}
    for profile in ['Debug','Release']:
        info=c['profiles'][profile];assert all(sha(p['path'])==p['sha256'] for p in info['extern'].values())
        for mode in ['baseline','candidate']:
            p=info['modes'][mode];assert sha(p['executable'])==p['executable_sha256']
            results=dict(re.findall(r'^test (\S+) \.\.\. (ok|FAILED|ignored)$',(run/profile/mode/'run/stdout.log').read_text('utf-8-sig'),re.M))
            expected={'candidate::native_login::rental_return_tests::'+n:('FAILED' if mode=='baseline' and n=='cancel_live_world_retires_snapshot_without_losing_generation' else 'ok') for n in c['test_names']};assert results==p['results']==expected
else:
    assert a.session is not None and a.session>0 and a.exit_code is None and a.terminal_chunk is None and any(same(x,c['pipeline']) for x in live)
    assert not w['active_workloads'] and not w['source_and_corresponding_QA_frozen']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=run/('checkpoint-'+a.mode+'-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'reader.py').write_bytes(Path(__file__).read_bytes());(out/'owned-CIM.json').write_bytes(obs.stdout)
record=dict(status='qualified_cancel_retirement_components_both_profiles' if terminal else 'running_cancel_retirement_components',source_manifest_sha256=source,run=str(run),qualified=terminal,candidate=c['candidate'],candidate_sha256=c['candidate_sha256'],original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=a.exit_code,original_return_assertions_preserved=True,original_tests_each=10,new_cancel_regression_each=1,baseline_passed_each=10,baseline_expected_failures_each=1,candidate_passed_each=11,profiles=c['profiles'],original_identities_terminal=terminal,normal_product_modified=False,candidate_applied=False,GUI_verified=False,whole_goal_complete=False)
if terminal:
    files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and p.suffix not in ['.exe','.pdb'] and not any(part.startswith('checkpoint-') for part in p.relative_to(run).parts)}
    for n,p in proposal['candidate_files'].items():files['candidate/'+n]=Path(p)
    files.update({'reader.py':Path(__file__),'candidate.json':Path(c['candidate'])});hashes={n:sha(p) for n,p in files.items()}
    with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
        for n,p in files.items():z.write(p,n)
    with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
    record.update(raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
    (out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record))
work='取消退役隔離元件'+('兩版均核對原10項返回測試全過、新取消回歸原版預期失敗1／候選通過；候選每版11過0敗。正式取消候選另已雙版8sessions通過，尚須彙整套用審查與當版完整來源／GUI重驗。' if terminal else '沿原批次編譯／執行；原10項返回斷言逐行保留，新增取消回歸先反例再候選，來源及對應QA固定。')+'正常產品仍A695、未套用候選，既有獨立資格保留，不交人工。'
staged={}
for n in before:
    p=R/n
    if p.suffix=='.json':
        j=read(p);j['current_native_cancel_retirement_components']=record;j.update(active_workloads={} if terminal else {'cancel_retirement_components':record},active_workload_sessions={} if terminal else {'cancel_retirement_components':a.session},source_and_corresponding_QA_frozen=not terminal,next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',whole_goal_complete=False,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=p.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(checkpoint=str(out),qualified=terminal,normal_product_modified=False),ensure_ascii=False))
