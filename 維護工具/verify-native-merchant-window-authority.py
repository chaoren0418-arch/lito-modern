"""Compare all GUI merchant transactions with a separately sampled formal trace."""
import json, hashlib, sys
from pathlib import Path
p=Path(sys.argv[1]).resolve()
read=lambda name:json.loads((p/name).read_text(encoding='utf-8-sig'))
sha=lambda path:hashlib.sha256(path.read_bytes()).hexdigest().upper()
r=read('receipt.json');plan=read('merchant-gui-plan.json')
assert r['status']=='verified_native_merchant_widget_flow'
assert r['action_profile']=='rental-merchants' and r['all_78_merchant_widget_rows_verified']
assert r['plan_sha256']==sha(p/'merchant-gui-plan.json')
from npc_route_reference import assert_route_reference
assert_route_reference(p,r,plan,'LiTo381-Modern/tests/fixtures/native-gui/merchant-plan.json','merchant-gui-plan.json')
assert len(plan['shops'])==4 and sum(len(s['items']) for s in plan['shops'])==78
trace=[v['state'] for line in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines() if 'state' in (v:=json.loads(line))]
proof=[];retired=set()
for role,own in [('a',1),('b',3)]:
    transactions=[x for x in r['merchant_transactions'] if x['role']==role]
    assert len(transactions)==78
    states=[x for x in trace if x['object_id']==own and x['phase']=='BATTLE' and x['rental_town']]
    cursor=0;transaction=0;movements=[]
    for shop in plan['shops']:
        movements.extend(shop['steps'])
        for row,item in enumerate(shop['items']):
            record=transactions[transaction];transaction+=1
            assert [record[k] for k in ['npc','row','catalog_id','name','price']]==[shop['npc'],row,item['catalog_id'],item['name'],item['gems']]
            indices=[]
            for stage in ['before','rented','returned']:
                cursor=next(i for i in range(cursor,len(states)) if states[i]==record[stage]);indices.append(cursor)
            before,rented,returned=[record[k] for k in ['before','rented','returned']]
            assert all(x['rental_map']==500 and x['shop_points']==5000 for x in [before,rented,returned])
            assert [x['rental_gems'] for x in [before,rented,returned]]==[20,20-item['gems'],20]
            inv=lambda state:{x['id']:x for x in state['rental_inventory']}
            old,mid,end=[inv(x) for x in [before,rented,returned]]
            identity=record['item'];assert identity not in retired;retired.add(identity)
            assert set(mid)-set(old)=={identity} and {k:v for k,v in mid.items() if k!=identity}==old
            assert mid[identity]['catalog_id']==item['catalog_id'] and mid[identity]['count']==1 and not mid[identity]['equipped']
            assert end==old
            proof.append({'role':role,'npc':shop['npc'],'catalog_id':item['catalog_id'],'item':identity,'formal_indices':indices})
    moves=[s for s in r['steps'] if s['role']==role and s['step']=='native-key-message-movement']
    assert len(moves)==len(movements)==23
    cursor=0
    for move,planned in zip(moves,movements):
        assert all(move[key]==planned[key] for key in ['before','after','key'])
        cursor=next(i for i in range(cursor,len(states)) if [states[i]['x'],states[i]['y']]==move['after'])
for step in r['steps']:
    if 'png' in step:
        image=Path(step['png']);assert image.parent==p and sha(image)==step['png_sha256']
server=read('result.json')
assert server['actual_client_sessions']==4 and server['memberships_empty']
assert server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
out={'status':'verified_all_78_merchant_rows_on_two_native_windows','exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p/'receipt.json'),'trace_sha256':sha(p/'server-trace.jsonl'),'items_per_role':78,'rented_and_returned_objects':156,'native_key_message_steps':46,'checks':proof,'extra_equipment_bonuses_verified':False,'physical_input_verified':False,'full_product_verified':False}
(p/'merchant-authority-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'transactions':len(proof)}))
