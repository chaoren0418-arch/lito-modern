"""Concise source-scoped handoff for the rental-return reconnect repair."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
link=lambda label,p:'['+label+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='38CF45B68805DCEF4EF2A2733A444C4B6D7FA74F6C16FE80565DCDFBEB82CA38'
app=w['current_native_installed_resource_profile_application'];assert app['source_manifest_sha256']==source
application=next(p for p in (V/'native-installed-resource-profile-normal').glob('application-*/application.json')if read(p)['source_manifest_sha256']==source)
prior=w['qualification_before_rental_return_reconnect'];assert prior['source_manifest_sha256']=='603C835908762CAF643F3ECA995039745DD6BE428FE9F533261B226BA2AD744A'
for record in [prior['normal'],prior['resource'],prior['shared_resource']]:assert sha(record['qualification'])==record['qualification_sha256']
assert sha(prior['library']['completion'])==prior['library']['completion_sha256']
failure=prior['guard_failure'];assert sha(failure['raw_archive'])==failure['raw_archive_sha256']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
rows=[]
f=w.get('current_native_rental_return_reconnect_focused',{})
if f.get('source_manifest_sha256')==source and f.get('qualified')is True:
 assert sha(f['qualification'])==f['qualification_sha256'];rows.append('| Debug返回定位回歸 | 10過0敗／2452篩除 | '+link('限定資格',f['qualification'])+'；一條未認證owned-loopback連線，非正式伺服器 |')
else:rows.append('| Debug十項返回回歸 | 編譯／執行／核對待完成 | 見原活動批次 |')
previous_failure=w.get('current_native_rental_return_reconnect_failure',{})
if previous_failure:
 assert sha(previous_failure['raw_archive'])==previous_failure['raw_archive_sha256']
 rows.append('| 前次33E93E93返回定位 | Debug9過1敗，未授資格 | '+link('失敗封存',Path(previous_failure['raw_archive']).with_name('evidence.json'))+('；當版十項已全過 |'if f.get('source_manifest_sha256')==source and f.get('qualified')else'；取消競態已補，待原斷言重驗 |'))
n=w.get('current_native_installed_resource_profile_normal_qualification',{});normal_ok=n.get('source_manifest_sha256')==source and n.get('qualification_granted')is True
if normal_ok:
 assert sha(n['qualification'])==n['qualification_sha256'];q=read(n['qualification']);rows.append('| 正常Debug／Release與冒煙 | 各19過0敗 | '+link('限定資格',n['qualification'])+' |')
 for profile,h in q['normal_mains'].items():rows.append('| '+profile+'正常EXE SHA-256 | `'+h+'` | 唯一入口建置 |')
else:rows.append('| 正常Debug／Release與冒煙 | 尚未通過當版資格 | 前版正常EXE不可移記當版 |')
lib=w.get('current_native_catalog_library',{});lib_ok=lib.get('source_manifest_sha256')==source and w['current_full_default_library_verified']is True
if lib_ok:
 assert sha(lib['completion'])==lib['completion_sha256'];rows.append('| 完整library兩版 | '+'／'.join(p+str(i['passed'])+'過'+str(i['failed'])+'敗'+str(i['ignored'])+'忽略'for p,i in lib['profiles'].items())+' | '+link('完成核對',lib['completion'])+' |')
else:rows.append('| 完整library兩版 | 尚未通過當版資格 | 新增4個unit，預期1974過／488忽略，須依實際inventory核對 |')
if not lib_ok and 'full_library'in w['active_workloads']:
 active_run=Path(w['active_workloads']['full_library']['run'])
 for profile in ['Debug','Release']:
  qp=active_run/profile/'full-default-qualified/qualified.json'
  if qp.exists():
   partial=read(qp);assert partial['source_manifest_sha256']==source and partial['status']=='independently_qualified_current_full_library_default_selection'
   assert partial['failed']==partial['filtered_out']==partial['Rust_exit_code']==0
   rows.append('| 完整library '+profile+' | '+str(partial['passed'])+'過0敗'+str(partial['ignored'])+'忽略／0篩除 | '+link('單版獨立核對',qp)+'；兩版總結與程序終止尚待 |')
for key,label in [('current_native_library_resource_optin','原生77十項'),('current_native_library_shared_resource_optin','現行71項資源'),('current_native_rental_return_reconnect_api_qualification','正式API入口／一次重新認證返回')]:
 r=w.get(key,{})
 if r.get('source_manifest_sha256')==source and r.get('qualified')is True:
  assert sha(r['qualification'])==r['qualification_sha256'];proof=read(r['qualification'])
  counts='／'.join(profile+str(info['passed'])+'過'+str(info['failed'])+'敗'for profile,info in proof['profiles'].items())
  if key=='current_native_rental_return_reconnect_api_qualification':counts+='；共'+str(proof['actual_sessions'])+'sessions'
  rows.append('| '+label+' | '+counts+' | '+link('資格',r['qualification'])+'；非GUI |')
for key,label in [('current_normal_GUI_rental_entry_keyboard','GUI入口'),('current_normal_GUI_hotel_return_keyboard','GUI旅館返回')]:
 for profile,r in w.get(key,{}).items():
  if r.get('source_manifest_sha256')==source and r.get('qualification'):
   assert sha(r['qualification'])==r['qualification_sha256'];rows.append('| '+label+' '+profile+' | 鍵盤流程限定通過 | '+link('資格',r['qualification'])+'；非完整GUI |')
active=[]
for key,r in w['active_workloads'].items():
 run=Path(r.get('run',r.get('case','')));receipt=run/('cohort.json'if(run/'cohort.json').exists()else'receipt.json');c=read(receipt)
 assert c['source_manifest_sha256']==source
 active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**；'+c['status']+'；'+link('原批次',run)+'。續原程序，勿另啟同批。')
current='\n'.join(active)if active else'目前沒有已登記活動批次；依下列順序接續。'
gui=prior['GUI_entry']['Debug'];assert sha(gui['qualification'])==gui['qualification_sha256']
triage_path=V/'native-rental-return-reconnect-library-triage/20261019-225252-673497/triage.json';triage=read(triage_path);assert triage['source_manifest_sha256']==source and triage['ignored']==488

resource_ok=all(w.get(k,{}).get('source_manifest_sha256')==source and w.get(k,{}).get('qualified')is True for k in ['current_native_library_resource_optin','current_native_library_shared_resource_optin'])
api_ok=w.get('current_native_rental_return_reconnect_api_qualification',{}).get('source_manifest_sha256')==source and w.get('current_native_rental_return_reconnect_api_qualification',{}).get('qualified')is True
gui_entry_ok=all(w.get('current_normal_GUI_rental_entry_keyboard',{}).get(profile,{}).get('source_manifest_sha256')==source and w.get('current_normal_GUI_rental_entry_keyboard',{}).get(profile,{}).get('qualification') for profile in ['Debug','Release'])
step_library='當版定位十項、正常兩版19冒煙及完整library兩版均已完成，沿表格保留各自證據，不重跑已通過的原批次。'if lib_ok else'先沿已登記原session完成完整library Debug／Release，核對inventory、實際PE、忽略本文、原始輸出及程序終止；失敗先封存再修。'
step_api='當版81項額外資源與正式API兩版各8sessions均已獨立核對；保留原C10/S65後8秒、真實LOGIN／GAME、原來源角色目錄及再次選角守門證據。此為重新認證恢復，不聲稱F000握手成功。'if resource_ok and api_ok else'完整library後執行原生77十項、現行71項資源與run／qualify／checkpoint-native-rental-return-reconnect-api.py；兩版各8sessions，原GameHub建構、handler與8秒／peer60秒／30秒不動。核對真實重新認證、原來源、舊權威退役及再次選角守門。'
step_gui=('當版正常GUI租裝入口Debug／Release已完成；下一項為兩版旅館入房與rental-hotel-return-reconnect返回，使用prepared-20261019-231730-823574及qualify-native-rental-return-reconnect-GUI-hotel-return.py。'if gui_entry_ok else'當版library核對後prepare正常GUI，Debug／Release各先rental-entry，再rental-hotel-return-reconnect返回。')+' 新旅館情境為8sessions；旅館原3600／7200秒自然到期的外層8000秒框架已準備但未執行，需獨立補完整讀取器。原4sessions入口API與舊旅館返回讀取器不移記；正常程式不加診斷API或source overlay。'
body=f"""# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成剩餘原生開發，先完整library Debug／Release，再GUI全流程／Release回歸；自行修復與驗證後才交人工。完整G01–G12未完成，未交人工。

## 1. 目前停點

當版 **{source[:8]}**：新增租裝返回一次性重新登入恢復及四個unit；修改native_login.rs與native_rental_return_tests.rs。來源1,048檔／編譯輸入1,049；SHA-256：`{source}`。{link('套用／回復與來源清單',application)}；{link('工作佇列',R/'工作佇列.json')}；{link('最新八份接續檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。

{current}

| 當版範圍 | 結果 | 證據與限制 |
|---|---|---|
{chr(10).join(rows)}

## 2. 目前修復與原失敗

603C8359完整library、81項額外資源與Debug GUI入口已通過，但正式入口API的村莊map500返回失敗：兩人均已入村，C10／S65／D9立即完成，原8秒內未收到F000；Debug0過1敗、Rust101、wrapper1，Release僅編譯未執行。{link('失敗、封包及正式方法封存',Path(failure['raw_archive']).with_name('evidence.json'))}。分段讀取假說已被封包排除；正式75的quitFight／exitRoom在BATTLE先走settleRequestedBattleExit，未完成租裝返回確認。旅館的當版結果以上表為準，村莊封包不直接當作旅館資格。

新修復已補取消競態：舊worker已終止時，cancel也必須轉Closing，禁止落回一般handoff。原33E93E93十項Debug9過1敗／Rust101，取消後留下HandoffClosing使redirect.unwrap崩潰；原測試未放寬、原來源與PE已封存。新修復保留原C10交握，僅在已收到一次S65且原8秒等待逾時後退役舊世界，等待舊worker終止，再經原LOGIN端點、既有Zeroizing帳密、實收的原來源，取得新的GAME角色目錄；不自選角色、不沿用舊房間／配裝／背包。一次恢復失敗即停止；明確start／cancel／fail清除恢復意圖與帳密。四個unit檢查未確認／缺來源不可連線、取消不可重連、舊權威先退役且僅一條owned-loopback連線；unit的局部deadline注入不改正常8秒／peer60秒。候選與套用資料已封存；正式API與正常GUI須依各自當版證據核對。

## 3. 前版已核對基準（只屬603C8359）

| 範圍 | 結果 | 證據 |
|---|---|---|
| 正常兩版與冒煙 | 各19過0敗 | {link('限定資格',prior['normal']['qualification'])} |
| 完整library兩版 | 各1970過0敗488忽略／0篩除；11記錄程序皆終止 | {link('完成核對',prior['library']['completion'])} |
| 額外資源 | 原生77各10過；現行71各71過 | {link('原生77',prior['resource']['qualification'])}；{link('現行71',prior['shared_resource']['qualification'])} |
| Debug正常GUI入口 | 4sessions：快速／自然超30秒來源交接、map500移動、45／43背包、技能顯示恢復、正常退出 | {link('限定資格',gui['qualification'])}；不是旅館、Release或完整GUI資格 |

71項已修復Text現行索引37497／28361、現行launcher TW及繪製角色ID／遮罩斷言；所有原像素／owner雜湊保留。488預設忽略中，額外81本文已於前版實跑，仍407未執行：5項缺指定舊根目錄TW封包，另402有各自Java／TLS／SQL等條件，不能fallback或全開忽略。{link("當版488逐名條件盤點",triage_path)}僅靜態清單，不授執行資格。更早BC2C／9F／4018失敗、A9 GUI與控制器、AB廣域結果仍留原來源；{link('原source603C GUI證據',gui['qualification'])}。

## 4. 接續順序與剩餘工作

1. {step_library}
2. {step_api}
3. {step_gui}
4. 其後完成共享鑰匙／訪客離房／自然3600與7200秒到期、傳送／取消恢復、普通服務、雙玩家戰鬥／退出；再控制器96sessions、同端點／非預設素質、廣域117整合目標＋main118程式與Release全流程。非預設14個伺服器欄位／9個封包素質、同端點候選與廣域117工具已重綁當版並準備，尚未編譯／實跑；非預設候選native-current-nondefault-passive-20261019-224956-689470。原始5項素質不得聲稱由線上封包收到。
5. 完成G01–G12剩餘玩法、生命週期、全地圖資源／遮擋／相機、物品技能矩陣、正常GUI像素／滑鼠／IME／多DPI、實體音效、效能／長時間／故障恢復及乾淨主機Release可攜交付，才安排人工。

Sky文字／鍵盤可用；圖像擷取既有FrameArrived／window capture timeout、geometry unavailable，未有恢復證據，不重複相同擷取或以CPU PNG當正常GUI像素資格。桌面WinSta0／Default無鎖定證據，不要求解鎖。GUI只用Computer Use的node_repl＋@oai/sky，一次輸入後立即新觀測；不請使用者登入或找視窗。編譯不可與計時伺服器並行，原活動source／對應QA固定。

## 5. 邊界與接續入口

重建版與登入器／伺服器獨立、產物不同；相鄰專案與原資源只讀，替換登入器JAR舊提案撤回。不碰玩家實表／正式程序，不執行或包裝舊LiTo.bin。產物限重建版，總報告為授權例外；不派代理、其他對話或使用Git。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。正式JAR `75DA69B0D2F7F9D394EE6BF2FDB396B153C75D9B938069B0999ED8DAC4F50371`，catalog `B64ED920792805C3380AD0E50B1AE089A8779B4ADDDEC2FBF3780C2FA88A3C01`，只用重建版內固定副本。

接續讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。報告更新工具為update-master-report-rental-return-reconnect.py；舊installed-resource-profile報告工具不再適用。
"""
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);prior_bytes=REPORT.read_bytes();(out/'before.md').write_bytes(prior_bytes);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes()==prior_bytes and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
record=dict(source_manifest_sha256=source,normal_qualified=normal_ok,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(prior_bytes).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(record,ensure_ascii=False))
