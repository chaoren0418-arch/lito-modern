import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003'
initial=p/'target/r537-native-pet-race-return-session';final=p/'target/r537-native-pet-race-return-session-final'
manifest=json.loads((l/'r537-production-source-snapshot.json').read_text(encoding='utf-8-sig'))
initial_tools=initial/'tools';initial_tools.mkdir()
for n in ['prepare-r537-native-return.py','freeze-r537-native-return.py','build-r537-native-return.ps1']:
    shutil.copyfile(p/'tools'/n,initial_tools/n)
rows=[];changed=[]
for d in manifest['files']:
    f=p/d['path'];h=hashlib.sha256(f.read_bytes()).hexdigest().upper();rows.append({'path':d['path'],'bytes':f.stat().st_size,'sha256':h})
    if h!=d['sha256']:changed.append(d['path'])
assert changed==['src/modern_lito/native_pet_race_quit_session.rs']
prior=(initial/'source/src/modern_lito/native_pet_race_quit_session.rs').read_text(encoding='utf-8')
current=(p/'src/modern_lito/native_pet_race_quit_session.rs').read_text(encoding='utf-8')
assert prior.replace('super::native_java_room_catalog::decode(body)?','super::native_city_runtime::java_room_catalog::decode(body)?')==current
shutil.copytree(p/'src',final/'source/src')
manifest.update({'state':'R537_FINAL_DECODER_MODULE_PATH_REPAIR_SOURCE_FROZEN','source_root':str(final/'source/src'),
    'files':rows,'initial_snapshot_retained':True,'repair_vs_initial':changed,'initial_compile_error':'E0433 wrong room catalog module path',
    'initial_tests_executed':0,'prior_regular_library_compile_succeeded':False})
with (l/'r537-final-production-source-snapshot.json').open('x',encoding='utf-8') as s:json.dump(manifest,s,ensure_ascii=False,indent=2);s.write('\n')
print('R537 initial source/tools retained; one module path correction frozen separately.')
