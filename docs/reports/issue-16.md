# Issue #16 / T004-4：C_USE_SPELL 技能 (row,column) → 尾段形狀對照表

## 做了什麼

- 在 `docs/PROTOCOL-CONTRACT.md` 新增 **§6.3**。內容包括：判定順序（外層 wrapper → TELEPORT → 依記錄類型與實例欄位）、範圍與限制、模式差異、產生結果摘要，以及 **181 列**對照表。每列有 skillId、常數名、row、column、記錄類型、尾段形狀，以及判定依據（handler 方法加 `CS:` 行號，或 wrapper 名稱）。
- 新增 `維護工具/generate-spell-request-shapes.py`。它只讀 lito-launcher-server 來源，從四個檔案靜態產生整張表，不是手抄：
  - `BattleSpellCatalog.java`
  - `ClientSession.java`
  - `IllusionSkillRequests.java`
  - `DragonSkillRequests.java`
- 只改了契約中與本任務直接相關的兩處交叉引用：§6.2「尾段形狀」列的最後一句，以及 §8「未涵蓋」中 T004-4 的指向，兩處都改成指向 §6.3。Issue 範圍已同步更新。
- 沒有修改伺服器 repo。

### 過程中確認的重點（來源）

- **外層 wrapper 會改變線上格式**：`useSpell` CS:32161 會先經過 `DragonSkillRequests.dispatch` 和 `IllusionSkillRequests.dispatch`。這兩層攔下的 21 個技能，線上必須送 11 bytes 的 ⑩ 形狀；伺服器把封包截成 3 bytes 後才交給內層 handler。所以只看內層 handler 會得到錯誤的格式結論。
- **skillId 與 row/column 一致**：`BattleSpellCatalog` 共有 180 筆技能實例，全部滿足 `skillId = row*8 + column + 1`（`Protocol.SpellSelector.skillId` P:4055）。每個技能只屬於一個記錄類型（包裝記錄 `new ConfusionSpell(new AttackSpell(...))` 這類，只計入內層實例）。
- **多形狀類型依實例欄位判定**：一個記錄類型有多種形狀時，依該筆實例的欄位判定，條件逐一讀過 dispatcher 原碼：
  - Shield：`targeting`（CS:34349 起）
  - AbilityBuff、Invisibility：`requestShape`（CS:39586、45996 起）
  - Freeze：CS:33607–33617
  - Area：CS:33683–33697
  - Attack：CS:33698–33717

## 怎麼驗證

本輪在 Linux 雲端工作環境實際執行。伺服器 clone 位於 `/home/user/lito-launcher-server`，HEAD 為 `2066774824b314b2c65834f9afe72b68f7391f06`。

1. 產生與完整性檢查：只要有任何技能無法判定、skillId 與 row/column 不一致，或來源 HEAD 不符，腳本就以非零結束碼結束。

   ```
   $ python3 -I 維護工具/generate-spell-request-shapes.py /home/user/lito-launcher-server; echo "exit=$?"
   （第 2 次嘗試輸出）
   source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; skills=181 unresolved=0 wrapper-not-in-catalog=[]
   shapes: ① 無尾段=70; ② d(clientTick)=1; ③ h(x) h(y)=2; ④ d(target)=35; ⑤ h(map) h(x) h(y)=1; ⑥ h(map) h(x) h(y)=1; ⑦ s(name)=1; ⑧ s(name)=1; ⑨ d(selection)=1; ⑩ d(target) h(x) h(y)=67; ⑪ d(target) h(x) h(y) s(text)=1
   source=git:2066774; mode check per skill: BATTLE／WORLD 不同 handler、逐筆比對形狀相同=54; wrapper（兩種模式都先經過外層）=21; 兩種模式共用同一 handler=106
   exit=0
   ```

   181 = catalog 的 180 個技能加上 TELEPORT（5）。TELEPORT 由 dispatcher 特別處理，不在 catalog 中。

2. 表格內容由 `--markdown` 參數直接輸出後貼入 §6.3。重產指令：`python3 -I 維護工具/generate-spell-request-shapes.py <clone> --markdown`。

3. 與重建版既有驗證交叉核對（歷史文件證據，不是本輪重跑）。每項標的是該文件自己記載的 JAR 版本，完整表格見 §6.3：

   | 技能 | 文件 | JAR 版本 | §6.3 |
   |---|---|---|---|
   | 治癒 1 | NATIVE-SKILLS | `589733435518D92C…67A74AA7` | ④ |
   | 護盾 3 | NATIVE-SKILLS | `589733435518D92C…67A74AA7` | ① |
   | 火牆 58 | NATIVE-GROUND-SKILLS | `589733435518D92C…67A74AA7` | ③ |
   | 風刃 7 | NATIVE-WIND-CUTTER | `BBC59308C43DF89E…2DD5DCE43` | ⑩ |
   | 光箭 4 | NATIVE-ATTACK-SKILLS | 只寫「固定正式 JAR」，**版本待核對** | ⑩ |
   | 冰箭 6 | NATIVE-ATTACK-SKILLS | 同上，**版本待核對** | ⑩ |
   | 生命之泉 63 | NATIVE-REAGENT-SKILLS | 只寫「現行 JAR」，**版本待核對** | ③ |

   有文件寫明封包格式的 1、3、4、58、63 五項都一致；6、7 的文件只記載「已接入」，沒有封包格式。第 1 次嘗試時把這些證據一律歸給 205DC1BA，這是錯的，已改正。

4. 既有引用核對沒有退步：`python3 -I 維護工具/check-protocol-contract-citations.py /home/user/lito-launcher-server` 的輸出是 `140/140 citations OK; spellSelector(payload) parsers=11`，exit=0。

### 驗收對照

- **重建版已接線的技能都有對照列**：NATIVE-* 文件提到的技能是 1、3、4、6（冰箭，NATIVE-ATTACK-SKILLS:38）、7、58、63、131，全部都在表內。文件中另外出現的 221 不是 catalog 的技能 ID，搜尋整個 `BattleSpellCatalog` 也沒有值為 221 的常數。
- **同一個 (row,column) 若因模式不同而改變形狀，要分開列出**：第 2 次嘗試改成**逐技能**比對 BATTLE 與 WORLD 兩條路徑的 handler 和 parser。結果：21 個 wrapper 技能、54 個不同 handler 但形狀相同、106 個共用同一 handler，共 181 個，0 筆形狀不同，所以不需要分列。第 1 次嘗試的「22 類比對」其實跳過了 3 類條件分支（審查第 3 點指出），已整段改寫。

### 第 2 次嘗試的修正（對應 codex-local 審查的 3 點）

1. **每列都有 handler 與 `檔案:行號`**：
   - Shield、AbilityBuff、Invisibility、Freeze、Area、Attack：補上實際使用的 handler 方法名和行號，並附條件分支的行號。
   - wrapper 的 21 列：列出外層的 `dispatch` 方法、長度 11 檢查、parser 呼叫、截 3 bytes 轉交這四處行號，以及內層 handler。
   - TELEPORT：列出 `useTeleportSpell` 的行號。
   - 腳本只要有任何一列缺 handler 行號，就會列為 UNRESOLVED。
2. **歷史證據逐文件標出版本**，並補上冰箭 6。
3. **WORLD 改為逐技能比對**，見上方的驗收對照。另外，腳本改成優先用 `git show 2066774:<path>` 讀伺服器來源，所以伺服器 checkout 已經移到其他版本時，仍可以直接重現同一結果（審查者第一次重現時就遇到這個情況）。

## 未驗證什麼

- 判定是靜態推導：腳本以 handler 方法簽章中的 `BattleSpellCatalog.<Record>` 參數，對應到該方法直接呼叫的 parser。如果某個 handler 透過不帶記錄類型的 helper 再解析一次，會被漏掉。目前沒有發現這種情況，但不能完全排除。
- 沒有對正式 JAR e1498871 核對，這部分屬於 T004-1（#13）。
- 沒有逐一核對技能的可用條件，例如學會與否、MP、射程、冷卻、WORLD 白名單；本表只是格式契約。
- 沒有跑測試、沒有建置、沒有啟動遊戲或伺服器。

## 產出路徑

- `docs/PROTOCOL-CONTRACT.md`（新增 §6.3，另改兩處交叉引用）
- `維護工具/generate-spell-request-shapes.py`
- `docs/reports/issue-16.md`（本檔）

## 給下一棒的提示

- T004-2（#14）的 Rust 端可以直接依 §6.3 實作「技能 → 尾段形狀」查表。wrapper 那 21 個技能一定要送 11 bytes。
- 伺服器版本更新時，重跑這支腳本和 `check-protocol-contract-citations.py`；HEAD 不符時兩支腳本都會失敗，先更新 `EXPECTED_HEAD` 再重產表格。
