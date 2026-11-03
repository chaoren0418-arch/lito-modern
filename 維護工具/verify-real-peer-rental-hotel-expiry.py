"""Real wall-clock lease expiry, independently observed on the unmodified formal core."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile'] in ('rental-hotel-expiry','rental-hotel-middle-expiry') and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty'] and r['server']['active_hotel_leases_after_disconnect']==0
rows=[json.loads(s) for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
log=(p.parent/'server.log').read_text(encoding='utf-8-sig')
checks=[]
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;q=c['hotel'];natural=q['natural_expiry'];n=q['quantity'];seconds=7200 if n>=5 else 3600
 assert c['worker_quiescent'] and c['shutdown']=='closed' and not natural['clock_or_authority_injection']
 assert natural['expected_seconds']==seconds and (seconds-2)*1000<=natural['elapsed_ms']<=(seconds+120)*1000
 before=q['before']['inventory']['items'];keys=set(q['rented']['inventory']['items'])-set(before)
 returned=natural['returned'];w=returned['world'];actor=w['actors'][str(own)]
 assert len(keys)==n and returned['inventory']['items']==before
 assert [w['map_id'],actor['x'],actor['y'],actor['heading']]==[500,32779,32832,2]
 assert w['generation']==q['before']['world']['generation']+2 and w['completed_portals']==q['before']['world']['completed_portals']+2 and w['portal'] is None
 assert w['status']['object_id']==own and '1812987995' in w['actors']
 assert returned['merchant']['owner'] is None and returned['merchant']['dialog'] is None
 states=[v for v in rows if v['state']['object_id']==own and v['state']['rental_town']]
 granted=next(j for j,v in enumerate(states) if v['state']['rental_hotel'] is not None and v['state']['rental_hotel']['keys']==n and {i['id'] for i in v['state']['rental_inventory']} >= {int(k) for k in keys})
 lease=states[granted]['state']['rental_hotel'];token=lease['token']
 arrived=next(j for j in range(granted,len(states)) if states[j]['state']['rental_map']==q['arrival']['world']['map_id'])
 expired=next(j for j in range(arrived+1,len(states)) if states[j]['state']['rental_map']==500 and states[j]['state']['rental_hotel'] is None and not ({i['id'] for i in states[j]['state']['rental_inventory']}&{int(k) for k in keys}))
 elapsed=states[expired]['ms']-states[granted]['ms'];assert (seconds-3)*1000<=elapsed<=(seconds+120)*1000
 assert all(v['state']['rental_gems']==20 for v in states[granted:expired+1])
 assert f'rental-hotel lease invalidated token={token} reason=rental-hotel-expiry keys={n}' in log
 image=Path(natural['render']['png']);assert image.is_file()
 checks.append({'role':role,'map':q['arrival']['world']['map_id'],'keys':n,'lease_token':token,'expected_seconds':seconds,'observed_client_ms':natural['elapsed_ms'],'observed_server_ms':elapsed,'granted_index':granted,'expired_index':expired,'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
start=json.loads((p.parent/'natural-expiry-start.json').read_text(encoding='utf-8-sig'))
assert start['exe_sha256']==r['exe_sha256'] and start['jar_sha256']==r['jar_sha256'] and not start['clock_or_authority_injection']
with Path(start['immutable_exe']).open('rb') as f:assert hashlib.file_digest(f,'sha256').hexdigest().upper()==r['exe_sha256'].upper()
report={'status':'verified_natural_hotel_expiry','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'gui_verified':False,'manual_guest_exit_verified':False,'authority_or_clock_injection':False}
(p.parent/'rental-hotel-expiry-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'maps':[v['map'] for v in checks]}))
