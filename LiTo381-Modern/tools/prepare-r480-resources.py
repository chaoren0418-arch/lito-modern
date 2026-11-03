from pathlib import Path
import json,hashlib,shutil
P=Path(__file__).resolve().parent.parent; L=P/'logs/continuation-20261003'; O=P/'target/r480-multisector'
def pin(p):
 p=Path(p); b=p.read_bytes(); return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def read(p):return json.loads(p.read_text(encoding='utf-8'))
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2),encoding='utf-8')
O.mkdir(exist_ok=True); (O/'reference').mkdir(exist_ok=True); (O/'native-map/552').mkdir(parents=True,exist_ok=True)
old=read(L/'r479-resource-inventory-input.json'); row=next(r for r in old['rows'] if r['map_id']==552)
assert row['mode']==1 and [(s['x'],s['y']) for s in row['sectors']]==[(32767,32767),(32767,32768),(32768,32767),(32768,32768)]
original=Path(row['directory']); assert sorted(p.name for p in original.glob('*.s32'))==sorted(Path(s['pin']['path']).name for s in row['sectors'])
sectors=[]
for s in row['sectors']:
 assert pin(s['pin']['path'])==s['pin']; dest=O/'native-map/552'/Path(s['pin']['path']).name
 if dest.exists():assert pin(dest)['sha256']==s['pin']['sha256']
 else:shutil.copyfile(s['pin']['path'],dest)
 sectors.append({**s,'pin':pin(dest)})
for a in old['archives']:
 actual=pin(a['path']); assert all(actual[k]==a[k] for k in ['path','bytes','sha256'])
batch=dict(mode=1,map_id=552,name='Sky Gate',namespace='NATIVE_LITO_LTMAP_V1',map_root=str(O/'native-map'),archive_root=str(Path(old['archives'][0]['path']).parent),sectors=sectors,archives=old['archives'])
save(L/'r480-resource-input.json',dict(state='R480_COMPLETE_ORIGINAL_552_FOUR_SECTOR_INPUT',batch=batch,original_sectors=row['sectors'],original_directory=row['directory'],catalog_record=row['catalog_record'],previous_inventory=pin(L/'r479-resource-inventory-input.json'),selection='Complete 2x2 map covers X/Y seams and central four-sector junction without unsupported Layer7 variants. All original directory sectors included.',GUI=0,LOADED=False))
print('R480 complete map552, four exact sectors copied; four archive pins verified')
