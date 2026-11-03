from r491_common import *
import shutil,difflib
O=P/'target/r493-source-producer-study'
f=O/'player-prelink-attempt1.txt';f.write_text('Nested Windows PowerShell5 defaultGetContent decoded UTF8 Chinese path asCP950;ConvertFromJson rejected escape before any player rustc/link/copy. Productionlibrary build alreadyPASS. Currentpwsh andexplicitEncodingUTF8 repair. No player execution/consumerexecution.\n',encoding='utf-8')
for n in ['build-r493-player-candidate.ps1','build-r493-source-identity-consumer.ps1']:
 p=P/'tools'/n;s=p.read_text(encoding='utf-8');shutil.copyfile(p,O/(n+'.attempt1'));s=s.replace('-Raw|','-Raw -Encoding UTF8|');p.write_text(s,encoding='utf-8')
p=P/'tools/r493-source-identity-consumer.rs';s=p.read_text(encoding='utf-8').replace('let flow=rt.source_flow().clone();','let flow=format!("{:?}",rt.source_flow());').replace('assert_eq!(rt.source_flow(),&flow);','assert_eq!(format!("{:?}",rt.source_flow()),flow);');p.write_text(s,encoding='utf-8')
old=read(L/'r492-production-source-snapshot.json');deltas=[];diffs=[]
for q in old['source_pins']:
 assert pin(q['snapshot'])['sha256']==q['sha256']
 if pin(q['path'])['sha256']!=q['sha256']:
  deltas.append(Path(q['path']).name);diffs.append(''.join(difflib.unified_diff(Path(q['snapshot']).read_text(encoding='utf-8').splitlines(True),Path(q['path']).read_text(encoding='utf-8').splitlines(True),fromfile=q['snapshot'],tofile=q['path'])))
assert deltas==['native_city_runtime.rs','native_source_connection.rs'],deltas
new=P/'src/modern_lito/native_source_identity.rs';diffs.append(''.join(difflib.unified_diff([],new.read_text(encoding='utf-8').splitlines(True),fromfile='/dev/null',tofile=str(new))))
diff=L/'r493-source-difference.patch';diff.write_text('\n'.join(diffs),encoding='utf-8')
allfiles=[p for p in sorted((P/'src').rglob('*'))if p.is_file()];assert len(allfiles)==607
out=P/'target/r493-production-source';out.mkdir(exist_ok=False);pins=[]
for p in allfiles:
 dest=out/p.relative_to(P);dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(p,dest);q=pin(p);assert pin(dest)['sha256']==q['sha256'];q['snapshot']=str(dest);pins.append(q)
save(L/'r493-production-source-snapshot.json',dict(state='R493_ALL607_PRODUCTION_SOURCE_FROZEN_BEFORE_NEW_CONSUMER_EXECUTION',source_pins=pins,source_difference=pin(diff),GUI=0))
scope=read(L/'r492-source-scope.json');scope.update(state='R493_TYPED_NATIVE_ROSTER_SELECTED_IDENTITY_SOURCE_SCOPE',source_files=607,changed_from_fixed_R492=deltas,added_from_fixed_R492=['native_source_identity.rs'],authorization='T363 finite source qualified producer and selectedidentity lifetime work',scope='Only Runtime+decodedvalidation changed/new sourceidentity. Native3/203 parser and selectedcontrol;normalNativewire/actor205/209/version/tableproducer/window/GUI/phase6OPEN. Allother604productionfiles unchanged fromR492.')
scope['new_source_files'].append(dict(path='src/modern_lito/native_source_identity.rs',sha256=pin(new)['sha256']));scope.pop('changed_from_fixed_R491',None);scope.pop('added_from_fixed_R491',None)
for q in scope['new_source_files']:q['sha256']=pin(P/q['path'])['sha256']
for q in scope['frozen_binary_pins']:assert pin(q['path'])['sha256']==q['sha256']
save(L/'r493-source-scope.json',scope);save(L/'source-change-scope.json',scope)
save(L/'r493-preparation-history.json',dict(player_prelink_failures=1,prelink_failure=pin(f),failed_build_script=pin(O/'build-r493-player-candidate.ps1.attempt1'),Rust_production_build_failures=0,consumer_compile_failures=0,consumer_test_failures=0,prebuild_consumer_correction='Flow hasnoPartialEq;comparecompleteDebugstamp beforefirstcompile,productionunchanged',source_prep_attempts=3,source_preparation_failures=0,PE_original_execution=False))
print('R493 scope frozen607/2changed+1new;UTF8prelink failure preserved;no oldPASS')
