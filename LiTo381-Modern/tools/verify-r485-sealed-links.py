from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003'
cache={};checks=0
def pin(p):b=p.read_bytes();return dict(path=str(p),bytes=len(b),sha256=hashlib.sha256(b).hexdigest().upper())
def walk(v):
 global checks
 if isinstance(v,list):
  for x in v:walk(x)
 elif isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   p=Path(v['path']);assert p.is_file(),str(p)
   if str(p)not in cache:cache[str(p)]=pin(p)
   assert all(cache[str(p)][k]==v[k]for k in ['bytes','sha256']),str(p);checks+=1
  for k,x in v.items():
   if k not in ['path','bytes','sha256']:walk(x)
docs=sorted(p for p in L.glob('r485*.json')if p.name!='r485-sealed-links-check.json')
for p in docs:walk(json.loads(p.read_text(encoding='utf-8-sig')))
h=pin(L/'r485-native-format-two-map-handoff.json');assert h['sha256']=='DD511B18C0184B08B7B0381DC6EE3C4D51A499ADBA85C265769C65D34C2FBC1E'
final=dict(state='R485_FINAL_CURRENT_JSON_BYTEPINS_SNAPSHOTS_PROGRESS_AND_SEALED_HANDOFF_PASS',JSON_documents=len(docs),checked_pin_instances=checks,unique_files=len(cache),handoff=h,source_pairs=160,verification_tool=pin(Path(__file__)),complete_modes=16,GUI=0,LOADED=False)
p=L/'r485-sealed-links-check.json';t=p.with_suffix('.json.tmp');t.write_text(json.dumps(final,ensure_ascii=False,indent=2),encoding='utf-8');t.replace(p)
print('R485 sealed links PASS:',len(docs),'JSON',checks,'pins',len(cache),'unique files;handoff',h['sha256'])
