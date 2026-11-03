"""Read-only exact38 inventory and four complete same-format Native single sectors."""
from pathlib import Path
import json,hashlib,shutil,struct
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r479-map-batch';O.mkdir(exist_ok=False);(O/'native-map').mkdir();(O/'probe').mkdir()
def pin(p):
 p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8')
m=json.loads((L/'r474-mode-map-binding.json').read_text(encoding='utf-8'));assert pin(L/'r474-mode-map-binding.json')['sha256']=='EB3FF940E1944812D0AA30A6016E9EAEAAC37ADF38210703B43BC96E5B44AFD2'
selected=[509,508,505,516];rows=[];batch=[]
for row in m['rows']:
 if (row['mode'],row['logical_map']) in [(1,564),(1,514)]:continue
 directory=Path(row['native_directory']);current=sorted(directory.glob('*.s32'));expected={Path(v['path']).name:v for v in row['native_sector_files']};actual={f.name:pin(f) for f in current}
 assert set(expected)==set(actual) and all(actual[n]==expected[n] for n in actual),'Original sector list or byte drift'
 records=[]
 for name,v in actual.items():
  assert len(name)==12 and name.endswith('.s32');records.append(dict(map_id=row['logical_map'],x=int(name[:4],16),y=int(name[4:8],16),pin=v))
 rec=dict(mode=row['mode'],map_id=row['logical_map'],name=row['original_table']['name'],directory=str(directory),catalog_record=row['original_table'],sectors=records,extra_non_s32_files=[f.name for f in sorted(directory.iterdir()) if f.is_file() and f.suffix.lower()!='.s32'],exact_R474_sector_names_and_bytes=True,selected_for_new_static_preview=row['mode']==1 and row['logical_map'] in selected)
 rows.append(rec)
 if rec['selected_for_new_static_preview']:
  assert len(records)==1
  owned=O/'native-map'/str(row['logical_map']);owned.mkdir();src=Path(records[0]['pin']['path']);dst=owned/src.name;shutil.copyfile(src,dst);assert pin(src)==records[0]['pin'] and pin(dst)['sha256']==records[0]['pin']['sha256']
  batch.append(dict(mode=1,map_id=row['logical_map'],name=rec['name'],map_root=str(O/'native-map'),archive_root=str(Path(m['selected_archives'][0]['path']).parent),sectors=[dict(map_id=row['logical_map'],x=records[0]['x'],y=records[0]['y'],pin=pin(dst))],original_sectors=records,archives=m['selected_archives']))
assert len(rows)==38 and [b['map_id'] for b in batch]==[509,508,505,516]
archive_members={}
for i,stem in [(0,'LTtile'),(2,'LTsprite')]:
 for a in m['selected_archives'][i:i+2]:assert pin(a['path'])['sha256']==a['sha256']
 idx=Path(m['selected_archives'][i]['path']).read_bytes();pak_size=m['selected_archives'][i+1]['bytes'];count=struct.unpack_from('<I',idx)[0];assert len(idx)==4+count*28;entries=[]
 for off,name,size in struct.iter_unpack('<I20sI',idx[4:]):
  name=name.split(b'\0')[0].decode('ascii').lower();assert off+size<=pak_size;entries.append(dict(name=name,offset=off,bytes=size))
 assert len({e['name'] for e in entries})==count;archive_members[stem]=entries
inventory=dict(state='R479_EXACT_REMAINING38_SECTOR_AND_NATIVE_ARCHIVE_INDEX_INVENTORY',catalog_source=pin(L/'r474-mode-map-binding.json'),rows=rows,archives=m['selected_archives'],archive_members=archive_members,batch=batch,selection_reason='Four remaining mode1 maps each have one complete actual sector in exact R474/native directory list and use existing S32/Native archive format. No partial multi-sector recipe chosen; other34 retain full actual sector lists for later multi-sector work.',static_scope='All source floor/overlay/object tile slots qualified for selected batch only; opacity/effects/Layer6/actors/mode admission remain distinct. Archive index availability alone does not claim decode/playback completeness.',GUI=0,LOADED=False)
save(L/'r479-resource-inventory-input.json',inventory)
print(json.dumps(dict(remaining_rows=38,total_sectors=sum(len(r['sectors']) for r in rows),selected_new_maps=[b['map_id'] for b in batch])))
