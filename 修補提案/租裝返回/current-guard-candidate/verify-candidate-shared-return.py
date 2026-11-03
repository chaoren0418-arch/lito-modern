"""Independent active guest eviction caused by owner source restart, not NPC refund."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve()
read=lambda x:json.loads(x.read_text(encoding='utf-8-sig'))
sha=lambda x:hashlib.sha256(x.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');base=read(p/'candidate-return-comparison.json')
assert r['return_shared'] and not r['return_services'] and base['receipt_sha256']==sha(p/'receipt.json')
assert r['server']['active_hotel_leases_after_disconnect']==0 and r['server']['authority_mutations']==0
for role in ['a','b']:
 assert r['clients'][role]['return_shared_requested']
 assert r['clients'][role]['hotel_share']['owner_restart_requested']
a=r['clients']['a']['hotel_share'];b=r['clients']['b']['hotel_share']
keys=set(a['rented']['inventory']['items'])-set(a['before']['inventory']['items']);assert len(keys)==2
transferred=keys-set(a['dropped']['inventory']['items']);assert len(transferred)==1
assert set(b['picked_up']['inventory']['items'])-set(b['before']['inventory']['items'])==transferred
assert b['entered']['world']['map_id']==502 and set(b['entered']['world']['actors'])=={'3'}
assert b['returned']['world']['map_id']==500 and b['returned']['inventory']['items']==b['before']['inventory']['items']
assert b['returned']['world']['generation']==b['before']['world']['generation']+2
assert b['returned']['world']['completed_portals']==b['before']['world']['completed_portals']+2
assert [b['returned']['world']['actors']['3'][x] for x in ['x','y']]==[32779,32832]
assert b['returned']['merchant']['owner'] is None
assert keys.isdisjoint(a['return_observation']['after']['inventory']['items'])
assert keys.isdisjoint(a['return_observation']['reentered']['inventory']['items'])
assert keys.isdisjoint(b['return_observation']['reentered']['inventory']['items'])
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
# Track real account/session identity through the reset of character object IDs.
owner=next(x['session_identity'] for x in trace if x['test_account']=='PeerFixture' and x['rental_town'])
guest=next(x['session_identity'] for x in trace if x['test_account']=='PeerFixture2' and x['rental_town'])
inside=next(i for i,x in enumerate(trace) if x['session_identity']==guest and x['rental_map']==502 and x['rental_hotel'])
lease=trace[inside]['rental_hotel'];assert lease['keys']==2 and lease['map']==502
owned=next(x for x in trace[:inside] if x['session_identity']==owner and x['rental_hotel'] and x['rental_hotel']['token']==lease['token'])
assert {int(k) for k in keys}.issubset({x['id'] for x in owned['rental_inventory']})
requested=next(i for i in range(inside+1,len(trace)) if trace[i]['session_identity']==owner and trace[i]['pending_transition']=='RESTART_AWAIT_QUIT_FIGHT')
evicted=next(i for i in range(requested+1,len(trace)) if trace[i]['session_identity']==guest and trace[i]['rental_map']==500 and trace[i]['rental_hotel'] is None and not ({int(k) for k in keys}&{v['id'] for v in trace[i]['rental_inventory']}))
assert trace[evicted]['phase']=='BATTLE' and trace[evicted]['pending_transition']=='NONE'
frames=[json.loads(x) for x in Path(base['capture']).read_text(encoding='utf-8-sig').splitlines()]
first_restart=next(i for i,x in enumerate(frames) if x['direction']=='client_to_server' and x['plain']=='10')
removed={int.from_bytes(bytes.fromhex(x['plain'])[1:5],'little') for x in frames[first_restart:] if x['direction']=='server_to_client' and x['opcode']=='0x17'}
assert {int(k) for k in keys}.issubset(removed)
# There must be an actual eviction portal after the restart request.
assert any(x['direction']=='server_to_client' and x['opcode']=='0x5A' for x in frames[first_restart:])
images={}
for role,q in [('a',a),('b',b)]:
 for k,v in q.items():
  if k.endswith('_render'): images[role+'-'+k]=sha(Path(v['png']))
assert len(images)==4
out=dict(status='verified_candidate_owner_restart_evicts_shared_guest_then_both_reenter',receipt_sha256=sha(p/'receipt.json'),base_comparison_sha256=sha(p/'candidate-return-comparison.json'),key_ids=sorted(keys),transferred_key=next(iter(transferred)),lease_token=lease['token'],trace_indices=dict(guest_inside=inside,owner_requested=requested,guest_evicted=evicted),images=images,formal_deployed=False,physical_gui_verified=False,clock_or_authority_injection=False)
(p/'candidate-shared-return-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'keys':len(keys),'images':len(images)}))
