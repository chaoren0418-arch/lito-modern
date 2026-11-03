from r491_common import *
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if all(k in v for k in ['path','bytes','sha256']):
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['bytes']==v['bytes'] and cache[p]['sha256']==v['sha256'],p
   if 'snapshot'in v:assert pin(v['snapshot'])['sha256']==v['sha256']
  for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
paths=sorted(L.glob('r496*.json'))
for p in paths:walk(read(p))
h=read(L/'r496-native-version-actor-admission-selection-controls-handoff.json');assert not h['normal_wire_connected'] and not h['actor205_209_connected'] and not h['Native_actor_identifier_installed'] and not h['authentication_success_claimed'] and not h['phase6_callback_connected'] and not h['overall_finished'] and h['GUI']==0 and h['player_executions']==0
assert h['tests']['flow_cases']==14 and h['tests']['malformed_cases']==1025 and h['execution_history']['consumer_executions']==1
save(L/'r496-sealed-links-check.json',dict(state='R496_CURRENT_SEALED_LINKS_PASS',JSON_records=len(paths),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r496-native-version-actor-admission-selection-controls-handoff.json'),GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R496 sealedlinks PASS',len(paths),'JSON',instances,'pininstances',len(cache),'unique')
