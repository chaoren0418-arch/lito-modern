"""Read only selected timer/status wire rows from the sealed current GUI cohort."""
from pathlib import Path
import collections
import datetime
import hashlib
import json

ROOT = Path(__file__).resolve().parents[1]
COHORT = ROOT / '驗證紀錄/current/native-teleport-current/20261016-014931-306-TW-shared'
STAGE = ROOT / '驗證紀錄/current/native-game-timers/20261016-004010-201637/current-regression'
OUT = STAGE / 'current-GUI-game-timer-observation.json'
read = lambda path: json.loads(path.read_text(encoding='utf-8-sig'))
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest().upper()
assert not OUT.exists()
assert read(STAGE / 'current-followup.json')['status'] == 'completed_current_default93_and_TW14_defined_scopes'
qualification = COHORT / 'qualified.json'
cohort = read(qualification)
assert cohort['raw_archive_readback_verified'] and cohort['scenarios'] == 14
cases = []
totals = collections.Counter()
vectors = []
for profile, group in cohort['profiles'].items():
    for item in group['cases']:
        case = Path(item['case'])
        receipt = read(case / 'receipt.json')
        pin = read(case / 'current-default-source-qualification.json')
        paths = list((Path(receipt['runtime']) / 'captures').glob('*/frames.jsonl'))
        assert len(paths) == 1
        path = paths[0]
        digest = sha(path)
        assert digest == pin['capture_sha256']
        counts = collections.Counter()
        for line in path.read_text(encoding='utf-8-sig').splitlines():
            frame = json.loads(line)
            request = frame['direction'] == 'client_to_server' and frame['opcode'] in ['0x20', '0x1D']
            response = frame['direction'] == 'server_to_client' and frame['opcode'] in ['0x0C', '0x21']
            if not request and not response:
                continue  # Unrelated rows may redact credentials or have no decoded payload.
            body = bytes.fromhex(frame['plain'])
            opcode = int(frame['opcode'], 16)
            assert body and body[0] == opcode
            if request:
                assert body == bytes([opcode]) and frame['wire_length'] == 3
                counts['C20' if opcode == 0x20 else 'C1D'] += 1
                continue
            if opcode == 0x0c:
                assert len(body) == 37 and frame['wire_length'] == 39
                raw = int.from_bytes(body[25:29], 'little')
                tag = 'S0C'
                # Pinned C02 Protocol.characterStatus bytecode519..520 writes
                # lconst_0; Writer.d. Preserve the received zero; it is not a
                # current-clock sample and must not be inferred from wall time.
                assert raw == 0
                counts[tag] += 1
                vectors.append({'profile': profile, 'scenario': item['scenario'], 'opcode': tag,
                                'time': frame['time'], 'raw': raw, 'reference_delta': None})
                continue
            else:
                assert len(body) == 5 and frame['wire_length'] == 7
                raw = int.from_bytes(body[1:], 'little')
                tag = 'S21'
            timestamp = datetime.datetime.fromisoformat(frame['time'].replace('Z', '+00:00'))
            milliseconds = int(timestamp.timestamp() * 1000)
            expected = (milliseconds - 1735689600000) * 6 // 1000
            expected -= expected % 3
            assert raw % 3 == 0 and abs(raw - expected) <= 3, (profile, item['scenario'], tag, raw, expected)
            counts[tag] += 1
            vectors.append({'profile': profile, 'scenario': item['scenario'], 'opcode': tag,
                            'time': frame['time'], 'raw': raw, 'reference_delta': raw - expected})
        totals.update(counts)
        cases.append({'profile': profile, 'scenario': item['scenario'], 'case': str(case),
                      'capture_sha256': digest, 'counts': dict(counts)})
proof = {'status': 'retained_current_GUI_timer_and_status_time_wire_observation',
         'cohort_qualification_sha256': sha(qualification), 'counts': dict(totals),
         'cases': cases, 'time_vectors': vectors,
         'wire_lengths_and_clock_scale_alignment_verified': True,
         'clock_scale_assertions_apply_to_S21_only': True,
         'C02_S0C_fixed_zero_time_matches_actual_writer': True,
         'C02_writer_disassembly_sha256': sha(ROOT / '驗證紀錄/current/native-lighting-environment-wire/C02-Protocol-status-clock.javap.txt'),
         'per_connection_timer_pairing_verified_in_this_multiplexed_capture': False,
         'five_minute_save_natural_interval_executed_by_this_GUI_supplement': False,
         'separate_original_interval_evidence': '驗證紀錄/current/native-game-timers/20261016-004010-201637/natural',
         'prior_observer_failure': 'GUI-timer-observer-first-failure.json',
         'prior_status_clock_assumption_failure': 'GUI-timer-observer-second-failure.json',
         'observer_sha256': sha(Path(__file__)), 'new_game_sessions': 0, 'whole_goal_complete': False}
OUT.write_text(json.dumps(proof, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps({'status': proof['status'], 'counts': dict(totals), 'time_vectors': len(vectors)}))
