# Issue #299 — T040-1：修補提案四項移交伺服器 repo

## 結論

四項修補提案已各開一張 lito-launcher-server 的 Issue，都標 `needs:claude`，等伺服器主控裁定是否接收。**`修補提案/` 還沒刪**：任務規定伺服器接收後才刪，目前四張都還沒有回覆。

| 提案 | 伺服器 Issue | 性質 |
|---|---|---|
| 租裝返回 | [lito-launcher-server#368](https://github.com/chaoren0418-arch/lito-launcher-server/issues/368) | 缺陷，當版 src/main 仍在 |
| 同端點傳送 | [lito-launcher-server#369](https://github.com/chaoren0418-arch/lito-launcher-server/issues/369) | 先核對：現行已有另一種守衛，是否足夠未知 |
| 房間最後離開 | [lito-launcher-server#370](https://github.com/chaoren0418-arch/lito-launcher-server/issues/370) | 缺陷，當版 src/main 仍在 |
| 商人屬性來源 | [lito-launcher-server#371](https://github.com/chaoren0418-arch/lito-launcher-server/issues/371) | 缺陷，當版 src/main 的計算來源仍缺 |

## 做了什麼

- 讀完 `修補提案/` 四個目錄的 README、patch、candidate／validation JSON（共 37 個追蹤檔）。
- 查 `docs/REUSE-MAP.md`，依它的讀法唯讀對照伺服器 `origin/main` `edfb5c82` 的 `ClientSession.java`、`RentalEquipmentBonuses.java`、`NativeLitoRentalMerchantCatalog.java`，確認四項在當版原始碼的現況。
- 先搜尋伺服器 repo 既有 Issue（關鍵字：租裝返回、同端點、最後離開、商人、quitFight、broadcastDefaultRoomReset、RentalEquipmentBonuses、rentalItemSelectionContext、修補提案），沒有重複的；最接近的 #366（觀戰者離場）已關閉，改的是同兩個方法的另一個分支。
- 每張 Issue 寫：問題、當版現況（附檔案與行號）、提案修法、證據路徑、提案紀錄的驗證範圍、建議驗收條件。
- 證據路徑用本 repo commit `8a9fa0753da753b4489f40f89dc358df0b729c71` 的固定連結，之後刪掉 `修補提案/` 也讀得到。只在本機、不在 git 的 `驗證紀錄/…` 與候選 JAR 另列本機路徑。
- ASM 工具（`Patch*.java`、`current-guard-candidate/` 的腳本）依任務**沒有搬**，Issue 裡只當紀錄提到。
- 本 repo 的變更只有這份報告。

## 已驗證

以下都是唯讀靜態核對，沒有執行任何程式。

- 伺服器 `origin/main` `edfb5c82` 的 `ClientSession.java`：
  - `quitFight()`（約 L13459–L13468）：`phase == BATTLE` 時除了 #366 的觀戰者分支，一律 `settleRequestedBattleExit()`；租裝轉場的處理（約 L13473、L13548）在 `else` 裡。
  - `exitRoomBeforePolyMe(boolean)`（約 L13149–L13155）：BATTLE 先結算並 return；`RESTART_AWAIT_RETURN → completeRestartReturn(true)`（約 L13168）在後面。
  - `rentalItemSelectionContext()`（約 L8960–L8970）：有 `hasRentalBattleHandoff` 守衛，沒有提案的 `rentalBattleLoadoutActive` 判斷。
  - `broadcastDefaultRoomReset(long)`（約 L13316–L13321）：直接 `orElseThrow(IOException)`，沒有 `roomExists` 檢查；呼叫點約 L13231。
- `RentalEquipmentBonuses.java` 約 L69 只查 `RentalItemDefinitionCatalog.source(...)`，整檔沒有引用 `NativeLitoRentalMerchantCatalog`。`NativeLitoRentalMerchantCatalog.findBySourceItemId`（約 L103）與 `rental/retained-lito-rental-merchant-2010.tsv`（含 29047）存在。
- 四張 Issue 建立成功，標題與內文中文正常（讀回 #369 確認），標籤 `needs:claude`。
- Issue 裡列的本機證據檔 2026-11-12 都存在：`驗證紀錄/current/native-world-integration/` 的 8 個 JSON、`native44FE-same-endpoint/…/boundary.json`、`native-current-merchant-bonus-audit/qualified.json`，以及 `target/current/server-reference/` 的兩個候選 JAR。

## 推測

- 租裝城鎮內的 session 是 `Phase.BATTLE`，所以 #368 的攔截在當版仍會發生。依據是提案 README 與 `exitRoomBeforePolyMe` 約 L13177–L13183 的結構，沒有執行重現。
- #370 的觸發時序是「結算返回晚於空的自訂房被移除」。依據是提案 patch 的註解。
- #371 的 78 個商人商品 ID 都不在 `RentalItemDefinitionCatalog`。依據是提案的離線比對紀錄（舊 JAR BBC59308）。
- #369 的可能原因是 handoff 紀錄在新 GAME 接續時已被消耗，之後的 `C_CA` 查不到。

## 待驗證

- 四項都沒有在 E149 或當版 src/main 上執行重現。提案的實測基底是舊 JAR（BBC59308、6928A147、9719F45D）。
- #369：`hasRentalBattleHandoff` 守衛與提案守衛是否等價。2026-10-18 對 75DA69B0 的同端點實測仍失敗，但沒有確認 75DA69B0 是否已含這個守衛。
- #371：E149／當版是否有別的路徑補上商人商品屬性；修好後穿戴與卸除的屬性封包。
- 伺服器主控是否接收這四張。

## 過程中要說明的事

- 為了讀到伺服器最新的 `origin/main`，我對 `D:\lineage開發\3-lito開發` 跑了一次 `git fetch origin`。這只更新該 repo 的遠端追蹤 ref，沒有 checkout、沒有寫工作目錄檔案、沒有開分支。`docs/REUSE-MAP.md` 記的版本是 `291474f2`，現在是 `edfb5c82`。
- 四張 Issue 是用 `scratch/t040/create.py`（python 呼叫 `gh issue create --body-file`）建立的，避開 PowerShell 把中文標題變成「?」。內文草稿在 `scratch/t040/issue-*.md`（scratch 不進 git）。
- Issue 開頭放了伺服器 repo 慣用的 yaml 區塊（`stage`、`runner: claude-local`），但沒有加 `task:queued`，只標 `needs:claude`，由伺服器主控決定排不排。

## 產出路徑

- `docs/reports/issue-299.md`（本檔）
- lito-launcher-server Issue #368、#369、#370、#371

## 給下一棒的提示

- **刪 `修補提案/` 是下一步，條件是伺服器接收**。四張 Issue 都被接收（排入佇列或完成），或主控明確回覆不採用之後，再開一張小任務：`git rm -r 修補提案/`，並更新引用它的地方：`PLAN.md` L53 與 L64（事實 6）、`TASKS.md` 的 T040 狀態、`docs/SKILLS-STATUS.md` L320、`docs/reports/issue-141.md` L10（歷史報告可不改）。
- 刪除前不必再備份：內容在 commit `8a9fa07` 的 git 歷史裡，四張 Issue 都用這個 commit 的固定連結。
- `驗證紀錄/` 與 `target/current/server-reference/` 的候選 JAR 只在本機。伺服器那邊要用的話，請它的本機執行器直接唯讀讀取，不要複製進伺服器 repo。
- 伺服器修完 #368／#369 後，客戶端要回頭處理 T055（G06.3c）與 U-M3-1（同端點）；#371 修完後 G05.4 的 78 項屬性要重核，客戶端不用改碼（只顯示權威值）。
- #368 的修法要和伺服器 #366 加的觀戰者分支合併，兩者改的是同一個 `if`。
