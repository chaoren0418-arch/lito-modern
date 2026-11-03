from pathlib import Path
import json,hashlib,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r483-city-visible-rows'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
cache={};checked=0
def pin(p):
 p=Path(p);k=str(p.resolve()).lower()
 if k not in cache:
  h=hashlib.sha256()
  with p.open('rb') as f:
   for b in iter(lambda:f.read(4<<20),b''):h.update(b)
  cache[k]=dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
 return cache[k]
def check(v):
 global checked
 if isinstance(v,list):
  for x in v:check(x)
 elif isinstance(v,dict):
  if {'path','sha256'}<=v.keys():
   a=pin(v['path']);assert a['sha256']==v['sha256'].upper(),v['path']
   if 'bytes'in v:assert a['bytes']==v['bytes'],v['path']
   if 'snapshot'in v:assert pin(v['snapshot'])['sha256']==v['sha256'].upper()
   checked+=1
  for x in v.values():check(x)
details=read(L/'r483-city-details.json');host=read(L/'r483c-consumer-candidate.json');player=read(L/'r483b-player-candidate.json');binding=read(L/'r483b-map-binding.json');tested=read(L/'r483-map-binding.json');reference=read(O/'reference/reference.json');inputs=read(L/'r483-resource-input.json');exit_result=read(L/'r483c-consumer-exit.json');old=read(L/'r482-candidate.json')
for doc in [details,host,player,binding,tested,reference,inputs,exit_result,read(L/'r483-reference-candidate.json'),read(L/'r483b-entry-source-contract.json')]:check(doc)
assert exit_result['exit_code']==0 and exit_result['executions']==1
assert len(host['source_pins'])==155
assert host['production_library']==read(L/'r483-player-candidate.json')['production_library']
assert host['production_library']['sha256']==player['production_library']['sha256']
changed=[]
for s in old['source_pins']:
 assert pin(s['snapshot'])['sha256']==s['sha256']
 if pin(s['path'])['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert sorted(changed)==['mod.rs','native_resources.rs'],changed
old_resource=next(s for s in old['source_pins'] if Path(s['path']).name=='native_resources.rs')
text=(P/'src/modern_lito/native_resources.rs').read_text(encoding='utf-8')
addition='\n/// Separate full-city profile. Existing load_static and its total draw cap are unchanged.\n#[path = "native_city_resources.rs"]\npub mod city;\n'
assert text.endswith(addition) and text[:-len(addition)]==Path(old_resource['snapshot']).read_text(encoding='utf-8')
assert (P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8')==(P/'src/modern_lito/native_multisector_runtime.rs').read_text(encoding='utf-8').replace('native_multisector_flow::{','native_city_flow::{')
assert (P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8')==(P/'src/modern_lito/native_cross_mode_window.rs').read_text(encoding='utf-8').replace('native_multisector_runtime::{','native_city_runtime::{').replace('R482','R483').replace('0x482','0x483')
assert pin(L/'r482-nine-cross-mode-complete-map-handoff.json')['sha256']=='4096AB39B677BDEFDAEFFA78C058E39F13459CB0FABCAE0A29C96F3E74EF26B9'
assert pin(P/'target/r482-player-entry-candidate/LiToModern-R482.exe')['sha256']=='0FC532ADBF580B76DADFBFDD6749D75F80E73CA32FA9BA428D020821A51AA5B4'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
assert binding['bindings']==tested['bindings'] and binding['catalog40']==tested['catalog40']
assert binding['bindings'][:22]==read(L/'r482-map-binding.json')['bindings']
assert [(v['mode'],v['map_id'])for v in binding['bindings'][22:]]==[(1,534),(5,535)]
assert binding['remaining_unqualified_maps']==binding['unqualified_count']==16 and not binding['measured_unqualified_city_ids']
catalog=read(binding['catalog_source']['path']);assert binding['catalog40']==[r['original_table']for r in catalog['rows']]
assert len(binding['bindings'])==24 and len(binding['catalog40'])==40
assert reference['passed']==2 and reference['failed']==reference['measured_not_qualified']==0
for b in inputs['batch']:
 authority=next(r for r in catalog['rows']if r['logical_map']==b['map_id']);assert authority['mode']==b['mode']
 assert len(b['sectors'])==len(authority['native_sector_files'])==len(list(Path(b['original_directory']).glob('*.s32')))
 for own,original in zip(b['sectors'],authority['native_sector_files']):assert Path(own['pin']['path']).name==Path(original['path']).name and own['pin']['sha256']==original['sha256'] and own['pin']['bytes']==original['bytes']
 plan=next(v for v in binding['bindings']if v['map_id']==b['map_id']);assert plan['sectors']==b['sectors']
 rr=next(v for v in reference['results']if v['map_id']==b['map_id']);assert plan['canonical_sha256']==rr['canonical_sha256'];assert b['expected_bounds']==rr['bounds']
 a=b['sectors'][0];positions=[(1536*(s['x']-a['x']+s['y']-a['y']),768*(-s['x']+a['x']+s['y']-a['y']))for s in b['sectors']]
 assert rr['bounds']==[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)]
 assert all(s['unknown_suffix']==0 for s in rr['sector_stats'])
 for image in rr['images'][1:3]:
  assert image['visible_static_objects']>20
  for sx,sy in rr['seam']['source_pair']:assert f'{sx:04x}{sy:04x}'in image['visible_sectors']
assert details['new_identities']==[[1,534],[5,535]] and details['new_complete_sectors']==41
assert len(details['images'])==20 and details['negative_cases']==45 and details['stale_results_rejected']==4
assert len(details['failure_cases'])==2 and len(details['runtime_cases'])==2 and details['close_active_worker_retired']
assert len(details['load_phase_cancellations'])==8
for c in details['load_phase_cancellations']:assert c['observed_phase']==c['requested_phase'] and c['cancel_to_join_ms']<10000 and 'cancel'in c['error'].lower() and not c['hard_OS_deadline_promised']
for actual in details['images']:
 expected=next(i for r in reference['results']if (r['mode'],r['map_id'])==tuple(actual['identity'])for i in r['images']if i['name']==actual['name'])
 assert actual['camera']==expected['camera'] and actual['reference']==expected['image'] and actual['image']['rgba_sha256']==expected['image']['rgba_sha256']
 # The same PNG writer and exact pixels produce identical full file bytes.
 assert actual['image']['sha256']==expected['image']['sha256']
for c in details['map_cases']:
 id=c['identity'][1];rr=next(r for r in reference['results']if r['map_id']==id);m=c['metrics']
 assert m['source_references']==rr['budget']['draw_references'] and m['decoded_bytes']==rr['budget']['decoded_bytes']
 assert m['non_floor_references']==rr['non_floor_references'] and m['row_item_bytes']==12
 assert m['index_payload_capacity_bytes']==m['non_floor_references']*12<8<<20
 assert m['decoded_bytes']<64<<20 and m['encoded_source_bytes_validated']<32<<20
 assert c['assets_drop_freed_requested_heap_bytes']>m['decoded_bytes']+m['index_payload_capacity_bytes']
 assert c['load_incremental_peak_requested_heap_bytes']>=c['assets_drop_freed_requested_heap_bytes']
 assert c['whole_map_bounds']==rr['bounds'] and c['whole_map_canonical']==rr['canonical_sha256']
 assert all(f['metrics']['visible_fragments']<=262144 and f['metrics']['visited_index_rows']<=m['index_rows'] for f in c['frames'])
 assert any(f['metrics']['visited_index_rows']<m['index_rows'] and f['metrics']['visible_fragments']>0 for f in c['frames'])
 for s in binding['bindings'][22:]:
  if s['map_id']==id:
   last=s['sectors'][-1];assert not (O/f'missing-native-{id}'/str(id)/Path(last['pin']['path']).name).exists()
assert len(details['fixture_pins'])==44
assert details['retained_original_regular_load_static_total_cap']==details['per_frame_fragment_limit']==262144
assert details['new_index_payload_limit']==8<<20 and details['decoded_pixel_limit']==64<<20
assert details['remaining_unqualified']==16 and details['qualified_CPU_catalog_previews']==24
assert (details['complete_modes'],details['total_modes'],details['GUI'],details['LOADED'])==(16,40,0,False)
scope=read(L/'source-change-scope.json');assert scope['source_files']==601 and len(scope['new_source_files'])==122 and len(scope['changed_source_files'])==10
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0'and struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
root=Path(player['original_main']['path']).read_bytes();t=Path(player['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n'if'\r\n'in t else'\n'
for a in ['    CandidateEntry,'+nl+'    OfflineMap,'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl,'        "candidate_revision": "r483",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl]:assert t.count(a)==1;t=t.replace(a,'')
a=t.index('fn candidate_offline_entry(');z=t.index('fn version_document()',a);t=t[:a]+t[z:];assert t.encode('utf-8')==root
result=dict(state='R483_INDEPENDENT_SOURCE_SCOPE_WHOLE_CITY_MEMORY_ROW_BUDGET_REFERENCE_IDENTITY_CANCEL_FAILURE_CANDIDATE_PASS',source_pairs=155,unique_files=len(cache),checked_pin_instances=checked,new_identities=[[1,534],[5,535]],new_complete_sectors=41,CPU_reference_pairs=20,negative_cases=45,stale_results_rejected=4,focused_exit_code=0,focused_seconds=details['focused_seconds'],load_phase_cancellations=8,regular_load_static_body_unchanged=True,R482_changed_current_sources=changed,old22_map_plans_unchanged=True,tested_and_final_metadata_corrected_binding_plans_identical=True,source_files=601,new_sources=122,previously_accepted_previews=22,new2_review='PENDING',remaining_unqualified=16,complete_modes=16,GUI=0,LOADED=False)
f=L/'r483-independent-contract.json';tmp=f.with_suffix('.json.tmp');tmp.write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');tmp.replace(f)
print(json.dumps(result,ensure_ascii=False))
