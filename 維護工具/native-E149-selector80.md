# E149 #80 無目標選單修補與共用分類

#80冰雪颶風選單舊目標來源缺口已修入隔離候選：request與原生menu共用selector_only分類，保留原18個selector-only ID，live／stale角色選取均先清除；直接request附帶target仍拒絕。另五種self-menu特殊路徑2／3／13／60／131及actor／ground／item／summon／mass格式與Owner／book／MP守衛保留。無新技能／視覺／timer／ACK。兩Runtime覆寫仍42檔28 Runtime／11 Rust QA／3 Java、371module／原六nested；兩版18metadata退出0，新2純QA＋前453共455案全未跑；入口86／剩94不增加。歷史來源gap原檔保留，僅候選來源修正，GUI／runtime未驗。正常1049／既有PE／JAR／registry／G06640次不變，未套用／連結／測試／桌面／播放。

接續97／98與剩餘94入口；selector-only新技能只擴充native_skills::selector_only，request與menu共用，來源coverage解析須包含此helper。逐一核對正式方法／協定／原資源；續查4183 wire-to-model／原clock import初始化／S0B refresh、G06物品／source/account／timer與LT hurt/base。使用者明確恢復測試後，驗#80帶live／stale選取及Owner／learned負向、90／91三有限特效Hit後續播／自然64秒，再依總報告完成元件／G06640次、完整library Debug／Release、Release回歸及租裝等完整GUI；核對後才交人工。

來源證據：`驗證紀錄/current/native-self-only-menu-gap/20261024-182503-920876/source-gap.json`；正式#80方法與既有守衛核對：`驗證紀錄/current/native-E149-selector80-review/20261024-183118-058894/review.json`。舊gap與前三個來源SHA完整保留，不改寫歷史的source_fix_applied:false。新qualified-development將另列resolved_source_issues，僅表示本輪未選入的候選來源已修；實際正常來源／GUI仍未套用或驗證。

原因：request80本來就要求target.is_none，原生menu卻漏列80，會把live目標傳入或先拒絕stale目標。現在18個既有selector-only ID集中於native_skills::selector_only，request用guard arm，menu在resolve目標前用同一helper清除target。2／3／13／60／131保留既有特殊自身規則；41／61／75屍體、actor／ground／item／summon／mass格式與Owner／book／MP皆沿用，80直接帶target仍拒絕。

無新技能或協定擴展；來源coverage parser必須同時計入request數字arms、selector_only helper、ground、item／summon／mass四類既有入口，避免guard-arm重構造成虛假缺口。實際仍86／剩94，下一組97／98。兩個新案例走原生Model→choice→request，涵蓋live／過期角色選取、權威值不變及targeted／Owner／learned負向，不單獨測同一份分類清單。依使用者要求，兩案只編譯未執行；原資源、TCP、GAME、自然期限及GUI資格均不增加。

下一組97／98第一批原始方法與Text已唯讀擷取：`驗證紀錄/current/native-E149-hide97-venom98-inspection/20261024-183937-685740/inspection.json`，SHA256 5CEF02A997634D5439B5B5C625F0679DEF06506EA2FCE608EF5E1CEC724E112D（9方法／5 class Code）。97名為暗隱術，2941只有type9、無action；初批名稱篩選沒有涵蓋共用InvisibilitySpell，因此另補10完整方法／10 class Code：`驗證紀錄/current/native-E149-hide97-supplement/20261024-184556-713087/supplement.json`，SHA256 764C6693082A4A0A842954E9637CBD38EBFEA2882BFFFCD5F4CF352411661FD8。這兩批尚待完整review，未加入實作／編譯或runtime資格；不能直接把2941接成普通有限SPR。98正式名稱附加劇毒，後續先查材料owner／實際回呼與poison authority，不在本機roll或設timer。
