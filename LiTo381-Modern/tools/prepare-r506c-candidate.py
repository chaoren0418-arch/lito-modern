from r491_common import *
import shutil,difflib,re
from r495_source_oracle import source_cipher,counter_add_adc
old=read(L/'r505-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
diff=L/'r506c-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r506c-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
save(L/'r506c-production-source-snapshot.json',dict(source_pins=pins,changed=delta,added=[],unchanged=len(pins)-len(delta),source_difference=pin(diff),GUI=0))
oracle=read(L/'r506-source-oracle.json');pe=Path(read(L/'r506-API-adoption.json')['originals'][1]['path']).read_bytes()
def vector(body,counter):
 b=bytes(body);encrypted,steps=source_cipher(b,counter,pe);return dict(plain=list(b),wire=list((len(b)+2).to_bytes(2,'little')+encrypted),before=counter,after=counter_add_adc(counter,len(b)),source_instruction_steps=steps)
row=bytearray(oracle['same_receive'][3]['plain']);offset=9+len(b'Header\0')+14+len(b'Arena1\0')+10;assert int.from_bytes(row[offset:offset+4],'little')==5;row[offset:offset+4]=(0x80000000).to_bytes(4,'little')
bodies=[v['plain']for v in oracle['same_receive']];bodies.insert(4,list(row));counter=oracle['setup']['first_roster']['after'];vs=[]
for b in bodies:v=vector(b,counter);vs.append(v);counter=v['after']
oracle.update(same_receive=vs,previous_attempt_oracle=pin(L/'r506-source-oracle.json'),signed_capacity_probe=0x80000000);save(L/'r506c-source-oracle.json',oracle)
body=(P/'tools/r506-transfer-consumer-body.rs').read_text(encoding='utf-8');body=re.sub(r'same_receive"\]\[(\d+)\]',lambda m:'same_receive"][%d]'%(int(m[1])+1 if int(m[1])>=4 else int(m[1])),body)
mark=' e.steps.send(Step::Send(bytes(&o["same_receive"][5]["wire"])))?;'
assert mark in body
body=body.replace(mark,''' e.steps.send(Step::Send(bytes(&o["same_receive"][4]["wire"])))?;sent(&e);until(&mut r,&mut m,|r|r.source_transfer_snapshot().rows_revision==2);assert!(r.source_transfer_server_control(0).is_none());record(&base,"signed_capacity_guard","actual source row with capacity80000000 remains unavailable under Native signed cmp/jg; no flag or packet supplied by UI",snapshot(&r,&m),&mut trace);
'''+mark).replace('rows_revision==2);let before=', 'rows_revision==3);let before=')
body=body.replace('r506','r506c').replace('R506','R506C')
helpers=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8').split('fn main(')[0].replace('r503','r506c').replace('R503','R506C').replace('r498-native-menu-skin-binding.json','r506-native-menu-skin-binding.json').replace('Duration::from_secs(10);','Duration::from_secs(15);')
(P/'tools/r506c-transfer-consumer.rs').write_text(helpers+body,encoding='utf-8')
root=P/'target/r506c-player-entry-candidate';root.mkdir(exist_ok=False);main=root/'LiToModern-R506C.rs';main.write_text((P/'target/r506-player-entry-candidate/LiToModern-R506.rs').read_text(encoding='utf-8').replace('r506','r506c').replace('R506','R506C'),encoding='utf-8');entry=read(L/'r506-entry-source-contract.json');entry.update(candidate_main=pin(main),previous_attempt=pin(L/'r506-player-candidate.json'));save(L/'r506c-entry-source-contract.json',entry)
s=(P/'tools/build-r506-player-candidate.ps1').read_text(encoding='utf-8').replace('r506','r506c').replace('R506','R506C').replace('r506cb-library','r506c-library');(P/'tools/build-r506c-player-candidate.ps1').write_text(s,encoding='utf-8')
s=(P/'tools/build-r506-transfer-consumer.ps1').read_text(encoding='utf-8').replace('r506','r506c').replace('R506','R506C').replace('r506cb-library','r506c-library');(P/'tools/build-r506c-transfer-consumer.ps1').write_text(s,encoding='utf-8')
save(L/'r506c-new-gates.json',dict(first_attempt_preserved=pin(L/'r506-first-attempt/preservation.json'),signed_capacity='Native cmp ecx,eax followed jg; positive occupancy/capacity available subset only, cap0 Native bypass retained',source_branch=pin(L/'r506-source-fragments/pe-00475F40.txt'),production_source_snapshot=pin(L/'r506c-production-source-snapshot.json'),source_oracle=pin(L/'r506c-source-oracle.json'),GUI=0))
print(f'R506C619sources/{len(delta)}changed; failed attempt preserved; signed capacity risk added')
