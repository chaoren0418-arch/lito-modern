# Issue #303 — T031（M3 1／2）：`packaging/**` clean host 可攜交付（打包）

階段：code，執行者：claude-local，分支：`task/303`。
本張只做「打包」；乾淨主機實測是同一任務的第 2 張。

## 結論

可攜包的打包流程已改完，並把原本已經壞掉的三處一併修好（見「做了什麼」第 6 點）。本機只能編譯與靜態檢查，**沒有執行任何編譯出來的程式**；`package.ps1` 整條流程（會執行 `LiToModern.exe --version`）交給 GitHub Actions 的 `portable` workflow 驗證，結果見文末「CI 結果」。

有一件事需要使用者決定（見「需要使用者決定」）：本機只裝了 Visual Studio 2026 **Insiders（預覽）**的 MSVC 工具組，本機建出的 EXE 雖然不再需要任何 VC runtime DLL，但靜態連結進去的 C 執行階段程式碼來自預覽工具組。

## 做了什麼

### 1. 正式版 VC runtime → 改成靜態連結 C 執行階段

- `tools/build.ps1` 的 `Release`／`TestRelease` 與 `啟動現代客戶端.ps1 -BuildProfile Release` 都加上 `RUSTFLAGS=-C target-feature=+crt-static`（結束後還原環境變數）。三個入口用同一個旗標，因為它們共用 `target/current/x86_64-pc-windows-msvc/release`，旗標不同會整包重編。
- 結果：`LiToModern.exe` 不再匯入 `VCRUNTIME140.dll` 或任何 `api-ms-win-crt-*`，不需要安裝 Visual C++ 可轉散發套件，也不需要在 EXE 旁放 DLL。舊包「依賴 VS18 預覽 runtime DLL」的問題因此消失。
- `tools/package.ps1` 打包前會讀 EXE 的匯入表，只要出現 VC／UCRT runtime DLL 就拒絕打包。

### 2. 可雙擊的啟動入口

- 新增 `packaging/Start-LiToModern.cmd`：只用 `cmd.exe`，從自己所在資料夾啟動 `LiToModern.exe`；EXE 不在旁邊時（例如在 ZIP 預覽裡直接執行）顯示提示並以結束碼 1 結束。內容是純 ASCII、CRLF，不受系統語系影響。
- 直接雙擊 `LiToModern.exe` 效果相同（本來就是預設入口：首次啟動出現連線設定）。兩者都不需要 PowerShell 7。

### 3. 版本清單（G10：來源／EXE／資源）

`tools/package.ps1 -Candidate <名稱> -ResourceRoot <資源根> [-EffectCatalog <effectlist2.xml>] [-Zip]` 產出 `dist/<名稱>/`：

| 檔案 | 內容 |
|---|---|
| `LiToModern.exe`、`Start-LiToModern.cmd`、`README.md` | 程式、入口、使用說明 |
| `manifest/source.json` | git commit、建置輸入摘要（`BuildInputSHA256`）與逐檔 SHA-256 |
| `manifest/executable.json` | EXE SHA-256、`--version` 內容、PE 資訊（匯入 DLL、連結器版本）、`rustc -Vv`、最低 Windows 版本 |
| `manifest/resources.json` | 外部資源根目錄逐檔長度與 SHA-256、清單摘要（資源本身不複製） |
| `SHA256.json` | 以上所有檔案的長度與 SHA-256（schema 3） |

- `-ResourceRoot` 必填（或用 `LITO_CLIENT_ROOT`）；沒有資源清單就不出包。
- 有未提交的追蹤檔變更時拒絕打包（`-AllowUncommitted` 可改成只記錄）。
- `-Zip` 另外產生 `dist/<名稱>.zip`，ZIP 內只有一個頂層資料夾。
- 包內只允許上表 7 個檔案；不含 DLL、`LiTo.bin`、pak 等資源。`packaging/profile.example.json`（歷史 Modern 協定範例，內含開發機路徑）不再放進包裡，檔案本身留在 repo（單元測試在用）。

### 4. 移除開發診斷 hardcode

`docs/HISTORY.md` L1569 只寫「5 處開發診斷 hardcode」，沒有列出位置。這次把正式 EXE 會編進去的全部找出來處理，共兩類 24 處：

**(a) 寫死資源根目錄 `D:\lineage開發\0-客戶端`：17 處／10 檔**

| 檔案 | 處數 |
|---|---|
| `native_login.rs`（`probe_lobby`） | 4 |
| `native_login_window.rs`（`run(peer)`，沒有任何呼叫者，已刪除） | 1 |
| `native_peer_hotel_probe.rs` | 3 |
| `native_peer_hotel_share_probe.rs`、`native_peer_rental_probe.rs` | 各 2 |
| `native_combat_probe.rs`、`native_peer_probe.rs`、`native_peer_citizen_probe.rs`、`native_peer_merchants_probe.rs`、`native_recovery_probe.rs` | 各 1 |

改成 `native_startup::diagnostic_resource_root()`：先讀環境變數 `LITO_CLIENT_ROOT`（必須是本機絕對路徑），沒有就讀 EXE 旁的 `LiToModern.native.json`，都沒有就回報錯誤。

**(b) 編譯時把專案絕對路徑（`env!("CARGO_MANIFEST_DIR")`）寫進 EXE：7 處／7 檔**

`native_login.rs`、`native_combat_probe.rs`、`native_peer_probe.rs`、`native_recovery_probe.rs`、`native_actor_state_probe.rs`、`native_emblem_probe.rs`、`native_label_update_probe.rs`。這一類是靜態掃描第一次建出的 EXE 才發現的（EXE 裡有建置機的專案路徑）。改成 `native_startup::diagnostic_project_root()`：執行時從 EXE 位置往上找 `<專案>/target`（專案要有 `Cargo.toml`）；可攜包裡找不到，診斷會拒絕執行。

這些都只在 `--probe-native-*` 開發診斷裡用到；一般登入流程本來就由設定畫面傳入資源根目錄，沒有改動。測試模組（`#[cfg(test)]`）裡的 `D:\...` 範例字串不會編進 EXE，沒有動。

`tools/package.ps1` 打包前會掃 EXE：出現 `lineage開發`、`0-客戶端`、`客戶端xml`、`lito-dev-runtime` 或建置機的專案路徑（UTF-8／UTF-16）就拒絕打包。

### 5. 最低 Windows 版本

- **Windows 10 版本 1703（組建 15063）x64 以上，或 Windows 11。**依據：Rust 1.95 的 `x86_64-pc-windows-msvc` 標準庫需要 Windows 10；程式靜態匯入 `GetDpiForWindow`（1607）並使用 per-monitor DPI awareness v2（1703）。
- 寫在 `packaging/README.md`、`manifest/executable.json`、`SHA256.json`；`windows/app.manifest` 加上 Windows 10／11 的 `supportedOS` 宣告。

### 6. 順手修好：打包流程在 main 上原本跑不起來

改之前 `tools/package.ps1` 一定失敗，原因有三個，都已修正：

- `Assert-ModernVersion` 要求剛好 5 個擴充，但 `--version` 現在回報 7 個（多了 `full_vip_v1`、`full_vip_v2`）。
- `verify-independent.ps1` 要求只有一個 `[[bin]]`（現在有 `LiToModern-485F-Candidate` 第二個）、平台還寫 `i686`、target 目錄只認 `target`。
- `build-evidence.ps1`／`verify-independent.ps1` 要求 `LiTo381-Modern/.cargo/` 存在，但它不在 git 裡（`.gitignore`），新 clone 與 worktree 都沒有。現在沒有就略過。

`tools/test-build-evidence.ps1` 的 fixture 也還是 i686／5 個擴充（#60 報告已記錄），一併更新並加一案「沒有 `.cargo` 的新 clone」。

### 7. 新增的檢查工具與 CI

- `tools/portable-evidence.ps1`：PE 匯入表解析、開發路徑掃描、資源清單、ZIP。只讀檔，相容 Windows PowerShell 5.1。
- `tools/verify-portable-package.ps1 -Zip … -ExtractRoot … -ResourceRoot …`：解壓 ZIP、逐檔比對 `SHA256.json`、靜態檢查 EXE，再只執行包內 EXE 的 `--version`、`--check-native-config`（兩者都不開視窗、不連線、不寫設定）與 `.cmd` 入口。相容 PowerShell 5.1，乾淨主機不必先裝 PowerShell 7。**這支會執行 EXE，本機執行器不可跑**，給 CI 與第 2 張用。
- `.github/workflows/portable.yml`：只在打包相關檔案變更或手動觸發時跑（Release 建置慢，不想每個 PR 都跑）。步驟：fixture 測試 → `build.ps1 -Mode Release` → `package.ps1 -Zip`（用合成的假資源目錄）→ 在含中文與空白的路徑分別用 PowerShell 7 與 5.1 跑 `verify-portable-package.ps1` → 上傳 ZIP artifact（保留 14 天）。CI 用的是 windows-latest 的**正式版** MSVC 工具組。
- `docs/TESTING.md` 補上 `LITO_CLIENT_ROOT` 對診斷的新要求。

## 怎麼驗證

### 已驗證

- 本機 `cargo build --release --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --bin LiToModern`（`RUSTFLAGS=-C target-feature=+crt-static`）成功；`cargo check --locked --offline -j1 --all-targets` 成功。
- 靜態讀本機建出的 Release EXE（10,539,008 bytes，SHA-256 `42EBD551…BBEBF9`，未執行）：
  - PE32+ x64、GUI 子系統、連結器 14.51。
  - 匯入表只有 13 個系統 DLL：`ADVAPI32`、`api-ms-win-core-synch-l1-2-0`、`bcrypt`、`bcryptprimitives`、`comctl32`、`gdi32`、`imm32`、`kernel32`、`ntdll`、`oleaut32`、`user32`、`winmm`、`ws2_32`；沒有 delay-load；沒有 `VCRUNTIME*`、`MSVCP*`、`api-ms-win-crt-*`。
  - EXE 內不含 `D:\lineage`、專案路徑、`CARGO_MANIFEST`；含 `supportedOS` GUID。
  - 修正前的第一次建置，同一個檢查確實抓到 EXE 內的 `D:\lineage開發\…\LiTo381-Modern`（(b) 類 hardcode），修正後歸零。
- `portable-evidence.ps1` 的函式在 Windows PowerShell 5.1 用 fixture 跑過：PE 解析（`notepad.exe`，因匯入 `api-ms-win-crt-*` 被正確拒絕）、UTF-16 開發路徑偵測、含中文檔名的資源清單、`.cmd` 轉存、ZIP。
- 8 支 PowerShell 腳本語法解析 0 錯誤。

### 推測

- HISTORY L1569 的「5 處」最可能是 `native_login.rs` 4 處加 `native_login_window.rs` 1 處；原審核紀錄沒有列位置，無法確認。這次處理的是目前原始碼裡的全部 24 處。
- 動態載入：EXE 字串裡另有 `dbghelp.dll`（Rust 標準庫印 backtrace 時才載入）與 `mscoree.dll`（靜態 CRT 結束程序時只探測、不載入）。兩者都是系統元件或可缺，推測不影響乾淨主機；要第 2 張實測確認。
- 最低版本 1703 是從匯入的 API 推出來的，沒有在 1607／1703 實機上跑過。

### 待驗證（本機不可做，或需要實機）

- `package.ps1`、`build.ps1 -Mode Release`、`test-build-evidence.ps1`、`verify-portable-package.ps1` 的實際執行：本機沒有 PowerShell 7，而且流程會執行 EXE。交給 CI，見「CI 結果」。
- 乾淨主機（沒裝任何 VC runtime）解壓 ZIP、雙擊 `Start-LiToModern.cmd` 與 `LiToModern.exe`、首次設定畫面、設定存檔、中文路徑、LoadLibrary 動態載入閉包：第 2 張。
- `supportedOS` 宣告與靜態 CRT 對視窗行為沒有影響：沒有開過 GUI。
- `維護工具/verify-native-*.ps1`（16 支）與 `tools/run-item-sessions.ps1` 現在需要呼叫端先設 `LITO_CLIENT_ROOT`；沒有實跑。

## 需要使用者決定

**本機正式包要用哪個工具組建置？** 本機只有 Visual Studio 2026 Insiders（18.9，MSVC 14.51.36231，預覽版）。靜態連結後 EXE 不再依賴任何 runtime DLL，但連進去的 C 執行階段程式碼來自這個預覽工具組。選項：

1. 接受：本機建置、本機打包（有真實資源清單）。最省事。
2. 本機另裝正式版 Build Tools 再建置（要改系統，需使用者自己裝）。
3. EXE 取自 CI（正式版工具組），但 CI 沒有遊戲資源，資源清單是假的，不能直接當正式包。

我的建議是 1，等第 2 張乾淨主機實測通過就夠；如果要求交付物完全不沾預覽工具組，就選 2。

## 產出路徑

- `LiTo381-Modern/tools/package.ps1`、`portable-evidence.ps1`（新）、`verify-portable-package.ps1`（新）、`build.ps1`、`build-evidence.ps1`、`verify-independent.ps1`、`test-build-evidence.ps1`
- `LiTo381-Modern/packaging/README.md`、`Start-LiToModern.cmd`（新）
- `LiTo381-Modern/windows/app.manifest`
- `LiTo381-Modern/src/modern_lito/native_startup.rs` 與 13 個 `native_*` 檔
- `啟動現代客戶端.ps1`、`.github/workflows/portable.yml`（新）、`docs/TESTING.md`
- 本報告 `docs/reports/issue-303.md`

## 給下一棒的提示（第 2 張：乾淨主機驗證）

1. 產生正式包（需要 PowerShell 7；會執行 EXE，所以由 Codex／使用者做）：
   ```powershell
   pwsh .\LiTo381-Modern\tools\build.ps1 -Mode Release
   pwsh .\LiTo381-Modern\tools\package.ps1 -Candidate t031-<日期> -ResourceRoot 'D:\lineage開發\0-客戶端' -Zip
   ```
   資源根目錄約 10.8 GB／1.3 萬檔，雜湊要幾分鐘。`.cargo/config.toml` 是本機未追蹤檔，會算進建置輸入摘要。
2. 無視窗檢查（PowerShell 5.1 即可，把 `verify-portable-package.ps1` 與 `portable-evidence.ps1` 一起帶到乾淨主機）：
   ```powershell
   powershell -File verify-portable-package.ps1 -Zip <zip> -ExtractRoot 'D:\中文 路徑\解壓' -ResourceRoot <資源根>
   ```
3. 還要人工做的：雙擊 `.cmd` 與 EXE、首次設定畫面、勾選記住後 `LiToModern.native.json` 出現在 EXE 旁、重開不再問、放在唯讀位置時的錯誤訊息、沒裝 VC runtime 的主機能啟動。
4. 第一次用 `build.ps1 -Mode Release` 或 `啟動現代客戶端.ps1 -BuildProfile Release` 會因為新旗標整包重編一次（本機約 6–7 分鐘）。直接下 `cargo build --release`／`cargo test --release` 而不帶同一個 `RUSTFLAGS`，會和這兩支腳本互相觸發重編。
5. EXE 內還有相依套件的 panic 位置字串（`D:\dev-cache\cargo\registry\…`，約 180 處）。它們只是訊息文字，不是執行時會用的路徑；要清掉可以在 Release 旗標加 `--remap-path-prefix`，這次沒做。
