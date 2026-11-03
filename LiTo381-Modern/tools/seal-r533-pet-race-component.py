import datetime
import hashlib
import json
import pathlib
import re
import shutil

project = pathlib.Path(__file__).resolve().parent.parent
logs = project / 'logs/continuation-20261003'
target = project / 'target/r533-native-pet-race-component'

def read(path):
    return json.loads(path.read_text(encoding='utf-8-sig'))

def pin(path):
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return {'path': str(path), 'bytes': path.stat().st_size, 'sha256': digest.hexdigest().upper()}

def save(path, data):
    with path.open('x', encoding='utf-8') as stream:
        json.dump(data, stream, ensure_ascii=False, indent=2)
        stream.write('\n')

root = logs / 'r533-full-backlog-and-pet-race-handoff.json'
assert not root.exists(), 'Preserve first R533 handoff'
build = read(logs / 'r533-library-build-accounting.json')
test = read(logs / 'r533-targeted-tests-accounting.json')
assert build['exit_code'] == test['exit_code'] == 0
stdout = (logs / 'r533-targeted-tests.stdout.log').read_text(encoding='utf-8-sig')
assert '2 passed; 0 failed' in stdout and '2349 filtered out' in stdout
comparison = read(logs / 'r533-pet-race-java-comparison.json')
assert len(comparison['cases']) == 96 and comparison['movement_or_panel_callbacks'] == 147
stderr = (logs / 'r533-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
binary = re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)', stderr).group(1)
test_exe = target / 'r533-focused-tests.exe'
assert not test_exe.exists()
shutil.copyfile(project / binary, test_exe)
tool_dir = target / 'tools'
tool_dir.mkdir()
for name in ['build-r533-pet-race-component.ps1', 'seal-r533-pet-race-component.py']:
    shutil.copyfile(project / 'tools' / name, tool_dir / name)
source = read(logs / 'r533-production-source-snapshot.json')
assert source['source_count'] == 646 and source['unchanged_previous_sources'] == 643
assert source['existing_sources_changed'] == ['modern_lito/mod.rs']
adoption = read(logs / 'r533-pet-race-adoption.json')
assert adoption['manifest_checked'] == 36
rows = [json.loads(line) for line in (logs / 'r533-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines() if line.strip()]
warnings = [r['message'] for r in rows if r.get('reason') == 'compiler-message' and r['message']['level'] == 'warning']
new_warnings = [r for r in warnings if any('native_pet_race_component' in s.get('file_name', '') for s in r.get('spans', []))]
assert not new_warnings
refs = [logs / name for name in [
    'r533-pet-race-adoption.json', 'r533-pet-race-server-root.json', 'r533-pet-race-contract.json',
    'r533-pet-race-replay-inputs.json', 'r533-pet-race-java-vectors.json',
    'r533-pet-race-java-comparison.json', 'r533-full-backlog-audit.json',
    'r533-background-consumer-boundary.json', 'r533-audit-initial-inspection-errors.txt',
    'r533-production-source-snapshot.json', 'r533-library-build-accounting.json',
    'r533-targeted-tests-accounting.json', 'r533-targeted-tests.stdout.log',
    'r533-targeted-tests.stderr.log', 'r533-library-build.jsonl', 'r533-library-build.stderr.log']]
refs += [target / 'source/src/modern_lito' / name for name in [
    'mod.rs', 'native_pet_race_component.rs', 'native_pet_race_component_tests.rs']]
refs += [tool_dir / 'build-r533-pet-race-component.ps1', tool_dir / 'seal-r533-pet-race-component.py']
library = pin(target / 'library/liblito_modern.rlib')
report = {
    'schema': 'MODERN_R533_BACKLOG_AND_NATIVE_PET_RACE_COMPONENT_V1',
    'state': 'NEW_RUST_COMPONENT_COMPILED_96_JAVA_OBSERVATIONS_MATCH_FULL_GAMEPLAY_PENDING',
    'updated_at': datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
    'server_root_adopted': {'path': adoption['root'], 'bytes': adoption['root_bytes'], 'sha256': adoption['root_sha256']},
    'server_payload_files_checked': 36,
    'implemented': [
        'Pinned Native component contract parse and namespace-bound course selection for566/569/570.',
        'Listed inclusive gate order; outside-to-inside callback; at most one gate per callback; four laps and frozen finished runner.',
        'First finisher deadline saturates and clamps to race deadline; equality expires; Java wrapped elapsed subtraction.',
        'Panel point entry, independent Transform/Speed cooldowns, equality admission and trailing progress; opportunity/duration only, no local buff/form grant.',
        'Full backlog reaudit: forty exact static plans, sixteen existing flows and twenty-four full-flow gaps; finite Native branches distinguished from missing live producers.'
    ],
    'production_library': library,
    'focused_test_executable': pin(test_exe),
    'source_count': 646, 'existing_sources_changed': source['existing_sources_changed'],
    'new_sources': source['new_sources'], 'prior_sources_unchanged': 643,
    'tests': {'new_checks_passed': 2, 'failed': 0, 'Java_observations_matched': 96,
              'synthetic_callbacks': 147, 'old_tests_filtered': 2349, 'old_PASS_replays': 0,
              'execution_seconds': 0.08, 'first_compile_and_test_attempts': 1},
    'warnings': {'regular_library': len(warnings), 'new_component': 0, 'test_harness': 92},
    'execution': {'new_Java_calls': 0, 'SQL': 0, 'TLS': 0, 'GUI': 0, 'provider': 0,
                  'credentials_reads': 0, 'PID_queries': 0, 'player_EXE_created': False,
                  'own_ASCII_scratch': build['scratch_retained'], 'original_BIN': False},
    'remaining': [
        'Actual Native outer battleStarted/RUNNING/registered-roster and movement/collision admission; reducer deliberately does not invent this guard.',
        'Native walkable spawns and directed gate/panel route witnesses; synthetic reducer coordinates are not walkability proof.',
        'Native actor/form/timing resource profile, authoritative buffs and effect expiry, wire/finish/return and lifecycle integration.',
        'Live Native appearance Work/cancellation/owner bridge, actual scheduler pointers/clock/collision inputs, controlled roof aliases/free lifetime and retained campaign callback effects.',
        'BIN6314/7482 scene owner/draw-position mapping; existing IMG-to-RGBA decoder sufficient, no Directory assignment.',
        'Actual GUI/DPI/input/ReadyCancel/Close and R532 fresh provider acceptance; all remain unexecuted.'
    ],
    'counts': {'Native_static_plans': 40, 'existing_full_flow_acceptance': 16,
               'remaining_full_flows': 24, 'new_full_bindings': 0, 'GUI': 0, 'LOADED': False},
    'preservation': {'R532_frozen_source_exe_main_library_request_handoff_changed': False,
                     'other_group_writes': False, 'private_BIN_branch_entered': False},
    'references': [pin(path) for path in refs], 'overall_completed': False
}
save(root, report)
human = project / 'Native-R533_完整待辦與PetRace元件交接.txt'
with human.open('x', encoding='utf-8') as stream:
    stream.write('R533：完整待辦重核與 Native PetRace Rust 元件\n\n'
        '已採用新封存的 server 契約，核對36份 payload；Rust 補上566/569/570門檻順序、四圈完成、收尾期限及面板各自冷卻。\n'
        '兩項新CPU測試PASS；96個Java觀測值一致、147次合成callback。舊2349測試filtered、未重跑舊PASS。\n'
        '一般library首次編譯成功；291既有warnings，新元件0。測試harness92既有warnings。646份來源另封存：1份既有mod.rs變更、2份新增、643份既有不變。\n'
        '40張Native靜態圖皆有方案；完整流程仍16/40、其餘24未完成，GUI0、LOADEDfalse。\n'
        'idle49、roof與Layer8已做有限來源步進；當次owner／actor／collision／scheduler clock及指標、campaign初始化與卸載仍缺實際producer。\n'
        'Native PetRace的實際有向地形路徑、出生點、外層movement/RUNNING守門、人物／效果／wire／結算返回尚未接線。面板回傳opportunity與30秒duration，並未本地授予buff／form。\n'
        '6314/7482新RGBA表示已提供，既有decode_img足夠；scene owner與draw位置未交付，不指派Directory。\n'
        'R532候選／request／root全部保留；未建新玩家EXE，未啟Java、SQL、socket、GUI、provider或原版BIN。暫存只在Modern自有ASCII目錄。\n\n'
        '主交接：' + str(root) + '\n完整待辦：' + str(logs / 'r533-full-backlog-audit.json') + '\n')
print(json.dumps({'handoff': pin(root), 'library': library, 'checks_passed': 2,
                  'Java_observations': 96, 'complete_modes': 16, 'total_modes': 40}, ensure_ascii=False))
