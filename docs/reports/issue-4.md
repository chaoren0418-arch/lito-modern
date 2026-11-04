# Issue #4 / T004 — 協定契約 docs/PROTOCOL-CONTRACT.md

## 做了什麼

- 新建 `docs/PROTOCOL-CONTRACT.md`，內容分九節：
  - §0 適用範圍與證據等級
  - §1 基本型別
  - §2 框架與加密
  - §3 端點與 phase
  - §4 握手、LOGIN／GAME 登入、錯誤碼
  - §5 大廳、頻道、房間
  - §6 戰鬥
  - §7 退出與重新開始
  - §8 未涵蓋與待驗證
- 比對來源：`chaoren0418-arch/lito-launcher-server` main @ `2066774` 的 `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/**`。全程唯讀，主要對照 `Protocol.java`、`ClientSession.java`、`Main.java`、`ServerConfig.java`、`UnifiedConfiguration.java`、`NativeRoomDirectoryRow.java`、`NativeMovement.java`。
- 每列都標了伺服器類別或方法，以及 `檔案:行號`，證據分成三級：
  - 已驗證：對重建版既有 oracle 的 JAR `205DC1BA` 核對過；
  - 來源：從原始碼直接讀出；
  - 推測：從命名或呼叫關係推論，沒讀到完整的 builder 或 parser。
- 新建 `docs/plans/issue-4.json`，列出四個後續任務：
  - T004-1（resource）：用 javap 對正式 JAR `e1498871` 做靜態核對；
  - T004-2（code）：在 Rust 端集中 opcode 常數；
  - T004-3（plan）：NATIVE-* 文件改為指向契約；
  - T004-4（design）：C_USE_SPELL 每個技能的 (row,column) → 尾段形狀對照表。

## 怎麼驗證

### 本輪實際執行（第 2 次嘗試，2026-11-04）

在 Linux 雲端工作環境執行。伺服器 clone 為 `git clone --depth 1 https://github.com/chaoren0418-arch/lito-launcher-server /home/user/lito-launcher-server`，HEAD `2066774824b314b2c65834f9afe72b68f7391f06`。

1. 引用行號核對（腳本已加入 repo：`維護工具/check-protocol-contract-citations.py`，只讀）：

   ```bash
   python3 -I 維護工具/check-protocol-contract-citations.py /home/user/lito-launcher-server
   ```

   實際輸出：

   ```
   source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; 122/122 citations OK
   ```

   腳本逐條檢查「`檔案:行號` 這一行是否含有預期字串」，涵蓋 §1–§7 的主要引用：Protocol／ClientSession／GameStateStore／NumericBreakthrough／ServerConfig／UnifiedConfiguration／Main，以及登入器的三個 `.cs` 檔。§5–§6 部分 handler 的內部行號（例如 CS:9890、CS:26701）不在清單內，這些仍是讀碼結果，沒有機器核對。

2. C_USE_SPELL parser 呼叫點計數：

   ```bash
   cd lito伺服器/LiTo381-Server/src/main/java/local/lito381/server
   for f in selectorOnlySpellRequest objectSpellRequest attackSpellRequest; do printf '%s=%s ' $f $(grep -c "Protocol.$f(" ClientSession.java); done
   ```

   實際輸出：`selectorOnlySpellRequest=71 objectSpellRequest=33 attackSpellRequest=44`

3. plans 格式：`python3 -m json.tool docs/plans/issue-4.json > /dev/null`，結束碼 0，共 4 筆。

### 第 1 次嘗試的聲明（只是過程紀錄，不算可追溯證據）

第 1 次寫的「用 `grep -n -F` 逐條核對並修正偏差」、「抽查 5 處子代理結果」沒有保存原始輸出。它們已被上面第 1 項的腳本與輸出取代，不另外宣稱已驗證。

### 歷史文件證據（不是本輪執行）

§2、§4 標「已驗證」的列，證據是重建版既有的 `docs/NATIVE-PROTOCOL.zh-TW.md`、`NATIVE-LOGIN`、`NATIVE-GAME`，比對對象是 JAR `205DC1BA`。本輪沒有重跑這些驗證，也沒有把它們當成對 0.6.71 的驗證。

### 第 2 次嘗試的修正（對應 codex-local 審查的 6 點）

1. `S_LITO_ROOM_USER`：寫明 certificate×12、badge×5 的長度、值域與意義，引用 `GameStateStore.TournamentAwards`。
2. `S_STATUS`：寫明能力順序 STR/INT/WIS/DEX/CON/CHA、抗性順序 FIRE/WATER/WIND/EARTH、有號／無號，以及 schema 0／1／2–3／4 的 body 長度 37／48／66／78。新增「數值 schema」一列：schema 由環境變數 `LITO_NUMERIC_BREAKTHROUGH` 決定、正式登入器帶 4、wire 上沒有協商、client 依長度判斷。
3. 新增 `S_ADD_INVENTORY_BATCH 0x7C`、`S_TIME 0x21`、`S_LITO_VS_SERVER_ALLOW_JOIN 0xF5`；`C_USE_SPELL` 改成列出尾段形狀與 parser（第 3 次嘗試時補成 11 種）。技能和形狀的逐一對照拆給 T004-4，Issue 驗收範圍已同步調整。
4. §1 文字規則改寫：`Writer.s` 本身不做檢查；分別寫出帳密、名稱類、公告／物品名、告示板四類欄位的規則；寫明公告允許 CR/LF/Tab。
5. 本節：補上可重跑的指令、來源 commit 與實際輸出，並把歷史聲明和本輪驗證分開。
6. `docs/plans/issue-4.json` 是調度器規定的 stage:design 產出（調度器會依它建立後續 Issue）。Issue 範圍已明列 plans、reports 與核對腳本。

### 第 3 次嘗試的修正（對應 codex-local 第 2 輪審查的 2 點）

1. `C_USE_SPELL` 補上第 ⑪ 種形狀 `trueTargetRequest`（P:9389，skillId 113，`d h h s(text ≤0x34)`）。核對方式：`grep -n "spellSelector(payload)" Protocol.java` 命中 11 處，正好就是契約列出的 11 個 parser，見下方輸出。
2. §6.1 開戰序列改成依 `startBattle` 實際的送出順序：WORLD → START → OBSERVER_BOX → [TIMELIMIT，TIME_ATTACK 不送] → TIME → [背包重建]。補上 `S_LITO_OBSERVER_BOX 0xE2` 的欄位（P:14785），也把租裝城鎮的序列分開列出。
   - 背包初始化時機和審查意見不同，以來源為準：`sendInventoryBootstrap` 有一次性旗標（CS:9734），**第一次**是在選角成功的 S_C8 之後送（CS:8886）；startBattle 的呼叫（CS:12034）只有旗標被重設過才會重送（重設點在 CS:9723、20272、20413、20483）。契約照這個寫：client 在選角後和開戰時都必須能接收背包批次。
3. 核對腳本改成：有任何一條失敗，或來源 HEAD 不是預期 commit，就以非零結束碼結束（審查的非阻擋建議）。另外新增本輪引用的檢查項。第 3 次嘗試的實際輸出：

   ```
   $ python3 -I 維護工具/check-protocol-contract-citations.py /home/user/lito-launcher-server; echo "exit=$?"
   source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; 140/140 citations OK; spellSelector(payload) parsers=11
   exit=0
   ```

## 未驗證什麼

- 全文都還沒有對正式 JAR `e1498871` 核對（交給 T004-1）。正式 JAR 是用 `tools/*/build.py` 對 `base-0.6.70.jar` 打 patch 建出來的，`src/main` 不一定等於正式 JAR。
- 正式 JAR 的 `NumericBreakthrough` 是否與來源一致（交給 T004-1）。
- 交易、血盟、商店、觀戰、配對、經典 WORLD、`S_ATTACK` 的變體都沒寫；技能與尾段形狀的對照交給 T004-4。
- 未跑測試、未建置、未啟動遊戲或伺服器、未連 DB（測試仍暫緩）。

## 產出路徑

- `docs/PROTOCOL-CONTRACT.md`
- `docs/plans/issue-4.json`
- `docs/reports/issue-4.md`（本檔）
- `維護工具/check-protocol-contract-citations.py`（引用行號核對，只讀伺服器來源）

## 給下一棒的提示

- T004-1 必須在本機讀正式 JAR，交給 `codex-local`，只做靜態讀取。發現不一致時，交回 Claude 修訂契約。
- T004-2 交給 Cursor：常數名稱沿用伺服器的 `Protocol.java`。同一個值在 C 和 S 方向意義不同，不可以共用常數。
- 本任務沒有改 `docs/REPORT.md`，索引由調度器負責。
