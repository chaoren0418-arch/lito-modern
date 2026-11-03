from r491_common import *
import shutil,difflib
base=read(P/'logs/relocation-baseline-20261003/source-manifest.json');previous=read(L/'r491-source-scope.json');oldchanged={x['path']:x['current_sha256']for x in previous['changed_source_files']};changed=[];names=set()
for b in base['entries']:
 path=b['path'].replace('\\','/');names.add(path);q=pin(P/path)
 if q['sha256']!=b['sha256']:assert oldchanged[path]==q['sha256'];changed.append(dict(path=path,baseline_sha256=b['sha256'],current_sha256=q['sha256']))
assert len(changed)==12 and len(names)==479
old=read(L/'r491-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert delta==['native_city_runtime.rs']
module=P/'src/modern_lito/native_source_connection.rs';diffs.append(''.join(difflib.unified_diff([],module.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(module))))
allfiles=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(allfiles)==606
new=[dict(path=str(p.relative_to(P)).replace('\\','/'),sha256=pin(p)['sha256'])for p in allfiles if str(p.relative_to(P)).replace('\\','/')not in names];assert len(new)==127
for q in previous['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
scope=dict(state='R492_ONE_EXISTING_RUNTIME_AND_NEW_BOUND_NATIVE_DECODED_CONNECTION_CANDIDATE_MODULE',source_files=606,baseline_source_files=479,unchanged_source_files=467,changed_source_files=changed,new_source_files=new,changed_from_fixed_R491=delta,added_from_fixed_R491=['native_source_connection.rs'],frozen_binary_pins=previous['frozen_binary_pins'],original_main_sha256=pin(P/'src/main.rs')['sha256'],authorization='T357 sourceproducertrace andnecessarybounddecodedingress/disconnectcandidatewithinModernownscope;normalNativewireOPEN.',scope='Runtime sourceevent candidateownsopaqueepochs/lifecycle;normalTLS/profile/session/transport/SourceFlow/actor/class/dispatch/window/menu/cache/strictdecoder/all40plans unchanged. No crossnamespaceconversion.')
save(L/'r492-source-scope.json',scope);save(L/'source-change-scope.json',scope);diff=L/'r492-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8')
out=P/'target/r492-production-source';out.mkdir(exist_ok=False);pins=[]
for p in allfiles:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r492-production-source-snapshot.json',dict(state='R492_ALL606_PRODUCTION_SOURCES_FROZEN_BEFORE_NEW_CONSUMER_EXECUTION',source_pins=pins,production_library=read(L/'r492-player-candidate.json')['production_library'],full_test_suite_executed=False,library_test_compile_performed_this_revision=False,source_difference=pin(diff),GUI=0,LOADED=False))
print('R492 scopePASS:606frozen/180consumer expected;Runtime changed/one newmodule;normalwire/sourceflow/rootmain/R408 preserved')
