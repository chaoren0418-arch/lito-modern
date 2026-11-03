"""Independently qualify only the Text-reference candidate, preserving the red baseline."""
from pathlib import Path
from datetime import datetime,timezone
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
save=lambda p,x:Path(p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',type=int,required=True);ap.add_argument('--terminal-chunk',required=True);ap.add_argument('--terminal-exit-code',type=int,required=True);a=ap.parse_args()
run=a.run.resolve(strict=True);assert run.is_relative_to(V/'native-text-index-order-components') and a.session>0 and a.terminal_exit_code==0 and a.terminal_chunk not in ['','unknown','pending']
c=read(run/'cohort.json');assert c['status']=='executed_baseline_counterexample_and_candidate_both_profiles' and c['owned_process'] is None
cp=Path(c['candidate']);proposal=read(cp);assert sha(cp)==c['candidate_sha256'] and proposal['before_source_manifest_sha256']==c['source_manifest_sha256'] and proposal['only_cfg_test_reference_changed'] and not proposal['Runtime_changed']
for p,h in c['pinned_inputs'].items():assert sha(p)==h
assert len(c['process_history'])==10 and [j['exit_code'] for j in c['process_history']]==[0,101,0,0,0]*2 and not any(j.get('owned_timeout_cleanup') for j in c['process_history'])
name=next(iter(proposal['candidate_files']));original=(R/name).read_text('utf-8-sig');candidate=Path(proposal['candidate_files'][name]).read_text('utf-8-sig');prefix='    fn real_shared_text_matches_csharp_reference() {'
assert original[:original.index(prefix)]==candidate[:candidate.index(prefix)]
needle='''        let current_profile = format!("{:X}", Sha256::digest(&installed_index))
            == "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"'''
replacement='''        // The 2026-10-20 index only reorders duplicate-name records. The
        // independent C# reader verified the complete 37497-record multiset.
        // Accept only either exact observed index, with the same pinned pak.
        let current_profile = matches!(format!("{:X}", Sha256::digest(&installed_index)).as_str(),
            "CF4D420DDAB51BADB9CFF20F60C9ABB2E04156404A443897FBB8A44FED909D7E"
            | "B85FAF2685CAB65357800368228ED3A5A96108E512CF883FA63AD2F74A44A91B")'''
assert original.count(needle)==1 and candidate==original.replace(needle,replacement)
assert all(line in candidate for line in original.splitlines() if 'assert' in line)
profiles={};test='resources::tests::real_shared_text_matches_csharp_reference'
for profile in ['Debug','Release']:
    for variant in ['baseline','candidate']:
        info=c['profiles'][profile][variant];assert sha(info['executable'])==info['executable_sha256']
        text=(run/profile/(variant+'-text')/'stdout.log').read_text('utf-8-sig');expected='FAILED' if variant=='baseline' else'ok'
        assert re.findall(r'^test (\S+) \.\.\. (ok|FAILED)$',text,re.M)==[(test,expected)]
        if variant=='baseline':assert 'left: 37497' in text and 'right: 37496' in text
        else:assert re.search(r'test result: ok\. 1 passed; 0 failed; 0 ignored;',text)
    text=(run/profile/'candidate-default/stdout.log').read_text('utf-8-sig');assert re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;',text)==[('33','0','4','0','0')]
    profiles[profile]=dict(original_expected_reference_failure=1,candidate_reference_passed=1,candidate_default_passed=33,failed=0,ignored=4,actual_candidate_sha256=c['profiles'][profile]['candidate']['executable_sha256'])
ids=[c['pipeline'],*c['process_history']];filters=' OR '.join('ProcessId='+str(j['pid']) for j in ids);cmd="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process -Filter '"+filters+"'|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
probe=subprocess.run(['pwsh','-NoProfile','-Command',cmd],capture_output=True,creationflags=subprocess.CREATE_NO_WINDOW);assert probe.returncode==0 and not probe.stderr;live=json.loads(probe.stdout.decode('utf-8-sig')) if probe.stdout.strip() else[];live=[live] if isinstance(live,dict) else live
assert not any(x['ProcessId']==j['pid'] and abs((datetime.fromisoformat(x['created'])-datetime.fromisoformat(j['creation_utc'])).total_seconds())<.001 for x in live for j in ids)
out=run/'qualified';out.mkdir();(out/'original-owned-CIM.json').write_bytes(probe.stdout)
files={p.relative_to(run).as_posix():p for p in run.rglob('*') if p.is_file() and not p.is_relative_to(out)};files.update({'qualifier.py':Path(__file__),'candidate.json':cp,'candidate-resources.rs':Path(proposal['candidate_files'][name]),'original-resources.rs':R/name,'comparison.json':Path(proposal['comparison']),'original-owned-CIM.json':out/'original-owned-CIM.json'})
hashes={n:sha(p) for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert set(z.namelist())==set(hashes) and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_text_index_order_candidate_only',qualified=True,source_manifest_sha256=c['source_manifest_sha256'],candidate=str(cp),candidate_sha256=sha(cp),profiles=profiles,original_session=a.session,terminal_tool_chunk=a.terminal_chunk,observed_exit_code=0,recorded_owned_identities=len(ids),recorded_original_owned_processes_terminal=True,original_assertions_preserved=True,Runtime_changed=False,source_applied=False,GUI_verified=False,whole_goal_complete=False,raw_files=hashes,raw_archive_sha256=sha(out/'raw.zip'))
save(out/'qualified.json',record);print(json.dumps(dict(qualification=str(out/'qualified.json'),profiles=profiles,terminal_identities=len(ids)),ensure_ascii=False))
