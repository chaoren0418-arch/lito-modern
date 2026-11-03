"""Separate bounded Python SPR reader from the documented Rust/PakViewer contract.
Cross-check the known11722 trace; this does not execute or independently oracle the old client.
"""
from pathlib import Path
import hashlib,json,struct,zipfile
from PIL import Image
ROOT=Path(__file__).resolve().parents[1];OUT=ROOT/'驗證紀錄/current/native-shared-healing-family-SPR'
assert not OUT.exists();OUT.mkdir()
read=lambda p:json.loads(Path(p).read_text(encoding='utf-8-sig'))
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
class Cursor:
 def __init__(self,b):self.b=b;self.at=0
 def take(self,fmt):
  n=struct.calcsize(fmt);assert 0<=self.at<=len(self.b)-n,('truncated',self.at,n);v=struct.unpack_from(fmt,self.b,self.at);self.at+=n;return v[0] if len(v)==1 else v

def decode(b):
 assert 0<len(b)<=1024*1024 and b[:3] not in [b'SP2',b'SPX'];c=Cursor(b);first=c.take('<B');assert first!=254;pal=[]
 if first==255:
  size=c.take('<B') or 256;pal=[c.take('<H') for _ in range(size)];count=c.take('<B')
 else:count=first
 assert count==21;defs=[];refsum=0
 for _ in range(count):
  l,t,r,bt,u1,u2,n=c.take('<hhhhHHH');refs=[c.take('<bbBH') for _ in range(n)];refsum+=n;assert refsum<=65536;defs.append((l,t,r,bt,u1,u2,refs))
 n=c.take('<I');assert n<=16384;offsets=[c.take('<I') for _ in range(n)];table_meta=c.take('<I');body=b[c.at:];bounds=sorted(set(offsets));assert bounds[0]==0 and bounds[-1]<len(body);bounds.append(len(body));end={bounds[i]:bounds[i+1] for i in range(len(bounds)-1)};blocks=[]
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
source=read(ROOT/'驗證紀錄/current/native-shared-healing-family-source/raw.json');entries=source['SPR_sources'][0];known=read(ROOT/'驗證紀錄/current/native-shared-basic-heal/original-frame-metadata.json');observations={};pngs=[]
for entry in sorted(entries['entries'],key=lambda x:x['member']):
 pak=Path(entries['root'])/Path(entry['index']).with_suffix('.pak')
 with pak.open('rb') as f:f.seek(entry['offset']);payload=f.read(entry['bytes'])
 assert hashlib.sha256(payload).hexdigest().upper()==entry['sha256'];metadata,rgba,header=decode(payload);gfx=int(entry['member'].split('-')[0])
 if gfx==11722:
  assert len(metadata)==len(known)==21
  for actual,expect in zip(metadata,known):
   assert all(actual[k]==v for k,v in expect.items()),(actual,expect)
 # Contact sheets preserve each frame's original offsets relative to a fixed anchor.
 tilew,tileh=360,360;sheet=Image.new('RGBA',(tilew*7,tileh*3),(12,18,24,255))
 for m,pixels in zip(metadata,rgba):
  if not pixels:continue
  frame=Image.frombytes('RGBA',(m['width'],m['height']),pixels);sub=m['subframe'];at=((sub%7)*tilew+180+m['x'],(sub//7)*tileh+180+m['y']);sheet.alpha_composite(frame,at)
 path=OUT/(str(gfx)+'-raw-frame-contact-sheet.png');sheet.save(path);pngs.append(path)
 observations[str(gfx)]={'source':entry,'header':header,'frames':metadata,'visible_subframes':[x['subframe'] for x in metadata if x['opaque_pixels']>0],'blank_subframes':[x['subframe'] for x in metadata if x['rgba_bytes']==0],'contact_sheet_sha256':sha(path)}
raw=dict(status='separate_Python_bounded_healing_SPR_inspection_crosschecked_only_against_retained11722_Rust_trace',SPR_reader_contract='Port of current documented Rust/PakViewer semantics; not an independent format-source oracle.',original11722_21_slots_match_retained_Rust_metadata_and_RGBA_SHA=True,sprites=observations,actual_game_sessions=0,original_native_executable_run=False,new_core_playback_of11723_11724_11725_verified=False,full_world_pixel_oracle=False,whole_goal_complete=False)
(OUT/'inspection.json').write_text(json.dumps(raw,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
files={'inspector.py':Path(__file__),'inspection.json':OUT/'inspection.json','source-audit.json':ROOT/'驗證紀錄/current/native-shared-healing-family-source/qualified.json','Rust-SPR-contract.rs':ROOT/'LiTo381-Modern/src/modern_lito/sprite.rs','known11722-metadata.json':ROOT/'驗證紀錄/current/native-shared-basic-heal/original-frame-metadata.json'}
files.update({p.name:p for p in pngs});fp={n:sha(p) for n,p in files.items()};archive=OUT/'SPR-inspection-evidence.zip'
with zipfile.ZipFile(archive,'x',zipfile.ZIP_DEFLATED,compresslevel=1) as z:
 for n,p in files.items():z.write(p,n)
with zipfile.ZipFile(archive) as z:
 for n,h in fp.items():assert hashlib.sha256(z.read(n)).hexdigest().upper()==h
summary={k:{'visible_subframes':v['visible_subframes'],'blank_subframes':v['blank_subframes']} for k,v in observations.items()}
(OUT/'qualified.json').write_text(json.dumps(dict(status=raw['status'],SPR_slot_summary=summary,known11722_all21_metadata_and_RGBA_SHA_match=True,archive_file_pins=fp,archive_sha256=sha(archive),archive_readback_verified=True,new_core_playback_verified=False,full_world_pixel_oracle=False,whole_goal_complete=False),ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(summary))
