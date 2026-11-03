import pathlib,json,hashlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r540-native-pet-race-start-component-final';v=json.loads((l/'r540-production-source-snapshot.json').read_text(encoding='utf-8-sig'));changed=[]
for d in v['files']:
 f=p/d['path'];h=hashlib.sha256(f.read_bytes()).hexdigest().upper()
 if h!=d['sha256']:changed.append(d['path'])
 d.update(bytes=f.stat().st_size,sha256=h)
assert changed==['src/modern_lito/native_pet_race_start_component.rs'],changed
shutil.copytree(p/'src',t/'source/src');v.update(source_root=str(t/'source/src'),repair_changed=changed,initial_failed_source_retained=str(l/'r540-production-source-snapshot.json'))
with (l/'r540-final-production-source-snapshot.json').open('x',encoding='utf-8') as f:json.dump(v,f,ensure_ascii=False,indent=2);f.write('\n')
