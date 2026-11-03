"""Preserve the original Release operator-budget failure; grant no GUI acceptance."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
case=V/'native-installed-resource-profile-sky-gui/case-20261020-030250-211437-release-rental-hotel-return-reconnect'
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert c['profile']=='Release' and c['scenario']=='rental-hotel-return-reconnect' and c['status']=='failed_preserved'
assert c['recorded_owned_processes_terminal'] and not w['active_workloads']
terminal=next(case.glob('checkpoint-terminal-*/evidence.json'));t=read(terminal)
assert t['original_session_id']==8317 and t['terminal_tool_chunk']=='65f173' and t['observed_actual_exit_code']==1 and t['original_identities_terminal']
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h
assert len(p['compiled_inputs'])==1049
assert c['runtime_deadline_seconds']==900 and c['allowed_cleanup_seconds']==30
assert [x['exit_code']for x in c['processes']]==[1,0,1]
assert c['processes'][2]['terminated_during_failure_cleanup'] is True
assert 'peer sessions did not finish' in (case/'server.stderr.log').read_text('utf-8-sig')
assert 'Owned GUI case did not close within fixture and cleanup budgets' in c['error']
events=read(case/'Sky-events.json')
failure=[e for e in events if e['role']=='b' and e['label']=='hotel-return-failure-readback'];assert len(failure)==1 and 'native peer EOF (Clean)' in failure[0]['tree']
assert any(e['role']=='a' and e['label']=='hotel-return-characters-ready' and 'PeerFixture(測試乙)' in e['tree'] for e in events)
assert not any(e['role']=='b' and e['label']=='hotel-return-characters-ready' and any('角色 Value: PeerFixture2(測試乙)' in line and 'ID: 203' in line for line in e['tree'].splitlines()) for e in events)
frames=next((case/'captures').glob('*/frames.jsonl'));rows=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()]
packets=[(x,bytes.fromhex(x['plain']))for x in rows if x['plain']!='MASKED_C_LOGIN']
login_count=sum(x['plain']=='MASKED_C_LOGIN'for x in rows)
assert login_count<8 and sum(x['direction']=='client_to_server' and b==b'\x10'for x,b in packets)==2
out=case/('failure-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={x.relative_to(case).as_posix():x for x in case.iterdir()if x.is_file()}
files.update({'captures/frames.jsonl':frames,'terminal.json':terminal,'prepared.json':Path(c['prepared'])/'prepared.json','fixture.java':next(Path(c['prepared']).glob('fixture-src/NativePeerHarness.java')),'sealer.py':Path(__file__)})
for role,product in p['products']['Release'].items():assert sha(product['path'])==product['sha256'];files[role+'.exe']=Path(product['path'])
hashes={n:sha(x)for n,x in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='original_Release_GUI_operator_outer_budget_failure_preserved',case=str(case),source_manifest_sha256=source,profile='Release',original_session_id=8317,terminal_tool_chunk='65f173',observed_actual_exit_code=1,original_identities_terminal=True,
    actual_masked_login_frames=login_count,aggregate_C10_requests=2,server_exit_code=1,operator_budget_seconds=900,cleanup_seconds=30,
    observed_B_error='native peer EOF (Clean)',server_error='peer sessions did not finish',pipeline_error=c['error'],
    failed_close_UI_attempt_error='foreground window did not report a process id; fresh Sky list then showed no owned windows; original runner had already killed its exact retained B handle during failure cleanup',
    B_normal_close_claimed=False,product_defect_proven=False,operator_wait_framework_requires_repair=True,stale_room_member_text_in_EOF_tree_observed=True,EOF_cleanup_GUI_requires_followup=True,GUI_qualification_granted=False,GUI_full_flow_verified=False,whole_goal_complete=False,
    raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
save(out/'evidence.json',record)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={}
work='當版Release旅館返回原8317因GUI外層900秒結束而失敗；原exit1及精確owned B清理保留，不授資格。修正僅GUI操作等待框架後重驗Release；Debug與其他當版已通過批次保留，不改遊戲時限／authority／JAR，不交人工。'
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_normal_GUI_hotel_return_failure']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
print(json.dumps(dict(evidence=str(out/'evidence.json'),login_frames=login_count,GUI_qualification_granted=False),ensure_ascii=False))
