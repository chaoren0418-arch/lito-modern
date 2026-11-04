# REPORT — lito-modern

本檔由各輪任務追加「本輪變更」與「下一步」。同一主題只留最新版，歷史靠 git。

## 本輪變更
- Issue #3（T005）：`啟動現代客戶端.ps1` 與 `LiTo381-Modern/tools/build.ps1` 不再把客戶端資源寫死成唯一路徑；改讀 `LITO_CLIENT_ROOT`（未設時預設仍為 `D:\lineage開發\0-客戶端`），專案／Cargo 路徑改相對 repo 根。Cargo 參數維持原樣（啟動腳本 `--locked --offline -j1` + `target/current`；`build.ps1` 仍 `--offline --locked` + `target` + `i686-pc-windows-msvc`）。兩腳本支援 `-WhatIf`，必要路徑缺失時丟明確錯誤。詳見 `docs/reports/issue-3.md`。

## 下一步
- Issue #3：請 Claude 審查 `task/3` PR，確認 CI `cargo check` 綠、`-WhatIf` 與缺路徑錯誤可接受後合併。測試檔與 `維護工具/` 裡其餘 `D:\lineage開發\...` 硬編不在本任務範圍。
