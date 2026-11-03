"""No injected authority: match rented keys and return against formal lease state."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-hotel' and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty']
trace=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];tokens=set()
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;q=c['hotel'];states=[s for s in trace if s['object_id']==own and s['rental_town']]
 assert not q['server_authority_injection'] and not q['gui_verified'] and not q['room_entry_verified']
 assert c['worker_quiescent'] and c['shutdown']=='closed'
 before=q['before']['inventory']['items'];rented=q['rented']['inventory']['items'];returned=q['returned']['inventory']['items'];n=q['quantity']
 assert n==(8 if role=='a' else 1) and returned==before and q['repeated']['inventory']['items']==rented
 ids=set(rented)-set(before);assert len(ids)==n and all(rented[k]['count']==1 for k in ids)
 assert {k:v for k,v in rented.items() if k not in ids}==before
 request=q['requested']['merchant']['input'];assert [request[k] for k in ['minimum','current','maximum','mode']]==[1,1,8,0]
 assert q['opening']['html']=='inn' and q['rented']['merchant']['dialog']['html']=='inn4' and q['returned']['merchant']['dialog']['html']=='inn20'
 cursor=0
 for step in q['steps']:
  index=next((j for j in range(cursor,len(states)) if [states[j]['x'],states[j]['y']]==step['after']),None);assert index is not None;cursor=index
 pending=next(j for j in range(cursor,len(states)) if states[j]['hotel_input_pending']);
 granted=next(j for j in range(pending,len(states)) if states[j]['rental_hotel'] is not None and states[j]['rental_hotel']['keys']==n and {v['id'] for v in states[j]['rental_inventory']} >= {int(k) for k in ids})
 token=states[granted]['rental_hotel']['token'];assert token>0 and token not in tokens;tokens.add(token)
 released=next(j for j in range(granted+1,len(states)) if states[j]['rental_hotel'] is None and not ({v['id'] for v in states[j]['rental_inventory']} & {int(k) for k in ids}))
 assert all(s['rental_gems']==20 for s in states[pending:released+1])
 image=Path(q['render']['png']);assert image.is_file()
 checks.append({'role':role,'quantity':n,'key_ids':sorted(ids),'pending_index':pending,'granted_index':granted,'released_index':released,'lease_token':token,'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
report={'status':'verified_hotel_key_request_and_return','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'gui_verified':False,'room_entry_verified':False,'shared_keys_and_expiry_verified':False}
(p.parent/'rental-hotel-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'checks':len(checks)}))
