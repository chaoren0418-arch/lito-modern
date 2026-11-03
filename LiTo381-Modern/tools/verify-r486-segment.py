from pathlib import Path
import json,hashlib,struct,copy
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r486-paged-city';cache={};checks=0
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):
 p=Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def verify(v,skip_sources=False):
 global checks
 if isinstance(v,list):
  for x in v:verify(x,skip_sources)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   p=Path(v['path']);k=str(p)
   if k not in cache:cache[k]=pin(p)
   assert(cache[k]['bytes'],cache[k]['sha256'])==(v['bytes'],v['sha256']),k;checks+=1
  for k,x in v.items():
   if k not in ['path','bytes','sha256']and not(skip_sources and k=='source_pins'):verify(x,skip_sources)
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(p)
docs=['r486-resource-input.json','r486-reference-candidate.json','r486-map-binding.json','r486-entry-source-contract.json','r486-player-candidate.json','r486-consumer-candidate.json','r486-consumer-exit.json','r486-batch-details.json','r486-source-scope.json','r486-tight-budget-candidate.json','r486-tight-budget-audit.json','r486-late-negative-source.json','r486-final-test-compile.json']
for n in docs:verify(read(L/n))
unit=read(L/'r486-tile-test-candidate.json');verify(unit,True);assert len(unit['source_pins'])==601
for s in unit['source_pins']:assert pin(Path(s['snapshot']))['sha256']==s['sha256']
assert unit['test_executions']==4 and not unit['full_test_suite_executed']and unit['PortalRecord_missing_field_compile_errors']==0
for r in unit['runs']:assert r['exit_code']==0 and '1 passed; 0 failed'in Path(r['stdout']['path']).read_text(encoding='utf-8-sig')
final=read(L/'r486-final-test-compile.json');assert len(final['source_pins'])==602 and final['executions']==0
for s in final['source_pins']:assert pin(Path(s['snapshot']))['sha256']==s['sha256']
c=read(L/'r486-consumer-candidate.json');assert len(c['source_pins'])==164
for s in c['source_pins']:assert pin(Path(s['snapshot']))['sha256']==s['sha256']
old=read(L/'r485-candidate.json');changed=[]
for s in old['source_pins']:
 assert pin(Path(s['snapshot']))['sha256']==s['sha256']
 if pin(Path(s['path']))['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert set(changed)=={'tile.rs','native_city_resources.rs','native_city_flow.rs'}
oldtile=next(s for s in old['source_pins']if Path(s['path']).name=='tile.rs');assert Path(oldtile['snapshot']).read_text(encoding='utf-8').split('#[cfg(test)]')[0]==Path(oldtile['path']).read_text(encoding='utf-8').split('#[cfg(test)]')[0]
oldflow=next(s for s in old['source_pins']if Path(s['path']).name=='native_city_flow.rs');assert Path(oldflow['snapshot']).read_text(encoding='utf-8').replace('CityAssets::load_whole(', 'CityAssets::load_whole_bounded(')==Path(oldflow['path']).read_text(encoding='utf-8')
binding=read(L/'r486-map-binding.json');assert len(binding['bindings'])==40 and binding['bindings'][:39]==read(L/'r485-map-binding.json')['bindings'];assert binding['catalog40']==read(L/'r485-map-binding.json')['catalog40']
plan=binding['bindings'][39];assert(plan['mode'],plan['map_id'],len(plan['sectors']))==(3,563,19)
inp=read(L/'r486-resource-input.json')['batch'][0];assert len(inp['original_sectors'])==19 and sorted(p.name for p in Path(inp['original_directory']).glob('*.s32'))==sorted(Path(x['pin']['path']).name for x in inp['original_sectors'])
for orig,owned in zip(inp['original_sectors'],inp['sectors']):assert(orig['pin']['bytes'],orig['pin']['sha256'])==(owned['pin']['bytes'],owned['pin']['sha256'])
refdoc=read(O/'reference/reference.json');verify(refdoc);assert(refdoc['passed'],refdoc['failed'])==(1,0);r=refdoc['results'][0];assert len(r['images'])==25 and len(r['sector_stats'])==19
assert r['canonical_sha256']=='ADBC67E6FDA50CBC2F9E41D316D5FB99E9816D02227AE823DDC2378100FA874E'and r['bounds']==[0,-3840,12288,3072]
assert r['budget']['whole_required_slots']==31444 and r['budget']['whole_required_decoded_bytes']==72446976 and r['budget']['draw_references']==286665
assert r['maximum_visible_decoded_bytes']==9469440 and r['cumulative_visible_decoded_bytes']==29592576>16777216 and r['reference_allocations_not_production_profile']
refc=read(L/'r486-reference-candidate.json');assert refc['fixed_production_library']==read(L/'r485-player-candidate.json')['production_library']and refc['executions']==1 and refc['exit_code']==0
d=read(L/'r486-batch-details.json');assert d['new_static_CPU_reference_pairs']==len(d['images'])==54 and d['worker_CPU_reference_pairs']==25 and d['worker_negative_cases']==36 and d['stale_results_rejected']==2
assert d['actual_worker_retired']and d['actual_active_cached_load_Back_cancel']and d['actual_cached_to_regular_to_new_cached_flow']and d['held_cached_frames_revoked']
assert(d['complete_modes'],d['GUI'],d['LOADED'])==(16,0,False)and d['historical_PASS_consumers_rerun']==0 and not d['full_test_suite_executed']
for pair in d['images']:
 actual=Image.open(pair['image']['path']);expected=Image.open(pair['reference']['path']);assert actual.mode==expected.mode=='RGBA'and actual.size==expected.size==(800,600);pixels=actual.tobytes();assert pixels==expected.tobytes();assert hashlib.sha256(pixels).hexdigest().upper()==pair['image']['rgba_sha256']==pair['reference']['rgba_sha256']
dc=d['direct_cache'];m=dc['metrics_after_full_initial_qualification'];assert(m['qualified_slots'],m['decoded_bytes'],m['decoded_slots'],m['encoded_resident_bytes'])==(31444,0,0,19558705)
assert dc['negative_cases']==9 and dc['old_cache_progress_does_not_retain_assets']and dc['fresh_generation_progress_distinct_empty_cache']
for x in dc['frames']:
 cm=x['cache'];assert cm['capacity_bytes']==16777216 and cm['peak_resident_decoded_bytes']<=16777216 and cm['validation_peak_decoded_bytes']==2304 and x['frame']['visible_fragments']<=262144
assert dc['after25']['evictions']==5611 and dc['after25']['peak_resident_decoded_bytes']==16775424
assert dc['after_revisit'][0]['misses']>dc['after25']['misses']and dc['after_revisit'][1]['misses']==dc['after_revisit'][0]['misses']
assert dc['asset_drop_freed_requested_heap_bytes']>=dc['after25']['resident_decoded_bytes']+19558705+1572204
assert dc['actual_decode_cancel']['slots_decoded_at_request']>=10 and dc['actual_decode_cancel']['partial_cache']['completed_frames']==0 and dc['actual_decode_cancel']['recovered_metrics']['evictions']>0
assert dc['initial_qualification_cancel']['validated_slots_at_request']>=10000 and 'cancel'in dc['initial_qualification_cancel']['error']and not dc['hard_OS_deadline_promised']
assert d['late_required_invalid_tile']['validated_prefix_slots']==31443 and d['late_required_invalid_tile']['no_initial_partial_canvas']and 'type 255'in d['late_required_invalid_tile']['actual_worker_status']and d['missing_last_source']['no_fallback_frame']
neg=read(L/'r486-late-negative-source.json');original=Path(neg['original_archive_pin']['path']).read_bytes();bad=Path(neg['owned_invalid_archive']['path']).read_bytes();pos=neg['pak_byte_offset'];assert original[:pos]==bad[:pos]and original[pos+1:]==bad[pos+1:]and(original[pos],bad[pos])==(3,255)
tight=read(L/'r486-tight-budget-audit.json');assert tight['error']=='city cache visible pixel budget'and tight['validated_slots']==31444 and tight['no_canvas_returned']and tight['residence_misses_evictions_completed_frames_unchanged']
normalized=copy.deepcopy(d);normalized['direct_cache']['tight8MiB_whole_visible_view_refusal']['error']=tight['error'];normalized['metadata_correction']=dict(raw_original=pin(L/'r486-batch-details.json'),focused_audit=pin(L/'r486-tight-budget-audit.json'),reason='Original consumer reused variable error for later qualification cancellation. The tight visible-budget assertion passed; its JSON field was shadowed. Raw source/result/EXE retained; one focused new negative audit records actual error. No finite PASS batch rerun.');save(L/'r486-batch-details-normalized.json',normalized)
player=read(L/'r486-player-candidate.json');assert player['production_library']==c['production_library']and player['only_executed_candidate_flag']=='--version'and not player['native_GUI_executed'];exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
before=read(L/'r485-player-candidate.json');assert Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('R486','R485').replace('r486','r485')==Path(before['candidate_main']['path']).read_text(encoding='utf-8')
assert pin(L/'r485-native-format-two-map-handoff.json')['sha256']=='DD511B18C0184B08B7B0381DC6EE3C4D51A499ADBA85C265769C65D34C2FBC1E'
scope=read(L/'r486-source-scope.json');assert(scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(602,467,12,123)
assert read(L/'r486-consumer-exit.json')['exit_code']==0 and read(L/'r486-tight-budget-candidate.json')['exit_code']==0
out=dict(state='R486_INDEPENDENT_SOURCE_HASH_PIXEL_BUDGET_CACHE_WORKER_TEST_COMPILE_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=164,final_test_compile_source_pairs=602,source_files=602,changed_from_fixed_R485=changed,new_cache_module=True,complete_source_sectors=19,qualified_slots=31444,CPU_reference_pairs=54,worker_negative_cases=36,stale_results_rejected=2,peak_decoded_resident_bytes=16775424,cache_cap_bytes=16777216,decoded_limit=67108864,reference_uses_fixed_R485_library=True,finite_consumer_executions=1,focused_budget_metadata_audit_executions=1,related_unit_test_executions=4,full_test_suite_executed=False,raw_metadata_retained=True,normalized_details=pin(L/'r486-batch-details-normalized.json'),old39_plans_preserved=True,qualified_static_CPU_plans=40,complete_modes=16,GUI=0,LOADED=False)
save(L/'r486-independent-contract.json',out);save(L/'r486-candidate.json',c|dict(validation='PASS one finite complete563 bounded-cache/worker consumer plus focused error metadata audit',independent_contract=pin(L/'r486-independent-contract.json')))
print('R486 independent PASS:',checks,'pins',len(cache),'unique files;164consumer/602compile sourcepairs;54RGBA comparisons;40CPUplans/GUI0')
