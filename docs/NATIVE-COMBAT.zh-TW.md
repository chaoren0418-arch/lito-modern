# 原生近身戰鬥、動作事件及返回

現行原生 x64 控制器接入普通 Tournament 的近身攻擊、伺服器傷害／死亡、結算與返回。新增的自己死亡確認、自然敗北、未回覆重試及可捲動結果列表見 [死亡確認與結算明細](NATIVE-OUTCOMES.zh-TW.md)。完整可玩客戶端尚未完成；本文件只描述下列明確子集。當前執行證據以 `驗證紀錄/current/native-combat/receipt.json` 和 `驗證紀錄/20261012/current-build-receipt.json` 為準。

## 封包與狀態

| 方向 | 已接欄位／用途 |
| --- | --- |
| C17 | 目標 u32 ID、u16 X／Y；9 bytes |
| S23 | 動作、攻擊者／目標 ID、傷害標記、方向、尾端 effect；近身零 effect 分支 16 bytes |
| S20 | 物件 ID、動作；6 bytes，死亡 action8 |
| S68 | 物件 ID、0–100 HP 比例；6 bytes |
| S0D | 既有自己的目前／最大 HP 權威更新 |
| SD3 | 有號 i32 結果：-1 無勝方、0 第一隊、1 第二隊 |
| SDB | MS950 名字、points／kills／deaths／assists 四個 u32、兩個 force byte |
| CD9 → SF0 → CD0 → SD0 | 結算確認、普通分支 [F0,0]、離房及自己名字的離房回覆；再用既有房間快照返回大廳 |

SD3 將控制器置為 BattleResult，移動／攻擊停止，死亡畫面仍可播放。按返回後進 ResultReturning；只有完成真正離房及重新載入大廳，才回到 LobbyReady。SF0 本身不是完成返回的證據。

攻擊要求已啟動世界、自己有 HP、沒有已知移動限制、自己和目標存在且未死亡，以及 Chebyshev 距離不超過一格。弓／投擲 stance20／62 尚未接入。伺服器依自己的座標及裝備規則作最後判斷。發送後最多等待兩秒自己的攻擊回覆；沒有回覆只解除等待，不猜測命中。畫面動作未完成時，GUI 不排入下一次攻擊或走步。

S23 的 damage marker 只選取受傷動作，不能拿來扣血或推導實際 HP。自己的 HP 來自既有權威封包，其他物件血量比例來自 S68。Hit 影格事件也不改變世界血量。未知物件動作記為 orphan，不建立虛構物件；截斷、尾端多餘或無效基本欄位在修改世界前拒絕。S23 非零 effect 的遠程／特殊分支明確保留未處理。

## 原資源動作及聲音

動作仍由共享 Text/list.spr 的邏輯 gfx 經 reference 解析到 Sprite 圖檔。攻擊、受傷依接收順序播放一次，完成後回原待機；死亡優先清除待播動作，播放後保持最後影格。這是本客戶端目前的順序策略，尚未證明與舊引擎所有中斷／取消規則一致。相同 ID 的新 S_PUT_OBJECT 會退休先前死亡播放狀態。

每個物件最多保留 16 個動作 cue，播放佇列有相同上限；clip 快取最多 256 個。動作時鐘與既有 speed／brave 有理倍率共用，使用原 duration_ticks／FPS。新收到的動作從實際被 renderer 觀察到時開始，不捏造封包抵達時間。

影格事件只在進入該影格時產生一次；跳過多個影格也依序產生期間事件，包括零停留影格。音效按明確 Sound ID 讀 `Sound/<id>.wav`，經既有嚴格 PCM 解碼器準備，保留 bytes／SHA256。單檔上限 1 MiB，最多 128 個／16 MiB 快取，每幀最多 32 個可見物件聲音；額外事件在 receipt 記錄丟棄數。Hit 等其他事件只記錄，尚未映射額外特效。

UI 使用四個既有 WinMM PcmVoice；只在新發布畫面播放，WM_PAINT 不重播音效。連線／世界更換、取消或失去前景時停止；跨連線／世界世代的過期背景結果不能發布舊畫面及聲音。四個 voice 全忙時略過新聲音，裝置錯誤會停止新播放並顯示診斷。隔離 CLI 驗證只準備 PCM 並核對影格事件，沒有宣告實際音效裝置或完整 GUI 通過。

## 原生操作

`啟動現代客戶端.ps1 -NativeLoginEndpoint <登入 IP:port>` 的既有世界畫面支援點擊物件選取，HUD 顯示名稱及 HP 比例；Space 按下觸發一次近身攻擊，不採用作業系統重複按鍵。仍以方向鍵／數字鍵盤走近。點擊使用與畫面相同的 DPI／letterbox 逆轉換，選取依 renderer 逐像素 owner 平面（只有完全不透明的 sprite 像素算命中，透明與半透明像素不算），即 alpha 命中。T029-2（#168）起：沒有選取目標時 Space 送原生空揮 `17 00 00 00 00 <x> <y>`（目標 0、傷害 0，座標為面向前一格），本地先依正式 EmptyMelee 拒絕死亡、觀戰、隱身、麻痺、超重（weight240 ≥ 197）、弓／投擲 stance20／62 與不合法座標，並顯示原因；伺服器回送的目標 0 S23 視為空揮回覆。目標超出範圍時 Space，或再次點選已選取的目標，會以既有的檢查式移動逐步追擊（貪婪單步、最多 64 步，不做尋路），進入範圍後改為攻擊。選取目標腳下畫目標圈，顏色區分可攻擊／超出範圍／已死亡／不可攻擊（配色為本客戶端選擇，非還原原版）。

結算結果顯示勝方與已收到的結算人數，可按「返回大廳」。原生個人計分明細已由後續階段接入；滑鼠自動追擊、目標圈、拒絕提示（本地拒絕原因及伺服器 S_MESSAGE_CODE 110 超重）已在 #168 接入程式，但完整 GUI 與玩家實機驗收仍待完成。

## 隔離驗證方法

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-combat.ps1
```

NativeCombatOracle 從固定正式 JAR 直接產生正常攻擊／miss／傷害 clamp、死亡、HP 比例和三種結局封包；C17 交給正式 parser 檢查。計分列按已核對的 ClientSession writer 欄位建立，真正結算另由實際 ClientSession 測試。

兩個真實 LOGIN／GAME session 共用 SourceRoutedAccountRepository、MemoryAccountRepository、非持久化 GameStateStore 及正式 GameHub。沒有啟動正式 Main、SQL 或玩家資料庫，所有 Path 配置均檢查在重建版內。

- `room-world-attack`：等完整世界必要物件到齊，以原 LTmap 通行走 11 格到 ArenaBot 鄰格，再送一次 C17。驗證自己的攻擊、Bot 反擊、自己 HP 50→45、Bot HP 比例下降，動作結束回待機。
- `room-world-victory`：在隔離 GameHub 正式啟動戰鬥後呼叫其 damageBot 將 Bot 從 100 預設為 1 HP，再由同一原生路徑走 11 格及真正攻擊完成擊殺。測試 death8、0% HP、SD3、計分列、CD9／SF0／CD0／SD0 及 LobbyReady；不是完整 100 HP 對戰證明，沒有合成 S23／SD3 回覆代替伺服器。

CLI 使用同一 LoginController、World、Model／Animator 和音效準備流程。尋路是有界隔離 probe 專用（4,096 節點、64 步上限），不是玩家自動尋路功能。初版 9C97F868 快照兩案例共捕捉 51 幀、22 張 PNG；7 個實際 WAV 來源雜湊核對相等，11 個影格事件沒有重複。每個案例捕捉攻擊後約 2.5 秒的真實更新與原資源畫面；勝利前的世界與返回後控制器分別留存。正常斷線後核對無殘留房間、頻道及交接成員，worker 已回收。

首輪測試在自己的角色先到而 Bot 尚未收到時嘗試讀目標，明確失敗於 `fixture bot`。已改為等完整性 challenge 與 Bot 到齊後開始，後續同程式情境需重新通過。沒有將首輪失敗算作成功。

## 版本與剩餘範圍

本輪先在 E35CB91C 固定快照完成兩案例，摘要保留 `E35-baseline-combat.json`。外部正式 JAR 於開發中更新為 AE063385DCDF3898AEB003137F807E6C3365073AECE62B8531FB7E86886DCD06；新快照共 6 個 class 差異，集中於龍騎技能／圖示、持武器與變身狀態。9 個原生 oracle 契約及 24 組 cipher 向量內容相等，正式來源與執行範圍另記 reference-migration／formal-source-drift 收據。不能只因向量相等就聲稱完整新核心相容。

AE063385 完成 30 種 TCP 回歸後，又觀測 9C97F86821738CD4E039F20BF69CAE286BB1F095BDDA77E3305799A77F5C1A23；唯一差異是 ClientSession.useFoeSlayerSpell 的一處發送分支。現行 pin 已同步，9 個契約及 cipher 再核對相等；最終實際測試結果另外固定到此版本，不以 AE063385 成功結果冒充。

尚未完成：遠程投射物與 effect 分支、技能／裝備／消耗品、動態占位與門地形失效、完整動作取消規則／特殊速度、多人／持續完整對戰、完整計分表、租裝村及跨服务、GUI／DPI／IME／實際音效裝置、長時間效能與 Release 執行驗收。


初版近身驗收（schema12）：9C97F868 快照及最後來源一致，65 Rust／30 TCP 全通過；既有 18 張動畫加 1 張路線像素一致。最終 EXE 25,787,904 bytes，SHA256 D46126FEAF2903B4FA28258A1A3425AE5D9D81A973A24D068EF704AB846E015D，737 份來源固定在 schema12 收據。Release 僅編譯檢查。
