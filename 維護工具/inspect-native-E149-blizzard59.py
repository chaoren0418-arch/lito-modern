"""Static original blizzard forest and formal centered request, no game execution."""
from pathlib import Path
from datetime import datetime,timezone
import json,re,struct,capstone
from native_E149_recovery_contract import R,P,SOURCE,FORMAL,read,sha,encoded,require,check_pins
T=R/'維護工具';V=R/'驗證紀錄/current'
def methods(p):
 s=Path(p).read_text('utf-8-sig');hs=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));return [(h.group().strip(),s[h.start():hs[i+1].start() if i+1<len(hs) else len(s)]) for i,h in enumerate(hs)]
def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'] and not w['active_workloads'] and not w['active_workload_sessions'],'development-only idle expected');ref=w['current_E149_berserk55_disease56_development'];require(sha(ref['qualification'])==ref['qualification_sha256'],'predecessor differs');q=read(ref['qualification']);check_pins(q['source_pins']);out=V/'native-E149-blizzard59-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 source=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(source)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','session differs');names=['useClassicWorldCenteredAreaAttackSpell','useCenteredAreaAttackSpell','nativeMaterialResurrectionBody','nativeMaterialWorldResurrectionBody','nativeResurrectionAnswerBody'];selected=[(h,s) for h,s in methods(source) if any(' '+n+'(' in h for n in names)];require(len(selected)==5,'centered/next methods differ')
 for i,(h,s) in enumerate(selected):(out/('method-'+str(i)+'.txt')).write_text(s,encoding='utf-8')
 for h,s in selected[:]:
  if ' useCenteredAreaAttackSpell(' in h:require('4: invokestatic  #28647' in s and '2942: invokestatic  #28084' in s and 'Protocol.selectorOnlySpellRequest:' in s and 'Protocol.attackMany:' in s,'battle centered branch differs')
  if ' useClassicWorldCenteredAreaAttackSpell(' in h:require('9: invokestatic  #28647' in s and '3962: invokestatic  #28084' in s and 'Protocol.selectorOnlySpellRequest:' in s and 'Protocol.attackMany:' in s,'world centered branch differs')
 text=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr');require(sha(text)=='523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127','Text differs');raw=text.read_bytes();m=re.search(rb'(?m)^#757[ \t]+.*?(?=^#|\Z)',raw,re.S|re.M);require(m is not None,'757 absent');(out/'757.definition.txt').write_bytes(m.group())
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9','PE differs');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];ss=e+24+struct.unpack_from('<H',b,e+20)[0]
 def at(a,n):
  for i in range(struct.unpack_from('<H',b,e+6)[0]):
   size,start,physical,pos=struct.unpack_from('<IIII',b,ss+i*40+8)
   if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return b[pos+a-0x400000-start:pos+a-0x400000-start+n]
  raise RuntimeError(hex(a))
 md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);decoded={}
 for name,lo,hi in [('start',0x44f350,0x450028),('offset',0x4b9390,0x4b93d4),('direction_init',0x4b8f40,0x4b8fc0)]:
  data=at(lo,hi-lo);rows=list(md.disasm(data,lo))
  if name!='start':
   end=next(i for i,x in enumerate(rows) if x.mnemonic=='ret');rows=rows[:end+1];data=data[:rows[-1].address+rows[-1].size-lo]
  require(sum(x.size for x in rows)==len(data) and all(x.address+x.size==y.address for x,y in zip(rows,rows[1:])),'partial instruction '+name);decoded[name]=rows;(out/(name+'.bin')).write_bytes(data);(out/(name+'.asm')).write_text('\n'.join(f'{x.address:08X} {x.bytes.hex()} {x.mnemonic} {x.op_str}' for x in rows)+'\n',encoding='utf-8')
 regs={};stores={}
 for x in decoded['direction_init']:
  if x.mnemonic=='ret':break
  if x.mnemonic=='xor':a,c=x.op_str.split(', ');require(a==c,'unexpected init xor');regs[a]=0
  elif x.mnemonic=='mov':
   a,c=x.op_str.split(', ');value=regs[c] if c in regs else int(c,0);value=value-(1<<32) if value>0x7fffffff else value
   if a.startswith('dword ptr ['):stores[int(a[11:-1],0)]=value
   else:regs[a]=value
  else:raise RuntimeError('unexpected initializer instruction')
 vectors=[(stores[0x768828+i*8],stores[0x76882c+i*8]) for i in range(8)];require(vectors==[(0,-1),(2,-1),(2,0),(2,1),(0,1),(-2,1),(-2,0),(-2,-1)],'native direction declarations differ')
 pushes=[];children=[]
 for x in decoded['start']:
  if not 0x44f63e<=x.address<0x44fe76:continue
  if x.mnemonic=='push':pushes.append((x.address,x.op_str))
  if x.mnemonic=='call' and x.op_str=='0x4b9390':
   args=pushes[-3:];radius=int(args[0][1],0);heading=0 if args[1][1]=='edi' else int(args[1][1],0);require(0<=heading<8 and radius in [3,5,7],'group vector differs');children.append(dict(ordinal=len(children),heading=heading,radius=radius,native_offset=[vectors[heading][0]*radius,vectors[heading][1]*radius],world_offset=[vectors[heading][0]*radius//2,vectors[heading][1]*radius],offset_call=x.address))
 require([(c['heading'],c['radius']) for c in children]==[(h,r) for h in range(8) for r in [5,3]]+[(h,7) for h in range(8)],'24-child construction order differs');entries=[x for x in decoded['start'] if 0x44fe76<=x.address<0x44fffb and x.mnemonic=='call' and x.op_str=='0x44ef80'];require(len(entries)==24,'child entry count differs');group=children+[dict(ordinal=24,heading=0,radius=0,native_offset=[0,0],world_offset=[0,0],root=True)];(out/'group.json').write_bytes(encoded(group));(out/'direction-declaration.json').write_bytes(encoded(dict(declared_native_vectors=vectors,uninitialized_PE_data=list(at(0x768828,64)),initializer_executed=False,world_x_scale=2)))
 jar=P/'target/current/server-reference/formal-server-E1498871.jar';require(sha(jar)==FORMAL,'formal differs');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p) for p in out.iterdir() if p.is_file()};pins.update({str(p):sha(p) for p in [source,jar]});record=dict(schema='E149-blizzard59-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,predecessor_qualification=ref,source_pins=pins,external_readonly_pins={str(text):sha(text),str(pe):sha(pe)},definitions=[d for d in q['next_definitions'] if d['skillId']==59],raw_definition=str(out/'757.definition.txt'),group=group,group_file=str(out/'group.json'),group_copies=25,child_target_lists_absent=True,root_target_list_only=True,raw_terminal_dwell_ticks=3,Sound123_source_frame=1,Hit_source_frame=3,direction_initializer_execution_qualified=False,per_node_constructor_clock_skew_qualified=False,next_skill61_requires_explicit_question321_answer=True,tests_executed=0,game_classes_loaded=0,game_methods_invoked=0,PE_executed=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),group_copies=25,tests_executed=0),ensure_ascii=False))
if __name__=='__main__':inspect()
