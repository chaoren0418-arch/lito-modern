"""Supplement cross-endpoint proof with all item fields and exact extended S7C details."""
from pathlib import Path
import argparse,hashlib,json,struct
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
a=argparse.ArgumentParser();a.add_argument('case',type=Path);args=a.parse_args();case=args.case.resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-gui')
out=case/'TW-inventory-continuity-comparison.json';assert not out.exists()
r=read(case/'receipt.json');assert r['scenario']=='cross-endpoint' and r['status']=='verified_native_teleport_GUI_cross_endpoint_shared_battle'
assert r['resource_profile']=='tw13081901-v1' and r['extended_item_details'] and r['inputs_unchanged'] and r['reference_is_current']
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()]
batches=[];endpoints={}
def decode(b):
 assert b[0]==0x7c and len(b)<=1024*1024;at=2;items={}
 for _ in range(b[1]):
  assert at+14<=len(b);oid,use,charge,icon,blessing,count,identified=struct.unpack_from('<IBBHBIB',b,at);at+=14
  assert oid not in items and count>0 and blessing<=3 and identified in [0,1];end=b.index(0,at);name=b[at:end].decode('cp950');at=end+1;assert at<len(b);size=b[at];at+=1;assert at+size<=len(b);detail=b[at:at+size];at+=size
  items[oid]=dict(id=oid,use_type=use,charge=charge,icon=icon,blessing=blessing,count=count,identified=identified,name=name,detail_hex=detail.hex())
 assert at==len(b);return items
for i,x in enumerate(frames):
 if x['direction']!='server_to_client':continue
 if x['plain'].startswith('7C '):batches.append((i,x['note'],decode(bytes.fromhex(x['plain']))))
 elif x['plain'].startswith('F4 ') and 'rental-teleport' in x['note']:
  b=bytes.fromhex(x['plain']);assert b[1] not in endpoints;endpoints[b[1]]=i
assert set(endpoints)=={1,2};checks=[]
for role,slot,expected_count in [('a',1,14),('b',2,11)]:
 prior=r[role+'-before'];dest=r[role+'-destination'];battle=r['battle'][role];old={x['id']:x for x in prior['rental_inventory']};assert len(old)==expected_count
 assert all(set(x)=={'id','catalog_id','name','count','equipped','expires_at'} and x['expires_at']==0 for x in old.values())
 for label,state in [('destination',dest),('battle',battle)]:
  now={x['id']:x for x in state['native_inventory']};assert now==old,(role,label,'all-six-field authority snapshot differs')
 assert prior['character']==dest['character']==battle['character'] and prior['test_account']==dest['test_account']==battle['test_account']
 source=[x for x in batches if x[0]<endpoints[slot] and 'rental-town' in x[1] and set(old).issubset(x[2])];assert len(source)==1
 target=next(x for x in batches if x[0]>endpoints[slot]);assert set(target[2])==set(old)
 assert {oid:source[0][2][oid] for oid in old}==target[2],(role,'wire item/extended details changed')
 assert all(len(x['detail_hex'])>0 for x in target[2].values())
 checks.append(dict(role=role,character=dest['character'],items=expected_count,authority_fields=['id','catalog_id','name','count','equipped','expires_at'],source_wire_row=source[0][0],destination_wire_row=target[0],exact_wire_items_and_extended_details=target[2],source_destination_and_battle_items_identical=True,wire_detail_bytes_preserved=True))
proof=dict(status='verified_TW_cross_endpoint_all_six_authority_item_fields_and_exact_extended_wire_details',receipt_sha256=sha(case/'receipt.json'),capture_sha256=sha(cap),main_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],resource_profile='tw13081901-v1',extended_item_details=True,checks=checks,actual_ClientSessions=6,objects=25,active_expiring_rental_items_tested=False,client_inventory_attribute_UI_verified=False,continuous_HP_MP_across_lobby_claimed=False,whole_goal_complete=False,validator_sha256=sha(__file__))
out.write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:proof[k] for k in ['status','objects','active_expiring_rental_items_tested']}))
