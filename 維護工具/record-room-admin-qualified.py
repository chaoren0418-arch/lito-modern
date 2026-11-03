"""Record a bounded same-binary room-service result, without replacing schema40."""
import hashlib,json
from pathlib import Path
root=Path(__file__).resolve().parent.parent
def read(p):return json.loads((root/p).read_text(encoding='utf-8-sig'))
def save(p,v):(root/p).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
room='驗證紀錄/current/native-real-peers/20261013-102418-875'
melee='驗證紀錄/current/native-real-peers/20261013-102502-033'
a,b=read(room+'/receipt.json'),read(melee+'/receipt.json')
assert a['exe_sha256']==b['exe_sha256'] and a['jar_sha256']==b['jar_sha256']
assert read(room+'/room-admin-comparison.json')['status']=='verified'
exe=root/'LiTo381-Modern/target/current/x86_64-pc-windows-msvc/debug/LiToModern.exe'
assert hashlib.sha256(exe.read_bytes()).hexdigest().upper()==a['exe_sha256']
evidence='驗證紀錄/current/native-world-integration/room-admin-qualified.json'
save(evidence,{'schema':1,'scope':'G06 room service subset','exe_sha256':a['exe_sha256'],'jar_sha256':a['jar_sha256'],
 'rust_tests':12,'rust_suites':4,'rust_log':'驗證紀錄/current/native-world-integration/rust-room-admin-fixed.log',
 'live':[room+'/receipt.json',room+'/room-admin-comparison.json',melee+'/receipt.json'],
 'verified':['create/password rejection/correct join','edit name/capacity/options/remove password and reset readiness','observer on/off and local ready guard','owner transfer','kick accepted/departure/rejected rejoin','two subsequent melee rounds regression'],
 'not_verified':['player GUI','all room modes and admission refusals','rental and ordinary world','full current-binary regression','Release']})
q=read('工作佇列.json');g=next(i for i in q['items'] if i['id'].startswith('G06'))
g['checks'][0].update(state='verified_controller_live_subset',qualified_scope='建立密碼房、錯誤密碼拒絕、正確加入、修改名稱／容量／選項／移除密碼、準備重置、房主轉移、踢人及重入拒絕；全部模式與其他准入拒絕及 GUI 待驗',evidence=evidence)
g['checks'][1]['qualified_scope']='死鬥切榮譽、兩隊開戰已有證據；房間內觀戰開關及禁止觀戰者準備已由正式雙端驗證。實際觀戰場景需兩名參戰者加觀戰者，仍待驗。'
g['findings'].append('房間管理 14 份客戶端快照與被動 GameHub 狀態相符；同 EXE 雙場近戰回歸通過。結果保存在 room-admin-qualified.json，非 G06 全分支完成。')
if evidence not in g['evidence']:g['evidence'].append(evidence)
q['last_verified_baseline']['development_note']='schema40 保留；後續 G04.5 與 G06 房間管理有分別綁定 EXE 的限定證據，新工作樹不等同已通過全回歸。'
save('工作佇列.json',q)
print(a['exe_sha256'])
