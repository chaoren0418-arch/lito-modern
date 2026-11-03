# 原生遠程攻擊與普通箭矢

本階段接入現行正式 C_FAR_ATTACK（0x18）與 S_ATTACK（0x23）的普通遠程分支。正式 JAR 仍為 9C97F868；沒有使用歷史 Modern TLS／v37 協定。視窗沿用點選目標、按住 Space、放開停止；世界給出的 walk action 20／62 選用遠程請求，其餘仍走近身。

## 正式封包與權威

請求是 `18,d target,h x,h y`。回覆前 16 bytes 是 `23,c action,d attacker,d target,c damage,c heading,d projectile`；非零 projectile 的普通箭矢尾段為 `h graphics,7F,h fromX,h fromY,h toX,h toY,00,00`，總長 29 bytes。零 projectile 的既有 16-byte 近身分支保留。其他 effect discriminator 明確保持未處理，不能當作箭矢解析。

`NativeRangedOracle.java` 直接呼叫固定 JAR 的 Protocol 產生請求、命中、零傷害命中、未命中與 unsigned 邊界五個向量，另外保留 Protocol bytecode 摘錄。已驗證每個截斷、額外尾段、錯誤 heading／graphics／target-marker 組合均不能部分修改 World。

伺服器以 target 非零表示命中，與 damage marker 大小分開；因此零傷害命中仍在發射時取當時目標座標，飛行開始後不再追蹤。未命中保持封包終點。World 只記錄遠程動作、自己的攻擊回覆及有界投射物 cue，不根據 marker 或飛行結果修改 HP／比例。

客戶端目前以 19 格作遠程呈現範圍上限；精確武器射程、彈藥資格及扣除仍由正式伺服器裁定。尚未提供裝備／彈藥 UI 或宣稱此上限等於每一種武器的射程。沒有回覆時沿用兩秒逾時並停止自動連打。

## 原資源呈現

`native_projectiles` 接在共同 World Model 的角色動畫之後。只在該角色、該 action sequence 的第一個原 Hit 影格事件發射一次，使用既有 `ranged_effect` 的普通 #66／class3 資源實作。它會重新核對原 Text/list.spr 的三影格、八方向及 type／effect 屬性，載入八個原 Sprite bank；飛行用既有 30ms fixed-point 路徑，終止影格立即退休，不另造爆炸或聲音。

箭矢資源在弓 stance 或遠程 cue 出現時由背景工作預載，避免到了 Hit 事件才開始載入。World 最多保留 128 cue，呈現最多同時 64 個 flight；容量遺漏有計數，未知圖形有 omitted 記錄。物件移除或世界世代更換會清除 flight；一次 cue 不因重複取樣再次發射。既有角色音效仍由原影格事件準備。

本輪的完整效果支援僅普通箭矢 #66。throwing／特殊箭圖形、法術 effect、所有遮擋與原 BIN 回呼時序尚未完成；schema15 當時受傷動作仍隨權威 S23 接收排入，沒有宣稱與原客戶端的遠程受傷回呼時序完全相同。其他同 ID 復活／替換及完整長時間飛行生命週期仍需延伸驗證。

## 隔離驗證

`維護工具/verify-native-combat.ps1 -RangedOnly` 啟動真正正式 LOGIN／GAME ClientSession。測試用記憶體來源明確提供 item19/type4/range15 的弓、item40007 的普通箭 20 支；沒有修改 JAR 或注入攻擊／傷害回覆。角色正常由房間进入 map564，在出生點 32729,32797 射向 Bot 32741,32801，沒有走近或瞬移。測試檢查 C18 正式回覆、弓動作、原箭矢實際畫出及權威箭量 20→19。

離線資源測試另以正式向量驱動同一 Model，保存 0–1500ms 共 31 幀及飛行 PNG；這組清楚標示為向量／原資源測試，不當作真正 TCP 情境。測試也覆蓋目標在發射前移動、發射後固定終點、零傷害命中、一次發射、物件移除及新世界清除。

所有測試帳號、配置、輸出及狀態只在重建版，SQL 與 state persistence 關閉。完整 GUI／焦點／DPI、實際音效裝置、所有技能及可玩客戶端尚未驗收。

## 最終驗證收據

2026-10-12 最終 Debug x64 EXE 通過 76 項 Rust（16 組）、34 種隔離 TCP 與既有 19 張像素回歸；離線資源 RGBA 基準不變。Release 僅編譯檢查，Debug 281／Release 282 警告。六個戰鬥情境、七場次共 919 幀／165 PNG、274 個不重複影格事件及 9 WAV 來源；遠程案例有四個飛行取樣、8 個 #66 Sprite bank 與箭量 20→19，兩張飛行場景 PNG 已檢視。另有 31 幀／5 PNG 的向量與原資源測試，沒有混算成 TCP。

最終 EXE 25,970,688 bytes，SHA256 BA25CA97DFEFDC1CE183532326A0FD527A95CC7917717E9392850C6C70F31A0A。schema15 固定 742 份來源；前一階段為 native-continuous-build-receipt.json。正式 JAR 最後唯讀雜湊仍 9C97F868。主要證據在 `驗證紀錄/current/native-ranged`，真正 session 六案例在 `current/native-combat/receipt.json`，本階段收據已保存為 `驗證紀錄/20261012/native-ranged-build-receipt.json`。

下一步：核對原 BIN 遠程受傷回呼時序，再接技能請求、物件／座標 effect、MP／冷卻及拒絕回饋；擴充特殊投射物和裝備／彈藥 UI。完整可玩目標仍進行中。

後續普通箭矢抵達回呼與同 ID 世代防護見 [抵達與受傷呈現](NATIVE-IMPACT.zh-TW.md)；上述 76 Rust／schema15 數字保留為本階段歷史。現行證據以 current-build-receipt 為準。
