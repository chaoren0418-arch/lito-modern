from r491_common import *
import re,struct
cache={};checks=0
def check(path,sha,size=None):
 global checks
 p=Path(path);p=p if p.is_absolute()else P/p;k=str(p.resolve()).casefold()
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
names=['r492-source-evidence.json','r492-source-findings.json','r492-preparation-history.json','r492-entry-source-contract.json','r492-player-candidate.json','r492-event-input.json','r492-consumer-candidate.json','r492-consumer-exit.json','r492-event-details.json','r492-source-scope.json','r492-production-source-snapshot.json']
for n in names:walk(read(L/n))
old=read(L/'r491-production-source-snapshot.json');assert len(old['source_pins'])==605;fixed={Path(x['path']).name:x for x in old['source_pins']}
for q in old['source_pins']:
 check(q['snapshot'],q['sha256'],q['bytes'])
 if Path(q['path']).name!='native_city_runtime.rs':check(q['path'],q['sha256'],q['bytes'])
runtime=Path(fixed['native_city_runtime.rs']['path']).read_text(encoding='utf-8');before=Path(fixed['native_city_runtime.rs']['snapshot']).read_text(encoding='utf-8');module=(P/'src/modern_lito/native_source_connection.rs').read_text(encoding='utf-8')
assert before[before.index('fn worker('):before.index('impl Runtime {')]==runtime[runtime.index('fn worker('):runtime.index('impl Runtime {')]
assert before[before.index(' pub fn poll('):]==runtime[runtime.index(' pub fn poll('):]
apply=runtime[runtime.index(' fn apply_source_packet('):runtime.index(' /// A real explicit list-open boundary')];consume=before[before.index(' pub fn consume_source_packet('):before.index(' /// A real explicit list-open boundary')];assert apply.replace(' fn apply_source_packet',' pub fn consume_source_packet')==consume
assert before[before.index(' /// A real explicit list-open boundary'):before.index(' pub fn menu(&self)')]==runtime[runtime.index(' /// A real explicit list-open boundary'):runtime.index(' pub fn menu(&self)')]
assert 'pub fn begin_source_connection_candidate(&mut self)->Result<SourceEpoch>'in runtime and 'self.source_scope.prepare_begin()?'in runtime
boundary=runtime[runtime.index(' fn commit_source_boundary('):runtime.index(' pub fn consume_source_event(')];ordered=['self.next_generation()?','self.invalidate()','menu.clear_items()','self.source_flow=SourceFlow::default()','self.source_scope=next'];idx=[boundary.index(x)for x in ordered];assert idx==sorted(idx)
entry=runtime[runtime.index(' pub fn consume_source_event('):runtime.index(' /// Shared command route')];assert entry.index('self.source_scope.admit(event)?')<entry.index('self.apply_source_packet(event.body())')
assert 'ensure!(!self.source_scope.bound()'in runtime and 'if self.source_scope.bound(){self.source_scope.retire();self.source_flow=SourceFlow::default();}'in runtime
assert 'Arc::ptr_eq(&self.owner,&epoch.owner)&&self.active==Some(epoch.serial)'in module and 'self.next.checked_add(1)'in module and 'next.active=None;'in module and 'next.bound=true;'in module
assert 'pub const MAX_DECODED_BODY:usize=155;'in module and module.index('body.len()<=MAX_DECODED_BODY')<module.index('Snapshot::decode(body)?')<module.index('body:Arc::from(body)')and 'body.len()==7'in module
assert 'pub struct Epoch {owner:Arc<()>,serial:u64}'in module and 'pub struct DecodedPacket {epoch:Epoch,body:Arc<[u8]>}'in module and 'set_phase'not in module and 'Context{phase:'not in module
e=read(L/'r492-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');A=P/'target/r492-source-connection-study/authority'
for f in e['fragments']:
 if f['path'].endswith('-through-next-proc.asm'):
  m=re.search('^'+f['name']+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+end.start()]
t=lambda n:(A/(n+'-through-next-proc.asm')).read_text(encoding='utf-8')
findings=read(L/'r492-source-findings.json');roster=t('sub_4A7550');assert 'lea     esi, ds:62FAA0h[eax*4]'in roster and 'lea     eax, [esi+5Ch]'in roster and '"sscchhhcccccccccddddddddddcdddddc"'in roster
assert len(findings['roster']['field_map'])==33 and findings['roster']['field_map'][26]==dict(index=26,format='c',destination='5C')and findings['roster']['field_map'][32]['destination']=='UNPAIRED_STACK_ARGUMENT_OPEN'
select=t('sub_4928C0');assert 'mov     dl, [ebx+5Ch]\nmov     byte_64AC29, dl'in select and 'mov     cl, [ebx+2Dh]\nmov     byte_64AC26, cl'in select and 'mov     edi, offset dword_64ABF8'in select
actor=t('sub_4A7F60');assert 'mov     [ebp+var_104], eax'in actor and 'push    offset dword_64ABF8'in actor and 'mov     ecx, 23h'in actor and 'mov     edi, offset dword_64ABF4\nrep movsd'in actor and 'cmp     ecx, 9F0Ah'in actor
cmp=t('sub_4D3A80');assert 'byte_834A60[eax]'in cmp and 'byte_834960[edi]'in cmp and 'byte_834960[eax]'in cmp
name=t('sub_4A9550');assert 'push    offset dword_64ABF8'in name and 'dword_64AB70'not in name
dispatch=t('sub_4B2970');assert 'jumptable 004B2A0C case 203'in dispatch and 'call    sub_4A7550'in dispatch and 'jumptable 004B2A0C case 205'in dispatch and 'call    sub_4A7F60'in dispatch
assert len(e['dispatch_direct_named_writes'])==25 and len(e['fragments'])==39
for w in e['dispatch_direct_named_writes']:assert w['instruction']in t(w['procedure'])
assert 'mov     dword_64BBD4, 3'in t('sub_46BA10')and 'call    sub_46BA10'in t('sub_4A7850')and 'cmp     eax, 5'in t('sub_4AA0A0')and 'push    offset dword_64ABF8'in t('sub_4AA0A0')and 'mov     dword_64BBD4, 4'in t('sub_4AA0A0')
profile=(P/'src/modern_lito/profile.rs').read_text(encoding='utf-8');session=(P/'src/modern_lito/session.rs').read_text(encoding='utf-8');transport=(P/'src/modern_lito/transport.rs').read_text(encoding='utf-8');assert 'pub enum GameplayProtocol {\n    LegacyV37,\n    ModernV47,\n}'in profile
assert 'application::read_response_selected('in transport and 'if response.request_id != request_id'in transport and 'fn retire_active(&mut self)'in session and 'fn disconnect(&mut self, closing: bool)'in session and 'self.retire_active();'in session and 'self.install_connection('in session
assert 'consume_source_event'not in session and 'SourceFlow'not in session and 'source_connection_candidate'not in session and findings['normal_wire_connected']is False
player=read(L/'r492-player-candidate.json');consumer=read(L/'r492-consumer-candidate.json');assert player['candidate_executions']==0 and len(consumer['source_pins'])==180 and consumer['production_library']==player['production_library']
assert Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('R492','R491').replace('r492','r491')==Path(read(L/'r491-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8')and player['binding']==read(L/'r491-player-candidate.json')['binding']and player['skin_binding']==read(L/'r491-player-candidate.json')['skin_binding']
b=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert struct.unpack_from('<H',b,pe+4)[0]==332 and struct.unpack_from('<H',b,pe+24+68)[0]==2
d=read(L/'r492-event-details.json');assert d['flow_cases']==len(d['trace'])==10 and all(x['passed']for x in d['trace'])and d['candidate_epochs']==[1,2,3,4]and (d['actual_Runtime_owners'],d['actual_active_563_loads'],d['actual_completed_506_loads'],d['frames_published'])==(2,3,1,1)
for k in ['foreign_same_serial_rejected','late206_and213_rejected','raw_bypass_after_disconnect_denied','current_epoch_wrong_id_atomic','old_frame_retired','reconnect_cancel','disconnect_cancel','close_cancel','both_workers_retired']:assert d[k]
assert (d['GUI'],d['LOADED'],d['complete_modes'],d['historical_PASS_batches_rerun'],d['normal_wire_connected'],d['phase6_callback_connected'])==(0,False,16,0,False,False)
consumer_rs=(P/'tools/r492-source-connection-consumer.rs').read_text(encoding='utf-8');assert 'Context{phase:'not in consumer_rs and 'rt.apply_native_state_packet'not in consumer_rs and 'set_phase'not in consumer_rs
ex=read(L/'r492-consumer-exit.json');assert ex['exit_code']==0 and ex['executions']==1 and 'R492 focused PASS:10candidateingressflows'in Path(ex['stdout']['path']).read_text(encoding='utf-8-sig')
build=[__import__('json').loads(x)for x in (L/'r492a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [x['success']for x in build if x['reason']=='build-finished']==[True];paths=[f for x in build if x['reason']=='compiler-artifact'and x['target']['name']=='lito_modern'for f in x['filenames']if f.endswith('.rlib')];assert len(paths)==1;check(paths[0],player['production_library']['sha256'],player['production_library']['bytes'])
scope=read(L/'r492-source-scope.json');assert (scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(606,467,12,127)
out=dict(state='R492_INDEPENDENT_NATIVE_AUTHORITY_GAPS_BOUND_CANDIDATE_EPOCH_FRAME_WORKER_SOURCE_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=180,all_production_source_snapshots=606,source_fragments=39,dispatch_named_direct_sites=25,source_difference=pin(L/'r492-source-difference.patch'),changed_from_fixed_R491=scope['changed_from_fixed_R491'],new_module='native_source_connection.rs',candidate_event_flows=10,actual_Runtime_owners=2,actual_active_563_loads=3,actual_completed_506_loads=1,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],production_library_builds=1,consumer_compile_failures=0,player_candidate_executions=0,normal_wire_connected=False,actor_class_dispatch_producers_connected=False,phase6_callback_connected=False,request_response_success_association_closed=False,Session_TLS_protocol_profile_unchanged=True,sourceflow_worker_window_menu_cache_decoder_unchanged=True,complete_modes=16,GUI=0,LOADED=False)
save(L/'r492-independent-contract.json',out);save(L/'r492-candidate.json',consumer|dict(validation='PASS one newbounddecodedcandidate lifecycleconsumer;normalNativewire/actorclassdispatch/phase6OPEN;GUI0/player0',independent_contract=pin(L/'r492-independent-contract.json')));print('R492 independentPASS',checks,'pins',len(cache),'unique;180pairs606production39ASM25dispatchsites;10flows2owners3cancels1frame;normalNativewireOPEN')
