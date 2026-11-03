# #67 變形術／#68 聖結界來源候選

僅隔離來源開發及 metadata 編譯。使用者要求測試與桌面操作暫緩；新增案例全部未執行，沒有遊戲、GUI、音效或完整 library 通過聲明。正常來源、既有 EXE、registry、相鄰登入器與正式 JAR 不變。

兩技能均為 C14 七 byte：`14 08 02/03`＋目標角色 u32。沒有選取時對自己，明確失效的目標不能退回自己；存活／可見與原 screen diamond 在本機檢查。#67 reuse 10ms、#68 1500ms；正式 MP80／30、材料40318×1／×2、#68同非空血盟與玩家資格、控制能力／機率／狀態／自然7200／32秒一律由伺服器判定，不在本機造成功或期限。

有控制能力時，伺服器把 S2A `monlist` 給被施法角色的 session。接收者不需要學會#67、不需要本機cast pending／MP或背包戒指推測。沒有控制能力時由伺服器選隨機外觀；本機只接權威 S27／S0B，不自開選單。

新增 `native_shape67`：完整解析 S2A 的物件ID、兩個NUL字串、u16參數數目、所有參數及精確尾端後才改選單。只接受自己物件的空parameter／零args `monlist`；別人ID的monlist消化後不落進租裝NPC流程。自己空html關閉，其他自己dialog取代選單；格式壞掉不改現有grant。

選單保留正式CSV的96個standard_menu字串與最低等級資料，不暴露隱藏選項、不用推測等級禁用。選單有佔位項，必須明確選「解除目前變身」或一種外觀，再按套用。回覆為 C_HACTION `27`＋自己ID＋canonical action NUL＋空parameter NUL。送出前即消耗grant，傳送失敗不給重送權限。關閉／Escape僅本機收起，不會送`none`；S37 #230或#228並非變形或結界成功ACK。

Owner綁連線epoch、world generation／map、phase／room、source、背包角色owner、自己actor ID／incarnation／target_revision及control interruption。重連／退役／新生命／S27外觀變更會失效；一般背包數量修訂、技能書或普通action sequence不冒充新角色。三個原生互動控制項均有不回繞token命令ID，處理通知同時核對來源HWND與token；更新選單後的舊排隊點擊不能套到新grant。命令ID耗盡明示錯誤，不重用舊ID。

視窗新增非模態提示、下拉選項、套用、關閉。選單期間清除持續移動／攻擊和舊目標，阻擋世界捷徑及畫面點擊。Enter在下拉只收起列表；只有聚焦且啟用的套用／關閉按鈕才走當前token的原生通知，忽略autorepeat。Escape本機關閉。來源檢查再補上新選單聚焦下拉、允許Tab、關閉後返回世界焦點；首版18metadata的資格限修正前，最終來源重新編譯18個目標。完整控制項／輸入／像素／IME／DPI行為尚未執行驗證。

#228沿既有原SPR七幀與Sound132，改用world actor0的typed audio，避免目標離場篩掉已接收音效；#230沿既有七幀／Sound124。時間和圖層仍須原資源與GUI實測。沒有新增本機32秒圖示、傷害減半推算或7200秒變身計時。

本輪新增16案來源：9個選單核心、4個請求／角色lease、2個控制器、1個原資源opt-in。40檔overlay：26 Runtime／11 Rust QA／3 Java，369 modules、原六個nested children保持原路徑。入口來源辨識74、剩106，下一步#69／70。這些數字是來源盤點；338個逐輪新增案例全未跑，不等於完整library inventory。
