"""Publish the actual GUI case stop without granting unverified GUI scope."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, re
R = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser(); ap.add_argument('case', type=Path); ap.add_argument('--session', required=True, type=int)
a = ap.parse_args()
case = a.case.resolve(strict=True)
assert case.is_relative_to(R / '驗證紀錄/current/native-catalog-sky-gui')
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
c = read(case / 'cohort.json'); q = read(R / '工作佇列.json'); e = q['current_native_catalog_gui_active']
assert e['case'] == str(case) and e['original_session_id'] == a.session
assert q['current_full_default_library_verified'] and not e['GUI_verified']
assert e['source_manifest_sha256'] == c['source_manifest_sha256']
report = R.parent / 'LiTo381-總開發報告.md'; old = report.read_bytes(); body = old.decode('utf-8-sig')
out = case / ('report-progress-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')); out.mkdir()
(out / 'before.md').write_bytes(old)
now = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
body, n = re.subn(r'^更新：.*?。', '更新：' + now + '。', body, count=1, flags=re.MULTILINE); assert n == 1
message = f'正常 GUI 原 session {a.session}：{c["profile"]}／{c["scenario"]}，實際狀態 {c["status"]}。當版兩版廣域與完整 library 預設資格保留；GUI 全流程尚未整體驗收，確認自測後才交人工。'
body, n = re.subn(r'^- \*\*本次停點：.*$', '- **本次停點：' + message + '**', body, count=1, flags=re.MULTILINE); assert n == 1
if c.get('automation_scope') == 'accessibility-keyboard':
    limitation = '- GUI 擷取限制：Sky 在重新選取、啟用／最大化及重設連線後仍出現 FrameArrived／window capture timeout，座標輸入回報 geometry unavailable。原跨端點案例已失敗封存；單視窗探針只有建立／關閉 exit 0，沒有取得畫面或執行遊戲。本批只推進可存取性文字與鍵盤操作，不能宣告畫面、滑鼠、IME／多 DPI 或 GUI 全流程通過。'
    if re.search(r'^- GUI 擷取限制：.*$', body, flags=re.MULTILINE):
        body = re.sub(r'^- GUI 擷取限制：.*$', limitation, body, count=1, flags=re.MULTILINE)
    else:
        at = body.index('\n\n## 2.')
        body = body[:at] + '\n' + limitation + body[at:]
starts = [m.start() for m in re.finditer(r'^\*\*(?:library 原批次已結束|GUI 正常視窗自動測試正從)', body, flags=re.MULTILINE)]
assert len(starts) == 1
end = body.index('\n正常建置使用', starts[0])
body = body[:starts[0]] + f'''**GUI 正常視窗自動測試正從原 session {a.session} 接續；先核對原批次，勿重啟。**

- [目前 GUI 原始收據](<{(case / 'cohort.json').as_posix()}>); profile={c['profile']}，scenario={c['scenario']}，status={c['status']}。
- [目前 GUI checkpoint](<{Path(q['last_goal_turn_evidence']).with_name('readback.json').as_posix()}>); pipeline PID {c['pipeline']['pid']}，建立時間 {c['pipeline']['creation_utc']}。工具 session 無法連回時，先以 PID＋建立時間和收據核對，避免重複啟動。
- 使用 computer-use／Sky 操作唯一匹配的自有測試視窗；自行輸入隔離測試帳密，逐段核對 UI、實際封包及角色狀態。原始失敗保留，正常 EXE 與 fixture 診斷資格分開。
- 本版 GUI 仍須逐項完成兩版跨端點、失敗／取消恢復、同端點租裝、旅館、完整玩家操作、IME／多 DPI／音效等必要範圍；一案結束不代表整體合格。正常程序與服務均終止後才能解除該案保留及套用必要修正。
''' + body[end:]
assert report.read_bytes() == old
(out / 'after.md').write_text(body, encoding='utf-8', newline='\n'); report.write_bytes((out / 'after.md').read_bytes())
assert report.read_bytes() == (out / 'after.md').read_bytes()
receipt = dict(status='GUI_case_progress_report_updated', session=a.session, case=str(case), actual_status=c['status'],
    before_sha256=hashlib.sha256(old).hexdigest().upper(), after_sha256=hashlib.sha256(report.read_bytes()).hexdigest().upper(),
    GUI_qualified=False, whole_goal_complete=False)
(out / 'readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps(dict(report=str(report), readback=str(out / 'readback.json'), actual_GUI_case_status=c['status']), ensure_ascii=False))
