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
   source HEAD 2066774824b314b2c65834f9afe72b68f7391f06; skills=181 unresolved=0 wrapper-not-in-catalog=[]
   shapes: ① 無尾段=70; ② d(clientTick)=1; ③ h(x) h(y)=2; ④ d(target)=35; ⑤ h(map) h(x) h(y)=1; ⑥ h(map) h(x) h(y)=1; ⑦ s(name)=1; ⑧ s(name)=1; ⑨ d(selection)=1; ⑩ d(target) h(x) h(y)=67; ⑪ d(target) h(x) h(y) s(text)=1
   world record types with classic-world handlers=22; world-shape-differs=0
   exit=0
   ```

   181 = catalog 的 180 個技能加上 TELEPORT（5）。TELEPORT 由 dispatcher 特別處理，不在 catalog 中。

2. 表格內容由 `--markdown` 參數直接輸出後貼入 §6.3。重產指令：`python3 -I 維護工具/generate-spell-request-shapes.py <clone> --markdown`。

3. 與重建版既有驗證交叉核對（歷史文件證據，對 JAR 205DC1BA，不是本輪重跑）：

   | 技能 | 既有文件記載 | 文件 | §6.3 |
   |---|---|---|---|
   | 光箭 4 | C14 帶目標物件 ID 及目前座標 | NATIVE-ATTACK-SKILLS | ⑩ |
   | 治癒 1 | 帶物件 ID | NATIVE-SKILLS | ④ |
   | 護盾 3 | 只帶 selector | NATIVE-SKILLS | ① |
   | 火牆 58 | `14 07 01 x:u16 y:u16` | NATIVE-GROUND-SKILLS | ③ |
   | 生命之泉 63 | row7／column6 加 u16 座標 | NATIVE-REAGENT-SKILLS | ③ |

   五項都一致。

4. 既有引用核對沒有退步：`python3 -I 維護工具/check-protocol-contract-citations.py /home/user/lito-launcher-server` 的輸出是 `140/140 citations OK; spellSelector(payload) parsers=11`，exit=0。

### 驗收對照

- **重建版已接線的技能都有對照列**：NATIVE-* 文件以 ID 提到的技能是 1、3、4、7、58、63、131，全部都在表內。文件中另外出現的 221 不是 catalog 的技能 ID，搜尋整個 `BattleSpellCatalog` 也沒有值為 221 的常數。
- **同一個 (row,column) 若因模式不同而改變形狀，要分開列出**：腳本比對了 22 個有 classic-world handler 的記錄類型，結果是 0 個差異，所以不需要分列。比對方法見 §6.3「模式差異」。

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
