from r491_common import *
cache={};instances=0
def walk(v):
 global instances
 if isinstance(v,dict):
  if {'path','bytes','sha256'}<=v.keys():
   instances+=1;p=v['path']
   if p not in cache:cache[p]=pin(p)
   assert cache[p]['sha256']==v['sha256']and cache[p]['bytes']==v['bytes'],p
  for q in v.values():walk(q)
 elif isinstance(v,list):
  for q in v:walk(q)
h=read(L/'r502-player-login-event-loop-handoff.json');names=[L/'r502-player-login-event-loop-handoff.json']+[Path(q['path'])for q in h['artifacts']]+[L/'r502-final-audit.json']
for p in names:walk(read(p))
assert h['source_difference']['bytes']==71952 and h['production_source_files']==617 and h['source_pairs']==211 and h['tests']['flows']==14 and h['tests']['late_event_rejections']==5 and h['tests']['exact_source_socket_frames']==16 and h['player_executions']==0 and h['GUI']==0 and not h['production_provider_or_authentication_connected']and not h['worker_second_protocol_controller']
save(L/'r502-sealed-link-audit.json',dict(state='R502_SEALED_JSON_LINKS_PASS_NO_CONSUMER_RERUN',json_files=len(names),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r502-player-login-event-loop-handoff.json'),GUI=0,LOADED=False))
print('R502 sealedlinksPASS',len(names),'JSON',instances,'pins',len(cache),'unique',pin(L/'r502-player-login-event-loop-handoff.json')['sha256'])
