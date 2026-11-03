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
paths=sorted(L.glob('r497*.json'))
for p in paths:walk(read(p))
h=read(L/'r497-native-source-map-entry-resource-refresh-handoff.json');assert h['presentation_revision_gate'] and h['visible_source_menu_invoker_definition'] and not h['original_character_UI_complete'] and not h['Native_GUI_executed'] and not h['normal_wire_connected'] and not h['overall_finished'] and h['GUI']==0 and h['player_executions']==0
assert h['tests']['flow_cases']==10 and h['execution_history']['consumer_executions']==2 and h['execution_history']['consumer_test_failures']==1 and not h['execution_history']['firstfourassertion_batches_rerun']
save(L/'r497-sealed-links-check.json',dict(state='R497_CURRENT_SEALED_LINKS_PASS',JSON_records=len(paths),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r497-native-source-map-entry-resource-refresh-handoff.json'),GUI=0,LOADED=False,complete_modes=16,total_modes=40))
print('R497 sealedlinks PASS',len(paths),'JSON',instances,'pininstances',len(cache),'unique')
