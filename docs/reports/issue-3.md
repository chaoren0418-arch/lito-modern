# Issue #3 / T005 — 啟動與建置腳本去硬編路徑

## 做了什麼

- `啟動現代客戶端.ps1`：資源根依序取 `-ResourceRoot`、`LITO_CLIENT_ROOT`、原預設值 `D:\lineage開發\0-客戶端`；相對路徑以 repo 根展開。需要資源的模式在目錄不存在時明確報錯，缺少 `Cargo.toml` 也會報錯。
- `LiTo381-Modern/tools/build.ps1`：以 `$PSScriptRoot` 解析 repo／專案根，解析同一環境變數與預設值；缺少 manifest 或 helper 時報錯，缺少客戶端樹時警告。
- 兩腳本提供 `SupportsShouldProcess`／`-WhatIf`，在建目錄、取得 lock、呼叫 Cargo 或啟動程式之前返回；不複製 pak/map/sprite。
- Cargo 參數保留原樣：啟動腳本為 `build -j 1 --offline --locked --target x86_64-pc-windows-msvc --target-dir target/current --bin LiToModern`；build.ps1 仍為 `--offline --locked --manifest-path <project>/Cargo.toml --target-dir <project>/target --target i686-pc-windows-msvc`，Release 模式仍加 `--release`。後者既有的 i686／target／無 `-j1` 不在本次修改範圍。
- 第 3 次嘗試撤回 `TASKS.md` 的 T005 `todo → review` 差異；修正本報告與 PR #5 的驗證敘述。兩腳本沿用審查版本 `85f53315a0f9f859d6a6987b6a0138ea1310eea0`，本輪未再改動。
- 本輪未編輯 `docs/REPORT.md`；保留前兩次提交已存在的 Issue #2／#3 索引，由調度器處理後續索引。

## 怎麼驗證

### 已驗證：本輪靜態核對與既有 CI 紀錄

- 本輪唯讀核對兩腳本的資源根優先序、repo 相對路徑、缺路徑訊息、ShouldProcess 返回位置與 Cargo 參數；這是靜態核對，不是重跑 dry-run 或功能驗收。
- CI [run 41785721942](https://github.com/chaoren0418-arch/lito-modern/actions/runs/41785721942) 對應 head `85f53315a0f9f859d6a6987b6a0138ea1310eea0`。本輪透過 `gh run view` 核對 head SHA、步驟結果及原始日誌：
  - `cargo check --locked --all-targets`：success。
  - `cargo test --locked --lib --no-fail-fast`：success；1888 passed、0 failed、575 ignored。
- 上述 CI 證據只對應該歷史 head；第 3 次提交的 CI 應以 PR #5 最新 head 的 Checks 為準，不能用舊 run 宣稱新提交已通過。

### 歷史 dry-run：執行者回報、原始輸出未保留

來源是 [85f5331 的舊報告](https://github.com/chaoren0418-arch/lito-modern/blob/85f53315a0f9f859d6a6987b6a0138ea1310eea0/docs/reports/issue-3.md)。該版本記述 Linux 雲端 PowerShell 7.4.13 的執行摘要，但沒有可追溯的原始 stdout／stderr 或 artifact；無法獨立確認當時執行，以下不列為本輪已驗證結果，也不是原始輸出節錄。

| 歷史報告中的命令／情境 | 執行者回報的結果（未獨立驗證） |
| --- | --- |
| `pwsh -NoProfile -File ./啟動現代客戶端.ps1 -WhatIf -BuildOnly` | 退出碼 0，列出原 Cargo 參數，未建驗證紀錄或 target |
| 同上加 `-BuildProfile Release` | Cargo 命令加 `--release` |
| `-WhatIf -Verify`，預設資源不存在 | 退出碼 1，`Resource root not found` |
| `-WhatIf -Verify -ResourceRoot /no/such/lito-client-root` | 退出碼 1，明確缺路徑 |
| `LITO_CLIENT_ROOT=/tmp` 或 `-ResourceRoot docs`，搭配 `-WhatIf -Verify` | 退出碼 0，相對路徑展開到 repo 根 |
| 把啟動腳本拷到空目錄執行 | 退出碼 1，`Cargo.toml not found` |
| `pwsh -NoProfile -File ./LiTo381-Modern/tools/build.ps1 -WhatIf` | 退出碼 0，缺客戶端樹只警告，保留原 target／i686 參數 |
| 同上加 `-Mode Release` | Cargo build 加 `--release`，target 不變 |

## 未驗證什麼

- 執行者本機／雲端工作環境未手動執行 `cargo test`；本輪也未執行建置、dry-run 或遊戲。這不包含 GitHub Actions：上述 CI 已執行 `cargo test --locked --lib --no-fail-fast` 並成功。
- 尚未在 Windows 原開發機對實體客戶端樹完成真實建置、資源驗證或 GUI 驗收；clone 到其他路徑的實際建置仍待驗證。
- 未連正式服務或 DB，未啟動、附加、停止既有遊戲程序。測試與桌面操作仍維持暫緩。
- 測試檔及 `維護工具/` 內其他硬編路徑未改；本報告不宣稱 G06 或完整測試計畫已通過。

## 產出路徑

- `啟動現代客戶端.ps1`、`LiTo381-Modern/tools/build.ps1`：本任務既有實作，本輪保留。
- `docs/reports/issue-3.md`：本輪修正驗證證據、限制與交接提示。
- `TASKS.md`：僅依退回意見撤回越界狀態變更，使其與本分支基底一致。

## 給下一棒的提示

- 請 Claude／調度器重新審查 PR #5：確認 TASKS.md 不再出現在 PR 差異、歷史 dry-run 已標為無原始輸出、本機手動測試與 CI 結果已區分，並核對最新 head 的 CI。通過後由使用者合併；本報告不代表已獲審查通過。
- `build.ps1` 的 i686／target 與專案 x64／target/current 規範差異為既有問題，另開任務處理。
- 後續獲准執行需資源的模式時，設定 `LITO_CLIENT_ROOT` 指向既有客戶端樹，維持唯讀引用；不要複製資源進 repo。
