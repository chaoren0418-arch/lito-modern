from r491_common import *
import re
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
names=['r494-source-evidence.json','r494-source-findings.json','r494-event-input.json','r494-entry-source-contract.json','r494-production-source-snapshot.json','r494-source-scope.json','r494-player-candidate.json','r494-consumer-candidate.json','r494-consumer-exit.json','r494-event-details.json','r494-preparation-history.json']
for n in names:walk(read(L/n))
old=read(L/'r493-production-source-snapshot.json');delta=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:delta.append(Path(q['path']).name)
assert delta==['native_city_runtime.rs']
snapshot=read(L/'r494-production-source-snapshot.json');assert len(snapshot['source_pins'])==608
c=read(L/'r494-consumer-candidate.json');player=read(L/'r494-player-candidate.json');assert len(c['source_pins'])==186 and c['production_library']==player['production_library'] and player['candidate_executions']==0
exit=read(L/'r494-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and exit['historical_PASS_reruns']==0
d=read(L/'r494-event-details.json');assert d['flow_cases']==13 and d['invalid_length_cases']==5 and all(x['passed']for x in d['trace'])
assert d['same_existing_epoch_decoder_and_Runtime'] and d['before_dispatch_counter_advance'] and d['batch_event_bound']==8 and not d['arbitrary_counter_injection']
assert all(d[k] for k in ['header_eof','body_eof','clean_eof']) and not any(d[k]for k in ['normal_wire_connected','login480560_effects_connected','native_socket_executed','phase6_connected','actor205_209_connected']) and d['GUI']==0 and d['complete_modes']==16
e=read(L/'r494-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8')
for f in e['fragments']:
 m=re.search('^'+f['name']+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()]
O=P/'target/r494-native-receive-study';source=lambda n:(O/(n+'.asm')).read_text(encoding='utf-8')
seed=source('sub_4AE050');assert 'inc     ecx'in seed and 'push    offset aD'in seed and 'call    sub_4A2A50'in seed and 'or      eax, 87546CA1h'in seed
assert 'mov     byte_8317B8, 1\nmov     dword_8317CC, eax\nmov     dword_8317C8, ecx\nmov     dword_8317C0, ecx\nmov     dword_8317C4, eax\ncall    sub_480560'in seed
open=source('sub_4C62A0');assert open.index('mov     byte_8317B8, 0')<open.index('call    sub_4C65F0')
reset=source('sub_4B38D0');assert 'mov     byte_7607A8, 0\nmov     dword_7607A4, 2'in reset
receive=source('sub_4B37C0');assert receive.index('call    sub_4C66C0')<receive.index('mov     dword_8317C0, ecx')<receive.index('call    sub_4B2970') and 'adc     eax, edx'in receive and 'lea     eax, [ecx+eax-2]'in receive
assert 'cmp     byte ptr dword_7607B0, bl'in receive and 'mov     al, byte_8317B8'in receive
cipher=source('sub_4C66C0');assert 'and     ebx, 7'in cipher and 'xor     bl, dl\nmov     dl, cl'in cipher
close=source('sub_4C65F0');assert 'mov     byte_8317B8,'not in close and 'mov     dword_8317C0,'not in close
pe=Path(e['analysis_PE']['path']).read_bytes();assert bytes.fromhex('0da16c5487')in pe[0xae050:0xae097] and bytes.fromhex('c605b817830001')in pe[0xae050:0xae097] and bytes.fromhex('c605b817830000')in pe[0xc62a0:0xc62d0]
v=read(L/'r494-event-input.json');ctr=v['initial_counter'];assert ctr==(0x87546ca1<<32)|0xfffffffe and v['plain_seed_body']==[1,254,255,255,255]
for step in v['steps']:
 assert step['before']==ctr;key=ctr.to_bytes(8,'little');previous=0;out=[]
 for i,ciph in enumerate(step['cipher']):out.append(ciph^key[i&7]^previous);previous=ciph
 assert out==step['plain'];ctr=(ctr+len(out))&((1<<64)-1);assert step['after']==ctr
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');module=(P/'src/modern_lito/native_source_stream.rs').read_text(encoding='utf-8')
assert 'pub(crate) struct Receiver'in module and 'pub fn set_counter'not in module and 'if let Some(counter)=self.counter'in module and 'counter.wrapping_add(body_len as u64)'in module
assert 'epoch.decoded_packet(&bytes)'in rt and 'self.consume_source_identity_event(&packet).map(StreamIngress::Roster)'in rt and 'self.consume_source_event(&packet).map(StreamIngress::Mode)'in rt
assert rt.count('self.source_stream=source_stream::Receiver::default();')==2 and 'self.disconnect_source_connection_candidate(epoch)?;'in rt
record=dict(state='R494_INDEPENDENT_SEED_FLAG_COUNTER_ORDER_SOURCE_AND_FIXED_RUNTIME_STREAM13FLOWS_PASS',pin_instances=instances,unique_paths=len(cache),consumer_source_pairs=186,production_source_files=608,changed_from_R493=delta,new_source_files=['native_source_stream.rs'],source_ASM_fragments=len(e['fragments']),source_PE_fragments=len(e['PE_fragments']),source_difference=snapshot['source_difference'],seed_high=0x87546ca1,seed_low_source='Nativeopcode1/d',counter_advance_before_dispatch=True,bounded_events=8,frame_byte_bound=157,flow_cases=13,finite_consumer_executions=1,player_executions=0,normal_wire_connected=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r494-independent-contract.json',record)
save(L/'r494-candidate.json',dict(state='R494_SOURCE_SEEDED_NATIVE_STREAM_EXISTING_RUNTIME_FINITE_CANDIDATE',source_pins=c['source_pins'],production_library=player['production_library'],player=player['executable'],consumer={k:c[k]for k in ['path','bytes','sha256']},source_scope=read(L/'r494-source-scope.json'),independent=record,GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R494 independent PASS',instances,'pininstances',len(cache),'unique;186pairs608production/seedflagadvance/source13flows')
