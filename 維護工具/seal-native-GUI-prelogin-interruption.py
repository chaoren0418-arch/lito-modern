"""Preserve a zero-session interrupted GUI attempt; attribute no cause or pass."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['status']=='failed_preserved'and c['recorded_owned_processes_terminal']and not w['active_workloads']and [r['exit_code']for r in c['processes']]==[4294967295,0,0]
assert all(not r.get('terminated_during_failure_cleanup')for r in c['processes'])
terminals=list(case.glob('checkpoint-terminal-*/evidence.json'));assert len(terminals)==1;t=read(terminals[0]);assert t['observed_actual_exit_code']==1 and t['original_identities_terminal']
errors=read(case/'Sky-input-errors.json');assert any('user input was detected'in e.get('actual_error','')for e in errors)and any(e.get('matching_owned_windows')==0 for e in errors)
frames=list((case/'captures').glob('*/frames.jsonl'));assert all(not p.read_text('utf-8-sig').strip()for p in frames)
assert not(case/'server-trace.jsonl').read_text('utf-8-sig').strip()
stop=read(case/'owned-server-stop.json');assert stop['pid']==c['processes'][0]['pid']and stop['original_process_handle_retained']and stop['exit_code']==-1 and stop['scope']==str(case)
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']and p['source_manifest_sha256']==source
assert len(p['compiled_inputs'])==1049 and all(sha(R/n)==h for n,h in p['compiled_inputs'].items())
out=case/('interruption-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={f.relative_to(case).as_posix():f for f in case.iterdir()if f.is_file()};files.update({'reader.py':Path(__file__),'terminal.json':terminals[0],'prepared.json':Path(c['prepared'])/'prepared.json'})
for role,product in p['products'][c['profile']].items():assert sha(product['path'])==sha(product['original_path'])==product['sha256'];files[role+'.exe']=Path(product['path'])
hashes={n:sha(f)for n,f in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,f in files.items():z.write(f,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='normal_GUI_prelogin_interruption_preserved_no_acceptance',source_manifest_sha256=source,profile=c['profile'],case=str(case),original_session_id=t['original_session_id'],terminal_tool_chunk=t['terminal_tool_chunk'],observed_actual_exit_code=1,actual_formal_sessions=0,original_GUI_process_exit_codes=[0,0],normal_GUI_close_by_automation_claimed=False,external_input_notice_observed=True,cause_proven=False,product_defect_proven=False,original_identities_terminal=True,only_exact_owned_waiting_server_stopped=True,GUI_qualification_granted=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'evidence.json').write_bytes(data(record));before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版'+c['profile']+' GUI原'+str(t['original_session_id'])+'連線前中斷，0sessions；Space後未觀測交握，兩原GUI先後exit0並有external-input通知，原因未證明。僅停止等待的精確owned server，原wrapper1與四程序終止已核對封存；新批次重驗，不把中斷算通過，不重跑已通過library／81資源／API16。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_normal_GUI_prelogin_interruption']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))));print(json.dumps(dict(evidence=str(out/'evidence.json'),actual_formal_sessions=0,GUI_acceptance_granted=False),ensure_ascii=False))
