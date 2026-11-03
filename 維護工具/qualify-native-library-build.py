"""Retain one measured library-test build, without claiming runtime success."""
from pathlib import Path
import argparse
import hashlib
import json
import zipfile

root = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest().upper()
ap = argparse.ArgumentParser()
ap.add_argument('run')
run = Path(ap.parse_args().run).resolve(strict=True)
assert run.is_relative_to(root / '驗證紀錄/current')
assert not (run / 'product-retention.json').exists(), 'Already retained; inspect instead of overwriting'
receipt = read(run / 'receipt.json')
assert receipt['exit_code'] == 0 and receipt['sources_unchanged']
assert receipt['test_debug_information'] == 0 and receipt['features'] == []
pins = read(run / 'source-inputs.json')
assert len(pins) == receipt['source_files']
assert sha(run / 'source-inputs.json') == receipt['source_manifest_sha256']
for name, expected in pins.items():
    assert sha(root / name) == expected, name
messages = []
for line in (run / 'cargo-artifacts.jsonl').read_text(encoding='utf-8-sig').splitlines():
    if line.startswith('{'):
        messages.append(json.loads(line))
assert any(d.get('reason') == 'build-finished' and d['success'] for d in messages)
tests = [d for d in messages if d.get('reason') == 'compiler-artifact' and d.get('executable') and d['profile']['test']]
assert len(tests) == 1 and tests[0]['target']['kind'] == ['lib'] and tests[0]['features'] == []
exe = Path(tests[0]['executable']).resolve(strict=True)
assert exe.is_relative_to(root / 'LiTo381-Modern/target/current')
digest = sha(exe)
archive = run / 'library-test-product.zip'
assert not archive.exists()
with zipfile.ZipFile(archive, 'w', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    z.write(exe, exe.name)
with zipfile.ZipFile(archive) as z:
    assert hashlib.sha256(z.read(exe.name)).hexdigest().upper() == digest
samples = [json.loads(x) for x in (run / 'memory-samples.jsonl').read_text(encoding='utf-8-sig').splitlines() if x.strip()]
peak = max((p['private_bytes'], p['name']) for s in samples for p in s['processes'])
result = {'status': 'retained_full_library_test_debug0_ready_for_execution',
          'product': str(exe), 'product_sha256': digest, 'bytes': exe.stat().st_size,
          'features': [], 'test_debug_information': 0, 'codegen_units': receipt.get('codegen_units', 256),
          'library_opt_level': receipt.get('library_opt_level', 0), 'archive': str(archive),
          'archive_sha256': sha(archive), 'archive_readback_verified': True,
          'sampled_peak_private_bytes': peak[0], 'sampled_peak_process_name': peak[1],
          'minimum_sampled_host_available_commit_kib': min(s['host_memory']['available_commit_kib'] for s in samples),
          'source_manifest_sha256': sha(run / 'source-inputs.json'),
          'qualifier_sha256': sha(Path(__file__)), 'runtime_verified': False, 'whole_goal_complete': False}
(run / 'product-retention.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({k: result[k] for k in ['status', 'product_sha256', 'sampled_peak_private_bytes']}, ensure_ascii=False))
