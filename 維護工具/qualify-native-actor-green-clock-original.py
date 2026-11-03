"""Independently check native green callback bytes, private relocations and CSVs."""
from pathlib import Path
from datetime import datetime, timezone
import argparse, csv, hashlib, json, struct, zipfile

ROOT = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser()
ap.add_argument('directory', type=Path)
a = ap.parse_args()
out = a.directory.resolve(strict=True)
assert out.is_relative_to(ROOT / '驗證紀錄/current/native-actor-green-clock-original')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
digest = lambda b: hashlib.sha256(b).hexdigest().upper()
sha = lambda p: digest(p.read_bytes())
def rows(name):
    with (out / name).open(encoding='utf-8-sig', newline='') as f:
        return list(csv.DictReader(f))

r = read(out / 'run-receipt.json')
assert r['status'] == 'executed_original_green_callback_private_queue'
assert r['build_exit_code'] == r['native_exit_code'] == 0
assert r['source_files'] == 1019 and r['product_sources_unchanged']
assert r['normal_mains_before'] == r['normal_mains_after']
pins = read(out / 'source-inputs.json')
assert len(pins) == 1019
for name, h in pins.items():
    assert sha(ROOT / name) == h, name
normal_path = Path(r['normal_qualification']).resolve(strict=True)
assert normal_path.is_relative_to(ROOT / '驗證紀錄/current')
normal = read(normal_path)
assert normal['source_manifest_sha256'] == sha(out / 'source-inputs.json')
for profile, item in normal['profiles'].items():
    assert sha(Path(item['path'])) == item['sha256'] == r['normal_mains_after'][profile]
summary = read(out / 'oracle.json')
assert summary == dict(status='original_green_callback_and_queue_fragments_executed',
    scalar_vectors=532, queue_snapshots=16, original_functions=10,
    guard_failures=0, game_sessions=0, whole_client_executed=False)
prep = read(out / 'preparation-readback.json')
with zipfile.ZipFile(out / 'prepared-inputs.zip') as z:
    assert set(z.namelist()) == set(prep['files'])
    for name, h in prep['files'].items():
        assert digest(z.read(name)) == h == sha(out / name), name
assert sha(out / 'prepared-inputs.zip') == prep['archive_sha256']
addresses = {}
for line in (out / 'addresses.tsv').read_text(encoding='utf-8').splitlines():
    key, value = (int(x, 16) for x in line.split())
    assert key not in addresses
    addresses[key] = value
functions = {}
for line in (out / 'functions.tsv').read_text(encoding='utf-8').splitlines():
    name, address, size = line.split()
    functions[name] = (int(address, 16), int(size))
assert len(functions) == 10 and functions['green'] == (0x4878f0, 31)
assert functions['scene_cleanup'] == (0x4d6470, 122)
relocations = {name: [] for name in functions}
for line in (out / 'relocations.tsv').read_text(encoding='utf-8').splitlines():
    name, kind, offset, target, next_offset = line.split()
    relocations[name].append((kind, int(offset), int(target, 16), int(next_offset)))
for name, (address, size) in functions.items():
    expected = bytearray((out / (name + '-original.bin')).read_bytes())
    assert len(expected) == size
    for kind, offset, target, next_offset in relocations[name]:
        assert 0 <= offset <= size - 4
        original = int.from_bytes(expected[offset:offset + 4], 'little')
        if kind == 'relative':
            assert (address + next_offset + original) & 0xffffffff == target
            value = (addresses[target] - addresses[address] - next_offset) & 0xffffffff
        else:
            assert kind == 'absolute' and original == target
            value = addresses[target]
        expected[offset:offset + 4] = value.to_bytes(4, 'little')
    assert bytes(expected) == (out / (name + '-relocated.bin')).read_bytes(), name
image = (out / 'oracle.exe').read_bytes()
assert digest(image) == r['helper_sha256'] and image[:2] == b'MZ'
pe = struct.unpack_from('<I', image, 60)[0]
assert image[pe:pe + 4] == b'PE\0\0'
assert struct.unpack_from('<H', image, pe + 4)[0] == 0x14c
assert struct.unpack_from('<H', image, pe + 24)[0] == 0x10b

scalar = rows('green-vectors.csv')
seeds = list(range(128)) + [0x7ffffffe, 0x7fffffff, 0x80000000, 0xfffffffe, 0xffffffff]
expected_inputs = [(seed, now) for seed in seeds for now in [0, 49, 1000, 0xfffffff0]]
assert len(scalar) == len(expected_inputs) == 532
for row, (seed, now) in zip(scalar, expected_inputs):
    value = (seed + 1) & 0xffffffff
    pulse = abs(12 - value % 24) + 16
    assert {k: int(v) for k, v in row.items()} == dict(initial=seed,
        counter_out=value, schedule_clock=now, deadline=(now + 50) & 0xffffffff,
        clock_reads=1, pulse=pulse, step=pulse // 2)

states = rows('green-state-vectors.csv')
# Conditional calls and map filtering, rather than a claim about every game entry.
expected = [
    ('entry', 1, 1, 1000, 1, 0), ('early', 1, 1, 1049, 2, 0),
    ('due', 2, 1, 1050, 4, 1), ('late', 2, 1, 1014, 3, 1),
    ('late_not_due', 2, 1, 1056, 4, 0), ('late_due', 3, 1, 1057, 6, 1),
    ('duplicate', 4, 2, 1021, 5, 1), ('cancel_owned', 42, 1, 0, 2, 0),
    ('cancel_other_owner_dispatched', 43, 1, 50, 4, 1),
    ('map_before_filter', 42, 12, 1000, 12, 0),
    ('map_after_filter', 42, 10, 1000, 13, 0),
    ('explicit_restart_after_filter', 43, 11, 1000, 14, 0),
    ('scene_before_cleanup', 78, 13, 1000, 13, 0),
    ('scene_after_cleanup', 78, 7, 1000, 13, 0),
    ('explicit_restart_after_cleanup', 79, 8, 1000, 14, 0),
    ('cancel_then_restart', 1, 1, 0, 2, 0)]
assert len(states) == len(expected) == 16
for row, (name, counter, count, now, reads, fired) in zip(states, expected):
    assert row['scenario'] == name
    actual = {k: int(v) for k, v in row.items() if k != 'scenario'}
    pulse = abs(12 - counter % 24) + 16
    assert actual == dict(counter=counter, pulse=pulse, step=pulse // 2,
        queued=count, clock_now=now, clock_reads=reads, fired=fired), name
one = lambda due, owner=0: [(due, 'green', owner)]
kept = [(4000 + i, 'keep' + str(i), i + 1) for i in range(9)]
aux = [(11000, 'map_aux', 0xffffffff)]
queues = dict(entry=one(1050), early=one(1050), due=one(1100),
    late=one(1057), late_not_due=one(1057), late_due=one(1107),
    duplicate=[(1057, 'green', 0), (1064, 'green', 0)],
    cancel_owned=one(50, 77), cancel_other_owner_dispatched=one(100),
    map_before_filter=one(1050) + one(1050, 77) + [(1070, 'note1', 88)] + kept,
    map_after_filter=kept + aux, explicit_restart_after_filter=one(1050) + kept + aux,
    scene_before_cleanup=one(1050) + one(1050, 77) + [(1060, 'map_aux', 0xffffffff), (1070, 'note1', 88)] + kept,
    scene_after_cleanup=kept[:7], explicit_restart_after_cleanup=one(1050) + kept[:7],
    cancel_then_restart=one(50))
queue_rows = rows('green-queue-vectors.csv')
expected_queue = [dict(scenario=name, index=str(i), deadline=str(due), callback=kind, owner=str(owner))
    for name, *_ in expected for i, (due, kind, owner) in enumerate(queues[name])]
assert queue_rows == expected_queue
raw = read(out / 'run-archive-readback.json')
assert sha(out / 'run-raw.zip') == raw['archive_sha256']
with zipfile.ZipFile(out / 'run-raw.zip') as z:
    assert set(z.namelist()) == set(raw['files'])
    for name, h in raw['files'].items():
        assert digest(z.read(name)) == h == sha(out / name), name
result = dict(status='independently_verified_original_green_callback_and_private_queue',
    source_files=1019, scalar_vectors=532, queue_snapshots=16, queue_rows=len(queue_rows),
    original_functions=10, original_function_bytes=1121, helper_machine='x86_PE32',
    helper_sha256=digest(image), archive_sha256=raw['archive_sha256'],
    callback_original_sha256=sha(out / 'green-original.bin'),
    observed_utc=datetime.now(timezone.utc).isoformat(),
    whole_client_or_all_entry_paths_verified=False, product_binding_verified=False,
    game_sessions=0, whole_goal_complete=False)
dest = out / 'qualified.json'
assert not dest.exists()
dest.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n', encoding='utf-8', newline='\n')
print(json.dumps(result, ensure_ascii=False))
