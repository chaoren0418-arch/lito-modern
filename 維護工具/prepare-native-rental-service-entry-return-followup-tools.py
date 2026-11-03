"""Prepare future-source resource/GUI/controller tools; execute no workloads."""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json
R = Path(__file__).resolve().parents[1]
T = R / '維護工具'
V = R / '驗證紀錄/current'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
library_receipt = V / 'native-rental-service-entry-return-tooling/readiness-refinement-20261019-190705-556531/library-tools.json'
library = read(library_receipt)
source = library['predicted_source_manifest_sha256']
assert sha(library['predicted_source_manifest']) == source
for info in library['tools'].values(): assert sha(info['path']) == info['sha256']
old = 'A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
names = [
    'run-native-initial-source-confirmation-library-resource-optin.py',
    'checkpoint-native-initial-source-confirmation-library-resource-optin.py',
    'prepare-native-initial-source-confirmation-sky-gui.py',
    'start-native-initial-source-confirmation-sky-gui-case.py',
    'checkpoint-native-initial-source-confirmation-sky-gui.py',
    'qualify-native-initial-source-confirmation-GUI-rental-entry.py',
    'qualify-native-initial-source-confirmation-GUI-hotel-entry.py',
    'run-native-initial-source-confirmation-controller.py',
    'qualify-native-initial-source-confirmation-controller-case.py',
    'qualify-native-initial-source-confirmation-controller-cohort.py',
    'observe-native-initial-source-confirmation-controller-completion.py',
    'checkpoint-native-initial-source-confirmation-controller-active.py',
    'checkpoint-native-initial-source-confirmation-controller-complete.py',
]
out = V / 'native-rental-service-entry-return-tooling' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
records = {}
for name in names:
    origin = T / name
    body = origin.read_text('utf-8-sig')
    body = body.replace('native-initial-source-confirmation', 'native-rental-service-entry-return')
    body = body.replace('initial_source_confirmation', 'rental_service_entry_return')
    body = body.replace(old, source).replace('來源A9FE9AD5', '來源' + source[:8])
    body = body.replace('1048', '__COMPILED_INPUT_COUNT__').replace('1047', '1048').replace('__COMPILED_INPUT_COUNT__', '1049')
    ast.parse(body)
    path = T / name.replace('native-initial-source-confirmation', 'native-rental-service-entry-return')
    assert not path.exists()
    path.write_text(body, encoding='utf-8', newline='\n')
    records[path.name] = dict(path=str(path), sha256=sha(path), origin=str(origin), origin_sha256=sha(origin))
record = dict(status='future_rental_service_entry_return_followup_tools_prepared_not_executed',
    predicted_source_manifest_sha256=source, source_files=1048, compiled_input_files=1049,
    library_tooling=str(library_receipt), library_tooling_sha256=sha(library_receipt), tools=records,
    resource_selection_tests_per_profile=10, controller_cases_per_profile=6,
    normal_GUI_initial_source_expiry_and_entry_assertions_retained=True,
    hotel_qualifier_scope='Entry and room lease only; does not qualify new C10 hotel return behavior.',
    new_API_guard_wire_runner_or_hotel_return_qualifier_prepared_by_this_tool=False,
    product_source_modified=False, tests_executed=False, GUI_verified=False, whole_goal_complete=False)
(out / 'followup-tools.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tooling=str(out / 'followup-tools.json'), predicted_source_manifest_sha256=source,
    tools=len(records), tests_executed=False, product_source_modified=False), ensure_ascii=False))
