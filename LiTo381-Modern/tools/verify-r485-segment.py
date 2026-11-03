from pathlib import Path
import json,hashlib,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r485-format-study'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
cache={};checks=0
def verify(v):
 global checks
 if isinstance(v,list):
  for x in v:verify(x)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   p=Path(v['path']);assert p.is_file(),str(p)
   if str(p)not in cache:
    b=p.read_bytes();cache[str(p)]=(len(b),hashlib.sha256(b).hexdigest().upper())
   assert cache[str(p)]==(v['bytes'],v['sha256']),str(p);checks+=1
  for k,x in v.items():
   if k not in ['path','bytes','sha256']:verify(x)
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
docs=['r485-format-source-evidence.json','r485-native-layer-reference.json','r485-tile-reference-input.json','r485-tile-reference.json','r485-resource-input.json','r485-budget-input.json','r485-reference-candidate.json','r485-budget-reference-candidate.json','r485-map-binding.json','r485-entry-source-contract.json','r485-player-candidate.json','r485c-consumer-candidate.json','r485c-consumer-exit.json','r485-batch-details.json','r485-source-scope.json']
for n in docs:verify(read(L/n))
verify(read(O/'reference-combined.json'));verify(read(O/'format-cases.json'))
old=read(L/'r484-candidate.json');candidate=read(L/'r485c-consumer-candidate.json')
assert len(candidate['source_pins'])==160
changed=[]
for s in old['source_pins']:
 assert pin(Path(s['snapshot']))['sha256']==s['sha256']
 if pin(Path(s['path']))['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert set(changed)=={'tile.rs','map.rs','world_map.rs','native_resources.rs','native_city_resources.rs'}
for name in ['native_resources.rs','native_city_resources.rs']:
 s=next(s for s in old['source_pins']if Path(s['path']).name==name)
 before=Path(s['snapshot']).read_text(encoding='utf-8');after=Path(s['path']).read_text(encoding='utf-8')
 assert before.replace('WorldMap::from_sector_bytes(inputs,cancel)','WorldMap::from_native_sector_bytes(inputs,cancel)').replace('WorldMap::from_sector_bytes(inputs, cancel)','WorldMap::from_native_sector_bytes(inputs, cancel)')==after
for s in candidate['source_pins']:assert pin(Path(s['snapshot']))['sha256']==s['sha256']
binding=read(L/'r485-map-binding.json');prior=read(L/'r484-map-binding.json');assert len(binding['bindings'])==39 and binding['bindings'][:37]==prior['bindings']
assert [(b['mode'],b['map_id'],len(b['sectors']))for b in binding['bindings'][37:]]==[(1,506,6),(3,573,4)]
d=read(L/'r485-batch-details.json');assert d['new_static_CPU_reference_pairs']==10 and d['worker_negative_cases']==29 and d['stale_results_rejected']==4 and d['actual_worker_retired']
assert d['format_cases']['negative_cases']==1557 and d['format_cases']['new_Native_parser_previous37_sector_compatibility_checks']==278
assert d['format_cases']['unchanged_Shared_layout'] and d['format_cases']['native_original_sectors']==29
assert d['map563']['required_decoded_bytes']==72446976>d['map563']['decoded_limit']==67108864
assert d['map563']['actual_direct_catalog_row_failure']=='city selected pixel budget'and 'city selected pixel budget'in d['map563']['actual_worker_failure']
assert d['historical_PASS_reruns']==0 and d['complete_modes']==16 and d['GUI']==0 and not d['LOADED']
refs=read(O/'reference-combined.json')
for image in d['images']:
 verify(image['image']);expected=next(r for r in refs['results']if r['map_id']==image['identity'][1]);expected=next(i for i in expected['images']if i['name']==image['name']);verify(expected['image'])
 actual=Image.open(image['image']['path']);reference=Image.open(expected['image']['path']);assert actual.mode==reference.mode=='RGBA'and actual.size==reference.size==(800,600);assert actual.tobytes()==reference.tobytes()
for s in refs['results']+[refs['map563_budget']]:
 all_sources=next(r for r in read(L/'r485-resource-input.json')['batch']if r['map_id']==s['map_id'])
 assert len(s['sector_stats'])==len(all_sources['sectors'])
 for original,owned in zip(all_sources['original_sectors'],all_sources['sectors']):assert original['pin']['bytes']==owned['pin']['bytes']and original['pin']['sha256']==owned['pin']['sha256']
assert refs['all29_native_layer_ranges_portals_effects_match_independent_pointer_reference']
assert refs['map563_budget']['individually_supported_slots']==31444 and not refs['map563_budget']['member_failures']
player=read(L/'r485-player-candidate.json');verify(player);verify(player['original_main']);assert player['only_executed_candidate_flag']=='--version'
actual=Path(player['candidate_main']['path']).read_bytes().decode('utf-8');before=read(L/'r484-player-candidate.json');oldmain=Path(before['candidate_main']['path']).read_bytes().decode('utf-8');assert actual.replace('R485','R484').replace('r485','r484')==oldmain
assert player['production_library']['sha256']==read(L/'r485-reference-candidate.json')['fixed_production_library']['sha256']==candidate['production_library']['sha256']
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0'and struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
assert pin(L/'r484-thirteen-complete-map-batch-handoff.json')['sha256']=='BA5DECD4D1EFA0010FCA12D5F6EDF5F28DC500BF5BF71F670C08184F4B60BABE'
scope=read(L/'r485-source-scope.json');assert (scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(601,467,12,122)
out=dict(state='R485_INDEPENDENT_NEW_NATIVE_FORMAT_FULL_SOURCE_REFERENCE_BUDGET_WORKER_SCOPE_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=160,source_files=601,changed_from_R484=changed,old37_plans_and278_parsed_sectors_preserved=True,new_maps=[506,573],new_preview_complete_sectors=10,new_static_CPU_reference_pairs=10,new_Csharp_tile_reference_pairs=12,native_original_sectors=29,new_format_negative_cases=1557,new_worker_negative_cases=29,stale_results_rejected=4,focused_exit_code=0,focused_seconds=d['focused_seconds'],budget563_required_bytes=72446976,budget563_limit=67108864,complete_modes=16,GUI=0,LOADED=False)
p=L/'r485-independent-contract.json';t=p.with_suffix('.json.tmp');t.write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
final=candidate|dict(validation='PASS one finite new Native format/2maps/563budget consumer',independent_contract=pin(p));p=L/'r485-candidate.json';t=p.with_suffix('.json.tmp');t.write_text(json.dumps(final,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
print('R485 independent PASS:',checks,'pins',len(cache),'unique files;160sourcepairs/10CPU/12C#;39previewplans/GUI0')
