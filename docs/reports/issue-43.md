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
  - T005 現況中的「測試暫緩」改為「由 T010-A 驗證」。
- **`docs/plans/issue-43.json`**：拆出 6 個本機任務。依 #43 的指示，front matter 不指定 runner，由調度器的 test 路由派工。

  | id | stage | 依賴 | 內容 |
  |---|---|---|---|
  | T010-A | test | — | `cargo check --all-targets`，以及 `cargo test --lib` 的 Debug 與 Release，和 CI 基線（1888／0／575）比較 |
  | T010-B | test | A | 需資源、證據的 ignored 測試（488＋87），只讀資源路徑；需要視窗的不執行 |
  | T010-C | test | A | `cargo test --tests` 整合測試，只用 loopback 隔離 harness 與正式 JAR（唯讀，先核 SHA256） |
  | T010-D | resource | — | 盤點 12 gate 案、四元件 inventory、17 原控制＋6 反例、G06 16 案 640 次的定義、腳本與指令（只在使用者電腦上） |
  | T010-E | test | D、A | 依 D 的盤點，執行 gate、inventory 與控制／反例 |
  | T011-A | test | D、E | G06 兩版 16 案 640 次；首個失敗就停止，不自動重試 |

  每個 test 任務都附共通規則：cargo 參數、不連 DB、不啟停遊戲、不開 GUI 驗收、略過不算通過、首個失敗保存原始收據且不自動重試、只新增報告。

- **原生 GUI 驗收（T030）**：依 #43 的指示，等 M2 完成後再排，這次沒有拆出任務。

## 怎麼驗證

- `python3 -m json.tool docs/plans/issue-43.json > /dev/null; echo "json.tool exit=$?"` 輸出 `json.tool exit=0`，共 6 筆。
- 三份規則檔只改同一行、內容一致，原本就都是 LF，沒有改換行格式。差異共 5 個檔案：+11／−10（`git diff --stat`）。

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

- 調度器依 plans 建立 6 個 Issue。T010-A 和 T010-D 沒有依賴，可以同時開始。
- 測試結果回來後，由 Claude 審查並同步 TASKS.md；失敗另開 bug Issue。
