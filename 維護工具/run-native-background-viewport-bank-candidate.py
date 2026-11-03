"""Validate an isolated modern background extent candidate against the pinned Debug library."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib, json, os, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
BUILD = ROOT / '驗證紀錄/current/native-terrain-archive-retention/validation-20261016-220154-787254/Debug'
RUST = Path(r'C:\Users\CHAOREN\.rustup\toolchains\stable-x86_64-pc-windows-msvc\bin')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

q = read(BUILD / 'profile-qualified/qualified.json')
assert q['source_files'] == 1024 and q['counts']['failed'] == 0
pins = read(BUILD.parent / 'compiled-inputs.json')
assert len(pins) == 1025
for name, digest in pins.items():
    assert sha(ROOT / name) == digest
deps = PROJECT / 'target/current/x86_64-pc-windows-msvc/debug/deps'
libraries = {}
native_paths = set()
for line in (BUILD / 'build/stdout.log').read_text(encoding='utf-8-sig').splitlines():
    if not line.startswith('{'):
        continue
    message = json.loads(line)
    if message.get('reason') == 'build-script-executed':
        native_paths.update(message.get('linked_paths', []))
    if message.get('reason') == 'compiler-artifact' and message['target']['name'] in ['lito_modern', 'serde_json', 'sha2', 'anyhow']:
        paths = [Path(p) for p in message['filenames'] if p.endswith('.rlib') and Path(p).parent == deps]
        if paths:
            assert len(paths) == 1
            if message['target']['name'] == 'lito_modern':
                assert message['features'] == []
            libraries[message['target']['name']] = paths[0]
assert set(libraries) == {'lito_modern', 'serde_json', 'sha2', 'anyhow'}
stamp = datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out = ROOT / '驗證紀錄/current/native-background-viewport-bank-candidate' / stamp
runtime = PROJECT / 'target/current/native-background-viewport-bank-candidate' / stamp
out.mkdir(parents=True)
runtime.mkdir(parents=True)
source = ROOT / '維護工具/oracle/NativeBackgroundViewportBankProbe.rs'
(out / 'probe-template.rs').write_bytes(source.read_bytes())
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
candidate_directory = ROOT / '維護工具/candidates/native-background-viewport'
candidate_files = {name: sha(candidate_directory / name) for name in ['native_background_geometry.rs', 'native_background.rs', 'metadata.json']}
for name in candidate_files:
    (out / ('candidate-' + name)).write_bytes((candidate_directory / name).read_bytes())
# Keep the exact existing struct/paint methods; omit only the unchanged loader
# that cannot reference its crate-private NativeSurfaceArchive from an external crate.
bank=(candidate_directory/'native_background.rs').read_text(encoding='utf-8-sig')
start=bank.index('    pub fn load_map303(');body=bank.index('{',start);depth=0
for at in range(body,len(bank)):
    if bank[at]=='{':depth+=1
    elif bank[at]=='}':
        depth-=1
        if depth==0:
            end=at+1;break
extracted=bank[:start]+bank[end:]
extracted=extracted.replace('    native_resources::NativeSurfaceArchive,\n','')
extracted+='\npub fn make_probe_bank(base:OwnedImage,overlay:OwnedImage,sources:Value)->Bank {Bank{base,overlay,sources}}\n'
extract=out/'bank-method-extract.rs';extract.write_text(extracted,encoding='utf-8',newline='\n')
(out/'probe.rs').write_text(source.read_text(encoding='utf-8-sig').replace('@BANK_METHOD_EXTRACT@',extract.as_posix()),encoding='utf-8',newline='\n')
save(out/'method-extraction.json',dict(candidate_sha256=candidate_files['native_background.rs'],
    extracted_sha256=sha(extract),omitted_method='load_map303',
    removed_import='native_resources::NativeSurfaceArchive',probe_factory_added=True,
    Bank_struct_paint_and_paint_viewport_text_unchanged=True,product_source_changed=False))
for source_name, dest in [('native-background-geometry-original.json', 'geometry-fixture.json'), ('native-background-real-original.json', 'frame-fixture.json')]:
    (out / dest).write_bytes((PROJECT / 'tests/fixtures' / source_name).read_bytes())
save(out / 'compiled-inputs.json', pins)
state = dict(status='compiling_diagnostic_against_retained_current_Debug_library', output=str(out),
    runtime=str(runtime), source_files=1024, compiled_input_files=1025, source_manifest_sha256=q['source_manifest_sha256'],
    original_qualification=str(BUILD / 'profile-qualified/qualified.json'), original_qualification_sha256=sha(BUILD / 'profile-qualified/qualified.json'),
    libraries={name: dict(path=str(p), sha256=sha(p)) for name, p in libraries.items()},
    candidate_files=candidate_files, diagnostic_source_sha256=sha(source), normal_product_source_changed=False, actual_formal_sessions=0,
    normal_scene_map303_enabled=False, full_map_rendering_verified=False, GUI_verified=False,
    whole_goal_complete=False, started_utc=datetime.now(timezone.utc).isoformat(), processes=[])
persist = lambda: save(out / 'receipt.json', state)
persist()
print(json.dumps(dict(output=str(out)), ensure_ascii=False), flush=True)
binary = runtime / 'background-viewport-candidate.exe'
command = [str(RUST / 'rustc.exe'), str(out / 'probe.rs'), '--edition=2021', '--crate-name', 'native_background_viewport_bank_candidate',
    '--target', 'x86_64-pc-windows-msvc', '-L', 'dependency=' + str(deps),
    '-L', 'dependency=' + str(PROJECT / 'target/current/debug/deps'), '-o', str(binary)]
for name, path in libraries.items():
    command += ['--extern', name + '=' + str(path)]
for path in sorted(native_paths):
    command += ['-L', path]
env = os.environ.copy()
env.update(TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'),
    PYTHONDONTWRITEBYTECODE='1', PYTHONIOENCODING='utf-8')

def execute(command, name):
    save(out / (name + '-command.json'), command)
    with (out / (name + '-stdout.log')).open('wb') as stdout, (out / (name + '-stderr.log')).open('wb') as stderr:
        child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr,
            creationflags=subprocess.CREATE_NO_WINDOW)
        item = dict(pid=child.pid, command=command, step=name, started_utc=datetime.now(timezone.utc).isoformat())
        state['processes'].append(item)
        state['owned_process'] = item
        persist()
        code = child.wait()
        item.update(exit_code=code, finished_utc=datetime.now(timezone.utc).isoformat())
        state['owned_process'] = None
        persist()
    assert code == 0, (name, code, 'Retain original diagnostic failure.')

try:
    execute(command, 'compile')
    state['status'] = 'validating_isolated_background_viewport_candidate'
    persist()
    execute([str(binary), r'D:\lineage開發\0-客戶端', str(out)], 'validate')
    report = read(out / 'candidate-results.json')
    assert report['partition_cases'] == 13440 and report['native_geometry_cases'] == 2688
    assert report['native_ordered_calls'] == 17472 and len(report['frames']) == 36
    assert report['native_bank_paint_viewport_method_compiled'] and report['candidate_bank_checked_frames']==36 and report['rejected_viewports_preserve_target']==6
    for name, digest in candidate_files.items():
        assert sha(candidate_directory / name) == digest
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest
    for name, info in state['libraries'].items():
        assert sha(info['path']) == info['sha256']
    state.update(status='isolated_background_viewport_candidate_component_passed', diagnostic_sha256=sha(binary),
        report_sha256=sha(out / 'candidate-results.json'), finished_utc=datetime.now(timezone.utc).isoformat())
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    persist()
    files = {p.name: p for p in out.iterdir() if p.is_file() and p.name not in ['raw.zip', 'readback.json']}
    if binary.exists():
        files['diagnostic.exe'] = binary
    for name, path in libraries.items():
        files['libraries/' + path.name] = path
    hashes = {name: sha(path) for name, path in files.items()}
    with zipfile.ZipFile(out / 'raw.zip', 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
        for name, path in files.items():
            z.write(path, name)
    with zipfile.ZipFile(out / 'raw.zip') as z:
        assert len(z.namelist()) == len(hashes) and set(z.namelist()) == set(hashes)
        for name, digest in hashes.items():
            with z.open(name) as stream:
                assert hashlib.file_digest(stream, 'sha256').hexdigest().upper() == digest
    save(out / 'readback.json', dict(status='diagnostic_archive_independently_readback',
        archive_sha256=sha(out / 'raw.zip'), files=hashes, gameplay_or_scene_acceptance=False))
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
