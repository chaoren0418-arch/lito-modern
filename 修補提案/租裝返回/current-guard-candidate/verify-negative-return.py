"""Independently compare test steps, passive authority snapshots and actual wire."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_candidate_negative_return_requests'
assert r['test_exit_code']==r['server_exit_code']==0
s=r['server'];assert s['actual_client_sessions']==4 and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent'] and s['active_hotel_leases_after_disconnect']==0
root=Path(r['runtime']);cap=next((root/'captures').glob('*/frames.jsonl'))
frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
controls=[bytes.fromhex(x['plain']) for x in frames if x['direction']=='client_to_server' and x['opcode'] in ['0x10','0xD9','0xD0']]
for op,count in [(0x10,6),(0xd9,8),(0xd0,8)]:assert controls.count(bytes([op]))==count # D9 includes two ordinary rental-entry handshakes
server=[bytes.fromhex(x['plain']) for x in frames if x['direction']=='server_to_client']
assert server.count(bytes([0x65]))==2 and server.count(bytes([0xf0,0]))==4
assert sum(x['direction']=='server_to_client' and x['note']=='S_LITO_QUIT_FIGHT restart-transition' for x in frames)==2
assert 2<=sum(b[0]==0xd0 for b in server)<=4 # self acknowledgements plus peer notifications
assert server.count(bytes([0xf5]))==4 # two rental entry permissions, two completed returns
checks=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 rows=read(p/f'{role}-negative.json');assert len(rows)==10
 invalid=0
 for x in rows:
  assert x['send_confirmed']
  for side in ['before','after']:
   path=root/f'{role}-observe-{x["label"]}-{side}.json'
   assert read(path)==x[side] and x[side]['test_account']==account
  before=x['before'];after=x['after']
  if x['label'] not in ['restart','quit','return']:
   assert before==after, f'{role}/{x["label"]} changed authoritative state'
   identity=f'{account}(測試乙)'
   assert not any(b[0] in [0x65,0xf0,0xf5] or (b[0]==0xd0 and bytes(b[5:-1]).decode('cp950')==identity) for b in x['received'])
   invalid+=1
  elif x['label']=='restart':assert before['pending_transition']=='NONE' and after['pending_transition']=='RESTART_AWAIT_QUIT_FIGHT'
  elif x['label']=='quit':assert before['pending_transition']=='RESTART_AWAIT_QUIT_FIGHT' and after['pending_transition']=='RESTART_AWAIT_RETURN'
  else:
   assert before['pending_transition']=='RESTART_AWAIT_RETURN' and after['pending_transition']=='RESTART_AWAIT_SOURCE_REFRESH'
   assert after['room']==0 and not after['rental_town'] and not after['rental_inventory']
  if x['label']!='return':
   for key in ['phase','room','rental_inventory','rental_hotel','rental_gems','character']:assert before[key]==after[key]
 checks.append(dict(role=role,invalid_stages=invalid,valid_stages=3,steps_sha256=sha(p/f'{role}-negative.json')))
out=dict(status='verified_fourteen_ignored_negative_requests_and_two_complete_return_handshakes',jar_sha256=r['jar_sha256'],test_binary_sha256=r['test_binary_sha256'],receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),checks=checks,actual_client_sessions=4,application_auto_ack_bypassed_only_in_test_binary=True,authority_mutations=0,formal_deployed=False,malformed_payloads_verified=False,normal_reentry_verified_by_separate_candidate_evidence=True,full_product_verified=False)
(p/'negative-return-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'invalid_requests':14}))
