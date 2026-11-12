# Issue #300 報告 — T034：`docs/GUI-ACCEPTANCE.md`（T030 的檢查清單）

執行者：claude-local。分支 `task/300`，從 `64a3796` 開始。本張是 T034 的 1／1。

## 做了什麼
新增 `docs/GUI-ACCEPTANCE.md`：T030 實機驗收用的檢查清單，共 237 項（前置 6 項、操作 231 項），每一項有編號、操作、預期、要核對的面向、結果欄與證據欄。全部項目目前都是未執行。

- §0 填寫規則：結果只有「通過／失敗／待伺服器」，空白是未執行，略過不算通過；五個面向代碼（實體輸入、像素、IME、多 DPI、聲音）；證據命名。
- §1 前置：版本記錄、帳號、第二玩家、裝置；**數值 schema 與登入器的啟動方式**（§1.1）；客戶端啟動入口（§1.2）。
- §2–§18 依 G08 子項與 T034 列的流程分節：
  - 登入與唯一入口（G08.4）17 項；房間（G08.1）24 項；租裝入口與 NPC 對話（G08.2）13 項；市民與徽章（G08.2a）6 項；商人（G08.2b）15 項；旅館（G08.2c）13 項；
  - 傳送 NPC 與 TeleportMe（G08.2d）7 項，owner 退役與傳送恢復 7 項；鍵盤走訪（G08.2e）7 項，Enter 負向 6 項；
  - 雙玩家戰鬥與退出 15 項，新技能與背包 11 項；世界畫面（G08.3，含屋頂進出、開門 O 鍵）19 項；
  - IME 11 項；多 DPI 8 項，焦點與取消 4 項；EOF／取消 11 項；觀戰與小地圖的觀戰 GUI（G07.5）13 項；聲音（G09）12 項；聊天（T060）10 項；數值 schema 負向 2 項。
- §19 列出已知會記「待伺服器」或有前置任務的項目。
- §20 追溯表：T034 在 TASKS.md 的每一條（12 個流程、5 個面向、8 個其他清單項、G08.1／G08.2a–e／G08.3／G08.4／G07.5、數值 schema）各對到哪些項目編號。
- §21 留給 T030 填結論。

查過 `docs/REUSE-MAP.md`：本任務是文件，唯讀參考了 lito-launcher-server `origin/main` `e6b8539e` 的登入器原始碼（`LauncherService.cs`、`LauncherSettings.cs`、`LocalServerService.cs`、`MainForm.cs`）與 `Start-LiTo381-Server.bat`，只用 `git show`／`git grep`／`git ls-tree`，沒有 checkout 或寫入。tw-bin-engine 沒有用到（TW 沒有大廳、房間、租裝）。

## 怎麼驗證
都是靜態核對，沒有執行任何編譯出來的程式，沒有開視窗。

- 控件文字與提示訊息：從 `native_login_window.rs`、`native_rental_window.rs`、`native_startup_window.rs` 抽出字串後填入清單。
- 按鍵：讀 `native_login_window.rs:2821-2960`（Enter、Esc、F1–F3、P、T、O、Space、方向鍵）、`native_skill_window.rs` 的 `hotbar_key` 與其測試（F5–F12）、`native_navigation.rs` 的 `heading`。
- G08、G08.1–G08.4、G07.5、G09 的 work／acceptance／state：讀 `docs/G01-G12.md` 第 3083–3720 行原文。
- T034 點名的四個出處逐行讀過：`NATIVE-ROOM:46`、`NATIVE-TELEPORT:22`、`NATIVE-WORLD-ROOFS:35`、`NATIVE-WORLD-INTEGRATION:123`。
- 清單編號沒有重複；追溯表引用的編號都存在（用腳本數過）。

## 已驗證／推測／待驗證

### 已驗證（靜態）
- **重建版客戶端目前只接受數值 schema 0**：`native_status_receipt.rs:29` 要求 `S_STATUS` 剛好 37 bytes；`native_world.rs:1358-1360` 把 `S_HIT_POINT`／`S_MANA_POINT` 讀成兩個 `h`。
- **登入器啟動的本機伺服器是 schema 4**：`LauncherSettings.cs:54` 預設 `true`，`MainForm.cs:290`、`:296` 載入時設為 `true`，`LocalServerService.cs:228-231` 設 `LITO_NUMERIC_BREAKTHROUGH=4`。伺服器的 `Start-LiTo381-Server.bat` 與 `tools/Start-LiToServer.ps1` 裡沒有這個變數。所以 T030 不能用登入器啟動伺服器，要獨立啟動（清單 §1.1）。
- 原生入口沒有聊天控件：`native_login_window.rs` 搜尋 `chat`／`whisper`／`S_SAY` 沒有結果。聊天 10 項要等 T060。
- 清單裡引用的控件文字、提示訊息、按鍵都在 commit `64a3796` 的原始碼裡。

### 推測
- 租裝村 NPC 名稱（阿里亞、席巴、瑪雅、菲佛、克里、勒夫、四位傳送師）取自 `tests/fixtures/native-gui/*.json`，那些 fixture 綁的是較舊的 JAR `BBC59308`；E149 是否同名沒有核對。清單已註明以畫面為準。
- 第四位商人是「飾品商人-勒夫」：只看到 `citizen-routes.json` 有這個名稱，`merchant-plan.json` 前段只確認到瑪雅、菲佛、克里。
- 證據檔的存放位置（截圖進 `docs/screenshots/t030/`、錄影放 `驗證紀錄/`）是建議，T030 執行時可以調整。
- 部分預期行為依文件描述寫成（例如被踢後不能再進、設定變更重置準備、取消後 33 秒逾期要重新選角），來源是 `NATIVE-*` 文件記錄的舊版 controller 證據，不是當版 GUI 的結果。

### 待驗證
- 清單的 237 項全部未執行，要由 T030 在真機做。
- 連到 schema 4 伺服器時，客戶端畫面實際顯示什麼（N02）。
- E149 部署包的伺服器啟動入口檔名是否與 `origin/main` 的 `Start-LiTo381-Server.bat` 相同（lito-launcher-server 的 CLAUDE.md 寫的是 `1.啟動伺服器.bat`）。
- 凡是預期欄寫「記下實際…」的項目，表示現有文件與原始碼不足以決定預期值，要在 T030 觀察後補回清單。
- CI：本張只新增兩個 Markdown 檔，沒有改程式；push 後的 CI 結果還沒看到。

## 沒有做的事
- 沒有改 `docs/REPORT.md`、`TASKS.md`、`PLAN.md`。
- 沒有執行客戶端、測試或任何編譯產物；沒有連伺服器或 DB。
- 沒有宣稱缺資源，所以沒有用 PakViewer 查 pak。

## 產出路徑
- `docs/GUI-ACCEPTANCE.md`
- `docs/reports/issue-300.md`

## 給下一棒的提示
1. **需要決定：數值 schema。** 客戶端只支援 schema 0，而登入器帶起來的伺服器固定是 schema 4。清單用「獨立啟動伺服器」繞過，T030 可以照做。但是廠商實際環境如果是用登入器啟動伺服器，重建版客戶端進不了世界。`TASKS.md` 目前沒有「支援 schema 1–4」的任務（搜尋 `schema` 的四處都不是這件事）。建議主控對話開一張 Issue 讓使用者決定：要補客戶端對 37／48／66／78 與 5／9 bytes 的解析，還是驗收與交付都限定 schema 0。這會影響 M3 的驗收條件，屬於 `needs:user`。
2. T030 開始前，這些前置任務合併後要回頭更新清單的控件名稱與預期：T060（聊天，§17 的控件名稱現在是空的）、T049（通知與系統訊息文字）、T035（版面缺陷）、T058（屋頂）、T059（音效裝置）、T031（可攜包入口）。
3. 清單的控件文字綁 commit `64a3796`。之後改了 `native_login_window.rs` 或 `native_rental_window.rs` 的按鈕文字，請同步改清單。
4. T037（G01–G12 稽核）可以直接引用 §20 追溯表，把 G08.1–G08.4、G07.5、G09 對到項目編號。
5. H11、H12（3600／7200 秒自然租期）與 M11–M13（商品到期）各要等一到兩小時，T030 排程時建議先開始計時，再做其他項目。
