"""Bounded inspection of both original149 SPR sources. Reader derived from the existing documented SPR contract; not an old-executable framebuffer oracle."""
from pathlib import Path
import hashlib,json,struct
class Cursor:
 def __init__(self,b):self.b=b;self.at=0
 def take(self,fmt):
  n=struct.calcsize(fmt);assert 0<=self.at<=len(self.b)-n,('truncated',self.at,n);v=struct.unpack_from(fmt,self.b,self.at);self.at+=n;return v[0] if len(v)==1 else v

def decode(b):
 assert 0<len(b)<=1024*1024 and b[:3] not in [b'SP2',b'SPX'];c=Cursor(b);first=c.take('<B');assert first!=254;pal=[]
 if first==255:
  size=c.take('<B') or 256;pal=[c.take('<H') for _ in range(size)];count=c.take('<B')
 else:count=first
 assert 0<count<=255;defs=[];refsum=0
 for _ in range(count):
  l,t,r,bt,u1,u2,n=c.take('<hhhhHHH');refs=[c.take('<bbBH') for _ in range(n)];refsum+=n;assert refsum<=65536;defs.append((l,t,r,bt,u1,u2,refs))
 n=c.take('<I');assert n<=16384;offsets=[c.take('<I') for _ in range(n)];table_meta=c.take('<I');body=b[c.at:];bounds=sorted(set(offsets));assert (not bounds and not body) or (bounds and bounds[0]==0 and bounds[-1]<len(body));bounds.append(len(body));end={bounds[i]:bounds[i+1] for i in range(len(bounds)-1)};blocks=[]
 for offset in offsets:
  q=Cursor(body[offset:end[offset]]);sx,sy,ew,lines=q.take('<BBBB');assert sx<=24 and sy+lines<=24;pixels=[0x8000]*576
  for y in range(sy,sy+lines):
   x=sx
   for _ in range(q.take('<B')):
    skip,run=q.take('<BB');assert skip%2==0;x+=skip//2;assert x+run<=24
    for _ in range(run):
     color=pal[q.take('<B')] if pal else q.take('<H');pixels[y*24+x]=color;x+=1
  assert q.at==len(q.b);blocks.append(pixels)
 frames=[];metadata=[];total=0
 for sub,(l,t,r,bt,u1,u2,refs) in enumerate(defs):
  raw_bounds=[l,t,r,bt];sentinel=not refs and ((l,t,r,bt,u1,u2) in [(192,120,-128,-120,65535,0),(192,120,-128,-120,0x0f0f,0xf1f1),(432,132,-368,-468,65535,0)])
  placements=[]
  for a,v,depth,block in refs:
   assert block<len(blocks);half=a//2;placements.append((24*(v+a-half),12*(v-half),block,depth))
  if sentinel:w=h=0
  elif placements:
   l=min(x for x,y,block,depth in placements);t=min(y for x,y,block,depth in placements);r=max(x+23 for x,y,block,depth in placements);bt=max(y+23 for x,y,block,depth in placements);w=r-l+1;h=bt-t+1
  else:w=r-l+1;h=bt-t+1
  assert 0<=w<=1024 and 0<=h<=1024;total+=w*h;assert total<=16*1024*1024;rgba=bytearray(w*h*4)
  for px,py,block,depth in placements:
   for i,color in enumerate(blocks[block]):
    if color==0x8000:continue
    x=px-l+i%24;y=py-t+i//24;assert 0<=x<w and 0<=y<h;at=(y*w+x)*4;rgb=[(color>>10)&31,(color>>5)&31,color&31];rgba[at:at+4]=bytes([(v<<3)|(v>>2) for v in rgb]+[255])
  frames.append(bytes(rgba));metadata.append(dict(subframe=sub,x=l,y=t,width=w,height=h,unknown1=u1,unknown2=u2,raw_declared_bounds=raw_bounds,block_references=len(refs),blank_sentinel=sentinel,rgba_bytes=len(rgba),opaque_pixels=sum(rgba[i]!=0 for i in range(3,len(rgba),4)),rgba_sha256=hashlib.sha256(rgba).hexdigest().upper()))
 return metadata,frames,dict(has_palette=bool(pal),palette_entries=len(pal),block_count=n,block_table_metadata=table_meta,block_references=refsum)
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'驗證紀錄/current/native-light-spell/149-source-variants.json'
assert not OUT.exists()
root=Path(r'D:\lineage開發\0-客戶端');rows=[];name=b'149-0.spr';shard=sum(name)&15
for idx in [root/'Sprite.idx',root/f'Sprite{shard:02}.idx']:
 raw=idx.read_bytes();count=struct.unpack_from('<I',raw)[0];assert len(raw)==4+28*count
 entries=[struct.unpack_from('<I20sI',raw,4+28*i) for i in range(count)]
 matches=[(o,n,z) for o,n,z in entries if n.split(b'\0')[0].lower()==name];assert len(matches)==1
 offset,_,size=matches[0];assert 0<size<1024*1024
 with idx.with_suffix('.pak').open('rb') as f:f.seek(offset);b=f.read(size)
 assert len(b)==size
 metadata,frames,header=decode(b)
 rows.append({'index':str(idx),'index_sha256':hashlib.sha256(raw).hexdigest().upper(),'offset':offset,'bytes':size,'sha256':hashlib.sha256(b).hexdigest().upper(),'frames':metadata,'header':header})
selected=rows[1];assert selected['frames'][0]['rgba_sha256']=='6DB65FD59FD356F6729140571B5BCD6BB3B83492A16E1BF0A3884442FC3C8A0E'
OUT.write_text(json.dumps({'selection':'existing SpriteReader explicitly tries ASCII-name checksum shard before base; these duplicate bytes are different','selected_shard':shard,'sources':rows,'source_order_is_not_independent_original_executable_proof':True,'actual_sessions':0,'whole_goal_complete':False},ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
print(json.dumps({'selected_shard':shard,'sources':[(v['bytes'],v['sha256'],v['frames']) for v in rows]}))
