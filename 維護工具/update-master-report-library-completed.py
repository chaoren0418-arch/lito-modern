"""Publish completed library scope, with the next GUI action and exact evidence."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, re
R = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(R / '驗證紀錄/current/native-catalog-library-default')
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
q = read(R / '工作佇列.json')
e = q['current_native_catalog_library']
assert e['run'] == str(run) and e['observed_actual_exit_code'] == 0
assert e['recorded_original_owned_processes_all_terminal'] and not e['GUI_verified']
assert not q['active_workloads'] and not q['source_and_corresponding_QA_frozen']
profiles = e['profiles']
counts = '／'.join(p + str(profiles[p]['passed']) + ' 通過、0 失敗、' + str(profiles[p]['ignored']) + ' 忽略' for p in ['Debug', 'Release'])
for profile in ['Debug', 'Release']:
    p = profiles[profile]
    data = Path(p['qualification']).read_bytes()
    assert hashlib.sha256(data).hexdigest().upper() == p['qualification_sha256']
    assert read(p['qualification'])['source_manifest_sha256'] == e['source_manifest_sha256']
report = R.parent / 'LiTo381-總開發報告.md'
old = report.read_bytes()
body = old.decode('utf-8-sig')
out = run / ('report-completed-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'before.md').write_bytes(old)
now = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
body, n = re.subn(r'^更新：.*?。', '更新：' + now + '。', body, count=1, flags=re.MULTILINE)
assert n == 1
body, n = re.subn(r'^- \*\*本次停點：.*$',
    f'- **本次停點：當版 Debug／Release 廣域與完整 library 預設案例已完成並獨立核對；下一步開始正常 GUI 全流程自動測試及必要修正。** library 原 session {e["original_session_id"]} 已實際 exit 0，記錄程序均終止；忽略本文未執行，GUI 尚未驗收，G01–G12 仍未整體完成。',
    body, count=1, flags=re.MULTILINE)
assert n == 1 and '| 完整 library 預設全庫 |' not in body
marker = '| 跨服務控制器 |'
at = body.index(marker)
row = f'| 完整 library 預設全庫 | {counts} | [整批完成核對](<{(run / "completion-observation.json").as_posix()}>); 忽略本文未執行，GUI 另驗 |\n'
body = body[:at] + row + body[at:]
body = body.replace('廣域通過不涵蓋完整 library。', '廣域與 library 各自依本版紀錄合格，均不代表忽略本文或 GUI 通過。')
body = body.replace('| G03 完整回歸 | 執行當版 Debug／Release 完整 library；盤點忽略案例，必要案例另行具備資源並實跑 |',
                    '| G03 完整回歸 | 當版兩版預設全庫已合格；必要忽略案例仍須具備資源另行實跑，產品變更後重新建立對應資格 |')
start = body.index('**完整 library 正沿原 session ')
end = body.index('\n正常建置使用', start)
body = body[:start] + f'''**library 原批次已結束，請從正常 GUI 自動測試接續，不重跑已合格的未變更來源。**

- library：{counts}；[原始收據](<{(run / 'cohort.json').as_posix()}>)、[整批核對](<{(run / 'completion-observation.json').as_posix()}>)。下一階段不得將這批結果推廣成 GUI 通過。
- GUI 啟動前執行維護工具 prepare-native-catalog-sky-gui.py，參數為上述 library run；確認來源與正常兩版 EXE 未變，再以 start-native-catalog-sky-gui-case.py 啟動各 profile／scenario。這兩個工具只準備隔離服務及自有正常程序；實際視窗輸入一律透過 computer-use 的 Sky API。
- 首先完成兩版正常登入→租裝→雙玩家→跨端點→新場景／房間→登出，接著逾時、拒絕、錯服務、取消重試／較晚取消／到期，以及同端點、旅館、IME／多 DPI／音效等必要範圍。每案保留原始失敗並獨立核對；確認無誤後才交人工測驗。
- 若佇列已有 GUI 活動批次，先核對原 PID＋建立時間、收據與原 session，接續原批次；不得重複啟動。
''' + body[end:]
assert report.read_bytes() == old
(out / 'after.md').write_text(body, encoding='utf-8', newline='\n')
report.write_bytes((out / 'after.md').read_bytes())
assert report.read_bytes() == (out / 'after.md').read_bytes()
receipt = dict(status='completed_library_scope_report_updated', report=str(report), source=e['source_manifest_sha256'],
    original_session=e['original_session_id'], profiles=profiles, before_sha256=hashlib.sha256(old).hexdigest().upper(),
    after_sha256=hashlib.sha256(report.read_bytes()).hexdigest().upper(), GUI_qualified=False, whole_goal_complete=False)
(out / 'readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps(dict(report=str(report), counts=counts, receipt=str(out / 'readback.json')), ensure_ascii=False))
