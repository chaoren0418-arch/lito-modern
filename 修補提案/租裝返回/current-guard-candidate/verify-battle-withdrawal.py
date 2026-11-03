"""Actual ordinary battle withdrawals: no synthetic deaths or injected authority."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');assert r['battle_withdrawal'] and r['formal_server_unchanged']
assert r['jar_sha256']==('6887A337E347D561366E0EB7EB8E93A2B7EA03ADD7A3888FC4BE2B70C3479DA7' if r['candidate_server'] else 'BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43')
s=r['server'];assert s['actual_client_sessions']==4 and s['game_sessions']==2 and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
trace=[json.loads(x)['state'] for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()];checks=[]
for role,own in [('a',1),('b',3)]:
 c=r['clients'][role];assert c['battle_withdrawal'] and len(c['rounds'])==2 and c['worker_quiescent'] and c['shutdown']=='closed'
 assert c['final_lobby']['phase']=='lobby_ready' and int(c['transport']['epoch'])==2
 results=[];last=None
 for x in trace:
  if x['object_id']==own:
   if x['phase']=='RESULT' and last!='RESULT':results.append(x)
   last=x['phase']
 assert len(results)==2
 for n,row in enumerate(c['rounds']):
  w=row['end']['world'];initial=row['start']['world'];actor=w['actors'][str(own)];formal=results[n]
  assert row['withdrawal_requested']==((role=='a')==(n==0))
  assert row['attacks_sent']==row['moves_sent']==w['own_attack_packets']==w['death_confirmations_sent']==0
  assert w['battle_result']==1-n and len(w['scores'])==2 and all(x['kills']==x['deaths']==0 for x in w['scores'])
  assert not actor['dead'] and w['hp'][0]==initial['hp'][0]>0 and w['hp'][0]==formal['hp'] and w['mp'][0]==formal['mp']
  assert [actor['x'],actor['y']]==[formal['x'],formal['y']] and row['end']['joined_room']['id']==formal['room']
  assert not row['end']['battle_withdrawal_pending'] and initial['generation']==n+1
  images=[]
  for image in [row['start']['render'],row['render']]:
   assert image['frame']['map_id']==564;images.append(sha(Path(image['png'])))
  other=r['clients']['b' if role=='a' else 'a']['rounds'][n]['end']['world'];assert other['scores']==w['scores'] and other['battle_result']==w['battle_result']
  checks.append(dict(role=role,round=n+1,requested=row['withdrawal_requested'],result=w['battle_result'],hp=w['hp'],images=images))
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(x) for x in cap.read_text(encoding='utf-8-sig').splitlines()]
counts={'quit_requests':sum(x['direction']=='client_to_server' and x['plain']=='D9' for x in frames),'room_exit_requests':sum(x['direction']=='client_to_server' and x['plain']=='D0' for x in frames),'withdrawal_results':sum(x['direction']=='server_to_client' and x['note'].startswith('S_LITO_END_GAME ') and 'reason=player-withdrawal' in x['note'] for x in frames)}
assert counts==dict(quit_requests=6,room_exit_requests=4,withdrawal_results=4),counts
out=dict(status='verified_two_live_ordinary_battle_withdrawals',exe_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],candidate_server=r['candidate_server'],receipt_sha256=sha(p/'receipt.json'),trace_sha256=sha(p/'server-trace.jsonl'),capture_sha256=sha(cap),checks=checks,wire_counts=counts,duplicate_client_request_rejected_before_send=True,combat_cooldown_live_verified=False,formal_deployed=False,full_product_verified=False)
(p/'battle-withdrawal-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({'status':out['status'],'candidate_server':r['candidate_server'],'wire_counts':counts}))
