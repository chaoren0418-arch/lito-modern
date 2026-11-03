import hashlib,json,pathlib,shutil
p=pathlib.Path(__file__).resolve().parent.parent;l=p/'logs/continuation-20261003';t=p/'target/r537-native-pet-race-return-session'
old={d['path']:d for d in json.loads((l/'r536-production-source-snapshot.json').read_text(encoding='utf-8-sig'))['files']}
rows=[];changed=[];added=[];unchanged=0
for f in sorted((p/'src').rglob('*.rs')):
    rel=f.relative_to(p).as_posix();h=hashlib.sha256(f.read_bytes()).hexdigest().upper();rows.append({'path':rel,'bytes':f.stat().st_size,'sha256':h})
    if rel not in old:added.append(rel.removeprefix('src/'))
    elif h!=old[rel]['sha256']:changed.append(rel.removeprefix('src/'))
    else:unchanged+=1
assert len(rows)==655 and unchanged==650
assert changed==['modern_lito/mod.rs','modern_lito/native_pet_race_quit_session.rs','modern_lito/native_pet_race_session.rs']
assert added==['modern_lito/native_pet_race_return_profile.rs','modern_lito/native_pet_race_return_profile_tests.rs']
prior_session=(p/'target/r536-native-pet-race-quit-session/source/src/modern_lito/native_pet_race_session.rs').read_text(encoding='utf-8')
new_session=(p/'src/modern_lito/native_pet_race_session.rs').read_text(encoding='utf-8')
helper='''    pub(super) fn local_exit_matches(&self, body: &[u8]) -> Result<bool> {
        match decode(body)? {
            Event::Exit(room, alias) => Ok(room == self.snapshot.room_id && alias == self.local_alias),
            _ => Ok(false),
        }
    }
'''
assert new_session.replace(helper,'')==prior_session
shutil.copytree(p/'src',t/'source/src')
with (l/'r537-production-source-snapshot.json').open('x',encoding='utf-8') as s:
    json.dump({'state':'R537_NEW_RETURN_PROFILE_SOURCE_FROZEN','source_root':str(t/'source/src'),'source_count':655,
        'prior_sources_unchanged':650,'existing_sources_changed':changed,'new_sources':added,
        'historical_session_change':'New sibling-only pure local_exit_matches helper; prior methods unchanged.',
        'historical_quit_profile':'Old source SHA/factory/return policy retained; successor CF/D505/local-exit guard and partial-failure gated by5350 source profile.',
        'files':rows},s,ensure_ascii=False,indent=2);s.write('\n')
print(json.dumps({'source_count':655,'unchanged':650,'changed':changed}))
