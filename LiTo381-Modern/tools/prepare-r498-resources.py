from r491_common import *
import struct,io
from PIL import Image
e=read(L/'r498-source-study.json');b=Path(e['analysis_PE']['path']).read_bytes();O=P/'target/r498-character-ui-study'
def ints(va,n):return list(struct.unpack('<'+'i'*n,b[va-0x400000:va-0x400000+n*4]))
mapping=[]
for cls,variant,va in [(-1,0,0x62FD70),(0,0,0x630130),(0,1,0x6300D0),(1,0,0x62FFB0),(1,1,0x62FF50),(2,0,0x62FEF0),(2,1,0x62FE90),(3,0,0x630070),(3,1,0x630010),(4,0,0x62FE30),(4,1,0x62FDD0),(5,0,0x630190)]:
 raw=b[va-0x400000:va-0x400000+96];p=O/f'animation-{va:X}.bin';p.write_bytes(raw);states=[ints(va+i*24,6)for i in range(4)];mapping.append(dict(class_byte=cls,variant_nonzero=variant,table_va=hex(va),states=[dict(base_png=s[0],frame_limit=s[1],delay=s[2],draw_mode=s[3],dx=s[4],dy=s[5])for s in states],raw=pin(p)))
skin=read(L/'r497-native-menu-skin-binding.json');idx=Path(skin['LTsurf_index']['path']).read_bytes();pak=Path(skin['LTsurf_pak']['path']).read_bytes();rows={n.split(b'\0')[0].decode('ascii'):(i,off,size)for i,(off,n,size)in enumerate(struct.iter_unpack('<I20sI',idx[4:]))}
requests=set(f'{i}.img'for i in [79,80,6201,6202,6303,6304,6314,7482])
for m in mapping:
 for s in m['states']:
  requests.update(f'{s["base_png"]+i}.png'for i in range(s['frame_limit']+1))
members=[];missing=[]
for name in sorted(requests):
 if name not in rows:missing.append(name);continue
 i,off,size=rows[name];raw=pak[off:off+size];p=O/name;p.write_bytes(raw);member=dict(name=name,index_row=i,offset=off,container_bytes=size,raw=pin(p))
 if name.endswith('.img'):
  w,h,flag,key=struct.unpack('<hhHH',raw[:8]);member.update(width=w,height=h,flag=flag,key=key,format='Native IMG header; pixel encoding qualified separately')
 else:
  try:
   im=Image.open(io.BytesIO(raw));im.load();member.update(width=im.width,height=im.height,format=im.format);dst=O/(name+'.bgra');dst.write_bytes(im.convert('RGBA').tobytes('raw','BGRA'));member['bgra']=pin(dst)
  except Exception as ex:member.update(format='OPEN_PNG_DECODING',error=str(ex))
 members.append(member)
save(L/'r498-character-resource-mapping.json',dict(state='R498_STATIC_SOURCE_MAPPING',source=pin(L/'r498-source-study.json'),LTsurf_index=skin['LTsurf_index'],LTsurf_pak=skin['LTsurf_pak'],slots=[dict(slot=i,table_va=hex(0x630DF8+i*60),constructor='495520 case3 ->415A30 ->4154E0 ->414910',image_id=79,x=19+157*i,y=0,callback='4934A0',receiver=i)for i in range(4)],animation_tables=mapping,members=members,missing_members=missing,variant_rule='494E00 tests EDX !=0; all nonzero variants use variant1 table. class -1 empty; class0..4 explicit; other classes use630190.',draw_chain='497150(class@+2D,variant@+2E) ->494E00 ->415D40 ->415D70 state ->415DA0. base_png+frame index via834BD0; draw mode0->4B9F60,mode1->83155C.',animation_lifecycle_open='4D65F0 timer ->4159B0 ->496660 prefetch; selected3Dh startup/expiry and draw integration not yet reproduced.',GUI=0))
print('members',len(members),'missing',len(missing));print('IMG',[(m['name'],m.get('width'),m.get('height'),m.get('flag'))for m in members if m['name'].endswith('.img')]);print('PNG',[(m['name'],m.get('format'),m.get('width'),m.get('height'))for m in members if m['name'].endswith('.png')][:12])
