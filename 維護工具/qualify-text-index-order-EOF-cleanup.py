"""Read back a declared owned EOF fault; grant only the observed cleanup scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(j):return(json.dumps(j,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
utc=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'))
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args();case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
assert c['scenario']=='rental-entry'and c['status']=='failed_preserved'and c['recorded_owned_processes_terminal']and not w['active_workloads']
assert [x['exit_code']for x in c['processes']]==[4294967295,0,0]and all(not x.get('terminated_during_failure_cleanup')for x in c['processes'])
terminals=list(case.glob('checkpoint-terminal-*/evidence.json'));assert len(terminals)==1;t=read(terminals[0]);assert t['original_identities_terminal']and t['observed_actual_exit_code']==1
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']and p['source_manifest_sha256']==source
assert len(p['compiled_inputs'])==1049 and all(sha(R/n)==h for n,h in p['compiled_inputs'].items())
for group in ['fixture_sources','fixture_origins']:
    for n,h in p[group].items():assert sha(n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
assert sha(p['jar'])==p['jar_sha256']and sha(p['catalog'])==p['catalog_sha256']
normal=read(p['normal_qualification']);assert sha(p['normal_qualification'])==p['normal_qualification_sha256']and normal['source_manifest_sha256']==source and normal['passed_per_profile']==19
lib=Path(p['library'])/'completion-observation.json';assert sha(lib)==p['library_completion_sha256']and read(lib)['source_manifest_sha256']==source and w['current_full_default_library_verified']
plan=read(case/'EOF-negative-plan.json');inject=read(case/'owned-EOF-injection.json');assert plan['source_manifest_sha256']==source and plan['expected_pipeline_exit_code']==1 and not plan['positive_scenario_qualification_requested']
assert inject['status']=='intentional_owned_server_EOF_injection'and inject['original_process_handle_retained']and inject['plan_sha256']==sha(case/'EOF-negative-plan.json')and inject['exit_code']==-1
server=c['processes'][0];assert inject['pid']==server['pid']and abs((utc(inject['creation_utc'])-utc(server['creation_utc'])).total_seconds())<.001
events=read(case/'Sky-events.json');own=[e for e in events if e['role']=='a'];by={}
for e in own:
    if e['label']in by:assert by[e['label']]['action']is None and e['action']is None
    by[e['label']]=e
assert utc(plan['declared_utc'])<utc(events[0]['observed_utc'])and not any(e['role']=='b'and e['label']=='login'for e in events)
required=['eof-world-before','eof-inventory-before','eof-world-restored-before','eof-after','eof-after-stable','close-normal'];assert set(required)<=set(by)
assert [own.index(by[label])for label in required]==sorted(own.index(by[label])for label in required)
for label in plan['required_labels']:assert by[label]['action']is None
assert '地圖 500'in by['eof-world-before']['tree']and 'ID: 318'in by['eof-world-before']['tree']and '背包（45）'in by['eof-world-before']['tree']
assert 'ID: 209'in by['eof-inventory-before']['tree']and '清單項目'in by['eof-inventory-before']['tree']
assert '地圖 500'in by['eof-world-restored-before']['tree']and 'ID: 318'in by['eof-world-restored-before']['tree']
assert utc(inject['pre_fault_observed_utc'])==utc(by['eof-world-restored-before']['observed_utc'])and utc(inject['started_utc'])>=utc(inject['pre_fault_observed_utc'])
assert utc(by['eof-after']['observed_utc'])>=utc(inject['finished_utc'])and(utc(by['eof-after-stable']['observed_utc'])-utc(by['eof-after']['observed_utc'])).total_seconds()>=.25
for label in ['eof-after','eof-after-stable']:
    tree=by[label]['tree'];assert '連線失敗：'in tree and '大廳房間'in tree and '房間成員'not in tree and '(測試乙)'not in tree
    assert not re.search(r'地圖 50[0-3]\b',tree)
    assert not any(re.search(r'\bID:\s*'+str(i)+r'\b',tree)for i in [208,209,217,219,220,222,224,318])
    for i in [103,104,105,107,108,109,110,111,112,114,118]:
        rows=[line for line in tree.splitlines()if not line.startswith('The focused')and re.search(r'\bID:\s*'+str(i)+r'\b',line)];assert len(rows)==1 and '(disabled' in rows[0],(label,i,rows)
    assert '背包（0）'in tree
for role in ['a','b']:
    closes=[e for e in events if e['role']==role and e['label']=='close-normal'];assert len(closes)==1 and closes[0]['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
    product=p['products'][c['profile']][role];assert sha(product['path'])==sha(product['original_path'])==product['sha256']==normal['normal_mains'][c['profile']]
frames=next((case/'captures').glob('*/frames.jsonl'));rows=[json.loads(l)for l in frames.read_text('utf-8-sig').splitlines()];assert sum(x['plain']=='MASKED_C_LOGIN'for x in rows)==2
outgoing=[bytes.fromhex(x['plain'])for x in rows if x['direction']=='client_to_server'and x['plain']!='MASKED_C_LOGIN']
assert [b[1:].rstrip(b'\0').decode('cp950')for b in outgoing if b[0]==0xc8]==['測試乙']*2
assert [b[1:].rstrip(b'\0').decode('cp950')for b in outgoing if b[0]==0xc9]==['PeerFixture(測試乙)']
# RentalReady enters the advertised room through one D9/F000/F5/CF arm.
# It precedes the observed world and is distinct from an action after EOF.
assert not any(b[0]in [0x10,0x51]for b in outgoing)
packets=[(row,bytes.fromhex(row['plain']))for row in rows if row['plain']!='MASKED_C_LOGIN']
arms=[i for i,(row,body)in enumerate(packets)if row['direction']=='client_to_server'and body[0]==0xd9]
assert len(arms)==1
arm_at=arms[0];arm=packets[arm_at:arm_at+4];assert len(arm)==4
states=[json.loads(line)['state']for line in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
town=[state for state in states if state.get('test_account')=='PeerFixture'and state.get('rental_town')]
assert town and all(state['source']=='測試乙'and state['rental_map']==500 and state['rental_set']==1 for state in town)
room_ids={state['room']for state in town};assert len(room_ids)==1
room_id=next(iter(room_ids));assert room_id>0
assert [(row['direction'],body)for row,body in arm]==[
    ('client_to_server',bytes([0xd9])),('server_to_client',bytes([0xf0,0])),
    ('server_to_client',bytes([0xf5])),('client_to_server',bytes([0xcf])+room_id.to_bytes(4,'little')+bytes([0]))]
assert [body for body in outgoing if body[0]==0xcf]==[arm[-1][1]]
assert [body for body in outgoing if body[0]==0xca]==[bytes([0xca,1])]
assert by['enter-town']['action']==dict(method='press_key',args={'key':'space'})
assert utc(by['enter-town-focus']['observed_utc'])<=utc(arm[0][0]['time'])<=utc(arm[-1][0]['time'])<=utc(by['eof-world-before']['observed_utc'])
assert not any(row['direction']=='client_to_server'and body[0]in [0x10,0x51,0xd9]and utc(row['time'])>=utc(by['eof-world-before']['observed_utc'])for row,body in packets)
assert not any(row['direction']=='client_to_server'and utc(row['time'])>=utc(inject['finished_utc'])for row,body in packets)
login_source=R/'LiTo381-Modern/src/modern_lito/native_login.rs'
assert sha(login_source)==p['compiled_inputs']['LiTo381-Modern/src/modern_lito/native_login.rs']
login_text=login_source.read_text('utf-8')
assert 'pub fn join_rental_room(&mut self,index:usize)->Result<()>'in login_text
assert 'self.state.phase=LoginPhase::RentalArmPending'in login_text and 'self.state.phase=LoginPhase::RentalAwaitPermission'in login_text
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=case/'qualified-EOF-cleanup-keyboard';out.mkdir()
files={x.relative_to(case).as_posix():x for x in case.iterdir()if x.is_file()};files.update({'captures/frames.jsonl':frames,'terminal.json':terminals[0],'prepared.json':Path(c['prepared'])/'prepared.json','normal-qualification.json':Path(p['normal_qualification']),'library-completion.json':lib,'qualifier.py':Path(__file__),'normal-native-login.rs':login_source})
for role,product in p['products'][c['profile']].items():files[role+'.exe']=Path(product['path'])
hashes={n:sha(path)for n,path in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED,compresslevel=1)as z:
    for n,path in files.items():z.write(path,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_normal_GUI_owned_EOF_cleanup_limited',source_manifest_sha256=source,profile=c['profile'],case=str(case),actual_formal_sessions=2,original_session_id=t['original_session_id'],observed_actual_exit_code=1,original_identities_terminal=True,intentional_negative_test=True,normal_positive_case_pass_claimed=False,GUI_EOF_cleanup_observed=True,stale_world_rooms_inventory_services_and_character_not_exposed=True,normal_GUI_closes=2,credential_memory_erasure_claimed=False,pixel_verified=False,GUI_full_flow_verified=False,whole_goal_complete=False,raw_archive=str(out/'raw.zip'),raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes)
record.update(exact_pre_world_rental_arm_D9_F000_F5_CF_verified=True,advertised_room_id=room_id,no_C10_C51_anywhere=True,no_post_world_restart_quantity_or_quit=True,no_post_fault_client_packets=True,normal_login_source_sha256=sha(login_source),prior_reader_failure_preserved=[str(path)for path in case.glob('reader-global-D9-rejection-*/evidence.json')])
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'));(out/'evidence.json').write_bytes(data(record))
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
work='當版'+c['profile']+'正常GUI自有EOF負向清理已獨立核對：正常入世界／背包後停止精確owned測試server；兩次純觀測確認舊世界、房間、物品、服務及角色不再顯示，兩GUI正常關閉。原wrapper1為預先宣告的故障，僅授EOF負向範圍；其他正向GUI及G01–G12仍待，不交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        j=read(path);j.setdefault('current_normal_GUI_EOF_cleanup',{})[c['profile']]=record;j.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False);j['current_focus']=dict(item='normal_GUI_full_flow_automation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):j['next_action']=work
        staged[n]=data(j)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
assert all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=c['profile'],actual_formal_sessions=2,GUI_full_flow_verified=False),ensure_ascii=False))
