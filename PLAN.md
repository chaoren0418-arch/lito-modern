# PLAN — lito-modern

更新：2026-11-04。本檔由 Claude 維護；任務細目在 `TASKS.md`。

## 一句話目標
Rust 原生 Windows x64 客戶端（G01–G12），沿用既有資源與 UI，重現 LiTo 已驗證的玩家行為：登入→大廳→進房→戰鬥→退出，含 HUD、聊天、背包、快捷列、技能特效、音效。

## 現況（事實，來自 2026-10-25 總開發報告）
| 項目 | 狀態 |
|---|---|
| 正常來源 | SHA `3E58268A…`，1048 來源檔；Debug `AB99BED1`／Release `4340DA5F` |
| 正式唯讀 JAR | `E1498871…`（對應 lito-launcher-server 0.6.71） |
| 最新隔離候選 | 44 檔（30 Runtime／11 Rust QA／3 Java），372 module；**不可套用** |
| 開發停點 | **#97**（Scene／History 接線、資源就緒 gate、暗隱術）；原共享 callback／重入繪圖／固定 sample 交錯未補 |
| 測試 | **475 案全未跑**；G06 兩版 16 案 640 次未跑；歷史 r75 兩版各 1975 過 0 敗 488 忽略（非當版） |
| GUI | 僅歷史 r75 有限資格；FrameArrived 逾時／geometry unavailable 未證明恢復 |
| 可攜交付 | 舊 21 檔版 ZIP 已過期，依賴 VS18 預覽 runtime |
| 使用者指示 | 2026-10-25「只做開發，測試與桌面操作暫緩」；**2026-11-05 已同意恢復測試**（Issue #43），測試交本機 `dot`／`codex-local` 執行；GUI 驗收等 M2 之後 |

## 真正的缺口
1. **太多未驗證的候選疊在一起**：44 檔候選 + 475 案未跑 + 640 次 G06 未跑。再往上疊只會更難接回現實。
2. 技能逐條接線（#45–#98）是巨大的長尾，每條都「來源已接、server 權威、未驗」。
3. 沒有 CI；`cargo check` 從未在乾淨機器上跑過。
4. 與 lito-launcher-server 的介面（正式 JAR、協定）靠 SHA pin，沒有契約文件。

## 里程碑
### M0 可建置（本週）
- CI `cargo check --locked --all-targets` 綠（T001）。
- `cargo test --lib` 能跑：先把不需視窗／socket 的單元測試跑起來，統計通過／失敗（T002）。
- 把 44 檔候選的狀態寫清楚：哪些已併入 `src/`、哪些還在候選目錄（T003）。

### M1 停止疊加，先驗證（使用者已於 2026-11-05 同意恢復測試，本機執行）
- 跑當版 12 個 gate 案（4 案需原資源）與四元件 inventory。
- 跑 G06 16 案 640 次，核對 PID／FILETIME／Job／DB 退出與 33／31 秒自然期限。
- 首個失敗保存原收據，變成 TASKS 的 bug。
- 這一步之前**不接新技能**。

### M2 完成 #97–#100 與 G06
- #97 共享 callback／重入繪圖／固定 sample／巢狀 drain；#98–#100；剩 92 技能依優先序。
- G06 傳送／角色接續／舊世界退役／失敗恢復。

### M3 GUI 與交付
- 完整 GUI：普通房、租裝、旅館返回、Enter 負向、EOF／取消、新技能、退役與傳送恢復；實體輸入／像素／IME／多 DPI／聲音。
- Release 廣域回歸、clean host 可攜交付（不依賴 VS 預覽 runtime）、G01–G12 逐要求稽核。

## 與 lito-launcher-server 的關係
重建版只**唯讀**引用正式 JAR 與協定；任何伺服器端修改在那個 repo 做，這裡只更新 pin。T004 要寫一份協定契約文件取代純 SHA。

## 分工
Claude：PLAN／TASKS、審 PR、CI、架構。Cursor：Rust 實作。ChatGPT：Rust 棘手錯誤第二意見。使用者：驗收與觀測（GUI 實機驗收在 M2 之後）。測試由本機 `dot`／`codex-local` 執行。

## 停用的舊流程
`工作佇列.json`（11.8 MB）、`驗證紀錄/` 每輪 evidence JSON、`維護工具/candidates/` 時間戳目錄、總開發報告的 checkpoint 機制。歷史保留，不再新增。
