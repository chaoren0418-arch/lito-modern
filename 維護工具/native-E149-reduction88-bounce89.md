# E149 #88 增幅防禦／#89 尖刺盔甲來源候選

#88增幅防禦／#89尖刺盔甲來源已接：三byte selector-only C14，選單清除舊角色目標、reuse3000／5000；正式HP50／120及MP5基值、192／64秒與戰鬥效果由server決定。原4824十九frame／4648二十二frame為class0 Screen，source3／4 overlap3089／1924、82／123ms，有限圖形2366／2281ms，不造狀態timer或ACK。補S10/279 HP不足辨識與回饋，實際HP／MP／AC只收權威封包；89武器損壞機率／近戰資格留server。五Runtime覆寫仍42檔28 Runtime／11 Rust QA／3 Java、371module／原六nested；兩版18metadata退出0，新10（8純／2原資源opt-in）＋前431共441案全未跑；入口84／剩96。正常1049／既有PE／JAR／registry／G06640次不變，未套用／連結／測試／桌面／播放。

接續90／91與剩餘96入口，逐一核對正式方法／協定／原資源；續查4183 wire-to-model／原clock import初始化／S0B refresh、G06物品／source/account／timer與LT hurt/base。使用者明確恢復測試後，驗88／89實際HP成本邊界與279／278、權威HP／MP／AC、自然192／64秒與overlap／History，再依總報告完成元件／G06640次、完整library Debug／Release、Release回歸及租裝等完整GUI；核對後才交人工。

原始證據：`驗證紀錄/current/native-E149-reduction88-bounce89-inspection/20261024-180259-031734/inspection.json`；獨立方法核對：`驗證紀錄/current/native-E149-reduction88-bounce89-review/20261024-180754-075366/review.json`。完整方法中兩技能均在同步區核對phase／位置／map／instance或room、reuse，再要求HP嚴格大於成本（等於亦279）、MP足額（278）。只有接受後啟用server能力、扣HP／實際MP並commit reuse；S20 action與S37只負責呈現，不據此宣告192／64秒狀態。89另發ownCharacterAttrDef，AC由該權威封包更新。

89近戰武器損壞在server核對攻擊方soulOfFlame、技能尚存／武器存在且非遠程，再於1..100判定；客戶端不roll、不改耐久。

原4824／4648 animation/action0、undirected、24FPS、單bank、attr8與effect(0 -1)及每格image/subframe/ticks/events皆嚴格核對。`<3089`／`<1924`解析為Effect但原語義為允許重疊音效，不是子SPR，無Hit。既有finite History／收包座標快照保留；typed音效用world actor0域避免目標移除截斷。晚到callback只進一格、從實際dispatch續排，82／123ms只是無延遲模型。

新增10案來源（8純／2顯式原資源），待恢復測試後才執行；metadata不提供runtime、完整inventory、原像素、音效播放、GAME、自然期限或GUI資格。測試及桌面操作依使用者要求暫緩。
