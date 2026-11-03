"""Read-only Native assets/ASM; immutable owned snapshot and independent RGB555 reference."""
from pathlib import Path
import hashlib,json,re,shutil,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r476-native-menu-skin'
O.mkdir(exist_ok=False);(O/'source').mkdir();(O/'members').mkdir();(O/'reference').mkdir()
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8')
root=Path(r'D:\lineage開發\0-客戶端\LiTo');alt=Path(r'D:\380\天堂(Lineage 3.81C) (1)\LiTo')
containers=[]
for name,h in [('LTsurf.idx','C699CCF22C5F59EB37E8936D939D2632C15AC381980A6FE3F26F2565185193BF'),('LTsurf.pak','42E676C255B248C295A71097A53B1B67BF73A0C2C3DC0FE11D12BDB9EE79621E')]:
 before=pin(root/name);assert before['sha256']==h;dst=O/'source'/name;shutil.copyfile(root/name,dst);after=pin(root/name);assert before==after and pin(dst)['sha256']==h
 other=pin(alt/name);assert other['sha256']==h;containers.append(dict(original=before,alternate=other,snapshot=pin(dst)))
i=(O/'source/LTsurf.idx').read_bytes();p=(O/'source/LTsurf.pak').read_bytes();n=struct.unpack_from('<I',i)[0];assert n==1623 and len(i)==4+n*28
rows={}
for j in range(n):
 off,name,size=struct.unpack_from('<I20sI',i,4+j*28);name=name.split(b'\0')[0].decode('ascii');assert name not in rows and off+size<=len(p);rows[name]=(j,off,size)
members=[]
for rid in [6113,6232,6233,6234,6235]:
 name=f'{rid}.img';j,off,size=rows[name];d=p[off:off+size];w,h,flag,mask=struct.unpack_from('<hhHH',d);assert w>0 and h>0 and size==8+w*h*2 and flag in (0,1)
 dst=O/'members'/name;dst.write_bytes(d);rgba=bytearray()
 for v, in struct.iter_unpack('<H',d[8:]):
  if flag==1 and v==mask:rgba.extend([0,0,0,0])
  else:
   ch=[(v>>10)&31,(v>>5)&31,v&31];rgba.extend([(c<<3)|(c>>2) for c in ch]+[255])
 png=O/'reference'/f'{rid}.png';Image.frombytes('RGBA',(w,h),bytes(rgba)).save(png)
 members.append(dict(id=rid,name=name,index_row=j,offset=off,bytes=size,sha256=pin(dst)['sha256'],raw=pin(dst),reference_png=pin(png),rgba_sha256=hashlib.sha256(rgba).hexdigest().upper(),width=w,height=h,mask_flag=flag,mask_word=mask,localized_candidates_present=[f'{rid}{x}.img' for x in 'echj' if f'{rid}{x}.img' in rows]))
asm=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm');s=asm.read_text(encoding='utf-8');assert pin(asm)['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
fragments=[]
for a in ['46B7D0','4645D0','4646C0','474280','4744F0','4746A0','4D4940','4D4D40','4D4D90','414910','4E99C0','4E9A40','477AB0','477930','477990','474350','474360','4B9F60','4BA220','4BB450']:
 m=re.search(r'^sub_'+a+r' proc near',s,re.M);assert m;e=re.search(r'^sub_[0-9A-F]+ proc near',s[m.end():],re.M);end=m.end()+e.start() if e else len(s)
 dst=O/'source'/f'asm-{a}-through-next-proc.txt';dst.write_text(s[m.start():end],encoding='utf-8');fragments.append(dict(address=a,pin=pin(dst),scope='declared body plus post-endp continuation through next proc boundary; data/unknown calls retained, not executed'))
viewer=Path(r'D:\lineage開發\PakViewer-main\src\Lin.Helper.Core\Image\ImageConverter.cs');shutil.copyfile(viewer,O/'source/ImageConverter.cs')
manifest=dict(state='QUALIFIED_NATIVE_MAP_MENU_PANEL_AND_ARROW_IMAGES_RGB555_REFERENCE',LTsurf_index=containers[0]['snapshot'],LTsurf_pak=containers[1]['snapshot'],containers=containers,members=members,original_asm=pin(asm),asm_fragments=fragments,viewer_reference=pin(O/'source/ImageConverter.cs'),mapping=dict(panel=dict(id=6113,registry_byte_offset='5F84h',native_size=[158,82],crop=[0,0,157,81],native_destination=[424,310],native_hit=[424,310,581,391],source='46B7D0 -> 474280 vtable4F9058 -> 4744F0; full panel copied at0,0 then source157x81 Blt at424,310'),up=dict(normal=6232,pressed=6233,native_hit=[566,313,577,325],source='477AB0 post-endp 4781C8..4781EF; group64B40C child8 ->4E99C0/414910;4E9A40 +2Dh state adds1'),down=dict(normal=6234,pressed=6235,native_hit=[565,375,576,387],source='477AB0 post-endp 478216..47823D; group64B40C child9;4E9A40 +2Dh state adds1'),row_rect=[433,312,568,387],row_origin=[2,2],row_pitch=12,original_callbacks='477930/477990 dispatch virtual+0C/+10 to current64B434; mapmenu474350/474360 change row scroll by1',modern_adaptation='One qualified map564 row, arrows therefore disabled; native normal/pressed image states can be examined via CPU contract only. Modern enter/back/close/network remain standard controls.',palette='No palette; direct little-endian RGB555; optional native555->565 conversion4BB450 is separate, RGB555 CPU reference only',transparent='Panel opaque copy4B9F60; arrow color-key compare4BA220 word+6; asset flag1 mask6342',language='All5 selected IDs have no e/c/h/j members in complete pinned LTsurf index; plain fallback only; no full runtime overlay/language grant'),GUI=0,LOADED=False,complete_modes=16,total_modes=40)
save(L/'r476-native-menu-skin-binding.json',manifest)
print(json.dumps(dict(binding=pin(L/'r476-native-menu-skin-binding.json'),members=len(members),index_members=n)))
