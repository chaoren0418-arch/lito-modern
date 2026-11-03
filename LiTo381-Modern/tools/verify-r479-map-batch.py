"""Independent source/pin/identity and fixed-library/reference provenance verification."""
from pathlib import Path
import json,hashlib,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def check(v):
 a=pin(v['path']);assert(a['bytes'],a['sha256'])==(v['bytes'],v['sha256'].upper()),v['path']
def read(n):return json.loads((L/n).read_text(encoding='utf-8-sig'))
entry=read('r479b-entry-source-contract.json');player=read('r479-player-candidate.json');host=read('r479-candidate.json');details=read('r479-map-batch-details.json');binding=read('r479b-map-binding.json');inventory=read('r479-resource-inventory-input.json');audit=read('r479-resource-completeness-audit.json');raw=read('r479-native38-inventory.json');probe=json.loads((P/'target/r479-map-batch/probe/batch-results.json').read_text(encoding='utf-8'))
for v in [entry['original_main'],entry['candidate_main'],entry['binding'],entry['skin_binding'],player['executable'],player['production_library'],player['version_stdout'],player['version_stderr']]:check(v)
t=Path(entry['candidate_main']['path']).read_bytes().decode('utf-8');nl='\r\n' if '\r\n' in t else '\n'
for line in ['    CandidateEntry,','    OfflineMap,','        Some(x) if x == OsStr::new("--candidate-menu") && args.len() == 2 => Ok(Route::CandidateEntry),','        Some(x) if x == OsStr::new("--offline-map-preview") && args.len() == 2 => Ok(Route::OfflineMap),','        Route::CandidateEntry => candidate_offline_entry(false)?,','        Route::OfflineMap => candidate_offline_entry(true)?,','        "candidate_revision": "r479b",','        "offline_entry_flags": ["--candidate-menu", "--offline-map-preview"],','        "offline_window_gui_verified": false,']:
 assert t.count(line+nl)==1;t=t.replace(line+nl,'')
start=t.index('fn candidate_offline_entry(');end=t.index('fn version_document()',start);t=t[:start]+t[end:];assert t.encode('utf-8')==Path(entry['original_main']['path']).read_bytes()
b=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert b[pe:pe+4]==b'PE\0\0';assert struct.unpack_from('<H',b,pe+4)[0]==332 and struct.unpack_from('<H',b,pe+24+68)[0]==2
assert len(host['source_pins'])==136
for v in host['source_pins']:check(v);assert pin(v['snapshot'])['sha256']==v['sha256']
window=(P/'src/modern_lito/native_map_batch_window.rs').read_bytes();assert window.replace(b'R479',b'R478').replace(b'0x479',b'0x478')==(P/'src/modern_lito/native_mode_menu_window.rs').read_bytes()
original=read('r474-mode-map-binding.json');assert binding['catalog40']==[r['original_table'] for r in original['rows']]
assert [(r['mode'],r['map_id']) for r in binding['bindings']]==[(1,564),(1,514),(1,509),(1,508),(1,505),(1,516)]
assert len(inventory['rows'])==38 and sum(len(r['sectors']) for r in inventory['rows'])==305 and len(raw['rows'])==38
for r in inventory['rows']:
 expected=next(v for v in original['rows'] if (v['mode'],v['logical_map'])==(r['mode'],r['map_id']));assert [s['pin'] for s in r['sectors']]==expected['native_sector_files']
 for s in r['sectors']:check(s['pin'])
assert len(audit['decoder_rejected_rows'])==2 and {r['map_id'] for r in audit['decoder_rejected_rows']}=={573,563};assert audit['fully_parsed_rows']==36 and audit['remaining_multi_sector_maps']==34
assert not any(v['missing_tile_index_members'] for v in raw['rows'])
new=inventory['batch'];assert [b['map_id'] for b in new]==[509,508,505,516]
for b,r,i in zip(new,probe['results'],details['images']):
 assert b['map_id']==r['map_id']==i['identity'][1] and r['mode']==i['identity'][0]==1
 assert len(b['sectors'])==len(b['original_sectors'])==1;check(b['sectors'][0]['pin']);check(b['original_sectors'][0]['pin']);assert b['sectors'][0]['pin']['sha256']==b['original_sectors'][0]['pin']['sha256']
 for a in b['archives']:check(a)
 check(i);check(r['image']);cpu=Image.open(i['path']).convert('RGBA');ref=Image.open(r['image']['path']).convert('RGBA');assert cpu.size==ref.size==(800,600) and cpu.tobytes()==ref.tobytes();assert hashlib.sha256(cpu.tobytes()).hexdigest().upper()==i['rgba_sha256']==r['image']['rgba_sha256']
 assert r['all_complete_source_static_slots_decoded'] and r['nonbackground_pixels']>10000
 plan=next(v for v in binding['bindings'] if v['map_id']==b['map_id']);assert plan['canonical_sha256']==r['canonical_sha256'] and plan['sectors']==b['sectors'] and plan['namespace']=='NATIVE_LITO_LTMAP_V1'
assert len({i['rgba_sha256'] for i in details['images']})==4 and details['negative_cases']==13 and len(details['failure_cases'])==7
assert details['actual_queued_old564_and_new516_pan_rejections']>=2 and details['actual_new516_load_Back_cancel_and_old564_select_retirement']
for v in details['failure_cases']:assert v['no_fallback_frame']
metadata=read('r479-metadata-refinement.json');oldbinding=read('r479-map-binding.json');assert {k for k in oldbinding if oldbinding[k]!=binding[k]}=={'unqualified_count','selection_policy'};assert binding['unqualified_count']==34 and metadata['production_library_byte_identical'];assert metadata['focused_PASS_reruns']==0
assert details['GUI']==0 and details['LOADED'] is False and details['historical_PASS_reruns']==0
assert pin(L/'r478-native-mode-menu-thumb-handoff.json')['sha256']=='E4F316DBA83EAACD54B2A94A6109D41805F80387246A47660FEE6991EE740C56'
assert pin(P/'target/r408-source-selector/i686-pc-windows-msvc/debug/LiToModern.exe')['sha256']=='58F4B73E2E975BDB63153643A88ED2F364778D90FDF3B190C03A730D0157CCB3'
result=dict(state='PASS_R479_EXACT38_INVENTORY_FOUR_COMPLETE_NEW_MAPS_SOURCE_MAIN_PE_FIXED_LIBRARY_CPU_AND_FAILURE_CONTRACT',source_pairs=136,new_complete_maps=4,qualified_preview_maps=6,remaining_unqualified_maps=34,scene_pixel_pairs=4,original_inventory_sectors=305,known_source_decoder_rejections=2,GUI=0,LOADED=False,player=player['executable'],details=pin(L/'r479-map-batch-details.json'),verifier=pin(Path(__file__)))
(L/'r479-independent-contract.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8');print(json.dumps(result,ensure_ascii=False))
