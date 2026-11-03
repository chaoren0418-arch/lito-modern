# 原生 TCP 基礎與隔離驗證

現行 `LiToModern.exe` 的 `--probe-native-loopback <IP:port>` 與原生連線控制器，使用正式部署 JAR 的 TCP 框架／加密／版本契約。預設玩家入口仍是離線共享資源場景。版本接受只表示握手成功；帳密／來源選擇已由後續原生登入控制器接入，見 NATIVE-LOGIN.zh-TW.md；GAME 交接、通知與選角子集見 NATIVE-GAME.zh-TW.md，完整玩法尚未接入。

## 來源與重跑

測試固定使用 SHA256 `205DC1BAF5598523E67899E6C92A4B9BB07494D42DFA995CD741E67330D9B542` 的正式 JAR 唯讀快照（6,234,416 bytes）。來源與快照收據在 `驗證紀錄/20261012/native-handshake/reference-snapshot.json`；快照在 `LiTo381-Modern/target/current/server-reference/formal-server.jar`，可重建，沒有複製遊戲資源。

```powershell
pwsh -NoProfile -File .\維護工具\verify-native-handshake.ps1
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -ProbeNativeEndpoint 127.0.0.1:<隔離測試端口>
```

驗證工具只啟動自己的 Java Protocol 測試端與 x64 客戶端，使用 ephemeral loopback port。Java 直接反射指定 JAR 的 `Protocol.readFrame/writeFrame/versionRequest/supportedVersion/versionAccepted/versionRejected` 與 `CipherState`；沒有建立 `Main`、`ClientSession`、repository 或 SQL 連線。它不是完整正式伺服器实例。JDK 17 路徑在工具內明示，須依本機安裝調整。

原始 stdout／stderr 與六種模式收據集中 `驗證紀錄/current/native-handshake`。正式來源可能被其他工作更新，這份固定快照的測試不會自動證明後續 JAR 相容。重建快照時必須重新核對來源 SHA、生成 oracle 與 Rust fixture；不能把旧向量的來源身份改成新版本。

## 封包與欄位

| 項目 | 目前核對的契約 |
| --- | --- |
| frame | u16 little endian 總長，含 2-byte header；有效總長 2..65535，body 最多 65533 |
| S_KEY | 未加密 body 恰好 5 bytes：opcode 1、u32 little endian seed |
| cipher | RX／TX 各自初始化 `(0x87546ca1 << 32) \| seed`；little endian 8-byte key 加 chained XOR；每完整 body 推進 body 長度，header 不計入 |
| C_VERSION | 10-byte body；opcode 0、u16 magic `0x8033`、u32 product 950、u8 fromCd（目前 0）、u16 build `0x14f8` |
| S_VERSION_CHECK | opcode `0x3a`；拒絕恰好 `[0x3a, 1]`；成功 selector 0，含 protocol minor、revision、4 個相容欄位、2 flags、country；revision ≥ `0x9e38` 另有 server type dword，總長 30，較舊 revision 總長 26 |

當前正式 JAR 的成功回覆有 revision `0x0734fd33`、protocol minor 13、country 3、server type `0x00077d82`。數值用固定寬度無號型別；網路欄位不因 x64 而放大。JSON 中 u64 計數器與 epoch 用十進位字串，避免超過 JavaScript 的精確整數範圍。

## 程序生命週期與覆蓋界線

`native_connection::Controller` 每次連線建立獨立 epoch、取消旗標與 8-event 有界佇列；單一 worker 保有 TCP、分段框架與雙向 cipher。UI 可有界輪詢，取消先發訊號、持續輪詢到 worker 結束再關閉；worker 未回收前拒絕重連。最後事件在 join 後再次排空，peer EOF 不會丟掉已接收的尾端封包。

預設連線上限 2 秒、握手 5 秒、每次 socket IO 50 ms、read/write chunk 最大 4096 bytes。握手持續核對取消與絕對期限，EOF 區分完整邊界、部分 header、部分 body。框架失效即退役；事件佇列滿時保留一筆待交付事件並暫停讀取，持續檢查取消，不能無界累積。正常 UI 必須取消／輪詢至靜止；Drop 的 join 是最後清理手段，連線階段仍可能等待 connect timeout。

Java 六種模式包括接受、拒絕、尾端多餘欄位、截斷 header、截斷 body、版本逾時。它逐 byte 寫入，客戶端限制 2-byte reads／3-byte writes；接收端實際仍可能合併 TCP bytes，因此 Rust 另以固定分段／合包測試驗證框架。接受案例比對客戶端 RX=Java TX、客戶端 TX=Java RX，再確認 socket／worker 取消後回收。

Rust 測試另外涵蓋最大 u16 框架、無號計數進位、取消停滯握手、重連新 epoch、版本後合包與 EOF。最大 body cipher 測試是 Rust 往返；24 個小封包向量才是獨立正式 JAR oracle。舊 Native Sender／Receiver 的 155-byte 候選限制保持原樣，新連線使用完整 u16 框架。

後續已新增正式 `ClientSession` 的 LOGIN endpoint 記憶體帳密／來源選服驗證，詳 NATIVE-LOGIN.zh-TW.md。GAME 端來源交接／認證、通知及選角子集另見 NATIVE-GAME.zh-TW.md；中文 IME、頻道／大廳、遊戲封包及完整玩家 UI 仍未驗收。
