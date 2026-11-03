from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import re
O=P/'target/r496-actor-study';old=read(L/'r495-source-evidence.json');asm=Path(old['original_disassembly']['path']).read_text(encoding='utf-8');pe=Path(old['analysis_PE']['path']).read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32)
fragments=[dict(name=p.stem,**pin(p))for p in sorted(O.glob('sub_*.asm'))]
pefrags=[]
for start,end in [(0x4A33E0,0x4A36EC),(0x4A7F60,0x4A82AB),(0x4A83ED,0x4A8411),(0x4A9550,0x4A95A8),(0x4D39F0,0x4D3A80),(0x4D3A80,0x4D3ADB)]:
 p=O/f'pe-{start:X}.bin';p.write_bytes(pe[start-0x400000:end-0x400000]);d=p.with_suffix('.disasm');d.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(p.read_bytes(),start))+'\n',encoding='utf-8');pefrags.extend([pin(p),pin(d)])
lines=asm.splitlines();refs=[];proc=''
for i,l in enumerate(lines):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if any(x in l for x in ['byte_834A60','byte_834960','dword_63D54C','sub_4D39F0'])and not l.strip().startswith(';'):
  refs.append(dict(procedure=proc,line=i+1,text=l,context=lines[max(0,i-2):i+4]))
save(O/'version-comparison-lifecycle-refs.json',refs)
contexts=[]
for sym in ['dword_4F727C','aCdddddcc','aDscccdddcscsdc','aDscccdddcscsdc_0','aDscccdddcscsdc_1','aDscccdddcscsdc_2','aDscccdddcscsdc_3','aSc','dword_63D54C']:
 i=next(i for i,l in enumerate(lines)if l.startswith(sym+' '));p=O/(sym+'.asm');p.write_text('\n'.join(lines[i:i+8])+'\n',encoding='utf-8');contexts.append(pin(p))
save(L/'r496-source-evidence.json',dict(state='R496_SOURCE58_BRANCH_VERSION_AND205_SCHEMA_209_PHASE_GATE_COMPARISON_LIFECYCLE',previous=pin(L/'r495-native-character-send-partial-write-handoff.json'),original_disassembly=old['original_disassembly'],analysis_PE=old['analysis_PE'],fragments=fragments,PE_fragments=pefrags,contexts=contexts,refs=pin(O/'version-comparison-lifecycle-refs.json'),Native_PE_or_socket_execution=False))
print('R496 source evidence pinned:7 ASM/12 PE/9 contexts;no Native execution')
