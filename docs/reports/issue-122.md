# Issue #122 — T011-H runner 程序身分、清理與日誌修正

**已驗證：指定 runner 修正完成；PowerShell 7 語法錯誤 0、五案 WhatIf 結束碼全為 0、四種合成程序檢查與證據複製檢查通過。沒有執行 G06、Java harness、Rust 測試、遊戲或 GUI；G06 結果仍待驗證。**

## 做了什麼

- 日期：2026-11-06（Asia/Taipei）；分支 `task/122`，起點 `241121ba81b8cffc65921677ff112b953e8a017f`。沿用既有 worktree 與分支，沒有 checkout／建分支。
- 工作目錄 `W = D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-122`；已讀 AGENTS.md、docs/REPORT.md、TASKS.md 的 T011，以及 #91、`task/118` 的 #118 報告。
- 僅追蹤修改 `維護工具/run-teleport-failure.ps1` 與本報告。不改 Rust、Java harness、TASKS.md、docs/REPORT.md 或其他追蹤檔。沒有操作既有程序、正式服務／DB、機密設定、封存或硬連結。
- 既有 `task-118` 可攜 PowerShell 僅讀取執行；正式 E149 JAR 的既有副本僅讀取、核對 SHA 後複製到 W。所有測試寫入都在 W；TEMP／TMP 指向 W 的 scratch。

### Diff 摘要

| 修改 | 行為 |
|---|---|
| `Initialize-ProcessImageApi`／`Read-ProcessImage` | P/Invoke `QueryFullProcessImageNameW`，傳入 `Start()` 所得 Process 的原始 `SafeProcessHandle`。interop 在啟動子程序前完成編譯，不占用 image 查詢期限。 |
| `Get-OwnedProcessImage` | 每次失敗後最多等 100ms 重試；Stopwatch 預算 5000ms。逾時丟出 `Blocked`，receipt 留下查詢次數、耗時與 `identity_error`；主流程標記 `blocked`。清理／漂移另有錯誤時仍標 `failed`。 |
| `Start-OwnedProcess`／Rust 前守門 | 登記自有 Process／handle 後讀 PID、StartTime、image，image 必須與指定完整 executable 路徑相等，才設 `identity_bound=true`。Rust 啟動前再明確檢查 javac 與 Java 三項身分均綁定；正式呼叫仍固定 JDK 的 javac.exe、java.exe 與本次 immutable-test.exe。 |
| `Stop-OwnedProcess` | 未綁定時只使用原始 Process 的既有 handle 結束自己啟動的程序，不呼叫 `GetProcessById`。receipt 記 `original_handle_unbound_image`（或 image 不符時的 `original_handle_unbound_identity`）、killed、WaitForExit、退出碼與 cleanup_errors。已綁定分支重新查詢程序的 PID／StartTime／MainModule image 比對保留原式；Kill 仍使用原始 Process。 |
| 日誌 | javac／Java／Rust 啟動後，stdout／stderr 以非同步串流直接寫 R 的 `<kind>.stdout.txt`／`<kind>.stderr.txt`，關閉 FileStream 緩衝以保留已到達的部分輸出。清理獨立等待串流並複製到 V，即使 drain 失敗仍嘗試複製，錯誤寫入 cleanup_errors。Rust 的通過文字檢查改讀 drain 完成後的 R 日誌，原有判定不變。 |
| ready 證據 | 保留原 finally 的 `R\runtime\ready.json` → `V\ready.json` 複製流程；新增合成驗證確認阻塞狀態也會複製。未產生 ready 時不捏造檔案。 |

R／V 仍使用 runner 原有路徑契約：`LiTo381-Modern/target/current/native-teleport-failure/<stamp>` 與 `LiTo381-Modern/target/current/verification/native-teleport-failure/<stamp>`。本次沒有執行主流程或建立 scenario run。

## 怎麼驗證（已驗證）

驗證使用 PowerShell **7.6.5**：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe
```

在 W 執行以下輔助腳本，兩者最終結束碼皆 **0**；外層先將 TEMP／TMP 設為 `W\scratch\issue-122\tmp`。

```powershell
& '..\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -NonInteractive -File scratch/issue-122/verify.ps1
& '..\task-118\scratch\tools\pwsh\pwsh.exe' -NoProfile -NonInteractive -File scratch/issue-122/check-evidence.ps1
```

| 檢查 | 結果 |
|---|---|
| `Parser.ParseFile` | 最終腳本 **0 個語法錯誤**。 |
| `timeout -WhatIf` | **0** |
| `refused -WhatIf` | **0** |
| `wrong-service -WhatIf` | **0** |
| `cancel-retry -WhatIf` | **0** |
| `cancel-expiry -WhatIf` | **0** |
| WhatIf 隔離 | 五案 JSON 都是 E149；每案預覽的 run、output、lease 均未建立。`TestBinary` 是 target/current 下不存在的 `not-built.exe`，未建置或執行 Rust。 |
| JAR SHA256 | `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`；副本位於 `W\LiTo381-Modern\target\current\server-reference\formal-server-E1498871.jar`。 |
| 身分負例 | 對正常／重試成功收據分別移除 PID、StartTime、image 或 identity_bound，共 8 項均由 `Assert-BoundIdentity` 拒絕。 |
| 重新查詢守門 | 對照 HEAD 的原始程式，PID／StartTime／image 拒絕條件逐字相同（忽略縮排）。 |
| ready 複製 | AST 擷取 runner 原 finally 的證據複製敘述，在合成 blocked 收據下執行；合成 `runtime/ready.json` 與 V 副本 SHA 相同，cleanup_errors 為空。內容明確標記為合成文字，不是 Java ready 或 scenario 證據。 |
| diff | `git -c core.whitespace=cr-at-eol diff --check` 通過。 |

### 合成程序結果

`verify.ps1` 透過 AST 載入 runner 的實際函式，不執行 runner 主流程。每例只啟動自己的無害 `pwsh -NoProfile -NonInteractive -Command ... Start-Sleep 30`，子程序先寫 stdout／stderr 中文標記。失敗注入僅替換輔助腳本範圍內的 `Read-ProcessImage`，正式 runner 不含測試開關。

成功取得的 image 完整路徑：

```text
D:\lineage開發\3-lito開發\lito重建版\scratch\wt\task-118\scratch\tools\pwsh\pwsh.exe
```

| 情況 | PID | image 查詢 | 清理方式 | WaitForExit／退出碼／cleanup_errors |
|---|---:|---|---|---|
| 正常查詢 | 24908 | 1 次；**17ms**；路徑等於預期 | `original_handle_verified_identity` | true／-1／`[]` |
| 前兩次模擬失敗，第三次呼叫真實 API | 6428 | 3 次；**239ms**；路徑等於預期 | `original_handle_verified_identity` | true／-1／`[]` |
| 持續模擬取得失敗 | 12420 | 45 次；**5007ms**；image=null、identity_bound=false，拋出 Blocked | `original_handle_unbound_image` | true／-1／`[]` |
| 模擬錯誤 image | 24084 | 1 次；**0ms**；回傳合成錯誤路徑，拒絕綁定 | `original_handle_unbound_identity` | true／-1／`[]` |

四例均 `killed=true`；退出碼 -1 是合成程序被 Kill 的實測結果，不代表自然退出成功。5007ms 是 5000ms 預算加排程／拋例外開銷。四例 stdout／stderr 的 R、V SHA 全部一致；前三例均含預期中文輸出，image 不符案例可能在子程序寫字前已被停止，沒有將空日誌算成初始化成功。最終 receipt 在 R／V 各有一份。

模擬取得失敗的收據重點：

```json
{
  "pid": 12420,
  "start_utc": "2026-11-06T00:25:34.8405559Z",
  "image": null,
  "identity_bound": false,
  "image_query_attempts": 45,
  "image_query_elapsed_ms": 5007,
  "identity_error": "Blocked: process image unavailable within 5000ms: Synthetic image query unavailable",
  "cleanup_method": "original_handle_unbound_image",
  "killed": true,
  "wait_for_exit": true,
  "exit_code": -1,
  "cleanup_errors": []
}
```

初次合成檢查揭露 interop 冷編譯耗時：正常查詢被計為 7425ms，超出需求；當時輔助腳本沒有成功查詢的期限斷言，不能視為期限通過。已將 Add-Type 移到子程序 Start 前，補上成功查詢須小於 5000ms 的斷言，再執行完整五案 WhatIf／合成檢查；以上表格只採最終結果。初版原始摘要保留在 `scratch/issue-122/summary.json`，不混作最終證據。

## 產出路徑

| 產出 | W 內相對路徑 |
|---|---|
| 追蹤腳本／報告 | `維護工具/run-teleport-failure.ps1`、`docs/reports/issue-122.md` |
| 合成檢查／證據複製檢查 | `scratch/issue-122/verify.ps1`、`scratch/issue-122/check-evidence.ps1` |
| 最終摘要 | `scratch/issue-122/final-summary.json`、`scratch/issue-122/evidence-check.json` |
| 五案 WhatIf 原始輸出 | `scratch/issue-122/<scenario>.whatif.stdout.txt`、同前綴 `.stderr.txt` |
| 合成 R／V 的收據與日誌 | `scratch/issue-122/run-final/`、`scratch/issue-122/verification-final/` |

scratch、target/current 均不納入 git；輔助腳本採新建目錄，重跑應先指定新的自有證據目錄，避免覆寫已保存的合成 R／V。

## 未驗證什麼／推測

- **待驗證**：實際 Java／javac／immutable Rust exe 的啟動與退出鏈、listener 所有權、E149 memory 初始化、所有 G06 scenario 與完整原生 GUI。沒有 Rust `running 1 test`／1 passed，也沒有實際 result.json；本任務不將合成檢查計入 G06 通過數。
- **待驗證**：真實系統發生 Kill／權限／磁碟寫入失敗時的整條主流程；本次未注入這些環境失敗。程式會保留錯誤，部分日誌不能當完整輸出。
- **推測**：#118 啟動當下 MainModule 失敗可能涉及模組載入或枚舉時序；本次沒有定案其根因。修正避開啟動時的 MainModule 依賴，並實測原始 handle 的替代查詢與清理。

## 給下一棒的提示

由 **Claude／調度器**審查 `task/122`，維持 T011-H 為 doing，審查合併後重新排 **#118 的本機執行器**驗收。TASKS.md 狀態由調度器處理，本任務依指定範圍未修改。下一批次從合併後 main 在自己的 worktree 重建 exe，再依原順序跑 G06，首個失敗／阻塞即停止；請核對新的 R／V 日誌、ready 副本及 receipt。

安全契約沒有放寬：Rust 啟動前 javac 與 Java 的 PID＋StartTime＋image 必須全綁定，image 必須等於預期路徑；Rust 自身啟動後也立即綁定 immutable image。只有未綁定程序的**自有原始 handle 清理**新增合法出口，重新查詢程序的三項比對仍保留。
