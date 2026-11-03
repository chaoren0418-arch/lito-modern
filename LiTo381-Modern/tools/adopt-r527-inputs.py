from r491_common import *
import shutil
B=P.parent.parent/'lito脫殼資源';S=P.parent.parent/'lito伺服器';root=L/'r527-inputs';root.mkdir();verified={};refs=0;context=None
prior={e['path']:e for e in read(L/'r525-production-source-snapshot.json')['source_pins']};prior.update({str(P/'target/r525-player-entry-candidate/LiToModern-R525.rs'):pin(P/'target/r525-player-entry-candidate/LiToModern-R525.rs')})
def walk(o):
 global refs
 if isinstance(o,dict):
  if isinstance(o.get('path'),str)and isinstance(o.get('sha256'),str)and len(o['sha256'])==64:
   p=Path(o['path']);p=p if p.is_absolute()else context/p;actual=verified.get(str(p))
   if actual is None:actual=pin(p)
   if actual['sha256']!=o['sha256'].upper()and str(p)in prior:
    old=prior[str(p)];actual=pin(old.get('snapshot',old['path']));actual['historical_live_path']=str(p)
   assert actual['sha256']==o['sha256'].upper(),str(p)
   if isinstance(o.get('bytes'),int):assert actual['bytes']==o['bytes']
   verified[str(p)]=actual;refs+=1
  for v in o.values():walk(v)
 elif isinstance(o,list):
  for v in o:walk(v)
roots=[]
for p,h in [(S/'server-ready-explicit-cancel-485F-network.json','3D1640BC79384E211BD8049DEF422C80719CAD3625292A571312E29B92FC4AE0'),(B/'Modern-character-ui-reachability-contract.json','FC83ED02485F28661AC5550A8C6D8DB0F4ABA3EE06FB6B68EB26ED38CE4BDC9B')]:
 e=pin(p);assert e['sha256']==h;v=read(p);context=Path(v['manifest']['path']).parent if 'manifest'in v else p.parent;walk(v);dst=root/p.name;shutil.copyfile(p,dst);roots.append(dict(original=e,copy=pin(dst)))
cancel=read(root/'server-ready-explicit-cancel-485F-network.json');context=Path(cancel['manifest']['path']).parent;walk(read(cancel['manifest']['path']))
for e in cancel['cases']+[cancel['Modern_delta']]:shutil.copyfile(e['path'],root/Path(e['path']).name);walk(read(e['path']))
bin34=read(root/'Modern-character-ui-reachability-contract.json');anchors=[]
for f in bin34['findings']:
 for a in f.get('anchors',[]):
  source=Path(a['source']['path']);actual=pin(source);old=prior.get(str(source))
  if actual['sha256']!=a['source']['sha256']:assert old;source=Path(old.get('snapshot',old['path']))
  assert pin(source)['sha256']==a['source']['sha256'];lines=source.read_text('utf-8').splitlines();assert a['required_start_token']in'\n'.join(lines[a['start_line']-1:a['end_line']]);anchors.append(dict(id=f['id'],source=pin(source),historical=True,start=a['start_line'],end=a['end_line'],token=a['required_start_token']))
assert len(anchors)==47
save(L/'r527-input-adoption.json',dict(roots=roots,verified_pins=list(verified.values()),reference_occurrences=refs,BIN34_historical_anchor_checks=anchors,BIN34_limit='R5250..8 dropdownChooseSubmitC9 route already reachable source-only; runtime/send historicalsnapshots, notcurrentR526compatibility;noGUI/default main promotion orprotocolcount change',new485F_cancel_limit=cancel['unknowns'],new485F_cancel_upgrade='Two separate ordinaryCOUNTDOWN cases explicitd200/d202→ownD1state0/2+E100; Hostreadytrue retained Guestfalse;ROOM/WAITING;~4s watch no positive tick. Only these two upgrade fromsource-only,3F31static history immutable;notatomic/drain/freshness',new_server_processes=0,oldPASS=0))
print('R527 inputs verified',len(verified),'unique pins',refs,'references,47 historicalanchors')
