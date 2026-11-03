# E149 #90 堅固防護／#91 反擊屏障來源候選

#90堅固防護／#91反擊屏障來源已接：三byte selector-only C14，選單清除舊目標、reuse各3000；91精確walk50，實際type3武器、90防具資格及HP100／MP10基值／64秒由server掌管。5831十四frame overlap3973 source1，0／1494 Hit／2324ms；5832十七frame overlap3974 source2，83／1993 Hit／2491ms；獨立10710六frame無聲，374 Hit／665ms。均原class0 Screen，Hit只診斷並續播，不造傷害／hurt／反擊／狀態ACK；觸發S37及攻擊方S20 action2由server決定。四Runtime覆寫仍42檔28 Runtime／11 Rust QA／3 Java、371module／原六nested；兩版18metadata退出0，新12（9純／3原資源opt-in）＋前441共453案全未跑；入口86／剩94。正常1049／既有PE／JAR／registry／G06640次不變，未套用／連結／測試／桌面／播放。

先修#80選單未清舊目標的來源缺口，再接續97／98與剩餘94入口，逐一核對正式方法／協定／原資源；續查4183 wire-to-model／原clock import初始化／S0B refresh、G06物品／source/account／timer與LT hurt/base。使用者明確恢復測試後，驗90／91防具與type3武器／walk50、HP成本邊界與279／278、原三有限特效Hit後續播／音效／History、自然64秒／換裝移除／實際反擊，再依總報告完成元件／G06640次、完整library Debug／Release、Release回歸及租裝等完整GUI；核對後才交人工。

原始證據：`驗證紀錄/current/native-E149-solid90-counter91-inspection/20261024-181628-946038/inspection.json`；獨立方法核對：`驗證紀錄/current/native-E149-solid90-counter91-review/20261024-181840-931340/review.json`。完整方法均在同步區重新核對phase／位置／map／instance或room、reuse與裝備。HP須嚴格大於100（等於亦279），MP依實際智力修正成本判278；接受後才啟用能力／扣HP／實際MP與commit reuse，客戶端保留低HP／MP的server裁決，不先扣點或裝狀態。

90資格由SolidCarriageRules.runtime及SourceEquipmentPhysicalStats.supportsSolidCarriage決定，含設定與原armor metadata；不得以本機名稱或舊useType25單一路徑代替。沒有合格裝備時原server發S_MESSAGE type9「你並未裝備盾牌」；沿既有system-message呈現，不改寫成新施法ACK。91必須有實際present/type3武器且walk50，54不允許；client只阻擋已知不合格步態，其餘server決定。裝備移除由server撤能力；自然64秒由server管理。

90原S37與S20同行呈現；91 cast的S20傳給同場，5832 S37另送施法者。真正反擊時，server commit方法在防守方S37/10710，攻擊方S20/action2，另用server damage函式更新權威狀態。觸發機率、fixture D100、sourceDamage與反射均留server；本機不補反擊、hurt或扣血。

原5831／5832／10710單bank undirected、24FPS、type0／attr8／無109（class0），每格image/subframe/ticks/events完整核對。前兩者3973／3974用Effect overlap音效；10710無音效不借用3974或製造Sound0。既有loader允許這些經嚴格限定的Hit frame；本輪補finite draw對僅此三圖Hit的診斷續播，其他不支援non-audio仍拒絕。診斷隨Playback occurrence進度保存，不加入音效callback、不賦予客戶端傷害權威。

收包座標凍結、world actor0 typed音效與同來源History保留；晚到callback只進一格。0／83ms音效、1494／1993／374ms Hit、2324／2491／665ms末幀均為無延遲模型；真正時鐘／像素／音效／自然期限／GAME及GUI仍未驗。新增12案來源（9純／3顯式原資源）全未執行；metadata不提供runtime或完整library inventory資格。

靜態接線核對另發現既有#80選單未列在清除target清單，傳入既有角色選取時會碰到request80的target.is_none守衛。僅來源推導，未操作GUI或呼叫request。證據：`驗證紀錄/current/native-self-only-menu-gap/20261024-182503-920876/source-gap.json`（SHA256 5185D453294EBF2D39E26EDB30ABFE26B5D222C251395C3D79B58BF42E906B73）。本輪90／91 pinned候選未修此既有缺口；下一步先修它，再97／98。86是請求格式來源數，不是完整可玩技能數。
