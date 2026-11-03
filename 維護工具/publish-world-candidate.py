"""Publish only a completed, single-EXE world regression; preserve schema39 pins."""
import datetime,hashlib,json,re,struct,sys
from pathlib import Path
root=Path(__file__).resolve().parent.parent
def load(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with p.open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def rel(p):return Path(p).relative_to(root).as_posix()
ip=Path(sys.argv[1]).resolve();assert ip.is_relative_to(root/'驗證紀錄/current/candidate-regression')
index=load(ip);assert index['status']=='verified_current_candidate_subset' and index['tcp_scenarios']==56
assert index['rust']['passed']==220 and index['rust']['failed']==0 and index['rust']['suites']==46
log=ip.parent/'rust.log';assert sha(log)==index['rust']['log_sha256']
tests=re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;',log.read_text(encoding='utf-8-sig'))
assert len(tests)==46 and sum(int(x) for x,y in tests)==220 and all(int(y)==0 for x,y in tests)
pin=load(root/'驗證紀錄/current/reference-snapshot.json')
assert sha(Path(pin['source']))==sha(Path(pin['snapshot']))==pin['sha256']==index['jar_sha256']
exe=root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe';exe_sha=sha(exe)
assert exe_sha==index['exe_sha256']
data=exe.read_bytes();pe=struct.unpack_from('<I',data,0x3c)[0]
assert data[pe:pe+4]==b'PE\0\0' and struct.unpack_from('<H',data,pe+4)[0]==0x8664
single=[];peers={};recovery={}
for category in ['single_cases','peer_profiles','recovery_modes']:
    for row in index[category]:
        p=Path(row['receipt']);assert p.is_relative_to(root) and sha(p)==row['sha256']
        r=load(p);assert r['exe_sha256']==exe_sha and r['jar_sha256']==pin['sha256'] and r['status'].startswith('verified')
        if category=='single_cases':
            assert len(r['cases'])==row['count'];single.extend(dict(group=row['group'],mode=c['mode'],evidence=rel(p)) for c in r['cases'])
        elif category=='peer_profiles':
            comparison=p.parent/'authority-comparison.json';a=load(comparison)
            assert a['receipt_sha256']==sha(p) and a['exe_sha256']==exe_sha and a['jar_sha256']==pin['sha256']
            peers[row['profile']]=dict(evidence=rel(p),comparison=rel(comparison),comparison_sha256=sha(comparison))
        else:recovery[row['mode']]=dict(evidence=rel(p))
assert len(single)==43 and set(peers)==set(['melee','heal','bolt','bolt-refused','wall','wall-refused','ranged','item-heal','doors','movement','honor-respawn'])
assert set(recovery)=={'room','world'}
current=root/'驗證紀錄/20261012/current-build-receipt.json';previous=load(current);assert previous['schema']==39
archive=root/'驗證紀錄/20261012/native-real-peers-build-receipt.json';assert load(archive)==previous
out=ip.parent/'published-world';assert not out.exists();out.mkdir()
files=[]
for folder in ['src','tests']:
    files.extend(p for p in (root/'LiTo381-Modern'/folder).rglob('*') if p.is_file() and p.suffix in ['.rs','.java','.json'])
files.extend(root/'LiTo381-Modern'/n for n in ['Cargo.toml','Cargo.lock','build.rs'])
files.append(root/'啟動現代客戶端.ps1')
files.extend(p for p in (root/'維護工具').iterdir() if p.is_file() and p.suffix in ['.py','.ps1'])
pins=[dict(path=rel(p),sha256=sha(p),bytes=p.stat().st_size) for p in sorted(set(files))]
write(out/'source-pins.json',pins)
receipt=dict(schema=40,date='2026-10-13',utc=datetime.datetime.now(datetime.timezone.utc).isoformat(),
    status='verified_current_world_subset',project=str(root),formal_reference=pin,source_still_matches_snapshot=True,
    exe=dict(path=str(exe),sha256=exe_sha,bytes=exe.stat().st_size,machine='0x8664 AMD64'),
    tests=dict(rust_passed=220,rust_failed=0,rust_suites=46,debug_isolated_tcp_cases=56,single_client_scenarios=43,
        two_player_scenarios=11,two_player_melee_rounds=2,controller_recovery_scenarios=2,release_current_cases=0),
    scope='One BBC59308 Debug EXE: existing action/recovery regression plus real two-player arena doors, eight-direction movement, player overlap, metadata and two natural honor respawns. Full world modes/gameplay/resources, GUI, audio, Release and longevity remain open.',
    index=rel(ip),index_sha256=sha(ip),single_cases=single,peer_profiles=peers,recovery=recovery,
    source_pins=dict(path=rel(out/'source-pins.json'),sha256=sha(out/'source-pins.json'),count=len(pins)),
    previous_baseline=rel(archive),gui_verified=False,audio_device_verified=False,product_complete=False,goal_complete=False)
write(out/'receipt.json',receipt)
write(root/'驗證紀錄/20261012/native-world-integration-build-receipt.json',receipt);write(current,receipt)
q=load(root/'工作佇列.json');q['last_verified_baseline']=dict(schema=40,evidence=rel(out/'receipt.json'),tests=receipt['tests'],current_working_tree_verified=True)
q['formal_reference'].update(validation_state='verified_current_debug_subset',evidence=rel(out/'receipt.json'),latest_source_verified_scope='220 Rust / 56 Debug TCP scenarios on one EXE; full product and Release incomplete')
g=next(i for i in q['items'] if i['id']=='G04-multiplayer-world');g['evidence'].append(rel(out/'receipt.json'))
for profile,check in [('doors','G04.2'),('movement','G04.1')]:
    next(c for c in g['checks'] if c['id']==check)['exe_sha256']=exe_sha
    g['evidence'].append(peers[profile]['comparison'])
next(i for i in q['items'] if i['id']=='G03-current-regression')['evidence'].append(rel(out/'receipt.json'))
q['goal_complete']=False;write(root/'工作佇列.json',q)
summary_path=root/'驗證紀錄/current/native-world-integration/summary.json';summary=load(summary_path)
summary['previous_qualified_runs']=summary['qualified_runs']
summary.update(last_baseline_schema=40,last_baseline_is_current_working_tree=True,
    qualified_runs=[dict(profile=n,exe_sha256=exe_sha,evidence=peers[n]['comparison']) for n in ['doors','movement','honor-respawn']],
    in_progress=['held input lifetime and coalesced movement restrictions','remaining mode-specific world lifecycles and obstacles'])
write(summary_path,summary)
p=load(root/'監製進度.json');p.update(tests_run=220,tests_failed=0,native_tcp_cases=56,last_published_results_schema=40,
    current_working_tree_verified=True,previous_build_receipt=rel(archive),current_candidate_evidence=rel(out/'receipt.json'),
    current_development_evidence=rel(ip),results_scope=receipt['scope'],next_task=q['next_task'])
p['remaining'][0]='schema40 已發布：同一 BBC59308 Debug EXE 的 220 Rust／56 TCP 限定通過；Release、完整模式與整體產品仍未驗收。'
write(root/'監製進度.json',p)
print('Published schema40 limited world regression; full product remains incomplete.')
