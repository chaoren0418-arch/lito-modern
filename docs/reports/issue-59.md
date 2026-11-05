# Issue #59 — T010-F 本機建置腳本驗證

## 結論與範圍

**已驗證**：六次 `-WhatIf` 與兩次 `-BuildOnly` 均成功，結束碼全為 0。worktree／clone 實際建置各耗時 316.233／341.031 秒；兩份 exe 均為 AMD64 且位於各自 repo 的 `target/current`。T005 在本次 clone 路徑的實際建置已驗證。驗證限於本次 Windows 主機、固定來源與專案內兩個工作目錄；不推論所有機器／所有路徑都能建置。

待驗證：`LiTo381-Modern/tools/build.ps1` 實際執行、原始共享工作目錄及其本機設定、Release 腳本建置、資源診斷執行、單元／整合／ignored／GUI／G06。本次沒有執行測試案例，passed／failed／ignored 皆為 N/A，預覽不算測試案例通過。推測：無額外功能完成推論。

## 固定版本與執行環境

- 日期：2026-11-05（UTC）。本任務開始時遠端 main 為 `bb96171d2938437dd224af73da466ab8bbab8f73`，兩個工作目錄皆固定在此版本。
- 原共享工作目錄 main 是 `92bbda01c30488cea855f84c6ba7d1e5b23c7c98`，未切換或覆寫。
- 分支 `task/59`；專屬 worktree：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-59`（下稱 W）。第一組命令在 W 根目錄執行，並非原共享 repo 根目錄；此範圍差異保留給審查。
- clone：`W\scratch\clone-test`（下稱 C），與 W 是不同 repo 工作目錄。執行 `git clone --no-hardlinks --no-checkout . scratch/clone-test`，再 `git -C scratch/clone-test checkout --detach bb96171d2938437dd224af73da466ab8bbab8f73`；兩命令結束碼 0。來源是本機完整物件庫，沒有網路 clone，未使用硬連結或沿用其他任務的 target。
- 開始前 #59 為 OPEN、`runner:dot`／`task:running`，僅派工留言，無 RESULT、task/59 遠端分支／PR、本機同名 worktree／分支。前置 #53 已 CLOSED／task:done／gate:pass／review:pass，PR #67 已合併。
- 本機 Codex 對話直接使用 PowerShell／Git／gh CLI。工作階段是 workspace-write／auto_review，無法設定或證實完全存取。一般權限 gh 初次遇到 socket 10013，經工具必要核准後 GitHub CLI 可用，沒有繞過核准；建置在現有允許範圍執行，未出現 setup refresh had errors。
- PowerShell 7.6.5、cargo 1.95.0、rustc 1.95.0；CARGO_HOME=`D:\dev-cache\cargo`、RUSTUP_HOME=`D:\dev-cache\rustup`。

## 執行前安全檢查

已讀固定版本 AGENTS.md、PLAN.md、TASKS.md 的 T005／T010、兩支腳本、build-evidence.ps1、build.rs 與 Cargo.toml。依主控限制未讀寫 docs/REPORT.md。僅新增本報告，沒有修改 src/tests 或其他追蹤文件。

入口腳本的 `-WhatIf` 分支在 Cargo 與程式啟動前返回；`-BuildOnly` 在 Cargo 成功後輸出 exe 路徑並返回。build.rs 只編譯 Windows manifest；Cargo 使用 offline/locked、單工作與 x64 target。未執行 Verify、NativeLoginEndpoint、OfflineScene、ProbeNativeEndpoint 或產物的 `--version`，沒有開 GUI、連正式 DB／服務、操作 BIN／機密設定／封存／既有遊戲程序，也未改 ACL／安全設定。

已檢查 W 與祖先的 `.cargo/config`／`config.toml`、CARGO_HOME 設定，未發現設定檔；未把原共享 `LiTo381-Modern/.cargo` 設定複製過來。一般步驟沒有 LITO_CLIENT_ROOT 設定，僅 clone 相對路徑預覽暫時設定後還原。

## 命令與關鍵輸出

先做四項預覽，再執行 W 建置；W 建置期間準備 clone 與其預覽，兩次實際 Cargo 建置依序執行。步驟編號對應驗收排列，不代表起始時間排序。每項只執行一次，沒有重試。

入口腳本四次預覽及兩次建置使用的 Cargo 命令：

```text
cargo build -j 1 --offline --locked --target x86_64-pc-windows-msvc --target-dir target/current --bin LiToModern
```

`-j 1` 與規定的 `-j1` 等效。tools/build.ps1 的兩項預覽輸出如下，W 代表前述完整絕對路徑；原始完整輸出另存日誌。

```text
cargo test --locked --offline -j1 --manifest-path W\LiTo381-Modern\Cargo.toml --target-dir W\LiTo381-Modern\target\current --target x86_64-pc-windows-msvc
cargo build --locked --offline -j1 --manifest-path W\LiTo381-Modern\Cargo.toml --target-dir W\LiTo381-Modern\target\current --target x86_64-pc-windows-msvc
```

固定來源已包含 T010-G（提交 `c8a6462`）。Issue 原先提及的 i686／target/ 是舊版狀態；本次已驗證預覽為 x64／target/current／-j1。**tools/build.ps1 實際建置仍待驗證（Issue 原排程：等 T010-G；現已含修正，仍需另行安排實測）**，本次沒有擴大範圍實際執行它。

預覽關鍵欄位：W 的 RepoRoot／Project／Exe 均以 W 為根，C 的對應欄位以 C 為根。步驟 06 清除 LITO_CLIENT_ROOT，ResourceRoot 是既定絕對路徑 `D:\lineage開發\0-客戶端`，不是 clone 下的路徑。步驟 07 設為 `relative-client-assets`，解析結果是 `C\relative-client-assets`；這個不存在的路徑僅用於 BuildOnly 預覽，沒有建立或讀取資源。所有預覽 WhatIf=True，入口腳本 BuildProfile=Debug。步驟 08 還原為未設定環境變數。

## 證據保存與給下一棒的提示

原始輸出與一次性腳本放在 `W\scratch\issue-59\`，不進 git。summary.csv 包含逐步指令、cwd、UTC 起訖、結束碼與耗時；artifacts.csv 包含來源 SHA、產物路徑、大小、SHA256、PE Machine。只讀取 exe 檔案內容／PE header，未執行。原腳本寫出的 W/驗證紀錄/current/build.log 保留，並複製到 scratch。clone 清理前先保留其 Cargo 日誌、指令輸出、產物雜湊；清理結果另見後附紀錄。

交調度器／Claude 審查單檔報告與原始證據，特別確認隔離 worktree 取代共享根目錄的差異。不要把本結果當成工具腳本實測、GUI 或整體 M1 通過。後續 tools/build.ps1 實測應另行指派並檢查 receipt/version 執行路徑，遵守不啟動遊戲的界線。

## 已驗證結果明細

六次預覽與兩次 BuildOnly 均結束碼 0；兩份產物均為 AMD64，且各自在其 repo 的 target/current 之下。T005 的本次 clone 異路徑實際建置已驗證。兩次實際建置各有 296 項編譯警告（含 10 duplicates），沒有編譯失敗；這些警告不是測試結果。

| 步驟／日誌 | cwd | 完整指令 | 結束碼 | 秒 |
|---|---|---|---:|---:|
| `01-launch-build-whatif` | W | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -WhatIf -BuildOnly` | 0 | 0.657 |
| `02-launch-verify-whatif` | W | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -WhatIf -Verify` | 0 | 0.613 |
| `03-launch-build` | W | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -BuildOnly` | 0 | 316.233 |
| `04-tools-test-whatif` | W | `pwsh -NoProfile -File .\LiTo381-Modern\tools\build.ps1 -WhatIf -Mode Test` | 0 | 0.628 |
| `05-tools-debug-whatif` | W | `pwsh -NoProfile -File .\LiTo381-Modern\tools\build.ps1 -WhatIf -Mode Debug` | 0 | 0.620 |
| `06-clone-default-whatif` | C | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -WhatIf -BuildOnly` | 0 | 0.662 |
| `07-clone-relative-whatif` | C | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -WhatIf -BuildOnly` | 0 | 0.699 |
| `08-clone-build` | C | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -BuildOnly` | 0 | 341.031 |

### UTC 起訖

| 步驟 | 開始 | 結束 |
|---|---|---|
| 01-launch-build-whatif | 2026-11-05T04:48:30.6174977Z | 2026-11-05T04:48:31.2792497Z |
| 02-launch-verify-whatif | 2026-11-05T04:48:31.3117178Z | 2026-11-05T04:48:31.9250837Z |
| 03-launch-build | 2026-11-05T04:48:39.9997432Z | 2026-11-05T04:53:56.2370569Z |
| 04-tools-test-whatif | 2026-11-05T04:48:31.9321768Z | 2026-11-05T04:48:32.5600947Z |
| 05-tools-debug-whatif | 2026-11-05T04:48:32.5718079Z | 2026-11-05T04:48:33.1915554Z |
| 06-clone-default-whatif | 2026-11-05T04:49:26.1371598Z | 2026-11-05T04:49:26.8043236Z |
| 07-clone-relative-whatif | 2026-11-05T04:49:26.8353823Z | 2026-11-05T04:49:27.5346215Z |
| 08-clone-build | 2026-11-05T04:54:09.9428405Z | 2026-11-05T04:59:50.9807937Z |

### 實際產物

- worktree：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-59\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe`；33145344 bytes；PE `0x8664`；SHA256 `6D5C55206D99A5246D06D2105F87CFBDEC277052A02B991DC41E23BC3713E00F`。
- clone：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-59\scratch\clone-test\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe`；33145344 bytes；PE `0x8664`；SHA256 `1C9B97D51B3A3572F2A725FE25BF0CC8609201DB4FE29B307114C33F827155D3`。

clone 的 exe 已隨本次 clone 清除，未另存副本。雜湊記錄是清理前的唯讀取證。兩份 exe 的 SHA256 不同，本次未驗證位元組可重現建置，不宣稱兩檔相同。

### worktree Cargo 輸出末尾

```text
   |                                     ^^^^^^^^ help: convert the identifier to snake case: `tls_port`

warning: `lito-modern` (lib) generated 296 warnings (10 duplicates) (run `cargo fix --lib -p lito-modern` to apply 20 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 15s
```
### clone Cargo 輸出末尾

```text
   |                                     ^^^^^^^^ help: convert the identifier to snake case: `tls_port`

warning: `lito-modern` (lib) generated 296 warnings (10 duplicates) (run `cargo fix --lib -p lito-modern` to apply 20 suggestions)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5m 39s
```

### 清理收據

```text
UTC=2026-11-05T05:00:06.3212609Z
Target=D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-59\scratch\clone-test
Validation=Exact expected path; within task worktree scratch; no reparse points; fixed HEAD; clean tracked state; evidence preserved
Result=Removed only the clone created for issue #59; Test-Path=False
```

### 原始證據 SHA256

以下檔案位於 W/scratch/issue-59，未加入 git。

| 檔案 | SHA256 |
|---|---|
| `01-launch-build-whatif.log` | `E249E91885A3F62E9D2FDD41A39EB62C838BD9B443F5963B2B183CDCBDBCFE01` |
| `02-launch-verify-whatif.log` | `E249E91885A3F62E9D2FDD41A39EB62C838BD9B443F5963B2B183CDCBDBCFE01` |
| `03-launch-build.log` | `24B20E3BDD50A0D4FF0E47D6BA3105CFE2EEF5E62BE5D4B8D9A330843F4152D7` |
| `04-tools-test-whatif.log` | `FE64124FEB05F849F949CCA6526B405253E14BCDE4363964809FA6D8ED72E392` |
| `05-tools-debug-whatif.log` | `021F526FAC3689B6D548D9772574EA404EEF842B58885CE6E5D6BF85739A64EC` |
| `06-clone-default-whatif.log` | `6FEEE72FFBB21C912856D97F2CD454808BCBA0CC0C1AAED65031B5E217830A55` |
| `07-clone-relative-whatif.log` | `BA37FED5B82B545DF2FB31FA0FA6D55484F9C6D40B1390CF272E295E55C3B594` |
| `08-clone-build.log` | `50DE19BBF107A3D6F9FCF1D1D627B07B22801962CAE3804061AC0C4F33B3AED9` |
| `append-evidence.ps1` | `E390E343AC8B0531368CB53356AD479FEF7F5FAC036EE384D32B76FBCB7808D5` |
| `artifacts.csv` | `20A717C800AAB3378770553D90661BBE56D39FA2B24007D88F366CB3406C164E` |
| `cleanup-clone.ps1` | `AC5ECE5EF344F2AA1F5741F8DBC78CF3CA658BFAC528F89FC98D7F539E4AE1A0` |
| `clone-cargo-build.log` | `73372A256E1015447253BFC83EF522E4CAA1B4DBA62FEED9BD0A4501305BD7DD` |
| `clone-checkout.log` | `7C0060A3C305C097341EA78DCCED365FBE4887F1C2B6B2BDBAE952C51703ECA9` |
| `clone-cleanup.log` | `E5947AF12ADE032582AE4BE98ECCB27588073C4AF552263D01E62FEDAB7ED6CF` |
| `clone-create.log` | `04CAC18731B65ED7CFA2AA1E3C6A2A09D90B2CA7614DDC15C49685DB2F73454B` |
| `inspect-artifact.ps1` | `3B1C62DE537FCA77DD7B71402088A6A90752E08D4F121AA7EF6D6912C841FE34` |
| `run.ps1` | `9FE9E90A3DEAE058D60A6FADB3BCA06A6AD731AC97EA0C9B6600C6C89690767A` |
| `summary.csv` | `AA2582B6BEB38115BF9C488BC7CA8E2A1A1343D78E4B39361470AD9FCF0507C8` |
| `worktree-cargo-build.log` | `6C5500AD6A7062084AE8B3E0F2A9D43EDD49F6EC8D2F99EFABFBF776C4EC75C6` |
