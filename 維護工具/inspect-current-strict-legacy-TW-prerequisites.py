"""Read-only prerequisite inspection; no tests, fallback, or source changes."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import stat

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
SOURCE = 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

w = read(R / '工作佇列.json')
assert w['current_product_identity']['source_manifest_sha256'] == SOURCE
review_path = V / 'native-library-remaining-optin-review/20261020-041932-322366/review.json'
review = read(review_path)
assert review['source_manifest_sha256'] == SOURCE
tests = [row for row in review['tests'] if row['requires_strict_legacy_TW']]
assert len(tests) == 5
for row in tests:
    assert sha(R / row['source']) == row['source_sha256']
root = R.parent.parent / '0-客戶端'
observations = {}
for relative in ['TW13081901.pak', 'launcher/TW13081901.pak', 'TW13081901.txt']:
    path = root / relative
    try:
        before = path.lstat()
    except FileNotFoundError as error:
        observations[relative] = dict(path=str(path), exists=False,
                                      observed_exception=type(error).__name__)
        continue
    assert stat.S_ISREG(before.st_mode) and not before.st_file_attributes & 0x400
    digest = sha(path)
    after = path.lstat()
    assert (before.st_size, before.st_mtime_ns) == (after.st_size, after.st_mtime_ns)
    observations[relative] = dict(path=str(path), exists=True, bytes=before.st_size, sha256=digest)
assert not observations['TW13081901.pak']['exists']
assert observations['launcher/TW13081901.pak']['sha256'] == 'B3788DF4C7F077EF3FF15B070A5D020CCB224E48DEC3757C4CD34528FCA4B218'
assert observations['TW13081901.txt']['sha256'] == '7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008'
resources = R / 'LiTo381-Modern/src/modern_lito/resources.rs'
text = resources.read_text('utf-8')
legacy = text.split('fn read_shared_tw_action_pack(', 1)[1].split('\n}', 1)[0]
current = text.split('fn read_shared_launcher_tw_action_pack(', 1)[1].split('\n}', 1)[0]
assert '&[],' in legacy and '&["launcher".to_owned()]' in current
catalog = R / 'LiTo381-Modern/src/modern_lito/tw_action_catalog.rs'
expected = 'FC7B4CAC77379A98E2A2EDF3F645BB82CA8BE9EE3F2B7FAE76FCC61423EE8798'
assert expected.lower() in catalog.read_text('utf-8')
result = dict(status='strict_legacy_TW_root_input_absent_static_prerequisite_only',
              recorded_utc=datetime.now(timezone.utc).isoformat(), source_manifest_sha256=SOURCE,
              review=str(review_path), review_sha256=sha(review_path), tests=tests,
              resource_observations=observations, expected_legacy_root_pak_sha256=expected,
              loader_source=str(resources), loader_source_sha256=sha(resources),
              catalog_source=str(catalog), catalog_source_sha256=sha(catalog),
              prerequisite_satisfied=False, tests_executed=0,
              fallback_or_input_copy_performed=False, product_changes=False,
              current_launcher_qualification_invalidated=False, whole_goal_complete=False)
out = V / 'native-strict-legacy-TW-prerequisites' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'inspection.py').write_bytes(Path(__file__).read_bytes())
(out / 'evidence.json').write_text(json.dumps(result, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')
assert read(out / 'evidence.json') == result
print(json.dumps(dict(evidence=str(out / 'evidence.json'), sha256=sha(out / 'evidence.json'), missing_input='TW13081901.pak', affected_tests=5, tests_executed=0), ensure_ascii=False))
