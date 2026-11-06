# Issue #173 — T041 第 2 次：八案預檢與前置阻擋

**結論：未完成驗收，不能標為通過。** 本輪基線 80／80、技能 41／41 通過；G06 因 T023 第 2 張的三案尚未實作，在啟動 harness 前拒絕整組八案，完整套件結束碼 **1**。這次修改改善缺項診斷，**沒有解決前次審查要求的 G06 八案實跑全通過**。請調度器先安排前置實作，再重排本張驗收，避免在相同基準重跑。

## 做了什麼

沿用既有 worktree `D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-173` 與 `task/173`，起始提交 `a8b416d`；未 checkout、建分支或併入其他任務。

- 保留前次 T041 的三類調度、41 項技能精確白名單、receipt 核對與 T012 成功門檻。
- G06 先檢查 owned runner 的 `Scenario` 是否完整支援八案。缺任一案立即列出全部缺項，八列均為 `FAIL: precondition`，結束碼／測試數為 `N/A`；不啟動 G06 Cargo、pwsh 或 Java 子程序。
- 支援完整時，建置後以當次 Cargo artifact 執行 `--list --ignored --format terse`，要求八個精確 Rust 測試名稱各出現一次，再啟動 owned harness。列舉非零碼、缺項、重複、大小寫錯誤均拒絕；列舉本身不算測試通過。
- 補回歸斷言及白名單 README。未改 Rust、Java、owned runner、TASKS.md、docs/REPORT.md 或其他張的實作。

## 已驗證

### 前置缺項的證據

`TASKS.md:207–219` 明確將三個舊 scenario 的恢復列在 **T023 第 2 張**，而 T041 僅擴充本機套件。現行 `run-teleport-failure.ps1` 的 ValidateSet 只有五案。另以 `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-run --message-format=json` 取得本 worktree EXE，再執行 `--list --ignored --format terse`，建置與列舉皆結束碼 0，實際只有下列五項：

```text
cancellation_retires_destination_and_expired_handoff_requires_fresh_selection: test
cancellation_then_immediate_retry_consumes_staged_character_once: test
destination_timeout_retires_old_authority_and_allows_fresh_login: test
refused_destination_retires_authority_and_allows_fresh_login: test
wrong_service_destination_retires_authority_and_allows_fresh_login: test
```

此命令只列舉，不執行測試，沒有 `test result:`，不能充作五案通過。原始證據在 `scratch/issue-173/retry-inventory-build.log`、`retry-inventory-build.stderr.log`、`retry-inventory.log`。

`git -c http.sslBackend=openssl ls-remote origin refs/heads/main` 回傳 `6629b96e8855a4c6f4b1d03d288d0f2eabe74034`，與已存在的 `origin/main` 相同；唯讀檢查該提交的 owned runner 與 Rust 測試，也僅五案。未將 main 合併進本分支。另唯讀 `task/169` 報告確認它是 T023 第 1 張，將 dual-expiry scenario 接入留給第 2 張；未搬入該分支變更。

### 腳本自身回歸

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/test-local-test-suite.ps1
```

兩者結束碼均 **0**：

```text
Runner QA: 77 assertions passed; PowerShell=5.1.19041.6456. Synthetic checks only.
Runner QA: 77 assertions passed; PowerShell=7.6.5. Synthetic checks only.
```

覆蓋完整八案調度、五案契約拒絕且無子程序、Rust inventory 缺項／重複／大小寫／非零碼／空輸出、0 tests、failed／ignored／measured、結果解析失敗、receipt／清理錯誤及首敗停止。八案正向使用合成 adapter，**不代表 G06 實跑通過**。證據：`scratch/issue-173/retry-runner-qa-ps51.log`、`retry-runner-qa-ps7.log`。

### 完整套件與 Cargo

在 worktree 根目錄執行：

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/run-local-test-suite.ps1
```

完整結束碼 **1**，129 列的結果如下：

| 類別 | 列數 | PASS | FAIL／未執行 |
|---|---:|---:|---:|
| Baseline | 80 | 80 | 0 |
| 技能 (Skills) | 41 | 41 | 0 |
| G06 | 8 | 0 | 8（前置不足，未執行） |

指定命令 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 **0**（001.log；check 不產生 `test result:`）。

Debug／Release lib 命令為 `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast`，Release 另加 `--release`。結束碼皆 **0**，原文：

```text
002.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 3.36s
003.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 2.00s
```

技能 41 項均按 skills.tsv 使用 `cargo test`、相同共同參數及 `--test <target>`（或 `--lib`）、`--no-fail-fast -- --include-ignored --exact <name>`；每項結束碼 0，完整 `test result:` 原文列於附錄，命令及名稱見各原始 log。

八個 G06 列皆為下列前置失敗，沒有 Rust `test result:`，不填入前次執行數字：

```text
FAIL: precondition: incomplete-owned-scenarios (T023-2): cancel-repeat, refused-cancel-repeat, cancel-expiry-source
Steps=129 | Final exit code=1
```

完整總表與 001–121.log：`LiTo381-Modern/target/current/verification/local-test-suite/20261106-155833-653-12608/`。外層 console 與結束碼：`scratch/issue-173/retry-final-suite.log`、`retry-final-suite.exit`。本輪未啟動 G06 harness、遊戲或 GUI，未連正式服務／DB；新產物僅在此 worktree 的 scratch／target。

## 推測

沒有以推測填入通過結果。新增的 inventory 守門可攔截只擴充 runner 名稱、尚未加入 Rust 測試的整合錯誤；此行為已有合成反例驗證，但完整八案仍須在前置實作完成後實跑。

## 待驗證／未完成事項

- 缺 `cancel-repeat`、`refused-cancel-repeat`、`cancel-expiry-source` 的 Rust／Java／owned runner 實作。依使用者「只做本張」與 TASKS 分張，不在 T041 擅自承接 T023 第 2 張，也不以其他測試替代。
- G06 八案實跑全通過、完整套件結束碼 0 及 M2 完成，均未達成。
- 技能表 143 列「尚無」測試、575 個 ignored lib 項（其中基線另精確執行白名單 1 項）、完整 GUI 與伺服器同源性均不由本次結果推定通過。

## 產出路徑

- `維護工具/run-local-test-suite.ps1`
- `維護工具/test-local-test-suite.ps1`
- `維護工具/test-whitelist/README.md`
- `docs/reports/issue-173.md`

## 給下一棒的提示

調度器應將 #173 標為等待 T023 第 2 張的前置實作，而不是以相同 main 反覆重排驗收。T023 執行者補齊三案及 runner ValidateSet／精確名稱／清理契約後，先完成該張審查與整合，再交本機 codex-local 重跑此套件；Claude 應依八案真實結果審查。不可把本報告當成 #173 或 M2 完成證據。REPORT 索引仍由調度器處理。

## 附錄：本輪技能 `test result:` 原文

```text
081.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
082.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
083.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
084.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
085.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
086.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
087.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
088.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
089.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
090.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
091.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.40s
092.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
093.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
094.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
095.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
096.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s
097.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.11s
098.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
099.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
100.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
101.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
102.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
103.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
104.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
105.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
106.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
107.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s
108.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
109.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
110.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
111.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
112.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
113.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.04s
114.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
115.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.06s
116.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
117.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s
118.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
119.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
120.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
121.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```
