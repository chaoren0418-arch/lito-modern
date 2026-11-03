from r491_common import *
import struct,re
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
O=P/'target/r493-source-producer-study'
p=Path('D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe')
meta=read(p.with_suffix('.exe.json'));assert pin(p)['sha256']==meta['output_sha256']
b=p.read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32)
out=[]
for start,end in [(0x4A7550,0x4A76D0),(0x4D39F0,0x4D3A80)]:
 raw=b[start-0x400000:end-0x400000];q=O/f'pe-{start:X}.bin';q.write_bytes(raw);dis='\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,start));d=q.with_suffix('.disasm');d.write_text(dis+'\n',encoding='utf-8');out.extend([pin(q),pin(d)]);print(dis)
needle=struct.pack('<I',0x63D54C);hits=[m.start()+0x400000 for m in re.finditer(re.escape(needle),b[:0xF7000])]
contexts=[]
for va in hits:
 a=va-12;raw=b[a-0x400000:va-0x400000+10];contexts.append(dict(address=hex(va),start=hex(a),hex=raw.hex(),disassembly=[f'{i.address:08X} {i.mnemonic} {i.op_str}' for i in cs.disasm(raw,a)]))
q=O/'pe-version-address-hits.json';save(q,contexts);out.append(pin(q))
fmt=b'sscchhhcccccccccddddddddddcdddddc\0';at=b.find(fmt);assert at>=0;print('format VA',hex(at+0x400000),'len',len(fmt)-1,'version address hits',len(hits))
save(L/'r493-pe-evidence.json',dict(PE=pin(p),metadata=pin(p.with_suffix('.exe.json')),analysis_PE_not_executed=True,image_base=0x400000,sections_identity_mapping=meta['sections'],format_va=hex(at+0x400000),format=fmt[:-1].decode(),fragments=out))
