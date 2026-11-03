import hashlib
import json
import pathlib
import shutil

p = pathlib.Path(__file__).resolve().parent.parent
l = p / 'logs/continuation-20261003'
t = p / 'target/r535-native-pet-race-session'
def read(f): return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h = hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda: s.read(1024 * 1024), b''): h.update(b)
    return {'path': str(f), 'bytes': f.stat().st_size, 'sha256': h.hexdigest().upper()}
def check(d):
    actual = pin(pathlib.Path(d['path']))
    assert (actual['bytes'], actual['sha256']) == (d['bytes'], d['sha256'].upper()), actual
def save(f, v):
    with f.open('x', encoding='utf-8') as s:
        json.dump(v, s, ensure_ascii=False, indent=2); s.write('\n')

source = p.parent.parent / 'lito伺服器/Native-PetRace-D9-cleanup候選交接.json'
check({'path': str(source), 'bytes': 3844, 'sha256': '64A40A2CC2115FC844D9F8FB7278794935C0D476C66CF1DA580ED00B3C7F1390'})
v = read(source)
refs = [d for d in v.values() if isinstance(d, dict) and {'path', 'bytes', 'sha256'} <= d.keys()]
assert len(refs) == 10
for d in refs: check(d)
m = read(pathlib.Path(v['manifest']['path']))
assert len(m['files']) == 672
owner = pathlib.Path(v['root']).resolve()
for d in m['files']:
    assert pathlib.Path(d['path']).resolve().is_relative_to(owner)
    check(d)
shutil.copyfile(source, l / 'r535-D9-cleanup-server-root.json')
dest = p / 'target/r535-native-pet-race-cleanup-evidence'
dest.mkdir()
copied = []
for key in ['manifest', 'source_diff', 'verification', 'candidate_result', 'baseline_result', 'version_binding']:
    f = pathlib.Path(v[key]['path'])
    target = dest / (key + f.suffix)
    shutil.copyfile(f, target)
    copied.append(pin(target))
save(l / 'r535-D9-cleanup-evidence-adoption.json', {
    'state': 'READ_ONLY_CANDIDATE_EVIDENCE_PINNED_NOT_PRODUCTION_ADOPTED',
    'server_root': pin(source), 'references_checked': len(refs), 'manifest_files_checked': len(m['files']),
    'copied_own_evidence': copied, 'candidate_source': v['candidate_source'],
    'historical_outer_source_retained': v['historical_outer_root_preserved'],
    'modern_contract_profile_changed': False, 'actual_new_Java_execution': 0,
    'scope': 'Actual methods on constructorless GameHub field fixture and memory sockets; map566 two sessions. Clears gfx and timedStatus before failed notification.',
    'limits': ['Notification IOException still retains room battleStarted and both BATTLE phases.',
               'Six next-prepare calls require explicit fixture room/phase reset, not full reentry.',
               'Neither existing backend485F overlay nor deployed server proven adopted; not leave/ACK proof.']})

prior = read(l / 'r534-final-production-source-snapshot.json')
old = {d['path']: d for d in prior['files']}
files = sorted((p / 'src').rglob('*.rs'))
rows = []
changed = []
added = []
unchanged = 0
for f in files:
    rel = f.relative_to(p).as_posix()
    a = pin(f); a['path'] = rel; rows.append(a)
    if rel not in old: added.append(rel.removeprefix('src/'))
    elif a['sha256'] != old[rel]['sha256']: changed.append(rel.removeprefix('src/'))
    else: unchanged += 1
assert len(rows) == 651 and unchanged == 648
assert changed == ['modern_lito/mod.rs']
assert added == ['modern_lito/native_pet_race_session.rs', 'modern_lito/native_pet_race_session_tests.rs']
shutil.copytree(p / 'src', t / 'source/src')
save(l / 'r535-production-source-snapshot.json', {
    'state': 'R535_NEW_SESSION_SOURCE_FROZEN', 'source_root': str(t / 'source/src'),
    'source_count': len(rows), 'prior_sources_unchanged': unchanged,
    'existing_sources_changed': changed, 'new_sources': added, 'files': rows})
print(json.dumps({'source_count': len(rows), 'unchanged': unchanged, 'candidate_manifest_checked': 672}))
