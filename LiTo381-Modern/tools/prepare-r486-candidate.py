from pathlib import Path
import json,hashlib,shutil,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r486-paged-city'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):t=p.with_suffix('.json.tmp');t.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
a=read(L/'r486-resource-input.json');b=a['batch'][0];ref=read(O/'reference/reference.json');assert ref['passed']==1 and not ref['failed'];r=ref['results'][0];assert len(r['images'])==25 and r['maximum_visible_decoded_bytes']==9469440 and r['cumulative_visible_decoded_bytes']==29592576
old=read(L/'r485-map-binding.json');assert len(old['bindings'])==39
new={k:b[k]for k in ['mode','map_id','namespace','map_root','archive_root','sectors','archives']}|dict(canonical_sha256=r['canonical_sha256'])
binding=old|dict(state='R486_FORTY_WHOLE_MAP_PLANS_LAST563_BOUNDED_CACHE_CPU_PENDING',bindings=old['bindings']+[new],unqualified_count=0,remaining_unqualified_maps=0,new_complete_multisector_ids=[563],multisector_probe=pin(O/'reference/reference.json'),multi_sector_scope='563 complete19sectors/31444required slots all preflight decoded one-by-one; immutable encoded member residence<=32MiB, opt-in cache16MiB<=64MiB decoded limit; all visible tiles fit or frame refuses atomically. Existing39 plans remain unchanged.',measured_unqualified_city_ids=[],remaining_preview_gap=None,bounded_cache_policy='Modern per-city residence only; qualification keeps every necessary slot. Source grammar/whole geometry/original budgets unchanged; no native residency/spawn claim.')
save(L/'r486-map-binding.json',binding)
# One late necessary block is invalid; only a new owned Pak copy is altered.
idx=Path(b['archives'][0]['path']).read_bytes();entries={}
for rec in [idx[i:i+28]for i in range(4,len(idx),28)]:
 name=rec[4:24].split(b'\0')[0].decode('ascii').lower();entries[name]=(struct.unpack_from('<I',rec)[0],struct.unpack_from('<I',rec,24)[0])
required={}
for s in b['sectors']:
 raw=Path(s['pin']['path']).read_bytes();off=0
 def tile(pos):slot=raw[pos];id=struct.unpack_from('<H',raw,pos+1)[0];required.setdefault(id,set()).add(slot)
 for off in range(0,32768,4):tile(off)
 off=32768;count=struct.unpack_from('<H',raw,off)[0];off+=2
 for _ in range(count):tile(off+2);off+=6
 off+=16384;groups=struct.unpack_from('<i',raw,off)[0];off+=4
 for _ in range(groups):
  n=struct.unpack_from('<H',raw,off+2)[0];off+=4
  for _ in range(n):
   slot=raw[off+3];id=struct.unpack_from('<h',raw,off+4)[0];assert id>=0;required.setdefault(id,set()).add(slot);off+=7
assert sum(len(s)for s in required.values())==31444 and len(required)==176
id=max(required);slot=max(required[id]);offset,length=entries[f'{id}.til']
archive=Path(b['archives'][1]['path'])
with archive.open('rb')as f:f.seek(offset);member=f.read(length)
count=struct.unpack_from('<I',member)[0];body=4+4*(count+1);start=struct.unpack_from('<I',member,4+slot*4)[0];at=offset+body+start
negroot=O/'invalid-late-tile';negroot.mkdir();newpak=negroot/'LTtile.pak';shutil.copyfile(archive,newpak)
for role in [0,2,3]:shutil.copyfile(b['archives'][role]['path'],negroot/Path(b['archives'][role]['path']).name)
assert newpak.resolve().is_relative_to(O.resolve())
with newpak.open('r+b')as f:f.seek(at);original_type=f.read(1)[0];f.seek(at);f.write(bytes([255]))
bad=json.loads(json.dumps(binding));bad['state']='R486_INTENTIONAL_LATE_REQUIRED_UNSUPPORTED_TILE_NO_ASSET_READY'
bad['bindings'][39]['archive_root']=str(negroot)
for n in range(4):bad['bindings'][39]['archives'][n]|=pin(negroot/Path(b['archives'][n]['path']).name)
save(O/'invalid-late-tile.json',bad)
save(L/'r486-late-negative-source.json',dict(map_id=563,last_required_member=id,last_required_slot=slot,pak_byte_offset=at,original_type=original_type,invalid_type=255,original_archive_pin=b['archives'][1],owned_invalid_archive=pin(newpak),qualification_slots_before_last_member=sum(len(slots)for k,slots in required.items()if k<id),fixture=pin(O/'invalid-late-tile.json'),original_or_prior_sources_written=False))
entry=(P/'tools/prepare-r485-candidate.py').read_text(encoding='utf-8').replace('R485','R486').replace('r485','r486').replace('==39','==40')
entry=entry.replace("record['UI_assets']='R48639whole-map plans pending new2map consumer; existing37 plans unchanged;5066sector and5734sector whole previews using existing worker/window.563 full19sector format passes but required72446976decoded bytes exceeds unchanged64MiB budget. Native-only portal suffix; observed22/23 uses existing cloud RGBA adaptation. All budgets preserved; GUI pending.'","record['UI_assets']='R48640whole-map plans; existing39 unchanged; last563 complete19sectors uses opt-in per-city16MiB RGBA residence and bounded immutable encoded members. Every required slot validated initially, all visible contents rendered, overflow/cancel refuses without partial frame. Existing shared worker/window/menu reused; full mode/GUI acceptance pending.'")
(P/'tools/generate-r486-entry.py').write_text(entry,encoding='utf-8')
build=(P/'tools/build-r485-player-candidate.ps1').read_text(encoding='utf-8').replace('R485','R486').replace('r485','r486').replace('r486a-library-build','r486b-library-build')
(P/'tools/build-r486-player-candidate.ps1').write_text(build,encoding='utf-8')
manual=(P/'tools/start-r485-player-candidate.ps1').read_text(encoding='utf-8').replace('R485','R486').replace('r485','r486');(P/'tools/start-r486-player-candidate.ps1').write_text(manual,encoding='utf-8')
print('R486 forty plans pending last563 consumer; exact late required member/slot255 negative prepared:',id,slot,at)
