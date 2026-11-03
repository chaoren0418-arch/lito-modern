"""Prepare the next candidate's full-library tools; no builds or product writes."""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json

R = Path(__file__).resolve().parents[1]
T = R / '維護工具'
V = R / '驗證紀錄/current'
C = T / 'candidates/native-rental-service-ui-and-hotel-return-20261019-180930-820793'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
encode = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
sha = lambda p: hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
w = read(R / '工作佇列.json')
base = Path(w['current_native_catalog_library']['run'])
old = 'A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
assert sha(base / 'source-inputs.json') == w['current_product_identity']['source_manifest_sha256'] == old
pins = read(base / 'source-inputs.json')
candidate = read(C / 'candidate.json')
test = read(C / 'test-contract.json')
for name, digest in candidate['before_files'].items(): assert sha(R / name) == digest
for name, digest in candidate['candidate_files'].items():
    assert sha(C / Path(name).name) == digest
    pins[name] = digest
assert sha(test['candidate']) == test['candidate_sha256']
pins[test['intended_product_path']] = test['candidate_sha256']
assert len(pins) == 1048
source = hashlib.sha256(encode(pins)).hexdigest().upper()
out = V / 'native-rental-service-entry-return-tooling' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
(out / 'predicted-source-inputs.json').write_bytes(encode(pins))
names = ['run-native-initial-source-confirmation-library-default.py',
    'qualify-native-initial-source-confirmation-library-default.py',
    'observe-native-initial-source-confirmation-library-default.py',
    'checkpoint-native-initial-source-confirmation-library-active.py',
    'checkpoint-native-initial-source-confirmation-library-completed.py']
records = {}
for name in names:
    original = T / name
    body = original.read_text('utf-8-sig')
    body = body.replace('native-initial-source-confirmation', 'native-rental-service-entry-return')
    body = body.replace('initial_source_confirmation', 'rental_service_entry_return')
    body = body.replace(old, source)
    body = body.replace('1048', '__COMPILED_INPUT_COUNT__').replace('1047', '1048').replace('__COMPILED_INPUT_COUNT__', '1049')
    # Each candidate has its own source namespace; no prior qualification is
    # promoted. Existing library inventory/ignore/body assertions are unchanged.
    body = body.replace('來源A9FE9AD5', '來源' + source[:8])
    body = body.replace('初次GAME C8來源確認之自然超30秒／快速GUI與租裝目的地／恢復、廣域／控制器仍待；4F2F成果與Release49194來源到期失敗各保留原版。',
        '租裝入口API／UI、旅館返回、初次C8與傳送恢復及廣域／控制器仍待；A9FE成果與4F2F來源到期失敗各保留原版。')
    ast.parse(body)
    target = T / name.replace('native-initial-source-confirmation', 'native-rental-service-entry-return')
    assert not target.exists()
    target.write_text(body, encoding='utf-8', newline='\n')
    records[target.name] = dict(path=str(target), sha256=sha(target), origin=str(original), origin_sha256=sha(original))
record = dict(status='future_rental_service_entry_return_full_library_tools_prepared_not_executed',
    predicted_source_manifest_sha256=source, predicted_source_manifest=str(out / 'predicted-source-inputs.json'),
    source_files=1048, compiled_input_files=1049, tools=records,
    library_inventory_or_ignored_body_assertions_changed=False, default_library_inventory_expected=2453,
    product_source_modified=False, tests_executed=False, GUI_verified=False, whole_goal_complete=False)
(out / 'library-tools.json').write_bytes(encode(record))
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tooling=str(out / 'library-tools.json'), predicted_source_manifest_sha256=source,
    tests_executed=False, product_source_modified=False), ensure_ascii=False))
