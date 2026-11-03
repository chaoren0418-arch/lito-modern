# 原生帳密登入與來源選服

`LiToModern.exe --native-login <IP:port>` 使用原生 TCP 登入控制器，提供 Unicode 帳號、密碼與來源清單控件。唯一現行入口亦可呼叫：

```powershell
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:<隔離登入端口>
pwsh -NoProfile -File .\維護工具\verify-native-login.ps1
```

視窗在使用者按「連線」後才建立連線；帳密提交後清空密碼控件與 undo buffer，關閉時先取消並輪詢到 network worker 回收。原生 EDIT 控件支援 Windows Unicode 輸入；IME 組字、跨螢幕 DPI、键盤焦點與實際 UI 操作仍須獨立驗收。

本輪已實際確認視窗顯示、無障礙樹的中文控件／初始啟用狀態，以及 Alt+F4 正常關閉後程序結束。修正了隱藏命令列啟動器使第一次 ShowWindow 保持隱藏的問題。截图兩次失敗分別為 `FrameArrived timed out: timed out waiting on channel` 與 `window capture timed out: timed out waiting on channel`；按鈕點擊回報 `coordinate input geometry is unavailable`，後續以鍵盤正常關閉。完整 GUI 登入與版面未列為通過，證據見 gui-receipt.json。

LOGIN 階段到來源選擇與取得遊戲轉址為止；視窗後續已有「取得角色」與選角流程，詳見 [GAME 交接](NATIVE-GAME.zh-TW.md)。`RedirectReady` 尚不代表進入遊戲端或選角完成。LOGIN endpoint 的正式 ClientSession 此時進入 `LOBBY`，這是伺服器內部轉址階段，不等於新客戶端已呈現遊戲大廳。預設啟動仍是離線資源場景。

## 真正的正式登入程式驗證

固定正式 JAR SHA256：`205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542`。`NativeLoginOracle.java` 直接使用其中的 Protocol builder/parser 產生中文 MS950 對照；`NativeLoginHarness.java` 從該 JAR 建立真正的 ClientSession、SourceRoutedAccountRepository、MemoryAccountRepository、GameStateStore 與 GameHub。

測試端只接受 ephemeral loopback 連線，兩個來源「測試甲／測試乙」各自使用新建的記憶體測試帳號庫。配置、封包捕獲及結果位於重建版 `target/current/native-login`；配置所有 Path 欄位在建立 session 前逐一核對範圍。state persistence 關閉，沒有呼叫 SQL repository、正式服務 Main 或穩定專案腳本。這份獨立配置使用正式預設 bootstrap 次序；沒有啟用舊 BIN 限定的特殊 probe 分支。

三個真實 TCP 案例為正確密碼、錯誤密碼、更正密碼後在同一 session 重試。接受／重試案例選第二筆來源，核對 ClientSession 最終 phase=LOBBY、selectedClassicServerName=測試乙；拒絕案例仍為 VERSION_OK。每例確認新客戶端退出後 worker 靜止、正式 session 執行緒結束、沒有 state.properties 持久化。

原始客戶端 JSON、伺服器 stdout/stderr、oracle 與收據在 `驗證紀錄/current/native-login`。測試帳密只傳 stdin，沒有加入 command line；產品的 snapshot／錯誤訊息不含帳密。測試用 CaptureLog 是正式 JAR 的捕獲實作，僅限這些明示的隔離測試資料。

## 已實作的正式欄位

| 封包 | 目前實作 |
| --- | --- |
| C_LOGIN 0x01 | MS950 account NUL、password NUL、4-byte 本地 IPv4；DIRECT layout，不附舊啟動器 preamble／connector |
| S_LOGIN_CHECK 0x00 | 完整 `ccdsch`，成功與失敗均解析 value、serverName、flag、nativeWord；拒絕 selector 5/8/26/22 有明確狀態 |
| S_LITO_LOGIN_OK 0xe7 | opcode-only 成功回覆 |
| S_LITO_TIMELIMIT 0xe9 | u32 時間值；不改變認證狀態 |
| S_SERVER_LIST 0x3b | 1..127 筆來源，每筆 MS950 名稱（最多 63 bytes）／IPv4／u16 port |
| C_C8 0xc8 | 送出從當次伺服器清單選取的名稱；不假設清單序號等於來源 ID |
| S_LITO_VS_SERVER_IP 0xf4 | u8 index、IPv4、u16 port，保留轉址等待後續 GAME 流程 |

帳號／密碼採正式 server 的 UTF-16 長度限制 64／128，拒絕 NUL、控制字元、無法以 MS950 精確表示的內容及截斷。轉碼禁止 best-fit／replacement fallback，中文 fixture 必須和 Java MS950 完全相同。未使用舊固定角色或 READY／lease 才能登入的候選條件。

傳送佇列最多 4 個批次（各 1..2 個封包），事件佇列最多 8 筆；只有目前 epoch 且已完成版本交換的 owner 可接受傳送。先使用暫存 cipher 加密，完整寫出才更新 TX 計數器；部分寫入失敗就結束該連線。Zeroizing buffer 在排隊取消、傳送與編碼完成後清除；帳密另由控制器以 Zeroizing 保留到 GAME 重新認證送出，取消／失敗清除。認證與選服各有 8 秒回覆期限；失敗重试由正式完整 S_LOGIN_CHECK 驅動。

登入 bootstrap 中尚未消費的其他 opcode 保留為未處理封包，不會單憑它們取得認證或觸發轉址。後續 GAME 交接已補上來源拒絕、遊戲端重新認證、通知 ACK 與選角子集；完整 VS-server directory、頻道、大廳及玩家操作驗收仍待完成。
