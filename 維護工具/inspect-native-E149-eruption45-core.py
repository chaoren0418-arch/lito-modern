"""Read-only, gap-free primary-code and shared-Text inspection; never run game code."""
from pathlib import Path
from datetime import datetime, timezone
import capstone, json, re, struct
from native_E149_recovery_contract import R, SOURCE, FORMAL, read, sha, encoded, require

V=R/'驗證紀錄/current'
PE=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')
TEXT=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/decoded/Text/row15937-list.spr')
PRIMARY={str(PE):'4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9',str(TEXT):'523C61AD5CC362A56A36F58B262C84EFA8B4435F017E7074D35B5834E8B9F127'}

def inspect():
 w=read(R/'工作佇列.json');require(w['current_user_requested_goal']['test_execution_deferred_by_user'],'development-only phase expected')
 parent=w['current_E149_zombie41_development'];require(sha(parent['qualification'])==parent['qualification_sha256'],'predecessor differs')
 q=read(parent['qualification']);require(q['next_definition']['skillId']==45,'catalog successor differs')
 for p,h in PRIMARY.items():require(sha(p)==h,'primary source differs')
 b=PE.read_bytes();e=struct.unpack_from('<I',b,60)[0];sections=e+24+struct.unpack_from('<H',b,e+20)[0]
 def raw(a,n):
  for i in range(struct.unpack_from('<H',b,e+6)[0]):
   size,start,physical,at=struct.unpack_from('<IIII',b,sections+i*40+8)
   if start<=a-0x400000 and a-0x400000+n<=start+min(size,physical):return b[at+a-0x400000-start:at+a-0x400000-start+n]
  raise ValueError(hex(a))
 ranges={'constructor7':(0x4238f0,0x4239fc),'advance7':(0x423a40,0x423cd0),'timer7':(0x423cd0,0x423ce1),'entry7':(0x423cf0,0x423d9d),
  'constructor6':(0x4235f0,0x4236b7),'advance6':(0x423700,0x423890),'timer6':(0x423890,0x4238a1),'entry6':(0x4238b0,0x4238f0),
  'heading':(0x4b9210,0x4b92d0),'vector':(0x4b92d0,0x4b9356),'increment':(0x4e5b50,0x4e5c00),
  'wire':(0x4a5ed0,0x4a5fe0),'factory7':(0x4a60ff,0x4a6166),'fresh_identity':(0x4e78b0,0x4e78dd)}
 out=V/'native-E149-eruption45-core-inspection'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
 md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);decoded={};files={}
 for name,(lo,hi) in ranges.items():
  data=raw(lo,hi-lo);rows=list(md.disasm(data,lo));require(sum(i.size for i in rows)==len(data),'partial instruction range '+name)
  require(rows[0].address==lo and all(a.address+a.size==b.address for a,b in zip(rows,rows[1:])),'instruction gap '+name)
  (out/(name+'.bin')).write_bytes(data);(out/(name+'.asm')).write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in rows)+'\n',encoding='utf-8');decoded[name]=rows
  for ext in ['.bin','.asm']:files[str(out/(name+ext))]=sha(out/(name+ext))
 expected={'advance7':[('test','al, al'),('call','0x4e5b50'),('or','edx, 5'),('call','0x4d65f0'),('cmp','al, 2'),('call','0x4238f0'),('and','ecx, 0xfffffffe'),('cmp','dword ptr [esi + 0x154], 1'),('call','0x4235f0')],
  'advance6':[('call','0x4e5b50'),('or','edx, 5'),('call','0x4e8160'),('call','0x4e8230'),('call','0x4d65f0'),('mov','byte ptr [esi + 0x13c], 1')],
  'vector':[('and','edi, 0xfffffffe'),('sar','esi, 1'),('shl','eax, 0x10'),('idiv','esi'),('idiv','ecx'),('shl','ecx, 1')]}
 for name,ops in expected.items():
  actual=[(i.mnemonic,i.op_str) for i in decoded[name]]
  for op in ops:require(op in actual,'primary operation missing '+repr(op))
 text=TEXT.read_bytes();selected={}
 for identifier in [129,130,123,124]:
  match=re.search(rb'(?m)^#'+str(identifier).encode()+rb'[ \t]+.*?(?=^#|\Z)',text,re.S|re.M);require(match is not None,'definition absent');selected[str(identifier)]=match.group().decode('latin1').strip()
 (out/'shared-definitions.json').write_bytes(encoded(selected));files[str(out/'shared-definitions.json')]=sha(out/'shared-definitions.json')
 session=V/'native-formal-reference-refresh/20261023-192435-953238/ClientSession.stdout.txt';require(sha(session)=='DE8F393A4E77FF15478DDE258E45E14F680EC9C15BC7FADDF3D5F33B7A333B25','formal session differs')
 s=session.read_text('utf-8-sig');headers=list(re.finditer(r'^  [^ \n].*\([^\n]*\)[^\n]*;\s*$',s,re.M));methods=[]
 for i,h in enumerate(headers):
  if re.search(r'\b(useAttackSpell|useClassicWorldAttackSpell)\(',h.group()):
   body=s[h.start():headers[i+1].start() if i+1<len(headers) else len(s)];p=out/('formal-overload-'+str(len(methods))+'.txt');p.write_text(body,encoding='utf-8');files[str(p)]=sha(p);methods.append(dict(header=h.group().strip(),path=str(p),sha256=sha(p)))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());files[str(out/'inspector.py')]=sha(out/'inspector.py')
 record=dict(schema='E149-eruption45-core-primary-v1',source_manifest_sha256=SOURCE,formal_target_sha256=FORMAL,original_pins={**PRIMARY,str(session):sha(session)},predecessor_qualification=parent['qualification'],predecessor_sha256=parent['qualification_sha256'],definition=q['next_definition'],files=files,overloads=methods,
  scope='gap-free disassembly and raw shared definitions only; overloads retained individually; no per-skill branch acceptance',
  coordinate_policy='u32 fixed16 wrapping, logical rounding shift and even native X; vector dominant distance uses abs nativeX/2 but remaining uses abs nativeX',
  timing_policy='class7 terminal check before advance; class6 draws before terminal check; each node independent; source0 dwell retained',
  limitations=['Analysis PE is not a runnable original.','Exact #45 per-skill dispatch, shared resource loading, hurt relative graphics binding and runtime scene integration remain pending.'],tests_executed=0,desktop_inputs=0,outputs_executed=False,whole_goal_complete=False)
 (out/'inspection.json').write_bytes(encoded(record));print(json.dumps(dict(inspection=str(out/'inspection.json'),sha256=sha(out/'inspection.json'),ranges=len(ranges)),ensure_ascii=False))

if __name__=='__main__':inspect()
