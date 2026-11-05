# Issue #86：任務切大一點＋「打開就看得到」的里程碑（V1）

## 做了什麼

1. **使用者確認**：#86 是本機調度器轉述的。其中「讓本機執行者啟動客戶端截圖」和規則「GUI 驗收等 M2 之後」衝突，雲端主控另外直接向使用者確認，使用者選擇「同意，並更新規則」（2026-11-06）。
2. **規則**：`CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc` 三份同步加入例外：本機執行者可以啟動**自己建置的**客戶端，進入不連線的畫面（設定畫面、登入畫面但不按連線、離線場景）並截圖，結束後依 PID 關閉自己啟動的程序。完整的 GUI 驗收（T030）仍等 M2。
3. **PLAN／TASKS**：新增「V1 打開就看得到」里程碑，分成 T012 與 T013 兩張，**平行**進行、沒有依賴，各約 1～2 小時。
4. **`docs/plans/issue-86.json`**：

   | id | stage | runner | 驗收物（使用者直接看的東西） |
   |---|---|---|---|
   | T012 | code | codex-local | `pwsh -NoProfile -File 維護工具/run-local-test-suite.ps1` 一個指令跑完 check、lib Debug／Release、白名單 ignored、白名單整合測試，印出總表；另有 `-Quick` 日常版。報告原樣貼出實際總表 |
   | T013 | test | dot | `docs/screenshots/` 三張 PNG（設定畫面、登入畫面、離線場景），加上 `docs/DEMO.md`：每張圖的重現指令與預期畫面，在 GitHub 上可以直接看 |

5. **任務切大**（#86 第 1 點）：
   - #82 的 T011-H1、T011-H2 已合併成一張 T011-H（`task/82` 的 `e77325b`）；
   - T012 把白名單檔、腳本與實際執行合在同一張；
   - T013 把建置、三個畫面的截圖與 DEMO.md 合在同一張。

   之後規劃時，只改文件或只改一兩行的小修改，一律併進相鄰任務。

### 第 2 次嘗試的修正（對應 codex-local 審查）

1. **T013 不能用錯誤紀錄代替截圖**：撤回「三張 PNG（或明確記錄失敗原因）」。改成完成條件是三張對應實際畫面的 PNG、DEMO.md 可以直接顯示圖片，並附重現指令與預期畫面。缺任何一張就算未完成（blocked），要如實記錄錯誤原文與恢復方式，但不能宣告 T013 或 V1 完成。plans、TASKS、PLAN 都已同步。
2. **T012 的綠燈門檻**：明定 check 結束碼 0；Debug 與 Release 都必須有有效的 libtest 結果，而且 failed=0；白名單每一項都必須 `running 1 test` 而且通過。出現 0 tests、結果缺失或解析失敗、failed>0、命令非 0，腳本最終結束碼都必須非 0，任務也不能驗收通過。測試數的合理變動可以說明，但不能免除門檻。plans、TASKS、PLAN 都已同步。

### 第 3 次嘗試的修正（對應 #93 審查的 6 點）

1. **[P1] 離線場景視窗不可見**：靜態核對屬實。`啟動現代客戶端.ps1:72` 用 `-WindowStyle Hidden` 啟動；`scene_preview.rs:245` 只呼叫一次 `ShowWindow`；`native_login_window.rs:2350-2356` 與 `native_startup_window.rs:325-328` 已有 `IsWindowVisible` 加第二次 show 的修正，離線場景沒有。上一輪讀過啟動腳本第 60–80 行，卻沒有指出這個差異，是我的疏漏。採用 (a)：把 `scene_preview.rs` 的同款修正併入 T013（stage 改為 code），驗收加上「照 DEMO.md 的重現指令啟動時視窗確實可見」。
2. **[P2] 規則檔第 55 行矛盾**：`CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc` 三份同步改為「完整原生 GUI 驗收只由使用者執行；不連線畫面截圖的例外見『目前狀態』」。
3. **[P2] pwsh 前置檢查**：T012、T013 第一步都要記錄 `(Get-Command pwsh).Source` 與版本，7.0 以上才繼續，找不到就 blocked。T012 給使用者的兩行指令要在使用者環境實際可用。
4. **[P3] PID 取得方式**：T013 寫明先記錄啟動前的 `LiToModern` 清單，啟動後只認 `Path` 等於自建 exe、而且不在啟動前清單內的程序。
5. **[P3] T012 子程序環境**：TEMP／TMP 指向 `LiTo381-Modern/target/current/scratch`，清除繼承的 `LITO_*`。
6. **[P3] DEMO 連結**：T012 還沒合併時寫「待 T012 合併」，由後合併的那一張補上連結。

## 怎麼驗證

- 雲端讀 `啟動現代客戶端.ps1` 第 60–80 行，確認三個模式（`-ConfigureNative`、`-NativeLoginEndpoint`、`-OfflineScene`）都是 `Start-Process` 啟動 exe 後就返回。`docs/NATIVE-LOGIN.zh-TW.md` 記載登入視窗要在使用者按「連線」後才建立連線，所以登入畫面截圖不會連線；T013 另外要求連線目標是一個已關閉、OS 配發的 loopback 埠。
- `python3 -m json.tool docs/plans/issue-86.json > /dev/null; echo "json.tool exit=$?"` 輸出 `json.tool exit=0`，共 2 筆。
- 三份規則檔只改同一行。

## 未驗證什麼

- 三個畫面能否成功截圖（歷史上曾出現 `FrameArrived timed out`），以及離線場景能否正常渲染：交給 T013 實際執行，失敗要如實記錄。
- T012 的總表數字：交給本機實際執行。

## 產出路徑

- `CLAUDE.md`、`AGENTS.md`、`.cursor/rules/project.mdc`
- `PLAN.md`、`TASKS.md`
- `docs/plans/issue-86.json`
- `docs/reports/issue-86.md`（本檔）
