"""Keep prepared, unapplied work discoverable while the original broad job runs."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib
import json
import zipfile
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
read = lambda path: json.loads(Path(path).read_text('utf-8-sig'))
sha = lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest().upper()
data = lambda value: (json.dumps(value, ensure_ascii=False, indent=2) + '\n').encode('utf-8')
w = read(R / '工作佇列.json')
assert w['active_workload_sessions'] == {'broad_default': 41862} and w['source_and_corresponding_QA_frozen']
source = w['current_product_identity']['source_manifest_sha256']
assert source == 'A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
run = Path(w['current_native_catalog_broad_active']['run'])
cohort = read(run / 'cohort.json')
assert cohort['status'] == 'executing_current_default117_profile' and cohort['active_profile'] == 'Release'
assert cohort['profiles']['Debug']['qualified'] and cohort['profiles']['Debug']['counts'] == dict(passed=544, failed=0, ignored=24)
assert all(sha(R / name) == digest for name, digest in read(Path(cohort['normal_qualification']).parent.parent / 'compiled-inputs.json').items())
assert all(sha(R / '維護工具' / name) == digest for name, digest in cohort['validation_pins'].items())
references = {
    'skill_packet_hints': V / 'native-current-skill-request-scope/emitted-packet-hints-20261020-081904-095828/hints.json',
    'haste43_candidate': R / '維護工具/candidates/native-haste43-20261020-074819-989309/candidate.json',
    'haste43_components': R / '維護工具/candidates/native-haste43-20261020-074819-989309/component-preparation-20261020-075539-367040/preparation.json',
    'haste43_component_runner': R / '維護工具/run-current-native-haste43-components.py',
    'haste43_component_reader': R / '維護工具/checkpoint-current-native-haste43-components.py',
    'haste_effect_resource_plan': V / 'native-current-library-haste-resource2/execution-plan-20261020-083528-461441/plan.json',
    'haste_effect_resource_runner': R / '維護工具/run-current-library-haste-resource2.py',
    'haste_effect_resource_reader': R / '維護工具/checkpoint-current-library-haste-resource2.py',
    'rejected_candidate': R / '維護工具/candidates/native-haste43-20261020-074307-266851/rejection.json',
}
candidate = read(references['haste43_candidate'])
assert not candidate['source_applied'] and not candidate['actual_component_or_library_tests_executed']
assert candidate['distinct_written_files'] == len(set(candidate['candidate_files'].values())) == 4
assert all(sha(R / name) == digest for name, digest in candidate['before'].items())
assert all(sha(candidate['candidate_files'][name]) == digest for name, digest in candidate['after'].items())
plan = read(references['haste_effect_resource_plan'])
assert sha(references['haste_effect_resource_runner']) == plan['runner_sha256']
assert sha(references['haste_effect_resource_reader']) == plan['reader_sha256']
assert not read(references['haste43_components'])['tests_executed']
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
out = V / 'native-current-background-preparations' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
record = dict(status='background_preparations_preserved_while_original_Release_broad_compiles', source_manifest_sha256=source,
              original_broad_session=41862, original_broad_run=str(run), qualified_broad_profiles={'Debug': dict(passed=544, failed=0, ignored=24)},
              references={name: dict(path=str(path), sha256=sha(path)) for name, path in references.items()},
              product_source_modified=False, corresponding_broad_QA_modified=False, new_tests_or_casts_by_this_checkpoint=0,
              candidate_applied=False, candidate_component_tests_executed=False, haste_resource_tests_executed=False,
              normal_GUI_input_runtime_failure_preserved=True, GUI_or_whole_goal_qualification_granted=False)
(out / 'evidence.json').write_bytes(data(record))
(out / 'checkpoint.py').write_bytes(Path(__file__).read_bytes())
work = 'A695原41862廣域仍沿原批次執行：Debug544過0敗24忽略已獨立核對，Release待完整結果與原始終止。背景已備正式75封包提示、正確四路徑#43候選／3反例＋20項元件來源及獨立runner/reader、#43／54原資源兩項補驗；全部未執行／未套用，不改83已跑與405忽略未跑計數。GUI最小化／input guard失敗與首批候選同名覆寫拒收均已封存；自行測試完成前不交人工，正常來源及對應QA保持凍結。'
staged = {}
for name in before:
    path = R / name
    if path.suffix == '.json':
        value = read(path)
        value['current_background_development_preparations'] = record
        value.setdefault('current_source_goal_validation', {}).setdefault('G05-gameplay-matrix', {}).update(unapplied_haste43_preparation=record['references']['haste43_candidate'], complete=False)
        if value.get('current_normal_GUI_input_runtime_failure'):
            scope = value.setdefault('current_source_goal_validation', {}).setdefault('G08-player-gui', {})
            scope.update(state='normal_GUI_input_runtime_unavailable_last_attempt_failed', complete=False)
            for item in value.get('items', []):
                if item.get('id') == 'G08-player-gui':
                    item.update(state=scope['state'], complete=False)
        value.update(next_task=work, next_focus=work, last_goal_turn_evidence=str(out / 'evidence.json'), last_goal_turn_classification='progress', goal_complete=False, whole_goal_complete=False)
        if name.endswith('next-validation-action.json'):
            value['next_action'] = work
        staged[name] = data(value)
    else:
        first, sep, rest = path.read_text('utf-8-sig').partition('\n\n')
        assert sep
        staged[name] = (datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d') + '：' + work + '\n\n' + rest).encode('utf-8')
with zipfile.ZipFile(out / 'before.zip', 'x', zipfile.ZIP_DEFLATED) as archive:
    for name in before:
        archive.write(R / name, name)
with zipfile.ZipFile(out / 'before.zip') as archive:
    assert all(hashlib.sha256(archive.read(name)).hexdigest().upper() == digest for name, digest in before.items())
assert all(sha(R / name) == digest for name, digest in before.items())
for name, payload in staged.items():
    (R / name).write_bytes(payload)
(out / 'readback.json').write_bytes(data(dict(before_files=before, after_files={name: sha(R / name) for name in before}, before_archive_sha256=sha(out / 'before.zip'), evidence_sha256=sha(out / 'evidence.json'))))
print(json.dumps(dict(checkpoint=str(out), original_session=41862, new_tests_executed=0, candidate_applied=False), ensure_ascii=False))
