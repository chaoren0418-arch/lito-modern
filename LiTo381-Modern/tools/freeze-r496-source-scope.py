from r491_common import *
import shutil,difflib
old=read(L/'r495-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert delta==['native_city_runtime.rs','native_city_window.rs','native_source_connection.rs','native_source_send.rs'],delta
new=P/'src/modern_lito/native_source_actor.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r496-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8');files=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(files)==610
out=P/'target/r496-production-source';out.mkdir(exist_ok=False);pins=[]
for p in files:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r496-production-source-snapshot.json',dict(state='R496_ALL610_SOURCE_FROZEN_BEFORE_FIRST_NEW_CONSUMER',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r495-source-scope.json');scope.pop('changed_from_fixed_R494',None);scope.pop('added_from_fixed_R494',None)
scope.update(state='R496_SOURCE58_205_209_ADMISSION_SUBSET_AND_VISIBLE_CONTROL_CANDIDATE_SCOPE',source_files=610,changed_from_fixed_R495=delta,added_from_fixed_R495=['native_source_actor.rs'],authorization='R496 typedreceive/revision/refusal oractualvisibletypedselectedcontrol candidate;authorizedsameModern ASCII scratch',scope='4changed+1newproductionfiles;605 prior production files unchanged;no normalNativeSocket/auth/provider/JavaSQL/server/BIN/GUI execution.')
scope['new_source_files'].append(dict(path='src/modern_lito/native_source_actor.rs',sha256=pin(new)['sha256']))
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r496-source-scope.json',scope);save(L/'source-change-scope.json',scope)
print('R496 frozen610production/4changed+1new;605previous unchanged')
