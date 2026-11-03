"""Preserve the unapplied candidate-output collision; product bytes stay intact."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import json
import zipfile
R = Path(__file__).resolve().parents[1]
out = R / '維護工具/candidates/native-haste43-20261020-074307-266851'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
c = read(out / 'candidate.json')
assert sha(out / 'candidate.json') == '50EFE3A1FA06FEE8DF11E9AC0D0FB4C78652B1C64E6A3A17F203DE1FB33637B1'
assert not c['source_applied'] and not c['actual_component_or_library_tests_executed']
assert all(sha(R / name) == digest for name, digest in c['before'].items())
names = list(c['after'])
assert len(names) == 4 and len({Path(name).name for name in names}) == 2
assert c['after'][names[0]] == c['after'][names[2]] and c['after'][names[1]] == c['after'][names[3]]
assert 'fn packet(' in (out / 'native_skills.rs').read_text('utf-8-sig')
files = {path.name: path for path in out.iterdir() if path.is_file()}
assert 'rejection.json' not in files
pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(out / 'rejected-original.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, path in files.items():
        archive.write(path, name)
with zipfile.ZipFile(out / 'rejected-original.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in pins.items())
record = dict(status='rejected_unapplied_candidate_filename_collision', observed_utc=datetime.now(timezone.utc).isoformat(),
              candidate_sha256=sha(out / 'candidate.json'), cause='Four logical paths were written using two duplicate basenames; test files overwrote the two Runtime candidates.',
              correction_required='Keep four distinct paths and verify all writes and before/after identities.',
              original_files=pins, raw_archive_sha256=sha(out / 'rejected-original.zip'), normal_product_unchanged=True,
              source_applied=False, compiler_or_tests_executed=False, candidate_qualification_granted=False)
(out / 'rejection.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(dict(rejection=str(out / 'rejection.json'), source_applied=False, product_unchanged=True), ensure_ascii=False))
