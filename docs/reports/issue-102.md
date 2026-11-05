# #102 / T013：Hidden 啟動可見性與不連線畫面

## 結論

**#102 / T013 指定驗收通過，交主控審查；不代表整個 V1 或完整 GUI 驗收完成。** 本輪在 2026-11-06（主機 UTC+08:00；UTC 日期 2026-11-05）完成指定原始碼修正、cargo check、PowerShell 7 建置及三個不連線畫面驗證。三張真實 PNG 全部已實際檢視，三個視窗在原腳本 Hidden 啟動後均可見，自己的程序全部清理。

## 做了什麼

- 來源基準：`b2d5b84c6f86936dd00fa7914bed960add8402d0`（當版 `origin/main`）。
- 分支：`task/102`；隔離工作目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-102`。
- 開始前確認 #102 為 open、`runner:dot`、`task:running`，最新留言為派工且無 RESULT；本機 worktree、分支與遠端均無 task/102 產物。既有對話未見另一個 #102 執行工作。
- 已讀當版 `AGENTS.md`、`CLAUDE.md`、`docs/REPORT.md`（唯讀）；專案與所查上層 `.agents/skills` 未發現適用技能。根工作區未切換、未更新。
- 唯一原始碼變更為 `LiTo381-Modern/src/modern_lito/scene_preview.rs`：明列 `IsWindowVisible` import，第一次 `ShowWindow` 後若仍隱藏則再次顯示。未更動其他原始碼或正式腳本。
- 新增 DEMO、三張實際畫面 PNG 及本報告；驗證用一次性腳本與日誌只留在工作目錄的 `scratch/`，不提交。

## 已驗證：環境與編譯

- 主機：`DESKTOP-CV2L01A`，本機 Codex 工作階段；未使用 CUA。
- PowerShell：`7.6.5`；`(Get-Command pwsh).Source` 為 `C:\Users\CHAOREN\.cache\codex-runtimes\codex-primary-runtime\dependencies\native\powershell\pwsh.exe`。
- 在 `LiTo381-Modern` 執行：

  ```powershell
  cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
  ```

  結束碼 **0**，耗時 4 分 14 秒；有編譯警告，未以 cargo fix 擴大修改範圍。完整本機日誌為 `scratch/cargo-check.log`。這是編譯檢查，沒有執行測試案例。

## 建置、視窗與清理

在上述隔離 worktree 根目錄執行：

```powershell
pwsh -NoProfile -File .\啟動現代客戶端.ps1 -BuildOnly
```

建置結束碼 **0**，Cargo 顯示耗時 3 分 46 秒，有編譯警告。EXE 完整路徑：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-102\LiTo381-Modern\target\current\x86_64-pc-windows-msvc\debug\LiToModern.exe
```

SHA256：`783E3112F1E0075F8EC27E7362CC70589F9CCA6A534808CE02693B5CFAEF2C80`。每次啟動前後及最終檢查皆吻合。建置日誌為 `scratch/build-only.log` 與 `驗證紀錄/current/build.log`（後者會被後續腳本建置更新）。

每次執行前均記錄 `Get-Process LiToModern`，三次既有清單皆為空。啟動後依新 PID 及上述完整 EXE 路徑認領程序，透過 `EnumWindows`／`GetWindowThreadProcessId` 找到唯一可見視窗。**沒有另外呼叫 ShowWindow、強制顯示或修改啟動腳本**；三次都是既有腳本的 `Start-Process -WindowStyle Hidden`。以原生 `IsWindowVisible` 記錄結果，`PrintWindow` 擷取該程序的視窗範圍，沒有桌面全螢幕截圖。

| 畫面 | 在 repo 根目錄執行的指令 | PID | HWND | IsWindowVisible | 啟動腳本結束碼 | 清理 |
|---|---|---:|---:|---|---:|---|
| 設定 | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -ConfigureNative` | 14292 | 16255776 | True | 0 | WM_CLOSE，10 秒內退出，PID 無殘留 |
| 登入 | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -NativeLoginEndpoint 127.0.0.1:62812` | 24260 | 21104276 | True | 0 | WM_CLOSE，10 秒內退出，PID 無殘留 |
| 離線場景 | `pwsh -NoProfile -File .\啟動現代客戶端.ps1 -OfflineScene` | 5344 | 9702396 | True | 0 | WM_CLOSE，10 秒內退出，PID 無殘留 |

登入用 `TcpListener(Loopback, 0)` 取得 62812 後立刻 `Stop()`，以 OS listener 清單確認該埠無服務，再啟動登入視窗。未輸入帳密、未按連線；完整可重現的空埠取得指令見 DEMO，後續執行不能硬用本輪埠號。設定、登入各等待 3 秒；離線場景等待 25 秒。三次擷取及清理腳本皆結束碼 0。`Get-Process` 取得的物件在退出後未回傳可用的 EXE `ExitCode`，因此不宣稱 EXE 結束碼 0；已驗證的是 `WaitForExit(10000)=True` 與各 PID 不存在。

最終再次檢查：14292、24260、5344 均不存在；`Get-Process LiToModern` 為空；62812 無 listener；EXE 旁沒有 `LiToModern.native.json`，沒有儲存設定。未使用強制終止，未關閉任何既有程序。

本機逐步日誌：`scratch/configure.log`、`scratch/login.log`、`scratch/offline-scene.log`；一次性驗證腳本：`scratch/capture-window.ps1`。

## 已驗證：真實 PNG 與檢視

三張皆為第一次 `PrintWindow` 擷取成功，無需替代方法、縮圖或影像編修。已逐張載入檢視，確認不是空白、錯誤紀錄或其他視窗；沒有帳密、玩家資料或不相關桌面內容。

| 路徑 | 尺寸 | 檔案大小（bytes） | 實際觀察 |
|---|---:|---:|---|
| `docs/screenshots/v1-configure.png` | 1280×680 | 29,967 | 端點與資源根欄位空白、記住設定核取方塊、開啟登入與取消按鈕；文字清楚 |
| `docs/screenshots/v1-login.png` | 1200×1640 | 56,243 | 127.0.0.1:62812、空白帳密、尚未連線、後續功能停用；完整登入視窗 |
| `docs/screenshots/v1-offline-scene.png` | 1040×820 | 766,143 | 岸邊水面、地圖植被建物與中央角色；無載入中或錯誤畫面 |

均低於 2 MB。離線視窗當時標題：`共享場景驗證（非連線遊戲） | Sprite/gfx=3225 heading=0 | Idle frame=2 cycle=17 | sound=0 hit-cue=0 expired=0 | audio=0/0`。未觸發操作或音效；不據此聲稱音訊驗收通過。

DEMO 用 `screenshots/…` 相對路徑嵌入三張 PNG，每張均附實際重現指令、預期與觀察。

## 未驗證與範圍

- 本次未重跑 `cargo test`。#53 的 Debug／Release 各 lib 1889 passed、0 failed、575 ignored 僅為既有基準；ignored 不算通過。
- T012 / #101 尚未合併，當版 main 不存在 `維護工具/run-local-test-suite.ps1`；DEMO 標示「待 T012 合併」，不建立死連結。
- 未驗證登入連線、帳密、正式服務相容性、移動、戰鬥、音效操作或完整 GUI / V1。
- 不讀機密 `config/server.properties`，不修改原 BIN、正式客戶端、封存、硬連結或 `docs/REPORT.md`；資源僅由自己的 exe 唯讀使用，不複製進 repo。
- 未連任何 DB 或正式服務，未啟停 server；未觸碰 task/83 的 3307 或其資料。

## 產物與下一棒

- `LiTo381-Modern/src/modern_lito/scene_preview.rs`
- `docs/DEMO.md`
- `docs/screenshots/v1-configure.png`
- `docs/screenshots/v1-login.png`
- `docs/screenshots/v1-offline-scene.png`
- `docs/reports/issue-102.md`

請主控審查 `task/102` 的單檔修正及三張畫面；T012 合併後由後合併的任務補上 DEMO 測試腳本連結。整個 V1 的裁定仍需其他任務條件，不由本輪代為宣告。本輪不合併 PR、不關閉 Issue、不改權限。
