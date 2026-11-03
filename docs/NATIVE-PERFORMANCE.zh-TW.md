# 原生世界地形快取與載入量測

2026-10-14現況：同一份972項固定輸入，Debug D3B6A244與Release 389CBE4E的正式十步導航場景最終像素雜湊一致。逐幀中位數分別90.536／22.582ms，p95 114.049／36.483ms；本機同時有其他測試執行，這是限定場景量測，不能作隔離最佳化或完整GUI幀率承諾。證據為`驗證紀錄/current/native-world-integration/current-debug-release-performance.json`。以下保留較早實作歷史，其JAR、版本及待辦不代表當前基準。

本輪先同步 E35CB91C43A18A690CC304DF8E5A9B2BA15014822261651616AE9420FA8CC5A2 正式核心。舊 205DC1BA… JAR 保留在 target/current/server-reference/formal-server-205DC1BA.jar；現行 formal-server.jar 與 current/reference-snapshot.json 共同固定新基準。八份 oracle 由新 JAR 重新產生，除來源 hash 外與前版完全相同。契約未變不等於完整玩法驗收。

## 地形快取

native_terrain_cache 保留一個鏡頭四周各多 128 像素的静態地形畫面。800×600 視窗最多保留 1056×856×4 = 3,615,744 bytes；模組硬限制視窗 1024×1024、快取 1280×1280。鏡頭在範圍內時逐列複製準確像素，不重新混色、不保留角色；超出邊界先完整繪製新區域，成功且未取消才替換。重建失敗保留上一個有效區域。Model 仍隨連線／世界世代與地圖取消重建。

角色的待機／走路、方向、圖像與鏡頭插值仍每幀更新；此快取不實作遮擋、陰影或動態地形事件。將來接入屋頂／地圖改變時必須明確失效快取，不能把動態結果當作靜態保存。

Canvas.crop 以 checked 範圍和有界新畫布逐列複製。測試包含來源 stride、最後一列／一像素、整數越界、鏡頭邊界命中、跨界重建、清除、取消和解碼失敗。既有正式世界 18 張時序圖用作真實 LTmap／半透明地形／角色合成的像素回歸。

## 量測

Model.load_profile_us 分別記錄 geometry、archive pins、static assets、scene、catalog 與 Sprite index 開啟耗時；完整來源／字節核對仍保留。每幀 render_profile_us 記錄总耗時、地形耗時、快取建立／命中次數及 bytes。

room-world-route 保留 navigation_render_samples（最多 2048 筆）及逐步時間；實際載入耗時 navigation_load_ms 與斷線後另載入的圖像 Model 分開解讀。這些是 Debug 執行的局部量測，不能直接當作 Release、完整 GUI 或長時間效能驗收。

## 開發版 SHA 最佳化

快取版本量到 archive pins 12.17 秒、static assets 12.38 秒，而 catalog 0.68 秒；其中完整 archive 在建立 pin 與重新開啟驗證時各計算一次 SHA。保留兩次核對與唯讀 handle 邊界，在 Cargo.toml 僅設定 profile.dev.package.sha2 opt-level=3；不移除每幀摘要，也不更動演算法／依賴版本。引擎程式仍保留 Debug 與邊界檢查。新核心的 24 組 cipher 向量另由 oracle 產生并對照歷史基準，相等而不改寫其原始來源紀錄。

第一輪 cache-only EXE 的 28 個新核心 TCP 情境已完成，摘要保存在 new-core-cache-only-revalidation.json；後续最終收據須以最佳化後同一 EXE 再驗證。

## 本機 Debug 實測

同一條 10 步牆邊往返，cache-only 階段載入 26,742ms、逐幀中位數 140.986ms；加入 sha2 最佳化後載入 3,110ms、逐幀中位數 42.056ms、步距 721–762ms。141 次渲染只建立一次 3,615,744-byte 地形區域；命中地形中位數 6.552ms、RGBA 摘要 6.887ms、BGRA 轉換 24.289ms。首次地形建立仍需約 434ms，這及 BGRA／大型場地、Release 與完整 GUI 仍需後續驗收。

斷線後圖像驗證另載入的 Model 用 2,868,953μs：archive pins 819,594、static assets 1,017,300、catalog 725,825、Sprite index 247,324、geometry 42,840、scene 16,058。這些為本機測量，不承諾其他硬體或冷啟動均相同。

最終 61 項 Rust、28 種 TCP、18 張動畫及往返終點共 19 張像素比對全部通過。Release 只做編譯檢查；Debug／Release 警告仍 281／282。schema11 收據固定 730 份來源，完整目標持續進行。

重跑順序：先以根入口 -BuildOnly 建置，再執行 verify-native-navigation.ps1 -SkipBuild、verify-native-animation.ps1 -SkipBuild，最後 verify-native-performance.ps1 讀取同一 EXE 的最新收據。後者只核對結果，不自行啟動測試。完整 28 情境還須執行 room／lobby／game／login／handshake 驗證工具。
