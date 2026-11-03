"""Refresh a compact current-source handoff, with prior-source results kept separate."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
def read(p):return json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def link(label,path):return'['+label+'](<'+Path(path).as_posix()+'>)'
q=read(R/'工作佇列.json');identity=q['current_product_identity'];source=identity['source_manifest_sha256']
assert source=='AB3CF26C4227A3E9E171AEA2E7765C1BA29120681BCB226B36FC53BA2635AE2D'
np=Path(q['current_native_keyboard_focus_normal_qualification']['qualification']);n=read(np)
assert sha(np)==q['current_native_keyboard_focus_normal_qualification']['qualification_sha256']and n['source_manifest_sha256']==source
fp=Path(q['current_keyboard_focus_GUI_regression']['qualification']);f=read(fp)
assert sha(fp)==q['current_keyboard_focus_GUI_regression']['qualification_sha256']and f['source_manifest_sha256']==source
assert f['actual_formal_sessions']==4 and f['normal_keyboard_focus_regression_qualified']and not f['GUI_full_flow_verified']
for profile in ['Debug','Release']:assert sha(n['profiles'][profile]['normal']['path'])==n['normal_mains'][profile]==identity[profile+'_normal_sha256']
last=Path(q['last_goal_turn_evidence']).with_name('readback.json');check=read(last)['after_files'];assert len(check)==8 and all(sha(R/name)==h for name,h in check.items())
active=q['active_workloads'];sessions=q['active_workload_sessions'];lines=[]
for key,value in active.items():
    path=Path(value.get('run',value.get('case','')));receipt=read(path/'cohort.json')
    assert receipt['source_manifest_sha256']==source
    lines.append(f'- {key}：原 session **{sessions[key]}**，狀態 `{receipt["status"]}`，profile={receipt.get("active_profile",receipt.get("profile"))}，case={receipt.get("active_scenario",receipt.get("scenario"))}；{link("原始收據",path/"cohort.json")}。')
if not lines:lines=['- 目前無登記中的活動批次；從下方下一步及最新 checkpoint 接續。']
current_rows=[f'| 正常 Debug／Release 建置 | 唯一入口兩版皆完成、每版 services 2＋teleport 14＝16 過／0 敗／0 忽略 | {link("正常兩版資格",np)}；協定冒煙，非完整 GUI |',
 f'| GUI 焦點定向回歸 | 兩版各 1 LOGIN＋1 GAME，合 4 sessions；6 組記錄程序皆終止 | {link("獨立資格",fp)}；Sky 文字／鍵盤登入→選來源→Tab 取得角色→角色清單→正常關閉 |']
controller=q.get('current_native_catalog_controller_qualification',{});broad=q.get('current_native_catalog_broad_qualification',{});library=q.get('current_native_catalog_library',{})
controller_ok=controller.get('source_manifest_sha256')==source and controller.get('qualified')is True
broad_ok=broad.get('source_manifest_sha256')==source and broad.get('observed_actual_exit_code')==0
library_ok=library.get('source_manifest_sha256')==source and q.get('current_full_default_library_verified')is True
current_rows.append('| 服務控制器六分支 | '+('兩版 12 案／96 sessions 已獨立核對'if controller_ok else'執行中或待執行，尚未整批授予資格')+' | '+(link('資格',controller['qualification'])if controller_ok else'跨端點、逾時、拒絕、錯服務、取消重試／到期；以原批次收據為準')+' |')
if broad_ok:
    counts='／'.join(p+str(broad['profiles'][p]['counts']['passed'])+'過'+str(broad['profiles'][p]['counts']['failed'])+'敗'+str(broad['profiles'][p]['counts']['ignored'])+'忽略'for p in ['Debug','Release'])
    current_rows.append('| 廣域預設整合／main | '+counts+' | '+link('完成核對',broad['completion_observation'])+'；非完整 library |')
else:
    progress=q.get('current_native_catalog_broad_progress',{})
    verified=progress.get('qualified_profiles',{}) if progress.get('source_manifest_sha256')==source else {}
    details=[]
    for profile,info in verified.items():
        qp=Path(info['qualification']);pq=read(qp)
        assert sha(qp)==info['sha256'] and pq['source_manifest_sha256']==source and pq['counts']==info['counts']
        details.append(profile+str(pq['counts']['passed'])+'過0敗'+str(pq['counts']['ignored'])+'忽略，'+link('該版資格',qp))
    current_rows.append('| 廣域預設整合／main | '+('；'.join(details)+'；整批仍執行中' if details else '當版尚待；每版 116 個整合目標＋main')+' | 每版 117 個實際映像；非完整 library |')
if library_ok:
    counts='／'.join(p+str(library['profiles'][p]['passed'])+'過0敗'+str(library['profiles'][p]['ignored'])+'忽略'for p in ['Debug','Release'])
    current_rows.append('| 完整 library 預設全庫 | '+counts+' | '+link('完成核對',library['completion'])+'；忽略本文未執行 |')
else:
    progress=q.get('current_native_catalog_library_active',{})
    verified=progress.get('independently_qualified_profiles',{}) if progress.get('source_manifest_sha256')==source else {}
    details=[]
    for profile,info in verified.items():
        qp=Path(info['qualification']);pq=read(qp)
        assert sha(qp)==info['qualification_sha256'] and pq['source_manifest_sha256']==source
        details.append(profile+str(pq['passed'])+'過0敗'+str(pq['ignored'])+'忽略，'+link('該版資格',qp))
    current_rows.append('| 完整 library Debug／Release | '+('；'.join(details)+'；整批仍執行中' if details else '焦點修正版尚待；先前 2DAF 版本已實跑通過')+' | 當版須實跑及獨立核對；忽略本文未執行 |')
regression_status=('修正版當版六分支服務、兩版廣域與完整預設 library 均已獨立核對；接續正常 GUI 全流程與剩餘開發。' if library_ok and broad_ok and controller_ok else '修正版正在重新建立當版服務／廣域／完整 library 資格，舊版本 PASS 不轉記到新來源。')
resource=q.get('current_native_library_resource_optin',{})
if resource.get('source_manifest_sha256')==source and resource.get('qualified')is True:
    rp=Path(resource['qualification']);rq=read(rp)
    assert sha(rp)==resource['qualification_sha256'] and rq['source_manifest_sha256']==source
    assert rq['qualified'] and all(info['passed']==10 and info['failed']==0 for info in rq['profiles'].values())
    current_rows.append('| 額外原生 77 資源案例 | Debug／Release 各 10 過、0 敗、0 忽略 | '+link('獨立資格',rp)+'；與預設全庫分列，非 GUI／實際施法／裝置 |')
gui_entries=q.get('current_normal_GUI_rental_entry_keyboard',{})
for profile,info in gui_entries.items():
    qp=Path(info['qualification']);gq=read(qp)
    assert sha(qp)==info['qualification_sha256'] and gq['source_manifest_sha256']==source
    assert gq['observed_exit_code']==0 and gq['recorded_original_owned_processes_terminal'] and not gq['GUI_full_flow_verified']
    current_rows.append('| '+profile+' 正常 GUI 入村／背包／退出 | 兩帳號依序入 map500、各一步權威移動、完整背包 A45／B43 件開關，均於 30 秒內正常關閉；4 sessions | '+link('獨立限定資格',qp)+'；非雙玩家同場、圖像或完整 GUI |')
hotel_entries=q.get('current_normal_GUI_hotel_entry_keyboard',{})
for profile,info in hotel_entries.items():
    qp=Path(info['qualification']);gq=read(qp)
    assert sha(qp)==info['qualification_sha256'] and gq['source_manifest_sha256']==source
    assert gq['observed_exit_code']==0 and gq['recorded_original_owned_processes_terminal'] and not gq['GUI_full_flow_verified']
    checks='／'.join(role+str(row['quantity'])+'鑰匙→map'+str(row['map_id']) for role,row in gq['roles'].items())
    current_rows.append('| '+profile+' 正常 GUI 旅館入房 | '+checks+'；各 13 步數字鍵盤路線、原始席巴對話、鑰匙新增與房間轉場、正常關閉；4 sessions | '+link('獨立限定資格',qp)+'；未涵蓋退租、共享、訪客主動離房、自然到期或畫面 |')
stamp=datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out=V/'master-report-current-work'/stamp;out.mkdir(parents=True)
old=REPORT.read_bytes();(out/'before.md').write_bytes(old)
now=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
old_library=V/'native-catalog-library-default/20261019-120942-174855/completion-observation.json'
old_broad=V/'native-catalog-default116/20261019-005048-222068/completion-observation.json'
archive=V/'master-report-consolidation/20261019-044931-124021/LiTo381-總開發報告.整理前完整原文.md'
body=f'''# LiTo381 總開發報告與接續指南

更新：{now}。真正 Windows x64 遊戲核心重建持續進行；完整 G01–G12 與 GUI 全流程尚未完成，尚未安排人工測驗。

## 1. 目前停點

先前兩版完整 library 已實跑通過；隨後 GUI 自動測試找出控制項停用造成的鍵盤失焦，現已修正一個正常來源檔，並通過兩版正常建置、協定冒煙與限定焦點 GUI 回歸。**{regression_status}**

{chr(10).join(lines)}

{link('最新八份接續檔 checkpoint',last)}；{link('工作佇列',R/'工作佇列.json')}。有活動批次時核對原 session／PID＋建立時間及收據，接續原批次，不因新對話接不到工具 session 就重啟。

## 2. 當版身分與已核對成果

來源為清單檔 SHA-256，共 1,047 個來源檔／1,048 個編譯輸入：

```text
Source  {source}
Debug   {identity['Debug_normal_sha256']}
Release {identity['Release_normal_sha256']}
JAR     75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371
Catalog B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01
```

本輪產品僅修改 native_login_window.rs：當自有前景視窗的焦點空缺，或焦點屬於已停用的自有控制項時，恢復父視窗焦點；保留有效控制項、背景視窗及關閉流程。{link('來源套用／回復證據',V/'native-keyboard-focus-normal/application-20261019-130716-020285/application.json')}；{link('隔離 API 原失焦與四條件核對',V/'native-keyboard-focus-probe/20261019-130334-204342/receipt.json')}。焦點定向 GUI 資格僅到取得角色；後續 5094 中帳號 A 曾選角入村並送出一步權威移動，整案未完成，未授予入村案例資格。

| 當版範圍 | 實際狀態 | 證據及限制 |
|---|---|---|
{chr(10).join(current_rows)}

## 3. GUI 限制與待補開發

Sky 圖像擷取在重新選取、啟用／最大化、重設連線及單視窗探針後仍回報 FrameArrived／window capture timeout，座標輸入回報 geometry unavailable。啟動程序與輸入桌面已唯讀核對為 WinSta0／Default，未切換桌面或更動系統設定。文字與鍵盤可用，故已自行完成限定登入／焦點回歸；**沒有取得畫面，不宣告圖像、滑鼠、IME／多 DPI、音效裝置或 GUI 全流程通過。**

原 2DAF GUI session 48480／69439 因擷取排查／失焦未完成，原 exit 1 保留；capture-only 43968 的 exit 0 只代表視窗建立／關閉，沒有執行 GAME 或取得圖像。新焦點兩案各 exit 0 與 4 sessions 的範圍另見上表，沒有覆蓋原失敗。

當版 GUI 原 5094 的 A 已入村至 map500／32771,32817，B 尚未登入；隔離 fixture 的接收等待 180 秒及總長 240 秒到期，server／wrapper 原 exit1、兩個正常視窗由保留的自有程序 handle 清理，四組程序皆終止。{link('原失敗核對',V/'native-keyboard-focus-sky-gui/case-20261019-153432-581147-debug-rental-entry/failure-readback.json')} 保留，未轉記通過。新 GUI fixture 只調整操作者等待：接收 900 秒、租裝／旅館總長 900 秒、傳送／失敗流程總長 1800 秒；關閉仍另核對 30 秒，遊戲 8 秒／peer 60 秒、自然到期與 G11 限制保持，無效能資格。另發現技能下拉 ID208 的可存取性名稱誤為「新密碼」，產品修正與回歸尚待。

| 項目 | 剩餘工作 |
|---|---|
| G02／G04 | 最終來源雙場戰鬥、結算、多人同步、碰撞及世界生命週期 |
| G03 | 修正版完整回歸；逐名稱盤點忽略案例，必要案例具備資源後另外實跑 |
| G05 | 物品、裝備、技能與狀態矩陣；歷史技能書 74 項辨識 37／缺 37，辨識不等於施法成功 |
| G06（優先 G06.3d） | 普通 CA00 同端點通道的已知失敗、非預設素質接續、旅館自然到期（3600／7200 秒）、訪客主動離房、其他可達服務與正常 GUI 操作；C10 返回選角不等於回村 |
| G07 | 全地圖、動作圖、遮擋／屋頂／排序、相機與完整 frame；局部像素與診斷 PNG 不等於正常 GUI |
| G08 | 兩版正常登入→選來源／角色→入場→雙玩家→租裝／旅館→傳送／取消／恢復→登出；完整視窗、滑鼠、IME、多 DPI 與退休生命週期 |
| G09 | 正常 GUI 與實際裝置播放／切換／失效／恢復；診斷 Player 不代表裝置通過 |
| G10–G12 | Release 全流程、乾淨主機可攜交付、效能／長時間／故障恢復與逐項完成稽核 |

## 4. 下一步與新對話接續

1. **接續現有批次**，實際終止後獨立核對來源、正常／測試 EXE、原始結果、逐案封包／角色狀態及全部記錄程序終止；失敗保留，修正後新批次另記。
2. **保留有效的當版回歸結果**：六分支控制器、116 整合目標＋main、完整 library 預設與額外 10 個原生 77 資源案例的兩版結果已見上表；相同來源不因接續對話重跑。484 個預設忽略案例另盤點，已額外實跑 10 個，其餘 474 個本文未執行；必要案例另立明確選取器，不啟動舊 TLS／SQL 測試。{link('當版回歸工具清單',V/'native-keyboard-focus-regression-tooling/20261019-131624-778485/readback.json')}。
3. **繼續正常 GUI 全流程自動測試與剩餘開發**：自行建立隔離帳號與本機服務，使用 computer-use／Sky 操作唯一匹配的自有視窗。依 {link('新 GUI fixture 與操作者等待政策',V/'native-keyboard-focus-sky-gui/prepared-20261019-154828-420656/prepared.json')} 續測，舊 5094 的失敗保留。先分別查兩帳號入村、完整背包（不能用租裝接續子集數量代替）、移動及 30 秒內關閉，再補雙玩家同場、傳送／恢復／旅館、正常畫面／滑鼠／IME／多 DPI 與裝置；技能欄錯誤名稱待修。不要求使用者登入、找視窗或輸入測試帳密。{link('非預設素質初始 fixture',V/'native-nondefault-character-fixture/prepared-20261019-141645-451848/prepared.json')} 僅準備 14 欄初始資料來源，尚未編譯／實跑或授予接續資格。
4. 產品再變更時按新來源完成相關驗證及必要 Release／全庫／端到端回歸；來源未變且資格有效時保留結果。全部必要自測確認後，才安排人工測驗與最終交付。

當版主要工具（參數先查 --help）：

```text
run-native-keyboard-focus-controller.py --related-run <當版正常 validation> --harness-source <維護工具/candidates/native-teleport-fixture-arm/NativeTeleportHarness.java>
run-native-keyboard-focus-default116-both.py --normal-run <當版正常 validation> --G06-qualification <當版控制器 qualified/qualified.json>
run-native-keyboard-focus-library-default.py <當版廣域 cohort>
run-native-keyboard-focus-library-resource-optin.py <當版完整 library cohort>
prepare-native-keyboard-focus-sky-gui.py <當版完整 library cohort>
start-native-keyboard-focus-sky-gui-case.py <GUI prepared> --profile Debug --scenario rental-entry --automation-scope accessibility-keyboard
```

每階段使用同名 qualify／observe／checkpoint 工具記錄原始 session、實際 terminal chunk 與 exit；不得猜測 exit 或用重跑替代原終止。{link('當版正常 validation',np.parent.parent)}；{link('當版焦點 GUI 資格',fp)}。舊紀錄中的 target/current 路徑可能已被新建置更新，歷史 EXE 依原 products.zip 與 SHA 追溯。

接手時先讀本報告，再讀 {link('AGENTS.md',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')} 與 {link('歷史',R/'docs/HISTORY.md')}。其他檔案仍有歷史段落，以來源 SHA 與最新 checkpoint 交叉核對。

## 5. 專案邊界與歷史

重建版與登入器／伺服器是獨立專案、產物不同；可只讀參考線索，替換登入器 JAR 的舊提案已撤回，不是本專案依賴。唯一正常入口為啟動現代客戶端.ps1；正常建置 -BuildOnly -BuildProfile Debug／Release，Cargo 固定 --locked --offline -j1、x86_64-pc-windows-msvc 及 LiTo381-Modern/target/current。Release 的 z／thin LTO／16 codegen-units／strip 與 8 秒移動／就緒、60 秒 peer 階段及 G11 預算不變。計時服務測試不並行編譯，測試期間固定來源與對應 QA。

所有開發與隔離資料留在重建版；相鄰登入器／伺服器及原始客戶端資源只讀，不碰正式玩家表或正式程序，不執行／包裝舊 LiTo.bin。原資源在 D:/lineage開發/0-客戶端；正式 JAR 只讀參考在登入器 app/server，測試使用重建版內的 75DA 固定副本。

修正前 **2DAF**：兩版完整 library 各 1969 過／0 敗／484 忽略、inventory 2453，{link('實際整批核對',old_library)}；廣域 Debug544／Release540 過、各 0 敗／23 忽略，{link('整批核對',old_broad)}；服務控制器96＋同端點12＋音效診斷96＋旅館32＝236 sessions，各有原版限定資格。這些數字不轉記修正版；忽略本文沒有執行，診斷圖與音效 Player 不等於 GUI 或裝置驗收。

舊總報告最後記錄 2026-09-07 Server0.7.862／Launcher0.5.54，歷史「總目標完成」限當時登入器／伺服器範圍，非重建版完成。{link('整理前完整原文',archive)} 已逐位元封存，SHA-256：65C64E6068EB50FD3355F3C352F54ED46FA73A808A4DBA8F962979B6A9CB4176。後續以修訂本報告現況與待辦為主，不反覆堆疊相同進度。
'''
assert all(Path(path).exists()for path in re.findall(r'\]\(<([^>]+)>\)',body))
(out/'after.md').write_text(body,encoding='utf-8',newline='\n');assert REPORT.read_bytes()==old;REPORT.write_bytes((out/'after.md').read_bytes());assert REPORT.read_bytes()==(out/'after.md').read_bytes()
assert all(sha(R/name)==h for name,h in check.items())
receipt=dict(status='compact_current_source_report_updated',report=str(REPORT),source_manifest_sha256=source,active_sessions=sessions,
    before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),new_lines=len(body.splitlines()),new_bytes=REPORT.stat().st_size,
    current_controller_qualified=controller_ok,current_broad_qualified=broad_ok,current_library_qualified=library_ok,whole_GUI_qualified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(receipt,ensure_ascii=False))
