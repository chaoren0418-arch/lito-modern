# Issue #82 / T011-H：當版 G06 owned harness 的最小開發範圍

## 裁定

**採用 memory 模式的 `維護工具/run-teleport-failure.ps1`，只做最小適配，不走 E149 自有 DB 鏈。** 這項工作是驗證基礎設施，不是新功能，所以留在 **M1**。PLAN M2 的 G06 開發（T023 等）不受影響。

理由（依 #80 盤點，加上雲端讀 `維護工具/run-teleport-failure.ps1` 全文 82 行）：

- **當版的 5 個 scenario 本來就是 memory 模式**。當版 `tests/oracle/NativeTeleportFailureHarness.java`（2 個參數：root、scenario）用記憶體帳號；#80 也確認「當版 memory harness 沒有自行啟動 DB」。所以**不需要自有 MySQL**，自有隔離 DB 的條件也不適用。
- **run-teleport-failure.ps1 的大部分機制已經符合需求**：
  - 每次在 `target/current/native-teleport-failure/<stamp>` 建立新的 root（第 11–12 行）；
  - 測試執行檔與 JAR 必須位於 `target/current`（第 5–6 行）；
  - 以當版的兩個 Java 來源對指定 JAR 執行 javac（第 44–45 行）；
  - 等待 ready、設定三個環境變數、執行單一測試並帶 `--exact --ignored`（第 47–56 行）；
  - 清理時比對 PID、啟動時間與命令列（第 16–23 行）；
  - 檢查 `result.json` 的 sessions 數、memberships 與 authority_mutations，並做 source／binary 漂移檢查（第 64–72 行）。
- **E149 runner（`run-native-E149-recovery-case.py`）不適合**：它綁定候選的 class 與 image，需要 v4 plan，還會改舊的工作佇列與進度檔（#80）。不採用。

## 最小開發範圍（第 2 次嘗試改為 producer → runner 兩個任務，見 `docs/plans/issue-82.json`）

依據是 #80 報告（`task/80` 的 `8bd1634`，尚未進 main），以及雲端閱讀 main 上的 `維護工具/run-teleport-failure.ps1`（82 行）、`tests/oracle/NativeTeleportFailureHarness.java`（96 行）、`tests/native_teleport_failure.rs`。

### T011-H1（producer，Java harness）

| 缺口 | 修正 | 驗收 |
|---|---|---|
| `ready.json`（Java 第 76 行）只有 LOGIN `port` 與 `destination_port`；實際的 GAME redirect 是第 34–35 行另開的 `town` listener（127.0.0.2），設定中的 `port`、`advertised.port`、`lito.rental.town.directory.port` 都取自它 | ready.json 補上 `login_host`、`town_host`、`town_port`、`destination_host`、`pid` | 本機用 JDK 17 對 E149 JAR 副本實際 javac，結束碼 0；列出來源、class、JAR 的 SHA256 |
| 第 31 行用 `Files.createDirectories(root)`，可以重用既有 root | 上一層必須已存在；用 `createDirectory` 建立，已存在就失敗；`toRealPath` 必須等於 normalize 後的路徑（拒絕 reparse） | 同上（只編譯，不執行） |

### T011-H2（runner，依賴 H1）

| # | 缺口 | 修正 |
|---|---|---|
| 1 | 白名單（第 8 行）沒有 E149 | 加入 `E1498871…` |
| 2 | 第 4 行硬編 repo 根 | 改由 `$PSScriptRoot` 推得 |
| 3 | 輸出寫到 `驗證紀錄/`（第 13 行） | 改寫到 `target/current/verification/…` |
| 4 | fresh root 只有毫秒時間戳，且用 `New-Item -Force`（第 11–14 行） | 名稱加 GUID；不帶 `-Force` 建立，已存在就失敗；root 到 `target/current` 之間每一層都不能是 ReparsePoint；拒絕既存的 ready |
| 5 | Java 的 temp 與繼承的 JVM options 沒有隔離（#80） | 三個子程序都改用 `ProcessStartInfo`；`TEMP`／`TMP` 與 `java.io.tmpdir` 都設為 `<root>/tmp`；移除 `JAVA_TOOL_OPTIONS`、`_JAVA_OPTIONS`、`JDK_JAVA_OPTIONS`、`CLASSPATH`；環境變數清單寫進 receipt |
| 6 | 沒有來源與 class 的可核對綁定（#80 要求交付 classes SHA） | 記錄兩個來源、每個 .class、JAR 的 SHA256；確認 `ready.jar_sha256` 等於 JAR 副本且是 E149；執行後再核對一次 |
| 7 | 沒有驗證埠所有權，漏了實際 GAME redirect 用的 town | login、town、destination 三個 listener 都用 `Get-NetTCPConnection -LocalAddress/-LocalPort` 確認 OwningProcess 等於 java PID 與 `ready.pid`（`refused` 則確認 destination 沒有 listener）；拒絕 2000、3306 與小於 1024 的埠；查詢失敗就在啟動 Rust 前中止 |
| 8 | 第 78 行把 Rust `$client` 與 java `$server` 都交給同一個 `Stop-OwnedTeleport`，只比對 java 會讓 Rust 的清理失敗，丟出的例外還可能跳過 java 的清理；而且依賴 CIM | 兩個程序分別綁定 Id、StartTime 與映像路徑（java.exe／root 內的 immutable exe）；finally 中逐一 try/catch 清理，確認 `WaitForExit` 的結果；每個 PID 的結束碼、是否被 Kill、清理錯誤都寫進 receipt；移除 CIM |
| 9 | 重複執行的偵測依賴 CIM | 用 `CreateNew` 原子建立 lease 檔，執行期間持有 handle；已存在就拒絕，不自動刪除；finally 關閉 handle 後才刪除 |
| 10 | 不存在的選用 pin 檔會讓腳本失敗 | 記為 `absent` |
| 11 | 沒有 dry-run | 新增 `-WhatIf` |
| 12 | Rust 端沒有守門 | `port` 或 `town_port` 是 2000／3306 就 panic；timeout 入口必須明確指定 scenario |

H2 的驗收（本機）：pwsh 語法檢查、5 次 `-WhatIf`、`Get-NetTCPConnection` 查詢權限確認（只查詢）、依 H1 方式實際 javac、`cargo check --tests`。**退出鏈與實際 listener 所有權只能在實跑中驗證**，H2 報告要列為待驗證。

## 目錄契約與啟動順序（第 3 次嘗試新增，H1 與 H2 共用）

| 路徑 | 誰建立 | 內容 |
|---|---|---|
| `<run>` = `target/current/native-teleport-failure/<stamp>-<GUID>/` | runner（不帶 `-Force`，已存在就失敗；每一層都不能是 ReparsePoint） | 工作目錄；Java 與 Rust 的 cwd |
| `<run>/classes/` | runner | javac 輸出 |
| `<run>/tmp/` | runner | javac、java、Rust 的 TEMP／TMP 與 java.io.tmpdir |
| `<run>/immutable-test.exe`、`<run>/receipt.json` | runner | 測試執行檔副本與收據 |
| `<run>/runtime/` = `LITO_TELEPORT_ROOT` = Java 的第一個參數 | **Java**（`createDirectory`，已存在就失敗；`toRealPath` 檢查） | ready.json、server.properties、trace、captures |

啟動順序：lease → 建立 `<run>`、`classes`、`tmp`，確認 `runtime` 不存在 → immutable exe 與 SHA → 隔離環境下 javac → 隔離環境下 java（建立 runtime）→ ready、jar SHA、三個 listener 的所有權 → Rust（`LITO_TELEPORT_ROOT=<run>/runtime`）→ result.json → finally 清理、核對漂移、寫 receipt、釋放 lease。runner 與 Java 不再搶著建立同一個目錄。

**隔離環境**（javac、java、Rust 一律適用，**包括 H1／H2 驗收時的 javac**）：`ProcessStartInfo`；`TEMP`／`TMP` 指向 owned 的 `tmp`；javac 加 `-J-Djava.io.tmpdir`，java 加 `-Djava.io.tmpdir`；移除 `JAVA_TOOL_OPTIONS`、`_JAVA_OPTIONS`、`JDK_JAVA_OPTIONS`、`CLASSPATH`；環境變數清單寫進 receipt 或報告。

## T011-B（#81）的條件（第 3 次嘗試改為三層，消除循環依賴）

1. **可排程條件**：H1 與 H2 都合併並通過審查。滿足後，雲端主控就更新 #81 的本文並改回 queued。這一層不包含任何只有實跑才能驗證的項目。
2. **每案啟動 Rust 前的檢查**：lease、目錄契約、隔離環境下的 javac、來源與 JAR 的 SHA 綁定、三個 listener 的所有權。任何一項失敗都中止，回報為阻塞，**不啟動 Rust scenario**。
3. **實跑後的驗收**：`running 1 test` 而且 1 passed；result.json 的 sessions、memberships、authority_mutations 檢查通過；java 與 Rust 都有退出證據；E149 資源在 memory 模式下的初始化有實際證據。沒有實跑，不能宣稱通過。

### 第 2 次嘗試的修正（對應 codex-local 審查的 5 點）

1. temp、JVM options、fresh root 與來源綁定：H1 第 2 點，H2 第 4–6 點。
2. GAME redirect（town）的埠與所有權：H1 第 1 點補 ready 契約，H2 第 7 點驗證三個 listener。
3. java 與 Rust 分開清理、lease 原子性：H2 第 8–9 點。
4. 拆成 producer（H1）→ runner（H2）兩個任務，H2 依賴 H1；#81 依賴兩者；並記錄 #80 的 SHA。
5. 驗收與解鎖一致：javac 在 H1 與 H2 都要實際執行；`Get-NetTCPConnection` 權限在 H2 確認；退出鏈與 E149 初始化留給 T011-B，並明定前置失敗就中止。

### 第 3 次嘗試的修正（對應 codex-local 第 2 輪審查的 3 點）

1. **root 建立責任互斥**：改成 runner 擁有 `<run>`，Java 新建 `<run>/runtime`（見上方目錄契約表），並寫出完整的啟動順序與 stale runtime／ready 的拒絕條件；H1、H2 的 plans 文字同步。
2. **javac 驗收的隔離**：H1、H2 驗收時的 javac 也套用同一個隔離環境（owned tmp、`-J-Djava.io.tmpdir`、清除繼承的 JVM options），並附上指令與環境變數清單。
3. **T011-B 條件的循環**：分成可排程、啟動前檢查、實跑後驗收三層，TASKS.md 同步。

## 怎麼驗證

- 雲端閱讀 `維護工具/run-teleport-failure.ps1` 全文（main，82 行），以及 `tests/native_teleport_failure.rs` 的第 26、105–132 行，上表引用的行號都來自這次閱讀。
- `python3 -m json.tool docs/plans/issue-82.json > /dev/null` 結束碼 0。

## 未驗證什麼

- 當版 Java harness 對 E149 JAR 能否編譯：由 H1、H2 在本機實際 javac 驗證。
- `Get-NetTCPConnection` 在使用者電腦的權限：由 H2 確認。
- 退出鏈、實際 listener 所有權、E149 資源在 memory 模式的初始化：只能在 T011-B 實跑時驗證。
- 雲端沒有執行任何測試或程序。

## 產出路徑

- `docs/plans/issue-82.json`
- `TASKS.md`（T011 改為 `blocked(T011-H1、T011-H2)`，並寫明路線與解鎖條件）
- `docs/reports/issue-82.md`（本檔）
