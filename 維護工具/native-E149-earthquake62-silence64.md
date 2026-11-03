# #62 震裂術／#64 魔法封印來源範圍

測試及桌面操作暫緩；本輪僅來源、靜態核對及型別編譯，沒有GAME／GUI／音效播放資格。正常1049輸入、所有PE、正式JAR、registry與G06 binding／16案640次不變，候選未套用。

62為C14 `14 07 05`三bytes無目標，原AreaAttackSpell range0／radius3走居中流程；MP40、傷害32＋6d10／地屬性、reuse1500均基值與server權威。S39 action18／type0／gfx1812保存完整原target list；原PE比較鏈靜態到45001A，不走兩格或多child分支，定位固定wire caster origin，不取第一target、不跟隨施法者移動。

1812原class0、單方向16frame，subframe與source同號（沒有原其他profile的末格移到source0），Sound716在source1／Hit在source5；最後四格各48ticks，整數native callback Hit499ms／完整10038ms，late poll不catch up。保留1812 Over、1813 Shadow、1814 Screen三層，整組預檢；原guard／LT相對hurt未綁前Hit只診斷，不送替代Impact或修改HP。typed716用world actor0、真sequence／Scope。重建失敗只退役一次並釋活動序號，沿既有Many History與root ABA守衛。

64為C14 `14 07 07`接角色u32 ID／x/y十一bytes，其他可見活體、三格與reuse500基值，沿原Target／Owner／controller。原NativeSilenceMaterialAction battle／world、實際Bootstrap目標篩選及activateSilence分支已靜態核對；MP50、一寶石40318、機率／MR／INT、16秒、玩家／NPC／counter資格與失敗280由server決定。原2177十七格各4ticks、Sound808 source1、Screen仍用既有嚴格qualifier，視覺2822ms不代表16秒狀態。2177 typed音效改world actor0保留真key，target離開不截斷已收到的有限視覺音效；不安裝本機沉默timer或造成功ACK。

新增14案來源（12純＋2共享原資源opt-in），前276合290案全未跑，非完整library inventory。四Runtime差分仍36overlay／22 Runtime、367module／原六nested；來源入口70／剩110不等於70玩法驗收。原圖三層、bank空槽／上限、音效／時序／重建、server接受／拒絕／材料／自然16秒、owner與GUI全流程仍待明確恢復測試後驗；不能沿歷史75或舊映像資格。
