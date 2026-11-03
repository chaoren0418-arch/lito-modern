from r491_common import *
import shutil,difflib
old=read(L/'r493-production-source-snapshot.json');deltas=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  deltas.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert deltas==['native_city_runtime.rs'],deltas
new=P/'src/modern_lito/native_source_stream.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r494-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8')
allfiles=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(allfiles)==608
out=P/'target/r494-production-source';out.mkdir(exist_ok=False);pins=[]
for p in allfiles:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r494-production-source-snapshot.json',dict(state='R494_ALL608_PRODUCTION_SOURCE_FROZEN_BEFORE_NEW_CONSUMER_EXECUTION',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r493-source-scope.json');scope.update(state='R494_RUNTIME_OWNED_SOURCE_SEEDED_NATIVE_STREAM_SCOPE',source_files=608,changed_from_fixed_R493=deltas,added_from_fixed_R493=['native_source_stream.rs'],authorization='T370 finite staticcounter/flag/framing andnecessaryexistingRuntimecandidate',scope='Runtime plusnewstreamreceiver;607otherproductionfiles unchanged. Sameepochdecoder/roster/mode sourcedbodyentrypoints;no normalNativeSocket/profile/auth/480560effects/LTM1alias/GUI/phase6.')
scope['new_source_files'].append(dict(path='src/modern_lito/native_source_stream.rs',sha256=pin(new)['sha256']));scope.pop('changed_from_fixed_R492',None);scope.pop('added_from_fixed_R492',None)
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r494-source-scope.json',scope);save(L/'source-change-scope.json',scope)
print('R494 scope frozen608/1changed+1new;no historicalPASS reruns')
