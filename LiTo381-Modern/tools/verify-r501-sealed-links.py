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
h=read(L/'r501-native-normal-login-loopback-handoff.json');names=[L/'r501-native-normal-login-loopback-handoff.json']+[Path(q['path'])for q in h['artifacts']]+[L/'r501-final-audit.json']
for p in names:walk(read(p))
assert h['source_difference']['bytes']==39037 and h['production_source_files']==615 and h['source_pairs']==207 and h['tests']['flows']==21 and h['tests']['preliminary18_PASS_reruns']==0 and h['tests']['total_exact_socket_frames']==13 and h['player_executions']==0 and h['GUI']==0 and not h['production_provider_or_authentication_connected']
save(L/'r501-sealed-link-audit.json',dict(state='R501_SEALED_JSON_LINKS_PASS_NO_CONSUMER_RERUN',json_files=len(names),pin_instances=instances,unique_paths=len(cache),handoff=pin(L/'r501-native-normal-login-loopback-handoff.json'),GUI=0,LOADED=False))
print('R501 sealedlinksPASS',len(names),'JSON',instances,'pins',len(cache),'unique',pin(L/'r501-native-normal-login-loopback-handoff.json')['sha256'])
