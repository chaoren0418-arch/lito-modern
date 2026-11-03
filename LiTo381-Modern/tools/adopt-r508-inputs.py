from r491_common import *
import re, shutil
from capstone import Cs, CS_ARCH_X86, CS_MODE_32
q=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-R507-login-producer-contract.json')
v=read(q); root=Path(v['evidence_root']); checks=[]
for item in [v['packet_fields'],v['new_memory_validation']['goldens'],v['new_memory_validation']['chain'],v['manifest'],v['closure_erratum']]:
 assert pin(item['path'])==item,item
 checks.append(item)
for item in read(root/'manifest.json')['files']:
 actual=pin(root/item['path']); assert actual['sha256']==item['sha256'] and actual['bytes']==item['bytes'],item
 checks.append(actual)
copy=L/'r508-producer-inputs';copy.mkdir(exist_ok=False)
for name in ['login-to-GAME-packet-fields.json','goldens/protocol-goldens.json','goldens/server-continuation.tsv','protection-postflight-and-closure-erratum.json']:
 dest=copy/Path(name).name;shutil.copyfile(root/name,dest)
shutil.copyfile(q,copy/q.name)
study=read(L/'r508-source-study-initial.json');asm=Path(study['originals'][0]['path']);pe=Path(study['originals'][1]['path']);assert pin(asm)==study['originals'][0] and pin(pe)==study['originals'][1]
lines=asm.read_text(encoding='utf-8-sig').splitlines();data=pe.read_bytes();fragments=[];out=L/'r508-source-fragments'
for name in ['sub_48F500','sub_49A2A0','sub_4999B0']:
 a=next(i for i,s in enumerate(lines) if s.startswith(name+' proc '));b=next(i for i in range(a,len(lines)) if lines[i].startswith(name+' endp'))
 dest=out/(name+'.asm');dest.write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(a,b+1))+'\n',encoding='utf-8');fragments.append(pin(dest))
 # Fixed entry to the next alignment, retain callback after E6 receiver too.
 va=int(name.split('_')[1],16);end=0x48F580 if va==0x48F500 else int(next(re.match(r'sub_([0-9A-F]+) proc ',s)[1] for s in lines[b+1:] if re.match(r'sub_([0-9A-F]+) proc ',s)),16)
 raw=data[va-0x400000:end-0x400000];dest=out/(name+'-PE.txt');dest.write_text(raw.hex()+'\n'+'\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n');fragments.append(pin(dest))
# Compressed dispatcher tables are inspected from PE, independently of ASM comments.
dispatch=[]
for opcode in [0,59,230,244,251]:
 for compressed,jumps in [(0x4B3470,0x4B2A0C),(0x4B36B4,0x4B3156)]:
  # jump instruction points at a separate DWORD table; resolve its absolute operand.
  ins=list(Cs(CS_ARCH_X86,CS_MODE_32).disasm(data[jumps-0x400000:jumps-0x400000+7],jumps))[0]
  base=int(re.search(r'0x([0-9a-f]+)',ins.op_str)[1],16)
  idx=data[compressed-0x400000+opcode];target=int.from_bytes(data[base-0x400000+idx*4:base-0x400000+idx*4+4],'little')
  dispatch.append(dict(opcode=opcode,compressed_table=hex(compressed),jump_table=hex(base),index=idx,target=hex(target),instruction_bytes=ins.bytes.hex()))
save(L/'r508-input-adoption.json',dict(actual_handoff=pin(q),announced_SHA='3D0148E9B845F7FB425A11E46DFB6098186F487E5F4BC497E9839C9AFB495824',announced_handoff_SHA_matches=False,revision_caveat='Current top-level file has closure_erratum and127 payloads; announced126-payload SHA does not match. All current referenced payload bytes are independently pinned; do not claim announced top-level identity.',verified=checks,copies=[pin(p) for p in sorted(copy.iterdir())],additional_native_fragments=fragments,dispatcher=dispatch,golden_path_caveat='Embedded Java golden JAR path contains mojibake. Do not resolve it or grant access; bind vector bytes to independently verified actual AD368 JAR.',native_boundaries=['4903D0 cdsch followed by version>=5226 DWORD; Java lacks extra DWORD, explicit adapter required','S0 result0 returns without source phase change; no ACK','48FAA0 signed c count, row stride72/name63+dLE+hLE; zero leaves old rows','48F580 source-selected raw CString C8; strict Modern slot<count','48F500 E6: private62C224 controls immediate E2 or modal callback48F550 E2; private flag unqualified; explicit Java continuation control','F4 Java source-bootstrap reconnect differs from selected-character Native branch; explicit adapter only'],GUI=0,historical_PASS_reruns=0))
v=read(P.parent/'監製進度.json');v['r508']['producer_adoption']=pin(L/'r508-input-adoption.json');v['r508']['icon_handoff_pending']={'path':r'D:/lineage開發/3-lito開發/lito脫殼資源/native-icon-draw-contract.json','sha256':'9BA690374734FECFE86CF95651E864515FFFDC78D71FE58C556974625AD95803','state':'pending_after_R508'};save(P.parent/'監製進度.json',v)
print('Current producer127 payloads verified; announced top-level SHA mismatch explicitly recorded; Native dispatch:',dispatch)
