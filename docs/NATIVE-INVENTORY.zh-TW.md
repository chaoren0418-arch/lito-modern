# 原生背包與角色生命週期

本頁記錄 schema22 背包模型；schema23 已接上原始中文名稱與 LiTo 字表，名稱呈現的最新範圍見 [原始物品名稱](NATIVE-INVENTORY-NAMES.zh-TW.md)。

目前的 Win32 視窗提供「背包」只讀清單，接收正式伺服器物品名稱、數量與更新。角色背包屬於 GAME 連線與所選角色，不跟著戰鬥世界重設。物品操作、原始名稱 token 與圖示、完整 GUI 驗收仍未完成。

## 封包契約

來源為正式部署 JAR `589733435518D92C05BAEEAE37A686D384EC3F7B9897E975AD1EF50D67A74AA7` 的 Protocol；實際完整 SHA 以 `驗證紀錄/current/reference-snapshot.json` 為準。NativeInventoryOracle 直接呼叫該 JAR，產生 `tests/fixtures/native-inventory-current.json`，不是另造伺服器契約。

| Opcode | 更新 |
| --- | --- |
| S16 | 加入一筆物品 |
| S7C | byte 筆數的追加批次；空批次保留原資料 |
| S17 | 依物件 ID 移除 |
| S18 | 名稱 |
| S19 | charge byte |
| S1A | 無號數量；零仍保留至 S17 |
| S0E | blessing byte |
| S6F | 名稱、正數數量、detail |

新增記錄保留無號物件 ID、use_type、charge、icon、blessing、count、identified、MS950 名稱及不解釋的 detail bytes。名稱與 detail 各最多 255 wire bytes。初始 blessing 0..3，更新另允許 128..131；初始與 S6F count 必須正數。

解析成功才原子提交；截斷、尾隨資料、非法欄位或超出資源界限不改變原資料。未知物件更新計入 orphan_updates，不自行創造物件。物件 ID 並非 catalog ID，不從 ID、名稱或圖示猜耗材身份，也不由客戶端自行扣除數量。

## 實際重選行為

首次真正 ClientSession 測試成功收到兩件物品。第一輪重選測試失敗，因客戶端清空資料，但同連線重選相同角色時，正式伺服器不再送 S7C。`initial-tcp.log` 保留此失敗。

修正後返回選角時清除可見 owner 與 rows，內部保留最多一份該連線角色資料。完全相同 connection／character 才恢復，selection 序號另行遞增；不同角色起始空資料，接受選角後丟棄前一份回退資料，拒絕則可恢復前一角色。選角期間舊角色的權威更新寫入不可見快取。取消、失敗與斷線清除快取，重新連線不繼承。

這些行為不把 S7C 當作 replace-all；正式 bootstrap 先送既有物品移除，再送追加內容。世界重設不清角色背包。

## 原版靜態證據與差異

對只讀 recovered PE `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9` 重新解碼：S16 `004A4D90`、S7C `004B2650` 都呼叫 `00443D10`；格式 `dcchcdcsc`，批次逐筆加入。`native-receiver.json` 的有界反組譯含函式尾端後的鄰接指令，判讀以地址及 ret 為界。沒有執行原版 BIN。

原版加入入口觀察到 256 筆配置上限；現代模型採 4096 筆有界、依 ID upsert 儲存。這是明確的現代儲存策略，不能宣稱完全模擬原版容量或重複物件配置行為。

## 視窗與驗證邊界

背包面板保留同 owner 的選取 ID 與捲動位置，owner 失效或結算時關閉；開啟時抑制世界移動／攻擊／施法輸入，但仍處理死亡確認。面板只顯示文字與數量，控制字元只在呈現時清理，模型仍保留原始名稱。

伺服器名稱可能已含 `(數量)` 或 `$26` 等 token。清單另列權威 count，所以呈現時會去掉名稱結尾與權威 count 完全相同的 ` (數量)`（只在 count 大於 1 時；T035，`native_inventory::display_name`），不再重複顯示；數字不同（充能次數、尚未隨數量更新的名稱）時照伺服器字串顯示。模型保存的伺服器名稱不變。原版 token 展開、格式顏色、物品圖示、使用／裝備／丟棄與耗材技能是後續工作。

七項 Rust 測試涵蓋八種更新、全部向量逐 byte 截斷及尾隨拒絕、255 筆批次、欄位上限、原子失敗、角色／連線隔離與同角色恢復。正式 TCP 兩案使用實際 ClientSession 與記憶體 GameStateStore，放入 40318×4、40010×2 後逐欄核對實際 RuntimeInventory；不碰 SQL／玩家資料、不注入背包回覆、不啟用預設關閉的 native tournament economy 分支。

只有首次 bootstrap 與同角色重選通過此次真正 TCP。不同角色切換／拒絕回退／遲到更新是模型驗證；八種 delta 的 oracle 向量通過不等於所有更新都已由玩家操作端到端觸發。GUI 和音效裝置尚未驗收。最終通過數字與 EXE 雜湊見 `驗證紀錄/current/native-inventory/summary.json` 與 `驗證紀錄/20261012/current-build-receipt.json`；完整遊戲目標仍進行中。
