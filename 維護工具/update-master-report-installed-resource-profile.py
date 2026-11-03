"""Compact current handoff; retain precise source-scoped qualifications and failures."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
link=lambda label,path:'['+label+'](<'+Path(path).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];app=w['current_native_installed_resource_profile_application']
assert app['source_manifest_sha256']==source=='603C835908762CAF643F3ECA995039745DD6BE428FE9F533261B226BA2AD744A'
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
old=w['qualification_before_render_placement_repair'];assert old['source_manifest_sha256']=='BC2CA7E90123F5CA6194737AF8ACA41D5B1997A86BC9BFC8350E3ECDC24A9D3B'
rows=[]
normal=w.get('current_native_installed_resource_profile_normal_qualification',{})
normal_ok=normal.get('source_manifest_sha256')==source and normal.get('qualification_granted')is True
if normal_ok:
    assert sha(normal['qualification'])==normal['qualification_sha256'];n=read(normal['qualification'])
    rows.append('| 正常Debug／Release與冒煙 | 各19過0敗0忽略 | '+link('限定資格',normal['qualification'])+'；新繪製重現案由library驗證 |')
    for profile in ['Debug','Release']:rows.append('| '+profile+'正常EXE SHA-256 | `'+n['normal_mains'][profile]+'` | 唯一入口建置 |')
else:rows.append('| 正常Debug／Release與19冒煙 | 尚在建置／核對，未通過 | 見原活動批次 |')
lib=w.get('current_native_catalog_library',{});lib_ok=lib.get('source_manifest_sha256')==source and w['current_full_default_library_verified']is True
if lib_ok:
    assert sha(lib['completion'])==lib['completion_sha256']
    rows.append('| 完整library預設兩版 | '+'／'.join(p+str(lib['profiles'][p]['passed'])+'過'+str(lib['profiles'][p]['failed'])+'敗'+str(lib['profiles'][p]['ignored'])+'忽略'for p in ['Debug','Release'])+' | '+link('完成核對',lib['completion'])+'；忽略本文另列 |')
else:rows.append('| 完整library預設兩版 | 尚未通過 | 保留新增1項fragment重現案，另新增4項明選現行TW忽略案；須核對實際名稱／總數 |')
focused=w.get('current_native_render_fixture_regression',{})
if focused.get('source_manifest_sha256')==source and focused.get('qualified')is True:
    assert sha(focused['qualification'])==focused['qualification_sha256']
    rows.append('| Debug兩項原失敗定位回歸 | 2過0敗0忽略、2456篩除 | '+link('限定資格',focused['qualification'])+'；不是完整library或兩版正常資格 |')
shared_focused=w.get('current_native_shared_world_fixture_regression',{})
if shared_focused.get('source_manifest_sha256')==source and shared_focused.get('qualified')is True:
    assert sha(shared_focused['qualification'])==shared_focused['qualification_sha256']
    assert shared_focused['profiles']['Debug']==dict(passed=1,failed=0,ignored=0,filtered_out=2457)
    rows.append('| 原資源角色順序Debug定位回歸 | 1過0敗0忽略、2457篩除 | '+link('限定資格',shared_focused['qualification'])+'；原畫面／owner雜湊及移動斷言保留 |')
for key,label in [('current_native_library_resource_optin','原生77資源'),('current_native_library_shared_resource_optin','額外原資源'),('current_native_rental_entry_guard_qualification','入口API封包'),('current_native_catalog_controller_qualification','傳送／恢復控制器')]:
    record=w.get(key,{})
    if record.get('source_manifest_sha256')==source and record.get('qualified')is True:
        assert sha(record['qualification'])==record['qualification_sha256']
        counts='／'.join(profile+str(value.get('passed',''))+'過'+str(value.get('failed',0))+'敗' for profile,value in record.get('profiles',{}).items())
        rows.append('| '+label+' | '+(counts or '限定核對完成')+' | '+link('資格',record['qualification'])+'；不等於GUI |')
for key,label in [('current_normal_GUI_rental_entry_keyboard','正常GUI租裝入口'),('current_normal_GUI_hotel_return_keyboard','正常GUI旅館入房／房主返回')]:
    for profile,record in w.get(key,{}).items():
        if record.get('source_manifest_sha256')==source:
            qp=record.get('qualification')
            if qp and Path(qp).is_file():
                assert sha(qp)==record['qualification_sha256']
                rows.append('| '+label+' '+profile+' | 鍵盤流程限定核對完成 | '+link('資格',qp)+'；圖像／滑鼠及完整GUI仍待 |')
resource71=w.get('current_native_library_shared_resource_optin',{})
resource71_ok=resource71.get('source_manifest_sha256')==source and resource71.get('qualified')is True
entry_failure=w.get('current_native_rental_entry_guard_failure',{})
if entry_failure.get('source_manifest_sha256')==source:
    assert sha(entry_failure['raw_archive'])==entry_failure['raw_archive_sha256']
    rows.append('| 租裝入口API／村莊返回 | Debug0過1敗；Release已編譯未執行 | '+link('失敗與正式方法封存',Path(entry_failure['raw_archive']).with_name('evidence.json'))+'；map500 C10／S65／D9後無F000，非分段讀取問題，旅館返回仍待實跑 |')
prior=w.get('qualification_before_shared_world_identity_fixture_repair',{})
if prior:
    f=prior['shared_failure'];assert sha(f['raw_archive'])==f['raw_archive_sha256']
    rows.append('| 前版4018AE86原資源 | Debug70過1敗；Release未啟動 | '+link('失敗封存',Path(f['raw_archive']).with_name('evidence.json'))+('；當版修正後71項兩版已通過 |'if resource71_ok else'；當版修正後待整批重驗 |'))
active=[]
for key,record in w['active_workloads'].items():
    run=Path(record.get('run',record.get('case','')));receipt=run/('cohort.json'if(run/'cohort.json').exists()else'receipt.json');c=read(receipt)
    assert c['source_manifest_sha256']==source
    if key=='full_library'and not lib_ok:
        for profile in ['Debug','Release']:
            partial=run/profile/'full-default-qualified/qualified.json'
            if partial.is_file():
                value=read(partial)
                assert value['source_manifest_sha256']==source and value['failed']==value['Rust_exit_code']==value['filtered_out']==0
                rows.append('| '+profile+'完整library單版 | '+str(value['passed'])+'過0敗'+str(value['ignored'])+'忽略；單版獨立核對完成 | '+link('單版資格',partial)+'；整批及另一版仍待 |')
    active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**，`'+c['status']+'`'+('（'+c['active_profile']+'）'if c.get('active_profile')else'')+'；'+link('原收據',receipt)+'。')
checkpoint=Path(w['last_goal_turn_evidence']).with_name('readback.json')
applications=[p for p in(V/'native-installed-resource-profile-normal').glob('application-*/application.json')if read(p)['source_manifest_sha256']==source];assert len(applications)==1
failed=old['resource_failure'];assert sha(failed['raw_archive'])==failed['raw_archive_sha256']
body=f'''# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標維持：完成剩餘原生開發，先完整library Debug／Release，再正常GUI全流程／Release回歸；自行修復與驗證後才交人工。完整G01–G12未完成。

## 1. 目前停點

當版**{source[:8]}**保留原生繪製／資源修補，再以entity ID修正原資源測試的角色順序假設；來源1,048檔／編譯輸入1,049。來源SHA-256：`{source}`。當版已通過與未通過範圍如下，未交人工。

{chr(10).join(active)if active else'目前無已登記活動批次；依下列順序繼續。'}

{link('最新八份接續檔checkpoint',checkpoint)}；{link('工作佇列',R/'工作佇列.json')}。活動工作續原session／PID＋建立時間，等待實際terminal chunk及exit；不得重啟原批次或將工具回傳session當完成。

## 2. 當版修補與資格

{link('原生修補／回復',V/'native-render-placement-normal/application-20261019-200357-854061/application.json')}：光柵仍按原序插入；可見性標號改由場景深度的固定名次分配，回傳角色／目標／NPC也按深度排列。修復反轉目標物件輸入時RenderInfo順序及遮罩標號變動，保留像素混色、命中遮罩、原資源斷言及所有遊戲時限；新增不依賴外部資源的native fragment重現測試。候選與前版失敗皆封存。

{link('資源測試更新／回復',V/'native-installed-resource-profile-normal/application-20261019-201457-707212/application.json')}只改測試：Text現行參考以完整idx／pak雜湊及{link('獨立C#1.5.4核對',V/'native-installed-Text-reference/20261019-200910/readback.json')}綁定37497筆／28361名稱，保留舊37496／28360參考與全部payload／重名拒絕斷言。另新增四項明選現行launcher TW的忽略案例，共用原SPR／合成像素／時鐘／所有權斷言；舊根目錄案例與載入入口均保留，不做fallback。{'四項新本文已隨現行71項兩版實跑通過。'if resource71_ok else'四項新本文尚待當批實跑／獨立核對。'}

{link('最新fixture修正／回復',applications[0])}：原資源雙角色案例以ID1001／1002核對原subframe及owner像素oracle，新增深度順序1002／1001斷言；完整像素／owner雜湊、SPR、移動與其他斷言不變。前次屋頂ID1與512×256視口修正保留。

| 當版範圍 | 結果 | 證據與限制 |
|---|---|---|
{chr(10).join(rows)}

初次GAME C8確認、租裝入口拒送普通CA00／隱藏頻道控制、旅館501–503既有C10返回選角及技能焦點修正保留。JAR、封包、來源暫存30秒、旅館數量30秒、原生8秒／peer60秒與Release z／thin／16／strip均未改。

## 3. 前版結果與本次失敗

| 原來源／範圍 | 已核對結果 | 證據與限制 |
|---|---|---|
| B0628AC9正常兩版 | 各19過0敗0忽略；11記錄程序皆終止 | {link('限定資格',w['qualification_before_installed_resource_profiles']['normal']['qualification'])}；該版library未跑，非當版資格 |
| 9F99C120正常兩版 | 各19過0敗0忽略 | {link('限定資格',w['qualification_before_render_test_fixture_repair']['normal']['qualification'])}；非當版資格 |
| 9F99C120完整library | Debug1968過2敗488忽略；Rust101／wrapper1，Release未啟動；6記錄程序皆終止 | {link('失敗封存',Path(w['qualification_before_render_test_fixture_repair']['library_failure']['raw_archive']).with_name('evidence.json'))}；兩項為測試角色索引／視口配置；後續4018AE86完整library兩版通過，本版依當版表格重驗 |
| BC2CA7E9正常兩版 | 各19過0敗0忽略 | {link('限定資格',old['normal']['qualification'])} |
| BC2CA7E9完整library預設 | 各1969過0敗484忽略，2453名稱／0篩除；11記錄程序皆終止 | {link('完成核對',old['library']['completion'])} |
| BC2CA7E9原生77 | 兩版各10過0敗 | {link('限定資格',old['resource']['qualification'])} |
| BC2CA7E9額外72原資源 | Debug65過7敗，Rust101／wrapper1；Release未啟動 | {link('原失敗與34份raw封存',failed['analysis'])}；未授資格 |
| A9FE9AD5正常GUI | 兩版各4sessions：快速／自然超30秒來源交接、map500單步、45／43件背包、技能標籤與退出 | {link('Debug',w['qualification_before_rental_service_entry_return']['GUI_entry']['Debug']['qualification'])}；{link('Release',w['qualification_before_rental_service_entry_return']['GUI_entry']['Release']['qualification'])} |
| A9FE9AD5控制器 | 兩版六分支12案／96sessions，完整物品接續與C9拒重播 | {link('限定資格',w['qualification_before_rental_service_entry_return']['controller']['qualification'])}；非GUI |

額外資源七敗中，五項缺指定舊根目錄TW封包（現行launcher命名空間／SHA不同，不能當同一份），Text索引實際37497筆／舊C#參考37496筆已完成獨立來源核對並套測試擴充；目標回傳排序／遮罩也已修。{'兩項已在當版71項兩版實跑通過。'if resource71_ok else'兩項的原資源本文尚待當批獨立核對。'}原資源metadata未變、歷史診斷輸出已還原；另402項預設忽略本文未執行，逐項有Java／TLS／SQL／資源等不同條件。{link('484名稱條件表',V/'native-library-shared-resource-optin/remaining-disposition-20261019-195738-429004/disposition.json')}為BC2C停點，不是當版資格。

4F2F的Release來源到期失敗、A9FE讀取器修正、AB3C廣域544／540過各23忽略及舊旅館成果都留原版。CPU診斷PNG、fixture完成或runtime exit0不等於正常GUI通過；不得移記新版。

## 4. 接續順序與剩餘開發

1. 當版先完成正常兩版各19冒煙與完整library兩版，各自依表格及活動session核對；正常三工具為run／qualify／checkpoint-native-installed-resource-profile-normal.py。完整library入口run-native-installed-resource-profile-library-default.py <當版normal validation> --input-kind normal-related，逐版qualify、原terminal後observe及checkpoint-completed；工具均在{link('當版八工具收據',V/'native-installed-resource-profile-tooling/validation-preparation-20261019-201342-436781/tooling.json')}。
2. 新fragment重現案已通過完整library；原生77十項需按當版兩版重驗，前版4018AE86已各10過0敗。現行71項{'兩版各71過0敗，合計仍有407項預設忽略本文未執行（五項舊根目錄＋402項其他條件）。'if resource71_ok else'沿原活動session完成逐版執行及獨立核對。'}五項舊根目錄TW案例缺其歷史封包，保留缺條件，不能移記現行四項資格；其餘402原本文條件逐項處理。獨立來源核對後擴充明選參考，不能直接改舊SHA期待、接受缺檔、fallback或全開忽略規避條件。
3. {link('當版22個資源／API／GUI／控制器後續工具',Path('D:/lineage開發/3-lito開發/lito重建版/驗證紀錄/current/native-shared-world-identity-tooling/rebind-20261019-212410-388835/tooling.json'))}已重綁，實跑資格依本報告表格與最新checkpoint；原生77入口run-native-installed-resource-profile-library-resource-optin.py <當版library>，租裝入口API入口run-native-installed-resource-profile-entry-guard.py <當版library>。{link('額外71項工具',Path('D:/lineage開發/3-lito開發/lito重建版/驗證紀錄/current/native-shared-world-identity-tooling/rebind-20261019-212410-388835/tooling.json'))}已按實際library產生71項逐名計畫並完成本文審查，沿原批次接續，勿另啟同批。
   租裝入口API已在map500返回遇到原始8秒逾時（見失敗表）；先正常GUI入村，再實跑旅館返回並確認正式合法路徑，依證據修復；不得放寬時限或將村莊／旅館分支混為同案。同端點驗傳送後依實收目錄重新入村。{w['current_native_nondefault_initial_fixture_preparation']['source_manifest_sha256'][:8]}{link('初始fixture（未編譯）',w['current_native_nondefault_initial_fixture_preparation']['prepared'])}及{w['current_native_nondefault_same_endpoint_probe_preparation']['source_manifest_sha256'][:8]}{link('非預設probe／被動觀測候選',w['current_native_nondefault_same_endpoint_probe_preparation']['candidate'])}僅準備、未編譯／連線。{link('正式75DA素質／傳送九方法靜態契約',w['current_native_nondefault_stat_contract']['readback'])}及{link('選角素質投影原始碼契約',V/'native-stat-projection-contract/20261019-205216-618042/readback.json')}不等於runtime；將14個伺服器欄位的唯讀觀測與9個封包素質分列，不聲稱其餘5項由線上封包收到。非預設候選只新增不可變角色紀錄的被動觀測，未改handler／建構流程／時限。{link('當版非預設compile／run／qualify／checkpoint工具（僅準備）',V/'native-nondefault-stat-tooling/runtime-review-20261019-214548-922114/tooling.json')}將直接解讀79-byte S_CB suffix與37-byte S0C；仍未編譯／連線／授資格。{link('當版廣域117目標／118程式工具（僅準備）',V/'native-installed-resource-profile-broad-tooling/20261019-213704-825525/review-20261019-213841-532113/tooling.json')}需當版控制器96sessions資格後執行。
4. 自行完成正常GUI兩版：快速／自然超30秒來源交接、租裝入口隱藏普通頻道、入村／背包／技能、旅館入房與房主C10返回、共享鑰匙／訪客離房／自然3600與7200秒到期、傳送取消恢復、普通服務、雙玩家戰鬥與退出。原房主C10讀取器只證返回選角，不能代替訪客／共享租約／自然到期。編譯不可與計時服務並行。
5. 完成當版廣域、Release全流程、可攜交付與剩餘G01–G12，再安排人工測驗；開發期間不要求使用者登入、找視窗或輸入測試帳密。

| 範圍 | 尚待 |
|---|---|
| G02–G04 | 最終來源雙場戰鬥／結算／多人同步／碰撞／世界生命週期、當版廣域／控制器及忽略條件 |
| G05–G06 | 物品裝備技能矩陣、非預設素質、有效同端點服務、旅館返回／共享鑰匙／訪客／自然租約及其他服務；辨識技能書不等於施法 |
| G07–G09 | 全地圖動作／遮擋屋頂排序相機完整frame、正常GUI像素／滑鼠／IME／多DPI、生命週期與實體音效 |
| G10–G12 | 最終Release全流程、乾淨主機可攜交付、效能／長時間／故障恢復／逐項稽核 |

Sky圖像擷取已多次為FrameArrived／window capture timeout，座標geometry unavailable；文字／鍵盤可用，圖像／滑鼠未通過。唯讀桌面WinSta0／Default無鎖定證據，不要求解鎖，不改用替代擷取驅動。GUI外層等待900／1800秒不改遊戲時限；自然租約長測需另設足夠外層生命週期。

## 5. 邊界與接續入口

重建版與登入器／伺服器獨立、產物不同；相鄰專案與原客戶端只讀，替換登入器JAR舊提案撤回。不碰正式玩家表／程序，不執行或包裝舊LiTo.bin。產物限重建版，總報告為授權例外；不派代理或其他對話。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current。固定正式JAR 75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371，catalog B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01；只用重建版內副本。

接續讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。{link('整理前完整總報告',V/'master-report-consolidation/20261019-044931-124021/LiTo381-總開發報告.整理前完整原文.md')} SHA-256：65C64E6068EB50FD3355F3C352F54ED46FA73A808A4DBA8F962979B6A9CB4176。
'''
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True)
prior=REPORT.read_bytes();(out/'before.md').write_bytes(prior);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes()==prior and all(sha(R/n)==h for n,h in before.items())
REPORT.write_bytes((out/'after.md').read_bytes())
receipt=dict(report=str(REPORT),source_manifest_sha256=source,normal_qualified=normal_ok,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(prior).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(receipt,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(receipt,ensure_ascii=False))
