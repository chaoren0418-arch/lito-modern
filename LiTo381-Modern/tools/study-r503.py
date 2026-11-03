from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
base=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收')
asm=base/'ida-client-oep-v2/full-disassembly.asm';pe=base/'runtime/LiTo.unpacked-oep-current.exe'
lines=asm.read_text(encoding='utf-8-sig').splitlines();out=L/'r503-source-fragments'
for start,end in [(236645,236803),(552014,552040),(553324,553334),(180715,180748),(309609,309726),(316360,316376),(349800,349832),(316732,316774),(226470,226675),(236105,236320),(275295,275425),(309077,309145),(236330,236515),(349917,349992),(363290,363355),(273454,273546),(232830,232931),(230220,230295)]:
 (out/f'asm-{start}-{end}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(start-1,end))+'\n',encoding='utf-8')
data=pe.read_bytes();cs=Cs(CS_ARCH_X86,CS_MODE_32)
for va,n in [(0x492fa0,0x3e),(0x46f010,0x223),(0x6343d8,8),(0x502778,8),(0x4928c0,0x90),(0x492e6b,0x128),(0x46ba10,0x39)]:
 raw=data[va-0x400000:va-0x400000+n]
 content=f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'
 if va<0x500000:content+='\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in cs.disasm(raw,va))+'\n'
 (out/f'pe-{va:08X}.txt').write_text(content,encoding='utf-8')
save(L/'r503-source-study-initial.json',dict(originals=[pin(asm),pin(pe)],fragments=[pin(f) for f in sorted(out.glob('*.txt'))],method='Readonly fixed original ASM + static Capstone PE bytes; never execute original. T68/T75 hints only.'))
