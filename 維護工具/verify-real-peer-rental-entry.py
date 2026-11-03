import hashlib
import json,sys
from pathlib import Path
from native_resident_map_reference import qualify as qualify_resident_terrain
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'));assert r['action_profile'] in ('rental-entry','rental-shop','rental-citizen','rental-hotel','rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-share','rental-merchants','rental-weight','rental-item-expiry','rental-equipped-expiry','rental-hotel-expiry','rental-hotel-middle-expiry') and r['source_matches_snapshot']
states=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checked=[];terrain_checks={}
for role,c in r['clients'].items():
    own=1 if role=='a' else 3;other=3 if role=='a' else 1
    assert c['worker_quiescent'] and c['shutdown']=='closed'
    assert c['refused']['channel_result']!=0 and c['refused']['rental_set'] is None
    assert c['selected']['rental_set']==0 and c['selected']['channel'] is None
    s=c['end'];w=s['world'];room=s['joined_room'];assert room['mode']==6 and room['map_id']==w['map_id']==500 and room['id']==60001
    assert s['channel'] is None and w['status']['object_id']==own
    assert s['shop_points']==5000 and w['battle_result'] is None and not w['scores']
    for actor_id in [own,other]:
        a=w['actors'][str(actor_id)]
        rows=[t for t in states if t['object_id']==actor_id and t['phase']=='BATTLE' and t['rental_town'] and t['rental_map']==500 and t['rental_set']==0 and t['room']==60001 and t['room_state'] is None and t['x']==a['x'] and t['y']==a['y'] and t['shop_points']==5000]
        assert rows,(role,actor_id,'formal rental snapshot missing');checked.append({'role':role,'actor':actor_id,'authority_samples':len(rows)})
    assert Path(c['render']['png']).is_file()
    budget=c['render']['frame']['static_resource_budget']
    terrain=qualify_resident_terrain(c['render'],w);terrain_checks[role]=terrain
    assert terrain['complete_sector_count']==18 and terrain['whole_source_draw_references']==351509
    assert terrain['whole_source_decoded_slots']==17653 and 0<budget['decoded_bytes']<=budget['decoded_limit_bytes']
    assert budget['draw_references']<=budget['draw_limit']==1048576
    assert w['hp'][0]>0 and len(w['actors'])>2
assert r['server']['memberships_empty'] and r['server']['authority_mutations']==0
shop_checks=[]
if r['action_profile']=='rental-shop':
    for role,c in r['clients'].items():
        shop=c['shop'];own=1 if role=='a' else 3;gem='1811939330';item=str(shop['item']);npc=1812987916
        before=shop['before'];refused=shop['refused'];rented=shop['rented'];returned=shop['returned'];price=shop['price'];refund=shop['refund']
        assert shop['dialog']['dialog']['html']=='Love_Start' and shop['dialog']['dialog']['npc']==npc
        assert len(shop['buy_list']['listing']['Buy']['rows'])==17
        assert before['inventory']['items']==refused['inventory']['items']
        assert refused['world']['system_message_count']>before['world']['system_message_count'] and refused['world']['last_system_message']==[9,'光之寶石不足。']
        capacity=shop['capacity_refused'];assert shop['capacity_quantity']==257
        assert capacity['inventory']['items']==before['inventory']['items'] and capacity['world']['last_message']==[263,[]]
        assert capacity['world']['message_count']>refused['world']['message_count'] and capacity['merchant']['listing'] is None
        assert item not in before['inventory']['items'] and item in rented['inventory']['items'] and item not in returned['inventory']['items']
        gems=before['inventory']['items'][gem]['count']
        assert rented['inventory']['items'][gem]['count']==gems-price
        assert returned['inventory']['items'][gem]['count']==gems-price+refund
        for stage,snapshot in [('rented',rented),('returned',returned)]:
            matching=[t for t in states if t['object_id']==own and t['rental_town'] and t['rental_gems']==snapshot['inventory']['items'][gem]['count'] and ((any(v['id']==int(item) and v['count']==1 and v['expires_at']>0 for v in t['rental_inventory']))==(stage=='rented'))]
            assert matching,(role,stage,'missing passive transaction authority')
            shop_checks.append({'role':role,'stage':stage,'item':int(item),'gems':snapshot['inventory']['items'][gem]['count'],'authority_samples':len(matching)})
        assert not shop['server_authority_injection']
    assert (p.parent/'server.log').read_text(encoding='utf-8-sig').count('reason=inventory-capacity')==2
report={'status':'verified_entry_subset','checks':checked,'map':500,'normal_return_verified':False,'shop_verified':r['action_profile']=='rental-shop','shop_checks':shop_checks,'gui_verified':False,'all_resources_verified':False}
report['independent_original_S32_residency']=terrain_checks
report['receipt_sha256']=hashlib.sha256(p.read_bytes()).hexdigest()
(p.parent/'rental-entry-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'checks':len(checked)}))
