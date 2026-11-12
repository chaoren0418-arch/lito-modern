# 原生頻道、大廳清單與返回

> 協定欄位以 [PROTOCOL-CONTRACT.md](PROTOCOL-CONTRACT.md) 為準（框架、欄位、opcode、錯誤碼）。本文只記錄重建版 client 的處理方式、上限與驗證證據；以下表格的「欄位定義」欄直接連到契約章節，不再重複寫 wire 版型。

同一 `LiToModern.exe` 的原生視窗在選角後顯示四個正式頻道入口，接收進入結果、頻道人數與房間清單。按「返回選角」經正式來源刷新往返取得新角色清單；重選角色後可再次進入頻道。後續房間進出、成員與準備子集已接線，詳見 [原生房間驗證](NATIVE-ROOM.zh-TW.md)；戰鬥與返回場景尚待完成。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-lobby.ps1
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:<隔離LOGIN端口>
```

## 協定與狀態

| 封包／流程 | 欄位定義 | 重建版 client 處理 |
| --- | --- | --- |
| C_CA 0..3 | [§5.2](PROTOCOL-CONTRACT.md#52-頻道) | 四個固定 Tournament 頻道；名稱與順序以正式 JAR 的 GameHub.CHANNEL_NAMES oracle 核對，畫面編號為 1..4 |
| S_C9 | [§5.2](PROTOCOL-CONTRACT.md#52-頻道) | 沒收到成功回覆和對應的人數資料之前，不顯示「進入完成」 |
| S_CA | [§5.2](PROTOCOL-CONTRACT.md#52-頻道) | 人數與容量保留無號範圍 |
| C_CE → S_CC subtype 0 | [§5.3](PROTOCOL-CONTRACT.md#53-房間) | 房間依伺服器 ID 更新，最多 512 筆 |
| 房間更新結束 | — | C_CE 後接同頻道 C_CA；正式 session 對重複進入只回 S_CA，人數回覆在該次房間列之後。收到它才原子發布本次快照，不把空清單誤認為逾時 |
| 切換頻道 | — | 同一命令批次依序送 C_CB、C_CA 新頻道，等待新的 S_C9/S_CA 與房間快照 |
| 返回選角 | [§7](PROTOCOL-CONTRACT.md#7-退出與重新開始) | C_CB（若在頻道）、C_CA 100 → S_F5 → C_C8 當前來源 → S_03/S_CB；完整清單到齊才增加 directory generation、開放選角 |

命令佇列仍是四個有界項目，每項可原子接納一或兩個有序封包。先驗證整批再入列，避免只排入「離開」卻沒排入後續動作；實際 socket 寫入仍逐封包檢查取消、期限，完整寫出才更新 TX 計數器，部分寫入失敗就退役連線。排入並不表示伺服器已完成操作，畫面必須等相應回覆。

大廳房間快照的結束確認依賴此固定 JAR 的 session 處理次序，不是協定中的專用 terminator。自動進入大廳後查詢一次，也提供「更新房間」。未處理的 S_CC 子型別不當作已完成房間／玩家狀態；完整房間事件仍需下一階段接線。

## 實際證據

固定 JAR SHA256 `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542`。NativeLobbyOracle 直接呼叫其中 Protocol builder/parser，核對四頻道名稱、成功／拒絕、無號人數與中文房間列。共享 NativeGameHarness 延伸為五種真正 LOGIN／GAME ClientSession 模式：

- enter：完整登入／選角，進入頻道 0，核對人口與正式預設房間。
- switch：頻道 0 → 3，每次核對各頻道實際房間 ID。
- return：離開頻道，經 S_F5/C_C8 取得第二代角色清單；正式 pendingTransition 回到 NONE。
- reselect：返回後重選角色，再進入頻道 3，同一 GAME epoch 保持有效。
- restricted：只將記憶體 repository 回傳角色的 level 改為 14，真正 JAR 拒絕進入（result 10），沒有頻道／房間快照。

每例核對伺服器最終 phase、來源、角色選取消耗狀態、頻道及 disconnect 後四個頻道都沒有殘留成員；所有 Path 配置限本重建版、state persistence 關閉，無 SQL。客戶端 snapshot、stdout/stderr 與收據在 `驗證紀錄/current/native-lobby`。測試帳密仍由 stdin 輸入。

本次 37 項 Rust 測試、5 種大廳情境及原 13 種 TCP 回歸通過；Release 僅编譯檢查。原資源驗證保持既有像素雜湊。原生控件可見性、初始狀態與正常關閉另有 gui-receipt；GUI 登入／選角／切換／房間操作、版面、IME／多 DPI 未列為通過。

## 尚未完成的分支

本輪使用正式 JAR 的普通 Tournament 模式。`rental-source` 服務模式會將相同 C_CA 解作租裝組選擇，並使用 S_FB 目錄、租装村與跨服務交接，仍需獨立接入；不是四頻道成功即代表此分支完成。ARGO 99 封包可解析，但未開放 UI 入口。專用 channel ports 在現行 ServerConfig 被移除，四個普通頻道共用端口。

LTstring 已接上選角、進頻道、進房三種拒絕訊息（#329，對應表與其餘尚未接的列見 [NATIVE-MESSAGES](NATIVE-MESSAGES.zh-TW.md)）。原版 UI 版面與其餘 LTstring 文字、完整角色狀態、世界、移動、戰鬥、勝敗及返回尚待完成，不能以 LOBBY/CHANNEL/ROOM 伺服器 phase 宣告完整可玩。
