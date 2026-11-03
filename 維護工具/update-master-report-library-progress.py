"""Update only the current stop and continuation pointers, retaining prior bytes."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import argparse, hashlib, json, re

ROOT = Path(__file__).resolve().parents[1]
ap = argparse.ArgumentParser()
ap.add_argument('run', type=Path)
ap.add_argument('--session', type=int, required=True)
a = ap.parse_args()
run = a.run.resolve(strict=True)
assert run.is_relative_to(ROOT / '驗證紀錄/current/native-catalog-library-default')
c = json.loads((run / 'cohort.json').read_text('utf-8-sig'))
q = json.loads((ROOT / '工作佇列.json').read_text('utf-8-sig'))
assert c['source_manifest_sha256'] == q['current_product_identity']['source_manifest_sha256']
assert q['active_workload_sessions'] == {'full_library': a.session}
assert c['status'] in ['compiling_current_full_library_test_image', 'running_current_full_library_default_selection']
report = ROOT.parent / 'LiTo381-總開發報告.md'
old = report.read_bytes()
body = old.decode('utf-8-sig')
out = run / ('report-progress-' + datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'))
out.mkdir()
(out / 'before.md').write_bytes(old)
now = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
body, count = re.subn(r'^更新：.*?。', '更新：' + now + '。', body, count=1, flags=re.MULTILINE)
assert count == 1
body, count = re.subn(r'^- \*\*本次停點：.*$',
    f'- **本次停點：當版完整 library 的 Debug／Release 批次已啟動，原始 session {a.session}；目前 {c["active_profile"]}，狀態 {c["status"]}。** 來源與對應 QA 固定；原廣域兩版已合格，完整 library 尚未整批通過。接著完成正常 GUI 全流程自動測試與必要修正，自測確認後才交人工測驗。',
    body, count=1, flags=re.MULTILINE)
assert count == 1
starts = [m.start() for m in re.finditer(r'^\*\*(?:第一個待執行動作是完整 library，|完整 library 正沿原 session )', body, flags=re.MULTILINE)]
assert len(starts) == 1
start = starts[0]
end = body.index('\n正常建置使用', start)
body = body[:start] + f'''**完整 library 正沿原 session {a.session} 執行，請接續原批次，勿重啟。**

- 原始收據：[cohort.json](<{(run / 'cohort.json').as_posix()}>)；目前階段 {c['active_profile']}／{c['status']}。
- 已登記的目標：先當版 Debug／Release 完整 library，再正常 GUI 全流程自動測試與必要修正，再完成剩餘 G01–G12 與最終來源 Release 回歸；代理自測確認後才安排人工測驗。
- pipeline PID {c['pipeline_pid']}，建立時間 {c['pipeline_creation_utc']}。新對話先核對原 PID＋建立時間、收據及工作佇列，不能因工具 session 無法連回就另開重複批次。
- 結束後使用維護工具目錄 qualify-native-catalog-library-default.py（--profile Debug／Release）、observe-native-catalog-library-default.py 與 checkpoint-native-catalog-library-completed.py 核對並更新狀態；參數先查 --help。忽略本文未跑不得算通過。
''' + body[end:]
qualified_profiles = []
for profile in ['Debug', 'Release']:
    path = run / profile / 'full-default-qualified/qualified.json'
    if not path.exists(): continue
    p = json.loads(path.read_text('utf-8-sig'))
    assert p['source_manifest_sha256'] == c['source_manifest_sha256']
    assert p['failed'] == p['filtered_out'] == p['Rust_exit_code'] == 0
    qualified_profiles.append(profile + str(p['passed']) + ' 通過／0 失敗／' + str(p['ignored']) + ' 忽略')
if qualified_profiles:
    marker = '來源與對應 QA 固定；原廣域兩版已合格，完整 library 尚未整批通過。'
    assert marker in body
    body = body.replace(marker, '來源與對應 QA 固定；已獨立核對 ' + '、'.join(qualified_profiles) + '；另一版與整批終止核對仍待。')
assert report.read_bytes() == old
staged = out / 'after.md'
staged.write_text(body, encoding='utf-8', newline='\n')
report.write_bytes(staged.read_bytes())
assert report.read_bytes() == staged.read_bytes()
sha = lambda b: hashlib.sha256(b).hexdigest().upper()
receipt = dict(status='library_progress_report_updated', original_session=a.session,
               source_manifest_sha256=c['source_manifest_sha256'], report=str(report),
               before_sha256=sha(old), after_sha256=sha(report.read_bytes()),
               archived_before=str(out / 'before.md'), library_qualified=False, GUI_qualified=False)
(out / 'readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2)+'\n', encoding='utf-8')
print(json.dumps(receipt, ensure_ascii=False))
