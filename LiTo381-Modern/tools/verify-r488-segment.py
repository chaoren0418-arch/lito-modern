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
names=['r488-source-evidence.json','r488-additional-alias-source-evidence.json','r488-source-findings.json','r488-event-input.json','r488-entry-source-contract.json','r488-player-candidate.json','r488-consumer-candidate.json','r488-consumer-exit.json','r488-event-details.json','r488-production-source-snapshot.json','r488-source-scope.json']
for n in names:walk(read(L/n))
source=read(L/'r488-production-source-snapshot.json');assert len(source['source_pins'])==603 and not source['full_test_suite_executed']and not source['library_test_compile_performed_this_revision']
old=read(L/'r487-production-source-snapshot.json');changes=[];diffs=[]
for s in old['source_pins']:
 check(s['snapshot'],s['sha256'],s['bytes'])
 path=Path(s['path'])
 if pin(path)['sha256']!=s['sha256']:
  changes.append(path.name);diffs.append(''.join(difflib.unified_diff(Path(s['snapshot']).read_text(encoding='utf-8').splitlines(True),path.read_text(encoding='utf-8').splitlines(True),fromfile=s['snapshot'],tofile=s['path'])))
assert changes==['native_city_runtime.rs']
module=P/'src/modern_lito/native_mode_selection.rs';assert str(module)not in {s['path']for s in old['source_pins']}
diffs.append(''.join(difflib.unified_diff([],module.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(module))))
runtime=next(s for s in old['source_pins']if Path(s['path']).name=='native_city_runtime.rs');before=Path(runtime['snapshot']).read_text(encoding='utf-8');after=Path(runtime['path']).read_text(encoding='utf-8')
assert before[before.index('fn worker('):before.index('impl Runtime {')]==after[after.index('fn worker('):after.index('impl Runtime {')]
assert before[before.index(' pub fn menu(&self)'):]==after[after.index(' pub fn menu(&self)'):]
apply=after[after.index(' pub fn apply_native_mode_packet('):after.index(' /// Explicit fresh construction')]
assert 'self.native_mode.apply(packet,phase)'in apply and 'invalidate('not in apply and 'self.menu='not in apply
opening=after[after.index(' pub fn open_native_mode_menu('):after.index(' pub fn menu(&self)')]
assert 'self.native_mode.catalog_mode()?'in opening and 'self.next_generation()?;self.invalidate();self.selected=None;self.menu=Some(menu)'in opening
mode=module.read_text(encoding='utf-8');assert 'self.raw_mode=(previous&0xff00)|u16::from(packet[6]);'in mode and 'if self.raw_mode<=5'in mode
oldc=read(L/'r487-candidate.json');assert len(oldc['source_pins'])==166
for s in oldc['source_pins']:check(s['snapshot'],s['sha256'],s['bytes'])
for name,expected in [('r487-native-arrow-hold-player-flow-handoff.json','35A28DBD8715DC4B667F05154DBCFD28C007BE59D90F6A12C7DB05C7096EDF70'),('r486-bounded-city563-handoff.json','990A6A0042F20C5D43D2A259148BE15C446CEB16D4C9C99959501839D1EFEF5D')]:check(L/name,expected)
for revision in ['r487','r486']:
 player=read(L/(revision+'-player-candidate.json'))
 for key in ['executable','production_library','candidate_main','original_main']:walk(player[key])
evidence=read(L/'r488-source-evidence.json');extra=read(L/'r488-additional-alias-source-evidence.json');asm=Path(evidence['original_disassembly']['path']).read_text(encoding='utf-8');fragments=evidence['fragments']+extra['additional_fragments'];assert len(fragments)==39
for f in fragments:
 p=Path(f['path'])
 if '-through-next-proc.asm'in p.name:
  name=p.name.replace('-through-next-proc.asm','');start=re.search('^'+name+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[start.end():],re.M);assert p.read_text(encoding='utf-8')==asm[start.start():start.end()+end.start()]
authority=P/'target/r488-mode-source-study/authority';text=lambda name:(authority/(name+'-through-next-proc.asm')).read_text(encoding='utf-8')
dispatch=text('sub_4B2970');start=dispatch.index('loc_4B2F67:');next_label=re.search(r'^loc_[0-9A-F]+:',dispatch[start+1:],re.M);assert next_label;block=dispatch[start:start+1+next_label.start()];assert 'case 213'in block and 'call    sub_4A9E50'in block
handler=text('sub_4A9E50');assert 'cmp     eax, 6'in handler and 'mov     dword_64ABF4, eax'in handler and 'jnz     def_4A9EA4'in handler and 'push    offset word_64ACCA              ; jumptable 004A9EA4 case 2'in handler
tail=handler[handler.index('loc_4AA05B:'):handler.index('def_4A9EA4:')];assert 'add     esi, 6'in tail and 'push    offset dword_5025A8'in tail and 'call    sub_4A2A50'in tail and '46B7D0'not in tail and '450230'not in tail
assert 'dword_5025A8 dd 63h'in (authority/'dword_5025A8-context.asm').read_text(encoding='utf-8')
parser=text('sub_4A2A50');assert 'mov     [edx], al'in parser and 'mov     [eax], cx'in parser
for name in extra['all_named_group_reference_procedures']:assert (authority/(name+'-through-next-proc.asm')).is_file()
assert 'mov     [esi+28h], eax'in text('sub_450090')and 'mov     dword ptr [esi+4Ch], 0FFFFFFFFh'in text('sub_450090')and 'mov     [esi+28h], ebx'in text('sub_4645D0')
assert 'push    2Ah'in text('sub_477AB0')and 'sub     ecx, 0Ah'in text('sub_474440')and 'mov     byte ptr [ecx+14h], 1'in text('sub_41AA90')
assert evidence['direct_constant_group_child10_getter_sites']==[]
spec=read(L/'r488-event-input.json');d=read(L/'r488-event-details.json');assert len(spec['cases'])==len(d['traces'])==d['event_cases']==33
for expected,actual in zip(spec['cases'],d['traces']):assert actual['input']==expected and actual['actual']==expected['expected']and actual['matches_expected']
assert d['runtime_flow_cases']==len(d['runtime_flow'])==7 and all(x['passed']for x in d['runtime_flow'])and d['actual_active_load_observed']and d['actual_worker_retired']and d['frames_published']==0
assert(d['historical_PASS_batches_rerun'],d['GUI'],d['LOADED'],d['complete_modes'])==(0,0,False,16)
player=read(L/'r488-player-candidate.json');assert player['candidate_executions']==0 and not player['native_GUI_executed']and player['binding']==read(L/'r487-player-candidate.json')['binding']and player['skin_binding']==read(L/'r487-player-candidate.json')['skin_binding']
newmain=Path(player['candidate_main']['path']).read_text(encoding='utf-8');oldmain=Path(read(L/'r487-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8');assert newmain.replace('R488','R487').replace('r488','r487')==oldmain
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
consumer=read(L/'r488-consumer-candidate.json');assert len(consumer['source_pins'])==169 and consumer['production_library']==player['production_library'];exit=read(L/'r488-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and 'R488 focused PASS:33fixed source events/7runtime flow cases'in Path(exit['stdout']['path']).read_text(encoding='utf-8-sig')
build=[json.loads(x)for x in (L/'r488a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [b['success']for b in build if b['reason']=='build-finished']==[True]
artifact=[f for b in build if b['reason']=='compiler-artifact'and b['target']['name']=='lito_modern' for f in b['filenames']if f.endswith('.rlib')];assert len(artifact)==1;check(artifact[0],player['production_library']['sha256'],player['production_library']['bytes'])
scope=read(L/'r488-source-scope.json');assert(scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(603,467,12,124)
diff=L/'r488-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8')
out=dict(state='R488_INDEPENDENT_SOURCE_SCOPED_MODE_EVENTS_ACTUAL_RUNTIME_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=169,all_production_source_snapshots=603,changed_from_fixed_R487=changes,new_module='native_mode_selection.rs',source_difference=pin(diff),source_fragments=39,actual_fixed_event_cases=33,actual_runtime_flow_cases=7,actual_active_load_cancelled=True,actual_worker_retired=True,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],player_candidate_executions=0,old40_plans_cache_decoder_menu_window_preserved=True,live_network_dispatch_connected=False,full_snapshot_decoded=False,thumb_track_geometry_unresolved=True,full_test_suite_executed=False,complete_modes=16,GUI=0,LOADED=False)
save(L/'r488-independent-contract.json',out);save(L/'r488-candidate.json',consumer|dict(validation='PASS one finite scoped source-event/runtime-flow consumer;player candidate unexecuted',independent_contract=pin(L/'r488-independent-contract.json')));print('R488 independent PASS:',checks,'pins',len(cache),'unique;169consumer/603production pairs;39source fragments;33events/7runtime flows;GUI0/player0')
