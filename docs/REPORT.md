# REPORT — lito-modern 本輪紀錄

本檔給調度器／下一棒看「這一輪改了什麼、下一步誰接」。歷史細節在 `docs/HISTORY.md` 與 git。不取代 `PLAN.md`／`TASKS.md`。

## 現況（截至 Issue #2）

- 正常來源 SHA `3E58268A…`；正式 JAR `E1498871…`。
- 最新隔離候選 44 檔（30 Runtime／11 Rust QA／3 Java）**不可套用**；盤點見 `docs/CANDIDATES.md`。
- 475 案全未跑；G06 640 次未跑；#97 未完整。
- 使用者指示測試與桌面暫緩。

## 本輪變更

- Issue #2 / T003：新建 `docs/CANDIDATES.md`，盤點 44 檔隔離候選（候選路徑／對應 `src/`／是否已併入／差異行數／阻擋原因／建議順序）。候選目錄不在 repo，來源為 `docs/HISTORY.md` 與 `維護工具/` 的 prepare／develop 腳本；`continuation-20261003` 已排除。詳 `docs/reports/issue-2.md`。

## 下一步

- Issue #2 / T003：Claude 審閱 `docs/CANDIDATES.md`，決定整包或不套用（不要單檔 cherry-pick）。有候選本體時再補 SHA 與 diff 行數。T020（#97）仍等此審閱；恢復測試仍要使用者同意（T010）。
