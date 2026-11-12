# Issue #152 — T022 批 2 技能 110–118

## 做了什麼

本張把 110–118 九個技能的請求、選單與控制器接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 的 110–118 列，射程與 reuse 依 E149 catalog。分支 `task/152`，從 `ce85b82` 開始，之後合併了 `origin/main`（`617fbbc`）。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 |
|---|---|---|---|---|---|
| 110 敏捷提升 | 13／5 | ① | 3 bytes | 自施 | 0 |
| 111 閃避提升 | 13／6 | ① | 3 bytes | 自施 | 0 |
| 112 破壞盔甲 | 13／7 | ① | 3 bytes | 不送目標，一格內目標由伺服器 `resolveArmorBreakTarget` 決定 | 2000 |
| 113 精準目標 | 14／0 | ⑪ | 11 bytes＋文字＋NUL | 目標存活、可見、在畫面菱形內；未選目標時對自己（伺服器候選包含自己） | 0 |
| 114 激勵士氣 | 14／1 | ① | 3 bytes | 自施 | 0 |
| 115 鋼鐵士氣 | 14／2 | ① | 3 bytes | 自施 | 0 |
| 116 呼喚盟友 | 14／3 | ⑦ | 3 bytes＋名稱＋NUL | 不接受物件目標；名稱 1..27 bytes、前後無空白 | 0 |
| 117 衝擊士氣 | 14／4 | ① | 3 bytes | 自施 | 0 |
| 118 援護盟友 | 14／5 | ⑧ | 3 bytes＋名稱＋NUL | 同 116 | 0 |

- **`native_skills.rs`**
  - `selector_only` 加入 110、111、112、114、115、117。
  - 新增 `text_skill`（113／116／118）與 `text_request`。文字用既有的 `native_login::encode_text` 精確轉成 MS950，不做 best-fit，控制字元一律拒絕。長度上限照伺服器 parser：113 是 `0x34`，116／118 是 `0x1B`（Protocol.java `trueTargetRequest`／`callClanSpellRequest`／`runClanSpellRequest`）。113 的文字可以留空，只送一個 NUL。
  - `request()` 新增 `id if text_skill(id)` 分支，不帶文字時轉給 `text_request(…, "")`：113 送空白標記，116／118 會要求輸入名稱。
  - `admit_named` 讓 116／118 在還沒輸入名稱時就能通過選單 gate；送出時 `text_request` 會再全部重核一次。
  - reuse 依 catalog：112 是 2000，其他 0。
  - 新增 `Pending::observe_clan` 與 `Feedback` 文案 73／414／629／647／1192。
- **`native_world.rs`**：新增 `last_clan_skill_rejection`，記錄附序號的 S10 73／414／629／647／1192。只有 116／118 施法時會讀這個欄位，一般施法與物品施法不看（比照 #151 處理 79 的方式）。這幾個碼不放進共用的 `last_skill_rejection`，避免其他功能送出的 73 被當成施法拒絕。
- **`native_login.rs`**：新增 `LoginController::cast_text`。內部的 `cast_request_full` 改為呼叫新的 `cast_request_text`（多一個 `text` 參數），所以既有呼叫點與測試都不用改。帶文字的施法不走「移動中施法」的捷徑，也不能和地面、物品、傳送、召喚目標混用。新增 `skill_clan_pending` 決定要用 `observe_clan`。
- **`native_skill_selection.rs`**：116／118 一律清掉 hover 目標，改走 `admit_named`；選單加上 110–118 的中文標籤。
- **`native_login_window.rs`**：新增 EDIT 控制項 `SKILL_TEXT`（id 230；217–224 已被租裝視窗使用，所以不用）。只有在技能書選了 113／116／118 時才顯示。施放時只有「目前選取的就是這個技能」才讀這個欄位；從快捷列施放其他文字技能時不沿用，113 會送空白標記，116／118 會出現「請輸入盟友名稱」。
- **coverage parser**（`維護工具/audit-native-E149-skill-catalog.py`）：`source_ids` 會辨識 `text_skill`，並輸出 `text_request_ids`。如果 helper 沒接進 `request`、或沒有 `text_request`，或語法認不出來，就直接拒絕，不產出錯誤統計。
- **測試**：新增 `tests/native_skills_110_118.rs`，共 7 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF）。`維護工具/test_native_skill_coverage.py` 加 2 案，合計 6 案。

### 同步 main

合併 `origin/main`（`617fbbc`）時有兩處衝突：
- `native_skills.rs` 的 `reuse_millis`：保留 main 的 `68=>1500`，也保留本張的 110–118。
- `native_login.rs` 的 `interrupt_world_actions`：保留 main 的 `skill_target_pending=None`，也保留本張的 `skill_clan_pending=false`。

第二次合併 `origin/main`（`ed9a9d8`，PR #284 顯示衝突後）有一處衝突：
- `native_login.rs` 的 `observe_skill`：採用 main 的 `observed`＋`teleport_released`（→ `TeleportReleased`）結構，並在 `observed` 的判斷鏈保留本張的 `skill_clan_pending → observe_clan` 分支。

main 已經把 73 加進 `item_targeted`。那裡的 73 是技能 ID，本張的 73 是 S10 訊息碼，兩者無關。

## 已驗證

### 來源（唯讀）

- 形狀與 row／column 依 `docs/PROTOCOL-CONTRACT.md` 的 §6.3 對照表（第 415–423 行）。文字規則依 §1 與 `trueTargetRequest` 的說明。
- 伺服器原始碼（上層 repo，唯讀）：
  - `lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/Protocol.java`：9203–9295 是 116／118 名稱 parser（1..0x1B bytes、trim、無控制字元、可精確編碼、沒有剩餘 bytes），9387–9414 是 113 parser（≤0x34、可以是空字串）。
  - `ClientSession.java`：44864 起是 `resolveTrueTargetCandidate`（候選包含自己），44983 起是 `useTrueTargetSpell`（伺服器重新查目標、檢查畫面與視線，不採用 client 座標），45322／45754 起是 `useCallClanSpell`／`useRunClanSpell`（拒絕碼 73／414／629／647／1192／280），另外 `useArmorBreakSpell`（伺服器自己選目標）、`useRoyalAuraSpell`（targetTo 8＝隊伍、4＝血盟）。
  - `BattleTrueTarget.java`：`resolve` 依物件 ID 查目標，ID 重複時拒絕。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 為 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #151 相同）。110–118 的 row／column、reuseDelayMillis（112＝2000、其他 0）、maximumRange（112＝1、113＝-1）都依此表。

### 命令與結果（第二次合併 main `ed9a9d8` 之後重跑）

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令都在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile；只有既存警告）

cargo test --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --test native_skills_110_118 --test native_skills_101_109 --test native_selector_only --test native_skill_selection --no-run
結束碼 0（四個測試執行檔都編譯成功，未執行）

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 6 tests in 0.060s；OK
```

第一次合併（`617fbbc`）前後也各跑過同樣的 `cargo check`（結束碼 0）與 `--no-run`（結束碼 0）。

## 推測

- 原版客戶端在 113 未輸入標記時，是否也送空字串，本張沒有反組譯 `sub_4CADF0` 確認。伺服器 parser 接受 0 bytes，所以本張預設送空字串。
- 647／1192 的中文文案是依伺服器 reason（`source-not-escapable`、`destination-map-instance-or-war-area`）寫的，沒有對照客戶端訊息表原文。
- 原版 116／118 的名稱輸入 UI 樣式未核對。本張用一般 EDIT 控制項代替，只保證送包內容與 parser 一致。

## 待驗證與未驗證什麼

- 新的 7 案 `native_skills_110_118` 與既有回歸的 `test result:`，以 PR 最新 commit 的 CI 為準。本機沒有執行，在 CI 結果出來前不算通過。
- 控制器的 `cast_text`（含 `skill_clan_pending` → `observe_clan`）只在 World／Pending 層測過，沒有控制器整合測試，因為需要 `Frame`。
- `SKILL_TEXT` 控制項的顯示、版面（和召喚／傳送下拉選單在同一位置，三者同時只會顯示一個）、中文輸入與 IME，都沒有開 GUI 確認，交給 Codex／使用者驗收。
- 沒有跑正式伺服器施法，也沒驗證盟友是否真的被拉過來／傳送過去、精準目標標記（S_COMMAND_TARGET）的接收與畫面、士氣光環的隊伍／血盟範圍，以及 112 的 HP 消耗與材料（`docs/SKILLS-STATUS.md` 標「伺服器待定：材料 40321×2」）。
- 本張沒有接 110–118 的特效／音效資源與渲染（catalog effectGfx 3910、11766、3400、1765、2277、3941、3942）。S_COMMAND_TARGET 的收包也還沒實作。
- `docs/SKILLS-STATUS.md` 的「客戶端」欄沒有更新（批 1 也沒改），留給 T026 統一同步。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_login.rs`
- `LiTo381-Modern/src/modern_lito/native_login_window.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`
- `LiTo381-Modern/tests/native_skills_110_118.rs`
- `維護工具/audit-native-E149-skill-catalog.py`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-152.md`

## 給下一棒的提示

- Claude 審查重點：
  - 113 沒選目標時預設對自己，這點是否可以接受。
  - 116／118 的拒絕碼為什麼要獨立成 `last_clan_skill_rejection`。
  - 快捷列施放文字技能時不讀輸入框的政策。
- CI 綠燈後由使用者合併。
- 後續批次：
  - 純 selector 技能直接加進 `selector_only`。
  - 21 個 wrapper 技能（183、184、187、188、192–194、202–204、207–209、211–214、216–219）一律走 11 bytes 的 ⑩，不屬於 `text_skill`。
  - helper 有變動時，要同步跑 `python -B 維護工具/test_native_skill_coverage.py`。
- 之後若要接 S_COMMAND_TARGET（113 的結果顯示），伺服器 builder 是 `Protocol.commandTarget(objectId, x, y, text)`。
