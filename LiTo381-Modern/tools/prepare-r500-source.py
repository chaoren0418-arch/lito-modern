from r491_common import *
import re,struct
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
e=read(L/'r499-source-study.json');a=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');b=Path(e['analysis_PE']['path']).read_bytes();o=P/'target/r500-login-lifecycle-study';o.mkdir(exist_ok=True)
procs=list(re.finditer(r'^(sub_[0-9A-F]+) proc near',a,re.M));by={m.group(1):i for i,m in enumerate(procs)};fr=[]
names=['41A430','41A490','496DA0','495E40','4962B0','48FAA0','4B2970','480560','4AE050','4A33E0','493150','496E30','46B670','4C62A0']
names+=['41A4E0','4C3CC0','4C3D20','492270','48FCF0','4900B0','494160','494770','4D4840','48F580','48FC40','4A7450','4A7550','497010']
names+=['492540','48FEA0','492090','492190','492240','4C3D30','4D3860','4966A0','496A60','4952F0','440E50','440E90','401680']
names+=['490390','4C3D90','491ED0','49AD70']
for addr in names:
 i=by['sub_'+addr];m=procs[i];p=o/(m.group(1)+'.asm');p.write_text(a[m.start():procs[i+1].start()],encoding='utf-8');fr.append(dict(name=m.group(1),line_start=a[:m.start()].count('\n')+1,**pin(p)))
cs=Cs(CS_ARCH_X86,CS_MODE_32)
for lo,hi in [(0x496D40,0x496DA0),(0x48FAA0,0x48FEE0),(0x41A430,0x41A550),(0x492270,0x4928C0),(0x4C3CC0,0x4C3D90),(0x4B3060,0x4B30A0),(0x4B315D,0x4B3181),(0x492090,0x492190),(0x4C3D30,0x4C3E30)]:
 raw=b[lo-0x400000:hi-0x400000];p=o/f'{lo:X}-{hi:X}.bin';p.write_bytes(raw);d=p.with_suffix('.disasm');d.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo)),encoding='utf-8');fr.extend([pin(p),pin(d)])
refs=[]
for line in re.finditer(r'^.*(?:75F755|75F756|641348|6412A0|636584|631F68|496D40|496D60).*$',a,re.M):
 if line.start()>a.find('dword_631F68 dd'):continue
 prior=[m for m in procs if m.start()<=line.start()];refs.append(dict(line=a[:line.start()].count('\n')+1,proc=prior[-1].group(1),text=line.group(0)))
hits={}
for va in [0x75F754,0x75F755,0x75F756,0x641348,0x6412A0,0x636584]:
 needle=struct.pack('<I',va);hits[f'{va:X}']=[hex(i+0x400000)for i in range(len(b))if b.startswith(needle,i)]
save(L/'r500-source-study.json',dict(state='R500_LOGIN_CALLBACK_FLAG_INPUT_LIFECYCLE_TRACE_IN_PROGRESS',original_disassembly=e['original_disassembly'],analysis_PE=e['analysis_PE'],previous_R499=pin(L/'r499-native-back-retained-epoch-reselection-handoff.json'),fragments=fr,source_references=refs,PE_address_encoding_hits=hits,GUI=0))
progress('R500_active_normal_login_callback_input_lifecycle_source_trace','R499 finiteaccepted19JSON1149pins757unique/33sourcefragments/43127bytediff/10flows4wirevectors;R500 trace496DA0callbacks,75F755/756,641348/6412A0,480560environmentand58/48FAA0->new3/203. ModernownedASCII only;noJavaSQL/provider/sharedBIN/credentials/GUI;overallunfinished16of40.')
print('R500 preliminaryfragments',len(fr));print(json.dumps({k:v for k,v in hits.items()if k in ['75F754','75F755','75F756','641348','6412A0']}))
