"""Replace the current stop with the new source, retaining the prior qualified table."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import argparse,hashlib,json,re
R=Path(__file__).resolve().parents[1];ap=argparse.ArgumentParser();ap.add_argument('run',type=Path);ap.add_argument('--session',required=True,type=int);a=ap.parse_args()
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
run=a.run.resolve(strict=True);assert run.is_relative_to(R/'驗證紀錄/current/native-keyboard-focus-normal')
c=read(run/'cohort.json');q=read(R/'工作佇列.json');source=c['source_manifest_sha256']
assert q['active_workload_sessions']=={'catalog_normal':a.session}and q['current_product_identity']['source_manifest_sha256']==source
assert q['source_and_corresponding_QA_frozen']and not q['current_full_default_library_verified']
report=R.parent/'LiTo381-總開發報告.md';old=report.read_bytes();body=old.decode('utf-8-sig')
out=run/('report-progress-'+datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f'));out.mkdir();(out/'before.md').write_bytes(old)
date=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
body,n=re.subn(r'^更新：.*?。','更新：'+date+'。',body,count=1,flags=re.MULTILINE);assert n==1
stop=f'- **本次停點：鍵盤焦點修正已套用一個正常來源檔；新來源 {source[:8]} 的正常 Debug／Release 建置與既有 services／teleport 16 案冒煙回歸，正沿原 session {a.session} 執行，狀態 {c["status"]}。** 修正前 2DAF 的兩版 library 已實跑合格；修正版的相關／廣域／完整 library 與正常 GUI 資格仍待。Sky 畫面擷取問題尚未恢復，未達人工測驗條件。'
body,n=re.subn(r'^- \*\*本次停點：.*$',lambda _:stop,body,count=1,flags=re.MULTILINE);assert n==1
body=body.replace('## 2. 當版基準與已完成範圍','## 2. 修正前合格基準與目前變更')
body=body.replace('當版來源識別為來源清單檔 SHA-256（1,047 個來源檔、1,048 個編譯輸入）：','以下為修正前 **2DAF 版本**的合格來源與產物（1,047 個來源檔、1,048 個編譯輸入），不代表焦點修正版已合格：')
body=body.replace('| 當版驗證 | 實際結果 | 資格限制／證據 |','| 2DAF 版本驗證 | 實際結果 | 資格限制／證據 |')
body=body.replace('最近正常程式修正：','先前同端點正常程式修正：')
marker='| 2DAF 版本驗證 |'
at=body.index(marker)
focus=f'''目前正常來源為 **{source}**，僅變更 native_login_window.rs：前景自有視窗的控制項被停用、焦點空缺時，恢復父視窗鍵盤焦點；保留有效控制項焦點、背景視窗與關閉流程。原 GUI 中已記錄到來源選擇後無焦點且 Tab 不前進；[隔離 Win32 API 證據](<{(R/'驗證紀錄/current/native-keyboard-focus-probe/20261019-130334-204342/receipt.json').as_posix()}>) 已重現原行為並通過候選四項條件，但正常修正版 GUI 尚未驗收。

[來源套用與回復證據](<{(Path(c['application'])/'application.json').as_posix()}>)、[目前正常建置批次](<{(run/'cohort.json').as_posix()}>)。舊紀錄中的 target/current 路徑會由新建置更新，修正前 EXE 以原 products.zip 與 SHA 追溯；不能用舊路徑名稱推定目前二進位版本。

'''
assert '目前正常來源為 **'not in body;body=body[:at]+focus+body[at:]
body=body.replace('| G03 完整回歸 | 當版兩版預設全庫已合格；必要忽略案例仍須具備資源另行實跑，產品變更後重新建立對應資格 |','| G03 完整回歸 | 2DAF 兩版預設全庫已合格；焦點修正版須重新完成正常建置、相關冒煙、廣域與完整 library，必要忽略案例另外實跑 |')
starts=[m.start()for m in re.finditer(r'^\*\*GUI 正常視窗自動測試正從',body,flags=re.MULTILINE)];assert len(starts)==1
end=body.index('\n正常建置使用',starts[0])
body=body[:starts[0]]+f'''**焦點修正版正常建置正沿原 session {a.session} 執行；接續原批次，勿重啟。**

- [原始正常建置收據](<{(run/'cohort.json').as_posix()}>)；source={source}，runner PID {c['runner']['pid']}，建立時間 {c['runner']['creation_utc']}。先核對原收據／PID＋建立時間與工作佇列，避免重複啟動。
- 原 session 實際終止後，使用維護工具 qualify-native-keyboard-focus-normal.py 與 checkpoint-native-keyboard-focus-normal.py 核對兩版正常 EXE、每版 16 既有協定冒煙案、來源、原始輸出與程序終止；參數先查 --help。焦點 API 四條件是隔離證據，正常 GUI 另驗。
- 再建立新來源的廣域與完整 library Debug／Release 資格，然後繼續正常 GUI 全流程與焦點修正回歸。2DAF 的 1969／1969 全庫及 236 服務 sessions 保留原版本限定資格，不能直接改成修正版結果。
- 已失敗的 GUI 原 session 48480／69439 與 capture-only 探針 43968 均已結束，所有記錄程序身分終止；其原始收據保留在 native-catalog-sky-gui。探針 exit 0 只代表正常視窗建立／關閉，沒有取得圖像或執行 GAME；擷取限制尚待解決。
- 開發期間不請使用者登入／尋找視窗；自動驗證確認後才安排人工測驗。完整 GUI、滑鼠、IME／多 DPI／音效裝置及其他 G01–G12 均不得先標完成。
'''+body[end:]
assert report.read_bytes()==old;(out/'after.md').write_text(body,encoding='utf-8',newline='\n');report.write_bytes((out/'after.md').read_bytes())
receipt=dict(status='keyboard_focus_current_source_report_updated',source_manifest_sha256=source,run=str(run),original_session=a.session,
    before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=hashlib.sha256(report.read_bytes()).hexdigest().upper(),current_source_library_qualified=False,GUI_qualified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(receipt,ensure_ascii=False))
