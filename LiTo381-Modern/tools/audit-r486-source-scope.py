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
old=read(L/'r485-candidate.json');prior_changes=[]
for s in old['source_pins']:
 assert pin(Path(s['snapshot']))['sha256']==s['sha256']
 if pin(Path(s['path']))['sha256']!=s['sha256']:prior_changes.append(Path(s['path']).name)
assert set(prior_changes)=={'tile.rs','native_city_resources.rs','native_city_flow.rs'}
scope=read(L/'source-change-scope.json')
for p in scope['frozen_binary_pins']:assert pin(Path(p['path']))['sha256']==p['sha256']
assert len(names)==479 and len(changed)==12 and len(new)==123
out=dict(state='R486_SOURCE_SCOPE_ALL_R485_SNAPSHOTS_PRESERVED',source_files=602,baseline_source_files=479,unchanged_source_files=467,changed_source_files=changed,new_source_files=new,changed_from_fixed_R485=prior_changes,frozen_binary_pins=scope['frozen_binary_pins'],allowed_new_baseline_changes=[],authorization='T315/T323 Modern-only tile test repair and complete563 bounded residence. T306/T308 baseline map/world authorization retained.',scope='R485 current delta:tile.rs tests only; native_city_resources.rs bounded storage opt-in/all-required prevalidation; native_city_flow.rs selects bounded entry; one new native_city_tile_cache.rs. Runtime/window/menu/native_resources/map/world/production tile decoder unchanged. No raised budgets.',player_rebuilt_or_replaced=False,original_main_sha256=pin(P/'src/main.rs')['sha256'])
save(L/'r486-source-scope.json',out)
save(L/'source-change-scope.json',out)
print('R486 source602=479baseline/467unchanged/12allowedchanged/123new;3R485 current deltas plus new cache;old snapshots/binaries preserved')
