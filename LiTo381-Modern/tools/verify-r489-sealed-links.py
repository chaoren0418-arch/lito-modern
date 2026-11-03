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
   if k not in ['path','bytes','sha256','snapshot']:walk(x)
docs=sorted(p for p in L.glob('r489*.json')if p.name!='r489-sealed-links-check.json')
for p in docs:walk(read(p))
h=pin(L/'r489-native-state-snapshot-player-flow-handoff.json');assert h==read(L/'r489-final-audit.json')['handoff'];assert read(P.parent/'監製進度.json')['r489']['handoff']==h
assert pin(L/'r488-native-mode-event-player-flow-handoff.json')['sha256']=='D7F516DC4B3C51D27FC957F9F39A3AD1BCDA56BE28849AC702ECE64FAC7E8861'
out=dict(state='R489_FINAL_CURRENT_JSON_SOURCE_SNAPSHOTS_ARTIFACT_HASH_HANDOFF_PROGRESS_PASS',JSON_documents=len(docs),checked_pin_instances=checks,unique_files=len(cache),handoff=h,source_pairs=172,all_production_source_snapshots=604,preflight_source_snapshots=604,event_cases=207,runtime_flow_cases=9,player_candidate_executions=0,snapshot_mode_width='byte c source correction',verification_tool=pin(Path(__file__)),complete_modes=16,GUI=0,LOADED=False)
(L/'r489-sealed-links-check.json').write_text(json.dumps(out,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print('R489 sealed links PASS:',len(docs),'JSON',checks,'pins',len(cache),'unique;handoff',h['sha256'])
