from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['status']=='verified_native_battle_withdrawal_widget_flow' and r['battle_withdrawal_widgets_verified']
assert r['jar_sha256']=='BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43'
s=r['server'];assert s['actual_client_sessions']==4 and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];checks=[]
for role,own in [('a',1),('b',3)]:
 before=r['withdrawal_before'][role];after=r['withdrawal_result'][role]
 assert before['phase']=='BATTLE' and after['phase']=='RESULT' and before['hp']==after['hp']>0 and before['room']==after['room']
 states=[x for x in trace if x['object_id']==own];start=next(i for i,x in enumerate(states) if x['phase']=='BATTLE');result=next(i for i in range(start+1,len(states)) if states[i]['phase']=='RESULT');left=next(i for i in range(result+1,len(states)) if states[i]['phase']=='CHANNEL' and states[i]['room']==0)
 checks.append(dict(role=role,formal_indices=[start,result,left],hp=after['hp']))
images=[]
for x in r['steps']:
 image=Path(x['png']);assert image.parent==p and sha(image)==x['png_sha256']
 if 'withdrawal-' in x['step']:images.append(image.name)
 if x['step'] in ['withdrawal-active','withdrawal-peer-active']:assert '地圖 564' in x['status'], 'active world was not rendered before withdrawal'
assert len(images)==6
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['direction']=='client_to_server' and x['plain']=='D9' for x in frames)==3
assert sum(x['direction']=='server_to_client' and x['note'].startswith('S_LITO_END_GAME winning-force=1 ') and 'reason=player-withdrawal' in x['note'] for x in frames)==2
out=dict(status='verified_native_battle_withdrawal_widgets_and_formal_authority',receipt_sha256=sha(p/'receipt.json'),exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],trace_sha256=sha(p/'server-trace.jsonl'),capture_sha256=sha(cap),checks=checks,images=images,physical_input_verified=False,full_product_verified=False)
(p/'battle-withdrawal-widget-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(out['status'])
