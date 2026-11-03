"""Apply the staged native rental UI/API changes only after the A9FE regressions.

Preparing this tool does not modify product inputs or grant validation.
"""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, zipfile

R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
C = R / '維護工具/candidates/native-rental-service-ui-and-hotel-return-20261019-180930-820793'
OLD = 'A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
encode = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest().upper()

queue = read(R / '工作佇列.json')
assert queue['current_product_identity']['source_manifest_sha256'] == OLD
assert not queue['active_workloads'] and not queue['source_and_corresponding_QA_frozen']
assert queue['current_full_default_library_verified']
normal = queue['current_native_initial_source_confirmation_normal_qualification']
assert normal['source_manifest_sha256'] == OLD and normal['qualification_granted']
library = queue['current_native_catalog_library']
assert library['source_manifest_sha256'] == OLD
resource = queue['current_native_library_resource_optin']
assert resource['source_manifest_sha256'] == OLD and resource['qualified']
controller = queue['current_native_catalog_controller_qualification']
assert controller['source_manifest_sha256'] == OLD and controller['qualified']
assert controller['actual_cases'] == 12 and controller['actual_sessions'] == 96
assert controller['recorded_owned_all_terminal'] and controller['original_exit_code'] == 0
for profile in ['Debug', 'Release']:
    proof = queue['current_normal_GUI_rental_entry_keyboard'][profile]
    assert proof['source_manifest_sha256'] == OLD and sha(proof['qualification']) == proof['qualification_sha256']
    proof = read(proof['qualification'])
    assert proof['initial_GAME_public_C8_confirmation_verified'] and proof['natural_initial_source_handoff_expiry_verified']
for record in [normal, resource, controller]:
    assert sha(record['qualification']) == record['qualification_sha256']
assert sha(library['completion']) == library['completion_sha256']
from native1030_broad_gate import require_idle_runtime
require_idle_runtime(R)

base = Path(library['run'])
pins = read(base / 'source-inputs.json')
inputs = read(base / 'compiled-inputs.json')
assert len(pins) == 1047 and len(inputs) == 1048 and sha(base / 'source-inputs.json') == OLD
assert all(sha(R / name) == digest for name, digest in inputs.items())
before = read(Path(queue['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
candidate = read(C / 'candidate.json')
test = read(C / 'test-contract.json')
assert candidate['base_source_manifest_sha256'] == test['base_source_manifest_sha256'] == OLD
assert not candidate['compiled'] and not test['compiled']
assert sha(candidate['formal_class_disassembly']) == candidate['formal_class_disassembly_sha256']
assert sha(candidate['selected_methods']) == candidate['selected_methods_sha256']
payloads = {}
for name, digest in candidate['candidate_files'].items():
    assert sha(R / name) == candidate['before_files'][name]
    path = C / Path(name).name
    assert sha(path) == digest
    payloads[name] = path.read_bytes()
new_test = test['intended_product_path']
assert new_test == 'LiTo381-Modern/tests/native_rental_entry_guard.rs' and not (R / new_test).exists()
assert sha(test['candidate']) == test['candidate_sha256']
payloads[new_test] = Path(test['candidate']).read_bytes()
assert len(payloads) == 3
new_pins = dict(pins)
new_inputs = dict(inputs)
for name, payload in payloads.items():
    digest = hashlib.sha256(payload).hexdigest().upper()
    new_pins[name] = digest
    new_inputs[name] = digest
assert len(new_pins) == 1048 and len(new_inputs) == 1049
source_bytes = encode(new_pins)
source = hashlib.sha256(source_bytes).hexdigest().upper()
assert source != OLD
out = V / 'native-rental-service-entry-return-normal' / ('application-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir(parents=True)
record = dict(status='applied_rental_service_entry_return_pending_validation',
    before_source_manifest_sha256=OLD, source_manifest_sha256=source,
    changed_files=list(payloads), created_files=[new_test],
    after_files={name: new_inputs[name] for name in payloads}, canonical_before=before,
    source_files=1048, compiled_input_files=1049, candidate=str(C / 'candidate.json'),
    candidate_sha256=sha(C / 'candidate.json'), candidate_test_contract=str(C / 'test-contract.json'),
    candidate_test_contract_sha256=sha(C / 'test-contract.json'),
    normal_validation_complete=False, normal_product_source_modified=True, source_overlays_used=False,
    diagnostic_API_extension=False, formal_JAR_modified=False, clocks_changed=False,
    normal_GUI_verified=False, whole_goal_complete=False,
    change_scope='Reject unsupported ordinary CA00 at advertised rental entry before writing; hide its controls. Expose existing hotel C10 return to character selection with eligibility/layout guards. Add separate opt-in actual-wire regression; preserve initial C8 and staged recovery behavior.')
work = ('來源' + source[:8] + '已套租裝入口／旅館返回兩檔修補及一份獨立opt-in測試；尚未建置或驗證。'
    '先唯一入口正常Debug／Release與19相關案，再完整library、必要原資源及新API實際封包測試，再正常GUI租裝入口／旅館返回與傳送恢復、最終Release回歸。'
    'A9FE完整library、資源、初次來源自然到期GUI與控制器96sessions保留原版；不移記、不改JAR／遊戲時限、不要求使用者操作。')
date = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d')
staged = dict(payloads)
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        identity = value['current_product_identity']
        assert identity['source_manifest_sha256'] == OLD
        value['qualification_before_rental_service_entry_return'] = dict(
            source_manifest_sha256=OLD, normal=value.get('current_native_initial_source_confirmation_normal_qualification'),
            library=value.get('current_native_catalog_library'), resource=value.get('current_native_library_resource_optin'),
            GUI_entry=value.get('current_normal_GUI_rental_entry_keyboard'), controller=value.get('current_native_catalog_controller_qualification'))
        identity.update(source_manifest_sha256=source, source_files=1048, compiled_input_files=1049,
            source_qualification_pending=True, Debug_normal_qualified_for_current_source=False,
            Release_normal_qualified_for_current_source=False, GUI_verified=False, whole_goal_complete=False)
        value.update(current_full_default_library_verified=False,
            current_native_rental_service_entry_return_application=record,
            working_tree_matches_current_normal_qualification=False,
            working_tree_modified_since_current_normal_qualification=True,
            next_task=work, next_focus=work, last_goal_turn_classification='progress',
            last_goal_turn_evidence=str(out / 'evidence.json'), goal_complete=False, whole_goal_complete=False)
        for info in value['current_source_goal_validation'].values():
            info.update(complete=False, current_source_manifest_sha256=source, prior_evidence_source_retained=OLD,
                prior_whole_source_qualification_applies_to_current_source=False)
        for item in value.get('items', []):
            item.update(complete=False, current_source_manifest_sha256=source, current_evidence_applies_to_current_source=False)
        value['current_focus'] = dict(item='rental_service_entry_return_validation', work=work,
            applies_to_current_source=True, goal_complete=False)
        if name.endswith('next-validation-action.json'): value['next_action'] = work
        if 'updated_date' in value: value['updated_date'] = date
        staged[name] = encode(value)
    else:
        first, separator, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert separator
        staged[name] = (date + '：' + work + '\n\n' + rest).encode('utf-8')
assert len(staged) == 11 and all(sha(R / name) == digest for name, digest in before.items())
assert all(sha(R / name) == digest for name, digest in inputs.items()) and not (R / new_test).exists()
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in [*before, *candidate['before_files']]: archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert set(archive.namelist()) == set(before) | set(candidate['before_files'])
    for name, digest in {**before, **candidate['before_files']}.items():
        assert hashlib.sha256(archive.read(name)).hexdigest().upper() == digest
with zipfile.ZipFile(out / 'applied-payload.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name, payload in payloads.items(): archive.writestr(name, payload)
for filename, payload in [('before-source-inputs.json', (base / 'source-inputs.json').read_bytes()),
    ('before-compiled-inputs.json', (base / 'compiled-inputs.json').read_bytes()),
    ('applied-source-inputs.json', source_bytes), ('applied-compiled-inputs.json', encode(new_inputs))]:
    (out / filename).write_bytes(payload)
(out / 'application.py').write_bytes(Path(__file__).read_bytes())
for name, payload in staged.items(): (R / name).write_bytes(payload)
assert all(sha(R / name) == digest for name, digest in new_inputs.items())
record.update(before_archive_sha256=sha(out / 'before.zip'), payload_archive_sha256=sha(out / 'applied-payload.zip'),
    before_files={**candidate['before_files'], **before})
(out / 'application.json').write_bytes(encode(record))
(out / 'evidence.json').write_bytes(encode(record))
(out / 'readback.json').write_bytes(encode(dict(before_files=before,
    after_files={name: sha(R / name) for name in before}, before_archive_sha256=sha(out / 'before.zip'),
    evidence_sha256=sha(out / 'evidence.json'), created_files=[new_test], source_after_files=record['after_files'])))
print(json.dumps(dict(application=str(out), source_manifest_sha256=source,
    changed_files=list(payloads), normal_validation_complete=False), ensure_ascii=False))
