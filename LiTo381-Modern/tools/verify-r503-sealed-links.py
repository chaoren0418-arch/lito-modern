from r491_common import *
count=0;cache={}
def walk(v):
 global count
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   count+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert (cache[p]['sha256'],cache[p]['bytes'])==(v['sha256'],v['bytes']),p
   if v.get('snapshot'):
    p=v['snapshot']
    if p not in cache:cache[p]=pin(p)
    assert (cache[p]['sha256'],cache[p]['bytes'])==(v['sha256'],v['bytes']),p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
for n in ['r503-native-navigation-subset-handoff.json','r503-candidate.json','r503-final-audit.json']:walk(read(L/n))
pp=read(P.parent/'監製進度.json');h=pin(L/'r503-native-navigation-subset-handoff.json');assert pp['r503']['handoff']==h and pp['r503']['full_channel_roundtrip']==False and pp['r503']['GUI']==0
assert pp['r502']['new_player_flow_review']=='FINITE_ACCEPTED_21JSON1157pins745unique'
save(L/'r503-sealed-link-verification.json',dict(state='R503_NEW_SEALED_LINKS_VALID_AFTER_HANDOFF',handoff=h,pin_instances=count,unique_paths=len(cache),prior_full_segment_audit=pin(L/'r503-independent-contract.json'),verifier=pin(P/'tools/verify-r503-sealed-links.py'),all_checked=True,GUI=0,full_channel_roundtrip=False))
print('R503 sealedlinksPASS',count,'pins/',len(cache),'unique;handoff',h['sha256'])
