# Issue #173 — T041 第 4 次執行報告（claude-local）

## 結論

T023 第 2 張（#170）合併以後，G06 的前置阻擋**已經解除**。`run-teleport-failure.ps1` 的 `ValidateSet` 和 `native_teleport_failure.rs` 都已經有八個 scenario，跟本套件的對照表逐字相同。

本輪把分支同步到 main，修正同步後出現的兩個腳本缺口：

1. 技能表新增了整檔參照與 `target::name` 參照。
2. PowerShell 5.1 讀 runner 原始碼時編碼錯誤，導致 G06 預檢失敗。

修正後，腳本自身回歸在 5.1／7 都通過，`cargo check` 結束碼 0。

依使用者 2026-11-09 的本機限制，本輪**沒有在本機跑任何 Rust 測試執行檔**，所以完整套件（基線、技能 66 項、G06 八案）的實跑結果是**待驗證**，要交給 Codex 本機執行。

## 做了什麼

- `git merge origin/main`：分支原本落後 137 個提交，合併沒有衝突。
- `維護工具/run-local-test-suite.ps1`
  - 新增 `Read-TargetTestNames`：靜態掃描 `tests/<target>.rs` 的 `#[test]` 函式，不編譯、不執行。
  - `Read-SkillEntries` 支援兩種新參照：
    - 整檔參照（`native_t027_batch1`、`native_t027_batch2`）：清單必須和原始碼的 `#[test]` 集合完全一致。狀態表若寫了「（N 案」，數量也要相符。檔案內有 `mod`／`include!`、有無法解析的 `#[test]`、或檔案不存在，一律失敗。
    - 限定參照（`native_t021::…`）。
  - `Read-G06SupportedScenarios` 改成先以 UTF-8 讀 runner 原始碼，再用 `ParseInput` 解析。原本用 `ParseFile`，在 5.1 會把沒有 BOM 的 UTF-8 中文當成 ANSI 解碼，造成解析錯誤，G06 八列因此全部預檢失敗。
- `維護工具/test-whitelist/skills.tsv`：從 41 項增加到 66 項。新增的是 batch1 13 項、batch2 11 項、t021 1 項，各列內容取自 main 已審查的 `tests.tsv`。
- `維護工具/test-local-test-suite.ps1`：新增 17 條合成斷言，涵蓋：
  - 整檔清單完整
  - 漏列、多列、限定參照遺漏
  - 「N 案」數量不符
  - `include!`
  - 原始碼新增測試
  - 註解夾在 `#[test]` 與 `fn` 之間
  - 檔案缺失
  - runner 已支援 T023-2 的三案
- `維護工具/test-whitelist/README.md`：同步新的範圍與數量。
- 成功門檻沒有改，和 T012 相同：每項只能有一個 harness，`running 1 test`、1 passed、failed／ignored／measured 都是 0，結束碼 0。0 tests、`failed>0`、解析失敗都不算通過。G06 只要有一案失敗，後續各案就列為「未執行」的失敗。

## 已驗證

- **G06 契約靜態對照。** 合併後的 runner `ValidateSet` 是 `timeout, cancel-expiry, cancel-expiry-source, cancel-retry, cancel-repeat, refused, refused-cancel-repeat, wrong-service`。`native_teleport_failure.rs` 裡有八個 `#[ignore]` 測試，名稱和套件對照表逐字相同：
  - `destination_timeout_…`
  - `refused_destination_…`
  - `wrong_service_destination_…`
  - `cancellation_then_immediate_retry_…`
  - `cancellation_retires_destination_…`
  - `repeated_cancellation_preserves_only_one_authenticated_recovery`
  - `failed_handoff_then_repeated_cancel_preserves_the_authenticated_recovery`
  - `expired_stage_and_source_handoff_require_the_requested_source_directory`

  runner 的 receipt 欄位也和套件的檢查一致：`status=verified_<scenario>`、`test_binary`、`jar_sha256`、`inputs_unchanged`、`cleanup_errors`、`drift_errors`、`root`（位於 `target/current/native-teleport-failure/`），以及 `rust.stdout.txt`。
- **技能清單對照**（以 PowerShell 靜態解析，未執行 Rust）：`Skills: 181 catalog rows; 36 references (2 whole targets); 66 exact tests; 123 rows have no tests (not verified).`
- **腳本自身回歸**：只跑 PowerShell 合成檢查，沒有 Cargo、Java 或 Rust 執行檔。
  - `powershell -NoProfile -ExecutionPolicy Bypass -File 維護工具/test-local-test-suite.ps1`：結束碼 0，輸出 `Runner QA: 94 assertions passed; PowerShell=5.1.19041.6456. Synthetic checks only.`
  - `pwsh -NoProfile -File 維護工具/test-local-test-suite.ps1`：結束碼 0，輸出 `Runner QA: 94 assertions passed; PowerShell=7.6.5. Synthetic checks only.`
  - 修正前，5.1 版在 `Cannot parse owned G06 runner` 失敗，這是本輪修正的依據。
  - 原始輸出存放在 `scratch/issue-173/attempt4-qa-ps51.log` 和 `attempt4-qa-ps7.log`。
- **cargo check**：`cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets` 結束碼 **0**（`Finished dev profile … in 3m 53s`，log：`scratch/issue-173/attempt4-cargo-check.log`）。check 不會產生 `test result:`。

## 推測

- 本機資源（`0-客戶端`）與 E149 JAR 備妥後，技能 66 項應能逐項通過 Exact 門檻。理由是這 25 個新增項目已經在 main 的基線 `tests.tsv` 裡，由 #145、#146 回報通過。這一點尚未在本分支實跑。
- G06 八案能否全部通過，取決於 owned harness 的實際環境（JDK、E149 JAR、loopback 埠、自然 33 秒與 31 秒的到期觀察），本輪無法推定。

## 待驗證（交 Codex 本機執行）

1. 在本 worktree（或合併後的 main）把已驗證的 E149 JAR 放到 `LiTo381-Modern/target/current/server-reference/formal-server-E1498871.jar`，然後執行 `pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1`。
2. 驗收條件：
   - 總表的 Baseline、技能（66 列）、G06（8 列）全部是 `PASS`。
   - 最後一行是 `Final exit code=0`。
   - 附上 summary.txt 和各 log 的 `test result:` 原文。
3. 也可以分類執行 `-Category Skills` 和 `-Category G06`，方便定位問題，但 M2 完成定義以 `-Category All` 的結果為準。
4. CI（`build.yml`）只跑 `cargo check` 和 `cargo test --lib`，不會執行本套件；本套件的實跑一律在本機完成。

## Cargo 原文

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
exit=0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3m 53s
```

本輪沒有執行 `cargo test`，原因是本機限制禁止執行編譯出的測試程式，所以沒有 `test result:` 可以附。測試結果以 PR CI 和 Codex 本機實跑為準。

## 產出路徑

- `維護工具/run-local-test-suite.ps1`
- `維護工具/test-local-test-suite.ps1`
- `維護工具/test-whitelist/skills.tsv`
- `維護工具/test-whitelist/README.md`
- `docs/reports/issue-173.md`
- 未追蹤的證據：`scratch/issue-173/attempt4-*.log`

## 給下一棒的提示

- 前置（#170）已完成，現在只差本機實跑。請由 Codex 本機（`codex-local`）依「待驗證」的步驟跑完整套件，以八案真實結果和完整套件結束碼 0 驗收。
- 以後 `docs/SKILLS-STATUS.md` 新增測試參照時，技能類會因為「Unmapped」或「case count mismatch」而失敗。修法是依錯誤訊息補 `skills.tsv`（可以從 `tests.tsv` 取已審查的列），不能放寬門檻。
- 123 列「尚無」測試的技能不在本套件的驗證範圍內；套件通過不代表那些技能已驗證。
