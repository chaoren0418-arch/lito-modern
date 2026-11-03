from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r482-cross-mode-batch'
def pin(p):p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,v):tmp=p.with_suffix('.tmp');tmp.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(p)
ids=[534,535,565,569,519,520,553,547,540,510,528];O.mkdir(exist_ok=False);(O/'native-map').mkdir();(O/'reference').mkdir()
m=read(L/'r479-resource-inventory-input.json');batch=[]
for id in ids:
 row=next(r for r in m['rows'] if r['map_id']==id);assert sorted(p.name for p in Path(row['directory']).glob('*.s32'))==[Path(s['pin']['path']).name for s in row['sectors']];owned=O/'native-map'/str(id);owned.mkdir();sectors=[]
 for s in row['sectors']:
  assert pin(s['pin']['path'])==s['pin'];dest=owned/Path(s['pin']['path']).name;shutil.copyfile(s['pin']['path'],dest);a=pin(dest);assert a['bytes']==s['pin']['bytes'] and a['sha256']==s['pin']['sha256'];sectors.append({**s,'pin':a})
 anchor=(sectors[0]['x'],sectors[0]['y']);pos=[(1536*(s['x']-anchor[0]+s['y']-anchor[1]),768*(-s['x']+anchor[0]+s['y']-anchor[1])) for s in sectors];bounds=[min(x for x,y in pos),min(y for x,y in pos),max(x+3072 for x,y in pos),max(y+1536 for x,y in pos)]
 batch.append(dict(mode=row['mode'],map_id=id,name=row['name'],shape='source-exact-'+str(len(sectors))+'sector',measurement_only=id in [534,535],namespace='NATIVE_LITO_LTMAP_V1',map_root=str(O/'native-map'),archive_root=str(Path(m['archives'][0]['path']).parent),sectors=sectors,archives=m['archives'],original_sectors=row['sectors'],original_directory=row['directory'],catalog_record=row['catalog_record'],expected_bounds=bounds,anchor=anchor,source_bytes=sum(s['pin']['bytes'] for s in sectors)))
for a in m['archives']:
 b=pin(a['path']);assert all(b[k]==a[k] for k in ['path','bytes','sha256'])
assert sum(len(b['sectors']) for b in batch)==119
pair=[next(b for b in batch if b['map_id']==id) for id in [519,520]];assert [(s['x'],s['y'],s['pin']['bytes'],s['pin']['sha256']) for s in pair[0]['sectors']]==[(s['x'],s['y'],s['pin']['bytes'],s['pin']['sha256']) for s in pair[1]['sectors']]
catalog=read(L/'r481-map-binding.json')['catalog40'];assert len({r['map_id'] for r in catalog})==40
save(L/'r482-resource-input.json',dict(state='R482_CROSS_MODE_NINE_MAPS_AND_TWO_WHOLE_CITY_MEASUREMENTS',batch=batch,previous_inventory=pin(L/'r479-resource-inventory-input.json'),previous_R481_handoff=pin(L/'r481-six-complete-multisector-batch-handoff.json'),source_identical_map_pair=[519,520],catalog_map_ids_unique_across_modes=True,wrong_mode_same_map_id_must_refuse=True,known_format_gaps=[dict(map_id=506,types=[22,23],members=['198.til','199.til']),dict(map_id=573,format='Layer7'),dict(map_id=563,format='Layer7')],GUI=0,LOADED=False))
print('R482 nine cross-mode complete-map inputs plus16/25sector whole-city measurements;119sector copies exact')
