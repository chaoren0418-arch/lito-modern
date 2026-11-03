"""Compare complete staged/reselected recovery item snapshots and exact S7C metadata."""
from pathlib import Path
import argparse,collections,hashlib,json,struct
ROOT=Path(__file__).resolve().parents[1]
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def decode(b):
 assert b[0]==0x7c and len(b)<=1024*1024;at=2;items={}
 for _ in range(b[1]):
  assert at+14<=len(b);oid,use,charge,icon,blessing,count,identified=struct.unpack_from('<IBBHBIB',b,at);at+=14
  assert oid not in items and count>0 and blessing<=3 and identified in [0,1];end=b.index(0,at);name=b[at:end].decode('cp950');at=end+1;assert at<len(b);size=b[at];at+=1;assert at+size<=len(b);detail=b[at:at+size];at+=size
  items[oid]=dict(id=oid,use_type=use,charge=charge,icon=icon,blessing=blessing,count=count,identified=identified,name=name,detail_hex=detail.hex())
 assert at==len(b);return items
def by_id(items):
 out={x['id']:x for x in items};assert len(out)==len(items);return out
def without_ids(items):
 return collections.Counter(json.dumps({k:v for k,v in x.items() if k!='id'},sort_keys=True,ensure_ascii=False) for x in items)
a=argparse.ArgumentParser();a.add_argument('case',type=Path);a.add_argument('--expected-resource-profile',choices=['lito-legacy','tw13081901-v1'],required=True);a.add_argument('--output',type=Path);args=a.parse_args();case=args.case.resolve(strict=True);assert case.is_relative_to(ROOT/'驗證紀錄/current/native-teleport-gui');out=(args.output or case/'recovery-inventory-continuity-comparison.json').resolve();assert out.is_relative_to(ROOT/'驗證紀錄/current') and out.parent.is_dir() and not out.exists()
r=read(case/'receipt.json');scenario=r['scenario'];assert scenario in ['timeout','refused','wrong-service','cancel-retry','cancel-late','cancel-expiry'];assert r['status']=='verified_native_teleport_GUI_'+scenario+'_and_fresh_reentry'
mode=r.get('resource_profile','lito-legacy');assert mode==args.expected_resource_profile
if mode=='tw13081901-v1':
 launch=read(case/'server-launch-identity.json');assert r['extended_item_details'] is launch['extended_item_details'] is True and launch['resource_profile']==mode
 assert r['common_catalog_sha256']==launch['common_catalog_sha256']=='B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
 assert '-Dlito.resourceProfile=tw13081901-v1' in launch['command_line'] and '-Dlito.extendedItemDetails=true' in launch['command_line']
base=read(case/'teleport-recovery-gui-comparison.json');assert base['receipt_sha256']==sha(case/'receipt.json');assert r['inputs_unchanged'] and r['private_desktop'] and r['reference_is_current']
cap=next((Path(r['runtime'])/'captures').glob('*/frames.jsonl'));assert sha(cap)==base['capture_sha256'];frames=[json.loads(l) for l in cap.read_text(encoding='utf-8-sig').splitlines()];batches=[];selections=[];endpoints={}
for i,row in enumerate(frames):
 if row['direction']!='server_to_client':continue
 text=row['plain']
 if text.startswith('7C '):batches.append((i,row['note'],decode(bytes.fromhex(text))))
 elif bytes.fromhex(text)==bytes([0xc8,0]):selections.append(i)
 if text.startswith('F4 ') and 'rental-teleport' in row['note']:
  b=bytes.fromhex(text);assert b[1] not in endpoints;endpoints[b[1]]=i
assert len(selections)==4 and set(endpoints)=={1,2};checks=[]
for n,(role,slot,count) in enumerate([('a',1,14),('b',2,11)]):
 old=by_id(r[role+'-before']['rental_inventory']);continued=by_id(r[role+'-continued']['native_inventory']);reentered=by_id(r[role+'-reentered']['rental_inventory']);moved=by_id(r[role+'-after-move']['rental_inventory']);assert len(old)==len(reentered)==count and moved==reentered
 assert set(old).isdisjoint(reentered) and without_ids(old.values())==without_ids(reentered.values())
 assert continued==by_id(r[role+'-reentered']['native_inventory'])==by_id(r[role+'-after-move']['native_inventory'])
 assert all(set(x)=={'id','catalog_id','name','count','equipped','expires_at'} and x['expires_at']==0 for x in old.values())
 source=[x for x in batches if x[0]<endpoints[slot] and 'rental-town' in x[1] and set(old).issubset(x[2])];assert len(source)==1
 target=next(x for x in batches if x[0]>selections[n+2]);assert set(target[2])==set(continued)
 if scenario=='cancel-expiry':
  assert set(old).isdisjoint(continued) and set(old).isdisjoint(reentered)
  assert without_ids(old.values())==without_ids(reentered.values())
  assert all(set(old).isdisjoint(x[2]) for x in batches if x[0]>selections[n+2])
  new_rental=[x for x in batches if x[0]>target[0] and 'rental-town' in x[1] and set(reentered).issubset(x[2])];assert len(new_rental)==1
  assert without_ids(source[0][2][oid] for oid in old)==without_ids(new_rental[0][2][oid] for oid in reentered)
  wire_result={oid:new_rental[0][2][oid] for oid in reentered}
 else:
  assert old==continued
  assert {oid:source[0][2][oid] for oid in old}==target[2]
  new_rental=[x for x in batches if x[0]>target[0] and 'rental-town' in x[1] and set(reentered).issubset(x[2])];assert len(new_rental)==1
  assert without_ids(source[0][2][oid] for oid in old)==without_ids(new_rental[0][2][oid] for oid in reentered)
  wire_result={oid:new_rental[0][2][oid] for oid in reentered}
 if mode=='tw13081901-v1':assert all(x['detail_hex'] for x in wire_result.values())
 checks.append(dict(role=role,objects=count,continued_stage_all_six_fields_preserved=scenario!='cancel-expiry',fresh_rental_reentry_new_IDs_and_all_nonID_fields_match=True,reentry_native_inventory_still_matches_continued_snapshot=True,expired_old_IDs_never_regranted=scenario=='cancel-expiry',exact_wire_fields_and_extended_details_preserved=True,source_wire_row=source[0][0],fresh_directory_wire_row=target[0],result_wire_items=wire_result))
proof=dict(status='verified_recovery_complete_item_fields_and_wire_metadata',resource_profile=mode,scenario=scenario,main_sha256=r['exe_sha256'],jar_sha256=r['jar_sha256'],receipt_sha256=sha(case/'receipt.json'),capture_sha256=sha(cap),original_recovery_comparison_sha256=sha(case/'teleport-recovery-gui-comparison.json'),validator_sha256=sha(__file__),checks=checks,objects=25,actual_case_ClientSessions=r['server']['actual_client_sessions'],new_ClientSessions_executed=0,stage_expiry_is_distinct_from_active_rental_item_expiry=True,active_rental_item_expiry_continuity_verified=False,client_inventory_attribute_UI_verified=False,whole_goal_complete=False)
out.write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps({k:proof[k] for k in ['status','scenario','resource_profile','objects','new_ClientSessions_executed']}))
