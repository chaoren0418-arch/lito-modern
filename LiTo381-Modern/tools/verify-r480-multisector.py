from pathlib import Path
import json,hashlib,struct,re
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r480-multisector'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def pin(p):p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
checks=0;unique=set()
def check(v):
 global checks
 if isinstance(v,list):
  for x in v:check(x)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   a=pin(v['path']);assert a['bytes']==v['bytes'] and a['sha256']==v['sha256'],v['path'];checks+=1;unique.add(v['path'])
  for x in v.values():check(x)
input=read(L/'r480-resource-input.json');binding=read(L/'r480-map-binding.json');details=read(L/'r480-multisector-details.json');reference=read(O/'reference-b/reference.json');source=read(L/'r480b-candidate.json');player=read(L/'r480-player-candidate.json');geometry=read(L/'r480-original-geometry-order.json');probe=read(L/'r480b-probe-candidate.json')
for d in [input,binding,details,reference,source,player,geometry,probe]:check(d)
assert len(source['source_pins'])==143
for s in source['source_pins']:
 a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256'];checks+=1;unique.add(s['snapshot'])
previous=read(L/'r479-candidate.json');changed=[]
for s in previous['source_pins']:
 a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256']
 if pin(s['path'])['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert changed==['mod.rs'],changed
assert pin(L/'r479-six-native-map-preview-handoff.json')['sha256']=='E8DA26794E4DACAA73BB1BA4FEB132969EF0F4BD40F110A8D72632998FFDB584'
assert pin(P/'target/r479b-player-entry-candidate/LiToModern-R479b.exe')['sha256']=='71A694E44E6BAFD4FB4960F0F836042283B308D5A03CBCE30E5814A7901001E1'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
raw=Path(player['candidate_main']['path']).read_bytes(); original=Path(player['original_main']['path']).read_bytes();t=raw.decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for addition in ['    CandidateEntry,'+nl+'    OfflineMap,'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl,'        "candidate_revision": "r480",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl]:assert t.count(addition)==1;t=t.replace(addition,'')
a=t.index('fn candidate_offline_entry(');z=t.index('fn version_document()',a);t=t[:a]+t[z:];assert t.encode('utf-8')==original
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0';assert struct.unpack_from('<H',exe,pe+4)[0]==332;assert struct.unpack_from('<H',exe,pe+24+68)[0]==2
assert player['only_executed_candidate_flag']=='--version' and player['version']['candidate_revision']=='r480' and player['GUI']==0
assert [(b['mode'],b['map_id']) for b in binding['bindings']]==[(1,564),(1,514),(1,509),(1,508),(1,505),(1,516),(1,552)]
assert binding['unqualified_count']==binding['remaining_unqualified_maps']==33
old=read(L/'r479b-map-binding.json');assert binding['bindings'][:6]==old['bindings']
full=binding['bindings'][6];sectors=full['sectors'];assert [(s['x'],s['y']) for s in sectors]==[(32767,32767),(32767,32768),(32768,32767),(32768,32768)]
assert full['canonical_sha256']==reference['canonical_sha256'];assert len(list(Path(input['original_directory']).glob('*.s32')))==4
for own,orig in zip(sectors,input['original_sectors']):assert own['pin']['bytes']==orig['pin']['bytes'] and own['pin']['sha256']==orig['pin']['sha256']
catalog=read(binding['catalog_source']['path']);authority=next(r for r in catalog['rows'] if r['logical_map']==552);assert len(authority['native_sector_files'])==4;assert binding['catalog40']==[r['original_table'] for r in catalog['rows']]
for actual,original_pin in zip(input['original_sectors'],authority['native_sector_files']):assert actual['pin']==original_pin
positions=[(1536*((s['x']-32767)+(s['y']-32767)),768*(-(s['x']-32767)+(s['y']-32767))) for s in sectors];bounds=[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)];assert bounds==reference['bounds']==details['signed_bounds']==[0,-768,6144,2304]
assert details['negative_cases']==24 and details['actual_old_multisector_load_and_pan_results_rejected']==2 and len(details['failure_cases'])==6
assert len(details['images'])==len(reference['images'])==9
for actual,expected in zip(details['images'],reference['images']):assert actual['name']==expected['name'] and actual['camera']==expected['camera'];assert actual['image']['sha256']==expected['image']['sha256'] and actual['image']['rgba_sha256']==expected['image']['rgba_sha256']
assert len(reference['images'][0]['visible_sector_references'])==4
assert reference['images'][1]['visible_sector_references']['80007fff']<reference['images'][2]['visible_sector_references']['80007fff']
assert reference['images'][3]['visible_sector_references']['7fff8000']<reference['images'][4]['visible_sector_references']['7fff8000']
log=(L/'r480b-consumer.stdout.log').read_text(encoding='utf-8');assert '1 passed; 0 failed; 0 ignored;' in log and 'finished in 35.97s' in log
for proc in ['sub_4B8FC0','sub_483770','sub_488860']:
 f=next(f for f in geometry['fragments'] if Path(f['path']).stem==proc);assert re.match(proc.encode()+rb'\s+proc near',Path(f['path']).read_bytes())
scene=(P/'src/modern_lito/native_multisector_scene.rs').read_text(encoding='utf-8');assert 'draws.sort_by_key(|d|d.key)' in scene and 'cell.overlay_key(d,insertion)' in scene and 'camera.x' in scene
runtime=(P/'src/modern_lito/native_multisector_runtime.rs').read_text(encoding='utf-8');oldruntime=(P/'src/modern_lito/native_mode_menu_runtime.rs').read_text(encoding='utf-8');assert runtime==oldruntime.replace('native_multi_map_resources','native_multisector_resources').replace('native_offline_map_flow','native_multisector_flow')
window=(P/'src/modern_lito/native_multisector_window.rs').read_text(encoding='utf-8');oldwindow=(P/'src/modern_lito/native_map_batch_window.rs').read_text(encoding='utf-8');assert window==oldwindow.replace('native_mode_menu_runtime','native_multisector_runtime').replace('R479','R480').replace('0x479','0x480')
scope=read(L/'source-change-scope.json');assert scope['source_files']==593 and len(scope['new_source_files'])==114 and len(scope['changed_source_files'])==10
out=dict(state='R480_INDEPENDENT_SOURCE_FULL_ORIGINAL552_GEOMETRY_NATIVE_SCAN_CAMERA_REFERENCE_PLAYER_NEGATIVE_CONTRACT_PASS',checked_pin_instances=checks,unique_files=len(unique),source_pairs=143,prior_R479_current_source_changes=['mod.rs'],root_main_exact_inverse=True,PE_machine=332,PE_subsystem=2,complete_original_sector_list=True,sectors=4,catalog_previews=7,remaining_catalog_unqualified=33,actual_CPU_reference_pairs=9,negative_cases=24,focused_seconds=35.97,original_native_projection_order_fragments=geometry,Modern_adaptations=['Whole map bounding rectangle camera, retained background outside sector diamonds; no collision or native spawn','Canonical sector/source serialization for equal cell/depth insertion; original streaming insertion history unobserved'],known_Layer7_gaps=[573,563],complete_modes=16,total_modes=40,GUI=0,LOADED=False)
dest=L/'r480-independent-contract.json';tmp=dest.with_suffix('.tmp');tmp.write_text(json.dumps(out,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(dest)
print(json.dumps(dict(PASS=True,source_pairs=143,images=9,negative=24,pin_instances=checks,unique_files=len(unique))))
