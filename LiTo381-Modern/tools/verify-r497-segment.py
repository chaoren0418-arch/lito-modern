from r491_common import *
from r495_source_oracle import source_cipher,counter_add_adc
import re,struct
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==v['bytes'] and cache[p]['sha256']==v['sha256'],p
   if 'snapshot'in v:assert pin(v['snapshot'])['sha256']==v['sha256']
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
for n in ['source-evidence','source-oracle','native-menu-skin-binding','entry-source-contract','production-source-snapshot','source-scope','player-candidate','consumer-candidate','consumer-exit','event-details','preparation-history']:walk(read(L/('r497-'+n+'.json')))
old=read(L/'r496-production-source-snapshot.json');delta=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:delta.append(Path(q['path']).name)
assert delta==['native_city_runtime.rs','native_city_window.rs','native_mode_menu_skin.rs']
snapshot=read(L/'r497-production-source-snapshot.json');assert len(snapshot['source_pins'])==611
c=read(L/'r497-consumer-candidate.json');player=read(L/'r497-player-candidate.json');assert len(c['source_pins'])==195 and c['production_library']==player['production_library'] and player['candidate_executions']==0
exit=read(L/'r497-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and exit['historical_PASS_reruns']==0
history=read(L/'r497-preparation-history.json');assert history['consumer_test_failures']==1 and history['already_verified_flows']==4
O=P/'target/r497-ui-entry-study/attempt1';failure=read(O/'r497-consumer-exit.json');assert failure['exit_code']==101
assert len(read(O/'r497-event-partial.json'))==4 and 'left: Mode { mode: 3' in (O/'r497-consumer.stderr.log').read_text(encoding='utf-8')
first=read(O/'r497-consumer-candidate.json');assert first['production_library']==c['production_library']
for q in first['source_pins']:assert pin(q['snapshot'])['sha256']==q['sha256']
d=read(L/'r497-event-details.json');assert d['flow_cases']==10 and d['current_execution_new_flows']==6 and d['previous_failed_execution_verified_flows']==4 and d['consumer_executions']==2 and not d['firstfourassertion_batches_rerun'] and d['idle_snapshots_skipped']==550 and all(x['passed']for x in d['trace'])
assert all(d[k]for k in ['source6279_BGRA_matches_independent_pin','actual_skin_worker_loaded','same_Runtime_map_callback','same_named_roster_token_retired','pending_disable_and_retirement','wrong_id_receive_no_UI_refresh','cancel_return_scene_refresh'])
assert not any(d[k]for k in ['original_character_UI_complete','actual_GUI_or_HWND_or_WM_COMMAND_executed','actual_provider_or_socket_executed','normal_wire_connected','phase6_connected']) and d['GUI']==0 and d['complete_modes']==16
e=read(L/'r497-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(e['analysis_PE']['path']).read_bytes()
for f in e['fragments']:
 m=re.search('^'+f['name']+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()]
native=(P/'target/r497-ui-entry-study/sub_46BA10.asm').read_text(encoding='utf-8');assert re.search(r'push    2\nmov     ecx, 246h\npush    offset sub_46B7D0\npush    edx\npush    ecx\npush    1885h\npush    1887h\nmov     ecx, eax\ncall    sub_4764F0',native)
receiver=(P/'target/r497-ui-entry-study/sub_46B7D0.asm').read_text(encoding='utf-8');assert 'push    2\ncall    sub_41B410'in receiver and 'dword_64A968'in receiver
skin=read(L/'r497-native-menu-skin-binding.json');prior=read(L/'r478-native-menu-skin-binding.json');assert skin['members']==prior['members'] and skin['LTsurf_index']==prior['LTsurf_index'] and skin['LTsurf_pak']==prior['LTsurf_pak']
idx=Path(skin['LTsurf_index']['path']).read_bytes();pak=Path(skin['LTsurf_pak']['path']).read_bytes()
for q in e['entry_members']:
 off,name,size=list(struct.iter_unpack('<I20sI',idx[4:]))[q['index_row']];assert name.split(b'\0')[0]==f"{q['id']}.img".encode() and off==q['offset'] and size==q['bytes'] and pak[off:off+size]==Path(q['raw']['path']).read_bytes()
raw=Path(e['entry_members'][1]['raw']['path']).read_bytes();bgra=bytearray()
for (p,)in struct.iter_unpack('<H',raw[8:]):bgra.extend(tuple(((c<<3)|(c>>2))for c in [p&31,(p>>5)&31,(p>>10)&31])+(255,))
v=read(L/'r497-source-oracle.json');assert bytes(bgra)==Path(v['entry_bgra']['path']).read_bytes()
previous=read(L/'r496-source-oracle.json');assert v['setup_wire']==previous['setup_wire'] and v['mode206_wire']==previous['vectors']['mode206']['wire']
ctr=previous['vectors']['mode206']['receive_after']
for q in v['deltas']:
 cipher,_=source_cipher(bytes(q['plain']),ctr,pe);assert list(b'\x09\x00'+cipher)==q['wire'] and q['before']==ctr;ctr=counter_add_adc(ctr,7);assert q['after']==ctr
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8');gate=(P/'src/modern_lito/native_city_presentation.rs').read_text(encoding='utf-8');skin_src=(P/'src/modern_lito/native_mode_menu_skin.rs').read_text(encoding='utf-8')
assert 'if self.last.as_ref()==Some(&next){false}'in gate and 'self.source_roster_revision'in rt and 'pending:self.source_send.snapshot()'in rt
assert window.index('if !s.refresh_gate.borrow_mut().changed')<window.index('*s.character_controls.borrow_mut()')<window.index('label(c[TITLE]')
assert 'SOURCE_ENTRY:usize=17;const COUNT:usize=18'in window and 'i==SOURCE_ENTRY{usize::from(SOURCE_MENU_COMMAND_ID)}'in window and 'Runtime::source_menu_invoker_enabled'in window and 'paint_source_entry(hwnd,s,dc)'in window and 'SOURCE_ENTRY=>entry&&'in window
assert 'self.source_scope.active_serial().is_some()&&self.skin.is_some()&&self.source_flow.phase()==2' in rt and 'self.source_flow.mode().catalog_mode().is_ok()'in rt
assert 'table.get("6279.img")'in skin_src and 'CopyMode::Opaque'in skin_src and 'row["id"]==6279'in skin_src
proof=dict(state='R497_INDEPENDENT_NATIVE_COMBO_RESOURCE_VISIBLE_ENTRY_AND_REFRESH_GATE_FINITE_PASS',pin_instances=instances,unique_paths=len(cache),production_source_files=611,consumer_source_pairs=195,changed_from_R496=delta,new_source_files=['native_city_presentation.rs'],source_difference=snapshot['source_difference'],source_ASM_fragments=6,flow_cases=10,first_verified_flows=4,resume_new_flows=6,consumer_executions=2,consumer_test_failures=1,firstfourassertion_batches_rerun=False,idle_snapshots_skipped=550,player_executions=0,original_character_UI_complete=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r497-independent-contract.json',proof)
save(L/'r497-candidate.json',dict(state='R497_VISIBLE_SOURCE_MAP_ENTRY_RESOURCE_BADGE_REVISION_GATE_FINITE_CANDIDATE',source_pins=c['source_pins'],production_library=player['production_library'],player=player['executable'],consumer={k:c[k]for k in ['path','bytes','sha256']},source_scope=read(L/'r497-source-scope.json'),independent=proof,GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R497 independent PASS',instances,'pinrefs',len(cache),'unique;195pairs611source/10flows550idleskip;failure1preserved/resume6only')
