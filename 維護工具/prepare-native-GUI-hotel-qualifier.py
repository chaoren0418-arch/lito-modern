"""Derive an independently readable hotel checker from the pinned GUI boilerplate."""
from pathlib import Path
import ast,hashlib,json
T=Path(__file__).resolve().parent
base=T/'qualify-native-keyboard-focus-GUI-rental-entry.py'
body=base.read_text('utf-8-sig')
prefix=body.split('assert sum(body[0]==0xc9 for body in outgoing)==2')[0]
prefix=prefix.replace("c['scenario']=='rental-entry'","c['scenario']=='rental-hotel-entry'")
prefix=prefix.replace('Qualify normal-window keyboard entry only; never grant visual/full-GUI scope.','Qualify normal-window keyboard hotel entry only; never grant visual/full-GUI scope.')
middle=r'''
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
    dialog_observations=[row for row in own if utc(by_label['hotel-talk']['observed_utc'])<=utc(row['observed_utc'])<=utc(by_label['hotel-rent-choice']['observed_utc'])]
    assert any('席巴:'in row['tree'] and '歡迎來到阿里亞的休憩所'in row['tree'] and 'ID: 220'in row['tree']for row in dialog_observations)
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
'''
suffix=body[body.index("rows=[c['pipeline'],*c['processes']]"):]
suffix=suffix.replace('qualified-rental-entry-keyboard','qualified-hotel-entry-keyboard').replace('independently_qualified_normal_keyboard_rental_entry_and_authority_limited','independently_qualified_normal_keyboard_hotel_entry_and_authority_limited')
suffix=suffix.replace('numpad_verified=False','numpad_verified=True').replace("current_normal_GUI_rental_entry_keyboard","current_normal_GUI_hotel_entry_keyboard")
suffix=suffix.replace("work='當版'+profile+'正常兩玩家鍵盤GUI入村／各一步權威移動／背包開關／正常關閉已限定核對，4sessions與4程序皆終止；未授予畫面、滑鼠、IME、多DPI、裝置或完整GUI資格。繼續另一版本及傳送／恢復／旅館全流程，先自行測試才交人工。'","work='當版'+profile+'正常兩帳號鍵盤GUI旅館入房已限定核對：13步路線、原始對話、8／1鑰匙及map503／501、正常關閉；4sessions及4程序皆終止。自然到期、訪客離房、共享鑰匙、畫面／滑鼠／IME／裝置與完整GUI仍待，先自行測試才交人工。'")
suffix=suffix.replace("files.update({'prepared.json':", "files.update({'route.json':route_path,'rooms.json':room_path,'hotel-reference.py':Path(__file__).with_name('native_hotel_catalog_reference.py'),'prepared.json':")
suffix=suffix.replace("record.update(GUI_operator_wait_policy=", "record.update(route_only_compatibility=route_proof,room_catalogue_only_compatibility=room_proof,natural_expiry_verified=False,voluntary_exit_verified=False,shared_keys_verified=False)\nrecord.update(GUI_operator_wait_policy=")
target=T/'qualify-native-keyboard-focus-GUI-hotel-entry.py'
assert not target.exists()
result=prefix+middle+suffix
ast.parse(result)
target.write_text(result,encoding='utf-8',newline='\n')
print(json.dumps(dict(qualifier=str(target),base_sha256=hashlib.sha256(base.read_bytes()).hexdigest().upper(),sha256=hashlib.sha256(target.read_bytes()).hexdigest().upper(),runtime_started=False)))
