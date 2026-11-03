from r491_common import *
cache={};checks=0
def check(path,sha,size=None):
 global checks
 p=Path(path);p=p if p.is_absolute()else P/p;k=str(p.resolve()).casefold()
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
files=sorted(p for p in L.glob('r492*.json')if p.name!='r492-sealed-links-check.json')
for p in files:walk(read(p))
h=read(L/'r492-native-authority-bound-ingress-handoff.json');assert h['GUI']==0 and h['complete_modes']==16 and h['player_candidate_executions']==0 and h['phase6_callback_connected']is False and h['normal_wire_connected']is False and h['overall_finished']is False
save(L/'r492-sealed-links-check.json',dict(state='R492_ALL_CURRENT_RECORD_PINS_AND_FROZEN_SNAPSHOT_LINKS_PASS',inspected_json_files=len(files),checked_pin_instances=checks,unique_files=len(cache),records=[pin(p)for p in files],handoff=pin(L/'r492-native-authority-bound-ingress-handoff.json'),GUI=0,LOADED=False,complete_modes=16));print('R492 sealedlinksPASS',len(files),'JSON',checks,'pininstances',len(cache),'unique')
