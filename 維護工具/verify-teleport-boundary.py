from pathlib import Path
import json,hashlib,sys,re
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_unconfigured_teleport_boundary' and r['test_exit_code']==r['server_exit_code']==0
assert r['jar_sha256']=='BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'
s=r['server'];assert s['actual_client_sessions']==4 and s['memberships_empty'] and s['active_hotel_leases_after_disconnect']==0 and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
runtime=Path(r['runtime']);config=(runtime/'server.properties').read_text(encoding='utf-8-sig')
assert 'lito.rental.teleport.routes=' not in config
cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
requests=[bytes.fromhex(x['plain']) for x in frames if x['direction']=='client_to_server' and x['opcode']=='0x27']
assert len(requests)==8
# LOGIN source selection legitimately emits S_F4 before entering the town.
# This shared CaptureLog contains all four sessions; distinguish its notes.
redirects=[x for x in frames if x['direction']=='server_to_client' and x['opcode']=='0xF4']
assert len(redirects)==2 and all('rental-teleport' not in x['note'] for x in redirects)
assert sum(x['direction']=='client_to_server' and x['opcode']=='0xD9' for x in frames)==2 # entry only
assert sum(x['direction']=='client_to_server' and x['opcode']=='0x29' for x in frames)==16
log=(p/'server.log').read_text(encoding='utf-8-sig')
deferred=re.findall(r'teleporter deferred resource=(\d+) html=TeleportMe action=(LT_TELEPORT_[BC]) result=server-owned-endpoint-not-configured',log)
assert sorted(deferred)==[(str(n),action) for n in range(92,96) for action in ['LT_TELEPORT_B','LT_TELEPORT_C']]
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];all_npcs=[]
for role,own in [('a',1),('b',3)]:
 rows=read(p/f'{role}-teleport-boundary.json');assert len(rows)==4
 positions=[x for x in trace if x['object_id']==own and x['phase']=='BATTLE'];cursor=0;visited=set();movement=0
 for x in rows:
  npc=x['npc'];all_npcs.append(npc);before=x['before'];after=x['after']
  wire=bytes([0x27])+npc.to_bytes(4,'little')+x['action'].encode('ascii')+bytes(2)
  assert requests.count(wire)==1 and x['dialog']['html']=='TeleportMe'
  assert x['elapsed_ms']>=8000 and after['merchant']['timed_out'] and not after['merchant']['waiting']
  assert not x['reopened']['timed_out'] and not x['reopened']['waiting'] and x['reopened']['dialog']['npc']==npc
  assert x['authority_before']==x['authority_after']
  assert x['authority_after']['phase']=='BATTLE' and x['authority_after']['room']==60001 and x['authority_after']['pending_transition']=='NONE'
  assert before['world']['generation']==after['world']['generation'] and before['inventory']==after['inventory']
  assert before['selected_character']==after['selected_character']
  if npc not in visited:
   visited.add(npc)
   for step in x['steps']:
    cursor=next(i for i in range(cursor,len(positions)) if [positions[i]['x'],positions[i]['y']]==step['before'])
    cursor=next(i for i in range(cursor+1,len(positions)) if [positions[i]['x'],positions[i]['y']]==step['after'])
    movement+=1
 checks.append(dict(role=role,actions=len(rows),npcs=sorted(visited),actual_movement_steps=movement,steps_sha256=sha(p/f'{role}-teleport-boundary.json')))
assert sorted(all_npcs)==sorted([n for n in range(1812987996,1812988000) for _ in range(2)])
out=dict(status='verified_four_unconfigured_npcs_eight_bounded_timeouts_and_dialogue_recovery',jar_sha256=r['jar_sha256'],test_binary_sha256=r['test_binary_sha256'],receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),checks=checks,teleport_endpoints_received=0,gameplay_handoff_verified=False,gui_verified=False,authority_mutations=0,formal_deployed=False,full_product_verified=False)
(p/'teleport-boundary-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'actions':8,'movement_steps':sum(x['actual_movement_steps'] for x in checks)}))
