"""Compact source-aware handoff after the one-file accessibility repair."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def link(label,path):return'['+label+'](<'+Path(path).as_posix()+'>)'
q=read(R/'工作佇列.json');source=q['current_product_identity']['source_manifest_sha256']
assert source=='4F2F911B4B70BDA16C4EE8C3EDD60223F3E72A22C8A3B1B808C3F5EABA60E6A2'
application=q['current_native_accessibility_labels_application'];assert application['source_manifest_sha256']==source
check=read(Path(q['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(check)==8 and all(sha(R/n)==h for n,h in check.items())
normal=q.get('current_native_accessibility_labels_normal_qualification',{})
normal_ok=normal.get('source_manifest_sha256')==source and normal.get('qualification_granted')is True
rows=[]
if normal_ok:
    path=Path(normal['qualification']);n=read(path);assert sha(path)==normal['qualification_sha256']and n['source_manifest_sha256']==source
    rows.append('| 正常 Debug／Release | 兩版唯一入口建置完成、各 services2＋teleport14＝16過0敗0忽略 | '+link('限定建置／協定冒煙資格',path)+'；尚非 GUI |')
    for profile in ['Debug','Release']:rows.append('| '+profile+' 正常 EXE | `'+n['normal_mains'][profile]+'` | 當版正常映像 SHA-256 |')
else:rows.append('| 正常 Debug／Release | 執行中或尚待；不能使用舊版 EXE 代表修正版 | 原批次收據與下列活動狀態 |')
library=q.get('current_native_catalog_library',{})
library_ok=library.get('source_manifest_sha256')==source and q.get('current_full_default_library_verified')is True
if library_ok:
    rows.append('| 完整 library 預設 | '+ '／'.join(p+str(library['profiles'][p]['passed'])+'過0敗'+str(library['profiles'][p]['ignored'])+'忽略'for p in ['Debug','Release'])+' | '+link('完成核對',library['completion'])+'；忽略本文未執行 |')
else:rows.append('| 修正版完整 library | 尚未整批完成核對 | 預定兩版原預設全庫，不以選取測試代替 |')
if not library_ok:
    for record in q.get('active_workloads',{}).values():
        run=Path(record.get('run',record.get('case','')))
        for profile in ['Debug','Release']:
            path=run/profile/'full-default-qualified/qualified.json'
            if not path.is_file():continue
            partial=read(path)
            assert partial['source_manifest_sha256']==source and partial['failed']==partial['filtered_out']==partial['Rust_exit_code']==0
            assert partial['recorded_profile_owned_processes_terminal']
            rows.append('| '+profile+' library | '+str(partial['passed'])+'過0敗'+str(partial['ignored'])+'忽略；單版已獨立核對 | '+link('單版資格',path)+'；整批終止及另一版仍待 |')
resource=q.get('current_native_library_resource_optin',{})
resource_ok=resource.get('source_manifest_sha256')==source and resource.get('qualified')is True
if resource_ok:
    path=Path(resource['qualification']);assert sha(path)==resource['qualification_sha256']
    rows.append('| 額外原生77資源案例 | Debug／Release各10過0敗 | '+link('獨立資格',path)+'；另列，非預設忽略本文全數執行 |')
for field,label in [('current_normal_GUI_rental_entry_keyboard','正常 GUI 入村'),('current_normal_GUI_hotel_entry_keyboard','正常 GUI 旅館')]:
    for profile,record in q.get(field,{}).items():
        if record.get('source_manifest_sha256')!=source:continue
        path=Path(record['qualification']);assert sha(path)==record['qualification_sha256']
        rows.append('| '+profile+' '+label+' | 限定核對完成 | '+link('證據',path)+'；非完整 GUI |')
failure=q.get('current_normal_GUI_source_handoff_failure',{})
failure_current=failure.get('source_manifest_sha256')==source and failure.get('qualification_granted')is False
if failure_current:
    path=Path(failure['evidence']);assert sha(path)==failure['evidence_sha256']
    rows.append('| Release GUI 來源接續 | 未通過：B選測試乙，30.645秒後GAME落到測試甲；選角前停止 | '+link('失敗與正式契約',path)+'；原始exit0不代表GUI通過 |')
active=[]
for key,record in q['active_workloads'].items():
    path=Path(record.get('run',record.get('case','')));cohort=read(path/'cohort.json')
    assert cohort['source_manifest_sha256']==source
    active.append('- '+key+'：原 session **'+str(q['active_workload_sessions'][key])+'**，`'+cohort['status']+'`'+('（'+cohort['active_profile']+'）'if cohort.get('active_profile')else'')+'；'+link('原始收據',path/'cohort.json')+'。')
if not active:active=['- 無登記中的活動批次；依下一步及最新 checkpoint 接續。']
old=q['qualification_before_accessibility_labels_change'];assert old['source_manifest_sha256'].startswith('AB3CF26C')
old_lib=old['library'];old_broad=old['broad'];old_normal=old['normal']
application_path=V/'native-accessibility-labels-normal/application-20261019-164206-825853/application.json'
now=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
status_line=('當版正常兩版與完整 library 已完成獨立核對，GUI仍待接續。'if library_ok else '當版正常兩版已核對，完整 library 正在執行，GUI仍待接續。')
if failure_current:status_line+='Debug限定入村GUI已通過；Release發現初次GAME來源接續到期問題，優先修正。'
library_next=('當版完整 library 及原生77資源選取已核對；保留原預設忽略數，接續正常GUI。'if resource_ok else '當版兩版完整 library 已完成核對；接續必要原資源 opt-in，分開記錄忽略與選取範圍。'if library_ok else '接續修正版完整 library 的 Debug／Release 原預設全庫，再執行必要原資源 opt-in，分開記錄忽略與選取範圍。')
body=f'''# LiTo381 總開發報告與接續指南

更新：{now}。真正 Windows x64 遊戲核心重建持續進行；完整 G01–G12、GUI 全流程與人工驗收尚未完成。

## 1. 目前停點

已先完成 AB3C 版本兩版完整 library，再自行操作正常 GUI，完成兩版限定入村及旅館入房。GUI 找出技能選單被 Windows 誤讀為「新密碼」；現已修正一個視窗來源檔，建立 **4F2F** 修正版。{status_line}舊版通過結果不轉記新來源。

{chr(10).join(active)}

{link('最新八份接續檔 checkpoint',Path(q['last_goal_turn_evidence']).with_name('readback.json'))}；{link('工作佇列',R/'工作佇列.json')}。接續原 session／PID＋建立時間，不因新對話重啟活動批次；只依工具實際 terminal chunk 與 exit 判定終止。

## 2. 當版變更與資格

來源清單 SHA-256（1,047 個來源檔／1,048 個編譯輸入）：

```text
{source}
```

{link('套用／回復證據',application_path)}：只改 native_login_window.rs，於技能下拉前建立相鄰的「技能」標籤，調整同列寬度，背包／結算／NPC 面板隱藏標籤。原焦點修正保留；協定、遊戲邏輯、資源、8 秒／60 秒與其他計時不變。正常 UIA 名稱、鍵盤巡覽與面板切換仍須實測。

| 當版範圍 | 狀態 | 證據與限制 |
|---|---|---|
{chr(10).join(rows)}

## 3. 已完成的前版基準（AB3C，保留原版範圍）

| 範圍 | 實際結果 | 證據 |
|---|---|---|
| 正常兩版與相關冒煙 | 每版16過0敗0忽略 | {link('資格',old_normal['qualification'])} |
| 完整 library 預設 | Debug／Release 各1969過0敗484忽略，inventory2453；484個忽略本文未執行 | {link('完成核對',old_lib['completion'])} |
| 額外原生77資源案例 | 兩版各10過0敗；另外474個預設忽略本文尚未執行 | {link('獨立資格',V/'native-keyboard-focus-library-resource-optin/20261019-153227-485047/checkpoint-terminal-20261019-153333-426499/qualified.json')} |
| 廣域整合／main | Debug544／Release540過，各0敗23忽略；每版117映像 | {link('完成核對',old_broad['completion_observation'])} |
| 服務控制器 | 兩版六分支12案、96 LOGIN／GAME sessions | {link('資格',q['current_native_catalog_controller_qualification']['qualification'])} |
| 正常 GUI 限定入村 | 每版兩帳號依序入map500、各一步移動、完整背包A45／B43件開關與正常退出；合8 sessions | {link('Debug',old['GUI_entry']['Debug']['qualification'])}、{link('Release',old['GUI_entry']['Release']['qualification'])} |
| 正常 GUI 旅館入房 | 每版各13步數字鍵盤路線、原始席巴對話、8／1鑰匙→map503／501、角色／物品接續與正常退出；合8 sessions | {link('Debug',old['GUI_hotel']['Debug']['qualification'])}、{link('Release',old['GUI_hotel']['Release']['qualification'])} |

旅館數量回覆均在既定30秒內；Debug A首次請求曾到期，重新對話後成功，原始觀察保留。入村核對的重複觀察與旅館非同步回覆讀取錯誤各自保留，僅修正讀取器後重讀原結果，未用重跑覆蓋。所有已列批次皆有原始退出與記錄程序終止證據。

## 4. 下一步與剩餘開發

1. 當版正常兩版建置與每版16案已完成獨立核對，保留此資格；不因接續對話重跑。
2. {library_next}前版廣域與控制器仍保留原版，不藉修正版 library 宣告它們重驗完成。
3. 優先修正初次GAME來源接續：正式來源保留30秒，逾時可回落預設來源；擬以既有GAME C8確認原選來源，確認後才開放角色清單，保留租裝傳送／恢復自動接續。以自然超過30秒及快速交接兩種情境驗證，不延長時限。再用computer-use／Sky繼續兩玩家同場、戰鬥、租裝／旅館、傳送／取消／恢復／登出等GUI流程；不要求使用者操作測試視窗。
4. 全部必要開發、自動回歸與 Release 全流程確認後，才安排人工測驗。

當版工具已準備：{link('正常建置／核對三工具',V/'native-accessibility-labels-tooling/20261019-164237-601129/normal-tools.json')}、{link('完整library五工具',V/'native-accessibility-labels-tooling/20261019-164532-490133/library-tools.json')}、{link('資源與GUI七工具',V/'native-accessibility-labels-tooling/20261019-164631-244574/followup-tools.json')}。參數先查--help；library入口為 run-native-accessibility-labels-library-default.py <當版正常validation> --input-kind normal-related，GUI準備入口為 prepare-native-accessibility-labels-sky-gui-current.py <當版完整library cohort>（只訂正歷史失敗證據路徑，原工具保留）。非預設素質當版fixture已{link('準備原始碼',V/'native-accessibility-labels-nondefault-character-fixture/prepared-20261019-165606-505816')}，尚未編譯／實際接續驗證。同來源已核對項目保留，不因新對話重跑。

| 項目 | 尚待 |
|---|---|
| G02／G04 | 最終來源雙場戰鬥、結算、多人同步、碰撞與世界生命週期 |
| G03 | 修正版回歸；逐名稱盤點必要忽略案例及資源條件 |
| G05 | 物品、裝備、技能與狀態矩陣；歷史技能書74項辨識37／缺37，辨識不等於施法 |
| G06（優先G06.3d） | 普通CA00同端點已知失敗、非預設素質接續、旅館自然到期3600／7200秒、退租／共享鑰匙／訪客主動離房與其他服務；C10返回選角不等於回村 |
| G07 | 全地圖／動作圖／遮擋／屋頂／排序／相機與完整frame；局部診斷圖不等於正常GUI |
| G08／G09 | 兩版正常完整GUI、滑鼠、IME、多DPI、退休生命週期與實際音效裝置 |
| G10–G12 | Release全流程、乾淨主機可攜交付、效能／長時間／故障恢復及逐項完成稽核 |

旅館正常GUI已觀察到房內缺少RETURN107入口；已有{link('單檔C10返回選角候選',R/'維護工具/candidates/native-hotel-return-20261019-171731-517856/candidate.json')}，尚未套用、編譯或實測。擬在501–503房內沿用NPC按鈕位置，接既有返回選角流程，並於地圖變更刷新版面、背包開啟時隱藏入口；不是房內回村或退租／共享鑰匙通過證據。

Sky圖像擷取經重新選取、啟用／最大化、重設連線、靜態及已載入動畫世界嘗試仍回報 FrameArrived／window capture timeout；座標輸入回報 geometry unavailable。文字與鍵盤可用，沒有取得正常GUI圖像，因此圖像／滑鼠／完整GUI仍未通過。程序及輸入桌面唯讀核對為WinSta0／Default，未切換桌面或更動系統設定。

原GUI 48480／69439／5094的失敗保留；capture-only43968只有視窗建立／關閉，非GAME或圖像資格。GUI fixture操作者等待為接收900秒、租裝／旅館總長900秒、傳送／失敗1800秒；正常關閉另核對30秒。遊戲8秒、peer60秒、自然到期及G11限制不變，未授予效能資格。

## 5. 專案邊界與接續入口

重建版與登入器／伺服器為獨立專案、產物不同；相鄰專案及D:/lineage開發/0-客戶端只讀，可參考線索。替換登入器JAR的舊提案已撤回，不是本專案依賴。不碰正式玩家表或正式程序，不執行／包裝舊LiTo.bin；所有開發、測試資料與快取留在重建版。

唯一正常入口啟動現代客戶端.ps1，建置使用-BuildOnly -BuildProfile Debug／Release；Cargo固定--locked --offline -j1、x86_64-pc-windows-msvc、LiTo381-Modern/target/current。Release z／thin LTO／16 codegen-units／strip不變。計時服務不並行編譯，活動測試固定來源與對應QA。

正式參考JAR固定75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371；catalog固定B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01。測試使用重建版內的固定副本，相鄰來源不寫入。

先讀本報告，再讀{link('AGENTS.md',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}與{link('歷史',R/'docs/HISTORY.md')}；以來源SHA及最新checkpoint交叉核對，勿採用歷史PASS當作當版完成。

舊總報告Server0.7.862／Launcher0.5.54與歷史「總目標完成」只屬當時專案範圍。{link('整理前完整原文',V/'master-report-consolidation/20261019-044931-124021/LiTo381-總開發報告.整理前完整原文.md')} SHA-256為65C64E6068EB50FD3355F3C352F54ED46FA73A808A4DBA8F962979B6A9CB4176。後續修訂現況與待辦，不反覆堆疊相同進度。
'''
assert all(Path(path).exists()for path in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
old_bytes=REPORT.read_bytes();(out/'before.md').write_bytes(old_bytes);(out/'after.md').write_text(body,encoding='utf-8',newline='\n')
assert REPORT.read_bytes()==old_bytes;REPORT.write_bytes((out/'after.md').read_bytes())
assert all(sha(R/n)==h for n,h in check.items())
receipt=dict(report=str(REPORT),source_manifest_sha256=source,normal_qualified=normal_ok,library_qualified=library_ok,active_sessions=q['active_workload_sessions'],before_sha256=hashlib.sha256(old_bytes).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),whole_GUI_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(receipt,ensure_ascii=False))
