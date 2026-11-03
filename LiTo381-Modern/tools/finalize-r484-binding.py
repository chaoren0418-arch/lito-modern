from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r484-remaining-batch'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(p)
m=read(L/'r484-resource-input.json');probe=read(O/'reference/reference.json');row=read(O/'row-reference/reference.json');old=read(L/'r483b-map-binding.json')
assert (probe['passed'],probe['measured_not_qualified'],probe['failed'])==(12,1,0)
measure=next(r for r in probe['results']if r['map_id']==527);assert measure['fixed_adapter_result']=='Native draw budget exceeded'and not measure['refusal_members']
assert row['passed']==1 and row['failed']==0
results=[next(r for r in (row['results']if b['map_id']==527 else probe['results'])if r['map_id']==b['map_id'])for b in m['batch']]
assert all(r.get('budget')and len(r['images'])==5 for r in results)
combined=dict(state='R484_TWELVE_REGULAR_FIXED_R483_AND_ONE_ROW_FULL_VECTOR_REFERENCE_NEW_SCOPE_ONLY',results=results,passed=13,measured_not_qualified=0,failed=0,regular_probe=pin(O/'reference/reference.json'),map527_reference=pin(O/'row-reference/reference.json'),GUI=0,LOADED=False)
save(O/'reference-combined.json',combined)
new=[{k:b[k]for k in ['mode','map_id','namespace','map_root','archive_root','sectors','archives']}|dict(canonical_sha256=r['canonical_sha256'])for b,r in zip(m['batch'],results)]
assert sum(len(b['sectors'])for b in new)==115
binding=old|dict(state='R484_THIRTY_SEVEN_WHOLE_MAP_PLANS_PENDING_NEW_BATCH_CONSUMER',bindings=old['bindings']+new,unqualified_count=3,remaining_unqualified_maps=3,new_complete_multisector_ids=[b['map_id']for b in new],multisector_probe=pin(O/'reference-combined.json'),camera_adaptation='Unchanged signed whole bounding camera and canonical immutable anchor; Modern inspection policy. Original streaming insertion unobserved.',multi_sector_scope='Thirteen new complete exact maps115sectors; twelve use unchanged regular API; map52718sectors362852drawrefs uses generic exact-catalog whole-plan row API. No decoder or budget expansion.',measured_unqualified_city_ids=[],row_profile_policy='Modern flow chooses bounded rows for complete plans with at least16sectors;534/535 old profiles preserved,527 new scale qualified; regular below16 remains originalAPI. This is a Modern allocation policy, not a native mode rule.')
save(L/'r484-map-binding.json',binding)
print('R48437pending CPU preview bindings;115newsectors;12regular+527rows;3known format gaps remain')
