from pathlib import Path
import json,hashlib,struct,re
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r481-multisector-batch'
def read(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def pin(p):p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
cache={};checked=0
def check(v):
 global checked
 if isinstance(v,list):
  for x in v:check(x)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   a=cache.setdefault(v['path'],None)
   if a is None:a=pin(v['path']);cache[v['path']]=a
   assert a['bytes']==v['bytes'] and a['sha256']==v['sha256'],v['path'];checked+=1
  for x in v.values():check(x)
m=read(L/'r481-resource-input.json');binding=read(L/'r481-map-binding.json');probe=read(O/'reference/reference.json');diag=read(O/'map506-diagnostic.json');details=read(L/'r481-multisector-batch-details.json');host=read(L/'r481b-candidate.json');player=read(L/'r481-player-candidate.json')
for d in [m,binding,probe,diag,details,host,player,read(L/'r481-probe-candidate.json'),read(L/'r481b-506-diagnostic-candidate.json')]:check(d)
assert probe['passed']==6 and probe['failed']==1 and probe['results'][2]['map_id']==506
ids=[551,504,526,525,531,529];results=[r for r in probe['results'] if r.get('budget')];assert [r['map_id'] for r in results]==ids
assert len(host['source_pins'])==146
for s in host['source_pins']:a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256']
oldsource=read(L/'r480-candidate.json');changed=[]
for s in oldsource['source_pins']:
 a=pin(s['snapshot']);assert a['bytes']==s['bytes'] and a['sha256']==s['sha256']
 if pin(s['path'])['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
assert changed==['mod.rs'],changed
for file in ['native_resources.rs','native_multisector_scene.rs','native_multisector_resources.rs','native_multisector_flow.rs','native_multisector_runtime.rs']:
 old=next(s for s in oldsource['source_pins'] if Path(s['path']).name==file);assert pin(old['path'])['sha256']==old['sha256']
assert pin(L/'r480-complete-multisector552-handoff.json')['sha256']=='11B2C8DCA38FFBBA4F747105C09BE5A28651EA2AA78F136A0F4DC5F2F132D3B5'
assert pin(P/'target/r480-player-entry-candidate/LiToModern-R480.exe')['sha256']=='13F1C353F9631FB6D8D26C9381981FE7E71D66EBF4ED34628E8050753765773A'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
old=read(L/'r480-map-binding.json');assert binding['bindings'][:7]==old['bindings'];assert [b['map_id'] for b in binding['bindings'][7:]]==ids;assert binding['unqualified_count']==binding['remaining_unqualified_maps']==27
catalog=read(binding['catalog_source']['path']);assert binding['catalog40']==[r['original_table'] for r in catalog['rows']]
total=0;shapes=set();negative_y=[]
for b,r in zip(binding['bindings'][7:],results):
 input=next(x for x in m['batch'] if x['map_id']==b['map_id']);original=next(x for x in catalog['rows'] if x['logical_map']==b['map_id']);sectors=b['sectors'];assert len(sectors)==len(original['native_sector_files'])==len(list(Path(input['original_directory']).glob('*.s32')))
 assert input['original_sectors']==[{'map_id':b['map_id'],'x':int(Path(p['path']).stem[:4],16),'y':int(Path(p['path']).stem[4:],16),'pin':p} for p in original['native_sector_files']]
 for s,o in zip(sectors,input['original_sectors']):assert s['x']==o['x'] and s['y']==o['y'] and s['pin']['bytes']==o['pin']['bytes'] and s['pin']['sha256']==o['pin']['sha256']
 anchor=sectors[0];positions=[(1536*(s['x']-anchor['x']+s['y']-anchor['y']),768*(-s['x']+anchor['x']+s['y']-anchor['y'])) for s in sectors];bounds=[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)]
 assert bounds==input['expected_bounds']==r['bounds'];assert r['anchor']==[anchor['x'],anchor['y']];assert b['canonical_sha256']==r['canonical_sha256']
 budget=r['budget'];assert 0<budget['sector_bytes']<=budget['map_byte_limit'] and 0<budget['decoded_bytes']<=budget['decoded_limit'] and 0<budget['draw_references']<=budget['draw_limit'];assert all(s['unknown_suffix']==0 for s in r['sector_stats'])
 for picture in r['images'][1:3]:
  for sx,sy in r['seam']['source_pair']:assert f'{sx:04x}{sy:04x}' in picture['visible_sectors']
  assert picture['visible_static_objects']>20
 total+=len(sectors);shapes.add(r['shape'])
 if bounds[1]<0:negative_y.append(b['map_id'])
assert total==34 and len(shapes)==6 and len(negative_y)==5
assert details['new_whole_map_ids']==ids and details['negative_cases']==74 and details['actual_new_cross_map_queued_load_and_pan_rejections']==12 and len(details['images'])==30 and len(details['map_cases'])==6 and len(details['failure_cases'])==7
for actual in details['images']:
 expected=next(i for r in results if r['map_id']==actual['map_id'] for i in r['images'] if i['name']==actual['name']);assert actual['camera']==expected['camera'];assert actual['image']['rgba_sha256']==expected['image']['rgba_sha256'] and actual['image']['sha256']==expected['image']['sha256']
assert len(diag['failures'])==10 and {f['tile_member'] for f in diag['failures']}=={'198.til','199.til'} and {f['error'] for f in diag['failures']}=={'unsupported classic TIL block type 22','unsupported classic TIL block type 23'}
archives=diag['archives'];index=Path(archives[0]['path']).read_bytes();pak=Path(archives[1]['path']).read_bytes();entries={name.split(b'\0')[0].decode('ascii').lower():(off,size) for off,name,size in struct.iter_unpack('<I20sI',index[4:])}
located=[]
for f in diag['failures']:
 off,size=entries[f['tile_member']];assert (off,size)==(f['archive_member_offset'],f['archive_member_bytes']);raw=pak[off:off+size];assert hashlib.sha256(raw).hexdigest().upper()==f['member_sha256'];count=struct.unpack_from('<I',raw)[0];slot_offset=struct.unpack_from('<I',raw,4+4*f['slot'])[0];data_start=4+4*(count+1);code=raw[data_start+slot_offset];assert code in [22,23] and str(code) in f['error'];located.append(dict(tile=f['tile_member'],slot=f['slot'],type=code,offset_in_member=data_start+slot_offset))
log=(L/'r481b-consumer.stdout.log').read_text(encoding='utf-8');assert '1 passed; 0 failed; 0 ignored;' in log;seconds=float(re.search(r'finished in ([\d.]+)s',log).group(1))
root=Path(player['original_main']['path']).read_bytes();t=Path(player['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for addition in ['    CandidateEntry,'+nl+'    OfflineMap,'+nl,'        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),'+nl+'        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),'+nl,'        Route::CandidateEntry => candidate_offline_entry(false)?,'+nl+'        Route::OfflineMap => candidate_offline_entry(true)?,'+nl,'        "candidate_revision": "r481",'+nl+'        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],'+nl+'        "offline_window_gui_verified": false,'+nl]:assert t.count(addition)==1;t=t.replace(addition,'')
a=t.index('fn candidate_offline_entry(');z=t.index('fn version_document()',a);t=t[:a]+t[z:];assert t.encode('utf-8')==root
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert exe[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
window=(P/'src/modern_lito/native_multisector_batch_window.rs').read_text(encoding='utf-8');assert window==(P/'src/modern_lito/native_multisector_window.rs').read_text(encoding='utf-8').replace('R480','R481').replace('0x480','0x481')
scope=read(L/'source-change-scope.json');assert scope['source_files']==595 and len(scope['new_source_files'])==116 and len(scope['changed_source_files'])==10
record=dict(state='R481_INDEPENDENT_COMPLETE_SIX_MAP_SOURCE_GEOMETRY_BUDGET_NATIVE_ORDER_PLAYER_PIXELS_CANCEL_STALE_FAILURE_AND_TYPE22_23_PROVEN',checked_pin_instances=checked,unique_files=len(cache),source_pairs=146,new_maps=ids,new_complete_sectors=34,shapes=sorted(shapes),negative_projected_Y_maps=negative_y,actual_player_reference_pairs=30,negative_cases=74,stale_results_rejected=12,focused_seconds=seconds,R480_current_source_changes=['mod.rs'],shared_adapter_renderer_flow_runtime_byte_unchanged=True,root_main_exact_inverse=True,PE_machine=332,PE_subsystem=2,source506_required_unsupported_slots=located,known_Layer7_gaps=[573,563],qualified_catalog_previews=13,remaining_unqualified_catalog_maps=27,complete_modes=16,total_modes=40,GUI=0,LOADED=False)
dest=L/'r481-independent-contract.json';tmp=dest.with_suffix('.tmp');tmp.write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(dest);print(json.dumps(dict(PASS=True,source_pairs=146,new_maps=6,source_sectors=34,images=30,negative=74,seconds=seconds,unique_files=len(cache))))
