"""Compare current request routes with the learned books in one qualified case.

This is a static request inventory with retained wire evidence, not new runtime
casts, GUI acceptance, or a promotion of unrelated older-source results.
"""
from pathlib import Path
from datetime import datetime, timezone
import argparse
import csv
import hashlib
import json
import re
import zipfile

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    return json.loads(Path(path).read_text(encoding='utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()


ap = argparse.ArgumentParser()
ap.add_argument('case_qualification', type=Path)
a = ap.parse_args()
qp = a.case_qualification.resolve(strict=True)
assert qp.is_relative_to(ROOT / '驗證紀錄/current/native-G06-controller')
q = read(qp)
assert q['status'] == 'independently_readback_qualified_current_G06_TW_extended_controller_case'
assert q['scenario'] == 'cross-endpoint' and q['actual_formal_sessions'] == 6
assert q['original_owned_processes_terminal']
run = qp.parent.parent.parent
c = read(run / 'cohort.json')
assert c['source_manifest_sha256'] == q['source_manifest_sha256']
pins = read(run / 'source-inputs.json')
assert len(pins) == 1028 and sha(run / 'source-inputs.json') == q['source_manifest_sha256']
for name, expected in pins.items():
    assert sha(ROOT / name) == expected, name
formal = ROOT.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
assert sha(formal) == q['reference_jar_sha256'] == c['reference_jar_sha256']
raw_path = qp.parent / 'raw.zip'
assert sha(raw_path) == q['raw_archive_sha256']
with zipfile.ZipFile(raw_path) as archive:
    assert set(archive.namelist()) == set(q['raw_files'])
    for name, expected in q['raw_files'].items():
        with archive.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == expected
    captures = [n for n in archive.namelist() if n.endswith('/frames.jsonl')]
    assert len(captures) == 1
    wire = []
    for line_number, line in enumerate(archive.read(captures[0]).decode('utf-8-sig').splitlines(), 1):
        row = json.loads(line)
        if row.get('direction') != 'server_to_client' or not re.fullmatch('[0-9A-Fa-f ]*', row.get('plain', '')):
            continue
        b = bytes.fromhex(row['plain'])
        if not b or b[0] != 0x1e:
            continue
        assert len(b) >= 2 and b[1] <= 28 and len(b) == b[1] + 2
        assert b[1] < 28 or b[-1] & 0xf0 == 0
        ids = sorted(j * 8 + k + 1 for j, bits in enumerate(b[2:]) for k in range(8) if bits & (1 << k))
        wire.append(dict(line=line_number, note=row['note'], payload_hex=b.hex(), learned_ids=ids))
    roles = {}
    for role in ['a', 'b']:
        report = json.loads(archive.read('case/' + role + '-handoff.json').decode('utf-8-sig'))
        ids = report['before']['world']['learned_skills']
        assert report['before']['phase'] == 'battle_pending'
        # This retained checkpoint precedes destination world bootstrap. Its
        # empty learned list is world retirement, not a source-book mismatch.
        assert report['destination']['phase'] == 'character_selected'
        assert report['before']['selected_character'] == report['destination']['selected_character']
        matching = [packet for packet in wire if packet['learned_ids'] == ids]
        assert any(packet['note'] == 'S_ADD_SPELL selected-character' for packet in matching)
        assert any(packet['note'] == f'S_ADD_SPELL learned-count={len(ids)}' for packet in matching)
        roles[role] = dict(learned_ids=ids, matching_wire_books=matching)

src = ROOT / 'LiTo381-Modern/src/modern_lito/native_skills.rs'
text = src.read_text(encoding='utf-8-sig')
request = text[text.index('pub fn request('):text.index('pub fn reuse_millis(')]
recognized = set()
for match in re.finditer(r'^\s*((?:\d+\s*\|\s*)*\d+)\s*=>', request, re.M):
    recognized.update(map(int, re.findall(r'\d+', match.group(1))))
ground = text[text.index('pub fn ground_request('):text.index('pub fn in_native_screen(')]
match = re.search(r'ensure!\(matches!\(skill,\s*([\d |]+)\)', ground)
assert match
ground_ids = set(map(int, re.findall(r'\d+', match.group(1))))
selector_match = re.search(r'^\s*([\d |]+)\s*=>\s*\{ensure!\(target.is_none\(\), "此技能不接受指定目標"\)', request, re.M)
assert selector_match
selector_ids = set(map(int, re.findall(r'\d+', selector_match.group(1))))
assert selector_ids == {14, 31, 32, 49, 52, 54, 72, 78, 206}
contract = ROOT / '驗證紀錄/current/native-current-skill-contract-audit'
factory_q = read(contract / 'qualified.json')
assert factory_q['jar_sha256'] == q['reference_jar_sha256'] and factory_q['archive_readback_verified']
assert sha(contract / 'current-skill-contract-evidence.zip') == factory_q['archive_sha256']
assert sha(contract / 'current-learned-factories.json') == factory_q['file_pins']['current-learned-factories.json']
factories = {row['skill_id']: row for row in read(contract / 'current-learned-factories.json')['rows']}
learned = set().union(*(set(role['learned_ids']) for role in roles.values()))
assert learned == set(factories) and len(learned) == 74
missing = learned - (recognized | ground_ids)
rows = []
for skill in sorted(learned):
    kind = ('ground' if skill in ground_ids else 'selector' if skill in selector_ids else
            'self_tick' if skill == 60 and skill in recognized else
            'object_or_selector' if skill in recognized else 'missing')
    rows.append(dict(skill_id=skill, request_recognized=skill not in missing, request_kind=kind,
        formal_catalog_definitions=factories[skill]['matches'],
        current_cast_success_inferred=False, GUI_verified=False))
out = ROOT / '驗證紀錄/current/native-current-skill-support' / ('selector-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
result = dict(status='current_source_request_inventory_bound_to_retained_qualified_wire_books',
    source_manifest_sha256=q['source_manifest_sha256'], source_files=1028,
    request_source_sha256=sha(src), reference_jar_sha256=q['reference_jar_sha256'],
    retained_runtime_profile=q['profile'], retained_runtime_book_scope='source battle world before transfer',
    destination_book_continuity_inferred=False, retained_runtime_qualification=str(qp),
    retained_runtime_qualification_sha256=sha(qp), retained_runtime_raw_sha256=sha(raw_path),
    observed_unique_learned_ids=len(learned), request_recognized_learned_ids=len(learned - missing),
    missing_unique_learned_requests=len(missing), missing_ids=sorted(missing),
    selector_ids=sorted(selector_ids), roles=roles, rows=rows,
    new_sessions_executed=0, new_casts_executed=0,
    request_recognition_is_not_gameplay_acceptance=True, GUI_verified=False,
    all_learned_skills_native_supported=False, whole_goal_complete=False,
    observed_utc=datetime.now(timezone.utc).isoformat())
matrix = out / 'support-matrix.json'
matrix.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
with (out / 'support-matrix.csv').open('w', encoding='utf-8-sig', newline='') as stream:
    writer = csv.writer(stream)
    writer.writerow(['skill_id', 'request_recognized', 'request_kind', 'formal_factory', 'new_casts_executed'])
    for row in rows:
        writer.writerow([row['skill_id'], row['request_recognized'], row['request_kind'],
                         '/'.join(f['factory'] for f in row['formal_catalog_definitions']), 0])
files = {'auditor.py': Path(__file__), 'source-inputs.json': run / 'source-inputs.json',
    'native_skills.rs': src, 'runtime-qualification.json': qp,
    'factory-qualification.json': contract / 'qualified.json',
    'current-learned-factories.json': contract / 'current-learned-factories.json',
    'support-matrix.json': matrix, 'support-matrix.csv': out / 'support-matrix.csv'}
hashes = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'raw.zip') as archive:
    assert set(archive.namelist()) == set(hashes)
    for name, expected in hashes.items():
        with archive.open(name) as stream:
            assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == expected
(out / 'archive-readback.json').write_text(json.dumps(dict(archive_sha256=sha(out / 'raw.zip'),
    files=hashes), ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
for name, expected in pins.items():
    assert sha(ROOT / name) == expected
print(json.dumps(dict(output=str(out), learned=len(learned), recognized=len(learned - missing),
                       missing=len(missing), new_casts=0), ensure_ascii=False))
