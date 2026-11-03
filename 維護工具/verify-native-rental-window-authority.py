"""Native rental widget flow cross-checked against passive formal states."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]).resolve()
def read(name):return json.loads((p/name).read_text(encoding='utf-8-sig'))
def sha(name):return hashlib.sha256((p/name).read_bytes()).hexdigest().upper()
r=read('receipt.json');assert r['status']=='verified_native_rental_widget_flow' and r['action_profile']=='rental-shop'
states=[row['state'] for line in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines() if 'state' in (row:=json.loads(line))]
proof=[]
for object_id,role in [(1,'a'),(3,'b')]:
    owned=[s for s in states if s['object_id']==object_id and s['phase']=='BATTLE' and s['rental_town'] and s['rental_map']==500 and s['rental_set']==0]
    start=next(i for i,s in enumerate(owned) if s['rental_gems']==20)
    rented=next(i for i in range(start+1,len(owned)) if owned[i]['rental_gems']==17)
    returned=next(i for i in range(rented+1,len(owned)) if owned[i]['rental_gems']==20)
    before,after,end=[owned[i] for i in [start,rented,returned]]
    items=lambda s:{v['id']:v for v in s['rental_inventory']}
    added=set(items(after))-set(items(before));assert len(added)==1
    assert items(end)==items(before)
    assert all(s['shop_points']==5000 for s in [before,after,end])
    steps={s['step']:s for s in r['steps'] if s['role']==role}
    assert '光之寶石不足' in steps['rent-refused']['status']
    assert '光之寶石：20' in steps['rent-refused']['rental_money']
    assert '背包空間不足' in steps['rent-capacity-refused']['status']
    assert '光之寶石：20' in steps['rent-capacity-refused']['rental_money']
    assert '背包空間不足' not in steps['rented']['status']
    assert '光之寶石：17' in steps['rented']['rental_money'] and '光之寶石：20' in steps['returned']['rental_money']
    assert '勒夫:' in steps['npc-original']['rental_body'] and '有啥需要幫忙嗎?' in steps['npc-original']['rental_body']
    proof.append({'role':role,'object_id':object_id,'added_then_removed':list(added),'before':before,'rented':after,'returned':end})
for s in r['steps']:
    if 'png' in s:
        image=Path(s['png']);assert image.parent==p and sha(image.name)==s['png_sha256']
navigation=None
if r.get('npc_navigation_widget_verified'):
    owned=[s for s in states if s['object_id']==1 and s['phase']=='BATTLE' and s['rental_town']]
    index=0;grants=[]
    for count in [1,2,3,4]:
        index=next(i for i in range(index,len(owned)) if len(owned[i]['rental_emblems'])==count)
        grants.append(owned[index])
    final=next(s for s in owned[index:] if s['rental_citizen'] and not s['rental_emblems'] and s['rental_gems']==21)
    assert final['shop_points']==5000
    movements=[s for s in r['steps'] if s['step']=='native-key-message-movement']
    plan=read('rental-npc-routes.json');assert sha('rental-npc-routes.json')==r['route_plan_sha256'];expected=[step for route in plan['routes'][:5] for step in route['steps']];assert len(movements)==len(expected)
    previous=[32770,32818];index=0
    for move,planned in zip(movements,expected):
        assert move['before']==planned['before'] and move['after']==planned['after'] and move['key']==planned['key']
        assert move['before']==previous and max(abs(a-b) for a,b in zip(move['before'],move['after']))==1
        index=next(i for i in range(index,len(owned)) if [owned[i]['x'],owned[i]['y']]==move['after'])
        previous=move['after']
    assert previous==plan['routes'][4]['end']
    navigation={'single_steps':len(movements),'emblem_grants':grants,'citizenship':final,'key_event_scope':'owned native WM_KEYDOWN/UP with actual GUI thread focus and controller movement checks on unswitched private desktop; not physical keyboard input'}
server=read('result.json');assert server['actual_client_sessions']==4 and server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
assert (p/'server.log').read_text(encoding='utf-8-sig').count('reason=inventory-capacity')==2
out={'status':'verified_native_rental_widget_authority','receipt_sha256':sha('receipt.json'),'trace_sha256':sha('server-trace.jsonl'),'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],'clients':proof,'normal_rental_return_verified':False,'physical_input_verified':False,'full_product_verified':False}
out['npc_navigation']=navigation
(p/'rental-authority-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified two native rental widget flows against independent inventory and currency:',p)
