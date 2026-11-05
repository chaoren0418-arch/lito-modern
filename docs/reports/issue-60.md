# Issue #60 — T010-G 建置參數與產物路徑對齊

## 做了什麼

- `build.ps1` 的四種模式統一使用 `--locked --offline -j1`、`--target x86_64-pc-windows-msvc` 與專案內的 `target/current`。
- `build-evidence.ps1` 的 Debug／Release 執行檔解析與收據 Target 寫入、驗證改用 x64，新增 `target/current` 目錄安全檢查。舊 i686 收據必須重新建置取得 x64 收據。
- 建置與打包透過 `target/project-lease.lock` 共用互斥鎖。收據繼續由 `target/player-<mode>.build.json` 提供給既有呼叫端。
- 正式修改範圍為兩支指定腳本與本任務報告。

## 怎麼驗證

### 已驗證

- 在 Windows PowerShell 7 執行 scratch 檢查，24 項斷言通過，涵蓋語法解析、四種模式的完整 Cargo 指令、WhatIf 回傳與目錄檢查、預設／相對／絕對資源路徑、缺資源警告、非法路徑、缺 Cargo.toml、缺 helper，以及啟動腳本的 Debug／Release 產物路徑比對。
- 將既有 `tools/test-build-evidence.ps1` 複製到 scratch，只調整 helper／fixture 位置與 x64 常數，14 個既有 fixture 案例通過。
- 額外確認 i686 收據遭拒、x64 收據寫入後可通過驗證，以及 `target/current` 被檔案佔用時遭拒。收據寫入案例以 fixture 版本資料替代執行檔診斷，驗證過程未執行 Cargo 或遊戲。
- `git diff --check` 通過。

### 修改前的 `build.ps1 -WhatIf`

預設模式為 Test。以下為實際印出的 Cargo 欄位。

```text
cargo test --offline --locked --manifest-path D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-60\LiTo381-Modern\Cargo.toml --target-dir D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-60\LiTo381-Modern\target --target i686-pc-windows-msvc
```

### 修改後的 `build.ps1 -WhatIf`

```text
cargo test --locked --offline -j1 --manifest-path D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-60\LiTo381-Modern\Cargo.toml --target-dir D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-60\LiTo381-Modern\target\current --target x86_64-pc-windows-msvc
```

## 未驗證什麼

- 待驗證的實際 Windows 建置、clone 位置建置與真實執行檔收據診斷，交給 T010-F 之後的本機任務。
- GUI、登入、正式服務及 DB 均未執行。
- CI `cargo check` 於推送並建立 PR 後確認，最終狀態與執行連結記於 PR。
- reparse point 分支經靜態檢視，未建立 junction／symbolic link 執行測試。

## 產出路徑

- `LiTo381-Modern/tools/build.ps1`
- `LiTo381-Modern/tools/build-evidence.ps1`
- `docs/reports/issue-60.md`
- 本機未追蹤驗證材料位於 `scratch/issue-60/`，包含 `verify.ps1`、`verification.txt`、`before-whatif.txt`、`after-whatif.txt` 及 fixture。

## 給下一棒的提示

- Claude 審查兩支腳本與 PR 上的 CI 結果，使用者依專案流程合併。
- 本機執行器於 T010-F 之後執行實際 Windows 建置，檢查 `target/current/x86_64-pc-windows-msvc/<debug|release>/LiToModern.exe` 與 x64 收據。
- 既有 `tools/test-build-evidence.ps1` 仍含 i686 fixture 常數，直接執行會與新的產物路徑不相容。該檔超出本次指定範圍，請調度器安排後續更新。本次僅執行 scratch 中已調整的副本，14 案結果屬於該副本。
