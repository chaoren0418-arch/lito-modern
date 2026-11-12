# Issue #325 — T037（M3 2/2）G12 稽核：46 項逐項核對，map563 與 ARGO 的範圍判定

執行者：claude-local（本機，靜態稽核）。日期 2026-11-12。基準：客戶端 `origin/main` `d7a0ecb`、正式 JAR E149（`E1498871…D498`）。

## 結論

- 產出 `docs/G01-G12-AUDIT.md`：46 項各一列，寫明來源 state、當版併入的工作、結論、已知限制、重跑指令；另有來源漂移、收據核對、兩項範圍判定。
- **46 項在當版沒有任何一項驗收成立**：30 項「程式已在 main，等當版實跑或實機」、14 項「還沒做完」、1 項等伺服器（G05.4）、1 項排除（G06.4）。G12 的 acceptance 不成立。
- **map563：判定在範圍內**（U-M3-6）。它是房間模式 3（攻城）的地圖，E149 與重建版的建房介面都有；當版與 E149 沒有任何攻城房間的實跑結果，舊 runner 也不能對 E149 執行。
- **ARGO：判定在範圍內**（U-M3-8）。E149 有完整的 `/argo` 觀戰目錄（虛擬頻道 99）；重建版收得到回覆、送不出請求，沒有 UI 入口。
- 歷史收據都還在：`docs/G01-G12.md` 引用的 139 個路徑全部存在，67 組 SHA256 重算相符、0 組不符。但它們都綁在舊來源與舊 JAR，不能移記到當版。

## 做了什麼

1. 讀 `CLAUDE.md`、`docs/REPORT.md`、`docs/REUSE-MAP.md`、`PLAN.md`、`TASKS.md` 的 T037 與 M3 完成定義、`docs/reports/issue-324.md`、`docs/reports/issue-92.md` 附錄 A、C、E1、F。
2. 把 `docs/G01-G12.md` 的 46 項逐項讀完（title、work、acceptance、state、remaining、qualified_scope 等欄位；G06 的 `rental_acceptance_steps` 只取其中引用的路徑與雜湊）。
3. 對每一項查當版 main 併入了哪些 Issue（`gh issue list`，2026-11-12），讀對應報告的結論與「待驗證」段落：#53、#55、#56、#101、#118、#139–#141、#167–#195 中的 27 份、#299–#301。
4. 寫唯讀工具 `維護工具/g12-audit-evidence-check.py`，核對 `docs/G01-G12.md` 引用的收據是否還在、SHA256 是否相符；再對 `target` 下的 EXE 算雜湊，找出被收據引用的。
5. 重算 E149 JAR（登入器路徑與固定副本）、TW 資源包的 SHA256；量 `92bbda0..d7a0ecb` 的來源變更；查 CI 最近一次完成的 lib 測試結果；查伺服器 repo 的最新發布紀錄（唯讀）。
6. map563：讀重建版 `native_room.rs`、`native_login_window.rs`、`tests/native_arena_portal.rs`、`維護工具/run-native-arena-portal.ps1`，對 E149 JAR 的 class 找地圖常數，讀歷史收據 `native-startup-siege563` 的兩份 `qualified.json`。
7. ARGO：對 E149 JAR 的 class 找字串；讀伺服器當版原始碼 `ClientSession.java`、`GameHub.java`（`git show origin/main:…`，沒有 checkout、沒有寫檔到對方 repo）；讀重建版 `native_lobby.rs`、`transport.rs`。

REUSE-MAP：稽核本身沒有對應項。引用的參考來源是 lito-launcher-server `origin/main` `edfb5c8` 的 `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/ClientSession.java`（8800–8856、9192–9296 行）與 `GameHub.java`（4815–4837 行）。沒有引用 tw-bin-engine。

## 已驗證／推測／待驗證

**已驗證**（都是靜態：讀檔、重算雜湊、查 GitHub）

- 46 項的來源 state、work、acceptance 原文（`docs/G01-G12.md`）。
- 各 Issue 的開關狀態（2026-11-12 查詢）與各報告寫的「待驗證」內容。
- 139 個引用路徑全部存在；68 組附 SHA256 的，67 組相符、1 組是目錄、0 組不符。
- E149 JAR 兩處的 SHA256 相同，都是 `E1498871C7BE6B287821C57EF5D95933DB1740D9B4D4A80F6C3465F21D99D498`。
- CI `b3d2a27` 那次（`d7a0ecb` 的前一個 commit）的 lib 測試：2326 通過、0 失敗、597 ignored。
- 來源變更量：`src/` 187 檔（+29,452／−1,152），`tests/` 53 檔（+8,910／−526）。
- E149 的 `BattleRules$Mode.class` 等 4 個 class 含 547／563／573；`ClientSession.class`、`GameHub.class` 等含 ARGO 相關的方法名與日誌字串。
- 重建版 `mode_maps(3)` 是 `[573, 563, 547]`，房間介面有地圖選單；`enter(id)` 只收 0–3，99 送不出去。
- `run-native-arena-portal.ps1` 的專案根寫死、JAR 白名單沒有 E149。
- `target` 下 4,111 個 EXE 裡，有 152 檔（63 個不同雜湊，1.33 GiB）的 SHA256 被 `docs/G01-G12.md` 引用的收據記載；全部在 `target/current/` 內。

**推測**

- ARGO 在 E149 的詳細流程（先 C_CB 再 C_CA 99、清單只含已開戰且允許觀戰的房間、加入不檢查密碼、S_E5 先於 S_CE）：讀的是伺服器當版原始碼。方法名與日誌字串和 E149 相同，但沒有反編譯 E149 逐行比對。
- 「普通頻道的房間清單進不了已開戰的房間」：依伺服器的 `BATTLE_STARTED` 加入失敗碼推得，沒有實測。
- 各項「待實跑」的程式在當版是否真的能通過：不知道。lib 測試通過不代表整合測試或實機會通過。
- U-M3-5（畫面等價研究）在 46 項原文裡沒有依據：這是附帶紀錄，本張沒有被要求判定。

**待驗證**

- 46 項的實際驗收全部待驗證。本張沒有執行任何測試。
- G01 的「已發布 EXE 雜湊」：來源 `3E58268A` 的 Debug `AB99BED1`／Release `4340DA5F` 兩個檔本次沒有去找，也沒有重算。
- 伺服器四張移交 Issue（lito-launcher-server#368–#371）與觀戰離場（#366）的處理結果。
- `驗證紀錄/` 裡沒有被 `docs/G01-G12.md` 直接引用的收據（#324 記有 102,449 檔）沒有核對。

## 未做的事

- 沒有執行任何編譯出來的程式，沒有建置，沒有開視窗，沒有連伺服器或 DB。
- 沒有在原始專案目錄寫檔；沒有動封存、硬連結、客戶端資源、伺服器 repo。
- 沒有改 `TASKS.md`、`PLAN.md`、`docs/REPORT.md`、`docs/G01-G12.md`、`docs/GUI-ACCEPTANCE.md`。
- 沒有寫 `docs/plans/issue-325.json`：本張是 code 階段。U-M3-6、U-M3-8 要不要開張由主控決定，建議內容在稽核表 §4.3、§5.4。
- 沒有刪除 #324 列出的任何冗餘產物。

## 產出路徑

- `docs/G01-G12-AUDIT.md`：稽核表（本張的主要產出）
- `docs/reports/issue-325.md`：本檔
- `docs/reports/issue-325/evidence-summary.tsv`、`evidence-detail.tsv`：收據核對結果
- `docs/reports/issue-325/cited-exe.tsv`：被收據引用的 EXE
- `維護工具/g12-audit-evidence-check.py`：收據核對工具（唯讀）

## 給下一棒的提示

1. **主控（雲端對話）要裁定的三件事**：
   - 依 U-M3-6 開「攻城房間（map563／547／573）當版重驗」一張，內容見稽核表 §4.3。第 1 步要先改 `run-native-arena-portal.ps1`。
   - 依 U-M3-8 開「ARGO 觀戰目錄」一張，內容見稽核表 §5.4。動手前要先核對 C_CB 格式與原版送包順序。
   - TASKS.md 第 432、439 行把「攻城 P 鍵」列為 WORLD 專屬。HISTORY L1199 記的是攻城房間的 101／102 入口。請確認是否改列在範圍內（程式已在 main，只缺重驗）。
2. **沒有任務實作的限制**：G05.1 的數值 schema 1–4（當版只收 schema 0；GUI 清單的做法是用 schema 0 啟動伺服器）。這個限制要不要接受，請主控決定。
3. **Codex（本機測試執行者）**：稽核表 §9 的 R2–R5 是各項的重跑指令。30 項「待實跑」裡，不需要 GUI 的部分可以先用 R2（`run-local-test-suite.ps1`）與 R3 取得當版結果。
4. **更新稽核表的時機**：T030、T031、T036、T038 各自完成後，回頭改 §3 對應列的結論，並補上當時的客戶端 commit 與 JAR SHA。做法寫在稽核表 §10。
5. #324 的冗餘產物刪除仍等使用者同意。本張補了一個判斷依據（稽核表 §7.2）：A 類不含 EXE，可以刪；`target/current/` 以外的歷史輪次目錄沒有 EXE 被 G 收據引用；`target/current/` 內 cargo 版面以外的部分有 152 個被引用的 EXE，不能整批刪。B 類的 rlib／pdb 沒有比對過雜湊，去重前要另外核對。
