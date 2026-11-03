from r491_common import *
import re,difflib,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
from capstone.x86 import X86_OP_MEM
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;q=v;p=q['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==q['bytes']and cache[p]['sha256']==q['sha256'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
names=['r500-source-study.json','r500-login-input-oracle.json','r500-event-state-data-mapping.json','r500-production-source-snapshot.json','r500-source-scope.json','r500-entry-source-contract.json','r500-player-candidate.json','r500-consumer-candidate.json','r500-consumer-exit.json','r500-event-details.json']
for n in names:walk(read(L/n))
e=read(L/'r500-source-study.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(e['analysis_PE']['path']).read_bytes();assert len(e['fragments'])==66
cs=Cs(CS_ARCH_X86,CS_MODE_32);cs.detail=True
for q in e['fragments']:
 p=Path(q['path'])
 if 'name'in q:
  m=re.search('^'+q['name']+r' proc near',asm,re.M);z=re.search(r'^sub_[0-9A-F]+ proc near',asm[m.end():],re.M);assert p.read_text(encoding='utf-8')==asm[m.start():m.end()+z.start()];assert q['line_start']==asm[:m.start()].count('\n')+1
 elif 'dispatch_jump'in q:
  table=int(q['table'],16);mapva=int(q['map'],16);assert p.read_bytes()==pe[table-0x400000:table-0x400000+640]+pe[mapva-0x400000:mapva-0x400000+256]
 elif 'table'in q:assert p.read_bytes()==pe[0x231E38:0x231E38+300]
 else:
  lo,hi=[int(x,16)for x in p.stem.split('-')];raw=pe[lo-0x400000:hi-0x400000]
  if p.suffix=='.bin':assert raw==p.read_bytes()
  else:assert p.read_text(encoding='utf-8')=='\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo))
for q in e['archived_global_observations']['lines']:assert asm.splitlines()[q['line']-1]==q['text']
assert pe[0x236584:0x236588]==b'\xff'*4 and pe[0x35F754:0x35F757]==bytes(3);assert any('3B6h'in q['text']for q in e['archived_global_observations']['lines'])
oracle=read(L/'r500-login-input-oracle.json')
for index,(row,t)in enumerate(zip(oracle['rows'],[7,7,2,2,10])):assert row[0]==t and tuple(row)==struct.unpack_from('<15I',pe,0x231E38+60*index)
assert oracle['rows'][2][4]==0x492270
for t in oracle['dispatcher']:
 jump=int(t['jump'],16);i=next(cs.disasm(pe[jump-0x400000:jump-0x400000+7],jump));assert i.operands[0].type==X86_OP_MEM and i.operands[0].mem.disp==int(t['table'],16)
 for opcode,q in t['handlers'].items():assert q['index']==pe[int(t['map'],16)-0x400000+int(opcode)];assert int(q['handler'],16)==struct.unpack_from('<I',pe,int(t['table'],16)-0x400000+q['index']*4)[0]
 assert t['handlers']['58']['handler']=='0x4b306f'and t['handlers']['59']['handler']=='0x4b307b'and t['handlers']['203']['handler']=='0x4b3093'
assert oracle['dispatcher'][1]['handlers']['3']['handler']=='0x4b3175'
def source(n):return(P/f'target/r500-login-lifecycle-study/sub_{n}.asm').read_text(encoding='utf-8')
for term in ['push    1Ch','push    0Ch','sub_4C3CC0','sub_4C3D20','dword_641348']:assert term in source('41A430')
for term in ['push    0Eh','push    0Ch','dword_6412A0']:assert term in source('41A490')
assert 'mov     edx, 1Bh'in source('492540')and'cmp     edi, 2'in source('401680')
for term in ['cmp     ecx, 4','cmp     ecx, 0Dh']:assert term in source('48FEA0')
for term in ["cmp     al, 61h",'cmp     al, 7Ah','sub     al, 20h','mov     [edx], al']:assert term in source('4D3860')
for term in ['sub_4C3D30','rep stosd','sub_440E90','byte_75F754','sub_492190']:assert term in source('492270')
for term in ['mov     byte_75F755, 0','mov     byte_75F756, 0','sub_48F580','dword_75F758']:assert term in source('48FAA0')
assert 'lineage.cfg'in source('41A4E0')and 'fn_LoadStringTable_Check'in source('4D4840')
production=read(L/'r500-production-source-snapshot.json');old=read(L/'r499-production-source-snapshot.json');newby={q['path']:q for q in production['source_pins']};assert len(newby)==613;delta=[];diffs=[]
for q in old['source_pins']:
 walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
 if newby[q['path']]['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
new=P/'src/modern_lito/native_login_form.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))));assert '\n'.join(diffs)==Path(production['source_difference']['path']).read_text(encoding='utf-8');assert len(delta)==3
for q in production['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
candidate=read(L/'r500-consumer-candidate.json');player=read(L/'r500-player-candidate.json');assert len(candidate['source_pins'])==203 and candidate['production_library']==player['production_library']and player['candidate_executions']==0
for q in candidate['source_pins']:walk(dict(path=q['snapshot'],bytes=q['bytes'],sha256=q['sha256']))
tests=read(L/'r500-event-details.json');assert read(L/'r500-consumer-exit.json')['exit_code']==0 and tests['flow_cases']==9 and tests['normalized_source_cases']==6 and tests['input_boundary_cases']==11 and tests['GUI']==0 and tests['historical_PASS_reruns']==0
form=new.read_text(encoding='utf-8');rt=(P/'src/modern_lito/native_city_runtime.rs').read_text(encoding='utf-8');win=(P/'src/modern_lito/native_city_window.rs').read_text(encoding='utf-8')
for term in ['Zeroizing<[u8;28]>','Zeroizing<[u8;14]>','Arc::ptr_eq','self.password.zeroize()','self.staged.take()']:assert term in form
assert 'derive(Clone)'not in form[form.index('struct Staged'):]
for term in ['ES_PASSWORD','EM_SETLIMITTEXT','EN_CHANGE','login_publishing','Zeroizing::new([0u16;14])','stage_source_login_form','cancel_source_login_form','open_source_login_form']:assert term in win
stage=rt[rt.index(' pub fn stage_source_login_form'):rt.index(' pub fn cancel_source_login_form')];assert 'source_send'not in stage and 'source_flow'not in stage
save(L/'r500-candidate.json',dict(state='R500_FIXED_LIBRARY_PLAYER_AND_NEW_LOGIN_LIFECYCLE_CONSUMER_SAME_SOURCE',source_pins=candidate['source_pins'],production_library=player['production_library'],player=player['executable'],consumer=dict(path=candidate['path'],bytes=candidate['bytes'],sha256=candidate['sha256']),source_pairs=203,GUI=0,LOADED=False))
save(L/'r500-independent-contract.json',dict(state='R500_SOURCE_PIN_TABLE_DISPATCH_INPUT_LIMIT_DIFF_SECRET_LIFECYCLE_VERIFICATION_PASS',pin_instances=instances,unique_paths=len(cache),source_pairs=203,production_files=613,source_difference=production['source_difference'],changed=delta,added=[new.name],unchanged_previous_sources=609,source_fragments=66,flows=9,normalized_source_cases=6,boundary_cases=11,player_executions=0,consumer_executions=1,historical_PASS_reruns=0,GUI=0,LOADED=False,verified_json=[pin(L/n)for n in names]))
print('R500 independentPASS',instances,'pins',len(cache),'unique/203pairs613source/9flows/6norm11boundary')
