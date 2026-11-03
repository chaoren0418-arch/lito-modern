"""Read canonical archive bytes and SPR headers only. No resource probe or test body."""
from pathlib import Path
from datetime import datetime,timezone
import json,struct,hashlib
from native_E149_recovery_contract import R,read,sha,encoded,require
V=R/'驗證紀錄/current'
def header(data):
 at=0
 def take(fmt):
  nonlocal at
  n=struct.calcsize(fmt);require(at+n<=len(data),'SPR header truncated');v=struct.unpack_from(fmt,data,at);at+=n;return v
 first=take('<B')[0];palette=first==255
 if palette:
  n=take('<B')[0] or 256;at+=2*n;count=take('<B')[0]
 else:count=first
 require(1<=count<=255,'SPR frame header shape');frames=[]
 for i in range(count):
  left,top,right,bottom,u1,u2,refs=take('<hhhhHHH');require(refs<=65536,'SPR ref count');blocks=[]
  for _ in range(refs):a,b,depth,block=take('<bbBH');blocks.append(dict(a=a,b=b,depth=depth,block=block))
  frames.append(dict(index=i,left=left,top=top,right=right,bottom=bottom,unknown1=u1,unknown2=u2,references=blocks))
 blocks=take('<I')[0];require(blocks<=65536 and at+(blocks+1)*4<=len(data),'SPR table header');offsets=[take('<I')[0]for _ in range(blocks)];metadata=take('<I')[0]
 return dict(has_palette=palette,frame_count=count,frames=frames,block_count=blocks,block_offsets=offsets,block_table_metadata=metadata,body_offset=at,body_bytes=len(data)-at,pixels_decoded=False)
def main():
 manifest=Path('D:/lineage開發/3-lito開發/lito脫殼資源/resources/archive-manifest.json');m=read_external(manifest);a=next(x for x in m['archives']if x['stem']=='Sprite');index=Path(a['index_source']['path']);pak=Path(a['pak_source']['path']);require(sha(index)==a['index_source']['sha256'],'canonical index changed');require(pak.stat().st_size==a['pak_source']['bytes'],'canonical pak size changed');print('hashing readonly canonical Sprite.pak; no executable invoked',flush=True);before=pak.stat();digest=sha(pak);require(digest==a['pak_source']['sha256'],'canonical pak generation changed');raw=index.read_bytes();count=struct.unpack_from('<I',raw)[0];require(len(raw)==4+count*28,'index layout changed')
 out=V/'native-E149-hide97-canonical'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);members=[]
 with pak.open('rb')as f:
  for name in ['2941-0.spr','1120-0.spr','2942-0.spr']:
   records=[]
   for row in range(count):
    off,word,size=struct.unpack_from('<I20sI',raw,4+row*28);actual=word.split(b'\0')[0].decode('ascii')
    if actual.lower()==name:records.append((row,off,actual,size))
   require(len(records)==1,'canonical member collision/missing '+name);row,off,actual,size=records[0];require(off+size<=before.st_size and size<=20000,'member bound');f.seek(off);data=f.read(size);require(len(data)==size,'member short read');p=out/actual;p.write_bytes(data);members.append(dict(requested_name=name,actual_name=actual,row=row,offset=off,bytes=size,path=str(p),sha256=sha(p),header=header(data)))
 after=pak.stat();require((before.st_size,before.st_mtime_ns)==(after.st_size,after.st_mtime_ns),'pak changed during snapshot');(out/'inspector.py').write_bytes(Path(__file__).read_bytes());pins={str(p):sha(p)for p in out.iterdir()};p=out/'canonical.json';p.write_bytes(encoded(dict(schema='E149-hide97-canonical-member-header-v1',source_pins=pins,external_readonly_pins={str(manifest):sha(manifest),str(index):sha(index)},canonical_pak=dict(path=str(pak),bytes=before.st_size,sha256_observed=digest,stat_before_mtime_ns=before.st_mtime_ns,stat_after_mtime_ns=after.st_mtime_ns),members=members,pixels_decoded=False,archive_or_SPR_probe_executed=False,tests_executed=0,PE_executed=False,whole_goal_complete=False)));print(json.dumps(dict(canonical=str(p),sha256=sha(p),members=[dict(name=m['actual_name'],bytes=m['bytes'],sha256=m['sha256'],frames=m['header']['frame_count'],first_refs=len(m['header']['frames'][0]['references']))for m in members],tests_executed=0),ensure_ascii=False))
def read_external(p):return json.loads(p.read_text('utf-8-sig'))
if __name__=='__main__':main()
