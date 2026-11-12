# Issue #174 — T043-1 雙真實玩家戰鬥 fixture

## 範圍與做了什麼

- 本張交付 E149 `startBattle` 靜態核對，以及雙玩家進入同場戰鬥的可重用連線 fixture。
- 工作基底為 `de46199`，分支為既有的 `task/174`。所有新產物寫入此 worktree。
- 新增 `tests/support/two_player_battle.rs`，重用既有 `tests/oracle/NativePeerHarness.java`。兩個獨立 `LoginController` 分別以記憶體測試帳號登入，經正式加密 TCP、選來源、選角、進頻道、同房、對立陣營與 ready 進入戰鬥。
- 固定採既有 `melee` profile、來源索引 1（測試乙）、角色索引 0、頻道 0 與第一個預設房間。這些是本張選定的最小 fixture 條件。
- JVM 明確指定 `lito.numeric.breakthrough=0`，對齊當版 `native_status_receipt::decode` 的 37-byte S_STATUS。E149 的 numeric schema 4 曾實跑並被此解碼器拒絕，本張交付涵蓋 schema 0。這個 numeric schema 與歷史 schema37 伺服器版本標記是不同欄位。
- 自動編譯並啟動自有 Java harness。JAR 必須符合完整 E149 SHA256。LOGIN 固定 loopback `127.0.0.1`，GAME 固定 `127.0.0.2`，連接埠由 harness 綁定 0 取得。GAME 轉址必須等於本次 ready 宣告的端點。
- 重用 Java harness 的 memory repository、唯讀 DB 模式、停用 state persistence、worktree 內設定及 capture 路徑。`GameHub` 使用相同來源的 `MemoryAccountRepository`，沿用 E149 明確的記憶體 fixture 分支。
- 每次執行建立全新目錄，排除舊 ready／result 重用。Rust 用持有的 `Child` 清理自有 JVM，正常結束等待 Java 清除 session／membership，失敗或 unwind 時取消兩個控制器並回收自有 JVM。
- 新增 `tests/native_two_player_battle.rs`，包含 reference／ready 負向門檻與 opt-in 真實連線測試。

## 已驗證

### E149 的 bot 判定

唯讀核對的檔案為 `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`。

SHA256 為 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。

E149 的 `startBattle` 使用條件式 bot 政策。`NATIVE-EMBLEMS` 所記載 EBF5EE66 的強制關閉結論，無法直接套用到 E149。

| 類別／方法 | javap 方法內 offset | 實際位元碼行為 |
|---|---|---|
| `ClientSession.startBattle()` | 448–504 | 計算舊模式／人數條件後 `pop` |
| 同上 | 505–506 | `iconst_0; pop`，這個 false 沒有存入後續使用的 local |
| 同上 | 507–511 | 呼叫 `NativeRoomRules.activeBot(this)`，結果存入 local 7 |
| 同上 | 1011–1063 | local 7 為 true 時送出 ID 2、名稱 `ArenaBot` 的物件，false 時跳到 1066 |
| `NativeRoomRules.bot(Mode,GameHub,long)` | 0–30 | 同時要求 DEATHMATCH、`testBotEnabled`、`testRoom` |
| `NativeRoomRules.testRoom(GameHub,long)` | 12–73 | 房間 ID 必須等於某頻道的 `defaultRoomId` |
| `NativeRoomRules.activeBot(GameHub,long)` | 4–47 | `bot(...)` 為 true、`roomCombatantCount == 1` 且 `roomVirtualPlayers.isEmpty()` 才回傳 true |
| `NativeRoomRules.install(...)` | 0–20 | memory repository／null 的 fixture 政策為 `Policy(true,true)` |

雙真實參戰者使 `roomCombatantCount == 1` 條件不成立。本張的連線驗證另外檢查兩端 World 都包含真實角色 1 與 3，且不含 bot ID 2。

重做靜態核對可在 worktree 根目錄執行下列命令。輸出只進 `scratch/`。

```powershell
$jarPath = 'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
Get-FileHash -LiteralPath $jarPath -Algorithm SHA256
New-Item -ItemType Directory -Force scratch/issue-174 | Out-Null
javap -classpath $jarPath -p -c local.lito381.server.ClientSession |
    Set-Content -Encoding UTF8 scratch/issue-174/ClientSession.javap.txt
javap -classpath $jarPath -p -c local.lito381.server.NativeRoomRules |
    Set-Content -Encoding UTF8 scratch/issue-174/NativeRoomRules.javap.txt
```

### 建置與測試

環境為 Windows x64、Rust／Cargo 1.95.0、Microsoft OpenJDK 17.0.17。`JAVA_HOME` 指向 `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot`，compiler 與 runtime 固定取自同一個 JDK。Java 輸出指定 UTF-8，Windows verbatim 路徑轉為 JDK 支援的本機磁碟路徑。

最終 check 與三個測試均完成。每個測試以 `--exact` 獨立執行，套用現有 `維護工具/run-local-test-suite.ps1` 的 `Measure-TestResult -Kind Exact` 原函式，三者均為 PASS。只擷取此函式做判定，完整本機套件留給其專屬任務。

| 最終執行項目 | 結束碼 | 門檻／結果 |
|---|---:|---|
| `cargo check --all-targets`，含下列共通參數 | 0 | `Finished dev ... in 4.20s`，check 的測試數為 N/A |
| `reference_rejects_missing_file_and_legacy_fixture` | 0 | 1 passed，0 failed，0 ignored，PASS |
| `ready_contract_rejects_wrong_jar_missing_and_forbidden_ports` | 0 | 1 passed，0 failed，0 ignored，PASS |
| `two_real_players_enter_same_battle_without_training_bot`，明確帶 `--ignored` | 0 | 1 passed，0 failed，0 ignored，PASS |

最終三次 `test result:` 原文依上表順序如下。

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.02s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 12.91s
```

每次皆有 `running 1 test`。三個唯一測試全數實跑，filtered out 表示該次只選一項，沒有把略過項目計為通過。最初全新 target 的 check 亦以結束碼 0 完成，耗時 3m 37s。既有編譯警告留在日誌中。

### 實際雙玩家證據

最終成功執行目錄為 `LiTo381-Modern/target/current/t043-two-player/run-6864-1793943760122231800-0/`。

- ready 宣告 LOGIN `127.0.0.1:54349`、GAME `127.0.0.2:54350`，JAR hash 等於 E149。
- `server-peers.json` 觀測到獨立 GAME session identity `2067375134` 與 `1979602185`，帳號分別為 `PeerFixture`、`PeerFixture2`，角色為 `TestKnight`／ID 1、`TestMage`／ID 3。兩者皆為 `BATTLE`、room 1、observer false、來源測試乙。
- `bootstrap.json` 的雙方 World 都是 map 564，皆包含 actor 1 與 3，皆無 actor 2。控制器的 own ID 分別為 1 與 3，階段皆為 `battle_pending`，world ready 且已收到 integrity challenge。
- 正常清理後 Java 結束碼為 0。`result.json` 為 `sessions_closed`、4 個 ClientSession（2 LOGIN＋2 GAME）、`memberships_empty=true`、`authority_mutations=0`、`sql_access=false`、`state_persistent=false`。兩個 Rust controller 都確認 `is_quiescent()`。
- 執行後再次計算唯讀 JAR SHA256，仍為上列 E149。

### 重現指令

在本 worktree 根目錄準備環境後，進入 Cargo 目錄執行。

```powershell
$env:PATH = 'D:\dev-cache\cargo\bin;' + $env:PATH
$env:JAVA_HOME = 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot'
$env:LITO_T043_REFERENCE_JAR = 'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
New-Item -ItemType Directory -Force scratch/issue-174/tmp | Out-Null
$env:TEMP = (Resolve-Path scratch/issue-174/tmp).Path
$env:TMP = $env:TEMP
Set-Location LiTo381-Modern
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle reference_rejects_missing_file_and_legacy_fixture -- --exact --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle ready_contract_rejects_wrong_jar_missing_and_forbidden_ports -- --exact --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle two_real_players_enter_same_battle_without_training_bot -- --exact --nocapture --ignored
```

本機實跑連線測試使用工具核准的沙箱外程序。沙箱內曾接受 TCP 但在送出 S_KEY 前逾時，該輪失敗保留，沒有放寬 controller 的 5 秒 handshake timeout。使用 PowerShell 暫存驗收腳本時，僅對該次程序使用 `-ExecutionPolicy Bypass`，腳本存為 UTF-8 BOM。

### 修正前的失敗紀錄

下列四次連線測試的 Cargo 結束碼均為 101，Measure-TestResult 均拒絕通過。各輪錯誤走 Drop 清理路徑。對應的 fixture 目錄與日誌保留於 target／scratch。

| 嘗試 | 原因與處置 | 原始日誌 |
|---|---|---|
| 1 | JDK 拒絕 `\\?\` 路徑，加入本機路徑轉換及回歸檢查 | `scratch/issue-174/initial-jdk-path-failure.log` |
| 2 | PATH 的 java 指向舊 Oracle shim，改為 JAVA_HOME 同一 JDK 的 java／javac | `scratch/issue-174/java-path-resolution-failure.log` |
| 3 | 沙箱內握手逾時，經工具核准在沙箱外重跑 | `scratch/issue-174/sandbox-handshake-failure.log` |
| 4 | numeric schema 4 的 S_STATUS 被當版 schema0 解碼器拒絕，fixture 明確對齊 schema0 | `scratch/issue-174/numeric-schema-failure.log` |

四次失敗的 `test result:` 原文依上表順序如下。

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.30s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.59s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 8.28s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out; finished in 10.04s
```

## 推測

- 靜態條件允許 E149 在符合政策的單人預設 DEATHMATCH 房間啟用訓練 bot。單人動態場景留待另案核對。
- 正式服務實際使用的房間政策取決於它的資料與設定，本張僅核對 JAR 位元碼及隔離 memory fixture。
- 沙箱內的 S_KEY 前逾時可能與 Windows 執行環境有關。沙箱外重跑能完成握手，底層阻塞原因未單獨定位。

## 待驗證

- T043 第 2 張的攻擊、HP／MP 變化、死亡、結算與重入，以及 T028／T029／T048 的後續戰鬥驗證。
- schema37 的 54 個舊情境仍待新版逐案核對。
- integrity challenge 的後續核驗流程、numeric schema 1–4、畫面、音效、完整 GUI 與正式 SQL／跨來源路由的驗收。
- 本張測試在雙方進場後取消連線，清理過程出現的勝負／退場事件未納入戰鬥結算驗收。

## 產出路徑

- `LiTo381-Modern/tests/support/two_player_battle.rs`
- `LiTo381-Modern/tests/native_two_player_battle.rs`
- `docs/reports/issue-174.md`
- 本機未追蹤證據位於 `scratch/issue-174/` 及 `LiTo381-Modern/target/current/t043-two-player/`。每次 fixture 目錄包含編譯日誌、server stdout／stderr、ready、capture、server trace、雙端 bootstrap、server peers 與正常清理 result。

## 給下一棒的提示

- T043 第 2 張由本機執行者接手。以 `#[path = "support/two_player_battle.rs"]` 引入 helper，依序呼叫 `TwoPlayerBattle::start()` 與 `enter_battle()`，使用 `players[0]`、`players[1]` 的正式 controller API 進行戰鬥。等待時呼叫 fixture 的 `poll()`，持續服務雙方連線，結束時呼叫 `close()`。
- `save_snapshots()` 可保存兩端觀測值。角色對照為 `PeerFixture` → ID 1、`PeerFixture2` → ID 3，兩者皆選測試乙。每次重跑都從全新 memory server 開始。
- 現有 Java `melee` harness 整體 lease 為 240 秒。新增較長情境時需明確處理此上限，並核對戰鬥所需的 integrity 流程及導航條件。保留已明列的 numeric schema 0 範圍。
- 沿用 E149 條件式 bot 政策結論。不要把舊 schema37 的情境通過狀態繼承到新測試。
- 調度器於合併後加入 `docs/REPORT.md` 索引。
