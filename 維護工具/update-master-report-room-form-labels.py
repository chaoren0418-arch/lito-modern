"""Compact, source-bound handoff after the actual normal room form label defect."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
link=lambda text,p:'['+text+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256']
assert source=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
prior=w['qualification_before_room_form_label_repair'];assert prior['source_manifest_sha256']=='D9B0F468FA65E186B246269A6F00B8D6B4EB76742049E3FBCC744357A780022D'
prior=dict(prior)
prior['GUI_hotel_return']=prior.get('GUI_hotel_return',w['current_normal_GUI_hotel_return_keyboard'])
prior['room_form_label_defect']=prior.get('room_form_label_defect',prior.get('hotel_room_form_label_defect'))
assert set(prior['GUI_hotel_return'])=={'Debug','Release'} and all(x['source_manifest_sha256']==prior['source_manifest_sha256']for x in prior['GUI_hotel_return'].values())
for proof in [prior['normal'],prior['resource'],prior['shared_resource'],prior['return_API'],prior['focused'],*prior['GUI_entry'].values(),*prior['GUI_hotel_return'].values()]:assert sha(proof['qualification'])==proof['qualification_sha256']
assert sha(prior['library']['completion'])==prior['library']['completion_sha256']
defect=prior['room_form_label_defect'];assert sha(defect['raw_archive'])==defect['raw_archive_sha256'] and not defect['normal_GUI_room_form_labels_correct']
app=V/'native-installed-resource-profile-normal/application-20261020-012736-485092/application.json';application=read(app);assert application['source_manifest_sha256']==source
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
rows=[]
for key,label,flag in [('current_native_installed_resource_profile_normal_qualification','正常Debug／Release及19冒煙','qualification_granted'),('current_native_library_resource_optin','原生77十項兩版','qualified'),('current_native_library_shared_resource_optin','現行71資源兩版','qualified'),('current_native_rental_return_reconnect_api_qualification','正式返回API兩版','qualified')]:
 proof=w.get(key,{})
 if proof.get('source_manifest_sha256')==source and proof.get(flag)is True:
  assert sha(proof['qualification'])==proof['qualification_sha256'];q=read(proof['qualification']);counts='／'.join(p+str(v['passed'])+'過'+str(v['failed'])+'敗'for p,v in q['profiles'].items());rows.append('| '+label+' | '+counts+' | '+link('限定核對',proof['qualification'])+' |')
 else:rows.append('| '+label+' | 當版待完成 | 依活動批次／後續順序執行 |')
lib=w.get('current_native_catalog_library',{});lib_ok=lib.get('source_manifest_sha256')==source and w['current_full_default_library_verified']is True
if lib_ok:
 assert sha(lib['completion'])==lib['completion_sha256'];counts='／'.join(p+str(v['passed'])+'過'+str(v['failed'])+'敗'+str(v['ignored'])+'忽略'for p,v in lib['profiles'].items());rows.insert(1,'| 完整library兩版 | '+counts+' | '+link('完成核對',lib['completion'])+' |')
else:rows.insert(1,'| 完整library兩版 | 當版待完成 | 2462項inventory；預期各1974／0／488，依實際逐名核對，不把忽略當通過 |')
if not lib_ok and 'full_library' in w['active_workloads']:
 run=Path(w['active_workloads']['full_library']['run'])
 for profile in ['Debug','Release']:
  qp=run/profile/'full-default-qualified/qualified.json'
  if qp.exists():
   q=read(qp);assert q['source_manifest_sha256']==source and q['failed']==q['filtered_out']==q['Rust_exit_code']==0
   rows.append('| 完整library '+profile+'單版 | '+str(q['passed'])+'過0敗'+str(q['ignored'])+'忽略 | '+link('單版核對',qp)+'；兩版總結待完成 |')
for key,label in [('current_normal_GUI_room_service_keyboard','GUI普通房間／表單'),('current_normal_GUI_rental_entry_keyboard','GUI租裝入口'),('current_normal_GUI_hotel_return_keyboard','GUI旅館入房／返回')]:
 for profile in ['Debug','Release']:
  proof=w.get(key,{}).get(profile,{})
  if proof.get('source_manifest_sha256')==source and proof.get('qualification'):
   assert sha(proof['qualification'])==proof['qualification_sha256'];rows.append('| '+label+' '+profile+' | 鍵盤流程限定核對 | '+link('資格',proof['qualification'])+'；非完整GUI |')
  else:rows.append('| '+label+' '+profile+' | 當版待完成 | 不移記D9資格 |')
active=[]
for key,item in w['active_workloads'].items():
 run=Path(item.get('run',item.get('case','')));receipt=run/('cohort.json'if(run/'cohort.json').exists()else'receipt.json');c=read(receipt);assert c['source_manifest_sha256']==source
 active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**；'+c['status']+'；'+link('原批次',run)+'。只續原程序，勿重啟；產品與對應QA凍結。')
current='\n'.join(active)if active else'沒有已登記活動批次；接續下列尚未完成項目。'
normal_now=w.get('current_native_installed_resource_profile_normal_qualification',{})
normal_ok=normal_now.get('source_manifest_sha256')==source and normal_now.get('qualification_granted')is True
step_library=('當版正常兩版與19冒煙已獨立核對；'+('完整library兩版也已核對，保留原通過批次，接續下項。'if lib_ok else'沿已登記full_library原session完成兩版，勿重啟。'))if normal_ok else'沿活動normal原session完成正常兩版及19冒煙，其後同來源完整library Debug／Release。'
resource_ok=all(w.get(k,{}).get('source_manifest_sha256')==source and w.get(k,{}).get('qualified') is True for k in ['current_native_library_resource_optin','current_native_library_shared_resource_optin'])
resource_status='當版兩版已實跑互斥81本文，仍407未執行' if resource_ok else '當版81本文尚未全數完成；D9成果僅留歷史'
focused=w.get('current_native_rental_return_reconnect_focused',{})
if focused.get('source_manifest_sha256')==source and focused.get('qualified') is True:
 assert sha(focused['qualification'])==focused['qualification_sha256']
 rows.append('| 返回／取消十項回歸 | 當版兩版各10過0敗 | '+link('完整library內逐名核對',focused['qualification'])+'；未另重跑 |')
controller=w.get('current_native_catalog_controller_qualification',{})
if controller.get('source_manifest_sha256')==source and controller.get('qualified') is True:
 assert sha(controller['qualification'])==controller['qualification_sha256'] and controller['actual_sessions']==96
 rows.append('| 兩版控制器六分支 | 12案／96sessions | '+link('限定核對',controller['qualification'])+'；非GUI／完整G06 |')
else:rows.append('| 兩版控制器六分支 | 當版待完成 | 以原活動批次逐案核對 |')
positive_plan=V/'native-room-form-label-tooling/GUI-positive-plan-20261020-013403-619696/plan.json'
assert sha(positive_plan)=='371A24D1DDA0D92632CDD54D28FBF0939C91CA177A47E26F840AD08545E4BE22'
body=f'''# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成剩餘原生開發，先完整library Debug／Release，再正常GUI全流程與Release回歸；自行修復及自動驗證後才交人工。完整G01–G12未完成，未交人工。

## 1. 當版與停點

當版 **D308C3E2** 僅重排native_login_window.rs內房間表單三組既有STATIC／EDIT建立順序：314→213房名、315→214人數、316→215新密碼。控制項ID、文字、樣式、佈局及EDIT鍵盤順序不變；返回焦點修復與控制器、JAR、8秒／30秒／60秒原時限保留。來源1048檔／編譯輸入1049；SHA-256：`{source}`。{link('套用／回復',app)}；{link('工作佇列',R/'工作佇列.json')}；{link('最新八檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。

{current}

| 當版範圍 | 結果 | 證據／限制 |
|---|---|---|
{chr(10).join(rows)}

## 2. 本次實際GUI發現

D9B0F468 Debug普通房間正常GUI已實跑4sessions，原37239實際exit0、四組程序身分皆終止。建立GUI測試房／4人、第二角色加入、切隊2、準備與取消、房主改名GUI改名房／5人、雙方離房及房間移除，均有原鍵盤／文字、封包及權威狀態核對；無SQL或authority注入。但建立及修改表單ROOM_NAME213誤標「新密碼」，CAPACITY214缺可及名稱，因此不授普通房間或完整GUI資格。{link('缺陷與限定觀測／raw',Path(defect['raw_archive']).with_name('evidence.json'))}。當版修補的實跑結果見上表；必須核對建立、修改及啟用密碼後的真正EDIT名稱與CC／DD／CF封包，不以靜態排序代替GUI。

背景視窗的即時UIA回覆曾短暫保留舊成員；啟用後純觀察可收到正確切隊／準備，原觀測保留。擷取檔為合併封包：僅核對聚合雙份狀態通知，不聲稱已逐socket證明投遞。此批未測戰鬥、觀戰／踢人、密碼加入、模式／地圖切換、滑鼠或畫面像素。

## 3. 修補前最近完整基準（只屬D9B0F468）

| 範圍 | 實際結果 | 證據 |
|---|---|---|
| 定位／正常兩版 | Debug定位10過；正常兩版各19過0敗 | {link('正常',prior['normal']['qualification'])}；{link('定位',prior['focused']['qualification'])} |
| 完整library兩版 | 各1974過0敗488忽略／0篩除，11組程序終止 | {link('完成核對',prior['library']['completion'])} |
| 額外資源／返回API兩版 | 各10＋71資源全過；API各8sessions，合16 | {link('原生77',prior['resource']['qualification'])}；{link('現行71',prior['shared_resource']['qualification'])}；{link('API',prior['return_API']['qualification'])} |
| 正常GUI入口兩版 | 各4sessions：快／自然超30秒交接、map500移動、45／43背包與技能顯示恢復、正常退出 | {link('Debug',prior['GUI_entry']['Debug']['qualification'])}；{link('Release',prior['GUI_entry']['Release']['qualification'])} |
| 正常GUI旅館兩版 | 各8sessions：13步、8／1鑰匙、map503／501、返回焦點保持、重新認證回原乙角色目錄，有效租約／鑰匙退役及正常退出 | {link('Debug',prior['GUI_hotel_return']['Debug']['qualification'])}；{link('Release',prior['GUI_hotel_return']['Release']['qualification'])}；非共享／自然到期或完整GUI |

正式租裝返回保留C10/S65/D9原流程；原8秒未完成F000後退役舊世界與worker，再用原LOGIN端點、既有Zeroizing帳密及實收原來源走一次真實LOGIN／GAME，取得空角色目錄，不自選、不沿用舊物品或房間。取消／失敗清掉恢復意圖，一次失敗即停止。38CF旅館焦點失敗、33E取消競態與603C原返回失敗保留原始證據，不改記成功。舊session租約參照可能仍存在，有效租約須以已核對唯讀registry getter確認。

## 4. 接續順序

1. {step_library}逐名核對結果、實際PE、來源、忽略盤點及所有記錄程序終止。通過批次不重跑；不在計時伺服器旁編譯。
2. 依上表接續同來源尚未完成的原生77十項、現行71資源與兩版正式返回API，再編譯當版GUI fixture。49個後續工具已重綁D308；舊D9 prepared-20261020-001753-857137不可當成新來源GUI產物。原488忽略中{resource_status}，須逐項審查前提：5缺指定舊TW、402其他Java／TLS／SQL等條件；不全開、不fallback、不移記新來源。
3. 依上表接續尚未完成的兩版GUI。普通房間依{link('當版正向計畫',positive_plan)}與讀取器qualify-native-room-form-labels-GUI-room-service.py回驗建立／修改三個可及名稱、啟用密碼、雙角色密碼加入／切隊／準備取消／修改／離房；再租裝入口與旅館返回兩版。旅館每角色兩次純焦點107觀測相隔≥0.25秒後，單次Space返回；核對原時限、真實重新認證與有效租約／鑰匙退役。
4. 繼續共享鑰匙／訪客離房／自然3600與7200秒到期、傳送／取消恢復、普通服務、雙玩家戰鬥退出；同來源控制器96sessions、同端點／非預設素質與廣域117整合目標＋main118。當版同端點候選native-installed-resource-profile-same-endpoint-D308C3E2、非預設fixture prepared-20261020-013842-356008及候選native-current-nondefault-passive-20261020-013850-888696已準備，未編譯／實跑；非預設14初始欄位僅9個在線上投影。{link('自然到期兩版四情境計畫',V/'native-room-form-label-tooling/natural-expiry-GUI-plan-20261020-014501-282698/plan.json')}已準備，未實跑；原有中型到期profile已接到GUI工具，8000秒外層框架不改遊戲租期，獨立完整讀取器仍待。
5. 完成其餘G01–G12玩法／生命週期、全地圖資源／遮擋／相機、物品技能矩陣、GUI像素／滑鼠／IME／多DPI、實體音效、效能／長時間／故障恢復、乾淨主機Release可攜交付及最終稽核，才安排人工驗收。

## 5. 邊界與工具

重建版與登入器／伺服器獨立、產物不同；可只讀參考線索。相鄰專案與D:/lineage開發/0-客戶端等原資源只讀；替換登入器JAR舊提案撤回。不碰玩家實表／正式程序、不執行或包裝舊LiTo.bin。產物限重建版，總報告為授權例外；不派代理、其他對話或Git，不要求使用者登入／找視窗。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。正式JAR固定副本75DA69B0、catalog B64ED920，完整雜湊在各cohort／qualification。GUI只用Computer Use node_repl＋@oai/sky，一次輸入立即新觀測，焦點未確認不按Space。既有圖像擷取FrameArrived／timeout及geometry unavailable未恢復，不重複同條件嘗試，不把CPU診斷PNG當GUI像素證明；無桌面鎖定證據，不要求解鎖。

下次先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。現用報告工具update-master-report-room-form-labels.py；舊hotel-return-focus／rental-return-reconnect／installed-resource-profile報告工具不再適用。
'''
failure=w.get('current_normal_GUI_hotel_return_failure',{})
if failure.get('source_manifest_sha256')==source:
 assert sha(failure['raw_archive'])==failure['raw_archive_sha256'] and not failure['GUI_qualification_granted']
 note='當版Release旅館原8317失敗保留：外層900秒結束時僅6／8登入，B返回後EOF；server／pipeline exit1，B由原retained handle清理，未正常退出或通過。'+link('失敗／raw',Path(failure['raw_archive']).with_name('evidence.json'))+'。僅此八連線GUI情境的外層操作框架改1800秒；'+link('重編fixture核對',V/'native-installed-resource-profile-sky-gui/prepared-20261020-032053-361901/orchestration-readback.json')+'證明1049輸入、四正常PE、權威與原遊戲／協定時限未改。僅重驗未通過的Release；EOF樹仍含舊房間成員文字，須另查失敗清理，不以重試成功抹去。\n\n'
 body=body.replace('## 3. 修補前最近完整基準',note+'## 3. 修補前最近完整基準',1)
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
quantity_failure=w.get('current_normal_GUI_hotel_return_quantity_failure',{})
if quantity_failure.get('source_manifest_sha256')==source:
 assert sha(quantity_failure['raw_archive'])==quantity_failure['raw_archive_sha256'] and not quantity_failure['positive_hotel_return_acceptance_granted']
 note='Release重驗原35904仍未通過：兩次數量Space操作的觀測耗時33.26／33.65秒，捕獲C51為0，不能稱伺服器收到後拒絕；完整物品／素質／有效租約皆未變。可及點擊亦回geometry unavailable，兩GUI正常關閉後僅清理精確owned伺服器；原exit1、2sessions及'+link('原始證據',Path(quantity_failure['raw_archive']).with_name('evidence.json'))+'保留。先推進獨立控制器／Release整合，同時檢查原時限內的數量鍵盤操作及EOF清理；不得將此批升格正向返回或完整GUI。\n\n'
 body=body.replace('## 3. 修補前最近完整基準',note+'## 3. 修補前最近完整基準',1)
candidate=R/'維護工具/candidates/native-quantity-enter-failure-retirement-20261020-034156-331023/candidate.json'
if candidate.exists():
 cp=read(candidate);assert cp['before_source_manifest_sha256']==source and not cp['production_source_modified'] and not cp['compiled']
 note='已準備隔離候選'+cp['candidate_source_manifest_sha256'][:8]+'，'+link('差異與候選',candidate)+'：數量欄Enter走原ORDER handler並排除按鍵重複；fail在保存必要傳送關聯後清除世界／房間／服務／角色，擴充兩個原有負向測試。僅3檔候選，未套用／編譯，不改原30秒或JAR；先等活動控制器原批次完成，再驗證缺陷及候選，不覆寫凍結來源。\n\n'
 body=body.replace('## 3. 修補前最近完整基準',note+'## 3. 修補前最近完整基準',1)
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);b=REPORT.read_bytes();(out/'before.md').write_bytes(b);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes()==b and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
record=dict(source_manifest_sha256=source,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(b).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(record,ensure_ascii=False))
