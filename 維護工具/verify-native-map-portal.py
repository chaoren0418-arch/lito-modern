"""Compare real C73/S5A/C08 and server authority with original map-source rows."""
from pathlib import Path
import json, hashlib, sys, collections
p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');runtime=Path(r['runtime'])
assert r['status']=='verified_original_map_portal_roundtrip' and r['test_exit_code']==r['server_exit_code']==0 and r['inputs_unchanged']
assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
assert r['server']['actual_client_sessions']==4 and r['server']['memberships_empty']
assert r['server']['authority_mutations']==0 and not r['server']['sql_access'] and not r['server']['state_persistent']
for f,h in r['source_pins'].items():assert sha(p/'source-snapshot'/f)==h
reference=read(Path(__file__).resolve().parents[1]/'驗證紀錄/current/native-world-integration/native-map-portal-reference.json')
assert reference['jar_sha256']==r['jar_sha256']
routes={tuple(v['source']):v for v in reference['rows']}
cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==4
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
enters=[b for x,b in packets if x['direction']=='client_to_server' and b[0]==0x73]
portals=[b for x,b in packets if x['direction']=='server_to_client' and b[0]==0x5a]
assert len(enters)==len(portals)==4
assert sum(x['direction']=='client_to_server' and b==b'\x08' for x,b in packets)==4
expected_sources=collections.Counter();expected_dest=collections.Counter();checks=[]
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 d=read(p/f'{role}-map-portal.json');assert len(d['transitions'])==2 and d['epoch']==2
 assert set(map(tuple,d['original_entrances']))==set(routes)
 name=d['entered']['selected_character'];initial_world=d['entered']['world'];object_id=initial_world['status']['object_id'];wire_name=initial_world['actors'][str(object_id)]['name'];owner=d['entered']['inventory']['owner'];items=d['entered']['inventory']['items']
 for step in d['transitions']:
  source=tuple(step['source']);route=routes[source];before=step['before'];after=step['after'];expected_sources[source]+=1;expected_dest[(route['target_map'],*route['target'])]+=1
  own=before['world']['actors'][str(before['world']['status']['object_id'])]
  assert max(abs(own['x']-source[0]),abs(own['y']-source[1]))<=3
  assert after['world']['generation']==before['world']['generation']+1
  own_after=after['world']['actors'][str(after['world']['status']['object_id'])]
  assert [own_after['x'],own_after['y']]==route['target'] and after['world']['map_id']==route['target_map']
  assert after['selected_character']==name and after['inventory']['owner']==owner and after['inventory']['items']==items
  assert any(t['test_account']==account and t['character']==wire_name and t['object_id']==object_id and [t['x'],t['y']]==route['target'] and t['rental_map']==route['target_map'] for t in trace)
 target=d['walk_target'];assert any(t['test_account']==account and [t['x'],t['y']]==target for t in trace)
 assert d['after_walk']['world']['completed_portals']==2
 checks.append({'role':role,'transitions':2,'directory_identity':name,'world_wire_identity':wire_name,'object_id':object_id,'inventory_items':len(items),'post_portal_walk':target})
decode=lambda b:[int.from_bytes(b[n:n+2],'little') for n in range(1,len(b),2)]
assert collections.Counter(tuple(decode(b)) for b in enters)==expected_sources
assert collections.Counter(tuple(decode(b)) for b in portals)==expected_dest
out={'status':'verified_original_map_portal_wire_roundtrip_and_authority','checks':checks,'actual_sessions':4,'formal_C73_requests':4,'formal_S5A_replies':4,'formal_C08_acks':4,'test_binary_sha256':r['test_binary_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p/'receipt.json'),'capture_sha256':sha(cap),'server_trace_sha256':sha(p/'server-trace.jsonl'),'validator_sha256':sha(Path(__file__)),'GUI_verified':False,'physical_input_verified':False,'ordinary_WORLD_entry_verified':False,'full_product_verified':False}
(p/'map-portal-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'checks':checks}))
