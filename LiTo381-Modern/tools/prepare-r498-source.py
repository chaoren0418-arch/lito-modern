from r491_common import *
import re,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
e=read(L/'r497-source-evidence.json');a=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');b=Path(e['analysis_PE']['path']).read_bytes()
O=P/'target/r498-character-ui-study';O.mkdir(exist_ok=True)
names=['sub_4928C0','sub_4934A0','sub_4934F0','sub_494120','sub_494160','sub_494770','sub_494DE0','sub_493150','sub_495520','sub_495E40','sub_4966A0','sub_496EB0','sub_41B410','sub_41B6C0','sub_415A30','sub_415AE0','sub_415F30','sub_4154E0','sub_494E00']
names+=['sub_414910','sub_415B70','sub_415BD0','sub_415C20','sub_415C40','sub_415C90','sub_415CF0','sub_415DA0','sub_4159B0','sub_414CC0','sub_497150','sub_497600','sub_414A10','sub_414A80','sub_415CD0','sub_415D40','sub_415D70','sub_40CFF0']
names+=['sub_415AD0','sub_415CB0','sub_4D5270','sub_4D4A50','sub_43FFC0']
fr=[]
for n in names:
 m=re.search('^'+n+r' proc near',a,re.M);z=re.search(r'^\w+ proc near',a[m.end():],re.M);s=a[m.start():m.end()+z.start()];p=O/(n+'.asm');p.write_text(s,encoding='utf-8');fr.append(dict(name=n,line_start=a[:m.start()].count('\n')+1,**pin(p)))
md=Cs(CS_ARCH_X86,CS_MODE_32)
for lo,hi in [(0x495520,0x495E40),(0x496EB0,0x497010)]:
 p=O/f'{lo:X}-{hi:X}.disasm';p.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in md.disasm(b[lo-0x400000:hi-0x400000],lo)),encoding='utf-8');fr.append(pin(p))
tables=[]
for va,count in [(0x630DF8,9),(0x631EC8,2),(0x62FD70,24),(0x4F7C68,2)]:
 raw=b[va-0x400000:va-0x400000+60*count];p=O/f'{va:X}.bin';p.write_bytes(raw);rows=[list(struct.unpack('<15I',raw[i*60:(i+1)*60]))for i in range(count)];tables.append(dict(va=hex(va),rows=rows,raw=pin(p)))
save(L/'r498-source-study.json',dict(state='R498_RESEARCH_IN_PROGRESS',original_disassembly=e['original_disassembly'],analysis_PE=e['analysis_PE'],fragments=fr,tables=tables,GUI=0,continuation_history=['Previous turn answered an obsolete Java/server-directory question after compaction and ended before R498 delivery.','Supervisor resumed the existing authorized R498 scope; a retry failed before work due to model capacity.','This continuation uses Modern-owned outputs and the previously authorized ASCII Modern scratch; no Java/server permission is needed.']))
print('R498 extracted',len(fr),'source/disassembly fragments')
