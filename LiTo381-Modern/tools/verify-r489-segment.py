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
names=['r489-source-evidence.json','r489-additional-source-evidence.json','r489-source-findings.json','r489-preparation-failure.json','r489-preflight-build.json','r489-event-input.json','r489-entry-source-contract.json','r489-player-candidate.json','r489-consumer-candidate.json','r489-consumer-exit.json','r489-event-details.json','r489-production-source-snapshot.json','r489-source-scope.json']
for n in names:walk(read(L/n))
source=read(L/'r489-production-source-snapshot.json');assert len(source['source_pins'])==604 and not source['full_test_suite_executed']and not source['library_test_compile_performed_this_revision']
old=read(L/'r488-production-source-snapshot.json');changes=[];diffs=[]
for s in old['source_pins']:
 check(s['snapshot'],s['sha256'],s['bytes']);path=Path(s['path'])
 if pin(path)['sha256']!=s['sha256']:
  changes.append(path.name);diffs.append(''.join(difflib.unified_diff(Path(s['snapshot']).read_text(encoding='utf-8').splitlines(True),path.read_text(encoding='utf-8').splitlines(True),fromfile=s['snapshot'],tofile=s['path'])))
assert set(changes)=={'native_city_runtime.rs','native_mode_selection.rs'}
module=P/'src/modern_lito/native_mode_snapshot.rs';assert str(module)not in {s['path']for s in old['source_pins']};diffs.append(''.join(difflib.unified_diff([],module.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(module))))
runtime=next(s for s in old['source_pins']if Path(s['path']).name=='native_city_runtime.rs');before=Path(runtime['snapshot']).read_text(encoding='utf-8');after=Path(runtime['path']).read_text(encoding='utf-8')
assert before[before.index('fn worker('):before.index('impl Runtime {')]==after[after.index('fn worker('):after.index('impl Runtime {')]
assert before[before.index(' pub fn menu(&self)'):]==after[after.index(' pub fn menu(&self)'):]
newapi=after[after.index(' pub fn apply_native_state_packet('):after.index(' /// Offline inspection only:')];assert newapi.index('Snapshot::decode(packet)?')<newapi.index('self.native_mode.apply_snapshot')<newapi.index('self.native_snapshot=Some')and 'invalidate('not in newapi
modepin=next(s for s in old['source_pins']if Path(s['path']).name=='native_mode_selection.rs');oldmode=Path(modepin['snapshot']).read_text(encoding='utf-8');mode=Path(modepin['path']).read_text(encoding='utf-8');assert oldmode[oldmode.index(' pub fn apply(&mut self,packet:'):]==mode[mode.index(' pub fn apply(&mut self,packet:'):]
assert '(context.phase as i32)>=3'in mode and 'self.raw_mode=(previous&0xff00)|u16::from(s.mode_byte());'in mode and 'self.status="請選擇場地".into();'in after
preflight=read(L/'r489-preflight-build.json');assert preflight['executions']==0 and len(preflight['sources'])==604
pre_changes=[]
for s in preflight['sources']:
 if pin(Path(s['source_origin']))['sha256']!=s['sha256']:pre_changes.append(Path(s['source_origin']).name)
assert pre_changes==['native_mode_selection.rs'];premode=next(s for s in preflight['sources']if Path(s['source_origin']).name=='native_mode_selection.rs');assert Path(premode['path']).read_text(encoding='utf-8').replace('context.dispatch_word==4&&context.phase>=3','context.dispatch_word==4&&(context.phase as i32)>=3')==mode
oldc=read(L/'r488-candidate.json');assert len(oldc['source_pins'])==169
for s in oldc['source_pins']:check(s['snapshot'],s['sha256'],s['bytes'])
check(L/'r488-native-mode-event-player-flow-handoff.json','D7F516DC4B3C51D27FC957F9F39A3AD1BCDA56BE28849AC702ECE64FAC7E8861')
for key in ['executable','production_library','candidate_main','original_main']:walk(read(L/'r488-player-candidate.json')[key])
evidence=read(L/'r489-source-evidence.json');extra=read(L/'r489-additional-source-evidence.json');asm=Path(evidence['original_disassembly']['path']).read_text(encoding='utf-8');fragments=evidence['fragments']+extra['additional_fragments'];assert len(fragments)==24
for f in fragments:
 p=Path(f['path'])
 if '-through-next-proc.asm'in p.name:
  name=p.name.replace('-through-next-proc.asm','');start=re.search('^'+name+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[start.end():],re.M);assert p.read_text(encoding='utf-8')==asm[start.start():start.end()+end.start()]
A=P/'target/r489-mode-snapshot-study/authority';text=lambda name:(A/(name+'-through-next-proc.asm')).read_text(encoding='utf-8')
# Reconstruct varargs consumption independently from the actual push list and
# exact data-section format;never infer widths from IDA symbol storage types.
def paired(block,fmt,resolved):
 args=[l.split('push    ',1)[1].split(';',1)[0].strip()for l in block.splitlines()if l.startswith('push    ')][:-2][::-1];out=[]
 for token in fmt:
  cap=None
  if token=='s':literal=args.pop(0);cap=int(literal[:-1],16)if literal.endswith('h')else int(literal)
  dest=args.pop(0);dest=dest.replace('offset ','');dest=resolved.get(dest,dest);out.append([token,dest,cap])
 assert not args,args
 return out
full=text('sub_4A8BA0');start=full.index('lea     ecx, [ebp+var_1]');end=full.index('call    sub_4A2A50',start);fmt=re.search(r"db '([^']+)'",(A/'aCdscshhchccccc-context.asm').read_text(encoding='utf-8'))[1];assert paired(full[start:end],fmt,{'eax':'var_2','ecx':'var_1','edx':'var_4'})==evidence['format_mapping']['full206'];assert fmt[7]=='c'
handler=text('sub_4A9E50');start=handler.index('loc_4A9EAB:');end=handler.index('call    sub_4A2A50',start);fmt0=re.search(r"db '([^']+)'",(A/'aSchcscccc-context.asm').read_text(encoding='utf-8'))[1];assert paired(handler[start:end],fmt0,{'edx':'var_4'})==evidence['format_mapping']['subtype0_213'];assert fmt0[1]=='c'
dispatch=text('sub_4B2970');assert 'cmp     dword_64BBD4, 4'in dispatch and 'cmp     dword_64AF98, 3\njl      loc_4B313F'in dispatch and 'jumptable 004B3156 case 206'in dispatch and 'call    sub_4A8BA0'in dispatch
assert 'cmp     eax, 6'in handler and 'mov     dword_64ABF4, eax'in handler and 'jnz     def_4A9EA4'in handler
phase=text('sub_469CA0');assert 'movsx   eax, cx'in phase and 'mov     dword_64AF98, 2'in phase and 'mov     ecx, 2'in full and 'call    sub_469CA0'in full
raw=Path(evidence['phase6_DD']['raw_prefix']['path']).read_bytes();signature=bytes.fromhex('C70598AF640006000000');assert raw.index(signature)==47 and evidence['phase6_DD']['instruction_address']=='0x0045454F'
assert text(extra['callback_registration_owner']).count('push    offset dword_454520')==2
identity=text('sub_4A7F60');assert 'mov     edi, offset dword_64ABF4\nrep movsd'in identity and 'mov     [ebp+var_104], eax'in identity and 'mov     ecx, 23h'in identity
parser=text('sub_4A2A50');assert 'mov     byte ptr [ecx+edi-1], 0'in parser and 'call    near ptr 774095E0h'in parser and 'inc     eax\nmov     ebx, eax'in parser
spec=read(L/'r489-event-input.json');d=read(L/'r489-event-details.json');assert len(spec['cases'])==len(d['traces'])==d['event_cases']==207
for expected,actual in zip(spec['cases'],d['traces']):assert actual['input']==expected and actual['actual']==expected['expected']and actual['matches_expected']
assert d['runtime_flow_cases']==len(d['runtime_flow'])==9 and all(x['passed']for x in d['runtime_flow'])and d['actual_active_load_observed']and d['actual_worker_retired']and d['frames_published']==0
assert(d['historical_PASS_batches_rerun'],d['GUI'],d['LOADED'],d['complete_modes'])==(0,0,False,16)
player=read(L/'r489-player-candidate.json');assert player['candidate_executions']==0 and not player['native_GUI_executed']and player['binding']==read(L/'r488-player-candidate.json')['binding']and player['skin_binding']==read(L/'r488-player-candidate.json')['skin_binding']
assert Path(player['candidate_main']['path']).read_text(encoding='utf-8').replace('R489','R488').replace('r489','r488')==Path(read(L/'r488-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8')
exe=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',exe,0x3c)[0];assert struct.unpack_from('<H',exe,pe+4)[0]==332 and struct.unpack_from('<H',exe,pe+24+68)[0]==2
consumer=read(L/'r489-consumer-candidate.json');assert len(consumer['source_pins'])==172 and consumer['production_library']==player['production_library'];exit=read(L/'r489-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and 'R489 focused PASS:207fixed new snapshot cases/9runtime flows'in Path(exit['stdout']['path']).read_text(encoding='utf-8-sig')
for letter in ['a','b']:
 build=[json.loads(x)for x in (L/f'r489{letter}-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [b['success']for b in build if b['reason']=='build-finished']==[True]
artifact=[f for b in build if b['reason']=='compiler-artifact'and b['target']['name']=='lito_modern'for f in b['filenames']if f.endswith('.rlib')];assert len(artifact)==1;check(artifact[0],player['production_library']['sha256'],player['production_library']['bytes'])
scope=read(L/'r489-source-scope.json');assert(scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(604,467,12,125)
diff=L/'r489-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');out=dict(state='R489_INDEPENDENT_FORMAT_PUSH_PAIRING_BYTE_WIDTH_PHASE_SIGNED_DISPATCH_SOURCE_EVENT_RUNTIME_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=172,all_production_source_snapshots=604,preflight_source_snapshots=604,changed_from_fixed_R488=changes,new_module='native_mode_snapshot.rs',source_difference=pin(diff),source_fragments=24,snapshot_mode_width_corrected_from_R488='c/lowbyte for206and213subtype0',actual_fixed_event_cases=207,actual_runtime_flow_cases=9,actual_active_load_cancelled=True,actual_worker_retired=True,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],production_library_builds=2,player_candidate_executions=0,old40_plans_cache_decoder_menu_window_worker_preserved=True,live_network_dispatch_connected=False,full_native_state_side_effects_reproduced=False,thumb_track_geometry_unresolved=True,full_test_suite_executed=False,complete_modes=16,GUI=0,LOADED=False)
save(L/'r489-independent-contract.json',out);save(L/'r489-candidate.json',consumer|dict(validation='PASS one finite complete known snapshot/source-context/runtime-flow consumer;player candidate unexecuted',independent_contract=pin(L/'r489-independent-contract.json')));print('R489 independent PASS:',checks,'pins',len(cache),'unique;172consumer/604production+604preflight;24source fragments;207events/9runtime flows;mode c source correction;GUI0/player0')
