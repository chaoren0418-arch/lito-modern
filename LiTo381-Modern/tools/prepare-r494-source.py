from r491_common import *
import re,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
O=P/'target/r494-native-receive-study';O.mkdir(parents=True,exist_ok=True)
old=read(L/'r493-source-evidence.json');a=Path(old['original_disassembly']['path']);assert pin(a)==old['original_disassembly'];s=a.read_text(encoding='utf-8');ls=s.splitlines()
symbols=['8317C0','8317C4','8317C8','8317CC','8317B8','8317D0','7607A8','7607A4','7607AC','760750']
refs=[];proc='';names={'sub_4B37C0','sub_4C66C0','sub_4C6700','sub_4C65F0','sub_4C6B30','sub_4B2970'}
for i,l in enumerate(ls):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if any(t in l for t in symbols) and not l.strip().startswith(';'):
  refs.append(dict(procedure=proc,line=i+1,text=l,context=ls[max(0,i-6):i+9]));names.add(proc)
fragments=[]
for n in sorted(names):
 m=re.search('^'+n+r' proc near',s,re.M)
 if not m:continue
 e=re.search(r'^\w+ proc near',s[m.end():],re.M);p=O/(n+'.asm');p.write_text(s[m.start():m.end()+e.start()],encoding='utf-8');fragments.append(dict(name=n,**pin(p)))
save(O/'named-references.json',refs)
oldpe=read(L/'r493-pe-evidence.json');p=Path(oldpe['PE']['path']);assert pin(p)==oldpe['PE'];b=p.read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32);hits=[]
for sym in symbols:
 va=int(sym,16)
 for m in re.finditer(re.escape(struct.pack('<I',va)),b[:0xF7200]):
  at=m.start();start=max(0,at-12);raw=b[start:at+16];hits.append(dict(symbol=sym,address=hex(at+0x400000),start=hex(start+0x400000),hex=raw.hex(),raw_context_disassembly=[f'{i.address:X} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,start+0x400000)]))
save(O/'pe-address-hits.json',hits)
save(L/'r494-source-evidence.json',dict(state='R494_RECEIVE_HANDSHAKE_COUNTER_BYPASS_AND_RESET_SOURCE_TRACE',previous=pin(L/'r493-native-roster-selected-identity-handoff.json'),original_disassembly=old['original_disassembly'],analysis_PE=oldpe['PE'],fragments=fragments,named_references=pin(O/'named-references.json'),PE_address_hits=pin(O/'pe-address-hits.json'),Native_socket_or_PE_executed=False))
print('Pinned',len(fragments),'routines',len(refs),'namedrefs',len(hits),'PE literal-address hits')
for r in refs:
 if r['text'].startswith(('mov     dword_8317','mov     byte_8317','mov     byte_7607','mov     dword_7607')):print(r['procedure'],r['line'],r['text'])
for x in hits:
 if re.search(r'(?:a3|c705|c605|68|b9)'+struct.pack('<I',int(x['symbol'],16)).hex(),x['hex']):print(x)
