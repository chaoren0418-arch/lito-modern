# 原生 GAME 交接與角色清單

同一個 x64 `LiToModern.exe`、原生登入視窗與連線 owner 接上 LOGIN → GAME。選來源取得 S_F4 後，按「取得角色」關閉並回收 LOGIN worker；確認轉址與所選來源公告的 endpoint 一致，才以新 epoch 連線 GAME、重新交換版本並認證。2026-10-14 跨端點傳送新增後，帳密在 Zeroizing 記憶體中保留至當次連線使用期結束，以供正式 GAME 端重新認證；取消、失敗或開始新的 LOGIN 均清除，從不進入 Snapshot／收據／日誌。失敗只保留一次重試的登入端點、帳號、來源與角色關聯，仍需重新輸入密碼並認證。詳見[傳送交接](NATIVE-TELEPORT.zh-TW.md)。

GAME 端維持正式 GameHub 的來源交接權威。角色清單只在完整收到 count 指定的所有列後發布；空清單有獨立呈現，上限八筆，拒絕重複名字、額外列與欄位域錯誤。選角 C_C9 回送該列原始 MS950 名字位元組，不從畫面文字重建。S_C8 回覆成功後顯示已選角色；普通頻道、大廳清單與返回流程已由後續接入，見 [原生大廳驗證](NATIVE-LOBBY.zh-TW.md)。

## 正式封包

| 封包 | 處理 |
| --- | --- |
| S_NOTICE 0x4b | u16 訊息 ID，438 確認送 C_LOGIN_RESULT 0x61，其餘送 C_READ_NOTICE 0x46；等待使用者確認時暫停回覆期限 |
| S_E6 → C_E2 | opcode-only bootstrap readiness，一次就緒確認後才接受角色清單 |
| S_NEWS 0x38 | MS950 NUL 公告，保留換行；確認僅關閉本地公告。C_READ_NEWS 0x33 是再次索取公告，不是 ACK |
| S_03 | u8 count 0..8 |
| S_CB | 原始 name/clan 與 79-byte 數值尾段；職業、性別、lawful、HP/MP、AC、等級、六能力、時間、五種排名與異動、戰績、校驗值 |
| C_C9 | 目前完整清單中的角色原始名字 NUL |
| S_C8 | 0 接受、1 拒絕、7 參與限制、8 名字無效；拒絕返回清單，成功保留選取狀態 |

正式配置要求訊息 438 為通知序列最後一項。數字通知在目前視窗只顯示 ID；原 Text 訊息資源映射仍未完成。公告文字與一般名字的控制字元規則分開，公告可有 CR/LF/Tab，名字不可有控制字元。

角色 UI 名稱可能是伺服器產生的別名。例如本測試收到「重建帳號(測試乙)」，正式記憶體庫內部角色是 TestKnight；JAR 自行驗證兩者映射。新客戶端只回送收到的別名，沒有內建 TestKnight 或自選第一個來源替代權威。

## 重跑與證據邊界

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-game.ps1
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:<隔離LOGIN端口>
```

驗證使用固定正式 JAR `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542`。NativeGameOracle 直接呼叫 JAR Protocol；NativeGameHarness 由真正 LOGIN／GAME ClientSession 共用 SourceRoutedAccountRepository、記憶體帳號庫與 GameHub。所有配置 Path 必須在各模式自身 current 目錄下，state persistence 關閉，沒有 Main 或 SQL。

四種情境是選角、兩則通知加中文多行公告、空角色來源、未授權來源。最後一種讓第二來源的記憶體密碼不同，正式 LOGIN 不接受該來源選擇，客戶端期限到後退出，GAME 保持未連線。空清單以記憶體 repository 的 characters 方法回傳空列表，其餘方法仍委派給正式記憶體實作。

客戶端與伺服器各自提供狀態證據：新 epoch、完整清單、選角成功碼、內部角色／顯示別名、來源、交接消耗及 worker/session 回收。原始輸出和最後收據在 `驗證紀錄/current/native-game`。CLI 測試帳密只由 stdin 輸入，不放命令列；測試只使用 ephemeral loopback 端口。

收送事件佇列仍限制八筆；滿時 worker 保留單一待交付事件並等待消費端，不丟封包。等待過程檢查取消，UI 每次有界排空，支援角色列合包突發；32-packet 測試核對順序、計數器及滿佇列取消回收。

完整 GUI 登入／選角、通知原文資源、IME／多 DPI、房間成員、進入世界、戰鬥與返回場景仍待驗收。普通頻道返回子集另見 NATIVE-LOBBY.zh-TW.md。自動確認通知僅由 loopback 測試 probe 執行，產品視窗保留使用者確認。現階段不代表可玩客戶端已完成。
