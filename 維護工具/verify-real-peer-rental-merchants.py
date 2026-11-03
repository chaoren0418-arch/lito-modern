"""Every received merchant row checked against the formal catalogue and passive inventory."""
import hashlib,json,sys
from pathlib import Path
p=Path(sys.argv[1]);r=json.loads(p.read_text(encoding='utf-8-sig'))
assert r['action_profile']=='rental-merchants' and r['source_matches_snapshot'] and r['server']['authority_mutations']==0 and r['server']['memberships_empty']
fixture=Path(__file__).resolve().parents[1]/'LiTo381-Modern/tests/fixtures/native-rental-merchant-current.json'
catalog=json.loads(fixture.read_text(encoding='utf-8-sig'));assert catalog['jar_sha256'].upper()==r['jar_sha256'].upper()
shops={s['npc']:s for s in catalog['shops']};assert len(shops)==4 and sum(len(s['items']) for s in shops.values())==78
trace=[json.loads(s)['state'] for s in (p.parent/'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
checks=[];all_ids=set()
for role,c in r['clients'].items():
 own=1 if role=='a' else 3;q=c['merchants'];states=[s for s in trace if s['object_id']==own and s['rental_town']];cursor=0
 assert c['worker_quiescent'] and c['shutdown']=='closed' and not q['server_authority_injection'] and not q['gui_verified']
 assert {s['npc'] for s in q['shops']}==set(shops)
 for shop in q['shops']:
  formal=shops[shop['npc']];received=shop['buy_list']['listing']['Buy']['rows'];base=shop['before']['items'];gem='1811939330';gems=base[gem]['count']
  assert shop['dialog']['html'].lower()==formal['html'].lower() and shop['refusal']==[9,'光之寶石不足。']
  assert shop['local_page'].lower()==('maya_trade' if formal['html'].lower()=='maya_start' else formal['html'].lower())
  assert len(received)==len(formal['items'])==len(shop['transactions'])
  for index,(row,item) in enumerate(zip(received,formal['items'])):
   assert [row['id'],row['name'],row['price'],row['icon']]==[index,item['name'],item['gems'],item['icon']]
  for step in shop['steps']:
   cursor=next(j for j in range(cursor,len(states)) if [states[j]['x'],states[j]['y']]==step['after'])
  rental_base={i['id']:i['count'] for i in states[cursor]['rental_inventory']}
  assert all(str(k) in base and base[str(k)]['count']==v for k,v in rental_base.items())
  transactions=[]
  for index,(t,item) in enumerate(zip(shop['transactions'],formal['items'])):
   assert t['row_index']==t['order_id']==index and t['price']==t['refund']==item['gems']
   rented=t['rented']['items'];returned=t['returned']['items'];key=str(t['item'])
   assert t['item'] not in all_ids;all_ids.add(t['item'])
   assert set(rented)-set(base)=={key} and returned==base and rented[key]['count']==1 and rented[key]['name'].endswith(item['name'])
   assert rented[gem]['count']==gems-t['price'] and {k:v for k,v in rented.items() if k not in [key,gem]}=={k:v for k,v in base.items() if k!=gem}
   expected=dict(rental_base);expected[t['item']]=1
   granted=next(j for j in range(cursor,len(states)) if states[j]['rental_gems']==gems-t['price'] and {v['id']:v['count'] for v in states[j]['rental_inventory']}==expected)
   authority=next(v for v in states[granted]['rental_inventory'] if v['id']==t['item'])
   assert authority['catalog_id']==item['catalog_id'] and authority['name']==item['name'] and authority['expires_at']>0
   released=next(j for j in range(granted+1,len(states)) if states[j]['rental_gems']==gems and {v['id']:v['count'] for v in states[j]['rental_inventory']}==rental_base)
   cursor=released;transactions.append({'row':index,'source_id':item['source_id'],'item':t['item'],'granted_index':granted,'returned_index':released})
  image=Path(shop['render']['png']);assert image.is_file()
  checks.append({'role':role,'npc':shop['npc'],'count':len(transactions),'transactions':transactions,'image_sha256':hashlib.sha256(image.read_bytes()).hexdigest()})
assert len(checks)==8 and len(all_ids)==156
report={'status':'verified_all_78_merchant_rows_on_two_clients','receipt_sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'catalog_sha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),'checks':checks,'rented_and_returned_items':156,'gui_verified':False,'capacity_weight_and_natural_expiry_verified':False}
(p.parent/'rental-merchants-comparison.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps({'status':report['status'],'items':len(all_ids),'checks':len(checks)}))
