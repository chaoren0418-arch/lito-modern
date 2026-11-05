# Issue #101：T012 本機測試套件（第 2 次嘗試，前置阻擋）

## 結論

**blocked，未完成、未通過驗收。** 2026-11-06 第 2 次嘗試重新執行本機前置檢查，仍找不到 `pwsh`，無法確認可用的 PowerShell 7.0 以上版本。依本任務步驟 0「找不到 pwsh 或版本低於 7，就標 blocked 並回報，不改用 Windows PowerShell 5.1」，停止後續實作與測試；沒有以 Windows PowerShell 5.1 代跑。

## 做了什麼／怎麼驗證

- 工作目錄：`D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-101`；沿用 `task/101`，未 checkout 或建立分支。
- 本次起始提交：`8fc1592ad572e256ce1405270839d2bb0e7f4d63`（前次阻擋報告）；起始工作樹乾淨。
- 本次已讀 `AGENTS.md`、`docs/REPORT.md`、前次阻擋報告，核對 `TASKS.md` 的 T012／T013 與啟動腳本首行 `#Requires -Version 7.0`。尚未進入白名單擷取或實作階段。
- `docs/DEMO.md` 在此基準不存在，未變更。

### 已驗證：PowerShell 前置結果

執行 `(Get-Command pwsh).Source` 失敗，沒有可記錄的 pwsh 實際路徑；錯誤原文：

```text
無法辨識 'pwsh' 詞彙是否為 Cmdlet、函數、指令檔或可執行程式的名稱。請檢查名稱拼字是否正確，如果包含路徑的話，請確認路徑是否正確，然後再試一次。
```

目前工具所啟動的 shell（僅用於前置查核與報告交付）：

```text
(Get-Process -Id $PID).Path
C:\WINDOWS\System32\WindowsPowerShell\v1.0\powershell.exe

$PSVersionTable.PSVersion.ToString()
5.1.19041.6456
```

此版本是 **Windows PowerShell 5.1**，不是 pwsh 的版本。PowerShell 7 路徑／版本均為「未取得」，不可把上述數值當成符合前置要求。

本次補充只讀搜尋結果：

- `C:\Program Files\PowerShell\7\pwsh.exe`、`C:\Program Files\PowerShell\7-preview\pwsh.exe`、`C:\Users\CHAOREN\AppData\Local\Microsoft\WindowsApps\pwsh.exe`：`Test-Path` 均為 False。
- `C:\Program Files\PowerShell`、`C:\Users\CHAOREN\AppData\Local\Microsoft\WinGet\Packages`、`D:\dev-tools` 均不存在。
- `rg` 不在本次 PATH，改以 `Get-ChildItem -Recurse -File -Filter pwsh.exe` 搜尋。使用者 `AppData\Local\Programs`、`D:\dev-orchestrator\tools`、`D:\lito-dev-runtime` 搜尋均無結果，且搜尋錯誤數為 0。
- `D:\dev-cache` 遞迴搜尋未取得 `pwsh.exe`，但有 2 筆搜尋錯誤，因此此目錄不能視為完整排除；沒有為了搜尋變更權限。

這些結果只證明本工作階段 PATH 與上述位置沒有找到可用 pwsh，不推論整台電腦絕無其他安裝位置。未下載或安裝 PowerShell、未變更 PATH 或其他系統設定。

## 未驗證什麼

- 尚未新增 `維護工具/run-local-test-suite.ps1`、`維護工具/test-whitelist/lib.txt`、`維護工具/test-whitelist/tests.tsv`。
- 完整版與 `-Quick` 均未執行，沒有 libtest 總表、summary.txt 或腳本最終結束碼。**結束碼為未取得，不是 0。**
- #53 的 lib `1889 passed／0 failed／575 ignored`、#55 的 `1／1`、#56 的 `76／76` 僅為任務給定的歷史比較基準，本輪沒有新測試數據可比較；未把歷史通過數算成本輪結果。
- 白名單以外的 ignored／整合測試、正式服務／DB、GUI 與完整 M1／G06 仍待驗證；本輪沒有執行任何 Cargo、遊戲或測試程序。

以下是解除阻擋後必須實跑的預定指令，**不是本輪已驗證可直接使用的交付**（腳本尚不存在）：

```powershell
pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1
pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1 -Quick
```

預期完整版總表包含 check、lib Debug／Release、1 項 ignored lib 與 76 項白名單整合測試；Quick 只含 check 與 lib Debug。每列須有步驟、結束碼、passed、failed、ignored、未實際執行與耗時，且全部符合任務成功門檻才可回傳 0。這是待實作規格，不是實際輸出；不得以此代替原樣總表或實跑收據。若 pwsh 不在 PATH，下一輪應改用經確認的完整路徑實跑並記錄。

## 產出路徑

本輪唯一追蹤變更：`docs/reports/issue-101.md`。未修改 `docs/REPORT.md`、src、tests 或其他任務檔案；原始專案只讀，未操作封存、硬連結、機密設定或既有遊戲程序。

## 給下一棒的提示

第 2 次嘗試仍重現相同前置阻擋。由調度器安排本機執行器確認／補齊 PowerShell 7 的可用執行環境，取得實際 `pwsh.exe` 路徑與版本收據後再重派 #101；若有既有安裝，提供完整路徑即可繼續前置查核，無須要求使用者代跑測試。此阻擋報告不代表 T012 已完成，請保留 blocked／未驗收狀態。

解除後先記錄該 pwsh 的實際路徑與版本，再按 #54 原文建立 UTF-8／LF 白名單、實作腳本與嚴格成功門檻，實跑完整與 Quick 兩條命令。以本報告更新替換阻擋內容，附原樣總表、最終結束碼與基線比較；若當時 T013 已合併，再按授權更新 DEMO 測試連結。完成後交 Claude 審查、使用者合併。
