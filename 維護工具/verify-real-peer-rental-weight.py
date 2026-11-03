import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-weight' and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty']
assert 'strength/constitution/originalStrength/originalConstitution 1' in r['server']['initial_fixture']
states=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[]
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;q=c['weight'];before=q['before']['inventory']['items'];gems=before['1811939330']['count']
 assert not q['live_authority_mutations'] and not q['normal_character_weight_reachability_verified'] and not q['gui_verified']
 assert len(q['refusals'])==3 and c['worker_quiescent'] and c['shutdown']=='closed'
 assert q['before']['world']['status']['strength']==2 and q['before']['world']['status']['constitution']==1
 for step,(quantity,code) in zip(q['refusals'],[(1,82),(257,263),(1,82)]):
  assert step['quantity']==quantity and step['expected_code']==code
  assert step['snapshot']['world']['last_message']==[code,[]] and step['snapshot']['inventory']['items']==before
  assert step['snapshot']['merchant']['listing'] is None
  rows=step['listing']['listing']['Buy']['rows'];assert min(row['price'] for row in rows)<=gems
 authority=[s for s in states if s['object_id']==own and s['rental_town']]
 base=next(s for s in authority if s['rental_inventory'])
 assert all(s['rental_gems']==gems and s['rental_inventory']==base['rental_inventory'] for s in authority if s['rental_inventory'])
 checks.append({'role':role,'expected_codes':[82,263,82],'balance':gems,'unchanged_inventory':True})
log=(p.parent/'server.log').read_text(encoding='utf-8-sig')
assert log.count('reason=inventory-weight')==4 and log.count('reason=inventory-capacity')==2
assert 'C_BUY_SELL rental-town accepted' not in log
out={'status':'verified_weight_and_capacity_rejections_with_explicit_initial_low_stat_fixture','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'live_authority_mutations':False,'ordinary_character_reachability_verified':False,'gui_verified':False}
(p.parent/'rental-weight-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':out['status'],'checks':len(checks)}))
