"""Reconcile version-qualified peer evidence; never declares the product complete."""
import datetime
import hashlib
import json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
out=root/'驗證紀錄/current/native-real-peers'
def read(p): return json.loads(p.read_text(encoding='utf-8-sig'))
def write(p,v): p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
def sha(p):
    with p.open('rb') as f: return hashlib.file_digest(f,'sha256').hexdigest().upper()
pin=read(root/'驗證紀錄/current/reference-snapshot.json')
existing_queue=read(root/'工作佇列.json')
if next(x for x in existing_queue['items'] if x['id']=='G03-current-regression')['complete']:
    raise SystemExit('G03 already has a finalized unified receipt. Preserve its completion; reconcile later work under the active goal.')
current=sha(root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe')
matrix=read(out/'regression-matrix.json')
verified={}
for receipt in sorted(out.glob('2026*/receipt.json')):
    r=read(receipt); comparison=receipt.parent/'authority-comparison.json'
    if r['jar_sha256']!=pin['sha256'] or not comparison.is_file(): continue
    a=read(comparison)
    if a['receipt_sha256']!=sha(receipt): continue
    profile=r.get('action_profile','melee')
    verified[profile]={'profile':profile,'exe_sha256':r['exe_sha256'],'jar_sha256':r['jar_sha256'],
        'receipt':str(receipt.relative_to(root)),'evidence':str(comparison.relative_to(root)),
        'matches_current_exe':r['exe_sha256']==current}
for row in matrix['legacy_cases']:
    if row.get('exe_sha256'):
        row['current_status']='verified_on_recorded_build'
        row['matches_current_exe']=row['exe_sha256']==current
    mode=row['mode'].removeprefix('room-world-')
    if mode in verified and mode!='melee':
        e=verified[mode]
        row.update(current_status='verified_real_peer_replacement',replacement='real_peer_'+mode,
            evidence=e['evidence'],exe_sha256=e['exe_sha256'],matches_current_exe=e['matches_current_exe'],
            scope='actual_player_action_with_declared_initial_fixture')
for row in matrix['additional_cases']:
    if row['mode']=='two_real_peers_two_rounds' and 'melee' in verified:
        e=verified['melee'];row.update(status='verified_on_recorded_build',evidence=e['evidence'],
            exe_sha256=e['exe_sha256'],matches_current_exe=e['matches_current_exe'])
    elif row.get('exe_sha256'):
        row.update(status='verified_on_recorded_build',matches_current_exe=row['exe_sha256']==current)
matrix['remaining_unverified_legacy_count']=sum(row['current_status']=='pending' for row in matrix['legacy_cases'])
matrix['all_current_cases_passed']=False
matrix['additional_acceptance_gaps']=[
    'Unify required regression results on the final candidate EXE after action migration.',
    'Verify controller recovery after actual room/battle disconnect and reconnect; transport-only recovery is narrower.',
    'Keep multiplayer collision, full skill reachability, GUI, Release and longevity gates in G04-G11.'
]
matrix['observed_debug_exe_sha256']=current
write(out/'regression-matrix.json',matrix)
write(out/'action-index.json',{'updated_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),
    'current_exe_sha256':current,'profiles':list(verified.values()),'all_seven_action_profiles_verified':
    all(p in verified for p in ['heal','bolt','bolt-refused','wall','wall-refused','ranged','item-heal'])})
q=read(root/'工作佇列.json')
g2=next(x for x in q['items'] if x['id']=='G02-real-peer-battle')
g2['findings']=[
    '初次隔離雙玩家被正式核心以同隊攻擊拒絕；已補 C_D3／S_D4 換隊與 GUI 按鈕，GUI 點擊仍待 G08。',
    '兩場互換勝敗、GAME epoch 2、世界世代 1→2、死亡確認與比分已完成；雙端／伺服器權威交叉比對通過。',
    '其後發現同時離房順序競態；已補退休成員的一次性通知消耗、原始封包雙順序重播，43 相關測試與雙場重驗通過。',
    '修復後即時重跑未再觀察到相同競態順序；該順序本身由捕獲封包確定性重播驗證。'
]
g3=next(x for x in q['items'] if x['id']=='G03-current-regression')
g3.update(state='in_progress',complete=False)
g3['findings']=[f"舊 54 情境已映射；尚有 {matrix['remaining_unverified_legacy_count']} 個互動情境未完成遷移。各證據均記錄 EXE，不能相加宣稱單一新版全通過。",
    '新增加真正房間／戰鬥斷線後控制器恢復驗收；既有 transport 世代測試不足以代替。']
for name in ['action-index.json','race-fix-evidence/index.json']:
    path='驗證紀錄/current/native-real-peers/'+name
    if path not in g3['evidence']:g3['evidence'].append(path)
q['formal_reference'].update(evidence='驗證紀錄/current/native-real-peers/action-index.json',
    latest_source_verified_scope='BBC59308 Debug version-qualified peer/melee and action subsets; current unified regression pending')
write(root/'工作佇列.json',q)
progress=read(root/'監製進度.json')
progress.update(formal_server='BBC59308: real-peer melee and version-qualified subsets; unified current regression in progress',
    optimized_release_current_tcp_cases=0,optimized_profile_tested=False,
    current_development_evidence='驗證紀錄/current/native-real-peers/action-index.json')
write(root/'監製進度.json',progress)
print('Refreshed peer matrix:',matrix['remaining_unverified_legacy_count'],'legacy migrations pending;',len(verified),'verified profiles across recorded builds.')
