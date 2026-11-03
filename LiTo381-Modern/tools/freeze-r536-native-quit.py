import hashlib
import json
import pathlib
import shutil
p=pathlib.Path(__file__).resolve().parent.parent
l=p/'logs/continuation-20261003'
t=p/'target/r536-native-pet-race-quit-session'
prior=json.loads((l/'r535-production-source-snapshot.json').read_text(encoding='utf-8-sig'))
old={d['path']:d for d in prior['files']}
rows=[];changed=[];added=[];unchanged=0
for f in sorted((p/'src').rglob('*.rs')):
    rel=f.relative_to(p).as_posix();h=hashlib.sha256(f.read_bytes()).hexdigest().upper()
    rows.append({'path':rel,'bytes':f.stat().st_size,'sha256':h})
    if rel not in old:added.append(rel.removeprefix('src/'))
    elif h!=old[rel]['sha256']:changed.append(rel.removeprefix('src/'))
    else:unchanged+=1
assert len(rows)==653 and unchanged==649
assert changed==['modern_lito/mod.rs','modern_lito/native_pet_race_session.rs']
assert added==['modern_lito/native_pet_race_quit_session.rs','modern_lito/native_pet_race_quit_session_tests.rs']
old_session=(p/'target/r535-native-pet-race-session/source/src/modern_lito/native_pet_race_session.rs').read_text(encoding='utf-8')
new_session=(p/'src/modern_lito/native_pet_race_session.rs').read_text(encoding='utf-8')
assert old_session.replace('    fn scope(', '    pub(super) fn scope(').replace('    fn close_commands(', '    pub(super) fn close_commands(')==new_session
shutil.copytree(p/'src',t/'source/src')
with (l/'r536-production-source-snapshot.json').open('x',encoding='utf-8') as s:
    json.dump({'state':'R536_NEW_CANDIDATE_QUIT_CONSUMER_SOURCE_FROZEN','source_root':str(t/'source/src'),
        'source_count':len(rows),'prior_sources_unchanged':unchanged,'existing_sources_changed':changed,'new_sources':added,
        'predecessor_session_behavior_unchanged_two_sibling_visibility_changes_only':True,'files':rows},s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'source_count':len(rows),'unchanged':unchanged,'existing_changes':changed}))
