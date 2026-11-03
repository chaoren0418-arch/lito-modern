from pathlib import Path
import json,hashlib,re,struct,difflib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';cache={};checks=0
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):
 p=Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def check(p,sha,size=None):
 global checks
 p=Path(p);p=p if p.is_absolute()else P/p;k=str(p.resolve()).casefold()
 if k not in cache:cache[k]=pin(p)
 assert cache[k]['sha256']==sha and(size is None or cache[k]['bytes']==size),str(p);checks+=1
def walk(v):
 if isinstance(v,list):
  for x in v:walk(x)
 elif isinstance(v,dict):
  if {'path','sha256'}<=v.keys():check(v['path'],v['sha256'],v.get('bytes'))
  if {'snapshot','sha256'}<=v.keys():check(v['snapshot'],v['sha256'],v.get('bytes'))
  if {'path','current_sha256'}<=v.keys():check(v['path'],v['current_sha256'])
  for k,x in v.items():
   if k not in ['path','bytes','sha256','snapshot']:walk(x)
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
names=['r490-source-evidence.json','r490-source-findings.json','r490-stream-input.json','r490-entry-source-contract.json','r490-player-candidate.json','r490-consumer-build-failure.json','r490-consumer-candidate.json','r490-consumer-exit.json','r490-stream-details.json','r490-production-source-snapshot.json','r490-source-scope.json']
for n in names:walk(read(L/n))
source=read(L/'r490-production-source-snapshot.json');assert len(source['source_pins'])==605 and not source['full_test_suite_executed']and not source['library_test_compile_performed_this_revision']
old=read(L/'r489-production-source-snapshot.json');changes=[];diffs=[]
for s in old['source_pins']:
 check(s['snapshot'],s['sha256'],s['bytes']);path=Path(s['path'])
 if pin(path)['sha256']!=s['sha256']:
  changes.append(path.name);diffs.append(''.join(difflib.unified_diff(Path(s['snapshot']).read_text(encoding='utf-8').splitlines(True),path.read_text(encoding='utf-8').splitlines(True),fromfile=s['snapshot'],tofile=s['path'])))
assert set(changes)=={'native_city_runtime.rs','native_mode_menu.rs','native_mode_selection.rs'}
module=P/'src/modern_lito/native_mode_state_flow.rs';diffs.append(''.join(difflib.unified_diff([],module.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(module))))
runtime=next(s for s in old['source_pins']if Path(s['path']).name=='native_city_runtime.rs');before=Path(runtime['snapshot']).read_text(encoding='utf-8');after=Path(runtime['path']).read_text(encoding='utf-8')
assert before[before.index('fn worker('):before.index('impl Runtime {')]==after[after.index('fn worker('):after.index('impl Runtime {')]
assert before[before.index(' pub fn menu(&self)'):]==after[after.index(' pub fn menu(&self)'):]
consume=after[after.index(' pub fn consume_source_packet('):after.index(' /// A real explicit list-open boundary')];assert consume.index('self.source_flow.prepare_packet(packet)?')<consume.index('self.next_generation()?')<consume.index('menu.clear_items();')<consume.index('self.source_flow=prepared.next;')and 'self.selected=None'in consume and 'Command::Back'in consume
flow=module.read_text(encoding='utf-8');assert 'pub fn prepare_packet(&self,packet:&[u8])'in flow and 'Context{phase:self.phase,dispatch_word:self.dispatch_word}'in flow and 'next.phase=native_phase_after'in flow and 'pub fn set'not in flow
modepin=next(s for s in old['source_pins']if Path(s['path']).name=='native_mode_selection.rs');oldmode=Path(modepin['snapshot']).read_text(encoding='utf-8');mode=Path(modepin['path']).read_text(encoding='utf-8');assert oldmode.replace('#[path="native_mode_snapshot.rs"]pub mod snapshot;','#[path="native_mode_snapshot.rs"]pub mod snapshot;\n#[path="native_mode_state_flow.rs"]pub mod state_flow;')==mode
menupin=next(s for s in old['source_pins']if Path(s['path']).name=='native_mode_menu.rs');oldmenu=Path(menupin['snapshot']).read_text(encoding='utf-8');menu=Path(menupin['path']).read_text(encoding='utf-8');addition=' /// Source450230 clears existing list items/count/top. Modern also discards\n /// the now-invalid selection and held repeat/capture state.\n pub fn clear_items(&mut self){self.rows.clear();self.top=0;self.selected=None;self.cancel_press();}\n';assert oldmenu.replace(' pub fn key(&mut self,',addition+' pub fn key(&mut self,')==menu
oldc=read(L/'r489-candidate.json');assert len(oldc['source_pins'])==172
for s in oldc['source_pins']:check(s['snapshot'],s['sha256'],s['bytes'])
check(L/'r489-native-state-snapshot-player-flow-handoff.json','85E57FD652A030143AA2AD665AA8F1CEC48D359E91538D7E0B015EF874D42287')
for key in ['executable','production_library','candidate_main','original_main']:walk(read(L/'r489-player-candidate.json')[key])
evidence=read(L/'r490-source-evidence.json');asm=Path(evidence['original_disassembly']['path']).read_text(encoding='utf-8');assert len(evidence['fragments'])==12
for f in evidence['fragments']:
 p=Path(f['path'])
 if '-through-next-proc.asm'in p.name:
  name=p.name.replace('-through-next-proc.asm','');start=re.search('^'+name+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[start.end():],re.M);assert p.read_text(encoding='utf-8')==asm[start.start():start.end()+end.start()]
A=P/'target/r490-state-flow-study/authority';text=lambda name:(A/(name+'-through-next-proc.asm')).read_text(encoding='utf-8')
full=text('sub_4A8BA0');clear=text('sub_450230');builder=text('sub_46B7D0');control=text('sub_474280');assert 'mov     dword_64A968, eax'in builder and 'mov     [eax+8], ecx'in control
ordered=['mov     ecx, dword_64A950','mov     ecx, dword_64A94C','mov     ecx, dword_64A968','mov     ecx, dword_64A96C','push    offset word_64ACCA','mov     ecx, 2','call    sub_469CA0'];positions=[full.index(x)for x in ordered];assert positions==sorted(positions)
assert 'mov     [esi+2Ch], ebx'in clear and 'mov     [esi+28h], ebx'in clear and '+4Ch'not in clear and 'call    dword ptr [edx]'in clear
handler=text('sub_4A9E50');assert 'sub_450230'not in handler and 'cmp     eax, 6'in handler and 'mov     dword_64ABF4, eax'in handler
assert 'mov     dword_64AF98, 2'in text('sub_469CA0')and 'dword_64AF98 dd 0'in (A/'dword_64AF98-context.asm').read_text(encoding='utf-8')and 'dword_64BBD4 dd 0'in (A/'dword_64BBD4-context.asm').read_text(encoding='utf-8')
spec=read(L/'r490-stream-input.json');d=read(L/'r490-stream-details.json');assert spec['input_context_fields']==[] and d['transfers']==spec['expected_source_transitions']and len(d['transfers'])==5
assert d['flow_cases']==len(d['trace'])==11 and all(x['passed']for x in d['trace'])and d['actual_active_loads']==3 and d['automatic_206_generation_cancel']and d['Back_cancel']and d['close_cancel']and d['actual_worker_retired']and d['frames_published']==0 and d['inspection_isolated']
assert(d['persistent_input_phase_arguments'],d['persistent_input_dispatch_arguments'],d['historical_PASS_batches_rerun'],d['GUI'],d['LOADED'],d['complete_modes'])==(0,0,0,0,False,16)
consumer_text=(P/'tools/r490-state-flow-consumer.rs').read_text(encoding='utf-8');assert consumer_text.count('Context{phase:')==1 and 'rt.apply_native_state_packet'in consumer_text and 'source_flow().phase()'in consumer_text
player=read(L/'r490-player-candidate.json');assert player['candidate_executions']==0 and not player['native_GUI_executed']and player['binding']==read(L/'r489-player-candidate.json')['binding']and player['skin_binding']==read(L/'r489-player-candidate.json')['skin_binding']
assert Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('R490','R489').replace('r490','r489')==Path(read(L/'r489-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8')
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
consumer=read(L/'r490-consumer-candidate.json');assert len(consumer['source_pins'])==175 and consumer['production_library']==player['production_library'];exit=read(L/'r490-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and 'R490 focused PASS:11continuous Runtime flow cases/5applied source transfers/3actual active loads'in Path(exit['stdout']['path']).read_text(encoding='utf-8-sig')
build=[json.loads(x)for x in (L/'r490a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [b['success']for b in build if b['reason']=='build-finished']==[True];artifact=[f for b in build if b['reason']=='compiler-artifact'and b['target']['name']=='lito_modern'for f in b['filenames']if f.endswith('.rlib')];assert len(artifact)==1;check(artifact[0],player['production_library']['sha256'],player['production_library']['bytes'])
failure=read(L/'r490-consumer-build-failure.json');assert failure['executions']==0 and any('E0422'in Path(f['path']).read_text(encoding='utf-8-sig')for f in failure['files']if f['path'].endswith('stderr.log'))
scope=read(L/'r490-source-scope.json');assert(scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(605,467,12,126)
diff=L/'r490-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');out=dict(state='R490_INDEPENDENT_SOURCE_LIST_ALIAS_ORDER_PERSISTENT_RUNTIME_STREAM_CANCEL_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=175,all_production_source_snapshots=605,changed_from_fixed_R489=changes,new_module='native_mode_state_flow.rs',source_difference=pin(diff),source_fragments=12,actual_continuous_runtime_flow_cases=11,actual_applied_source_transfers=5,actual_active_loads=3,persistent_packet_phase_arguments=0,persistent_packet_dispatch_arguments=0,automatic_206_generation_cancel=True,Back_and_close_cancel=True,actual_worker_retired=True,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],consumer_build_attempts=2,consumer_compile_failures=1,production_library_builds=1,player_candidate_executions=0,old40_plans_cache_decoder_window_worker_preserved=True,inspection_separate=True,supported_persistent_phases=[0,2],phase6_callback_connected=False,dispatch_producers_connected=False,live_network_dispatch_connected=False,full_native_state_side_effects_reproduced=False,thumb_track_geometry_unresolved=True,full_test_suite_executed=False,complete_modes=16,GUI=0,LOADED=False)
save(L/'r490-independent-contract.json',out);save(L/'r490-candidate.json',consumer|dict(validation='PASS one finite actualRuntime persistent source stream;206 auto list/generation clear;no perpacket context;player unexecuted',independent_contract=pin(L/'r490-independent-contract.json')));print('R490 independent PASS:',checks,'pins',len(cache),'unique;175consumer/605production;12source fragments;11Runtime flows/5source transfers/3active loads;0context args;GUI0/player0')
