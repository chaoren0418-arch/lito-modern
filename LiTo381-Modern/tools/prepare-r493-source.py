from r491_common import *
import re
O=P/'target/r493-source-producer-study';O.mkdir(parents=True,exist_ok=True)
old=read(L/'r492-source-evidence.json');a=Path(old['original_disassembly']['path']);assert pin(a)==old['original_disassembly'];s=a.read_text(encoding='utf-8');ls=s.splitlines()
names={'sub_4A7450','sub_494120','sub_4D39F0','sub_4A2A50','sub_4A7550','sub_4928C0','sub_4D3A80','sub_4A7F60','fn_ClientApplication_Run','sub_4C6770','sub_4C68D0','sub_4B2970','sub_4D6970','sub_4A33E0','sub_4B37C0','sub_4C66C0','sub_4C6700','sub_4C65F0','sub_4C6B30'}
refs=[];proc=''
for i,l in enumerate(ls):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if any(t in l for t in ['dword_63D54C','byte_834960','byte_834A60','dword_62FAA0','byte_62FACD','7607A0','76079C']) and not l.strip().startswith(';'):
  refs.append(dict(procedure=proc,line=i+1,text=l,context=ls[max(0,i-4):i+7]))
  if l.startswith(('mov     dword_','mov     byte_','mov     edi, offset byte_834','lea     edi, byte_834')):names.add(proc)
fragments=[]
for n in sorted(names):
 m=re.search('^'+n+r' proc near',s,re.M)
 if not m:continue
 e=re.search(r'^\w+ proc near',s[m.end():],re.M);p=O/(n+'.asm');p.write_text(s[m.start():m.end()+e.start()],encoding='utf-8');fragments.append(dict(name=n,**pin(p)))
save(O/'references.json',refs)
contexts=[]
for t in ['aSscchhhccccccc','dword_5025A8','dword_4F727C','dword_63D54C']:
 i=next(i for i,l in enumerate(ls)if l.startswith(t+' '));p=O/(t+'.asm');p.write_text('\n'.join(ls[i:i+8])+'\n',encoding='utf-8');contexts.append(pin(p));print(p.read_text(encoding='utf-8'))
save(L/'r493-source-evidence.json',dict(original_disassembly=old['original_disassembly'],fragments=fragments,contexts=contexts,references=pin(O/'references.json'),previous=pin(L/'r492-native-authority-bound-ingress-handoff.json')))
print('Pinned',len(fragments),'source routines')
print((O/'fn_ClientApplication_Run.asm').read_text(encoding='utf-8')[:2500])
