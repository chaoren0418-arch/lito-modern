"""Read back the bounded current-host Release CLI loader checks; no relaunch."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, hashlib, json, runpy, zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
out = a.run.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-release-loader')
r = read(out / 'qualified.json')
assert r['status'] == 'current_Release_loader_and_missing_resource_failure_verified_in_empty_owned_output'
assert not r['clean_machine_verified'] and not r['portable_package_accepted']
normal = read(r['normal_qualification'])
exe = Path(r['immutable_exe']).resolve(strict=True)
assert exe.is_relative_to(out / 'empty-output')
assert list(exe.parent.iterdir()) == [exe]
assert sha(exe) == r['normal_Release_sha256'] == normal['profiles']['Release']['sha256']
assert sha(normal['profiles']['Release']['path']) == r['normal_Release_sha256']
version = read(out / 'version.stdout.json')
assert version['application'] == 'LiToModern' and version['build'] == 'release' and version['pointer_bits'] == 64
assert not version['launches_native_bin'] and not version['launches_tw']
missing = read(out / 'missing-resources.stdout.json')
assert missing['schema'] == 1 and missing['status'] == 'failed' and missing['error']
for name, expected in [('version', 0), ('missing-resources', 2)]:
    result = r['commands'][name]
    assert result['exit_code'] == expected and result['command'][0] == str(exe)
    assert sha(out / (name + '.stdout.json')) == result['stdout_sha256']
    assert sha(out / (name + '.stderr.txt')) == result['stderr_sha256']
assert r['commands']['version']['command'] == [str(exe), '--version']
assert r['commands']['missing-resources']['command'] == [str(exe), '--verify-resources', str(out / 'empty-resources'), '--output', str(out / 'missing-resource-diagnostics')]
assert not list((out / 'empty-resources').iterdir())
tool = ROOT / '維護工具/inspect-release-dependencies.py'
inspected = runpy.run_path(str(tool), run_name='independent_dependency_readback')['inspect'](exe)
def ordered_report(value):
    result = value.copy()
    # The inspector sorts a set by lowercase names; KERNEL32/kernel32 can
    # exchange order across Python processes. Preserve both exact records.
    for key in ['ordinary_imports', 'delayed_imports']:
        result[key] = sorted(result[key], key=lambda name: (name.lower(), name))
    result['dependencies'] = sorted(result['dependencies'], key=lambda row: (row['name'].lower(), row['name']))
    return result
assert ordered_report(inspected) == ordered_report(read(out / 'PE-dependencies.json'))
assert sha(out / 'PE-dependencies.json') == r['PE_dependency_report_sha256']
assert inspected['machine'] == 'AMD64' and inspected['optional_header'] == 'PE32+'
assert not inspected['clean_machine_verified'] and not inspected['portable_package_accepted']
assert r['runtime_PATH'].lower().split(';') == [r'c:\windows\system32', r'c:\windows']
(out / 'independent-qualifier.py').write_bytes(Path(__file__).read_bytes())
(out / 'dependency-inspector.py').write_bytes(tool.read_bytes())
files = {p.name: p for p in out.iterdir() if p.is_file() and p.suffix.lower() != '.zip'}
hashes = {name: sha(p) for name, p in files.items()}
archive = out / 'raw-readback.zip'
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(archive) as z:
    assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
    for name, digest in hashes.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
q = dict(status='independently_readback_verified_current_host_Release_version_and_missing_resource_CLI_only',
    executable_sha256=r['normal_Release_sha256'], raw_archive_sha256=sha(archive), raw_files=hashes,
    original_checks_rerun=False, clean_machine_verified=False, portable_package_accepted=False,
    GUI_verified=False, whole_goal_complete=False, observed_utc=datetime.now(timezone.utc).isoformat())
(out / 'independent-readback.json').write_text(json.dumps(q, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps({k: q[k] for k in ['status', 'clean_machine_verified', 'portable_package_accepted']}, ensure_ascii=False))
