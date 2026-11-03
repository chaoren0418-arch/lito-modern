"""Publish the completed, version-qualified G02/force regression baseline once."""
raise SystemExit('Obsolete draft: use finalize-current-candidate.py with the verified unified index. Do not combine earlier EXE results.')
from pathlib import Path
import datetime, hashlib, json, re, struct
root=Path(__file__).resolve().parent.parent;out=root/'驗證紀錄/current/native-real-peers'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
current=root/'驗證紀錄/20261012/current-build-receipt.json'
assert read(current)['schema']==38,'Already published or baseline changed; do not reset progress'
pin=read(root/'驗證紀錄/current/reference-snapshot.json');expected=pin['sha256']
assert sha(Path(pin['source']))==sha(Path(pin['snapshot']))==expected
exe=root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';exe_sha=sha(exe)
data=exe.read_bytes();pe=struct.unpack_from('<I',data,0x3c)[0]
assert data[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',data,pe+4)[0]==0x8664
rust=read(out/'rust-current-summary.json');assert rust['passed']==201 and rust['failed']==0 and rust['suites']==40
cases=read(out/'debug-current-evidence/index.json');assert cases['total']==34 and cases['exe_sha256']==exe_sha and cases['jar_sha256']==expected
matches=[]
for p in out.glob('2026*/receipt.json'):
    r=read(p)
    if r.get('exe_sha256')==exe_sha and r.get('source_matches_snapshot') and r['jar_sha256']==expected:
        a=read(p.parent/'authority-comparison.json')
        assert a['receipt_sha256']==sha(p)
        matches.append((p,r,a))
assert matches,'Final EXE has no completed two-peer authority comparison'
p,peer,authority=sorted(matches,key=lambda x:str(x[0]))[-1]
files=[]
for folder in ['src','tests']:
    files += [p for p in (root/'LiTo381-Modern'/folder).rglob('*') if p.is_file() and p.suffix in {'.rs','.java','.json'}]
files += [root/'LiTo381-Modern'/n for n in ['Cargo.toml','Cargo.lock','build.rs']]
files += [root/'啟動現代客戶端.ps1']
pins=[dict(path=str(p.relative_to(root)),sha256=sha(p),bytes=p.stat().st_size) for p in sorted(set(files))]
write(out/'source-pins.json',pins)
receipt=dict(schema=39,date='2026-10-13',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    project=str(root),status='verified_current_native_force_and_real_peer_subset',
    exe=dict(path=str(exe),sha256=exe_sha,bytes=exe.stat().st_size,machine='0x8664 AMD64'),
    formal_reference=pin,source_sha256_at_finalize=sha(Path(pin['source'])),source_still_matches_snapshot=True,
    tests=dict(rust_passed=201,rust_failed=0,rust_suites=40,debug_isolated_tcp_cases=35,
               single_client_scenarios=34,two_player_scenarios=1,two_player_rounds=2,
               actual_peer_sessions=4,peer_pngs=8,release_current_cases=0),
    scope='Current BBC59308 Debug: room force request/authority and two-player melee/result/reentry plus 34 existing scenarios. Full gameplay, GUI, audio device, Release and longevity incomplete.',
    debug_cases='驗證紀錄/current/native-real-peers/debug-current-evidence/index.json',
    peer_evidence=str(p.relative_to(root)),authority_comparison=str((p.parent/'authority-comparison.json').relative_to(root)),
    source_pins=dict(path='驗證紀錄/current/native-real-peers/source-pins.json',sha256=sha(out/'source-pins.json'),count=len(pins)),
    historical_release='驗證紀錄/20261012/native-emblems-build-receipt.json (EBF5EE66 only)',
    first_failure='驗證紀錄/current/native-real-peers/20261013-055206-695/a-client.json',
    unresolved_matrix='驗證紀錄/current/native-real-peers/regression-matrix.json',
    gui_verified=False,audio_device_verified=False,product_complete=False,goal_complete=False)
write(out/'summary.json',receipt)
write(root/'驗證紀錄/20261012/native-real-peers-build-receipt.json',receipt)
write(current,receipt)
qpath=root/'工作佇列.json';q=read(qpath)
q['last_verified_baseline']=dict(schema=39,evidence='驗證紀錄/current/native-real-peers/summary.json',tests=receipt['tests'],current_working_tree_verified=True)
q['formal_reference'].update(validation_state='verified_current_debug_subset',evidence='驗證紀錄/current/native-real-peers/summary.json',
    latest_source_verified_scope='201 Rust / 34 single-client scenarios / 1 real-two-player scenario (two rounds), same Debug EXE; full gameplay and Release unverified')
write(qpath,q)
pp=root/'監製進度.json';progress=read(pp)
progress.update(tests_run=201,tests_failed=0,native_tcp_cases=35,last_published_results_schema=39,
    previous_build_receipt='驗證紀錄/20261012/native-emblems-build-receipt.json',
    current_candidate_evidence='驗證紀錄/current/native-real-peers/summary.json',
    current_development_evidence=receipt['authority_comparison'],
    results_scope=receipt['scope'],optimized_profile_tested=False,optimized_release_current_tcp_cases=0,
    formal_server='BBC59308 current Debug subset verified; full compatibility incomplete')
write(pp,progress)
print('Published schema39: 201 Rust / 35 Debug scenarios including two actual-player rounds; all remaining goals stay open.')
