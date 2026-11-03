"""Preserve the original timed-out Debug case without granting GUI acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1]; V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(x):return (json.dumps(x,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
case=V/'native-installed-resource-profile-sky-gui/case-20261020-104045-027037-debug-ordinary-room-service'
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
assert c['profile']=='Debug' and c['scenario']=='ordinary-room-service' and c['status']=='failed_preserved'
assert c['recorded_owned_processes_terminal'] and not w['active_workloads']
terminal=case/'checkpoint-terminal-20261020-105625-062375/evidence.json';t=read(terminal)
assert t['original_session_id']==54314 and t['terminal_tool_chunk']=='a8e9fb' and t['observed_actual_exit_code']==1 and t['original_identities_terminal']
assert c['runtime_deadline_seconds']==900 and c['allowed_cleanup_seconds']==30
assert [x['exit_code'] for x in c['processes']]==[1,0,0] and not any(x.get('terminated_during_failure_cleanup') for x in c['processes'])
events=read(case/'Sky-events.json')
assert all(any(e['role']==role and e['label']=='close-normal' for e in events) for role in ['a','b'])
assert any(e['role']=='a' and e['label']=='room-edited' and 'GUI改名房' in e['tree'] for e in events)
assert not any(e['role']=='a' and e['label']=='leave-room' for e in events)
assert any(e['role']=='a' and e['label']=='outer-deadline-EOF' and 'native peer EOF (Clean)' in e['tree'] for e in events)
assert 'peer sessions did not finish' in (case/'server.stderr.log').read_text('utf-8-sig')
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
assert all(sha(R/n)==h for n,h in p['compiled_inputs'].items())
out=case/('failure-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={x.relative_to(case).as_posix():x for x in case.iterdir() if x.is_file()}
for x in (case/'captures').rglob('*'):
    if x.is_file():files[x.relative_to(case).as_posix()]=x
files.update({'terminal.json':terminal,'prepared.json':Path(c['prepared'])/'prepared.json','sealer.py':Path(__file__)})
hashes={n:sha(x) for n,x in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(out/'raw.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='original_Debug_ordinary_GUI_operator_deadline_failure_preserved',source_manifest_sha256=source,case=str(case),profile='Debug',original_session_id=54314,terminal_tool_chunk='a8e9fb',observed_actual_exit_code=1,original_identities_terminal=True,operator_deadline_seconds=900,cleanup_seconds=30,both_GUI_normal_close=True,server_exit_code=1,owner_leave_not_executed=True,product_defect_proven=False,GUI_qualification_granted=False,GUI_full_flow_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
(out/'evidence.json').write_bytes(data(record))
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED) as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip') as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='Debug普通房間原54314在房主最後離房前達原900秒GUI操作外層期限，server1／pipeline1；兩GUI正常AltF4退出0，四原身分已終止並封存。換隊／準備同步及修改完成僅保留原始觀測，不授完整資格。保留Release與其他已核對資格；先完成剩餘兩項效果資源及隔離候選元件，再修正操作框架並新批次驗證未完成Debug普通房間，遊戲／協定時限不變，不交人工。'
staged={}
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_normal_GUI_ordinary_operator_deadline_failure']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n) for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(evidence=str(out/'evidence.json'),GUI_qualification_granted=False),ensure_ascii=False))
