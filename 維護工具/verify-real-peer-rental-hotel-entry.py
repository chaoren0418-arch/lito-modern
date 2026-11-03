"""Compare actual S5A/C08 arrivals with passive formal authority and original room frames."""
import hashlib,json,sys
from pathlib import Path
from native_hotel_catalog_reference import fixture_reference
from native_resident_map_reference import qualify as qualify_resident_terrain
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile'] in ('rental-hotel-entry','rental-hotel-middle-entry','rental-hotel-expiry','rental-hotel-middle-expiry') and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty']
assert r['server']['active_hotel_leases_after_disconnect']==0
fixture_path=Path(__file__).with_name('native-portal-current.json')
if not fixture_path.exists():fixture_path=Path(__file__).resolve().parents[1]/'LiTo381-Modern/tests/fixtures/native-portal-current.json'
fixture=json.loads(fixture_path.read_text(encoding='utf-8'))
fixture_proof=fixture_reference(fixture_path,r['jar_sha256'])
trace=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];tokens=set()
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;q=c['hotel'];states=[s for s in trace if s['object_id']==own and s['rental_town']]
 assert c['worker_quiescent'] and c['shutdown']=='closed' and q['room_entry_verified'] and not q['gui_verified'] and not q['server_authority_injection']
 n=q['quantity'];expected=(4 if role=='a' else 2) if r['action_profile'] in ('rental-hotel-middle-entry','rental-hotel-middle-expiry') else (8 if role=='a' else 1)
 assert n==expected
 destination=next(s for s in fixture['rooms'] if s['keys_min']<=n<=s['keys_max'])
 before=q['before']['inventory']['items'];rented=q['rented']['inventory']['items'];arrival=q['arrival'];w=arrival['world'];ids=set(rented)-set(before)
 assert len(ids)==n and all(rented[k]['count']==1 for k in ids)
 assert {k:v for k,v in rented.items() if k not in ids}==before
 assert arrival['inventory']['items']==rented==q['repeated']['inventory']['items']
 assert q['returned'] is None and w['portal'] is None
 assert w['completed_portals']==q['before']['world']['completed_portals']+1 and w['generation']==q['before']['world']['generation']+1
 assert set(w['actors'])=={str(own)} and w['status']['object_id']==own
 actor=w['actors'][str(own)]
 assert [w['map_id'],actor['x'],actor['y'],actor['heading']]==[destination[k] for k in ['map','x','y','heading']]
 assert arrival['joined_room']['map_id']==w['map_id'] and arrival['joined_room']['mode']==6
 assert arrival['merchant']['owner'] is None and arrival['merchant']['dialog'] is None and arrival['merchant']['input'] is None and not arrival['merchant']['waiting']
 request=q['requested']['merchant']['input'];assert [request[k] for k in ['minimum','current','maximum','mode']]==[1,1,8,0]
 cursor=0
 for step in q['steps']:
  cursor=next(j for j in range(cursor,len(states)) if [states[j]['x'],states[j]['y']]==step['after'])
 pending=next(j for j in range(cursor,len(states)) if states[j]['hotel_input_pending'])
 granted=next(j for j in range(pending,len(states)) if states[j]['rental_hotel'] is not None and states[j]['rental_hotel']['keys']==n and {v['id'] for v in states[j]['rental_inventory']} >= {int(k) for k in ids})
 token=states[granted]['rental_hotel']['token'];assert token>0 and token not in tokens;tokens.add(token)
 arrived=next(j for j in range(granted,len(states)) if states[j]['rental_map']==w['map_id'] and states[j]['x']==actor['x'] and states[j]['y']==actor['y'] and states[j]['rental_hotel'] is not None and states[j]['rental_hotel']['token']==token)
 assert all(s['rental_gems']==20 for s in states[pending:arrived+1])
 for key in ids:assert any(i['id']==int(key) and i['count']==1 for i in states[arrived]['rental_inventory'])
 frame=q['room_render'];image=Path(frame['png']);assert image.is_file()
 budget=frame['frame']['static_resource_budget'];assert 0<budget['complete_sector_count']<=64 and 0<budget['decoded_bytes']<=budget['decoded_limit_bytes']
 assert 0<budget['draw_references']<=budget['draw_limit']
 terrain=qualify_resident_terrain(frame,w)
 checks.append({'role':role,'map':w['map_id'],'quantity':n,'key_ids':sorted(ids),'lease_token':token,'granted_index':granted,'arrived_index':arrived,'resource_budget':budget,'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest(),'independent_original_S32_residency':terrain})
report={'status':'verified_hotel_room_entry','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'disconnect_leases_empty':True,'gui_verified':False,'manual_exit_verified':False,'shared_keys_and_expiry_verified':False}
report['fixture_reference']=fixture_proof
(p.parent/'rental-hotel-entry-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'checks':len(checks),'maps':sorted({v['map'] for v in checks})}))
