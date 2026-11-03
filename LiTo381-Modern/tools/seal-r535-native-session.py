import datetime
import hashlib
import json
import pathlib
import re
import shutil

p = pathlib.Path(__file__).resolve().parent.parent
l = p / 'logs/continuation-20261003'
t = p / 'target/r535-native-pet-race-session'
root = l / 'r535-native-session-lifecycle-consumer-handoff.json'
assert not root.exists()
def read(f): return json.loads(f.read_text(encoding='utf-8-sig'))
def pin(f):
    h = hashlib.sha256()
    with f.open('rb') as s:
        for b in iter(lambda: s.read(1024 * 1024), b''): h.update(b)
    return {'path': str(f), 'bytes': f.stat().st_size, 'sha256': h.hexdigest().upper()}
def save(f, v):
    with f.open('x', encoding='utf-8') as s:
        json.dump(v, s, ensure_ascii=False, indent=2); s.write('\n')

preserved_roots = {
    'r532-coverage-witness-correction-handoff.json': '7BB72B2DB3B800F14C286E88B45A20EB5D8D32C217E5ADDB115DA7DE8F315119',
    'r533-full-backlog-and-pet-race-handoff.json': 'F0098FFE958638E2C52678E28970456709253861CB7843D5F6EABC8172E54AFA',
    'r534-native-terrain-offline-consumer-handoff.json': 'A14ED497A194DFEAFB535774170D1629139A6B9B4D84CF8165E265C03977FAFF'}
for n, digest in preserved_roots.items():
    assert pin(l / n)['sha256'] == digest

out = (l / 'r535-targeted-tests.stdout.log').read_text(encoding='utf-8-sig')
match = re.search(r'test result: ok\. (\d+) passed; (\d+) failed; .*?(\d+) filtered out; finished in ([0-9.]+)s', out)
assert match and match.group(1, 2) == ('2', '0'), out
err = (l / 'r535-targeted-tests.stderr.log').read_text(encoding='utf-8-sig')
exepath = re.search(r'Running unittests .*?\(([^\r\n]+\.exe)\)', err).group(1)
exe = t / 'r535-focused-tests.exe'
assert not exe.exists()
shutil.copyfile(p / exepath, exe)
source = read(l / 'r535-production-source-snapshot.json')
for d in source['files']:
    assert pin(p / d['path'])['sha256'] == d['sha256']
    assert pin(t / 'source' / d['path'])['sha256'] == d['sha256']
buildrows = [json.loads(s) for s in (l / 'r535-library-build.jsonl').read_text(encoding='utf-8-sig').splitlines()]
warnings = [r['message'] for r in buildrows if r.get('reason') == 'compiler-message' and r['message']['level'] == 'warning']
new_warnings = [w for w in warnings if any('native_pet_race_session' in s['file_name'] for s in w['spans'])]
assert not new_warnings
test_warnings = re.search(r'generated (\d+) warnings', err)
assert test_warnings
assert 'native_pet_race_session.rs:' not in err and 'native_pet_race_session_tests.rs:' not in err
assert read(l / 'r535-library-build-accounting.json')['exit_code'] == 0
assert read(l / 'r535-targeted-tests-accounting.json')['exit_code'] == 0
life = read(l / 'r535-session-lifecycle-check.json')
errors = read(l / 'r535-session-atomic-errors-check.json')
assert life['wire_snapshot']['view'] == 'Disconnected' and errors['snapshot_atomicity']
td = t / 'tools'; td.mkdir()
tools = ['prepare-r535-native-session.py', 'build-r535-native-session.ps1', 'seal-r535-native-session.py']
for n in tools: shutil.copyfile(p / 'tools' / n, td / n)
names = ['r535-outer-server-root.json', 'r535-outer-adoption.json',
         'r535-roomlist-source-supplement.json', 'r535-production-source-snapshot.json',
         'r535-D9-cleanup-server-root.json', 'r535-D9-cleanup-evidence-adoption.json',
         'r535-library-build.jsonl', 'r535-library-build.stderr.log', 'r535-library-build-accounting.json',
         'r535-targeted-tests.stdout.log', 'r535-targeted-tests.stderr.log', 'r535-targeted-tests-accounting.json',
         'r535-session-lifecycle-check.json', 'r535-session-atomic-errors-check.json']
refs = [l / n for n in names] + [td / n for n in tools]
refs += [t / 'source/src/modern_lito' / n for n in ['mod.rs', 'native_pet_race_session.rs', 'native_pet_race_session_tests.rs']]
outer = p / 'target/r535-native-pet-race-outer-inputs'
refs += [outer / n for n in ['manifest.json', 'outer-lifecycle-contract-v1.json', 'existing-wire-event-registry.json', 'source-excerpt-and-order-bindings.json', 'version-binding.json', 'match-boundary-results.json']]
refs += list((p / 'target/r535-native-pet-race-cleanup-evidence').iterdir())
terrain = p / 'target/r534-native-pet-race-route-inputs'
refs += [terrain / n for n in ['adopted-r533-component-contract.json', 's32-layer3-to-actual-reader-byte-comparison.json']]
refs += sorted((terrain / 'native/566').glob('*.s32'))
refs += [l / 'r534-native-terrain-offline-consumer-handoff.json']
now = datetime.datetime.now(datetime.timezone(datetime.timedelta(hours=8))).isoformat()
report = {
    'schema': 'MODERN_R535_NATIVE_SESSION_OUTER_LIFECYCLE_CONSUMER_V1',
    'state': 'OFFLINE_TYPED_WIRE_PREDICTION_RESULT_RETURN_AND_LOCAL_EPOCH_CPU_VERIFIED',
    'updated_at': now, 'namespace': 'NATIVE_LITO_LTMAP_V1',
    'profile': {'ClientSession_source': '013ADEF1613C627243ACB352DCA186E0668EFABFADA17FF597380E9B65DECEC9',
                'GameHub_source': '98E03203851000CA23461ABEDABC0BD3012B014FB01CD3C445748258DB59EC7D',
                'outer_root': '5ABC831237E0E73FBFC44AF22290EECC8545DBC9F7DF9E1CAB8E10AE5D381824',
                'outer_contract': 'DD31F3D3DADD8AC42F26D91A8808A255AA6C407B442110E010E05C8ADAF8020B',
                '485F_overlay_equivalence': False},
    'production_library': pin(t / 'library/liblito_modern.rlib'), 'focused_test_executable': pin(exe),
    'source_count': source['source_count'], 'prior_sources_unchanged': source['prior_sources_unchanged'],
    'existing_sources_changed': source['existing_sources_changed'], 'new_sources': source['new_sources'],
    'outer_payload_files_verified': 63,
    'source_policies': ['Wire FD0/1/2/3, D3, DB, F0, D0 and bounded CC0 fields separate from local terrain-checked prediction.',
                        'Prepared six-byte C_MOVE carries source position and one heading; no socket framing/write or server gate ACK.',
                        'Private receipt/command/confirm epochs revoke on roster replacement, result, matching exit or disconnect.',
                        'FD3 name/time cannot establish identity, placement or local terminal; raw lap values cannot advance local gates.',
                        'F0 retains RESULT room; offline one-use confirm intent plus matching D0 then CC0 row yields a local return view.',
                        'Active D0 producer, exact server generation, clock/deadline and atomic membership authority remain unqualified.'],
    'bounded_subset': {'roster_max': 10, 'name_raw_bytes_max': 1024, 'body_bytes_max': 16384, 'report_rows_max': 64,
                       'existing_CC0_decoder': 'native_java_room_catalog::decode shape only, independently source-supplemented',
                       'zero_room_rows_have_no_completion_marker': True, 'untagged_delayed_wire_epoch_resolved': False},
    'checks': {'lifecycle': life, 'atomic_errors': errors},
    'tests': {'passed': int(match[1]), 'failed': int(match[2]), 'old_tests_filtered': int(match[3]), 'seconds': float(match[4]),
              'old_R533_96_R534_routes_or_server_MatchState_tests_replayed': False},
    'builds': {'regular_library': 1, 'test_compiles': 1, 'regular_warnings': len(warnings),
               'new_component_warnings': len(new_warnings), 'test_warnings': int(test_warnings[1])},
    'server_cleanup_candidate_evidence': read(l / 'r535-D9-cleanup-evidence-adoption.json'),
    'remaining': ['Live same-epoch roster/RUNNING, server clock, movement admission and dynamic occupancy.',
                  'Actor/form/Brave/effect state, rendering timing and server expiry publishers.',
                  'Candidate clears transient state but notification IO leaves room battleStarted and BATTLE phase; full exit/reentry unproven.',
                  'Result rewards/persistence, atomic server terminal/return, zero-row return completion and untagged late packet generation.',
                  'Player GUI/renderer LOADED, background named scene/draw position, remaining mode qualification.'],
    'execution': {'Java_SQL_TLS_provider_GUI': 0, 'new_player_EXE': False, 'old_PASS_replays': 0, 'own_ASCII_scratch_only': True},
    'preservation': {'R532_R533_R534_frozen_artifacts_changed': False, 'prior_root_pins_checked': preserved_roots,
                     'server_or_BIN_written': False, 'historical_outer_contract_changed': False},
    'new_full_bindings': 0, 'complete_modes': 16, 'total_modes': 40, 'GUI': 0, 'LOADED': False,
    'references': [pin(f) for f in refs], 'overall_completed': False}
save(root, report)
human = p / 'Native-R535_離線賽跑生命週期與封包欄位交接.txt'
with human.open('x', encoding='utf-8') as s:
    s.write('R535：Native PetRace 離線生命週期與封包欄位\n\n'
            '新增session元件接R534真Native地形與R533 reducer，分開本地預測、wire欄位與本地epoch。FD3只有名稱和時間，不當名次／本人完成；FD2不快轉gate。\n'
            'D3／自己同名D0觀測／斷線退役舊命令；名單改變不能復活舊prediction。只有RESULT提供離房confirm，F0保留結果房間；本地confirm意圖、同房同名D0與後續CC0行到齊才進本地返回頻道view。無標記晚到封包世代與零房間列表完成仍未知。\n'
            f'2新CPU checks PASS、{match[3]}舊測試filtered、{match[4]}秒；{errors["malformed_prefix_or_domain_rejections"]}截短／不合規輸入原子拒絕。R53396、R534routes及伺服器8案未重跑。\n'
            f'651來源另封，648舊來源不變、mod一處及2新來源；新regular library1次、test compile1次，新warning0。\n'
            '新server D9清效果候選核對10引用／672manifest；僅map566兩session memory fixture實際方法，IO失敗仍留BATTLE／戰鬥房間。6次prepare需fixture重設room／phase，不能当完整重入。未切換013A歷史契約，也不推論485F backend已採用。\n'
            '仍16/40、GUI0、LOADEDfalse。這是離線元件驗證；沒有啟動Java／SQL／TLS／provider或玩家視窗，沒有實際socket傳送。\n\n主交接：' + str(root) + '\n')

progress_path = p.parent / '監製進度.json'
progress = read(progress_path)
progress['updated_at'] = now
progress['phase'] = 'R535_Native_offline_session_lifecycle_CPU_verified_live_authority_and_server_IO_cleanup_open'
progress['overall_status'] = 'in_progress'
progress['overall_completed'] = False
progress['current_task'] = 'R535新離線session完成：63outer payload、651來源／648旧不變、2新CPU PASS，prediction/wire/私有epoch分層，結果及返回守門。新D9候選10引用／672manifest核對，只清效果，IO仍留BATTLE／room；未切換契約／485F。R533與R534 PASS未重跑，仍16/40、GUI0、LOADEDfalse。'
progress['next_step'] = '消費server IO離房修正的精確候選交接後續證據；live同epoch roster/RUNNING／clock／occupancy、actor／Brave效果、終局獎勵與零行房間列表／未標記晚到封包、背景scene綁定與GUI仍待來源或合格執行，不能宣告Modern全完成。'
progress['r535'] = {'status': report['state'], 'handoff': pin(root), 'library': report['production_library'],
                    'source_count': 651, 'new_checks_passed': 2, 'old_tests_filtered': int(match[3]),
                    'new_component_warnings': 0, 'D9_candidate_manifest_verified': 672,
                    'server_IO_exit_gap_remaining': True, 'new_full_bindings': 0, 'complete_modes': 16, 'total_modes': 40, 'GUI': 0, 'LOADED': False}
progress['completed'].append('R535：離線Native session／prediction-wire分離／同epoch守門／result-return完成2新CPU checks，651來源另封，server D9清理候選不當完整退出，GUI0。')
progress['verified'].append(f'R535 2PASS0FAIL／{match[3]}filtered／{match[4]}s，{errors["malformed_prefix_or_domain_rejections"]}malformed原子拒絕，新warnings0，10候選引用／672manifest全核對。')
with progress_path.open('w', encoding='utf-8') as s:
    json.dump(progress, s, ensure_ascii=False, indent=2); s.write('\n')
print(json.dumps({'handoff': pin(root), 'library': report['production_library'], 'tests': report['tests'], 'warnings': report['builds']}, ensure_ascii=False))
