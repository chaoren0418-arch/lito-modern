from pathlib import Path
import json,hashlib,struct
from PIL import Image
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r478-mode-menu/reference'
def pin(p):
 b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
m=json.loads((L/'r478-native-menu-skin-binding.json').read_text(encoding='utf-8'));images={}
for row in m['members']:
 b=Path(row['raw']['path']).read_bytes();w,h,flag,key=struct.unpack('<hhHH',b[:8]);rgba=bytearray()
 for v, in struct.iter_unpack('<H',b[8:]):
  c=tuple(((x<<3)|(x>>2)) for x in ((v>>10)&31,(v>>5)&31,v&31))
  rgba.extend((0,0,0,0) if row['id']!=6113 and v==key else c+(255,))
 images[row['id']]=Image.frombytes('RGBA',(w,h),bytes(rgba))
refs=[]
for top,count,pressed in [(0,0,'normal'),(0,1,'normal'),(0,17,'normal'),(1,17,'down'),(8,17,'up'),(16,17,'normal'),(39,40,'normal'),(41,42,'normal')]:
 y=15 if count<=1 else 15+41*top//(count-1);canvas=images[6113].crop((0,0,157,81));canvas.alpha_composite(images[6231],(143,y));canvas.alpha_composite(images[6233 if pressed=='up' else 6232],(142,3));canvas.alpha_composite(images[6235 if pressed=='down' else 6234],(141,65))
 path=O/f'menu-{count}-{top}-{pressed}.png';canvas.save(path);refs.append(dict(top=top,count=count,pressed=pressed,thumb_y=y,pin=pin(path),rgba_sha256=hashlib.sha256(canvas.tobytes()).hexdigest().upper()))
(L/'r478-pixel-references.json').write_text(json.dumps(dict(thumb_reference=m['members'][-1],composites=refs,order='opaque panel crop; color-key thumb; color-key child arrows'),ensure_ascii=False,indent=2),encoding='utf-8')
print('Eight independent thumb/arrow composites prepared')
