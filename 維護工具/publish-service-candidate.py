"""Qualify one completed service/attribute candidate, including separate boundaries."""
import datetime
import hashlib
import json
import re
import struct
import sys
from pathlib import Path

root = Path(__file__).resolve().parent.parent
def load(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p, value): p.write_text(json.dumps(value, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f, 'sha256').hexdigest().upper()
def rel(p): return Path(p).relative_to(root).as_posix()
def equal(a, b): return a.upper() == b.upper()

ip = Path(sys.argv[1]).resolve()
assert ip.is_relative_to(root/'驗證紀錄/current/candidate-regression')
i = load(ip)
assert i['status'] == 'verified_current_candidate_subset'
assert i['tcp_scenarios'] == 61 and i['observed_boundary_cases'] == 2
assert (i['rust']['passed'], i['rust']['failed'], i['rust']['suites']) == (245, 0, 54)
assert equal(sha(ip.parent/'rust.log'), i['rust']['log_sha256'])
tests = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', (ip.parent/'rust.log').read_text(encoding='utf-8-sig'))
assert len(tests) == 54 and sum(int(a) for a,b in tests) == 245 and all(int(b)==0 for a,b in tests)
inputs = ip.parent/'development-input-pins.json'
assert equal(sha(inputs), i['development_input_pins_sha256'])
pins = load(inputs)
assert equal(pins['exe_sha256'], i['exe_sha256'])
assert len(pins['files']) == i['development_input_files_verified']
current_inputs=set()
for folder in ['LiTo381-Modern/src','LiTo381-Modern/tests','維護工具']:
    current_inputs.update(rel(p) for p in (root/folder).rglob('*') if p.is_file() and p.suffix in {'.rs','.java','.json','.py','.ps1'})
for name in ['LiTo381-Modern/Cargo.toml','LiTo381-Modern/Cargo.lock','LiTo381-Modern/build.rs','啟動現代客戶端.ps1']:
    if (root/name).is_file():current_inputs.add(name)
assert current_inputs==set(pins['files']), 'Development input inventory changed'
for name, digest in pins['files'].items():
    p = (root/name).resolve()
    assert p.is_relative_to(root) and equal(sha(p), digest), name
reference = load(root/'驗證紀錄/current/reference-snapshot.json')
assert sha(Path(reference['source'])) == sha(Path(reference['snapshot'])) == reference['sha256'] == i['jar_sha256']
exe = root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
assert equal(sha(exe), i['exe_sha256'])
data = exe.read_bytes(); pe = struct.unpack_from('<I', data, 0x3c)[0]
assert data[pe:pe+4] == b'PE\0\0' and struct.unpack_from('<H', data, pe+4)[0] == 0x8664

names = {'room-admin':'room-admin', 'classic-entry':'classic-entry', 'rental-entry':'rental-entry',
         'rental-shop':'rental-entry', 'rental-return':'rental-return'}
results = {}; single = []
for category in ['single_cases','peer_profiles','recovery_modes','boundary_observations']:
    results[category] = []
    for row in i[category]:
        p = Path(row['receipt']).resolve(); assert p.is_relative_to(root)
        assert equal(sha(p), row['sha256'])
        r = load(p)
        assert equal(r['exe_sha256'], i['exe_sha256']) and equal(r['jar_sha256'], i['jar_sha256'])
        assert r['status'].startswith('observed' if category=='boundary_observations' else 'verified')
        item = dict(row); item['receipt'] = rel(p)
        if category == 'single_cases':
            assert len(r['cases']) == row['count']
            single.extend(dict(group=row['group'],mode=c['mode'],evidence=rel(p)) for c in r['cases'])
        if category in ['peer_profiles','boundary_observations']:
            profile = row['profile']
            comparison = p.parent/(names.get(profile,'authority')+'-comparison.json')
            a = load(comparison); assert equal(a['receipt_sha256'], sha(p))
            item.update(comparison=rel(comparison),comparison_sha256=sha(comparison))
            if profile.startswith('rental-'):
                ap = p.parent/'attributes-comparison.json'; ar = load(ap)
                assert ar['status']=='verified_schema0_received_attributes' and equal(ar['receipt_sha256'],sha(p))
                item.update(attributes_comparison=rel(ap),attributes_sha256=sha(ap))
            if profile=='rental-shop': assert a['shop_verified'] is True
            if profile=='classic-entry': assert a['ordinary_world_verified'] is False
            if profile=='rental-return': assert a['normal_return_verified'] is False and a['reentry_verified'] is False
        results[category].append(item)
assert len(single)==43
assert {x['profile'] for x in i['peer_profiles']} == {'melee','heal','bolt','bolt-refused','fire-arrow','fire-arrow-refused','wall','wall-refused','ranged','item-heal','doors','movement','honor-respawn','room-admin','rental-entry','rental-shop'}
assert {x['mode'] for x in i['recovery_modes']} == {'room','world'}
assert {x['profile'] for x in i['boundary_observations']} == {'classic-entry','rental-return'}
current = root/'驗證紀錄/20261012/current-build-receipt.json'
previous = root/'驗證紀錄/20261012/native-world-integration-build-receipt.json'
assert load(current)['schema']==40 and load(previous)==load(current)
out = ip.parent/'published-services'; assert not out.exists(); out.mkdir()
receipt = dict(schema=41,date='2026-10-13',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    status='verified_current_service_attribute_subset',project=str(root),formal_reference=reference,
    source_still_matches_snapshot=True,exe=dict(path=str(exe),sha256=sha(exe),bytes=exe.stat().st_size,machine='0x8664 AMD64'),
    tests=dict(rust_passed=245,rust_failed=0,rust_suites=54,debug_isolated_tcp_cases=61,
        single_client_scenarios=43,two_player_scenarios=16,two_player_melee_rounds=2,
        controller_recovery_scenarios=2,observed_boundary_cases=2,release_current_cases=0),
    scope='One BBC59308 Debug EXE: room initialization force-order repair, Fire Arrow15 class4 flight and refusal, corrected adjacent Fire Wall probe, room administration, rental map500 entry and one merchant transaction, schema0 attributes, existing world/actions and reconnect cases. Classic C05 and rental restart remain separately observed incomplete boundaries. Full gameplay/resources, GUI, audio device, Release and longevity are open.',
    index=rel(ip),index_sha256=sha(ip),single_cases=single,results=results,
    source_pins=dict(path=rel(inputs),sha256=sha(inputs),count=len(pins['files'])),previous_baseline=rel(previous),
    gui_verified=False,audio_device_verified=False,all_assets_verified=False,product_complete=False,goal_complete=False)
write(out/'receipt.json',receipt)
write(root/'驗證紀錄/20261012/native-service-integration-build-receipt.json',receipt)
write(current,receipt)
q = load(root/'工作佇列.json')
q['last_verified_baseline']=dict(schema=41,evidence=rel(out/'receipt.json'),tests=receipt['tests'],current_working_tree_verified=True)
q['formal_reference'].update(validation_state='verified_current_debug_subset',evidence=rel(out/'receipt.json'),latest_source_verified_scope='245 Rust / 61 Debug TCP, plus 2 separately observed incomplete boundaries; full product and Release incomplete')
g=next(x for x in q['items'] if x['id']=='G03-current-regression')
g.update(state='verified_defined_scope',complete=True,current_candidate_scope=receipt['scope'])
g['evidence'].append(rel(out/'receipt.json'))
q.update(next_task='G08-player-gui',goal_complete=False,
    current_focus=dict(item='G08-player-gui',work='接上已具控制器證據的房間建立／設定／觀戰／成員管理，再接租裝服務控件',next='GUI事件與控制器防護、狀態更新、真實操作驗收；其他技能及服務返回缺口仍保留'))
write(root/'工作佇列.json',q)
p=load(root/'監製進度.json')
p.update(tests_run=245,tests_failed=0,native_tcp_cases=61,last_published_results_schema=41,
    current_working_tree_verified=True,previous_build_receipt=rel(previous),current_candidate_evidence=rel(out/'receipt.json'),
    current_development_evidence=rel(ip),results_scope=receipt['scope'],next_task=q['next_task'])
p['remaining'][0]='schema41限定範圍通過；C05與租裝restart仍為未完成邊界，完整玩法／資源、GUI、音效、Release與長時間驗收繼續。'
write(root/'監製進度.json',p)
print('Published schema41 limited candidate; remaining goals stay active.')
