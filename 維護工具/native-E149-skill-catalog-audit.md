# E149 當版技能目錄與來源缺口

這份開發清單取代以歷史75版37個已學會技能缺口推估當版剩餘工作的做法；歷史75的收據與清單保留，不能移記E149。

直接解析固定E149 JAR的classfile／Code，不使用Java反射、不載入遊戲類別、不執行任何Java方法。白名單解析<clinit>的字面值、new／dup、裝箱、Map.of／entry／ofEntries、Map$Entry陣列及putstatic；僅建立Python資料。summon／summonSpecial的31個呼叫保留為opaque entry，不執行也不猜回傳資料。

建構子欄位必須直接來自未改寫的參數local；AttackSpell兩種委派建構子只接受同類別呼叫、原參數與字面預設值，保留目標canonical ctor與offset／Code SHA。constructor validation與ResourceDatabase設定覆寫都未執行。

當版共180個技能Map字面定義／180個ID，所有skill Map物件均已展開；31個未展開helper都是非技能召喚資料，不能據此宣稱整個catalog初始化或召喚矩陣完整。正常請求識別38個，最新22檔候選48個，皆位於這180個ID內；候選尚無請求入口132個，正常來源142個。這不是技能成功／可學會／完整玩法的通過率。

122個factory方法、363處ClientSession factory呼叫另有索引。attackSpell會查8張Map；同方法中引用相同record的helper只是過度近似，未證明record參數資料流或每個ID的分支／Protocol envelope，不可自動用來新增封包。原javap的<clinit>5757條指令之offset、opcode及constant-pool參照獨立讀回核對。

下一個玩法開發批次優先從#22／#25／#30／#53的AreaAttackSpell開始：先核對E149真正選中的helper、Protocol reader、目標形狀／範圍、reuse與原特效／音效，才能實作。#27武器破壞、#39魔力奪取及其他132項仍需逐項完成；48個已識別也尚待各自協定／材料／效果／原資源與GUI驗證。G06仍為優先主線，已備16案640次；目前不重備、不執行。

初次單Map假設失敗與中間162項（尚缺AttackSpell委派）輸出完整封存；最新結果不覆寫它們。正常來源／所有PE、22檔候選、G06 binding／16案、原registry皆不變。所有測試、--list、GUI、DB／server／fixture及音效播放維持使用者暫緩，恢復後依總報告順序驗證。
