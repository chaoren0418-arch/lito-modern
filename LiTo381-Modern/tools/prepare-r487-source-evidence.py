from pathlib import Path
import json,hashlib,re
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r487-menu-source-study';O.mkdir(exist_ok=False);(O/'authority').mkdir()
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
old=json.loads((L/'r478-source-fragments.json').read_text(encoding='utf-8'));asm=Path(old['original_disassembly']['path']);assert pin(asm)['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A';s=asm.read_text(encoding='utf-8');lines=s.splitlines();fragments=[]
names=['fn_ClientApplication_Run','sub_46B7D0','sub_474280','sub_474350','sub_474360','sub_474440','sub_4744F0','sub_477AB0','sub_477930','sub_477990','sub_41B030','sub_414910','sub_4E99C0','sub_4E9A40','sub_450090','sub_450230','sub_4645D0','sub_4646C0']
for name in names:
 start=re.search('^'+name+r' proc near',s,re.M);assert start,name;end=re.search(r'^\w+ proc near',s[start.end():],re.M);text=s[start.start():start.end()+end.start()];p=O/'authority'/f'{name}-through-next-proc.asm';p.write_text(text,encoding='utf-8');fragments.append(pin(p))
for name in ['off_4F7ED4','off_4F9058']:
 i=next(i for i,line in enumerate(lines)if line.startswith(name+' '));p=O/'authority'/f'{name}-table-context.asm';p.write_text('\n'.join(lines[i-3:i+24])+'\n',encoding='utf-8');fragments.append(pin(p))
xrefs=[];proc=''
for i,line in enumerate(lines):
 if re.match(r'^\w+ proc near',line):proc=line.split()[0]
 if '64AF9F'in line:xrefs.append(dict(line=i+1,procedure=proc,text=line))
record=dict(state='R487_ORIGINAL_ASM_COMPLETE_PROCEDURE_AND_GLOBAL_XREF_SNAPSHOTS',original_disassembly=pin(asm),previous_R486_handoff=pin(L/'r486-bounded-city563-handoff.json'),fragments=fragments,flag64AF9F_xrefs=xrefs,GUI=0,LOADED=False)
(L/'r487-source-evidence.json').write_text(json.dumps(record,ensure_ascii=False,indent=2),encoding='utf-8');print('R487 original evidence frozen:',len(fragments),'complete procedures/tables',len(xrefs),'flag references')
