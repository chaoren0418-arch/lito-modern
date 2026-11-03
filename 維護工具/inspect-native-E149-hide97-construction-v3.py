"""Refine four earlier truncated spans; disassemble bytes, never execute the PE."""
from pathlib import Path
from datetime import datetime,timezone
import struct,json,capstone
from native_E149_recovery_contract import R,read,sha,encoded,require,check_pins
V=R/'驗證紀錄/current'

def main():
 prior=V/'native-E149-hide97-native/20261024-185445-218054/native-source.json';require(sha(prior)=='E7EA514AD4CFDAE0C05C58BE60B1F3DD3F90251908B78540F8D586B6E78CD862','old source differs');old=read(prior);check_pins(old['source_pins']);pe=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe');asm=pe.parents[1]/'ida-client-oep-v2/full-disassembly.asm';require(sha(pe)=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9' and sha(asm)=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A','native differs');b=pe.read_bytes();e=struct.unpack_from('<I',b,60)[0];section=e+24+struct.unpack_from('<H',b,e+20)[0];count=struct.unpack_from('<H',b,e+6)[0]
 def raw(va,n):
  for i in range(count):
   vs,rv,sz,off=struct.unpack_from('<IIII',b,section+40*i+8)
   if rv<=va-0x400000 and va-0x400000+n<=rv+min(vs,sz):return b[off+va-0x400000-rv:off+va-0x400000-rv+n]
  raise ValueError(hex(va))
 out=V/'native-E149-hide97-construction-refinement'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);functions={};rejected=[]
 for start,end in [(0x4adda0,0x4adf60),(0x44ede0,0x44eec0),(0x44f350,0x450040),(0x4e5b50,0x4e5c00),(0x4e5c00,0x4e63c0)]:
  name=f'{start:X}';prefix=(prior.parent/(name+'.bin')).read_bytes();data=raw(start,end-start);require(data[:len(prefix)]==prefix,'old prefix differs');oldrows=read(prior.parent/(name+'.json'));oldtail=[r for r in oldrows if r['mnemonic']!='nop'][-1];p=out/(name+'.bin');p.write_bytes(data);instructions=list(md.disasm(data,start));require(sum(i.size for i in instructions)==len(data),'undecoded native tail');rows=[dict(address=i.address,hex=i.bytes.hex(),mnemonic=i.mnemonic,operands=i.op_str)for i in instructions];tail=[r for r in rows if r['mnemonic']!='nop'][-1];require(tail['mnemonic']in ['ret','retf'],'full function tail absent '+name);(out/(name+'.json')).write_bytes(encoded(rows));functions[name]=dict(start=start,end=end,bytes=len(data),sha256=sha(p),prior_span_bytes=len(prefix),prior_last_instruction=oldtail,new_last_instruction=tail,prior_prefix_identical=True)
  if oldtail['mnemonic']not in ['ret','retf']:rejected.append(name)
 require(rejected==['4ADDA0','44EDE0','4E5B50','4E5C00'],'prior missing return inventory differs');s=asm.read_text('utf-8-sig');a=s.index('sub_44EDE0 proc near');z=s.index('sub_44EEC0 proc near',a);text=s[a:z];require('sub_44EDE0 endp'in text and 'dw 458Bh'in text and '900020C2h'in text,'IDA construction data boundary differs');(out/'44EDE0-through-inline-data.asm.txt').write_text(text,encoding='utf-8',newline='\n');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};pins[str(prior)]=sha(prior);p=out/'refinement.json';p.write_bytes(encoded(dict(schema='E149-hide97-five-native-ranges-full-ret-refinement-v3',source_pins=pins,external_readonly_pins={str(pe):sha(pe),str(asm):sha(asm)},prior_source=str(prior),prior_source_sha256=sha(prior),rejected_prior_full_function_claims=rejected,functions=functions,all_old_prefixes_identical=True,all_new_ranges_decode_completely=True,all_new_ranges_end_in_return_ignoring_alignment_nops=True,IDA_constructor_endp_precedes_inline_code_data=True,requires_constructor_and_base_defaults_review=True,hide97_2941_scene_integrated=False,tests_executed=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(refinement=str(p),sha256=sha(p),refined_functions=5,prior_truncated=4,tests_executed=0,PE_executed=False),ensure_ascii=False))

if __name__=='__main__':main()
