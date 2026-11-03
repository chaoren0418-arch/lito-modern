from pathlib import Path
import json,struct,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r485-format-study'
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
a=json.loads((L/'r485-format-source-evidence.json').read_text(encoding='utf-8'))
# Independent cursor follows the pinned native loc_484C49..484C91 pointer steps.
def native(b):
 o=0;r=[];portals=[]
 def get(fmt):
  nonlocal o
  n=struct.calcsize('<'+fmt);assert o+n<=len(b);v=struct.unpack_from('<'+fmt,b,o);o+=n;return v[0]if len(v)==1 else v
 def skip(n):
  nonlocal o
  assert 0<=n<=len(b)-o;o+=n
 skip(32768);r.append([0,o]);s=o;skip(get('H')*6);r.append([s,o]);s=o;skip(16384);r.append([s,o]);s=o
 for _ in range(get('i')):get('h');skip(get('H')*7)
 r.append([s,o])
 for size in [5,4]:
  if o==len(b):return dict(ranges=r,portals=portals,effects=0,complete=True)
  s=o;skip(get('i')*size);r.append([s,o])
 if o==len(b):return dict(ranges=r,portals=portals,effects=0,complete=True)
 s=o;count=get('H')
 for i in range(count):
  start=o;n=get('B');name=b[o:o+n];skip(n);x,y,target,payload=get('BBHi');kind=None
  if o<len(b)and 100<=b[o]<=102:kind=get('B')
  portals.append(dict(index=i,offset=start,end=o,name_hex=name.hex(),x=x,y=y,target_map=target,legacy_fixed_i32=payload,native_kind=kind))
 r.append([s,o]);effects=0;extended=False
 if o<len(b):s=o;count=get('H');extended=bool(count&32768);effects=count&32767;skip(effects*(10 if extended else 6));r.append([s,o])
 assert o==len(b),('unknown suffix',len(b)-o)
 return dict(ranges=r,portals=portals,effects=effects,extended=extended,complete=True)
studies=[]
for r in a['layer_studies']:
 p=Path(r['owned']['path']);assert pin(p)==r['owned'];studies.append(dict(map_id=r['map_id'],sector=pin(p),native_reference=native(p.read_bytes()),old_reference=r['analysis']))
save(L/'r485-native-layer-reference.json',dict(state='R485_ALL29_NATIVE_SECTORS_BOUNDED_INDEPENDENT_POINTER_REFERENCE_COMPLETE',sectors=studies,original_proc=a['native_procs'][0],source_rule='Optional signed byte100..102 only; not unconditional9B, not name width change, not skip malformed records',old_failed_sectors=sum(not r['old_reference']['complete']for r in studies)))
cases=[dict(name=f"{b['member'][:-4]}-{b['slot']}",block=b['block_pin'],actual=True,member=b['member'],slot=b['slot'])for b in a['blocks']]
colors=[0,0x421,0x7fff,0x7fe0,0x8000,0x8421,0xffff,0xffe0,0x400,0x20,1,0x739c,0x6318,0x3def,0x001f,0x03e0,0x7c00,0x7c1f,0x03ff,0x7bde,0x4210,0x5294,0x56b5,0x57b5]
for kind in [22,23]:
 b=bytes([kind,0,0,24,1,1,0,24])+b''.join(struct.pack('<H',c)for c in colors);p=O/'blocks'/f'synthetic-{kind}.block';p.write_bytes(b);cases.append(dict(name=f'synthetic-{kind}',block=pin(p),actual=False,source_colors=colors))
save(L/'r485-tile-reference-input.json',dict(cases=cases,authority=a['local_reference_sources'][0],output_root=str(O/'reference'),scope='Unchanged entire local L1Til.cs RenderBlockToBgra effects; original geometry/dispatch independently pinned; no original framebuffer proof'))
save(L/'r485-resource-input.json',dict(batch=a['all_three_complete_source_lists'],format_evidence=pin(L/'r485-format-source-evidence.json'),native_layer_reference=pin(L/'r485-native-layer-reference.json'),GUI=0,LOADED=False))
print('Native independent pointer reference:29 complete source sectors;5 prior shortages closed. Tile12 C# reference inputs prepared.')
