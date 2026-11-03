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
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(p)
names=['r487-source-evidence.json','r487-scroll-base-source-evidence.json','r487-repeat-callback-source-evidence.json','r487-preparation-failure.json','r487-source-findings.json','r487-event-input.json','r487-entry-source-contract.json','r487-player-candidate.json','r487-consumer-candidate.json','r487-consumer-exit.json','r487-event-details.json','r487-production-source-snapshot.json','r487-source-scope.json']
for n in names:walk(read(L/n))
source=read(L/'r487-production-source-snapshot.json');assert len(source['source_pins'])==602 and not source['full_test_suite_executed']and not source['library_test_compile_performed_this_revision']
old=read(L/'r486-final-test-compile.json');changes=[];diffs=[]
for s in old['source_pins']:
 check(s['snapshot'],s['sha256'],s['bytes'])
 p=Path(s['path']);before=Path(s['snapshot']).read_text(encoding='utf-8');after=p.read_text(encoding='utf-8')
 if pin(p)['sha256']!=s['sha256']:
  changes.append(p.name);diffs.append(''.join(difflib.unified_diff(before.splitlines(True),after.splitlines(True),fromfile=s['snapshot'],tofile=s['path'])))
assert set(changes)=={'native_mode_menu.rs','native_city_runtime.rs','native_city_window.rs'}
oldc=read(L/'r486-candidate.json');assert len(oldc['source_pins'])==164
for s in oldc['source_pins']:check(s['snapshot'],s['sha256'],s['bytes'])
for key in ['executable','production_library','candidate_main','original_main']:walk(read(L/'r486-player-candidate.json')[key])
runtime=next(s for s in old['source_pins']if Path(s['path']).name=='native_city_runtime.rs');before=Path(runtime['snapshot']).read_text(encoding='utf-8');after=Path(runtime['path']).read_text(encoding='utf-8');assert before.replace('fn invalidate(&mut self){','fn invalidate(&mut self){if let Some(m)=self.menu.as_mut(){m.cancel_press();}')==after
menu=next(s for s in old['source_pins']if Path(s['path']).name=='native_mode_menu.rs');before=Path(menu['snapshot']).read_text(encoding='utf-8');after=Path(menu['path']).read_text(encoding='utf-8');assert before.split('#[derive(Clone,Debug)]')[0]==after.split('#[derive(Clone,Debug)]')[0]
assert 'repeat_due=now_ms.checked_add(700)'in after and 'repeat_due=now_ms.checked_add(125)'in after and 'if now_ms<due{return false;}'in after and 'self.cancel_press();if pressed.is_none()'in after
window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8');assert 'clock:Instant'in window and 'clock:Instant::now()'in window and 'WM_CAPTURECHANGED|WM_KILLFOCUS=>{s.menu_mut(Menu::cancel_press);'in window and 'm.repeat_at(s.clock.elapsed()'in window and 'm.pointer_move_at(hit,s.clock.elapsed()'in window and 'm.down_at(hit,s.clock.elapsed()'in window
assert 'unsafe fn back(hwnd:HWND,s:&State){s.menu_mut(Menu::cancel_press);let _=ReleaseCapture();'in window
asm=Path(read(L/'r487-source-evidence.json')['original_disassembly']['path']).read_text(encoding='utf-8');fragments=[]
for n in ['r487-source-evidence.json','r487-scroll-base-source-evidence.json','r487-repeat-callback-source-evidence.json']:fragments.extend(read(L/n)['fragments'])
assert len(fragments)==46
for f in fragments:
 p=Path(f['path'])
 if '-through-next-proc.asm'in p.name:
  name=p.name.replace('-through-next-proc.asm','');start=re.search('^'+name+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[start.end():],re.M);assert p.read_text(encoding='utf-8')==asm[start.start():start.end()+end.start()]
authority=P/'target/r487-menu-source-study/authority';text=lambda name:(authority/(name+'-through-next-proc.asm')).read_text(encoding='utf-8')
run=text('fn_ClientApplication_Run');assert run.count('mov     byte_64AF9F, 1')==2 and 'aLteventpsvr' in run and 'aLteventcsvr'in run and 'lea     esi, [edi+1]'in run
refs=read(L/'r487-source-evidence.json')['flag64AF9F_xrefs'];assert len(refs)==15 and sum('mov     byte_64AF9F, 1'in x['text']for x in refs)==2
assert 'mov     ecx, 2BCh'in text('sub_414BD0')and 'mov     al, [ecx+2Ch]'in text('sub_414BD0')and 'call    sub_4D65F0'in text('sub_414BD0')
assert 'call    dword ptr [eax+40h]'in text('sub_41AA00')and 'mov     ecx, 7Dh'in text('sub_41AA00')
assert 'call    sub_4D6870'in text('sub_414C00')and 'call    dword ptr [eax+40h]'in text('sub_414C00')
assert 'jmp     dword ptr [eax+24h]'in text('sub_414C40')and 'dec     ecx'in text('sub_474350')and 'inc     ecx'in text('sub_474360')
assert 'sub     ecx, 0Ah'in text('sub_474440')and 'push    2Ah'in text('sub_477AB0')and 'mov     edx, 236h'in text('sub_477AB0')and 'mov     ecx, 16Dh'in text('sub_477AB0')
spec=read(L/'r487-event-input.json');d=read(L/'r487-event-details.json');assert len(spec['cases'])==len(d['traces'])==d['cases']==12 and spec['events']==d['events']==53
for expected,actual in zip(spec['cases'],d['traces']):
 assert expected['name']==actual['name']and len(expected['events'])==len(actual['actual_events'])
 for e,a in zip(expected['events'],actual['actual_events']):assert a['input']==e and a['actual']==e['expected']and a['matches_expected']
assert d['scaled_pointer_sequences']==3 and len(d['lifecycle'])==3 and d['lifecycle'][0]['no_late_repeat']and d['lifecycle'][1]['no_late_repeat']and d['lifecycle'][1]['selected_identity']==[3,563]and d['lifecycle'][1]['no_frame']and d['actual_active_Back_cancel']and d['actual_worker_retired']and d['no_preview_frame_published']
assert(d['historical_PASS_batches_rerun'],d['GUI'],d['LOADED'],d['complete_modes'])==(0,0,False,16)
player=read(L/'r487-player-candidate.json');walk(player);assert player['candidate_executions']==0 and player['only_executed_candidate_flag']is None and not player['native_GUI_executed']and player['binding']==read(L/'r486-player-candidate.json')['binding']
newmain=Path(player['candidate_main']['path']).read_text(encoding='utf-8');oldmain=Path(read(L/'r486-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8');assert newmain.replace('R487','R486').replace('r487','r486')==oldmain
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
consumer=read(L/'r487-consumer-candidate.json');assert len(consumer['source_pins'])==166 and consumer['production_library']==player['production_library'];exit=read(L/'r487-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and 'R487 focused PASS:12cases/53fixed events'in Path(exit['stdout']['path']).read_text(encoding='utf-8-sig')
assert pin(L/'r486-bounded-city563-handoff.json')['sha256']=='990A6A0042F20C5D43D2A259148BE15C446CEB16D4C9C99959501839D1EFEF5D'
scope=read(L/'r487-source-scope.json');assert(scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(602,467,12,123)
diffpath=L/'r487-source-difference.patch';diffpath.write_text('\n'.join(diffs),encoding='utf-8')
out=dict(state='R487_INDEPENDENT_ASM_TOKEN_CALLBACK_SOURCE_FIXED_EVENT_AND_LIFECYCLE_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=166,all_production_source_snapshots=602,source_files=602,changed_from_fixed_R486=changes,source_difference=pin(diffpath),source_procedure_and_table_fragments=46,named_flag_write_points=2,actual_fixed_event_cases=12,actual_fixed_events=53,scaled_pointer_sequences=3,actual_worker_lifecycle_sequences=3,actual_worker_retired=True,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],player_candidate_executions=0,old40_plans_cache_decoder_preserved=True,thumb_track_geometry_unresolved=True,full_test_suite_executed=False,complete_modes=16,GUI=0,LOADED=False)
save(L/'r487-independent-contract.json',out);save(L/'r487-candidate.json',consumer|dict(validation='PASS one finite source-derived arrow event/lifecycle consumer;GUI candidate not executed',independent_contract=pin(L/'r487-independent-contract.json')));print('R487 independent PASS:',checks,'pins',len(cache),'unique files;166consumer/602production pairs;46source fragments;53events;GUI0/player executions0')
