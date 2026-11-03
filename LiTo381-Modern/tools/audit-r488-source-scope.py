from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
base=read(P/'logs/relocation-baseline-20261003/source-manifest.json');previous=read(L/'r487-source-scope.json');names=set();changed=[]
oldchanged={x['path']:x['current_sha256'] for x in previous['changed_source_files']}
for b in base['entries']:
 path=b['path'].replace('\\','/');names.add(path);current=pin(P/path)
 if current['sha256']!=b['sha256']:
  assert oldchanged[path]==current['sha256'];changed.append(dict(path=path,baseline_sha256=b['sha256'],current_sha256=current['sha256']))
assert len(changed)==12
old=read(L/'r487-production-source-snapshot.json');delta=[]
for s in old['source_pins']:
 assert pin(Path(s['snapshot']))['sha256']==s['sha256']
 if pin(Path(s['path']))['sha256']!=s['sha256']:delta.append(Path(s['path']).name)
assert delta==['native_city_runtime.rs'],delta
new=[dict(path=str(p.relative_to(P)).replace('\\','/'),sha256=pin(p)['sha256'])for p in sorted((P/'src').rglob('*'))if p.is_file()and str(p.relative_to(P)).replace('\\','/')not in names]
assert len(names)==479 and len(new)==124
for frozen in previous['frozen_binary_pins']:assert pin(Path(frozen['path']))['sha256']==frozen['sha256']
scope=dict(state='R488_SOURCE_SCOPE_R487_SNAPSHOTS_AND_ALL_PREVIOUS_BINDINGS_PRESERVED',source_files=603,baseline_source_files=479,unchanged_source_files=467,changed_source_files=changed,new_source_files=new,changed_from_fixed_R487=delta,added_from_fixed_R487=['native_mode_selection.rs'],frozen_binary_pins=previous['frozen_binary_pins'],allowed_new_baseline_changes=[],authorization='T332/T335 scoped offline Rust mode event/menu flow;Modern-only writes and own approved progress. No Java/server fixture.',scope='Only native_city_runtime.rs changed plus nested native_mode_selection.rs. Existing menu/window/arrow/cache/maps/decoder/resource/flow unchanged from R487;12 historical baseline changes retain exact R487 bytes.',player_rebuilt_or_replaced=False,original_main_sha256=pin(P/'src/main.rs')['sha256'])
save(L/'r488-source-scope.json',scope);save(L/'source-change-scope.json',scope)
out=P/'target/r488-fixed-event-consumer/all-source';out.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if p.is_file():
  dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
assert len(pins)==603
save(L/'r488-production-source-snapshot.json',dict(state='R488_ALL603_PRODUCTION_SOURCES_FROZEN_AFTER_LIBRARY_BUILD',source_pins=pins,production_library=read(L/'r488-player-candidate.json')['production_library'],full_test_suite_executed=False,library_test_compile_performed_this_revision=False,GUI=0,LOADED=False))
print('R488 source603=479baseline/467unchanged/12historical allowed/124new;only Runtime delta+new mode module;full snapshots frozen')
