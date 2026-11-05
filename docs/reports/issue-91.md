# Issue #91 — T011-H：當版 G06 owned harness 適配

## 結論與範圍

**已驗證：指定三檔修改完成；H1／H2 對正式 E149 副本的隔離 javac、PowerShell 7 語法、五案 WhatIf、listener 查詢權限及 cargo check --tests 均通過。沒有執行任何 G06 scenario，沒有啟動 harness Java 或 Rust 測試執行檔。**

- 日期：2026-11-06（Asia/Taipei）。分支 `task/91`；起點 `288a425fc2b2eb072f68b08e3c6fca7b5784a16a`。
- 唯一可寫工作目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91`。未 checkout、未另建分支；主專案與相鄰登入器僅唯讀。
- 依據：AGENTS.md、docs/REPORT.md、TASKS.md、#82 設計，以及 `git show 8bd1634:docs/reports/issue-80.md`（#80 尚未合併之盤點）。
- 僅修改本報告與下列三個指定檔案；未改 docs/REPORT.md、正式 config/server.properties、候選、封存或硬連結；沒有 DB、正式服務、遊戲程序或 GUI 操作。

## 做了什麼

| 檔案 | 變更 |
|---|---|
| `LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java` | ready 保留既有欄位並新增 login_host、town_host、town_port、destination_host、pid；runtime 的父目錄必須存在，由 Java createDirectory 新建，再比對 toRealPath 與 normalize 路徑。scenario、session 數與 timeout 未改。 |
| `維護工具/run-teleport-failure.ps1` | repo 由 PSScriptRoot 推得；新增 E149 白名單，實跑固定要求 E149；全新 UTC＋GUID run、classes、tmp、immutable-test.exe、runtime 契約；驗收輸出移入 target/current/verification/native-teleport-failure；新增只讀 WhatIf。 |
| 同上 runner | CreateNew／ReadWrite／None 原子 lease，內容為 runner PID、StartTime UTC、root；持有 handle 至 finally。拒絕既有 lease，提示操作員檢查且不自動刪除。run 不用 Force；檢查路徑到 target/current 的每層 ReparsePoint。 |
| 同上 runner | javac／java／Rust 全採 ProcessStartInfo、CreateNoWindow、run cwd、owned TEMP／TMP，清除四個 JVM／classpath 變數；Java/Javac 明確指定 java.io.tmpdir。receipt 記錄實際 argv、環境名稱全集與受控變數值，避免寫出繼承環境中的機密值。 |
| 同上 runner | pin 兩個 Java source、所有 class、JAR、原始與 immutable Rust exe；選用 pin 缺檔記 absent；ready.jar_sha256 必須同時符合副本與 E149。finally 逐檔核對 source/class/JAR/exe 漂移。 |
| 同上 runner | ready PID 必須等於實際 Java PID；核對 login／town／destination host、port 與 scenario；拒絕小於 1024、2000、3306 及非有效埠號。非 refused 三個 listener 以指定 LocalAddress／LocalPort 查擁有者；refused 成功列舉 listener 後檢查目的端不存在，包含 IPv4／IPv6 wildcard bind。任何查詢錯誤直接中止。 |
| 同上 runner | Java 與 Rust 各自保留 Process、Id、StartTime、MainModule.FileName；獨立 try/catch 清理，Kill 使用原始 process handle，檢查 WaitForExit。每個程序的 exit code、killed、wait_for_exit、cleanup_errors 留在 receipt；javac 也納入。移除 Get-CimInstance。 |
| 同上 runner | Rust 必須真的 running 1 test／1 passed／0 failed／0 ignored；檢查 result.json 的 sessions（wrong-service 10、其餘 8）、memberships、authority_mutations、staged loadouts、memory／非持久化與 JAR 綁定。receipt 寫 run 與驗收輸出兩處，最後關閉 handle 才移除自有 lease。 |
| `LiTo381-Modern/tests/native_teleport_failure.rs` | timeout 入口要求 LITO_TELEPORT_SCENARIO 明確等於 timeout；讀 ready 後拒絕 port／town_port 為 2000 或 3306，再建立連線。 |

實跑順序可由 runner 本文核對：lease → fresh run/classes/tmp（runtime 不存在）→ immutable exe 與 SHA → javac 與 source/class SHA → 再次確認 runtime 不存在 → Java(runtime, scenario) → ready/JAR/PID/listeners → Rust → result → finally 各別清理、漂移檢查、receipt → 關閉與移除 lease。

## 怎麼驗證（已驗證）

| 驗收 | 結果與證據 |
|---|---|
| 正式 JAR 與工作樹副本 SHA256 | 兩者均為 E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498；先讀正式檔 hash，符合才複製，再驗證副本。 |
| H1 隔離 javac | Microsoft JDK 17.0.17（release 檔確認）；結束碼 0；3 個 class；未執行 class。 |
| PowerShell 語法 | PowerShell 7.6.5 Parser.ParseFile，0 個 parse error；`verification/issue-91/syntax.txt`。 |
| 五個 WhatIf | timeout、cancel-expiry、cancel-retry、refused、wrong-service 各別子程序結束碼均 0；解析 JSON 確認白名單/E149 與三個預定指令；run、output 與 lease 未建立，runner 輸出樹清單未變。證據 `verification/issue-91/whatif.json`。TestBinary 使用 target/current 下尚未建立的路徑，確認 dry-run 不必建置或執行測試。 |
| listener 查詢權限 | 本機 PowerShell 7 執行 `Get-NetTCPConnection -State Listen \| Select-Object -First 1` 成功，第一筆 `:::49673`、PID 1212（僅記錄，不接觸程序）；`verification/issue-91/listener-query.json`。沙箱原先回傳 0x80041003，經工具核准的本機查詢成功；這不代表沙箱內也有查詢權限。 |
| H2 隔離 javac | 從 runner AST 只取出實際 Start-OwnedProcess、Stop-OwnedProcess、Get-ClassPins 等 helper 與 javacArgs 表達式，在獨立 compile 目錄執行；不呼叫 runner 主流程。結束碼 0，3 個 class 與 H1 相同；javac 身分綁定成功、WaitForExit=true、未 Kill、清理錯誤空。 |
| Rust check | `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --tests`，cwd=本 worktree/LiTo381-Modern，結束碼 0；只編譯檢查，沒有執行測試。證據 `verification/issue-91/cargo.json` 與 cargo.stderr.txt。 |
| 守門單元檢查 | 13 項純記憶體合成資料檢查：正常／refused 無 listener 可接受；三埠 owner 不符、2000／3306／低埠、PID／JAR／host 不符、refused wildcard 佔用及查詢丟錯均拒絕。沒有真實連線、Java 或 Rust 測試程序；只證明守門分支，不能當成 scenario 通過。證據 `verification/issue-91/synthetic-guards.json`。 |
| 變更檢查 | `git -c core.whitespace=cr-at-eol diff --check`；保留原 Java／Rust 的 CRLF，不做無關格式化。 |

PowerShell 7 原未在此執行環境 PATH，且沒有找到可用安裝；使用官方 v7.6.5 Windows x64 zip 解壓至本 worktree `scratch/tools/pwsh/`，pwsh.exe 的 Authenticode 為 Valid／Microsoft Corporation。未安裝系統工具或改永久設定。H1 的 PowerShell 5.1 驗收輔助腳本使用該次程序的 ExecutionPolicy Bypass，沒有 Set-ExecutionPolicy 或永久政策變更。

驗收過程的非通過嘗試亦保留：H1 輔助腳本初版因 PowerShell 陣列加法括號錯誤將 javac options 合成一個參數，javac 回傳 1（`t011-h1-compile-2138d7fb666d4db0b91cdbf4aa117a92`）；修正驗收腳本後才產生下列 H1 成功證據。另一次 WhatIf 驗收清單比對受到同時進行的 Cargo 產物影響；改比對 runner 專屬兩個輸出樹後，重新五案全數成功。兩者不算 scenario 失敗或通過，也未據此修改 fixture 邏輯。

## 實際 javac 指令與隔離環境

以下指令由 ProcessStartInfo 執行，並非 shell 轉譯。兩次的 working directory 分別為各自 compile 目錄，TEMP／TMP 均為該目錄的 tmp；移除 JAVA_TOOL_OPTIONS、_JAVA_OPTIONS、JDK_JAVA_OPTIONS、CLASSPATH。receipt 保留全部實際環境變數名稱與受控值；繼承值不全量寫出以免洩漏機密。

### H1

```powershell
"C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe" "-J-Djava.io.tmpdir=D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h1-compile-c9fba615a14a4096ad924f8e21b36022\tmp" "-J-Dfile.encoding=UTF-8" "-encoding" "UTF-8" "-cp" "D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar" "-d" "D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h1-compile-c9fba615a14a4096ad924f8e21b36022\classes" "D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\tests\oracle\NativePeerHarness.java" "D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\tests\oracle\NativeTeleportFailureHarness.java"
```

工作目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h1-compile-c9fba615a14a4096ad924f8e21b36022`。

實際環境變數名稱（H1）：

```text
ALLUSERSPROFILE, APPDATA, CARGO_HOME, CODEX_APPLY_PATCH_PRESERVE_LINE_ENDINGS, CODEX_CI, CODEX_SESSION_ID, CODEX_THREAD_ID, CODEX_TOOL_CALL_ID, CODEX_VERSION, COLORTERM, COMMONPROGRAMFILES, COMMONPROGRAMFILES(X86), COMMONPROGRAMW6432, COMPUTERNAME, COMSPEC, DRIVERDATA, FPS_BROWSER_APP_PROFILE_STRING, FPS_BROWSER_USER_PROFILE_STRING, GH_PAGER, GIT_CONFIG_COUNT, GIT_CONFIG_KEY_0, GIT_CONFIG_KEY_1, GIT_CONFIG_VALUE_0, GIT_CONFIG_VALUE_1, GIT_PAGER, HOMEDRIVE, HOMEPATH, JAVA_HOME, LANG, LC_ALL, LC_CTYPE, LESS, LOCALAPPDATA, LOGONSERVER, MAVEN_HOME, NO_COLOR, NODE_REPL_TRUSTED_BROWSER_CLIENT_SHA256S, NUGET_PACKAGES, NUMBER_OF_PROCESSORS, ONEDRIVE, ORCH_ISSUE, ORCH_REPO, OS, PAGER, PATH, PATHEXT, PROCESSOR_ARCHITECTURE, PROCESSOR_IDENTIFIER, PROCESSOR_LEVEL, PROCESSOR_REVISION, PROGRAMDATA, PROGRAMFILES, PROGRAMFILES(X86), PROGRAMW6432, PROMPT, PSExecutionPolicyPreference, PSMODULEPATH, PUBLIC, PY, PYTHONIOENCODING, PYTHONUTF8, RUSTUP_HOME, SESSIONNAME, SYSTEMDRIVE, SYSTEMROOT, TEMP, TERM, TMP, USERDOMAIN, USERDOMAIN_ROAMINGPROFILE, USERNAME, USERPROFILE, WINDIR
```

### H2

```powershell
& 'C:\Program Files\Microsoft\jdk-17.0.17.10-hotspot\bin\javac.exe' '-J-Djava.io.tmpdir=D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h2-compile-3d076656cb4243c7aab71a411cd68201\tmp' '-J-Dfile.encoding=UTF-8' '-encoding' 'UTF-8' '-cp' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar' '-d' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h2-compile-3d076656cb4243c7aab71a411cd68201\classes' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\tests\oracle\NativePeerHarness.java' 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\tests\oracle\NativeTeleportFailureHarness.java'
```

工作目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-91\LiTo381-Modern\target\current\verification\t011-h2-compile-3d076656cb4243c7aab71a411cd68201`。

實際環境變數名稱（H2）：

```text
ALLUSERSPROFILE, APPDATA, CARGO_HOME, CODEX_APPLY_PATCH_PRESERVE_LINE_ENDINGS, CODEX_CI, CODEX_SESSION_ID, CODEX_THREAD_ID, CODEX_TOOL_CALL_ID, CODEX_VERSION, COLORTERM, COMMONPROGRAMFILES, COMMONPROGRAMFILES(X86), COMMONPROGRAMW6432, COMPUTERNAME, COMSPEC, DRIVERDATA, FPS_BROWSER_APP_PROFILE_STRING, FPS_BROWSER_USER_PROFILE_STRING, GH_PAGER, GIT_CONFIG_COUNT, GIT_CONFIG_KEY_0, GIT_CONFIG_KEY_1, GIT_CONFIG_VALUE_0, GIT_CONFIG_VALUE_1, GIT_PAGER, HOMEDRIVE, HOMEPATH, JAVA_HOME, LANG, LC_ALL, LC_CTYPE, LESS, LOCALAPPDATA, LOGONSERVER, MAVEN_HOME, NO_COLOR, NODE_REPL_TRUSTED_BROWSER_CLIENT_SHA256S, NUGET_PACKAGES, NUMBER_OF_PROCESSORS, ONEDRIVE, ORCH_ISSUE, ORCH_REPO, OS, PAGER, PATH, PATHEXT, PROCESSOR_ARCHITECTURE, PROCESSOR_IDENTIFIER, PROCESSOR_LEVEL, PROCESSOR_REVISION, PROGRAMDATA, PROGRAMFILES, PROGRAMFILES(X86), PROGRAMW6432, PROMPT, PSMODULEPATH, PUBLIC, PY, PYTHONIOENCODING, PYTHONUTF8, RUSTUP_HOME, SESSIONNAME, SYSTEMDRIVE, SYSTEMROOT, TEMP, TERM, TMP, USERDOMAIN, USERDOMAIN_ROAMINGPROFILE, USERNAME, USERPROFILE, WINDIR
```

## SHA256 明細（H1 與 H2 相同）

正式唯讀 JAR：`D:\lineage開發\3-lito開發\lito登入器\LiTo381-Launcher\app\server\LiTo381-SharedResources-candidate.jar`。
副本：`LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`。

| 檔案 | SHA256 |
|---|---|
| 正式 JAR 與副本 | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498` |
| tests/oracle/NativePeerHarness.java | `E6108F193F70EE219FD342695C95E5499668BE4FBA8E9A5B2B3D99875751863D` |
| tests/oracle/NativeTeleportFailureHarness.java | `C1B7DE7BFC8E2C1BC804929FC921CB5D37390E3C8F4F20D57515F38E7FBB0295` |
| classes/local/lito381/server/NativePeerHarness$Session.class | `AF277F5127AD1CD4F1677D412E00DF5D4F3C0559A801CCCB59618E4DF735B989` |
| classes/local/lito381/server/NativePeerHarness.class | `47A0DB1F3223DB036BEB6B1682CBA4FB8BE40CB3960F1AE39290470242A475FE` |
| classes/local/lito381/server/NativeTeleportFailureHarness.class | `8A0788009FA2D33EC7DB9D3AEE750478A770D3B2AC65AAEE3875DAF77D06A74A` |

兩次 javac 只有 NativePeerHarness.java 既有 unchecked/unsafe operations 提示，無編譯錯誤；H1/H2 逐個 class SHA 一致。

## 未驗證什麼

- **待驗證**：五個真實 G06 scenario、E149 memory 模式的完整資源初始化、實際 LOGIN／town／destination 所有權、refused 真實拒接、恢復與退出結果。
- **待驗證**：Java/Rust 正常退出、異常 Kill／WaitForExit、cleanup 故障時仍逐一回收、lease 並行互斥／殘留處理、runtime 重用／reparse 實際拒絕、執行中 source/class/JAR 漂移。這次只能靜態核對程式；javac 的正常退出證據不能替代 Java／Rust 退出鏈。
- **推測**：依 #80，E149 memory 模式的資源初始化仍可能在實跑才暴露問題；本次 javac／cargo check 成功不證明 runtime 可用。
- 未執行 class、測試 exe、cargo test、GUI、正式服務或 DB；不以略過／ignored／零測試算通過。

## 產出路徑

追蹤交付：

- `LiTo381-Modern/tests/oracle/NativeTeleportFailureHarness.java`
- `維護工具/run-teleport-failure.ps1`
- `LiTo381-Modern/tests/native_teleport_failure.rs`
- `docs/reports/issue-91.md`

不進 git 的本機證據（均在本 worktree）：

- `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`
- `LiTo381-Modern/target/current/verification/t011-h1-compile-c9fba615a14a4096ad924f8e21b36022/`（classes、tmp、指令、環境、來源/class/JAR SHA、exit code 收據與 logs）
- `LiTo381-Modern/target/current/verification/t011-h2-compile-3d076656cb4243c7aab71a411cd68201/`（同上，使用 runner helper）
- `LiTo381-Modern/target/current/verification/issue-91/`（syntax、五案 WhatIf、listener 查詢、守門單元檢查與 Cargo logs／receipt）
- `scratch/compile91.ps1`、`scratch/verify91.ps1`、`scratch/listener91.ps1`、`scratch/gates91.ps1`、`scratch/check91.ps1`（一次性驗收輔助腳本）；`scratch/tools/pwsh/`（官方可攜工具）。

## 給下一棒的提示

1. 由 Claude 審查本分支；使用者合併。T011-H 合併並通過審查後，雲端主控才可將 T011-B（#81）重新排程。
2. T011-B 由本機執行器執行。須在實跑所在 worktree 重新建置當版 native_teleport_failure 測試 exe（本次 cargo check 不產生可交付測試 exe）；CARGO_MANIFEST_DIR 的 target/current 必須與 runner 的 target/current 一致，不能拿其他 worktree 的舊 binary。
3. 正式 E149 JAR 副本與測試 exe 都須位於該 worktree 的 target/current。先 WhatIf，再依授權執行單案；首次失敗停止，不自動重試或跳過守門。若 Get-NetTCPConnection 權限不足，就在 Rust 前中止，不能把查詢失敗解讀成沒有 listener。
4. 每次由 runner 自行建立新 run；不要預建 runtime、提供舊 ready 或人工刪除 lease 來繞過互斥。既有 lease 必須由本機操作員核對擁有程序與 receipt，決定後續處理。
5. 實跑交付須有 running 1 test／1 passed、result.json、三埠所有權，以及 Java/Rust 每個程序的退出與清理證據。合成守門檢查、編譯成功、WhatIf 均不能取代這些實跑條件。
