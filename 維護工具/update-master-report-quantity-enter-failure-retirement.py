"""Compact handoff for the quantity keyboard and failure-retirement repair."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'))
def sha(p):
    with Path(p).open('rb')as f:return hashlib.file_digest(f,'sha256').hexdigest().upper()
link=lambda text,p:'['+text+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');source=w['current_product_identity']['source_manifest_sha256'];assert source=='A695C12799E2D7A508223ABE55CE5B5BDC81A15A85CE05451DF60672FC8823C6'
h=w['qualification_before_quantity_enter_failure_retirement'];assert h['source_manifest_sha256']=='D308C3E29A667B76F5B495D09EF212B4EF445646F5E8B42924261CB8AE963041'
for p in [h['normal'],h['resource'],h['shared_resource'],h['return_API'],h['focused'],h['controller'],h['negative_baseline'],*h['GUI_room'].values(),*h['GUI_entry'].values(),*h['GUI_hotel_return'].values()]:assert sha(p['qualification'])==p['qualification_sha256']
assert sha(h['library']['completion'])==h['library']['completion_sha256']
for key in ['GUI_operator_budget_failure','GUI_quantity_failure']:assert sha(h[key]['raw_archive'])==h[key]['raw_archive_sha256']
application=w['current_native_installed_resource_profile_application'];assert application['source_manifest_sha256']==source
apps=[p for p in (V/'native-installed-resource-profile-normal').glob('application-*/application.json')if read(p).get('source_manifest_sha256')==source];assert len(apps)==1
app=apps[0];assert sha(application['candidate'])==application['candidate_sha256']
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==digest for n,digest in before.items())
rows=[]
for key,label,flag in [('current_native_installed_resource_profile_normal_qualification','正常Debug／Release及冒煙','qualification_granted'),('current_native_library_resource_optin','額外原資源10項兩版','qualified'),('current_native_library_shared_resource_optin','現行71資源兩版','qualified')]:
    p=w.get(key,{})
    if p.get('source_manifest_sha256')==source and p.get(flag)is True:
        assert sha(p['qualification'])==p['qualification_sha256'];q=read(p['qualification']);counts='／'.join(profile+str(x['passed'])+'過'+str(x['failed'])+'敗'for profile,x in q['profiles'].items());rows.append('| '+label+' | '+counts+' | '+link('限定核對',p['qualification'])+' |')
    else:rows.append('| '+label+' | 當版待完成 | 不移記D308資格 |')
motion2=w.get('current_library_motion_resource2',{})
motion2_ok=motion2.get('source_manifest_sha256')==source and motion2.get('qualified')is True
if motion2_ok:
    assert sha(motion2['qualification'])==motion2['qualification_sha256']
    mq=read(motion2['qualification']);assert mq['additional_ignored_bodies_executed']==2 and mq['original_identities_terminal']
    rows.append('| 新增腳步／NPC資源兩版 | Debug2過0敗／Release2過0敗 | '+link('限定核對',motion2['qualification'])+'；無音效裝置／GUI |')
resource_total=83 if motion2_ok else 81
haste2=w.get('current_library_haste_resource2',{})
haste2_ok=haste2.get('source_manifest_sha256')==source and haste2.get('qualified')is True
if haste2_ok:
    assert motion2_ok and sha(haste2['qualification'])==haste2['qualification_sha256']
    hq=read(haste2['qualification']);assert hq['additional_ignored_bodies_executed']==2 and hq['original_identities_terminal']
    assert not set(hq['names'])&set(motion2['names'])
    rows.append('| #43／54效果資源兩版 | Debug2過0敗／Release2過0敗 | '+link('原SPR／RGBA／時序限定核對',haste2['qualification'])+'；非施放／GUI |')
    resource_total+=2
resource_remaining=488-resource_total
resource_queue_note='其中新增兩項已另列資格，從原407清單扣除；其餘仍須逐名前提審查。'if motion2_ok else'分類僅供搜尋，不代表前提成立或已執行。'
if haste2_ok:resource_queue_note='其中腳步／NPC及#43／54效果共4項已另列資格，從原407清單扣除；其餘仍須逐名前提審查。'
haste_resource_plan=V/'native-current-library-haste-resource2/execution-plan-20261020-083528-461441/plan.json'
if haste_resource_plan.exists()and not haste2_ok:
    hrp=read(haste_resource_plan);assert hrp['source_manifest_sha256']==source and hrp['expected_tests_per_profile']==2
    assert sha(Path(hrp['library'])/'completion-observation.json')==hrp['library_completion_sha256']
    assert sha(R/'維護工具/run-current-library-haste-resource2.py')==hrp['runner_sha256']and sha(hrp['reader'])==hrp['reader_sha256']
    resource_queue_note+=' '+link('#43／54效果兩項補驗計畫',haste_resource_plan)+'與獨立reader已備、尚未執行；不扣減剩餘數。'
legacy_preflight=V/'native-strict-legacy-TW-prerequisites/20261020-084852-631354/evidence.json'
if legacy_preflight.exists():
    lp=read(legacy_preflight);assert lp['source_manifest_sha256']==source and lp['tests_executed']==0 and not lp['prerequisite_satisfied']
    assert all(sha(R/row['source'])==row['source_sha256']for row in lp['tests'])
    resource_queue_note+=' 5個嚴格舊TW項缺根目錄TW13081901.pak（要求FC7B4CAC）；launcher封存檔B3788DF4及純文字不能替代，故未執行。'+link('唯讀前提核對',legacy_preflight)+'。'
library=w.get('current_native_catalog_library',{});lib_ok=library.get('source_manifest_sha256')==source and w['current_full_default_library_verified']is True
if lib_ok:
    assert sha(library['completion'])==library['completion_sha256'];counts='／'.join(p+str(x['passed'])+'過'+str(x['failed'])+'敗'+str(x['ignored'])+'忽略'for p,x in library['profiles'].items());rows.insert(1,'| 完整library兩版 | '+counts+' | '+link('逐名／程序完成核對',library['completion'])+' |')
else:
    active_lib=w.get('current_native_catalog_library_active',{});partial=active_lib.get('independently_qualified_profiles',{})if active_lib.get('source_manifest_sha256')==source else{}
    for profile,proof in partial.items():
        assert sha(proof['qualification'])==proof['qualification_sha256'];q=read(proof['qualification']);assert q['source_manifest_sha256']==source and q['profile']==profile and q['failed']==q['Rust_exit_code']==0
    counts='；'.join(profile+(str(partial[profile]['passed'])+'過0敗'+str(partial[profile]['ignored'])+'忽略'if profile in partial else'待完成')for profile in ['Debug','Release'])
    refs='／'.join(link(profile+'限定核對',proof['qualification'])for profile,proof in partial.items())or'實際逐名核對；忽略另列'
    rows.insert(1,'| 完整library兩版 | '+counts+' | '+refs+'；雙版資格仍待 |')
for key,label,count in [('current_native_rental_return_reconnect_api_qualification','正式返回API兩版',16),('current_native_catalog_controller_qualification','控制器六分支兩版',96)]:
    p=w.get(key,{})
    if p.get('source_manifest_sha256')==source and p.get('qualified')is True:
        assert sha(p['qualification'])==p['qualification_sha256'];q=read(p['qualification']);assert q.get('actual_sessions',q.get('actual_formal_sessions'))==count
        rows.append('| '+label+' | '+str(count)+' sessions限定核對 | '+link('資格',p['qualification'])+'；非GUI |')
    else:rows.append('| '+label+' | 當版待完成 | 保留前版獨立結果 |')
broad=w.get('current_native_catalog_broad_qualification',{})
if broad.get('source_manifest_sha256')==source:
    assert sha(broad['completion_observation'])==broad['completion_observation_sha256']and broad['recorded_original_owned_processes_all_terminal']
    counts='／'.join(profile+str(info['counts']['passed'])+'過'+str(info['counts']['failed'])+'敗'+str(info['counts']['ignored'])+'忽略'for profile,info in broad['profiles'].items())
    rows.append('| 廣域117整合目標＋main兩版 | '+counts+' | '+link('236映像／程序核對',broad['completion_observation'])+'；非library／GUI |')
else:
    broad_progress=w.get('current_native_catalog_broad_progress',{})
    partial=broad_progress.get('qualified_profiles',{})if broad_progress.get('source_manifest_sha256')==source else{}
    if partial:
        for profile,proof in partial.items():
            assert sha(proof['qualification'])==proof['sha256']
            assert read(proof['qualification'])['source_manifest_sha256']==source and proof['actual_test_executables']==118
        counts='／'.join(profile+(str(partial[profile]['counts']['passed'])+'過0敗'+str(partial[profile]['counts']['ignored'])+'忽略'if profile in partial else'待完成')for profile in ['Debug','Release'])
        rows.append('| 廣域117整合目標＋main兩版 | '+counts+' | '+'／'.join(link(profile+'限定讀回',proof['qualification'])for profile,proof in partial.items())+'；雙版及程序終止待完成 |')
for key,label in [('current_normal_GUI_room_service_keyboard','普通房間／密碼與表單'),('current_normal_GUI_rental_entry_keyboard','租裝入口'),('current_normal_GUI_hotel_return_keyboard','旅館入房／返回'),('current_normal_GUI_EOF_cleanup','斷線清理負向')]:
    for profile in ['Debug','Release']:
        p=w.get(key,{}).get(profile,{})
        if p.get('source_manifest_sha256')==source and p.get('qualification'):
            assert sha(p['qualification'])==p['qualification_sha256'];q=read(p['qualification']);rows.append('| GUI '+label+' '+profile+' | '+str(q['actual_formal_sessions'])+' sessions限定核對 | '+link('資格',p['qualification'])+'；非完整GUI |')
        else:rows.append('| GUI '+label+' '+profile+' | 當版待完成 | 須新來源實跑 |')
enter_profiles=[]
for profile,p in w.get('current_normal_GUI_quantity_Enter_negatives',{}).items():
    if p.get('source_manifest_sha256')==source:
        assert sha(p['qualification'])==p['qualification_sha256'];q=read(p['qualification'])
        assert q['no_request_C51']==q['additional_consumed_C51']==q['expired_request_C51']==0 and q['total_C51']==2
        enter_profiles.append(profile)
        rows.append('| GUI數量Enter三負向 '+profile+' | 無請求／已消耗／自然失效均無額外C51 | '+link('原8sessions讀回',p['qualification'])+'；非按住重複／像素 |')
enter_pending=[profile for profile in ['Debug','Release']if profile not in enter_profiles]
enter_state=('已通過'+ '／'.join(enter_profiles)+'三負向限定讀回'+('；'+'／'.join(enter_pending)+'仍待'if enter_pending else'；不授像素／完整GUI資格'))if enter_profiles else'僅備妥、尚無實跑資格'
eof_reader_note=''
for profile,p in w.get('current_normal_GUI_EOF_cleanup',{}).items():
    if p.get('source_manifest_sha256')==source:
        assert sha(p['qualification'])==p['qualification_sha256'];q=read(p['qualification'])
        assert q['exact_pre_world_rental_arm_D9_F000_F5_CF_verified'] and q['no_post_fault_client_packets']
        for failure_path in q['prior_reader_failure_preserved']:
            failure=read(failure_path);assert sha(Path(failure_path).with_name('raw.zip'))==failure['raw_archive_sha256']
            eof_reader_note=' EOF原reader誤禁入村必要D9，'+link('原失敗／raw',failure_path)+'已保留；現精確核對D9→F000→F5→CF及原房號，斷線後零新客戶端包；只讀回原批次，產品／斷言意圖與期限不變。'
hotel_pending=[profile for profile in ['Debug','Release']if w.get('current_normal_GUI_hotel_return_keyboard',{}).get(profile,{}).get('source_manifest_sha256')!=source]
hotel_next=('／'.join(hotel_pending)+'旅館仍待'if hotel_pending else'旅館兩版及Enter三負向限定通過，見上表')
entry_pending=[profile for profile in ['Debug','Release']if w.get('current_normal_GUI_rental_entry_keyboard',{}).get(profile,{}).get('source_manifest_sha256')!=source]
entry_next=('／'.join(entry_pending)+'租裝入口仍待'if entry_pending else'租裝入口兩版快速／自然逾時交接、移動與背包限定通過')
eof_pending=[profile for profile in ['Debug','Release']if w.get('current_normal_GUI_EOF_cleanup',{}).get(profile,{}).get('source_manifest_sha256')!=source]
eof_next=('／'.join(eof_pending)+'EOF負向仍待'if eof_pending else'EOF負向兩版已確認舊世界／房間／物品／服務退役')
room_pending=[profile for profile in ['Debug','Release']if w.get('current_normal_GUI_room_service_keyboard',{}).get(profile,{}).get('source_manifest_sha256')!=source]
room_next=('先'+'／'.join(room_pending)+'普通房間／密碼與表單'if room_pending else'普通房間／密碼與表單兩版限定通過，接續其餘GUI')
ordinary_failure_note=''
ordinary_failure=w.get('current_normal_GUI_ordinary_operator_deadline_failure',{})
if ordinary_failure.get('source_manifest_sha256')==source:
    assert sha(ordinary_failure['raw_archive'])==ordinary_failure['raw_archive_sha256']
    assert ordinary_failure['original_identities_terminal'] and ordinary_failure['both_GUI_normal_close'] and not ordinary_failure['GUI_qualification_granted']
    ordinary_failure_note='Debug普通房間原54314在最後離房前達900秒GUI外層期限；server／pipeline各1，兩GUI正常退出0、四身分終止，'+link('原失敗／raw',Path(ordinary_failure['raw_archive']).with_name('evidence.json'))+'保留且不授資格。'
    room_debug=w.get('current_normal_GUI_room_service_keyboard',{}).get('Debug',{})
    if room_debug.get('source_manifest_sha256')==source:
        rq=read(room_debug['qualification']);assert rq['observed_original_exit_code']==0 and rq['original_owned_identities_terminal']
        ordinary_failure_note+=' 新批次13723事先改普通房間GUI外層1800秒、accept900及遊戲／協定時限不變；4sessions全流程限定讀回通過，見上表。reader原先誤限來源必用set_value，原拒讀已封存；現以實際焦點／選單甲→乙及四個精確C8核對鍵盤End，密碼亦綁實際EDIT206與CF，未重跑產品或減少欄位斷言。'
cancel_note=''
cancel_proof=w.get('current_native_cancel_retirement_counterexample',{})
if cancel_proof.get('source_manifest_sha256')==source:
    assert sha(cancel_proof['qualification'])==cancel_proof['qualification_sha256'] and cancel_proof['qualified_counterexample']
    cancel_note='新取消退役缺陷已'+link('雙版正式重現',cancel_proof['qualification'])+'：各4sessions，成功加速後755有限效果已退役，取消傳輸均靜止且Closed，快照仍保留map564／房間／角色／技能／速度；兩probe預期1、兩server0、八身分終止。不推定GUI顯示失敗。'
    cancel_candidates=[p for p in(R/'維護工具/candidates').glob('native-cancel-retirement-*/candidate.json')if read(p).get('base_source_manifest_sha256')==source]
    assert len(cancel_candidates)<=1
    if cancel_candidates:
        cp=read(cancel_candidates[0]);assert not cp['source_applied'] and all(sha(R/n)==h for n,h in cp['before'].items())
        cancel_note+=' '+link('兩檔隔離修補候選',cancel_candidates[0])+'已備：先保存傳送回復關聯再無條件退役場景，另增一個回歸測試、原斷言保留；未套用／未驗證。優先完成候選反例、雙版實際取消與來源回歸，再接續其餘GUI。'
    cancel_formal=w.get('current_native_cancel_retirement_formal_repair',{})
    cancel_components=w.get('current_native_cancel_retirement_components',{})
    if cancel_formal.get('source_manifest_sha256')==source and cancel_formal.get('qualified'):
        assert sha(cancel_formal['qualification'])==cancel_formal['qualification_sha256'] and cancel_formal['actual_formal_sessions']==8 and cancel_formal['original_identities_terminal']
        cancel_note=cancel_note.replace('未套用／未驗證。優先完成候選反例、雙版實際取消與來源回歸，再接續其餘GUI。','未套用產品；候選'+link('雙版實際取消',cancel_formal['qualification'])+'各4sessions已核對，755有限退役後取消，世界／角色／技能／房間／服務清空且generation保留。')
    if cancel_components.get('source_manifest_sha256')==source and cancel_components.get('qualified'):
        assert sha(cancel_components['qualification'])==cancel_components['qualification_sha256'] and cancel_components['candidate_passed_each']==11
        cancel_note+=' '+link('元件原斷言與反例',cancel_components['qualification'])+'：兩版原10項返回測試全過，新取消回歸原版各失敗1、候選各11過0敗。下一步彙整六檔套用審查及新來源完整library／Release／GUI重驗；不移記A695資格。'
active=[]
gui_installed=[p for p in (V/'native-quantity-enter-failure-tooling').glob('GUI-readers-installed-*/readback.json')if read(p).get('source_manifest_sha256')==source];assert len(gui_installed)<=1
gui_rules_state='已安裝，執行資格見上表'if gui_installed else'已準備、未安裝'
for key,item in w['active_workloads'].items():
    run=Path(item.get('run',item.get('case','')));receipt=run/('cohort.json'if(run/'cohort.json').exists()else'receipt.json');c=read(receipt);assert c.get('source_manifest_sha256',c.get('normal_product_source_manifest_sha256'))==source
    active.append('- '+key+'：原session **'+str(w['active_workload_sessions'][key])+'**；'+c['status']+'；'+link('原批次',run)+'。只續原程序，產品及對應QA凍結。')
paused=w.get('development_state')=='paused_by_user'
if paused:
    assert not active and not w['active_workload_sessions'];pause=w['user_requested_pause'];assert pause['source_manifest_sha256']==source and pause['status']=='paused_by_explicit_user_request'
current='**已依使用者要求暫停，沒有活動建置／測試。** 收到繼續指示後先更新八檔的paused_by_user標記並保留暫停紀錄；不重跑已核對的當版library與81資源。先正式返回API16，再重備當版GUI fixture，驗證旅館Enter及EOF清理並接續其餘GUI。'if paused else'\n'.join(active)if active else'沒有已登記活動批次；依下列順序接續未完成範圍。'
baseline_next='當版正常兩版各19冒煙與完整library Debug／Release已完成，保留來源、PE、逐名結果與程序終止資格，不重跑。'if lib_ok else'沿活動原session完成當版正常兩版／各19冒煙與完整library Debug／Release，核對逐名結果、PE、來源及程序終止。'
repair_validation='當版完整library已通過，GUI正／負向仍待實跑。'if lib_ok else'當版修復仍須完整library及GUI正／負向實跑。'
if lib_ok and not enter_pending and not eof_pending:repair_validation='當版完整library、旅館Enter正／負向與EOF清理兩版均已限定核對；其餘完整GUI仍待。'
def evidence(p):return Path(p['raw_archive']).with_name('evidence.json')
interruption=w.get('current_normal_GUI_prelogin_interruption',{})
interruption_note=''
if interruption.get('source_manifest_sha256')==source:
    assert sha(interruption['raw_archive'])==interruption['raw_archive_sha256']and not interruption['GUI_qualification_granted']
    interruption_note='當版GUI原'+str(interruption['original_session_id'])+'連線前中斷：0sessions，兩原GUI先後exit0並有external-input通知；原因未證明，未宣稱由自動化正常關閉。僅精確owned等待server清理，原wrapper1及所有原程序終止已核對。'+link('中斷／raw',evidence(interruption))+'；需新批次重驗，不授GUI資格。'
obstruction=w.get('current_GUI_input_runtime_obstruction',{})
obstruction_note=''
if obstruction.get('source_manifest_sha256')==source:
    assert sha(obstruction['evidence'])==obstruction['evidence_sha256']
    assert sha(Path(obstruction['evidence']).with_name('raw.zip'))==obstruction['raw_archive_sha256']
    obstruction_note='原GUI'+str(obstruction['original_session'])+'觀測時兩自有視窗皆最小化；還原遇input guard、刷新仍最小化，已停止同條件重試。當時0sessions，未證明原因／使用者歸責或桌面鎖定；原程序與期限及清理責任不變。Enter負向reader已備、三個實際未合格批次拒授資格；未取得Enter負向或完整GUI通過。'+link('工具阻礙／reader核對',obstruction['evidence'])+'。'
runtime_failure=w.get('current_normal_GUI_input_runtime_failure',{})
if runtime_failure.get('source_manifest_sha256')==source:
    assert sha(runtime_failure['raw_archive'])==runtime_failure['raw_archive_sha256']and runtime_failure['original_identities_terminal']
    obstruction_note='原GUI'+str(runtime_failure['original_session'])+'：兩自有視窗最小化；還原遇input guard，刷新仍最小化。0sessions，精確核對後僅owned三子程序做失敗清理，原wrapper1與四身分終止已封存；不是正常關閉或GUI通過。原因／使用者歸責／桌面鎖定均未證明。Enter負向reader已備，三個實際未合格批次均拒授資格；同條件UI不重試，先續獨立控制器／廣域回歸。'+link('失敗／raw',Path(runtime_failure['raw_archive']).with_name('evidence.json'))+'。'
desktop=w.get('current_user_desktop_availability',{})
desktop_note=''
if desktop.get('source_manifest_sha256')==source:
    assert sha(desktop['evidence'])==desktop['evidence_sha256'] and desktop['user_reply']=='現在桌面可用'
    desktop_note='使用者已回覆「現在桌面可用」，GUI由代理自行登入及操作；可用回覆不等同工具或GUI通過。'+link('桌面可用紀錄',desktop['evidence'])+'。'
recovery=w.get('current_GUI_input_runtime_recovery',{})
gui_next='GUI工具狀態實際恢復後，'
if recovery.get('source_manifest_sha256')==source:
    assert recovery['normal_keyboard_and_accessibility_worked'] and sha(recovery['qualified_hotel_return'])==recovery['qualified_hotel_return_sha256']
    assert not recovery['visual_capture_recovered']
    if recovery.get('capture_attempt'):assert sha(recovery['capture_attempt'])==recovery['capture_attempt_sha256']
    desktop_note+=' 新批次已實測恢復鍵盤／可及性，'+'／'.join(enter_profiles)+'旅館各8sessions及三項Enter負向通過，兩GUI均正常退出；新視窗畫面擷取仍FrameArrived逾時，不授像素／完整GUI資格。'+link('最新恢復與限定資格',recovery.get('checkpoint_evidence',V/'native-installed-resource-profile-sky-gui/case-20261020-090441-063432-release-rental-hotel-return-reconnect/checkpoint-Enter-recovery-20261020-092655-889134/evidence.json'))+'。'
    gui_next='鍵盤／可及性已恢復，'
skill_paths=w.get('current_missing_skill_handler_paths',{})
skill_paths_note=''
if skill_paths.get('path'):
    assert sha(skill_paths['path'])==skill_paths['sha256']and skill_paths['static_only']
    skill_paths_note=' '+link('正式75逐名handler／讀包索引',skill_paths['path'])+'已涵蓋餘37個ID；僅靜態路徑，分支、目標／成本／狀態、資源與實跑仍待。'
packet_hints=V/'native-current-skill-request-scope/emitted-packet-hints-20261020-081904-095828/hints.json'
if packet_hints.exists():
    ph=read(packet_hints);assert ph['source_manifest_sha256']==source and not ph['packet_or_runtime_qualification_granted']
    assert sha(ph['selected_Protocol_methods'])==ph['selected_Protocol_methods_sha256']
    skill_paths_note+=' '+link('正式75封包／詞法路由提示',packet_hints)+'另備；不能以World詞法缺口推定產品解碼缺陷。'
haste_candidates=[path for path in (R/'維護工具/candidates').glob('native-haste43-*/candidate.json')if read(path).get('base_source_manifest_sha256')==source and not(path.parent/'rejection.json').exists()]
assert len(haste_candidates)<=1
if haste_candidates:
    candidate=haste_candidates[0];hc=read(candidate)
    assert hc['source_applied']is False and hc['actual_component_or_library_tests_executed']is False
    assert all(sha(R/name)==digest for name,digest in hc['before'].items())
    assert hc['distinct_written_files']==len(set(hc['candidate_files'].values()))==4
    assert all(sha(hc['candidate_files'][name])==digest for name,digest in hc['after'].items())
    component=w.get('current_native_haste43_components',{})
    component_note='元件尚未核對；先3個原版預期反例再20項候選測試。'
    if component.get('source_manifest_sha256')==source and component.get('qualified'):
        assert sha(component['qualification'])==component['qualification_sha256']
        cq=read(component['qualification']);assert cq['recorded_original_identities_terminal'] and cq['candidate_tests_each']==20 and cq['baseline_expected_failures_each']==3
        assert not cq['candidate_applied'] and cq['actual_formal75_casts']==0
        component_note='隔離元件Debug／Release各3個原版預期反例、候選20過0敗已核對；'+link('元件限定資格',component['qualification'])+'，正常來源與PE未變。'
    tooling_failure=w.get('current_native_haste43_component_tooling_failure',{})
    if tooling_failure.get('source_manifest_sha256')==source:
        assert sha(tooling_failure['original_archive'])==tooling_failure['original_archive_sha256']
        component_note+=' 原26263漏sha2連結，Rust測試未執行；'+link('原編譯失敗／工具修補',Path(tooling_failure['original_archive']).with_name('evidence.json'))+'已封存。'
    live=w.get('current_native_haste43_self_peer_formal75',{})
    live_note='正式75施放尚未核對。'
    if live.get('source_manifest_sha256')==source and live.get('qualified'):
        assert sha(live['qualification'])==live['qualification_sha256'];lq=read(live['qualification'])
        assert lq['actual_formal_sessions']==8 and lq['actual_formal_casts']==4 and lq['original_identities_terminal'] and not lq['candidate_applied']
        live_note='正式75原Mage書、自用／對人兩版共8sessions／4施放已'+link('限定核對',live['qualification'])+'：MP110→70→30、每次40、動作19／755／S29雙方通知、目標1200秒與旁觀0秒欄位；重按不多送且不預測。3個探針組裝失敗及C9併流順序誤限均保留於資格，未重跑已成功施放。低MP拒絕、目標／狀態／生命週期及正常GUI仍待，未實等1200秒。'
    mp_refused=w.get('current_native_haste43_low_MP_refusal',{})
    if mp_refused.get('source_manifest_sha256')==source and mp_refused.get('qualified'):
        assert sha(mp_refused['qualification'])==mp_refused['qualification_sha256'];mpq=read(mp_refused['qualification'])
        assert mpq['actual_formal_sessions']==8 and mpq['actual_formal_requests']==2 and mpq['successful_casts']==0 and mpq['original_identities_terminal']
        live_note=live_note.replace('低MP拒絕、目標／狀態／生命週期及正常GUI仍待，未實等1200秒。','低MP兩版另8sessions／2請求已'+link('限定核對',mp_refused['qualification'])+'：建立前MP2、原書保留，server cost40拒絕，MP／速度不變、無動作／755，原2秒NoResponse及重按不多送。目標／狀態／生命週期及正常GUI仍待，未實等1200秒。')
    skill_paths_note+=' '+link('#43加速術四檔隔離候選',candidate)+'（2個Runtime檔、2個既有測試延伸）未套用產品；'+component_note+live_note+'不計正常產品新增技能通過。首批同名覆寫錯誤'+link('已拒收封存',R/'維護工具/candidates/native-haste43-20261020-074307-266851/rejection.json')+'，只使用新四路徑候選。'
body=f'''# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成剩餘原生開發，先完整library Debug／Release，再正常GUI全流程與Release回歸；自行修復及自動驗證後才交人工。G01–G12未全部完成，未交人工。

## 當版與停點

當版 **{source[:8]}**：數量欄Enter僅轉原ORDER handler、抑制按鍵重複；fail先保存必要傳送關聯，再退役世界／房間／服務／角色。兩個原負向測試保留舊斷言並增加退役檢查。僅3檔，1048來源／1049編譯輸入，原8秒／30秒／60秒、遊戲租期、JAR與Release設定不變。SHA-256：`{source}`。{link('套用／回復',app)}；{link('工作佇列',R/'工作佇列.json')}；{link('最新八檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。

{current}

{obstruction_note}

{desktop_note}

{ordinary_failure_note}

{cancel_note}

| 當版範圍 | 結果 | 證據／限制 |
|---|---|---|
{chr(10).join(rows)}

## 缺陷與前版證據

{interruption_note}

D308完整基準：正常兩版各19過；完整library各1974過0敗488忽略；額外互斥81資源兩版全過；返回API16及控制器96sessions皆獨立核對。正常GUI房間8、租裝入口8、旅館Debug8，共24sessions限定通過，Release旅館未通過。均保留D308資格，不升格當版。{link('library',h['library']['completion'])}；{link('資源10',h['resource']['qualification'])}／{link('71',h['shared_resource']['qualification'])}；{link('API',h['return_API']['qualification'])}；{link('控制器',h['controller']['qualification'])}；{link('房間Debug',h['GUI_room']['Debug']['qualification'])}／{link('Release',h['GUI_room']['Release']['qualification'])}；{link('入口Debug',h['GUI_entry']['Debug']['qualification'])}／{link('Release',h['GUI_entry']['Release']['qualification'])}；{link('旅館Debug',h['GUI_hotel_return']['Debug']['qualification'])}。

原8317：900秒GUI外層期限結束，僅6／8登入，B返回時EOF；server／pipeline exit1，B由原retained handle清理。EOF樹仍含舊房間成員。{link('原失敗／raw',evidence(h['GUI_operator_budget_failure']))}。僅八連線GUI情境外層改1800秒，正式時限及正常PE不變；不把框架修正當GUI通過。

原35904：兩次數量Space操作觀測耗時33.26／33.65秒，C51為0，完整物品／素質／租約未變；不能稱伺服器收到後拒絕。可及點擊回geometry unavailable。兩GUI正常關閉後只清理精確owned伺服器，原exit1、2sessions保留。{link('原失敗／raw',evidence(h['GUI_quantity_failure']))}。另以僅測試斷言覆加確認兩個舊失敗分支保留joined_room，預期Rust101／2反例；原測試來源與原library PE已回復。{link('反例核對',h['negative_baseline']['qualification'])}。{repair_validation}

## 接續順序

1. {baseline_next} 不在計時伺服器旁編譯。
2. 額外資源、返回API16、控制器96與廣域兩版以表列當版資格為準，已通過者不重跑。488忽略中互斥{resource_total}項已另行實跑，餘{resource_remaining}項；{link('原407項逐名前提清單',V/'native-library-remaining-optin-review/20261020-041932-322366/review.json')}含5個嚴格舊TW項，其餘涉及Java／TLS／SQL、固定輸入、裝置或UI等條件。{resource_queue_note}不全開、不fallback。
3. {gui_next}沿已準備的同版fixture／reader接續表列未完成GUI；不重試相同擷取失敗，不重跑已核對library或GUI。{room_next}；{entry_next}；{hotel_next}；{eof_next}。旅館限定資格含原30秒內各一次有效C51、三項Enter負向、8／1鑰匙與map503／501、兩次純焦點107相隔≥0.25秒、單次返回重新認證、租約／鑰匙退役與正常退出；不推定全部GUI或像素通過。
4. 繼續共享鑰匙／訪客離房／原3600與7200秒自然到期、傳送與取消恢復、普通服務、雙玩家戰鬥退出；控制器96sessions及廣域兩版依上表接續未完成範圍，另補同端點／非預設素質。D308已準備的候選僅供重備參考，不移記新來源資格；自然到期須實際等待，不能注入時鐘或縮短租期。
5. {link('當版技能識別盤點',V/'native-current-skill-request-scope/20261020-044016-468694/scope.json')}：對既有74種觀測技能，現請求識別37、餘37待開發；未新觀測技能書或施放，通用decoder接受不等於實際handler契約。{skill_paths_note}繼續G01–G12玩法／生命週期、全地圖資源／遮擋／相機、物品技能矩陣、GUI像素／滑鼠／IME／多DPI、實體音效、效能／長時間／故障恢復、乾淨主機Release可攜交付及最終稽核，才安排人工驗收。

## 邊界與工具

重建版與登入器／伺服器為獨立專案、產物不同；可只讀參考線索，替換登入器JAR舊提案撤回。不碰玩家實表／正式程序，不執行或包裝舊LiTo.bin；產物限重建版，總報告為授權例外。不派代理、其他對話或Git，不要求使用者登入／找視窗。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。正式JAR固定75DA69B0、catalog B64ED920，完整SHA見各cohort。GUI只用Computer Use node_repl＋@oai/sky，一次輸入立即新觀測；已確認FrameArrived／timeout及geometry unavailable未恢復，不重試相同擷取或點擊，不把CPU PNG當GUI像素資格。

當版GUI規則{link(gui_rules_state,V/'native-quantity-enter-failure-tooling/GUI-readers-prepared-20261020-040710-552916/preparation.json')}；現用同版prepared-20261020-065410-390299，來源／正常PE／fixture class仍須逐次核對。Enter負向reader為qualify-native-quantity-Enter-GUI-negatives.py，{enter_state}。EOF負向工具prepare-native-EOF-cleanup-GUI-plan.py、inject-owned-native-GUI-EOF.ps1、qualify-native-EOF-cleanup-GUI.py已備，執行資格見表；須先宣告計畫再操作，只停止精確owned Java，保留預期wrapper1及兩GUI正常退出，與正向通過分開。{eof_reader_note} {link('當版原租期四情境計畫',V/'native-quantity-enter-failure-tooling/natural-expiry-GUI-plan-20261020-042617-965151/plan.json')}及qualify-native-current-natural-expiry-GUI.py已備；未以舊C02或CPU圖移記當版GUI資格。

下次先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。現用報告工具update-master-report-quantity-enter-failure-retirement.py；舊room-form-labels等工具只屬歷史版本。
'''
assert all(Path(p).exists()for p in re.findall(r'\]\(<([^>]+)>\)',body))
out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir(parents=True);old=REPORT.read_bytes();(out/'before.md').write_bytes(old);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes())
assert REPORT.read_bytes()==old and all(sha(R/n)==digest for n,digest in before.items());REPORT.write_bytes((out/'after.md').read_bytes())
record=dict(source_manifest_sha256=source,paused_by_user=paused,library_qualified=lib_ok,active_sessions=w['active_workload_sessions'],before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False)
(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8',newline='\n');print(json.dumps(record,ensure_ascii=False))
