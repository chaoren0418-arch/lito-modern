from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import re
old=read(L/'r495-source-evidence.json');s=Path(old['original_disassembly']['path']).read_text(encoding='utf-8');b=Path(old['analysis_PE']['path']).read_bytes();O=P/'target/r496-actor-study';O.mkdir(exist_ok=True)
for n in ['sub_4A33E0','sub_4A7F60','sub_4A9550','sub_4D3A80','sub_4D39F0','sub_4A2A50','sub_4B2970']:
 m=re.search('^'+n+r' proc near',s,re.M);z=re.search(r'^\w+ proc near',s[m.end():],re.M);(O/(n+'.asm')).write_text(s[m.start():m.end()+z.start()],encoding='utf-8')
cs=Cs(CS_ARCH_X86,CS_MODE_32)
for start,end in [(0x4A33E0,0x4A3508),(0x4D3A80,0x4D3ADB)]:
 p=O/f'pe-{start:X}.bin';p.write_bytes(b[start-0x400000:end-0x400000]);p.with_suffix('.disasm').write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(p.read_bytes(),start))+'\n',encoding='utf-8')
print((O/'sub_4A9550.asm').read_text(encoding='utf-8')[:11000])
