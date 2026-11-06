# PROTOCOL-CONTRACT — lito-modern 客戶端 ↔ lito-launcher-server 0.6.71

更新：2026-11-04（Issue #4 / T004）；2026-11-06 加版本綁定、§8 對齊與 §9 換版重核清單（T025）。本檔取代「只靠 JAR SHA pin」的作法，讓 client 端可以**不看 JAR**就實作 LOGIN → GAME → 大廳 → 房間 → 戰鬥 → 退出。

## 0. 適用範圍與證據等級

| 項目 | 值 |
|---|---|
| 伺服器正式版 | 0.6.71；正式 JAR `LiTo381-SharedResources-candidate.jar` SHA256 `e1498871c7be6b28…d99d498`（`lito登入器/LiTo381-Launcher/docs/maintenance/release-0.6.71.json` 的 `build.candidate_sha256`） |
| 本文讀的來源 | `chaoren0418-arch/lito-launcher-server` main @ `2066774`，`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/**` |
| 重建版已有 oracle 的 JAR | `205DC1BA…B542`（2026-10-12 快照，見 `docs/NATIVE-PROTOCOL.zh-TW.md`／`NATIVE-LOGIN`／`NATIVE-GAME`） |
| **版本綁定（T025）** | **M2、M3 的所有客戶端開發與驗收，都以 E149（`e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498`）為唯一伺服器對象。** 伺服器端的隔離候選（例如 `50635f7e…`、`046c58c5…`）與 Maven 產物都不是驗收對象。伺服器換版時，依 §9 的清單重核，通過後才能改這一列 |
| E149 的血統 | 登入器 repo `docs/maintenance/release-0.6.71.json` 的 `build`：base `daedec7c…c807`（0.6.70）→ candidate `e1498871…d498`；變動 4 個 entry：`ClientSession`、`GmTravel`、`GmPolyMe`、`GmPolyMe$Forms`（.polyme）。0.6.70 與 0.6.71 的相容 DLL `litoddraw.dll` 都是 `e47bd6c6…` |

**重要邊界**：伺服器 CLAUDE.md 寫明正式 JAR **不是**由 `src/main` 直接建出來的，而是 `tools/*/build.py` 對 `base-0.6.70.jar`（`daedec7c…`）逐層 patch 出來。所以 `src/main` 和 `e1498871` 在個別類別上可能不同。0.6.71 的增量是 GM `.polyme`，從描述看不涉及握手和登入，但這只是推測。

本文每一列都會標出證據等級：

- **已驗證**：重建版已經有 Rust 測試或 Java oracle，直接呼叫 `205DC1BA` JAR 的 `Protocol`／`ClientSession` 核對過（證據見對應的 `docs/NATIVE-*.md`）。注意：這是對 `205DC1BA` 驗證，**還沒有**對 `e1498871` 重跑。
- **來源**：從上述 `src/main` 原始碼直接讀出，附 `檔案:行號`。還沒有用 `e1498871` 對照。
- **推測**：從命名、註解或呼叫關係推論出來，沒有直接讀到完整的 builder 或 parser。

行號是 `src/main` 檔案**含空行**的實際行號。這些來源檔每行之間都夾一行空行，閱讀時可以用 `grep -v '^\s*$'` 去掉。

## 1. 基本型別（`Protocol.Reader`／`Protocol.Writer`）

對應類別：`Protocol.Reader`（Protocol.java:1951）、`Protocol.Writer`（Protocol.java:2049）。

| 記號 | 意義 | 證據 |
|---|---|---|
| `c` | u8 | 來源 1967 |
| `h` | u16 little endian | 來源 1969 |
| `d` | u32 little endian（Java 以 long 持有無號值） | 來源 1971、2087 |
| `s` | MS950（`Charset.forName("MS950")`）位元組 + 單一 `0x00` 結尾 | 來源 1975、2089；已驗證（NATIVE-LOGIN 的中文 fixture） |
| `S` | UTF-16LE + `0x00 0x00` 結尾，必須 well-formed | 來源 1983、2101 |
| `raw` | 原樣位元組 | 來源 2121 |
| IPv4 欄位 | `d`；第一個 octet 放在最低位元組，所以 wire 上就是自然順序（`127.0.0.2` → `7F 00 00 02`） | 來源 ClientSession.java:88455 `ipv4LittleEndian` |

文字規則：`Writer.s` 本身**不做**任何檢查，只做 MS950 編碼並補一個 NUL（P:2089–2097）。各欄位的限制由個別 builder 或 parser 負責，**規則不一致**，client 必須依欄位分別處理：

| 欄位類別 | 伺服器規則 | 對應 |
|---|---|---|
| 帳號／密碼（C_LOGIN） | 1..64／1..128 UTF-16 字元；parser 只拆出 NUL 結尾字串，**不檢查**控制字元 | P:4945；CS:7383–7386 |
| 名稱類（伺服器名、來源名、角色名、房間成員名、血盟名） | 不可含 ISO 控制字元，必須能精確往返 MS950，另有各自的長度上限（例如角色名 ≤27、血盟名 ≤16、來源名 ≤63、S_LOGIN_CHECK serverName ≤40） | `containsControl` P:8765、`requireBoundedLegacyString` P:13169、P:5109–5133 |
| 公告與物品名（S_NEWS、S_ADD_INVENTORY 的 name） | `requireNativeString` 只檢查 null 和長度 ≤255 bytes；**允許** CR／LF／Tab 等控制字元 | P:13191–13195、P:5551–5556 |
| 告示板文字 | 只允許 CR／LF／Tab 三種控制字元 | `containsUnsafeBoardControl` P:8773 |

client 的編碼政策（已驗證，NATIVE-LOGIN／NATIVE-GAME）：送出的文字一律精確轉成 MS950，不使用 best-fit 或 replacement；無法表示的字直接拒絕送出。收到的文字依上表分類解碼，公告類必須保留換行，不可以把含換行的合法公告當成格式錯誤。

## 2. 傳輸、框架與加密

對應類別：`Protocol.readFrame`／`writeFrame`（Protocol.java:2129／2153）、`Protocol.CipherState`（1853）、`ClientSession.run`（ClientSession.java:1087 起）、`ServerConfig`（ServerConfig.java:272）。

| 項目 | 契約 | 證據 |
|---|---|---|
| 傳輸 | 純 TCP，`TcpNoDelay=true`，`SoTimeout=0`（伺服器端沒有讀取逾時） | 來源 ClientSession 1092–1093 |
| 框架 | `u16 LE total` + body；total **包含** 2-byte header；伺服器接受 `2 ≤ total ≤ max.frame` | 來源 2129–2145；已驗證 |
| `max.frame` | 預設 8192，可設定範圍 64..65535；超過上限時伺服器丟 IOException 並結束 session | 來源 ServerConfig 272 |
| 伺服器送出上限 | total ≤ 0xFFFF | 來源 2153–2161 |
| 第一包 | 伺服器 accept 後**立即**送 `S_KEY`，這包是**唯一的明文框架**：`c(0x01) d(seed)`，body 恰好 5 bytes；seed 由 `SecureRandom.nextInt()` 產生，以無號 u32 寫出 | 來源 ClientSession 1094–1096；已驗證 |
| 金鑰 | RX、TX 各自持有 `u64 state = (0x87546CA1 << 32) \| seed`；兩方向的初值相同，但各自推進 | 來源 1859–1861、33；已驗證 |
| 加密 | `key = state 的 8 個 little endian bytes`；`out[0]=p[0]^k[0]`；`out[i]=p[i]^out[i-1]^k[i&7]`（和前一個**密文**位元組做鏈式 XOR） | 來源 1867–1885；已驗證 |
| 解密 | `out[i]=c[i]^c[i-1]^k[i&7]`（i≥1），`out[0]=c[0]^k[0]` | 來源 1891–1915；已驗證 |
| 推進 | 每處理完一個 body，`state += body.length`（u64 環繞）；header 不加密、不計入；空 body 不推進 | 來源 1883、1913；已驗證 |
| 收包順序 | 伺服器每讀到一個 frame，就先 `decrypt` 再 `handle`；沒有 opcode 白名單時直接忽略未知 opcode | 來源 1100–1107 |

client 實作要點（已驗證）：先用暫存 cipher 加密，**完整寫出之後**才提交 TX state；部分寫入失敗就關閉連線。這樣兩端的 state 才不會錯位。

## 3. 端點與連線拓撲

對應類別：`Main`（Main.java:40–120）、`ClientSession.Endpoint`（ClientSession.java:157）、`UnifiedConfiguration.apply`（UnifiedConfiguration.java:70 起）。

| 項目 | 契約 | 證據 |
|---|---|---|
| LOGIN 端點 | `bind:port`（範例 `127.0.0.1:32000`） | 來源 Main 67；`config/server.properties.example` |
| GAME 端點 | `game.bind:port`（範例 `127.0.0.2:32000`）。`lito.channel.ports` 已退役，四個頻道共用同一個埠 | 來源 Main 69–74、ServerConfig 256–264 |
| 轉址 | LOGIN 階段結束時，伺服器以 `S_LITO_VS_SERVER_IP` 告知 GAME 端點；正式設定下 `advertised.host = game.bind`、`advertised.port = port` | 來源 UnifiedConfiguration 79 |
| 每個連線 | 各自有一個 `ClientSession`，各自送 `S_KEY`，各自有獨立 cipher；GAME 連線要**重新**做版本交換與 `C_LOGIN` | 來源 ClientSession 1094、7350；已驗證（NATIVE-GAME） |

Session 階段（`ClientSession.Phase`，ClientSession.java:158）：
`KEY_SENT → VERSION_OK → AUTHENTICATED → LOBBY → CHANNEL → ROOM → BATTLE → RESULT`，另有 `WORLD`（經典世界，重建版目前不用）。

## 4. 握手與登入流程

### 4.1 版本交換（兩個端點相同）

對應：`Protocol.versionRequest`／`supportedVersion`／`versionAccepted`／`versionRejected`（Protocol.java:4997／5021／5051／5085）、`ClientSession.version`（ClientSession.java:7327）。

| 方向 | 封包 | 欄位 | 證據 |
|---|---|---|---|
| C→S | `C_VERSION 0x00` | `c(0x00) h(magic=0x8033) d(product=950=0x3B6) c(fromCd=0) h(build=0x14F8)`，共 10 bytes，**不可**有尾端多餘位元組 | 來源 4997–5007、1727–1731；已驗證 |
| S→C 成功 | `S_VERSION_CHECK 0x3A` | `c(0x3A) c(0) c(minor=0x0D) d(0x0734FD33) d(0x0734FD30) d(0x77CF6EBA) d(0x0734FD31) d(0) c(0) c(0) c(country=3) d(serverType=0x00077D82)`，共 30 bytes | 來源 5051–5066；已驗證 |
| S→C 拒絕 | `S_VERSION_CHECK 0x3A` | `c(0x3A) c(1)`，phase 維持 `KEY_SENT` | 來源 5085–5087 |

- 只有 `phase == KEY_SENT` 時才處理 `C_VERSION`，其他階段**靜默忽略**（來源 7328）。
- 伺服器只核對 magic、product、build 三個欄位；`fromCd` 只記錄，不核對（來源 5021–5025）。
- `version.response.delay.ms` 可以讓成功回覆延遲送出；client 的握手期限必須能容忍這段延遲（重建版預設 5 秒）。

### 4.2 LOGIN 端點：帳密 → 來源清單 → 轉址

對應：`ClientSession.login`（7350）、`sendLoginBootstrap`（7769）、`sendLoginFailure`（7805）、`sendClassicServerList`（7814）、`sendLitoServerList`（7848）、`selectClassicServer`（7880）、`enterLobby`（7919）、`requestLitoCharacterInfo`（8004）。

```
C: C_VERSION                       S: S_VERSION_CHECK(0)
C: C_LOGIN(account, password, ip)
S: ── 失敗：S_LOGIN_CHECK(result≠0)，phase 回到 VERSION_OK，可在同一連線重試
S: ── 成功：phase=AUTHENTICATED，依 login.bootstrap.sequence 依序送（正式設定：
        S_LOGIN_CHECK(0) → S_SERVER_LIST → S_LITO_SERVER_LIST + S_LITO_VS_SERVER_STATUS，每步間隔 250 ms）
C: C_LITO_LIN_CHARACTER_INFO 0xC8(sourceName)   ← 互動選服
   或 C_SERVER_SELECT 0x35（opcode-only，綁定清單第 0 筆）
S: S_LITO_VS_SERVER_IP 0xF4(index=0, game ip, game port)，phase=LOBBY（伺服器內部的轉址階段）
C: 關閉 LOGIN 連線，改連 GAME 端點
```

| 方向 | 封包 | 欄位／規則 | 證據 |
|---|---|---|---|
| C→S | `C_LOGIN 0x01` | `c(0x01) s(account) s(password) d(clientIPv4)`；有尾端時視為 CONNECTOR 版型（`s(connector)`，最多 31 bytes），重建版**不送**這個尾端 | 來源 Protocol 4945–4983；已驗證 |
| 規則 | | 只有 `phase == VERSION_OK` 時才處理；帳號必須 1..64 字元、密碼必須 1..128 字元（UTF-16 長度），不符合時回 `ACCESS_FAILED` | 來源 ClientSession 7351、7383–7386 |
| S→C | `S_LOGIN_CHECK 0x00` | `c(0x00) c(result) d(value) s(serverName) c(flag) h(nativeWord)`；成功時 `value=3600`，失敗時 `value=0`，`flag` 和 `nativeWord` 都是 0；serverName 1..40 bytes MS950 | 來源 Protocol 5109–5133、ClientSession 7800–7812；已驗證 |
| S→C | `S_LITO_LOGIN_OK 0xE7` | opcode-only；只在 bootstrap 序列含 `login_ok` 時才送（`ServerConfig` 的預設值包含，**正式 UnifiedConfiguration 不含**） | 來源 13933、7776、UnifiedConfiguration 77 |
| S→C | `S_LITO_TIMELIMIT 0xE9` | `c d(seconds)`，bootstrap 送 86400；正式設定不送 | 來源 2223、7777 |
| S→C | `S_SERVER_LIST 0x3B` | `c(0x3B) c(count 1..127)` 後接 count 筆 `s(name ≤63 bytes) d(ipv4) h(port)`；正式設定下所有筆的位址都是 GAME 端點，名稱來自 DB `lito_source_server`（最多 8 服） | 來源 5175–5200、7814–7823；已驗證 |
| S→C | `S_LITO_SERVER_LIST 0xFB` | `c d(roomCount) d(serverCount)` 後接 room 筆 `s d(ip) h(port) d(roomId≠0) d(users)`，再接 server 筆 `s d(ip) h(port) d(users) d(capacity≠0)`；正式設定為 0 個 room、1 個 server（`server.name`，users 0、capacity 100） | 來源 16333–16370、7848–7872 |
| S→C | `S_LITO_VS_SERVER_STATUS 0xF3` | `c c(count ≤10)` 後接 count 個 `d(status)`；0=AVAILABLE，0xFFFFFFFF=UNAVAILABLE | 來源 16285–16296、3317 |
| C→S | `C_LITO_LIN_CHARACTER_INFO 0xC8` | `c(0xC8) s(sourceName ≤63 bytes)`；送出的名稱必須是**這次**收到的 `S_SERVER_LIST` 裡的名稱（不分大小寫比對），而且帳號對該來源有權限；未知或無權限的來源**靜默忽略**（沒有錯誤回覆，client 必須靠期限判斷） | 來源 13981–13990、ClientSession 8004–8053；已驗證 |
| C→S | `C_SERVER_SELECT 0x35` | opcode-only，不可有 body；綁定 `classicServerNames[0]` | 來源 5229–5233、7880–7917 |
| S→C | `S_LITO_VS_SERVER_IP 0xF4` | `c(0xF4) c(index 0..9) d(ipv4) h(port)`；伺服器同時在 GameHub 暫存 account→source 的交接資訊 | 來源 5245–5257、7919–7926；已驗證 |

### 4.3 GAME 端點：重新認證 → 通知 → 角色清單 → 選角

對應：`ClientSession.login`（GAME 分支，7430–7470、7482–7505）、`beginGameBootstrap`（7519）、`sendNextLoginNoticeOrBootstrap`（7536）、`acknowledgeReadNotice`／`acknowledgeLoginResult`、`sendGameBootstrap`（7686）、`completeGameBootstrap`（7697）、`sendCharacterDirectory`（7928）。

```
C: C_VERSION → S: S_VERSION_CHECK(0)
C: C_LOGIN（同一組帳密；伺服器取出 LOGIN 端暫存的 source 交接）
S: 依 login.notice.message.ids 逐則送 S_NOTICE(id)，每則都要等 ACK：
     id==438 → C_LOGIN_RESULT 0x61；其他 → C_READ_NOTICE 0x46
S: S_LITO_NEWS 0xE6（opcode-only 的就緒屏障）
C: C_LITO_READ_NEWS 0xE2（opcode-only，一次性）
S: phase=LOBBY；可能先送 S_NEWS 0x38(公告)；接著 S_NUM_CHARACTER 0x03(count) + count 筆 S_LITO_CHARACTER_INFO 0xCB
C: C_LITO_SELECT_CHAR 0xC9(name) → S: S_LITO_SELECT_CHAR_RESULT 0xC8(code)
```

| 封包 | 欄位／規則 | 證據 |
|---|---|---|
| GAME `C_LOGIN` | 欄位和 LOGIN 相同。伺服器在 GAME 端**會忽略一次**「帳號長度 65..192、密碼長度 1..512」的舊啟動器 preamble，這是給原版 LiTo.bin 用的；重建版不送 | 來源 7376–7381、7510–7517 |
| 認證 | 帳號已在其他 session 登入時回 `ACCOUNT_IN_USE 0x16` | 來源 7473–7480 |
| `S_NOTICE 0x4B` | `c h(messageId 0..0x7FFF)`；正式設定要求 438 是**最後**一則 | 來源 5527–5531；已驗證 |
| `C_READ_NOTICE 0x46`／`C_LOGIN_RESULT 0x61` | opcode-only；ACK 的種類不對、或沒有待確認的通知時，伺服器只記 log、**不前進** | 來源 5589、5599、7550–7572；已驗證 |
| `S_LITO_NEWS 0xE6` → `C_LITO_READ_NEWS 0xE2` | 兩者都是 opcode-only；ACK 之前送 `C_LITO_LIN_CHARACTER_INFO` 會被忽略 | 來源 13923、7686–7695、8012–8018；已驗證 |
| `S_NEWS 0x38` | `c s(message)`，只有 `login.common.news` 不是空字串時才送；`C_READ_NEWS 0x33` 是**重新索取**公告，不是 ACK | 來源 5551–5556；已驗證 |
| `S_NUM_CHARACTER 0x03` | `c c(count 0..8)` | 來源 11705；已驗證 |
| `S_LITO_CHARACTER_INFO 0xCB` | 名稱、血盟名，加上固定長度數值尾段（職業、性別、lawful、HP/MP、AC、等級、六能力、排名、戰績）；欄位順序見 §5.1 | 來源 7979–7995；已驗證（79-byte 尾段，NATIVE-GAME） |
| `C_LITO_SELECT_CHAR 0xC9` | `c s(name)`；必須**原樣**回送 `S_CB` 收到的名字位元組（伺服器可能給別名，例如「帳號(來源)」） | 來源 7733–7737；已驗證 |
| `S_LITO_SELECT_CHAR_RESULT 0xC8` | `c c(code)`：0 接受、1 拒絕、7 參與限制、8 名字無效 | 已驗證（NATIVE-GAME）；來源見 §5.1 |

### 4.4 錯誤碼

| 封包 | 值 | 意義 | 伺服器之後的狀態 | 證據 |
|---|---|---|---|---|
| `S_VERSION_CHECK` | 0／1 | 接受／拒絕 | 拒絕時維持 KEY_SENT | 來源 1733–1735 |
| `S_LOGIN_CHECK` result | `0x00` ACCEPTED | 成功 | AUTHENTICATED | 來源 1577–1582 |
| | `0x05` SYSTEM_ERROR | 認證後端或角色查詢失敗 | 回到 VERSION_OK | 來源 7394、7470 |
| | `0x08` ACCESS_FAILED | 帳密錯誤、C_LOGIN 格式錯、長度不合 | 回到 VERSION_OK | 來源 1755、7357、7385、7398 |
| | `0x1A` ACCOUNT_BANNED | 帳號停權（DB 或 sidecar） | 回到 VERSION_OK | 來源 7398–7405 |
| | `0x16` ACCOUNT_IN_USE | 同一帳號在其他連線已登入 | 回到 VERSION_OK | 來源 7473–7480 |
| `S_NEW_ACCOUNT_CHECK 0x02` | `0x01`／`0x04`／`0x07` | 系統錯誤不轉場／成功返回登入／帳號已存在 | — | 來源 1625–1627（重建版不需要實作） |
| `S_LITO_SELECT_CHAR_RESULT` | 0／1／7／8 | 見 §4.3 | 拒絕時返回角色清單 | 已驗證 |

**靜默忽略**（沒有錯誤回覆，client 必須靠自己的期限判斷）：phase 不對的 `C_VERSION`／`C_LOGIN`、未知或無權限的 `C_C8` 來源、不符合待確認種類的通知 ACK、未知 opcode。重建版目前的期限：連線 2 秒、握手 5 秒、認證與選服回覆各 8 秒（已驗證，NATIVE-LOGIN）。

## 5. 大廳、頻道與房間

證據等級：本章全部是**來源**，例外會另外標示。`native_lobby`／`native_room` 的 205DC1BA 驗證範圍以 `docs/NATIVE-LOBBY.zh-TW.md`、`docs/NATIVE-ROOM.zh-TW.md` 為準。
分派方式：`ClientSession.handle`（CS:1518）先進 `ForcedCombat.dispatch`，再進 `handleBeforeForcedCombat` 的 `switch(opcode)`（CS:1539–1705）。**switch 本身不檢查 phase**，phase 由各 handler 自己檢查。格式錯的封包會丟 `IllegalArgumentException`，再交給 `rejectMalformedPacket`（CS:1749）；C9／CA／CC／CF 等 opcode 在這裡有固定的失敗回覆。

### 5.1 選角（補 §4.3）

| 封包 | 欄位 | 對應 | phase |
|---|---|---|---|
| `C_LITO_SELECT_CHAR 0xC9` | `c s(name ≤27 bytes)`，不可有尾端 | `Protocol.litoCharacterSelectRequest` P:14407；`ClientSession.characterSelect` CS:8239 | LOBBY |
| `S_LITO_SELECT_CHAR_RESULT 0xC8` | `c c(result)`；`LitoCharacterSelectResult` P:3355：0 ACCEPTED、1 SILENT_REJECTED、7 PARTICIPATION_RESTRICTED、8 INVALID_CHARACTER_NAME | `litoCharacterSelectResult` P:14449 | — |
| `S_LITO_CHARACTER_INFO 0xCB` | `c s(name≤27) s(clan≤16) c(type 0..6) c(sex) h(lawful) h(hp) h(mp) c(ac) c(level) c(str) c(dex) c(con) c(wis) c(cha) c(int) c(0) d(time) d(rank×5: overall/level/source/channel/race) d(disconnects) d(winRateDenominator) d(wins) d(losses) c(rankingSelector) d(rankDelta×5) c(check)`；`check = level^str^dex^con^wis^cha^int` | `litoCharacterDirectoryRow` P:11727（writer 11791–11831） | — |

### 5.2 頻道

| 封包 | 欄位 | 對應 | phase |
|---|---|---|---|
| `C_LITO_ENTER_CHANNEL 0xCA` | `c c(selection)`，body 恰好 2 bytes；selection 的值（`LitoChannelSelection` P:3679）：0..3 是頻道、99 是 ARGO、100 是返回選服（restart／source refresh） | `litoEnterChannelRequest` P:14039；`enterChannel` CS:9820 | 進入時要求 LOBBY → 轉成 CHANNEL（CS:9890／9922）；selection 100 只在 LOBBY 處理（CS:9835） |
| `S_LITO_ENTER_CHANNEL_RESULT 0xC9` | 成功：`c c(0) c(sel) d(0) h(0) d(0)`；失敗：`c c(code) c(sel)`，code 1 表示頻道未開放、10 表示等級限制（`LitoEnterChannelFailure` P:3787）；格式錯：`c c(1) c(0)` | P:14057／14077／14103 | — |
| `S_LITO_CHANNEL_INFO 0xCA` | `c c(sel) h(users) h(capacity)` | `litoChannelInfo` P:14117 | — |
| `C_LITO_LEAVE_CHANNEL 0xCB` | opcode-only | `leaveChannel` CS:10081 | CHANNEL → LOBBY（CS:10086／10102） |

### 5.3 房間

| 封包 | 欄位 | 對應 | phase |
|---|---|---|---|
| `C_LITO_ROOM_LIST 0xCE` | opcode-only | `sendRoomList` CS:10106 | CHANNEL／ROOM |
| `S_LITO_ROOM_INFO 0xCC` | 一個封包帶一筆子記錄：`c(0) d(id) s(name) c(attrs) h(users) h(cap) h(0) c(option0)` 新增；`c(1) d(id) s(name)` 改名；`c(2) d(id) c(attrs)`；`c(3) d(id) h(users)`；`c(4) d(id) h(cap)`；`c(5) d(id) h(0) c(option0)` | `NativeRoomDirectoryRow.send`（NativeRoomDirectoryRow.java:25–48） | — |
| `C_LITO_CREATE_ROOM 0xCC` | `c s(name) c(attrs) s(password) h(capacity 2..200)`，之後可選擇性接 `c(opt1..opt4) c(count≤8) c(selection)×count`；成功時伺服器內部合成一個 `C_LITO_JOIN` 並執行 | `litoRoomSettingsRequest` P:14175；`createRoom` CS:11461／11495 | CHANNEL |
| `C_LITO_JOIN 0xCF` | `c d(roomId) s(password)`，不可有尾端 | `joinRoom` CS:10218–10223 | CHANNEL → ROOM（CS:10235／10361） |
| `S_LITO_JOIN_RESULT 0xCE` | 接受：`c c(0) d(roomId) s(name) c(attrs) s(password) h(max(2,combatants)) h(capacity) c(option0) h(mapId) c(0)×5 s(owner) c(opt1..opt4) c(selCount) c(sel)×N`；拒絕：`c c(code)` | CS:10332–10345；`joinRejected` P:2295 | — |
| 加入失敗碼 | `LitoJoinFailure` P:3407：1 靜默清理、3 房間已滿、4 已開戰、5 房間已關閉、7 被踢、10 上位限制、11 密碼錯、12 戰鬥已結束、13 僅限隊伍房、14 不開放觀戰、20 職業限制、21 伺服器限制 | | |
| `C_LITO_ROOM_USER 0xCD` | `c d(roomId)` | `sendRoomUser` CS:10165 | CHANNEL |
| `S_LITO_ROOM_USER 0xCD` | `c d(roomId≠0) s(name 1..27) c(force 0/1) c(type 0..6) c(sex 0/1) d(winRateDen) d(wins) d(losses) c(ready) s(affiliation 0..16) c(0) s("Ready") d(disconnects) c(channelSel −1..4，有號) c(ready) d(channelRank) c(grade 0..14) c(certificate)×**12** c(badgeEdition)×**5** d(litoMaster 0/1)`。ready：0 未準備、1 已準備、2 商店清單。certificate 每格是 0，或 `generation<<4 \| material`（generation≥1，material 只能是 1 秘銀／2 金／4 銀）。badgeEdition 依序對應死鬥、榮譽、任務、計時、攻城五種模式的屆數，各 1 byte。litoMaster 目前只用到 bit0 | `litoRoomParticipant` P:2419–2533；`GameStateStore.TournamentAwards` 的固定長度 GSS:893–894，驗證在 919 行起，`validTournamentLegacyCertificate` GSS:5742 | — |
| `C_LITO_EXIT_ROOM 0xD0` | opcode-only | `exitRoom` CS:14568 | ROOM／BATTLE／RESULT → CHANNEL（CS:14627／14675）；在 BATTLE 中送出時會先結算（CS:14605） |
| `S_LITO_EXIT_ROOM 0xD0` | `c d(roomId) s(displayName)` | CS:14641 | — |
| `S_LITO_DELETE_ROOM 0xCF` | `c d(roomId)` | `broadcastRoomDeleted` CS:14784 | — |
| `S_LITO_GAME_INFO 0xD5`（房間資訊） | 共通開頭是 `c(0xD5) c(subtype) d(roomId≠0)`，後面依 subtype 而定；各 subtype 見下方 6 列。同一房間的狀態分散在 subtype 0（摘要）和 subtype 1（可編輯設定）兩筆記錄 | opcode P:701 | — |
| ↳ subtype 0 摘要 | `s(name 1..0x59 bytes) c(option0＝模式) h(mapId) c(0) s(owner 1..0x1B bytes) c(0)×4` | `litoRoomSummary` P:16953 | — |
| ↳ subtype 1 設定 | `s(name 1..0x59) s(password 0..4 bytes) c(attributes) c(capacity 2..200) c(option1) c(option2) c(option3) c(option4) c(count ≤8) c(selection≠0)×count`。builder 會驗證 eventFlags 0..7、eventType 0..2、winnerCount 1..0x7F、allowedClassMask 0..0x1F（這四個欄位加上 selections 全為 0／空時視為舊版，不檢查），實際寫出的則是 option1..4。**推測**：option1..4 依序對應這四個欄位；這層對應在 `GameHub.RoomSettings` 內，本文沒有逐一核對 | `litoRoomConfiguration` P:16981 | — |
| ↳ subtype 2 模式 | `c(mode)`，值域是 `BattleRules.Mode` 的 wire 值 0..6 | `litoRoomMode` P:17073 | — |
| ↳ subtype 3 地圖 | `h(mapId)`；`0xFFFF`、`0xFF9C` 是 client 端的哨兵值，必須原樣保留 | `litoRoomMap` P:17097 | — |
| ↳ subtype 4、6、7、8、9 | `c(value)`（摘要純量）。**伺服器目前沒有任何發送點**，原始碼註解說明 client 映像中這五個目的地沒有讀取者；client 收到時只需解析並忽略 | `litoRoomSummaryScalar` P:17127 | — |
| ↳ subtype 5 房主 | `s(owner 1..0x1B bytes)` | `litoRoomOwner` P:17159 | — |
| 發送時機（一般房間） | 依請求類型分開列，「單播」＝只回給送出者，「廣播」＝房間全體成員：<br>**A. 改房間設定 `C_LITO_EDIT_ROOM 0xDD`**（`editRoom` CS:16721）：容量不合法 → 單播 0＋1（CS:16732–16736）；`hub.updateSettings` 成功 → 廣播 0＋1（CS:16755–16757，`broadcastGameInfoSnapshot` CS:16468）；`updateSettings` 被拒 → 單播 0＋1（CS:16759–16760，`sendGameInfoSnapshot` CS:16455）。<br>**B. 換模式 `C_LITO_GAME_INFO` subtype 2**（`changeRoomMode` CS:16482）：目前模式不是戰鬥場景 → 單播 0＋1（CS:16486–16492）；`updateSettings` 被拒 → 單播 0＋1（CS:16503–16505）；成功 → 廣播 2＋3（CS:16509–16515），之後可能附帶 ArenaBot 的 `S_LITO_EXIT_ROOM`／`S_LITO_ROOM_USER`。<br>**C. 換地圖 `C_LITO_GAME_INFO` subtype 3**（`changeRoomMap` CS:16537）：地圖不屬於目前模式或等於 0xFFFF → 單播原本的 3（CS:16540–16546）；`hub.updateMap` 被拒（非房主、已開戰、地圖不合法，GameHub.java:9874–9878）→ **單播 0＋1**（CS:16549–16551）；成功 → 廣播 3（CS:16555–16558）。<br>**D. 房主轉移**（`broadcastRoomOwner` CS:14802，在斷線 CS:1251、離房 CS:14670、restart CS:15234 時觸發）→ 廣播 5。<br>client 送出 B 或 C 之後，要同時接受 0＋1 或 2／3 兩種回覆，不能只等 subtype 3 | 同左 | ROOM |
| 發送時機（租裝城鎮房間） | 一律**單播**回送出者，不經過 GameHub：<br>**換地圖 subtype 3**（`handleRentalTownGameInfo` CS:16423）：合法 → 更新後回 3；不合法 → 保留原值並回 3；兩者都在 CS:16435 送出並 return。<br>**換模式 subtype 2**（CS:16442–16443）→ 回 2＋3（`sendRentalTownGameInfoSnapshot` CS:16446，模式固定為 TOWN＝6）。<br>**改房間設定 `C_LITO_EDIT_ROOM`**（CS:16725–16730）→ 一律拒絕並回 2＋3 | 同左 | ROOM |
| `C_LITO_GAME_INFO 0xD1`（房主改模式／地圖） | `c(0xD1) c(2) c(direction)`：direction 1＝下一個、2＝上一個；或 `c(0xD1) c(3) h(mapId)`。其他 subtype 和長度都是格式錯誤 | `litoGameInfoRequest` P:14351；`LitoModeStepDirection` P:3847；`gameInfo` CS:16402（dispatch CS:1665） | ROOM，且必須在房間內；其他 phase 靜默忽略 |

## 6. 戰鬥

### 6.1 開戰與結束

```
ROOM: C_LITO_READY_GAME 0xD2 c(state)          ← 參加者切換準備狀態
      S_LITO_READY_GAME 0xD1 s(name) c(state)  ← 廣播
      C_LITO_READY_GAME 0xD2（opcode-only）    ← 房主開戰
      S_LITO_COUNTDOWN 0xE1 c(value)（推測：開戰前倒數）
      ── 一般對戰（startBattle CS:11968；BATTLE 在 CS:12021 設定）：
      S_WORLD 0x28 h(mapId) c(0)
      S_LITO_START_GAME 0xD2
      S_LITO_OBSERVER_BOX 0xE2 h(minX) h(minY) h(maxXExcl) h(maxYExcl)
      [S_LITO_TIMELIMIT 0xE9 d(seconds)]       ← 只有 mode ≠ TIME_ATTACK 才送
      S_TIME 0x21 d(gameSeconds)
      [S_ADD_INVENTORY_BATCH 0x7C …]           ← 只有背包需要重建時才送（見 §6.2）
      之後是 HP／狀態／物件等封包；phase=BATTLE
      ── 租裝城鎮（CS:10484–10513）：S_WORLD → S_START_GAME → S_OBSERVER_BOX → S_TIME → S_ADD_INVENTORY_BATCH（必送），不送 TIMELIMIT
BATTLE: …戰鬥…  → S_LITO_END_GAME 0xD3 d(winner) → RESULT
```

| 封包 | 欄位 | 對應 |
|---|---|---|
| `C_LITO_READY_GAME 0xD2` | 只有 opcode 表示房主開戰；`c c(state 0/1/2)` 表示切換準備狀態 | `litoReadyRequest` P:16643；`setReady` CS:11627（ROOM） |
| `S_LITO_READY_GAME 0xD1` | `c s(name) c(state)` | `readyState` P:16609 |
| 開戰序列 | 前兩包固定是 `S_WORLD c h(mapId) c(0)` → `S_LITO_START_GAME c`。S_WORLD 必須先送，因為 S_D2 會用它設定的地圖狀態（CS:12002–12005 的註解）。接著是 `S_LITO_OBSERVER_BOX`；`S_LITO_TIMELIMIT` **依模式**送出（TIME_ATTACK 不送，CS:12017）；然後是 `S_TIME`，以及條件式的背包重建。client **不可以**把 TIMELIMIT 當成必收封包，也不可以把 OBSERVER_BOX、S_TIME 當成異常 | builder `battleBootstrap` P:2209–2218；實際送出順序見 `startBattle` CS:12008–12034 |
| `S_LITO_OBSERVER_BOX 0xE2` | `c h(minX) h(minY) h(maxXExclusive) h(maxYExclusive)`，必須 `min < maxExclusive ≤ 0xFFFF`；內容是場地的可視範圍 | `observerBox` P:14785–14799 |
| `S_LITO_END_GAME 0xD3` | `c d(winner)`：0xFFFFFFFF 平手、0 是 force0、1 是 force1 | `battleResult` P:2255；`LitoBattleResult` P:1437 |
| `S_LITO_COUNTDOWN 0xE1` | `c c(value)` | `serverCountdown` CS:11690 |
| `C_LITO_QUIT_FIGHT 0xD9` → `S_LITO_QUIT_FIGHT 0xF0` | 請求是 opcode-only；回覆是 `c c(0)`；BATTLE 時結算、RESULT 時等待確認、ROOM 時回到房間 | `quitFight` CS:14891 |
| `S_LITO_CONFIRM_DEATH 0xE3` → `C_LITO_CONFIRM_DEATH 0xE0` | 兩者都是 opcode-only；只在 BATTLE 中、有待確認的死亡、不是觀戰者且 hp ≤ 0 時才接受 | CS:80145／80208；`confirmDeath` CS:26644／26701 |

### 6.2 戰鬥中的核心封包（BATTLE；WORLD 也共用）

| 封包 | 欄位 | 對應 |
|---|---|---|
| `C_MOVE 0x0A` | `c h(srcX) h(srcY) c(heading 0..7)`；送的是**起點**，伺服器依 heading 推進一格 | `moveRequest` P:8891；`move` CS:26832（要求 hp>0） |
| `C_CHANGE_DIRECTION 0x09` | `c [c(heading)]`；heading 可省略，省略時視為 0 | CS:32085；廣播 `S_CHANGE_DIRECTION 0x1C d(objId) c(heading)` |
| `C_ATTACK 0x17`／`C_FAR_ATTACK 0x18` | `c d(targetId≠0) h(x) h(y)` | `attackRequest` P:8849；CS:74608 |
| `C_USE_SPELL 0x14` | `c c(row) c(column 0..7)` 後接尾段。尾段形狀由**該技能**決定，伺服器依技能呼叫對應的 parser；長度不符時丟 `IllegalArgumentException`，交給 `rejectMalformedPacket` 處理。十一種形狀見下一列；各形狀欄位名取自 P:4059–4107 的 record | `spellSelector` P:8933；`useSpell` CS:32161 |
| ↳ 尾段形狀 | ① 無尾段：`selectorOnlySpellRequest` P:8965（71 個呼叫點）<br>② `d(clientTick)`：`selfSpellRequest` P:9003<br>③ `h(x) h(y)`：`groundSpellRequest` P:9043<br>④ `d(targetId≠0)`：`objectSpellRequest` P:9083（33 個呼叫點）<br>⑤ `h(mapId) h(x) h(y)`：`teleportSpellRequest` P:9125<br>⑥ `h(mapId) h(x) h(y)`：`massTeleportSpellRequest` P:9165<br>⑦ `s(targetName)`：`callClanSpellRequest` P:9205<br>⑧ `s(targetName)`：`runClanSpellRequest` P:9259<br>⑨ `d(selection≠0)`：`summonSpellRequest` P:9311<br>⑩ `d(targetId≠0) h(x) h(y)`：`attackSpellRequest` P:9347（44 個呼叫點）<br>⑪ `d(targetId≠0) h(x) h(y) s(text)`：`trueTargetRequest` P:9389–9405，只用於 skillId 113（True Target）；text 必須是 MS950 ≤0x34 bytes、可精確編碼、無控制字元，且不可有剩餘 bytes；BATTLE 與 WORLD 都接受（`useTrueTargetSpell` CS:44995–45004）。<br>以上 11 種就是 Protocol.java 中所有呼叫 `spellSelector(payload)` 的 parser（P:8967–9391，共 11 處）。<br>每個技能 (row,column) 用哪一種形狀，見 §6.3 對照表 | 呼叫點數以 `grep -c "Protocol.<parser>("` 計 |
| `C_USE_ITEM 0x1C` | `c d(invObjId≠0)` 後接依道具而定的尾段（預設是 `s(arg)`） | P:4529／4611；`useInventoryItem` CS:23071 |
| `C_SAY 0x13`／`C_CHAT 0x77` | `c c(type) s(msg 1..255 bytes)`；`C_CHAT` 的 type 必須是 12 | `chatRequest` P:5707；`say` CS:15860（ROOM／BATTLE／CHANNEL／WORLD） |
| `S_SAY 0x13` | `c c(type) d(objId) s(msg≤511) h(x) h(y)` | P:11335 |
| `S_PUT_OBJECT 0x0B` | `c h(x) h(y) d(objId) h(gfx) c(action) c(heading) c(light) c(speed) d(0) h(lawful) s(name) s(title) c(status) d(emblem) s(clan) s("") c(0) c(hpRatio) c(0) c(level) s("") h(0) c(0)`；`status&4` 為真時再接 `c(0xFF) d(0)`。寵物、NPC 等其他版型見 P:14975–16013 | `battleActorWithHeading` P:14895 |
| `S_MOVE_OBJECT 0x12` | `c d(objId) h(srcX) h(srcY) c(heading)`；client 從起點推進一格 | NativeMovement.java:5–12 |
| `S_REMOVE_OBJECT 0x15` | `c d(objId)` | P:12307 |
| `S_HIT_POINT 0x0D`／`S_MANA_POINT 0x4D` | schema 0：`c h(cur) h(max)`，body 5 bytes；schema ≥1：`c d(cur) d(max)`，body 9 bytes（`Writer.c` 的特例：第一個 byte 是這兩個 opcode 時，接下來兩個 `h` 改寫成 `d`）。schema 怎麼決定見下方「數值 schema」 | P:2057–2071；P:10691 |
| `S_STATUS 0x0C` | 依序為：`c(0x0C) d(objId) c(min(level,255)) d(exp u32)`；六能力，順序 **STR、INT、WIS、DEX、CON、CHA**（注意和 §5.1 角色清單的順序不同）；`cur HP, max HP, cur MP, max MP, AC`；`d(0) c(foodGauge) c(weightGauge) h(lawful，有號 i16)`；四屬性抗性，順序 **FIRE、WATER、WIND、EARTH**。各段寬度依 schema 而定：<br>schema 0：能力 `c`×6、HP/MP `h`×4（無號）、AC `c`（有號 i8）、抗性 `c`×4（有號 i8）→ body **37** bytes<br>schema 1：HP/MP/AC 改成 `d`×5（AC 為有號 i32，範圍 ±999999）→ **48**<br>schema 2、3：能力也改成 `d`×6 → **66**<br>schema 4：抗性也改成 `d`×4（有號 i32）→ **78**<br>food／weight gauge 是 0..29 的投影值，不是 DB 原值 | `characterStatus` P:10711–10917（能力順序 10857、gauge 10895） |
| 數值 schema | `NumericBreakthrough.schemaVersion()` **每次呼叫都重新讀取**：先讀系統屬性 `-Dlito.numeric.breakthrough`，沒有設定時才讀環境變數 `LITO_NUMERIC_BREAKTHROUGH`；沒有快取（NumericBreakthrough.java:26–27，已由 #13 以正式 JAR e1498871 的 javap 確認）。值 1..4 有效，其他值都視為 0。實務上這兩個值都在程序啟動時決定，執行途中不會變，但 client 不應假設 schema 一定不變。**wire 上沒有協商**。正式登入器預設 `NumericBreakthroughEnabled=true`，啟動本機伺服器時帶 schema **4**；伺服器若獨立啟動、沒設變數，就是 schema 0。client 必須依 body 長度判斷 schema：`S_STATUS` 37／48／66／78，`S_HIT_POINT` 5／9；長度不在這些值裡時視為協定錯誤 | `NumericBreakthrough` 第 6、20、25–27 行；登入器 `LauncherService.cs:18`、`LauncherSettings.cs:54`、`LocalServerService.cs:230` |
| `S_ATTACK 0x23` | 一般物理攻擊：`c c(action) d(attacker) d(target) c(hit?10:0) c(heading) d(0)`；其他變體（傷害、技能、遠程）的尾段**未讀** | P:9501；變體 P:9551–9787 |
| `S_EFFECT 0x37` | `c d(objId) h(gfx)` | P:16117 |
| `S_ADD_INVENTORY 0x16` | `c` 後接一筆 inventory record：`d(objId) c(useType) c(charge) h(icon) c(bless 0..3) d(count≥1) c(identified) s(name ≤255 bytes) c(detailLen) raw(detail)` | P:12039；`appendInventoryRecord` P:12741 |
| `S_ADD_INVENTORY_BATCH 0x7C` | `c c(count 0..255)` 後接 count 筆和上一列相同的 inventory record；超過 255 筆時，剩下的改用單筆 `S_ADD_INVENTORY` 補送 | `inventoryBatch` P:12053；`sendInventoryBootstrap` CS:9733–9761 |
| 背包初始化時機 | `sendInventoryBootstrap` 有一次性旗標：`inventoryBootstrapSent` 為真時直接返回（CS:9734）。**第一次發送在選角成功時**：S_C8(0) 之後送 S_ADD_INVENTORY_BATCH，再送 S_ADD_SPELL（CS:8886–8891；原註解說明大廳的背包和技能視窗會用到）。`startBattle` 也會呼叫一次（CS:12034），只有旗標被重設過才會真的再送；重設點是載入角色資料、商店買賣、點數同步、結算後重置經濟（CS:9723、20272、20413、20483）。重建前，伺服器會先對 client 已有的物件送 `S_REMOVE_INVENTORY`（CS:9735–9740）。所以 client 必須在 LOBBY 選角後就能接收背包批次，在 BATTLE 開場也要能接受重建 | 同左 |
| `C_ALIVE 0x20` → `S_TIME 0x21` | 請求恰好 1 byte；伺服器回 `c(0x21) d(gameSeconds 0..0x7FFFFFFF)`（client 當作 32-bit time_t），不是 `S_PING`。任何 phase 都接受 | CS:15270–15282；`worldTime` P:12023 |

### 6.3 `C_USE_SPELL` 技能 → 尾段形狀對照（T004-4）

證據等級：**來源**。全表由 `維護工具/generate-spell-request-shapes.py` 從 lito-launcher-server `2066774` 的 `BattleSpellCatalog`、`ClientSession`、`IllusionSkillRequests`、`DragonSkillRequests` 靜態產生，不是手抄。重產指令：

```bash
python3 -I 維護工具/generate-spell-request-shapes.py <lito-launcher-server clone> --markdown
```

**判定順序**和伺服器 dispatch 相同（`useSpell` CS:32161 → `DragonSkillRequests.dispatch` → `IllusionSkillRequests.dispatch` → `useSpellBeforeDragonTargets` CS:32165）：

1. **外層 wrapper**：`IllusionSkillRequests.ATTACK`（202,203,207,208,213）、`IllusionSkillRequests.OTHER`（204,209,211,212,214,216,217,218,219）、`DragonSkillRequests.TARGETED`（183,184,187,188,192,193,194）。這 21 個技能在線上**一律**用形狀 ⑩，body 必須剛好 11 bytes（`c c c d h h`），其他長度直接拒絕；targetId ≤ 0 時靜默忽略。伺服器把封包截成 3 bytes 後才交給內層，所以內層 handler 的形狀**不代表線上格式**。
2. **TELEPORT（5）**：⑤ `teleportSpellRequest`。dispatcher 在共用冷卻檢查之後、其他 early return 之前就先處理它。
3. **其餘技能**：依 `BattleSpellCatalog` 記錄類型找到 BATTLE 路徑的 handler 與它呼叫的 parser。一個類型有多種形狀時，依該筆實例的欄位判定：
   - `ShieldSpell.targeting`：PLAYER_OBJECT → ④，其他 → ①；
   - `AbilityBuffSpell`／`InvisibilitySpell.requestShape`：OBJECT_TARGET → ④、SELF_TICK → ②、SELECTOR_ONLY → ①；
   - `FreezeAttackSpell`：areaRadius > 0 或 lineTargeted → ①，其他 → ⑩；
   - `AreaAttackSpell`：maximumRange = 0 或 MAGMA_BREATH → ①，其他 → ⑩；
   - `AttackSpell`：⑩（它的 selector-only 變體 192/202/203/207/208/213 都已被第 1 步攔下）。

**範圍與限制**：
- **模式差異**：腳本對每個技能都分別算出 BATTLE 與 WORLD（經典世界）兩條路徑實際使用的 handler 和 parser，再逐筆比對形狀，不是只比對類型集合。
  - 21 個 wrapper 技能：兩種模式都先經過外層，形狀相同。
  - 54 個技能：BATTLE 與 WORLD 用不同 handler，判定依據欄的「WORLD：」寫出 WORLD 的 handler 與行號，逐筆比對後形狀相同。Freeze／Area／Attack 的 WORLD 條件分支另附 `分支 CS:` 行號。
  - 106 個技能：兩種模式共用同一個 handler。
  - 只要任何一筆形狀不同，腳本就把它列為 UNRESOLVED 並以非零結束碼結束；本次結果為 0 筆，所以**沒有 (row,column) 因模式不同而改變形狀**。
  - WORLD 另有支援技能白名單，以及只在 WORLD 檢查的保留空槽（`ReservedSkillSlot`，① 或 ④）；這兩者只影響「是否接受」，不改變格式。
- 表中的形狀只是**格式**契約；技能是否可用（學會與否、MP、冷卻、射程、狀態）仍由伺服器判定，失敗時多半靜默或回 `S_MESSAGE_CODE`。
- 判定依據欄的 `CS:` 行號是 handler 方法的開始行；wrapper 列另附外層檔案的 dispatch、長度檢查、parser 呼叫與截短轉交的行號，以及內層 handler。

**181 與 180**：表中 181 項＝`BattleSpellCatalog` 的 180 個 catalog 記錄（產生器 `維護工具/generate-spell-request-shapes.py:85` 從 `new (\w+Spell)\(` 收列）＋TELEPORT（skill 5），它是 dispatcher 特例，不在 catalog 裡（同檔 `:192` 另外補一列）。歷史紀錄說的「180 項目錄」就是指 catalog 記錄數，不是缺了一項。E149 與 `2066774` 的 catalog 常數 223／223 相同（#92 證據 E2）。

產生結果摘要（原樣輸出）：

```
source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; skills=181 unresolved=0 wrapper-not-in-catalog=[]
shapes: ① 無尾段=70; ② d(clientTick)=1; ③ h(x) h(y)=2; ④ d(target)=35; ⑤ h(map) h(x) h(y)=1; ⑥ h(map) h(x) h(y)=1; ⑦ s(name)=1; ⑧ s(name)=1; ⑨ d(selection)=1; ⑩ d(target) h(x) h(y)=67; ⑪ d(target) h(x) h(y) s(text)=1
source=git:2066774; mode check per skill: BATTLE／WORLD 不同 handler、逐筆比對形狀相同=54; wrapper（兩種模式都先經過外層）=21; 兩種模式共用同一 handler=106
```

| skillId | 常數 | row | col | 記錄類型 | 尾段形狀 | 判定依據 |
|---:|---|---:|---:|---|---|---|
| 1 | `BASIC_HEAL` | 0 | 0 | HealingSpell | ④ d(target) | useHealingSpell CS:74148；WORLD：useClassicWorldHealingSpell CS:73700（同形狀） |
| 2 | `LIGHT` | 0 | 1 | LightSpell | ① 無尾段 | useLightSpell CS:34284 |
| 3 | `SHIELD` | 0 | 2 | ShieldSpell | ① 無尾段 | useShieldSpell CS:34349；targeting=SELF_SELECTOR |
| 4 | `LIGHT_ARROW` | 0 | 3 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 5 | `TELEPORT` | 0 | 4 | （dispatcher 特例） | ⑤ h(map) h(x) h(y) | useTeleportSpell CS:27905；dispatcher 在 phase 檢查前轉交（useSpellBeforeDragonTargets CS:32165 起） |
| 6 | `ICE_ARROW` | 0 | 5 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 7 | `WIND_CUTTER` | 0 | 6 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 8 | `HOLY_WEAPON` | 0 | 7 | WeaponBuffSpell | ④ d(target) | useWeaponBuffSpell CS:39818 |
| 9 | `CURE_POISON` | 1 | 0 | CurseRemovalSpell | ④ d(target) | useCurseRemovalSpell CS:70516；WORLD：useClassicWorldCurseRemovalSpell CS:70274（同形狀） |
| 10 | `CHILL_TOUCH` | 1 | 1 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 11 | `CURSE_POISON` | 1 | 2 | PoisonSpell | ⑩ d(target) h(x) h(y) | usePoisonSpell CS:69474；WORLD：useClassicWorldPoisonSpell CS:69162（同形狀） |
| 12 | `ENCHANT_WEAPON` | 1 | 3 | ItemWeaponEnchantSpell | ④ d(target) | useItemWeaponEnchantSpell CS:55148 |
| 13 | `DETECTION` | 1 | 4 | DetectionSpell | ① 無尾段 | useDetectionSpell CS:55819 |
| 14 | `DECREASE_WEIGHT` | 1 | 5 | WeightSpell | ① 無尾段 | useWeightSpell CS:44495 |
| 15 | `FIRE_ARROW` | 1 | 6 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 16 | `STALAC` | 1 | 7 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 17 | `LIGHTNING` | 2 | 0 | LineAttackSpell | ⑩ d(target) h(x) h(y) | useLineAttackSpell CS:59792；WORLD：useClassicWorldLineAttackSpell CS:64711（同形狀） |
| 18 | `TURN_UNDEAD` | 2 | 1 | TurnUndeadSpell | ⑩ d(target) h(x) h(y) | useTurnUndeadSpell CS:46440；WORLD：useClassicWorldTurnUndeadSpell CS:46651（同形狀） |
| 19 | `EXTRA_HEAL` | 2 | 2 | HealingSpell | ④ d(target) | useHealingSpell CS:74148；WORLD：useClassicWorldHealingSpell CS:73700（同形狀） |
| 20 | `CURSE_BLIND` | 2 | 3 | BlindSpell | ⑩ d(target) h(x) h(y) | useBlindSpell CS:47793；WORLD：useClassicWorldBlindSpell CS:47507（同形狀） |
| 21 | `BLESSED_ARMOR` | 2 | 4 | ItemArmorEnchantSpell | ④ d(target) | useItemArmorEnchantSpell CS:55310 |
| 22 | `FROZEN_CLOUD` | 2 | 5 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | useAreaAttackSpell CS:60738；maximumRange=6（分支 CS:33683–33697）；WORLD：useClassicWorldAreaAttackSpell CS:67161（同形狀） |
| 23 | `REVEAL_WEAKNESS` | 2 | 6 | RevealWeaknessSpell | ④ d(target) | useRevealWeaknessSpell CS:55611 |
| 25 | `BURNING_FIREBALL` | 3 | 0 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | useAreaAttackSpell CS:60738；maximumRange=6（分支 CS:33683–33697）；WORLD：useClassicWorldAreaAttackSpell CS:67161（同形狀） |
| 26 | `PHYSICAL_ENCHANT_DEX` | 3 | 1 | AbilityBuffSpell | ④ d(target) | useAbilityBuffSpell CS:39586；requestShape=OBJECT_TARGET |
| 27 | `WEAPON_BREAK` | 3 | 2 | WeaponBreakSpell | ⑩ d(target) h(x) h(y) | useWeaponBreakSpell CS:56464；WORLD：useClassicWorldWeaponBreakSpell CS:56687（同形狀） |
| 28 | `VAMPIRIC_TOUCH` | 3 | 3 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 29 | `SLOW` | 3 | 4 | EntangleSpell | ⑩ d(target) h(x) h(y) | useEntangleSpell CS:48957 |
| 30 | `EARTH_JAIL` | 3 | 5 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | useAreaAttackSpell CS:60738；maximumRange=8（分支 CS:33683–33697）；WORLD：useClassicWorldAreaAttackSpell CS:67161（同形狀） |
| 31 | `COUNTER_MAGIC` | 3 | 6 | CounterMagicSpell | ① 無尾段 | useCounterMagicSpell CS:34538 |
| 32 | `MEDITATION` | 3 | 7 | RegenSpell | ① 無尾段 | useRegenSpell CS:44707 |
| 33 | `CURSE_PARALYZE` | 4 | 0 | ParalysisSpell | ⑩ d(target) h(x) h(y) | useParalysisSpell CS:70025；WORLD：useClassicWorldParalysisSpell CS:69705（同形狀） |
| 34 | `CALL_LIGHTNING` | 4 | 1 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 35 | `GREATER_HEAL` | 4 | 2 | HealingSpell | ④ d(target) | useHealingSpell CS:74148；WORLD：useClassicWorldHealingSpell CS:73700（同形狀） |
| 36 | `TAMING_MONSTER` | 4 | 3 | TamingSpell | ⑩ d(target) h(x) h(y) | useTamingSpell CS:72350 |
| 37 | `REMOVE_CURSE` | 4 | 4 | CurseRemovalSpell | ④ d(target) | useCurseRemovalSpell CS:70516；WORLD：useClassicWorldCurseRemovalSpell CS:70274（同形狀） |
| 38 | `CONE_OF_COLD` | 4 | 5 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 39 | `MANA_DRAIN` | 4 | 6 | ManaDrainSpell | ⑩ d(target) h(x) h(y) | useManaDrainSpell CS:47245；WORLD：useClassicWorldManaDrainSpell CS:46914（同形狀） |
| 40 | `DARKNESS` | 4 | 7 | BlindSpell | ⑩ d(target) h(x) h(y) | useBlindSpell CS:47793；WORLD：useClassicWorldBlindSpell CS:47507（同形狀） |
| 41 | `CREATE_ZOMBIE` | 5 | 0 | CreateZombieSpell | ④ d(target) | useCreateZombieSpell CS:71326 |
| 42 | `PHYSICAL_ENCHANT_STR` | 5 | 1 | AbilityBuffSpell | ④ d(target) | useAbilityBuffSpell CS:39586；requestShape=OBJECT_TARGET |
| 43 | `HASTE` | 5 | 2 | HasteSpell | ④ d(target) | useHasteSpell CS:40143 |
| 44 | `CANCELLATION` | 5 | 3 | CancellationSpell | ④ d(target) | useWorldRoiCancellation CS:42498；useCancellationSpell CS:43199 |
| 45 | `ERUPTION` | 5 | 4 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 46 | `SUNBURST` | 5 | 5 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 47 | `WEAKNESS` | 5 | 6 | WeaknessSpell | ⑩ d(target) h(x) h(y) | useWeaknessSpell CS:57343；WORLD：useClassicWorldWeaknessSpell CS:57013（同形狀） |
| 48 | `BLESS_WEAPON` | 5 | 7 | WeaponBuffSpell | ④ d(target) | useWeaponBuffSpell CS:39818 |
| 49 | `HEAL_ALL` | 6 | 0 | GroupHealingSpell | ① 無尾段 | useGroupHealingSpell CS:73409 |
| 50 | `ICE_LANCE` | 6 | 1 | FreezeAttackSpell | ⑩ d(target) h(x) h(y) | useFreezeAttackSpell CS:62540；areaRadius=0 lineTargeted=false（分支 CS:33607–33617）；WORLD：useClassicWorldFreezeAttackSpell CS:62072（同形狀） |
| 51 | `SUMMON_MONSTER` | 6 | 2 | SummonSpell | ⑨ d(selection) | useSummonSpell CS:70920 |
| 52 | `HOLY_WALK` | 6 | 3 | MovementBuffSpell | ① 無尾段 | useMovementBuffSpell CS:41258 |
| 53 | `TORNADO` | 6 | 4 | AreaAttackSpell | ① 無尾段 | useCenteredAreaAttackSpell CS:61245；maximumRange=0（分支 CS:33683–33697）；WORLD：useClassicWorldCenteredAreaAttackSpell CS:66546（同形狀） |
| 54 | `GREATER_HASTE` | 6 | 5 | SelfHasteSpell | ① 無尾段 | useSelfHasteSpell CS:40464 |
| 55 | `BERSERKERS` | 6 | 6 | CombatBuffSpell | ④ d(target) | useCombatBuffSpell CS:40561 |
| 56 | `DISEASE` | 6 | 7 | WeaknessSpell | ⑩ d(target) h(x) h(y) | useWeaknessSpell CS:57343；WORLD：useClassicWorldWeaknessSpell CS:57013（同形狀） |
| 57 | `FULL_HEAL` | 7 | 0 | HealingSpell | ④ d(target) | useHealingSpell CS:74148；WORLD：useClassicWorldHealingSpell CS:73700（同形狀） |
| 58 | `FIRE_WALL` | 7 | 1 | GroundSpell | ③ h(x) h(y) | useGroundSpell CS:72741；WORLD：useClassicWorldGroundSpell CS:72875（同形狀） |
| 59 | `BLIZZARD` | 7 | 2 | AreaAttackSpell | ① 無尾段 | useCenteredAreaAttackSpell CS:61245；maximumRange=0（分支 CS:33683–33697）；WORLD：useClassicWorldCenteredAreaAttackSpell CS:66546（同形狀） |
| 60 | `INVISIBILITY` | 7 | 3 | InvisibilitySpell | ② d(clientTick) | useInvisibilitySpell CS:45996；requestShape=SELF_TICK |
| 61 | `RESURRECTION` | 7 | 4 | ResurrectionSpell | ④ d(target) | useCallOfNatureSpell CS:71846；useResurrectionSpell CS:72140；WORLD：useClassicWorldResurrectionSpell CS:71568（同形狀） |
| 62 | `EARTHQUAKE` | 7 | 5 | AreaAttackSpell | ① 無尾段 | useCenteredAreaAttackSpell CS:61245；maximumRange=0（分支 CS:33683–33697）；WORLD：useClassicWorldCenteredAreaAttackSpell CS:66546（同形狀） |
| 63 | `LIFE_STREAM` | 7 | 6 | GroundSpell | ③ h(x) h(y) | useGroundSpell CS:72741；WORLD：useClassicWorldGroundSpell CS:72875（同形狀） |
| 64 | `SILENCE` | 7 | 7 | SilenceSpell | ⑩ d(target) h(x) h(y) | useSilenceSpell CS:48326；WORLD：useClassicWorldSilenceSpell CS:48014（同形狀） |
| 65 | `LIGHTNING_STORM` | 8 | 0 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | useAreaAttackSpell CS:60738；maximumRange=10（分支 CS:33683–33697）；WORLD：useClassicWorldAreaAttackSpell CS:67161（同形狀） |
| 66 | `FOG_OF_SLEEPING` | 8 | 1 | SleepSpell | ⑩ d(target) h(x) h(y) | useSleepSpell CS:59301；WORLD：useClassicWorldSleepSpell CS:58903（同形狀） |
| 67 | `SHAPE_CHANGE` | 8 | 2 | PolymorphSpell | ④ d(target) | usePolymorphSpell CS:34086 |
| 68 | `IMMUNE_TO_HARM` | 8 | 3 | DamageProtectionSpell | ④ d(target) | useDamageProtectionSpell CS:40701 |
| 69 | `MASS_TELEPORT` | 8 | 4 | MassTeleportSpell | ⑥ h(map) h(x) h(y) | useMassTeleportSpell CS:40875 |
| 70 | `FIRE_STORM` | 8 | 5 | AreaAttackSpell | ① 無尾段 | useCenteredAreaAttackSpell CS:61245；maximumRange=0（分支 CS:33683–33697）；WORLD：useClassicWorldCenteredAreaAttackSpell CS:66546（同形狀） |
| 71 | `DECAY_POTION` | 8 | 6 | DecayPotionSpell | ⑩ d(target) h(x) h(y) | useDecayPotionSpell CS:54838；WORLD：useClassicWorldDecayPotionSpell CS:54519（同形狀） |
| 72 | `COUNTER_DETECTION` | 8 | 7 | CounterDetectionSpell | ① 無尾段 | useCounterDetectionSpell CS:56184；WORLD：useClassicWorldCounterDetectionSpell CS:55970（同形狀） |
| 73 | `CREATE_MAGICAL_WEAPON` | 9 | 0 | MagicalWeaponSpell | ④ d(target) | useMagicalWeaponSpell CS:55450 |
| 74 | `METEOR_STRIKE` | 9 | 1 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | useAreaAttackSpell CS:60738；maximumRange=10（分支 CS:33683–33697）；WORLD：useClassicWorldAreaAttackSpell CS:67161（同形狀） |
| 75 | `GREATER_RESURRECTION` | 9 | 2 | ResurrectionSpell | ④ d(target) | useCallOfNatureSpell CS:71846；useResurrectionSpell CS:72140；WORLD：useClassicWorldResurrectionSpell CS:71568（同形狀） |
| 76 | `MASS_SLOW` | 9 | 3 | MassSlowSpell | ⑩ d(target) h(x) h(y) | useMassSlowSpell CS:58571；WORLD：useClassicWorldMassSlowSpell CS:58138（同形狀） |
| 77 | `DISINTEGRATE` | 9 | 4 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 78 | `ABSOLUTE_BARRIER` | 9 | 5 | AbsoluteBarrierSpell | ① 無尾段 | useAbsoluteBarrierSpell CS:34656 |
| 79 | `ADVANCE_SPIRIT` | 9 | 6 | AdvanceSpiritSpell | ④ d(target) | useAdvanceSpiritSpell CS:34743 |
| 80 | `FREEZING_BLIZZARD` | 9 | 7 | FreezeAttackSpell | ① 無尾段 | useCenteredFreezeAttackSpell CS:61654；areaRadius=10 lineTargeted=false（分支 CS:33607–33617）；WORLD：useClassicWorldCenteredFreezeAttackSpell CS:65703（同形狀） |
| 87 | `SHOCK_STUN` | 10 | 6 | ShockStunSpell | ⑩ d(target) h(x) h(y) | useShockStunSpell CS:63012；WORLD：useClassicWorldShockStunSpell CS:63356（同形狀） |
| 88 | `REDUCTION_ARMOR` | 10 | 7 | ReductionArmorSpell | ① 無尾段 | useReductionArmorSpell CS:36151 |
| 89 | `BOUNCE_ATTACK` | 11 | 0 | BounceAttackSpell | ① 無尾段 | useBounceAttackSpell CS:36289 |
| 90 | `SOLID_CARRIAGE` | 11 | 1 | SolidCarriageSpell | ① 無尾段 | useSolidCarriageSpell CS:36431 |
| 91 | `COUNTER_BARRIER` | 11 | 2 | CounterBarrierSpell | ① 無尾段 | useCounterBarrierSpell CS:36595 |
| 97 | `BLIND_HIDING` | 12 | 0 | InvisibilitySpell | ① 無尾段 | useInvisibilitySpell CS:45996；requestShape=SELECTOR_ONLY |
| 98 | `ENCHANT_VENOM` | 12 | 1 | EnchantVenomSpell | ① 無尾段 | useEnchantVenomSpell CS:46140 |
| 99 | `SHADOW_ARMOR` | 12 | 2 | ShieldSpell | ① 無尾段 | useShieldSpell CS:34349；targeting=SELF_SELECTOR |
| 100 | `BRING_STONE` | 12 | 3 | BringStoneSpell | ④ d(target) | useBringStoneSpell CS:46253 |
| 101 | `MOVING_ACCELERATION` | 12 | 4 | MovementBuffSpell | ① 無尾段 | useMovementBuffSpell CS:41258 |
| 102 | `BURNING_SPIRIT` | 12 | 5 | PhysicalProcBuffSpell | ① 無尾段 | usePhysicalProcBuffSpell CS:41359 |
| 103 | `DARK_BLIND` | 12 | 6 | BlindSpell | ⑩ d(target) h(x) h(y) | useBlindSpell CS:47793；WORLD：useClassicWorldBlindSpell CS:47507（同形狀） |
| 104 | `VENOM_RESIST` | 12 | 7 | PoisonResistanceSpell | ① 無尾段 | usePoisonResistanceSpell CS:41897 |
| 105 | `DOUBLE_BRAKE` | 13 | 0 | PhysicalProcBuffSpell | ① 無尾段 | usePhysicalProcBuffSpell CS:41359 |
| 106 | `UNCANNY_DODGE` | 13 | 1 | DodgeBuffSpell | ① 無尾段 | useDodgeBuffSpell CS:41632 |
| 107 | `SHADOW_FANG` | 13 | 2 | ItemWeaponEnchantSpell | ④ d(target) | useItemWeaponEnchantSpell CS:55148 |
| 108 | `FINAL_BURN` | 13 | 3 | AttackSpell | ⑩ d(target) h(x) h(y) | useAttackSpell CS:63784；一般（分支 CS:33698–33717）；WORLD：useClassicWorldAttackSpell CS:68066（同形狀） |
| 109 | `DRESS_MIGHTY` | 13 | 4 | AbilityBuffSpell | ① 無尾段 | useAbilityBuffSpell CS:39586；requestShape=SELECTOR_ONLY |
| 110 | `DRESS_DEXTERITY` | 13 | 5 | AbilityBuffSpell | ① 無尾段 | useAbilityBuffSpell CS:39586；requestShape=SELECTOR_ONLY |
| 111 | `DRESS_EVASION` | 13 | 6 | RangedEvasionBuffSpell | ① 無尾段 | useRangedEvasionBuffSpell CS:41769 |
| 112 | `ARMOR_BREAK` | 13 | 7 | ArmorBreakSpell | ① 無尾段 | useArmorBreakSpell CS:34881 |
| 113 | `TRUE_TARGET` | 14 | 0 | TrueTargetSpell | ⑪ d(target) h(x) h(y) s(text) | useTrueTargetSpell CS:44995 |
| 114 | `GLOWING_AURA` | 14 | 1 | RoyalAuraSpell | ① 無尾段 | useRoyalAuraSpell CS:45159 |
| 115 | `SHINING_AURA` | 14 | 2 | RoyalAuraSpell | ① 無尾段 | useRoyalAuraSpell CS:45159 |
| 116 | `CALL_CLAN` | 14 | 3 | CallClanSpell | ⑦ s(name) | useCallClanSpell CS:45334 |
| 117 | `BRAVE_AURA` | 14 | 4 | RoyalAuraSpell | ① 無尾段 | useRoyalAuraSpell CS:45159 |
| 118 | `RUN_CLAN` | 14 | 5 | RunClanSpell | ⑧ s(name) | useRunClanSpell CS:45766 |
| 129 | `RESIST_MAGIC` | 16 | 0 | MagicResistanceSpell | ① 無尾段 | useMagicResistanceSpell CS:42018 |
| 130 | `BODY_TO_MIND` | 16 | 1 | BodyToMindSpell | ① 無尾段 | useBodyToMindSpell CS:42112 |
| 131 | `WORLD_TREE_CALL` | 16 | 2 | WorldTreeCallSpell | ① 無尾段 | useWorldTreeCallSpell CS:42356 |
| 132 | `TRIPLE_ARROW` | 16 | 3 | TripleArrowSpell | ⑩ d(target) h(x) h(y) | useTripleArrowSpell CS:42206 |
| 133 | `ELEMENTAL_FALL_DOWN` | 16 | 4 | ElementalFallDownSpell | ⑩ d(target) h(x) h(y) | useElementalFallDownSpell CS:48612 |
| 134 | `COUNTER_MIRROR` | 16 | 5 | CounterMirrorSpell | ① 無尾段 | useCounterMirrorSpell CS:50125 |
| 137 | `CLEAR_MIND` | 17 | 0 | ClearMindSpell | ① 無尾段 | useClearMindSpell CS:50255 |
| 138 | `RESIST_ELEMENTAL` | 17 | 1 | ResistElementalSpell | ① 無尾段 | useResistElementalSpell CS:50397 |
| 145 | `RETURN_TO_NATURE` | 18 | 0 | ReturnToNatureSpell | ⑩ d(target) h(x) h(y) | useReturnToNatureSpell CS:49742 |
| 146 | `BLOODY_SOUL` | 18 | 1 | BodyToMindSpell | ① 無尾段 | useBodyToMindSpell CS:42112 |
| 147 | `ELEMENTAL_PROTECTION` | 18 | 2 | ElementalProtectionSpell | ① 無尾段 | useElementalProtectionSpell CS:50548 |
| 148 | `FIRE_WEAPON` | 18 | 3 | FireWeaponSpell | ④ d(target) | useFireWeaponSpell CS:50714 |
| 149 | `WIND_SHOT` | 18 | 4 | WindShotSpell | ④ d(target) | useWindShotSpell CS:50899 |
| 150 | `WIND_WALK` | 18 | 5 | MovementBuffSpell | ① 無尾段 | useMovementBuffSpell CS:41258 |
| 151 | `EARTH_SKIN` | 18 | 6 | ShieldSpell | ④ d(target) | useShieldSpell CS:34349；targeting=PLAYER_OBJECT |
| 152 | `ENTANGLE` | 18 | 7 | EntangleSpell | ⑩ d(target) h(x) h(y) | useEntangleSpell CS:48957 |
| 153 | `ERASE_MAGIC` | 19 | 0 | EraseMagicSpell | ⑩ d(target) h(x) h(y) | useEraseMagicSpell CS:49375 |
| 154 | `LESSER_ELEMENTAL` | 19 | 1 | ElementalSummonSpell | ① 無尾段 | useElementalSummonSpell CS:70760 |
| 155 | `FIRE_BLESS` | 19 | 2 | FireBlessSpell | ① 無尾段 | useFireBlessSpell CS:51046 |
| 156 | `STORM_EYE` | 19 | 3 | StormEyeSpell | ① 無尾段 | useStormEyeSpell CS:51236 |
| 157 | `EARTH_BIND` | 19 | 4 | BindSpell | ⑩ d(target) h(x) h(y) | useBindSpell CS:57936；WORLD：useClassicWorldBindSpell CS:57649（同形狀） |
| 158 | `NATURES_TOUCH` | 19 | 5 | NaturesTouchSpell | ④ d(target) | useNaturesTouchSpell CS:52825 |
| 159 | `EARTH_BLESS` | 19 | 6 | EarthBlessSpell | ① 無尾段 | useEarthBlessSpell CS:51578 |
| 160 | `AQUA_PROTECTER` | 19 | 7 | AquaProtectSpell | ④ d(target) | useAquaProtectSpell CS:53191 |
| 161 | `AREA_OF_SILENCE` | 20 | 0 | AreaSilenceSpell | ① 無尾段 | useAreaSilenceSpell CS:53364 |
| 162 | `GREATER_ELEMENTAL` | 20 | 1 | ElementalSummonSpell | ① 無尾段 | useElementalSummonSpell CS:70760 |
| 163 | `BURNING_WEAPON` | 20 | 2 | FireWeaponSpell | ④ d(target) | useFireWeaponSpell CS:50714 |
| 164 | `NATURES_BLESSING` | 20 | 3 | NaturesBlessingSpell | ① 無尾段 | useNaturesBlessingSpell CS:73147 |
| 165 | `CALL_OF_NATURE` | 20 | 4 | ResurrectionSpell | ④ d(target) | useCallOfNatureSpell CS:71846；useResurrectionSpell CS:72140；WORLD：useClassicWorldResurrectionSpell CS:71568（同形狀） |
| 166 | `STORM_SHOT` | 20 | 5 | StormShotSpell | ④ d(target) | useStormShotSpell CS:51427 |
| 167 | `WIND_SHACKLE` | 20 | 6 | WindShackleSpell | ⑩ d(target) h(x) h(y) | useWindShackleSpell CS:53677 |
| 168 | `IRON_SKIN` | 20 | 7 | IronSkinSpell | ④ d(target) | useIronSkinSpell CS:51761 |
| 169 | `EXOTIC_VITALIZE` | 21 | 0 | ExoticVitalizeSpell | ① 無尾段 | useExoticVitalizeSpell CS:44227 |
| 170 | `WATER_LIFE` | 21 | 1 | WaterLifeSpell | ④ d(target) | useWaterLifeSpell CS:53002 |
| 171 | `ELEMENTAL_FIRE` | 21 | 2 | PhysicalProcBuffSpell | ① 無尾段 | usePhysicalProcBuffSpell CS:41359 |
| 172 | `STORM_WALK` | 21 | 3 | OtherNoEffectSpell | ① 無尾段 | useOtherNoEffectSpell CS:41532 |
| 173 | `POLLUTE_WATER` | 21 | 4 | PolluteWaterSpell | ⑩ d(target) h(x) h(y) | useTargetNegativeStatusSpell CS:54082 |
| 174 | `STRIKER_GALE` | 21 | 5 | StrikerGaleSpell | ⑩ d(target) h(x) h(y) | useTargetNegativeStatusSpell CS:54082 |
| 175 | `SOUL_OF_FLAME` | 21 | 6 | PhysicalProcBuffSpell | ① 無尾段 | usePhysicalProcBuffSpell CS:41359 |
| 176 | `ADDITIONAL_FIRE` | 21 | 7 | AdditionalFireSpell | ① 無尾段 | useAdditionalFireSpell CS:44358 |
| 181 | `DRAGON_SKIN` | 22 | 4 | DragonSkinSpell | ① 無尾段 | useDragonSkinSpell CS:51889 |
| 182 | `BURNING_SLASH` | 22 | 5 | BurningSlashSpell | ① 無尾段 | useBurningSlashSpell CS:52717 |
| 183 | `GUARD_BRAKE` | 22 | 6 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useGuardBrakeSpell CS:35319 |
| 184 | `MAGMA_BREATH` | 22 | 7 | AreaAttackSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useSelectorOnlyTargetedAreaAttackSpell CS:60362 |
| 185 | `AWAKEN_ANTHARAS` | 23 | 0 | AwakenSpell | ① 無尾段 | useAwakenSpell CS:51996 |
| 186 | `BLOODLUST` | 23 | 1 | BloodlustSpell | ① 無尾段 | useBloodlustSpell CS:52203 |
| 187 | `FOE_SLAYER` | 23 | 2 | FoeSlayerSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useFoeSlayerSpell CS:52356 |
| 188 | `RESIST_FEAR` | 23 | 3 | ResistFearSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useResistFearSpell CS:35895 |
| 189 | `SHOCK_SKIN` | 23 | 4 | AreaAttackSpell | ① 無尾段 | useCenteredAreaAttackSpell CS:61245；maximumRange=0（分支 CS:33683–33697）；WORLD：useClassicWorldCenteredAreaAttackSpell CS:66546（同形狀） |
| 190 | `AWAKEN_FAFURION` | 23 | 5 | AwakenSpell | ① 無尾段 | useAwakenSpell CS:51996 |
| 191 | `MORTAL_BODY` | 23 | 6 | MortalBodySpell | ① 無尾段 | useMortalBodySpell CS:36733 |
| 192 | `THUNDER_GRAB` | 23 | 7 | AttackSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 193 | `HORROR_OF_DEATH` | 24 | 0 | GuardBrakeSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useGuardBrakeSpell CS:35319 |
| 194 | `FREEZING_BREATH` | 24 | 1 | FreezeAttackSpell | ⑩ d(target) h(x) h(y) | DragonSkillRequests.TARGETED：DragonSkillRequests.dispatch DragonSkillRequests.java:22（長度 11 檢查 :28、parser :32、截 3 bytes 轉交 :35） → 內層 useSelectorOnlyTargetedFreezeAttackSpell CS:60389 |
| 195 | `AWAKEN_VALAKAS` | 24 | 2 | AwakenSpell | ① 無尾段 | useAwakenSpell CS:51996 |
| 201 | `MIRROR_IMAGE` | 25 | 0 | DodgeBuffSpell | ① 無尾段 | useDodgeBuffSpell CS:41632 |
| 202 | `CONFUSION` | 25 | 1 | AttackSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.ATTACK：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 203 | `SMASH` | 25 | 2 | AttackSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.ATTACK：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 204 | `ILLUSION_OGRE` | 25 | 3 | IllusionOgreSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useIllusionOgreSpell CS:38689 |
| 205 | `CUBE_IGNITION` | 25 | 4 | CubeIgnitionSpell | ① 無尾段 | useCubeIgnitionSpell CS:36920 |
| 206 | `CONCENTRATION` | 25 | 5 | RegenSpell | ① 無尾段 | useRegenSpell CS:44707 |
| 207 | `MIND_BREAK` | 25 | 6 | AttackSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.ATTACK：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 208 | `BONE_BREAK` | 25 | 7 | AttackSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.ATTACK：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 209 | `ILLUSION_LICH` | 26 | 0 | IllusionLichSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useIllusionLichSpell CS:39238 |
| 210 | `CUBE_QUAKE` | 26 | 1 | CubeQuakeSpell | ① 無尾段 | useCubeQuakeSpell CS:37042 |
| 211 | `PATIENCE` | 26 | 2 | PatienceSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 usePatienceSpell CS:37696 |
| 212 | `PHANTASM` | 26 | 3 | PhantasmSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 usePhantasmSpell CS:37835 |
| 213 | `ARM_BREAKER` | 26 | 4 | AttackSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.ATTACK：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useSelectorOnlyTargetedAttackSpell CS:60410 |
| 214 | `ILLUSION_DIAMOND_GOLEM` | 26 | 5 | DiamondGolemSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useDiamondGolemSpell CS:38871 |
| 215 | `CUBE_SHOCK` | 26 | 6 | CubeShockSpell | ① 無尾段 | useCubeShockSpell CS:37161 |
| 216 | `INSIGHT` | 26 | 7 | InsightSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useInsightSpell CS:39053 |
| 217 | `PANIC` | 27 | 0 | PanicSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 usePanicSpell CS:38131 |
| 218 | `JOY_OF_PAIN` | 27 | 1 | JoyOfPainSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useJoyOfPainSpell CS:38446 |
| 219 | `ILLUSION_AVATAR` | 27 | 2 | IllusionAvatarSpell | ⑩ d(target) h(x) h(y) | IllusionSkillRequests.OTHER：IllusionSkillRequests.dispatch IllusionSkillRequests.java:9（長度 11 檢查 :12、parser :13、截 3 bytes 轉交 :19） → 內層 useIllusionAvatarSpell CS:39437 |
| 220 | `CUBE_HARMONY` | 27 | 3 | CubeHarmonySpell | ① 無尾段 | useCubeHarmonySpell CS:37280 |

**與重建版既有驗證的交叉核對**（歷史文件證據，不是本輪重跑；版本以各文件自己記載的為準）：

| 技能 | 文件記載 | 文件 | 文件記載的 JAR | §6.3 |
|---|---|---|---|---|
| 治癒 1 | 帶物件 ID | NATIVE-SKILLS | `58973343…7A74AA7` | ④ |
| 護盾 3 | 只帶 selector | NATIVE-SKILLS | `58973343…7A74AA7` | ① |
| 光箭 4 | C14 帶目標物件 ID 及目前座標 | NATIVE-ATTACK-SKILLS | 只寫「固定正式 JAR」，沒有 SHA → **待核對** | ⑩ |
| 冰箭 6 | 已接入（第 38 行） | NATIVE-ATTACK-SKILLS | 同上 → **待核對** | ⑩ |
| 風刃 7 | 由 BattleSpellCatalog／Protocol 取得 | NATIVE-WIND-CUTTER | `BBC59308…2DD5DCE43` | ⑩ |
| 火牆 58 | `14 07 01 x:u16 y:u16` | NATIVE-GROUND-SKILLS | `58973343…7A74AA7` | ③ |
| 生命之泉 63 | row7／column6 加 u16 座標 | NATIVE-REAGENT-SKILLS | 只寫「現行 JAR」，沒有 SHA → **待核對** | ③ |

其中有文件明確記載封包格式的五項（1、3、4、58、63）都和上表一致；6、7 的文件只證明「已接入」，沒有寫出封包格式。

## 7. 退出與重新開始

| 封包 | 欄位 | 對應 |
|---|---|---|
| `C_QUIT 0x0F` | opcode-only；伺服器設定 `quitRequested` 後關閉 socket；在 CHANNEL 中送出時可能先暫存頻道交接資訊，以便接續 | `requestQuit` CS:15020–15050 |
| `C_LOGOUT 0x02` | opcode-only（嚴格） | P:5569；`requestLogout` CS:15056 |
| `C_RESTART 0x10` → `S_RESTART 0x65` | 兩者都是 opcode-only；要求 phase ≥ AUTHENTICATED，且沒有其他待處理的轉場 | `requestRestart` CS:15075；P:16431 |
| 回到選服 | 原生順序是 `S_RESTART → C_LITO_QUIT_FIGHT → S_LITO_QUIT_FIGHT → C_LITO_LEAVE_CHANNEL／C_LITO_EXIT_ROOM → S_LITO_VS_SERVER_ALLOW_JOIN → C_LITO_LIN_CHARACTER_INFO`（`PendingTransition` CS:177–182）；另一條路是在 LOBBY 送 `C_LITO_ENTER_CHANNEL(100)`，伺服器回 `S_LITO_VS_SERVER_ALLOW_JOIN` | CS:8074、9836–9840、15252 |
| `S_LITO_VS_SERVER_ALLOW_JOIN 0xF5` | opcode-only（`allowPostTransition`）。client 收到後要用目前選定的來源名稱送 `C_LITO_LIN_CHARACTER_INFO 0xC8`，重新取得角色清單 | P:1551、16441–16443；CS:9839、14996、15252 |
| `S_KICK 0x66` | `c c(reason)`；目前只使用 reason 0 | P:12415；CS:88026 |
| 斷線 | socket 關閉後，伺服器在 `finally` 中做清理：交易、隊伍、持久 buff、房主轉移（ROOM） | CS:1112 起 |

## 8. 未涵蓋與待驗證

- **已驗證（靜態，T004-1，`docs/reports/issue-13.md`）**：對正式 JAR `e1498871` 用 javap 核對，結果如下：
  - §1–§6 以名稱搭配十六進位數值列出的 71 個 opcode，加上 §2 的 S_KEY，共 72 個全部一致。
  - KEY_HIGH、版本常數、versionAccepted 的全部欄位常數、5 個 LoginCheckResult、2 個 LitoVsServerAvailability、max.frame、兩層 login.bootstrap.sequence 都一致。
  - 唯一不一致的是一處文字敘述：NumericBreakthrough 的讀取時機，已在 §6.2 更正為「每次呼叫都重新讀取」。
- **仍待驗證**：T004-1 只是**靜態**核對，以下都沒有對 `e1498871` 動態驗證：parser 與 builder 的完整欄位、handler 的執行時順序、端到端流程。「已驗證」標記的 oracle 仍是對 `205DC1BA` 做的。§6.3 的尾段形狀讀的是 `src/main @2066774`，不是 E149 本身；但 E149 與 `2066774` 的 `BattleSpellCatalog` 常數 223／223 相同、`new *Spell(` 都是 186 處（#92 證據 E2）。
- **已釐清**：數值 schema 的 `NumericBreakthrough.schemaVersion()` 每次呼叫都先讀系統屬性 `-Dlito.numeric.breakthrough`，沒有才讀環境變數，有效值 1..4，其他值回退 0（T004-1）。正式登入器帶 4。client 依 body 長度判斷，見 §6.2「數值 schema」。
- **未涵蓋**：交易、血盟、好友、商店（`C_LITO_BUY_LIST`／`C_LITO_SELL_LIST` 等）、觀戰、配對、經典 WORLD 的 `C_ENTER_WORLD`、GM 指令，以及 `S_ATTACK` 的各個變體。每個技能（row,column）對應哪種 `C_USE_SPELL` 尾段，見 §6.3（BATTLE 路徑）。以上依 TASKS 的優先順序另開任務補充。
- **規則**：伺服器版本變更時，依 §9 的清單重核並更新本文；不可以只改 SHA。

## 9. 伺服器換版時客戶端要重核的清單（T025）

觸發條件：伺服器正式 JAR 不再是 `e1498871`。登入器 repo 的 `app/server/LiTo381-SharedResources-candidate.jar` 或 `docs/maintenance/release-*.json` 有新版，就算換版。

**重核原則**：
- 新 JAR 的血統要用**發布紀錄與 manifest pin**追溯。同一個 builder 腳本 SHA 可能搭配不同的輸入 Java，產出不同的 JAR（#120 新 N3）。所以只比 builder SHA 不夠，要比 base 與 candidate 的完整 SHA，並確認走的是哪一條分支。例如 `df9464f0` 底下有兩條 magical-weapon 後繼，正式鏈只走其中一條，不能混為同一條鏈（#92 報告附錄 D、#120）。
- 伺服器 `lito伺服器/LiTo381-Server/docs/maintenance/` 最新的正式 JAR 紀錄只到 0.6.68 `c02aaae8`；0.6.69 之後的正式版要看登入器 repo 的 `docs/maintenance/release-*.json`。
- 每一項的結果寫進該次換版的報告，分「已驗證／推測／待驗證」。未通過前，§0 的版本綁定不改。

| # | 項目 | 怎麼核對 | 對應的客戶端位置 |
|---:|---|---|---|
| 1 | JAR SHA 與血統分支 | 新 release 紀錄的 `build.base_sha256`、`candidate_sha256`、`changed_entries`；沿 manifest 回溯到上一個正式版（E149），確認經過哪些層 | §0、`docs/reports/issue-92.md` 附錄 D |
| 2 | `Protocol` 與 `Protocol$*` 的變動 | `changed_entries` 或逐 class 雜湊比對；有變動時重跑 T004-1 的 javap 核對（opcode、常數、版本、schema） | §1–§6、`native_opcodes.rs` |
| 3 | `ClientSession`／`GameHub` 的變動 class 與新增 `Native*` class | 列出變動與新增的 class，對照客戶端的 handler 與 controller | 各 `native_*.rs` |
| 4 | 技能結果與狀態圖示 ID | `BattleSpellCatalog` 常數與 `new *Spell(` 計數（#92 E2 的方法）、§6.3 重新產生（`維護工具/generate-spell-request-shapes.py`）、狀態圖示 ID（例如龍之護鎧 3080、職業圖示） | §6.3、`docs/SKILLS-STATUS.md`（T026） |
| 5 | 房間與結算 | 房間列表、人數（含 ArenaBot）、准入、模式規則、結算與斷線結算、回大廳 | §5、§6.1、§7 |
| 6 | 移動與傳送 | S_BLINK／C_BLINK 與 S_PARALYSE 模式 17 解鎖、權威移動、portal 完成、沒有 S_PORTAL 的解鎖（奪命之雷、技能 69 失敗）、`.loc`／`.move` | G06 測試（`tests/native_teleport_failure.rs`） |
| 7 | 裝備與租賃 | 穿脫只送一份最終描述、NativeArmorStatus、租賃交接（RentalBattleHandoff）與租賃城鎮移動 | 背包與租裝相關模組 |
| 8 | GM 指令 | `.polyme`（7200 秒、到期還原、離房清除）、Ctrl 強制目標、指令權限 | 聊天（T060）、外型（T047） |
| 9 | 登入流程與來源伺服器 | 來源登入、選服後重新登入；來源伺服器清單（原生限制 8 格，目前只啟用 380_ac_1、380_ac_2） | §4 |
| 10 | 相容 DLL 與 hook 位址 | `litoddraw.dll` 通常和 JAR 成對換版（目前是 `e47bd6c6…`）；原生 hook 位址（例如 forced_combat_send_hook）要一起核對。Rust 客戶端不依賴這個 DLL，但 LFC1（T061）等擴充格式要以它為準 | T061 |
| 11 | 候選協定變更是否被新正式版採用 | 下列目前只在隔離候選、E149 沒有的變更，換版時逐項確認：ICONS1（36 種狀態）、ICON_I2H、OwnCharStatus、1016／1017 修正、娃娃 29 的 buffDuration `writeH`、S_HPMeter 104 的 hide255、施法時間窗與 SprTable 截斷、裝備只送一份最終描述、舊角色封包抑制、173／174 的 opcode 修正、portal 在落點前改 showId | T046、T047、T023 |
| 12 | Maven 產物與正式 JAR 的差異 | 若伺服器改發 Maven 產物：E149 有、`src/main` 沒有的 `NativeDeVisibilitySwitch`；只在 Maven 產物的 1320 個 `Modern*` 類與 49 個非 Modern 類（lito-launcher-server `docs/server-source-gap.md:37-1454`）。客戶端不以 `Modern*` 協定為目標 | §0 |
| 13 | 資源 pin | 共用客戶端資源 `TW13081901.txt`（SHA `7ebe2b46…`）；特效資源資格（例如 `disintegrate_assets.rs` 的 `CURRENT_PACK`／`CURRENT_TEXT`） | T057、T011-F |
| 14 | 回歸 | 跑 `維護工具/run-local-test-suite.ps1`（完整版）與 G06 5 個 scenario（`維護工具/run-teleport-failure.ps1`），全部結束碼 0 才算通過 | M1、T041 |
