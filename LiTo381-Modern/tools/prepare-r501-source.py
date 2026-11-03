from r491_common import *
import re
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
e=read(L/'r500-source-study.json');a=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');b=Path(e['analysis_PE']['path']).read_bytes();o=P/'target/r501-normal-auth-send-study';o.mkdir(exist_ok=True);ms=list(re.finditer(r'^(sub_[0-9A-F]+) proc near',a,re.M));by={m.group(1):i for i,m in enumerate(ms)};fr=[]
names=['492090','492270','492190','490390','49AD70','49A770','49AAC0','49A940','49AE10','491ED0','491E80','491FE0','492010','4C68D0','4C6AE0','4C6770','4C6700','480560','4AE050','4A33E0','4EFE50']
names+=['48E4D0','48E500','48E5B0','48E620','48EF40','48EFE0','48F000']
for n in names:
 assert 'sub_'+n in by,n;i=by['sub_'+n];m=ms[i];p=o/(m.group(1)+'.asm');p.write_text(a[m.start():ms[i+1].start()],encoding='utf-8');fr.append(dict(name=m.group(1),line_start=a[:m.start()].count('\n')+1,**pin(p)))
cs=Cs(CS_ARCH_X86,CS_MODE_32)
for lo,hi in [(0x492090,0x492190),(0x492190,0x492240),(0x492270,0x492432),(0x491E80,0x492090),(0x49AD70,0x49AE90),(0x4C68D0,0x4C6AE0),(0x4EFE50,0x4EFE56),(0x480560,0x480620)]:
 raw=b[lo-0x400000:hi-0x400000];p=o/f'{lo:X}-{hi:X}.bin';p.write_bytes(raw);d=p.with_suffix('.disasm');d.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo)),encoding='utf-8');fr.extend([pin(p),pin(d)])
save(L/'r501-source-study.json',dict(state='R501_NORMAL_INPUT_ENV_FORMATTER_LOCALADDR_SEND_TRACE_IN_PROGRESS',original_disassembly=e['original_disassembly'],analysis_PE=e['analysis_PE'],previous_R500=pin(L/'r500-native-login-form-input-lifecycle-handoff.json'),fragments=fr,GUI=0))
progress('R501_active_normal_login_formatter_environment_localaddr_send_source_trace','R500 finiteaccepted18JSON1185pins789unique/65origins55128byte diff/9flows6normalize11boundary. R501 close492090 input/environment/cssd/cssds andgetsockname/480560 timing;typedsource/candidate serializer andminimal syntheticloopbackonlyifsourceclosed. Norealcredentials/provider/JavaSQL/sharedBIN/GUI;overallunfinished16of40.')
print('R501 fragments',len(fr))
