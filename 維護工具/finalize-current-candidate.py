"""Publish the verified candidate subset, never the unfinished whole product."""
import datetime, hashlib, json, re, struct, sys
from pathlib import Path
root=Path(__file__).resolve().parent.parent
out=root/'驗證紀錄/current/native-real-peers'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def rel(p):return str(Path(p).relative_to(root))
ip=Path(sys.argv[1]).resolve();index=read(ip)
assert ip.is_relative_to(root/'驗證紀錄/current/candidate-regression')
assert index['status']=='verified_current_candidate_subset' and index['tcp_scenarios']==53
assert index['rust']['passed']==204 and index['rust']['suites']==41 and index['rust']['failed']==0
assert sha(ip.parent/'rust.log')==index['rust']['log_sha256']
tests=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', (ip.parent/'rust.log').read_text(encoding='utf-8-sig'))
assert len(tests)==41 and sum(int(x) for x,y in tests)==204 and all(int(y)==0 for x,y in tests)
pin=read(root/'驗證紀錄/current/reference-snapshot.json'); expected=pin['sha256']
assert sha(Path(pin['source']))==sha(Path(pin['snapshot']))==expected==index['jar_sha256']
exe=root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'; exe_sha=sha(exe)
assert exe_sha==index['exe_sha256']
data=exe.read_bytes();pe=struct.unpack_from('<I',data,0x3c)[0]
assert data[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',data,pe+4)[0]==0x8664
single=[];peers={};recovery={}
for category in ['single_cases','peer_profiles','recovery_modes']:
    for row in index[category]:
        p=Path(row['receipt']);assert p.is_relative_to(root) and sha(p)==row['sha256']
        r=read(p);assert r['exe_sha256']==exe_sha and r['jar_sha256']==expected and r['status'].startswith('verified')
        if category=='single_cases':
            assert len(r['cases'])==row['count']
            single += [dict(group=row['group'],mode=c['mode'],evidence=rel(p)) for c in r['cases']]
        elif category=='peer_profiles':
            a=read(p.parent/'authority-comparison.json');assert a['receipt_sha256']==sha(p)
            peers[row['profile']]=dict(evidence=rel(p),comparison=rel(p.parent/'authority-comparison.json'))
        else:recovery[row['mode']]=dict(evidence=rel(p))
assert len(single)==43 and set(peers)=={'melee','heal','bolt','bolt-refused','wall','wall-refused','ranged','item-heal'}
assert set(recovery)=={'room','world'}
current=root/'驗證紀錄/20261012/current-build-receipt.json';assert read(current)['schema']==38
files=[]
for folder in ['src','tests']:
    files += [p for p in (root/'LiTo381-Modern'/folder).rglob('*') if p.is_file() and p.suffix in {'.rs','.java','.json'}]
files += [root/'LiTo381-Modern'/n for n in ['Cargo.toml','Cargo.lock','build.rs']]+[root/'啟動現代客戶端.ps1']
files += [root/'維護工具'/n for n in ['run-current-candidate-regression.ps1','verify-native-real-peers.ps1','verify-real-peer-actions.py','verify-real-peer-authority.py','verify-native-recovery.ps1']]
pins=[dict(path=rel(p),sha256=sha(p),bytes=p.stat().st_size) for p in sorted(set(files))];write(out/'source-pins.json',pins)
receipt=dict(schema=39,date='2026-10-13',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    status='verified_current_real_peer_and_recovery_subset',project=str(root),
    exe=dict(path=str(exe),sha256=exe_sha,bytes=exe.stat().st_size,machine='0x8664 AMD64'),formal_reference=pin,source_still_matches_snapshot=True,
    tests=dict(rust_passed=204,rust_failed=0,rust_suites=41,debug_isolated_tcp_cases=53,single_client_scenarios=43,
        two_player_scenarios=8,two_player_melee_rounds=2,controller_recovery_scenarios=2,release_current_cases=0),
    scope='One BBC59308 Debug candidate: 43 retained/self/ground cases, eight real-peer scenarios and two actual GAME disconnect/relogin scenarios. Full multiplayer/gameplay/resources, GUI, audio device, Release and longevity remain open.',
    index=rel(ip),index_sha256=sha(ip),single_cases=single,peer_profiles=peers,recovery=recovery,
    source_pins=dict(path=rel(out/'source-pins.json'),sha256=sha(out/'source-pins.json'),count=len(pins)),
    historical_release='驗證紀錄/20261012/native-emblems-build-receipt.json (EBF5EE66 only)',
    gui_verified=False,audio_device_verified=False,product_complete=False,goal_complete=False)
matrix=read(out/'regression-matrix.json')
for row in matrix['legacy_cases']:
    matches=[x for x in single if x['mode']==row['mode'] and (x['group']==row['group'] or row['mode'] not in ['accept','reject','retry'])]
    profile=row['mode'].removeprefix('room-world-')
    if matches:row.update(evidence=matches[0]['evidence'],current_status='verified_current_candidate')
    elif profile in peers:row.update(evidence=peers[profile]['comparison'],current_status='verified_real_peer_replacement')
    else:
        assert row['replacement']=='two_real_peers_two_rounds',(row['mode'],row)
        row.update(evidence=peers['melee']['comparison'],current_status='replacement_verified_limited')
    row.update(exe_sha256=exe_sha,matches_current_exe=True)
matrix.update(remaining_unverified_legacy_count=0,all_current_cases_passed=True,observed_debug_exe_sha256=exe_sha,
    additional_acceptance_gaps=['Full multiplayer, gameplay, GUI, resource/audio, Release and longevity remain governed by G04-G11.'],unified_candidate_evidence=rel(ip))
write(out/'summary.json',receipt);write(root/'驗證紀錄/20261012/native-real-peers-build-receipt.json',receipt);write(current,receipt)
write(out/'regression-matrix.json',matrix)
q=read(root/'工作佇列.json');q['last_verified_baseline']=dict(schema=39,evidence=rel(out/'summary.json'),tests=receipt['tests'],current_working_tree_verified=True)
q['formal_reference'].update(validation_state='verified_current_debug_subset',evidence=rel(out/'summary.json'),
    latest_source_verified_scope='204 Rust / 53 current Debug TCP scenarios on one recorded EXE; full product/Release incomplete')
g3=next(x for x in q['items'] if x['id']=='G03-current-regression')
g3.update(state='verified_defined_scope',complete=True,findings=['舊 54 情境逐一映射；5 個 bot 切片由 1 個雙玩家雙場情境涵蓋。單一 EXE 的現行 53 情境與 204 Rust 通過。',
    '房間／戰鬥 GAME 斷線後同控制器重登通過；世界世代不重用，舊畫面不能授權新世界。'])
g3['evidence'].append(rel(ip));next(x for x in q['items'] if x['id']=='G04-multiplayer-world')['state']='in_progress'
q['next_task']='G04-multiplayer-world';q['goal_complete']=False;write(root/'工作佇列.json',q)
progress=read(root/'監製進度.json');progress.update(tests_run=204,tests_failed=0,native_tcp_cases=53,last_published_results_schema=39,
    previous_build_receipt='驗證紀錄/20261012/native-emblems-build-receipt.json',current_candidate_evidence=rel(out/'summary.json'),
    current_development_evidence=rel(ip),results_scope=receipt['scope'],optimized_profile_tested=False,optimized_release_current_tcp_cases=0,
    formal_server='BBC59308 current Debug subset verified; full compatibility incomplete',next_task=q['next_task'])
write(root/'監製進度.json',progress)
print('Published schema39: 204 Rust / 53 Debug scenarios on one EXE. Continue G04; overall goal incomplete.')
