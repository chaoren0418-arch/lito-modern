"""Prepare resource and GUI readers, including a natural >30s source-handoff regression."""
from pathlib import Path
from datetime import datetime,timezone
import ast,hashlib,json
R=Path(__file__).resolve().parents[1];T=R/'維護工具';V=R/'驗證紀錄/current'
q=json.loads((R/'工作佇列.json').read_text('utf-8-sig'));source=q['current_product_identity']['source_manifest_sha256']
assert source==q['current_native_initial_source_confirmation_application']['source_manifest_sha256']
old='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
names=['run-native-accessibility-labels-library-resource-optin.py','checkpoint-native-accessibility-labels-library-resource-optin.py','prepare-native-accessibility-labels-sky-gui-current.py','start-native-accessibility-labels-sky-gui-case.py','checkpoint-native-accessibility-labels-sky-gui.py','qualify-native-accessibility-labels-GUI-rental-entry.py','qualify-native-accessibility-labels-GUI-hotel-entry.py']
staged={};origins={}
for name in names:
    raw=(T/name).read_bytes();origins[name]=hashlib.sha256(raw).hexdigest().upper()
    body=raw.decode('utf-8-sig').replace('\r\n','\n').replace('native-accessibility-labels','native-initial-source-confirmation').replace('accessibility_labels','initial_source_confirmation').replace(old,source)
    if name=='prepare-native-accessibility-labels-sky-gui-current.py':
        needle="products=products, product_source_modified=False, GUI_verified=False, whole_goal_complete=False,"
        assert body.count(needle)==1
        body=body.replace(needle,"products=products, product_source_modified=False, GUI_verified=False, whole_goal_complete=False,\n     initial_GAME_source_confirmation_expected=True, required_natural_source_handoff_expiry_regression=True,")
    if name=='qualify-native-accessibility-labels-GUI-rental-entry.py':
        anchor="assert sum(body[0]==0xc9 for body in outgoing)==2"
        assert body.count(anchor)==1
        body=body.replace(anchor,anchor+"\nsource_requests=[body for body in outgoing if body[0]==0xc8]\nassert len(source_requests)==4 and all(body[1:].rstrip(b'\\0').decode('cp950')=='測試乙'for body in source_requests)")
        anchor="    assert '請選擇角色。'in by_label['character-response']['tree']"
        assert body.count(anchor)==1
        body=body.replace(anchor,anchor+"\n    assert account+'(測試乙)'in by_label['character-response']['tree']\n    assert all(account+'(測試甲)'not in row['tree']for row in own)")
        anchor="    close=by_label['close-normal'];assert close['action']==dict(method='press_key',args={'key':'Alt_L+F4'})"
        assert body.count(anchor)==1
        addition='''    own_packets=[(row,body)for row,body in packets if start<=datetime.fromisoformat(row['time'].replace('Z','+00:00'))<end]
    c8=[row for row,body in own_packets if row['direction']=='client_to_server'and body[0]==0xc8]
    version=next(row for row,body in own_packets if row['direction']=='client_to_server'and body[0]==0x00)
    directory_rows=[row for row,body in own_packets if row['direction']=='server_to_client'and row['note'].startswith('S_LITO_CHARACTER_INFO directory-row name='+account+'(')]
    assert len(c8)==len(directory_rows)==2
    stamp=lambda row:datetime.fromisoformat(row['time'].replace('Z','+00:00'))
    assert stamp(c8[0])<stamp(version)<stamp(directory_rows[0])<stamp(c8[1])<stamp(directory_rows[1])
    assert 'classic-source=測試乙'in directory_rows[1]['note']
    assert (stamp(directory_rows[1])-stamp(c8[1])).total_seconds()<8
    elapsed=(stamp(version)-stamp(c8[0])).total_seconds()
    if role=='b':
        assert elapsed>30 and 'classic-source=測試甲'in directory_rows[0]['note']
        assert 'handoff-wait-ready'in by_label and 'handoff-wait-finished'in by_label
        wait_start=datetime.fromisoformat(by_label['handoff-wait-ready']['observed_utc'].replace('Z','+00:00'))
        wait_end=datetime.fromisoformat(by_label['handoff-wait-finished']['observed_utc'].replace('Z','+00:00'))
        assert (wait_end-wait_start).total_seconds()>30
    source_confirmation=dict(initial_selection_to_GAME_version_seconds=elapsed,public_C8_requests=2,
        first_wire_directory_source=('測試甲'if 'classic-source=測試甲'in directory_rows[0]['note']else'測試乙'),
        confirmed_wire_directory_source='測試乙',wrong_source_directory_observed_in_UI=False,
        natural_initial_handoff_expiry_exercised=role=='b',server_TTL_unchanged=True)
'''
        body=body.replace(anchor,addition+anchor)
        anchor="        GUI_inventory_items=count,native_source_items=source_count,rental_carry_subset=carry_count,"
        assert body.count(anchor)==1;body=body.replace(anchor,"        initial_GAME_source_confirmation=source_confirmation,\n"+anchor)
        anchor="record.update(normal_skill_accessible_name_verified=True,normal_skill_label_visibility_verified=True)"
        assert body.count(anchor)==1;body=body.replace(anchor,anchor+"\nrecord.update(initial_GAME_public_C8_confirmation_verified=True,natural_initial_source_handoff_expiry_verified=True,server_source_handoff_TTL_seconds=30,staged_rental_recovery_verified_by_this_case=False)")
    ast.parse(body)
    target=name.replace('native-accessibility-labels','native-initial-source-confirmation').replace('-sky-gui-current.py','-sky-gui.py')
    assert not(T/target).exists();staged[target]=body.encode('utf-8')
out=V/'native-initial-source-confirmation-tooling'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
for n,b in staged.items():(T/n).write_bytes(b);(out/n).write_bytes(b)
record=dict(source_manifest_sha256=source,templates=origins,tools={n:hashlib.sha256(b).hexdigest().upper()for n,b in staged.items()},rental_entry_GUI_requires_natural_B_handoff_delay_over_30_seconds=True,server_source_TTL_unchanged=True,library_or_GUI_runtime_started=False)
(out/'followup-tools.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(tooling=str(out),tools=len(staged),runtime_started=False),ensure_ascii=False))
