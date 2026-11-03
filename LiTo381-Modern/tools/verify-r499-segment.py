from r491_common import *
import difflib,re,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_IMM,X86_OP_MEM
from r495_source_oracle import source_cipher,counter_add_adc
cache={};instances=0
def check(q):
 global instances
 instances+=1;p=q['path']
 if p not in cache:cache[p]=pin(p)
 assert cache[p]['bytes']==q['bytes']and cache[p]['sha256']==q['sha256'],p
def walk(v):
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():check(v)
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
names=['r499-source-study.json','r499-call-state-mapping.json','r499-initial-field-inspection.json','r499-source-oracle.json','r499-production-source-snapshot.json','r499-source-scope.json','r499-entry-source-contract.json','r499-player-candidate.json','r499-consumer-candidate.json','r499-consumer-exit.json','r499-event-details.json']
for n in names:walk(read(L/n))
study=read(L/'r499-source-study.json');asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(study['analysis_PE']['path']).read_bytes();assert len(study['fragments'])==38
cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True
for q in study['fragments']:
 p=Path(q['path'])
 if 'name'in q:
  m=re.search('^'+q['name']+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);assert p.read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()];assert q['line_start']==asm[:m.start()].count('\n')+1
 else:
  lo,hi=[int(x,16)for x in p.stem.split('-')];raw=pe[lo-0x400000:hi-0x400000]
  if p.suffix=='.bin':assert raw==p.read_bytes()
  else:assert p.read_text(encoding='utf-8')=='\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo))
code=list(cs.disasm(pe[0x6B670:0x6B6CD],0x46B670));calls=[i.operands[0].imm for i in code if i.mnemonic=='call'];assert calls==[0x4C6AE0,0x46D2E0,0x49A510,0x496DA0,0x4962B0]
assert code[0].mnemonic=='push'and code[0].operands[0].imm==15;assert code[1].operands[0].imm==0x5025A8 and pe[0x1025A8:0x1025AA]==b'c\0'
phase=[i for i in code if i.mnemonic=='mov'and i.operands[0].type==X86_OP_MEM and i.operands[0].mem.disp==0x64BBD4];assert len(phase)==1 and phase[0].operands[1].imm==5
def source(n):return(P/f'target/r499-back-login-study/sub_{n}.asm').read_text(encoding='utf-8')
assert source('493150').index('sub_496E30')<source('493150').index('sub_46B670')
for sym in ['dword_6414A4','dword_6414A0','dword_64149C','byte_75F6E5']:assert sym in source('496E30')and sym in source('496DA0')
for sym in ['dword_760054','dword_760058','5B10318h']:assert sym in source('49A510')
assert '64BBD4'not in source('49A510');assert 'dword_8343E8'in source('4C65F0')and 'sub_4EFE9E'in source('4C65F0')
assert 'dword_636584'in source('480560')and 'aFromcd'in source('480560')and 'aChdch'in source('480560')
assert struct.unpack('<I',pe[0x236584:0x236588])[0]==0xFFFFFFFF and struct.unpack('<I',pe[0x1033C4:0x1033C8])[0]==0x114F8
version_code=list(cs.disasm(pe[0xA33E0:0xA36F0],0x4A33E0));mem={o.mem.disp for i in version_code for o in i.operands if o.type==X86_OP_MEM};imms={o.imm for i in version_code for o in i.operands if o.type==X86_OP_IMM}
assert {0x75F755,0x8343E8}<=mem and {0x4EFE50,0x641348,0x6412A0,0x632620}<=imms;assert pe[0x232620:0x232625]==b'cssd\0'
o=read(L/'r499-source-oracle.json');ctr=o['seed'];wire=[]
def frame(body,ctr):
 raw=bytes(body);cipher,_=source_cipher(raw,ctr,pe);return list((len(raw)+2).to_bytes(2,'little')+cipher)
for i,body in enumerate(o['setup_bodies']):
 wire+=list((len(body)+2).to_bytes(2,'little')+bytes(body))if i==0 else frame(body,ctr)
 if i:ctr=counter_add_adc(ctr,len(body))
assert wire==o['setup_wire']and ctr==o['receive_after_setup'];assert frame(o['late_version_body'],ctr)==o['late_version_wire'];ctr=counter_add_adc(ctr,len(o['late_version_body']));assert ctr==o['receive_after_late'];wire=[]
for body in o['new_roster_bodies']:wire+=frame(body,ctr);ctr=counter_add_adc(ctr,len(body))
assert wire==o['new_roster_wire']and ctr==o['receive_after_roster'];ctr=o['seed']
for v in o['send_vectors']:assert v['before']==ctr and frame(v['plain'],ctr)==v['wire'];ctr=counter_add_adc(ctr,len(v['plain']));assert v['after']==ctr
assert [v['plain'][0]for v in o['send_vectors']]==[201,15,201,15]
mode=o['late_mode_after_setup'];assert mode['before']==o['receive_after_setup']and frame(mode['plain'],mode['before'])==mode['wire'];assert mode['after']==counter_add_adc(mode['before'],len(mode['plain']));assert frame([58,1],mode['after'])==o['refusal_after_mode']['wire']
production=read(L/'r499-production-source-snapshot.json');old=read(L/'r498-production-source-snapshot.json');newby={q['path']:q for q in production['source_pins']};assert len(newby)==612 and set(newby)=={q['path']for q in old['source_pins']};delta=[];diffs=[]
for q in old['source_pins']:
 check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
 if newby[q['path']]['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert len(delta)==7;assert '\n'.join(diffs)==Path(production['source_difference']['path']).read_text(encoding='utf-8')
for q in production['source_pins']:check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
candidate=read(L/'r499-consumer-candidate.json');player=read(L/'r499-player-candidate.json');assert len(candidate['source_pins'])==200 and candidate['production_library']==player['production_library']and player['candidate_executions']==0
for q in candidate['source_pins']:check(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
assert read(L/'r499-consumer-exit.json')['exit_code']==0;tests=read(L/'r499-event-details.json');assert tests['flow_cases']==10 and tests['independent_PE_cipher_send_vectors']==4 and tests['retained_RX_partial_body']and tests['historical_PASS_reruns']==0 and tests['GUI']==0
rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');win=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8')
for term in ['control.revision==self.source_roster_revision&&control.scene_revision==self.character_scene.key().scene_revision','prepare_return_login()?','awaiting_source_roster=true','ensure!(!self.awaiting_source_roster','source_flow.returned_to_login()']:assert term in rt
assert 's.runtime.borrow_mut().as_mut().unwrap().handle_source_character_back(control)'in win and 'BACK=>!entry||s.character_back.borrow().is_some()'in win
save(L/'r499-candidate.json',dict(state='R499_FIXED_LIBRARY_PLAYER_AND_NEW_BACK_CONSUMER_SAME_SOURCE',source_pins=candidate['source_pins'],production_library=player['production_library'],player=player['executable'],consumer=dict(path=candidate['path'],bytes=candidate['bytes'],sha256=candidate['sha256']),source_pairs=200,GUI=0,LOADED=False))
save(L/'r499-independent-contract.json',dict(state='R499_INDEPENDENT_PIN_DIFF_SOURCE_ORDER_CIPHER_RUNTIME_VERIFICATION_PASS',pin_instances=instances,unique_paths=len(cache),source_pairs=200,production_files=612,source_difference=production['source_difference'],changed=delta,added=[],unchanged_previous_sources=605,source_fragments=38,flows=10,source_cipher_wire_vectors=4,player_executions=0,consumer_executions=1,historical_PASS_reruns=0,GUI=0,LOADED=False,verified_json=[pin(L/n)for n in names]))
print('R499 independentPASS',instances,'pins',len(cache),'unique;200pairs612sources/7changed/10flows4wirevectors')
