"""Verify fixture provenance and current packet parity without relabeling old data."""
from pathlib import Path
import argparse
import hashlib
import json

root = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser()
ap.add_argument('fixture')
ap.add_argument('historical_oracle')
ap.add_argument('current_oracle')
ap.add_argument('current_jar')
ap.add_argument('output')
a = ap.parse_args()
paths = [Path(x).resolve(strict=True) for x in [a.fixture, a.historical_oracle, a.current_oracle, a.current_jar]]
assert all(p.is_relative_to(root) for p in paths)
destination = Path(a.output).resolve()
assert destination.is_relative_to(root)
def unique_object(pairs):
    result = {}
    for key, value in pairs:
        assert key not in result, ('duplicate JSON key', key)
        result[key] = value
    return result
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'), object_pairs_hook=unique_object)
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
fixture, old, current = [read(p) for p in paths[:3]]
assert fixture == old, 'Fixture differs from the oracle executed against its retained original JAR'
current_hash = sha(paths[3])
assert current['jar_sha256'].upper() == current_hash
before = fixture.copy(); after = current.copy()
old_hash = before.pop('jar_sha256').upper(); after.pop('jar_sha256')
assert before == after, 'Current protocol payload/schema differs from the historical unit vector'
result = {'status': 'verified_original_fixture_source_and_current_protocol_payload_identity',
          'fixture_jar_sha256': old_hash, 'current_jar_sha256': current_hash,
          'fixture_sha256': sha(paths[0]), 'original_oracle_sha256': sha(paths[1]),
          'current_oracle_sha256': sha(paths[2]), 'all_non_jar_fields_equal': True,
          'historical_fixture_relabelled': False, 'actual_ClientSession_verified': False,
          'whole_goal_complete': False}
assert not destination.exists()
destination.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
