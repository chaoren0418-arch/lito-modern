"""Verify actual one-hour equipped-glove leases against independent formal observations."""
import hashlib,json,sys
from pathlib import Path
read=lambda p:json.loads(p.read_text(encoding='utf-8-sig'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest().upper()
p=Path(sys.argv[1]).resolve();r=read(p);assert r['action_profile']=='rental-equipped-expiry'
assert r['source_matches_snapshot'] and r['server']['authority_mutations']==0 and r['server']['memberships_empty']
start=read(p.parent/'natural-expiry-start.json')
assert not start['clock_or_authority_injection'] and start['expected_maximum_seconds']==3600
assert sha(Path(start['immutable_exe']))==r['exe_sha256'].upper()==start['exe_sha256'].upper()
fixture=Path(__file__).with_name('native-rental-merchant-current.json')
if not fixture.exists():fixture=Path(__file__).resolve().parents[1]/'LiTo381-Modern/tests/fixtures/native-rental-merchant-current.json'
catalog=read(fixture);assert catalog['jar_sha256'].upper()==r['jar_sha256'].upper()
trace=[json.loads(line) for line in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[]
for role,c in r['clients'].items():
 q=c['equipped_expiry'];own=1 if role=='a' else 3;item=q['item'];gem='1811939330'
 assert c['worker_quiescent'] and c['shutdown']=='closed' and q['expected_seconds']==3600
 assert not q['clock_or_authority_injection'] and q['stale_item_and_listing_rejected']
 shop=next(s for s in catalog['shops'] if s['npc']==q['npc']);entry=shop['items'][q['row_index']]
 assert entry['use_type']==20 and entry['seconds']==3600 and entry['gems']==q['price']
 row=q['listing']['listing']['Buy']['rows'][q['row_index']]
 assert [row['name'],row['price'],row['icon']]==[entry['name'],entry['gems'],entry['icon']]
 states=[v for v in trace if v['state']['object_id']==own and v['state']['rental_town']]
 find=lambda s:next((v for v in s['state']['rental_inventory'] if v['id']==item),None)
 granted=next(i for i,s in enumerate(states) if find(s))
 equip=next(i for i in range(granted,len(states)) if find(states[i]) and find(states[i])['equipped'])
 expired=next(i for i in range(equip+1,len(states)) if find(states[i]) is None)
 assert all(find(states[i]) and find(states[i])['equipped'] for i in range(equip,expired))
 authority=find(states[equip]);assert authority['catalog_id']==entry['catalog_id'] and authority['count']==1 and authority['expires_at']>0
 elapsed=states[expired]['ms']-states[granted]['ms'];assert 3598000<=elapsed<=3720000
 assert 3598000<=q['observed_ms']-q['requested_ms']<=3720000
 for action in ['unequip','equip','restore']:assert q[action]['no_prediction'] and q[action]['duplicate_pending_rejected']
 assert str(item) in q['equipped']['inventory']['items'] and str(item) not in q['expired']['inventory']['items']
 original=q['original_glove'];assert any(v['id']==original and v['equipped'] for v in states[-1]['state']['rental_inventory'])
 for base,end in [('baseline','expired'),('before','restored')]:
  before=q[base]['inventory']['items'];after=q[end]['inventory']['items']
  assert set(before)==set(after) and {k:v for k,v in before.items() if k!=gem}=={k:v for k,v in after.items() if k!=gem}
  assert after[gem]['count']==before[gem]['count']-q['price']
  assert q[base]['world']['generation']==q[end]['world']['generation'] and q[end]['world']['map_id']==500
 ac=lambda key:q[key]['world']['status']['armor_class']
 assert ac('baseline')-ac('equipped')==10 and ac('expired')==ac('baseline') and ac('restored')==ac('before')
 for name in ['equipped_render','expired_render']:assert Path(q[name]['png']).is_file()
 checks.append({'role':role,'item':item,'catalog_id':entry['catalog_id'],'formal_elapsed_ms':elapsed,'client_elapsed_ms':q['observed_ms']-q['requested_ms'],'armor_class':[ac(k) for k in ['baseline','equipped','expired','restored']]})
assert len(checks)==2
out={'status':'verified_two_natural_equipped_glove_expiries','receipt_sha256':sha(p),'checks':checks,'clock_or_authority_injection':False,'weapon_expiry_verified':False,'merchant_extra_bonus_verified':False,'gui_verified':False}
(p.parent/'equipped-expiry-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'checks':len(checks)}))
