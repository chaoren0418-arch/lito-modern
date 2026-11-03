from pathlib import Path
import json,hashlib,re,datetime,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';assert not (L/'r489-source-evidence.json').exists();O=P/'target/r489-mode-snapshot-study';O.mkdir(exist_ok=True);A=O/'authority';A.mkdir(exist_ok=True)
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
old=read(L/'r488-source-evidence.json');asm=Path(old['original_disassembly']['path']);assert pin(asm)==old['original_disassembly'];s=asm.read_text(encoding='utf-8');lines=s.splitlines();proc='';identity=[];phase=[];calls=[]
for i,l in enumerate(lines):
 if re.match(r'^\w+ proc near',l):proc=l.split()[0]
 if '64ABF4'in l:identity.append(dict(line=i+1,procedure=proc,text=l,context=lines[max(0,i-8):i+12]))
 if '64AF98'in l and (l.startswith('mov     dword_64AF98')or 'dd 'in l):phase.append(dict(line=i+1,procedure=proc,text=l,context=lines[max(0,i-8):i+12]))
 if l=='call    sub_469CA0'and any(x=='mov     ecx, 6'for x in lines[max(0,i-12):i]):calls.append(dict(line=i+1,procedure=proc,text=l,context=lines[max(0,i-12):i+6]))
names={x['procedure']for x in identity if not x['text'].startswith('dword_64ABF4 ')}|{x['procedure']for x in phase if x['text'].startswith('mov     dword_64AF98')}|{'sub_469CA0','sub_4A8BA0','sub_4A9E50','sub_4A2A50','sub_4B2970','sub_405100','sub_453170','sub_4AAE70'}
fragments=[]
for name in sorted(names):
 m=re.search('^'+name+r' proc near',s,re.M);assert m,name
 end=re.search(r'^\w+ proc near',s[m.end():],re.M);p=A/(name+'-through-next-proc.asm');p.write_text(s[m.start():m.end()+end.start()],encoding='utf-8');fragments.append(pin(p))
for token in ['aCdscshhchccccc','aSchcscccc','dword_454520','dword_4F72EC','dword_4F7318','dword_64ABF4','word_64ACCA','dword_64AF98']:
 i=next(i for i,l in enumerate(lines)if l.startswith(token+' '));n=30 if token=='dword_454520'else 18;p=A/(token+'-context.asm');p.write_text('\n'.join(lines[i:i+n])+'\n',encoding='utf-8');fragments.append(pin(p))
# Exact little-endian DD reconstruction. Decode only the explicit absolute
# mov instruction signature, not an instruction path or unreachable-code claim.
i=next(i for i,l in enumerate(lines)if l.startswith('dword_454520 '));values=[]
for l in lines[i:i+5]:
 if 'dd offset'in l:break
 if 'dd 'not in l:continue
 for token in l.split('dd ',1)[1].split(';',1)[0].split(','):
  token=token.strip();m=re.fullmatch(r'(\d+) dup\(([0-9A-F]+h)\)',token)
  if m:values.extend([int(m[2][:-1],16)]*int(m[1]))
  else:values.append(int(token[:-1],16)if token.endswith('h')else int(token))
raw=b''.join(struct.pack('<I',v)for v in values);signature=bytes.fromhex('C7 05 98 AF 64 00 06 00 00 00');offset=raw.index(signature);p=O/'454520-dd-prefix.bin';p.write_bytes(raw)
full_fields=[('c','var_2',None),('d','dword_64ABF4',None),('s','dword_64AB70',90),('c','byte_64ABE6',None),('s','byte_64ABE8',5),('h','word_64ABEE',None),('h','dword_64ABF0',None),('c','word_64ACCA',None),('h','var_4',None),('c','dword_64ACCC',None),('c','byte_64ACD0',None),('c','byte_64ACD3',None),('c','byte_64ACD1',None),('c','byte_64ACD2',None),('s','word_64ABCA',28),('c','byte_64ACD4',None),('c','byte_64ACD5',None),('c','byte_64ACD6',None),('c','byte_64ACD7',None),('c','var_1',None)]
sub0_fields=[('s','dword_64AB70',90),('c','word_64ACCA',None),('h','var_4',None),('c','dword_64ACCC',None),('s','word_64ABCA',28),('c','byte_64ACD0',None),('c','byte_64ACD1',None),('c','byte_64ACD2',None),('c','byte_64ACD3',None)]
assert ''.join(f[0]for f in full_fields)=='cdscshhchcccccsccccc'and ''.join(f[0]for f in sub0_fields)=='schcscccc'
record=dict(state='R489_EXACT_FORMAT_STACK_MAPPING_MODE_BYTE_CORRECTION_AND_ID_PHASE_SOURCE',previous_R488_handoff=pin(L/'r488-native-mode-event-player-flow-handoff.json'),original_disassembly=old['original_disassembly'],fragments=fragments,identifier_named_refs=identity,phase_named_and_DD_stores=phase,constant_ecx6_phase_calls=calls,format_mapping=dict(full206=full_fields,subtype0_213=sub0_fields),phase6_DD=dict(base='0x454520',offset=offset,instruction_address=f'0x{0x454520+offset:08X}',bytes=signature.hex().upper(),meaning='mov dword ptr [0x64AF98],6',raw_prefix=pin(p),scope='Static bytes only;no execution/control-flow proof of this callback.'),correction='R488 source-findings falsely described these two mode writers as h/full-word. Exact format stack pairing proves c/low-byte for both. R488 original artifacts remain immutable;R488 subtype2 implementation itself was already c and remains correct.',GUI=0,LOADED=False)
save(L/'r489-source-evidence.json',record)
progress_path=P.parent/'監製進度.json';v=read(progress_path);v['updated_at']=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat();v['phase']='R489_active_bounded_snapshot_format_stack_mode_byte_correction';v['current_task']='R489 active:exact varargs format shows206/subtype0 and213/subtype0 both write mode low-byte c,not full-word h. PreserveR488 immutable and explicitly correct its source description. Decode complete bounded known layouts/raw byte strings atomically,wire production Runtime snapshot->delta->explicit menu/cancel. Traceidentifier andphase6 static sources,no guessed thumb geometry/noGUI/noJava.';v['r488_T340_finite_acceptance']=dict(handoff=record['previous_R488_handoff'],accepted_scope='33scoped subtype2 source events/7runtime flow finite offline only;source-description full-writer correction recorded inR489',GUI=0,LOADED=False,complete_modes=16);save(progress_path,v)
print('R489 source fragments',len(fragments),'phase6 DD',record['phase6_DD']['instruction_address'],'mode c widths corrected;progress active')
