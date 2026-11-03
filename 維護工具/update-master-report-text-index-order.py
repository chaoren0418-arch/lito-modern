"""Compact current-source report with actual tests and immutable prior failures."""
from pathlib import Path
from datetime import datetime,timezone,timedelta
import hashlib,json,re
R=Path(__file__).resolve().parents[1];V=R/'驗證紀錄/current';REPORT=R.parent/'LiTo381-總開發報告.md'
read=lambda p:json.loads(Path(p).read_text('utf-8-sig'));sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest().upper()
link=lambda label,p:'['+label+'](<'+Path(p).as_posix()+'>)'
w=read(R/'工作佇列.json');SOURCE='3E58268A74B9908597221E71BDEC646C1DFEB82A8658444BDF1C24BFABE94628';assert w['current_product_identity']['source_manifest_sha256']==SOURCE
deferred=w.get('current_user_desktop_development',{}).get('test_execution_deferred',False)
mode_note='依使用者最新要求：目前只做開發，桌面操作及測試暫緩；不要自行重啟GUI或測試批次。' if deferred else ''
formal_reference=w.get('current_formal_reference_identity',{})
formal_changed=bool(formal_reference and formal_reference['formal_sha256']!=formal_reference['previous_formal_sha256'])
formal_note=''
formal_next=''
if formal_changed:
    assert sha(formal_reference['formal'])==formal_reference['formal_sha256']
    assert sha(formal_reference['cached_reference'])==formal_reference['cached_reference_sha256']
    assert sha(formal_reference['inspection'])==formal_reference['inspection_sha256']
    assert sha(formal_reference['callee_review'])==formal_reference['callee_review_sha256']
    formal_note='**正式協定目標已變更為 E1498871**，完整SHA-256：`'+formal_reference['formal_sha256']+'`。已固定新參考；原75DA69B0的API／GUI資格及當時Java編譯保留原範圍，**不能當作E149通過**。'+link('新舊契約比對',formal_reference['inspection'])+'；'+link('委派函式與剩餘審查',formal_reference['callee_review'])+'。GameHub交接6／Protocol8個所選方法一致；ClientSession10個僅3一致，選角、場景提交與物品owner已換委派函式，轉移語意仍未證明相容。\n\n'
    formal_next='先追E149的NativePlayerCommit.assign、NativeDeItemApplication.Owner／validateReplacement與sourcePoisonBody$characterSelect，完成fixture／固定JAR期待遷移；新來源／物品生命週期契約未核對前，不套用或宣告現正式相容。'
    ed=w.get('current_E149_recovery_development',{})
    if ed.get('verification_source_reference_migrated'):
        assert ed['compile_complete'] and ed['Java_reference_sha256']==formal_reference['formal_sha256']
        assert sha(ed['contract'])==ed['contract_sha256']
        assert sha(ed['readonly_observer_review'])==ed['readonly_observer_review_sha256']
        formal_note+='G06驗證程式已改固定E149，新增只讀observer直接讀session／角色／來源／帳號、背包owner、原始物品欄位與revision，避開會更新view的items()及通用遊戲時鐘getter；'+link('遷移契約與範圍',ed['contract'])+'；'+link('編譯後只讀位元碼／18存取器核對',ed['readonly_observer_review'])+'。兩次讀取相同僅是觀察穩定，不算原子授權快照。原memory GameHub初始化未涵蓋production item application安裝，**完整新版物品owner生命週期仍待原始資源初始化及實跑**，不能以null owner或省略bootstrap授予資格。\n\n'
        formal_next='G06固定JAR期待與只讀observer已完成E149編譯。接續準備原始production item application資源初始化與隔離fixture，核對真實啟用及完整owner／物品接續；保留原constructor與時限，不以空資料或強制設定owner替代。T鍵／#20之E149契約亦續核對，實際相容性未證明前不套用。'

bootstrap=w.get('current_E149_resource_bootstrap_development',{})
if bootstrap:
    assert sha(bootstrap['qualification'])==bootstrap['qualification_sha256']
    bq=read(bootstrap['qualification'])
    assert bq['formal_target_sha256']==formal_reference['formal_sha256'] and bq['table_count']==19 and bq['rows']==6277
    assert bq['original_statements']==6296 and bq['selected_statement_bytes_independently_reconstructed'] and bq['compiler_exit_code']==0
    assert len(bq['classes'])==2 and all(sha(p)==h for p,h in bq['classes'].items())
    assert sha(bq['SQL'])==bq['SQL_sha256'] and not bq['SQL_executed'] and not bq['database_started'] and not bq['original_factories_invoked']
    formal_note+='原始資源初始化的開發接法已編譯：原'+bootstrap['observed_original_tool_chunk']+' exit0／2 class；'+link('編譯、位元碼順序與原SQL片段核對',bootstrap['qualification'])+'。從既有D4111D8A靜態匯出取出原18查詢所需19表、6277列／6296陳述式（1.52MB），保留原CREATE／INSERT位元組，未選玩家表。'+link('資料清單',bootstrap['static_manifest'])+'。接法使用隔離真實JDBC供應器，仍呼叫原templates.read／definitions.read／item.install與lifetime／cleanup；沒有造snapshot或owner，亦不等於完整Main程序驗收。**尚未啟動DB、匯入SQL、呼叫原factory或整合案例**；現有Rust／13檔候選未變，沿用其編譯結果。\n\n'
    formal_next='接續完成owned MySQL啟動／19表匯入證據與新接法整合fixture的開發：程序PID／出生時間／執行檔／datadir／loopback埠必須實際核對，再走原SQL白名單、rollback／close／freeze與owner安裝。尚未執行任何DB或原factory，不得從編譯或memory fixture移記完整owner資格。'
owned_mysql=w.get('current_E149_owned_mysql_development',{})
if owned_mysql:
    assert sha(owned_mysql['qualification'])==owned_mysql['qualification_sha256']
    oq=read(owned_mysql['qualification'])
    assert oq['formal_target_sha256']==formal_reference['formal_sha256']
    assert oq['AST_parsed_and_compiled_in_memory'] and not oq['launcher_module_imported']
    assert oq['observed_plan_actual_exit_code']==0 and oq['tests_executed']==oq['database_executables_invoked']==0
    assert not oq['SQL_executed'] and not oq['database_started'] and not oq['launcher_runtime_qualified']
    assert sha(oq['launcher'])==oq['launcher_sha256'] and sha(oq['plan'])==oq['plan_sha256']
    assert sha(oq['bundle_manifest'])==oq['bundle_manifest_sha256']
    assert sha(oq['JDBC']['cached'])==oq['JDBC']['cached_sha256']
    formal_note+='隔離資料庫工具已完成來源初稿：私有x64 MySQL 5.5.22共'+str(oq['copied_files'])+'檔／19.13MB，補齊原固定JAR缺少的Connector/J 8.0.33；啟動器包含全新data／tmp、實際PID／FILETIME／映像／命令／listener核對、19表逐表列數與唯讀授權讀回、成功後才產生proof，以及期限／孤兒handle回收。'+link('來源、AST與檔案讀回',owned_mysql['qualification'])+'；'+link('工具範圍與後續用法',R/'維護工具/native-E149-owned-mysql-README.md')+'。原'+oq['observed_plan_original_tool_chunk']+' exit0只建立計畫；**資料庫啟動、SQL匯入、JDBC相容及回收流程均未實跑**，沒有READY／實際import proof。\n\n'
    formal_next='先完成Java bootstrap與G06 fixture之原accept／submit／Future／cleanup整合開發；owned MySQL來源已備妥，尚未實跑。恢復測試後先驗隔離初始化、19表／6277列與唯讀權限、實際JDBC factory及退出回收，再驗完整owner／物品接續；不得從計畫、AST或memory fixture移記資格。'
integrated_fixture=w.get('current_E149_integrated_fixture_development',{})
if integrated_fixture:
    assert sha(integrated_fixture['qualification'])==integrated_fixture['qualification_sha256']
    iq=read(integrated_fixture['qualification'])
    assert iq['formal_target_sha256']==formal_reference['formal_sha256'] and iq['Java_compiler_exit_code']==0
    assert len(iq['Java_classes'])==6 and all(sha(p)==h for p,h in iq['Java_classes'].items())
    assert iq['Rust_compiler_exit_codes']==[0,0] and iq['tests_executed']==iq['classes_instantiated']==0
    assert iq['original_identities_terminal'] and not iq['SQL_executed'] and not iq['fixture_runtime_qualified']
    assert iq['existing_thirteen_file_overlay_unchanged'] and not iq['production_applied']
    formal_note+='原始資源接法已整合為**獨立4檔G06驗證候選**，尚未合併既有13檔覆蓋或正常來源：factory成功後才原生bind／accept／submit，使用Future並走原cleanup，成功退出後才發布result；stage快照釋放hub鎖後才讀inventory。原'+iq['observed_original_tool_chunk']+' exit0：Java6 class，Debug／Release各1個正式案例metadata，沿用原library metadata；**沒有重跑完整library或執行案例**。'+link('編譯、原呼叫鏈與身分核對',integrated_fixture['qualification'])+'。只讀observer只改接收ClientSession，原始呼叫集合核對一致；首輪變數同名編譯失敗與reader定位拒絕均保留。DB／factory／session／完整owner及GUI實際流程仍未驗證。\n\n'
    formal_next='接續開發owned case orchestrator與獨立結果reader，固定JDBC／原JAR／來源／PE／實際proof雜湊，保留程序身分、期限與失敗回收。恢復測試後才驗隔離DB初始化／19表匯入／唯讀授權、真實JDBC原factory、完整owner／物品／wire與退出清理；計畫、型別分析與memory帳號不可授予完整Main或當版資格。'
case_reader=w.get('current_E149_recovery_case_reader_development',{})
if case_reader:
    assert sha(case_reader['qualification'])==case_reader['qualification_sha256']
    rq=read(case_reader['qualification'])
    assert all(sha(p)==h for p,h in rq['source_pins'].items())
    if rq.get('plan'):assert sha(rq['plan'])==rq['plan_sha256']
    assert rq['AST_compiled_modules']==3 and not rq['reader_executed'] and rq['tests_executed']==0
    assert rq['effective_overlay_files']==14 and rq['integration_overlap_files']==3
    if not rq.get('runtime_producer_source_complete'):
        assert rq['runtime_producer_implementation_pending']
        formal_note+='被動reader與案例契約僅來源／AST審查，尚未執行；'+link('開發紀錄',case_reader['qualification'])+'。17項既有物品欄與S7C檢查不代表全部NativeDe內部屬性或完整owner資格。\n\n'
    formal_next='先開發隔離rlib／正式案例PE builder與獨立連結證據核對，再完成owned case orchestrator的實際程序身分、期限及失敗回收；現有被動reader只有來源／AST審查，尚未實跑。恢復測試後才驗真實DB／JDBC原factory、owner與17欄／wire接續、原版反例／候選案例及自然33／31秒，再做最終當版完整library／Release／GUI。'
linked_images=w.get('current_E149_recovery_linked_image_qualifications',{})
if linked_images:
    image_links=[]
    for profile in ['Debug','Release']:
        if profile not in linked_images:continue
        entry=linked_images[profile];assert sha(entry['qualification'])==entry['qualification_sha256']
        lq=read(entry['qualification']);assert lq['profile']==profile and lq['status']=='qualified_linked_isolated_E149_recovery_controller'
        assert lq['compiler_exit_codes']==[0,0] and lq['recorded_original_compilers_terminal']
        assert lq['x64_PE32_plus_executable_verified'] and lq['rlib_archive_verified'] and lq['full_source_and_overlay_pins_verified']
        assert not lq['outputs_executed'] and lq['tests_executed']==0 and not lq['candidate_applied']
        assert sha(lq['product'])==lq['product_sha256'] and Path(lq['library']).stat().st_size==lq['library_bytes']
        assert sha(lq['compile_cohort'])==lq['compile_cohort_sha256'] and lq['compiler_control_versions_archived']
        image_links.append(link(profile+'隔離連結資格',entry['qualification']))
    formal_note+='**隔離連結已完成 '+('Debug／Release兩版' if set(linked_images)=={'Debug','Release'} else '／'.join(linked_images))+'**：每版實際連結候選rlib與正式案例x64 PE，原Debug 30960／351c1e、Release 38145／4b9bf3均actual exit0，各兩個compiler exit0；362模組完整graph、1049正常輸入、14檔覆蓋、依賴／原生搜尋路徑、映像與記錄身分終止皆核對。'+'；'.join(image_links)+'。Release保留z／thin／16／strip；工具版本已另存不可變證據。兩版輸出均未執行，正常兩EXE未變、候選未套用；**完整library、功能／owner、Release回歸及GUI仍待實跑**。初次數量欄誤讀與active觀測跨階段退件均保留，原建置未重啟；目前活動批次已清空、來源凍結解除。\n\n'
    formal_next='接續開發owned case orchestrator的實際程序身分、期限與失敗回收，並重新產生包含producer及兩版映像資格的計畫。被動reader尚未實跑；使用者恢復測試後才驗真實DB／JDBC原factory、owner與17欄／wire接續、原版反例／候選案例及自然33／31秒，再做最終當版完整library／Release／GUI。'
producer_development=w.get('current_E149_recovery_execution_development',{})
if producer_development:
    assert sha(producer_development['qualification'])==producer_development['qualification_sha256']
    pq=read(producer_development['qualification'])
    assert pq['runtime_producer_source_complete'] and pq['total_AST_compiled_modules'] in [5,6,7,8,9]
    assert pq['process_job_objects_created']==pq['workload_processes_started']==pq['tests_executed']==0
    assert not pq['runtime_producer_runtime_qualified'] and not pq['reader_runtime_qualified']
    assert all(sha(p)==h for p,h in pq['source_pins'].items())
    prepared=read(producer_development['prepared_case_plan'])
    assert sha(producer_development['prepared_case_plan'])==producer_development['prepared_case_plan_sha256']
    assert prepared['linked_image_qualified'] and not prepared['runtime_producer_implementation_pending'] and prepared['test_execution_deferred_at_preparation']
    # Replace intermediate milestone narration with the final current state;
    # immutable preceding qualifications remain reachable from this record.
    formal_note='**現正式目標 E1498871**（完整SHA：`'+formal_reference['formal_sha256']+'`）。所選GameHub6／Protocol8個方法一致，ClientSession10個僅3一致，選角／場景提交／物品owner已換委派；'+link('契約差異',formal_reference['inspection'])+'。原75DA69B0的API／GUI資格僅保留歷史範圍，**E149 runtime／完整owner／Main仍未通過**。\n\n'
    formal_note+='G06候選仍未套用：原13檔加整合4檔有3重疊，為14檔有效覆蓋。原factory接法、只讀observer與真實接續LOBBY確認點已編譯為Java6 class；'+link('Java／兩版正式案例metadata與觀測契約',integrated_fixture['qualification'])+'。19靜態表／6277列／6296陳述式、私有x64 MySQL5.5.22與JDBC已備妥，**DB／SQL／factory未執行**；接法使用原memory帳號與隔離靜態JDBC，非完整Main。Debug／Release各已實際連結rlib與正式案例x64 PE，各兩命令exit0，完整362模組／1049正常輸入／14檔／依賴與原程序退出皆核對；'+'；'.join(image_links)+'。兩版輸出未啟動，正常兩EXE未變。\n\n'
    formal_note+='自動執行與獨立reader已有初稿，5個Python僅AST／來源審查：暫停建立→核對PID／FILETIME／映像→加入私有Job並保存身分→放行；真實DB匯入後才執行原factory與單一精確controller，最後核對原handle及Job零程序；失敗保留、無自動重試。'+link('來源審查與前階段索引',producer_development['qualification'])+'；'+link('已固定映像／工具版本的新準備計畫',producer_development['prepared_case_plan'])+'；'+link('操作格式與限制',R/'維護工具/native-E149-recovery-case-contract.md')+'。**Job、程序、reader、案例均未實跑**，source-ready旗標不是測試授權；使用者暫緩仍true。reader現覆蓋17個既有物品觀察欄及S7C完整資料，仍缺全部NativeDe元素／卡片／魔石內部欄位與逐session wire ID；穩定取樣不算原子授權。\n\n'
    formal_next='先核對SharedClientResources至原物品factory的初始化閉包，再補NativeDe元素／卡片／魔石唯讀觀測與完整owner／物品核對。Job／producer／reader仍須實跑；使用者恢復測試後先驗隔離Job／DB／SQL／JDBC與回收，再跑原版6反例／候選14案、真實33／31秒、T鍵／#20，核對後套用，最後重做新當版完整library兩版／Release回歸與完整GUI。'
extended_items=w.get('current_E149_item_attribute_observer_development',{})
if extended_items:
    assert sha(extended_items['qualification'])==extended_items['qualification_sha256']
    eq=read(extended_items['qualification'])
    assert eq['Java_compiler_exit_code']==0 and len(eq['Java_classes'])==6
    assert all(sha(p)==h for p,h in eq['Java_classes'].items())
    assert eq['Rust_files_changed']==eq['Rust_compilers_invoked']==eq['tests_executed']==0
    assert eq['permanent_fields']==8 and eq['diagnostic_fields']==14 and eq['embedded_duration_rows']==60303
    assert not eq['integrated_into_active_fixture'] and not eq['integrated_into_producer_reader']
    assert not eq['original_initializer_invoked'] and not eq['full_attribute_matrix_qualified']
    formal_note+='新Java-only候選已明確保留原共用初始化：catalog／JAR內60303筆duration已靜態讀回；新增native_attributes sidecar，含8項永久欄（元素／鑲卡／魔石／覺醒）、cached ability及14項診斷，原17欄不變。原'+eq['observed_original_tool_chunk']+' exit0／6 class；'+link('原始初始化、唯讀bytecode與編譯核對',extended_items['qualification'])+'；'+link('欄位範圍及後續整合',R/'維護工具/native-E149-extended-item-observation.md')+'。Timestamp／EnumMap先複製，atomic.get與18個原record accessor路徑已核對；首輪Enum pattern型別錯誤exit1保留。純資料比較器僅AST；全零／空值不能授予完整屬性矩陣資格。**新候選未接入active fixture／producer／reader或客戶端映像context**，Rust未改／未重編，原兩版連結資格保留原範圍；初始化／案例未實跑。\n\n'
    formal_next='先把新Java-only初始化／屬性候選接入reader與producer，分別綁定相同Rust位元組的原客戶端資格及新Java class資格，重新產生計畫；不要直接把舊14檔整合context改標為新fixture。補正常遊戲流程產生的非預設元素／鑲卡／魔石／覺醒與拒絕／期限矩陣。使用者恢復測試後才驗Job／真實DB／JDBC／退出回收、原版6反例／候選14案、真實33／31秒，及最終新當版完整library兩版／Release／完整GUI。'
fixture_binding=w.get('current_E149_fixture_binding',{})
if fixture_binding:
    assert sha(fixture_binding['qualification'])==fixture_binding['qualification_sha256']
    fb=read(fixture_binding['qualification'])
    assert fb['schema'] in ['E149-fixture-source-binding-v1','E149-fixture-source-binding-v2'] and fb['status']=='source_binding_only_no_runtime'
    assert pq['total_AST_compiled_modules'] in [6,7,8,9] and pq['fixture_source_binding_sha256']==fixture_binding['qualification_sha256']
    selected_fixture=read(fb['fixture_qualification']);assert sha(fb['fixture_qualification'])==fb['fixture_qualification_sha256']
    allowed_fixtures={extended_items['qualification'],w.get('current_E149_owner_guard_development',{}).get('qualification'),w.get('current_E149_mutation_denial_development',{}).get('qualification')}
    assert fb['fixture_qualification'] in allowed_fixtures and selected_fixture['Java_compiler_exit_code']==0
    assert all(b['Rust_source_files_unchanged']==11 and len(b['changed_Java_context'])==2 for b in fb['client_bindings'].values())
    assert not fb['client_images_relinked'] and not fb['Java_runtime_compatibility_qualified']
    assert all(sha(p)==h for p,h in fb['Java_source_pins'].items())
    assert set(producer_development['prepared_case_plans'])=={'Debug','Release'}
    for profile,item in producer_development['prepared_case_plans'].items():
        assert sha(item['path'])==item['sha256'];plan=read(item['path'])
        assert plan['schema'] in ['E149-recovery-case-plan-v2','E149-recovery-case-plan-v3'] and plan['profile']==profile
        assert plan['fixture_source_binding_sha256']==fixture_binding['qualification_sha256'] and plan['test_execution_deferred_at_preparation']
    formal_note='**現正式目標 E1498871**（完整SHA：`'+formal_reference['formal_sha256']+'`）。ClientSession所選10個方法僅3個與舊75一致，選角／場景提交／物品owner已換委派；'+link('契約差異',formal_reference['inspection'])+'。舊75的API／GUI資格保留歷史範圍，**E149 runtime／完整owner／Main未通過**。\n\n'
    formal_note+='G06候選為14檔有效覆蓋，未套用正常來源。Debug／Release已實際連結隔離rlib與正式案例x64 PE，各兩命令exit0；'+'；'.join(image_links)+'。新Java-only fixture已接入v2準備計畫：'+link('分開綁定Java編譯與原Rust映像',fixture_binding['qualification'])+'。11個Rust輸入逐byte相同，只有兩個Java檔不同；原連結map／cohort／資格原樣保留，未重編Rust、未重標功能結果。所選Java原'+selected_fixture['observed_original_tool_chunk']+' exit0／6 class；'+link('編譯與bytecode範圍',fb['fixture_qualification'])+'。19靜態表／6277列、私有MySQL／JDBC已備妥，**DB／SQL／initializer／factory皆未執行**，非完整Main。正常1049輸入及兩EXE未變。\n\n'
    formal_note+='producer／獨立reader已整合新版契約，'+str(pq['total_AST_compiled_modules'])+'個Python僅AST／來源審查；'+link('本輪開發核對與前階段索引',producer_development['qualification'])+'；'+link('Debug準備計畫',producer_development['prepared_case_plans']['Debug']['path'])+'；'+link('Release準備計畫',producer_development['prepared_case_plans']['Release']['path'])+'；'+link('接續操作契約',R/'維護工具/native-E149-recovery-case-contract.md')+'。執行前先檢查使用者暫緩旗標，再以私有Job／原PID及FILETIME、原handle與零程序屏障收尾；**Job、producer、reader與案例尚未實跑**。reader原17欄／S7C之外，新增8項永久屬性、cached ability及14項診斷；同ID未到期stage才比永久欄，時間／generation不強作不變值，非預設覆蓋與完整矩陣分開，完整矩陣仍false。'+link('屬性範圍與限制',R/'維護工具/native-E149-extended-item-observation.md')+'。穩定取樣不是原子授權，逐session wire歸屬、dark payload／timer lease及完整owner仍待。\n\n'
    formal_next='接續開發正常玩法產生非預設元素／鑲卡／魔石／覺醒的效果、拒絕／期限及owner矩陣，不以反射造狀態。恢復測試後才驗Job／真實DB／JDBC／退出回收、原版6反例／候選14案與既有案、自然33／31秒、T鍵／#20；獨立核對後套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
owner_identity=w.get('current_E149_owner_identity_development',{})
if owner_identity:
    assert sha(owner_identity['qualification'])==owner_identity['qualification_sha256']
    owner_review=read(owner_identity['qualification']);assert owner_review['owner_identity_comparison_corrected_from_bare_to_source_qualified']
    assert not owner_review['owner_identity_helper_executed'] and owner_review['tests_executed']==0
    assert sha(owner_identity['owner_identity_contract'])==owner_identity['owner_identity_contract_sha256']
    owner_contract=read(owner_identity['owner_identity_contract'])
    assert owner_contract['separator_code_point']==31 and not owner_contract['case_folding_at_owner_construction']
    formal_note+='本輪修正reader的owner身份誤比：正式owner.source／account存「來源＋U+001F＋角色／帳號」，先前直接比裸來源／帳號會誤拒。已依9個原方法excerpt與concat位元碼，改用Java trim範圍並保留大小寫；'+link('原始識別契約',owner_identity['owner_identity_contract'])+'。Debug／Release準備計畫已綁定修正版本；**helper／reader尚未執行**，舊計畫原樣保留，本輪Java／Rust未改／未重編。2554個class的setter／fieldref靜態追查目前只明確找到setItem→setGreater；非預設元素／卡片／魔石／覺醒的正常GAME產生路徑未證明，不排除通用反射或外部路由。原StateCopy拒絕活動timer／pending到期，應另驗拒絕條件，不能以造值或全零案例授予完整矩陣。\n\n'
    formal_next='接續完成可達G06身份／接續／退役與拒絕矩陣開發；非預設屬性的正常GAME產生前提繼續追查，不以反射造狀態。恢復測試後先驗Job／真實DB／JDBC與回收、原版6反例／候選14案及既有案、自然33／31秒、T鍵／#20；獨立核對後套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
owner_guard=w.get('current_E149_owner_guard_development',{})
if owner_guard and not w.get('current_E149_owner_guard_integration_development'):
    assert sha(owner_guard['qualification'])==owner_guard['qualification_sha256']
    guard=read(owner_guard['qualification'])
    assert guard['Java_compiler_exit_code']==0 and len(guard['Java_classes'])==6
    assert guard['Rust_compilers_invoked']==guard['Rust_files_changed']==guard['tests_executed']==0
    assert not guard['original_owner_guards_invoked'] and not guard['integrated_into_active_fixture'] and not guard['integrated_into_producer_reader']
    assert guard['same_live_owner_positive_control_before_negative'] and guard['retired_checks_before_hub_shutdown']
    assert all(sha(p)==h for p,h in guard['Java_classes'].items()) and all(sha(p)==h for p,h in guard['source_pins'].items())
    formal_note+='新增獨立owner guard候選，原'+guard['observed_original_tool_chunk']+' actual exit0／6 class；'+link('編譯、原guard路徑與來源核對',owner_guard['qualification'])+'；'+link('有限檢查與後續串接',R/'維護工具/native-E149-owner-guard-development.md')+'。接續ACK前先驗有效owner接受自身inventory，另觀察另一帳號實際inventory／session的拒絕；四個選角GAME原Future退出後、hub shutdown前才驗stale拒絕。原observer18個accessor不變，harness新增原requireActive／matches檢查；不造owner或改active／物品／時鐘，guard-only不等於完整mutation矩陣。**guard尚未呼叫，新候選未接入active fixture／producer／reader及計畫**；本輪未改／重編Rust，正常來源／EXE不變。JDK唯讀路徑的核對工具失敗已封存並修正，沿用原編譯結果。\n\n'
    formal_next='先把新owner guard候選接入producer／reader，固定guard trace並逐角色核對正向、外來inventory與退役證據，再分別綁定原Rust byte與新Java編譯資格、更新兩版計畫；不改寫目前舊計畫。正常GAME非預設屬性、完整mutation／timer拒絕矩陣仍待。恢復測試後才驗Job／真實DB／JDBC及退出回收、原6反例／候選14案與既有案、自然33／31秒、T鍵／#20；獨立核對後套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
guard_integration=w.get('current_E149_owner_guard_integration_development',{})
if guard_integration:
    assert sha(guard_integration['qualification'])==guard_integration['qualification_sha256']
    gi=read(guard_integration['qualification'])
    assert gi['total_AST_compiled_modules'] in [7,8,9] and gi['owner_guard_trace_reader_integrated']
    assert not gi['owner_guard_reader_functions_executed'] and not gi['original_owner_guards_invoked'] and gi['tests_executed']==0
    assert gi['guard_fixture_qualification']==owner_guard['qualification'] and gi['fixture_qualification']==fb['fixture_qualification']
    assert fb['owner_guard_probe_kind']=='G06_E149_owner_guard_v1'
    formal_note+='owner guard已接入未來案例的producer／reader與兩版新計畫；'+link('guard trace讀回契約',R/'維護工具/native-E149-owner-guard-readback.md')+'。reader核對兩筆接續正向控制、一／兩筆外來inventory拒絕與四筆實際選角owner退役，連回原trace／ACK及owner／session／inventory ID；外來inventory須等於另一admission保留的原參考，退役須有最後disconnected trace與精確stale例外。原Future退出後／fixture close前的順序另綁Java流程，非額外lifetime原子觀測。**原guard、純解析函式、producer／reader均未執行**；guard-only不授予物品mutation／完整來源帳號分支／timer矩陣。沿用原580956的6class，本輪Java／Rust未改／未重編；舊候選資格與計畫封存。\n\n'
    formal_next='接續補可獨立開發的G06失敗／拒絕與完整物品前提，非預設屬性正常GAME路徑仍待。恢復測試後先驗原guard取樣與真實行為、Job／隔離DB／JDBC與退出回收，再跑原6反例／候選14案及既有案、自然33／31秒、T鍵／#20；獨立核對後套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
suite_development=w.get('current_E149_recovery_suite_development',{})
if suite_development:
    assert sha(suite_development['qualification'])==suite_development['qualification_sha256']
    sq=read(suite_development['qualification']);assert sq['total_AST_compiled_modules'] in [8,9] and sq['suite_orchestrator_source_complete']
    assert not sq['runtime_suite_function_executed'] and not sq['suite_runtime_qualified'] and sq['tests_executed']==0
    assert sha(suite_development['prepared_suite_plan'])==suite_development['prepared_suite_plan_sha256']
    sp=read(suite_development['prepared_suite_plan']);assert len(sp['members'])==16 and sp['tests_executed']==0
    assert all(sha(row['plan'])==row['plan_sha256'] for row in sp['members'])
    formal_note+='新增八個精確恢復案×Debug／Release序列工具來源；'+link('16案準備索引',suite_development['prepared_suite_plan'])+'；'+link('順序、停止及範圍',R/'維護工具/native-E149-recovery-suite.md')+'。每案先沿原producer私有Job執行／回收，再由獨立reader核對；第一個失敗、來源變更或使用者重新暫緩即停止，不重試、不跳過，保留已核對前綴與原始輸出。沒有額外巢狀Job。**僅AST與純計畫準備，序列工具未執行**；16案不能代替完整library、原版反例／候選元件案、GUI或全部G06。\n\n'
    formal_next='接續補原版6反例／候選14元件案的可追溯執行接續，以及完整物品／非預設狀態前提。恢復測試後才驗實際序列停止與回收、原guard時序、Job／隔離DB／JDBC、原6反例／候選14案與八分支兩版、自然33／31秒、T鍵／#20；獨立核對後套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
component_matrix=w.get('current_E149_component_matrix_development',{})
if component_matrix:
    assert sha(component_matrix['qualification'])==component_matrix['qualification_sha256']
    cq=read(component_matrix['qualification'])
    assert cq['baseline_metadata_compiler_exit_codes']==[0,0] and cq['compiler_identities_terminal']
    assert cq['full_module_declarations']==362 and cq['normal_compiled_inputs']==1049
    assert cq['baseline_selected_cases']==23 and cq['candidate_selected_cases']==28
    assert cq['new_candidate_components']==11 and cq['new_formal_cases_separate']==3
    assert cq['tests_executed']==0 and not cq['baseline_counterexamples_observed'] and not cq['candidate_tests_passed']
    assert all(sha(p)==h for p,h in cq['source_pins'].items())
    assert sha(cq['plan'])==cq['plan_sha256']
    for path,row in cq['metadata_products'].items():assert sha(path)==row['sha256']
    formal_note+='原版／候選元件對照已完成來源隔離：每版原版23案＝17原有控制＋6指定預期反例，候選28案＝17原有控制＋11新元件；**11元件＋另3新正式才是提案14案**。原版僅native_login Runtime換回正常3E，其餘候選相同；完整362模組與配對QA已核對。原'+cq['observed_original_tool_chunk']+' actual exit0，原版Debug／Release各一個metadata compiler exit0，原程序終止核對；'+link('來源、編譯與案例清單核對',component_matrix['qualification'])+'；'+link('原版／候選計畫',cq['plan'])+'；'+link('範圍與接續',R/'維護工具/native-E149-component-matrix.md')+'。**未連結元件PE、未執行--list或案例，六反例尚未重現**；空test rmeta只是型別檢查marker，候選原全圖metadata保留原範圍。首輪工具誤比可變registry狀態已封存修正，未啟動compiler。\n\n'
    formal_next='接續完成原版／候選四份元件PE的獨立連結資格及精確執行／斷言reader；17原有控制須先過，6反例需指定assert panic，編譯錯／crash／其他panic不算重現。候選28元件案仍須實跑，另驗八分支兩版正式suite、實際guard／Job／隔離DB／JDBC回收與自然33／31秒；全部依使用者恢復測試後執行。正常GAME非預設屬性／完整物品前提仍待，獨立核對後才套用，再做新當版完整library Debug／Release、Release回歸與完整GUI。'
component_readback=w.get('current_E149_component_execution_development',{})
if component_readback:
    assert sha(component_readback['qualification'])==component_readback['qualification_sha256']
    cr=read(component_readback['qualification']);assert cr['AST_compiled_modules']==6
    assert cr['component_result_reader_source_complete'] and cr['component_image_qualifier_source_complete']
    assert cr['component_producer_source_complete'] and cr['diagnostic_preservation_source_complete']
    assert not cr['component_producer_executed'] and not cr['diagnostic_store_constructed'] and not cr['diagnostic_restoration_executed']
    assert not cr['parser_functions_executed'] and not cr['matrix_reader_executed'] and cr['tests_executed']==0
    assert all(sha(p)==h for p,h in cr['source_pins'].items())
    # Keep the current stop point compact. The immutable source reviews link
    # their predecessors; detailed intermediate narration stays in those files.
    formal_note=formal_note.split('\n\n',1)[0]+'\n\n'
    formal_note+='G06候選仍未套用正常來源：13檔原候選加fixture整合，去重為14檔；11個Rust輸入與原兩版隔離映像相同，'+ '；'.join(image_links)+'。所選Java原'+selected_fixture['observed_original_tool_chunk']+' exit0／6class，原factory接法、屬性唯讀觀測、owner來源限定識別與guard契約已接入'+str(pq['total_AST_compiled_modules'])+'個Python工具來源；'+link('正式案例工具及前階段索引',producer_development['qualification'])+'；'+link('八分支×兩版16案準備索引',suite_development['prepared_suite_plan'])+'。**Job／DB／SQL／initializer／guard／案例／GUI皆未執行**，僅編譯／AST／來源核對；舊75的runtime資格不移記E149。正常1049輸入及兩EXE未變。\n\n'
    formal_note+='元件對照：每版原版23案＝17原有控制＋6指定預期反例；候選28案＝17原有控制＋11新元件，另3新正式案才合計提案14案。原版兩版metadata各exit0，完整362模組與配對QA已核對；'+link('原版metadata／來源資格',component_matrix['qualification'])+'；'+link('元件計畫',cq['plan'])+'。元件producer、診斷JSON保護、libtest／四PE資格及獨立矩陣reader來源已完成，6模組僅AST；'+link('最新來源審查',component_readback['qualification'])+'；'+link('精確結果與資料回復契約',R/'維護工具/native-E149-component-readback-contract.md')+'。每案設計為先保存原handle／Job、封存新輸出，再逐檔回復原bytes／mtime；部分回復失敗保留資料並阻止新批次，reader另核對archive／delta／receipt及最終目錄。**producer／診斷回復／Job均未執行，四PE未連結、--list／102精確案／6原版反例尚未實跑**。反例須唯一同名panic、指定原檔／assert行號與訊息；編譯錯／crash／其他panic不算重現。非預設屬性正常GAME路徑、完整mutation／owner／Main仍待，所有完整資格維持未完成。\n\n'
    formal_next='接續四份元件PE連結與獨立資格。使用者恢復測試後才跑4次inventory及兩版原版23／候選28精確案；原17控制先過，6原版指定反例須真正重現。再驗正式八分支兩版16案、診斷回復／guard／隔離DB／JDBC回收與自然33／31秒、T鍵／#20。全部獨立核對後才套用，再做新當版完整library Debug／Release、Release回歸及完整GUI。'
component_images=w.get('current_E149_component_linked_images',{})
if component_images:
    assert sha(component_images['qualification'])==component_images['qualification_sha256']
    ci=read(component_images['qualification'])
    assert ci['schema']=='E149-component-images-qualified-v1' and ci['status']=='qualified_linked_selected_component_matrix_images'
    assert ci['source_manifest_sha256']==SOURCE and ci['formal_target_sha256']==w['current_formal_reference_identity']['formal_sha256']
    assert ci['original_compiler_exit_codes']==[0,0,0,0] and ci['original_compilers_terminal'] and ci['compiler_controls_archived']
    assert ci['tests_executed']==0 and not ci['PEs_executed'] and not ci['test_harness_inventory_executed']
    assert set(ci['images'])=={p+'-'+v for p in ['Debug','Release'] for v in ['baseline','candidate']}
    for row in ci['images'].values():assert sha(row['product'])==row['sha256'] and Path(row['product']).stat().st_size==row['bytes']
    assert sha(ci['compile_cohort'])==ci['compile_cohort_sha256'] and sha(component_images['raw_archive'])==component_images['raw_archive_sha256']
    old='**producer／診斷回復／Job均未執行，四PE未連結、--list／102精確案／6原版反例尚未實跑**。'
    assert formal_note.count(old)==1
    new='**Debug／Release×原版／候選四PE已實際連結，原'+component_images['observed_original_tool_chunk']+' actual exit0／四個rustc exit0**；'+link('四PE獨立連結資格',component_images['qualification'])+'；'+link('原命令／日誌與來源封存',component_images['raw_archive'])+'。x64 PE32+／SHA、原compiler終止與控制檔副本已核對。**四PE皆未執行，--list／102精確案／6原版反例／producer／診斷回復／Job尚未實跑**。'
    formal_note=formal_note.replace(old,new).replace('來源已完成，6模組僅AST；','來源已完成，前次6模組AST核對保留；')
    formal_next='四份元件PE已有獨立連結資格，不重編已核對來源。待使用者恢復測試後跑4次inventory與兩版原版23／候選28精確案；原17控制先過，6指定反例須真正重現，診斷資料須保存及回復。再驗正式八分支兩版16案、guard／隔離DB／JDBC回收、自然33／31秒與T鍵／#20。全部獨立核對後才套用，再做新當版完整library Debug／Release、Release回歸及完整GUI。'
mutation_denials=w.get('current_E149_mutation_denial_development',{})
if mutation_denials:
    assert sha(mutation_denials['qualification'])==mutation_denials['qualification_sha256']
    md=read(mutation_denials['qualification'])
    assert md['status']=='retired_bound_setter_probes_Java_compilation_and_reader_source_only'
    assert md['Java_compiler_exit_code']==0 and len(md['Java_classes'])==6 and md['original_compiler_identities_terminal']
    assert md['planned_owner_targets']==4 and md['planned_retired_setter_attempts']==32 and md['AST_compiled_tools']==4
    assert md['pure_receipt_reader_source_complete'] and not md['pure_receipt_reader_executed'] and not md['original_mutation_setters_invoked']
    assert md['tests_executed']==0 and not md['integrated_into_active_fixture'] and not md['integrated_into_producer_reader']
    mutation_integration=w.get('current_E149_mutation_denial_integration_development',{})
    assert md['prior_fixture_binding']==(fb['predecessor_fixture_binding'] if mutation_integration else w['current_E149_fixture_binding'])
    assert all(sha(p)==h for p,h in md['source_pins'].items()) and all(sha(p)==h for p,h in md['Java_classes'].items())
    assert sha(mutation_denials['raw_archive'])==mutation_denials['raw_archive_sha256']
    formal_note+='退役物品修改拒絕：保留四個實際owner各一個仍附著的BoundItem，原Future及退役guard完成、hub close前，每案規劃8個setter×4物件＝32次不同值呼叫；只接受精確stale-owner例外及前後觀測一致，不把detached／staged／參數錯誤當退役拒絕。原'+md['observed_original_tool_chunk']+' actual exit0／Java六class，兩類javap已核對，原observer18個record accessor不變；'+link('候選編譯與範圍',mutation_denials['qualification'])+'。'
    if mutation_integration:
        assert sha(mutation_integration['qualification'])==mutation_integration['qualification_sha256']
        mi=read(mutation_integration['qualification']);assert mi['total_AST_compiled_modules']==9 and mi['retired_item_mutation_reader_integrated']
        assert not mi['retired_item_mutation_reader_functions_executed'] and not mi['original_mutation_setters_invoked'] and mi['tests_executed']==0
        assert mi['fixture_source_binding_sha256']==fixture_binding['qualification_sha256'] and fb['retired_item_mutation_attempts']==32
        formal_note+='已接入fixture binding v2、case plan／execution v3、suite plan v2與正式producer／獨立reader；九模組僅AST，'+link('來源整合與16份新計畫',mutation_integration['qualification'])+'。producer保存完整mutation JSONL／SHA，reader逐筆對回原trace、owner／session／actor、不同值與精確拒絕。**每案32次、16案共512次全部未執行，純reader／producer／suite均未呼叫**；舊來源與計畫封存。'
        formal_next='新Java binding、mutation reader與16案計畫已準備；繼續補完整擴充屬性、來源帳號與timer前提。'+formal_next
    else:
        formal_note+='**32次皆未執行，純reader未呼叫；新Java／reader未接入canonical fixture／正式producer／16案計畫**。'
        formal_next='開發先接入新退役物品候選的Java binding、純reader與正式外層核對，保持原Rust映像來源綁定；每案32次只屬計畫，待恢復測試後驗證。'+formal_next
    formal_note+='原Rust及四元件PE不變；擴充屬性、source/account分支、timer／Main與完整owner仍未完成。\n\n'
slow29=w.get('current_native_slow29_development',{})
if slow29:
    assert sha(slow29['qualification'])==slow29['qualification_sha256']
    sl=read(slow29['qualification']);assert sl['status']=='unselected_slow29_request_menu_and_finite_frames_metadata_only'
    assert sl['successful_compiler_exit_codes']==[0]*8 and sl['reused_original_successful_compilers']==2 and sl['new_compilers_started']==6
    assert sl['original_failed_compiler_exit_code']==1 and sl['original_compiler_identities_terminal']
    assert sl['tests_executed']==0 and not sl['PEs_executed'] and not sl['PEs_linked'] and not sl['registered_overlay_selected'] and not sl['production_applied']
    assert sl['current_fixture_binding_unchanged']==w['current_E149_fixture_binding']
    assert sl['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development']
    assert all(sha(p)==h for p,h in sl['source_pins'].items()) and all(sha(row['path'])==row['sha256'] for row in sl['metadata_products'].values())
    formal_note+='離線玩法候選：#29緩速術補C14 03/04、可見目標目前座標／八格／500ms、學習清單及752原16個有限影格；MP／材料／機率／64秒與haste抵銷仍由E149原server決定。保留原#20六案，另寫六個#29案。兩版完整模組圖只換3個Runtime，library／library-test／#20／#29整合共8項metadata退出0；原'+sl['observed_original_tool_chunk']+' actual exit0，沿用2項Debug、接續6項；原整合退出1與路徑準備錯誤另封存，未刪改原結果。'+link('候選／編譯及失敗接續資格',slow29['qualification'])+'；'+link('玩法／752及#79後續',R/'維護工具/native-E149-slow29-development.md')+'。**未選入現行13檔registry、未套用，案例／SPR播放／實際施法／GUI均未跑**；原G06映像及16案計畫不移記新候選。#79原3935含子特效1356與attr109(0 -1)，原繪製語意及接入仍待；正常38個請求識別不變。\n\n'
    formal_next='離線續補#79子特效／控制欄位與剩餘玩法及完整物品前提；#29來源候選保留，恢復測試後需原資源／實際E149機率、材料、低MP、haste抵銷及GUI，再獨立合併到現行候選。'+formal_next
advance79=w.get('current_native_advance79_development',{})
if advance79:
    assert sha(advance79['qualification'])==advance79['qualification_sha256']
    av=read(advance79['qualification']);assert av['status']=='unselected_advance79_request_menu_selector_audio_metadata_only'
    assert av['successful_compiler_exit_codes']==[0]*10 and av['original_compiler_identities_terminal']
    assert av['tests_executed']==0 and not av['PEs_executed'] and not av['PEs_linked'] and not av['registered_overlay_selected'] and not av['production_applied']
    assert av['resolved_module_path_attributes']==362 and len(av['runtime_override_files'])==4
    assert av['current_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and av['current_formal_suite_unchanged']==w['current_E149_recovery_suite_development']
    assert all(sha(p)==h for p,h in av['source_pins'].items()) and all(sha(row['path'])==row['sha256'] for row in av['metadata_products'].values())
    assert '離線玩法候選：' in formal_note
    formal_note=formal_note[:formal_note.rindex('離線玩法候選：')]
    formal_note+='離線玩法候選：保留#20／#29，新增#79靈魂昇華之C14 09/06、自用或可見目標、原不對稱畫面菱形、已學選單及3935原19有限影格；MP／最大HPMP／1200秒仍由E149回覆決定。**更正前報告：Effect(1356)是原<模式0可重疊音效，不是子sprite**；固定原OEP位元碼與1356 WAV metadata只讀核對，沿既有PreparedCue的一次性／退役路徑。'+link('契約、音效更正及待驗範圍',R/'維護工具/native-E149-advance79-development.md')+'。\n\n'
    formal_note+='新六案加保留#20六案／#29六案均未跑；完整362模組只換4Runtime，兩版library／library-test／#20／#29／#79共10個metadata命令退出0，原'+av['observed_original_tool_chunk']+' actual exit0；'+link('新候選與編譯資格',advance79['qualification'])+'。'+link('前#29八項metadata及原失敗接續',slow29['qualification'])+'保留。**七檔候選未選入現行十三檔registry、未套用；無PE連結／執行、SPR播放、施法、音效或GUI資格**。正常38個請求識別、原G06映像／綁定與16案計畫不變。\n\n'
    old_next='離線續補#79子特效／控制欄位與剩餘玩法及完整物品前提；#29來源候選保留，恢復測試後需原資源／實際E149機率、材料、低MP、haste抵銷及GUI，再獨立合併到現行候選。'
    assert formal_next.startswith(old_next)
    formal_next='繼續剩餘玩法、完整物品擴充屬性／來源帳號／timer前提；#20／#29／#79七檔候選已準備，恢復測試後先驗原資源／實際E149自用與對人、機率／材料／低MP／最大值同步／期限與GUI，再獨立合併到現行候選。'+formal_next[len(old_next):]
morph=w.get('current_E149_morph_denial_development',{})
if morph:
    assert sha(morph['qualification'])==morph['qualification_sha256']
    mo=read(morph['qualification']);assert mo['status']=='retired_morph_probes_Java_compilation_and_reader_source_only'
    assert mo['Java_compiler_exit_code']==0 and len(mo['Java_classes'])==6 and mo['original_compiler_identities_terminal']
    assert mo['planned_additional_morph_attempts']==8 and mo['planned_aggregate_retired_setter_attempts']==40 and mo['canonical_planned_setter_attempts']==32
    assert mo['tests_executed']==0 and not mo['original_morph_setters_invoked'] and not mo['pure_receipt_reader_executed'] and not mo['canonical_fixture_or_plans_selected']
    assert mo['original_fixture_binding_unchanged']==w['current_E149_fixture_binding'] and mo['original_formal_suite_unchanged']==w['current_E149_recovery_suite_development']
    assert all(sha(p)==h for p,h in mo['source_pins'].items()) and all(sha(p)==h for p,h in mo['Java_classes'].items())
    formal_note+='G06模板／ID退役候選：保留原八setter／32筆trace，另從每個原inventory保留兩個不同模板的真實附著BoundItem，新增setItem／setItemId共規劃8次，擴充候選合計40次。只接受精確stale-owner且前後完整觀測不變；意外接受／其他RuntimeException也先保存實際receipt再失敗，不把參數驗證當owner拒絕。原'+mo['observed_original_tool_chunk']+' actual exit0／Java六class，兩類javap exit0；harness只新增兩個原入口，observer18個accessor及原八setter片段不變，四個Python工具僅AST。'+link('候選／編譯與獨立讀回來源',morph['qualification'])+'；'+link('範圍與整合停點',R/'維護工具/native-E149-morph-denial-development.md')+'。**新八筆／純reader／guard均未執行，未選入canonical binding／producer／reader／16案計畫，現行每案仍32次**；四個雙模板inventory是否齊全尚未實驗，完整擴充屬性、source/account、timer／Main與G06未完成。\n\n'
    formal_next='優先把模板／ID新八筆拒絕候選接入新fixture binding、producer／outer reader及計畫；保留原32與新8兩條trace及前後診斷，不能把未選入的40次移記現行16案。'+formal_next
typecheck=w.get('current_development_typecheck_qualification',{})
typecheck_note=''
if typecheck:
    assert sha(typecheck['qualification'])==typecheck['qualification_sha256']
    tq=read(typecheck['qualification'])
    total=sum(len(v) for v in tq['profiles'].values())
    assert tq['source_manifest_sha256']==SOURCE and tq['compiler_invocations']==total and tq['tests_executed']==0
    assert tq['original_identities_terminal'] and not tq['candidate_applied'] and not tq['normal_PE_built']
    assert sha(tq['raw_archive'])==tq['raw_archive_sha256']
    assert all(sha(p)==h for p,h in tq['normal_product_pins'].items())
    for profile in ['Debug','Release']:
        assert len(tq['profiles'][profile])>=5 and all(v['compiler_exit_code']==0 and not v['test_execution'] for v in tq['profiles'][profile].values())
    matches_current=all(tq['candidates'].get(k,{}).get('sha256')==v['sha256'] for k,v in w['current_isolated_development']['candidates'].items())
    typecheck_note=('現行三候選' if matches_current else '前一組候選，非本修訂')+'已完成Debug／Release各'+str(len(tq['profiles']['Debug']))+'項Rust編譯／型別分析，共'+str(total)+'項，原'+str(tq['original_session_id'])+'實際exit0；含完整library、library案例程式、正常入口、T鍵／#20'+('／正式恢復' if 'teleport-recovery-formal-code' in tq['profiles']['Debug'] else '')+'整合案例程式。'+link('獨立核對與原始紀錄',typecheck['qualification'])+'。僅rustc --emit=metadata，未執行案例、未連結正常EXE；完整模組、'+str(tq.get('candidate_overlay_files',9))+'個覆蓋檔、每版13直接依賴、1049正常輸入、正常兩EXE及程序終止均核對，仍有編譯警告。'
    if tq.get('java_compilation'):
        jc=tq['java_compilation'];typecheck_note+='Java fixture僅對固定'+jc['formal_reference'][:8]+'編譯出'+str(len(jc['classes']))+'個class，未啟動Java類別／伺服器；實際相容性與production owner資格仍待。'
    typecheck_note+='首次準備、換行前綴與reader目標清單等工具失敗及修正前後紀錄均封存，未改寫產品結果。'+('本次修訂尚未編譯，舊結果不可移記。' if not matches_current else '')+'\n\n'
candidate_note=''
kc=w.get('current_native_keyboard_target_candidate',{})
if kc.get('candidate'):
    assert sha(kc['candidate'])==kc['candidate_sha256']
    candidate_note='開發候選：'+link('T鍵切換已呈現目標',kc['candidate'])+'，3個Runtime檔與1測試檔，4個已寫案例。按穩定物件ID切換實際可見的有效目標，核對呈現環境／致盲、遮擋／小地圖、epoch／viewport及角色生命週期；不自行發包或扣MP。'+link('候選靜態審查',kc.get('static_review_path',w['current_user_desktop_development']['static_review']))+'。\n\n'
bc=w.get('current_native_blind20_candidate',{})
if bc.get('candidate'):
    assert sha(bc['candidate'])==bc['candidate_sha256']
    bp=read(bc['candidate']);assert sha(bp['contract'])==bp['contract_sha256']
    candidate_note+='新增'+link('#20致盲候選',bc['candidate'])+'：補正式C14目標／座標、一格範圍、500ms重用及已學技能選單，沿用原746有限效果與S0A視覺處理；MP／機率／16秒期限仍由伺服器決定。2個Runtime檔與1新增測試檔，6個已寫案例；'+link('正式位元碼契約',bp['contract'])+'。所有候選均**未執行測試、未連結正常PE、未套用**，編譯範圍見下段，不算正常3E功能、實際施放或GUI資格。\n\n'
elif candidate_note:
    candidate_note+='候選尚未編譯、測試或套用，不算正常3E功能或資格。\n\n'
cc=w.get('current_native_cancel_recovery_candidate',{})
cancel_cases=6;baseline_cases=2
if cc.get('candidate'):
    assert sha(cc['candidate'])==cc['candidate_sha256']
    cancel_candidate=read(cc['candidate']);cancel_cases=cancel_candidate['new_tests_written'];baseline_cases=cancel_candidate['baseline_additional_cases']
    dc=w['current_isolated_development'];assert dc['overlay_files_disjoint'] and dc['tests_executed']==dc['builds_executed']==0
    refresh_note=''
    if cancel_candidate.get('new_tests_in_this_revision') or cancel_candidate.get('formal_target_sha256'):
        assert sha(cancel_candidate['contract'])==cancel_candidate['contract_sha256']
        if cancel_candidate.get('new_formal_integration_cases'):
            refresh_note='候選保留首批完整清單沒有原角色時，也只對未取得授權的LOGIN回復做一次C8來源確認；正式目的地傳送與部分接續仍拒絕。正式驗證fixture改預設甲、按名稱選乙，新增連續取消、失敗後連續取消及stage自然33秒＋新LOGIN來源自然31秒三個ignored正式案；另新增缺角色的加密loopback案。'+link('契約與等待界線',cancel_candidate['contract'])+'。共11個元件與3個新正式案，均未執行；'+('固定JAR期待與只讀observer已遷移E149，實際production初始化／相容性另驗。' if cancel_candidate.get('formal_target_sha256','').startswith('E149') else 'Java固定75仍待E149遷移。')
        else:
            refresh_note='回復逾時且零選角確認／零背包批次時，送一次所選來源GAME C8、清空未重新確認清單，等新清單完成才開放選角；部分接續不重試。'+link('靜態契約',cancel_candidate['contract'])+'。尚未實跑自然到期。'
    candidate_note='G06.3d優先開發：'+link('取消／回復逾時候選',cc['candidate'])+'保留第二次cancel不再覆蓋未取用關聯的修補；取用時核對LOGIN端點／帳號／來源，仍只取一次，不新增自動登入，原世界退役／帳密清空／時限維持。'+refresh_note+'1個Runtime與'+str(len(cancel_candidate['test_files']))+'個既有測試檔，共'+str(cancel_cases)+'個新增案（'+str(baseline_cases)+'個原版反例對照），尚未實跑反例；'+link('靜態審查',cc['static_review_path'])+'。三候選共'+str(dc['overlay_total_files'])+'個互斥檔、'+str(dc['new_tests_written'])+'個已寫案例。\n\n'+candidate_note
history=w['qualification_before_text_index_order_reference'];assert sha(history['state'])==history['state_sha256'];prior=read(history['state']);app=w['current_native_installed_resource_profile_application'];assert app['source_manifest_sha256']==SOURCE
cp=Path(app['candidate']);c=read(cp);assert sha(cp)==app['candidate_sha256'];apps=[p for p in(V/'native-installed-resource-profile-normal').glob('application-*/application.json') if read(p).get('source_manifest_sha256')==SOURCE];assert len(apps)==1;app_path=apps[0]
assert all(sha(R/n)==h for n,h in read(app_path.with_name('applied-compiled-inputs.json')).items())
before=read(Path(w['last_goal_turn_evidence']).with_name('readback.json'))['after_files'];assert len(before)==8 and all(sha(R/n)==h for n,h in before.items())
review=read(app['candidate_review']);assert sha(app['candidate_review'])==app['candidate_review_sha256'] and sha(review['component_qualification'])==review['component_qualification_sha256']
rows=[]
def row(key,label,flag='qualified'):
    p=w.get(key,{})
    if p.get('source_manifest_sha256')==SOURCE and p.get(flag) and p.get('qualification'):
        assert sha(p['qualification'])==p['qualification_sha256'];rows.append('| '+label+' | 已限定核對 | '+link('資格',p['qualification'])+' |')
    elif p.get('source_manifest_sha256')==SOURCE and p.get('observed_exit_code') not in [None,0]:rows.append('| '+label+' | 失敗已封存 | '+link('原批次',p['run'])+' |')
    else:rows.append('| '+label+' | 當版待驗 | 前版資格保留 |')
row('current_native_installed_resource_profile_normal_qualification','正常兩版／各19冒煙','qualification_granted')
normal_ok=w.get('current_native_installed_resource_profile_normal_qualification',{}).get('source_manifest_sha256')==SOURCE and w.get('current_native_installed_resource_profile_normal_qualification',{}).get('qualification_granted',False)
lib=w.get('current_native_catalog_library',{});lib_ok=lib.get('source_manifest_sha256')==SOURCE and w['current_full_default_library_verified']
if lib_ok:
    assert sha(lib['completion'])==lib['completion_sha256'];rows.append('| 完整library兩版 | '+ '／'.join(p+str(x['passed'])+'過'+str(x['failed'])+'敗'+str(x['ignored'])+'忽略' for p,x in lib['profiles'].items())+' | '+link('逐名／程序核對',lib['completion'])+' |')
else:
    a=w.get('current_native_catalog_library_active',{});partial=a.get('independently_qualified_profiles',{}) if a.get('source_manifest_sha256')==SOURCE else{}
    for p in partial.values():assert sha(p['qualification'])==p['qualification_sha256']
    state='／'.join(p+(str(partial[p]['passed'])+'過0敗'+str(partial[p]['ignored'])+'忽略' if p in partial else'待完成') for p in ['Debug','Release'])
    rows.append('| 完整library兩版 | '+state+' | '+('／'.join(link(p+'限定核對',x['qualification']) for p,x in partial.items()) if partial else'預期1975過／488忽略，依實際結果')+' |')
for key,label in [('current_native_library_resource_optin','原資源10'),('current_native_library_shared_resource_optin','現行資源71'),('current_library_motion_resource2','腳步／NPC資源2'),('current_library_haste_resource2','加速效果資源2'),('current_native_rental_return_reconnect_api_qualification','正式返回API16'),('current_native_catalog_controller_qualification','控制器六分支96')]:row(key,label)
for key,label in [('current_native_haste43_production_self_peer_formal75','#43生產library自用／對人'),('current_native_haste43_production_low_MP_refusal','#43生產library低MP拒絕'),('current_native_cancel_retirement_production_formal75','生產library取消清理')]:row(key,label)
broad=w.get('current_native_catalog_broad_qualification',{})
if broad.get('source_manifest_sha256')==SOURCE and broad.get('completion_observation'):
    assert sha(broad['completion_observation'])==broad['completion_observation_sha256'];rows.append('| 廣域117目標＋main兩版 | 已限定核對 | '+link('236映像',broad['completion_observation'])+' |')
else:rows.append('| 廣域117目標＋main兩版 | 當版待驗 | 不含完整library／GUI |')
for key,label in [('current_normal_GUI_room_service_keyboard','普通房間／密碼'),('current_normal_GUI_rental_entry_keyboard','租裝入口'),('current_normal_GUI_hotel_return_keyboard','旅館返回'),('current_normal_GUI_quantity_Enter_negatives','Enter三負向'),('current_normal_GUI_EOF_cleanup','EOF清理負向')]:
    done=[]
    for p in ['Debug','Release']:
        q=w.get(key,{}).get(p,{})
        if q.get('source_manifest_sha256')==SOURCE and q.get('qualification'):assert sha(q['qualification'])==q['qualification_sha256'];done.append(p)
    rows.append('| GUI '+label+' | '+('／'.join(done)+'限定通過' if done else'當版待驗')+' | 完整GUI／像素另驗 |')
active='；'.join(n+' 原session **'+str(w['active_workload_sessions'][n])+'** '+link('原批次',p.get('run',p.get('case'))) for n,p in w['active_workloads'].items()) or '沒有已登記活動批次，依下列順序接續'
scope=V/'native-current-skill-request-scope/E0E6409E-20261020-120226-575433/scope.json';ss=read(scope);assert sha(ss['current_request_source'])==ss['current_request_source_sha256'] and ss['recognized_count']==38 and ss['missing_count']==36
inventory_failure=w.get('current_resource2_tooling_inventory_failure',{})
inventory_note=''
if inventory_failure.get('source_manifest_sha256')==SOURCE:
    assert sha(inventory_failure['raw_archive'])==inventory_failure['raw_archive_sha256']
    inventory_note='資源2工具曾沿用舊2460篩除數而wrapper1，原Debug Rust實際2過0敗／2461篩除已封存；工具改依實際inventory減2，獨立核對並沿用原Debug結果，只接續Release，未重跑Debug。'+link('原工具失敗與回復',Path(inventory_failure['raw_archive']).with_name('evidence.json'))+'。\n\n'
gui_rejections=w.get('current_GUI_readback_rejections',[])
gui_note=''
if gui_rejections:
    q=gui_rejections[-1];assert sha(q['raw_archive'])==q['raw_archive_sha256']
    gui_note='Release租裝入口原98757之Runtime退出0，但獨立reader原eae5f3退出1：A的世界就緒觀測仍在載入，未滿足原地圖500斷言。原紀錄封存，維持產品及reader不變；新批次先確認地圖500才移動，不能把原批次算通過。'+link('原核對拒絕',Path(q['raw_archive']).with_name('evidence.json'))+'。\n\n'
g=w.get('current_native_catalog_gui_active',{})
if g.get('status')=='normal_GUI_original_failure_preserved':
    gui_note+='最近GUI原'+str(g['original_session_id'])+'之'+g['profile']+'／'+g['scenario']+'已失敗保留，原exit '+str(g['observed_actual_exit_code'])+'、記錄程序均終止；'+link('原批次與錯誤',Path(g['case'])/'cohort.json')+'。不授資格，'+('依使用者要求暫緩測試，先做隔離候選開發。' if deferred else '先依原時限新批次完成尚缺流程。')+'\n\n'
deferred_next=''
if deferred:
    deferred_next='0. 目前先開發與隔離編譯，測試依使用者要求暫緩。'+formal_next
    if not fixture_binding:
        deferred_next+='恢復測試後優先驗G06.3d取消／回復逾時候選：原版'+str(baseline_cases)+'個反例對照、候選'+str(cancel_cases)+'案與原既有案，再驗正式服務端點／帳號／來源綁定及實際自然到期後C8新清單；接著T鍵／#20元件及正常建置。獨立核對後才套用並跑新當版完整library、控制器96及正常GUI。'
    deferred_next+='原正式技能書、成功／拒絕／低MP、舊目標／取消／呈現環境仍須實跑，Debug租裝／其餘GUI未完成不得交人工。'
failure=Path(c['failure_run']);body=f'''# LiTo381 總開發報告與接續指南

更新：{datetime.now(timezone(timedelta(hours=8))).strftime('%Y-%m-%d %H:%M')}（Asia/Taipei）。目標：完成原生剩餘開發，先當版完整library Debug／Release，再Release回歸與正常GUI全流程；自行修復、自動驗證後才交人工。G01–G12未完成，尚未交人工。

{mode_note}

{formal_note}## 當版與停點

當版 **3E58268A**，SHA-256：`{SOURCE}`；1048來源／1049編譯輸入。只改resources.rs之cfg(test)精確Text.idx參照，Runtime未改：保留舊CF4D420D與原歷史參照，新增B85FAF26，兩者均須搭配同一19BFE9D4 Text.pak；所有筆數／payload／重名拒絕斷言保留。{link('套用／原檔回復／審查',app_path)}。

{candidate_note}{typecheck_note}{active}。有活動批次時僅接續原程序，來源及對應QA固定。

{link('工作佇列',R/'工作佇列.json')}；{link('最新八檔checkpoint',Path(w['last_goal_turn_evidence']).with_name('readback.json'))}。舊來源資格保留，不移記當版。

| 3E58268A來源／歷史75DA69B0協定範圍 | 原限定結果 | 證據／限制 |
|---|---|---|
{chr(10).join(rows)}

## 修正依據與前版結果

E0E6409E已完成正常兩版各19、完整library兩版各1975過0敗488忽略（原5199 actual exit0，11記錄身分終止）、額外資源10兩版各10過0敗。其71項Debug實跑70過1敗、Rust101／wrapper1，Release未啟動；Text.idx雜湊不符既有參照，原結果與完整日誌保留。{link('原失敗',failure/'cohort.json')}；{link('E0完整接續報告',history['prior_report'])}；{link('E0資格／失敗索引',history['state'])}。

獨立C#1.5.4對新舊完整索引比對：37497筆名稱／位移／大小多重集合相同，192列只在同名組內換序，28361名稱／110組重名、PAK與list.spr雜湊均不變。隔離兩版各重現原Text預期失敗1、候選Text過1、相關預設33過0敗4忽略；11記錄程序身分終止。{link('完整索引比較',c['comparison'])}；{link('隔離資格',review['component_qualification'])}。套用前候選不代替當版完整回歸。

E0的#43加速術自用／對人／畫面範圍及取消先保存回復關聯再清空舊世界的Runtime修正維持。A695完整library1974／488、85資源、API16、控制器96、廣域Debug544／Release540各24忽略及兩版GUI房間／租裝／旅館／Enter／EOF限定資格，與所有反例、工具誤讀及編譯失敗，均可由E0索引接續追溯；不得當作當版GUI或完整G01–G12完成。

{inventory_note}{gui_note}## 接續順序

{deferred_next}

1. {'現有3E正常兩版與完整library僅保留原75範圍；新候選合併套用後，須重新核對當版完整library Debug／Release，不移記舊結果。' if advance79 else '正常兩版與完整library均已獨立核對，不重跑已核對範圍。' if lib_ok else'正常兩版各19冒煙已限定核對；只續原完整library批次，依實際逐名、PE、來源與程序終止核對兩版結果。' if normal_ok else'先續當版正常兩版與各19冒煙，實際退出後獨立核對；再跑完整library Debug／Release，預期1975過／488忽略，依實際逐名、PE、來源與程序終止核對。'} 不在計時GUI伺服器旁編譯。
2. 當版互斥85資源（10＋71＋2＋2）保留既有資格。API16、生產rlib之#43自用／對人、低MP及取消清理在75已核對；E149相關API／GUI需重新驗證，不能移記舊結果。其餘403個未執行的預設忽略項逐名前提審查；5個嚴格舊TW項缺根目錄TW13081901.pak（要求FC7B4CAC），不得用launcher檔替代，不算通過。
3. 完成正常GUI普通房間、租裝入口、旅館返回、Enter負向、EOF與取消清理，再補#43自用／對人／低MP／舊目標及owner退役。鍵盤／可及性已恢復；畫面擷取FrameArrived逾時與geometry unavailable尚未證明恢復，不重試相同條件，不把CPU PNG當GUI像素。
4. 接續當版控制器六分支96與廣域117目標＋main兩版Release回歸；若套用新增native_blind整合目標，先重新盤點全部目標，不沿用117固定數。完成共享鑰匙／訪客離房、原3600／7200秒自然到期、傳送取消／同端點恢復、普通服務與雙玩家戰鬥退出。自然到期須實際等待，不注入時鐘或縮短租期。既有計畫只供重備，不移記資格。
5. 技能靜態已識別38／原74種觀測ID，餘36；{link('靜態盤點',scope)}之請求來源檔與當版逐byte相同，未新增技能書或施放觀測，識別不等於完整技能完成。繼續玩法／世界生命週期、全地圖資源／遮擋／相機、物品技能矩陣、GUI像素／滑鼠／IME／多DPI、實體音效、效能與長時間故障恢復、乾淨主機Release可攜交付及最終稽核後，才交人工。

## 邊界與入口

重建版、登入器、伺服器是獨立專案與產物；相鄰專案與原客戶端只讀，登入器JAR替換提案已撤回。不碰玩家實表／正式程序，不包裝舊LiTo.bin；產物限重建版，總報告為授權例外。不派代理、其他對話或Git，不要求使用者登入／找視窗。

唯一入口啟動現代客戶端.ps1 -BuildOnly -BuildProfile Debug／Release；Cargo --locked --offline -j1、x86_64-pc-windows-msvc、target/current；Release z／thin／16／strip。現正式JAR E1498871；歷史資格綁75DA69B0，現行G06候選Java編譯綁E149；catalog B64ED920。原8／30／60秒及租期不變。GUI只用Computer Use node_repl＋@oai/sky，一次輸入立即新觀測。

下次先讀本報告、{link('AGENTS',R/'AGENTS.md')}、{link('README',R/'README.md')}、{link('架構',R/'docs/ARCHITECTURE.zh-TW.md')}、{link('歷史',R/'docs/HISTORY.md')}及最新checkpoint。當版報告工具update-master-report-text-index-order.py；E0使用haste43-cancel工具、3E使用text-index-order工具，不混用來源資格。
'''
assert all(Path(p).exists() for p in re.findall(r'\]\(<([^>]+)>\)',body));out=V/'master-report-current-work'/datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S-%f');out.mkdir();old=REPORT.read_bytes();(out/'before.md').write_bytes(old);(out/'after.md').write_text(body,encoding='utf-8',newline='\n');(out/'updater.py').write_bytes(Path(__file__).read_bytes());assert REPORT.read_bytes()==old and all(sha(R/n)==h for n,h in before.items());REPORT.write_bytes((out/'after.md').read_bytes());record=dict(source_manifest_sha256=SOURCE,before_sha256=hashlib.sha256(old).hexdigest().upper(),after_sha256=sha(REPORT),active_sessions=w['active_workload_sessions'],lines=len(body.splitlines()),GUI_full_flow_verified=False,whole_goal_complete=False);(out/'readback.json').write_text(json.dumps(record,ensure_ascii=False,indent=2)+'\n',encoding='utf-8');print(json.dumps(record,ensure_ascii=False))
