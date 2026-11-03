"""Adopt existing Batch13 source evidence; no candidate execution or source edits."""
from pathlib import Path
import datetime
import hashlib
import json
from r491_common import P, L, pin, read, save

contract = Path(r'D:\lineage開發\3-lito開發\lito脫殼資源\reconnect-source-index-contract.json')
expected = '004F07E864E160E78D7B87133F45B95739D01497DAD327C13DD36D52F94D0372'
root_pin = pin(contract)
assert root_pin['sha256'] == expected, root_pin
data = read(contract)
references = {}
witnesses = []

def visit(value):
    if isinstance(value, dict):
        if all(key in value for key in ('path', 'bytes', 'sha256')):
            actual = pin(value['path'])
            assert actual['bytes'] == value['bytes'], value['path']
            assert actual['sha256'] == value['sha256'].upper(), value['path']
            old = references.setdefault(value['path'], actual)
            assert old == actual
        if 'raw_hex' in value and 'file_offset' in value:
            witnesses.append(value)
        for child in value.values():
            visit(child)
    elif isinstance(value, list):
        for child in value:
            visit(child)

visit(data)
pe_source = data['sources'][0]
pe = Path(pe_source['path']).read_bytes()
assert hashlib.sha256(pe).hexdigest().upper() == pe_source['sha256']
raw_checks = []
for witness in witnesses:
    offset = int(witness['file_offset'], 16)
    raw = bytes.fromhex(witness['raw_hex'])
    assert pe[offset:offset + len(raw)] == raw, witness['va']
    raw_checks.append(dict(va=witness['va'], file_offset=witness['file_offset'],
                           bytes=len(raw), raw_hex=witness['raw_hex'], matches=True))
slice_checks = []
for inherited in data['inherited_raw']:
    raw = Path(inherited['range']['path']).read_bytes()
    offset = int(inherited['va'], 16) - 0x400000
    assert pe[offset:offset + len(raw)] == raw, inherited['va']
    slice_checks.append(dict(va=inherited['va'], source=inherited['range'], matches=True))

destination = L / 'batch13-reconnect-source-inputs'
destination.mkdir(exist_ok=True)
snapshot = destination / contract.name
if snapshot.exists():
    assert snapshot.read_bytes() == contract.read_bytes(), 'Existing snapshot differs'
else:
    snapshot.write_bytes(contract.read_bytes())
report_path = L / 'batch13-reconnect-source-adoption.json'
report = dict(
    schema='lito.modern.batch13.reconnect-source-adoption.v1',
    updated_at=datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
    status='EXISTING_SOURCE_CONTRACT_ADOPTED_WITH_LIMITS',
    input=root_pin, snapshot=pin(snapshot),
    references=list(references.values()), reference_count=len(references),
    fixed_PE=pe_source, raw_witnesses=raw_checks, raw_witness_count=len(raw_checks),
    reused_raw_slices=slice_checks, reused_raw_slice_count=len(slice_checks),
    adopted_constraints={
        'saved_startup_host': '64AD74 inline256; worker47C8A0 copies64B7E4 throughNUL after private050104CD returns; successful private return unobserved',
        'saved_character_alias': '64AD54 inline28; ordinary selected62FAA0+index*B4 record copied throughNUL afterC9call returns; cached4A76B2 passes savedalias toC9',
        'port': '64AD70 low16 port storage, not string length',
        'capacity_policy': '255host/27alias are in-bounds byte budgets excludingNUL, not proven Native enforced limits or character counts',
        'copy_policy': 'Opaque8bit/NUL; no local clamp or encoding conversion; tail afterNUL is not cleared',
        'clear_scope': '47C8A0 worker40-byte clear [64AD2C,64AD54) excludes savedalias/port/host',
        'COM_candidate': '481DC7 fragment zerosESI/EDI before scan; successfulCOMhost acquisition not qualified',
        'authority': 'Copies do not establish Modern endpoint, physical character identity, epoch, owner or GUI readiness',
    },
    new_wire_differences=0, new_opcodes=0, source_code_changes=0,
    candidate_test_runs=0, original_operations=0, Java_SQL_runs=0,
    old_fixture_or_PASS_replays=0, owner_ready=False, GUI_accepted=False,
    active_scope='ordinary explicit owned C_CC/C_CF intent -> S_CE reply scene/epoch transition; unchanged',
    remaining=data['Modern_not_inferable'],
)
save(report_path, report)
progress_path = P.parent / '監製進度.json'
progress = read(progress_path)
progress['updated_at'] = report['updated_at']
progress['batch13_reconnect_source_adoption'] = dict(
    status=report['status'], report=pin(report_path),
    reference_count=len(references), raw_witness_count=len(raw_checks),
    reused_raw_slice_count=len(slice_checks), new_tests=0,
    source_code_changes=0, room_transition_scope_unchanged=True,
    owner_ready=False, GUI_accepted=False,
)
save(progress_path, progress)
print(json.dumps(dict(report=pin(report_path), reference_count=len(references),
    raw_witness_count=len(raw_checks), reused_raw_slice_count=len(slice_checks),
    new_tests=0, source_code_changes=0), ensure_ascii=False))
