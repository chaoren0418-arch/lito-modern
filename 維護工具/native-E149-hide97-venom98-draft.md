# E149 97／98 未編譯來源草稿與待查路徑

目前正式metadata候選仍為selector80：42檔／28 Runtime／371module／原六nested，455個新增來源案例只編譯未跑，請求格式來源86／剩94。97／98僅另存四Runtime草稿及四案QA（3純／1原資源opt-in），沒有啟動編譯、測試或GUI；不可移記此前18metadata。

草稿：`維護工具/candidates/native-E149-hide97-venom98-draft-20261025-050000/draft.json`。request／menu共用selector_only，97三byte[20,12,0]／reuse9990，98三byte[20,12,1]／reuse0；正式成本97 MP10／32秒，98 MP10／HP1／40321一個／320秒皆server決定。98原2942九幀、source1 Sound1505／model0ms／有限1496ms、class1／attr8接既有finite History與world typed音效；未執行驗證。

正式完整方法揭示：97沿InvisibilitySpell依RequestShape分支，SELF_TICK走selfSpellRequest，其餘才selectorOnlySpellRequest；因此不能借60的client tick。兩者由NativeSelfCastAction實際bootstrap的lambda$invisibility$8／lambda$enchantVenom$9，經NativeLightAction.commitSelf及admitSelf入列。NativeSelfCastAction.admit尾端只呼叫NativeLightAction.admitSelf→NativeCastTimeline.admitSelf；下一步須讀NativeCastTimeline到NativeCastMaterial.commit的實際材料／owner鏈，不宣告完整原子資格。NativeCastMaterial.previous／next要求resolved及當前原鎖；客戶端不扣材料／HP／MP，不roll poison／320秒／30秒／3秒tick／damage。

97啟用或切換／取消走CombatInvisibilityRules與原state lease，server送自身S34/invisible、其他人S08/remove及現身重送；S37/2941本身不是隱身ACK。現行World的S34 draw-state及遠端選取revision守衛沿用，實際像素／32秒／切換／現身未驗。

重要來源修正：2941只有102.type(9)且沒有action，不代表無像素。原4E5B50在0 frame-count下回source0、4E5C00 action0空指標轉4E62A1靜態frame0讀圖，缺SPR可走0x460後備。Sprite.csv以casefold可找到2941-0.SPR（row34541、offset557931760、865 bytes）、1120-0.SPR（row3716、offset1276001422、211 bytes），2942-0.spr亦存在。這是快取索引取證；尚未綁定canonical pak實際成員／像素，不能以大小寫敏感搜尋無結果推定缺檔，也不能把2941當no-op或套一般有限Clip。

下一步優先：補完整44EF80（開始＋第一次callback）／44F330／4D65F0等scheduler，核對0 frame-count的靜態source0初次顯示、0ms callback與退役順序。現有native取證包含4ADDA0、44EDE0、44F350、4E5B50、4E5C00五段；44EF80目前僅唯讀瀏覽，尚未完整封存／模型接線。時序及全域交錯沒有runtime資格，不造32秒本機刪除。

證據：首批`native-E149-hide97-venom98-inspection/20261024-183937-685740/inspection.json`（5CEF02A9）；隱身補充`native-E149-hide97-supplement/20261024-184556-713087/supplement.json`（764C6693）；29 class Code／S34 codecs `native-E149-hide97-venom98-routes/20261024-185246-241843/routes.json`（B84E6AC9）；五native函式／17索引 `native-E149-hide97-native/20261024-185445-218054/native-source.json`（E7EA514A）。路徑均在驗證紀錄/current下，全文SHA見JSON與本輪來源包。

原生取證工具首版因Python關鍵字參數以數字開頭，parser報SyntaxError（297409 exit1），本文未執行；已保留原工具與`native-E149-hide97-tooling-failure/20261024-185441-752357/failure.json`，v2僅改參數名稱，ffb578 exit0。這不是遊戲／測試失敗，未啟動或重試任何測試。
