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
h=read(L/'r500-native-login-form-input-lifecycle-handoff.json');history=read(L/'r500-execution-history.json');names=[L/'r500-native-login-form-input-lifecycle-handoff.json']+[Path(q['path'])for q in h['artifacts']]+[L/'r500-final-audit.json']+[Path(q['path'])for q in history['preparation_records']]+[Path(history['library_failure']['path']),Path(history['verifier_failure']['path'])]
for p in names:walk(read(p))
assert h['source_difference']['bytes']==55128 and h['tests']['flows']==9 and h['production_source_files']==613 and h['source_pairs']==203 and h['player_executions']==0 and h['GUI']==0 and not h['login_authentication_connected']
save(L/'r500-sealed-link-audit.json',dict(state='R500_SEALED_LINK_PIN_PASS_NO_CONSUMER_RERUN',json_files=len(names),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r500-native-login-form-input-lifecycle-handoff.json'),GUI=0,LOADED=False))
print('R500 sealedlinksPASS',len(names),'JSON',instances,'pins',len(cache),'unique',pin(L/'r500-native-login-form-input-lifecycle-handoff.json')['sha256'])
