"""Qualify normal-window keyboard entry only; never grant visual/full-GUI scope."""
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
assert c['scenario']=='rental-entry'and c['status']=='runtime_exited_original_zero_independent_GUI_qualification_pending'
assert c['recorded_owned_processes_terminal']and all(row['exit_code']==0 and not row.get('terminated_during_failure_cleanup')for row in c['processes'])
assert sha(case/'runner.py')==c['launcher_sha256']
assert not w['active_workloads']and not w['source_and_corresponding_QA_frozen']and w['current_full_default_library_verified']
assert source==w['current_product_identity']['source_manifest_sha256']=='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628'
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
source_requests=[body for body in outgoing if body[0]==0xc8]
assert len(source_requests)==4 and all(body[1:].rstrip(b'\0').decode('cp950')=='測試乙'for body in source_requests)
moves=[body for body in outgoing if body[0]==0x0a]
expected_moves=[bytes([0x0a])+int(32770).to_bytes(2,'little')+int(32818).to_bytes(2,'little')+bytes([1])for _ in range(2)]
assert sorted(moves)==sorted(expected_moves)
batch_rows=[(row,inventory(body))for row,body in packets if row['direction']=='server_to_client'and body[0]==0x7c]
roles={}
required=['account','login','source-choice','commit-source','get-characters','character-response','select-character','rental-directory-ready','confirm-loadout','enter-town','world-focus','walk-1','world-after-walk','inventory-ready','scene-restored','close-normal']
b_start=datetime.fromisoformat(next(row['observed_utc']for row in events if row['role']=='b'and row['label']=='account').replace('Z','+00:00'))
for role,account,count,source_count,carry_count,key,after in [('a','PeerFixture',45,28,14,'Up',[32771,32817]),('b','PeerFixture2',43,29,11,'Up',[32771,32817])]:
    product=p['products'][profile][role];assert sha(product['path'])==sha(product['original_path'])==product['sha256']==normal['normal_mains'][profile]
    own=[row for row in events if row['role']==role];by_label={}
    for row in own:
        previous=by_label.get(row['label'])
        # Re-observation after asynchronous loading or a context interruption
        # does not repeat input. Preserve all raw events and reject repeated
        # action labels; the final readback must independently meet semantics.
        if previous is not None:assert previous['action'] is None and row['action'] is None
        by_label[row['label']]=row
    assert set(required)<=set(by_label)
    positions=[max(i for i,row in enumerate(own)if row['label']==label)for label in required];assert positions==sorted(positions)
    assert all(row['window']['app'].casefold()==('process:'+product['path']).casefold()for row in own)
    assert len({row['window']['id']for row in own})==1
    assert by_label['account']['action']['method']=='set_value'
    assert by_label['account']['action']['args']['value']==account
    assert by_label['source-choice']['action']['args']['value']=='測試乙'
    rental_entry_tree=by_label['rental-directory-ready']['tree']
    assert '確認配裝'in rental_entry_tree and '租裝服務：'in rental_entry_tree
    assert not re.search(r'\bID: (?:103|203|304)\b',rental_entry_tree), 'ordinary channel controls exposed at rental entry'
    for label in ['login','commit-source','get-characters','select-character','confirm-loadout','enter-town']:
        assert by_label[label]['action']==dict(method='press_key',args={'key':'space'})
    assert '請選擇角色。'in by_label['character-response']['tree']
    assert account+'(測試乙)'in by_label['character-response']['tree']
    assert all(account+'(測試甲)'not in row['tree']for row in own)
    for label in ['world-focus','world-after-walk','scene-restored']:
        assert '地圖 500'in by_label[label]['tree']and'世界資源載入失敗'not in by_label[label]['tree']
    assert by_label['world-focus']['focused_element'].startswith('0 視窗')
    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())
    assert 'ID: 318'in by_label['world-focus']['tree']
    assert not any('新密碼'in line and 'ID: 208'in line for line in by_label['world-focus']['tree'].splitlines())
    assert by_label['walk-1']['action']==dict(method='press_key',args={'key':key})
    assert '關閉背包'in by_label['inventory-ready']['tree']and f'背包（{count}）'in by_label['scene-restored']['tree']
    assert 'ID: 318'not in by_label['inventory-ready']['tree']and 'ID: 318'in by_label['scene-restored']['tree']
    assert any('技能 Value:'in line and 'ID: 208'in line for line in by_label['scene-restored']['tree'].splitlines())
    town=[row for row in trace if row['test_account']==account and row['source']=='測試乙'and row['phase']=='BATTLE'and row['rental_town']and row['rental_map']==500]
    assert town and any([row['x'],row['y']]==[32770,32818]for row in town)and any([row['x'],row['y']]==after for row in town)
    assert len({row['session_identity']for row in town})==1
    truths=[row for row in town if len(row['rental_inventory'])==carry_count and len(row['native_inventory'])==source_count]
    assert truths
    start=datetime.fromisoformat(by_label['account']['observed_utc'].replace('Z','+00:00'))
    end=b_start if role=='a'else datetime.fromisoformat(c['finished_utc'])
    received={}
    for frame,batch in batch_rows:
        if start<=datetime.fromisoformat(frame['time'].replace('Z','+00:00'))<end:received.update(batch)
    assert len(received)==count
    for field,n in [('native_inventory',source_count),('rental_inventory',carry_count)]:
        expected={item['id']:item['count']for item in truths[-1][field]};assert len(expected)==n
        assert all(item in received and received[item]['count']==quantity for item,quantity in expected.items())
    own_packets=[(row,body)for row,body in packets if start<=datetime.fromisoformat(row['time'].replace('Z','+00:00'))<end]
    c8=[row for row,body in own_packets if row['direction']=='client_to_server'and body[0]==0xc8]
    directory_rows=[row for row,body in own_packets if row['direction']=='server_to_client'and row['note'].startswith('S_LITO_CHARACTER_INFO directory-row name='+account+'(')]
    assert len(c8)==len(directory_rows)==2
    # Account entry follows LOGIN connection/version, so that UI time boundary
    # cannot assign version frames. GAME negotiation is uniquely bounded by
    # this account's LOGIN source choice and its first character directory.
    versions=[row for row,body in packets if row['direction']=='client_to_server'and body[0]==0x00
        and frames.index(c8[0])<frames.index(row)<frames.index(directory_rows[0])]
    assert len(versions)==1;version=versions[0]
    stamp=lambda row:datetime.fromisoformat(row['time'].replace('Z','+00:00'))
    # Capture order distinguishes frames even when the system clock returns
    # the same timestamp for a request and its immediate response.
    sequence=[c8[0],version,directory_rows[0],c8[1],directory_rows[1]]
    indices=[frames.index(row)for row in sequence]
    assert indices==sorted(indices)and len(set(indices))==len(indices)
    stamps=[stamp(row)for row in sequence];assert stamps==sorted(stamps)
    assert 'classic-source=測試乙'in directory_rows[1]['note']
    assert (stamp(directory_rows[1])-stamp(c8[1])).total_seconds()<8
    elapsed=(stamp(version)-stamp(c8[0])).total_seconds()
    if role=='b':
        assert elapsed>30 and 'classic-source=測試甲'in directory_rows[0]['note']
        assert 'handoff-wait-ready'in by_label and 'handoff-wait-finished'in by_label
        wait_start=datetime.fromisoformat(by_label['handoff-wait-ready']['observed_utc'].replace('Z','+00:00'))
        wait_end=datetime.fromisoformat(by_label['handoff-wait-finished']['observed_utc'].replace('Z','+00:00'))
        assert (wait_end-wait_start).total_seconds()>30
    else:
        assert 0<=elapsed<30 and 'classic-source=測試乙'in directory_rows[0]['note'], 'quick branch must retain the original 30s source handoff'
    source_confirmation=dict(initial_selection_to_GAME_version_seconds=elapsed,public_C8_requests=2,
        first_wire_directory_source=('測試甲'if 'classic-source=測試甲'in directory_rows[0]['note']else'測試乙'),
        confirmed_wire_directory_source='測試乙',wrong_source_directory_observed_in_UI=False,
        natural_initial_handoff_expiry_exercised=role=='b',server_TTL_unchanged=True)
    close=by_label['close-normal'];assert close['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
    child=next(row for row in c['processes']if row['role']==role)
    close_seconds=(datetime.fromisoformat(child['finished_utc'])-datetime.fromisoformat(close['input_started_utc'].replace('Z','+00:00'))).total_seconds()
    assert 0<=close_seconds<=30
    roles[role]=dict(account=account,source='測試乙',normal_exe_sha256=product['sha256'],map_id=500,accepted_movement=after,
        initial_GAME_source_confirmation=source_confirmation,
        GUI_inventory_items=count,native_source_items=source_count,rental_carry_subset=carry_count,
        inventory_wire_records=received,inventory_panel_toggled=True,observed_close_seconds=close_seconds,world_pixels_verified=False)
rows=[c['pipeline'],*c['processes']];ids=','.join(str(row['pid'])for row in rows)
command="[Console]::OutputEncoding=[Text.UTF8Encoding]::new($false);@(Get-CimInstance Win32_Process|Where-Object {$_.ProcessId -in @("+ids+")}|Select-Object ProcessId,@{n='created';e={$_.CreationDate.ToUniversalTime().ToString('o')}})|ConvertTo-Json -Compress"
raw=subprocess.check_output(['pwsh','-NoProfile','-Command',command]);live=json.loads(raw.decode('utf-8-sig'))if raw.strip()else[];live=[live]if isinstance(live,dict)else live
assert not any(x['ProcessId']==row['pid']and abs((datetime.fromisoformat(x['created'].replace('Z','+00:00'))-datetime.fromisoformat(row['creation_utc'])).total_seconds())<.001 for x in live for row in rows)
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
out=case/'qualified-rental-entry-keyboard';out.mkdir();(out/'qualifier.py').write_bytes(Path(__file__).read_bytes());(out/'original-owned-CIM.json').write_bytes(raw)
files={name:case/name for name in ['cohort.json','runner.py','Sky-events.json','ready.json','result.json','server-trace.jsonl','server.properties','server.stdout.log','server.stderr.log','a.stdout.log','a.stderr.log','b.stdout.log','b.stderr.log']}
files.update({'prepared.json':Path(c['prepared'])/'prepared.json','terminal.json':terminals[0]/'evidence.json','qualifier.py':out/'qualifier.py','original-owned-CIM.json':out/'original-owned-CIM.json','frames.jsonl':captures[0]})
for f in case.glob('*capture*.json'):files[f.name]=f
if(case/'Sky-driver-helpers.js').exists():files['Sky-driver-helpers.js']=case/'Sky-driver-helpers.js'
hashes={n:sha(p)for n,p in files.items()}
with zipfile.ZipFile(out/'raw.zip','x',zipfile.ZIP_DEFLATED)as z:
    for n,file_path in files.items():z.write(file_path,n)
with zipfile.ZipFile(out/'raw.zip')as z:assert set(z.namelist())==set(hashes)and all(hashlib.sha256(z.read(n)).hexdigest().upper()==h for n,h in hashes.items())
record=dict(status='independently_qualified_normal_keyboard_rental_entry_and_authority_limited',case=str(case),profile=profile,source_manifest_sha256=source,
    original_session=terminal['original_session_id'],terminal_tool_chunk=terminal['terminal_tool_chunk'],observed_exit_code=0,roles=roles,actual_formal_sessions=4,
    recorded_original_owned_identities=4,recorded_original_owned_processes_terminal=True,raw_archive_sha256=sha(out/'raw.zip'),raw_files=hashes,
    multiplayer_simultaneous_visibility_verified=False,numpad_verified=False,world_pixels_verified=False,screenshots_qualified=False,mouse_verified=False,IME_verified=False,multiple_DPI_verified=False,physical_audio_verified=False,GUI_full_flow_verified=False,whole_goal_complete=False)
record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True)
record.update(initial_GAME_public_C8_confirmation_verified=True,natural_initial_source_handoff_expiry_verified=True,server_source_handoff_TTL_seconds=30,staged_rental_recovery_verified_by_this_case=False)
record.update(rental_ordinary_channel_controls_hidden_verified=True,rental_API_no_wire_guard_verified_by_this_case=False)
record.update(GUI_operator_wait_policy=p['operator_wait_policy'],performance_qualification_granted=False)
(out/'qualified.json').write_bytes(data(record));record.update(qualification=str(out/'qualified.json'),qualification_sha256=sha(out/'qualified.json'))
(out/'evidence.json').write_bytes(data(record))
work='當版'+profile+'正常兩玩家鍵盤GUI入村／各一步權威移動／背包開關／正常關閉已限定核對，4sessions與4程序皆終止；未授予畫面、滑鼠、IME、多DPI、裝置或完整GUI資格。繼續另一版本及傳送／恢復／旅館全流程，先自行測試才交人工。'
staged={};date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
for n in before:
    path=R/n
    if path.suffix=='.json':
        v=read(path);assert v['current_product_identity']['source_manifest_sha256']==source
        v.setdefault('current_normal_GUI_rental_entry_keyboard',{})[profile]=record
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
