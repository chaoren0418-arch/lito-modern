# Issue #118 — T011-B 重跑：G06.3d 本機結果

**已驗證：首案 `timeout` 在 Rust 啟動前遭 runner 的 Java 程序身分守門阻塞；通過 0、測試失敗 0、阻塞 1、未執行 4。已停止，沒有重試。這次未驗證 #111 修正後的 G06 行為。**

## 做了什麼

- 日期：2026-11-06（Asia/Taipei）。工作目錄 `W = D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118`。
- 沿用 `task/118`，未 checkout／建立分支。起點、本機 `origin/main` 與實際 `git ls-remote origin refs/heads/main` 都是 **`241121ba81b8cffc65921677ff112b953e8a017f`**；已確認包含 PR #115 的 merge commit `2233fe9315bebf08c137bbb5a613ae9988244ded`。
- 已讀根目錄 AGENTS.md、docs/REPORT.md、TASKS.md 的 T011，以及 #81、#91、#111 報告與當版 runner／harness／Rust 測試。
- 本次唯一納入 git 的變更為本報告；未修改 src、tests、既有腳本或 docs/REPORT.md。輔助記錄腳本與證據放在 W 的 scratch／target/current。
- 未操作既有遊戲程序、開 GUI、連正式服務／DB、讀改機密設定或封存／硬連結；原始專案只讀，正式 JAR 驗證後才複製到 worktree。

## 五案結果與實際耗時

| 順序／scenario | WhatIf | 實跑結果 | 實際耗時 | 原因 |
|---|---|---|---|---|
| 1／timeout | 結束碼 0；1.0721331 秒；JSON 已保存 | **阻塞（已驗證）**；呼叫 runner 一次，結束碼 1；Rust 未啟動 | runner 退出等待 3.9790114 秒；含輸出串流收集至外層結束共 94.686386 秒 | Java 的 `MainModule.FileName` 讀取失敗；身分未綁定，清理拒絕 kill |
| 2／refused | 未執行 | 未執行（待驗證） | — | 首案守門阻塞後停止 |
| 3／wrong-service | 未執行 | 未執行（待驗證） | — | 同上 |
| 4／cancel-retry | 未執行 | 未執行（待驗證） | — | 同上 |
| 5／cancel-expiry | 未執行 | 未執行（待驗證） | — | 同上 |

`timeout` 外層實際時間為 UTC `2026-11-05T23:48:03.3694600Z`–`23:49:38.0558460Z`（本機 07:48:03–07:49:38）。外層 stopwatch 在 runner 退出時停止，但讀完 stdout／stderr 才寫結束時間，因此分別列出兩種耗時，不把 3.98 秒當成完整作業時間。

沒有再次執行 WhatIf 或實跑，沒有跳過守門、預建 runtime、手動刪除 lease，亦未修改 runner 來繞過錯誤。

## 前置環境與建置（已驗證）

原始 PATH 找不到 pwsh、cargo、rustc；外層是 Windows PowerShell `5.1.19041.6456`。只對本次程序使用既有 Rust 路徑與 worktree 的可攜 PowerShell，未安裝系統工具或改永久設定。

| 項目 | 實際值 |
|---|---|
| 補入程序 PATH 後 `(Get-Command pwsh).Source` | `W\scratch\tools\pwsh\pwsh.exe` |
| PowerShell | `7.6.5`；官方 ZIP：`https://github.com/PowerShell/PowerShell/releases/download/v7.6.5/PowerShell-7.6.5-win-x64.zip` |
| Authenticode | `Valid`；`CN=Microsoft Corporation, O=Microsoft Corporation, L=Redmond, S=Washington, C=US` |
| pwsh.exe SHA256 | `362A356CE7F0940EC74F73A8FC2C990A2CC24A38A11C90BBD8ECA947110AD139` |
| JDK 路徑 | `C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin` |
| Java／javac | OpenJDK `17.0.17+10-LTS`，Microsoft-12574423；javac `17.0.17` |
| Cargo | `D:\dev-cache\cargo\bin\cargo.exe`；`cargo 1.95.0 (f2d3ce0bd 2026-03-21)` |
| Rustc | `D:\dev-cache\cargo\bin\rustc.exe`；`rustc 1.95.0 (59807616e 2026-04-14)` |
| Cargo／Rustup home | 沿用 `D:\dev-cache\cargo`／`D:\dev-cache\rustup` |
| 外層 TEMP／TMP | `W\scratch\issue-118\tmp`；runner 子程序使用各 run 的 tmp |

沙箱內唯讀查詢遠端 main 遇到 `SEC_E_NO_CREDENTIALS`；官方 ZIP 下載與遠端 main 核對改經工具核准執行，成功。依 #91 已記錄的沙箱 `Get-NetTCPConnection` 權限限制，唯一一次 runner 實跑亦經工具核准使用本機權限；本次尚未走到 listener 查詢，不能宣稱其通過。

建置 cwd：`W\LiTo381-Modern`；指令：

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-run
```

第一次外層使用 PowerShell 5.1 的 `ErrorActionPreference=Stop`，將 Cargo stderr 的正常 `Compiling serde_core` 訊息視為 NativeCommandError 而中斷。確認編譯程序已不在後，僅修正外層記錄方式，再次執行上述 **no-run 建置**；不屬於 scenario 重跑。完整建置結束碼 **0**，耗時 **339.2031354 秒**，UTC `23:41:36.0151134Z`–`23:47:15.2232841Z`；有既有 warnings，未改碼處理。

此次 exe 在本 worktree 建置，沒有借用其他 worktree 的舊 binary；`CARGO_MANIFEST_DIR` 與 runner 的 `target/current` 一致。

| 檔案 | 路徑 | SHA256 |
|---|---|---|
| 新建測試 exe | `W\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe` | `693F7DBD1E868E9C1C7517B0B118D46C7EC6ACA858B363032E9951ED7B8EE17C` |
| 正式唯讀 JAR | `D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar` | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| JAR 副本 | `W\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar` | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |

正式 JAR 先驗 SHA，複製前再驗一次，相符才複製，再驗副本；該輪雙重雜湊與複製共 **0.4949598 秒**。最終核對 exe／JAR 雜湊未變。

## 完整指令與首個阻塞的原始證據

兩條 cwd 均為 W，各呼叫一次。第二條也是供後續修正後新測試批次使用的重現指令；本次沒有重跑。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout' '-WhatIf'

& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' '-NoProfile' '-File' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\維護工具\run-teleport-failure.ps1' '-TestBinary' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\deps\native_teleport_failure-df1b3ae88311d364.exe' '-ReferenceJar' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-Scenario' 'timeout'
```

首個阻塞的**完整原始收據**（沒有補寫或修正）：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\native-teleport-failure\20261105T234804138Z-9d5455933d0840fa9e5d8e9e9dac37fb\receipt.json
```

以下簡稱其所在目錄為 `R`。驗收輸出副本目錄 `V`：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\LiTo381-Modern\target\current\verification\native-teleport-failure\20261105T234804138Z-9d5455933d0840fa9e5d8e9e9dac37fb
```

R／V 的 receipt SHA256 一致：`1CF0EB583B1B6910118F658E4FEE622FC9EB08091779F7DF949764D1649D463B`。

| 證據 | 位置 |
|---|---|
| 工具路徑／版本、簽章 | `W\scratch\issue-118\preflight.json`、`pwsh-signature.json` |
| 建置指令／時間／結束碼／exe SHA 與完整輸出 | `W\scratch\issue-118\build.json`、`build.stdout.txt`、`build.stderr.txt` |
| JAR 來源與副本核對 | `W\scratch\issue-118\jar-source.json`、`jar-copy.json` |
| WhatIf 指令／時間／結束碼 | `W\scratch\issue-118\timeout.whatif.json` |
| WhatIf 已捕捉 JSON／stderr | `W\scratch\issue-118\timeout.whatif.stdout.txt`、`timeout.whatif.stderr.txt` |
| 實跑指令／時間／結束碼／PID 與 StartTime 核對 | `W\scratch\issue-118\timeout.run.json`；同前綴 `.stdout.txt`／`.stderr.txt` |
| javac 原始輸出 | `V\javac.stdout.txt`、`V\javac.stderr.txt` |
| Java 後續觀測、最終清理與檔案核對 | `W\scratch\issue-118\java-post-observation.json`、`final-audit.json` |
| 實際 ready | **`R\runtime\ready.json`**；沒有複製到 V，不能只找 V |

**輸出編碼限制（已驗證）**：外層串流解碼與子 pwsh 的輸出編碼不一致，WhatIf stdout 與實跑 stderr 的中文路徑出現替代字元；原始捕捉檔保留，未覆寫。指令 metadata、runner 自行以 UTF-8 寫的 receipt、ready 與雜湊不受此問題影響。另存 `timeout.whatif.normalized.json`，僅以已知的 W 修復預覽中相同的根目錄字串，明確屬衍生檔，不能冒充原始 stdout；沒有為修復編碼而重跑 WhatIf。

## 阻塞與退出鏈（已驗證）

runner 收據雖寫 `status=failed`，但本任務依「Rust 前守門失敗」規則分類為**阻塞**。原始錯誤：

```text
The property 'FileName' cannot be found on this object. Verify that the property exists.
```

當版 `維護工具/run-teleport-failure.ps1:114` 在 `Start-OwnedProcess` 讀取 `$process.MainModule.FileName`；本次 Java 有 PID／StartTime，但 `image=null`、`identity_bound=false`。其後 finally 的清理守門拒絕 kill：

```text
Unbound process identity: kill refused
```

| 程序 | PID | StartTime UTC | 結束碼 | killed | WaitForExit | 身分綁定／cleanup_errors |
|---|---:|---|---|---|---|---|
| javac | 2272 | `2026-11-05T23:48:05.0068082Z` | 0 | false | true | true／`[]` |
| Java harness | 12996 | `2026-11-05T23:48:06.4778627Z` | **null／未取得** | false | **false** | false／`["Unbound process identity: kill refused"]` |
| Rust | — | — | — | — | — | **未啟動** |

頂層 `cleanup_errors` 有兩筆相同的 `java: Unbound process identity: kill refused`；`drift_errors=[]`、`inputs_unchanged=true`，來源／class／JAR／exe／immutable exe 的 finally 核對沒有漂移。

Java 在 runner 退出後仍短暫存在。唯讀觀測曾核對 PID 12996 的 StartTime 與收據相同，image 為指定 JDK 的 java.exe；本次沒有 kill／附加它，也沒有改寫收據。到 UTC `23:49:38` 的外層核對及 `23:49:39` 的後續觀測，該 Java 已不在；最終核對也沒有自有程序殘留。Java 的實際退出碼與確切退出原因仍**待驗證**，後來不存在不能補成原收據的 WaitForExit 成功。

javac PID 2272 在第一次外層核對已重用，新的 StartTime 為 `2026-11-05T23:49:37.9167350Z`，與收據不同，因此只記 `pid_reused`，沒有操作該程序；最終再核對也已不在。Rust 從未啟動。`lease_remains=false`，由 runner 自行釋放，未手動移除。

## 驗收缺口、推測與給下一棒的提示

**已驗證**：javac 成功；Java 已建立新的隔離 runtime 與 ready。`R\runtime\ready.json` 記錄 PID 12996、E149 JAR SHA，以及 login `127.0.0.1:49153`、town `127.0.0.2:49154`、destination `127.0.0.3:49155`。這只是 ready 所載端點；runner 的 `listeners=[]`，沒有完成三埠所有權查詢，不可當成守門通過。沒有連 2000／3306。

**待驗證**：沒有 Rust `running 1 test`／1 passed；沒有 result.json，sessions（wrong-service 應為 10，其他為 8）、memberships、authority_mutations、staged loadouts、full_game_reentry、result 的 memory／非持久化與 JAR 綁定均未驗收。Java stdout／stderr 沒有被 runner 保存，V 只有 receipt 與 javac 日誌。harness 來源在寫 ready 前會檢查 memory／read-only／非持久化並建立 GameHub，但僅有 ready、缺少 harness log，不能宣稱本任務要求的 E149 memory 初始化證據完整，也不能以此代替 G06 通過。

**推測**：啟動後立即讀取 MainModule 可能存在程序載入／模組枚舉的時序問題；稍後可讀到 Java image 與此相容，但本次沒有重跑或實驗，不能定案為競態，也不能斷言是 JDK／權限問題。

下一棒由 **Claude／調度器**審查本報告，另派 **本機 code 任務**處理 runner 的程序身分取得及未完成綁定時的退出證據保存；維持 PID＋StartTime＋image 的安全契約，不直接取消身分檢查。處理後再排本機 T011-B 新批次，從最新 main 於該 worktree 重建 exe，依原順序各跑一次、首個失敗／阻塞即停。本任務不在 test 範圍內修程式，也不自行重試。

追蹤產出：`docs/reports/issue-118.md`。原始證據與可攜工具留在 W 的忽略目錄；請保留此 worktree 供下一棒查閱。
