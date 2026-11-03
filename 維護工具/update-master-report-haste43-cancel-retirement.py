"""Compact current-source handoff, keeping prior qualifications explicitly separate."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md';read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
link=lambda label,p:'['+label+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='E0E6409E0F436D705CA55486A9EB86AEE3F483C247201EC3AAD7596F8C49517D'
app=w['current_native_installed_resource_profile_application'];assert app['source_manifest_sha256']==source
application=Path(w['last_goal_turn_evidence'])
apps=[p for p in(V/'native-installed-resource-profile-normal').glob('application-*/application.json') if read(p).get('source_manifest_sha256')==source];assert len(apps)==1
app_path=apps[0];assert sha(app['candidate'])==app['candidate_sha256'] and sha(app['candidate_review'])==app['candidate_review_sha256']
assert all(sha(R/n)==h for n,h in read(app_path.with_name('applied-compiled-inputs.json')).items())
history=w['qualification_before_haste43_cancel_retirement'];assert history['source_manifest_sha256'].startswith('A695') and sha(history['state'])==history['state_sha256'];old=read(history['state'])
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
rows=[]
def current(key,label,flag='qualified'):
    q=w.get(key,{})
    if q.get('source_manifest_sha256')==source and q.get(flag) and q.get('qualification'):
        assert sha(q['qualification'])==q['qualification_sha256'];rows.append('| '+label+' | 已限定核對 | '+link('資格',q['qualification'])+' |')
    elif q.get('source_manifest_sha256')==source and q.get('observed_exit_code') not in [None,0] and q.get('run'):
        failed_run=Path(q['run']);failed_cohort=read(failed_run/'cohort.json');assert failed_cohort['status']=='failed_preserved'
        details='／'.join(p+str(x['passed'])+'過'+str(x['failed'])+'敗' for p,x in failed_cohort['profiles'].items() if 'passed' in x)
        rows.append('| '+label+' | '+(details+'；Release未啟動' if key=='current_native_library_shared_resource_optin' else '失敗已封存')+' | '+link('原失敗',failed_run/'cohort.json')+' |')
    else:rows.append('| '+label+' | 新來源待核對 | 前版資格保留 |')
current('current_native_installed_resource_profile_normal_qualification','正常EXE／19冒煙兩版','qualification_granted')
lib=w.get('current_native_catalog_library',{})
library_complete=lib.get('source_manifest_sha256')==source and w['current_full_default_library_verified']
if library_complete:
    assert sha(lib['completion'])==lib['completion_sha256'];counts='／'.join(p+str(x['passed'])+'過'+str(x['failed'])+'敗'+str(x['ignored'])+'忽略' for p,x in lib['profiles'].items());rows.append('| 完整library兩版 | '+counts+' | '+link('逐名／程序核對',lib['completion'])+' |')
else:
    active_library=w.get('current_native_catalog_library_active',{});partial=active_library.get('independently_qualified_profiles',{}) if active_library.get('source_manifest_sha256')==source else {}
    for proof in partial.values():assert sha(proof['qualification'])==proof['qualification_sha256']
    state='／'.join(p+(str(partial[p]['passed'])+'過0敗'+str(partial[p]['ignored'])+'忽略' if p in partial else'待完成') for p in ['Debug','Release'])
    rows.append('| 完整library兩版 | '+state+' | '+('／'.join(link(p+'限定核對',proof['qualification']) for p,proof in partial.items()) if partial else '預期1975過、488忽略；實際結果為準')+' |')
for key,label in [('current_native_library_resource_optin','原資源10'),('current_native_library_shared_resource_optin','現行資源71'),('current_library_motion_resource2','腳步／NPC資源2'),('current_library_haste_resource2','加速效果資源2'),('current_native_rental_return_reconnect_api_qualification','正式返回API16'),('current_native_catalog_controller_qualification','控制器六分支96')]:current(key,label)
broad=w.get('current_native_catalog_broad_qualification',{})
if broad.get('source_manifest_sha256')==source:
    assert sha(broad['completion_observation'])==broad['completion_observation_sha256'];rows.append('| 廣域117目標＋main兩版 | 已限定核對 | '+link('逐名／236映像',broad['completion_observation'])+' |')
else:rows.append('| 廣域117目標＋main兩版 | 新來源待核對 | 不含完整library／GUI |')
for key,label in [('current_normal_GUI_room_service_keyboard','普通房間／密碼表單'),('current_normal_GUI_rental_entry_keyboard','租裝入口'),('current_normal_GUI_hotel_return_keyboard','旅館返回'),('current_normal_GUI_quantity_Enter_negatives','Enter三負向'),('current_normal_GUI_EOF_cleanup','EOF清理負向')]:
    done=[]
    for p in ['Debug','Release']:
        q=w.get(key,{}).get(p,{})
        if q.get('source_manifest_sha256')==source and q.get('qualification'):
            assert sha(q['qualification'])==q['qualification_sha256'];done.append(p)
    rows.append('| GUI '+label+' | '+('／'.join(done)+'限定通過' if done else '新來源待驗')+' | 完整GUI與像素另驗 |')
active=[]
for name,info in w['active_workloads'].items():active.append(name+'：原session **'+str(w['active_workload_sessions'][name])+'**，'+link('原批次',info.get('run',info.get('case'))))
activity='；'.join(active)+'。只續原程序，不重啟；來源與對應QA固定。' if active else '沒有已登記活動批次，依下列順序接續。'
proofs=read(app['candidate'])['proofs'];assert all(sha(p['path'])==p['sha256'] for p in proofs.values())
skill_scope=V/'native-current-skill-request-scope/E0E6409E-20261020-120226-575433/scope.json';ss=read(skill_scope);assert ss['source_manifest_sha256']==source and ss['recognized_count']==38 and ss['missing_count']==36 and ss['static_only'] and sha(ss['current_request_source'])==ss['current_request_source_sha256']
library_next=('正常兩版各19冒煙及完整library兩版已獨立核對，不重跑；直接接續以下尚未核對範圍。' if library_complete else '正常兩版各19冒煙已限定核對；完整library只續原session 5199，完成後獨立核對兩版逐名結果、PE、來源與程序終止，再接續下列範圍。')
body=f'''# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成原生剩餘開發，先完整library Debug／Release，再Release回歸與正常GUI全流程；自行修復、自動驗證後才交人工。G01–G12未完成，尚未交人工。

## 當版與停點

當版 **E0E6409E**，SHA-256：`{source}`。已套用六檔（3個Runtime、3個測試檔），1048來源／1049編譯輸入：#43加速術自用／對人與原生畫面範圍；取消先保存必要傳送回復關聯，再退役世界／房間／角色／技能／服務。原測試斷言保留，新增取消回歸1項。未改正式JAR、原8／30／60秒、租期或Release設定。{link('套用／原檔回復／審查',app_path)}。

{activity}

{link('工作佇列',R/'工作佇列.json')}；{link('最新八檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。新來源不得沿用A695／D308資格，表內待驗即未完成。

| E0E6409E範圍 | 目前結果 | 證據／限制 |
|---|---|---|
{chr(10).join(rows)}

## 套用依據與前版結果

A695已完成：正常兩版各19；完整library各1974過0敗488忽略；額外互斥85資源兩版、餘403項；廣域Debug544／Release540過、各24忽略，236測試映像；API16與控制器96sessions。GUI普通房間8、租裝入口8、旅館16及EOF負向4sessions均兩版限定核對，Enter三負向另讀回旅館原sessions。{link('A695完整接續報告',history['prior_report'])}；{link('A695資格與失敗原始索引',history['state'])}。這些結果未移記新來源。

#43隔離候選：兩版各3個原版預期反例／20項通過；正式原Mage書自用／對人共8sessions、4施放，MP110→70→30、成本40，動作19／755／S29與個別1200／0秒通知、重按不多送且不預測。低MP另8sessions、2請求，MP2不變、無動作／755／速度，原2秒NoResponse。{link('技能元件',proofs['current_native_haste43_components']['path'])}；{link('正式正向',proofs['current_native_haste43_self_peer_formal75']['path'])}；{link('低MP負向',proofs['current_native_haste43_low_MP_refusal']['path'])}。均屬套用前候選，正常新來源仍須重驗；未實等1200秒。

取消缺陷：原控制器兩版各4sessions，傳輸已靜止且Closed仍留map564／房間／技能／速度。候選另8sessions確認有限755退役後取消即清空場景且generation保留；兩版原10項返回測試全過、新回歸原版各失敗1、候選各11過0敗。{link('原反例',old['current_native_cancel_retirement_counterexample']['qualification'])}；{link('正式修補',proofs['current_native_cancel_retirement_formal_repair']['path'])}；{link('元件回歸',proofs['current_native_cancel_retirement_components']['path'])}。未推定GUI顯示故障。

原54314普通房間900秒外層到期已封存；新13723事先改GUI外層1800、accept900不變並完整限定通過。EOF原reader誤禁必要D9、房間reader誤限set_value、C9併流順序，以及探針組裝編譯失敗均已保留原始拒讀／失敗；只修正工具後核對，不刪原結果。詳見A695索引與各資格raw。

## 接續順序

1. {library_next}不在計時GUI伺服器旁編譯。
2. 依新來源重新審查並執行85項互斥資源、廣域117目標＋main、正式返回API及控制器；已核對的新來源範圍不重跑。403其餘忽略逐名前提確認，不全開。5個嚴格舊TW項缺根目錄TW13081901.pak（要求FC7B4CAC），launcher封存檔不能替代；仍未執行。
3. 重備E0E6409E fixture／reader，完成正常GUI房間、入口、旅館、Enter負向、EOF及取消清理，再補#43正常GUI自用／對人／低MP／舊目標與owner退役。鍵盤／可及性已恢復；畫面擷取仍FrameArrived逾時、點擊geometry unavailable未證明恢復，不重試相同條件，不把CPU PNG當GUI像素。
4. 完成共享鑰匙／訪客離房、原3600／7200秒自然到期、傳送取消／同端點恢復、普通服務與雙玩家戰鬥退出。自然到期須實際等待，不能注入時鐘或縮短租期。既有A695／D308計畫僅供重備，不移記資格。
5. {link('新來源靜態技能盤點',skill_scope)}：原74種觀測ID已識別38／餘36；沒有新增技能書觀測或施放，識別不等於完整技能完成。須新來源正式與GUI核對。繼續其餘玩法／世界生命週期、全地圖資源／遮擋／相機、物品技能矩陣、GUI像素／滑鼠／IME／多DPI、實體音效、效能與長時間故障恢復、乾淨主機Release可攜交付及最終稽核，才安排人工驗收。

## 邊界與接續入口

重建版、登入器、伺服器是獨立專案與產物；相鄰專案及原客戶端資源只讀，替換登入器JAR提案已撤回。不碰玩家實表／正式程序，不執行或包裝舊LiTo.bin；產物限重建版，總報告為授權例外。不派代理、其他對話或Git，不要求使用者登入／找測試視窗。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。正式JAR75DA69B0、catalog B64ED920，完整SHA見cohort。GUI只用Computer Use node_repl＋@oai/sky，一次輸入立即新觀測。

下次先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新八檔checkpoint。現行報告工具update-master-report-haste43-cancel-retirement.py；舊quantity-enter版僅用A695，不可再對新來源執行。
'''
assert all(Path(p).exists() for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir();old_report=REPORT.read_bytes();(out/'before.md').write_bytes(old_report);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());assert REPORT.read_bytes()==old_report and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
j=dict(source_manifest_sha256=source,before_sha256=hashlib.sha256(old_report).hexdigest().upper(),after_sha256=sha(REPORT),active_sessions=w['active_workload_sessions'],lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_text(json.dumps(j,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(j,ensure_ascii=False))
