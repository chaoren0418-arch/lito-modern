"""Compare actual higher-heal inputs, formal authority and both native render probes."""
from pathlib import Path
import hashlib
import json
import re
import sys
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
read = lambda p: json.loads(Path(p).read_text(encoding='utf-8-sig'))


def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest().upper()


p = Path(sys.argv[1]).resolve(strict=True)
assert p.is_relative_to(ROOT / '驗證紀錄/current/native-real-peers')
r = read(p)
match = re.fullmatch(r'heal(19|35|57)(-full|-refused)?', r['action_profile'])
assert match
skill = int(match[1])
branch = match[2] or '-wounded'
cost, sound, tw_gfx, legacy_gfx, last = {
    19: (15, 114, 11723, 829, 13),
    35: (20, 115, 11724, 830, 15),
    57: (48, 126, 11725, 831, 17),
}[skill]
tw = r['resource_profile'] == 'tw13081901-v1'
assert tw or r['resource_profile'] == 'lito-legacy'
gfx = tw_gfx if tw else legacy_gfx
sprite = f'{tw_gfx}-0.spr' if tw else '744-0.spr'
launch = read(p.parent / 'server-launch-identity.json')
assert launch['exe_sha256'] == r['exe_sha256']
assert launch['jar_sha256'] == r['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
assert launch['resource_profile'] == r['resource_profile']
assert launch['extended_item_details'] == r['extended_item_details']
assert r['source_matches_snapshot'] and r['source_sha256_at_end'] == r['jar_sha256']
if tw:
    assert r['extended_item_details'] is True
    assert launch['common_catalog_sha256'] == r['common_catalog_sha256'] == 'B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01'
    assert '-Dlito.resourceProfile=tw13081901-v1' in launch['command_line']
    assert '-Dlito.extendedItemDetails=true' in launch['command_line']
server = r['server']
assert server['actual_client_sessions'] == 4 and server['game_sessions'] == 2
assert server['memberships_empty'] and server['authority_mutations'] == 0
assert server['classic_world_members_after_disconnect'] == server['active_hotel_leases_after_disconnect'] == 0
assert not server['sql_access'] and not server['state_persistent']
runtime = Path(r['runtime']).resolve(strict=True)
assert runtime.is_relative_to(ROOT / 'LiTo381-Modern/target/current')
assert sha(runtime / 'qualified-client.exe') == r['exe_sha256']
capture_files = list((runtime / 'captures').glob('*/frames.jsonl'))
assert len(capture_files) == 1
wire = [json.loads(l) for l in capture_files[0].read_text('utf-8-sig').splitlines()]
request = bytes([0x14, (skill-1)//8, (skill-1)%8, 3, 0, 0, 0])
requests = []
effects = []
for row in wire:
    if not re.fullmatch('[0-9a-fA-F ]*', row.get('plain', '')):
        continue
    raw = bytes.fromhex(row['plain'])
    if row['direction'] == 'client_to_server' and raw[:1] == b'\x14':
        assert raw == request
        requests.append(row)
    if row['direction'] == 'server_to_client' and raw[:1] == b'\x37':
        assert raw == b'\x37\x03\0\0\0' + gfx.to_bytes(2, 'little')
        effects.append(row)
assert len(requests) == 1
refused = branch == '-refused'
assert len(effects) == (0 if refused else 2)
trace = [json.loads(l)['state'] for l in (p.parent / 'server-trace.jsonl').read_text('utf-8-sig').splitlines()]
actions = r['clients']['b']['action_probe']['actions']
assert len(actions) == 1
action = actions[0]
assert action['no_prediction'] and action['duplicate_pending_rejected']
before = action['before']['world']
after = action['after']['world']
assert skill in before['learned_skills']
if refused:
    assert before['mp'] == after['mp'] == [2, 2]
    assert before['hp'] == after['hp']
    assert after['own_cast_packets'] == 0 and not after['effects']
    assert action['after']['skill_feedback'] == 'NoResponse'
else:
    assert before['mp'][0] - after['mp'][0] == cost
    assert after['own_cast_packets'] == before['own_cast_packets'] + 1
    assert action['after']['skill_feedback'] == 'CastAction'
    if branch == '-full':
        assert before['hp'][0] == before['hp'][1] == after['hp'][0] == after['hp'][1]
    else:
        assert before['hp'][0] < after['hp'][0] <= after['hp'][1]
        assert r['clients']['a']['action_probe']['attacks_sent'] > 0
logs = (p.parent / 'server.log').read_text('utf-8-sig').splitlines()
commits = [l for l in logs if f'C_USE_SPELL healing skill={skill} ' in l and ' hp-after=' in l]
assert len(commits) == (0 if refused else 1)
if commits:
    line = commits[0]
    hp = int(re.search(r' hp-after=(\d+)', line)[1])
    mp = int(re.search(r' mp=(\d+)/', line)[1])
    assert int(re.search(r' mp-cost=(\d+)', line)[1]) == cost
    assert any(s['object_id'] == 3 and s['phase'] == 'BATTLE' and s['hp'] == hp and s['mp'] == mp for s in trace)
    assert any(s['hp'][0] == hp and s['mp'][0] == mp for s in r['clients']['b']['action_probe']['observed_health'])
original = read(ROOT / '驗證紀錄/current/native-shared-healing-family-SPR/inspection.json')['sprites']
wav = next(v for v in read(ROOT / '驗證紀錄/current/native-shared-healing-family-source/raw.json')['WAV_PCM_sources'] if v['sound'] == sound)
checks = []
for role, own_id in [('a', 1), ('b', 3)]:
    c = r['clients'][role]
    probe = c['action_probe']
    w = probe['endpoint']['world']
    assert c['worker_quiescent'] and c['shutdown'] == 'closed' and int(c['transport']['epoch']) == 2
    assert c['end']['selected_source'] == '測試乙' and c['action_profile'] == r['action_profile']
    assert c['start']['world']['status']['object_id'] == w['status']['object_id'] == own_id
    assert len(probe['actions']) == (1 if role == 'b' else 0)
    assert probe['attacks_sent'] == w['own_attack_packets']
    assert any(s['object_id'] == own_id and s['phase'] == 'BATTLE' and s['hp'] == w['hp'][0] and s['mp'] == w['mp'][0] and [s['x'], s['y']] == [w['actors'][str(own_id)]['x'], w['actors'][str(own_id)]['y']] for s in trace)
    body = set()
    clothes = set()
    ended = set()
    sounds = []
    prepared = []
    cues = w['effects']
    assert len(cues) == (0 if refused else 1)
    for f in probe['frames']:
        fx = f['effect']
        assert fx['load_error'] is None and fx['omitted'] == 0
        if refused:
            assert fx['started'] == fx['active'] == 0 and not fx['effects'] and not fx['entered_events']
        for v in fx['effects']:
            assert v['graphics'] == gfx and v['target'][0] == 3 and v['sprite'] == sprite and 'error' not in v
            if tw:
                assert fx['tw_catalog']['namespace'] == 'launcher/TW13081901.pak'
                assert fx['tw_catalog']['decoded_sha256'].upper() == '7EBE2B464C8B1EA9C58AC985DD8FF4C823784D8231894262EDDCD7691CA94008'
                assert fx['tw_catalog']['supported_wire_graphics'] == [11722, 11723, 11724, 11725]
            assert v['subframe'] == (20 if v['source_frame'] == 0 else v['source_frame'] - 1)
            if v['finished']:
                assert not v['drawn'] and v['source_frame'] == 0 and v['next_callback_ms'] is None
                ended.add(v['sequence'])
            if v['drawn']:
                body.add(v['subframe'])
            if tw and skill == 57:
                assert [layer['graphics'] for layer in v['layers']] == [11725, 11726]
                assert all(layer['subframe'] == v['subframe'] for layer in v['layers'])
                for layer in v['layers']:
                    if layer['graphics'] == 11726 and layer['drawn']:
                        clothes.add(layer['subframe'])
            sources = {v['name']: v for v in fx['sources']}
            if tw:
                for n in ([tw_gfx, 11726] if skill == 57 else [tw_gfx]):
                    src = original[str(n)]['source']
                    assert sources[src['member']]['sha256'].upper() == src['sha256']
                    assert sources[src['member']]['bytes'] == src['bytes']
            else:
                assert sources['744-0.spr']['sha256'].upper() == '5532E36EDCD31152777B044F7F15CC1424C0D178BDE875C31B35358A99A2819C'
        sounds += fx['entered_events']
        prepared += [a for a in f['audio'] if a['event']['sound'] == sound]
    if not refused:
        assert len(cues) == 1 and cues[0]['graphics'] == gfx and cues[0]['target'][0] == 3
        assert ended == {cues[0]['sequence']}
        assert (set(range(1, last+1)) if tw else set(range(21))) <= body, (role, sorted(body))
        if tw and skill == 57:
            assert clothes == set(range(1, 10)), (role, sorted(clothes))
        assert len(sounds) == 1 and sounds[0]['sound'] == sound and sounds[0]['frame'] == 1
        assert len(prepared) == 1 and prepared[0]['prepared']
        assert prepared[0]['bytes'] == wav['bytes'] and prepared[0]['sha256'].upper() == wav['sha256']
    images = []
    for image in [c['start']['render'], c['render']] + probe['images']:
        name = image['png']
        if name.startswith('\\\\?\\'):
            name = name[4:]
        path = Path(name).resolve(strict=True)
        assert path.is_relative_to(runtime)
        with Image.open(path) as pixels:
            assert pixels.size == (800, 600)
            assert hashlib.sha256(pixels.convert('RGBA').tobytes()).hexdigest().upper() == image['frame']['rgba_sha256'].upper()
        images.append({'path': str(path), 'sha256': sha(path)})
    checks.append({'role': role, 'body_subframes': sorted(body), 'clothes_subframes': sorted(clothes), 'sounds': len(sounds), 'images': images, 'hp': w['hp'], 'mp': w['mp']})
report = {'status': 'verified_real_peer_action_authority', 'action_profile': r['action_profile'], 'resource_profile': r['resource_profile'], 'skill': skill, 'branch': branch, 'wire_graphics': gfx, 'actual_client_sessions': 4, 'C14_requests': 1, 'S37_copies': len(effects), 'receipt_sha256': sha(p), 'exe_sha256': r['exe_sha256'], 'jar_sha256': r['jar_sha256'], 'checks': checks, 'visual_roles_verified': [] if refused else ['a', 'b'], 'GUI_verified': False, 'physical_audio_verified': False, 'full_world_pixel_oracle': False, 'whole_goal_complete': False}
(p.parent / 'authority-comparison.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n', 'utf-8')
print('Verified higher heal:', skill, branch, r['resource_profile'])
