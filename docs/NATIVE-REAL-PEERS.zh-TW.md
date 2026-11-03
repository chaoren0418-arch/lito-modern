# 真實雙客戶端與房間換隊

此工作對應重寫目標 revision 2 的 G02。BBC59308 Debug 已通過兩名實際連線玩家完成兩場、互換勝敗、死亡確認、結算、返回及同 GAME 連線重入。重跑發現的同時離房競態已修復並通過封包重播及雙場重驗，G02 限定範圍完成。完整技能、GUI、Release 及其餘整體目標仍待驗收。

## 現行契約

2026-10-13 檢查期間，正式來源由 EBF5EE66 變為 BBC59308C43DF89EE5029D3B3C802BE8AD55FEE65C41FACA99A76912DD5DCE43。新舊候選分別保存在 `LiTo381-Modern/target/current/server-reference/candidate-BBC59308.jar`／`candidate-EBF5EE66.jar`；20 組舊正式向量重新產生且除 JAR 身份外相同，另核對 1 組新增換隊向量，現行固定副本已升為 BBC59308。schema38 收據仍保留為 EBF5EE66 歷史驗證。完整變更列表與方法差異見 `驗證紀錄/current/native-real-peers/formal-drift.json`、`method-diff-BBC59308.json`。新版變更包含戰鬥結算及斷線處理，仍需其餘回歸。

`NativePeerHarness` 使用兩個記憶體帳號的正式角色 TestKnight（ID 1）及 TestMage（ID 3），兩組 LOGIN／GAME ClientSession 共用同一個 GameHub。反射僅觀察階段、座標與 HP／MP；未修改 HP、傷害、勝敗或注入世界角色。記憶體資料及所有 Path 限於本重建版，SQL 與持久化關閉。

兩個 `LiToModern.exe` 透過同一個 GUI 使用的 LoginController 和世界渲染器實際傳送封包，分段 read=2／write=3。fixture 檔案只協調畫面已載入與已返回大廳，不能產生遊戲權威結果。影像直接讀取 `0-客戶端` 原始資源。

## 已發現的缺口

首次雙客戶端情境 `20261013-055206-695` 成功登入、選角、入房、開戰與接近對手；正式核心記錄 `rejected friendly/non-PvP attack`。原因是既有控制器缺少房間換隊，雙方均為 force 0。此失敗保留，沒有重試／修改 HP 冒充成功。

補入正式 C_D3（opcode + force 0／1）及 S_D4（有界 MS950 成員名稱 + force）；換隊等待期間禁止再次準備／離房送出，8 秒期限沿用控制器錯誤處理。成員隊伍只隨權威回覆改變；正式拒絕時維持伺服器回覆的隊伍。原生視窗新增「切換隊伍」，須先取消準備。

`NativeForceOracle` 實際遍歷正式核心的 256 個 force 值，接受 0／1、拒絕其餘 254，並產生中文回覆向量。對應 Rust 測試涵蓋範圍、截斷、尾隨資料、空名與離線送出拒絕；GUI 實際點擊仍待 G08 驗收。

## 重跑

除了預設雙場近戰，`-ActionProfile heal|bolt|bolt-refused|wall|wall-refused|ranged|item-heal` 分別驗證實際玩家治療、兩次光箭、MP 不足、火牆及藥水。拒絕情境只在初始記憶體角色資料設定 MP 2；遠程只設定初始弓與 20 支箭；藥水只設定初始 40010 數量 2。玩法中不注入 HP、傷害或結果。雙方均持續讀取正式權威，兩端保存完成快照後才關閉連線。

遠程最初兩次失敗保留於 `20261013-070329-138` 與 `20261013-071059-953`。捕獲正式 S_ATTACK 顯示 accepted-hit=true、source-damage=1，目標 HP 曾降到 99 再自然回到 100；不是未命中或未扣彈藥。修正 probe 以整段收到的 HP 觀察而非動畫結束後的单点判斷受傷。`20261013-071730-763` 已通過一箭、伺服器箭量 20→19、對方 HP 100→99→100、原始箭矢飛行／到達。光箭、治療、火牆／拒絕及藥水也已取得版本限定證據；schema39 的同一 AD02BC2A EXE 統一回歸已通過，索引為 `candidate-regression/20261013-072017-855/index.json`：204 Rust／53 Debug TCP，其中包含這八個雙玩家情境。後續 G04 的動態門／移動開發與此基準分開驗證。

```powershell
pwsh -NoProfile -File 維護工具/verify-native-real-peers.ps1 -ReferenceJar LiTo381-Modern/target/current/server-reference/candidate-BBC59308.jar
```

每次使用獨立時間戳目錄保存成功／失敗。Runner 先核對來源版本；版本漂移必須重新取證。程序均在本目錄隔離執行，最後回收自有程序並保存兩端輸出及正式狀態追蹤。

## 已通過的限定結果

`驗證紀錄/current/native-real-peers/20261013-055923-658` 保存首個雙場成功結果：GAME epoch 均維持 2，世界世代由 1 變 2；首場 A 送 61 次攻擊、B HP 100→0，第二場 B 送 27 次攻擊、A HP 50→0。88 次攻擊均收到權威回覆，共走 22 步，兩個敗方各只送一次死亡確認。四份比分一致（每場一殺一死），與伺服器觀察的 HP／MP、座標、結果階段交叉比對通過，四個 ClientSession 結束後房間／頻道成員及來源交接清空。

8 張開始／結果原資源 PNG 均保存 SHA256；已視覺檢查 A 首場起始及 B 首場死亡圖。這些是共用渲染器輸出，不能代替完整 GUI 操作或實際聲音裝置驗收。換隊新增與既有房間共 6 項 Rust 測試通過。候選證據見 `receipt.json` 及 `authority-comparison.json`，不與 schema38 的歷史計數合併。

## 後續重跑發現的返回競態

`20261013-062002-218` 的第二場已完成死亡與比分，但兩端同時離房時收到「自己的 S_D0 → 對方的 S_D0」；原控制器在自己的回覆後清空房間，將後到的已知成員通知誤判為 departure without room。捕獲原始封包與失敗追蹤均保留。修復僅記住自己離房時仍存在的成員與房間 ID，允許各自一筆遲到離房；未知姓名、錯誤房間、自己重複離房仍拒絕，新房間接受或連線重啟會清除記錄。

原始封包雙順序重播與 43 項相關 Rust 測試通過。修復後 `20261013-063239-564` 的雙玩家雙場及跨端權威比對通過，對應 Debug EXE `87AB55E3B0382588335E2193F84BD71281A317E149960743E3A096698C89C71F`。該次即時重跑未重現相同封包順序，所以競態順序本身以确定性重播驗證，不能宣稱即時競態已被再次觸發。

同一修復版另外通過 9 個自身／地面操作情境：盾牌、NPC 攻擊拒絕、丟棄及拒絕、材料地面技能三種條件、拾取及過期拾取。過期拾取為明示 fixture 移除地面物件；不等同雙玩家自然搶拾。結果在 `race-fix-evidence/index.json`。此前 201 Rust／34 TCP 結果對應 `3799A198…` EXE，與這個修復版分開記錄，不合併成單一新版總數。
