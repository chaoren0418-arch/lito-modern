from r491_common import *
import shutil,difflib
old=read(L/'r502-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
p=P/'src/modern_lito/native_entry_navigation.rs';diffs.append(''.join(difflib.unified_diff([],p.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(p))))
diff=L/'r503-final-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');root=P/'target/r503-final-production-source';root.mkdir(exist_ok=False);pins=[]
for p in sorted((P/'src').rglob('*')):
 if not p.is_file():continue
 d=root/p.relative_to(P);d.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,d);q=pin(p);q['snapshot']=str(d);pins.append(q)
save(L/'r503-final-production-source-snapshot.json',dict(source_pins=pins,source_difference=pin(diff),changed=delta,added=['native_entry_navigation.rs'],unchanged=617-len(delta),GUI=0))
initial=read(L/'r503-production-source-snapshot.json');guard=[]
for q in initial['source_pins']:
 if pin(q['path'])['sha256']!=q['sha256']:guard.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
(L/'r503-final-guard-difference.patch').write_text('\n'.join(guard),encoding='utf-8')
old=(P/'tools/r503-navigation-consumer.rs').read_text(encoding='utf-8');helpers=old[:old.index('fn main(')].replace('r503-event-partial','r503-guard-event-partial')
(P/'tools/r503-guard-consumer.rs').write_text(helpers+(P/'tools/r503-guard-consumer-body.rs').read_text(encoding='utf-8'),encoding='utf-8')
print('R503 final source frozen:618/7changed1new; initial16 source/candidate preserved; final4 affected increments only')
