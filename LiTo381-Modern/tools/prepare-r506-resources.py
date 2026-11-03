from r491_common import *
import struct
from PIL import Image
skin=read(L/'r498-native-menu-skin-binding.json');idx=Path(skin['LTsurf_index']['path']).read_bytes();pak=Path(skin['LTsurf_pak']['path']).read_bytes()
assert pin(skin['LTsurf_index']['path'])==skin['LTsurf_index'] and pin(skin['LTsurf_pak']['path'])==skin['LTsurf_pak']
rows={n.split(b'\0')[0].decode('ascii'):(i,off,size)for i,(off,n,size)in enumerate(struct.iter_unpack('<I20sI',idx[4:]))}
root=P/'target/r506-channel-resource-study';root.mkdir(exist_ok=False);members=[]
for ident in [7580,*range(7581,7589),6205,6206]:
 i,off,size=rows[f'{ident}.img'];raw=pak[off:off+size];path=root/f'{ident}.img';path.write_bytes(raw);w,h,flag,key=struct.unpack('<hhHH',raw[:8]);assert flag<=1 and len(raw)==8+w*h*2
 rgba=bytearray()
 for (v,)in struct.iter_unpack('<H',raw[8:]):
  assert v&0x8000==0
  if ident!=7580 and v==key:rgba.extend([0,0,0,0])
  else:rgba.extend([((x<<3)|(x>>2))for x in [(v>>10)&31,(v>>5)&31,v&31]]+[255])
 rp=root/f'{ident}.rgba';rp.write_bytes(rgba);im=Image.frombytes('RGBA',(w,h),bytes(rgba));png=root/f'{ident}.png';im.save(png)
 members.append(dict(id=ident,index_row=i,offset=off,bytes=size,sha256=pin(path)['sha256'],width=w,height=h,flag=flag,key=key,raw=pin(path),reference=pin(rp),reference_png=pin(png)))
skin.update(state='R506_SOURCE_PAGE40_CHANNEL_IMAGES',previous_R498_skin=pin(L/'r498-native-menu-skin-binding.json'),channel_background=members[0],channel_buttons=members[1:9],channel_back=members[9:],channel_source_table=pin(L/'r506-source-fragments/pe-00630B20.txt'),GUI=0)
save(L/'r506-native-menu-skin-binding.json',skin)
save(L/'r506-channel-resources.json',dict(members=members,skin_binding=pin(L/'r506-native-menu-skin-binding.json'),GUI=0,no_player_execution=True))
print([(m['id'],m['width'],m['height'])for m in members])
