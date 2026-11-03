"""Keep fixed-name historical JSON reports while retaining fresh test output."""
from pathlib import Path
import argparse
import hashlib
import json
import zipfile

root = Path(__file__).resolve().parents[1]
legacy = root / 'LiTo381-Modern/logs/continuation-20261003'
sha_bytes = lambda b: hashlib.sha256(b).hexdigest().upper()
ap = argparse.ArgumentParser()
ap.add_argument('mode', choices=['snapshot', 'collect-restore'])
ap.add_argument('run')
args = ap.parse_args()
run = Path(args.run).resolve(strict=True)
assert run.is_relative_to(root / '驗證紀錄/current')
archive = run / 'legacy-reports-before.zip'
manifest = run / 'legacy-reports-before.json'
if args.mode == 'snapshot':
    assert not archive.exists() and not manifest.exists()
    pins = {}
    with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for p in sorted(legacy.glob('*.json')):
            assert not p.is_symlink() and p.resolve().is_relative_to(legacy)
            data = p.read_bytes()
            pins[p.name] = sha_bytes(data)
            z.writestr(p.name, data)
    with zipfile.ZipFile(archive) as z:
        for name, expected in pins.items():
            assert sha_bytes(z.read(name)) == expected, name
    result = {'files': pins, 'archive_sha256': sha_bytes(archive.read_bytes()), 'readback_verified': True}
    manifest.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Protected {len(pins)} existing historical JSON reports; no game resource copies.')
else:
    assert not (run / 'legacy-report-restoration.json').exists()
    prior = json.loads(manifest.read_text(encoding='utf-8'))
    assert sha_bytes(archive.read_bytes()) == prior['archive_sha256']
    generated = run / 'legacy-generated-reports.zip'
    assert not generated.exists()
    changes = []
    with zipfile.ZipFile(archive) as before, zipfile.ZipFile(generated, 'w', zipfile.ZIP_DEFLATED, compresslevel=1) as after:
        for name, expected in prior['files'].items():
            p = legacy / name
            assert p.resolve().is_relative_to(legacy) and not p.is_symlink()
            original = before.read(name)
            assert sha_bytes(original) == expected
            current = p.read_bytes() if p.exists() else None
            if current is not None and sha_bytes(current) == expected:
                continue
            if current is not None:
                after.writestr(name, current)
            changes.append({'name': name, 'before_sha256': expected,
                            'generated_sha256': sha_bytes(current) if current is not None else None})
    # Verify every generated report before restoring the preserved originals.
    with zipfile.ZipFile(generated) as after, zipfile.ZipFile(archive) as before:
        for change in changes:
            if change['generated_sha256'] is not None:
                assert sha_bytes(after.read(change['name'])) == change['generated_sha256']
            p = legacy / change['name']
            p.write_bytes(before.read(change['name']))
            assert sha_bytes(p.read_bytes()) == change['before_sha256']
    result = {'status': 'generated_reports_retained_and_existing_history_restored', 'changes': changes,
              'before_archive_sha256': prior['archive_sha256'], 'generated_archive_sha256': sha_bytes(generated.read_bytes()),
              'existing_reports_restored': True, 'whole_goal_complete': False}
    (run / 'legacy-report-restoration.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(f'Retained {len(changes)} generated reports and restored their historical originals.')
