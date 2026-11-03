from r491_common import *
import re
count=0;cache={}
def check(v):
 global count
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   count+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==v['bytes'] and cache[p]['sha256']==v['sha256'],p
   if 'snapshot'in v:assert pin(v['snapshot'])['sha256']==v['sha256']
  for a in v.values():check(a)
 elif isinstance(v,list):
  for a in v:check(a)
names=['r493-source-evidence.json','r493-pe-evidence.json','r493-source-findings.json','r493-entry-source-contract.json','r493-production-source-snapshot.json','r493-source-scope.json','r493-player-candidate.json','r493-consumer-candidate.json','r493-consumer-exit.json','r493-event-input.json','r493-event-details.json','r493-preparation-history.json']
for n in names:check(read(L/n))
old=read(L/'r492-production-source-snapshot.json');delta=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:delta.append(Path(q['path']).name)
assert delta==['native_city_runtime.rs','native_source_connection.rs']
frozen=read(L/'r493-production-source-snapshot.json');assert len(frozen['source_pins'])==607
c=read(L/'r493-consumer-candidate.json');assert len(c['source_pins'])==183
player=read(L/'r493-player-candidate.json');assert c['production_library']==player['production_library'] and player['candidate_executions']==0
exit=read(L/'r493-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and exit['historical_PASS_reruns']==0
d=read(L/'r493-event-details.json');assert d['flow_cases']==11 and d['malformed_body_cases']==104 and all(x['passed']for x in d['trace'])
assert d['GUI']==0 and not d['normal_wire_connected'] and not d['actor205_209_connected'] and not d['phase6_connected'] and not d['native_socket_executed'] and d['complete_modes']==16
e=read(L/'r493-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8')
for f in e['fragments']:
 m=re.search('^'+f['name']+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()]
O=P/'target/r493-source-producer-study';text=lambda n:(O/(n+'.asm')).read_text(encoding='utf-8')
assert 'movsx   edx, [ebp+var_1]'in text('sub_4A7450') and 'add     eax, 0B4h'in text('sub_4A7450') and 'cmp     eax, offset word_62FD9E'in text('sub_4A7450')
assert (0x62FD9E-0x62FACE)//180==4 and (0x62FD9E-0x62FACE)%180==0
assert 'mov     edx, 3\nxor     ecx, ecx\ncall    sub_494120'in text('sub_4928C0')
assert 'mov     dl, [ebx+5Ch]\nmov     byte_64AC29, dl'in text('sub_4928C0') and 'mov     cl, [ebx+2Dh]\nmov     byte_64AC26, cl'in text('sub_4928C0')
assert 'push    esi'in text('sub_4D6970') and 'pop     esi'in text('sub_4D6970')
stack=read(O/'203-stack-field-map.json');fm=stack['field_map'];assert len(fm)==33 and fm[26]['destination']=='Record+0x5c' and fm[32]['destination']=='IncomingBody' and fm[32]['callee_EBP_offset']=='0x98'
assert 'mov     edx, [esi+4]'in text('sub_4A2A50') and 'mov     [edx], al'in text('sub_4A2A50')
pe=read(L/'r493-pe-evidence.json');pd=Path(pe['PE']['path']).read_bytes();assert pd[0xA758F]==0x56 and pd[0xA7636:0xA763C]==bytes.fromhex('81c490000000');assert pd[0x2343B4:0x2343B4+34]==b'sscchhhcccccccccddddddddddcdddddc\0'
assert pd[0xA3495:0xA349A]==bytes.fromhex('684cd56300') and pd[0x2342C0:0x2342C9]==b'cdddddcc\0'
assert 'lea     eax, [ecx+eax-2]'in text('sub_4B37C0') and 'call    sub_4C66C0'in text('sub_4B37C0') and 'call    sub_4B2970'in text('sub_4B37C0')
assert 'xor     bl, dl\nmov     dl, cl'in text('sub_4C66C0') and 'and     ebx, 7'in text('sub_4C66C0')
identity=(P/'src/modern_lito/native_source_identity.rs').read_text(encoding='utf-8');rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8')
assert 'arena_bound_byte: u8'in identity and 'actor_alias: Vec<u8>'in identity and 'pub fn actor_identifier(&self)->Option<u32> { None }'in identity
assert 'self.source_scope.admit(event)?;'in rt and 'self.source_scope.admit_epoch(epoch)?;'in rt and rt.count('self.source_identity=source_identity::State::default();')==2
assert 'set_phase'not in identity and 'set_actor'not in identity and 'CharacterSummary'not in identity.replace('No ModernCharacterSummary conversion','')
record=dict(state='R493_INDEPENDENT_SOURCE_PE_CONTRACT_AND_FIXED_CONSUMER11FLOWS104MALFORMED_PASS',pin_instances=count,unique_paths=len(cache),consumer_source_pairs=183,production_source_files=607,changed_from_R492=delta,new_source_files=['native_source_identity.rs'],source_ASM_fragments=len(e['fragments']),source_difference=frozen['source_difference'],PE_stack_fields=33,last_field_destination='savedincomingESI/bodyopcode NOTrecord94',class_field=2,bound_field=26,flow_cases=11,malformed_cases=104,finite_consumer_executions=1,player_executions=0,normal_wire_connected=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r493-independent-contract.json',record)
save(L/'r493-candidate.json',dict(state='R493_SOURCE_QUALIFIED_TYPED_ROSTER_SELECTED_IDENTITY_FINITE_CANDIDATE',source_pins=c['source_pins'],production_library=player['production_library'],player=player['executable'],consumer={k:c[k]for k in ['path','bytes','sha256']},source_scope=read(L/'r493-source-scope.json'),independent=record,GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R493 independent PASS',count,'pininstances',len(cache),'unique;183pairs607production/203tail savedESI/11flows104malformed')
