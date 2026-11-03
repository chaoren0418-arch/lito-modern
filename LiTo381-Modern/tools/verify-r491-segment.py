from r491_common import *
import re,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
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
names=['r491-source-evidence.json','r491-source-findings.json','r491-preparation-history.json','r491-entry-source-contract.json','r491-player-candidate.json','r491-command-input.json','r491-consumer-candidate.json','r491-consumer-exit.json','r491-command-details.json','r491-source-scope.json','r491-production-source-snapshot.json']
for n in names:walk(read(L/n))
old=read(L/'r490-production-source-snapshot.json');fixed={Path(x['path']).name:x for x in old['source_pins']};assert len(old['source_pins'])==605
for q in old['source_pins']:check(q['snapshot'],q['sha256'],q['bytes'])
runtime=Path(fixed['native_city_runtime.rs']['path']).read_text(encoding='utf-8');before=Path(fixed['native_city_runtime.rs']['snapshot']).read_text(encoding='utf-8');window=Path(fixed['native_city_window.rs']['path']).read_text(encoding='utf-8');oldwindow=Path(fixed['native_city_window.rs']['snapshot']).read_text(encoding='utf-8')
assert before[before.index('fn worker('):before.index('impl Runtime {')]==runtime[runtime.index('fn worker('):runtime.index('impl Runtime {')]
assert before[before.index(' pub fn consume_source_packet('):before.index(' /// A real explicit list-open boundary')]==runtime[runtime.index(' pub fn consume_source_packet('):runtime.index(' /// A real explicit list-open boundary')]
assert before[before.index(' pub fn poll('):]==runtime[runtime.index(' pub fn poll('):]
assert 'pub const SOURCE_MENU_COMMAND_ID:u16=0x4910;'in runtime and 'SOURCE_MENU_COMMAND_ID=>Some(Action::OpenSourceMenu)'in runtime and 'pub fn handle_player_command(&mut self,command:u16)'in runtime
route=runtime[runtime.index(' pub fn handle_player_command('):runtime.index(' /// Persistent source-event entry')];assert 'player_command_action(command).context('in route and 'self.handle(action)'in route and 'phase:'not in route
open=runtime[runtime.index(' pub fn open_source_state_menu('):runtime.index(' pub fn native_mode(')];assert open.index('self.source_flow.phase()==2&&self.source_flow.snapshot().is_some()')<open.index('catalog_mode()?')<open.index('Menu::for_filter')<open.index('self.next_generation()?')<open.index('self.invalidate();')and 'self.selected=None'in open
handle=runtime[runtime.index(' pub fn handle(&mut self,'):runtime.index(' pub fn poll(')];oldhandle=before[before.index(' pub fn handle(&mut self,'):before.index(' pub fn poll(')];assert handle.replace('   Action::OpenSourceMenu=>self.open_source_state_menu()?,\n','')==oldhandle
assert 'WM_COMMAND if (wparam.0&0xffff)==usize::from(SOURCE_MENU_COMMAND_ID)'in window and 'r.handle_player_command(SOURCE_MENU_COMMAND_ID)'in window and 's.screen.set(Screen::Maps)'in window
assert all(pin(q['path'])['sha256']==q['sha256']for q in old['source_pins']if Path(q['path']).name not in ['native_city_runtime.rs','native_city_window.rs'])
e=read(L/'r491-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');A=P/'target/r491-action-study/authority'
for f in e['fragments']:
 if f['path'].endswith('-through-next-proc.asm'):
  m=re.search('^'+f['name']+r' proc near',asm,re.M);end=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+end.start()]
t=lambda n:(A/(n+'-through-next-proc.asm')).read_text(encoding='utf-8')
constructor=t('sub_453B00');assert constructor.count('push    offset dword_454520')==2 and 'push    offset sub_4544E0'in constructor and 'mov     byte ptr [esi+0Ch], 0FFh'in constructor and 'mov     byte ptr [esi+0Dh], 7Fh'in constructor
assert 'mov     ecx, [eax+28h]'in t('sub_414C40')and 'jmp     dword ptr [eax+24h]'in t('sub_414C40')and 'call    dword ptr [eax+40h]'in t('sub_414C00')
v=(A/'off_4F7B60-source-context.asm').read_text(encoding='utf-8');vv=[x.split('offset ')[1].split()[0]for x in v.splitlines()if 'dd offset'in x];assert vv[16]=='sub_414C40'
parent_v=(A/'off_4F8524-source-context.asm').read_text(encoding='utf-8');vv=[x.split('offset ')[1].split()[0]for x in parent_v.splitlines()if'dd offset'in x];assert vv[8:10]==['sub_454230','sub_454250']
assert 'push    offset sub_46B6D0'in t('sub_46BA10')and 'mov     dword_648F18, eax'in t('sub_46BA10')and 'mov     ecx, dword_648F18'in t('sub_46B6D0')
assert 'call    dword ptr [eax+20h]'in t('sub_4EBBD0')and 'mov     byte ptr [esi+0Fh], 0'in t('sub_454230')and '64AF98'not in t('sub_454230')
for off in ['0Ch','0Dh','0Eh']:assert f'mov     [ecx+{off}], al'in t('sub_464AD0')
assert 'push    7Fh'in t('sub_454670')and 'mov     al, byte_64AC29'in t('sub_454670')and 'cmp     [edx+0Ch], bl'in t('sub_454670')
for r in e['raw_dd']:
 b=Path(r['raw']['path']).read_bytes();d=[dict(address=f'{i.address:08X}',bytes=i.bytes.hex().upper(),mnemonic=i.mnemonic,operands=i.op_str)for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(b,int(r['base'],16))];assert d==r['instructions']
d={x['address']:x for x in e['raw_dd'][0]['instructions']};assert d['0045454F']['bytes']=='C70598AF640006000000'and d['0045457C']['bytes']=='C6460F01'and d['00454574']['operands']=='0x4c6ae0'
assert [d[x]['operands']for x in ['00454559','0045455D','00454561']]==['eax, byte ptr [esi + 0xe]','ecx, byte ptr [esi + 0xc]','edx, byte ptr [esi + 0xd]']
assert 'mov     cl, [eax+4]'in t('sub_4C68D0')and 'mov     [ebx], cl'in t('sub_4C68D0')and 'lea     eax, [ebp+arg_4]'in t('sub_4C6AE0')
response=t('sub_4A9E50');assert 'cmp     eax, 6'in response and 'mov     dword_64ABF4, eax'in response and 'mov     dword_64AF98'not in response and '0Fh]'not in response
assert 'jumptable 004B2A0C case 235'in t('sub_4B2970')and 'jumptable 004B2A0C case 236'in t('sub_4B2970')
assert 'cmp     al, 10h'in t('sub_4AAE70')and 'cmp     al, 11h'in t('sub_4AAE70')and 'mov     dword_64AF98, 1'in t('sub_4AAF60')and 'mov     byte ptr [eax+0Fh], 0'in t('sub_4AAF60')
assert 'push    0E4h'in t('sub_4544F0')and 'mov     dword_64AF98'not in t('sub_4544F0')
labels=read(e['labels']['path']);ss=Path(e['label_table']['path']).read_bytes().decode('cp949').splitlines();assert int(ss[0])==296
for x in labels:assert ss[x['index']+1]==x['text']and x['text'].encode('cp949').hex().upper()==x['raw_hex']
findings=read(L/'r491-source-findings.json');assert findings['request']['identifier_present']is False and findings['phase6_callback_connected']is False
consumer=read(L/'r491-consumer-candidate.json');player=read(L/'r491-player-candidate.json');walk(player);assert len(consumer['source_pins'])==177 and consumer['production_library']==player['production_library']and player['candidate_executions']==0
main=Path(player['candidate_main']['path']).read_text(encoding='utf-8');assert main.replace('R491','R490').replace('r491','r490')==Path(read(L/'r490-player-candidate.json')['candidate_main']['path']).read_text(encoding='utf-8')
assert player['binding']==read(L/'r490-player-candidate.json')['binding']and player['skin_binding']==read(L/'r490-player-candidate.json')['skin_binding']
b=Path(player['executable']['path']).read_bytes();pe=struct.unpack_from('<I',b,0x3c)[0];assert struct.unpack_from('<H',b,pe+4)[0]==332 and struct.unpack_from('<H',b,pe+24+68)[0]==2
d=read(L/'r491-command-details.json');assert d['flow_cases']==len(d['trace'])==10 and all(x['passed']for x in d['trace'])and (d['actual_active_563_loads'],d['actual_completed_506_loads'],d['frames_published'])==(3,1,1)
for k in ['old_frame_retired','same_mode_command_generation_cancel','Back_cancel','close_cancel','worker_retired','wrong_id_atomic','unsupported_mode_atomic','BSS_source_admission_refused','unknown_command_atomic']:assert d[k]
assert (d['GUI'],d['LOADED'],d['complete_modes'],d['historical_PASS_batches_rerun'],d['phase6_callback_connected'])==(0,False,16,0,False)
consumer_rs=(P/'tools/r491-player-command-consumer.rs').read_text(encoding='utf-8');assert 'Context{phase:'not in consumer_rs and 'rt.apply_native_state_packet'not in consumer_rs and 'set_phase'not in consumer_rs
ex=read(L/'r491-consumer-exit.json');assert ex['exit_code']==0 and ex['executions']==1 and 'R491 focused PASS:10newplayercommandflows'in Path(ex['stdout']['path']).read_text(encoding='utf-8-sig')
build=[__import__('json').loads(x)for x in (L/'r491a-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()];assert [x['success']for x in build if x['reason']=='build-finished']==[True];paths=[f for x in build if x['reason']=='compiler-artifact'and x['target']['name']=='lito_modern'for f in x['filenames']if f.endswith('.rlib')];assert len(paths)==1;check(paths[0],player['production_library']['sha256'],player['production_library']['bytes'])
scope=read(L/'r491-source-scope.json');assert (scope['source_files'],scope['unchanged_source_files'],len(scope['changed_source_files']),len(scope['new_source_files']))==(605,467,12,126)
out=dict(state='R491_INDEPENDENT_SOURCE_ARENA_REQUEST_GAPS_PLAYER_COMMAND_GENERATION_FRAME_WORKER_AND_HASH_PASS',checked_pin_instances=checks,unique_files=len(cache),source_pairs=177,all_production_source_snapshots=605,source_fragments=len(e['fragments']),source_difference=pin(L/'r491-source-difference.patch'),changed_from_fixed_R490=scope['changed_from_fixed_R490'],new_production_files=0,actual_player_command_flows=10,actual_active_563_loads=3,actual_completed_506_loads=1,finite_consumer_executions=1,finite_seconds=d['focused_seconds'],production_library_builds=1,consumer_compile_failures=0,player_candidate_executions=0,phase6_callback_connected=False,request_response_success_association_closed=False,live_network_dispatch_connected=False,actual_WM_COMMAND_GUI_executed=False,worker_cache_decoder_menu_snapshot_sourceflow_unchanged=True,complete_modes=16,GUI=0,LOADED=False)
save(L/'r491-independent-contract.json',out);save(L/'r491-candidate.json',consumer|dict(validation='PASS one new productioncommandconsumer;phase6/requestresponse remainsunconnected;GUI0/player0',independent_contract=pin(L/'r491-independent-contract.json')));print('R491 independentPASS:',checks,'pin instances/',len(cache),'unique;177pairs/605production/42ASM;10flows/3activecancels/1realframe;phase6unconnected')
