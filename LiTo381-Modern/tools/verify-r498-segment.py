from r491_common import *
import difflib,struct,re
from r495_source_oracle import source_cipher,counter_add_adc
cache={};instances=0
def check(q):
 global instances
 instances+=1;p=q['path']
 if p not in cache:cache[p]=pin(p)
 assert cache[p]['bytes']==q['bytes'] and cache[p]['sha256']==q['sha256'],p
def walk(v):
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():check(v)
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
names=['r498-source-study.json','r498-character-resource-mapping.json','r498-archive-inventory.json','r498-native-menu-skin-binding.json','r498-entry-source-contract.json','r498-source-oracle.json','r498-independent-source-oracle.json','r498-production-source-snapshot.json','r498-source-scope.json','r498-player-candidate.json','r498-consumer-candidate.json','r498-consumer-exit.json','r498-event-details.json']
for name in names:walk(read(L/name))
source=read(L/'r498-source-study.json');asm=Path(source['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(source['analysis_PE']['path']).read_bytes()
for q in source['fragments']:
 if 'name'in q:
  match=re.search('^'+q['name']+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[match.end():],re.M);assert Path(q['path']).read_text(encoding='utf-8')==asm[match.start():match.end()+end.start()];assert q['line_start']==asm[:match.start()].count('\n')+1
for q in source['tables']:
 va=int(q['va'],16);raw=Path(q['raw']['path']).read_bytes();assert raw==pe[va-0x400000:va-0x400000+len(raw)]
mapping=read(L/'r498-character-resource-mapping.json');idx=Path(mapping['LTsurf_index']['path']).read_bytes();pak=Path(mapping['LTsurf_pak']['path']).read_bytes();table={n.split(b'\0')[0].decode():(i,o,z)for i,(o,n,z)in enumerate(struct.iter_unpack('<I20sI',idx[4:]))}
for slot in mapping['slots']:
 va=int(slot['table_va'],16);row=struct.unpack('<15I',pe[va-0x400000:va-0x400000+60]);assert (row[0],row[1],row[2],row[3],row[4],row[5])==(3,79,slot['x'],0,0x4934A0,slot['slot']);assert slot['x']==19+157*slot['slot']
for t in mapping['animation_tables']:
 raw=Path(t['raw']['path']).read_bytes();va=int(t['table_va'],16);assert raw==pe[va-0x400000:va-0x400000+96]
 for i,s in enumerate(t['states']):assert tuple(s.values())==struct.unpack('<6i',raw[i*24:i*24+24])
for m in mapping['members']:
 i,o,z=table[m['name']];raw=pak[o:o+z];assert(i,o,z)==(m['index_row'],m['offset'],m['container_bytes']);assert raw==Path(m['raw']['path']).read_bytes()
 if 'rgba_reference'in m:
  w,h,flag,key=struct.unpack('<hhHH',raw[:8]);assert(w,h,flag)==(51,23,1);rgba=bytearray()
  for(v,)in struct.iter_unpack('<H',raw[8:]):rgba.extend(bytes(4)if v==key else bytes(((c<<3)|(c>>2))for c in [(v>>10)&31,(v>>5)&31,v&31])+bytes([255]))
  assert bytes(rgba)==Path(m['rgba_reference']['path']).read_bytes()
assert all(n not in table for n in mapping['missing_members']);assert '79.img'in mapping['missing_members']and'80.img'in mapping['missing_members'];assert len([n for n in mapping['missing_members']if n.endswith('.png')])==803
oracle=read(L/'r498-source-oracle.json');counter=oracle['seed'];wire=bytearray()
for i,body in enumerate(oracle['setup_bodies']):
 raw=bytes(body)
 if i==0:cipher=raw
 else:cipher,_=source_cipher(raw,counter,pe);counter=counter_add_adc(counter,len(raw))
 wire+=(len(raw)+2).to_bytes(2,'little')+cipher
assert list(wire)==oracle['setup_wire']and counter==oracle['receive_after']
raw=bytes([3,4]);cipher,_=source_cipher(raw,counter,pe);assert list((len(raw)+2).to_bytes(2,'little')+cipher)==oracle['replacement_wire']
production=read(L/'r498-production-source-snapshot.json');old=read(L/'r497-production-source-snapshot.json');oldby={q['path']:q for q in old['source_pins']};newby={q['path']:q for q in production['source_pins']};assert len(newby)==612
delta=[];diffs=[]
for q in old['source_pins']:
 check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
 if newby[q['path']]['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
newfile=P/'src/modern_lito/native_character_selection.rs';diffs.append(''.join(difflib.unified_diff([],newfile.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(newfile))))
assert '\n'.join(diffs)==Path(production['source_difference']['path']).read_text(encoding='utf-8');assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_city_presentation.rs','native_mode_menu_skin.rs'}
for q in production['source_pins']:check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
candidate=read(L/'r498-consumer-candidate.json');player=read(L/'r498-player-candidate.json');assert len(candidate['source_pins'])==198 and candidate['production_library']==player['production_library'];assert player['candidate_executions']==0
for q in candidate['source_pins']:check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
assert read(L/'r498-consumer-exit.json')['exit_code']==0;tests=read(L/'r498-event-details.json');assert tests['flow_cases']==11 and tests['class_variant_mappings']==65536 and tests['source_button_boundary_points']==16 and tests['resource_images_verified']==4 and tests['historical_PASS_reruns']==0 and tests['GUI']==0
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');win=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8')
for term in ['character_scene.key().active','control.scene_revision==self.character_scene.key().scene_revision','submit.selection_revision==key.selection_revision','self.character_scene.retire()?','self.character_scene.enter()']:assert term in rt
for term in ['r.choose_source_character_control','BN_DOUBLECLICKED','BS_NOTIFY','r.handle_source_character_submit','WM_DRAWITEM','draw_character_button','return_source_character_scene()','r.enter_source_character_scene()','r.leave_source_character_scene()']:assert term in win
assert 'r.handle_source_character_control(control)'not in win
save(L/'r498-candidate.json',dict(state='R498_FIXED_LIBRARY_PLAYER_AND_NEW_CONSUMER_SAME_SOURCE',source_pins=candidate['source_pins'],production_library=player['production_library'],player=player['executable'],consumer=dict(path=candidate['path'],bytes=candidate['bytes'],sha256=candidate['sha256']),source_pairs=198,GUI=0,LOADED=False))
save(L/'r498-independent-contract.json',dict(state='R498_FIRST_INDEPENDENT_SOURCE_PIN_DIFF_RESOURCE_RUNTIME_VERIFICATION_PASS',pin_instances=instances,unique_paths=len(cache),source_pairs=198,production_files=612,source_difference=production['source_difference'],changed=delta,added=['native_character_selection.rs'],source_fragments=len(source['fragments']),animation_tables=12,class_variant_mappings=65536,button_boundaries=16,resource_images=4,flows=11,player_executions=0,consumer_executions=1,historical_PASS_reruns=0,GUI=0,LOADED=False,verified_json=[pin(L/n)for n in names]))
print('R498 independentPASS',instances,'pins',len(cache),'unique;198pairs612source/11flows/65536mapping/16boundaries/4IMG')
