"""Run after the immutable one-hour test finishes; never promotes a running lease."""
from pathlib import Path
import json,hashlib,sys
p=Path(sys.argv[1]).resolve();read=lambda f:json.loads(f.read_text(encoding='utf-8-sig'));sha=lambda f:hashlib.sha256(f.read_bytes()).hexdigest().upper()
r=read(p/'receipt.json');runtime=Path(r['runtime']);assert r['status']=='verified_two_real_equipped_weapon_expiries'
assert r['test_exit_code']==r['server_exit_code']==0 and r['expected_seconds']==3600 and not r['clock_or_authority_injection']
assert sha(Path(r['immutable_test_binary']))==r['test_binary_sha256']
s=r['server'];assert s['actual_client_sessions']==4 and s['memberships_empty'] and s['authority_mutations']==0 and not s['sql_access'] and not s['state_persistent']
trace=[json.loads(x) for x in (p/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
catalog=read(Path(__file__).resolve().parents[1]/'LiTo381-Modern/tests/fixtures/native-rental-merchant-current.json')
assert catalog['jar_sha256'].upper()==r['jar_sha256'];checks=[]
for role,account in [('a','PeerFixture'),('b','PeerFixture2')]:
 prep=read(p/f'{role}-weapon-preparation.json');end=read(p/f'{role}-weapon-result.json');item=prep['item'];original=prep['original_weapon'];gem='1811939330'
 row=next(shop for shop in catalog['shops'] if shop['npc']==prep['npc'])['items'][prep['row_index']]
 assert row['catalog_id']==prep['catalog_id']==19002 and row['seconds']==3600 and row['use_type']==1 and row['gems']==prep['price']
 states=[x for x in trace if x['state']['test_account']==account and x['state']['rental_town']]
 find=lambda x:next((i for i in x['state']['rental_inventory'] if i['id']==item),None)
 granted=next(i for i,x in enumerate(states) if find(x))
 equipped=next(i for i in range(granted,len(states)) if find(states[i]) and find(states[i])['equipped'])
 expired=next(i for i in range(equipped+1,len(states)) if find(states[i]) is None)
 assert all(find(states[i]) and find(states[i])['equipped'] for i in range(equipped,expired))
 assert find(states[equipped])['catalog_id']==19002 and find(states[equipped])['count']==1
 formal_ms=states[expired]['ms']-states[granted]['ms'];client_ms=end['observed_ms']-end['requested_ms']
 assert 3598000<=formal_ms<=3720000 and 3598000<=client_ms<=3720000
 assert any(i['id']==original and i['equipped'] for i in states[-1]['state']['rental_inventory'])
 for base,final in [('baseline','expired'),('before','restored')]:
  before=prep[base]['inventory']['items'];after=end[final]['inventory']['items'];assert set(before)==set(after)
  assert {k:v for k,v in before.items() if k!=gem}=={k:v for k,v in after.items() if k!=gem}
  assert after[gem]['count']==before[gem]['count']-prep['price']
  w=prep[base]['world'];f=end[final]['world'];assert w['generation']==f['generation']
  own=str(w['status']['object_id']);assert w['actors'][own]['walk_action']==f['actors'][own]['walk_action']
 assert end['stale_choices_rejected'] and not end['clock_or_authority_injection'] and str(item) not in end['expired']['inventory']['items']
 assert prep['unequip']['no_prediction'] and prep['equip']['duplicate_pending_rejected'] and end['restore']['no_prediction']
 checks.append(dict(role=role,item=item,catalog_id=19002,formal_elapsed_ms=formal_ms,client_elapsed_ms=client_ms,preparation_sha256=sha(p/f'{role}-weapon-preparation.json'),result_sha256=sha(p/f'{role}-weapon-result.json'),restored_png_sha256=sha(p/f'{role}-weapon-restored.png')))
out=dict(status='verified_two_real_one_hour_equipped_weapon_expiries',receipt_sha256=sha(p/'receipt.json'),test_binary_sha256=r['test_binary_sha256'],jar_sha256=r['jar_sha256'],checks=checks,clock_or_authority_injection=False,gui_verified=False,merchant_extra_bonus_verified=False,all_weapon_catalogue_expiry_verified=False,full_product_verified=False)
(p/'weapon-expiry-comparison.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'status':out['status'],'formal_elapsed_ms':[x['formal_elapsed_ms'] for x in checks]}))
