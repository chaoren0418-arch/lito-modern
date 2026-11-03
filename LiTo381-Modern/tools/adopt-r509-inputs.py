from r491_common import *
import re,shutil,subprocess
from capstone import Cs,CS_ARCH_X86,CS_MODE_32
q=Path(r'D:/lineage開發/3-lito開發/lito伺服器/server-character-directory-consumer-contract.json');assert pin(q)['sha256']=='412748DE715782AEC88D00251DEE791A472EF7C180F905EDF43AA3A36B9464C7';v=read(q);root=Path(v['evidence_root']);out=L/'r509-character-inputs';out.mkdir(exist_ok=True);checks=[]
for e in read(v['manifest']['path'])['files']:
 a=pin(root/e['path']);assert a['sha256']==e['sha256'] and a['bytes']==e['bytes'];checks.append(a)
for key in ['complete_fields','existing_actual_goldens','zero_multiple_boundaries','source_slices','Protocol_source_reference','finite_version_comparison']:
 e=v[key];assert pin(e['path'])==e;shutil.copyfile(e['path'],out/Path(e['path']).name)
shutil.copyfile(q,out/q.name);assert pin(v['actual_JAR']['path'])==v['actual_JAR'];assert pin(v['actual_CS_source']['path'])==v['actual_CS_source']
study=read(L/'r500-source-study.json');asm=Path(study['original_disassembly']['path']);pe=Path(study['analysis_PE']['path']);assert pin(asm)==study['original_disassembly'] and pin(pe)==study['analysis_PE'];lines=asm.read_text(encoding='utf-8-sig').splitlines();data=pe.read_bytes();fragments=[];native=L/'r509-native-fragments';native.mkdir(exist_ok=True)
for name in ['sub_4A7450','sub_4A7550','sub_4A76D0','sub_4B2650','sub_4A4D90','sub_4928C0']:
 a=next(i for i,s in enumerate(lines) if s.startswith(name+' proc '));b=next(i for i in range(a,len(lines)) if lines[i].startswith(name+' endp'));f=native/(name+'.asm');f.write_text('\n'.join(f'{i+1}: {lines[i]}' for i in range(a,b+1))+'\n',encoding='utf-8');fragments.append(pin(f))
 va=int(name.split('_')[1],16);end=int(next(re.match(r'sub_([0-9A-F]+) proc ',s)[1] for s in lines[b+1:] if re.match(r'sub_([0-9A-F]+) proc ',s)),16);raw=data[va-0x400000:min(end,va+0x3000)-0x400000];f=native/(name+'-PE.txt');f.write_text(raw.hex()+'\n'+'\n'.join(f'{i.address:08X} {i.bytes.hex()} {i.mnemonic} {i.op_str}' for i in Cs(CS_ARCH_X86,CS_MODE_32).disasm(raw,va))+'\n');fragments.append(pin(f))
# Extra inventory receipt is a necessary C9 continuation, not an inventory UI
# or owner emulation. Bind relevant serializer IL to the actual unchanged JAR.
javap=Path('C:/Program Files/Microsoft/jdk-17.0.17.10-hotspot/bin/javap.exe');il=L/'r509-AD368-Protocol.javap.txt';r=subprocess.run([str(javap),'-p','-c','-classpath',v['actual_JAR']['path'],'local.lito381.server.Protocol'],capture_output=True,timeout=30);assert r.returncode==0;il.write_bytes(r.stdout)
methods={};body=r.stdout.decode('utf-8','replace').splitlines();starts=[i for i,s in enumerate(body) if re.match(r'  (?:private |public |static |protected )',s) and s.endswith(';')]
for i in starts:
 if any(k in body[i] for k in ['inventoryBatch(','appendInventoryRecord(','litoChannelInfo(']):
  b=next((j for j in starts if j>i),len(body));methods[body[i].strip()]='\n'.join(body[i:b])
save(L/'r509-inventory-channel-AD368-IL.json',dict(JAR=v['actual_JAR'],javap=pin(javap),full_output=pin(il),selected_methods=methods,Protocol_reference=pin(Path(r'D:/lineage開發/3-lito開發/lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/Protocol.java')),authority='Actual AD368 javap invocation IL, fixed Native receiver fragments. Current ordinary source is field reference only.',native_inventory_owner_unqualified=True))
save(L/'r509-input-adoption.json',dict(sealed_character_contract=pin(q),payload_checks=checks,originals=[pin(asm),pin(pe)],native_fragments=fragments,inventory_channel_IL=pin(L/'r509-inventory-channel-AD368-IL.json'),Native_UI_vs_Java_count='Native4A7450 reset loop covers4 sentinels and existing494120 UI4. Java sender permits8; retain strict Native4 limit and offer Modern8-row controls only under explicit adapter, do not claim Native8 UI/storage qualification.',old_PASS_reruns=0,GUI=0))
v=read(P.parent/'監製進度.json');v.update(phase='R509_typed_character_source_selection_channel_delta',current_task='採用AD368角色34欄、MS950名稱及epoch/revision；保持Native4/Java8分流，新增S7C receipt及使用者CA接線，舊PASS不重跑');v['r509']={'state':'input_and_native_delta_qualified','input_adoption':pin(L/'r509-input-adoption.json'),'GUI':0};save(P.parent/'監製進度.json',v);print('R509 contract26 payloads and Native receiver/actualAD368 IL pinned')
