from pathlib import Path
import hashlib, json, re, shutil, struct
P=Path(__file__).resolve().parent.parent; L=P/'logs/continuation-20261003'; O=P/'target/r485-format-study'
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def read(p):return json.loads(Path(p).read_text(encoding='utf-8'))
def save(p,v):
 t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
O.mkdir(exist_ok=False);(O/'authority').mkdir();(O/'members').mkdir();(O/'blocks').mkdir();(O/'native-map').mkdir();(O/'reference').mkdir()
asm=Path(r'D:\lineage開發\3-lito開發\lito登入器\舊版分析與驗收\ida-client-oep-v2\full-disassembly.asm')
assert pin(asm)['sha256']=='2A3A6E1026EE65288C07EFF00990BDDDBEFA5ACEC9EED551F528A0CC0A7DEA9A'
lines=asm.read_bytes().splitlines(keepends=True); starts=[(i,re.match(rb'^(\w+) proc ',s).group(1).decode())for i,s in enumerate(lines)if re.match(rb'^\w+ proc ',s)]
procs=[]
for name in ['sub_483A80','sub_4853E0','sub_483770']:
 n=next(n for n,(_,s)in enumerate(starts)if s==name);start=starts[n][0];end=starts[n+1][0];dest=O/'authority'/f'{name}.asm';dest.write_bytes(b''.join(lines[start:end]));procs.append(dict(name=name,first_line=start+1,next_proc_line=end+1,pin=pin(dest),through_next_proc=True))
refs=[]
for src in [Path(r'D:\lineage開發\PakViewer-main\src\Lin.Helper.Core\Tile\L1Til.cs'),Path(r'D:\lineage開發\PakViewer-main\src\Lin.Helper.Core\Map\S32Reader.cs'),Path(r'D:\lineage開發\PakViewer-main\src\Lin.Helper.Core\Map\S32Data.cs')]:
 dest=O/'authority'/src.name;shutil.copyfile(src,dest);refs.append(dict(original=pin(src),snapshot=pin(dest),scope='Local reference source; not independently asserted to be the original binary version'))
inventory=read(L/'r479-resource-inventory-input.json');batch=[];layer_studies=[]
def layer_study(b):
 off=0;ranges=[];portals=[];layer=1
 def take(n):
  nonlocal off
  if n>len(b)-off:raise ValueError(dict(layer=layer,offset=off,required=n,available=len(b)-off))
  v=b[off:off+n];off+=n;return v
 def u(fmt):return struct.unpack('<'+fmt,take(struct.calcsize('<'+fmt)))[0]
 try:
  take(8192*4);ranges.append([1,0,off]);layer=2;start=off;take(u('H')*6);ranges.append([2,start,off]);layer=3;start=off;take(4096*4);ranges.append([3,start,off]);layer=4;start=off
  for _ in range(u('i')):u('h');take(u('H')*7)
  ranges.append([4,start,off])
  for layer,size in [(5,5),(6,4)]:
   if off==len(b):return dict(ranges=ranges,complete=True)
   start=off;count=u('i');assert 0<=count<=262144;take(count*size);ranges.append([layer,start,off])
  if off==len(b):return dict(ranges=ranges,complete=True)
  layer=7;start=off;count=u('H');count_offset=start
  for i in range(count):
   record=off;length=u('B');available=len(b)-off
   if length+8>available:raise ValueError(dict(layer=7,record=i,count=count,count_offset=count_offset,record_offset=record,length_byte=length,offset=off,required=length+8,available=available,remaining_hex=b[record:].hex()))
   name=take(length);fixed=take(8);portals.append(dict(offset=record,name_length=length,name_hex=name.hex(),fixed_hex=fixed.hex()))
  ranges.append([7,start,off]);layer=8
  if off<len(b):start=off;num=u('H');take((num&32767)*(10 if num&32768 else 6));ranges.append([8,start,off])
  return dict(ranges=ranges,portals=portals,complete=off==len(b),unknown_bytes=len(b)-off)
 except ValueError as e:return dict(ranges=ranges,portals=portals,complete=False,error=e.args[0],no_corruption_inferred=True)
for id in [506,573,563]:
 row=next(r for r in inventory['rows']if r['map_id']==id);directory=Path(row['directory']);assert sorted(p.name for p in directory.glob('*.s32'))==[Path(s['pin']['path']).name for s in row['sectors']];owned=O/'native-map'/str(id);owned.mkdir();sectors=[]
 for s in row['sectors']:
  assert pin(s['pin']['path'])==s['pin'];dest=owned/Path(s['pin']['path']).name;shutil.copyfile(s['pin']['path'],dest);sectors.append({**s,'pin':pin(dest)});layer_studies.append(dict(map_id=id,original=s['pin'],owned=pin(dest),analysis=layer_study(dest.read_bytes())))
 anchor=(sectors[0]['x'],sectors[0]['y']);pos=[(1536*(s['x']-anchor[0]+s['y']-anchor[1]),768*(-s['x']+anchor[0]+s['y']-anchor[1]))for s in sectors];bounds=[min(x for x,y in pos),min(y for x,y in pos),max(x+3072 for x,y in pos),max(y+1536 for x,y in pos)]
 batch.append(dict(mode=row['mode'],map_id=id,name=row['name'],shape='source-exact-'+str(len(sectors))+'sector',measurement_only=False,namespace='NATIVE_LITO_LTMAP_V1',map_root=str(O/'native-map'),archive_root=str(Path(inventory['archives'][0]['path']).parent),sectors=sectors,archives=inventory['archives'],original_sectors=row['sectors'],original_directory=row['directory'],catalog_record=row['catalog_record'],expected_bounds=bounds,anchor=anchor,source_bytes=sum(s['pin']['bytes']for s in sectors)))
diagnostic=read(L.parent.parent/'target/r481-multisector-batch/map506-diagnostic.json') if False else read(P/'target/r481-multisector-batch/map506-diagnostic.json')
blocks=[]
with Path(inventory['archives'][1]['path']).open('rb')as f:
 for member in ['198.til','199.til']:
  rows=[r for r in diagnostic['failures']if r['tile_member']==member];r=rows[0];f.seek(r['archive_member_offset']);raw=f.read(r['archive_member_bytes']);dest=O/'members'/member;dest.write_bytes(raw);assert pin(dest)['sha256']==r['member_sha256'];count=struct.unpack_from('<I',raw)[0];offsets=struct.unpack_from('<'+str(count+1)+'I',raw,4);body=4+4*(count+1);assert offsets[-1]==len(raw)-body
  for r in rows:
   slot=r['slot'];start=offsets[slot];end=min(o for o in offsets if o>start);block=raw[body+start:body+end];out=O/'blocks'/f'{member[:-4]}-{slot}.block';out.write_bytes(block);blocks.append(dict(member=member,member_pin=pin(dest),pak_offset=r['archive_member_offset'],slot=slot,offset_table_entry=4+4*slot,relative_offset=start,body_start=body,member_block_offset=body+start,pak_block_offset=r['archive_member_offset']+body+start,block_bytes=end-start,type=block[0],content_header=list(block[:5]),block_pin=pin(out),prior_required_source_reference=r['first_source_reference']))
save(L/'r485-format-source-evidence.json',dict(state='R485_ORIGINAL_RESOURCES_AND_FULL_NATIVE_PROCS_PINNED',original_disassembly=pin(asm),native_procs=procs,local_reference_sources=refs,blocks=blocks,all_three_complete_source_lists=batch,layer_studies=layer_studies,native_type22_23_evidence='483A80 +7BB bit2 RLE; +7BD bit0x10 selects draw flag8 before bit4 selects256.4853E0 same priority. Geometry/dispatch proven; color policy remains existing qualified local PakViewer RGBA adaptation, not original framebuffer proof.',Layer7_gap='Local C# uses identical u8 name length+8 and silently breaks on shortage. Native Layer7 field producer/variant remains unproven; no skipped records or guessed width accepted.',GUI=0,LOADED=False))
save(L/'r485-resource-input.json',dict(batch=[batch[0]],format_evidence=pin(L/'r485-format-source-evidence.json'),GUI=0,LOADED=False))
print('R485 pinned:3 native procs,3 C# reference sources,10 required22/23 blocks,3 complete original map lists. Layer7 failures:',sum(not r['analysis']['complete']for r in layer_studies))
