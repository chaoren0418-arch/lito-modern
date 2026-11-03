from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
baseline=read(P/'logs/relocation-baseline-20261003/source-manifest.json');changed=[];names=set()
allowed={'battle_equipment_controller_tests.rs','map_rotation_interop_tests.rs','world_integration_tests.rs','canonical_supplies_sql_tests.rs','canonical_map_rotation_tests.rs','battle_teleport.rs','battle_scene.rs','battle_runtime.rs','tile.rs','mod.rs','map.rs','world_map.rs'}
for b in baseline['entries']:
 path=b['path'].replace('\\','/');names.add(path);current=pin(P/path)
 if current['sha256']!=b['sha256']:assert Path(path).name in allowed;changed.append(dict(path=path,baseline_sha256=b['sha256'],current_sha256=current['sha256']))
new=[dict(path=str(p.relative_to(P)).replace('\\','/'),sha256=pin(p)['sha256'])for p in sorted((P/'src').rglob('*'))if p.is_file()and str(p.relative_to(P)).replace('\\','/')not in names]
old=read(L/'r484-candidate.json');prior_changes=[]
for s in old['source_pins']:
 assert pin(Path(s['snapshot']))['sha256']==s['sha256']
 if pin(Path(s['path']))['sha256']!=s['sha256']:prior_changes.append(Path(s['path']).name)
assert set(prior_changes)=={'tile.rs','map.rs','world_map.rs','native_resources.rs','native_city_resources.rs'}
scope=read(L/'source-change-scope.json')
for p in scope['frozen_binary_pins']:assert pin(Path(p['path']))['sha256']==p['sha256']
assert len(names)==479 and len(changed)==12 and len(new)==122
out=dict(state='R485_SOURCE_SCOPE_AND_ALL_R484_SNAPSHOTS_PRESERVED',source_files=601,baseline_source_files=479,unchanged_source_files=467,changed_source_files=changed,new_source_files=new,changed_from_fixed_R484=prior_changes,frozen_binary_pins=scope['frozen_binary_pins'],allowed_new_baseline_changes=['map.rs','world_map.rs'],authorization='T306/T308 R485 smallest Modern format correction; explicit Native parser preserves Shared entry.',scope='Only5 current production sources changed:observed22/23 enum/dispatch, Native-only optional portal suffix and Native WorldMap entry, two Native loaders select that entry. Shared entry/default wire rules remain; no budget/flow/runtime/window changes.',player_rebuilt_or_replaced=False,original_main_sha256=pin(P/'src/main.rs')['sha256'])
save(L/'r485-source-scope.json',out)
save(L/'source-change-scope.json',out)
print('R485 source601=479baseline/467unchanged/12allowedchanged/122new;5R484current changes only;old snapshots/binaries preserved')
