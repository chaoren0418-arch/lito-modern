"""Extend the current hotel entry reader with owner C10 return evidence.

All entry, original route, quantity, key and inventory assertions are retained.
This prepares a reader only; no runtime/UI or qualification is executed.
"""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
origin=T/'qualify-native-rental-service-entry-return-GUI-hotel-entry.py'
target=T/'qualify-native-rental-service-entry-return-GUI-hotel-return.py'
assert not target.exists()
contract=V/'native-hotel-return-contract/20261019-192705-656308/readback.json'
proof=json.loads(contract.read_text('utf-8-sig'))
assert proof['status']=='sealed_formal75_hotel_C10_departure_contract_static_only'and not proof['actual_runtime_executed']
assert sha(proof['selected_methods'])==proof['selected_methods_sha256']
body=origin.read_text('utf-8-sig')
def replace(old,new):
    global body
    assert body.count(old)==1,old
    body=body.replace(old,new)
replace('Qualify normal-window keyboard hotel entry only; never grant visual/full-GUI scope.',
    'Qualify normal-window hotel entry and owner C10 return; never visual/full-GUI scope.')
replace("assert sum(body[0]==0x08 for body in outgoing)==2", """assert sum(body[0]==0x08 for body in outgoing)==2
assert sum(body==bytes([0x10])for body in outgoing)==2
source_requests=[body for body in outgoing if body[0]==0xc8]
assert len(source_requests)==6 and all(body[1:].rstrip(b'\\0').decode('cp950')=='測試乙'for body in source_requests)
contract_path=R/'驗證紀錄/current/native-hotel-return-contract/20261019-192705-656308/readback.json'
contract=read(contract_path)
assert contract['formal_jar_sha256']==p['jar_sha256']and sha(contract['selected_methods'])==contract['selected_methods_sha256']
""")
replace("'hotel-enter-request','hotel-room-loaded','close-normal']",
    "'hotel-enter-request','hotel-room-loaded','hotel-return-focus','hotel-return','hotel-return-characters-ready','close-normal']")
replace("    assert '請選擇角色。'in by_label['character-response']['tree']", """    assert '請選擇角色。'in by_label['character-response']['tree']
    assert account+'(測試乙)'in by_label['character-response']['tree']
    assert all(account+'(測試甲)'not in row['tree']for row in own)
""")
replace("    close=by_label['close-normal'];assert close['action']==dict(method='press_key',args={'key':'Alt_L+F4'})", """    return_buttons=[line for line in arrival_text.splitlines()if re.search(r'\\bID:\\s*107\\b',line)]
    assert len(return_buttons)==1 and '返回選角'in return_buttons[0]and 'disabled'not in return_buttons[0]
    assert not re.search(r'\\bID:\\s*129\\b',arrival_text), 'hotel NPC toggle overlaps return operation'
    focus=by_label['hotel-return-focus']
    assert focus['action']is None and 'ID: 107'in focus['focused_element']and '返回選角'in focus['focused_element']
    assert by_label['hotel-return']['action']==dict(method='press_key',args={'key':'space'})
    returned_tree=by_label['hotel-return-characters-ready']['tree']
    assert '請選擇角色。'in returned_tree and account+'(測試乙)'in returned_tree
    assert 'ID: 318'not in returned_tree and 'ID: 220'not in returned_tree
    assert not re.search(r'地圖 50[0-3]\\b',returned_tree), 'old rental/hotel world still exposed'
    own_source=[row for row,b in outgoing_role if b[0]==0xc8]
    own_select=[row for row,b in outgoing_role if b[0]==0xc9]
    own_restart=[row for row,b in outgoing_role if b==bytes([0x10])]
    directory_rows=[row for row,b in role_packets if row['direction']=='server_to_client'and row['note'].startswith('S_LITO_CHARACTER_INFO directory-row name='+account+'(')]
    assert len(own_source)==3 and len(own_select)==len(own_restart)==1 and len(directory_rows)==3
    assert all('classic-source=測試乙'in row['note']for row in directory_rows[1:])
    versions=[row for row,b in role_packets if row['direction']=='client_to_server'and b[0]==0x00
        and frames.index(own_source[0])<frames.index(row)<frames.index(directory_rows[0])]
    assert len(versions)==1
    def next_frame(direction,payload,after):
        matches=[row for row,b in role_packets if row['direction']==direction and b==payload and frames.index(row)>frames.index(after)]
        assert len(matches)==1,(direction,payload.hex())
        return matches[0]
    restart=own_restart[0]
    ack=next_frame('server_to_client',bytes([0x65]),restart)
    quit_request=next_frame('client_to_server',bytes([0xd9]),ack)
    quit_ack=next_frame('server_to_client',bytes([0xf0,0]),quit_request)
    leave=next_frame('client_to_server',bytes([0xd0]),quit_ack)
    refresh=next_frame('server_to_client',bytes([0xf5]),leave)
    chain=[own_source[0],versions[0],directory_rows[0],own_source[1],directory_rows[1],own_select[0],restart,ack,quit_request,quit_ack,leave,refresh,own_source[2],directory_rows[2]]
    order=[frames.index(row)for row in chain];assert order==sorted(order)and len(order)==len(set(order))
    times=[utc(row['time'])for row in chain];assert times==sorted(times)
    return_seconds=(utc(directory_rows[2]['time'])-utc(restart['time'])).total_seconds()
    assert 0<=return_seconds<8
    assert utc(by_label['hotel-room-loaded']['observed_utc'])<utc(restart['time'])<utc(by_label['hotel-return-characters-ready']['observed_utc'])
    removed=[int.from_bytes(b[1:5],'little')for row,b in role_packets if row['direction']=='server_to_client'
        and b[0]==0x17 and len(b)==5 and frames.index(restart)<frames.index(row)<frames.index(directory_rows[2])]
    assert key_ids<=set(removed)and set(rented)<=set(removed), 'hotel/rental credentials retained after C10'
    arrival_index=trace.index(arrived)
    departure=[row for row in trace[arrival_index+1:]if row.get('session_identity')==arrived['session_identity']
        and row.get('test_account')==account and row.get('source')=='測試乙'and row.get('phase')=='LOBBY'
        and not row.get('rental_town')and row.get('rental_hotel')is None and row.get('room')==0 and not row.get('rental_inventory')]
    assert departure, 'no passive owner lease/rental cleanup before close'
    close=by_label['close-normal'];assert close['action']==dict(method='press_key',args={'key':'Alt_L+F4'})
""")
replace("        earlier_quantity_requests=len(requests)-1,all_raw_observations_preserved=True,natural_expiry_verified=False,voluntary_exit_verified=False,shared_keys_verified=False)",
    "        earlier_quantity_requests=len(requests)-1,all_raw_observations_preserved=True,natural_expiry_verified=False,voluntary_exit_verified=False,shared_keys_verified=False,hotel_owner_C10_return_verified=True,owner_lease_and_credentials_removed=True,return_to_source_character_selection_verified=True,return_wire_seconds=return_seconds,removed_wire_object_ids=removed)")
replace("out=case/'qualified-hotel-entry-keyboard'", "out=case/'qualified-hotel-return-keyboard'")
replace("hashes={n:sha(p)for n,p in files.items()}", "files.update({'hotel-return-contract.json':contract_path,'hotel-return-selected-methods.json':Path(contract['selected_methods'])})\nhashes={n:sha(p)for n,p in files.items()}")
replace("status='independently_qualified_normal_keyboard_hotel_entry_and_authority_limited'",
    "status='independently_qualified_normal_keyboard_hotel_entry_and_owner_return_limited'")
replace("record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True)",
    "record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True,hotel_entry_scope_also_qualified=True,hotel_owner_C10_return_verified=True,hotel_guest_departure_verified=False,shared_key_guest_invalidation_verified=False,initial_GAME_public_C8_confirmation_verified=True,natural_initial_handoff_expiry_verified_by_this_case=False)")
replace("work='當版'+profile+'正常兩帳號鍵盤GUI旅館入房已限定核對：13步路線、原始對話、8／1鑰匙及map503／501、正常關閉；4sessions及4程序皆終止。自然到期、訪客離房、共享鑰匙、畫面／滑鼠／IME／裝置與完整GUI仍待，先自行測試才交人工。'",
    "work='當版'+profile+'正常兩帳號鍵盤GUI旅館入房及房主C10返回選角已限定核對：原13步路線／對話／8與1鑰匙、map503與501、完整返回握手／來源角色清單、原租裝及鑰匙移除、被動租約清理與正常關閉；4sessions及4程序皆終止。訪客離房、共享鑰匙／訪客驅離、自然到期、畫面／滑鼠／IME／裝置及完整GUI仍待；自行完整測試後才交人工。'")
replace("        v.setdefault('current_normal_GUI_hotel_entry_keyboard',{})[profile]=record",
    "        v.setdefault('current_normal_GUI_hotel_entry_keyboard',{})[profile]=record\n        v.setdefault('current_normal_GUI_hotel_return_keyboard',{})[profile]=record")
ast.parse(body);target.write_text(body,encoding='utf-8',newline='\n')
out=V/'native-rental-service-entry-return-tooling'/('hotel-owner-return-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir(parents=True)
record=dict(status='hotel_owner_return_GUI_reader_prepared_not_executed',reader=str(target),reader_sha256=sha(target),
    origin=str(origin),origin_sha256=sha(origin),contract=str(contract),contract_sha256=sha(contract),
    source_manifest_sha256='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B',
    expected_scenario='rental-hotel-entry',additional_required_Sky_labels=['hotel-return-focus','hotel-return','hotel-return-characters-ready'],
    all_original_entry_route_quantity_inventory_assertions_retained=True,GUI_executed=False,qualification_granted=False,whole_goal_complete=False)
(out/'hotel-return-tools.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
(out/'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tooling=str(out/'hotel-return-tools.json'),GUI_executed=False),ensure_ascii=False))
