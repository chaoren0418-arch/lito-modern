# Issue #301 — T035（M3 1/2）截圖與輸入：FrameArrived 逾時、geometry unavailable、SendInput 探針（claude-local）

## 結論

本張做了兩件事，都只到「程式與純邏輯已驗證」，**實機是否因此恢復截圖與點擊，尚未驗證**：

1. 客戶端三個互動入口的視窗，啟動時一定會還原、移進工作區並立即繪製。以前只處理「啟動器要求隱藏」，沒有處理「啟動器要求最小化」和「落在螢幕外」。
2. 新增輸入／擷取前置檢查工具 `維護工具/owned-gui-window-probe.py`。它在送出任何輸入之前，查出視窗是否最小化、隱藏、被 cloak、在螢幕外、不是前景，或輸入桌面不可用；可以選擇只對自己啟動的視窗做還原；而且不論成功、失敗或例外，都會寫出 `result.json`。

依本機限制，本輪沒有執行任何編譯出的程式，也沒有開任何視窗。

## 原因分析

- **已確認的歷史觀測**（讀 repo 內的紀錄，不是本輪重現）：
  - `docs/NATIVE-LOGIN.zh-TW.md:14`：`FrameArrived timed out`、`window capture timed out`、`coordinate input geometry is unavailable`；同一輪無障礙樹與鍵盤可用。
  - `維護工具/checkpoint-native-GUI-minimized-input-guard.py:62`：自動化工具回報 `window is minimized; call activate_window…`，還原時被 input guard 擋下，重新觀測仍是最小化。該檔同時寫明 `cause_proven=False`。
  - `docs/HISTORY.md` L1414–L1417：Release 的 OS 輸入探針擷取逾時、click 缺 geometry，控制器提前收尾，沒有 `result.json`。
- **推測的機制**：視窗最小化時沒有合成畫面，控件也沒有螢幕座標，所以擷取等不到 frame、座標輸入沒有 geometry，而無障礙樹與鍵盤仍可用。這和上面三筆觀測一致，但歷史紀錄沒有證明最小化的來源。
- **程式裡找到的一個確定缺口**：三個入口原本是 `ShowWindow(SW_SHOW)`，再用 `IsWindowVisible` 補一次。啟動器的 `STARTUPINFO` 若帶 `SW_SHOWMINIMIZED`／`SW_SHOWMINNOACTIVE`，第一次 `ShowWindow` 會被換成最小化；這時視窗算「可見」，補救條件不成立，視窗就停在最小化。這個缺口是否就是歷史失敗的來源，屬於推測。

## 做了什麼

- `LiTo381-Modern/src/modern_lito/window_presentation.rs`（新增）
  - `show_recovery`：依「可見／最小化」決定要補 `SW_SHOWNORMAL` 還是 `SW_RESTORE`。
  - `fit_to_work_area`：視窗外框超出最近螢幕的工作區時移回；比工作區大時才縮小。
  - `present(hwnd)`：顯示 → 還原 → 移回工作區 → 嘗試設為前景 → `UpdateWindow`。已最大化的視窗不動。只操作傳入的那一個視窗。
- `native_login_window.rs`、`native_startup_window.rs`、`scene_preview.rs`：原本的兩次 `ShowWindow` 改成呼叫 `present`。這三個是正常入口（連線設定 → 原生登入）和離線場景。
- `mod.rs`：登記新模組。
- `維護工具/owned-gui-window-probe.py`（新增）
  - 先核對目標程序：PID、建立時間（誤差 1 ms 內）、exe 在本專案內。不符就不檢查也不操作。
  - 唯讀列出該程序的最上層視窗：可見、最小化、最大化、cloak、外框、client 區螢幕座標、DPI、工作區、是否前景；另記輸入桌面名稱、前景視窗的 PID。
  - 判定三個旗標：`capture_ready`、`click_geometry_ready`、`keyboard_input_ready`，並列出原因代碼（例如 `minimized`、`not_foreground`、`client_area_partly_outside_work_area`、`input_desktop_unavailable`）。
  - `--recover`：只對主視窗做 `ShowWindowAsync(SW_RESTORE)`、`SetWindowPos` 移回工作區、`SetForegroundWindow`，最多 2 輪，每個 API 的回傳值都記錄。隱藏、cloak、輸入桌面不可用時不嘗試恢復。
  - `result.json` 一律寫出（先寫暫存檔再置換）。`sessions_observed` 固定為 `null`、`GUI_acceptance` 固定為 `false`：本工具不送輸入、不擷取畫面，不能拿來當 GUI 通過的證據。
  - 結束碼：0 就緒、2 未就緒、3 程序不存在或身分不符、1 其他錯誤。
  - 輸出資料夾限定在本專案 `scratch/` 或 `LiTo381-Modern/target/`。
- `.github/workflows/build.yml`：加一步 `owned-gui-window-probe.py --self-test`（純邏輯，不碰視窗）。

沒有改的東西：舊的 `start-*-sky-gui-case.py` 等控制器。它們一開始就讀 `工作佇列.json`，該檔已不在 repo，這些腳本現在無法執行，所以沒有逐支補 `result.json`。

`docs/REUSE-MAP.md` 已查：本張是 Win32 視窗呈現與自動化工具，表內沒有對應列，沒有引用 tw-bin-engine 或 lito-launcher-server 的檔案。

## 已驗證

- `cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --lib --tests`：結束碼 0；`--all-targets`：結束碼 0。新增與修改的四個檔案沒有新警告。
- `python -X utf8 維護工具/owned-gui-window-probe.py --self-test`：輸出 `self-test ok`，結束碼 0（Python 3.12.10）。內容是純邏輯：工作區計算、就緒判定、主視窗選擇、時間比對、輸出路徑限制，以及用假的 API 物件走過「就緒／未就緒／恢復後就緒／恢復兩輪仍失敗／身分不符」五條流程。沒有呼叫任何 Win32 視窗函式。

## 推測

- 啟動器以最小化方式啟動時，三個入口現在會還原視窗。依據是 `ShowWindow` 的文件行為與 `show_recovery` 的判斷；沒有實機看過。
- 若歷史上的最小化是啟動後才發生的（例如其他程式或「顯示桌面」），客戶端的修改幫不上忙，要靠探針的 `--recover`。
- 探針從沒有前景權的程序呼叫 `SetForegroundWindow` 時，Windows 可能拒絕；這時 `keyboard_input_ready` 會是 `false`，原因 `not_foreground`。
- 原生登入視窗在 192 DPI 的最小高度是 1600 像素。工作區比它矮時，`present` 只能把左上角移回螢幕，下半部控件仍在工作區外，探針會回報 `client_area_partly_outside_work_area`。這屬於版面問題，歸第 2 張。

## 待驗證（需要實機，交 Codex 本機）

1. `window_presentation.rs` 的 7 個單元測試：等本 PR 的 CI（`cargo test --lib`）結果。本機沒有執行。
2. 以最小化方式啟動自己建置的客戶端（例如 `Start-Process -WindowStyle Minimized … --configure-native`，不連線），確認視窗會自動還原，結束後依 PID 關閉。
3. 用 `Start-Process -WindowStyle Hidden`（`啟動現代客戶端.ps1` 的現行做法）啟動，確認視窗仍會顯示，沒有退步。
4. 探針實跑：啟動時記下 PID 與建立時間，執行
   `python -X utf8 維護工具/owned-gui-window-probe.py --pid <PID> --creation-utc <ISO 時間> --output scratch/gui-probe/<case> --window-class LiToNativeLoginWindow`，
   確認 `result.json` 的視窗資料與實際相符；把視窗手動最小化後加 `--recover` 再跑一次，確認狀態變成 `recovered`。
5. 探針回報就緒之後，自動化工具的截圖與座標點擊是否不再出現 `FrameArrived timed out`／`geometry unavailable`。**這一項才是原缺陷是否修好的判定，本輪沒有任何證據。**
6. Release 建置下的 OS SendInput 鍵盤輸入是否送達（先看 `keyboard_input_ready`）。
7. 多螢幕（含負座標）與 192 DPI 下，`present` 的移回行為是否正確。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/window_presentation.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`、`native_startup_window.rs`、`scene_preview.rs`、`mod.rs`
- `維護工具/owned-gui-window-probe.py`
- `.github/workflows/build.yml`
- `docs/reports/issue-301.md`

## 給下一棒的提示

- 做 GUI 自動化之前先跑探針；狀態不是 `ready`／`recovered` 就不要送輸入，把 `result.json` 附在報告裡。這樣即使失敗也有收據。
- 探針的 `reasons` 只記觀測到的狀態，不代表原因。若 `input_desktop_unavailable` 出現，表示桌面鎖定或在安全桌面，還原視窗沒有用。
- 第 2 張（版面與文字）若調整原生登入視窗的最小尺寸，請一併確認小工作區下控件都在螢幕內；`fit_to_work_area` 會把視窗縮到工作區大小，但 `WM_GETMINMAXINFO` 目前會擋住縮小。
- 其他入口（`login_window.rs`、`mode_window.rs`、各 `native_*_window.rs`）仍是單次 `ShowWindow(SW_SHOW)`，不在正常入口上，本張沒有改；需要時直接改成呼叫 `window_presentation::present`。
- T035 第 4 項（G08：失焦後不能持續發招）不在本張範圍。靜態看，`native_login_window.rs` 在 `WM_KILLFOCUS`／`WM_ACTIVATEAPP` 已清空按住狀態，但沒有實機證據。
