"""Retain two late GUI submissions without C51 and unavailable pointer; no positive GUI qualification."""
from pathlib import Path
from datetime import datetime,timezone
import hashlib,json,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def save(p,j):Path(p).write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n')
utc=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'))
case=V/'native-installed-resource-profile-sky-gui/case-20261020-032152-540224-release-rental-hotel-return-reconnect'
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert source==w['current_product_identity']['source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
assert c['status']=='failed_preserved' and c['recorded_owned_processes_terminal'] and c['runtime_deadline_seconds']==1800 and not w['active_workloads']
terminal=next(case.glob('checkpoint-terminal-*/evidence.json'));t=read(terminal)
assert t['original_session_id']==35904 and t['terminal_tool_chunk']=='ca7b27' and t['observed_actual_exit_code']==1 and t['original_identities_terminal']
assert [r['exit_code']for r in c['processes']]==[4294967295,0,0]
stop=read(case/'owned-server-stop.json');assert stop['pid']==c['processes'][0]['pid'] and stop['exit_code']==-1 and stop['original_process_handle_retained'] and stop['scope']==str(case)
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h
events=read(case/'Sky-events.json');assert {r['role']for r in events if r['label']=='close-normal'}=={'a','b'}
errors=read(case/'Sky-input-errors.json');assert len(errors)==1 and errors[0]['error']=='coordinate input geometry is unavailable'
frames=next((case/'captures').glob('*/frames.jsonl'));fr=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()]
assert sum(r['plain']=='MASKED_C_LOGIN'for r in fr)==2
packets=[(r,bytes.fromhex(r['plain']))for r in fr if r['plain']!='MASKED_C_LOGIN']
replies=[(r,b)for r,b in packets if r['direction']=='client_to_server'and b[0]==0x51];assert len(replies)==0
measurements=[]
for label in ['hotel-quantity-submit','hotel-expiry-retry-submit']:
    e=next(x for x in events if x['role']=='a'and x['label']==label)
    assert e['action']==dict(method='press_key',args={'key':'space'})
    requests=[(x,v)for x,v in packets if x['direction']=='server_to_client'and v[0]==0x5b and utc(x['time'])<utc(e['observed_utc'])]
    assert requests
    request,body=requests[-1];assert int.from_bytes(body[13:17],'little')==1 and int.from_bytes(body[17:21],'little')==8
    elapsed=(utc(e['observed_utc'])-utc(request['time'])).total_seconds();assert elapsed>=30
    measurements.append(dict(request_utc=request['time'],post_action_observed_utc=e['observed_utc'],observed_elapsed_seconds=elapsed,quantity=8,C51_sent=False,server_rejection_claimed=False))
trace=[json.loads(l)['state']for l in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
truths=[r for r in trace if r['test_account']=='PeerFixture'and r['source']=='測試乙'and r['phase']=='BATTLE'and len(r['native_inventory'])==28 and len(r['rental_inventory'])==14]
assert truths and all(r['rental_map']==500 and r['rental_hotel']is None and not r['rental_hotel_owner_lease_active']for r in truths)
initial=truths[0]
for r in truths:
    for field in ['native_inventory','rental_inventory','hp','mp','shop_points','rental_gems','character','rental_set']:assert r[field]==initial[field],field
assert not any(b[0]==0x10 for r,b in packets if r['direction']=='client_to_server')
out=case/('failure-analysis-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir()
files={r.relative_to(case).as_posix():r for r in case.iterdir()if r.is_file()}
files.update({'captures/frames.jsonl':frames,'terminal.json':terminal,'prepared.json':Path(c['prepared'])/'prepared.json','sealer.py':Path(__file__)})
hashes={n:sha(x)for n,x in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,x in files.items():z.write(x,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='Release_GUI_two_late_quantity_submissions_without_C51_and_pointer_unavailable_preserved',source_manifest_sha256=source,case=str(case),profile='Release',original_session_id=35904,terminal_tool_chunk='ca7b27',observed_actual_exit_code=1,original_identities_terminal=True,actual_formal_sessions=2,quantity_attempts=measurements,C51_sent=0,server_rejection_claimed=False,all_observed_inventory_stats_and_lease_unchanged=True,GUI_A_and_unconnected_B_closed_normally=True,server_stopped_only_verified_owned_original_handle=True,pointer_error='coordinate input geometry is unavailable',original_30_second_quantity_clock_changed=False,positive_hotel_return_acceptance_granted=False,GUI_full_flow_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
save(out/'evidence.json',record)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
staged={};work='當版Release旅館重驗原35904兩次數量操作逾30秒後未送C51，物品／租約不變，可及點擊geometry unavailable；原exit1與兩GUI正常退出、精確owned伺服器清理已封存，不授正向返回資格，也不稱伺服器拒絕。接續獨立控制器96sessions與Release整合，另查數量鍵盤送出及EOF清理；不改原時限，不交人工。'
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j['current_normal_GUI_hotel_return_quantity_failure']=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=('2026-10-20：'+work+'\n\n'+rest).encode('utf-8')
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,payload in staged.items():(R/n).write_bytes(payload)
save(out/'readback.json',dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json')))
print(json.dumps(dict(evidence=str(out/'evidence.json'),quantity_attempts=measurements,positive_hotel_return_acceptance_granted=False),ensure_ascii=False))
