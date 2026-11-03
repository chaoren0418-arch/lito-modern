from r491_common import *
import re,struct
from PIL import Image
e=read(L/'r496-source-evidence.json');asm=Path(e['original_disassembly']['path']).read_text(encoding='utf-8');O=P/'target/r497-ui-entry-study';O.mkdir(exist_ok=False);fragments=[]
for name in ['sub_46BA10','sub_46B7D0','sub_4764F0','sub_494120','sub_4928C0','sub_4934A0']:
 m=re.search('^'+name+r' proc near',asm,re.M);z=re.search(r'^\w+ proc near',asm[m.end():],re.M);p=O/(name+'.asm');p.write_text(asm[m.start():m.end()+z.start()],encoding='utf-8');fragments.append(dict(name=name,**pin(p)))
skin=read(L/'r478-native-menu-skin-binding.json')
for q in [skin['LTsurf_index'],skin['LTsurf_pak']]:assert pin(q['path'])==q
idx=Path(skin['LTsurf_index']['path']).read_bytes();pak=Path(skin['LTsurf_pak']['path']).read_bytes();members=[]
for ident in [0x1885,0x1887]:
 name=f'{ident}.img'.encode();rows=[(i,off,size)for i,(off,n,size)in enumerate(struct.iter_unpack('<I20sI',idx[4:]))if n.split(b'\0')[0]==name];assert len(rows)==1
 i,off,size=rows[0];raw=pak[off:off+size];w,h,flag,key=struct.unpack('<hhHH',raw[:8]);assert size==8+w*h*2;rgba=bytearray()
 for(p,)in struct.iter_unpack('<H',raw[8:]):
  assert p<32768;rgba.extend(tuple(((c<<3)|(c>>2))for c in [(p>>10)&31,(p>>5)&31,p&31])+(255,))
 p=O/f'{ident}.img';p.write_bytes(raw);png=O/f'{ident}.png';Image.frombytes('RGBA',(w,h),bytes(rgba)).save(png)
 members.append(dict(id=ident,index_row=i,offset=off,bytes=size,sha256=pin(p)['sha256'],width=w,height=h,flag=flag,key=key,raw=pin(p),reference=pin(png)))
skin['state']='R497_SAME_QUALIFIED_MENU_SKIN_PLUS_SOURCE_ENTRY_ART';skin['entry_art']=members[1];skin['previous_skin']=pin(L/'r478-native-menu-skin-binding.json');save(L/'r497-native-menu-skin-binding.json',skin)
save(L/'r497-source-evidence.json',dict(state='R497_SOURCE_MAP_COMBO_RECEIVER2_AND_CHARACTER_INDEX_RESOURCE_MAPPING',previous=pin(L/'r496-native-version-actor-admission-selection-controls-handoff.json'),original_disassembly=e['original_disassembly'],analysis_PE=e['analysis_PE'],fragments=fragments,skin=pin(L/'r497-native-menu-skin-binding.json'),entry_members=members,map_mapping='46BA10 inline46D148 pushreceiver2,pushIMG1885,pushIMG1887 ->4764F0 atx246/y128. Child2 of64AD04 invokes46B7D0;not a Win32 IDC4910.',character_mapping='4928C0 calls494120 ECX0/EDX3;callbacktable630F34 and4934A0 altpath. Exact original art/class sprite/hitgeometry/constructor binding tothose4slots remains unqualified;genericModernbuttons are fallback only.',GUI=0))
print('R497 mapped source combo1885/1887:',[(m['id'],m['width'],m['height'],m['flag'])for m in members],';character art/control geometry OPEN')
