from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import re,struct,difflib
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['sha256']==v['sha256']and cache[p]['bytes']==v['bytes'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
names=['r501-source-study.json','r501-source-oracle.json','r501-event-state-data-mapping.json','r501-open-gaps.json','r501-production-source-snapshot.json','r501-source-scope.json','r501-entry-source-contract.json','r501-player-candidate.json','r501-consumer-candidate.json','r501-consumer-exit.json','r501-event-details.json','r501a-build-failure.json','r501-consumer-build-failure.json','r501-pre-coalesced-validation.json','r501-verifier-attempt1-failure.json','r501-coalesced-oracle.json','r501-final-incremental-details.json']
for n in names:walk(read(L/n))
study=read(L/'r501-source-study.json');asm=Path(study['original_disassembly']['path']).read_text(encoding='utf-8');asm_lines=asm.splitlines();pe=Path(study['analysis_PE']['path']).read_bytes();assert len(study['fragments'])==50
cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True
for q in study['fragments']:
 p=Path(q['path'])
 if 'name'in q:
  m=re.search('^'+q['name']+r' proc near',asm,re.M);z=re.search(r'^sub_[0-9A-F]+ proc near',asm[m.end():],re.M);assert p.read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()];assert q['line_start']==asm[:m.start()].count('\n')+1
 elif p.name.startswith('startup-producer-line'):
  lines=p.read_text(encoding='utf-8').splitlines()
  for line in lines:
   n,s=line.split(': ',1);assert asm_lines[int(n)-1]==s
 elif p.name=='492010-fixed-salt.bin':assert p.read_bytes()==pe[int(q['address'],16)-0x400000:int(q['address'],16)-0x400000+p.stat().st_size]
 else:
  lo,hi=[int(x,16)for x in p.stem.split('-')[:2]];raw=pe[lo-0x400000:hi-0x400000]
  if p.suffix=='.bin':assert raw==p.read_bytes()
  else:assert p.read_text(encoding='utf-8')=='\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo))
def source(n):return(P/f'target/r501-normal-auth-send-study/sub_{n}.asm').read_text(encoding='utf-8')
for term in ['offset nullsub_48','0B104FA87h','[edi+edx*4]','[ebp+ecx+var_3C]','sub_490390','sub_492010','byte_64BBF8','aCssd','aCssds']:assert term in source('492090'),term
for term in ['sub_49A770','sub_49AAC0','sub_49A940','sub_49AE10']:assert term in source('49AD70'),term
for term in ['add     edx, 6','20h','[ebp+var_F4], 0']:assert term in source('49AE10'),term
for term in ['[edi+210h], 6','35Ch','sub_491E80']:assert term in source('491ED0'),term
assert 'mov     edi, 10h'in source('491FE0')and'30h'in source('491FE0')
assert 'push    10h'in source('492010')and'48E500'in source('492010')and'48E5B0'in source('492010')
digest_instructions=list(cs.disasm(pe[0x92135:0x92181],0x492135));assert all('[ebp - 0x10]'not in i.op_str for i in digest_instructions)
for n in ['492190','492270']:assert 'sub_4EFE50'in source(n)and'sub_492090'in source(n)
assert 'sub_480560'in source('4AE050')and'87546CA1h'in source('4AE050')
for term in ['dword_5033C4','dword_636584','8033h','aChdch','mov     bl, 2']:assert term in source('480560'),term
assert pe[0x236584:0x236588]==b'\xff'*4 and pe[0x24BBF8]==0
o=read(L/'r501-source-oracle.json');assert o['environment_digest_not_in_login_varargs']and o['normal_body_length_range']==[13,47]
for profile in o['profiles']:
 env=profile['environment'];assert env['label'].startswith('R501_SYNTHETIC_');compat=bytes(profile['send_vectors'][0]['plain']);assert compat==struct.pack('<BHI BH'.replace(' ',''),0,0x8033,env['code_page'],2 if env['special']else env['from_cd_registry_dword']&255,env['revision_word_source']&0xFFFF)
 for vec in profile['send_vectors']:
  plain=bytes(vec['plain']);frame=bytes(vec['wire']);assert frame[:2]==struct.pack('<H',len(plain)+2);key=vec['before'].to_bytes(8,'little');decoded=bytes(c^key[i&7]^(frame[2+i-1]if i else 0)for i,c in enumerate(frame[2:]));assert decoded==plain;assert vec['after']==(vec['before']+len(plain))&0xFFFFFFFFFFFFFFFF
  if 'login'in vec['name']:assert plain[-(len(env['supplemental'])+1+4):-len(env['supplemental'])-1]==bytes([127,0,0,1])if env['special']else plain[-4:]==bytes([127,0,0,1])
production=read(L/'r501-production-source-snapshot.json');old=read(L/'r500-production-source-snapshot.json');newby={q['path']:q for q in production['source_pins']};assert len(newby)==615;delta=[];diffs=[]
for q in old['source_pins']:
 walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
 if newby[q['path']]['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
added=['native_login_wire.rs','native_login_transport.rs']
for name in added:
 p=P/'src/modern_lito'/name;diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
assert '\n'.join(diffs)==Path(production['source_difference']['path']).read_text(encoding='utf-8')and production['source_difference']['bytes']==39037
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs','native_login_form.rs','native_source_send.rs'}
for q in production['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
candidate=read(L/'r501-consumer-candidate.json');player=read(L/'r501-player-candidate.json');assert len(candidate['source_pins'])==207 and candidate['production_library']==player['production_library']and player['candidate_executions']==0
for q in candidate['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
tests=read(L/'r501-event-details.json');assert tests['flow_cases']==21 and read(L/'r501-consumer-exit.json')['exit_code']==0
assert tests['consumer_executions']==2 and tests['actual_server_received_exact_frames']==12 and tests['reported_failure_reservations']==2 and tests['actual_socket_EOF']==1 and not tests['Complete_is_authsuccess']and not tests['production_authentication_connected']and tests['GUI']==0
tails=[q['details']['reader']for q in tests['trace']if q['name'].startswith('nine actual58')];assert len(tails)==2 and all(q['events']==9 and q['max_batch_events']==8 and q['max_queued_tail']==32 for q in tails)
form=(P/'src/modern_lito/native_login_form.rs').read_text(encoding='utf-8');transport=(P/'src/modern_lito/native_login_transport.rs').read_text(encoding='utf-8');rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8')
for term in ['Zeroizing<[u8;28]>','Zeroizing<[u8;14]>','Stage::AwaitingReply','self.clear();','Arc::ptr_eq']:assert term in form
for term in ['stream.local_addr()','stream.peer_addr()','peer.ip().is_loopback()','stream.write(','t.stream.read(','end_source_stream_candidate','data[batch.consumed..]']:assert term in transport,term
assert 'login_transport=None;self.login_form.retire()'in rt
save(L/'r501-candidate.json',dict(state='R501_FIXED_LIBRARY_PLAYER_NEW_TYPED_LOOPBACK_CONSUMER_SAME_SOURCE',source_pins=candidate['source_pins'],production_library=player['production_library'],player=player['executable'],consumer=dict(path=candidate['path'],bytes=candidate['bytes'],sha256=candidate['sha256']),source_pairs=207,GUI=0,LOADED=False))
save(L/'r501-independent-contract.json',dict(state='R501_SOURCE_FRAGMENTS_BRANCH_VARARGS_WIRE_DIFFERENCE_AND_LIFECYCLE_INDEPENDENT_PASS',pin_instances=instances,unique_paths=len(cache),verified_json=[pin(L/n)for n in names],source_pairs=207,production_files=615,source_difference=production['source_difference'],changed=delta,added=added,unchanged_previous_sources=609,source_fragments=50,flows=21,preliminary_flows=18,final_source_incremental_flows=3,actual_exact_socket_frames=12,reported_failures=2,actual_EOF=1,actual_retained_tails=2,consumer_executions=2,final_fixed_consumer_executions=1,player_executions=0,historical_PASS_reruns=0,GUI=0,LOADED=False))
print('R501 independentPASS',instances,'pins',len(cache),'unique/207pairs615source/21flows/12actualframes/39037bytediff')
