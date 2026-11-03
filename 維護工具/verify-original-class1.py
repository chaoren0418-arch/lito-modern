"""Independent recovered-PE decode of the class1 dispatch and callback path."""
from pathlib import Path
import hashlib,json,struct
import pefile,capstone
root=Path(__file__).resolve().parent.parent
p=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\runtime\LiTo.unpacked-oep-current.exe')
raw=p.read_bytes();digest=hashlib.sha256(raw).hexdigest().upper()
assert digest=='4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9'
pe=pefile.PE(data=raw,fast_load=True);base=pe.OPTIONAL_HEADER.ImageBase;md=capstone.Cs(capstone.CS_ARCH_X86,capstone.CS_MODE_32)
def decode(start,end):
 data=pe.get_data(start-base,end-start);ins=list(md.disasm(data,start))
 return {'start':hex(start),'end':hex(end),'bytes_sha256':hashlib.sha256(data).hexdigest().upper(),
         'instructions':[{'address':hex(i.address),'mnemonic':i.mnemonic,'operands':i.op_str,'bytes':i.bytes.hex()} for i in ins]}
ranges={name:decode(start,end) for name,start,end in [
 ('class1_dispatch',0x4a6234,0x4a6288),('constructor',0x44ede0,0x44eeb3),
 ('default_start',0x44fffb,0x450027),('direction_table_initializer',0x4b8f40,0x4b9000),
 ('direction_shift',0x4b9390,0x4b93d0),('effect_hit_callback',0x44f1e3,0x44f2f7)]}
assert any(i['mnemonic']=='call' and i['operands']=='0x44ede0' for i in ranges['class1_dispatch']['instructions'])
assert any(i['mnemonic']=='call' and i['operands']=='0x4b9390' for i in ranges['default_start']['instructions'])
assert any(i['mnemonic']=='push' and i['operands']=='2' for i in ranges['default_start']['instructions'])
assert any(i['mnemonic']=='call' and i['operands']=='0x4e8160' for i in ranges['effect_hit_callback']['instructions'])
# Decode the initialized direction constants rather than read zeroed .bss.
regs={};writes={}
for i in md.disasm(pe.get_data(0x4b8f40-base,0xc0),0x4b8f40):
 if i.mnemonic=='ret':break
 parts=i.op_str.split(', ')
 if i.mnemonic=='xor' and parts[0]==parts[1]:regs[parts[0]]=0
 if i.mnemonic=='mov':
  dst,src=parts;value=regs[src] if src in regs else int(src,0)
  if dst.startswith('dword ptr [0x'):writes[int(dst[11:-1],0)]=value&0xffffffff
  else:regs[dst]=value
def signed(v):return v if v<0x80000000 else v-0x100000000
native=[(signed(writes[0x768828+8*n]),signed(writes[0x76882c+8*n])) for n in range(8)]
assert native==[(0,-1),(2,-1),(2,0),(2,1),(0,1),(-2,1),(-2,0),(-2,-1)]
out={'status':'verified_static_class1_target_snapshot_two_heading_steps_and_hit_callback',
     'original_pe_sha256':digest,'ranges':ranges,'native_directions':native,
     'inference':'S23 class1 constructs the common finite effect using wire target coordinates and heading. Start offsets the stored position by two native grid steps. Its list.spr Hit flag resolves target IDs and requests action2. No flight or secondary sprite is constructed for gfx252.',
     'coordinates':'Native x=2*wireX-32768; direction x deltas are twice world-cell deltas.',
     'native_execution_verified':False,'gui_verified':False}
(root/'驗證紀錄/current/native-world-integration/original-class1-qualified.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print('Verified original class1 dispatcher, start offset, directions and Hit callback')
