from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'))
sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');s=r['server'];runtime=Path(r['runtime'])
cancel_retry=r.get('scenario')=='cancel-retry'
wrong_service=r.get('scenario')=='wrong-service'
refused=r.get('scenario')=='refused'
assert r['status']==('verified_wrong_service_destination_and_fresh_login' if wrong_service else 'verified_cancel_immediate_retry' if cancel_retry else 'verified_destination_refused_and_fresh_login' if refused else 'verified_destination_timeout_and_fresh_login') and r['test_exit_code']==r['server_exit_code']==0 and r['inputs_unchanged']
assert r['jar_sha256']=='75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
if r.get('immutable_test_binary'):assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
assert s['actual_client_sessions']==(10 if wrong_service else 8) and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
assert s['full_game_reentry_verified'] and s['staged_loadouts_after_reentry']==0
for name,expected in r.get('source_pins',{}).items():assert sha(p/'source-snapshot'/name)==expected
cap=next((runtime/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['plain']=='MASKED_C_LOGIN' for x in frames)==(10 if wrong_service else 8)
packets=[(x,bytes.fromhex(x['plain'])) for x in frames if x['plain']!='MASKED_C_LOGIN']
if wrong_service:
 assert sum(x['note']=='S_KEY' for x in frames)==10
 assert sum(x['note']=='S_VERSION_CHECK accepted' for x in frames)==10
 assert sum(x['note']=='S_LITO_LOGIN_OK' for x in frames)==6 # two initial, two wrong service, two recovery LOGINs

outgoing=[b for x,b in packets if x['direction']=='client_to_server']
assert sum(b[0]==0xc9 for b in outgoing)==2 # Reentry consumes the staged server selection once, without another C_C9.
assert outgoing.count(bytes([0xd9]))==6 # Two initial room entries, two endpoint exits and two recovered room entries.
assert sum(x['direction']=='server_to_client' and b[0]==0xf4 and 'rental-teleport' in x['note'] for x,b in packets)==2
trace=[json.loads(x) for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];checks=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 data=read(p/f'{role}-failure.json');before=data['before'];failed=data['failed'];recovery=data['recovery'];continued=data['continued'];reentered=data['reentered']
 assert data['full_game_reentry_verified']
 if cancel_retry:
  assert data['scenario']=='cancel-retry' and data['elapsed_ms']<5000 and 600<=data['expiry_wait_ms']<5000 and not data['manual_selection'] and data['closed_samples']>0
 elif wrong_service:assert data['scenario']=='wrong-service' and data['elapsed_ms']<15000 and failed['error'] and read(p/'ready.json')['destination_wrong_service']
 elif refused:assert data['scenario']=='refused' and data['elapsed_ms']<5000 and 'native TCP connect' in failed['error'] and read(p/'ready.json')['destination_refused']
 else:assert 5000<=data['elapsed_ms']<15000
 assert [data['failed_epoch'],data['recovery_epoch'],data['reentered_epoch']]==[data['old_epoch']+n for n in [1,2,3]]
 assert failed['phase']==('closed' if cancel_retry else 'failed') and failed['world']['actors']=={} and failed['inventory']['owner'] is None and failed['selected_character'] is None and failed['joined_room'] is None and failed['merchant']['owner'] is None
 assert recovery['phase']=='sources_ready' and recovery['world']['actors']=={} and recovery['inventory']['owner'] is None
 assert before['selected_character']==continued['selected_character']==reentered['selected_character']
 assert continued['phase']=='character_selected' and continued['inventory']['owner']['connection']==data['reentered_epoch']
 assert reentered['phase']=='battle_pending' and reentered['world']['map_id']==500 and reentered['world']['generation']>before['world']['generation']
 own=reentered['world']['actors'][str(reentered['world']['status']['object_id'])];assert data['recovery_move']['before']==[own['x'],own['y']]
 old=next(x for x in trace if x['test_account']==account and x['rental_town'] and x['rental_inventory'])
 live=[x for x in trace if x['test_account']==account and x['rental_town'] and x['phase']=='BATTLE' and x['session_identity']!=old['session_identity']]
 assert live and all(x['character']==old['character'] for x in live)
 assert any([x['x'],x['y']]==data['recovery_move']['after'] for x in live)
 granted=continued['inventory']['items'];staged=old['rental_inventory']
 assert set(granted)==set(str(i['id']) for i in staged)
 for item in staged:assert granted[str(item['id'])]['count']==item['count']
 if cancel_retry:
  stages=[json.loads(x) for x in (p/'staged-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
  ids=set(i['id'] for i in staged)
  observed=[(x['observed_now_nanos'],v) for x in stages for v in x['staged'] if set(v['inventory_ids'])==ids]
  assert observed and all(not v['expired'] and now<v['expires_at_nanos'] for now,v in observed)
  assert failed['error'] is None
 checks.append(dict(role=role,elapsed_ms=data['elapsed_ms'],epochs=[data['old_epoch'],data['failed_epoch'],data['recovery_epoch'],data['reentered_epoch']],old_authority_cleared=True,formal_staged_loadout_consumed=True,new_world_generation=reentered['world']['generation'],authoritative_recovery_move=data['recovery_move'],report_sha256=sha(p/f'{role}-failure.json')))
result=dict(status=('verified_wrong_LOGIN_destination_and_full_same_source_reentry' if wrong_service else 'verified_cancel_immediate_retry_staged_character_and_loadout' if cancel_retry else 'verified_TCP_refused_and_full_same_source_reentry' if refused else 'verified_destination_handshake_timeout_and_full_same_source_reentry'),receipt_sha256=sha(p/'receipt.json'),capture_sha256=sha(cap),server_trace_sha256=sha(p/'server-trace.jsonl'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],checks=checks,staged_loadouts_after_reentry=0,formal_deployed=False,sql_access=False,authority_mutations=0,gui_verified=False,full_product_verified=False,validator_sha256=sha(Path(__file__)),reference_is_current=r.get("reference_is_current"),observed_source_before_sha256=r.get("observed_source_before_sha256"),observed_source_after_sha256=r.get("observed_source_after_sha256"))
(p/('teleport-wrong-service-comparison.json' if wrong_service else 'teleport-cancel-retry-comparison.json' if cancel_retry else 'teleport-refused-comparison.json' if refused else 'teleport-failure-comparison.json')).write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':result['status'],'players':2,'epochs':[x['epochs'] for x in checks]}))
