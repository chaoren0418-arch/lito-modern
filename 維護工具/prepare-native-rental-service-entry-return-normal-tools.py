"""Prepare future-source normal-build readers without applying product changes."""
from pathlib import Path
from datetime import datetime, timezone
import ast, hashlib, json

R = Path(__file__).resolve().parents[1]
T = R / '維護工具'
V = R / '驗證紀錄/current'
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
old_namespace = 'native-initial-source-confirmation-normal'
new_namespace = 'native-rental-service-entry-return-normal'
out = V / 'native-rental-service-entry-return-tooling' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
records = {}
for prefix in ['run', 'qualify', 'checkpoint']:
    original = T / (prefix + '-' + old_namespace + '.py')
    body = original.read_text('utf-8-sig')
    body = body.replace(old_namespace, new_namespace).replace('initial_source_confirmation', 'rental_service_entry_return')
    body = body.replace('1048', '__COMPILED_INPUT_COUNT__').replace('1047', '1048').replace('__COMPILED_INPUT_COUNT__', '1049')
    if prefix == 'qualify':
        start = body.index("failure=application['original_GUI_source_handoff_failure']")
        end = body.index("verify_archive(app/'applied-payload.zip'", start)
        body = body[:start] + '''candidate=read(application['candidate']);test=read(application['candidate_test_contract'])
assert sha(application['candidate'])==application['candidate_sha256'] and sha(application['candidate_test_contract'])==application['candidate_test_contract_sha256']
assert candidate['base_source_manifest_sha256']==test['base_source_manifest_sha256']==application['before_source_manifest_sha256']
assert application['before_source_manifest_sha256']=='A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
old=read(app/'before-compiled-inputs.json');assert set(old)<=set(inputs)
changed={name for name,digest in inputs.items()if old.get(name)!=digest}
assert changed==set(application['changed_files'])==set(candidate['candidate_files'])|{test['intended_product_path']}
assert set(inputs)-set(old)==set(application['created_files'])=={test['intended_product_path']}
assert application['after_files']=={**candidate['candidate_files'],test['intended_product_path']:test['candidate_sha256']}
assert len(old)==1048 and len(inputs)==1049
''' + body[end:]
        body = body.replace('original_GUI_source_failure_count=1,rental_service_entry_return_runtime_verified=False,',
            'rental_entry_guard_actual_wire_verified=False,hotel_return_GUI_verified=False,new_optin_test_compiled_by_this_scope=False,')
        # The template field describes the prior repair; it is not runtime proof
        # for this new source either, regardless of its historical spelling.
        body = body.replace('original_GUI_source_failure_count=1,initial_GAME_source_confirmation_runtime_verified=False,',
            'rental_entry_guard_actual_wire_verified=False,hotel_return_GUI_verified=False,new_optin_test_compiled_by_this_scope=False,')
    if prefix == 'checkpoint':
        start = body.index("stage='已actual exit0")
        end = body.index('\ndate=', start)
        body = body[:start] + '''stage='已actual exit0與獨立讀回：兩版正常EXE及每版19既有冒煙案通過；新opt-in測試尚未編譯／執行。'if qualified else'已actual exit'+str(a.exit_code)+'，原程序已終止，獨立讀回仍待。'if terminal else'沿原程序建置正常兩版及19相關冒煙案，來源固定，不重啟。'
work='來源'+c['source_manifest_sha256'][:8]+'租裝入口／旅館返回修正版原'+str(a.session)+'：'+stage+'當版完整library、必要原資源、新API實際封包、正常GUI租裝入口／旅館返回與傳送恢復、最終Release廣域回歸仍待。A9FE原完整library、來源自然到期GUI與控制器96sessions保留原版；不移記、不改JAR或時限、不要求使用者操作。'
''' + body[end:]
        body = body.replace('initial-source-confirmation-normal-build-and-existing-protocol-smoke-tests',
            'rental-service-entry-return-normal-build-and-existing-protocol-smoke-tests')
    ast.parse(body)
    target = T / (prefix + '-' + new_namespace + '.py')
    assert not target.exists()
    target.write_text(body, encoding='utf-8', newline='\n')
    records[target.name] = dict(path=str(target), sha256=sha(target), origin=str(original), origin_sha256=sha(original))
application = T / 'apply-native-rental-service-entry-return.py'
ast.parse(application.read_text('utf-8-sig'))
record = dict(status='future_rental_service_entry_return_normal_tools_prepared_not_executed',
    tools=records, application_tool=str(application), application_tool_sha256=sha(application),
    expected_source_files=1048, expected_compiled_inputs=1049, existing_smoke_tests_per_profile=19,
    new_optin_target='native_rental_entry_guard', new_optin_compiled_or_run_by_normal_scope=False,
    product_source_modified=False, tests_executed=False, GUI_verified=False, whole_goal_complete=False)
(out / 'normal-tools.json').write_text(json.dumps(record, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
(out / 'preparer.py').write_bytes(Path(__file__).read_bytes())
print(json.dumps(dict(tooling=str(out / 'normal-tools.json'), tests_executed=False, product_source_modified=False), ensure_ascii=False))
