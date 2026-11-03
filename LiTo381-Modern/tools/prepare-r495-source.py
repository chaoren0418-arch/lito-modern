from r491_common import *
import re
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
O=P/'target/r495-native-send-study';O.mkdir(parents=True,exist_ok=True)
old=read(L/'r494-source-evidence.json');a=Path(old['original_disassembly']['path']);assert pin(a)==old['original_disassembly'];s=a.read_text(encoding='utf-8');ls=s.splitlines();names={'sub_480560','sub_480400','sub_4803A0','sub_480620','sub_4C6770','sub_4C6700','sub_4C68D0','sub_4C6AE0','sub_4C65F0','sub_4AE050','sub_4B2970','sub_4A7450'};refs=[];proc=''
for i,l in enumerate(ls):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if 'sub_480560'in l and not l.strip().startswith(';'):refs.append(dict(procedure=proc,line=i+1,text=l,context=ls[max(0,i-4):i+8]))
fragments=[]
for n in sorted(names):
 m=re.search('^'+n+r' proc near',s,re.M);assert m,n
 z=re.search(r'^\w+ proc near',s[m.end():],re.M);p=O/(n+'.asm');p.write_text(s[m.start():m.end()+z.start()],encoding='utf-8');fragments.append(dict(name=n,**pin(p)))
save(O/'480560-refs.json',refs)
b=Path(old['analysis_PE']['path']).read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32);pefrags=[]
for start,end in [(0x480560,0x480620),(0x480400,0x480560),(0x4C6700,0x4C6738),(0x4C6770,0x4C6888),(0x4C6AE0,0x4C6B30)]:
 p=O/f'pe-{start:X}.bin';p.write_bytes(b[start-0x400000:end-0x400000]);d=p.with_suffix('.disasm');d.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(p.read_bytes(),start))+'\n',encoding='utf-8');pefrags.extend([pin(p),pin(d)])
save(L/'r495-source-evidence.json',dict(state='R495_POST_SEED_LOGIN_SEND_SERIALIZER_COUNTER_AND_ERRORS_SOURCE_TRACE',previous=pin(L/'r494-native-seeded-receive-stream-handoff.json'),original_disassembly=old['original_disassembly'],analysis_PE=old['analysis_PE'],fragments=fragments,PE_fragments=pefrags,refs=pin(O/'480560-refs.json'),Native_socket_or_PE_executed=False))
print((O/'sub_480560.asm').read_text(encoding='utf-8'));print((O/'sub_480400.asm').read_text(encoding='utf-8'));print((O/'sub_4803A0.asm').read_text(encoding='utf-8')[:4500])
