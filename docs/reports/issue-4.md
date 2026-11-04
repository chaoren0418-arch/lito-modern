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
- 新建 `docs/plans/issue-4.json`，列出三個後續任務：
  - T004-1（resource）：用 javap 對正式 JAR `e1498871` 做靜態核對；
  - T004-2（code）：在 Rust 端集中 opcode 常數；
  - T004-3（plan）：NATIVE-* 文件改為指向契約。

## 怎麼驗證

- §1–§4 的行號，用 `grep -n -F` 逐條對照伺服器來源核對過一次，並修正了初稿的偏差。
- §5–§7 的欄位由唯讀子代理整理。我抽查了五處，都與來源一致：`moveRequest`、`battleResult`、`LitoCharacterSelectResult`、`joinRoom` 的 Reader、`litoEnterChannelRequest`／`battleBootstrap`。
- §2、§4 標「已驗證」的列，證據是重建版既有的 `docs/NATIVE-PROTOCOL.zh-TW.md`、`NATIVE-LOGIN`、`NATIVE-GAME`，比對對象是 JAR `205DC1BA`。這些不是本輪重跑的結果。
- `docs/plans/issue-4.json` 用 `python3 -m json.tool` 檢查過格式。

## 未驗證什麼

- 全文都還沒有對正式 JAR `e1498871` 核對（交給 T004-1）。正式 JAR 是用 `tools/*/build.py` 對 `base-0.6.70.jar` 打 patch 建出來的，`src/main` 不一定等於正式 JAR。
- `NumericBreakthrough`、`levelAbility`、`elementalResistance` 這三個開關在正式設定下是否開啟還沒確認，它們會改變 `S_HIT_POINT`、`S_MANA_POINT`、`S_STATUS` 的長度。
- `S_LITO_ROOM_USER` 尾段 cert／badge 的長度只是推測。
- 交易、血盟、商店、觀戰、配對、經典 WORLD，以及各技能與攻擊變體的尾段都沒寫。
- 未跑測試、未建置、未啟動遊戲或伺服器、未連 DB（測試仍暫緩）。

## 產出路徑

- `docs/PROTOCOL-CONTRACT.md`
- `docs/plans/issue-4.json`
- `docs/reports/issue-4.md`（本檔）

## 給下一棒的提示

- T004-1 必須在本機讀正式 JAR，交給 `codex-local`，只做靜態讀取。發現不一致時，交回 Claude 修訂契約。
- T004-2 交給 Cursor：常數名稱沿用伺服器的 `Protocol.java`。同一個值在 C 和 S 方向意義不同，不可以共用常數。
- 本任務沒有改 `docs/REPORT.md`，索引由調度器負責。
