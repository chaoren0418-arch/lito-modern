# Issue #81 — T011-B：當版 G06 本機執行結果

## 結論

**已驗證：首案 `timeout` 實跑一次並失敗；其餘四案依首個失敗即停止規則未執行。通過 0、失敗 1、阻塞 0、未執行 4。不得視為 G06 通過。**

- 執行日期：2026-11-06（Asia/Taipei）；本次實跑為 06:12:38–06:14:10。
- 分支：既有 `task/81`，未 checkout 或建立分支。起點與本機 `origin/main` 均為 `fde15d48bb77ba4f1784edb8e1eddee377cdcb41`。
- 工作目錄（以下簡稱 `W`）：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81`。
- 已讀 AGENTS.md、docs/REPORT.md、TASKS.md 的 T011、#91 報告及當版 runner／測試／harness。
- 唯一納入 git 的變更為本報告；未改 src、tests、runner、docs/REPORT.md。輔助腳本及證據只寫入此 worktree 的 scratch／target/current。
- 未連正式服務或正式 DB，未啟動原生 GUI、操作既有遊戲程序、修改原始專案或正式設定。僅讀取指定正式 JAR 並驗證後複製；未碰封存與硬連結。

## 各案結果與耗時

| 順序／scenario | WhatIf | 實跑結果 | 實際耗時 | 原因與證據 |
|---|---|---|---|---|
| 1／timeout | 結束碼 0，1.0006268 秒；JSON 已保存 | **失敗（已驗證）**；一次；runner 結束碼 1、Rust 101 | runner 92.0109059 秒；Rust 自報 65.79 秒 | `running 1 test`；`0 passed; 1 failed; 0 ignored`；收據見下節 |
| 2／refused | 未執行 | 未執行（待驗證） | — | 首案失敗後停止，沒有重試或另跑 WhatIf |
| 3／wrong-service | 未執行 | 未執行（待驗證） | — | 同上 |
| 4／cancel-retry | 未執行 | 未執行（待驗證） | — | 同上 |
| 5／cancel-expiry | 未執行 | 未執行（待驗證） | — | 同上 |

`timeout` 的啟動前守門已完成，Java 與 Rust 都實際啟動，因此本案是測試失敗，不是 Rust 啟動前的阻塞。沒有自動重試、跳過守門、手動刪除 lease 或預建 runtime。

## 前置環境與建置（已驗證）

原始 PATH 的 `Get-Command pwsh -ErrorAction SilentlyContinue` 無結果，Cargo／Rustc 亦不在原始 PATH。僅對本次程序補入既有 Rust 路徑與 worktree 內 PowerShell 路徑，沒有永久設定變更。

| 項目 | 實際值 |
|---|---|
| 補入 PATH 後 `(Get-Command pwsh).Source` | `W\scratch\tools\pwsh\pwsh.exe` |
| PowerShell | `7.6.5`；官方 `https://github.com/PowerShell/PowerShell/releases/download/v7.6.5/PowerShell-7.6.5-win-x64.zip` 解壓於 worktree |
| Authenticode | `Valid`；`CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US` |
| JDK／javac | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin`；OpenJDK `17.0.17+10-LTS`，Microsoft-12574423；javac `17.0.17` |
| Cargo | `D:\dev-cache\cargo\bin\cargo.exe`；`cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Rustc | `D:\dev-cache\cargo\bin\rustc.exe`；`rustc 1.95.0 (59807616e 2026-04-14)` |
| Cargo／Rustup home | `D:\dev-cache\cargo`／`D:\dev-cache\rustup`，沿用既有值 |
| 建置與外層執行的 TEMP／TMP | `W\scratch\issue-81\tmp`；runner 子程序再使用各自 run 的 tmp |

PowerShell 下載前，沙箱內 Invoke-WebRequest 遇到 TLS 接收錯誤，curl 遇到 `SEC_E_NO_CREDENTIALS`；改由工具權限核准下載同一官方 ZIP，成功且簽章有效。這兩次僅為工具準備，並未呼叫 scenario。依 #91 已確認的 Get-NetTCPConnection 權限需求，唯一一次實跑也由工具權限核准執行，所有 listener 守門保持原樣。

建置工作目錄為 `W\LiTo381-Modern`，實際指令：

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-run
```

結束碼 0，耗時 **283.1353052 秒**（UTC `2026-11-05T22:07:21.2079584Z`–`22:12:04.3633460Z`）。Cargo 顯示 296 個既有 lib warnings（含 10 duplicates），沒有改碼處理。此次為本 worktree 直接建置；未使用其他 worktree 的 binary，因此 CARGO_MANIFEST_DIR 與 runner 的 target/current 對齊。

| 檔案 | 路徑 | SHA256 |
|---|---|---|
| 新建測試 exe | `W\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe` | `F31F9057DBC446ADFDECCACFFCF9928838907D97BF4BE1F56D0C87055703632F` |
| 正式唯讀 JAR | `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar` | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| worktree JAR 副本 | `W\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar` | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |

正式檔先驗雜湊，建置完成後複製前再驗一次，相符才複製，再驗副本；複製與該輪雙重雜湊驗證共 **3.1819099 秒**。runner 又將 exe 複製成當次 `immutable-test.exe` 並綁定雜湊，finally 確認來源、class、JAR、原始 exe、immutable exe 均無漂移。

## 完整指令與原始證據位置

以下兩次的 cwd 都是 `W`，本次環境 `TEMP`／`TMP` 為 `W\scratch\issue-81\tmp`。每條只執行一次；第二條亦是供下一個核准測試批次使用的重現指令，本次沒有再次執行。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout'
```

首個失敗的**完整原始收據**：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\native-teleport-failure\20261105T221238930Z-943ea06b6cd04dfea710d095ab8da709\receipt.json
```

驗收輸出副本目錄（以下簡稱 `V`）：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-81\LiTo381-Modern\target\current\verification\native-teleport-failure\20261105T221238930Z-943ea06b6cd04dfea710d095ab8da709
```

| 證據 | 位置 |
|---|---|
| PowerShell 簽章、工具路徑版本 | `W\scratch\issue-81\pwsh-signature.json`、`preflight.json` |
| 建置指令、時間、結束碼與雜湊／原始輸出 | `W\scratch\issue-81\build.json`、`build.stdout.txt`、`build.stderr.txt` |
| 正式 JAR 與副本雜湊 | `W\scratch\issue-81\jar-source.json`、`jar-copy.json` |
| WhatIf 原始 JSON | `W\scratch\issue-81\timeout.whatif.stdout.txt`（JSON 內容，保留原始輸出） |
| WhatIf 指令、耗時與結束碼 | `W\scratch\issue-81\timeout.whatif.json`；stderr 同前綴 `.stderr.txt` |
| 實跑指令、耗時、結束碼、PID／StartTime 殘留核對 | `W\scratch\issue-81\timeout.run.json`；原始 stdout／stderr 同前綴 `.stdout.txt`／`.stderr.txt` |
| runner 收據副本、Java／Rust／javac 日誌 | `V\receipt.json`、`V\{java,rust,javac}.{stdout,stderr}.txt` |
| ready、server trace、staged trace | `V\ready.json`、`V\server-trace.jsonl`、`V\staged-trace.jsonl` |
| 角色 A／B 原始錯誤快照 | 原始收據同層 `runtime\a-recovery-error.json`、`runtime\b-recovery-error.json`；runner 不會將這兩檔複製至 V |

以上原始證據及輔助執行腳本留在本機忽略目錄，未加入 git；請保留此 worktree，供下一棒讀取。

## 實跑驗收與退出鏈（已驗證）

Rust 原始輸出：

```text
running 1 test
test destination_timeout_retires_old_authority_and_allows_fresh_login ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 4 filtered out; finished in 65.79s

a=Err(unqualified current TW disintegrate namespace); b=Err(timeout at BattlePending)
```

runner 收據 `status=failed`、`error=Server cleanup did not complete`。Rust 自然退出後，Java 未於額外 8 秒內退出；finally 依保留的 process handle 與 PID／StartTime／image 身分核對終止自有 Java。這是失敗後完成清理，不能算正常 server cleanup 通過。

| 程序 | PID | StartTime UTC | 結束碼 | killed | WaitForExit | identity_bound／cleanup_errors |
|---|---:|---|---:|---|---|---|
| javac | 16480 | `2026-11-05T22:12:40.2135439Z` | 0 | false | true | true／`[]` |
| Java harness | 18812 | `2026-11-05T22:12:41.8202982Z` | -1 | true | true | true／`[]` |
| Rust | 7428 | `2026-11-05T22:12:46.3731486Z` | 101 | false | true | true／`[]` |

實跑結束後另依收據的三組 PID／StartTime 核對，三者均 `not_running`；外層紀錄見 `timeout.run.json`。收據頂層 `cleanup_errors=[]`、`drift_errors=[]`、`inputs_unchanged=true`；runner 自行完成 lease 釋放，檢查 `lease_remains=false`，沒有手動移除。

三個 listener 均核對屬於 ready 所載 Java PID 18812：login `127.0.0.1:52646`、town `127.0.0.2:52647`、destination `127.0.0.3:52648`。均非 2000／3306，非低埠；三筆查詢成功後 Rust 才啟動。WhatIf 不啟動 Java／Rust；建置只用了 `--no-run`。

E149 在 memory 模式下初始化的實際證據：

- `ready.json` 綁定 PID 18812 與 `jar_sha256=e1498871c7be6b287821c57ef5d95933db1740d9b4d4a80f6c3465f21d99d498`，與已驗證副本一致。
- 本次 harness **新建的隔離 runtime** 中 `server.properties` 記錄 `database.mode=memory`、`database.read.only=true`、`state.persistence.enabled=false`；這是測試產生的設定，未讀改正式 `config/server.properties`。當版 harness 在建立 GameHub／ready 前亦檢查這些值。
- `V\java.stdout.txt` 實際記錄兩名角色的 `loaded state`、`selected character`、`rental-town scene bootstrapped map=500` 與 `joined rental-town room=60001`，證明走過登入、角色與場景初始化；Java stderr 為空。
- Java 日誌的 `persisted ... disconnect` 字樣本身不能當成磁碟持久化證據；本次配置明確停用 state persistence，但沒有完成最終 result 驗收。

**`result.json` 未產生**，因此 sessions（timeout 應為 8）、memberships、authority_mutations、staged loadouts、full_game_reentry，以及 result 內 memory／非持久化／JAR 綁定全部未通過最終驗收。不能以初始化成功取代這些條件。

## 推測、未驗證事項與給下一棒的提示

已驗證的直接錯誤定位是 `LiTo381-Modern/src/modern_lito/disintegrate_assets.rs:207` 的 `require_current_profile`：同時要求 pack／text 雜湊符合 CURRENT_PACK／CURRENT_TEXT、decoded=5,974,928、definitions=4。A、B 錯誤快照均停在 `battle_pending`、transport_epoch=2。此次沒有修改資格檢查或資源，也未重跑。

**推測**：A 在資源資格檢查時中止，B 隨後等待 A 的完成旗標至 BattlePending 逾時；應先查本機唯讀資源與當版 profile 契約是否一致。錯誤訊息沒有揭露上述四個條件何者不符，不能直接判定某一檔案損毀，亦不能判定傳送 timeout／恢復邏輯本身有缺陷。

**待驗證**：timeout 的實際傳送／恢復行為、8 sessions 與最終狀態清理；其餘四個 scenario 的全部驗收；Java 未自行退出是否完全由 Rust 前置失敗導致。沒有完成五案、沒有擴大測試，也沒有連線／GUI 驗收。

下一棒請由 **Claude 雲端審查本次失敗證據並派 code／resource 任務，本機執行者查核資源 profile 契約**；釐清或修正後，再由調度器另排 T011-B 測試批次。保留本次 evidence；不要放寬守門、替換假資源或把未執行算通過。本任務只交付報告，不自行修正 src／tests／腳本或自動重試。

產出路徑：`docs/reports/issue-81.md`。
