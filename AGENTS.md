# lito-modern — LiTo381 重建版（Rust 原生客戶端）

本檔是所有 AI 工具（Claude、Cursor、Codex、ChatGPT）與人的共同規則。`AGENTS.md` 與 `.cursor/rules/project.mdc` 是本檔的複本，改動請只改本檔並重新同步。
本 repo 根目錄是 `D:\lineage開發\3-lito開發\lito重建版`。相鄰的登入器／伺服器是獨立 repo（lito-launcher-server），**唯讀**引用。

## 對話分工與語言（使用者規定，所有 AI 必讀）
- **全程使用繁體中文**：對使用者的回報、Issue／PR 留言、報告文件一律繁體中文（程式碼、指令、檔名維持原文）。
- 每個專案（含調度器本身）都有兩個 Claude 對話：
  - **雲端對話**：監控與回報——看 GitHub 佇列、規劃（plan／design）、審查裁定、處理 `needs:claude`；碰不到使用者電腦。
  - **本機對話**（Claude 桌面版「本機」環境）：處理需要本機的事——本機資源、Windows 建置與檢查、測試前置、本機檔案整理；**使用者平常只跟本機對話溝通**。
- 雲端對話需要本機動作時：開 Issue（`stage: test` 或 `stage: resource`、`runner: codex-local`）交給本機執行，或標 `needs:local` 交給本機對話；**不要叫使用者做**。
- 調度器本身的問題標 `needs:orchestrator`；只有真機驗收（啟動遊戲／登入／看畫面）與不可逆決策才標 `needs:user`。
- 使用者只負責驗收與觀測。

## 目標
以 Rust 重建 LiTo 客戶端為 Windows x64 原生程式（目標 G01–G12），沿用既有資源與 UI 重現已驗證行為：登入→大廳→進房→戰鬥→退出，含 HUD、聊天、背包、快捷列、技能特效。

## 目前狀態（2026-11-03）
- 正常來源 SHA `3E58268A…`（1048 來源檔）；正式唯讀 JAR `E1498871…`。
- 開發停在 **#97**（Scene／History 接線、資源就緒 gate）；最新隔離候選 44 檔**不可套用**。
- **475 個測試案全未跑**；G06（傳送、角色接續、退役、失敗恢復）640 次計畫未執行；完整 GUI 驗收未做。
- 使用者已同意恢復測試（2026-11-05，Issue #43）：測試一律交本機執行（`dot` 開本機 Codex 對話，備援 `codex-local`），雲端只負責規劃與審查。仍不連正式服務或 DB、不啟停既有遊戲程序；原生 GUI 的完整驗收（T030）等 M2 完成後再排；例外是 2026-11-06 使用者同意（#86）：本機執行者可以啟動**自己建置的**客戶端，進入不連線的畫面（設定畫面、登入畫面但不按連線、離線場景）並截圖，結束後依 PID 關閉自己啟動的程序。測試結果只報「已驗證／推測／待驗證」，略過的測試不算通過。
- 歷史報告 `LiTo381-總開發報告.md`（在上層目錄）、`docs/HISTORY.md`、`驗證紀錄/` 只供參考，不再是流程的一部分。

## 技術棧與建置
- Rust，`x86_64-pc-windows-msvc`，`Cargo.toml` 在 `LiTo381-Modern/`。
- 建置：`tools/build.ps1`；唯一正常入口 `啟動現代客戶端.ps1`。
- Cargo 參數：`--locked --offline -j1`，target 目錄 `target/current`；Release：`z`／thin LTO／16 codegen／strip。
- `CARGO_HOME=D:\dev-cache\cargo`、`RUSTUP_HOME=D:\dev-cache\rustup`（已設環境變數）。
- 資源：引用 `D:\lineage開發\0-客戶端` 與 `D:\lineage開發\客戶端xml\effectlist2.xml`（唯讀），不複製進本專案。
- 不包裝 `LiTo.bin`。

## 目錄
| 路徑 | 用途 | git |
|---|---|---|
| `LiTo381-Modern/src`, `tests`, `build.rs`, `Cargo.*` | 原始碼 | 追蹤 |
| `LiTo381-Modern/tools`, `config`, `packaging`, `windows` | 建置與打包腳本 | 追蹤 |
| `docs/` | 架構（`ARCHITECTURE.zh-TW.md`）、歷史 | 追蹤 |
| `LiTo381-Modern/target`, `dist`, `logs`, `reports`, `provenance` | 產物與證據 | **不追蹤** |
| `驗證紀錄/`, `維護工具/candidates/`, `封存/`, `交付/`, `整理紀錄/` | 歷史證據與封存 | **不追蹤** |
| `scratch/` | 暫存 | **不追蹤** |

## AI 產出規則（三個工具共用）
1. 只寫本專案資料夾內；不寫 `C:\`、桌面、`%TEMP%`、家目錄。
2. 暫存／實驗／一次性腳本一律進 `scratch/`。
3. 同一主題的報告只保留 `docs/` 下最新版，歷史靠 git；停止每輪新增時間戳 evidence JSON 與 `工作佇列.json`（舊檔已達 11 MB）。
4. 建置產物只進 `target/`、`dist/`；不進 git。
5. 不以 stub、假畫面冒充功能完成；報告分「已驗證／推測／待驗證」。
6. 不啟動、附加、停止既有遊戲程序；不連正式服務或 DB。

## 工作方式
- `PLAN.md`：里程碑與目前狀態。`TASKS.md`：任務清單，每項含範圍、驗收條件、狀態。
- 分支：`main` 永遠 `cargo check` 通過；工作在 `feat/<任務編號>-<短名>`，完成開 PR，Claude 審查、使用者合併。
- Cursor 執行任務時：先讀本檔與 `TASKS.md` 指定項，只改任務列出的檔案範圍，PR 描述寫「做了什麼／怎麼驗證／未驗證什麼」。
- 測試分層：`cargo test`（單元）→ 離線 fixture → 原生 GUI（完整原生 GUI 驗收只由使用者執行；不連線畫面截圖的例外見「目前狀態」）。
