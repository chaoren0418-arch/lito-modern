"""Compare native room/portal captures and passive siege authority."""
from pathlib import Path
import collections, hashlib, json, re, sys
p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');runtime=Path(r['runtime']);mapid=r['map_id']
assert r['status']=='verified_native_siege_portal' and r['inputs_unchanged']
assert r['test_exit_code']==r['server_exit_code']==0
assert r['server']['actual_client_sessions']==60 and r['server']['memberships_empty']
assert r['server']['authority_mutations']==0 and not r['server']['sql_access']
assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
for f,h in r['source_pins'].items():assert sha(p/'source-snapshot'/f)==h
root=Path(__file__).resolve().parents[1]
cat=root/'驗證紀錄/current/native-world-integration/C02-ArenaPortalCatalog.javap.txt'
rows={}
for m in re.finditer(r'// String ([0-9, -]+)',cat.read_text(encoding='utf-8-sig')):
 for word in m[1].split():
  if not re.fullmatch(r'\d+,\d+,\d+,\d+,\d+,-?\d+',word):continue
  parts=list(map(int,word.split(',')))
  if len(parts)==6 and parts[2]==mapid:rows[tuple(parts[:2])]=parts[2:]
assert rows
cap=next((runtime/'captures').glob('*/frames.jsonl'))
frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==60
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
client=[b for x,b in packets if x['direction']=='client_to_server']
server=[b for x,b in packets if x['direction']=='server_to_client']
decode=lambda b:tuple(int.from_bytes(b[n:n+2],'little') for n in range(1,len(b),2))
mapreq=[b for b in client if b[:2]==bytes([0xd1,3])]
assert [int.from_bytes(b[2:],'little') for b in mapreq]==([563,573] if mapid==573 else [573,563])
assert len([b for b in client if b[0]==0x73])==4
assert len([b for b in server if b[0]==0x5a])==2
assert client.count(bytes([8]))==2
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
expected_src=collections.Counter();expected_dst=collections.Counter();roles=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 d=read(p/f'{role}-arena-portal.json');assert d['map']==mapid and len(d['transitions'])==2
 initial=d['entered'];oid=initial['world']['status']['object_id'];wire=initial['world']['actors'][str(oid)]['name']
 room=initial['joined_room'];member=next(m for m in room['members'] if m['name']==initial['selected_character']);force=member['force']
 states=[t for t in trace if t['test_account']==account and t['object_id']==oid and t['phase']=='BATTLE' and t['room']==room['id'] and t['siege_attacker_force'] in [0,1]]
 assert states
 source_names={t['character'] for t in states};assert len(source_names)==1
 source_name=source_names.pop();assert source_name and all(t['source']=='測試乙' and source_name in t['available_characters'] for t in states)
 assert len(room['members'])==30 and sum(m['force']==0 for m in room['members'])==15 and sum(m['force']==1 for m in room['members'])==15
 attacker=states[0]['siege_attacker_force'];assert all(t['siege_attacker_force']==attacker for t in states)
 admitted=d['admitted'];assert admitted==(force!=attacker)
 for step in d['transitions']:
  before=step['before'];after=step['after'];assert step['admitted']==admitted
  a=before['world']['actors'][str(oid)];z=after['world']['actors'][str(oid)]
  source=min(rows,key=lambda s:max(abs(a['x']-s[0]),abs(a['y']-s[1])))
  assert max(abs(a['x']-source[0]),abs(a['y']-source[1]))<=3
  expected_src[source]+=1
  assert after['selected_character']==initial['selected_character']
  assert after['inventory']['owner']==initial['inventory']['owner'] and after['inventory']['items']==initial['inventory']['items']
  assert after['world']['map_id']==mapid and after['world']['local_corrections']==before['world']['local_corrections']
  assert after['world']['generation']==before['world']['generation']+int(admitted)
  if admitted:
   dest=rows[source];expected_dst[tuple(dest[:3])]+=1;assert [z['x'],z['y']]==dest[1:3]
   assert any(t['character']==source_name and [t['x'],t['y']]==dest[1:3] for t in states)
  else:assert [a['x'],a['y']]==[z['x'],z['y']]
 assert any([t['x'],t['y']]==d['walk_target'] for t in states)
 assert d['after_walk']['world']['completed_portals']==(2 if admitted else 0)
 roles.append({'role':role,'wire_display_name':wire,'source_character':source_name,'force':force,'attacker_force':attacker,'admitted':admitted,'original_tag':rows[source][-1],'inventory_items':len(initial['inventory']['items']),'post_portal_walk':d['walk_target']})
extras=list(p.glob('e*-siege-member.json'));assert len(extras)==28
identities=set()
for f in extras:
 d=read(f);w=d['battle']['world'];oid=w['status']['object_id'];assert w['map_id']==mapid and str(oid) in w['actors'] and w['hp'][0]>0
 assert oid not in identities;identities.add(oid)
 assert any(t['test_account']==d['account'] and t['object_id']==oid and t['phase']=='BATTLE' for t in trace)
assert sum(x['admitted'] for x in roles)==1
assert collections.Counter(decode(b) for b in client if b[0]==0x73)==expected_src
assert collections.Counter(decode(b) for b in server if b[0]==0x5a)==expected_dst
out={'status':'verified_native_siege_wire_and_authority','map':mapid,'checks':roles,'actual_sessions':60,'C_D1_map_requests':2,'C73_requests':4,'S5A_replies':2,'C08_acks':2,'test_binary_sha256':r['test_binary_sha256'],'jar_sha256':r['jar_sha256'],'receipt_sha256':sha(p/'receipt.json'),'capture_sha256':sha(cap),'trace_sha256':sha(p/'server-trace.jsonl'),'catalogue_bytecode_sha256':sha(cat),'validator_sha256':sha(Path(__file__)),'GUI_verified':False,'physical_input_verified':False,'full_product_verified':False}
(p/'arena-portal-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(out,ensure_ascii=False))
