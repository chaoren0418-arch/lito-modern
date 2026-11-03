"""Concise evidence-scoped continuation after the native hotel Return focus defect."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
link=lambda text,p:'['+text+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='D9B0F468FA65E186B246269A6F00B8D6B4EB76742049E3FBCC744357A780022D'
prior=w['qualification_before_hotel_return_focus_repair'];assert prior['source_manifest_sha256']=='38CF45B68805DCEF4EF2A2733A444C4B6D7FA74F6C16FE80565DCDFBEB82CA38'
app=next(p for p in(V/'native-installed-resource-profile-normal').glob('application-*/application.json')if read(p)['source_manifest_sha256']==source)
application=read(app);assert application['changed_files']==['LiTo381-Modern/src/modern_lito/native_login_window.rs']
for proof in [prior['normal'],prior['resource'],prior['shared_resource'],prior['return_API'],prior['focused'],*prior['GUI_entry'].values()]:assert sha(proof['qualification'])==proof['qualification_sha256']
assert sha(prior['library']['completion'])==prior['library']['completion_sha256']
failure=prior['hotel_focus_failure'];assert sha(failure['raw_archive'])==failure['raw_archive_sha256']and not failure['return_C10_sent']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
rows=[]
for key,label,flag in [('current_native_rental_return_reconnect_focused','返回定位十項','qualified'),('current_native_installed_resource_profile_normal_qualification','正常兩版與19冒煙','qualification_granted'),('current_native_library_resource_optin','原生77十項','qualified'),('current_native_library_shared_resource_optin','現行71資源','qualified'),('current_native_rental_return_reconnect_api_qualification','正式返回API','qualified')]:
 proof=w.get(key,{})
 if proof.get('source_manifest_sha256')==source and proof.get(flag)is True:
  assert sha(proof['qualification'])==proof['qualification_sha256'];q=read(proof['qualification']);counts='／'.join(profile+str(info['passed'])+'過'+str(info['failed'])+'敗'for profile,info in q['profiles'].items());rows.append('| '+label+' | '+counts+' | '+link('限定資格',proof['qualification'])+' |')
 else:rows.append('| '+label+' | 當版未完成 | 不移記38CF資格 |')
lib=w.get('current_native_catalog_library',{});lib_ok=lib.get('source_manifest_sha256')==source and w['current_full_default_library_verified']is True
if lib_ok:
 assert sha(lib['completion'])==lib['completion_sha256'];counts='／'.join(profile+str(info['passed'])+'過'+str(info['failed'])+'敗'+str(info['ignored'])+'忽略'for profile,info in lib['profiles'].items());rows.insert(2,'| 完整library兩版 | '+counts+' | '+link('完成核對',lib['completion'])+' |')
else:
 rows.insert(2,'| 完整library兩版 | 當版未完成 | 預期各1974過／488忽略，須依實際2462項inventory核對 |')
 if'full_library'in w['active_workloads']:
  run=Path(w['active_workloads']['full_library']['run'])
  for profile in ['Debug','Release']:
   qp=run/profile/'full-default-qualified/qualified.json'
   if qp.exists():
    q=read(qp);assert q['source_manifest_sha256']==source and q['failed']==q['filtered_out']==q['Rust_exit_code']==0;rows.append('| 完整library '+profile+' | '+str(q['passed'])+'過0敗'+str(q['ignored'])+'忽略 | '+link('單版核對',qp)+'；兩版總結待完成 |')
for key,label in [('current_normal_GUI_rental_entry_keyboard','GUI租裝入口'),('current_normal_GUI_hotel_return_keyboard','GUI旅館入房／返回')]:
 for profile,proof in w.get(key,{}).items():
  if proof.get('source_manifest_sha256')==source and proof.get('qualification'):
   assert sha(proof['qualification'])==proof['qualification_sha256'];rows.append('| '+label+' '+profile+' | 鍵盤流程限定核對 | '+link('資格',proof['qualification'])+'；非完整GUI |')
active=[]
for key,item in w['active_workloads'].items():
 run=Path(item.get('run',item.get('case','')));cohort=read(run/('cohort.json'if(run/'cohort.json').exists()else'receipt.json'));assert cohort['source_manifest_sha256']==source;active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**；'+cohort['status']+'；'+link('原批次',run)+'。續原程序，勿重啟。')
current='\n'.join(active)if active else'沒有已登記活動批次；依下列順序接續。'
room_defect=w.get('current_normal_GUI_room_form_label_defect',{})
room_note=''
if room_defect.get('source_manifest_sha256')==source:
 assert sha(room_defect['raw_archive'])==room_defect['raw_archive_sha256']
 room_note='普通房間Debug GUI已實跑4sessions並正常退出：建立／加入／切隊／準備取消／修改／離房之封包與權威狀態已核對，但建立及修改表單ROOM_NAME213誤標「新密碼」、CAPACITY214缺名稱，未授普通房間驗收。'+link('原始缺陷與限定觀測',Path(room_defect['raw_archive']).with_name('evidence.json'))+'。下一步先將三個STATIC標籤各自放在對應EDIT之前；保留控制項ID／樣式／佈局／鍵盤順序與協定，修後重跑完整library兩版及正常GUI。'
triage=V/'native-rental-return-reconnect-library-triage/20261019-225252-673497/triage.json'
normal_now=w.get('current_native_installed_resource_profile_normal_qualification',{});normal_ok=normal_now.get('source_manifest_sha256')==source and normal_now.get('qualification_granted')is True
step_library=('當版定位十項與正常兩版各19冒煙已完成；'+('完整library兩版已核對，保留原證據，勿重跑已過原批次。'if lib_ok else'沿已登記full_library原session完成兩版，再獨立核對實際PE／逐名結果／忽略盤點與程序終止；勿重啟。'))if normal_ok else'先完成當版返回定位十項，再正常兩版唯一入口與19冒煙，其後完整library兩版；逐層核對來源／PE／原始結果，失敗先封存再修。'
def current_gui(key,profile):
 proof=w.get(key,{}).get(profile,{})
 return proof.get('source_manifest_sha256')==source and bool(proof.get('qualification'))
debug_hotel=current_gui('current_normal_GUI_hotel_return_keyboard','Debug')
gui_remaining=[profile+' rental-entry' for profile in ['Debug','Release'] if not current_gui('current_normal_GUI_rental_entry_keyboard',profile)]
gui_remaining += [profile+' 旅館返回' for profile in ['Debug','Release'] if not current_gui('current_normal_GUI_hotel_return_keyboard',profile)]
gui_stage=('Debug旅館返回已獨立核對8sessions；'if debug_hotel else'')+('接續尚未通過的'+ '、'.join(gui_remaining)+'。'if gui_remaining else'當版租裝入口與旅館返回兩版均已通過限定鍵盤資格，接續其餘GUI情境。')
resource_complete=all(w.get(key,{}).get('source_manifest_sha256')==source and w.get(key,{}).get('qualified')is True for key in ['current_native_library_resource_optin','current_native_library_shared_resource_optin','current_native_rental_return_reconnect_api_qualification'])
resource_stage='當版原生77十項、現行71資源及正式返回API兩版均已獨立核對，勿重跑已過批次。'if resource_complete else'依當版資格表完成尚缺的原生77十項、現行71資源及兩版正式返回API。'
gui_compiled='當版GUI fixture已javac成功並核對來源／class／兩版四份正常EXE；Debug旅館入房／返回已完成限定鍵盤資格。'if debug_hotel else'當版GUI fixture已javac成功並核對來源／class／兩版四份正常EXE；GUI實跑範圍依上表。'
body=f"""# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成剩餘原生開發，先完整library Debug／Release，再正常GUI全流程與Release回歸；自行修復及自動驗證後才交人工。G01–G12未完成，未交人工。

## 1. 目前停點

當版 **{source[:8]}** 只修native_login_window.rs：一般UI更新跳過租裝RETURN，讓既有refresh_rental統一決定啟用狀態，避免先停用再啟用搶走鍵盤焦點。返回控制器、原C10與一次重新認證恢復、測試斷言、JAR及時限都不改。來源1,048檔／編譯輸入1,049；SHA-256：`{source}`。{link('套用與回復',app)}；{link('工作佇列',R/'工作佇列.json')}；{link('最新八檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。

{current}

| 當版範圍 | 結果 | 證據／限制 |
|---|---|---|
{chr(10).join(rows)}

{room_note}

## 2. 本次實際GUI失敗與修復

38CF45B6正常Debug GUI：A已走13步至旅館、收到8把鑰匙，背包45→53並進入map503；返回按鈕107可見，但Tab移入後焦點退回主視窗。一般更新先EnableWindow(false)，refresh_rental再EnableWindow(true)，造成返回按鈕焦點反覆退役。尚未按下返回、未送C10；B未連線，Release旅館未執行。兩視窗正常關閉後，只退役核對出生時間／映像的自有fixture server；原88712 wrapper1／server1是失敗清理，非客戶端返回崩潰。{link('完整raw／原來源與失敗',Path(failure['raw_archive']).with_name('evidence.json'))}。此次不授旅館或完整GUI資格。EOF後舊session.rental_hotel可保留失效參照；新GUI fixture已準備唯讀registry.lease(token,實際System.nanoTime)觀測來核對有效租約，不能以舊欄位是否null代替。{link("唯讀方法／class契約",V/"native-hotel-lease-readonly-contract/20261019-235209-650029/review.json")}；{gui_compiled}；{link("當版GUI準備核對",V/"native-installed-resource-profile-sky-gui/prepared-20261020-001753-857137/compile-readback.json")}。觀測不輸出每次時戳，保留原trace去重。

原恢復修補保留：正式75在租裝BATTLE的C10/S65/D9之後沒有完成F000；原8秒到期後先退役舊世界與worker，再用原LOGIN端點、既有Zeroizing帳密、實收原來源走一次真正LOGIN／GAME，取得空權威角色目錄，不自選角色、不沿用舊背包或房間。取消／失敗清掉恢復意圖；一次失敗即停止。33E93E93取消競態與603C8359原返回失敗仍保留各自raw，不改記成功。

## 3. 最近完整基準（只屬38CF45B6）

| 範圍 | 實際結果 | 證據 |
|---|---|---|
| 定位與正常兩版 | Debug十項全過；正常兩版各19過0敗 | {link('定位',prior['focused']['qualification'])}；{link('正常',prior['normal']['qualification'])} |
| 完整library兩版 | 各1974過0敗488忽略／0篩除；11組記錄程序終止 | {link('完成核對',prior['library']['completion'])} |
| 額外資源兩版 | 原生77各10過；現行71各71過 | {link('原生77',prior['resource']['qualification'])}；{link('現行71',prior['shared_resource']['qualification'])} |
| 正式返回API兩版 | 各1案全過／8sessions，合16；8組記錄身分終止 | {link('獨立封包／權威核對',prior['return_API']['qualification'])}；原8秒後重新認證，非F000成功或GUI |
| 正常GUI入口兩版 | 各4sessions：快／自然超30秒交接、map500移動、45／43背包、技能顯示恢復與正常退出 | {link('Debug',prior['GUI_entry']['Debug']['qualification'])}；{link('Release',prior['GUI_entry']['Release']['qualification'])}；非旅館或完整GUI |

2462項inventory含488預設opt-in；當版及38CF各實跑互斥81本文，仍407未執行：5項缺指定舊根目錄TW，另402需逐項審查Java／TLS／SQL等前提與協定版本，不能全開或fallback。{link('488逐名條件盤點',triage)}僅靜態清單。71資源保留現行Text37497／28361、現行launcher TW、全部原像素／owner雜湊與角色ID斷言；修復後的新來源仍須重驗。

## 4. 接續順序與剩餘工作

1. {step_library} 不在計時伺服器旁編譯。
2. {resource_stage}{link("當版71項計畫／審查",V/"native-installed-resource-profile-shared-resource-plan/20261020-000516-913970/review.json")}。工具及候選依當版來源重綁；QA不放寬原斷言、8秒／peer60秒／30秒或正式authority。
3. 使用當版prepared-20261020-001753-857137正常GUI EXE，{gui_stage}同原13步、8／1鑰匙、map503／501，每角色至少兩次純觀測hotel-return-focus、相隔≥0.25秒，均須焦點107，再按一次Space hotel-return；核對真實重新認證返回與registry有效租約／鑰匙退役。舊38CF prepared-20261019-231730-823574不可移記新來源。當版讀取器qualify-native-rental-return-reconnect-GUI-hotel-return.py已綁定；完整GUI尚未通過。
4. 繼續共享鑰匙／訪客離房／自然3600與7200秒到期、傳送／取消恢復、普通服務、雙玩家戰鬥／退出；控制器96sessions、同端點／非預設素質、廣域117整合目標＋main118與Release全流程。當版工具已重綁；同端點候選native-installed-resource-profile-same-endpoint-D9B0F468與非預設候選native-current-nondefault-passive-20261020-000249-855741已依當版正常資格準備，均未編譯／實跑。普通頻道／房間GUI情境ordinary-room-service已準備，使用原room-admin profile，未執行；非預設14個伺服器欄位只有9個線上素質投影，不聲稱其餘5項在線上收到。自然到期外層8000秒框架僅準備，獨立完整讀取器仍待。
5. G01–G12剩餘玩法／生命週期、全地圖資源／遮擋／相機、物品技能矩陣、GUI像素／滑鼠／IME／多DPI、實體音效、效能／長時間／故障恢復、乾淨主機Release可攜交付及最終稽核完成後才交人工。

## 5. 邊界與工具

重建版與登入器／伺服器獨立、產物不同。相鄰專案、D:/lineage開發/0-客戶端等原資源只讀；替換登入器JAR舊提案撤回。不碰玩家實表／正式程序、不執行或包裝舊LiTo.bin。產物限重建版，總報告為授權例外；不派代理、其他對話或使用Git，不要求使用者登入或尋找視窗。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。正式JAR SHA `75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371`，catalog `B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01`，只用重建版固定副本。

GUI只用Computer Use的node_repl＋@oai/sky，一次輸入後立即新觀測；焦點未確認不可按Space。Sky文字／鍵盤可用，既有圖像擷取FrameArrived／capture timeout及geometry unavailable尚無恢復證據，不重複同條件擷取、不以CPU PNG當GUI像素資格；桌面無鎖定證據，不要求解鎖。

下次先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。現用報告工具update-master-report-hotel-return-focus.py；舊rental-return-reconnect／installed-resource-profile報告工具不再適用。
"""
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);b=REPORT.read_bytes();(out/'before.md').write_bytes(b);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());assert REPORT.read_bytes()==b and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
record=dict(source_manifest_sha256=source,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(b).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(record,ensure_ascii=False))
