import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r540-native-pet-race-start-component'
old={d['path']:d for d in json.loads((l/'r539-final-production-source-snapshot.json').read_text(encoding='utf-8-sig'))['files']};rows=[];changed=[];added=[];unchanged=0
for f in sorted((p/'src').rglob('*.rs')):
 rel=f.relative_to(p).as_posix();h=hashlib.sha256(f.read_bytes()).hexdigest().upper();rows.append({'path':rel,'bytes':f.stat().st_size,'sha256':h})
 if rel not in old:added.append(rel.removeprefix('src/'))
 elif h!=old[rel]['sha256']:changed.append(rel.removeprefix('src/'))
 else:unchanged+=1
assert len(rows)==660 and unchanged==654
assert changed==['modern_lito/mod.rs','modern_lito/native_pet_race_quit_session.rs','modern_lito/native_pet_race_session.rs','modern_lito/native_pet_race_wire_boundary.rs']
assert added==['modern_lito/native_pet_race_start_component.rs','modern_lito/native_pet_race_start_component_tests.rs']
shutil.copytree(p/'src',t/'source/src')
with (l/'r540-production-source-snapshot.json').open('x',encoding='utf-8') as s:json.dump({'source_root':str(t/'source/src'),'source_count':660,'prior_sources_unchanged':654,'existing_sources_changed':changed,'new_sources':added,'existing_Receiver_Sender_reducer_and_READY_countdown_decoders_changed':False,'files':rows},s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'source_count':660,'unchanged':654,'changed':changed,'new':added}))
