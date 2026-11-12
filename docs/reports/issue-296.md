# Issue #296 — M3 開放

執行者：調度器雲端對話（主控對話未在線）。

## 做了什麼
- 使用者 2026-11-12 17:03 同意後，把 `TASKS.md` Backlog「預定 M3」的 44 張原樣寫成 `docs/plans/issue-296.json`（沒有加項目；T033 已完成不計）。
- `TASKS.md`、`PLAN.md` 把 M3 標為開放。
- 分派：30 張 `claude-local`；10 張 `codex-local`、4 張 `dot`（要執行測試或開遊戲，等 Codex）。

## 已驗證／推測／待驗證
- 已驗證：張數 44，與 TASKS.md 各任務的張數加總一致；依賴關係無循環。
- 推測：分張的範圍切法（TASKS.md 只給張數的任務，由本次依內容切成兩半）。
- 待驗證：`claude-local` 不能在本機跑 `cargo test`，T036 的審查張只做靜態審查，放行項目的實跑在 Codex 張。
