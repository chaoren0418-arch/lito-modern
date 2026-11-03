"""Prepare owned GUI fixtures after current library qualification. No UI automation."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, ctypes, hashlib, json, os, shutil, subprocess, zipfile
from ctypes import wintypes

R = Path(__file__).resolve().parents[1]
P = R / 'LiTo381-Modern'
V = R / '驗證紀錄/current'
T = R / '維護工具'
SOURCE = 'AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
JAR = '75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371'
CATALOG = 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
JDK = Path(r'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin')

def read(path): return json.loads(Path(path).read_text('utf-8-sig'))
def sha(path):
    with Path(path).open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def save(path, data): Path(path).write_text(json.dumps(data, ensure_ascii=False, indent=2)+'\n', encoding='utf-8', newline='\n')

ap = argparse.ArgumentParser()
ap.add_argument('library', type=Path)
a = ap.parse_args()
library = a.library.resolve(strict=True)
assert library.is_relative_to(V / 'native-keyboard-focus-library-default')
q = read(R / '工作佇列.json')
assert not q['active_workloads'] and not q['source_and_corresponding_QA_frozen']
completion = read(library / 'completion-observation.json')
assert completion['status'] == 'independently_qualified_current1047_library_both_profiles_and_terminal_processes'
assert completion['source_manifest_sha256'] == SOURCE
assert completion['observed_terminal_exit_code'] == 0 and completion['recorded_original_owned_processes_all_terminal']
assert q['current_native_catalog_library']['run'] == str(library)
assert q['current_full_default_library_verified']
c = read(library / 'cohort.json')
inputs = read(library / 'compiled-inputs.json')
assert len(inputs) == 1048
for name, digest in inputs.items(): assert sha(R / name) == digest, name
normal = read(c['normal_qualification'])
jar = P / 'target/current/server-reference/formal-server-75DA69B0.jar'
formal = R.parent / 'lito登入器/LiTo381-Launcher/app/server/LiTo381-SharedResources-candidate.jar'
catalog = P / 'target/current/shared-profile-inputs/common_catalog-B64ED920.bin'
assert sha(jar) == sha(formal) == JAR and sha(catalog) == CATALOG
origins = [P / 'tests/oracle/NativePeerHarness.java',
           T / 'candidates/native-teleport-fixture-arm/NativeTeleportHarness.java',
           P / 'tests/oracle/NativeTeleportFailureHarness.java']
assert sha(origins[1]) == 'DCF1A5BC267C149908E61C6C144BFA0F169F7AC1D67F1D75E72BD6E6424AD869'
out = V / 'native-keyboard-focus-sky-gui' / ('prepared-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
fixture_src = out / 'fixture-src'
fixture_src.mkdir()
derivations = []
sources = []
for origin in origins:
    body = origin.read_text('utf-8-sig')
    replacements = []
    if origin.name == 'NativePeerHarness.java':
        replacements = [
            ('String profile=args.length>1?args[1]:"melee";', 'boolean gui=args.length>1&&args[1].startsWith("gui-");\n        String profile=args.length>1?args[1].replaceFirst("^gui-",""):"melee";'),
            ('login.setSoTimeout(45000);game.setSoTimeout(45000);', 'login.setSoTimeout(gui?900000:45000);game.setSoTimeout(gui?900000:45000);'),
            ('long start=System.nanoTime(),end=start+(', 'long start=System.nanoTime(),end=start+(gui?900_000_000_000L:')]
    elif origin.name == 'NativeTeleportHarness.java':
        replacements = [
            ('boolean same=args.length>1 && args[1].equals("same-endpoint");', 'boolean same=args.length>1 && (args[1].equals("same-endpoint")||args[1].equals("gui-same-endpoint"));'),
            ('boolean gui=args.length>1 && args[1].equals("gui-cross-endpoint");', 'boolean gui=args.length>1 && args[1].startsWith("gui-");'),
            ('socket.setSoTimeout(gui?240000:60000);', 'socket.setSoTimeout(gui?900000:60000);'),
            ('(gui?300_000_000_000L:120_000_000_000L)', '(gui?1_800_000_000_000L:120_000_000_000L)')]
    elif origin.name == 'NativeTeleportFailureHarness.java':
        replacements = [
            ('unavailable.setSoTimeout(90000);', 'unavailable.setSoTimeout(gui?900000:90000);'),
            ('login.setSoTimeout((cancelExpiry||gui)?180000:90000);town.setSoTimeout((cancelExpiry||gui)?180000:90000);',
             'login.setSoTimeout(gui?900000:cancelExpiry?180000:90000);town.setSoTimeout(gui?900000:cancelExpiry?180000:90000);'),
            ('((cancelExpiry||gui)?320_000_000_000L:200_000_000_000L)', '(gui?1_800_000_000_000L:cancelExpiry?320_000_000_000L:200_000_000_000L)')]
    for before, after in replacements:
        assert body.count(before) == 1, (origin, before)
        body = body.replace(before, after)
    target = fixture_src / origin.name
    target.write_text(body, encoding='utf-8', newline='\n')
    sources.append(target)
    derivations.append(dict(origin=str(origin), origin_sha256=sha(origin), compiled_source=str(target),
                            compiled_source_sha256=sha(target), replacements=replacements,
                            gameplay_authority_or_clock_changes=False,
                            GUI_operator_orchestration_waits_changed=True,
                            application_protocol_deadlines_changed=False,
                            scope='Functional GUI operator waiting only; native 8s readiness/movement, peer 60s, gameplay expiry and G11 budgets unchanged. No performance qualification.'))
classes = out / 'classes'
classes.mkdir()
env = os.environ.copy()
env.update(TEMP=str(P / 'target/current/scratch'), TMP=str(P / 'target/current/scratch'))
command = [str(JDK / 'javac.exe'), '-J-Dfile.encoding=UTF-8', '-encoding', 'UTF-8', '-cp', str(jar), '-d', str(classes)] + [str(s) for s in sources]
with (out / 'javac.stdout').open('wb') as stdout, (out / 'javac.stderr').open('wb') as stderr:
    compiler = subprocess.Popen(command, cwd=out, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
    kernel = ctypes.WinDLL('kernel32', use_last_error=True)
    kernel.GetProcessTimes.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
    ts = [wintypes.FILETIME() for _ in range(4)]
    assert kernel.GetProcessTimes(wintypes.HANDLE(int(compiler._handle)), *[ctypes.byref(t) for t in ts])
    ticks = ts[0].dwLowDateTime | ts[0].dwHighDateTime << 32
    compiler_birth = (datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks//10)).isoformat()
    code = compiler.wait()
assert code == 0, 'Fixture compiler failure retained; no GUI launched.'
products = {}
for profile in ['Debug', 'Release']:
    original = normal['profiles'][profile]['normal']
    assert sha(original['path']) == c['normal_mains'][profile] == original['sha256']
    products[profile] = {}
    for role in ['a', 'b']:
        target = out / profile / role / ('LiToModern-GUI-' + role + '.exe')
        target.parent.mkdir(parents=True)
        shutil.copyfile(original['path'], target)
        assert sha(target) == original['sha256']
        products[profile][role] = dict(path=str(target), sha256=sha(target), original_path=original['path'])
for name, digest in inputs.items(): assert sha(R / name) == digest, name
save(out / 'prepared.json', dict(status='prepared_current_normal_gui_fixture_no_runtime_or_UI_yet',
     source_manifest_sha256=SOURCE, library=str(library), library_completion_sha256=sha(library / 'completion-observation.json'),
     normal_qualification=c['normal_qualification'], normal_qualification_sha256=sha(c['normal_qualification']),
     compiled_inputs=inputs, jar=str(jar), jar_sha256=JAR, catalog=str(catalog), catalog_sha256=CATALOG,
     classes=str(classes), class_files={str(x.relative_to(classes)): sha(x) for x in classes.rglob('*.class')},
     fixture_sources={str(s): sha(s) for s in sources}, fixture_origins={str(s): sha(s) for s in origins},
     fixture_derivations=derivations, fixture_compiler_exit_code=code, fixture_compiler_pid=compiler.pid,
     fixture_compiler_creation_utc=compiler_birth, fixture_compiler_original_handle_signalled=compiler.poll() is not None,
     products=products, product_source_modified=False, GUI_verified=False, whole_goal_complete=False,
     operator_wait_policy=dict(accept_seconds=900,rental_overall_seconds=900,transfer_failure_overall_seconds=1800,
         cleanup_seconds=30,application_protocol_deadlines_changed=False,gameplay_expiry_clocks_changed=False,
         performance_qualification_granted=False,original_failed_case=str(V/'native-keyboard-focus-sky-gui/case-20261019-153432-581147-debug-rental-entry/failure-readback.json')),
     scenarios=['rental-entry', 'cross-endpoint', 'same-endpoint', 'timeout', 'refused', 'wrong-service', 'cancel-retry', 'cancel-late', 'cancel-expiry', 'rental-hotel', 'rental-hotel-entry', 'rental-hotel-middle-entry', 'rental-hotel-share'],
     further_required_scopes=['IME and multiple DPI', 'normal audio device', 'natural hotel expiry and voluntary guest exit']))
print(json.dumps(dict(prepared=str(out), products=products, GUI_verified=False), ensure_ascii=False))
