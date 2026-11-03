"""Pending test: independent formal trace for ground key transfer and forced return."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-hotel-share' and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty'] and r['server']['active_hotel_leases_after_disconnect']==0
trace=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
a=r['clients']['a']['hotel_share'];b=r['clients']['b']['hotel_share']
before_a=a['before']['inventory']['items'];before_b=b['before']['inventory']['items']
keys=set(a['rented']['inventory']['items'])-set(before_a);assert len(keys)==2
transferred=keys-set(a['dropped']['inventory']['items']);assert len(transferred)==1
assert set(b['picked_up']['inventory']['items'])-set(before_b)==transferred
assert a['returned']['inventory']['items']==before_a and b['returned']['inventory']['items']==before_b
assert a['returned']['world']['map_id']==500 and a['returned']['world']['generation']==a['before']['world']['generation']
assert b['entered']['world']['map_id']==502 and b['returned']['world']['map_id']==500
assert b['returned']['world']['completed_portals']==b['before']['world']['completed_portals']+2
assert b['returned']['world']['generation']==b['before']['world']['generation']+2
assert b['returned']['merchant']['owner'] is None
assert [b['returned']['world']['actors']['3'][k] for k in ['x','y','heading']]==[32779,32832,2]
assert set(b['entered']['world']['actors'])=={'3'} and '1' in b['returned']['world']['actors']
proof=[];tokens=[]
for role,q,own in [('a',a,1),('b',b,3)]:
 c=r['clients'][role];assert c['worker_quiescent'] and c['shutdown']=='closed'
 assert not q['server_authority_injection'] and not q['gui_verified'] and not q['natural_expiry_verified']
 states=[s for s in trace if s['object_id']==own and s['rental_town']];cursor=0
 for step in q['steps']:
  cursor=next(j for j in range(cursor,len(states)) if [states[j]['x'],states[j]['y']]==step['after'])
 # The formal observer exposes the rental inventory, while the client combines
 # normal equipment and rental service items. Preserve the independent baseline
 # rental inventory and add/remove only the actually transferred key identities.
 rental_base={v['id']:v['count'] for v in states[cursor]['rental_inventory']}
 client_base=q['before']['inventory']['items']
 assert rental_base and not (set(rental_base)&{int(k) for k in keys})
 assert all(str(k) in client_base and client_base[str(k)]['count']==v for k,v in rental_base.items())
 stages=['rented','dropped','returned'] if role=='a' else ['picked_up','entered','returned']
 indexes=[]
 for stage in stages:
  snap=q[stage];items=snap['inventory']['items'];actor=snap['world']['actors'][str(own)]
  assert {k:v for k,v in items.items() if k not in keys}==client_base
  inventory=dict(rental_base);inventory.update({int(k):items[k]['count'] for k in keys if k in items})
  cursor=next(j for j in range(cursor,len(states)) if states[j]['rental_map']==snap['world']['map_id'] and [states[j]['x'],states[j]['y']]==[actor['x'],actor['y']] and {v['id']:v['count'] for v in states[j]['rental_inventory']}==inventory)
  s=states[cursor];assert s['rental_gems']==20
  if stage=='returned':assert s['rental_hotel'] is None
  else:
   assert s['rental_hotel']['map']==502 and s['rental_hotel']['keys']==2;tokens.append(s['rental_hotel']['token'])
  indexes.append({'stage':stage,'trace_index':cursor})
 images={}
 for k,v in q.items():
  if k.endswith('_render'):
   image=Path(v['png']);assert image.is_file();images[k]=hashlib.sha256(image.read_bytes()).hexdigest()
 proof.append({'role':role,'states':indexes,'images':images})
assert len(set(tokens))==1 and tokens[0]>0
report={'status':'verified_shared_key_transfer_and_owner_return','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'key_ids':sorted(keys),'transferred_key':next(iter(transferred)),'shared_lease_token':tokens[0],'checks':proof,'gui_verified':False,'natural_expiry_verified':False,'manual_guest_exit_verified':False}
(p.parent/'rental-hotel-share-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'checks':len(proof)}))
