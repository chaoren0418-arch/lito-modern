# PROTOCOL-CONTRACT — lito-modern 客戶端 ↔ lito-launcher-server 0.6.71

更新：2026-11-04（Issue #4 / T004）。本檔取代「只靠 JAR SHA pin」的作法，讓 client 端可以**不看 JAR**就實作 LOGIN → GAME → 大廳 → 房間 → 戰鬥 → 退出。

## 0. 適用範圍與證據等級

| 項目 | 值 |
|---|---|
| 伺服器正式版 | 0.6.71；正式 JAR `LiTo381-SharedResources-candidate.jar` SHA256 `e1498871c7be6b28…d99d498`（`lito登入器/LiTo381-Launcher/docs/maintenance/release-0.6.71.json` 的 `build.candidate_sha256`） |
| 本文讀的來源 | `chaoren0418-arch/lito-launcher-server` main @ `2066774`，`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/**` |
| 重建版已有 oracle 的 JAR | `205DC1BA…B542`（2026-10-12 快照，見 `docs/NATIVE-PROTOCOL.zh-TW.md`／`NATIVE-LOGIN`／`NATIVE-GAME`） |

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

文字規則：MS950 必須**精確往返**。builder 一律拒絕控制字元，以及無法無損編碼成 MS950 的字（`containsControl` 加上 round-trip 比對）。client 端不可以用 best-fit 或 replacement 轉碼（已驗證，NATIVE-LOGIN）。

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
| `S_LITO_ROOM_USER 0xCD` | `c d(roomId) s(name≤27) c(force 0/1) c(type) c(sex) d(winRateDen) d(wins) d(losses) c(ready) s(affiliation≤16) c(0) s("Ready") d(disconnects) c(channelSel) c(ready) d(channelRank) c(grade 0..14) c(cert)×N c(badge)×M d(master 0/1)`；ready 的值：0 未準備、1 已準備、2 商店清單。N、M 由 awards 決定，**推測**為固定長度，需核對 P:2509–2533 | `litoRoomParticipant` P:2419 | — |
| `C_LITO_EXIT_ROOM 0xD0` | opcode-only | `exitRoom` CS:14568 | ROOM／BATTLE／RESULT → CHANNEL（CS:14627／14675）；在 BATTLE 中送出時會先結算（CS:14605） |
| `S_LITO_EXIT_ROOM 0xD0` | `c d(roomId) s(displayName)` | CS:14641 | — |
| `S_LITO_DELETE_ROOM 0xCF` | `c d(roomId)` | `broadcastRoomDeleted` CS:14784 | — |

## 6. 戰鬥

### 6.1 開戰與結束

```
ROOM: C_LITO_READY_GAME 0xD2 c(state)          ← 參加者切換準備狀態
      S_LITO_READY_GAME 0xD1 s(name) c(state)  ← 廣播
      C_LITO_READY_GAME 0xD2（opcode-only）    ← 房主開戰
      S_LITO_COUNTDOWN 0xE1 c(value)（推測：開戰前倒數）
      S_WORLD 0x28 h(mapId) c(0) → S_LITO_START_GAME 0xD2 → S_LITO_TIMELIMIT 0xE9 d(seconds)；phase=BATTLE
BATTLE: …戰鬥…  → S_LITO_END_GAME 0xD3 d(winner) → RESULT
```

| 封包 | 欄位 | 對應 |
|---|---|---|
| `C_LITO_READY_GAME 0xD2` | 只有 opcode 表示房主開戰；`c c(state 0/1/2)` 表示切換準備狀態 | `litoReadyRequest` P:16643；`setReady` CS:11627（ROOM） |
| `S_LITO_READY_GAME 0xD1` | `c s(name) c(state)` | `readyState` P:16609 |
| 開戰三包 | `S_WORLD c h(mapId) c(0)`、`S_LITO_START_GAME c`、`S_LITO_TIMELIMIT c d(sec)`，順序固定 | `battleBootstrap` P:2209–2218；`startBattle` CS:11968（phase 在 12001 改成 BATTLE，12010 送出） |
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
| `C_USE_SPELL 0x14` | `c c(row) c(column≤7)` 後接依技能而定的尾段：無、`d(target)`、`h(x) h(y)`、`d(target) h(x) h(y)` 等 | `spellSelector` P:8933；P:8965–9347；`useSpell` CS:32161 |
| `C_USE_ITEM 0x1C` | `c d(invObjId≠0)` 後接依道具而定的尾段（預設是 `s(arg)`） | P:4529／4611；`useInventoryItem` CS:23071 |
| `C_SAY 0x13`／`C_CHAT 0x77` | `c c(type) s(msg 1..255 bytes)`；`C_CHAT` 的 type 必須是 12 | `chatRequest` P:5707；`say` CS:15860（ROOM／BATTLE／CHANNEL／WORLD） |
| `S_SAY 0x13` | `c c(type) d(objId) s(msg≤511) h(x) h(y)` | P:11335 |
| `S_PUT_OBJECT 0x0B` | `c h(x) h(y) d(objId) h(gfx) c(action) c(heading) c(light) c(speed) d(0) h(lawful) s(name) s(title) c(status) d(emblem) s(clan) s("") c(0) c(hpRatio) c(0) c(level) s("") h(0) c(0)`；`status&4` 為真時再接 `c(0xFF) d(0)`。寵物、NPC 等其他版型見 P:14975–16013 | `battleActorWithHeading` P:14895 |
| `S_MOVE_OBJECT 0x12` | `c d(objId) h(srcX) h(srcY) c(heading)`；client 從起點推進一格 | NativeMovement.java:5–12 |
| `S_REMOVE_OBJECT 0x15` | `c d(objId)` | P:12307 |
| `S_HIT_POINT 0x0D`／`S_MANA_POINT 0x4D` | `c h(cur) h(max)`；**注意**：`NumericBreakthrough.enabled()` 時兩個欄位都改成 `d`（Writer 的特例，P:2055–2071），client 必須以 body 長度 5 或 9 判斷 | P:10691；CS:12187 等 |
| `S_STATUS 0x0C` | `c d(objId) c(level) d(exp)` + 六能力 `c×6`（levelAbility 時改為 `d`） + `h(hp) h(maxHp) h(mp) h(maxMp) c(ac)`（NumericBreakthrough 時改為 `d×5`） + `d(0) c(food) c(weight) h(lawful)` + 四屬性 `c×4`（elementalResistance 時改為 `d`）。**變長版型由伺服器開關決定**，需由 T004-1 確認正式設定 | `characterStatus` P:10711 |
| `S_ATTACK 0x23` | 一般物理攻擊：`c c(action) d(attacker) d(target) c(hit?10:0) c(heading) d(0)`；其他變體（傷害、技能、遠程）的尾段**未讀** | P:9501；變體 P:9551–9787 |
| `S_EFFECT 0x37` | `c d(objId) h(gfx)` | P:16117 |
| `S_ADD_INVENTORY 0x16` | `c d(objId) c(useType) c(charge) h(icon) c(bless) d(count) c(identified) s(name) c(detailLen) raw(detail)` | P:12039／12741 |
| `C_ALIVE 0x20` | 恰好 1 byte；伺服器回 `S_TIME`，不是 `S_PING`。任何 phase 都接受 | CS:15270 |

## 7. 退出與重新開始

| 封包 | 欄位 | 對應 |
|---|---|---|
| `C_QUIT 0x0F` | opcode-only；伺服器設定 `quitRequested` 後關閉 socket；在 CHANNEL 中送出時可能先暫存頻道交接資訊，以便接續 | `requestQuit` CS:15020–15050 |
| `C_LOGOUT 0x02` | opcode-only（嚴格） | P:5569；`requestLogout` CS:15056 |
| `C_RESTART 0x10` → `S_RESTART 0x65` | 兩者都是 opcode-only；要求 phase ≥ AUTHENTICATED，且沒有其他待處理的轉場 | `requestRestart` CS:15075；P:16431 |
| 回到選服 | 原生順序是 `S_RESTART → C_LITO_QUIT_FIGHT → S_LITO_QUIT_FIGHT → C_LITO_LEAVE_CHANNEL／C_LITO_EXIT_ROOM → S_F5 → C_LITO_LIN_CHARACTER_INFO`（`PendingTransition` CS:177–182）；另一條路是在 LOBBY 送 `C_LITO_ENTER_CHANNEL(100)`，伺服器回 `S_LITO_VS_SERVER_ALLOW_JOIN 0xF5` | CS:8074、9836–9840 |
| `S_KICK 0x66` | `c c(reason)`；目前只使用 reason 0 | P:12415；CS:88026 |
| 斷線 | socket 關閉後，伺服器在 `finally` 中做清理：交易、隊伍、持久 buff、房主轉移（ROOM） | CS:1112 起 |

## 8. 未涵蓋與待驗證

- **待驗證**：§1–§7 的常數和版型都還沒有對正式 JAR `e1498871` 核對過，見 `docs/plans/issue-4.json` 的 T004-1。§2、§4 的版型已經對 `205DC1BA` 驗證過。
- **待驗證**：`NumericBreakthrough`、`levelAbility`、`elementalResistance` 在正式設定下是否開啟。這會決定 `S_HIT_POINT`、`S_MANA_POINT`、`S_STATUS` 的長度。
- **未涵蓋**：交易、血盟、好友、商店（`C_LITO_BUY_LIST`／`C_LITO_SELL_LIST` 等）、觀戰、配對、經典 WORLD 的 `C_ENTER_WORLD`、GM 指令，以及 `S_ATTACK` 的各個變體與 300 多種技能尾段。以上依 TASKS 的優先順序另開任務補充。
- **規則**：伺服器版本變更時，先重跑 T004-1 的靜態核對並更新本文；不可以只改 SHA。
