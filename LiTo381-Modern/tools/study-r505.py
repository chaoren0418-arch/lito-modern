from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
base=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收');asm=base/'ida-client-oep-v2/full-disassembly.asm';pe=base/'runtime/LiTo.unpacked-oep-current.exe';lines=asm.read_text(encoding='utf-8-sig').splitlines();out=L/'r505-source-fragments';out.mkdir(exist_ok=True)
ranges=[(226337,226617),(236158,236645),(253003,253275),(359797,359866),(414076,414339),(374130,374176)]
for start,end in ranges:
 (out/f'asm-{start}-{end}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(start-1,end))+'\n',encoding='utf-8')
data=pe.read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32)
for va,n in [(0x469ad0,0x70),(0x4ac570,0x70),(0x46ed50,0x240)]:
 raw=data[va-0x400000:va-0x400000+n];content=f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in cs.disasm(raw,va))+'\n';(out/f'pe-{va:08X}.txt').write_text(content,encoding='utf-8')
save(L/'r505-source-study-initial.json',dict(originals=[pin(asm),pin(pe)],fragments=[pin(p)for p in sorted(out.glob('*'))],GUI=0))
