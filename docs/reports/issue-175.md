# Issue #175 — T043-2 雙玩家戰鬥生命週期核對

## 做了什麼

- 以 `318b3abc6ebf0cca96f941cdcd59dfdafc203fd9` 為工作基底，沿用 #174 已合併的 `TwoPlayerBattle` 與 `NativePeerHarness`，只擴充本張 `tests/` 與報告。
- 新增 opt-in 整合測試 `two_real_players_attack_vitals_death_settlement_and_reentry`，使用兩個正式 Rust `LoginController`，經加密 TCP 連到本次測試擁有的 loopback Java 記憶體伺服器。
- 兩端使用 `PeerFixture`／`TestKnight`（ID 1、force 0）、`PeerFixture2`／`TestMage`（ID 3、force 1）。來源為測試乙、第一個預設 DEATHMATCH 房間、numeric schema 0。
- 依正式地圖導航靠近，施放 Energy Bolt 核對 HP／MP；使用正式 `HeldAttack` 與實際經過時間持續普攻，等待 HP 歸零、雙端死亡、死亡確認及雙端結算；回大廳後同一連線重入同房，再次施法。
- 不注入封包或改寫 world／伺服器權威，不固定傷害或亂數，不加速動畫時鐘；讀取既有客戶端資源產生離屏 frame，沒有啟動遊戲視窗。
- 每個關鍵階段保存雙端 snapshot。所有建置、暫存、capture 與 JVM 設定都只在本 worktree；不改正式設定、不連正式服務／DB、不操作既有遊戲程序。

## 已驗證

- 唯讀正式參考 JAR SHA256：`E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- #174 已核對的 bot 政策持續適用：本測試每次進場都要求雙端看到 ID 1、3，沒有 bot ID 2。
- 本輪另以 `javap -p -c` 查看 E149 `ClientSession.confirmDeath(byte[])`：方法內 offset 39–51 呼叫擊敗判定，259–320 在 ended 時向房員送出結算並結束該場。此靜態核對不取代動態測試。
- 首次 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 0，耗時 3m 51s；既有 warning 保留於日誌。

### 最終建置與測試結果

| 項目 | Cargo 結束碼 | 原套件門檻 | 日誌（worktree 根目錄相對路徑） |
|---|---:|---|---|
| 最終 `cargo check --all-targets`（完整參數見下） | 0 | PASS，測試數 N/A，18.12s | `scratch/issue-175/check-final.log` |
| `reference_rejects_missing_file_and_legacy_fixture` | 0 | PASS，1 passed | `scratch/issue-175/reference-final.log` |
| `ready_contract_rejects_wrong_jar_missing_and_forbidden_ports` | 0 | PASS，1 passed | `scratch/issue-175/ready-final.log` |
| `two_real_players_enter_same_battle_without_training_bot` | 0 | PASS，1 passed | `scratch/issue-175/bootstrap-final.log` |
| **`two_real_players_attack_vitals_death_settlement_and_reentry`** | **0** | **PASS，1 passed** | `scratch/issue-175/lifecycle-attempt3.log` |

四次 `test result:` 原文，依上表測試順序：

```text
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 12.75s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 154.75s
```

每項均有 `running 1 test`，四個唯一測試全部實跑；filtered out 是各次 `--exact` 篩掉其餘三項，沒有把略過當成通過。沒有執行無關的完整本機測試套件。

### 雙玩家動態證據

成功的生命週期目錄：`LiTo381-Modern/target/current/t043-two-player/run-10476-1793947562956250300-0/`。以下數字是本次觀測，不是固定傷害／攻擊次數的測試假設。

| 階段／snapshot | Knight（ID 1）HP／MP | Mage（ID 3）HP／MP | 驗證重點 |
|---|---|---|---|
| `lifecycle-bootstrap.json` | 50/50、100/100 | 100/100、110/110 | 兩端 generation 1，真實對手存在、沒有 ID 2 bot |
| `spell-hp-mp.json` | 39/50、100/100 | 100/100、107/110 | Mage Energy Bolt 收到施法回覆；Knight 扣 11 HP，Mage 扣 3 MP |
| `melee-hurt.json` | 39/50、100/100 | 97/100、107/110 | 第一次普攻回覆，對手 HP 與遠端血量比例下降 |
| `death.json` | 50/50、100/100 | 0/100、107/110 | 61 次普攻回覆後雙端看見 Mage 死亡，Mage 收到確認要求 |
| `settlement.json` | 50/50、100/100 | 0/100、107/110 | 雙端 `battle_result=0`；Mage 恰好送出一次死亡確認 |
| `returned-lobby.json` | HP／MP 清空 | HP／MP 清空 | 兩端 `lobby_ready`，沒有舊 actors／room／result／scores |
| `reentered.json` | 50/50、100/100 | 100/100、110/110 | 同一 GAME epoch、同房 1，generation 2；死亡及戰鬥計數清空 |
| `reentry-spell-hp-mp.json` | 40/50、100/100 | 100/100、107/110 | 第二場重新靠近、再次施法，收到新的 HP／MP 及施法回覆 |

- 施法後與死亡階段的 `*-server.json`，將雙端 HP／MP 與 Java 唯讀觀測逐值交叉核對。獨立 session identity 為 `1979602185`、`2067375134`，不是共用同一玩家狀態。
- 第一場兩端計分表完全一致：`PeerFixture(測試乙)` 為 1 kills／0 deaths／0 assists／force `[0,0]`，`PeerFixture2(測試乙)` 為 0 kills／1 deaths／0 assists／force `[1,1]`；兩者 points 都是 0。測試比對正式 selected display name，沒有把內部 `TestKnight`／`TestMage` 當計分表名稱。
- 普攻期間 Knight MP 維持 100；伺服器自然回血保留，因此 Knight HP 從施法後 39 回到死亡階段的 50，不是測試補血。
- 排入施法／普攻請求時，不得本地預扣 HP／MP；死亡後攻擊／移動與結算後攻擊均必須拒絕且不增加 queued TX。死亡確認不可重複。
- LOGIN 為 `127.0.0.1:61650`、GAME 為 `127.0.0.2:61651`。`result.json` 為 `sessions_closed`、4 ClientSession／2 GAME sessions、`memberships_empty=true`、`authority_mutations=0`、`sql_access=false`、`state_persistent=false`；fixture 成功等待自有 JVM 結束並確認兩個 controller quiescent。
- 第二場完成動作後才取消連線；取消期間產生的斷線勝負不計入本張結算驗收。

### 修正前失敗與限制

| 嘗試 | Cargo 結束碼 | 判定／原因 |
|---|---:|---|
| `lifecycle-attempt1.log` | 101 | FAIL；沙箱內 S_KEY 前握手逾時，與 #174 同型。保留正式 5 秒 handshake timeout，經工具核准改沙箱外執行隔離測試 |
| `lifecycle-attempt2.log` | 101 | FAIL；已完成死亡與結算，但測試用內部角色名查計分表。修正為登入快照提供的正式顯示名稱 |

失敗 `test result:` 原文依序：

```text
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 9.94s
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 125.98s
```

沒有修改 runtime 來掩蓋失敗。普攻等待上限為 150 秒、攻擊上限 150 次；維持 fixture 原有 240 秒 lease。重入出生點超出 Energy Bolt 範圍，因此第二場也必須重新導航靠近。

建置仍有既有 warnings；離屏 Model 載入回報 Disintegrate／Call Lightning／Cone Cold 的既有 TW 資源資格不足。本張未使用這三項技能，沒有將其呈現標為通過。

## 怎麼驗證

測試使用 `維護工具/run-local-test-suite.ps1` 原始 `Measure-TestResult` 函式判定。以 PowerShell AST 只載入該函式，不執行整套 runner；每項以 `--exact` 獨立執行，連線測試明確加 `--ignored`。0 tests、failed > 0、解析失敗均不算通過。

以下在 worktree 根目錄準備環境，再進 Cargo 目錄執行：

```powershell
$env:PATH = 'D:\dev-cache\cargo\bin;' + $env:PATH
$env:JAVA_HOME = 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot'
$env:LITO_T043_REFERENCE_JAR = 'D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar'
New-Item -ItemType Directory -Force scratch/issue-175/tmp | Out-Null
$env:TEMP = (Resolve-Path scratch/issue-175/tmp).Path
$env:TMP = $env:TEMP
Set-Location LiTo381-Modern
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle reference_rejects_missing_file_and_legacy_fixture -- --exact --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle ready_contract_rejects_wrong_jar_missing_and_forbidden_ports -- --exact --nocapture
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle two_real_players_enter_same_battle_without_training_bot -- --exact --nocapture --ignored
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_two_player_battle two_real_players_attack_vitals_death_settlement_and_reentry -- --exact --nocapture --ignored
```

## 推測

- 此測試可作為 T028、T029、T048 後續戰鬥測試的最小基礎；不代表這些任務已驗收。
- 沙箱內握手逾時可能與執行環境有關；沙箱外能完成相同流程，尚未獨立定位底層原因。

## 待驗證／未驗證什麼

- schema37 的 54 個舊情境、numeric schema 1–4、其他技能／職業／房間模式／來源路由。
- 正式 SQL 持久化、跨程序重連、網路失敗恢復、完整原生 GUI、聲音及人工輸入。
- 本張「重入」指雙方正常返回大廳後，以原 GAME session 加入同房開啟新戰鬥。

## 產出路徑

- `LiTo381-Modern/tests/native_two_player_battle.rs`
- `LiTo381-Modern/tests/support/two_player_lifecycle.rs`
- `docs/reports/issue-175.md`
- 本機未追蹤證據：`scratch/issue-175/`、`LiTo381-Modern/target/current/t043-two-player/`。

## 給下一棒的提示

- 請 Claude 審查本張測試與實跑證據；由使用者決定合併，調度器合併後加 `docs/REPORT.md` 索引。本張未修改該索引或 TASKS 狀態。
- 後續本機執行者可重用 #174 fixture。`Model::load` 僅唯讀使用 `D:\lineage開發\0-客戶端`，測試不依賴遊戲視窗；lease 維持原有 240 秒。
- 不將本張測試通過外推成全部 M2／T043 舊情境通過。
