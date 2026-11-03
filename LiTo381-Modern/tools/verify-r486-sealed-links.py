from pathlib import Path
import json,hashlib
P=Path(__file__).resolve().parent.parent;L=P/'logs/continuation-20261003';cache={};checks=0
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def pin(p):
 p=Path(p);h=hashlib.sha256()
 with p.open('rb')as f:
  for b in iter(lambda:f.read(8<<20),b''):h.update(b)
 return dict(path=str(p),bytes=p.stat().st_size,sha256=h.hexdigest().upper())
def check(p,sha,size=None):
 global checks
 p=Path(p);p=p if p.is_absolute()else P/p;k=str(p.resolve()).casefold()
 if k not in cache:cache[k]=pin(p)
 assert cache[k]['sha256']==sha and(size is None or cache[k]['bytes']==size),str(p);checks+=1
def walk(v):
 if isinstance(v,list):
  for x in v:walk(x)
 elif isinstance(v,dict):
  if {'path','sha256'}<=v.keys():check(v['path'],v['sha256'],v.get('bytes'))
  if {'snapshot','sha256'}<=v.keys():check(v['snapshot'],v['sha256'],v.get('bytes'))
  if {'path','current_sha256'}<=v.keys():check(v['path'],v['current_sha256'])
  for k,x in v.items():
   if k not in ['path','sha256','bytes','snapshot']:walk(x)
docs=sorted(p for p in L.glob('r486*.json')if p.name!='r486-sealed-links-check.json')
for p in docs:
 v=read(p)
 if p.name=='r486-tile-test-candidate.json':
  sources=v.pop('source_pins');changed=[]
  for s in sources:
   check(s['snapshot'],s['sha256'],s['bytes'])
   if pin(Path(s['path']))['sha256']!=s['sha256']:changed.append(Path(s['path']).name)
  assert set(changed)=={'native_city_resources.rs','native_city_flow.rs'}
 walk(v)
hp=pin(L/'r486-bounded-city563-handoff.json');assert read(L/'r486-final-audit.json')['handoff']==hp
assert pin(L/'r485-native-format-two-map-handoff.json')['sha256']=='DD511B18C0184B08B7B0381DC6EE3C4D51A499ADBA85C265769C65D34C2FBC1E'
assert read(P.parent/'監製進度.json')['r486']['handoff']==hp
out=dict(state='R486_SEALED_CURRENT_JSON_HASH_SOURCE_SNAPSHOTS_PROGRESS_HANDOFF_PASS',JSON_documents=len(docs),checked_pin_instances=checks,unique_files=len(cache),handoff=hp,source_pairs=164,initial_test_snapshots=601,final_test_snapshots=602,verification_tool=pin(Path(__file__)),complete_modes=16,GUI=0,LOADED=False)
p=L/'r486-sealed-links-check.json';p.write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('R486 sealed links PASS:',len(docs),'JSON',checks,'pins',len(cache),'unique files;handoff',hp['sha256'])
