from pathlib import Path
import json,hashlib,re,datetime
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r488-mode-source-study';O.mkdir(exist_ok=False);(O/'authority').mkdir()
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');t.replace(p)
old=read(L/'r487-source-evidence.json');asm=Path(old['original_disassembly']['path']);assert pin(asm)['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A';s=asm.read_text(encoding='utf-8');a=s.splitlines();fragments=[]
names=['sub_4A8BA0','sub_4A9E50','sub_4A2A50','sub_4B2970','sub_41B030','sub_41B0E0','sub_41B3F0','sub_41B410','sub_41B420','sub_41B4B0','sub_41B500','sub_41AA90','sub_474280','sub_474440','sub_4744F0','sub_4746A0','sub_477AB0','sub_478680','sub_473510','sub_473C30','sub_474100','sub_46ADB0','sub_46B7D0','sub_46D2E0','sub_46DFB0']
for name in names:
 m=re.search('^'+name+r' proc near',s,re.M)
 if not m:continue
 end=re.search(r'^\w+ proc near',s[m.end():],re.M);p=O/'authority'/f'{name}-through-next-proc.asm';p.write_text(s[m.start():m.end()+end.start()],encoding='utf-8');fragments.append(pin(p))
for token in ['dword_5025A8','word_64ACCA','dword_64ABF4','off_4F7ED4','off_4F9058']:
 i=next(i for i,l in enumerate(a)if l.startswith(token+' '));p=O/'authority'/f'{token}-context.asm';p.write_text('\n'.join(a[i:i+22])+'\n',encoding='utf-8');fragments.append(pin(p))
xrefs=[];proc='';group_sites=[];child10=[]
for i,l in enumerate(a):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if '64ACCA'in l or '64B41C'in l:xrefs.append(dict(line=i+1,procedure=proc,text=l))
 if '64B40C'in l:group_sites.append(dict(line=i+1,procedure=proc,text=l,next_lines=a[i:i+12]))
 if l=='call    sub_41B410'and any('push    0Ah'==x for x in a[max(0,i-5):i])and any('64B40C'in x for x in a[max(0,i-7):i]):child10.append(i+1)
record=dict(state='R488_NATIVE_MODE_WRITER_DISPATCH_PARSER_SHARED_CHILD_ALIAS_SOURCE_SNAPSHOTS',original_disassembly=pin(asm),previous_R487_handoff=pin(L/'r487-native-arrow-hold-player-flow-handoff.json'),fragments=fragments,mode_and_child_named_xrefs=xrefs,all_named_shared_group_sites=group_sites,direct_constant_group_child10_getter_sites=child10,scope='Complete named procedures through next named procedure include DD/unclassified continuations. Named global xrefs/group sites enumerated;not a proof excluding unlabelled/raw-DD/indirect alias stores.',GUI=0,LOADED=False)
save(L/'r488-source-evidence.json',record)
progress_path=P.parent/'監製進度.json';v=read(progress_path);v['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();v['phase']='R488_active_shared_scroll_alias_and_native_mode_writer_flow';v['current_task']='R488 active:trace64B41C geometry aliases/DD continuations andword64ACCA writers. Opcode213/subtype2 selected-mode low-byte update andidentifier gate found;implement bounded offline source event+menu construction without live wire/GUI. R487 arrow finite acceptanceT332 retained;40staticCPU/16of40/GUI0/LOADEDfalse.';v['r487_T332_finite_acceptance']=dict(handoff=record['previous_R487_handoff'],accepted_scope='source-derived arrow hold/reenter/release and Modern lifecycle cancellation only',events=53,cases=12,GUI=0,LOADED=False,complete_modes=16);save(progress_path,v)
print('R488 source snapshots',len(fragments),'fragments;progress active;GUI0')
