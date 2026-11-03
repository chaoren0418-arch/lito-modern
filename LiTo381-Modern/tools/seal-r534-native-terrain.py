import datetime
import hashlib
import json
import pathlib
import re
import shutil

p = pathlib.Path(__file__).resolve().parent.parent
l = p / 'logs/continuation-20261003'
t = p / 'target/r534-native-pet-race-terrain-final'
root = l / 'r534-native-terrain-offline-consumer-handoff.json'
assert not root.exists()
def read(f): return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    d = hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda: s.read(1024 * 1024), b''): d.update(b)
    return {'path': str(f), 'bytes': f.stat().st_size, 'sha256': d.hexdigest().upper()}
def save(f, d):
    with f.open('x', encoding='utf-8') as s:
        json.dump(d, s, ensure_ascii=False, indent=2); s.write('\n')
c = read(l / 'r534-native-terrain-comparison.json')
assert c['mask_differences'] == [] and c['composed_spawn_panel_routes'] == 260 and c['composed_spawn_four_lap_routes'] == 30
stdout = (l / 'r534-targeted-tests-repair1.stdout.log').read_text(encoding='utf-8-sig')
assert '2 passed; 0 failed' in stdout and '2351 filtered out' in stdout
stderr = (l / 'r534-targeted-tests-repair1.stderr.log').read_text(encoding='utf-8-sig')
exe = t / 'r534-focused-tests.exe'
assert not exe.exists()
shutil.copyfile(p / re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)', stderr).group(1), exe)
td = t / 'tools'; td.mkdir()
for n in ['build-r534-native-terrain.ps1', 'seal-r534-native-terrain.py']:
    shutil.copyfile(p / 'tools' / n, td / n)
save(l / 'r534-accounting-scope-correction.json', {
    'initial_accounting_preserved': True,
    'explanation': 'Initial build wrapper retained R533 generic synthetic_coordinates=true/walkability_proof=false labels. R534 uses actual Native witness coordinates and a synthetic offline clock; the detailed new check proves static terrain reachability, not full gameplay admission.',
    'actual_raw_sectors_read': 12, 'actual_witness_coordinates': True,
    'synthetic_clock': True, 'static_terrain_route_reachability_proven': True,
    'dynamic_occupancy_and_full_gameplay_proven': False,
    'sealer_preflight_error_preserved': 'One inline Python unmatched closing parenthesis failed before any filesystem mutation; corrected in the saved sealer. No build/test replay.'})
names = ['r534-route-adoption.json', 'r534-pet-race-route-server-root.json',
         'r534-production-source-snapshot.json', 'r534-final-production-source-snapshot.json',
         'r534-native-terrain-comparison.json', 'r534-accounting-scope-correction.json',
         'r534-library-build.jsonl', 'r534-library-build.stderr.log', 'r534-library-build-accounting.json',
         'r534-targeted-tests.stdout.log', 'r534-targeted-tests.stderr.log', 'r534-targeted-tests-accounting.json',
         'r534-targeted-tests-repair1.stdout.log', 'r534-targeted-tests-repair1.stderr.log',
         'r534-targeted-tests-repair1-accounting.json']
refs = [l / n for n in names] + [td / n for n in ['build-r534-native-terrain.ps1', 'seal-r534-native-terrain.py']]
refs += [t / 'source/src/modern_lito' / n for n in ['mod.rs', 'native_pet_race_terrain.rs', 'native_pet_race_offline.rs', 'native_pet_race_terrain_tests.rs']]
inputs = p / 'target/r534-native-pet-race-route-inputs'
refs += [inputs / n for n in ['manifest.json', 'reader-and-witness-contract.json', 'native-race-route-witness.json', 'native-spawn-connectors-and-boundary-rejections.json', 'adopted-r533-component-contract.json']]
report = {
    'schema': 'MODERN_R534_NATIVE_TERRAIN_OFFLINE_RACE_CONSUMER_V1',
    'state': 'ACTUAL_NATIVE_S32_LAYER3_AND_DIRECTED_ROUTE_COMPOSITION_CPU_VERIFIED',
    'updated_at': datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat(),
    'namespace': 'NATIVE_LITO_LTMAP_V1', 'adopted_owner_payload_files_checked': 54,
    'production_library': pin(p / 'target/r534-native-pet-race-terrain/library/liblito_modern.rlib'),
    'focused_test_executable': pin(exe), 'source_count': 649,
    'prior_sources_unchanged': 645, 'existing_source_changed': 'modern_lito/mod.rs',
    'new_sources': ['native_pet_race_terrain.rs', 'native_pet_race_offline.rs', 'native_pet_race_terrain_tests.rs'],
    'source_policies': ['Parse and pin original Native S32; derive collision directly from parsed Layer3, not exported masks.',
                        'Native sub_48B640 eight-direction/cardinal formulas, destination standing and same-cell standing; missing sector/bounds closed.',
                        'Keep existing Shared/pure-parser WorldMap policy and canonical digest unchanged.',
                        'OfflineRace terrain validation precedes mutation; panel before reducer progress matches source GameHub order; retirement drains local commands.'],
    'checks': c, 'tests': {'passed': 2, 'failed': 0, 'seconds': 1.61, 'old_tests_filtered': 2351,
                         'old_R533_96_cases_replayed': False, 'initial_test_compile_failed': 'E0277 fixed byte-array LowerHex',
                         'initial_tests_executed': 0, 'repair': 'test-only digest byte equality; no regular library rebuild'},
    'builds': {'regular_library': 1, 'test_compiles': 2, 'regular_warnings': 291, 'new_component_warnings': 0, 'final_test_warnings': 92},
    'remaining': ['Live same-epoch roster/RUNNING/movement command/clock authority and dynamic occupancy.',
                  'FIRST/SECOND_STAGE Brave/actor/body/action timing and server form/buff/effect expiry.',
                  'Server result/rewards/finish-return, wire publication and full lifecycle; new outer contract will be separately adopted next.',
                  'Actual player GUI and renderer LOADED; background named scene/draw binding still unknown.'],
    'preservation': {'R532_R533_frozen_artifacts_changed': False, 'R533_reducer_source_changed': False,
                     'initial_R534_source_and_failures_retained': True, 'other_group_writes': False},
    'execution': {'Java_SQL_TLS_provider_GUI': 0, 'original_BIN': False, 'old_PASS_replays': 0,
                  'new_player_EXE': False, 'own_ASCII_scratch_only': True},
    'new_full_bindings': 0, 'complete_modes': 16, 'total_modes': 40, 'GUI': 0, 'LOADED': False,
    'references': [pin(f) for f in refs], 'overall_completed': False}
save(root, report)
human = p / 'Native-R534_真Native地形與離線賽跑接線交接.txt'
with human.open('x', encoding='utf-8') as s:
    s.write('R534：真Native地形與R533 reducer離線接線\n\n'
            '新交付54檔核對；12原始Native S32讀入，98304個Layer3 bit0值與實際Java catalog逐byte一致。\n'
            '49152格Native masks／393216方向邊全部一致，未拿mask當地形輸入。保留既有WorldMap規則，新增Native standing／八方向／原地步進規則。\n'
            '30spawn、26panel、30connector、260spawn-panel、30spawn-fourlaps全部重播；111449離線地形檢查步、2180順序門檻、34拒絕／跨sector等witness。\n'
            '2新CPU checks PASS／2351filtered／1.61秒；R53396PASS未重跑。首次test-only LowerHex编譯錯誤保留，改為直接比較bytes後首次執行PASS，產品library不重編。\n'
            '649份來源另封；1既有mod／3新來源、645不變。R532／R533封存保留。\n'
            '仍16/40、GUI0、LOADEDfalse。動態occupancy、同epoch roster/RUNNING／clock、人物效果／speed／wire／終局返回仍缺整合，不把靜態route當完整模式驗收。\n\n主交接：' + str(root) + '\n')
print(json.dumps({'handoff': pin(root), 'library': report['production_library'], 'new_checks_passed': 2, 'terrain_checked_steps': 111449}, ensure_ascii=False))
