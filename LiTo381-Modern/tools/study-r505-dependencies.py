from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
base=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收');asm=base/'ida-client-oep-v2/full-disassembly.asm';pe=base/'runtime/LiTo.unpacked-oep-current.exe'
lines=asm.read_text(encoding='utf-8-sig').splitlines();out=L/'r505-source-fragments'
for start,end in [(359741,359796),(349838,350390),(372286,372449),(236646,236865),(501065,501153),(513420,513447),(551940,552043),(317005,317129),(251331,251482),(316360,316408),(550450,550480)]:
 (out/f'asm-{start}-{end}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(start-1,end))+'\n',encoding='utf-8')
for symbol in ['dword_62C6A4','dword_64AD2C','sub_475350','sub_497020','sub_496A00','sub_4A7550']:
 matches=[f'{i+1}: {s}' for i,s in enumerate(lines) if symbol in s];print(symbol+'\n'+'\n'.join(matches))
data=pe.read_bytes()
for va,n in [(0x502f2c,16),(0x502f28,16),(0x62c6a4,16),(0x4efe4a,24),(0x497020,0x200),(0x46f010,0x220),(0x475350,0x120)]:
 raw=data[va-0x400000:va-0x400000+n];cs=Cs(CS_ARCH_X86,CS_MODE_32);content=f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in cs.disasm(raw,va))+'\n';(out/f'pe-{va:08X}.txt').write_text(content,encoding='utf-8')
save(L/'r505-source-study-dependencies.json',dict(originals=[pin(asm),pin(pe)],fragments=[pin(p) for p in sorted(out.glob('*'))],GUI=0))
