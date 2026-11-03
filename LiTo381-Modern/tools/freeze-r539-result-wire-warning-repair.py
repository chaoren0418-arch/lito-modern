import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r539-native-pet-race-result-wire-final'
old=json.loads((l/'r539-production-source-snapshot.json').read_text(encoding='utf-8-sig'));rows=[];changed=[]
for d in old['files']:
    f=p/d['path'];h=hashlib.sha256(f.read_bytes()).hexdigest().upper();rows.append({'path':d['path'],'bytes':f.stat().st_size,'sha256':h})
    if h!=d['sha256']:changed.append(d['path'])
assert changed==['src/modern_lito/native_pet_race_result_wire_tests.rs']
a=(p/'target/r539-native-pet-race-result-wire/source'/changed[0]).read_bytes();b=(p/changed[0]).read_bytes();assert b==a.replace(b'and_EOF_retirement_fail_closed',b'and_eof_retirement_fail_closed')
shutil.copytree(p/'src',t/'source/src');(t/'library').mkdir();shutil.copyfile(p/'target/r539-native-pet-race-result-wire/library/liblito_modern.rlib',t/'library/liblito_modern.rlib')
old.update({'source_root':str(t/'source/src'),'files':rows,'test_name_only_repair':changed,'production_runtime_changes_since_initial_PASS':0,'regular_library_rebuilt':False,'initial_source_snapshot_retained':str(l/'r539-production-source-snapshot.json')})
with (l/'r539-final-production-source-snapshot.json').open('x',encoding='utf-8') as f:json.dump(old,f,ensure_ascii=False,indent=2);f.write('\n')
print('R539 test-name-only source repair frozen; library copied unchanged.')
