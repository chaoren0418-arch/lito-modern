# Issue #342 — T060-1：聊天（原生入口收發、過濾、密語；NPC 說話 opcode 45）

執行者：claude-local（本機）。分支 `task/342`。
本張是 T060 的 1／1：聊天視窗收發、過濾視窗、私訊的當版行為；NPC 對話 opcode 45。

**本機沒有執行任何編譯產物，也沒有開遊戲。** 本機只做了靜態讀碼、`javap`（唯讀反組譯正式 JAR）與 `cargo check`（只編譯）。單元測試交給 CI。畫面上實際長什麼樣子全部在「待驗證」。

## 結論

- 原生入口（`native_login_window.rs`）原本完全沒有聊天：`S_SAY`、`S_TELL`、`S_SAY_CODE` 進來後只被記成「未處理 opcode」，也沒有任何送出聊天的路徑。本張補上收、發、過濾、密語與 NPC 說話（opcode 45）的解析，並加了一個聊天面板。
- 封包格式與長度上限都用 `javap` 對正式 JAR E149 核對過（不是只看 lito-launcher-server 的原始碼）。
- **TASKS／驗收清單原本寫的「一般聊天 8 格、大喊 50 格、全服」不是 E149 的玩家聊天行為。** E149 的玩家聊天是送給同頻道／同房間／同租裝村／同 WORLD 地圖的所有人，和距離無關；type（一般／大喊／全體）只是原樣回傳的標籤。8 格與 50 格只存在於伺服器的 NPC 說話產生端。所以客戶端不做任何距離過濾，`docs/GUI-ACCEPTANCE.md` §17 的預期已照這個事實改寫。
- 密語在 E149 **沒有任何回音**：送達時只有對方收到；對象不在線、關閉密語或拒絕名單時伺服器只寫 log。客戶端因此把「已送出」記在自己的紀錄裡，並在提示列說明伺服器不會通知失敗。
- 舊的 Modern profile 聊天視窗（`chat_window.rs`、`direct_chat_window.rs`，走 v32 應用層協定）和原生入口是兩套東西，本張沒有改；它們的現況見下方「舊 Modern profile 聊天」。

## 做了什麼

### 1. 協定與模型（新檔 `LiTo381-Modern/src/modern_lito/native_chat.rs`）

| 封包 | 格式 | 本張的處理 |
|---|---|---|
| `S_SAY 0x13` | `c(type) d(objId) s(msg 1..511) h(x) h(y)` | 收進紀錄；type 0／2／3／12 分到 一般／大喊／全體／買賣，其餘歸「其他」 |
| `S_SAY_CODE 0x2D`（十進位 45） | `c(type 0..3) d(objId) s(msg 1..511) h(x) h(y)` | NPC 說話，全部三種：0 一般、2 大喊、3 全服；objId 0 是伺服器提示（`questHelpMessage`）。座標保留在紀錄裡 |
| `S_TELL 0x14` | `s(sender 1..27) s(msg 1..255)` | 收到的密語 |
| `C_SAY 0x13` | `c(type) s(msg 1..255)` | 一般（0）、大喊（2）、全體（3） |
| `C_CHAT 0x77` | `c(12) s(msg 1..255)` | 買賣（E149 只接受 type 12） |
| `C_TELL 0x1B` | `s(target 1..27) s(msg 1..255)` | 密語 |
| `C_ONOFF 0x0D` | `c(selector) c(0\|1)` | 全體（0）／密語（2）／買賣（6）的伺服器端收訊偏好 |

設計上的幾個決定：

- **結構嚴格、內容寬鬆。** 封包被截斷、超長、缺結尾、有多餘 byte、NPC type 超出 0..3 都算協定錯誤，而且出錯時紀錄不變。但文字內容是別的玩家打的，不能因為一個壞字就斷線：不是合法 MS950 的 byte 換成 `?`，控制字元換成空白。
- **不依座標過濾**（原因見結論）。座標只存著，之後要做頭頂對話泡泡時可以用。
- 送出前在客戶端擋掉伺服器會拒絕或會變亂碼的內容：空白訊息、控制字元與換行、超過 255 位元組、MS950 無法精確表示的字（表情符號、韓文等）。錯誤訊息是中文，會說明原因。
- 紀錄最多 200 則，超過時丟最舊的並記數。紀錄綁定「連線＋角色」：換連線或換角色就清空，不會看到上一個角色的密語。
- 顯示時去掉 `\fT`、`\fW` 這類顏色前綴；`$數字`（字串表編號）原樣保留，轉成文字屬 T049。
- 過濾是本機的顯示開關，關掉不會丟訊息。「全體／密語／買賣」另外送 `C_ONOFF` 讓伺服器停止傳送。

### 2. 接線（`native_login.rs`）

- `Snapshot` 多一個 `chat` 欄位。
- `accept_game_received`：聊天封包在進任何場景處理之前就收下，所以大廳、房間、戰鬥、WORLD 都收得到，也不會動到 `World` 的狀態。
- 新增 `chat_say`／`chat_tell`／`chat_preference`。只有在 `LobbyReady`、`RoomReady`、`BattlePending`、`WorldReady` 才送（對應伺服器的 CHANNEL／ROOM／BATTLE／WORLD；其他階段伺服器會直接丟掉）。
- 一般聊天不在本機先顯示，等伺服器回 `S_SAY` 才出現（伺服器的收件名單包含說話者自己）。

### 3. 聊天面板（新檔 `native_chat_window.rs`，接在 `native_login_window.rs`）

- 「聊天」按鈕：場景中在右上角「寵物／召喚物」下方；大廳與房間在右下角。有未讀時文字變成「聊天（新訊息）」。
- 面板做法和既有的寵物面板一樣：開啟時整頁換成面板，場景暫停接受移動／攻擊／技能鍵。內容是訊息紀錄、七個分類勾選框、頻道下拉（一般／大喊／全體／買賣／密語）、密語對象、訊息欄、送出、提示列。
- Enter 在訊息欄或對象欄送出（按住不重複送）。送出成功清空訊息欄，失敗保留內容並在提示列顯示原因。
- 中斷連線、回選角、開背包／租裝／寵物面板時面板自動關閉。
- 控件 ID 360–375，和既有 ID 不重複（建立時有重複檢查）。

### 4. 文件

- `docs/PROTOCOL-CONTRACT.md` §6.2：`S_SAY` 補上收件規則；新增 `S_SAY_CODE`、`C_TELL`／`S_TELL`、`C_ONOFF` 三列。
- `docs/GUI-ACCEPTANCE.md` §17：寫入操作方式與控件，Q01–Q10 的預期改成 E149 的實際行為，新增 Q11（流程切換）、Q12（紀錄上限與捲動）。
- `native_opcodes.rs`：加入七個聊天 opcode 常數。

## 怎麼驗證

### 已驗證（靜態，正式 JAR E149）

JAR：`D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`，SHA256 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。用 `javap -p -c -constants` 讀 `Protocol` 與 `ClientSession`（只反組譯，沒有載入或執行類別；輸出在 `scratch/issue-342/`，不進 git）。

| 項目 | 結果 |
|---|---|
| opcode 常數 | `C_ONOFF=13`、`C_SAY=19`、`C_TELL=27`、`C_CHAT=119`、`S_SAY=19`、`S_TELL=20`、`S_SAY_CODE=45` |
| `Protocol.say` | `c(19) c(type) d(id) s(msg) h(x) h(y)`；msg ≤511 bytes |
| `Protocol.sayCode` | `c(45) c(type) d(id) s(msg) h(x) h(y)`；type 限 0..3；msg 非空且 ≤511 bytes |
| `Protocol.questHelpMessage` | `c(45) c(0) d(0) s(msg) h(x) h(y)` |
| `Protocol.tell` | `c(20) s(sender ≤27) s(msg ≤255)` |
| `Protocol.chatRequest` | 只接受 opcode 19／119；msg 1..255 bytes；不可有剩餘 byte；opcode 119 時 type 必須是 12 |
| `Protocol.tellRequest` | target 會 trim，1..27 bytes；msg 1..255 bytes；不可有剩餘 byte |
| `Protocol.onOffRequest` | selector 0、2..8 或 0xFF；一般 selector 的 value 只能是 0／1 |
| `ClientSession.say` | 只在 ROOM／BATTLE／CHANNEL／WORLD 處理；GM 指令與禁言直接 return、不回音；收件人是 WORLD 地圖成員／租裝村成員／頻道成員／房間成員四選一，**沒有距離判斷**；每個收件人再過 `acceptsChatType` 與拒絕名單；名單為空時回給自己 |
| `ClientSession.acceptsChatType` | type 3 看 `showWorldChat`、type 12 看 `showTradeChat`、其餘一律收 |
| `ClientSession.tell` | 對象找不到、`canWhisper` 為假或被拒絕時只 `println` 後 return；成功時只對 target 送 `S_TELL` |
| `changeChatPreference` | selector 0→`showWorldChat`、2→`canWhisper`、6→`showTradeChat`；三者在建構子都設為 true |
| NPC 說話類別 | JAR 內有 `NativeDeChatChat`／`ChatShouting`／`ChatGlobal`／`NativeDeChatWire` 等 23 個 `NativeDeChat*` class |

### 已驗證（靜態，lito-launcher-server `origin/main` 原始碼，唯讀）

引用檔（`lito伺服器/LiTo381-Server/` 之下）：`docs/maintenance/native-de-chat-current.zh-TW.md`、`src/main/java/local/lito381/server/` 的 `Protocol.java`（:5708–5730、:5800–5820、:11316–11440）、`ClientSession.java`（:14313–14356、:14689–14709、:27475）、`NativeDeChatWire.java`、`NativeDeChatChat.java`、`NativeDeChatChatShouting.java`、`NativeDeChatChatGlobal.java`、`GameHub.java`（`roomMembers`／`channelMembers`）。

- NPC 說話的三種文字形狀：一般 `名稱: 文字`、大喊 `<名稱> 文字`、全服 `[名稱] 文字`；`NativeDeChatWire.content` 把來源的 opcode 161 轉成 `sayCode`（opcode 45）並補上說話者座標。
- 房間與頻道的成員清單包含說話者自己，所以自己的訊息會由伺服器回音。這一點只看了原始碼，沒有對 E149 的 `GameHub` 做 javap（列在「推測」）。
- tw-bin-engine 沒有用到：它對的是 L1J 的 opcode（NPC 說話是 161），依 `docs/REUSE-MAP.md` §0.1 不能照抄。

### 本機編譯

已驗證：在 worktree 的 `LiTo381-Modern/` 跑 `cargo check --locked --offline -j1 --all-targets`（target 目錄 `target/current`），結束碼 0；本張新增與修改的檔案沒有新的警告。這一步只編譯，沒有執行測試或任何產物。

### CI（單元測試）

CI_PLACEHOLDER_CI

新增的測試：

- `native_chat.rs` 14 個：opcode 45 三種 type 的解析與顯示、objId 0 提示與顏色前綴、壞封包不改變紀錄、511／512 byte 邊界、玩家聊天依 type 分類且不看座標、密語收發與長度邊界、不是聊天的 opcode 不被吃掉、惡意文字內容不會讓封包失敗、過濾不丟訊息、紀錄上限與角色綁定、`C_SAY`／`C_CHAT`／`C_TELL`／`C_ONOFF` 的位元組、送出前的拒絕條件、可送出的階段。
- `native_chat_controller_tests.rs` 6 個：沒有世界時也收得到、戰鬥中不動到 `World`、壞封包不改變 snapshot、換角色不帶走紀錄、非聊天階段不送、不合法文字不會碰到連線且失敗的密語不留紀錄。
- `native_opcodes.rs` 1 個：七個常數與 JAR 的十進位值相同。

### 舊 Modern profile 聊天（本張範圍「當版行為」的另一半）

這一套走 v32 應用層協定（`chat_protocol.rs`：request kind 50／result kind 51），由舊的 `login_window.rs` 開啟（:7480、:7628），和原生入口沒有共用程式。本張沒有改它。

靜態清點（數各檔的 `#[test]` 與 `#[ignore]`）：26 個檔、121 個測試，其中 78 個一般測試、43 個 ignored。

| 類別 | 檔案 | 測試數 | ignored |
|---|---|---|---|
| 協定 | `chat_protocol.rs`、`direct_chat_protocol_tests.rs`、`filtered_chat_protocol_tests.rs` | 27 | 0 |
| 狀態 | `chat_state.rs`、`chat_filtered_state_tests.rs`、`direct_chat_state_tests.rs` | 22 | 0 |
| 視窗邏輯 | `chat_window.rs`、`chat_filtered_window_tests.rs`、`direct_chat_window_tests.rs` | 21 | 2（GUI） |
| session | `session_chat_tests.rs`、`session_direct_chat_tests.rs`、`session_filtered_chat_tests.rs` | 10 | 0 |
| transport／SQL | `transport_*chat*_tests.rs` 9 檔、`quest_filtered_chat_sql_interop_tests.rs` | 41 | 41 |

- 2 個 GUI ignored 測試是 `chat_filtered_window_tests.rs:22` 與 `direct_chat_window_tests.rs:438`，會開真的視窗，依 TASKS 併入 T034 清單，本張沒有執行。
- 41 個 transport／SQL ignored 測試需要 loopback 或資料庫，屬 #54 的分類，本張沒有執行。
- 78 個一般測試的 CI 結果：CI_PLACEHOLDER_MODERN

## 推測

- 客戶端階段和伺服器階段的對應（`LobbyReady`↔CHANNEL、`RoomReady`↔ROOM、`BattlePending`↔BATTLE、`WorldReady`↔WORLD）是依兩邊的流程推的，沒有逐一實測。對應錯的後果是伺服器不回應，不會斷線。
- 自己的一般聊天會由伺服器回音（依 LS 原始碼的成員清單；E149 的 `GameHub` 沒有另外 javap）。如果實機發現不回音，要改成本機先顯示。
- `\fX` 是顏色前綴、`$數字` 是字串表編號：前者依伺服器原始碼的用法（`\fT`、`\fW` 放在提示最前面），後者依守衛台詞的寫法（`$` 接訊息編號）。原版客戶端 `sub_4A3140` 的實際處理沒有反組譯。
- type 4（血盟）、11（隊伍）等其他聊天種類：伺服器 `say` 不分 type 一律照房間成員發送，所以送了也只是換標籤。本張的頻道下拉沒有列這些，收到時歸在「其他」。
- 原版 `LiTo.bin` 的聊天是畫在遊戲畫面內的訊息區與頭頂文字；本張是 Win32 控件面板，外觀與原版不同（和專案其他原生畫面一致，見 `docs/REUSE-MAP.md` §4.1）。

## 待驗證（需要實機，交給 Codex／使用者）

依 `docs/GUI-ACCEPTANCE.md` §17 的 Q01–Q12。重點：

1. 面板版面：按鈕位置有沒有和其他控件重疊、七個勾選框在最小視窗寬度（520）下排不排得下、高 DPI 下是否正常。版面座標是照既有面板的寫法推算的，沒有看過畫面。
2. 中文輸入法在訊息欄的組字與送出（Enter 選字時會不會誤送出）。
3. 兩個客戶端互相收發：一般、大喊、全體、買賣、密語；自己的訊息是否由伺服器回音。
4. NPC 說話：E149 實際會不會在競技場或租裝村送出 opcode 45（修理 NPC 出現台詞、守衛警告、任務提示），以及 `$數字` 台詞出現的頻率。
5. 取消勾選「全體／密語／買賣」後，伺服器是否真的不再傳送。
6. GM 指令（`.loc` 等）經「一般」頻道送出後，伺服器的回覆顯示在哪裡。
7. 舊 Modern profile 的 2 個 GUI ignored 測試（T034）。

## 沒有做的事

- 頭頂對話泡泡與遊戲畫面內的訊息區（原版外觀）。座標已經存在紀錄裡。
- `$數字` 轉文字（T049）、系統訊息（`S_MESSAGE`／`S_MESSAGE_CODE`）併入聊天紀錄。現在系統訊息仍只顯示在狀態列。
- 拒絕名單（`C_EXCLUDE`）、`C_ONOFF` 的 0xFF 合併值、血盟／隊伍聊天。
- 沒有改 `TASKS.md`、`docs/REPORT.md`。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_chat.rs`（新）
- `LiTo381-Modern/src/modern_lito/native_chat_window.rs`（新）
- `LiTo381-Modern/src/modern_lito/native_chat_controller_tests.rs`（新）
- `LiTo381-Modern/src/modern_lito/native_login.rs`、`native_login_window.rs`、`native_opcodes.rs`、`mod.rs`
- `docs/PROTOCOL-CONTRACT.md`、`docs/GUI-ACCEPTANCE.md`、`docs/reports/issue-342.md`

## 給下一棒的提示

- 實機驗收照 `docs/GUI-ACCEPTANCE.md` §17；如果自己的訊息沒有回音，改 `LoginController::chat_say`（`native_login.rs`），在送出成功後自己加一筆紀錄。
- 要把系統訊息或頭頂泡泡接進來時，資料都在 `Snapshot.chat`（`native_chat::Log`）：每筆有來源、type、說話者 ID、座標。
- T034 清單裡「一般聊天 8 格、大喊 50 格」的舊描述如果還出現在別處（例如 `TASKS.md` T060 的文字），請以本報告與 `PROTOCOL-CONTRACT.md` §6.2 為準。要不要讓伺服器照距離發送玩家聊天，是伺服器 repo 的決定，不是客戶端能做的。
- 重做 javap：`javap -classpath <JAR> -p -c -constants local.lito381.server.Protocol`（`ClientSession` 同理），找 `say`、`sayCode`、`tell`、`chatRequest`、`tellRequest`、`onOffRequest`、`acceptsChatType`、`changeChatPreference`。
