"""Audit actual formal heal/shield sessions and original resource samples."""
import datetime
import argparse
import hashlib
import json
from pathlib import Path
from PIL import Image

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / '驗證紀錄/current/native-skills'

def read(path):
    return json.loads((ROOT / path).read_text(encoding='utf-8-sig'))

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()

def audit(frames):
    seen, audio_seen, pngs, sounds = set(), set(), [], {}
    catalog = {r['graphics']: r for r in read('LiTo381-Modern/target/current/verification/native-effect-catalog.json')}
    effects, terminal_samples = {}, {221: 0, 744: 0}
    for frame in frames:
        effect = frame['skill_effects']
        assert not effect['load_error'] and effect['omitted'] == 0
        for e in effect['effects']:
            assert 'error' not in e and e['graphics'] in (221, 744)
            resource = catalog[e['graphics']]
            tick = e['elapsed_ns'] * 24 // 1_000_000_000
            expected, end, finished = 0, 0, True
            for source in list(range(1, len(resource['frames']))) + [0]:
                end += resource['frames'][source]['ticks']
                if tick < end:
                    expected, finished = source, False
                    break
            assert e['source_frame'] == expected and e['finished'] == finished and e['drawn'] == (not finished)
            source = resource['frames'][expected]
            assert e['subframe'] == source['subframe'] and e['sprite'] == f"{e['graphics']}-{source['image']}.spr"
            fixed = {'graphics': e['graphics'], 'target': e['target'], 'position': e['position']}
            assert effects.setdefault(e['sequence'], fixed) == fixed
            if expected == 0 and not finished:
                terminal_samples[e['graphics']] += 1
        events = effect['entered_events'] + [e for a in frame['actors'] for e in a.get('entered_events', [])]
        for e in events:
            key = json.dumps(e, sort_keys=True)
            assert key not in seen, e
            seen.add(key)
        assert frame['audio_events_dropped'] == 0
        for a in frame['audio']:
            assert a['prepared']
            key = json.dumps(a['event'], sort_keys=True)
            assert key not in audio_seen and key in seen
            audio_seen.add(key)
            row = {'id': a['event']['sound'], 'bytes': a['bytes'], 'sha256': a['sha256']}
            assert sounds.setdefault(row['id'], row) == row
        if frame.get('png'):
            p = Path(frame['png']).resolve()
            p.relative_to(ROOT)
            with Image.open(p) as im:
                assert im.size == (800, 600)
                rgba = hashlib.sha256(im.convert('RGBA').tobytes()).hexdigest()
            assert rgba == frame['rgba_sha256']
            pngs.append({'path': str(p), 'sha256': sha(p), 'rgba_sha256': rgba})
    assert frames[-1]['skill_effects']['active'] == 0
    return {'frames': len(frames), 'pngs': pngs, 'frame_events': len(seen), 'audio_events': len(audio_seen),
            'sounds': sorted(sounds.values(), key=lambda r: r['id']), 'events_once': True,
            'class0_timing_verified': True, 'terminal_samples': terminal_samples, 'fixed_effect_positions': list(effects.values())}

def main():
    source = read('驗證紀錄/current/native-combat/skills-receipt.json')
    pin = read('驗證紀錄/current/reference-snapshot.json')
    exe = ROOT / 'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
    assert source['exe_sha256'] == sha(exe) and source['jar_sha256'] == pin['sha256']
    assert len(source['cases']) == 2
    cases = []
    for case in source['cases']:
        run, server = case['client']['combat'], case['server']
        world = run['result']['world']
        heal = case['mode'] == 'room-world-heal'
        gfx, mp, refusal = (744, 6, {'Rejected': 278}) if heal else (221, 2, 'NoResponse')
        assert server['actual_client_sessions'] and not server['sql_access'] and not server['state_persistent']
        assert run['skills_sent'] == 2 and world['own_cast_packets'] == 1
        assert world['mp'] == [mp, 10] and run['result']['skill_feedback'] == refusal
        assert world['hp'][0] == server['final_player_hp']
        assert not heal or run['hp_before_skill'] < world['hp'][0]
        assert len(world['effects']) == 1 and world['effects'][0]['graphics'] == gfx
        assert run['frames'][-1]['skill_effects']['started'] == 1
        drawn = [e for f in run['frames'] for e in f['skill_effects']['effects'] if e['drawn']]
        assert drawn and all(e['graphics'] == gfx for e in drawn)
        row = audit(run['frames'])
        row.update(mode=case['mode'], graphics=gfx, hp_before=run['hp_before_skill'], hp_after=world['hp'], mp_after=world['mp'],
                   refusal=refusal, drawn_samples=len(drawn), sources=run['frames'][-1]['skill_effects']['sources'])
        cases.append(row)
    vector = audit(read('LiTo381-Modern/target/current/verification/native-skills.json'))
    terminal = audit(read('LiTo381-Modern/target/current/verification/native-effect-terminal.json'))
    assert all(terminal['terminal_samples'][gfx] > 0 for gfx in [221, 744])
    receipt = {'schema': 2, 'utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
               'exe_sha256': source['exe_sha256'], 'jar_sha256': source['jar_sha256'], 'cases': cases,
               'frames': sum(c['frames'] for c in cases), 'pngs': sum(len(c['pngs']) for c in cases),
               'frame_events': sum(c['frame_events'] for c in cases), 'offline_resource_samples': vector,
               'terminal_resource_samples': terminal,
               'gui_tested': False, 'audio_device_tested': False, 'complete_skill_system': False}
    (OUT / 'skill-visual-receipt.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
    print(json.dumps({k:receipt[k] for k in ['frames','pngs','frame_events']}))

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--output', default=str(OUT))
    OUT = Path(parser.parse_args().output).resolve()
    OUT.relative_to(ROOT)
    OUT.mkdir(exist_ok=True)
    main()
