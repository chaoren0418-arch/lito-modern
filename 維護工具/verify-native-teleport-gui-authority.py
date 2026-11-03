from pathlib import Path
import hashlib, json, sys

p = Path(sys.argv[1]).resolve()
read = lambda f: json.loads(f.read_text(encoding='utf-8-sig'))
sha = lambda f: hashlib.sha256(f.read_bytes()).hexdigest().upper()
r = read(p/'receipt.json')
assert r['status'] == 'verified_native_teleport_GUI_cross_endpoint_shared_battle'
assert r['private_desktop'] and r['inputs_unchanged'] and not r['physical_input_verified']
s = r['server']
assert s['actual_client_sessions'] == 6 and s['memberships_empty'] and s['staged_loadouts_empty']
assert s['authenticated_accounts_empty'] and s['authority_mutations'] == 0
assert not s['sql_access'] and not s['state_persistent']
assert r['jar_sha256'] in ['6928A147DF1E784304B9080632C56121B173BF60B172023EBD263E82FD448628','76EBE06D89DBD19CDB577E229657883C16156C77A23EDE288880C25584D2E3CC','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539']
for name, digest in r['source_pins'].items():
    assert sha(p/'source-snapshot'/name) == digest
runtime = Path(r['runtime'])
ready = read(p/'ready.json')
cap = next((runtime/'captures').glob('*/frames.jsonl'))
frames = [json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain'] == 'MASKED_C_LOGIN' for x in frames) == 6
packets = [(x, bytes.fromhex(x['plain'])) for x in frames if x['plain'] != 'MASKED_C_LOGIN']
outgoing = [b for x,b in packets if x['direction'] == 'client_to_server']
assert sum(b[0] == 0xc9 for b in outgoing) == 2  # No duplicate destination selection.
assert outgoing.count(bytes([0xd9])) == 4
endpoints = [b for x,b in packets if x['direction'] == 'server_to_client' and b[0] == 0xf4 and 'rental-teleport' in x['note']]
assert len(endpoints) == 2
for slot,port in [(1,ready['b_port']), (2,ready['c_port'])]:
    assert any(b[1] == slot and b[2:6] == bytes([127,0,0,slot+2]) and int.from_bytes(b[6:8],'little') == port for b in endpoints)
checks = []
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
    old, dest, battle = r[role+'-before'],r[role+'-destination'],r['battle'][role]
    assert old['test_account'] == dest['test_account'] == battle['test_account'] == account
    assert old['rental_town'] and old['rental_map'] == 500
    assert old['session_identity'] != dest['session_identity'] == battle['session_identity']
    assert old['character'] == dest['character'] == battle['character']
    assert not dest['rental_town'] and dest['phase'] == 'LOBBY'
    old_items = {v['id']:v for v in old['rental_inventory']}
    slot = 1 if role == 'a' else 2
    start = next(i for i,(x,b) in enumerate(packets) if x['direction']=='server_to_client' and b[0]==0xf4 and b[1]==slot and 'rental-teleport' in x['note'])
    batch = next(b for x,b in packets[start+1:] if x['direction']=='server_to_client' and b[0]==0x7c)
    at=2;dest_items={}
    for _ in range(batch[1]):
        item=int.from_bytes(batch[at:at+4],'little');at+=4
        at+=5  # use_type, charge, icon(u16), blessing
        count=int.from_bytes(batch[at:at+4],'little');at+=5  # quantity + identified
        end=batch.index(0,at);batch[at:end].decode('cp950');at=end+1
        detail=batch[at];at+=1+detail
        assert item not in dest_items;dest_items[item]=count
    assert at==len(batch)
    assert len(old_items) == (14 if role == 'a' else 11)
    assert {item:v['count'] for item,v in old_items.items()} == dest_items
    assert battle['phase'] == 'BATTLE' and not battle['rental_town']
    moves = [v for v in r['steps'] if v['role'] == role and v['step'] == 'GUI-key-walk']
    assert len(moves) == 7 and moves[0]['before'] == [32770,32818] and moves[-1]['after'] == [32777,32819]
    labels = r[role+'-teleport-labels']
    assert labels == ['祝福之地','勇氣之地']
    screenshots = [v for v in r['steps'] if v['role'] == role and v['step'] in ['town','teleport-dialog','destination','destination-lobby','new-battle']]
    assert len(screenshots) == 5
    for shot in screenshots:
        assert sha(Path(shot['png'])) == shot['png_sha256']
    assert any('你想要去哪裡呢' in v['rental_body'] for v in screenshots if v['step'] == 'teleport-dialog')
    assert any(f'背包（{len(old_items)}）' == v['inventory_button'] for v in screenshots if v['step']=='destination')
    assert any('地圖 564' in v['status'] for v in screenshots if v['step'] == 'new-battle')
    checks.append(dict(role=role,character=dest['character'],loadout_objects=len(old_items),GUI_walks=7,old_dialog_retired=True,one_destination_selection=True))
assert r['battle']['a']['room'] == r['battle']['b']['room'] != 0
result = dict(status='verified_native_GUI_teleport_source_dialog_and_cross_endpoint_shared_battle',receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),server_trace_sha256=sha(p/'server-trace.jsonl'),validator_sha256=sha(Path(__file__)),exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],checks=checks,actual_client_sessions=6,private_desktop=True,physical_input_verified=False,original_HTML_icon_layout_verified=False,full_product_verified=False)
(p/'teleport-gui-authority-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':result['status'],'checks':checks},ensure_ascii=False))
