# 原生房間進出、成員與準備

同一 x64 `LiToModern.exe` 已從伺服器大廳快照選取房間 ID，送出加入要求，顯示正式房間設定與成員，並提供準備、取消準備和離開。離開後等待正式通知，再重新取得大廳快照；拒絕加入後可換頻道重試。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-room.ps1
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:<隔離LOGIN端口>
```

## 權威資料與邊界

| 封包 | 現行處理 |
| --- | --- |
| C_CF / S_CE | 以清單中的 u32 房間 ID 與最多 4-byte MS950 密碼要求加入。成功必須匹配要求的 ID；完整解析房名、屬性、容量、模式、地圖、摘要、房主、options 與 selections。拒絕保留正式結果碼 |
| S_CD | 逐筆正式成員資料，包含隊伍、職業、性別、賽事統計、準備／商店狀態、所屬、頻道、rank、grade 與獎章。收到自己選角的相同名稱後才開放房間動作 |
| C_D2 byte / S_D1 | 送出 0 或 1，等待正式成員狀態更新後解除待回覆狀態。接收端支援正式狀態 2（商店），本輪沒有商店操作 |
| C_D0 / S_D0 | 等待自己的正式離開通知，清除房間狀態，重新查詢大廳；其他成員通知按房間 ID 與名稱更新 |
| S_CF / S_D5 subtype 5 | 刪除大廳房間列／更新房主；未知子型別不宣告處理完成 |
| S_E1 / S_D2 | 顯示伺服器倒數；收到戰鬥開始只進入 BattlePending 並停用房間操作，世界場景尚未接入 |

房名支援最多 89 bytes，房間成員最多 256 筆。成員名稱是伺服器提供的 wire alias，不能推導 actor ID。S_CE 的 users summary 不是 S_CD 筆數或串流結束標記，成員持續增量更新。所有已支援封包均檢查截斷、多餘尾端與各自欄位界限。

房間密碼輸入送出後清除 EDIT 與 undo buffer。收包只驗證密碼編碼，不保存在房間狀態或 JSON；原始收包生命週期仍由網路層管理。加入／準備／離開沿用單一 GAME socket owner、epoch、有界發送與 8 秒回覆期限。

## 實際驗證

固定正式 JAR SHA256 `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542`。`NativeRoomOracle` 使用該 JAR 的 Protocol builders／Writer 產生完整設定、中文成員、全部正式加入拒絕碼、0/1/2 準備狀態、離開、房主及倒數向量。房間成功設定用 Writer 按正式 ClientSession 的完整欄位順序建立，沒有宣稱它是獨立高階 builder。

`NativeGameHarness` 建立同一 JAR 的 LOGIN／GAME ClientSession、共用 GameHub 與 MemoryAccountRepository。所有配置路徑位於重建版，關閉 state persistence，未使用 SQL 或玩家實表。六種實際 TCP 案例：

- room-join：加入頻道 0 的正式預設房間，核對房間 ID、自己與 ArenaBot。
- room-ready：準備再取消，核對自己狀態依序 0、1、0。
- room-leave：離開後回到 CHANNEL，大廳快照重新完整發布。
- room-rejoin：離開後再次加入，同一 GAME epoch 保持有效。
- room-reject：level 52 角色加入頻道 3 的預設房間，收到正式等級區間拒絕碼 10。
- room-retry：同一連線在拒絕後換至頻道 0，成功加入。

每例在連線結束後核對四頻道人口、房間成員和來源 handoff 都已回收。證據集中 `驗證紀錄/current/native-room`，重跑工具核對收據與實際 EXE/JAR 雜湊。

本輪相關 Rust 測試共 41 項；完整 TCP 回歸共 24 情境（房間 6、大廳 5、GAME 4、LOGIN 3、版本 6）。Release 只做編譯檢查，離線資源像素與動畫回歸另列收據。

## 尚未驗收

實際案例是一名新客戶端與正式內建 ArenaBot，未驗收兩名真人、多客戶端競態、私人房間密碼往返、踢人、建房或改房設定。中文密碼僅有正式編碼向量測試。GUI 連線到房間的全程操作、IME、跨 DPI 與視覺版面仍需驗收。

後續已把 S_WORLD、S_D2、物件／角色狀態與 map564 接到原資源首幀，見 [原生世界驗證](NATIVE-WORLD.zh-TW.md)。連續動畫、移動、戰鬥與返回仍待完成。普通 Tournament 房間成果不代表 rental-source 租裝村或 ARGO 已完成，也不代表完整遊戲可玩。
