# 原class1啟動位置判讀與來源修正

發現舊verify-original-class1.py只證明44FFFB有push2／4B9390，沒有證明252／236／1801會走到該分支。直接讀固定原PE4271D92F的44F350比較鏈與constructor欄位，三者都到45001A直接啟動；44FFFB只由0x9B3／0x9B4（2483／2484）進入。1804／1805同樣直接啟動，影響後續#22／#30的S39定位。

本批只修現有252寒冷戰慄、236吸血鬼之吻、1801 STALAC：position保留S23 wire destination，不再沿heading加兩格，仍拒絕無效heading。沒有把2483／2484加為支援profile；八方向選圖、既有shadow／clothes、音效／Hit／native callback clock／terminal dwell與target incarnation守衛不變。效果仍固定在封包快照位置，後來角色移動不重新追蹤，不改HP／MP。

原工具與original-class1-qualified.json都保留，舊結論明確更正；舊兩份chill／vampiric測試用position本身當期待值，不能獨立證明原生位置。新五案包含絕對wire座標與邊界、三profile／八heading的固定位置、舊target拒絕、場景退役及明示原資源的三profile／分層繪圖。純callback向量不是原資源、伺服器或GUI證據。

首次新分析誤把0x9B3／0x9B4轉成2473／2474，分支目的地guard當場失敗；原來源／錯誤保留，改為正確2483／2484後再解析。原PE沒有載入／執行，也沒有跑測試。

一Runtime差分（內含五案、一個原資源前提），疊成25檔：十一Runtime／十一Rust QA／三Java。完整362模組、六原nested參照及既有七skill整合保留；兩版library／library-test／七skill共18metadata只編譯，新增5與前60已寫案例皆未跑。當版請求識別仍49、目錄無入口仍131；#22／#25／#30仍待。

下一步以更正後的無偏移起點接#22／#30：S39在接收時取第一名target的座標，找不到時沿原4B9360由wire caster origin走一步；不是在caster Hit時重查或任選後面的target。保留原全名單、Hit／Sound714、1805的1806 shadow／1807 clothes與Sound718／兩個Hit，再接角色ID＋座標請求與選單。#25的171→218仍需S39 class5時序／名單整合。完整實際行為須另驗，不能移記舊錯誤位置的像素／GUI資格。

正常1049輸入／所有PE、原registry與G06 binding／16案640次不變；候選未套用／連結PE／執行Java或測試／GUI／音效。使用者測試與桌面暫緩維持；恢復後依總報告先元件／G06、當版完整library兩版、Release與完整GUI，自行確認後才交人工。
