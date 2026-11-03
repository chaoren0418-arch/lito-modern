"""Retain a real failed shared-profile presentation without relabeling it as success."""
from pathlib import Path
import argparse
import hashlib
import json
import re
import zipfile

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
assert not (out / 'failure-qualified.json').exists()
observation = read(out / 'observation.json')
assert observation['status'] == 'shared_profile_runtime_observed_pending_failure_or_success_review'
assert observation['peer_exit_code'] != 0 and observation['sources_unchanged']
assert observation['no_heavy_job_before'] and observation['no_heavy_job_after']
case = Path(observation['case']).resolve(strict=True)
assert case.is_relative_to(ROOT / '驗證紀錄/current/native-real-peers')
launch = read(case / 'server-launch-identity.json')
assert launch['resource_profile'] == observation['resource_profile'] == 'tw13081901-v1'
assert launch['extended_item_details'] is observation['extended_item_details'] is True
assert launch['common_catalog_sha256'] == observation['common_catalog_sha256']
assert '-Dlito.resourceProfile=tw13081901-v1' in launch['command_line'] and '-Dlito.extendedItemDetails=true' in launch['command_line']
normal = read(observation['normal_qualification'])
assert sha(observation['normal_qualification']) == observation['normal_qualification_sha256']
assert observation['main_sha256'] == launch['exe_sha256'] == normal['profiles'][observation['profile']]['sha256']
pins = read(out / 'source-inputs.json')
assert len(pins) == 968 and sha(out / 'source-inputs.json') == observation['source_manifest_sha256'] == normal['source_manifest_sha256']
for name, digest in pins.items():
    assert sha(ROOT / name) == digest, name
error = read(case / 'b-client.json')
assert error['status'] == 'failed' and error['error'].startswith('action resource missing: ')
effect = json.loads(error['error'].split(': ', 1)[1])
assert effect['started'] == effect['active'] == 0 and effect['omitted'] == 1
assert effect['load_error'] is None and effect['effects'] == []
assert '744-0.spr' in {s['name'] for s in effect['sources']} and '11722-0.spr' not in {s['name'] for s in effect['sources']}
server = read(case / 'result.json')
assert server['status'] == 'sessions_closed' and server['actual_client_sessions'] == 4 and server['game_sessions'] == 2
assert server['memberships_empty'] and server['classic_world_members_after_disconnect'] == server['authority_mutations'] == 0
assert not server['sql_access'] and not server['state_persistent']
assert server['jar_sha256'].upper() == launch['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
runtime = PROJECT / 'target/current/native-real-peers' / case.name
assert sha(runtime / 'qualified-client.exe') == observation['main_sha256']
captures = list((runtime / 'captures').glob('*/frames.jsonl'))
assert len(captures) == 1
rows = [json.loads(l) for l in captures[0].read_text(encoding='utf-8-sig').splitlines()]
assert sum(r['direction'] == 'server_to_client' and r.get('note') == 'S_KEY' for r in rows) == 4
request = []
effects = []
for row in rows:
    if not re.fullmatch('[0-9a-fA-F ]*', row.get('plain', '')):
        continue
    payload = bytes.fromhex(row['plain'])
    if row['direction'] == 'client_to_server' and payload == bytes.fromhex('14000003000000'):
        request.append(row)
    if row['direction'] == 'server_to_client' and payload == bytes.fromhex('3703000000ca2d'):
        effects.append(row)
assert len(request) == 1 and len(effects) == 2
authority = [json.loads(l) for l in (case / 'server-trace.jsonl').read_text(encoding='utf-8-sig').splitlines()]
health = [dict(ms=r['ms'], hp=r['state']['hp'], mp=r['state']['mp']) for r in authority
          if r.get('state', {}).get('object_id') == 3 and r['state'].get('phase') == 'BATTLE']
assert any(before['mp'] == 110 and after['mp'] == 106 and after['hp'] > before['hp'] for before, after in zip(health, health[1:]))
files = {'qualifier.py': Path(__file__)}
for prefix, folder in [('observation', out), ('case', case), ('runtime', runtime)]:
    for p in folder.rglob('*'):
        if p.is_file():
            assert p.resolve().is_relative_to(folder) and not p.is_symlink()
            files[prefix + '/' + p.relative_to(folder).as_posix()] = p
for name in ['native_skill_effects.rs', 'native_effect_timing.rs', 'tw_action_catalog.rs', 'resources.rs', 'tower_assets.rs', 'native_peer_actions.rs']:
    files['current-source/' + name] = PROJECT / 'src/modern_lito' / name
archive = out / 'failed-baseline-raw.zip'
assert not archive.exists()
file_pins = {name: sha(p) for name, p in files.items()}
with zipfile.ZipFile(archive, 'x', zipfile.ZIP_DEFLATED, compresslevel=1) as z:
    for name, p in files.items():
        z.write(p, name)
with zipfile.ZipFile(archive) as z:
    assert set(z.namelist()) == set(file_pins)
    for name, digest in file_pins.items():
        with z.open(name) as f:
            assert hashlib.file_digest(f, 'sha256').hexdigest().upper() == digest
proof = dict(status='confirmed_current_shared_profile_basic_heal_effect_omitted',
             main_sha256=observation['main_sha256'], jar_sha256=launch['jar_sha256'],
             source_manifest_sha256=observation['source_manifest_sha256'],
             resource_profile='tw13081901-v1', extended_item_details=True,
             actual_ClientSessions=4, LOGIN_sessions=2, GAME_sessions=2,
             C14_heal_requests=1, S37_target3_gfx11722_copies=2,
             server_mp_consumption_observed=4, authority_health_observations=health,
             presentation=effect, server_membership_cleanup_verified=True,
             gameplay_or_GUI_success=False, both_clients_final_receipts_available=False,
             authority_injection=False, sql_access=False, raw_file_pins=file_pins,
             archive_sha256=sha(archive), archive_readback_verified=True, whole_goal_complete=False)
(out / 'failure-qualified.json').write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(dict(status=proof['status'], actual_ClientSessions=4, native_effect_started=0, native_effect_omitted=1)))
