# 地面拾取與實際背包封包重播

原生「附近物品」面板提供地面物件選取、數量輸入與拾取。清單只收錄目前場景確實以原始 type9、單圖 SPR 條件呈現的地面物件，且限制在自己三格 Chebyshev 距離內。這個介面已接入控制器；GUI、IME、DPI 與實際滑鼠操作仍未人工驗收。

## 選取與權威狀態

每筆 Choice 保存完整 Actor、背包 owner、世界 generation、位置 correction，以及自己物件 ID／incarnation。送出時再次核對目前 Frame、連線、存活狀態、移動就緒、物件內容、數量及距離。清單刷新不會替既有選取自動換綁新物件；同 ID 的物件重建、數量改變或場景切換都會使舊選取失效，必須重新選取。

拾取與其他背包操作共用待回覆互斥。客戶端不先增加背包，也不先扣除或刪除地面物件。Pending 分別觀察所選地面物件與背包 revision；兩者都改變才顯示 ObservedUpdates。這表示觀察到更新，不是具有請求序號的成功 ACK。兩秒沒有足夠更新則為 NoResponse，可手動重試，不自動重送。所有權、世界或存活條件失效會中斷待回覆狀態。

## C0B 與原版唯讀證據

正式請求為 14 bytes `c h h d d c`：opcode 0B、轉換後的 X、原始 Y、地面物件 ID、正數數量及輔助 byte。X 使用有號運算 `32768 + floor((world_x - 32768) / 2)`。Rust 與正式 Java parser 的 oracle 比對全部 65,536 個 u16 X 輸入，另檢查 unsigned ID／數量與截斷、尾隨資料。

原生 PE 的完整堆疊路徑 `0x40646b..0x4064d5` 與數量回呼 `0x406668..0x4066db` 均使用物件 ID、座標及數量送出相同格式；X 的 signed subtract／sar／add 與正式 parser 相符。原版末 byte 來自 remembered-ID 集合查詢。新引擎目前明確送中性值 0；現行 BATTLE handler 不用此欄決定接受，沒有宣稱已重建原版集合的語意或 986ms 節流。原生證據固定 PE SHA256 並從原始位元組獨立解碼，不執行舊 BIN。

正式伺服器檢查 ground ID、座標、數量、三格距離、玩家狀態、容量／重量及持久化結果。成功會發送 S1C 朝向與 S20 action15，物品和地面更新仍由正式封包決定。新引擎播放原始 action15 後回待機，不把拾取動作當作攻擊或施法。

## 隔離 TCP 情境

- `room-world-pickup`：透過實際 C0C 丟出一疊四個測試材料，接著兩次 C0B 各拾取兩個。地面數量 4→2→移除，背包堆疊回到四個，兩次 action15 播放完成。每次舊 Choice 失效，待回覆期間重複拾取及其他物品操作均被阻擋。
- `room-world-pickup-stale`：先丟出四個，測試端在伺服器記憶體移除該地面物件，故意不廣播移除。客戶端仍看見舊物件，兩次手動請求皆無拾取回覆，沒有 action15、背包增加或本地預測刪除。這是明確注入的過期視圖條件，沒有宣稱完成多人競態驗收。

兩種情境均使用固定正式 JAR 的實際 ClientSession、隔離記憶體資料與 loopback TCP；不碰玩家 SQL、正式程序或正式 JAR。

## 堆疊數量與 opaque detail

成功情境揭露原有驗收器的錯誤假設：第一批拾取的 S16 detail 是 `[23,0,4,0,0,0]`。第二批合併後，伺服器更新數量與名稱，但沒有重送重新計算的 detail。此時內部 `wireItem()` 重新產生 `[23,0,8,0,0,0]`；那不是實際傳給客戶端的資料。

客戶端保留最後收到的 detail 原始位元組。驗收器獨立重播 capture 中 server-to-client 的 7C／16／17／18／19／1A／0E／6F 背包封包，逐欄比對全部客戶端 inventory，再另行記錄與伺服器重建 detail 的差異。其餘欄位仍核對正式 runtime 結果。沒有用再生的伺服器資料覆蓋客戶端，也沒有省略 detail 驗證。

## 原始資源與驗證

`verify-native-ground-pickup.py` 直接有界讀取原始 baseSprite.idx／pak 中實際使用的三個 SPR：3225-0、3225-88 與 773-0，核對 bytes／SHA256、action15 序列、回待機、地面呈現、PNG 對應 RGBA，以及完整實際 inventory 封包重播。這些新增畫面是輸出一致性檢查，不是獨立像素 golden，不增加既有 25 個像素比對。

新增六項 Rust 測試涵蓋完整 X oracle、選取與權威狀態、十二種失效／範圍條件、分開到達的地面／背包更新、畸形封包原子性，以及原始 action15 的離線呈現。

在重建版根目錄執行 `維護工具/verify-native-ground-pickup.ps1 -SkipBuild` 可重跑兩個隔離 TCP 情境；接著執行 `python 維護工具/verify-native-ground-pickup.py` 核對原資源與實際封包。完整最終回歸由 `驗證紀錄/current/native-ground-pickup/run-final.ps1` 固定來源後執行，通過後才由 `finalize-receipt.py` 發布 schema27。最終計數、EXE／JAR 身份與來源 pins 以該目錄 summary 及 `驗證紀錄/20261012/current-build-receipt.json` 為準。

地面完整排序／遮擋、世界畫面名稱、直接滑鼠選取與拾取、特殊物品目標／選項、其他技能、快捷列／buff，以及完整可玩流程仍待完成。

後續 schema28 已接入地圖／角色分片深度、透明洞與遮擋像素命中，畫面上選取可見地面物品會開啟指定數量面板。原版完整插入時序、屋頂、陰影／附件、特效交錯與實際GUI仍待完成，詳見 [原生分片遮擋](NATIVE-WORLD-LAYERS.zh-TW.md)。
