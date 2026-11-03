"""Observe real one-hour merchant leases without modifying clocks or authority."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-item-expiry' and r['source_matches_snapshot']
assert r['server']['authority_mutations']==0 and r['server']['memberships_empty']
start=json.loads((p.parent/'natural-expiry-start.json').read_text(encoding='utf-8-sig'))
assert not start['clock_or_authority_injection'] and start['expected_maximum_seconds']==3600
assert hashlib.sha256(Path(start['immutable_exe']).read_bytes()).hexdigest().upper()==r['exe_sha256'].upper()==start['exe_sha256'].upper()
fixture=Path(__file__).with_name('native-rental-merchant-current.json')
if not fixture.exists():fixture=Path(__file__).resolve().parents[1]/'LiTo381-Modern/tests/fixtures/native-rental-merchant-current.json'
catalog=json.loads(fixture.read_text(encoding='utf-8-sig'));assert catalog['jar_sha256'].upper()==r['jar_sha256'].upper()
shops={s['npc']:s for s in catalog['shops']}
trace=[json.loads(line) for line in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[]
for role,c in r['clients'].items():
 q=c['item_expiry'];own=1 if role=='a' else 3
 assert c['worker_quiescent'] and c['shutdown']=='closed' and q['expected_seconds']==3600 and not q['clock_or_authority_injection']
 assert len(q['grants'])==len(q['removed'])==4 and {g['npc'] for g in q['grants']}==set(shops)
 states=[s for s in trace if s['state']['object_id']==own and s['state']['rental_town']]
 for g in q['grants']:
  item=shops[g['npc']]['items'][g['row_index']];assert item['seconds']==3600 and item['gems']==g['price']
  row=g['listing']['listing']['Buy']['rows'][g['row_index']]
  assert [row['name'],row['price'],row['icon']]==[item['name'],item['gems'],item['icon']]
  present=lambda s:any(v['id']==g['item'] for v in s['state']['rental_inventory'])
  granted=next(i for i,s in enumerate(states) if present(s))
  authority=next(v for v in states[granted]['state']['rental_inventory'] if v['id']==g['item'])
  assert authority['catalog_id']==item['catalog_id'] and authority['count']==1 and authority['expires_at']>0 and not authority['equipped']
  expired=next(i for i in range(granted+1,len(states)) if not present(states[i]))
  elapsed=states[expired]['ms']-states[granted]['ms'];assert 3598000<=elapsed<=3720000
  removed=next(v for v in q['removed'] if v['item']==g['item'])
  assert 3598000<=removed['observed_ms']-g['requested_ms']<=3720000
  assert str(g['item']) not in removed['inventory']['items']
  checks.append({'role':role,'npc':g['npc'],'item':g['item'],'catalog_id':item['catalog_id'],'formal_elapsed_ms':elapsed,'client_elapsed_ms':removed['observed_ms']-g['requested_ms']})
 gem='1811939330';before=q['before']['inventory']['items'];after=q['expired']['inventory']['items']
 assert set(before)==set(after) and {k:v for k,v in before.items() if k!=gem}=={k:v for k,v in after.items() if k!=gem}
 expected=before[gem]['count']-sum(g['price'] for g in q['grants']);assert after[gem]['count']==expected
 assert q['expired']['world']['map_id']==500 and q['expired']['world']['generation']==q['before']['world']['generation']
 assert states[-1]['state']['rental_gems']==expected
 image=Path(q['render']['png']);assert image.is_file()
assert len(checks)==8
out={'status':'verified_eight_natural_one_hour_merchant_expiries','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'checks':checks,'clock_or_authority_injection':False,'equipped_expiry_verified':False,'gui_verified':False}
(p.parent/'rental-item-expiry-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':out['status'],'checks':len(checks)}))
