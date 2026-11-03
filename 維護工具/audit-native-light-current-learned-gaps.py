"""Compare current qualified spell books and request arms without new sessions."""
import hashlib
import json
import re
from pathlib import Path
import zipfile

ROOT = Path(__file__).resolve().parents[1]
CASE = ROOT / '驗證紀錄/current/native-light-gui/20261015-214309-182'
OUT = ROOT / '驗證紀錄/current/native-light-spell/learned-request-gaps'
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
normal_path = ROOT / '驗證紀錄/current/native-light-spell/normal/qualified.json'
normal = read(normal_path)
pins = read(normal_path.parent / 'source-inputs.json')
assert len(pins) == normal['source_files'] == 970
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
qualified = read(CASE / 'qualified.json')
receipt = read(CASE / 'receipt.json')
assert qualified['raw_archive_readback_verified'] and qualified['actual_sessions'] == 4
assert qualified['main_sha256'] == receipt['exe_sha256'] == normal['profiles']['Release']['sha256']
assert qualified['source_manifest_sha256'] == normal['source_manifest_sha256']
assert receipt['resource_profile'] == 'tw13081901-v1' and receipt['extended_item_details']
capture, = list((CASE / 'raw-capture').rglob('frames.jsonl'))
assert sha(capture) in qualified['raw_archive_files'].values()
books = {}
for frame in (json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()):
    if frame['direction'] != 'server_to_client' or int(frame['opcode'], 16) != 0x1e:
        continue
    payload = bytes.fromhex(frame['plain'])
    assert payload[0] == 0x1e and payload[1] <= 28 and len(payload) == payload[1] + 2
    assert payload[1] < 28 or payload[-1] & 0xf0 == 0
    ids = tuple(j*8+k+1 for j, value in enumerate(payload[2:]) for k in range(8) if value & (1 << k))
    books.setdefault(ids, []).append(frame['note'])
assert sorted(map(len, books)) == [19, 64]
for ids, notes in books.items():
    assert sorted(notes) == sorted(['S_ADD_SPELL selected-character', f'S_ADD_SPELL learned-count={len(ids)}'])
source = ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs'
text = source.read_text(encoding='utf-8')
request = text[text.index('pub fn request('):text.index('pub fn reuse_millis(')]
supported = set()
for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>', request, re.M):
    supported.update(map(int, re.findall(r'\d+', match.group(1))))
assert supported == {1,2,3,4,6,7,8,9,10,11,15,16,17,18,19,28,35,57,131}
ground = {58, 63}
roles = {}
for count, role in [(19, 'a'), (64, 'b')]:
    learned = next(set(ids) for ids in books if len(ids) == count)
    roles[role] = {'learned_ids': sorted(learned), 'missing_request_ids': sorted(learned - supported - ground)}
assert len(roles['a']['missing_request_ids']) == 5 and len(roles['b']['missing_request_ids']) == 52
unique = set().union(*(set(r['learned_ids']) for r in roles.values()))
missing = unique - supported - ground
assert len(unique) == 74 and len(missing) == 53
proof = {'status': 'current_request_recognition_gaps_confirmed_not_complete_gameplay',
         'normal_qualification_sha256': sha(normal_path), 'main_sha256': receipt['exe_sha256'],
         'source_files': 970, 'source_manifest_sha256': normal['source_manifest_sha256'],
         'qualified_GUI_case_sha256': sha(CASE / 'qualified.json'), 'capture_sha256': sha(capture),
         'native_request_source_sha256': sha(source), 'native_request_ids': sorted(supported),
         'native_ground_request_ids': sorted(ground), 'roles': roles,
         'unique_learned_ids': len(unique), 'missing_request_ids': sorted(missing),
         'unique_missing_request_ids': len(missing), 'new_sessions': 0,
         'recognition_is_not_all_target_gameplay_or_rendering_proof': True,
         'whole_goal_complete': False}
OUT.mkdir(exist_ok=False)
(OUT / 'gap.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
files = {'auditor.py': Path(__file__), 'gap.json': OUT / 'gap.json', 'frames.jsonl': capture,
         'GUI-qualified.json': CASE / 'qualified.json', 'receipt.json': CASE / 'receipt.json',
         'native_skills.rs': source, 'normal-qualified.json': normal_path,
         'source-inputs.json': normal_path.parent / 'source-inputs.json'}
file_pins = {name: sha(path) for name, path in files.items()}
archive = OUT / 'gap-evidence.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    for name, expected in file_pins.items():
        assert hashlib.sha256(z.read(name)).hexdigest().upper() == expected
(OUT / 'qualified.json').write_text(json.dumps({'status': proof['status'], 'file_pins': file_pins,
    'archive_sha256': sha(archive), 'archive_readback_verified': True,
    'unique_learned_ids': 74, 'unique_missing_request_ids': 53, 'new_sessions': 0,
    'whole_goal_complete': False}, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'unique_learned_ids': 74, 'unique_missing_request_ids': 53,
                  'missing_per_role': {r: len(v['missing_request_ids']) for r, v in roles.items()},
                  'new_sessions': 0}, ensure_ascii=False))
