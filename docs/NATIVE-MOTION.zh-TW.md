# 原生世界更新與移動子集

同一原生登入視窗的世界畫面已接入移動、方向及物件刪除。方向鍵按一次移動一格（螢幕上下左右）；數字鍵盤 8／9／6／3／2／1／4／7 對應原生八方向。忽略按住鍵盤的自動重複，移動必須已有本地角色與有效 HP。後續已接入待機／走路動畫及顯示插值，見 [原生動畫驗證](NATIVE-ANIMATION.zh-TW.md)；滑鼠尋路或完整戰鬥控制仍未完成。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-motion.ps1
```

## 正式協定與位置語意

正式 JAR 的 `Protocol.moveRequest` 定義 C_MOVE=0x0A，body 為 opcode、u16 起點 X、u16 起點 Y、方向 byte，共 6 bytes。送的是目前起點；正式解析器依方向推導下一格。所有 8 個方向以現行 JAR oracle 核對。

Tournament 成功移動會更新伺服器位置並廣播給其他玩家，不會回送自己的移動 ACK。客戶端在有界發送佇列接受指令後預測一步，標記 `position_predicted=true`；不可把沒有拒絕視為伺服器確認。S_MOVE_OBJECT（0x12）依 u32 物件 ID 覆寫位置／方向並清除預測標記。自己的這類封包計入 `local_corrections`；未知 ID 不新增幽靈物件，而是記錄 orphan_updates。

S_CHANGE_DIRECTION（0x1C）更新既有物件方向；S_REMOVE_OBJECT（0x15）依 ID 移除。截斷、尾端多餘 bytes 與非法方向先拒絕，不能部分改寫世界。S_BLINK（0x63）移動鎖由正式 opcode-only C_BLINK（0x5A）解鎖。自動回覆同樣走現有單一 TCP owner，沒有第二條 socket。

後續已處理正式七組 S_PARALYSE 限制／解除；目前尚未完整處理已在途移動、特殊場地等無座標回覆的拒絕條件，也沒有本地碰撞預判。這些情況可能留下暫時預測偏差，仍列為玩法缺口；移動時序、狀態限制和動畫必須一起補齊。

## 持續畫面與資源生命週期

`native_world_scene::Model` 一次載入唯讀地圖、catalog 與 archive handle，依世界 visual_revision 重繪並重用已解碼 SPR（總快取上限 64 MiB）。鏡頭持續跟隨本地物件，伺服器修正也會觸發重繪。這仍是 CPU／800×600 參考合成器，完整地形／角色遮擋尚未完成。

`native_world_view::Presentation` 同時只允許一個背景工作，使用連線 epoch、world generation 及 map ID 過濾完成結果。切換或取消先清除舊畫面，舊工作即使已完成也不能發布到新場景；回收後才開新工作。同一場景有更晚的修訂時接續重繪。收到不同本地物件身份也會更新畫面。此機制有世代回歸測試，完整 GUI 跨場景操作仍未驗收。

## 隔離驗證範圍

`NativeMotionOracle` 使用固定正式 JAR 的 Writer／移動解析器、方向／刪除／移動鎖 builder。Rust 測試涵蓋八方向、u16 邊界、死亡禁止移動、預測後修正、刪除、未知 ID、截斷／尾端及世代過期。

`room-world-move` 從真正 LOGIN／GAME ClientSession 登入並開始 map564，送出一步 C_MOVE。驗證以伺服器 session 最終座標為依據；客戶端快照仍保留預測標記。`room-world-correct` 是僅 loopback 診斷路徑：刻意發出過期起點，同時建立本地預測，要求真正伺服器回傳原座標與移動鎖，並確認 C_BLINK 已使伺服器解除鎖。這個錯誤注入不在 GUI 操作路徑。

每個情境在同一份 Model 上產生移動前後 PNG，驗證 own actor 持續位於 (400,300)、鏡頭／方向更新、視覺修訂增加。後續 gfx1544 已按單圖 type14 看板條件呈現；首輪缺圖及靜態首幀結果保留於 schema8 歷史收據。所有 server 配置、MemoryAccountRepository、日誌和圖像均在重建版內，未接 SQL、玩家實表或穩定服務。

上一輪 schema8 的 49 項相關 Rust 測試、27 種 TCP 情境、Debug x64 建置與 Release 編譯檢查通過；Release 未做執行驗收。移動後 RGBA SHA256 `d5b1e507d95caea5f1c93f480477d30d30b7b6ef15ad2d5a95d3b5e812264fb6`，修正後恢復首幀基準 `b71fb1cd4f1c46da545378ea69db0ad9eb3fd42a9a1a54d402130b5f00c62ae4`。

現行執行結果與 EXE／JAR 雜湊見 `驗證紀錄/current/native-motion/receipt.json`，完整建置與回歸索引見 `驗證紀錄/20261012/current-build-receipt.json`。PNG 使用 GUI 共用 renderer，但不是 GUI 全流程驗收。完整可玩目標仍未完成。

後續版本已接入共同速度倍率、按住輸入及 map564 原生地形檢查，實測與目前限制見 [原生速度與通行驗證](NATIVE-NAVIGATION.zh-TW.md)。本頁既有測試描述保留該階段範圍。
