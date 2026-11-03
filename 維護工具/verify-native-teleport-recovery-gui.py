from pathlib import Path
import hashlib, json, sys

p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json'); scenario=r['scenario']
assert scenario in ['timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry']
assert r['status']=='verified_native_teleport_GUI_'+scenario+'_and_fresh_reentry'
assert r['inputs_unchanged'] and r['private_desktop'] and not r['physical_input_verified']
assert r['jar_sha256'] in ['76EBE06D89DBD19CDB577E229657883C16156C77A23EDE288880C25584D2E3CC','C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539']
s=r['server'];assert s['actual_client_sessions']==(10 if scenario=='wrong-service' else 8) and s['memberships_empty'] and s['staged_loadouts_after_reentry']==0
assert s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
for name,digest in r['source_pins'].items():assert sha(p/'source-snapshot'/name)==digest
runtime=Path(r['runtime']);cap=next((runtime/'captures').glob('*/frames.jsonl'))
frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==(10 if scenario=='wrong-service' else 8)
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
if scenario=='wrong-service':
    assert sum(x['note']=='S_KEY' for x in frames)==10
    assert sum(x['note']=='S_VERSION_CHECK accepted' for x in frames)==10
    assert sum(x['note']=='S_LITO_LOGIN_OK' for x in frames)==6

outgoing=[b for x,b in packets if x['direction']=='client_to_server']
assert sum(b[0]==0xc9 for b in outgoing)==(4 if scenario=='cancel-expiry' else 2)
assert outgoing.count(bytes([0xd9]))==6
assert sum(x['direction']=='server_to_client' and b[0]==0xf4 and 'rental-teleport' in x['note'] for x,b in packets)==2
selections=[i for i,(x,b) in enumerate(packets) if x['direction']=='server_to_client' and b==bytes([0xc8,0])]
assert len(selections)==4
def inventory(batch):
    at=2;items={}
    for _ in range(batch[1]):
        item=int.from_bytes(batch[at:at+4],'little');at+=9
        count=int.from_bytes(batch[at:at+4],'little');at+=5
        end=batch.index(0,at);batch[at:end].decode('cp950');at=end+1
        at+=1+batch[at];assert item not in items;items[item]=count
    assert at==len(batch);return items
checks=[]
for n,(role,account) in enumerate([('a','PeerFixture'),('b','PeerFixture2')]):
    old,continued,reentered,moved=[r[role+'-'+k] for k in ['before','continued','reentered','after-move']]
    assert all(v['test_account']==account and v['character']==old['character'] for v in [continued,reentered,moved])
    assert old['session_identity']!=continued['session_identity']==reentered['session_identity']==moved['session_identity']
    assert reentered['rental_town'] and reentered['phase']=='BATTLE' and reentered['rental_map']==500
    target=[32770,32817] if role=='a' else [32771,32817]
    assert [moved['x'],moved['y']]==target
    shots={v['step']:v for v in r['steps'] if v['role']==role and v['step']!='GUI-key-walk'}
    assert shots['stopped']['inventory_button']=='背包（0）'
    if 'controls' in shots['stopped']:
        flags=shots['stopped']['controls'];assert flags['connect'] and not flags['cancel'] and not flags['inventory'] and not flags['npc_visible']
    elapsed=r[role+'-stop-ms']
    if scenario=='wrong-service':assert r['destination_wrong_service'] and 8000<=elapsed<15000
    elif scenario=='timeout':assert 5000<=elapsed<15000
    elif scenario=='refused':assert r['destination_refused'] and 0<elapsed<10000
    else:assert elapsed<5000
    if scenario=='cancel-late':
        assert 3500<=r[role+'-pre-cancel-ms']<=elapsed<5000
        assert r[role+'-pending-samples']>20 and 'destination-pending' in shots
    assert ('連線失敗：' in shots['stopped']['status']) if scenario in ['timeout','refused','wrong-service'] else shots['stopped']['status']=='尚未連線。'
    batch=next(b for x,b in packets[selections[n+2]+1:] if x['direction']=='server_to_client' and b[0]==0x7c)
    granted=inventory(batch);old_items={v['id']:v['count'] for v in old['rental_inventory']}
    assert len(old_items)==(14 if role=='a' else 11)
    assert shots['continued']['inventory_button']==f'背包（{len(granted)}）'
    if scenario=='cancel-expiry':
        assert set(granted).isdisjoint(old_items) and 'fresh-directory' in shots
        assert r[role+'-natural-wait-ms']>=33000 and r[role+'-closed-samples']>100
        stages=[json.loads(x) for x in (p/'staged-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
        observed=[(x['observed_now_nanos'],v) for x in stages for v in x['staged'] if set(v['inventory_ids'])==set(old_items)]
        assert observed and any(v['expired'] and now>=v['expires_at_nanos'] for now,v in observed)
    else:assert granted==old_items
    walks=[v for v in r['steps'] if v['role']==role and v['step']=='GUI-key-walk'];assert len(walks)==8 and walks[-1]['after']==target
    for shot in shots.values():assert sha(Path(shot['png']))==shot['png_sha256']
    checks.append(dict(role=role,stop_ms=elapsed,old_authority_cleared=True,fresh_character_selected_manually=scenario=='cancel-expiry',granted_items=len(granted),original_staged_loadout_retained=scenario!='cancel-expiry',authoritative_move=target))
result=dict(status='verified_native_GUI_'+scenario+'_and_same_source_world_recovery',scenario=scenario,checks=checks,actual_client_sessions=s['actual_client_sessions'],exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),server_trace_sha256=sha(p/'server-trace.jsonl'),validator_sha256=sha(Path(__file__)),reference_is_current=r.get('reference_is_current'),physical_input_verified=False,full_product_verified=False)
(p/'teleport-recovery-gui-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':result['status'],'checks':checks},ensure_ascii=False))
