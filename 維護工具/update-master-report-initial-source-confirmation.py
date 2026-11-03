"""Compact source-aware handoff for the normal GUI source-expiry repair."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
def link(label,path):return '['+label+'](<'+Path(path).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
app=w['current_native_initial_source_confirmation_application'];assert app['source_manifest_sha256']==source
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
old=w['qualification_before_initial_source_confirmation'];assert old['source_manifest_sha256'].startswith('4F2F911B')
failure=old['source_failure'];assert not failure['qualification_granted']and sha(failure['evidence'])==failure['evidence_sha256']
rows=[]
normal=w.get('current_native_initial_source_confirmation_normal_qualification',{})
normal_ok=normal.get('source_manifest_sha256')==source and normal.get('qualification_granted')is True
if normal_ok:
    path=Path(normal['qualification']);n=read(path);assert sha(path)==normal['qualification_sha256']and n['source_manifest_sha256']==source
    rows.append('| 正常Debug／Release | 各19過0敗0忽略；唯一入口建置及login3＋services2＋teleport14 | '+link('限定資格',path)+'；此列為建置／冒煙範圍 |')
    for profile in ['Debug','Release']:rows.append('| '+profile+'正常EXE SHA-256 | `'+n['normal_mains'][profile]+'` | 綁定當版來源 |')
else:rows.append('| 正常Debug／Release | 建置／相關19案執行中或尚待核對 | 原session與收據見目前停點 |')
library=w.get('current_native_catalog_library',{})
lib_ok=library.get('source_manifest_sha256')==source and w.get('current_full_default_library_verified')is True
if lib_ok:rows.append('| 完整library預設 | '+'／'.join(p+str(library['profiles'][p]['passed'])+'過0敗'+str(library['profiles'][p]['ignored'])+'忽略'for p in ['Debug','Release'])+' | '+link('完成核對',library['completion'])+'；忽略本文未執行 |')
else:rows.append('| 當版完整library／原資源／GUI | 尚未通過 | 前版結果保留原來源，不能移記 |')
resource=w.get('current_native_library_resource_optin',{})
resource_ok=resource.get('source_manifest_sha256')==source and resource.get('qualified')is True
if resource_ok:
    path=Path(resource['qualification']);assert sha(path)==resource['qualification_sha256'];rows.append('| 額外原生77資源選取 | 兩版各10過0敗 | '+link('資格',path)+'；非全部忽略案例執行 |')
controller=w.get('current_native_catalog_controller_qualification',{})
controller_ok=controller.get('source_manifest_sha256')==source and controller.get('qualified')is True
if controller_ok:
    assert sha(controller['qualification'])==controller['qualification_sha256']and controller['actual_cases']==12 and controller['actual_sessions']==96 and controller['recorded_owned_all_terminal']
    rows.append('| 傳送／取消恢復控制器 | 兩版六分支、12案／96sessions | '+link('限定資格',controller['qualification'])+'；非正常GUI |')
confirmed_profiles=[]
for field,label in [('current_normal_GUI_rental_entry_keyboard','正常GUI入村'),('current_normal_GUI_hotel_entry_keyboard','正常GUI旅館')]:
    for profile,record in w.get(field,{}).items():
        if record.get('source_manifest_sha256')!=source:continue
        path=Path(record['qualification']);assert sha(path)==record['qualification_sha256'];rows.append('| '+profile+' '+label+' | 限定範圍核對完成 | '+link('證據',path)+'；非完整GUI |')
        if field=='current_normal_GUI_rental_entry_keyboard':
            proof=read(path)
            assert proof['initial_GAME_public_C8_confirmation_verified']and proof['natural_initial_source_handoff_expiry_verified']
            confirmed_profiles.append(profile)
active=[]
for key,record in w['active_workloads'].items():
    path=Path(record.get('run',record.get('case','')));c=read(path/'cohort.json');assert c['source_manifest_sha256']==source
    active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**，`'+c['status']+'`'+('（'+c['active_profile']+'）'if c.get('active_profile')else'')+'；'+link('原始收據',path/'cohort.json')+'。')
    if key=='full_library'and not lib_ok:
        for profile in ['Debug','Release']:
            qp=path/profile/'full-default-qualified/qualified.json'
            if not qp.is_file():continue
            partial=read(qp)
            assert partial['source_manifest_sha256']==source and partial['failed']==partial['filtered_out']==partial['Rust_exit_code']==0 and partial['recorded_profile_owned_processes_terminal']
            rows.append('| '+profile+' library | '+str(partial['passed'])+'過0敗'+str(partial['ignored'])+'忽略；單版已獨立核對 | '+link('單版資格',qp)+'；整批終止與另一版仍待 |')
    if key=='catalog_normal'and not normal_ok:
        for profile,info in c.get('profiles',{}).items():
            targets=info.get('targets',{})
            if set(targets)=={'native_login','native_services','native_teleport'}:
                assert sum(x['counts']['passed']for x in targets.values())==19 and all(x['counts']['failed']==x['counts']['ignored']==0 for x in targets.values())
                rows.append('| '+profile+'正常建置／相關案例 | 原始19過0敗0忽略；整批獨立核對仍待 | '+link('原始收據',path/'cohort.json')+'；非新C8實際驗證 |')
if not active:active=['- 無登記中的活動批次；依最新checkpoint與下一步接續。']
now=datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
normal_step='正常Debug／Release與每版19案已獨立核對，保留同來源資格。'if normal_ok else'接續當版正常Debug／Release與每版19案，核對來源、映像、原始輸出與記錄程序終止。'
library_step=('當版完整library與原生77資源選取已核對，保留原預設忽略數與各自範圍。'if resource_ok else'當版兩版完整library預設已核對，接續必要原資源選取並分開記錄。'if lib_ok else'接續當版完整library的Debug／Release原預設全庫與必要原資源選取，逐項記錄忽略。')
confirmation_scope=('當版'+ '／'.join(confirmed_profiles)+'已以正常GUI、自然超30秒及實際C8／兩份清單核對；其他版本或接續分支未移記。'if confirmed_profiles else'修正版實際C8行為仍須驗證。')
gui_step=('當版兩版快速交接／自然超30秒初次來源確認及限定入村已核對，保留同來源資格。接續租裝傳送／取消／恢復、普通服務及兩玩家同場／戰鬥／旅館／退出。'if set(confirmed_profiles)=={'Debug','Release'}else'自行跑尚未核對版本正常GUI：快速初次交接，以及自然等待超過30秒再交接；第一份錯誤來源清單不可被開放／選取，確認C8只在正確階段送出。核對正式封包、角色／物品與來源，再測租裝傳送／取消／恢復、普通服務及兩玩家同場／戰鬥／旅館／退出。')
current_intro='當版**'+source[:8]+'**已套初次GAME來源確認修正，只改native_login.rs；'+('正常兩版及每版19冒煙案已核對，'if normal_ok else'正常建置／冒煙待核對，')+('兩版完整library預設已核對，'if lib_ok else'完整library仍待，')+('兩版快速交接／自然到期及限定GUI入村已核對。'if set(confirmed_profiles)=={'Debug','Release'}else'當版GUI資格依下表。')+('控制器六分支已核對；正常GUI傳送恢復、'if controller_ok else'傳送恢復、')+'其他服務與完整GUI／G01–G12仍待，尚未交人工。'
reader_evidence=V/'native-initial-source-confirmation-sky-gui/case-20261019-183653-014141-debug-rental-entry/reader-failure-20261019-184444-032389/failure.json'
reader_note=('入村讀取器兩次失敗保留於'+link('原讀取錯誤',reader_evidence)+'：LOGIN／GAME版本歸屬改以同帳號來源C8與首份角色清單間唯一GAME版本判定；封包採嚴格capture順序及非遞減時間，容許同一時鐘讀值，原時限／來源／物品斷言保留。只重讀原案，未重跑runtime。'if reader_evidence.exists()else'')
body=f'''# LiTo381 總開發報告與接續指南

更新：{now}。真正Windows x64遊戲核心持續開發；完整G01–G12、GUI全流程與人工驗收未完成。

## 1. 目前停點

{current_intro}

{chr(10).join(active)}

{link('最新八份接續檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}；{link('工作佇列',R/'工作佇列.json')}。接續原session／PID＋建立時間，不因新對話重啟活動批次；依工具實際terminal chunk與exit判定終止。

## 2. 當版變更與資格

來源SHA-256（1,047來源檔／1,048編譯輸入）：

```text
{source}
```

{link('套用／回復證據',Path(w['last_goal_turn_evidence']).parent/'application.json')if (Path(w['last_goal_turn_evidence']).parent/'application.json').exists()else link('套用／回復證據',V/'native-initial-source-confirmation-normal/application-20261019-174637-730097/application.json')}：普通初次GAME首份角色清單先不開放，經既有GAME C8重新確認原選來源，完整確認清單後才允許選角。租裝目的地／恢復自動選角不重送此C8；取消、失敗與新連線清除待確認狀態。技能標籤及焦點修正保留；JAR、封包格式、正式來源30秒、旅館數量30秒、原生8秒與peer60秒均未改。

| 當版範圍 | 狀態 | 證據與限制 |
|---|---|---|
{chr(10).join(rows)}

## 3. 已核對的前版與保留失敗

| 來源／範圍 | 實際結果 | 證據 |
|---|---|---|
| 4F2F正常兩版 | 各16過0敗0忽略 | {link('限定資格',old['normal']['qualification'])} |
| 4F2F完整library預設 | 兩版各1969過0敗484忽略、2453個名稱、0篩除；11記錄程序身分終止 | {link('整批核對',old['library']['completion'])} |
| 4F2F原生77資源選取 | 兩版各10過0敗；另外474個預設忽略本文仍未執行 | {link('資格',old['resource']['qualification'])} |
| 4F2F Debug正常GUI | A／B依序入map500、各一步、背包45／43件開關、技能正名與標籤隱藏／恢復、正常關閉；4sessions | {link('限定鍵盤資格',old['GUI_entry']['Debug']['qualification'])} |
| 4F2F Release GUI49194 | A完成限定流程；B選測試乙，30.645秒後GAME出現測試甲清單，選角前停止；原exit0但GUI不通過 | {link('封包／UIA與正式契約',failure['evidence'])} |

正式75DA的GameHub以30秒保存來源，逾時移除；GAME無有效來源時沿用預設來源。另已{link('核對到期時間界線',V/'native-source-handoff-expiry/contract-20261019-174028-047309/causal-timing-refinement.json')}：伺服器送端點後至GAME登入達30.687秒，來源已先於端點送出時暫存，因此登入時已到期。LOGIN進入LOBBY後忽略C8重播；GAME完成readiness且在LOBBY才接受合法／已授權來源C8重綁。{confirmation_scope}

{reader_note}

AB3C廣域Debug544／Release540過（各0敗23忽略、每版117映像）、控制器96sessions及兩版限定旅館入房，仍只屬AB3C；{link('原廣域核對',w['qualification_before_accessibility_labels_change']['broad']['completion_observation'])}。歷史PASS、局部CPU診斷圖、fixture完成或runtime exit0皆不等於當版完整GUI／產品完成。

## 4. 下一步與剩餘開發

1. {normal_step}{link('三工具與固定雜湊',V/'native-initial-source-confirmation-tooling/20261019-174733-882349/normal-tools.json')}；先查各工具--help。
2. {library_step}來源改動後舊版結果不移記。{link('完整library五工具',V/'native-initial-source-confirmation-tooling/20261019-175106-827724/library-tools.json')}；入口run-native-initial-source-confirmation-library-default.py <當版正常validation> --input-kind normal-related。
3. {gui_step}
4. 完成必要開發、自動回歸與Release全流程，才交人工測驗；開發期間不要求使用者登入、找視窗或輸入測試帳密。

{link('當版原資源／GUI七工具',V/'native-initial-source-confirmation-tooling/20261019-175314-494848/followup-tools.json')}已準備，通過範圍依當版表讀回；GUI準備入口prepare-native-initial-source-confirmation-sky-gui.py <當版完整library cohort>。入村讀取器要求B在GAME端點就緒後留下handoff-wait-ready／handoff-wait-finished唯讀觀察，兩次相隔自然超過30秒，再按取得角色；每帳號須有LOGIN C8及GAME C8各一次、確認清單為測試乙，且UIA不能出現測試甲角色列。原生8秒、正式TTL及所有原封包／物品斷言不放寬。

{link('當版控制器六工具',V/'native-initial-source-confirmation-tooling/20261019-175850-213580/controller-tools.json')}已準備；run-native-initial-source-confirmation-controller.py --related-run <當版正常validation> --input-kind catalog-normal --harness-source <重建版/維護工具/candidates/native-teleport-fixture-arm/NativeTeleportHarness.java>，沿用兩版六分支、原始物品全欄接續及拒絕重送C9斷言，另核對新來源C8請求。通過範圍依當版表讀回；不得與編譯／其他計時服務重疊，同來源已核對的library不因此重跑。

| 項目 | 尚待 |
|---|---|
| G02／G04 | 最終來源雙場戰鬥、結算、多人同步、碰撞與世界生命週期 |
| G03 | {'當版library預設已核對；'if lib_ok else'當版library、'}受影響廣域／控制器與逐名稱忽略案例／資源條件 |
| G05 | 物品、裝備、技能與狀態矩陣；歷史技能書74項辨識37／缺37，辨識不等於施法 |
| G06（優先G06.3d） | 正式租裝入口會將普通CA00解為配裝，原生UI／API需阻止誤用；非預設素質接續、旅館自然到期3600／7200秒、退租／共享鑰匙／訪客離房與其他服務；C10返回選角不等於回村 |
| G07 | 全地圖／動作圖／遮擋／屋頂／排序／相機與完整frame及正常GUI像素 |
| G08／G09 | 初次來源到期修正驗證、兩版完整GUI、滑鼠、IME、多DPI、生命週期與實際音效裝置 |
| G10–G12 | Release全流程、乾淨主機可攜交付、效能／長時間／故障恢復及逐項完成稽核 |

已有{link('租裝入口／旅館返回兩檔候選',R/'維護工具/candidates/native-rental-service-ui-and-hotel-return-20261019-180930-820793/candidate.json')}，基於A9FE，尚未套用／編譯：租裝選裝階段隱藏普通頻道操作並於API寫封包前拒絕；普通無租裝目錄的入口保留原功能。另在501–503房內接既有C10返回選角，並處理背包隱藏及地圖換頁；不代表本端新增普通頻道、回村或租約流程通過。{link('非預設素質fixture原始碼',V/'native-accessibility-labels-nondefault-character-fixture/prepared-20261019-165606-505816')}仍基於4F2F、未編譯／未授予實際接續資格，後續須重綁當版來源。

同候選另附{link('API實際封包測試契約',R/'維護工具/candidates/native-rental-service-ui-and-hotel-return-20261019-180930-820793/test-contract.json')}及{link('下一版正常建置三工具',V/'native-rental-service-entry-return-tooling/20261019-190343-825881/normal-tools.json')}、{link('完整library五工具',V/'native-rental-service-entry-return-tooling/readiness-refinement-20261019-190705-556531/library-tools.json')}、{link('資源／GUI／控制器十三工具',V/'native-rental-service-entry-return-tooling/GUI-entry-guard-20261019-191310-949268/followup-tools.json')}，均未編譯／執行。套用工具apply-native-rental-service-entry-return.py須先核對當版控制器96sessions及兩版來源GUI資格、確認無活動批次；預計改兩檔產品並新增一個opt-in整合目標，來源1048／編譯輸入1049，完整library名稱數不受新增整合目標影響；預測來源BC2CA7E9，所有新來源資格須重新核對。新API封包執行器及旅館C10專屬讀取器仍待，現有旅館讀取器只驗入房。候選等待條件已補服務目錄就緒，準備工具兩次錯誤及修正保留於library工具收據。

Sky圖像擷取經重新選取、啟用／最大化、重設連線、靜態及已載入動畫世界仍回報FrameArrived／window capture timeout；座標輸入geometry unavailable。文字與鍵盤可用，正常GUI圖像／滑鼠未通過。唯讀桌面核對為WinSta0／Default，未切換桌面或更動系統設定。保留48480／69439／5094等原GUI失敗，capture-only43968不具GAME／圖像資格。

GUI操作者等待：接收900秒、租裝／旅館總長900秒、傳送／失敗1800秒，正常關閉另核對30秒；只屬操作等待，遊戲時限不變，不授予G11效能資格。自然3600／7200秒旅館到期須另備足夠外層生命週期並保留實際時鐘。

## 5. 邊界與接續入口

重建版與登入器／伺服器是獨立專案、產物不同；相鄰專案與D:/lineage開發/0-客戶端只讀。替換登入器JAR的舊提案撤回，不是本專案依賴。不碰正式玩家表或正式程序，不執行／包裝舊LiTo.bin。所有開發／測試資料留重建版；唯一例外寫入是本總報告。不派子代理或其他對話。

正常入口啟動現代客戶端.ps1，建置-BuildOnly -BuildProfile Debug／Release；Cargo固定--locked --offline -j1、x86_64-pc-windows-msvc、LiTo381-Modern/target/current。Release z／thin LTO／16 codegen-units／strip不變。計時服務不並行編譯，活動批次固定來源與對應QA，清理只用原持有程序身分。

正式參考JAR SHA-256：75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371；catalog：B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01；測試用重建版內固定副本。

接續先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。Server0.7.862／Launcher0.5.54不是重建版完成。{link('整理前完整總報告',V/'master-report-consolidation/20261019-044931-124021/LiTo381-總開發報告.整理前完整原文.md')} SHA-256：65C64E6068EB50FD3355F3C352F54ED46FA73A808A4DBA8F962979B6A9CB4176。
'''
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
old_bytes=REPORT.read_bytes();(out/'before.md').write_bytes(old_bytes);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes()==old_bytes and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
receipt=dict(report=str(REPORT),source_manifest_sha256=source,normal_qualified=normal_ok,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(old_bytes).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(receipt,ensure_ascii=False))
