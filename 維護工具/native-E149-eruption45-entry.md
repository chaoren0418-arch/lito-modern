# E149 #45 地裂術本機入口候選

使用者仍暫緩所有測試、--list、桌面、server／DB與播放。只做來源、唯讀classfile／javap核對及metadata編譯。

沿當前33檔接收呈現候選，僅修改native_skills與native_skill_selection：#45加入角色施法，C14 `14 05 04`＋u32 actor ID＋u16 x/y共11bytes；不可自己、不可無目標，僅活體／可見／未毀guardian，基本八格Chebyshev距離。選單列「#45 地裂術（選取目標，八格內）」；reuse基值100ms，既有client-clock無45專用值。沿Target incarnation／revision／generation、Owner connection／correction／caster與controller互斥／pending，滑鼠／T鍵／施法鈕都沿實際現有入口，不加外部wrapper。物品與地面施法域不接受45。

正式分支已按同一E149核對：classic與battle兩個AttackSpell core；classfile BootstrapMethods將classic CP35958明確綁lambda$useClassicWorldAttackSpell$360（排除自己＋同map＋target ID），battle CP45726實際綁repair020$lambda$useAttackSpell$350。battle另於596／614明確拒絕自己；兩者保留原screen、地圖／視線、range8、動畫可用、狀態、目標類型與攻擊資格，並非只用同名方法索引。這是靜態分支證據，非GAME實際資格。

**資源覆寫更精確的界線**：ResourceDatabase.value先檢查CombatDatabase.bound，再走其value；否則依current／shared使用snapshot或基值，最後經CombatRules.intField。shared旗標取lito.resourceProfile；load另會讀JDBC資料庫。三段Code已直接讀JAR classfile保存；javap原stdout是CP950，唯讀解碼，不將它當UTF8。未呼叫load、getter、SQL、JDBC或任何遊戲方法；MP20／range8／100ms／gfx129仍只是catalog基值。客戶端只檢查已有MP>0，不預扣20、不推估INT／原成本折扣、不造命中／傷害、材料、HP或成功ACK；目前SERVER實際override待恢復測試後觀測。

S23仍由伺服器給action18／wire0／gfx129／source與endpoint，沿已接class7→class6獨立節點／共享資源／History／typed音效與Over＋Shadow。收到動作僅標「收到施法動作」，無立即hurt或成功判定；child source4 Hit仍只診斷，完整原hurt相對gfx／base+26與guard待綁定。原可選action0倍率、unsigned32 rollover與最大共存G11也未完成；不把完整入口等同技能驗收。

仍33檔（十九Runtime／十一Rust QA／三Java），365個頂層module與六nested參照不變。兩版library、library-test與七個既有skill integrations共18個metadata命令；沒有test body、PE連結或執行。新增十案（九純＋一原共享資源opt-in）均未跑，前175案保留，共185筆已寫案例，非library inventory。

新案包含高位元actor ID／精確11bytes／不變世界、八格對角邊界、自己／屍體／隱身／失蹤／錯目標域、真選單與learned更新、target ABA、connection／world／correction／caster失效、MP1保留伺服器成本權威、同incarnation座標更新、S23不造HP或提前hurt及拒絕優先；原資源案把Choice→請求→原S23 decode→class7雙層memory fragments／Sound128串起，無音效裝置或GUI。

來源請求識別新增45，60／180，剩120；此數字不是玩法完成或當版技能書／GUI資格。正常1049輸入／所有PE、registry與G06 binding／16案640次不變；未套用或選入正常來源。後續繼續原hurt／LT初始化／速度與當版46／47定義，及G06／完整library Debug／Release、Release回歸與GUI；使用者明確恢復測試前不得啟動驗證程序。
