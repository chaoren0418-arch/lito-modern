# Issue #76 / M1-R：M1 範圍改為當版 main；候選資格 gate 延後；G06 以當版重新設計

## 做了什麼

- **裁定**：原 M1 的 12 個 gate 案、四元件 inventory、17 原控制＋6 反例，以及舊版 G06 的 16 案 640 次，都是**凍結候選 overlay 的資格檢查**，在當版 main 上沒有對象，或依賴候選建置。#58（T010-E）與 #61（T011-A）以 not planned 關閉，不算通過；裁定理由寫在各自的留言中。
- **PLAN.md M1**：改成兩部分。
  - 「當版 main 驗證」：列出已驗證、待驗證項目，以及 G06（當版）的新路線。
  - 「候選資格（延後）」：上述四項，不計入 M1 通過。
- **TASKS.md**：
  - T010 → **done**，寫明已驗證範圍：#53、#54、#55、#56、#59。也寫明待驗證項目（白名單以外的 574 個 ignored 單元測試與 481 個整合測試、`build.ps1` 的實際執行），以及延後的候選資格檢查。
  - T011 改為 `blocked(T011-R)`，對象改成當版 `tests/native_teleport_failure.rs` 的 5 個 scenario。
- **`docs/plans/issue-76.json`**：2 個本機任務。
  - **T011-R（resource）**：盤點當版 owned harness，包括產生 `LITO_TELEPORT_ROOT`／`ready.json` 的工具、前置資源與 SHA256、是否指向候選，以及每個 scenario 的完整指令。不執行任何東西。
  - **T011-B（test，依賴 R）**：依 R 的判定，每個 scenario 只跑一次。重複次數等單次結果出來後再規劃。
- **自有隔離 DB 的界線**（寫在 plans 的共通規則中）：同時符合以下全部條件，才不算「連正式 DB」；任何一項做不到就不執行，回報為阻塞。
  - 本任務自行啟動，使用 `--no-defaults`；
  - datadir 與 tmpdir 都在專案的 `target/current` 或 `scratch/` 底下；
  - 只綁 127.0.0.1，port 由作業系統配發；
  - 只匯入固定的靜態表，不含玩家資料；
  - 結束時以 mysqladmin shutdown 關閉，並確認程序已退出。

## 怎麼驗證

- 依據都在 repo 或 Issue 中：#57 的「四、G06」節（plan、binding、fixture 都在 `維護工具/candidates/`），以及 #58 兩次 RESULT（12 gate 定義在候選 `native_connection.rs:827–850`）。
- 雲端在 main 上執行 `grep -rl static2941_queue_development_tests LiTo381-Modern/src LiTo381-Modern/tests`，結果為 0 筆。
- 舊 G06 的 8 個 scenario 名稱逐一 grep：當版 `tests/native_teleport_failure.rs` 只有 5 個，另外 3 個（repeated_cancellation…、failed_handoff_then_repeated_cancel…、expired_stage_and_source_handoff…）沒有。
- 當版 `tests/native_teleport_failure.rs` 的 5 個 `#[ignore]` 測試位於第 104、110、116、121、126 行，`run_owned_harness` 從第 129 行起；T010-S（#54）把它們分類為「缺前置條件」（未設定 `LITO_TELEPORT_SCENARIO`）。
- `python3 -m json.tool docs/plans/issue-76.json > /dev/null; echo "json.tool exit=$?"` 輸出 `json.tool exit=0`，共 2 筆。

## 未驗證什麼

- 當版 owned harness 的產生工具是否存在、是否可用，交給 T011-R 盤點。
- 自有隔離 DB 的條件能否在使用者電腦上滿足，同樣交給 T011-R 判定。
- 沒有執行任何測試。

## 產出路徑

- `PLAN.md`、`TASKS.md`
- `docs/plans/issue-76.json`
- `docs/reports/issue-76.md`（本檔）

## 給下一棒的提示

- 合併後，調度器依 plans 建立 T011-R 與 T011-B。
- 候選資格檢查（12 gate 等）只在重新評估候選時才需要，在那之前不要再排。
