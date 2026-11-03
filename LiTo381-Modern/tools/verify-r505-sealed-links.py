from r491_common import *
seen={};n=0
def walk(v):
 global n
 if isinstance(v,dict):
  if all(k in v for k in ('path','bytes','sha256')):
   n+=1;p=v['path']
   if p not in seen:seen[p]=pin(p)
   assert seen[p]['bytes']==v['bytes'] and seen[p]['sha256']==v['sha256'],p
  else:
   for x in v.values():walk(x)
 elif isinstance(v,list):
  for x in v:walk(x)
h=L/'r505-native-transfer-prefix-address-gates-handoff.json';v=read(h);walk(v)
for q in v['records']:
 record=read(q['path'])
 if 'source_pins' in record:
  for s in record['source_pins']:
   for p in [s['path'],s['snapshot']]:walk(dict(path=p,bytes=s['bytes'],sha256=s['sha256']))
 else:walk(record)
continuation=read(L/'r505-continuation.json');walk(continuation);walk(read(L/'r505-final-audit.json'))
parent=read(P.parent/'監製進度.json');assert parent['r505']['handoff']['sha256']==pin(h)['sha256'];assert not parent['r505']['minimum_player_path_achieved'];assert not v['full_channel_roundtrip'] and not v['minimum_player_path_achieved']
save(L/'r505-sealed-link-verification.json',dict(state='R505_SEALED_LINKS_SOURCE_PAIRS_VALID_PREFIX_ONLY_CHANNEL_OPEN',handoff=pin(h),pin_instances=n,unique_paths=len(seen),source_files=619,source_pairs=221,consumer_executions=1,player_executions=0,full_channel_roundtrip=False,minimum_player_path_achieved=False,GUI=0,overall_complete=False))
print(f'R505 sealed links {n}pins/{len(seen)}unique;handoff{pin(h)["sha256"]};channelOPEN')
