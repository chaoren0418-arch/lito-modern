# Issue #173 — T041 第 3 次執行報告

## 結論

**驗收仍受 T023 第 2 張前置缺項阻擋。** 2026-11-06 本輪完整套件結束碼 **1**。基線 80／80、技能 41／41 通過，G06 八案均因前置不足而未執行。T041 要求的完整套件結束碼 0 尚待達成。

## 做了什麼

- 沿用指定 worktree 與 `task/173`，起始提交 `359dcdef436fc551da6ff5f2034c23ecabf471c2`。
- 讀取 AGENTS.md、docs/REPORT.md、TASKS 的 T012／T026／T023／T041，以及前次報告與既有腳本。
- 核對遠端 main、owned runner 的 Scenario 契約與本輪 Cargo artifact 的實際 Rust 測試名稱。
- 重跑完整套件及 PowerShell 5.1／7 腳本回歸，更新本報告。本輪追蹤檔變更只有本報告，既有兩次提交的技能／G06 調度及門檻供後續整合驗收使用。
- 依分張範圍將三案實作留給 T023 第 2 張，保留缺項失敗門檻。

## 已驗證

### 遠端與前置缺項

`git -c http.sslBackend=openssl ls-remote origin refs/heads/main refs/heads/task/170 refs/heads/task/173` 結束碼 0，回傳 main `59f0c2bcd7dc4435ba43186f7767130ab7869af1` 與 task/173 `359dcdef436fc551da6ff5f2034c23ecabf471c2`。本機 origin/main 與遠端 main 一致。以 `git diff origin/main HEAD -- LiTo381-Modern/tests/native_teleport_failure.rs 維護工具/run-teleport-failure.ps1` 核對，兩檔無差異。

當版 owned runner 的 ValidateSet 只有 `timeout`、`cancel-expiry`、`cancel-retry`、`refused`、`wrong-service`。缺少 `cancel-repeat`、`refused-cancel-repeat`、`cancel-expiry-source`，對應 TASKS.md 的 T023 第 2 張。

本輪在 `LiTo381-Modern/` 執行下列建置與列舉，兩者結束碼均 0。建置使用 worktree 內 TEMP／TMP，執行檔取自這次 Cargo JSON artifact。

```powershell
cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_teleport_failure --no-run --message-format=json
& $binary --list --ignored --format terse
```

列舉原文如下。此步僅取得 inventory，沒有執行 Rust 測試。

```text
cancellation_retires_destination_and_expired_handoff_requires_fresh_selection: test
cancellation_then_immediate_retry_consumes_staged_character_once: test
destination_timeout_retires_old_authority_and_allows_fresh_login: test
refused_destination_retires_authority_and_allows_fresh_login: test
wrong_service_destination_retires_authority_and_allows_fresh_login: test
```

證據位於 `scratch/issue-173/attempt3-inventory-build.log`、`attempt3-inventory-build.stderr.log`、`attempt3-inventory.log`，重現腳本為 `scratch/issue-173/attempt3-inventory.ps1`。

### 腳本回歸

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/test-local-test-suite.ps1
```

兩者結束碼均 0，原文如下。

```text
Runner QA: 77 assertions passed; PowerShell=5.1.19041.6456. Synthetic checks only.
Runner QA: 77 assertions passed; PowerShell=7.6.5. Synthetic checks only.
```

回歸涵蓋八案合成調度、缺案拒絕且無子程序、inventory 缺項／重複／大小寫／非零碼／空輸出、0 tests、結果解析失敗、receipt／清理錯誤與首敗停止。合成 adapter 的結果僅驗證腳本行為，G06 實跑仍待前置完成。原始輸出為 `scratch/issue-173/attempt3-qa-ps51.log` 與 `attempt3-qa-ps7.log`。

### 完整套件與 Cargo

在 worktree 根目錄執行下列命令，結束碼 1。

```powershell
& 'D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -File 維護工具/run-local-test-suite.ps1
```

實際以 `scratch/issue-173/run-attempt3.ps1` 包裝相同入口，保存 console 與結束碼至 `attempt3-suite.log`／`attempt3-suite.exit`。

| 類別 | 列數 | PASS | FAIL／未執行 |
|---|---:|---:|---:|
| Baseline | 80 | 80 | 0 |
| 技能 (Skills) | 41 | 41 | 0 |
| G06 | 8 | 0 | 8，前置不足 |

完整總表與 001–121.log 位於 `LiTo381-Modern/target/current/verification/local-test-suite/20261106-160954-518-476/`。

指定的 `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 **0**，證據為 001.log 與 summary.txt 的 check 列。check 本身無 `test result:`。

Debug／Release 的命令為 `cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --no-fail-fast`，Release 另加 `--release`。兩者結束碼均 0。基線其餘 77 項與技能 41 項逐一通過 Exact 門檻，每項結束碼 0、`running 1 test`、1 passed、0 failed／ignored／measured。相關 `test result:` 原文附於文末。

八個 G06 列的失敗原因與最終結果原文如下。

```text
FAIL: precondition: incomplete-owned-scenarios (T023-2): cancel-repeat, refused-cancel-repeat, cancel-expiry-source
Steps=129 | Final exit code=1
```

G06 八列的程序結束碼與計數均為 N/A。本輪 G06 harness 未啟動，未啟動遊戲、GUI、正式服務或 DB。新產物寫入本 worktree 的 scratch／target。

## 推測

三案整合後，既有八案調度可進入完整實跑階段，仍須以實際輸出驗收。整合後的成功率及結果待驗證。

## 待驗證／未完成事項

- T023 第 2 張的三案 Rust／Java／owned runner 實作與整合。
- G06 八案實跑全通過及完整套件結束碼 0。
- 技能表 143 列「尚無」測試、完整 GUI 與伺服器同源性。本次只對表內已有名稱的 41 項測試回報結果。
- GitHub CLI 查詢 Issue #173 與 T023 清單時回傳 HTTP 401，最新審查留言無法取得。本輪依使用者提供的退回內容與實際 Git／測試證據處理。

## 產出路徑

- 本輪更新 `docs/reports/issue-173.md`。
- 既有實作 `維護工具/run-local-test-suite.ps1`、`維護工具/test-local-test-suite.ps1`、`維護工具/test-whitelist/skills.tsv`、`維護工具/test-whitelist/README.md`。
- 本輪原始證據位於上述 `scratch/issue-173/attempt3-*` 與 `target/current/verification/local-test-suite/20261106-160954-518-476/`，皆為未追蹤產物。

## 給下一棒的提示

調度器先安排 T023 第 2 張完成三案、精確測試名稱及 runner 契約，經審查整合後再排本機 codex-local 執行 T041。相同五案基準已重現相同阻擋，應將 #173 設為等待前置。Claude 審查以八案真實結果及完整套件結束碼為準。REPORT 索引由調度器處理。

## 附錄 — 本輪相關 Cargo 測試原文

下列每個 log 的命令結束碼均為 0，可與完整 summary.txt 逐列核對。002／003 為 Debug／Release lib，081–121 為技能類。

```text
002.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 3.32s
003.log; exit=0
test result: ok. 1915 passed; 0 failed; 575 ignored; 0 measured; 0 filtered out; finished in 1.83s
081.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
082.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.01s
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
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s
090.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
091.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 1.34s
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
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 15 filtered out; finished in 0.00s
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
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
114.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.01s
115.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
116.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
117.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.00s
118.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.01s
119.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
120.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2489 filtered out; finished in 0.00s
121.log; exit=0
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```
