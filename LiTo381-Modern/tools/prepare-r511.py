from r491_common import *
import shutil,difflib
prior={e['path']:e for e in read(L/'r510c-production-source-snapshot.json')['source_pins']};root=P/'target/r511-production-source';root.mkdir();pins=[];changes=[];diff=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);e=pin(p);e['snapshot']=str(d);pins.append(e);old=prior.get(str(p))
 if not old or old['sha256']!=e['sha256']:changes.append(str(p));diff.append(''.join(difflib.unified_diff(Path(old['snapshot']).read_text(encoding='utf-8').splitlines(True) if old else [],p.read_text(encoding='utf-8').splitlines(True),fromfile=old['snapshot'] if old else '/dev/null',tofile=str(p))))
patch=L/'r511-source-difference.patch';patch.write_text('\n'.join(diff),encoding='utf-8');save(L/'r511-production-source-snapshot.json',dict(source_pins=pins,changes=changes,source_difference=pin(patch),previous_R510=pin(L/'r510-bin-graphics-consumer-handoff.json'),GUI=0))
s=(P/'tools/build-r509-consumer.ps1').read_text(encoding='utf-8').replace('r509','r511').replace('r511c-library-build','r511-library-build').replace('r511-character-consumer','r511-layer-consumer');(P/'tools/build-r511-consumer.ps1').write_text(s,encoding='utf-8')
root=P/'target/r511-player-entry-candidate';root.mkdir();main=root/'LiToModern-R511.rs';main.write_text((P/'target/r509-player-entry-candidate/LiToModern-R509.rs').read_text(encoding='utf-8').replace('r509','r511').replace('R509','R511'),encoding='utf-8');v=read(L/'r509-entry-source-contract.json');v['candidate_main']=pin(main);v['new_graphics']='Conditional denseIMG CPUlayerAPI only;935 explicitunsupported and no liveowner';save(L/'r511-entry-source-contract.json',v)
s=(P/'tools/build-r509-player.ps1').read_text(encoding='utf-8').replace('r509','r511').replace('R509','R511').replace('r511c-library-build','r511-library-build');(P/'tools/build-r511-player.ps1').write_text(s,encoding='utf-8')
# New independent composite combines batch9 real933/934 IMG words and batch5
# sealed span words with batch7 LE16 selection table; Rust output never enters.
B=Path(r'D:/lineage開發/3-lito開發/lito脫殼資源');contract=read(B/'skill-pane-layer-composition-contract.json');fcontract=read(B/'skill-pane-frame-coordinate-contract.json');raws=[Path(contract['qualified_assets'][0]['member']['raw']['path']).read_bytes(),Path(fcontract['qualified_single_resource']['header']['resource']['path']).read_bytes()];vectors=[read(e['path']) for e in read(B/'native-icon-draw-contract.json')['reusable_vectors']]
for flag,name,folder in [(0,'555','RGB555'),(1,'565','RGB565')]:
 shifts=[0,6,11] if flag else [0,5,10];words=[0x2108]*(640*480)
 def convert(v):return (v+(v&0x7fe0))&65535 if flag else v
 for layer,raw in enumerate(raws):
  w=int.from_bytes(raw[0:2],'little');h=int.from_bytes(raw[2:4],'little');imgflag=int.from_bytes(raw[4:6],'little');key=int.from_bytes(raw[6:8],'little');key=convert(key) if imgflag else key;source=[convert(int.from_bytes(raw[i:i+2],'little')) for i in range(8,len(raw),2)];x,y=[441,10] if layer==0 else [433,0]
  for row in range(h):
   for col in range(w):
    src=source[row*w+col]
    if src==key:continue
    at=(y+row)*640+x+col;dst=words[at]
    words[at]=sum(((((dst>>shift)&31)*16+((src>>shift)&31)*15)//31)<<shift for shift in shifts) if layer==0 else src
 # Controls are explicit synthetic markers to verify composition order.
 for x in [470,471]:words[43*640+x]=convert(0x7c00)
 table=(B/'skill-pane-render-initialization'/folder/'selection-83673C.le16').read_bytes();lut=[int.from_bytes(table[i:i+2],'little') for i in range(0,len(table),2)]
 for slot,vector,timed,marked in [(0,vectors[0],False,False),(1,None,False,False),(2,vectors[3],True,False),(3,vectors[1],False,True)]:
  x=470+34*(slot%4);y=43+34*(slot//4)
  if vector:
   dx,dy=vector['offset_signed']
   for row in vector['rows']:
    for span in row['spans']:
     for j,raw in enumerate(span['words']):
      v=convert(raw)
      if timed:v=sum((((v>>shift)&31)//2)<<shift for shift in shifts)
      words[(y+dy+row['y'])*640+x+dx+span['x']+j]=v
  if marked:
   for yy in range(y-1,y+33):
    for xx in range(x-1,x+33):
     at=yy*640+xx;v=words[at];words[at]=(lut[v&255]+lut[256+(v>>8)])&65535
 out=L/f'r511-independent-prefix-{name}.le16';out.write_bytes(b''.join(v.to_bytes(2,'little') for v in words))
save(L/'r511-independent-prefix-inputs.json',dict(oracle='Independent Python dense/raw/span/table composite; no Rust-derived output',outputs=[pin(L/f'r511-independent-prefix-{n}.le16') for n in ['555','565']],final935='Not drawn, externalfloat2intLUT boundary',controls='Synthetic two-pixel callback markers; no actualNativecontrol qualification',native_GUI_oracle=False))
print('Frozen',len(pins),'files; changes',len(changes),'independent prefix vectors ready')
