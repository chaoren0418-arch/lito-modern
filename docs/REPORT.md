# REPORT — lito-modern 本輪紀錄

本檔給調度器／下一棒看「這一輪改了什麼、下一步誰接」。歷史細節在 `docs/HISTORY.md` 與 git。不取代 `PLAN.md`／`TASKS.md`。

## 現況（截至 Issue #2）

- 正常來源 SHA `3E58268A…`；正式 JAR `E1498871…`。
- 最新隔離候選 44 檔（30 Runtime／11 Rust QA／3 Java）**不可套用**；盤點見 `docs/CANDIDATES.md`。
- 475 案全未跑；G06 640 次未跑；#97 未完整。
- 使用者指示測試與桌面暫緩。

## 本輪變更

- Issue #2 / T003：新建 `docs/CANDIDATES.md`，盤點 44 檔隔離候選（候選路徑／對應 `src/`／是否已併入／差異行數／阻擋原因／建議順序）。候選目錄不在 repo，來源為 `docs/HISTORY.md` 與 `維護工具/` 的 prepare／develop 腳本；`continuation-20261003` 已排除。詳 `docs/reports/issue-2.md`。
- Issue #3（T005）：`啟動現代客戶端.ps1` 與 `LiTo381-Modern/tools/build.ps1` 不再把客戶端資源寫死成唯一路徑；改讀 `LITO_CLIENT_ROOT`（未設時預設仍為 `D:\lineage開發\0-客戶端`），專案／Cargo 路徑改相對 repo 根。Cargo 參數維持原樣（啟動腳本 `--locked --offline -j1` + `target/current`；`build.ps1` 仍 `--offline --locked` + `target` + `i686-pc-windows-msvc`）。兩腳本支援 `-WhatIf`，必要路徑缺失時丟明確錯誤。詳見 `docs/reports/issue-3.md`。

## 下一步

- Issue #2 / T003：Claude 審閱 `docs/CANDIDATES.md`，決定整包或不套用（不要單檔 cherry-pick）。有候選本體時再補 SHA 與 diff 行數。T020（#97）仍等此審閱；恢復測試仍要使用者同意（T010）。
- Issue #3：請 Claude 審查 `task/3` PR，確認 CI `cargo check` 綠、`-WhatIf` 與缺路徑錯誤可接受後合併。測試檔與 `維護工具/` 裡其餘 `D:\lineage開發\...` 硬編不在本任務範圍。
- 2026-11-04 — #3 T005: 啟動與建置腳本去硬編路徑（LITO_CLIENT_ROOT）（code，codex-local，PR #5 已合併）→ [報告](reports/issue-3.md)
- 2026-11-04 — #4 T004: docs/PROTOCOL-CONTRACT.md 協定契約（Claude 撰寫）（design，claude-chat，分支 task/4）→ [報告](reports/issue-4.md)
- 2026-11-05 — #4 T004: docs/PROTOCOL-CONTRACT.md 協定契約（Claude 撰寫）（design，claude-chat，PR #12 已合併）→ [報告](reports/issue-4.md)
- 2026-11-05 — #13 T004-1: T004-1: 以正式 JAR e1498871 靜態核對 PROTOCOL-CONTRACT 常數（resource，codex-local，PR #19 已合併）→ [報告](reports/issue-13.md)
