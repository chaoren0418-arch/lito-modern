from r491_common import *
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;q=v;p=q['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==q['bytes']and cache[p]['sha256']==q['sha256'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
h=read(L/'r499-native-back-retained-epoch-reselection-handoff.json');names=[L/'r499-native-back-retained-epoch-reselection-handoff.json']+[Path(q['path'])for q in h['artifacts']]+[L/'r499-final-audit.json']
history=read(L/'r499-execution-history.json');names += [Path(q['path'])for q in history['verifier_failure_records']]
for p in names:walk(read(p))
assert h['source_difference']['bytes']==43127 and h['tests']['flows']==10 and h['production_source_files']==612 and h['source_pairs']==200 and h['player_executions']==0 and h['GUI']==0
save(L/'r499-sealed-link-audit.json',dict(state='R499_SEALED_LINKS_PIN_PASS_NO_CONSUMER_RERUN',json_files=len(names),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r499-native-back-retained-epoch-reselection-handoff.json'),GUI=0,LOADED=False))
print('R499 sealedlinksPASS',len(names),'JSON',instances,'pins',len(cache),'unique',pin(L/'r499-native-back-retained-epoch-reselection-handoff.json')['sha256'])
