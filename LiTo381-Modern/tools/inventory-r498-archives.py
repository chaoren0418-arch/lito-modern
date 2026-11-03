from r491_common import *
import struct
root=Path('D:/lineage開發/0-客戶端/LiTo')
result=[]
for p in sorted(root.glob('*.idx')):
 b=p.read_bytes();names=[]
 if (len(b)-4)%28==0:
  for off,n,size in struct.iter_unpack('<I20sI',b[4:]):
   n=n.split(b'\0')[0]
   if n.endswith(b'.png') or n in [b'79.img',b'80.img']:names.append(dict(name=n.decode('ascii','replace'),offset=off,bytes=size))
 result.append(dict(index=pin(p),count=struct.unpack('<I',b[:4])[0],length=len(b),matching=names))
save(L/'r498-archive-inventory.json',dict(indices=result,search='plain old28 PNG names and79/80.img inall5available original client indices;protectedLTtext notclaimeddecoded',sourcehash_mod16='43FFC0 sums filenameASCII&15 ->6479E8 shards;4D5270 fallback64798C',GUI=0))
print([(Path(q['index']['path']).name,q['count'],len(q['matching']),q['matching'][:2])for q in result])
