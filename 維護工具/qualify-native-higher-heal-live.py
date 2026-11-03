"""Bind a completed higher-heal cohort to exact products, source and retained raw data."""
from pathlib import Path
import hashlib
import json
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()


out = Path(sys.argv[1]).resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-shared-higher-heals/live')
assert not (out / 'qualified.json').exists()
state = read(out / 'pipeline.json')
assert state['status'] == 'completed_pending_independent_qualification'
normal = read(state['normal_qualification'])
assert sha(state['normal_qualification']) == state['normal_qualification_sha256']
pins = read(out / 'source-inputs.json')
assert len(pins) == 968 and sha(out / 'source-inputs.json') == state['source_manifest_sha256'] == normal['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
for name, digest in state['tool_pins'].items():
    assert sha(out / name) == sha(ROOT / '維護工具' / name) == digest
assert len(state['cases']) == 15 and state['actual_higher_heal_sessions_observed'] == 60
expected = set()
for skill in [19, 35, 57]:
    for suffix in ['', '-full', '-refused']:
        expected.add(('Release', 'tw13081901-v1', f'heal{skill}{suffix}'))
    expected.add(('Release', 'lito-legacy', f'heal{skill}'))
    expected.add(('Debug', 'tw13081901-v1', f'heal{skill}'))
assert {(c['build_profile'], c['resource_profile'], c['action_profile']) for c in state['cases']} == expected
normal_archive = Path(normal['archive']['path'])
assert sha(normal_archive) == normal['archive']['sha256'] and normal['archive']['readback_verified']
with zipfile.ZipFile(normal_archive) as a:
    for profile in ['Debug', 'Release']:
        with a.open(profile + '/LiToModern.exe') as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == normal['profiles'][profile]['sha256']
files = {'qualifier.py': Path(__file__)}
for p in out.iterdir():
    if p.is_file():
        files['cohort/' + p.name] = p
checks = []
png_count = 0
for index, c in enumerate(state['cases']):
    assert c['exit_code'] == 0 and c['actual_client_sessions'] == 4 and c['no_heavy_job_before']
    case = Path(c['case']).resolve(strict=True)
    assert case.is_relative_to(ROOT / '驗證紀錄/current/native-real-peers')
    r = read(case / 'receipt.json')
    assert r['action_profile'] == c['action_profile'] and r['resource_profile'] == c['resource_profile']
    assert r['exe_sha256'] == normal['profiles'][c['build_profile']]['sha256']
    comparison = read(case / 'authority-comparison.json')
    assert comparison['status'] == 'verified_real_peer_action_authority'
    assert comparison['action_profile'] == c['action_profile'] and comparison['resource_profile'] == c['resource_profile']
    assert comparison['C14_requests'] == 1 and comparison['S37_copies'] == (0 if c['action_profile'].endswith('-refused') else 2)
    assert comparison['receipt_sha256'] == sha(case / 'receipt.json')
    marker = read(case / 'validation-completed.json')
    assert marker['receipt_sha256'] == sha(case / 'receipt.json') and not marker['runtime_receipt_rewritten']
    for name, digest in marker['comparison_pins'].items():
        assert sha(case / name) == digest
    runtime = Path(r['runtime']).resolve(strict=True)
    assert runtime.is_relative_to(ROOT / 'LiTo381-Modern/target/current')
    assert sha(runtime / 'qualified-client.exe') == r['exe_sha256']
    assert sha(c['log']) == c['log_sha256']
    for label, folder in [('case', case), ('runtime', runtime)]:
        for p in folder.rglob('*'):
            if p.is_file():
                assert p.resolve().is_relative_to(folder) and not p.is_symlink()
                if p.name == 'qualified-client.exe':
                    continue  # Already retained and read back in the normal product archive.
                files[f'{index:02d}/{label}/' + p.relative_to(folder).as_posix()] = p
    png_count += sum(p.suffix.lower() == '.png' for p in runtime.rglob('*') if p.is_file())
    checks.append({'case': str(case), 'profile': c['build_profile'], 'resource_profile': c['resource_profile'], 'action_profile': c['action_profile'], 'receipt_sha256': sha(case / 'receipt.json'), 'authority_comparison_sha256': sha(case / 'authority-comparison.json'), 'validation_marker_sha256': sha(case / 'validation-completed.json'), 'actual_client_sessions': 4})
archive = out / 'raw-evidence.zip'
file_pins = {name: sha(path) for name, path in files.items()}
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as a:
    for name, path in files.items():
        a.write(path, name)
with zipfile.ZipFile(archive) as a:
    assert set(a.namelist()) == set(file_pins)
    for name, digest in file_pins.items():
        with a.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest, name
for name, digest in pins.items():
    assert sha(ROOT / name) == digest
q = {'status': 'qualified15_actual_higher_heal_controller_scenarios', 'source_files': 968, 'source_manifest_sha256': state['source_manifest_sha256'], 'normal_qualification': state['normal_qualification'], 'normal_qualification_sha256': state['normal_qualification_sha256'], 'normal_mains': {k: v['sha256'] for k, v in normal['profiles'].items()}, 'cases': checks, 'actual_client_sessions': 60, 'LOGIN_sessions': 30, 'GAME_sessions': 30, 'C14_requests': 15, 'successful_casts': 12, 'MP2_refusals': 3, 'S37_copies': 24, 'runtime_PNG_count': png_count, 'runtime_authority_mutations': 0, 'archive_file_pins': file_pins, 'archive_sha256': sha(archive), 'archive_readback_verified': True, 'normal_product_archive': str(normal_archive), 'normal_product_archive_sha256': sha(normal_archive), 'actual_menu_GUI_verified': False, 'physical_audio_verified': False, 'full_world_pixel_oracle': False, 'full_default90_verified': False, 'full_library_verified': False, 'whole_goal_complete': False}
q['cast_target_scope']='self, original player3';q['other_player_or_NPC_healing_verified']=False;q['prior_failed_or_incomplete_scopes_observed_sessions']=20;q['prior_failed_scopes_included_in_qualified60']=False
(out / 'qualified.json').write_text(json.dumps(q, ensure_ascii=False, indent=2)+'\n', 'utf-8')
print(json.dumps({k: q[k] for k in ['status', 'actual_client_sessions', 'C14_requests', 'S37_copies', 'runtime_PNG_count', 'archive_readback_verified']}, ensure_ascii=False))
