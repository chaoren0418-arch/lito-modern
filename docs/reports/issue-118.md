# Issue #118 — T011-B 第三次派工：五案實跑結果

**已驗證：五個 scenario 依 timeout → refused → wrong-service → cancel-retry → cancel-expiry 各實跑一次，全部通過；0 失敗、0 阻塞、0 未執行。每案 Rust 都是 `running 1 test`、`1 passed; 0 failed; 0 ignored`。所有 receipt 的 `cleanup_errors=[]`、`drift_errors=[]`，Java／Rust／javac 均正常退出，沒有 kill refused。**

## 做了什麼／版本與安全範圍

- 日期：2026-11-06（Asia/Taipei；下列原始時間使用 UTC）。工作目錄 `W = D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118`。
- 沿用既有 `task/118`，未 checkout／建立分支。開工實際 HEAD 為 `910fd29`；fetch 並以 `git merge --ff-only origin/task/118` 快轉至 `5e74ab5ff0316422d524b0d6fa4d8a9ad53a624b`，保留兩次交付歷史。
- 遠端 fetch 後的 main 為 **`aac969e1a9f1ec883966b1cfdc71349e4cb403c4`**，包含 PR #115（`2233fe9315bebf08c137bbb5a613ae9988244ded`）與 PR #124（`f99d315ba1a5e11d0314ab4f482c277051b47fbb`）。main 為測試 HEAD 的 ancestor；`LiTo381-Modern`、`維護工具` 與該 main 無差異。
- 已讀 AGENTS.md、docs/REPORT.md、docs/TESTING.md、TASKS.md 的 T011、相關歷次報告，以及 runner／harness／Rust 測試。同步上游曾帶入已合併檔案；本輪新增提交只更新本報告，未自行修改 src、tests、既有腳本或 docs/REPORT.md。
- 所有本輪輸出寫在 W 的 scratch／target/current；原始專案及正式 JAR 僅讀取。未讀改機密設定、封存或硬連結，未開 GUI、未操作既有遊戲／正式伺服器、未連正式服務或 DB。
- 依本次明確授權，事前／事後程序核對使用經工具核准的本機 `Get-Process`；五次 runner 實跑亦經工具核准使用本機權限，保留內部 `Get-NetTCPConnection` 與所有守門。未使用 CIM／WMI、未改 ACL 或安全設定、未手動刪除 lease／預建 runtime。
- 事前既有 Java：PID **18784**、StartTime **`2026-11-06T02:13:27.5653514Z`**、Path `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\java.exe`，未對其操作。事前 runner.lease 不存在。

## 保留前兩次交付摘要

| 提交 | 當次結果與證據 |
|---|---|
| [`910fd29e4bf6036232581cc0759ff11a7d02cd66`](https://github.com/chaoren0418-arch/lito-modern/commit/910fd29e4bf6036232581cc0759ff11a7d02cd66) | 第一次 timeout 在 Rust 前讀 Java MainModule.FileName 失敗，Java 身分未綁定；阻塞 1、其餘 4 未執行。Java PID 12996 的 `cleanup_errors` 為 `Unbound process identity: kill refused`（頂層重複兩筆），WaitForExit=false、退出碼未取得；後續觀測已不存在，不能回填成清理成功。原始 receipt：`W\LiTo381-Modern\target\current\native-teleport-failure\20261105T234804138Z-9d5455933d0840fa9e5d8e9e9dac37fb\receipt.json`。完整歷史報告保存在此 commit。 |
| [`5e74ab5ff0316422d524b0d6fa4d8a9ad53a624b`](https://github.com/chaoren0418-arch/lito-modern/commit/5e74ab5ff0316422d524b0d6fa4d8a9ad53a624b) | 第二次在 `task-118-r2` 的前置 `Get-CimInstance Win32_Process` 遇到拒絕存取，五案均未建置／執行，無 receipt、無當輪 SHA。交付曾遭 Git dubious ownership 拒絕；本次 fetch 已實際取得遠端該提交，交付歷史已保留。這次授權明確改用 Get-Process，沒有重試 CIM。 |

舊結果不計入本輪五案；本輪沒有自動重試任何 scenario。

## 前置工具、建置與 SHA256（已驗證）

原始 PATH 找不到 pwsh／cargo／rustc。僅對本次程序補 PATH，使用 W 內既有官方可攜 PowerShell，未重新下載或安裝系統工具。

| 項目 | 實測 |
|---|---|
| `(Get-Command pwsh).Source` | `W\scratch\tools\pwsh\pwsh.exe` |
| PowerShell | `7.6.5`；Authenticode `Valid`；簽署者 `CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US` |
| JDK 17 | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin`；Java `17.0.17+10-LTS`、Microsoft-12574423；javac `17.0.17` |
| Cargo | `D:\dev-cache\cargo\bin\cargo.exe`；`cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Rustc | `D:\dev-cache\cargo\bin\rustc.exe`；`rustc 1.95.0 (59807616e 2026-04-14)` |
| Homes | `CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup` |
| 外層 TEMP／TMP | `W\scratch\issue-118-r3\tmp`；runner 子程序另使用各自 run 的 tmp |

建置 cwd 為 `W\LiTo381-Modern`，本輪重新執行指定 no-run 建置，結束碼 **0**，耗時 **6.6634119 秒**，UTC `02:30:55.1549251Z`–`02:31:02.4190970Z`。Cargo 使用本 worktree 的增量產物；輸出 exe 雜湊與第一次相同，未借用其他 worktree 的 binary，也不宣稱進行過 clean build。`CARGO_MANIFEST_DIR` 與 runner 均對應本 W 的 `target/current`。既有 296 warnings 未改碼處理。

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-run
```

| 檔案 | 完整位置／SHA256 |
|---|---|
| 測試 exe | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe` |
| exe SHA256 | **`693F7DBD1E868E9C1C7517B0B118D46C7EC6ACA858B363032E9951ED7B8EE17C`** |
| 正式 JAR | `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar` |
| JAR 副本 | `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar` |
| 正式／副本 SHA256 | **`E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`**，來源先驗相符才複製，再驗副本；共 0.6026459 秒 |

## 五案結果與耗時

每案 WhatIf 與實跑結束碼均為 **0**。表中實跑秒數包含 runner 啟動、javac、Java、Rust、清理及外層輸出收集；Rust 秒數取自原始 Rust stdout。每案 `4 filtered out` 是 `--exact` 選取其他四案的預期結果，不把 filtered out 計為通過。

| scenario | 結果 | WhatIf 秒 | 實跑秒 | Rust 秒 | sessions | cleanup_errors | drift_errors |
|---|---|---:|---:|---:|---:|---|---|
| timeout | 通過（已驗證） | 0.8170811 | 86.5866042 | 72.37 | 8 | `[]` | `[]` |
| refused | 通過（已驗證） | 0.6926186 | 72.2316348 | 64.26 | 8 | `[]` | `[]` |
| wrong-service | 通過（已驗證） | 0.6547248 | 84.8987694 | 77.82 | 10 | `[]` | `[]` |
| cancel-retry | 通過（已驗證） | 0.8126596 | 70.8919763 | 62.98 | 8 | `[]` | `[]` |
| cancel-expiry | 通過（已驗證） | 0.6950347 | 146.6777223 | 140.24 | 8 | `[]` | `[]` |

全部 result.json 均為 `status=sessions_closed`、`memberships_empty=true`、`authority_mutations=0`、`staged_loadouts_after_reentry=0`、`sql_access=false`、`state_persistent=false`、`full_game_reentry_verified=true`，JAR SHA 與 E149 相同。每案來源／class／原 exe／immutable exe／JAR 漂移檢查均通過。

## 完整 WhatIf／實跑指令與收據

以下每條指令均在 W 執行且只執行一次。WhatIf 僅預覽，因此其 JSON 中的隨機 run 路徑不等於隨後實跑路徑；實跑收據以各段列出的實際路徑為準。沒有首個失敗或阻塞，無須中止後續案。

### timeout

- WhatIf UTC：`2026-11-06T02:31:51.8841488Z`–`2026-11-06T02:31:52.7226588Z`；實跑 UTC：`2026-11-06T02:32:04.5257853Z`–`2026-11-06T02:33:31.1456440Z`。
- 完整原始 receipt：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261106T023205327Z-5213af1f4a9e4e9d82d196da074f6d8e\receipt.json`。
- 驗收副本目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261106T023205327Z-5213af1f4a9e4e9d82d196da074f6d8e`。
- 此 receipt 的頂層及 javac／Java／Rust 各自 `cleanup_errors=[]`，`drift_errors=[]`。
- receipt SHA256：`BF3E90110BC62634EE4E26BBF9085F12FDD338BBCAE9E53EA570B27C33C7AEB4`；R／V 副本相同。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout'
```

### refused

- WhatIf UTC：`2026-11-06T02:33:43.3372922Z`–`2026-11-06T02:33:44.0532688Z`；實跑 UTC：`2026-11-06T02:33:54.4426466Z`–`2026-11-06T02:35:06.7028530Z`。
- 完整原始 receipt：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261106T023355234Z-a68ceec6ba474248ae7c9dc797d5aec5\receipt.json`。
- 驗收副本目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261106T023355234Z-a68ceec6ba474248ae7c9dc797d5aec5`。
- 此 receipt 的頂層及 javac／Java／Rust 各自 `cleanup_errors=[]`，`drift_errors=[]`。
- receipt SHA256：`52B991DEF4418F26272B77DB7E7E99BDEDB0280B99258B397F1020A3B4A790B5`；R／V 副本相同。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'refused' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'refused'
```

### wrong-service

- WhatIf UTC：`2026-11-06T02:35:17.3364332Z`–`2026-11-06T02:35:18.0106801Z`；實跑 UTC：`2026-11-06T02:35:29.3483600Z`–`2026-11-06T02:36:54.2676774Z`。
- 完整原始 receipt：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261106T023529990Z-30bd2c63812946688d6ddd996c6d4742\receipt.json`。
- 驗收副本目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261106T023529990Z-30bd2c63812946688d6ddd996c6d4742`。
- 此 receipt 的頂層及 javac／Java／Rust 各自 `cleanup_errors=[]`，`drift_errors=[]`。
- receipt SHA256：`05C4CCE62B619EE343D627794469076BB319C4DF3D3FB77CC4FFF15C9BD7B713`；R／V 副本相同。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'wrong-service' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'wrong-service'
```

### cancel-retry

- WhatIf UTC：`2026-11-06T02:38:40.6359993Z`–`2026-11-06T02:38:41.4744969Z`；實跑 UTC：`2026-11-06T02:38:52.9729168Z`–`2026-11-06T02:40:03.8954533Z`。
- 完整原始 receipt：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261106T023853673Z-50bd3d50879541e5bc1c7e74dea19d01\receipt.json`。
- 驗收副本目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261106T023853673Z-50bd3d50879541e5bc1c7e74dea19d01`。
- 此 receipt 的頂層及 javac／Java／Rust 各自 `cleanup_errors=[]`，`drift_errors=[]`。
- receipt SHA256：`B53A15A849E9BCCBFDEC7BEA2B04BF2519DBACA3C6FF16208783CD47EFDCD1CC`；R／V 副本相同。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'cancel-retry' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'cancel-retry'
```

### cancel-expiry

- WhatIf UTC：`2026-11-06T02:40:18.1488092Z`–`2026-11-06T02:40:18.8679757Z`；實跑 UTC：`2026-11-06T02:40:31.2849109Z`–`2026-11-06T02:42:57.9845734Z`。
- 完整原始 receipt：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261106T024031888Z-9c1d0fd6080d4f51892ca343fdaad316\receipt.json`。
- 驗收副本目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261106T024031888Z-9c1d0fd6080d4f51892ca343fdaad316`。
- 此 receipt 的頂層及 javac／Java／Rust 各自 `cleanup_errors=[]`，`drift_errors=[]`。
- receipt SHA256：`5BEB726D73DFC4D8D44557EA722B8B397E7C4A77F872B7B3926D71702F69B32B`；R／V 副本相同。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'cancel-expiry' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'cancel-expiry'
```

## 程序退出鏈與事後核對（已驗證）

下表取自原始 receipt。**全部 15 個程序**均 `identity_bound=true`、`exit_code=0`、`killed=false`、`wait_for_exit=true`、`cleanup_errors=[]`；不需要 kill。每案結束後以 Get-Process 查詢該案 PID，均為 `not_running`，lease 均由 runner 自行釋放。

| scenario | 程序 | PID | StartTime UTC | 結束碼 | killed | WaitForExit | cleanup_errors |
|---|---|---:|---|---:|---|---|---|
| timeout | javac | 14676 | `2026-11-06T02:32:11.6221770Z` | 0 | false | true | `[]` |
| timeout | java | 23844 | `2026-11-06T02:32:13.4313947Z` | 0 | false | true | `[]` |
| timeout | rust | 23380 | `2026-11-06T02:32:18.0486065Z` | 0 | false | true | `[]` |
| refused | javac | 24788 | `2026-11-06T02:33:56.7110078Z` | 0 | false | true | `[]` |
| refused | java | 13696 | `2026-11-06T02:33:58.3061156Z` | 0 | false | true | `[]` |
| refused | rust | 23264 | `2026-11-06T02:34:01.8881240Z` | 0 | false | true | `[]` |
| wrong-service | javac | 20544 | `2026-11-06T02:35:31.2011547Z` | 0 | false | true | `[]` |
| wrong-service | java | 1324 | `2026-11-06T02:35:32.6359247Z` | 0 | false | true | `[]` |
| wrong-service | rust | 20940 | `2026-11-06T02:35:35.8559448Z` | 0 | false | true | `[]` |
| cancel-retry | javac | 9136 | `2026-11-06T02:38:55.0115916Z` | 0 | false | true | `[]` |
| cancel-retry | java | 15960 | `2026-11-06T02:38:56.6934935Z` | 0 | false | true | `[]` |
| cancel-retry | rust | 12448 | `2026-11-06T02:39:00.3142708Z` | 0 | false | true | `[]` |
| cancel-expiry | javac | 22124 | `2026-11-06T02:40:32.7495759Z` | 0 | false | true | `[]` |
| cancel-expiry | java | 20060 | `2026-11-06T02:40:34.0982509Z` | 0 | false | true | `[]` |
| cancel-expiry | rust | 23248 | `2026-11-06T02:40:37.1611891Z` | 0 | false | true | `[]` |

Java／javac image 分別為指定 JDK 17 的 java.exe／javac.exe；Rust image 為各 receipt 同目錄的 immutable-test.exe。完整路徑、啟動參數、環境覆寫、身分查詢次數及等待結果均保存在原始 receipt。

最終唯讀核對時間為 UTC `2026-11-06T02:43:08.8049407Z`，上述 15 個 PID 均不存在，`lease_remains=false`。事前既有 Java PID 18784 此時也已不存在；本輪未操作該程序，其退出原因與時間未調查，不計入本輪自有程序清理成果。

`維護工具/run-teleport-failure.ps1:207` 的清理重新查詢仍讀 `MainModule.FileName`。本輪程序皆自然退出，**沒有覆蓋「程序仍存活時重新查詢並 kill」分支**，故不能宣稱該潛在 kill refused 問題已被修正；只能確認本輪五份 receipt 沒有此錯誤。

## E149 memory 初始化、特效與恢復證據

每案都有實際 `runtime/ready.json`、`java.stdout.txt`／`java.stderr.txt`、`result.json` 與 server／staged trace，且已複製到驗收目錄，雜湊與原始檔相同。

- ready 記錄實際 Java PID、E149 SHA、三個隔離 loopback 端點；receipt 的 listener 所有權與該 PID 相同。refused 的 destination 預期不存在，receipt 為 `expected_absent=true`、`owners=[]`；其餘案 destination 有自有 listener。所有埠均非 2000／3306。
- memory 證據鏈：receipt 綁定的 `NativeTeleportFailureHarness.java:63` 在 ready 前實際執行 memory／read-only／非持久化檢查，使用 MemoryAccountRepository、GameStateStore(false) 建立 GameHub，之後才產出 ready；harness 日誌有 `qualified source`、`imported read-only source pets`、`loaded state`、`rental-town scene bootstrapped` 與移動接受紀錄，最終 result 再記錄 sql_access=false、state_persistent=false。ready 本身沒有名為 memory 的欄位，沒有將其寫成不存在的欄位或虛構初始化 log。
- Rust stderr 實際出現 `disintegrate presentation unavailable`、`call lightning presentation unavailable`、`cone cold presentation unavailable`；各案隨後完成測試與恢復。這是 #111 改為呈現不可用而不中止連線的實際觀測，**不代表特效呈現可用**。
- 各案 `a-failure.json`／`b-failure.json` 保留錯誤後狀態、epoch、完整重新進入遊戲與移動驗證；cancel-expiry 另有自然到期等待，未修改時鐘或權威狀態。

## 產出路徑與驗證方式

追蹤產出只有 `docs/reports/issue-118.md`。本輪輔助腳本及原始記錄位於 `W\scratch\issue-118-r3\`，不納入 git：

| 檔案 | 用途 |
|---|---|
| `preflight.json`、`processes-before.json` | 工具版本／簽章與事前程序清單 |
| `build.json`、`build.stdout.txt`、`build.stderr.txt`、`jar-copy.json` | 建置完整指令／耗時／結束碼／exe SHA、JAR 來源及副本 SHA |
| `<scenario>.whatif.json`、同前綴 `.stdout.txt`／`.stderr.txt`／`.preview.json` | 完整 WhatIf 指令／耗時與捕捉 JSON；preview.json 只是解析後副本 |
| `<scenario>.run.json`、同前綴 `.stdout.txt`／`.stderr.txt` | 完整實跑指令／耗時／結束碼、receipt 路徑、依 PID／StartTime／Path 的事後核對 |
| `final-audit.json` | 五份 receipt、Rust 原文、result、listener、15 個程序最終核對、R／V 原始證據雜湊一致性 |
| `prepare.ps1`、`invoke-scenario.ps1`、`audit.ps1`、`write-report.ps1` | 本輪一次性輔助記錄腳本；invoke 拒絕重複呼叫同案、拒絕前案未通過仍執行後案 |

**輸出編碼限制（已驗證）**：外層以 UTF-8 解碼子 pwsh 的 console 輸出，但子 pwsh 仍使用系統 code page，WhatIf stdout 及實跑 stdout 的中文路徑出現替代字元；捕捉檔原樣保留，不能視為無損原始位元組。WhatIf JSON 語法可解析，命令 metadata 的完整路徑與 runner 自行寫入的 UTF-8 receipt／ready／result／Java／Rust 日誌不受影響。沒有為修正輸出編碼重跑任何案，也未把解析副本宣稱為已修復的路徑證據；完整重現指令及實際 run 路徑以上述 metadata／receipt 為準。

未額外跑不相關測試。本任務只驗證指定的五個 scenario，沒有將歷史 475 案、G06 640 次或完整 GUI 視為完成。

## 未驗證什麼／推測／給下一棒的提示

- **待驗證**：完整原生 GUI、實際特效顯示、正式服務／DB、重複壓力測試，以及存活程序進入 MainModule 清理分支的行為；本任務不擴張範圍。
- **推測**：不推定前兩輪 MainModule／CIM 問題的根因，也不把五次正常退出當作存活程序 kill 守門的驗證。
- **下一棒：Claude／調度器**審查本報告與五份原始 receipt，依證據裁定 T011-B／G06.3d 狀態；使用者依既有流程合併。docs/REPORT.md 索引由調度器合併後更新，本輪未手改。請保留此 worktree 的 scratch／target/current 供審查。
