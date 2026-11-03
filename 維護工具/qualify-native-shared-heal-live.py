"""Bind actual shared-profile heal traffic, authority and original effect observations."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import zipfile
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
PROJECT = ROOT / 'LiTo381-Modern'
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))


def sha(path):
    with Path(path).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()


ap = argparse.ArgumentParser()
ap.add_argument('observation', type=Path)
a = ap.parse_args()
out = a.observation.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-shared-profile-baseline')
assert not (out / 'shared-heal-qualified.json').exists()
o = read(out / 'observation.json')
assert o['status'] == 'shared_profile_runtime_observed_pending_failure_or_success_review' and o['peer_exit_code'] == 0
assert o['sources_unchanged'] and o['no_heavy_job_before'] and o['no_heavy_job_after']
normal = read(o['normal_qualification'])
assert sha(o['normal_qualification']) == o['normal_qualification_sha256']
pins = read(out / 'source-inputs.json')
assert len(pins) == 968 and sha(out / 'source-inputs.json') == o['source_manifest_sha256'] == normal['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
case = Path(o['case']).resolve(strict=True)
assert case.is_relative_to(ROOT / '驗證紀錄/current/native-real-peers')
r = read(case / 'receipt.json')
launch = read(case / 'server-launch-identity.json')
assert r['resource_profile'] == launch['resource_profile'] == o['resource_profile'] == 'tw13081901-v1'
assert r['extended_item_details'] is launch['extended_item_details'] is o['extended_item_details'] is True
assert r['common_catalog_sha256'] == launch['common_catalog_sha256'] == o['common_catalog_sha256'] == 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
assert '-Dlito.resourceProfile=tw13081901-v1' in launch['command_line'] and '-Dlito.extendedItemDetails=true' in launch['command_line']
assert r['exe_sha256'] == launch['exe_sha256'] == o['main_sha256'] == normal['profiles'][o['profile']]['sha256']
assert r['jar_sha256'] == launch['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert r['source_matches_snapshot'] and r['source_sha256_at_end'] == r['jar_sha256']
comparison = read(case / 'authority-comparison.json')
assert comparison['status'] == 'verified_real_peer_action_authority' and comparison['receipt_sha256'].upper() == sha(case / 'receipt.json')
marker = read(case / 'validation-completed.json')
assert marker['receipt_sha256'] == sha(case / 'receipt.json') and not marker['runtime_receipt_rewritten']
for name, digest in marker['comparison_pins'].items():
    assert sha(case / name) == digest
server = r['server']
assert server['status'] == 'sessions_closed' and server['actual_client_sessions'] == 4 and server['game_sessions'] == 2
assert server['memberships_empty'] and server['authority_mutations'] == 0 and server['classic_world_members_after_disconnect'] == 0
assert not server['sql_access'] and not server['state_persistent']
runtime = Path(r['runtime']).resolve(strict=True)
assert runtime.is_relative_to(PROJECT / 'target/current') and sha(runtime / 'qualified-client.exe') == r['exe_sha256']
captures = list((runtime / 'captures').glob('*/frames.jsonl'))
assert len(captures) == 1
rows = [json.loads(l) for l in captures[0].read_text(encoding='utf-8-sig').splitlines()]
assert sum(x['direction'] == 'server_to_client' and x.get('note') == 'S_KEY' for x in rows) == 4
requests = effects = 0
for row in rows:
    if not re.fullmatch('[0-9A-Fa-f ]*', row.get('plain', '')):
        continue
    payload = bytes.fromhex(row['plain'])
    requests += row['direction'] == 'client_to_server' and payload == bytes.fromhex('14000003000000')
    effects += row['direction'] == 'server_to_client' and payload == bytes.fromhex('3703000000ca2d')
assert requests == 1 and effects == 2
observations = {}
png_count = 0
for role in ['a', 'b']:
    c = r['clients'][role]
    assert c['worker_quiescent'] and c['shutdown'] == 'closed' and int(c['transport']['epoch']) == 2
    probe = c['action_probe']
    assert [e['graphics'] for e in probe['endpoint']['world']['effects']] == [11722]
    visible = set()
    sounds = 0
    saw_terminal_or_retired = False
    for sample in probe['frames']:
        fx = sample['effect']
        assert fx['load_error'] is None and fx['omitted'] == 0
        if fx['started'] == 1:
            source = next(x for x in fx['sources'] if x['name'] == '11722-0.spr')
            assert source['bytes'] == 30637 and source['sha256'] == '85f6872052fa335cab1ee565c3e184e2d4e718a6137c87d28850c88ecca6f8e5'
            assert fx['tw_catalog']['decoded_sha256'] == '7ebe2b464c8b1ea9c58ac985dd8ff4c823784d8231894262eddcd7691ca94008'
            saw_terminal_or_retired |= fx['active'] == 0
        for frame in fx['effects']:
            assert frame['graphics'] == 11722 and frame['sprite'] == '11722-0.spr'
            sub = frame['subframe']
            assert 0 <= sub <= 20 and frame['drawn'] == (not frame['finished'] and 1 <= sub <= 12)
            if frame['drawn']:
                visible.add(sub)
        for event in fx['entered_events']:
            assert event['sound'] == 109
            sounds += 1
    assert visible == set(range(1, 13)) and sounds == 1 and saw_terminal_or_retired, (role, visible, sounds)
    for image in probe['images']:
        # Windows canonicalize() emits the extended local-drive prefix.  It
        # names the same file; normalize only that prefix before containment.
        image_path = image['png']
        if image_path.startswith('\\\\?\\'):
            image_path = image_path[4:]
        p = Path(image_path).resolve(strict=True)
        assert p.is_relative_to(runtime)
        with Image.open(p) as im:
            assert im.mode == 'RGBA' and im.size == (800, 600)
            reported = image['frame'].get('rgba_sha256')
            if reported:
                assert hashlib.sha256(im.tobytes()).hexdigest().upper() == reported.upper()
        png_count += 1
    observations[role] = dict(original_visible_subframes=sorted(visible), sound109_once=True, effect_retirement_observed=True)
files = {'qualifier.py': Path(__file__)}
for prefix, folder in [('observation', out), ('case', case), ('runtime', runtime)]:
    for p in folder.rglob('*'):
        if p.is_file():
            assert p.resolve().is_relative_to(folder) and not p.is_symlink()
            files[prefix + '/' + p.relative_to(folder).as_posix()] = p
file_pins = {name: sha(path) for name, path in files.items()}
archive = out / 'shared-heal-raw-evidence.zip'
assert not archive.exists()
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, path in files.items():
        z.write(path, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(file_pins)
    for name, digest in file_pins.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
proof = dict(status='qualified_actual_shared_profile_basic_heal_first_defined_scope',
             main_sha256=r['exe_sha256'], jar_sha256=r['jar_sha256'], source_manifest_sha256=o['source_manifest_sha256'],
             resource_profile='tw13081901-v1', extended_item_details=True, actual_ClientSessions=4,
             LOGIN_sessions=2, GAME_sessions=2, C14_requests=1, S37_gfx11722_copies=2,
             observations=observations, retained_PNGs=png_count, server_authority_comparison_sha256=sha(case / 'authority-comparison.json'),
             world_framebuffer_independent_pixel_oracle=False, GUI_verified=False, physical_audio_verified=False,
             all_shared_profile_gameplay_verified=False, G06_cross_endpoint_GUI_verified=False, whole_goal_complete=False,
             raw_file_pins=file_pins, archive_sha256=sha(archive), archive_readback_verified=True)
(out / 'shared-heal-qualified.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(dict(status=proof['status'], actual_ClientSessions=4, retained_PNGs=png_count)))
