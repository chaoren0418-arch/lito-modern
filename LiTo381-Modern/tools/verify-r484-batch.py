from pathlib import Path
import json,hashlib,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r484-remaining-batch'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8-sig'))
cache={};checked=0
def pin(p):
 p=Path(p);p=p if p.is_absolute()else P/p;k=str(p.resolve()).lower()
 if k not in cache:
  h=hashlib.sha256()
  with p.open('rb')as f:
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
m=read(L/'r484-resource-input.json');binding=read(L/'r484-map-binding.json');regular=read(O/'reference/reference.json');row=read(O/'row-reference/reference.json');reference=read(O/'reference-combined.json');details=read(L/'r484-batch-details.json');host=read(L/'r484-consumer-candidate.json');player=read(L/'r484-player-candidate.json');exit_result=read(L/'r484-consumer-exit.json')
for d in [m,binding,regular,row,reference,details,host,player,exit_result,read(L/'r484-probe-candidate.json'),read(L/'r484-row-reference-candidate.json')]:check(d)
ids=[(0,566),(0,570),(2,554),(2,513),(2,527),(2,518),(2,521),(2,522),(5,532),(5,512),(5,517),(5,524),(5,530)]
assert [(b['mode'],b['map_id'])for b in m['batch']]==ids
assert len(host['source_pins'])==157 and host['production_library']==player['production_library']
old=read(L/'r483-candidate.json');changed=[]
for s in old['source_pins']:
 assert pin(s['snapshot'])['sha256']==s['sha256']
 if pin(s['path'])['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert sorted(changed)==['native_city_flow.rs','native_city_resources.rs']
def old_text(name):return Path(next(s for s in old['source_pins']if Path(s['path']).name==name)['snapshot']).read_text(encoding='utf-8')
city=(P/'src/modern_lito/native_city_resources.rs').read_text(encoding='utf-8');prior=old_text('native_city_resources.rs');marker='  ensure!(plan.archives.len()==4'
assert city[city.index(marker):]==prior[prior.index(marker):], 'R483 row validation/index/decoder/render body must remain unchanged'
assert 'pub fn load_catalog('in city and 'native_multisector_resources::load(inputs)?'in city and 'pub(crate) fn load_whole('in city
assert 'let n=match plan.map_id{534=>16,535=>25' in city, 'legacy guarded API retained'
flow=(P/'src/modern_lito/native_city_flow.rs').read_text(encoding='utf-8');assert 'b.plan.sectors.len()>=16' in flow and 'CityAssets::load_whole('in flow
for n in ['native_city_runtime.rs','native_city_window.rs','native_resources.rs','native_multisector_resources.rs','native_multisector_flow.rs','native_multisector_scene.rs','native_multisector_runtime.rs','mod.rs']:assert (P/'src/modern_lito'/n).read_text(encoding='utf-8')==old_text(n)
assert pin(L/'r483-two-whole-city-visible-row-handoff.json')['sha256']=='85085C729B72B8CB1A44F7887F1B54248E0AC08608F796F0DADD3227A12A96F5'
assert pin(P/'target/r483b-player-entry-candidate/LiToModern-R483.exe')['sha256']=='F2E4C712AC6FF6F60CB1ABF98208F6AC210261AD68B646FBE79D2F1ABC6DD81D'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
assert binding['bindings'][:24]==read(L/'r483b-map-binding.json')['bindings']
assert [(b['mode'],b['map_id'])for b in binding['bindings'][24:]]==ids
assert binding['unqualified_count']==binding['remaining_unqualified_maps']==3 and len(binding['bindings'])==37
catalog=read(binding['catalog_source']['path']);assert binding['catalog40']==[r['original_table']for r in catalog['rows']] and len(binding['catalog40'])==40
assert (regular['passed'],regular['measured_not_qualified'],regular['failed'])==(12,1,0)
measure=next(r for r in regular['results']if r['map_id']==527)
assert measure['draw_references']==362852 and measure['supported_decoded_bytes']==10920960 and measure['fixed_adapter_result']=='Native draw budget exceeded' and not measure['refusal_members']
assert row['passed']==1 and row['failed']==0 and reference['passed']==13 and reference['failed']==0
assert [(r['mode'],r['map_id'])for r in reference['results']]==ids
total=0
for b in m['batch']:
 r=next(r for r in reference['results']if r['map_id']==b['map_id']);authority=next(r for r in catalog['rows']if r['logical_map']==b['map_id']);assert authority['mode']==b['mode']
 assert len(b['sectors'])==len(authority['native_sector_files'])==len(list(Path(b['original_directory']).glob('*.s32')));total+=len(b['sectors'])
 for own,original in zip(b['sectors'],authority['native_sector_files']):assert Path(own['pin']['path']).name==Path(original['path']).name and own['pin']['bytes']==original['bytes'] and own['pin']['sha256']==original['sha256']
 plan=next(p for p in binding['bindings']if p['map_id']==b['map_id']);assert plan['sectors']==b['sectors'] and plan['canonical_sha256']==r['canonical_sha256']
 a=b['sectors'][0];positions=[(1536*(s['x']-a['x']+s['y']-a['y']),768*(-s['x']+a['x']+s['y']-a['y']))for s in b['sectors']]
 bounds=[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)];assert bounds==b['expected_bounds']==r['bounds']
 assert all(s['unknown_suffix']==0 for s in r['sector_stats']) and len(r['sector_stats'])==len(b['sectors'])
 assert r['budget']['decoded_bytes']<64<<20
 if b['map_id']!=527:assert r['budget']['draw_references']<=262144 and len(b['sectors'])<16
 for image in r['images'][1:3]:
  assert image['visible_static_objects']>20
  for sx,sy in r['seam']['source_pair']:assert f'{sx:04x}{sy:04x}'in image['visible_sectors']
assert total==115
assert details['new_identities']==[list(i)for i in ids] and details['new_complete_sectors']==115
assert len(details['images'])==70 and len(details['map_cases'])==len(details['failure_cases'])==13
assert details['negative_cases']==178 and details['actual_cross_mode_or_map_queued_load_pan_rejections']==26
assert details['actual_worker_retired'] and details['thirteen_active_load_Back_cancel_switch_cases']
assert len([i for i in details['images']if i.get('profile')=='generic_catalog_row_direct'])==5
for actual in details['images']:
 expected=next(i for r in reference['results']if (r['mode'],r['map_id'])==tuple(actual['identity'])for i in r['images']if i['name']==actual['name'])
 assert actual['camera']==expected['camera'] and actual['image']['rgba_sha256']==expected['image']['rgba_sha256'] and actual['image']['sha256']==expected['image']['sha256']
for c in details['map_cases']:
 assert c['actual_load_and_five_player_frames']and c['wrong_mode_same_ID_refusal_keeps_current_frame']and c['queued_load_and_pan_cross_mode_or_map_rejected']and c['actual_active_load_Back_cancel_switch_no_frame']
 assert c['profile']==('generic_rows'if c['identity'][1]==527 else'unchanged_regular')
 g=next(b for b in binding['bindings']if b['map_id']==c['identity'][1]);last=g['sectors'][-1];assert not(O/f"missing-native-{g['map_id']}"/str(g['map_id'])/Path(last['pin']['path']).name).exists()
study=details['generic_row_map527'];metrics=study['metrics'];assert study['whole_catalog_authority_required']and study['budgets_unchanged']
assert (metrics['source_references'],metrics['decoded_bytes'],metrics['non_floor_references'],metrics['index_payload_capacity_bytes'])==(362852,10920960,215396,2584752)
assert study['asset_drop_freed_requested_heap_bytes']>metrics['decoded_bytes']+metrics['index_payload_capacity_bytes']
assert study['load_incremental_peak_requested_heap_bytes']>=study['asset_drop_freed_requested_heap_bytes']
assert len(study['phase_cancellations'])==4
for c in study['phase_cancellations']:assert c['requested_phase']==c['observed_phase']and 'cancel'in c['error'].lower()and not c['hard_OS_deadline_promised']
assert all(f['metrics']['visible_fragments']<=262144 and f['metrics']['visited_index_rows']<=metrics['index_rows']for f in study['frames'])
assert details['qualified_catalog_previews']==37 and details['remaining_unqualified']==3 and details['known_format_gaps']==[506,573,563]
qualified={(b['mode'],b['map_id'])for b in binding['bindings']};remaining=[r for r in binding['catalog40']if(r['mode'],r['map_id'])not in qualified];assert {r['map_id']for r in remaining}=={506,573,563}
assert (details['complete_modes'],details['total_modes'],details['GUI'],details['LOADED'])==(16,40,0,False)
assert exit_result['exit_code']==0 and exit_result['executions']==1
scope=read(L/'source-change-scope.json');assert scope['source_files']==601 and len(scope['new_source_files'])==122 and len(scope['changed_source_files'])==10
for s in scope['new_source_files']:assert pin(s['path'])['sha256']==s['sha256']
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0'and struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
root=Path(player['original_main']['path']).read_bytes();t=Path(player['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n'if'\r\n'in t else'\n'
for a in ['    CandidateEntry,'+nl+'    OfflineMap,'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl,'        "candidate_revision": "r484",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl]:assert t.count(a)==1;t=t.replace(a,'')
a=t.index('fn candidate_offline_entry(');z=t.index('fn version_document()',a);t=t[:a]+t[z:];assert t.encode('utf-8')==root
out=dict(state='R484_INDEPENDENT_THIRTEEN_FULL_SOURCE_MAP_BUDGET_REFERENCE_GENERIC_ROWS_SCOPE_IDENTITY_CANCEL_MISSING_PASS',source_pairs=157,source_files=601,new_sources=122,checked_pin_instances=checked,unique_files=len(cache),new_identities=[list(i)for i in ids],new_complete_sectors=115,regular_maps=12,generic_row_map=527,CPU_reference_pairs=70,worker_CPU_pairs=65,negative_cases=178,stale_results_rejected=26,focused_seconds=details['focused_seconds'],focused_exit_code=0,old_R483_current_sources_changed=changed,old_row_backend_budget_index_decode_render_tail_unchanged=True,no_new_runtime_or_window=True,old24_map_plans_unchanged=True,previously_accepted_previews=24,new13_review='PENDING',current_CPU_evidenced_previews=37,remaining_actual_catalog_rows=remaining,known_format_gaps=[506,573,563],complete_modes=16,GUI=0,LOADED=False)
f=L/'r484-independent-contract.json';tmp=f.with_suffix('.json.tmp');tmp.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');tmp.replace(f);print(json.dumps(out,ensure_ascii=False))
