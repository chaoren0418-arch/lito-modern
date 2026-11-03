from pathlib import Path
import json,hashlib,struct,re
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r482-cross-mode-batch'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def pin(p):p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
cache={};checked=0
def check(v):
 global checked
 if isinstance(v,list):
  for x in v:check(x)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   a=cache.get(v['path'])
   if a is None:a=pin(v['path']);cache[v['path']]=a
   assert a['bytes']==v['bytes'] and a['sha256']==v['sha256'],v['path'];checked+=1
  for x in v.values():check(x)
m=read(L/'r482-resource-input.json');binding=read(L/'r482-map-binding.json');probe=read(O/'reference/reference.json');design=read(L/'r482-large-map-measurement-design.json');details=read(L/'r482-cross-mode-details.json');host=read(L/'r482a-candidate.json');player=read(L/'r482-player-candidate.json')
for d in [m,binding,probe,design,details,host,player,read(L/'r482-probe-candidate.json')]:check(d)
assert (probe['passed'],probe['measured_not_qualified'],probe['failed'])==(9,2,0)
identities=[(0,565),(0,569),(2,519),(2,520),(2,553),(3,547),(4,540),(5,510),(5,528)];qualified=[r for r in probe['results'] if r.get('budget')];measured=[r for r in probe['results'] if r.get('measurement_only')];assert [(r['mode'],r['map_id']) for r in qualified]==identities
assert len(host['source_pins'])==149
for s in host['source_pins']:a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256']
oldsource=read(L/'r481-candidate.json');changed=[]
for s in oldsource['source_pins']:
 a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256']
 if pin(s['path'])['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert changed==['mod.rs'],changed
assert pin(L/'r481-six-complete-multisector-batch-handoff.json')['sha256']=='D812A428F1C7568AEFF891089A24B94D0C1F8A27F12D7483FBD940BCA275D4B7'
assert pin(P/'target/r481-player-entry-candidate/LiToModern-R481.exe')['sha256']=='4349487FF3752A9CD95A8BF6B93F7B296489078B3984B8B0F0403B9AA55F4F95'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
for name in ['native_resources.rs','native_multisector_resources.rs','native_multisector_flow.rs','native_multisector_scene.rs','native_multisector_runtime.rs','world_depth.rs']:
 s=next(s for s in oldsource['source_pins'] if Path(s['path']).name==name);assert pin(s['path'])['sha256']==s['sha256']
oldbinding=read(L/'r481-map-binding.json');assert binding['bindings'][:13]==oldbinding['bindings'];assert [(b['mode'],b['map_id']) for b in binding['bindings'][13:]]==identities;assert binding['unqualified_count']==binding['remaining_unqualified_maps']==18
catalog=read(binding['catalog_source']['path']);assert binding['catalog40']==[r['original_table'] for r in catalog['rows']];assert len({r['map_id'] for r in binding['catalog40']})==40
total=0
for b in m['batch']:
 r=next(r for r in probe['results'] if r['map_id']==b['map_id']);authority=next(r for r in catalog['rows'] if r['logical_map']==b['map_id']);assert authority['mode']==b['mode'];sectors=b['sectors'];assert len(sectors)==len(authority['native_sector_files'])==len(list(Path(b['original_directory']).glob('*.s32')))
 assert b['original_sectors']==[dict(map_id=b['map_id'],x=int(Path(p['path']).stem[:4],16),y=int(Path(p['path']).stem[4:],16),pin=p) for p in authority['native_sector_files']]
 for own,original in zip(sectors,b['original_sectors']):assert own['x']==original['x'] and own['y']==original['y'] and own['pin']['bytes']==original['pin']['bytes'] and own['pin']['sha256']==original['pin']['sha256']
 a=sectors[0];positions=[(1536*(s['x']-a['x']+s['y']-a['y']),768*(-s['x']+a['x']+s['y']-a['y'])) for s in sectors];bounds=[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)];assert bounds==b['expected_bounds']==r['bounds'];assert r['anchor']==[a['x'],a['y']];assert len(r['sector_stats'])==len(sectors) and all(s['unknown_suffix']==0 for s in r['sector_stats'])
 if not b['measurement_only']:
  plan=next(p for p in binding['bindings'] if p['map_id']==b['map_id']);assert plan['sectors']==sectors and plan['canonical_sha256']==r['canonical_sha256'];budget=r['budget'];assert 0<budget['sector_bytes']<=budget['map_byte_limit'] and 0<budget['draw_references']<=budget['draw_limit'] and 0<budget['decoded_bytes']<=budget['decoded_limit'];total+=len(sectors)
  for image in r['images'][1:3]:
   for sx,sy in r['seam']['source_pair']:assert f'{sx:04x}{sy:04x}' in image['visible_sectors']
   assert image['visible_static_objects']>20
assert total==78 and sum(len(b['sectors']) for b in m['batch'])==119
assert [(r['map_id'],r['sectors'],r['draw_references'],r['supported_decoded_bytes']) for r in measured]==[(534,16,347528,32601600),(535,25,558061,29475072)]
assert all(r['fixed_adapter_result']=='Native draw budget exceeded' and r['draw_references']>262144 and r['supported_decoded_bytes']<67108864 and not r['refusal_members'] for r in measured)
assert design['measurements']==measured and design['draw_excess']=={'map534':85384,'map535':295917}
assert details['new_identities']==[list(i) for i in identities] and details['negative_cases']==122 and details['actual_cross_mode_or_map_queued_load_pan_rejections']==18 and len(details['images'])==45 and len(details['map_cases'])==9 and len(details['failure_cases'])==11
for actual in details['images']:
 mode,id=actual['identity'];expected=next(i for r in qualified if (r['mode'],r['map_id'])==(mode,id) for i in r['images'] if i['name']==actual['name']);assert actual['camera']==expected['camera'] and actual['image']['rgba_sha256']==expected['image']['rgba_sha256'] and actual['image']['sha256']==expected['image']['sha256']
pair=[next(b for b in m['batch'] if b['map_id']==id) for id in [519,520]];assert [(s['x'],s['y'],s['pin']['bytes'],s['pin']['sha256']) for s in pair[0]['sectors']]==[(s['x'],s['y'],s['pin']['bytes'],s['pin']['sha256']) for s in pair[1]['sectors']]
pair_ref=[next(r for r in qualified if r['map_id']==id) for id in [519,520]];assert pair_ref[0]['canonical_sha256']!=pair_ref[1]['canonical_sha256'];assert [(i['name'],i['camera'],i['image']['rgba_sha256']) for i in pair_ref[0]['images']]==[(i['name'],i['camera'],i['image']['rgba_sha256']) for i in pair_ref[1]['images']]
log=(L/'r482a-consumer.stdout.log').read_text(encoding='utf-8');assert '1 passed; 0 failed; 0 ignored;' in log;seconds=float(re.search(r'finished in ([\d.]+)s',log).group(1))
root=Path(player['original_main']['path']).read_bytes();t=Path(player['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for addition in ['    CandidateEntry,'+nl+'    OfflineMap,'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl,'        "candidate_revision": "r482",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl]:assert t.count(addition)==1;t=t.replace(addition,'')
a=t.index('fn candidate_offline_entry(');z=t.index('fn version_document()',a);t=t[:a]+t[z:];assert t.encode('utf-8')==root
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
window=(P/'src/modern_lito/native_cross_mode_window.rs').read_text(encoding='utf-8');assert window==(P/'src/modern_lito/native_multisector_batch_window.rs').read_text(encoding='utf-8').replace('R481','R482').replace('0x481','0x482')
scope=read(L/'source-change-scope.json');assert scope['source_files']==597 and len(scope['new_source_files'])==118 and len(scope['changed_source_files'])==10
record=dict(state='R482_INDEPENDENT_NINE_COMPLETE_CROSS_MODE_RESOURCE_GEOMETRY_BUDGET_PIXEL_IDENTITY_CANCEL_STALE_MISSING_AND_FULL_CITY_REFUSAL_PASS',checked_pin_instances=checked,unique_files=len(cache),source_pairs=149,new_identities=identities,new_complete_sectors=78,all_input_sectors=119,actual_player_reference_pairs=45,negative_cases=122,stale_results_rejected=18,focused_seconds=seconds,background_queue_busy_retries=details['background_queue_transient_busy_retries'],source_identical_pair=[519,520],canonical_pair_distinct=True,wrong_mode_same_map_ID_source_rejected=True,full_city_measurements=measured,large_map_design_only_no_implementation=True,new_batch_unsupported_tile_types=[],known_unchanged_format_gaps=[506,573,563],R481_current_source_changes=['mod.rs'],shared_adapter_renderer_flow_runtime_byte_unchanged=True,root_main_exact_inverse=True,PE_machine=332,PE_subsystem=2,qualified_catalog_previews=22,remaining_unqualified_catalog_maps=18,complete_modes=16,total_modes=40,GUI=0,LOADED=False)
dest=L/'r482-independent-contract.json';tmp=dest.with_suffix('.tmp');tmp.write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(dest);print(json.dumps(dict(PASS=True,source_pairs=149,new_maps=9,sectors=78,images=45,negative=122,seconds=seconds,unique_files=len(cache))))
