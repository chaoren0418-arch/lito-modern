from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r481-multisector-batch'
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
m=read(L/'r481-resource-input.json');probe=read(O/'reference/reference.json');old=read(L/'r480-map-binding.json');assert probe['passed']==6 and probe['failed']==1
qualified=[r for r in probe['results'] if r.get('budget')];assert [r['map_id'] for r in qualified]==[551,504,526,525,531,529]
new=[]
for r in qualified:
 b=next(b for b in m['batch'] if b['map_id']==r['map_id']);new.append({k:b[k] for k in ['mode','map_id','namespace','map_root','archive_root','sectors','archives']}|dict(canonical_sha256=r['canonical_sha256']))
binding=old|dict(state='R481_CATALOG40_THIRTEEN_PREVIEWS_SIX_NEW_COMPLETE_MULTISECTOR_MAPS',bindings=old['bindings']+new,unqualified_count=27,remaining_unqualified_maps=27,new_complete_multisector_ids=[r['map_id'] for r in qualified],multisector_probe=pin(O/'reference/reference.json'),camera_adaptation='Existing R480 single immutable complete-map lexicographic sector anchor, signed projected bounds, 800x600 Modern inspection camera; no new runtime/renderer code per map.',multi_sector_scope='Six new complete mode1 maps551504526525531529 /34sectors. Source506 sectors parse but required Tile block type22 unsupported; unqualified.534 sixteen-sector city deferred.',source_rejected_new_maps=[dict(map_id=506,error=next(r['error'] for r in probe['results'] if r['map_id']==506))])
dest=L/'r481-map-binding.json';tmp=dest.with_suffix('.tmp');tmp.write_text(json.dumps(binding,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(dest)
print('R481 six new complete maps:13catalog previews27unqualified;506 type22 retained')
