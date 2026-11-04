# Issue #2 / T003 — 44 檔隔離候選盤點

## 做了什麼

- 讀 `AGENTS.md`／`CLAUDE.md`／`TASKS.md`／`PLAN.md`／`docs/HISTORY.md`。`docs/REPORT.md` 當時不存在，本輪新建。
- `維護工具/candidates/**` 不在 repo（`.gitignore`），未複製、未改候選。
- 用 `docs/HISTORY.md` 與 `維護工具/prepare-native-E149-*.py`、`develop-native-E149-*.py` 重建 **44** 個 overlay key（30 Runtime／11 Rust QA／3 Java）。`LiTo381-Modern/logs/continuation-20261003/` 是 2026-10-03 戰役紀錄，**不能**當 44 檔清單，已註明。
- 新建 `docs/CANDIDATES.md`：每檔一列（候選路徑／對應 `src/`／是否已併入／差異行數／阻擋原因／建議順序），並對正常來源是否有同名檔、插入腳本可追溯。
- 對 44 個相對路徑做存在檢查：20 個正常來源無同名檔（候選獨有模組／QA／Java）；其餘同名檔是 predecessor，契約皆 `production_applied=False`。
- 未改 `src/`、未跑測試、未啟動遊戲、未連 DB、未碰 `config/server.properties`／封存／releases／pak／map／sprite。

## 怎麼驗證

- 腳本計數：`develop-native-E149-static2941-queue.py` `require(len(files)==44 … runtime_files=30, Rust_QA_files=11, Java_files=3)`；HISTORY 2026-10-25 首條相同。
- 插入鏈：isolated 13 → bootstrap 14 → object374448 22（11 QA 固定）→ zombie41 30 → eruption45 33 → mass69-fire70 42 → ingress 43 → queue 44。每步 `require(len(files)==N)` 與新 path 對得上。
- 本 clone 檔案存在性：`python3` 對 44 條 overlay key 與 `compositor.rs` 等四個非 overlay pin 做 `Path.exists()`。
- 表格列數：30＋11＋3＝44，無重複 key。

## 未驗證什麼

- 候選本體 bytes／SHA／與 `src/` 的 diff（gitignore，無法計算差異行數）。
- 最新 candidates stamp 目錄名（在 gitignored `驗證紀錄/`）。
- 475 案、G06 640 次、GUI、PE、SPR（使用者暫緩；本任務禁止執行）。
- overlay 套用後能否 `cargo check`（未套用、未編譯）。
- Claude 尚未審閱，併入順序未定案。

## 產出路徑

- `docs/CANDIDATES.md`（任務指定）
- `docs/reports/issue-2.md`（本檔）
- `docs/REPORT.md`（本輪變更／下一步，Issue #2）

## 給下一棒的提示

- 接手：**Claude** 審 `docs/CANDIDATES.md`，決定整包或不套用；不要單檔 cherry-pick。
- 有候選本體的機器：用 `驗證紀錄/current/native-E149-static2941-ready-development/20261024-211350-252315/` 對 `維護工具/candidates/native-E149-static2941-ready-<stamp>/` 補 SHA 與 diff 行數。
- T020（#97）仍 blocked on T003；29–30 列（`native_connection.rs`、`native_static2941.rs`）分批時最後。
- 恢復測試需使用者同意（T010）。T004 協定契約可平行，不依賴本 overlay。
