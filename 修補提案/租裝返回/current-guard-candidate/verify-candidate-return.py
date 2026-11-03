"""Compare actual candidate return/reentry with passive session identity and wire captures."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['candidate_server'] and r['formal_server_unchanged']
assert r['jar_sha256']=='6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7'
assert r['formal_baseline_sha256']=='BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'
server=r['server'];assert server['actual_client_sessions']==4 and server['game_sessions']==2 and server['memberships_empty']
assert server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
trace=[json.loads(l) for l in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];sessions=set()
for role,account,own,other in [('a','PeerFixture',1,3),('b','PeerFixture2',3,1)]:
 c=r['clients'][role];q=c['return_observation'];assert c['normal_return_verified'] and c['worker_quiescent'] and c['shutdown']=='closed'
 assert q['characters_ready'] and q['reentry_verified']
 before=q['before'];retired=q['after'];reentry=q['reentered']
 assert retired['phase']=='characters_ready' and retired['joined_room'] is None and retired['world']['map_id'] is None
 assert retired['world']['actors']=={} and retired['inventory']['items']=={} and retired['inventory']['owner'] is None and retired['merchant']['dialog'] is None
 assert retired['directory_generation']>before['directory_generation'] and retired['rental_set'] is None
 assert reentry['inventory']['selection']>before['inventory']['selection'] and reentry['inventory']['owner']['selection']==reentry['inventory']['selection']
 assert reentry['inventory']['owner']['connection']==before['inventory']['owner']['connection']
 assert reentry['phase']=='battle_pending' and reentry['world']['map_id']==500 and reentry['world']['generation']>before['world']['generation']
 assert reentry['world']['status']['object_id']==own and str(other) in reentry['world']['actors'] and reentry['inventory']['items']
 initial=next(x for x in trace if x['state']['test_account']==account and x['state']['phase']=='BATTLE' and x['state']['rental_town'])
 identity=initial['state']['session_identity'];assert identity not in sessions;sessions.add(identity)
 states=[x for x in trace if x['state']['session_identity']==identity]
 first=next(i for i,x in enumerate(states) if x['state']['phase']=='BATTLE' and x['state']['pending_transition']=='NONE')
 requested=next(i for i in range(first+1,len(states)) if states[i]['state']['pending_transition']=='RESTART_AWAIT_QUIT_FIGHT')
 acked=next(i for i in range(requested+1,len(states)) if states[i]['state']['pending_transition']=='RESTART_AWAIT_RETURN')
 left=next(i for i in range(acked+1,len(states)) if states[i]['state']['pending_transition']=='RESTART_AWAIT_SOURCE_REFRESH')
 empty=states[left]['state'];assert empty['phase']=='LOBBY' and not empty['rental_town'] and empty['room']==0 and empty['rental_inventory']==[] and empty['rental_set']==-1
 fresh=next(i for i in range(left+1,len(states)) if states[i]['state']['phase']=='BATTLE' and states[i]['state']['rental_town'] and states[i]['state']['pending_transition']=='NONE')
 assert states[fresh]['state']['object_id']==own and states[fresh]['state']['rental_map']==500
 catalog=lambda s:sorted((x['catalog_id'],x['count'],x['equipped']) for x in s['rental_inventory'])
 assert catalog(states[first]['state'])==catalog(states[fresh]['state'])
 frame=Path(q['reentry_render']['png']);assert frame.is_file()
 checks.append(dict(role=role,session_identity=identity,formal_indices=[first,requested,acked,left,fresh],directory_generations=[before['directory_generation'],retired['directory_generation']],world_generations=[before['world']['generation'],reentry['world']['generation']],source_equipment_recreated=len(states[fresh]['state']['rental_inventory']),png=str(frame),png_sha256=sha(frame)))
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
counts={name:sum(1 for x in frames if condition(x)) for name,condition in {
 'restart_request':lambda x:x['direction']=='client_to_server' and x['plain']=='10',
 'restart_ack':lambda x:x['direction']=='server_to_client' and x['plain']=='65',
 'quit_ack':lambda x:x['note']=='S_LITO_QUIT_FIGHT restart-transition',
 'return_request':lambda x:x['direction']=='client_to_server' and x['plain']=='D0',
 'room_exit_ack':lambda x:x['note'].startswith('S_LITO_EXIT_ROOM rental-town restart player='),
 'source_permission':lambda x:x['note']=='S_LITO_VS_SERVER_ALLOW_JOIN restart source-refresh',
}.items()};assert all(n==2 for n in counts.values()),counts
out=dict(status='verified_two_candidate_rental_returns_and_reentries',receipt_sha256=sha(p/'receipt.json'),trace_sha256=sha(p/'server-trace.jsonl'),capture=str(cap),capture_sha256=sha(cap),exe_sha256=r['exe_sha256'],candidate_jar_sha256=r['jar_sha256'],formal_jar_sha256=r['formal_baseline_sha256'],checks=checks,wire_counts=counts,authority_mutations=False,formal_deployed=False,active_hotel_departure_verified=False,purchased_merchant_departure_verified=False,ordinary_battle_withdrawal_verified=False,duplicate_ack_verified=False,gui_verified=False,full_product_verified=False)
(p/'candidate-return-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps(dict(status=out['status'],roles=len(checks),wire_counts=counts)))
