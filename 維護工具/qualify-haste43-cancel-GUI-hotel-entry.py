"""Qualify normal-window keyboard hotel entry only; never grant visual/full-GUI scope."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re,subprocess,zipfile
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def data(v):return(json.dumps(v,ensure_ascii=False,indent=2)+'\n').encode('utf-8')
def inventory(body):
    assert body[0]==0x7c;at=2;items={}
    def take(n):
        nonlocal at
        assert 0<=n<=len(body)-at
        value=body[at:at+n];at+=n;return value
    def number(n):return int.from_bytes(take(n),'little')
    for _ in range(body[1]):
        item=number(4);assert item and item not in items
        use_type,charge,icon,blessing,count,identified=[number(n)for n in [1,1,2,1,4,1]]
        end=body.find(b'\0',at,at+256);assert end>=at
        name=take(end-at).decode('cp950');assert take(1)==b'\0'
        detail=list(take(number(1)));assert count>0
        items[item]=dict(id=item,count=count,name=name,use_type=use_type,charge=charge,icon=icon,blessing=blessing,identified=identified,detail=detail)
    assert at==len(body);return items
ap=argparse.ArgumentParser();ap.add_argument('case',type=Path);a=ap.parse_args()
case=a.case.resolve(strict=True);assert case.is_relative_to(V/'native-installed-resource-profile-sky-gui')
c=read(case/'cohort.json');w=read(R/'工作佇列.json');source=c['source_manifest_sha256'];profile=c['profile']
assert c['scenario']=='rental-hotel-entry'and c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert c['recorded_owned_processes_terminal']and all(row['exit_code']==0 and not row.get('terminated_during_failure_cleanup')for row in c['processes'])
assert sha(case/'runner.py')==c['launcher_sha256']
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']and w['current_full_default_library_verified']
assert source==w['current_product_identity']['source_manifest_sha256']=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
p=read(Path(c['prepared'])/'prepared.json');assert sha(Path(c['prepared'])/'prepared.json')==c['prepared_sha256']
assert p['source_manifest_sha256']==source and len(p['compiled_inputs'])==1049
library=Path(p['library']);assert sha(library/'completion-observation.json')==p['library_completion_sha256']
assert read(library/'completion-observation.json')['source_manifest_sha256']==source
normal=read(p['normal_qualification']);assert sha(p['normal_qualification'])==p['normal_qualification_sha256']
assert normal['source_manifest_sha256']==source
for n,h in p['compiled_inputs'].items():assert sha(R/n)==h,n
for n,h in p['fixture_sources'].items():assert sha(n)==h,n
for n,h in p['fixture_origins'].items():assert sha(n)==h,n
for n,h in p['class_files'].items():assert sha(Path(p['classes'])/n)==h,n
assert sha(p['jar'])==p['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
assert sha(p['catalog'])==p['catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
terminals=sorted(case.glob('checkpoint-terminal-*'));assert len(terminals)==1
terminal=read(terminals[0]/'evidence.json');assert terminal['observed_actual_exit_code']==0 and terminal['original_identities_terminal']
assert terminal['case']==str(case)and terminal['owned_processes']==c['processes']
result=read(case/'result.json');assert result['actual_client_sessions']==4 and result['game_sessions']==2
assert result['memberships_empty']and not result['sql_access']and not result['state_persistent']and result['authority_mutations']==0
assert result['classic_world_members_after_disconnect']==result['active_hotel_leases_after_disconnect']==0
events=read(case/'Sky-events.json');times=[datetime.fromisoformat(x['observed_utc'].replace('Z','+00:00'))for x in events];assert times==sorted(times)
trace=[json.loads(line)['state']for line in(case/'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
captures=list((case/'captures').glob('*/frames.jsonl'));assert len(captures)==1
frames=[json.loads(line)for line in captures[0].read_text('utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN'for x in frames)==4
packets=[(x,bytes.fromhex(x['plain']))for x in frames if x['plain']!='MASKED_C_LOGIN']
outgoing=[body for row,body in packets if row['direction']=='client_to_server']

assert sum(body[0]==0xc9 for body in outgoing)==2
assert sum(body[0]==0x08 for body in outgoing)==2
route_path=R/'LiTo381-Modern/tests/fixtures/native-gui/hotel-route.json'
room_path=R/'LiTo381-Modern/tests/fixtures/native-portal-current.json'
from native_hotel_catalog_reference import fixture_reference
route_proof=fixture_reference(route_path,p['jar_sha256'])
room_proof=fixture_reference(room_path,p['jar_sha256'])
route=read(route_path);rooms=read(room_path)['rooms'];assert len(route['steps'])==13
cursor=[32770,32818];expected_moves=[]
for step in route['steps']:
    assert step['before']==cursor and step['heading']in(3,5)
    expected_key={3:0x63,5:0x61}[step['heading']];assert step['key']==expected_key
    delta={3:[1,1],5:[-1,1]}[step['heading']]
    assert step['after']==[v+d for v,d in zip(cursor,delta)]
    expected_moves.append(bytes([0x0a])+int(cursor[0]).to_bytes(2,'little')+int(cursor[1]).to_bytes(2,'little')+bytes([step['heading']]))
    cursor=step['after']
assert cursor==[32779,32831]
assert [body for row,body in packets if row['direction']=='client_to_server'and body[0]==0x0a]==expected_moves*2
roles={};tokens=set()
required=['account','login','source-choice','commit-source','get-characters','character-response','select-character','confirm-loadout','enter-town','world-focus',*['hotel-walk-'+str(i)for i in range(1,14)],'hotel-talk','hotel-rent-choice','hotel-rent-request','hotel-quantity-requested','hotel-quantity-submit','hotel-keys-granted','hotel-enter-choice','hotel-enter-request','hotel-room-loaded','close-normal']
def utc(text):return datetime.fromisoformat(text.replace('Z','+00:00'))
b_start=utc(next(row['observed_utc']for row in events if row['role']=='b'and row['label']=='account'))
for role,account,count,source_count,carry_count,quantity in [('a','PeerFixture',45,28,14,8),('b','PeerFixture2',43,29,11,1)]:
    product=p['products'][profile][role]
    assert sha(product['path'])==sha(product['original_path'])==product['sha256']==normal['normal_mains'][profile]
    own=[row for row in events if row['role']==role];by_label={}
    for row in own:
        previous=by_label.get(row['label'])
        if previous is not None:assert previous['action']is None and row['action']is None
        by_label[row['label']]=row
    assert set(required)<=set(by_label)
    positions=[max(i for i,row in enumerate(own)if row['label']==label)for label in required]
    assert positions==sorted(positions)
    assert all(row['window']['app'].casefold()==('process:'+product['path']).casefold()for row in own)
    assert len({row['window']['id']for row in own})==1
    assert by_label['account']['action']['args']['value']==account
    assert by_label['source-choice']['action']['args']['value']=='測試乙'
    for label in ['login','commit-source','get-characters','select-character','confirm-loadout','enter-town','hotel-talk','hotel-rent-request','hotel-quantity-submit','hotel-enter-request']:
        assert by_label[label]['action']==dict(method='press_key',args={'key':'space'})
    assert by_label['hotel-rent-choice']['action']['args']['value']=='租借房間'
    assert by_label['hotel-enter-choice']['action']['args']['value']=='進入房間'
    assert '請選擇角色。'in by_label['character-response']['tree']
    assert '地圖 500'in by_label['world-focus']['tree']and by_label['world-focus']['focused_element'].startswith('0 視窗')
    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())
    assert 'ID: 318'in by_label['world-focus']['tree']
    assert not any('新密碼'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())
    dialog_observations=[row for row in own if utc(by_label['hotel-talk']['observed_utc'])<=utc(row['observed_utc'])<=utc(by_label['hotel-rent-choice']['observed_utc'])]
    assert any('席巴:'in row['tree'] and '歡迎來到阿里亞的休憩所'in row['tree'] and 'ID: 220'in row['tree']for row in dialog_observations)
    assert all('ID: 318'not in row['tree']for row in dialog_observations)
    assert 'ID: 318'in by_label['hotel-room-loaded']['tree']
    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['hotel-room-loaded']['tree'].splitlines())
    assert '請輸入 1～8 的數量'in by_label['hotel-quantity-requested']['tree']
    for i,step in enumerate(route['steps'],1):
        event=by_label['hotel-walk-'+str(i)]
        assert event['action']==dict(method='press_key',args={'key':{3:'KP_3',5:'KP_1'}[step['heading']]})
        assert event['focused_element'].startswith('0 視窗')
    room=next(row for row in rooms if row['keys_min']<=quantity<=row['keys_max'])
    arrival_text=by_label['hotel-room-loaded']['tree']
    assert f"地圖 {room['map']}"in arrival_text and '世界資源載入失敗'not in arrival_text
    assert f'背包（{count+quantity}）'in by_label['hotel-keys-granted']['tree']and f'背包（{count+quantity}）'in arrival_text
    assert 'ID: 220'not in arrival_text and by_label['hotel-room-loaded']['focused_element'].startswith('0 視窗')
    truths=[row for row in trace if row['test_account']==account and row['source']=='測試乙'and row['phase']=='BATTLE'and row['rental_town']]
    assert len({row['session_identity']for row in truths})==1
    before=next(row for row in truths if row['rental_map']==500 and [row['x'],row['y']]==cursor and row['rental_hotel']is None)
    for step in route['steps']:assert any(row['rental_map']==500 and [row['x'],row['y']]==step['after']for row in truths)
    granted=next(row for row in truths if row['rental_map']==500 and row['rental_hotel']is not None)
    arrived=next(row for row in truths if row['rental_map']==room['map'])
    assert granted['rental_hotel']==arrived['rental_hotel']
    lease=granted['rental_hotel'];assert lease['map']==room['map']and lease['keys']==quantity and lease['token']>0 and lease['token']not in tokens
    tokens.add(lease['token'])
    assert [arrived[k]for k in ['x','y','heading']]==[room[k]for k in ['x','y','heading']]
    assert len(before['native_inventory'])==source_count and len(before['rental_inventory'])==carry_count
    keyed=lambda row,field:{item['id']:item for item in row[field]}
    initial=keyed(before,'rental_inventory');rented=keyed(granted,'rental_inventory');key_ids=set(rented)-set(initial)
    assert len(key_ids)==quantity and {key:item for key,item in rented.items()if key not in key_ids}==initial
    assert all(rented[key]['catalog_id']==40312 and rented[key]['count']==1 and not rented[key]['equipped']for key in key_ids)
    assert keyed(arrived,'rental_inventory')==rented
    for row in [granted,arrived]:
        assert keyed(row,'native_inventory')==keyed(before,'native_inventory')
        assert all(row[k]==before[k]for k in ['hp','mp','shop_points','rental_gems','character','rental_set'])
    start=utc(by_label['account']['observed_utc']);end=b_start if role=='a'else utc(c['finished_utc'])
    role_packets=[(row,b)for row,b in packets if start<=utc(row['time'])<end]
    outgoing_role=[(row,b)for row,b in role_packets if row['direction']=='client_to_server']
    replies=[(row,b)for row,b in outgoing_role if b[0]==0x51];assert len(replies)==1
    reply_row,reply=replies[0];assert len(reply)==11 and int.from_bytes(reply[5:9],'little')==quantity and reply[9:]==b'\0\0'
    requests=[(row,b)for row,b in role_packets if row['direction']=='server_to_client'and b[0]==0x5b and utc(row['time'])<utc(reply_row['time'])]
    request_row,request=requests[-1]
    assert reply[1:5]==request[1:5]and int.from_bytes(request[9:13],'little')==1 and int.from_bytes(request[13:17],'little')==1 and int.from_bytes(request[17:21],'little')==8 and request[21]==0
    reply_seconds=(utc(reply_row['time'])-utc(request_row['time'])).total_seconds();assert 0<=reply_seconds<30
    additions={}
    for row,b in role_packets:
        if row['direction']=='server_to_client'and b[0]==0x16:additions.update(inventory(bytes([0x7c,1])+b[1:]))
    assert set(additions)==key_ids and all(additions[key]['count']==1 for key in key_ids)
    portals=[b for row,b in role_packets if row['direction']=='server_to_client'and b[0]==0x5a]
    expected_portal=bytes([0x5a])+b''.join(int(room[k]).to_bytes(2,'little')for k in ['map','x','y'])
    assert portals==[expected_portal]
    assert sum(b==bytes([0x08])for row,b in outgoing_role)==1
    close=by_label['close-normal'];assert close['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
    child=next(row for row in c['processes']if row['role']==role)
    close_seconds=(utc(child['finished_utc'])-utc(close['input_started_utc'])).total_seconds();assert 0<=close_seconds<=30
    roles[role]=dict(account=account,map_id=room['map'],arrival=[room[k]for k in ['x','y','heading']],quantity=quantity,
        key_ids=sorted(key_ids),lease=lease,initial_GUI_items=count,arrival_GUI_items=count+quantity,accepted_route_steps=13,
        numeric_reply_seconds=reply_seconds,observed_close_seconds=close_seconds,
        earlier_quantity_requests=len(requests)-1,all_raw_observations_preserved=True,natural_expiry_verified=False,voluntary_exit_verified=False,shared_keys_verified=False)
rows=[c['pipeline'],*c['processes']];ids=','.join(str(row['pid'])for row in rows)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(x['ProcessId']==row['pid']and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live for row in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=case/'qualified-hotel-entry-keyboard';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(raw)
files={name:case/name for name in ['cohort.json','runner.py','Sky-events.json','ready.json','result.json','server-trace.jsonl','server.properties','server.stdout.log','server.stderr.log','a.stdout.log','a.stderr.log','b.stdout.log','b.stderr.log']}
files.update({'route.json':route_path,'rooms.json':room_path,'hotel-reference.py':Path(__file__).with_name('native_hotel_catalog_reference.py'),'prepared.json':Path(c['prepared'])/'prepared.json','terminal.json':terminals[0]/'evidence.json','qualifier.py':out/'qualifier.py','original-owned-CIM.json':out/'original-owned-CIM.json','frames.jsonl':captures[0]})
for f in case.glob('*capture*.json'):files[f.name]=f
if(case/'Sky-driver-helpers.js').exists():files['Sky-driver-helpers.js']=case/'Sky-driver-helpers.js'
hashes={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,file_path in files.items():z.write(file_path,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_normal_keyboard_hotel_entry_and_authority_limited',case=str(case),profile=profile,source_manifest_sha256=source,
    original_session=terminal['original_session_id'],terminal_tool_chunk=terminal['terminal_tool_chunk'],observed_exit_code=0,roles=roles,actual_formal_sessions=4,
    recorded_original_owned_identities=4,recorded_original_owned_processes_terminal=True,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,
    multiplayer_simultaneous_visibility_verified=False,numpad_verified=True,world_pixels_verified=False,screenshots_qualified=False,mouse_verified=False,IME_verified=False,multiple_DPI_verified=False,physical_audio_verified=False,GUI_full_flow_verified=False,whole_goal_complete=False)
record.update(route_only_compatibility=route_proof,room_catalogue_only_compatibility=room_proof,natural_expiry_verified=False,voluntary_exit_verified=False,shared_keys_verified=False)
record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True)
record.update(GUI_operator_wait_policy=p['operator_wait_policy'],performance_qualification_granted=False)
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record))
work='當版'+profile+'正常兩帳號鍵盤GUI旅館入房已限定核對：13步路線、原始對話、8／1鑰匙及map503／501、正常關閉；4sessions及4程序皆終止。自然到期、訪客離房、共享鑰匙、畫面／滑鼠／IME／裝置與完整GUI仍待，先自行測試才交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        v=read(path);assert v['current_product_identity']['source_manifest_sha256']==source
        v.setdefault('current_normal_GUI_hotel_entry_keyboard',{})[profile]=record
        v['current_source_goal_validation'].setdefault('G08-player-gui',{}).update(complete=False,state='normal_keyboard_world_entry_partial_full_GUI_pending')
        v.update(next_task=work,next_focus=work,last_goal_turn_evidence=str(out/'evidence.json'),last_goal_turn_classification='progress',goal_complete=False,whole_goal_complete=False)
        v['current_focus']=dict(item='normal_GUI_full_flow_automation',work=work,applies_to_current_source=True,goal_complete=False)
        if n.endswith('next-validation-action.json'):v['next_action']=work
        staged[n]=data(v)
    else:
        first,sep,rest=path.read_text('utf-8-sig').partition('\n\n');assert sep;staged[n]=(date+'：'+work+'\n\n'+rest).encode('utf-8')
with zipfile.ZipFile(out/'before.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n in before:z.write(R/n,n)
with zipfile.ZipFile(out/'before.zip')as z:assert all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in before.items())
assert len(staged)==8 and all(sha(R/n)==h for n,h in before.items())
for n,b in staged.items():(R/n).write_bytes(b)
(out/'readback.json').write_bytes(data(dict(before_files=before,after_files={n:sha(R/n)for n in before},before_archive_sha256=sha(out/'before.zip'),evidence_sha256=sha(out/'evidence.json'))))
print(json.dumps(dict(qualification=str(out/'qualified.json'),profile=profile,actual_formal_sessions=4,GUI_full_flow_verified=False),ensure_ascii=False))
