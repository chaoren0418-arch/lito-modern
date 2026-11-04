# Issue #3 / T005 — 啟動與建置腳本去硬編路徑

## 做了什麼
- `啟動現代客戶端.ps1`
  - `-ResourceRoot` 未指定時改讀環境變數 `LITO_CLIENT_ROOT`；兩者皆空則預設仍為 `D:\lineage開發\0-客戶端`。
  - 相對路徑以 repo 根（`$PSScriptRoot`）展開；Windows 磁碟機路徑（`D:\...`）維持原樣。
  - `-Verify` / `-OfflineScene` / `-NativeLoginEndpoint` 在資源根不存在時立即丟明確錯誤（提示設定 `LITO_CLIENT_ROOT` 或 `-ResourceRoot`）；不複製 pak/map/sprite。
  - 專案路徑改為相對 repo 根；找不到 `LiTo381-Modern/Cargo.toml` 時丟明確錯誤。
  - `[CmdletBinding(SupportsShouldProcess)]`，`-WhatIf` 只列出路徑與 cargo 命令，不建置、不啟動遊戲。
  - Cargo 參數不變：`build -j 1 --offline --locked --target x86_64-pc-windows-msvc --target-dir target/current --bin LiToModern`。
- `LiTo381-Modern/tools/build.ps1`
  - 以 `$PSScriptRoot` 推 repo 根與 `LiTo381-Modern`；不再假設固定 `D:\...` 工作目錄。
  - 未設時把 `LITO_CLIENT_ROOT` 設成預設值；相對值相對 repo 根展開。
  - 缺少 `Cargo.toml` 或 `build-evidence.ps1` 時丟明確錯誤。客戶端樹不存在只警告（compile 不需要資源，符合「clone 到任意路徑可建置」）。
  - 支援 `-WhatIf`：不建目錄、不加 lock、不呼叫 cargo。
  - Cargo 參數不變：`--offline --locked --manifest-path <project>/Cargo.toml --target-dir <project>/target --target i686-pc-windows-msvc`（無 `-j1`、不是 `target/current`，與改前相同）。

## 怎麼驗證
- 靜態核對兩腳本：`D:\lineage開發\...` 只留在預設值與錯誤訊息，不是唯一可跑路徑。
- 在此 Linux 雲端環境用 PowerShell 7.4.13 跑 dry-run（**已驗證**）：
  - `pwsh -NoProfile -File ./啟動現代客戶端.ps1 -WhatIf -BuildOnly` → 0；印出 repo 相對路徑與 `cargo build -j 1 --offline --locked --target x86_64-pc-windows-msvc --target-dir target/current --bin LiToModern`；未建 `驗證紀錄/` 或 `target/`。
  - 同上加 `-BuildProfile Release` → cargo 列再加 `--release`。
  - `-WhatIf -Verify`（預設資源根不存在）→ 1，`Resource root not found: D:\lineage開發\0-客戶端`。
  - `-WhatIf -Verify -ResourceRoot /no/such/lito-client-root` → 1，明確缺路徑。
  - `LITO_CLIENT_ROOT=/tmp` 或 `-ResourceRoot docs` 時 `-WhatIf -Verify` → 0，相對路徑展開到 repo 根。
  - 把啟動腳本拷到空目錄跑 → 1，`Cargo.toml not found`。
  - `pwsh -NoProfile -File ./LiTo381-Modern/tools/build.ps1 -WhatIf` → 0；缺客戶端樹只警告；cargo 仍為 `test --offline --locked --manifest-path …/Cargo.toml --target-dir …/target --target i686-pc-windows-msvc`。
  - `-WhatIf -Mode Release` → `cargo build … --release`，target 仍是 `target` 不是 `target/current`。
- 未改 Rust 原始碼。PR #5 CI `cargo check (x86_64-pc-windows-msvc)` **已綠**（run `41780882831`）。
- 第 2 次嘗試：`git merge origin/main` 解 `docs/REPORT.md` add/add 衝突，Issue #2 與 Issue #3 條目都保留；腳本未再改。

## 未驗證什麼
- 未在 Windows 原開發機執行真實 `cargo build` / 啟動客戶端 / `-Verify` 對到實體 `D:\lineage開發\0-客戶端`。
- 未跑 `cargo test`（專案規則：恢復測試需使用者明確同意）。
- 未連正式服務或 DB；未啟動、附加既有遊戲程序。
- 測試案與 `維護工具/` 內其餘硬編路徑未改。

## 產出路徑
- `啟動現代客戶端.ps1`
- `LiTo381-Modern/tools/build.ps1`
- `docs/reports/issue-3.md`
- `docs/REPORT.md`
- `TASKS.md`（T005 → review）

## 給下一棒的提示
- Claude 已審查通過（LITO_CLIENT_ROOT、`-WhatIf`、缺路徑錯誤、Cargo 參數未變）；`build.ps1` 的 i686 vs CLAUDE.md x86_64 不一致另開任務，非本 PR。
- 第 2 次嘗試只合併 `origin/main` 並保留 REPORT 兩邊條目。可合併 PR #5。
- 合併後若要在新 clone 跑需要資源的模式，先設 `LITO_CLIENT_ROOT` 指向既有客戶端樹（唯讀引用，不要複製進本 repo）。
- 後續若要把測試裡的 `D:\lineage開發\0-客戶端` 也改成同一環境變數，另開任務，不要塞進本 PR。
