from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');base=read(p/'candidate-return-comparison.json');assert r['return_services'] and base['receipt_sha256']==sha(p/'receipt.json')
trace=[json.loads(l)['state'] for l in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
cap=Path(base['capture']);frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];start=next(i for i,f in enumerate(frames) if f['direction']=='client_to_server' and f['plain']=='10')
removed={int.from_bytes(bytes.fromhex(f['plain'])[1:5],'little') for f in frames[start:] if f['direction']=='server_to_client' and f['opcode']=='0x17'}
checks=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 c=r['clients'][role];q=c['return_observation'];hotel=c['hotel'];shop=c['shop'];assert c['return_services_requested'] and shop['retained_for_return'] and hotel['room_entry_verified'] and not hotel['server_authority_injection']
 item=shop['retained_item'];keys={int(x) for x in hotel['rented']['inventory']['items']} - {int(x) for x in hotel['before']['inventory']['items']}
 assert len(keys)==hotel['quantity'] and item not in keys
 assert all(str(x) in q['before']['inventory']['items'] for x in keys|{item})
 assert q['before']['world']['map_id']==(503 if role=='a' else 501)
 assert q['after']['inventory']['items']=={} and all(str(x) not in q['reentered']['inventory']['items'] for x in keys|{item})
 assert (keys|{item})<=removed
 owned=[s for s in trace if s['test_account']==account]
 lease=next(s for s in owned if s['rental_hotel'] is not None and s['rental_map']==q['before']['world']['map_id'])
 assert lease['rental_hotel']['keys']==hotel['quantity']
 empty=next(s for s in owned if s['pending_transition']=='RESTART_AWAIT_SOURCE_REFRESH')
 assert empty['rental_hotel'] is None and empty['rental_inventory']==[] and not empty['rental_town']
 fresh=[s for s in owned if s['phase']=='BATTLE' and s['rental_town']][-1]
 assert fresh['rental_hotel'] is None and fresh['rental_map']==500 and all(x['id']!=item for x in fresh['rental_inventory'])
 checks.append(dict(role=role,old_lease=lease['rental_hotel'],departed_map=q['before']['world']['map_id'],keys=sorted(keys),purchased_item=item,formal_remove_packets_verified=True,new_world_generation=q['reentered']['world']['generation']))
assert r['server']['active_hotel_leases_after_disconnect']==0
out=dict(status='verified_candidate_exit_with_purchased_items_and_active_hotel_leases',exe_sha256=r['exe_sha256'],candidate_jar_sha256=r['jar_sha256'],formal_jar_sha256=r['formal_baseline_sha256'],receipt_sha256=sha(p/'receipt.json'),base_comparison_sha256=sha(p/'candidate-return-comparison.json'),capture_sha256=sha(cap),checks=checks,formal_deployed=False,clock_or_authority_injection=False,shared_guest_eviction_verified=False,natural_expiry_verified=False,full_product_verified=False)
(p/'candidate-services-return-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(status=out['status'],roles=2,keys_removed=sum(len(x['keys']) for x in checks),purchased_items_removed=2)))
