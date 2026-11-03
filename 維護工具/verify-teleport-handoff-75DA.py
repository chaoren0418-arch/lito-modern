from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve()
read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');runtime=Path(r['runtime'])
assert r['status']=='verified_two_player_teleport_handoff' and r['test_exit_code']==r['server_exit_code']==0 and r['inputs_unchanged']
assert r['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
if r.get('immutable_test_binary'):assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
server=r['server'];assert server['actual_client_sessions']==6 and server['memberships_empty'] and server['authority_mutations']==0 and not server['sql_access'] and not server['state_persistent']
captures=list((runtime/'captures').glob('*/frames.jsonl'));assert len(captures)==1
frames=[json.loads(x) for x in captures[0].read_text(encoding='utf-8-sig').splitlines()]
redacted=[x for x in frames if x['plain']=='MASKED_C_LOGIN']
assert len(redacted)==6 # C_LOGIN is deliberately redacted by the formal CaptureLog.
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
outgoing=[b for x,b in packets if x['direction']=='client_to_server']
assert sum(b[0]==0xc9 for b in outgoing)==2 # Original choices only; no duplicate destination C_C9.
assert outgoing.count(bytes([0xd9]))==4 # Two direct-room entries plus two written cross-endpoint exits.
teleports=[(x,b) for x,b in packets if x['direction']=='server_to_client' and b[0]==0xf4 and 'rental-teleport' in x['note']]
assert len(teleports)==2
assert sum(x['direction']=='server_to_client' and b[0]==0xf3 and 'rental-teleport' in x['note'] for x,b in packets)==2
trace=[json.loads(x) for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
ready=read(p/'ready.json');same=r.get('scenario')=='same-endpoint';detached=r['jar_sha256']=='B33F9FA1D3173337A7BA8AC18872ADF4C037033D68F3266442C05C86A74896CC';assert not detached or r['reference_kind']=='detached_single_method_candidate';assert ready.get('same_endpoint',False)==same;checks=[];room_ids=[]
for role,account,index,ip in [('a','PeerFixture',1,'127.0.0.3'),('b','PeerFixture2',2,'127.0.0.4')]:
 data=read(p/f'{role}-handoff.json');before=data['before'];destination=data['destination'];battle=data['battle']
 assert data['new_epoch']==data['old_epoch']+1 and data['old_frame_rejected']
 assert before['selected_character']==destination['selected_character']==battle['selected_character']
 assert destination['inventory']['owner']['connection']==data['new_epoch'] and destination['inventory']['owner']['selection']>before['inventory']['owner']['selection']
 assert data.get('same_endpoint',False)==same and (data['old_peer']==data['new_peer'])==same
 assert destination['world']['actors']=={} and destination['joined_room'] is None and (same or destination['services'] is None) and destination['rental_set'] is None
 assert all(not t['world_ready'] and t['actors']==0 for t in data['transitions'])
 assert any(t['phase']=='handoff_closing' and t['inventory_owner'] is None and t.get('services') is None for t in data['transitions'])
 assert battle['world']['generation']>before['world']['generation'] and battle['world']['map_id']==564
 old=[x for x in trace if x['test_account']==account and x['rental_town'] and x['rental_inventory']][-1]
 owned=destination['inventory']['items'];assert set(str(i['id']) for i in old['rental_inventory'])==set(owned)
 for item in old['rental_inventory']:assert owned[str(item['id'])]['count']==item['count']
 live=[x for x in trace if x['test_account']==account and not x['rental_town'] and x['phase']=='BATTLE' and x['room_state']]
 assert live and all(x['character']==old['character'] and x['session_identity']!=old['session_identity'] for x in live)
 shared=next(x for x in live if len(x['room_state']['members'])==2)
 room_ids.append(shared['room'])
 old_id=str(before['world']['status']['object_id']);new_id=str(battle['world']['status']['object_id'])
 assert before['world']['actors'][old_id]['walk_action']==battle['world']['actors'][new_id]['walk_action']
 endpoint=next(b for x,b in teleports if b[1]==index)
 assert '.'.join(str(x) for x in endpoint[2:6])==('127.0.0.2' if same else ip) and int.from_bytes(endpoint[6:8],'little')==ready['b_port' if role=='a' else 'c_port']
 expected=bytes([0x27])+int(1812987996).to_bytes(4,'little')+data['action'].encode('ascii')+bytes(2)
 assert outgoing.count(expected)==1
 checks.append(dict(role=role,connection_epochs=[data['old_epoch'],data['new_epoch']],character_retained=True,loadout_objects=len(owned),loadout_matches_passive_authority=True,old_frame_rejected=True,new_map=564,report_sha256=sha(p/f'{role}-handoff.json')))
assert room_ids[0]==room_ids[1]
result=dict(status='verified_detached_same_endpoint_candidate_fresh_socket_continuation_into_shared_battle' if same and detached else 'verified_formal_same_endpoint_fresh_socket_continuation_into_shared_battle' if same else 'verified_detached_cross_endpoint_candidate_continuation_into_shared_battle' if detached else 'verified_formal_two_endpoint_continuation_into_shared_battle',receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(captures[0]),server_trace_sha256=sha(p/'server-trace.jsonl'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],checks=checks,duplicate_destination_character_requests=0,sql_access=False,authority_mutations=0,formal_deployed=False,detached_candidate=detached,gui_verified=False,full_product_verified=False)
(p/'teleport-handoff-comparison.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':result['status'],'players':2,'loadout_objects':[x['loadout_objects'] for x in checks]}))
