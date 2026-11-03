"""Verify the pinned PE's frame-delay arithmetic, without executing the client."""
from pathlib import Path
import hashlib,json
import pefile,capstone
R=Path(__file__).resolve().parents[1];out=R/'驗證紀錄/current/native-effect-native-clock'
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe');raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper();assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);decoder=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32);start=0x43e888;data=pe.get_data(start-pe.OPTIONAL_HEADER.ImageBase,0x10);instructions=list(decoder.disasm(data,start))
assert [(i.mnemonic,i.op_str) for i in instructions]==[('lea','eax, [eax + eax*4]')]*3+[('shl','eax, 3'),('cdq',''),('idiv','dword ptr [ebp - 0x1c]')]
def execute(ticks,fps):
 eax=ticks;edx=0
 for ins in instructions:
  if ins.mnemonic=='lea':eax=(eax+eax*4)&0xffffffff
  elif ins.mnemonic=='shl':eax=(eax<<3)&0xffffffff
  elif ins.mnemonic=='cdq':edx=0xffffffff if eax&0x80000000 else 0
  elif ins.mnemonic=='idiv':
   value=(edx<<32)|eax
   if value&(1<<63):value-=1<<64
   eax=(abs(value)//fps)*(-1 if value<0 else 1)
 return eax
samples=[{'ticks':n,'fps':24,'native_delay_ms':execute(n,24)} for n in [0,1,2,3,4,5,6,7,8,12,24,1440]]
assert [x['native_delay_ms'] for x in samples]==[0,41,83,125,166,208,250,291,333,500,1000,60000]
# This is an inference from verified timer insert/pump instructions, not a claim
# that the full native event loop ran. Each supplied millisecond is one dispatch.
def timer(ticks,dispatches):
 delays=[execute(n,24) for n in ticks];frame=1 if len(ticks)>1 else 0;due=delays[frame];finished=False;rows=[]
 for now in dispatches:
  count=0
  while due is not None and due<=now:
   count+=1;assert count<=len(ticks)
   if frame==0:finished=True;due=None;break
   frame=(frame+1)%len(ticks);due=now+delays[frame]
  rows.append({'now_ms':now,'source_frame':frame,'finished':finished,'next_callback_ms':due})
 return rows
vectors=[{'ticks':[2,1,3],'dispatches_ms':[0,40,41,165,166,248,249]},{'ticks':[2,1,3],'dispatches_ms':[0,120,120,244,245,327,328]},{'ticks':[1,0,1],'dispatches_ms':[0,40,41,81,82]}]
for v in vectors:v['samples']=timer(v['ticks'],v['dispatches_ms'])
assert vectors[0]['samples'][2]['source_frame']==2 and vectors[1]['samples'][1]['next_callback_ms']==245 and vectors[2]['samples'][0]['source_frame']==2
proof={'status':'verified_PE_parser_slice_integer_delay_and_inferred_timer_vectors','original_pe':str(p),'original_pe_sha256':digest,'start':hex(start),'bytes':data.hex(),'bytes_sha256':hashlib.sha256(data).hexdigest().upper(),'instructions':[{'va':hex(i.address),'mnemonic':i.mnemonic,'operands':i.op_str,'bytes':i.bytes.hex()} for i in instructions],'opcode_interpreted_samples':samples,'inferred_per_object_timer_vectors':vectors,'original_native_client_executed':False,'global_cross_object_timer_order_verified':False,'whole_goal_complete':False,'validator_sha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest().upper()}
(out/'native-delay-vectors.json').write_text(json.dumps(proof,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(proof['status'])
