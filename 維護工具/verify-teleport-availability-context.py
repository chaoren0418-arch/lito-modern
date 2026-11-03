"""Verify real captured page/action/status/endpoint ordering, without UI assumptions."""
from pathlib import Path
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
CASES = ['20261014-183652-396', '20261014-184029-780', '20261014-184151-507',
         '20261014-184304-749', '20261014-184913-868', '20261014-185033-682']


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest().upper()


def main():
    cases = []
    for name in CASES:
        case = ROOT / '驗證紀錄/current/native-teleport-gui' / name
        receipt = json.loads((case / 'receipt.json').read_text(encoding='utf-8-sig'))
        assert receipt['status'].startswith('verified_native_teleport_GUI_')
        assert receipt['exe_sha256'] == '681BAA004CCC9B425EA7998D6C0F7FD8445FDA0B1E9FFB8B5A5E031AB56D6153'
        assert receipt['jar_sha256'] == 'C02AAAE8D54A27E30460BE2E3C47709EE64D35F007A8BE27AC74A327268B9539'
        capture = next((Path(receipt['runtime']) / 'captures').glob('*/frames.jsonl'))
        frames = [json.loads(line) for line in capture.read_text(encoding='utf-8-sig').splitlines()]
        packets = [(f, None if f['plain'] == 'MASKED_C_LOGIN' else bytes.fromhex(f['plain'])) for f in frames]
        observations = []
        for at, (frame, body) in enumerate(packets):
            if frame['direction'] != 'client_to_server' or body is None or body[:1] != b'\x27':
                continue
            action = body[5:-2]
            if action not in [b'LT_TELEPORT_B', b'LT_TELEPORT_C']:
                continue
            assert body[-2:] == b'\0\0'
            npc = body[1:5]
            dialogs = [(i, b) for i, (f, b) in enumerate(packets[:at]) if
                       f['direction'] == 'server_to_client' and b and b[:5] == b'\x2a' + npc and
                       b[5:].startswith(b'TeleportMe\0')]
            assert dialogs
            dialog_at, _ = dialogs[-1]
            assert not any(f['direction'] == 'server_to_client' and b and b[0] == 0xf3
                           for f, b in packets[dialog_at + 1:at])
            status_frame, status = packets[at + 1]
            endpoint_frame, endpoint = packets[at + 2]
            assert status_frame['direction'] == endpoint_frame['direction'] == 'server_to_client'
            assert status[0] == 0xf3 and status[1] <= 10 and len(status) == 2 + status[1] * 4
            words = [int.from_bytes(status[i:i + 4], 'little') for i in range(2, len(status), 4)]
            assert all(w in [0, 0xffffffff] for w in words)
            index = 1 if action.endswith(b'B') else 2
            assert len(endpoint) == 8 and endpoint[:2] == bytes([0xf4, index]) and words[index] == 0
            observations.append(dict(action=action.decode(), npc=int.from_bytes(npc, 'little'),
                                     dialog_frame=dialog_at, action_frame=at, status_frame=at + 1,
                                     endpoint_frame=at + 2, availability_slots=[w == 0 for w in words],
                                     no_page_status_before_action=True))
        assert len(observations) == 2 and {x['action'] for x in observations} == {'LT_TELEPORT_B', 'LT_TELEPORT_C'}
        bootstrap = [b for f, b in packets if f['direction'] == 'server_to_client' and b and b[0] == 0xf3
                     and 'rental-teleport' not in f['note']]
        assert bootstrap and all(b == bytes([0xf3, 1, 0, 0, 0, 0]) for b in bootstrap)
        cases.append(dict(case=str(case), scenario=receipt['scenario'], capture_sha256=sha(capture),
                          receipt_sha256=sha(case / 'receipt.json'), observations=observations,
                          bootstrap_status_is_one_source_slot=True))
    result = dict(status='verified_current_formal_teleport_availability_is_action_scoped',
                  cases=cases, observed_page_actions=sum(len(c['observations']) for c in cases),
                  main_sha256=receipt['exe_sha256'], jar_sha256=receipt['jar_sha256'],
                  verifier_sha256=sha(Path(__file__)),
                  UI_rule='Opening the original page does not provide B/C availability. Do not reuse one-slot LOGIN status or invent disabled/available destination states. The received page-action F3 authorizes its matching F4.',
                  original_images_are_static_page_references=True,
                  original_client_image_state_semantics_verified=False,
                  GUI_images_integrated=False, full_game_complete=False)
    output = ROOT / '驗證紀錄/current/native-world-integration/teleport-availability-context.json'
    output.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({'status': result['status'], 'actions': result['observed_page_actions'], 'cases': len(cases)}))


if __name__ == '__main__':
    main()
