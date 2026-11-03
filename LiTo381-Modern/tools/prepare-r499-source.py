from r491_common import *
import re
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
e=read(L/'r498-source-study.json');a=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');b=Path(e['analysis_PE']['path']).read_bytes();O=P/'target/r499-back-login-study';O.mkdir(exist_ok=True);fr=[]
names=['sub_493150','sub_496E30','sub_46B670','sub_46D2E0','sub_496DA0','sub_4962B0','sub_495280','sub_495230','sub_4952F0','sub_49A510','sub_480560','sub_4AE050','sub_4A33E0','sub_47C870','sub_4D6390','sub_4C6AE0','sub_4C6770','sub_4C65F0','sub_4A9770','sub_47E4B0']
names+=['sub_4A7450','sub_4A7550','sub_48FAA0','sub_478680','sub_4803A0','sub_480400','sub_4966A0','sub_4963E0']
for n in names:
 m=re.search('^'+n+r' proc near',a,re.M);assert m,n;z=re.search(r'^\w+ proc near',a[m.end():],re.M);p=O/(n+'.asm');p.write_text(a[m.start():m.end()+z.start()],encoding='utf-8');fr.append(dict(name=n,line_start=a[:m.start()].count('\n')+1,**pin(p)))
cs=Cs(CS_ARCH_X86,CS_MODE_32)
for lo,hi in [(0x46B670,0x46B710),(0x46D2E0,0x46D710),(0x4A33E0,0x4A36F0),(0x47C870,0x47CB40),(0x496DA0,0x496E30)]:
 raw=b[lo-0x400000:hi-0x400000];p=O/f'{lo:X}-{hi:X}.bin';p.write_bytes(raw);d=O/f'{lo:X}-{hi:X}.disasm';d.write_text('\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}'for i in cs.disasm(raw,lo)),encoding='utf-8');fr.extend([pin(p),pin(d)])
save(L/'r499-source-study.json',dict(state='R499_NATIVE_BACK_LOGIN_DEPENDENCY_TRACE_IN_PROGRESS',original_disassembly=e['original_disassembly'],analysis_PE=e['analysis_PE'],fragments=fr,previous_R498=pin(L/'r498-native-character-ui-mapping-selection-scene-handoff.json'),previous_R496=pin(L/'r496-native-version-actor-admission-selection-controls-handoff.json'),GUI=0))
progress('R499_active_native_character_Back_opcode15_connection_retention_login_dependency_trace','R498有限接受18JSON1237pins836unique/58804byte diff/11flows65536mapping16boundary4IMG. R499 tracing493150receiver6->496E30->46B670 and58/480560;NativeBacksource sendsc15 beforecleanup;checkingconnection/counters vsModernwholeepochretirement. NohistoricalPASSrerun/cardassetsearch/provider/server/sharedBIN/GUI0.')
print('R499 extracted',len(fr),'sourcefragments')
