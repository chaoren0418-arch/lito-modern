from r491_common import *
import shutil,difflib
base=read(P/'logs/relocation-baseline-20261003/source-manifest.json');previous=read(L/'r490-source-scope.json');oldchanged={x['path']:x['current_sha256']for x in previous['changed_source_files']};changed=[];names=set()
for b in base['entries']:
 path=b['path'].replace('\\','/');names.add(path);q=pin(P/path)
 if q['sha256']!=b['sha256']:assert oldchanged[path]==q['sha256'];changed.append(dict(path=path,baseline_sha256=b['sha256'],current_sha256=q['sha256']))
assert len(changed)==12 and len(names)==479
old=read(L/'r490-production-source-snapshot.json');delta=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  delta.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert set(delta)=={'native_city_runtime.rs','native_city_window.rs'}
allfiles=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(allfiles)==605
new=[dict(path=str(p.relative_to(P)).replace('\\','/'),sha256=pin(p)['sha256'])for p in allfiles if str(p.relative_to(P)).replace('\\','/')not in names];assert len(new)==126
for q in previous['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
scope=dict(state='R491_ONLY_PLAYER_COMMAND_BINDING_AND_RECEIVED_SOURCE_ADMISSION_CHANGED',source_files=605,baseline_source_files=479,unchanged_source_files=467,changed_source_files=changed,new_source_files=new,changed_from_fixed_R490=delta,added_from_fixed_R490=[],frozen_binary_pins=previous['frozen_binary_pins'],original_main_sha256=pin(P/'src/main.rs')['sha256'],authorization='T350/T352 existingModernwrite scope. Closedsourcechain/callbackgaps;independentproductionplayercommandprogress. NoJavafixture/server/provider/GUI.',scope='Runtime localplayercommand->OpenSourceMenu withsourcephase2+snapshot guard/read-onlygeneration accessor;windowWM_COMMAND routesexactsharedhandler. PersistentFlow/snapshot/inspection/menu/worker/cache/strictdecoder/all40plans unchanged.')
save(L/'r491-source-scope.json',scope);save(L/'source-change-scope.json',scope);diff=L/'r491-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8')
out=P/'target/r491-production-source';out.mkdir(exist_ok=False);pins=[]
for p in allfiles:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r491-production-source-snapshot.json',dict(state='R491_ALL605_PRODUCTION_SOURCES_FROZEN_BEFORE_NEW_CONSUMER_EXECUTION',source_pins=pins,production_library=read(L/'r491-player-candidate.json')['production_library'],full_test_suite_executed=False,library_test_compile_performed_this_revision=False,source_difference=pin(diff),GUI=0,LOADED=False))
print('R491 scopePASS:605frozen/177consumer expected;two existingfileschanged;0newproductionfiles;rootmain/R408 preserved')
