# Issue #43：使用者同意恢復測試，更新規則並排 M1 測試任務

## 做了什麼

- **確認同意**：#43 是本機調度器對話轉述的。雲端主控另外在本對話直接向使用者確認，使用者選擇「同意，全部 M1」（2026-11-05）。
- **規則**：`CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc` 三份同步，把「使用者曾指示只做開發，測試與桌面操作暫緩」那一行改成：
  - 已同意恢復測試；
  - 測試一律交本機 `dot`，備援 `codex-local`；
  - 不連正式服務或 DB，不啟停遊戲程序；
  - GUI 驗收等 M2 之後；
  - 結果只報已驗證／推測／待驗證，略過的測試不算通過。
- **PLAN.md**：「使用者指示」列、M1 標題、分工那一行同步更新。
- **TASKS.md**：
  - M1 段落標題同步更新；
  - T010 由 `blocked(使用者同意)` 改為 `todo`，負責人改為本機 dot／codex-local＋Claude，並列出拆分；
  - T011 仍是 `blocked(T010)`，負責人同步更新；
  - T005 現況中的「測試暫緩」改為「仍待驗證，交給 T010-F」。第 1 次嘗試曾誤寫成「由 T010-A 驗證」，已更正。
- **`docs/plans/issue-43.json`**：拆出 9 個任務（第 3 次嘗試修正後）：8 個本機任務，加 1 個 code 任務 T010-G。依 #43 的指示，front matter 不指定 runner，由調度器的 test 路由派工。

  | id | stage | 依賴 | 內容 |
  |---|---|---|---|
  | T010-A | test | — | `cargo check --all-targets`，以及 `cargo test --lib` 的 Debug 與 Release（不帶 `--ignored`），和 CI 基線（1888／0／575）比較 |
  | T010-S | resource | A | **靜態**安全分類：用 `-- --ignored --list` 與 `--tests -- --list` 只列名稱、不執行，讀原始碼後把每個 ignored 單元測試與整合測試分成安全／GUI／DB／不明副作用／缺前置條件，產出白名單 |
  | T010-B | test | S | 只執行白名單內的 ignored 單元測試，每個測試以 `-- --ignored --exact <全名>` 單獨執行；**禁止**沒有篩選的 `--ignored` |
  | T010-C | test | B | 只執行白名單內的整合測試，每個以 `--test <target> [--features …] -- --include-ignored --exact <全名>` 單獨執行；正式 JAR 唯讀，並先核對 SHA256 |
  | T010-D | resource | — | 盤點 12 gate 案、四元件 inventory、17 原控制＋6 反例、G06 640 次的定義、腳本與指令（只在使用者電腦上），不執行 |
  | T010-E | test | D、C | 依 D 的盤點執行 gate、inventory 與控制／反例 |
  | T010-F | test | A | 驗證 T005 建置腳本：`-WhatIf`、`-BuildOnly`（只建置不啟動）；clone 到 `scratch\clone-test\`、固定 commit 後，**實際執行** `-BuildOnly`，並確認產物位於 clone 之下；`build.ps1` 只跑 `-WhatIf`，實際建置等 T010-G |
  | T010-G | code | — | `build.ps1`／`build-evidence.ps1` 改成 x86_64-pc-windows-msvc、`target/current`、`-j1`（#3 審查時就已記錄的既有問題） |
  | T011-A | test | E | G06 兩版 16 案 640 次；首個失敗就停止，不自動重試 |

  依賴鏈是 A → S → B → C → E → G06，另外 D → E。所以 G06 一定排在 A、B、C、E 全部完成之後，符合 #43「最後是 G06」的要求。每個 test 任務都附共通規則：
  - cargo 選項放在 `--` 之前，測試程式參數放在 `--` 之後；
  - 不連 DB、不啟停遊戲、不開 GUI 驗收；
  - 略過不算通過；
  - 首個失敗保存原始收據，不自動重試；
  - 前置任務阻塞時，不可以解除後續門檻；
  - 只新增報告。

- **原生 GUI 驗收（T030）**：依 #43 的指示，等 M2 完成後再排，這次沒有拆出任務。

## 怎麼驗證

- 第 2 次嘗試實際執行：`python3 -m json.tool docs/plans/issue-43.json > /dev/null; echo "json.tool exit=$?"` 輸出 `json.tool exit=0`。另外檢查 8 筆的 depends_on 都指向存在的 id：`8 items; deps ok: True`。
- 三份規則檔只改同一行、內容一致，原本就都是 LF，沒有改換行格式。
- 最終差異（`git diff --stat origin/main...task/43`，在第 3 次嘗試最後一次推送時實際執行）：7 個檔案，+176／−10。第 2 次嘗試時是 +160／−10；第 1 次的「5 檔 +11／−10」只是三份規則、PLAN、TASKS 的小計。

### 第 2 次嘗試的修正（對應 codex-local 審查）

1. **`--no-fail-fast` 的位置**：移到 `--` 之前，並在共通規則中寫明工作目錄、cargo 共通參數，以及 `--` 前後各放什麼。
2. **ignored 測試會碰到 GUI／DB**：新增 T010-S，先做靜態安全分類，再產出白名單；審查舉的 `canonical_slow_sql_tests.rs:458-465`（SQL）與 `r383_official_window_host_tests.rs:103`（官方 EXE 視窗）指定必須分到 DB 和 GUI。B、C 都改成只依白名單、以 `--exact` 逐一執行，並禁止沒有篩選的全量指令。
3. **依賴順序**：改成 A → S → B → C → E → G06（另有 D → E），G06 不能跳過 B、C；並明定前置任務阻塞時，不可以解除後續門檻。
4. **T005 驗收**：撤回「由 T010-A 驗證」，恢復為待驗證，並新增 T010-F 驗證建置腳本。T010-F 只建置不啟動遊戲，clone 測試也放在專案內的 `scratch\`。

### 第 3 次嘗試的修正（對應 codex-local 第 2 輪審查）

1. **整合測試的 ignored 與 features**：T010-S 改成記錄每個整合測試是否 `#[ignore]`，並靜態讀 `Cargo.toml` 的 `required-features`，例如 `native_teleport_controller` 需要 `teleport-wire-test`（第 24–27 行）。這個 target 另外帶 feature 列名，不可以從盤點中消失。白名單改成四欄：target、全名、是否 ignored、features。T010-C 改用 `--include-ignored --exact`，必要時帶 `--features`。T010-B、T010-C 的驗收都要求輸出確認 `running 1 test`；`0 tests` 或 ignored 不算通過。
2. **T010-F 的 clone 實際建置**：clone 先 `git checkout` 到固定 commit，再實際執行 `-BuildOnly`（不啟動程式），並確認產物位於 clone 之下。只有 `-WhatIf` 成功時，只能算「路徑解析已驗證」，不可以解除 T005 的建置驗收。TASKS.md 已同步。
3. **`build.ps1` 和規則衝突**：T010-F 對 `build.ps1` 只跑 `-WhatIf`，並把 i686／`target/`／沒有 `-j1` 的問題列為「待驗證（等 T010-G）」，32 位元建置不可以算成 x64 驗收。新增 **T010-G（code）** 修正 `build.ps1` 與 `build-evidence.ps1`，所以不會讓只准新增報告的本機執行者臨時改腳本。
4. 交接段的「建立 6 個 Issue」已改成 9 個。

## 未驗證什麼

- 12 gate 案等 M1 項目的確切定義，在 repo 內查不到（在上層的總開發報告與 `驗證紀錄/`），所以交給 T010-D 在本機盤點，T010-E 與 T011-A 都依賴它。
- 本機 Codex runtime 的 `setup refresh had errors`（見 #41）可能影響這些任務。#41 最後是改走核准途徑才完成；如果再發生，各任務應回報為阻塞，不算完成。
- 沒有執行任何測試。

## 產出路徑

- `CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc`
- `PLAN.md`、`TASKS.md`
- `docs/plans/issue-43.json`
- `docs/reports/issue-43.md`（本檔）

## 給下一棒的提示

- 調度器依 plans 建立 9 個 Issue。T010-A 和 T010-D 沒有依賴，可以同時開始。
- 測試結果回來後，由 Claude 審查並同步 TASKS.md；失敗另開 bug Issue。
