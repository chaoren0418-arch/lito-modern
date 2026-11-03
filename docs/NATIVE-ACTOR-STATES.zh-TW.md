# 原生角色中毒與隱身繪製

本階段限定普通視野下 S0B 初始狀態、S32 中毒更新與 S34 隱身更新。現行完整客戶端仍未完成。

原始 PE 固定 SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`。原始 dispatch、handler、初始化、RGB555 查表與定時器的位元組及反組譯保存於 `驗證紀錄/current/native-actor-states/native-state-raw.json`。正式 JAR 固定 SHA256 `589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7`。

S0B 狀態位 1 對應綠色、8 對應暗色、128 抑制繪製；128 未設置時，位 2 對應隱身。S32 的兩個 byte 分別更新綠色與暗色旗標；S34 更新隱身。新 S0B 身份覆寫、移除及換世界都清除舊狀態。更新不預測 HP/MP、不更改座標、身份或動作佇列。

RGB555 每色通道先縮為 5 bits。綠色只對 G 加上 `floor((abs(12-tick%24)+16)/2)`，飽和至 31；暗色為 `floor(source/2)+3`；自身隱身為 `floor((source+destination)/2)`。輸出展開為 `(v<<3)|(v>>2)`。原始 RGB555 green 入口 4C13A0、dim 入口 4C1170；565 入口是另組函式，未混用。來源1021已改取共享50ms回呼的wrapping u32計數；下方新增段落列出當版限定驗證，完整原版進場路徑仍待。

其他隱身角色與 suppressed 物件不提交 body、附件、陰影和可點選表；自身隱身保留半透明 body。Screen 附件優先於 Half；普通附件隨自身隱身變 Half；毒色不染附件。陰影保留既有 half 混色。可見半透明 body 保留點選像素是明示現代輸入政策。舊畫面 visual revision 失效；攻擊及目標技能拒絕目前不可見的角色。schema32 當時選取 token 僅保留 generation／incarnation，現身可恢復。後續世界名稱階段已修補：遠端進入隱身時增加 target_revision，退休既有技能／連打／Hover token，現身須重新捕捉；權威 incarnation 不變。新修改的完整回歸狀態見 NATIVE-WORLD-LABELS。

驗證包含 28 組真實 3225 body／6262 附件／3226 陰影，在牆前與空地各七個狀態、兩種附件模式的獨立 scalar RGBA 與 owner 比對。Over 附件使用真實素材，但模式是刻意的合成 fixture；原角色0的實際附件模式為 Screen。共用 SPR 解碼器，並非獨立解碼證據。

隔離 `room-world-states` 測試透過正式 Protocol 產生器、真實 ClientSession 加密 writer 傳送九個封包，搭配十階段畫面擷取。檔案只協調擷取時機，不寫入客戶端 World。這是受控封包注入，未代表已完成自然中毒／隱身技能玩法。既有 TCP 回歸需同一 EXE 全數重跑後才發佈 receipt。

待完成：see-invisible／GM 視野、其他 attr104 及狀態旗標、原版插入與 timer 歷史、RGB565 等價、地面名稱、GUI／IME／DPI／音效裝置及玩家驗收。

## 2026-10-16 原綠色回呼與退役佇列元件

原4878F0的31bytes回呼、原schedule／dispatch／cancel／map filter與scene cleanup等十片段共1121bytes已在獨立x86 helper實際執行；532計數向量、16狀態、72佇列列與所有guard經獨立Python讀回核對。每次回呼先遞增wrapping u32，再從新讀時鐘排50ms；延遲不按elapsed補跑，重複回呼不合併。換圖過濾保留九項、scene cleanup保留七項，兩者均移除Green但保留此計數；顯式再次呼叫會延續計數。這是條件式元件驗證，未執行整個client或所有進場路徑、0遊戲sessions，產品仍按elapsed計算，尚未宣稱修正。

[原元件獨立資格](../驗證紀錄/current/native-actor-green-clock-original/20261016-150226-468757/qualified.json)。

## 2026-10-17 source1021共享綠色回呼接線

2026-10-17來源1021：綠色效果已改由純Rust共享回呼計數，與Water同一佇列；延遲不按elapsed補跑，換圖／退役取消舊回呼並保留相位，較舊或已退役世代在讀時鐘前即被拒絕。不可變render binding攜帶本次相位，資源cache key不隨50ms計數抖動；正常世界使用owned binding，無綁定的資源檢視只採固定相位。Debug五實際AMD64/PE32+測試EXE共36過0敗0忽略與source/test/raw封存已獨立讀回，包含原532計數向量／16狀態／72queue rows、世代／owner守衛、固定動畫時間的真資源像素變化及零重播聲音。Release仍沿同48319編譯，來源與對應QA凍結；新版唯一入口正常兩主／GUI、default109+main、整庫與完整TCP仍待，不移記source1019資格，完整G01–G12未完成。

[Debug限定資格](../驗證紀錄/current/native-actor-green-clock/scoped-20261016-160106-048570-Debug/qualified.json)。


## 2026-10-17 source1021雙版共享時鐘通過

2026-10-17來源1021：共享Water/Green回呼、較舊/退役世代拒絕與不可變render相位已Debug/Release各五實際AMD64/PE32+ EXE、36過0敗0忽略獨立核對；兩版同180741A8來源與source/test/raw封存讀回。同48319實際exit0，原Cargo130780已退出。場景/移動/換代/地形駐留/光源生命週期六目標及Raster/Scene兩單元群正沿14731執行，其後唯一入口正常兩主仍待；當版GUI/default109+main/整庫/完整TCP與完整G01–G12尚未完成。


## 2026-10-17 source1021正常兩主及相關回歸

2026-10-17來源1021：共享Green/Water五目標兩版各36過0敗0忽略，追加六個場景/世界/移動/圖層/駐留/光源生命週期目標與Raster/Scene兩群各56過0敗6既有opt-in忽略，合每版12實際EXE、13執行群92過。唯一入口正常Debug251E35A4／Release139B5247、同180741A8來源與source/test/main/raw已獨立讀回，同14731實際exit0。目前Debug旅館租鑰匙/歸還正沿24769以C02/TW+extended執行；當版旅館runtime、GUI、default109+main、整庫/完整TCP及完整G01–G12尚未完成。
