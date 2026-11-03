from r491_common import *
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
import re
old=read(L/'r500-source-study.json');asm=Path(old['original_disassembly']['path']);pe=Path(old['analysis_PE']['path']);assert pin(asm)==old['original_disassembly'] and pin(pe)==old['analysis_PE']
lines=asm.read_text(encoding='utf-8-sig').splitlines();data=pe.read_bytes();root=L/'r508-source-fragments';root.mkdir(exist_ok=True);fragments=[]
names=['sub_4903D0','sub_48FAA0','sub_48F580','sub_48FCF0','sub_4900B0','sub_4A2A50','sub_4B2970','sub_4AC570','sub_480560','sub_492090','sub_49A510']
for name in names:
 a=next(i for i,s in enumerate(lines) if s.startswith(name+' proc '));b=next(i for i in range(a,len(lines)) if lines[i].startswith(name+' endp'))
 path=root/(name+'.asm');path.write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(a,b+1))+'\n',encoding='utf-8');fragments.append(dict(name=name,line_start=a+1,line_end=b+1,**pin(path)))
 va=int(name.split('_')[1],16)
 # End at the next reliable labeled procedure boundary; every instruction
 # begins at an original function entry, no mid-immediate raw slicing.
 later=next((int(m[1],16) for s in lines[b+1:] if (m:=re.match(r'sub_([0-9A-F]+) proc ',s)) and int(m[1],16)>va),va+0x800)
 n=min(later-va,0x3000);raw=data[va-0x400000:va-0x400000+n];out=root/(name+'-PE.txt')
 out.write_text(f'VA {va:08X} raw {va-0x400000:08X} length {n}\n'+raw.hex()+'\n'+'\n'.join(f'{i.address:08X}: {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n',encoding='utf-8');fragments.append(pin(out))
save(L/'r508-source-study-initial.json',dict(originals=[pin(asm),pin(pe)],fragments=fragments,previous_R507=pin(L/'r507-native-java-host-boundary-handoff.json'),method='Read-only fixed PE/ASM function-entry disassembly; Java unsealed hints are not adopted as Native semantics',GUI=0))
v=read(P.parent/'監製進度.json');v.update(phase='R508_native_LOGIN_CHECK_SERVER_LIST_receiver_qualification',current_task='追固定Native source0／source59 receiver與source choice／GAME bootstrap；待server sealed AD368 producer契約，僅新增必要案例，GUI0／16of40');v['r508']={'state':'qualifying_native_receivers','previous_R507':pin(L/'r507-native-java-host-boundary-handoff.json'),'GUI':0,'historical_PASS_reruns':0};save(P.parent/'監製進度.json',v)
print('R508 native receiver fragments pinned')
