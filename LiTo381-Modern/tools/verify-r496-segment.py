from r496_source_oracle import *
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
for n in ['source-evidence','source-findings','source-oracle','entry-source-contract','production-source-snapshot','source-scope','player-candidate','consumer-candidate','consumer-exit','event-details','preparation-history']:walk(read(L/('r496-'+n+'.json')))
old=read(L/'r495-production-source-snapshot.json');delta=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:delta.append(Path(q['path']).name)
assert delta==['native_city_runtime.rs','native_city_window.rs','native_source_connection.rs','native_source_send.rs']
snapshot=read(L/'r496-production-source-snapshot.json');assert len(snapshot['source_pins'])==610
c=read(L/'r496-consumer-candidate.json');player=read(L/'r496-player-candidate.json');assert len(c['source_pins'])==192 and c['production_library']==player['production_library'] and player['candidate_executions']==0
exit=read(L/'r496-consumer-exit.json');assert exit['exit_code']==0 and exit['executions']==1 and exit['historical_PASS_reruns']==0
d=read(L/'r496-event-details.json');assert d['flow_cases']==14 and d['revision_boundary_cases']==13 and d['malformed_cases']==1025 and d['actual_Runtime_owners']==2 and all(x['passed']for x in d['trace'])
assert all(d[k]for k in ['source_PE_branch_oracle','same_Runtime_bridge_as_actual_WM_COMMAND','opaque_epoch_roster_revision_controls','source58_version_connected','source58_refusal_retires_epoch','source209_inactive_phase_ignored','source205_dynamic_comparison_refused','source209_active_dynamic_comparison_refused'])
assert not any(d[k]for k in ['Native_actor_identifier_installed','Native_full_actor205_209_connected','authentication_success_claimed','actual_credentials_input','raw_capture_dump','actual_provider_or_socket_executed','normal_wire_connected','login480560_connected','phase6_connected','visible_controls_GUI_executed']) and d['GUI']==0 and d['complete_modes']==16
e=read(L/'r496-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(e['analysis_PE']['path']).read_bytes()
for f in e['fragments']:
 m=re.search('^'+f['name']+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert Path(f['path']).read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()]
for f in e['PE_fragments']:
 if f['path'].endswith('.bin'):
  p=Path(f['path']);offset=int(p.stem.split('-')[1],16)-0x400000;assert p.read_bytes()==pe[offset:offset+f['bytes']]
O=P/'target/r496-actor-study';source=lambda n:(O/(n+'.asm')).read_text(encoding='utf-8');dis=lambda a:(O/f'pe-{a:X}.disasm').read_text(encoding='utf-8')
v58=dis(0x4a33e0);assert 'cmp cl, 1'in v58 and 'jne 0x4a3474'in v58 and 'push 0x63d54c'in v58 and 'cmp ecx, 0x9e38'in v58
parsercalls=[l.split()[0]for l in v58.splitlines()if l.endswith('call 0x4a2a50')];assert parsercalls==['004A3423','004A34A1','004A34BA','004A34F3']
s209=source('sub_4A9550');assert '"sc"'in s209 and s209.index('cmp     eax, 2')<s209.index('call    sub_4D3A80') and 'cmp     eax, esi\njnz     loc_4A975A'in s209 and 'mov     esi, 6'in s209
copy=source('sub_4A7F60');assert 'push    offset dword_64ABF8'in copy and 'mov     ecx, 23h' in copy and 'mov     edi, offset dword_64ABF4'in copy
init=source('sub_4D39F0');assert 'rep stosd'in init and 'mov     edi, dword_4F727C'in init and 'mov     byte_834960[esi], al'in init and 'cmp     esi, 100h'in init
cmp=source('sub_4D3A80');assert 'mov     dl, byte_834A60[eax]'in cmp and 'mov     dl, byte_834960[edi]'in cmp and 'mov     dl, byte_834960[eax]'in cmp
v=read(L/'r496-source-oracle.json')
for case in v['revision_cases']:
 fmt,path=schema205(case['version'],pe);assert fmt==case['source_format'] and path==case['source_branch_path'];assert version_body(case['version'])==case['version_body'] and actor_body(fmt,case['version'])==case['actor_body']
ctr=v['initial_counter'];wire=list(v['setup_bodies'][0]);assert v['setup_wire'][:7]==[7,0]+wire
wire=v['setup_wire'][:7]
for body in v['setup_bodies'][1:]:
 cipher,_=source_cipher(bytes(body),ctr,pe);wire+=list((len(body)+2).to_bytes(2,'little')+cipher);ctr=counter_add_adc(ctr,len(body))
assert wire==v['setup_wire'] and ctr==v['receive_after_setup']
for name,q in v['vectors'].items():
 cipher,_=source_cipher(bytes(q['plain']),q['receive_before'],pe);assert list((len(q['plain'])+2).to_bytes(2,'little')+cipher)==q['wire'] and counter_add_adc(q['receive_before'],len(q['plain']))==q['receive_after']
ctr=v['initial_counter']
for field,name in [('control_send_frame',b'NewAlias'),('control_second_send_frame',b'AltAlias')]:
 body=b'\xc9'+name+b'\0';cipher,_=source_cipher(body,ctr,pe);assert list((len(body)+2).to_bytes(2,'little')+cipher)==v[field];ctr=counter_add_adc(ctr,len(body))
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');window=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8');actor=(P/'src/modern_lito/native_source_actor.rs').read_text(encoding='utf-8')
assert 'pub(crate) struct Session'in actor and 'pub fn set_version'not in actor and 'pub fn set_actor'not in actor and 'source205 dynamic comparison table authority unqualified;actor not admitted'in actor
assert 'SourceCharacterControl {epoch:SourceEpoch,revision:u64,slot:usize}'in rt and 'control.revision==self.source_roster_revision'in rt and 'self.source_roster_revision.checked_add(1)'in rt and rt.count('self.source_actor=source_actor::Session::default();')==2
assert 'source_actor.prepare(event.body(),self.source_identity.selected().is_some(),self.source_flow.phase())'in rt and 'self.source_send.prepare_character(next.selected().unwrap().actor_alias())?'in rt
assert 'CHARACTER_FIRST:usize=13;const COUNT:usize=17'in window and '"角色 1","角色 2","角色 3","角色 4"'in window and 'source_character_command_slot((wparam.0&0xffff)as u16)'in window and 'r.handle_source_character_control(control)'in window and 'source_character_control_enabled(i-CHARACTER_FIRST)'in window
for n in ['build-r496-player-candidate.ps1','build-r496-source-actor-consumer.ps1']:
 s=(P/'tools'/n).read_text(encoding='utf-8');assert 'modern\\r495-native-send'in s and 'r496b-library-build.jsonl'in s
record=dict(state='R496_INDEPENDENT_SOURCE58_REVISION_REFUSAL_205_SCHEMA_209_PHASE_AND_OPAQUE_DISPLAYED_CONTROL14FLOWS_PASS',pin_instances=instances,unique_paths=len(cache),consumer_source_pairs=192,production_source_files=610,changed_from_R495=delta,new_source_files=['native_source_actor.rs'],source_ASM_fragments=len(e['fragments']),source_PE_fragments=len(e['PE_fragments']),source_symbol_contexts=len(e['contexts']),source_difference=snapshot['source_difference'],flow_cases=14,revision_boundary_cases=13,malformed_cases=1025,finite_consumer_executions=1,player_executions=0,normal_wire_connected=False,full_actor205_209_connected=False,GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r496-independent-contract.json',record)
save(L/'r496-candidate.json',dict(state='R496_SOURCE_RECEIVE_VERSION_REFUSAL_AND_DISPLAYED_TYPED_SELECTION_CANDIDATE',source_pins=c['source_pins'],production_library=player['production_library'],player=player['executable'],consumer={k:c[k]for k in ['path','bytes','sha256']},source_scope=read(L/'r496-source-scope.json'),independent=record,GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R496 independent PASS',instances,'pininstances',len(cache),'unique;192pairs610production/14flows13boundaries1025malformed')
