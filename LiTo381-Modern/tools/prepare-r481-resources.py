from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';O=P/'target/r481-multisector-batch'
def pin(p):p=Path(p);b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,v):tmp=p.with_suffix('.tmp');tmp.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8');tmp.replace(p)
ids=[551,504,506,526,525,531,529];shapes=['L3','irregular7','stepped6','diagonal4','staircase11','diamond5','rectangle4']
O.mkdir(exist_ok=False);(O/'reference').mkdir();(O/'native-map').mkdir()
m=read(L/'r479-resource-inventory-input.json');inventory=read(L/'r479-native38-inventory.json');batch=[]
for id,shape in zip(ids,shapes):
 row=next(r for r in m['rows'] if r['map_id']==id);parsed=next(r for r in inventory['rows'] if r['map_id']==id);assert row['mode']==1 and parsed['parse_or_static_reference_error'] is None
 assert sorted(p.name for p in Path(row['directory']).glob('*.s32'))==[Path(s['pin']['path']).name for s in row['sectors']]
 owned=O/'native-map'/str(id);owned.mkdir();sectors=[]
 for s in row['sectors']:
  assert pin(s['pin']['path'])==s['pin'];dest=owned/Path(s['pin']['path']).name;shutil.copyfile(s['pin']['path'],dest);a=pin(dest);assert a['bytes']==s['pin']['bytes'] and a['sha256']==s['pin']['sha256'];sectors.append({**s,'pin':a})
 anchor=(sectors[0]['x'],sectors[0]['y']);positions=[(1536*(s['x']-anchor[0]+s['y']-anchor[1]),768*(-s['x']+anchor[0]+s['y']-anchor[1])) for s in sectors];bounds=[min(x for x,y in positions),min(y for x,y in positions),max(x+3072 for x,y in positions),max(y+1536 for x,y in positions)]
 batch.append(dict(mode=1,map_id=id,name=row['name'],shape=shape,namespace='NATIVE_LITO_LTMAP_V1',map_root=str(O/'native-map'),archive_root=str(Path(m['archives'][0]['path']).parent),sectors=sectors,archives=m['archives'],original_sectors=row['sectors'],original_directory=row['directory'],catalog_record=row['catalog_record'],expected_bounds=bounds,anchor=anchor,source_bytes=sum(s['pin']['bytes'] for s in sectors)))
for a in m['archives']:
 b=pin(a['path']);assert all(b[k]==a[k] for k in ['path','bytes','sha256'])
assert sum(len(b['sectors']) for b in batch)==40
save(L/'r481-resource-input.json',dict(state='R481_SEVEN_COMPLETE_MODE1_MAPS_FORTY_NATIVE_SECTORS',batch=batch,previous_inventory=pin(L/'r479-resource-inventory-input.json'),previous_R480_handoff=pin(L/'r480-complete-multisector552-handoff.json'),selection='Seven remaining same-mode1 maps cover L, irregular, stepped, diagonal, staircase, diamond, rectangle shapes; negative projected Y and more tile/source object dependencies than552. Entire original directories included.',deferred_mode1_map=dict(map_id=534,name='Lodun City (Death)',sectors=16,source_bytes=2265823,reason='Large 16-sector city held for subsequent measured budget/shape batch, not qualified here'),known_Layer7_gaps=[573,563],GUI=0,LOADED=False))
print('R481 seven complete new maps /40sector exact copies and archive pins ready')
