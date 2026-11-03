"""Current retained library images; five audited original-resource bodies only."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
from ctypes import wintypes
import argparse, ctypes, hashlib, json, os, re, shutil, subprocess, zipfile

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))
now = lambda: datetime.now(timezone.utc).isoformat()
TESTS = [
    'modern_lito::actor_assets::tests::real_explicit_sprite_bank_binds_eight_headings_and_unique_scope_rejects_collision',
    'modern_lito::animation::tests::real_shared_attack_uses_source_fps_ticks_and_once_only_presentation_events',
    'modern_lito::ammo_resource_reference_tests::actual_shared_bow_bodies_match_independent_pixels_and_launch_cues',
    'modern_lito::ammo_resource_reference_tests::actual_shared_arrow_eight_banks_match_independent_reference',
    'modern_lito::audio::tests::real_shared_sound_1539_matches_pcm_reference',
]
SCENE_TESTS = [
    'modern_lito::world_scene::tests::real_honor_effect_roof_cache_matches_csharp_and_counts_full_budget',
    'modern_lito::world_scene::tests::real_shared_armed_families_preload_and_knight_pixels_match_independent_csharp_goldens',
    'modern_lito::world_scene::tests::real_shared_combat_assets_match_qualified_one_shots_and_render_attack_hurt_death',
    'modern_lito::world_scene::tests::real_shared_roofs_fade_and_restore_without_archive_reads_during_render',
    'modern_lito::world_scene::tests::real_shared_world_all_base_appearances_decode_and_authoritative_camera_moves',
]

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()

def save(p, value):
    p.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')

def verify_normal_reference(path, compiled):
    path=Path(path).resolve(strict=True)
    normal=read(path)
    assert normal['status'] in {
        'independently_qualified_retained_archive_both_profiles',
        'independently_qualified_current_normal_selector_both_profiles',
    }
    if normal['status']=='independently_qualified_current_normal_selector_both_profiles':
        assert path.parent.is_relative_to(ROOT/'驗證紀錄/current/native-selector-integration')
        assert normal['observed_terminal_exit_code']==0 and normal['recorded_owned_processes_all_terminal']
        assert set(normal['profiles'])=={'Debug','Release'}
        assert len(compiled)==1029 and read(path.parent/'compiled-inputs.json')==compiled
        source=path.parent/'source-inputs.json'
        assert len(read(source))==1028 and sha(source)==normal['source_manifest_sha256']
        assert sha(ROOT/'啟動現代客戶端.ps1')==normal['entry_sha256']
        cohort=read(path.parent/'cohort.json')
        for profile,info in normal['profiles'].items():
            proof=Path(info['qualification']);assert sha(proof)==info['qualification_sha256']
            q=read(proof)
            assert q['status']=='independently_qualified_current_normal_selector_related_profile'
            assert q['source_manifest_sha256']==normal['source_manifest_sha256'] and q['counts']['failed']==0
            assert q['actual_test_executables']==29 and q['normal_sha256']==info['normal_sha256']
            assert sha(cohort['profiles'][profile]['normal']['path'])==info['normal_sha256']
    return normal

ap = argparse.ArgumentParser()
ap.add_argument('--library-run', required=True, type=Path)
ap.add_argument('--scope', choices=['base', 'scene'], default='base')
ap.add_argument('--retained-images-from', type=Path)
ap.add_argument('--library-kind', choices=['legacy-corrected', 'startup-current'], default='legacy-corrected')
a = ap.parse_args()
if a.scope == 'scene':
    TESTS = SCENE_TESTS
library = a.library_run.resolve(strict=True)
pins = read(library / 'source-inputs.json')
compiled_pins = None
if a.library_kind == 'startup-current':
    assert library.is_relative_to(ROOT / '驗證紀錄/current/native-startup-library-default')
    library_qualification = library / 'completion-observation.json'
    q = read(library_qualification)
    assert q['status'] == 'independently_qualified_current_full_library_both_profiles_and_terminal_original_runner'
    assert len(pins) == q['source_files'] == 1028 and q['observed_terminal_exit_code'] == 0
    compiled_pins = read(library / 'compiled-inputs.json')
    assert len(compiled_pins) == q['compiled_input_files'] == 1029
    library_cohort = read(library / 'cohort.json')
    normal_qualification = Path(library_cohort['normal_qualification'])
    normal_q = verify_normal_reference(normal_qualification, compiled_pins)
    assert normal_q['source_manifest_sha256'] == q['source_manifest_sha256'] == sha(library / 'source-inputs.json')
    normal = dict(profiles={profile:dict(path=str(PROJECT/'target/current/x86_64-pc-windows-msvc'/profile.lower()/'LiToModern.exe'),sha256=value)
        for profile,value in q['normal_mains'].items()})
    for profile,value in normal['profiles'].items():assert value['sha256'] == normal_q['profiles'][profile]['normal_sha256']
else:
    assert library.is_relative_to(ROOT / '驗證紀錄/current/native-teleport-wire')
    library_qualification = library / 'default-library-qualified.json'
    q = read(library_qualification)
    assert q['status'] == 'independently_qualified_corrected_both_default_library_profiles'
    assert len(pins) == q['source_files'] == 1021
    normal_qualification = library / 'normal-invariance.json'
    normal = read(normal_qualification)
original = Path(r'D:\lineage開發\0-客戶端').resolve(strict=True)
assert not original.is_relative_to(ROOT)
inputs = [original / 'Sound/1539.wav', original / 'Text.idx', original / 'Text.pak'] + sorted(original.glob('Sprite*.idx'))
if a.scope == 'scene':
    inputs += [original / 'Tile.idx', original / 'Tile.pak']
    inputs += sorted((original / 'Sound').glob('*.wav'))
    for map_id in ['514', '532']:
        inputs += sorted((original / 'map' / map_id).glob('*.s32'))
input_pins = {str(p): sha(p) for p in inputs}
pak_metadata = {str(p): dict(bytes=p.stat().st_size, mtime_ns=p.stat().st_mtime_ns) for p in original.glob('Sprite*.pak')}
assert pak_metadata and len(inputs) > 3
legacy = PROJECT / 'logs/continuation-20261003'
historical = {str(p.relative_to(legacy)).replace('\\', '/'): sha(p) for p in legacy.glob('*.json')}
out = ROOT / '驗證紀錄/current/native-resource-optin' / ('current'+str(len(pins))+'-' + a.scope + '-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
runtime = PROJECT / 'target/current/native-resource-optin' / out.name
out.mkdir(parents=True)
runtime.mkdir(parents=True)
(out / 'runner.py').write_bytes(Path(__file__).read_bytes())
(out / 'source-inputs.json').write_bytes((library / 'source-inputs.json').read_bytes())
if compiled_pins:(out/'compiled-inputs.json').write_bytes((library/'compiled-inputs.json').read_bytes())
save(out / 'original-input-pins.json', input_pins)
save(out / 'original-PAK-metadata.json', pak_metadata)
save(out / 'historical-report-pins.json', historical)
state = dict(status='prepared_current_five_original_resource_bodies', output=str(out), runtime=str(runtime),
    source_files=len(pins), source_manifest_sha256=sha(out / 'source-inputs.json'),
    library_kind=a.library_kind, library_qualification=str(library_qualification), library_qualification_sha256=sha(library_qualification),
    normal_qualification=str(normal_qualification), normal_qualification_sha256=sha(normal_qualification),
    normal_invariance_proof=str(normal_qualification) if a.library_kind=='legacy-corrected' else None, normal_profiles=normal['profiles'],
    tests=TESTS, original_root=str(original), original_access='read_only',
    original_IDX_Text_WAV_pins_sha256=sha(out / 'original-input-pins.json'), original_PAK_metadata_sha256=sha(out / 'original-PAK-metadata.json'),
    historical_reports=len(historical), profiles={}, owned_process=None, process_history=[],
    actual_formal_sessions=0, GUI_verified=False, audio_device_verified=False, all_ignored_bodies_verified=False,
    whole_goal_complete=False, started_utc=now())
state['resource_scope'] = a.scope
reused = None
if a.retained_images_from:
    prior = a.retained_images_from.resolve(strict=True)
    assert prior.is_relative_to(ROOT / '驗證紀錄/current/native-resource-optin')
    prior_q = read(prior / 'qualified/qualified.json')
    assert prior_q['status'] == 'independently_qualified_current_both_profiles_five_original_resource_bodies'
    assert prior_q['source_manifest_sha256'] == state['source_manifest_sha256']
    reused = read(prior / 'cohort.json')
    state['reused_immutable_images_from'] = str(prior)
if a.scope == 'scene':
    state['unexecuted_sixth_scene_precondition'] = dict(
        test='modern_lito::world_scene::tests::real_disintegrate_layers_share_world_depth_and_survive_target_removal',
        required_root_pack=str(original / 'TW13081901.pak'),
        root_pack_present=(original / 'TW13081901.pak').is_file(),
        nested_launcher_pack_not_substituted=True,
        qualified=False)
    assert not state['unexecuted_sixth_scene_precondition']['root_pack_present']
persist = lambda: save(out / 'cohort.json', state)
env = {k: v for k, v in os.environ.items() if not k.startswith('LITO_')}
env.update(LITO_SHARED_RESOURCE_ROOT=str(original), TEMP=str(PROJECT / 'target/current/scratch'), TMP=str(PROJECT / 'target/current/scratch'))

def check():
    for name, digest in pins.items():
        assert sha(ROOT / name) == digest, name
    if compiled_pins:
        for name,digest in compiled_pins.items():assert sha(ROOT/name)==digest,name
    for name, digest in input_pins.items():
        assert sha(name) == digest, name
    for name, item in pak_metadata.items():
        p = Path(name)
        assert (p.stat().st_size, p.stat().st_mtime_ns) == (item['bytes'], item['mtime_ns']), name
    for p in normal['profiles'].values():
        assert sha(p['path']) == p['sha256']
    assert sha(library_qualification)==state['library_qualification_sha256']
    assert sha(normal_qualification)==state['normal_qualification_sha256']

def execute(command, directory):
    directory.mkdir()
    with (directory / 'stdout.log').open('wb') as stdout, (directory / 'stderr.log').open('wb') as stderr:
        child = subprocess.Popen(command, cwd=PROJECT, env=env, stdout=stdout, stderr=stderr, creationflags=subprocess.CREATE_NO_WINDOW)
        times = [wintypes.FILETIME() for _ in range(4)]
        call = ctypes.WinDLL('kernel32', use_last_error=True).GetProcessTimes
        call.argtypes = [wintypes.HANDLE] + [ctypes.POINTER(wintypes.FILETIME)] * 4
        call.restype = wintypes.BOOL
        assert call(wintypes.HANDLE(int(child._handle)), *[ctypes.byref(t) for t in times])
        ticks = times[0].dwLowDateTime | times[0].dwHighDateTime << 32
        created = datetime(1601, 1, 1, tzinfo=timezone.utc) + timedelta(microseconds=ticks // 10)
        item = dict(pid=child.pid, creation_utc=created.isoformat(), command=command, log_directory=str(directory))
        state['owned_process'] = item
        state['process_history'].append(item)
        persist()
        code = child.wait()
        item.update(exit_code=code, finished_utc=now())
        state['owned_process'] = None
        persist()
    return code

persist()
print(json.dumps(dict(output=str(out), status=state['status']), ensure_ascii=False), flush=True)
try:
    for profile in ['Debug', 'Release']:
        check()
        d = out / profile
        d.mkdir()
        product = read(library / profile / 'product-retention.json')
        inventory = read(library / profile / 'inventory-audit/inventory.json')
        assert inventory['total_tests'] == 2410 and inventory['ignored_tests'] == 473
        assert set(TESTS) <= set(inventory['ignored_test_names'])
        assert sha(product['archive']) == product['archive_sha256']
        if reused:
            executable = Path(reused['profiles'][profile]['immutable_test_exe']).resolve(strict=True)
            assert executable.is_relative_to(PROJECT / 'target/current/native-resource-optin')
        else:
            executable = runtime / (profile + '-library-tests.exe')
            with zipfile.ZipFile(product['archive']) as z, z.open('lito_modern.exe') as source, executable.open('xb') as target:
                shutil.copyfileobj(source, target)
        assert sha(executable) == product['product_sha256'] == inventory['product_sha256']
        state.update(status='running_current_five_original_resource_bodies', active_profile=profile)
        state['profiles'][profile] = dict(actual_test_sha256=product['product_sha256'], original_product_archive=product['archive'],
            original_product_archive_sha256=product['archive_sha256'], immutable_test_exe=str(executable), results=[])
        persist()
        for index, name in enumerate(TESTS):
            command = [str(executable), name, '--exact', '--ignored', '--nocapture', '--test-threads=1']
            folder = d / str(index)
            assert execute(command, folder) == 0, 'Preserve actual test failure.'
            text = (folder / 'stdout.log').read_text(encoding='utf-8-sig')
            rows = re.findall(r'^test (.+?) \.\.\. (ok|FAILED|ignored)(?:,.*)?$', text, re.M)
            terminal = re.findall(r'test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out;', text)
            assert rows == [(name, 'ok')] and terminal == [('ok', '1', '0', '0', '0', '2409')]
            assert sha(executable) == product['product_sha256']
            state['profiles'][profile]['results'].append(dict(name=name, passed=1, failed=0, ignored=0, filtered=2409,
                stdout_sha256=sha(folder / 'stdout.log'), stderr_sha256=sha(folder / 'stderr.log')))
            persist()
        check()
        assert historical == {str(p.relative_to(legacy)).replace('\\', '/'): sha(p) for p in legacy.glob('*.json')}
        print(profile + ' five explicit original-resource bodies passed', flush=True)
    state.update(status='executed_current_both_profiles_five_original_resource_bodies', active_profile=None)
except BaseException as error:
    state.update(status='failed_preserved', error=repr(error))
    raise
finally:
    state['finished_utc'] = now()
    persist()
print(json.dumps(dict(status=state['status'], output=str(out)), ensure_ascii=False), flush=True)
