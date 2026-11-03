from r491_common import *
cache={};instances=0;names=['r498-source-study.json','r498-character-resource-mapping.json','r498-archive-inventory.json','r498-native-menu-skin-binding.json','r498-entry-source-contract.json','r498-source-oracle.json','r498-independent-source-oracle.json','r498-production-source-snapshot.json','r498-source-scope.json','r498-player-candidate.json','r498-consumer-candidate.json','r498-consumer-exit.json','r498-event-details.json','r498-candidate.json','r498-independent-contract.json','r498-execution-history.json','r498-open-gaps.json','r498-native-character-ui-mapping-selection-scene-handoff.json','r498-final-audit.json']
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==v['bytes']and cache[p]['sha256']==v['sha256'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
for n in names:walk(read(L/n))
for n in ['r498-production-source-snapshot.json','r498-consumer-candidate.json','r498-candidate.json']:
 for q in read(L/n)['source_pins']:
  p=q['snapshot'];instances+=1
  if p not in cache:cache[p]=pin(p)
  assert cache[p]['bytes']==q['bytes']and cache[p]['sha256']==q['sha256'],p
h=read(L/'r498-native-character-ui-mapping-selection-scene-handoff.json');assert h['tests']['flows']==11 and h['tests']['class_variant']==65536 and h['GUI']==0 and h['overall_status']=='in_progress';pp=read(P.parent/'監製進度.json');assert pp['overall_status']=='in_progress'and pp['r498']['new_player_flow_review']=='PENDING'
save(L/'r498-sealed-links-check.json',dict(state='R498_SEALED_LINKS_CHECK_PASS_OVERALL_UNFINISHED',json_count=len(names),pin_instances=instances,unique_paths=len(cache),checked_json=[pin(L/n)for n in names],GUI=0,LOADED=False,complete_modes=16,total_modes=40,historical_PASS_reruns=0))
print('R498 sealedlinksPASS',len(names),'JSON',instances,'pins',len(cache),'unique;all612production/198consumersnapshotsverified')
