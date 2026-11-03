from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
base=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收');asm=base/'ida-client-oep-v2/full-disassembly.asm';pe=base/'runtime/LiTo.unpacked-oep-current.exe'
lines=asm.read_text(encoding='utf-8-sig').splitlines();out=L/'r505-source-fragments'
for start,end in [(559640,559710),(415210,415276),(252791,252982),(509014,509035),(374362,374416),(513430,513442)]:
 (out/f'asm-{start}-{end}.txt').write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(start-1,end))+'\n',encoding='utf-8')
symbols=['dword_64AD74','dword_62C6A4','dword_64AD2C','word_64AD70','dword_64AE74']
references={s:[dict(line=i+1,text=v) for i,v in enumerate(lines) if s in v] for s in symbols};save(L/'r505-global-references.json',dict(original=pin(asm),references=references))
data=pe.read_bytes()
for va,n in [(0x4ac520,0x50),(0x4ac570,0x70),(0x4efe56,12),(0x64ad74,0x100),(0x64ad2c,0x28)]:
 raw=data[va-0x400000:va-0x400000+n];cs=Cs(CS_ARCH_X86,CS_MODE_32);content=f'VA {va:08X} raw offset {va-0x400000:08X}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in cs.disasm(raw,va))+'\n';(out/f'pe-{va:08X}.txt').write_text(content,encoding='utf-8')
save(L/'r505-source-study-final-gates.json',dict(originals=[pin(asm),pin(pe)],references=pin(L/'r505-global-references.json'),fragments=[pin(p) for p in sorted(out.glob('*'))],GUI=0))
