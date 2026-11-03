# 變形術前置：S27 原身體佇列元件

本輪是來源元件，尚未接入 World、Animator、controller或GUI；#67變形術、#68聖結界請求與選單仍未接入，入口保持72／剩108。測試與桌面操作暫緩；不得把編譯、來源元件或歷史結果當玩法完成。

正式E149 `Protocol.polymorph`送S27（0x27）八bytes：actor u32、graphics u16、walk family u8。舊原生4A70B0查actor、呼叫4E6AF0判斷身體忙碌；忙時4E6860排tag27，否則4E0490改外觀／動作family。不可在舊動作還在跑時，直接用新gfx取代舊clip。原native frame／動作佇列guard與callback交錯尚須完成現代caller綁定。

新native_appearance27元件嚴格解析S27，保存角色／生命／connection／generation／map／source_revision限定的S20／S23動作與外觀操作順序。Caller用真busy／idle邊界驅動；busy（含死亡末姿勢）不出列，每次明確邊界最多一個operation。動作取得當時body snapshot，不自行計時、發封包、造cast ACK、改HP、復活或播放音效。元件16佇列／4096admission滿載即回錯，不靜默丟棄；不是最大共存或G11資格。

重建History只保留數字metadata。同來源可恢復待處理佇列；來源退役永久清queue、隱藏body，但保留seen，A→B→A不得把退休graph復活。晚到worker只能合併同Owner的admission，不能導入queue／body或解除退役。這些是元件來源行為，尚未移記至目前遊戲身體播放。

graphics原signed WORD比較30000：0..29999屬shared、30000..32767為LT-relative、32768以上原符號值另列未資格；不猜LT初始化base，不把同數字當shared圖。元件保留raw值；shared_graphics只接受shared區域，未證明其SPR／class／所有資源可載入。

已唯讀保存正式67／68完整方法、material owner class Code、變身CSV及96個standard_menu選項。67使用object ID／原monlist與C_HACTION，是否有控制能力由server判定；68同非空clan／資格、兩寶石、32秒減傷皆server權威。後續須接完整選單／一次性Owner、S27異質佇列與body clock、同poll順序、死亡／來源退役、自然7200／32秒，再做當版自動測試；不能只新增C14分支便算完成。

新增7純案例來源，加前305共312案全未跑，非完整library inventory。正常1049、所有既有PE、正式JAR、registry與G06 binding／16案640次不變；未套用或PE連結。
