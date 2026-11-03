import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r538-native-pet-race-wire-boundary'
old={d['path']:d for d in json.loads((l/'r537-final-production-source-snapshot.json').read_text(encoding='utf-8-sig'))['files']}
rows=[];changed=[];added=[];unchanged=0
for f in sorted((p/'src').rglob('*.rs')):
    rel=f.relative_to(p).as_posix();h=hashlib.sha256(f.read_bytes()).hexdigest().upper();rows.append({'path':rel,'bytes':f.stat().st_size,'sha256':h})
    if rel not in old:added.append(rel.removeprefix('src/'))
    elif h!=old[rel]['sha256']:changed.append(rel.removeprefix('src/'))
    else:unchanged+=1
assert len(rows)==657 and unchanged==654
assert changed==['modern_lito/mod.rs']
assert added==['modern_lito/native_pet_race_wire_boundary.rs','modern_lito/native_pet_race_wire_boundary_tests.rs']
shutil.copytree(p/'src',t/'source/src')
with (l/'r538-production-source-snapshot.json').open('x',encoding='utf-8') as s:
    json.dump({'state':'R538_EXISTING_RECEIVER_WIRE_CONSUMER_SOURCE_FROZEN','source_root':str(t/'source/src'),'source_count':657,
        'prior_sources_unchanged':654,'existing_sources_changed':changed,'new_sources':added,'existing_Native_cipher_receiver_source_changed':False,
        'R537_consumer_source_changed':False,'files':rows},s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'source_count':657,'unchanged':654,'new_sources':added}))
