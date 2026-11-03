from r491_common import *
import shutil,difflib
root=P/'target/r510-production-source';root.mkdir();pins=[];changes=[];diff=[];prior={e['path']:e for e in read(L/'r509-production-source-snapshot.json')['source_pins']}
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);e=pin(p);e['snapshot']=str(d);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text(encoding='utf-8').splitlines(True) if old else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r510-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r510-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R509=pin(L/'r509-character-selection-channel-handoff.json'),GUI=0))
s=(P/'tools/build-r509-consumer.ps1').read_text(encoding='utf-8').replace('r509','r510').replace('r510c-library-build','r510b-library-build').replace('r510-character-consumer','r510-graphics-consumer');(P/'tools/build-r510-consumer.ps1').write_text(s,encoding='utf-8')
root=P/'target/r510-player-entry-candidate';root.mkdir();main=root/'LiToModern-R510.rs';main.write_text((P/'target/r509-player-entry-candidate/LiToModern-R509.rs').read_text(encoding='utf-8').replace('r509','r510').replace('R509','R510'),encoding='utf-8');v=read(L/'r509-entry-source-contract.json');v['candidate_main']=pin(main);v['new_graphics']='Explicit CPU inspection API only; no live skill owner/GUI added to candidate';save(L/'r510-entry-source-contract.json',v)
s=(P/'tools/build-r509-player.ps1').read_text(encoding='utf-8').replace('r509','r510').replace('R509','R510').replace('r510c-library-build','r510b-library-build');(P/'tools/build-r510-player.ps1').write_text(s,encoding='utf-8')
# Independent Python compositor uses sealed external row/span words and table
# outputs, rather than the new Rust decoder or its derived output.
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源');contract=read(B/'native-icon-draw-contract.json');vectors=[read(e['path']) for e in contract['reusable_vectors']]
for flag,name,folder in [(0,'555','RGB555'),(1,'565','RGB565')]:
 words=[0x2108]*(640*480);table=Path(B/'skill-pane-render-initialization'/folder/'selection-83673C.le16').read_bytes();lut=[int.from_bytes(table[i:i+2],'little') for i in range(0,len(table),2)]
 for slot,vector,timed,marked in [(0,vectors[0],False,False),(1,None,False,False),(2,vectors[4],True,False),(3,vectors[1],False,True),(4,None,False,True)]:
  x=470+34*(slot%4);y=43+34*(slot//4)
  if vector:
   dx,dy=vector['offset_signed']
   for row in vector['rows']:
    for span in row['spans']:
     for j,raw in enumerate(span['words']):
      v=((raw+(raw&0x7fe0))&65535) if flag else raw
      if timed:
       shifts=[0,6,11] if flag else [0,5,10];v=sum((((v>>s)&31)//2)<<s for s in shifts)
      words[(y+dy+row['y'])*640+x+dx+span['x']+j]=v
  if marked:
   for yy in range(y-1,y+33):
    for xx in range(x-1,x+33):
     at=yy*640+xx;v=words[at];words[at]=(lut[v&255]+lut[256+(v>>8)])&65535
 out=L/f'r510-independent-grid-{name}.le16';out.write_bytes(b''.join(v.to_bytes(2,'little') for v in words))
save(L/'r510-independent-grid-inputs.json',dict(oracle='New independent Python composite of sealed span words and LE16 selection tables; no Rust output input and no nativeGUIoracle',outputs=[pin(L/f'r510-independent-grid-{name}.le16') for name in ['555','565']],reference_only=True))
print('Frozen',len(pins),'production sources;',len(changes),'changed; prepared independent composites')
