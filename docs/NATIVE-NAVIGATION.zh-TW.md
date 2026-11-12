# 原生速度、連續輸入與地圖通行

原生世界視窗支援按住方向鍵／數字鍵盤連續移動，放開後完成目前一步便停止。每次只在最新本地角色的動畫已抵達、座標／motion revision／速度與畫面一致時送下一個 C_MOVE。作業系統鍵盤重複不會累積發送；快速按放仍保留一次步伐。失去前景、取消、世界／連線更換、權威座標修正或收到移動限制會清除待執行輸入。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-navigation.ps1
```

## 地圖來源與邊界

`native_navigation::Navigation` 從渲染器已核對的 LTmap S32 geometry 取 Layer3 的 Attribute1／Attribute2 bit0，建立每個 sector 4096 bytes 的不可變邊界表。這份表以 Arc 隨畫面提供給輸入，背景重繪期间可安全讀取；不用舊 Modern v1 的預計算 mask，也不把一般共享 map4 當成正式場地。

通行判斷對照固定正式 JAR 的 ArenaPassabilityCatalog／ArenaMapCatalog：來源與目的格必須在已載入 sector，目的格至少有一個可站立的正交方向；對角線須存在一條完整的兩段正交路徑。缺 sector 或不合法方向拒絕。原生 portal 不會因舊候選引擎規則而一律封閉。

map564 的原始 `7fff8000.s32` 共 4096 格，八方向合計 32,768 個判斷逐一對照正式 JAR oracle。被地圖邊界阻擋的步伐在送出前拒絕，不能更改本地預測；動態門、場地／任務邊界等仍由正式伺服器判斷並修正。此階段沒有自動尋路或完整動態物件占位。

## 速度封包與動畫

S_SPEED（0x29）及 S_EMOTION／brave（0x62）均為 `c d c h`，共八個 bytes，依物件 ID 更新模式與秒數。普通模式要求秒數為零；截斷、額外尾端、未知模式或零物件 ID 先拒絕。未知物件更新只計數，不建立物件。持續時間保留供後續 HUD，狀態解除依伺服器封包，沒有本地猜測到期。

本輪重新讀取並核對原版反組譯：4A7240 寫入 actor+1C／本地 831718；4AF9F0 寫入 actor+21／本地 831680；4E8750 的共同分支讀取這兩種值。速度 1 的時間倍率為 3/4，速度 2 為 2；brave1 為 3/4、brave3 為 7/8、brave4 僅在明列的走路動作為 3/4、brave5 為 3/8。brave6 在這個共同分支不變，不猜成其他倍率。兩組倍率組合後驅動同一份走路／待機虛擬時間。

抽樣間保留分數時間；速度變更在觀察到的邊界生效，保持目前步伐進度。連續輸入等待動畫抵達，因此也跟隨速度變化。這是新引擎的高精度時鐘，不聲稱逐項重現舊程式的整數毫秒截斷、+2 callback、非玩家 queue catch-up 或第三段特殊全域分支；這些仍待接入。攻擊專用間隔（#168）：`native_combat_ops::attack_interval_ms` 依正式 EmptyMelee.interval（引用 ActionIntervalCalculator 的 ATTACK 倍率：速度1 ×0.755、速度2 ×2、brave6 ×0.775、brave1／5 ×0.755、brave3 ×0.855×0.9，brave4 不加速攻擊），與上面走路／待機的動畫倍率不同；控制器的空揮節奏 gate 可接受此間隔，但 GUI 尚未取得自己攻擊動作的時長，目前傳 None（不啟用本地節奏，交由伺服器靜默丟棄並以兩秒逾時提示）；攻擊動畫播放速度未改。

反組譯全文／當前 LiTo.bin 與三份函數摘錄的 SHA256 見 `驗證紀錄/current/native-navigation/source-pins.json`。全程只讀取來源，沒有啟動、注入或操作原版程序。正式封包模式由 NativeNavigationOracle 直接使用固定 JAR 的 Protocol builders 產生。

## 驗證

`room-world-route` 使用真正 LOGIN／GAME ClientSession，從正常進場位置循已核對地形走到最近牆邊，嘗試一個本地封閉方向，再沿原路返回。每一步經過 GUI 共用的 Frame.movement_ready 與 LoginController.move_checked；牆邊嘗試必須不改變世界、預測數或發送計數，最後座標與伺服器 session 一致。路線與逐步時間記在 navigation_trace。

Rust 測試核對全 map564 邊界、三種速度與六種 brave producer 模式、格式原子性、多倍率、動作條件、步中換速不重啟，以及按鍵重複／快速按放／取消。完整 GUI 按鍵流程、真正施放速度道具／技能與長時間連續操作尚未人工驗收。

本輪仍不是完整可玩客戶端。下一步整合影格事件／音效／陰影、動態碰撞與點擊路徑，繼續正式戰鬥與返回。資料庫與穩定服務保持在驗證範圍外。

## 本輪實測紀錄

正式隔離路線為 0,0,0,1,1 → 封閉方向 1 → 5,5,4,4,4，合計 10 步；返回 (32729,32797)，與正式 session 相符，零座標修正。阻擋嘗試未改變世界、TX 或預測次數。實際渲染收到／呈現 12 個物件，往返後 PNG 已檢視。

首次在連線內載入 Model 花 26,701ms，原 25 秒 harness 期限不足；此較長案例改為 GAME 100 秒／客戶端流程 90 秒／外層程序 120 秒。成功路線繪製 31 次，步距 1068–1159ms。這是 Debug 正確性驗證，載入與 CPU 繪製仍明顯過慢；不能聲稱即時效能達標。完整 GUI 按住／放開流程尚未人工驗收。

同一最終 Debug x64 執行檔通過 58 項 Rust、28 種 TCP 情境與 18 張既有動畫像素回歸；Release 僅編譯檢查，726 份來源固定於 schema10。驗證對固定 205DC1BA… JAR；收尾時部署來源已由外部更新為 E35CB91C…，新核心尚未驗證。逐 entry 與方法差異、候選快照見 formal-source-drift.json，下一步優先重驗最新核心。

後續 E35CB91C 核心已由最終 EXE 完成既有 28 情境重驗，並改善載入與連續步距；最新量測及限制見 [原生效能驗證](NATIVE-PERFORMANCE.zh-TW.md)。前述較慢數字與漂移觀測保留為該階段記錄。
