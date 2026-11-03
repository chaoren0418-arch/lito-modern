from r491_common import *
count=0;cache={}
def walk(v):
 global count
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   count+=1
   for p in [v['path']]+([v['snapshot']]if v.get('snapshot')else[]):
    if p not in cache:cache[p]=pin(p)
    assert(cache[p]['bytes'],cache[p]['sha256'])==(v['bytes'],v['sha256']),p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
for n in ['r504-native-refusal-fresh-epoch-retry-handoff.json','r504-candidate.json','r504-final-audit.json']:walk(read(L/n))
hp=pin(L/'r504-native-refusal-fresh-epoch-retry-handoff.json');v=read(P.parent/'監製進度.json');assert v['r504']['handoff']==hp and v['r504']['flows']==9 and v['r504']['GUI']==0 and v['r504']['Native_alert_disconnect']==False
save(L/'r504-sealed-link-verification.json',dict(state='R504_NEW_SEALED_LINKS_PASS',handoff=hp,pin_instances=count,unique_paths=len(cache),prior_independent_contract=pin(L/'r504-independent-contract.json'),verifier=pin(P/'tools/verify-r504-sealed-links.py'),GUI=0,complete_modes=16,total_modes=40))
print('R504 sealedlinksPASS',count,'pins/',len(cache),'unique;',hp['sha256'])
