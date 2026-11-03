"""Compact next-source handoff; preserve prior-source qualifications separately."""
from pathlib import Path
from datetime import datetime, timezone, timedelta
import hashlib, json, re
R = Path(__file__).resolve().parents[1]
V = R / '驗證紀錄/current'
REPORT = R.parent / 'LiTo381-總開發報告.md'
read = lambda p: json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as stream: return hashlib.file_digest(stream, 'sha256').hexdigest().upper()
link = lambda label, path: '[' + label + '](<' + Path(path).as_posix() + '>)'
w = read(R / '工作佇列.json')
source = w['current_product_identity']['source_manifest_sha256']
application = w['current_native_rental_service_entry_return_application']
assert application['source_manifest_sha256'] == source
before = read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files']
assert len(before) == 8 and all(sha(R / name) == digest for name, digest in before.items())
applications = [p for p in (V / 'native-rental-service-entry-return-normal').glob('application-*/application.json')
    if read(p)['source_manifest_sha256'] == source]
assert len(applications) == 1
app = applications[0]
assert read(app)['after_files'] == application['after_files']
old = w['qualification_before_rental_service_entry_return']
assert old['source_manifest_sha256'] == 'A9FE9AD547E79DC1E0C97DC01CB6C20D0119477788E193524F5B2DFB96DD4B4E'
rows = []
normal = w.get('current_native_rental_service_entry_return_normal_qualification', {})
normal_ok = normal.get('source_manifest_sha256') == source and normal.get('qualification_granted') is True
if normal_ok:
    assert sha(normal['qualification']) == normal['qualification_sha256']
    n = read(normal['qualification'])
    assert n['source_manifest_sha256'] == source and n['passed_per_profile'] == 19
    rows.append('| 正常兩版／既有冒煙 | 各19過0敗0忽略 | ' + link('限定資格', normal['qualification']) + '；新opt-in未由此編譯／執行 |')
    for profile in ['Debug', 'Release']:
        rows.append('| ' + profile + '正常EXE SHA-256 | `' + n['normal_mains'][profile] + '` | 唯一入口建置 |')
else:
    rows.append('| 正常Debug／Release | 待完成建置／獨立核對 | 原批次見目前停點 |')
library = w.get('current_native_catalog_library', {})
lib_ok = library.get('source_manifest_sha256') == source and w['current_full_default_library_verified'] is True
if lib_ok:
    assert sha(library['completion']) == library['completion_sha256']
    rows.append('| 完整library預設 | ' + '／'.join(p + str(library['profiles'][p]['passed']) + '過0敗' + str(library['profiles'][p]['ignored']) + '忽略' for p in ['Debug','Release']) + ' | ' + link('完成核對', library['completion']) + '；忽略本文另列 |')
else:
    rows.append('| 完整library | 尚未通過 | 前版結果保留原來源 |')
resource = w.get('current_native_library_resource_optin', {})
if resource.get('source_manifest_sha256') == source and resource.get('qualified') is True:
    assert sha(resource['qualification']) == resource['qualification_sha256']
    rows.append('| 原生77原資源選取 | 兩版各10過0敗 | ' + link('限定資格', resource['qualification']) + '；非全部忽略本文 |')
shared=w.get('current_native_library_shared_resource_optin',{})
if shared.get('source_manifest_sha256')==source and shared.get('qualified')is True:
    assert sha(shared['qualification'])==shared['qualification_sha256']
    assert all(shared['profiles'][p]['passed']==72 for p in ['Debug','Release'])
    rows.append('| 原始共用資源額外選取 | 兩版各72過0敗；與原生77之10項不重複 | '+link('限定資格',shared['qualification'])+'；預設忽略本文尚餘402項，非GUI或實體音效 |')
elif shared.get('source_manifest_sha256')==source and shared.get('original_failure_preserved')is True:
    assert sha(shared['raw_archive'])==shared['raw_archive_sha256']
    rows.append('| 原始共用資源72項 | Debug65過7敗；Rust101／wrapper1，Release未執行 | '+link('失敗封存與診斷',shared['analysis'])+'；五項缺指定資源、Text索引差一筆、目標物件回傳排序／遮罩不一致，需修復再驗 |')
else:
    rows.append('| 原始共用資源72項 | 執行／核對待完成 | '+link('逐名計畫',V/'native-library-shared-resource-optin/plan-20261019-194354-677571/plan.json')+'；PNG輸出保全工具已準備，不能算通過 |')
for field, label in [('current_native_rental_entry_guard_qualification','租裝入口API實際封包'),
    ('current_native_catalog_controller_qualification','傳送／取消恢復控制器')]:
    record = w.get(field, {})
    if record.get('source_manifest_sha256') == source and record.get('qualified') is True:
        assert sha(record['qualification']) == record['qualification_sha256']
        rows.append('| ' + label + ' | 限定範圍核對完成 | ' + link('資格', record['qualification']) + '；非正常GUI |')
api=w.get('current_native_rental_entry_guard_qualification',{})
api_ok=api.get('source_manifest_sha256')==source and api.get('qualified')is True
for field, label in [('current_normal_GUI_rental_entry_keyboard','GUI入村／初次來源'),
    ('current_normal_GUI_hotel_entry_keyboard','GUI旅館入房'),
    ('current_normal_GUI_hotel_return_keyboard','GUI旅館C10返回')]:
    for profile, record in w.get(field, {}).items():
        if record.get('source_manifest_sha256') != source: continue
        assert sha(record['qualification']) == record['qualification_sha256']
        rows.append('| ' + profile + ' ' + label + ' | 限定範圍核對完成 | ' + link('資格', record['qualification']) + '；非完整GUI |')
for profile,record in w.get('current_native_same_endpoint_rental_qualification',{}).items():
    if record.get('source_manifest_sha256')!=source or record.get('qualified')is not True:continue
    assert sha(record['qualification'])==record['qualification_sha256']
    rows.append('| '+profile+'同端點租裝服務 | 限定接續核對完成 | '+link('資格',record['qualification'])+'；非普通頻道或正常GUI |')
active = []
for key, record in w['active_workloads'].items():
    run = Path(record.get('run', record.get('case','')))
    c = read(run / ('cohort.json'if(run/'cohort.json').is_file()else'receipt.json'))
    assert c['source_manifest_sha256'] == source
    active.append('- ' + key + '：原session **' + str(w['active_workload_sessions'][key]) + '**，`' + c['status'] + '`' + ('（' + c['active_profile'] + '）' if c.get('active_profile') else '') + '；' + link('原收據',run / 'cohort.json') + '。')
    if key=='catalog_normal'and not normal_ok:
        for profile,info in c.get('profiles',{}).items():
            targets=info.get('targets',{})
            if set(targets)=={'native_login','native_services','native_teleport'}:
                assert sum(item['counts']['passed']for item in targets.values())==19 and all(item['counts']['failed']==item['counts']['ignored']==0 for item in targets.values())
                rows.append('| '+profile+'正常建置／冒煙 | 原輸出19過0敗0忽略；整批核對仍待 | '+link('原收據',run/'cohort.json')+'；正常EXE `'+info['normal']['sha256']+'` |')
    if key=='full_library'and not lib_ok:
        for profile in ['Debug','Release']:
            path=run/profile/'full-default-qualified/qualified.json'
            if not path.exists():continue
            partial=read(path);assert partial['source_manifest_sha256']==source and partial['failed']==partial['filtered_out']==partial['Rust_exit_code']==0
            rows.append('| '+profile+' library | '+str(partial['passed'])+'過0敗'+str(partial['ignored'])+'忽略；單版獨立核對完成 | '+link('單版資格',path)+'；整批與另一版仍待 |')
if not active: active = ['- 無登記中的活動批次；接續下列尚未完成工作，不重跑已核對的同來源範圍。']
now = datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M（Asia/Taipei）')
body = f'''# LiTo381 總開發報告與接續指南

更新：{now}。真正Windows x64重建核心持續開發；完整G01–G12、GUI全流程與人工驗收未完成。

## 1. 目前停點

當版**{source[:8]}**已套租裝入口／旅館返回兩檔修補，並新增一個獨立opt-in整合測試。來源1,048檔／編譯輸入1,049；實際通過範圍見下表，未交人工。

{chr(10).join(active)}

{link('最新八份接續檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}；{link('工作佇列',R/'工作佇列.json')}。活動批次續原session／PID＋建立時間；工具回傳session不等於完成，須等實際terminal chunk及exit再做相依核對。

## 2. 當版變更與資格

來源SHA-256：`{source}`。

{link('套用／回復證據',app)}：有租裝目錄的選裝入口隱藏普通頻道操作，enter_channel在寫CA00前拒絕；普通入口保留原功能。旅館501–503房內提供既有C10「返回選角」，處理背包隱藏、可操作條件與換圖布局；不代表回村或租約流程已通過。新測試需實際封包證明拒絕不送CA00、正常配裝／入村與返回選角後仍可用。

A9FE的初次GAME來源確認保留：首份清單先不開放，經GAME C8確認選定來源後才允許選角；租裝目的地／恢復自動接續不額外重送。技能標籤與焦點修正保留。JAR、封包格式、來源暫存30秒、旅館數量30秒、原生8秒／peer60秒及Release設定均未改。

| 當版範圍 | 結果 | 證據與限制 |
|---|---|---|
{chr(10).join(rows)}

## 3. 已完成前版與保留失敗

| A9FE範圍 | 已核對結果 | 證據 |
|---|---|---|
| 正常Debug／Release | 各19過0敗0忽略 | {link('限定資格',old['normal']['qualification'])} |
| 完整library | 各1969過0敗484忽略、2453名稱、0篩除；11記錄程序皆終止 | {link('完成核對',old['library']['completion'])} |
| 原生77資源選取 | 各10過0敗，另474預設忽略本文未執行 | {link('限定資格',old['resource']['qualification'])} |
| 正常GUI兩版 | 各4sessions，快速／自然到期後C8確認、map500單步、45／43件背包開關、技能標籤及正常退出 | {link('Debug',old['GUI_entry']['Debug']['qualification'])}；{link('Release',old['GUI_entry']['Release']['qualification'])} |
| 控制器兩版 | 六分支12案／96sessions，完整物品接續及C9拒重播；非GUI | {link('限定資格',old['controller']['qualification'])} |

4F2F Release49194曾在選測試乙後30.645秒交接，GAME回落測試甲，選角前停止；原runtime exit0不代表GUI通過。{link('原失敗與正式75DA契約',V/'native-source-handoff-expiry/contract-20261019-174028-047309/readback.json')}。A9FE正常GUI B分別54.465／61.029秒交接，封包先測試甲、C8後測試乙，錯誤清單未開放給UI。以上全屬原來源，不移記當版。

GUI讀取器LOGIN／GAME版本歸屬與同時鐘讀值修正、Release提早讀取活動checkpoint的拒絕，以及候選準備工具錯誤皆保留；只修讀取／準備，不偽造重跑。AB3C廣域Debug544／Release540過、各23忽略及歷史旅館資格也只屬AB3C。歷史PASS、CPU診斷PNG、fixture完成或runtime exit0均不等於當版完整GUI。

## 4. 接續順序與剩餘開發

1. {'正常兩版與19冒煙案已核對，保留資格。'if normal_ok else'完成唯一入口正常Debug／Release與19既有冒煙案，核對來源、映像、原輸出與程序終止。'}{link('正常三工具',V/'native-rental-service-entry-return-tooling/20261019-190343-825881/normal-tools.json')}；入口run-native-rental-service-entry-return-normal.py <當版application>。
2. {'完整library預設已核對；接續未通過的必要原資源。'if lib_ok else'先完成當版Debug／Release完整library，再執行必要原資源選取，分列預設忽略本文。'}{link('完整library五工具',V/'native-rental-service-entry-return-tooling/readiness-refinement-20261019-190705-556531/library-tools.json')}；入口run-native-rental-service-entry-return-library-default.py <當版normal validation> --input-kind normal-related。
   共用資源另跑run-current-shared-resource-library.py <當版library>，以checkpoint-current-shared-resource-library.py登記原session／terminal並逐名核對72項，與原生77十項分列。
3. {'新API實際封包限定範圍已核對，保留同來源資格。'if api_ok else'新API三工具已準備，待當版library／原資源核對後編譯並執行，不能當作已通過。'}租裝入口前後拒絕不得送CA00，正常配裝／入村及C10返回選角需核對。{link('測試契約',R/'維護工具/candidates/native-rental-service-ui-and-hotel-return-20261019-180930-820793/test-contract.json')}；{link('API三工具',V/'native-rental-service-entry-return-tooling/API-guard-20261019-192140-017288/API-tools.json')}，入口run-native-rental-entry-guard.py <當版library cohort>。
   {link('同端點租裝服務四工具',V/'native-rental-service-entry-return-tooling/same-endpoint-checkpoint-20261019-193700-836988/same-endpoint-tools.json')}已重綁當版、未編譯／執行；保留初始交接完整物品斷言，傳送後依收到的目錄重新配裝／入租裝村並驗新世代及移動。不是普通頻道能力，原失敗不改記；入口compile-native-rental-service-same-endpoint.py --profile Debug／Release --library-cohort <當版library>，編譯與runtime均用checkpoint-native-rental-service-same-endpoint.py登記原程序，終止後逐版獨立核對。
4. 自行完成正常GUI：租裝入口隱藏普通頻道、來源自然超30秒／快速交接、旅館入房及C10返回、傳送／取消／恢復、普通服務與雙玩家戰鬥／退出。入村讀取器核對普通頻道三項控制均隱藏；另已備{link('房主旅館C10讀取器',V/'native-rental-service-entry-return-tooling/hotel-owner-return-20261019-192933-185343/hotel-return-tools.json')}，仍未執行，沿用rental-hotel-entry案例加hotel-return-focus／hotel-return／hotel-return-characters-ready觀察，再執行qualify-native-rental-service-entry-return-GUI-hotel-return.py <case>。{link('資源／GUI／控制器十三工具',V/'native-rental-service-entry-return-tooling/GUI-entry-guard-20261019-191310-949268/followup-tools.json')}；GUI準備入口prepare-native-rental-service-entry-return-sky-gui.py <當版library cohort>。
5. 完成剩餘開發、當版廣域與Release全流程，才交人工測驗；開發中不要求使用者登入、找視窗或輸入測試帳密。

| 項目 | 尚待 |
|---|---|
| G02／G04 | 最終來源雙場戰鬥、結算、多人同步、碰撞與世界生命週期 |
| G03 | 當版廣域／控制器及逐名稱忽略案例／資源條件；新增整合目標預設忽略1項，不能混入library統計 |
| G05 | 物品、裝備、技能與狀態矩陣；歷史74技能書辨識37／缺37，辨識不等於施法 |
| G06（優先G06.3d） | 當版租裝入口／同端點有效服務與旅館返回驗證、非預設素質接續、共享鑰匙／訪客離房、自然3600／7200秒到期及其他服務 |
| G07–G09 | 全地圖／動作／遮擋／屋頂／排序／相機／完整frame、正常GUI像素／滑鼠／IME／多DPI、生命週期及實際音效裝置 |
| G10–G12 | 最終Release全流程、乾淨主機可攜交付、效能／長時間／故障恢復與逐項稽核 |

{link('非預設素質fixture',V/'native-rental-service-nondefault-character-fixture/prepared-20261019-194001-977126/prepared.json')}已重綁當版、尚未編譯／連線；原4F2F版本保留。原始碼預設兩帳號各14項初始素質差異，初始差異不代表配裝／傳送後接續；先在library完成且無活動批次時，以prepare-native-rental-service-nondefault-character-fixture.py compile-describe <prepared目錄>核對初始描述，再另跑實際接續。{link('正式75DA旅館返回契約',V/'native-hotel-return-contract/20261019-192705-656308/readback.json')}已靜態核對：房主C10撤銷共享租約／鑰匙，抑制房主先回map500並續來源選角；訪客C10只丟棄其持有憑證，房主撤銷造成訪客回村另驗。這不是runtime資格。正式租裝入口將普通CA00解為配裝，未修改伺服器以新增普通頻道能力。

Sky圖像擷取經靜態／動畫世界與連線重設仍為FrameArrived／window capture timeout，座標輸入geometry unavailable；文字／鍵盤可用，圖像／滑鼠未通過。唯讀桌面為WinSta0／Default，無鎖定桌面證據，不要求使用者解鎖。不得用替代擷取驅動繞過。GUI操作等待900／1800秒只屬外層等待，遊戲時限不變；自然旅館長測另需足夠外層生命週期。

## 5. 邊界與接續入口

重建版與登入器／伺服器獨立、產物不同；相鄰專案與D:/lineage開發/0-客戶端只讀。替換登入器JAR舊提案撤回，不是依賴。不碰正式玩家表／程序，不執行或包裝舊LiTo.bin。資料限重建版，總報告為已授權例外；不派代理或其他對話。

唯一入口啟動現代客戶端.ps1，-BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current。Release z／thin／16／strip不變。計時服務不並行編譯；活動批次固定來源與對應QA，只清理原持有程序身分。

固定正式JAR 75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371；catalog B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01，使用重建版內副本。Server0.7.862／Launcher0.5.54不是本專案完成。

接續讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。{link('整理前完整總報告',V/'master-report-consolidation/20261019-044931-124021/LiTo381-總開發報告.整理前完整原文.md')} SHA-256：65C64E6068EB50FD3355F3C352F54ED46FA73A808A4DBA8F962979B6A9CB4176。
'''
assert all(Path(path).exists() for path in re.findall(r'\]\(<([^>]+)>\)', body))
out = V / 'master-report-current-work' / datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f')
out.mkdir(parents=True)
prior = REPORT.read_bytes()
(out / 'before.md').write_bytes(prior)
(out / 'after.md').write_text(body, encoding='utf-8', newline='\n')
(out / 'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes() == prior and all(sha(R / name) == digest for name, digest in before.items())
REPORT.write_bytes((out / 'after.md').read_bytes())
receipt = dict(report=str(REPORT), source_manifest_sha256=source, normal_qualified=normal_ok,
    library_qualified=lib_ok, active_sessions=w['active_workload_sessions'],
    before_sha256=hashlib.sha256(prior).hexdigest().upper(), after_sha256=sha(REPORT),
    lines=len(body.splitlines()), GUI_full_flow_verified=False, whole_goal_complete=False)
(out / 'readback.json').write_text(json.dumps(receipt, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
print(json.dumps(receipt, ensure_ascii=False))
