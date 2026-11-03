"""Static native fallback evidence. Never executes the original client or a probe."""
from pathlib import Path
from datetime import datetime,timezone
import struct,json,re,capstone,csv
from native_E149_recovery_contract import R,read,sha,encoded,require
T=R/'維護工具';V=R/'驗證紀錄/current'
def main():
 pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');asm=pe.parents[1]/'ida-client-oep-v2/full-disassembly.asm';require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9' and sha(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A','native source changed');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];section=e+24+struct.unpack_from('<H',b,e+20)[0];count=struct.unpack_from('<H',b,e+6)[0]
 def raw(va,n):
  for i in range(count):
   vs,rv,sz,off=struct.unpack_from('<IIII',b,section+40*i+8)
   if rv<=va-0x400000 and va-0x400000+n<=rv+min(vs,sz):return b[off+va-0x400000-rv:off+va-0x400000-rv+n]
  raise ValueError(hex(va))
 out=V/'native-E149-hide97-native'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);functions={};code={}
 for start,end in [(0x4adda0,0x4adf50),(0x44ede0,0x44eeb0),(0x44f350,0x450039),(0x4e5b50,0x4e5bf2),(0x4e5c00,0x4e63b8)]:
  p=out/f'{start:X}.bin';p.write_bytes(raw(start,end-start));rows=[dict(address=i.address,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str)for i in md.disasm(p.read_bytes(),start)];(out/f'{start:X}.json').write_bytes(encoded(rows));functions[f'{start:X}']=dict(start=start,end=end,bytes=end-start,sha256=sha(p));code.update({r['address']:r for r in rows})
 text=asm.read_text('utf-8-sig')
 for name in ['4ADDA0','44EDE0','44F350','4E5B50','4E5C00']:
  a=text.index('sub_'+name+' proc near');z=text.index('sub_'+name+' endp',a)+len('sub_'+name+' endp');(out/(name+'.asm.txt')).write_text(text[a:z]+'\n',encoding='utf-8',newline='\n')
 require(code[0x4e5ca7]['mnemonic']=='test' and code[0x4e5ca7]['operands']=='bl, bl','no-action branch differs');require(code[0x4e62a1]['mnemonic']=='mov' and code[0x4e62bc]['mnemonic']=='mov','static image fallback differs');require(any(r['mnemonic']=='mov'and r['operands']=='ecx, 0x460'for r in code.values()),'missing-SPR fallback differs')
 root=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/indices');indexes=[root/'Sprite.csv']+[root/f'Sprite{i:02}.csv'for i in range(16)];matches={name:[]for name in ['2941-0.spr','1120-0.spr','2942-0.spr']}
 for p in indexes:
  for row in csv.DictReader(p.open(encoding='utf-8-sig',newline='')):
   name=row['name'].lower()
   if name in matches:matches[name].append(dict(index=str(p),record=row))
 (out/'inspector.py').write_bytes(Path(__file__).read_bytes());p=out/'native-source.json';pins={str(p):sha(p)for p in out.iterdir()};p.write_bytes(encoded(dict(schema='E149-hide97-native-static-image-route-v1',source_pins=pins,external_readonly_pins={str(p):sha(p)for p in [pe,asm,*indexes]},functions=functions,cached_index_matches=matches,no_action_does_not_prove_no_pixels=True,native_no_action_zero_uses_static_frame_zero_lookup=True,native_missing_SPR_has_0x460_fallback=True,canonical_archive_absence_proven=False,hide2941_or_fallback_pixels_qualified=False,exact_effect_retirement_qualified=False,tests_executed=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(native_source=str(p),sha256=sha(p),functions=len(functions),cached_matches={n:len(rows)for n,rows in matches.items()},tests_executed=0),ensure_ascii=False))
if __name__=='__main__':main()
