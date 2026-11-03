from r495_source_oracle import *
import re,struct
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==v['bytes'] and cache[p]['sha256']==v['sha256'],p
   if 'snapshot'in v:
    sp=pin(v['snapshot']);assert sp['bytes']==v['bytes'] and sp['sha256']==v['sha256']
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
names=['source-evidence','source-findings','source-oracle','entry-source-contract','production-source-snapshot','source-scope','player-candidate','consumer-candidate','consumer-exit','event-details','preparation-history']
for n in names:walk(read(L/('r495-'+n+'.json')))
old=read(L/'r494-production-source-snapshot.json');delta=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:delta.append(Path(q['path']).name)
assert delta==['native_city_runtime.rs']
snapshot=read(L/'r495-production-source-snapshot.json');assert len(snapshot['source_pins'])==609
c=read(L/'r495-consumer-candidate.json');player=read(L/'r495-player-candidate.json')
assert len(c['source_pins'])==189 and c['production_library']==player['production_library'] and player['candidate_executions']==0
exit=read(L/'r495-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and exit['historical_PASS_reruns']==0
d=read(L/'r495-event-details.json');assert d['flow_cases']==12 and d['terminal_write_cases']==4 and d['actual_Runtime_owners']==2 and all(x['passed']for x in d['trace'])
assert all(d[k]for k in ['source_instruction_oracle','source_typed_record_alias','send_receive_counter_independent','send_advance_before_write','partial_write_no_reencrypt_or_doublecounter','completion_is_only_local_report'])
assert not any(d[k]for k in ['actual_provider_or_socket_executed','normal_wire_connected','login480560_connected','actor205_209_connected','phase6_connected','actual_credentials_input','raw_capture_dump']) and d['GUI']==0 and d['complete_modes']==16
e=read(L/'r495-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(e['analysis_PE']['path']).read_bytes()
for f in e['fragments']:
 m=re.search('^'+f['name']+r' proc near',asm,re.M);assert m
 z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert z
 assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()]
for f in e['PE_fragments']:
 if f['path'].endswith('.bin'):
  p=Path(f['path']);offset=int(p.stem.split('-')[1],16)-0x400000;assert p.read_bytes()==pe[offset:offset+f['bytes']]
O=P/'target/r495-native-send-study';source=lambda n:(O/(n+'.asm')).read_text(encoding='utf-8')
compat=source('sub_480560');assert 'mov     bl, 2'in compat and 'aFromcd'in compat and 'aSoftwareNcSoft'in compat
assert re.search(r'push    ecx\npush    edx\npush    eax\npush    8033h\npush    0\npush    offset aChdch',compat)
select=source('sub_4928C0');assert 'mov     edx, 3\nxor     ecx, ecx\ncall    sub_494120'in select and 'cmp     byte ptr [ebx+2Eh], 0FFh'in select and 'mov     al, [ebx+37h]'in select
assert select.index('push    ebx',select.index('mov     edi, offset byte_62FAA0')if 'mov     edi, offset byte_62FAA0'in select else 0)<select.index('push    0C9h')<select.index('push    offset aCs')<select.index('call    sub_4C6AE0')
serializer=source('sub_4C68D0');assert 'jumptable 004C690F case 99'in serializer and 'jumptable 004C690F case 115'in serializer and 'mov     cl, [eax+4]\nadd     eax, 4\nmov     [ebx], cl'in serializer and 'lea     ebx, [ebx+eax+1]'in serializer
frame=source('sub_4C6AE0');assert 'mov     edx, 13FEh'in frame and 'add     eax, 2\npush    1'in frame and frame.index('mov     [ebp+var_13FF], dl')<frame.index('call    sub_4C6770')
send=source('sub_4C6770');assert send.index('call    sub_4C6700')<send.index('mov     dword_8317CC, eax')<send.index('call    sub_4EFEA4')
assert 'add     ecx, eax\nmov     eax, dword_8317CC\nadc     eax, edx'in send and 'add     ebx, eax\nsub     esi, eax'in send
assert struct.unpack('<I',pe[0xc688c:0xc6890])[0]==0x4c6837 and pe[0xc68a0]==0
instructions=(O/'pe-4C6770.disasm').read_text(encoding='utf-8');assert 'lea ecx, [eax - 0x2733]'in instructions and 'mov dword ptr [0x8317d8], eax'in instructions and 'call 0x77409100'in instructions
v=read(L/'r495-source-oracle.json');ctr=v['initial_counter'];assert ctr==0x87546ca1fffffffe
for case in v['send_cases']:
 # Source c literal from4928C0;source s includes terminator per4C68D0.
 opcode=int(re.search(r'push    ([0-9A-F]+)h\nmov     edx, ecx',select).group(1),16)&255
 body=bytes([opcode])+v['synthetic_alias'].encode('ascii')+b'\0';assert list(body)==case['plaintext']
 cipher,steps=source_cipher(body,ctr,pe);assert case['frame']==list((len(body)+2).to_bytes(2,'little')+cipher)
 assert case['source_instruction_steps']==steps and case['counter_before']==ctr
 ctr=counter_add_adc(ctr,len(body));assert case['counter_after']==ctr
prior=read(L/'r494-event-input.json');ctr=v['initial_counter'];wire=[]
for body in [bytes([3,1]),bytes(prior['roster'])]:
 cipher,_=source_cipher(body,ctr,pe);wire.extend((len(body)+2).to_bytes(2,'little')+cipher);ctr=counter_add_adc(ctr,len(body))
assert wire==v['receive_setup_wire'] and ctr==v['receive_after_setup']
body=bytes(prior['mode206']);cipher,_=source_cipher(body,ctr,pe);assert list((len(body)+2).to_bytes(2,'little')+cipher)==v['mode206_wire'] and counter_add_adc(ctr,len(body))==v['receive_after_mode']
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');module=(P/'src/modern_lito/native_source_send.rs').read_text(encoding='utf-8')
assert 'pub(crate) struct Sender'in module and 'pub fn set_counter'not in module and 'pub fn prepare_character'not in module
assert 'let next=self.source_identity.prepare_select(slot)?'in rt and 'self.source_send.prepare_character(next.selected().unwrap().actor_alias())?'in rt
assert 'Native480560 FromCD/version/registry/auth sideeffects unqualified'in rt and 'source_stream::Frame::Seed{counter}=>{self.source_send.seed_from_source(counter)?;'in rt
assert rt.count('self.source_send=source_send::Sender::default();')==2 and 'if matches!(progress,source_send::Progress::Retired{..}){self.disconnect_source_connection_candidate(epoch)?;}'in rt
assert 'Arc::ptr_eq(&self.owner,&ticket.owner)'in module and 'p.offset==expected_offset'in module and 'impl Drop for Pending'in module and 'self.life.load(Ordering::Acquire)'in module
for n in ['build-r495-player-candidate.ps1','build-r495-source-send-consumer.ps1']:
 s=(P/'tools'/n).read_text(encoding='utf-8-sig');assert "$scratch='D:\\lito-dev-runtime\\20261003-supervision\\modern\\r495-native-send'"in s and '$env:TEMP=$scratch;$env:TMP=$scratch'in s
record=dict(state='R495_INDEPENDENT_SOURCE_CS201_SERIALIZER_CIPHER_PREWRITE_COUNTER_AND_RUNTIME12FLOWS_PASS',pin_instances=instances,unique_paths=len(cache),consumer_source_pairs=189,production_source_files=609,changed_from_R494=delta,new_source_files=['native_source_send.rs'],source_ASM_fragments=len(e['fragments']),source_PE_fragments=len(e['PE_fragments']),source_symbol_contexts=len(e['contexts']),source_difference=snapshot['source_difference'],source_instruction_oracle=True,source_oracle_vectors=2,flow_cases=12,terminal_write_cases=4,finite_consumer_executions=1,player_executions=0,normal_wire_connected=False,login480560_connected=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r495-independent-contract.json',record)
save(L/'r495-candidate.json',dict(state='R495_TYPED_CS201_EXISTING_RUNTIME_SEND_COUNTER_PARTIAL_WRITE_FINITE_CANDIDATE',source_pins=c['source_pins'],production_library=player['production_library'],player=player['executable'],consumer={k:c[k]for k in ['path','bytes','sha256']},source_scope=read(L/'r495-source-scope.json'),independent=record,GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R495 independent PASS',instances,'pininstances',len(cache),'unique;189pairs609production/12flows/sourcecs201+partialwrite')
