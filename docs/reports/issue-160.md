# Issue #160 — T022 批 10 技能 211–220

## 做了什麼

本張把 211–220 十個幻術師技能的 `C_USE_SPELL` 請求、選單標籤、reuse 與施法時鐘接上。封包形狀依 `docs/PROTOCOL-CONTRACT.md` §6.3 第 489–498 行；目標規則與射程依伺服器原始碼；reuse 與施法時鐘依 E149 catalog，施法時鐘另外對過原版 PE。分支 `task/160`，從 `8a9fa07` 開始。

| 技能 | row／column | 形狀 | body | 客戶端檢查 | reuse 毫秒 | 施法時鐘毫秒 |
|---|---|---|---|---|---|---|
| 211 耐力 | 26／2 | ⑩ | 11 bytes | 沒選目標就送自己（含自己的座標）；選了目標就送該目標，須存活、可見、三格內 | 1000 | 16 |
| 212 幻想 | 26／3 | ⑩ | 11 bytes | 必須選目標；不可是自己；存活、可見；三格內 | 3000 | 640 |
| 213 武器破壞者 | 26／4 | ⑩ | 11 bytes | 同 212 | 1200 | 32 |
| 214 幻覺：鑽石高崙 | 26／5 | ⑩ | 11 bytes | 同 211 | 2000 | 0 |
| 215 立方：衝擊 | 26／6 | ① | 3 bytes | 不送目標 | 5000 | 0 |
| 216 洞察 | 26／7 | ⑩ | 11 bytes | 同 211 | 1000 | 3600 |
| 217 恐慌 | 27／0 | ⑩ | 11 bytes | 同 212 | 4000 | 0 |
| 218 疼痛的歡愉 | 27／1 | ⑩ | 11 bytes | 同 212 | 3000 | 0 |
| 219 幻覺：化身 | 27／2 | ⑩ | 11 bytes | 同 211 | 5000 | 0 |
| 220 立方：和諧 | 27／3 | ① | 3 bytes | 不送目標 | 5000 | 32 |

211–214、216–219 是 TASKS.md 說的 wrapper 技能：伺服器 `IllusionSkillRequests.dispatch` 只收 11 bytes，其他長度直接丟棄，所以這八個一律送 ⑩ 形狀。

- **`native_skills.rs`**
  - `selector_only` 加入 215、220。
  - `request` 新增兩個分支，放在 `_ =>` 之前：
    - `211 | 214 | 216 | 219`：目標或自己，三格內。
    - `212 | 213 | 217 | 218`：必須選他人，三格內。
  - 新增 `illusion_targeted`（211–214、216–219）與 `illusion_hostile`（212、213、217、218）。目前只有測試用它們確認分類。
  - `reuse_millis`、`client_clock_millis` 補上表中的值。
  - `Feedback::label` 加上 1412：「此處無法設置立方，或附近已有立方」。
- **`native_world.rs`**：`last_skill_rejection` 認得的訊息碼加上 1412（立方放置失敗）。沒加的話，立方放不下去時客戶端只會顯示「未收到施法回覆」。
- **`native_skill_selection.rs`**：選單加上十個技能的中文標籤（名稱取自既有的 `native_skill_catalog.rs`）。
- **`tests/native_skill_selection.rs`**：既有測試拿 220 當「尚未支援」的代表。220 接上後這個斷言會失敗，所以改用 200（196–200 在 E149 沒有 catalog 列），並加一行確認 220 不再顯示「尚未支援」。
- **coverage parser**：`維護工具/audit-native-E149-skill-catalog.py` 不用改。`維護工具/test_native_skill_coverage.py` 加 1 案 `test_current_source_covers_tenth_batch`，合計 12 案。
- **測試**：新增 `LiTo381-Modern/tests/native_skills_211_220.rs` 共 8 案，已登錄到 `維護工具/test-whitelist/tests.tsv`（LF、無 BOM）。

沒有改 `native_login.rs`：成功時伺服器送的施法動作（211、212、214–220 是 `S_ACTION` 動作 19；213 是攻擊封包動作 18）既有收包都會算成施法回覆。

### 212、217、218 不送自己（客戶端比伺服器嚴）

伺服器的 `selectActor` 收到施法者自己的 ID 時會把目標定為施法者。212 是睡眠、217 是降低能力值、218 會讓持有狀態的一方在攻擊玩家時反受傷害，三個都是對敵人用的。客戶端因此不做「沒選目標就送自己」，選到自己也拒絕。213 走 `useAttackSpell`，伺服器本來就丟棄自己。

211 是減傷 buff，伺服器同樣接受自己，客戶端沒選目標時送自己。

### 與批 9（#159）重疊的部分

寫本張時批 9 還沒進 main。兩張都需要下面三樣，本張照批 9 的寫法各寫了一份：

- `illusion_targeted`：同名、同位置，清單不同（批 9 是 202、203、204、207、208、209）。合併時把兩邊的技能編號併在一起。
- 1412 的 `Feedback::label` 與 `native_world.rs` 那一行：內容與批 9 一字不差，git 應該能自動合併。
- `selector_only`、`reuse_millis`、`client_clock_millis`、選單標籤、`tests.tsv`：同一行或相鄰行，後合併的一方要解文字衝突，兩邊都保留。

## 已驗證

### 來源（唯讀）

- `docs/PROTOCOL-CONTRACT.md` §6.3 第 489–498 行：row／column 與形狀。
- 伺服器原始碼（上層 repo lito-launcher-server，commit `6bb4a27b`；`lito伺服器/LiTo381-Server/src/main/java/local/lito381/server/`）：
  - `IllusionSkillRequests.java`：`ATTACK` 含 213，`OTHER` 含 211、212、214、216、217、218、219（第 5–6 行）；長度不是 11 就丟棄（12）；`targetObjectId <= 0` 靜默忽略（14）。`selectPc`（28）與 `selectActor`（34）：目標 ID 等於施法者就回傳施法者，否則必須在合格名單內，不在就回 null（不回覆）。
  - `DragonSkillRequests.java` 第 47 行：`current()` 沒有自己的請求時改用 `IllusionSkillRequests.current()`，所以 213 的 `resolveSelectorOnlyPrimary`（`ClientSession.java` 60436）會直接採用明確目標。
  - `ClientSession.java`：
    - 211 `usePatienceSpell`（37684）→ `resolveSelectorOnlyPatienceTarget`（37524，`selectActor` 在 37536）。合格名單 `patienceTargets`（37389）：同場存活、未隱身、沒有絕對屏障的其他玩家，加上同伴、怪物、守護塔等，都要在 `maximumRange` 內且有視線（37510）。MP 不足送 278（37748）。
    - 212 `usePhantasmSpell`（37823）、217 `usePanicSpell`（38119）、218 `useJoyOfPainSpell`（38434）：都用 `phantasmTargets`（37558，比 211 多收有絕對屏障的玩家）與 `selectActor`（37614、37644、37668）。MP 不足送 278（37941、38268、38500）。212 的 HP 檢查是 `playerHp > spell.hpCost()`（37918），不足時只記 log、不回覆。
    - 218 的效果：持有狀態的一方攻擊玩家造成傷害後，自己受到回饋傷害（30312–30324、30502 起）。
    - 213 `useSelectorOnlyTargetedAttackSpell`（60398，`armBreaker` 在 60411）→ `useAttackSpell`：目標是自己就丟棄（63885）；畫面菱形與視線（63920）；超過 `maximumRange` 丟棄（63923）。
    - 214 `useDiamondGolemSpell`（38859）、216 `useInsightSpell`（39041）、219 `useIllusionAvatarSpell`（39425）：`selectPc`（38847、39029、39413）；MP 不足送 278（38936、39119、39511）。
    - 215 `useCubeShockSpell`（37149）、220 `useCubeHarmonySpell`（37268）：`selectorOnlySpellRequest`；MP 不足送 278（37191、37310）；缺材料只記 log、不回覆（37196、37315）；放置失敗送 1412（37209、37328）。
  - `ForcedCombat` 先拆掉 LFC1 尾段再分派（#157、#159 已查），按 Ctrl 不影響長度 11 的檢查；本張沒有重讀。
- E149 catalog：原專案唯讀 `驗證紀錄/current/native-E149-skill-catalog-audit/20261024-040044-690336/catalog-facts.json`，重算 SHA256 `1EE2338D209CE41154B42B9B51B2A564261AB07FF1340EC7BCCBB188E216B996`（與 #154–#159 相同）。211–220：
  - `reuseDelayMillis`：1000、3000、1200、2000、5000、1000、4000、3000、5000、5000。
  - `clientCastClockMillis`：16、640、32、0、0、3600、0、0、0、32。
  - `maximumRange`：八個 wrapper 技能都是 3。
  - `hpCost`：212＝25、217＝30、218＝0。`mpCost` 都大於 0（25、30、25、25、55、40、30、40、35、55），所以沒有技能加進 `hp_paid`。
  - 材料：215 是 49156×1，220 是 49156×2。
- 原版 PE（`D:/lineage開發/3-lito開發/lito登入器/舊版分析與驗收/runtime/LiTo.unpacked-oep-current.exe`，重算 SHA256 `4271D92FB2B9A2FA8D118DDE0767F6D8A67A896227BCC24127438FCB5C2AB6E9`），只用 Python 讀檔、沒有執行（腳本 `scratch/issue-160/probe.py`，scratch 不進 git）：施法時鐘表 `word_635774[skill-1]` 的 211–220 是 16、640、32、0、0、3600、0、0、0、32，與 catalog 一致。

### 命令與結果

依 2026-11-09 的本機限制，本機只編譯，不執行測試執行檔。Cargo 命令在 `LiTo381-Modern/` 執行。

```text
cargo check --locked --offline -j1 --target x86_64-pc-windows-msvc --target-dir target/current --all-targets
結束碼 0（Finished `dev` profile [unoptimized + debuginfo] target(s) in 7m 37s；只保留了輸出最後 40 行，其中都是既存檔案的警告，沒有逐行確認本張的檔案完全沒有警告）

沒有另外跑 cargo test：--all-targets 的 check 已涵蓋新測試檔的型別檢查。測試執行檔未執行，沒有 `test result:` 可附。

python -B 維護工具/test_native_skill_coverage.py
結束碼 0；Ran 12 tests in 0.295s；OK
```

## 推測

- 原版客戶端對這八個 wrapper 技能送的是 3 bytes（伺服器 `ClientSession.java` 38623 起的註解：`Selected LiTo.bin emits only 14 19 03 for skill204`；#159 也查過 PE 的 dispatch 表不涵蓋 201 之後）。本張依契約與 TASKS.md 送 11 bytes，這是 LiTo 伺服器 wrapper 的要求，不是原版行為。
- 214／216／219 的客戶端沒有判斷目標是不是玩家角色。選到怪物或塔時伺服器不回覆，畫面會在 2 秒後顯示「未收到施法回覆，可重新施放」。
- 客戶端只檢查 Chebyshev 射程，沒有檢查視線；伺服器視線不通時不回覆。三格內一定在畫面菱形內，所以沒有另外檢查菱形。
- 1412 的中文說明沿用批 9 的寫法（依伺服器 log 名稱 `nearby-or-invalid`），沒有對照客戶端文字表的 1412 原文。
- 選單標籤對 215／220 只寫「消耗施法材料」，沒有查 49156 的中文名稱。

## 待驗證與未驗證什麼

- **新的 8 案 `native_skills_211_220` 沒有執行過，不算通過。** `.github/workflows/build.yml` 只跑 `cargo check --locked --all-targets` 與 `cargo test --locked --lib`，不會執行 `tests/` 下的整合測試；PR 的 CI 綠燈只代表這個檔案能編譯。需要本機測試執行者（Codex）用 `run-local-test-suite.ps1` 跑出 `test result:` 才算數。
- 改過的 `tests/native_skill_selection.rs`（`all_learned_rows_visible_but_unsupported_and_invalid_choices_cannot_cast`）同樣沒有執行。
- `native_world.rs` 多認一個訊息碼 1412，會影響所有施法的回饋判斷。伺服器目前只有四個立方 handler 會送，但既有的技能回歸測試仍應重跑。
- 伺服器不回覆的情況，客戶端一律只顯示「未收到施法回覆」，沒有對應的提示：
  - 212／217 的 HP 不足。
  - 215／220 缺材料（SKILLS-STATUS 已記為伺服器 T025）。
  - 目標不合格、視線不通、冷卻中。
- 212／217 的機率失敗：伺服器照樣送施法動作，而且這十個 handler 的 `S_MESSAGE_CODE` 只有 278 與 1412（以 handler 的 log 標籤搜尋 `ClientSession.java` 的結果），沒有 280。所以機率失敗時客戶端仍顯示「收到施法動作」，玩家只能從目標有沒有出現效果判斷。
- 216 的 3600 毫秒施法時鐘會擋住所有技能 3.6 秒（PE 與 catalog 的值，和 182 的做法相同）。
- 沒有跑正式伺服器施法：耐力的減傷、幻想的睡眠、武器破壞者的傷害與狀態、三個幻覺 buff、恐慌、疼痛的歡愉、立方的放置與每秒效果、HP／MP／材料消耗都沒驗。
- 本張沒有接 211–220 的特效／音效資源與渲染（catalog effectGfx 6507、6530、6551、6576、6505、6506、6528、6575；立方 6718／6720／6721、6724／6727），也沒有接立方物件（NPC 80151／80152）的顯示。因為沒有動資源，所以沒有用 PakViewer 查 pak。
- `docs/SKILLS-STATUS.md` 沒有更新（先前各批也沒改），留給 T026 統一同步。其中 218 的 Ctrl 欄寫「不需要（自身／友方輔助）」，與伺服器實際效果（對敵人用）不符，同步時要改。
- 沒有跑完整 Full 套件、Release 建置與 T030 GUI 驗收。

## 產出路徑

- `LiTo381-Modern/src/modern_lito/native_skills.rs`
- `LiTo381-Modern/src/modern_lito/native_skill_selection.rs`
- `LiTo381-Modern/src/modern_lito/native_world.rs`（一行加一行註解）
- `LiTo381-Modern/tests/native_skills_211_220.rs`
- `LiTo381-Modern/tests/native_skill_selection.rs`
- `維護工具/test_native_skill_coverage.py`
- `維護工具/test-whitelist/tests.tsv`
- 本報告 `docs/reports/issue-160.md`

## 給下一棒的提示

- Claude 審查重點：
  - 212／217／218 客戶端拒絕以自己為目標（伺服器會接受）。
  - 218 歸在「必須選他人」而不是 SKILLS-STATUS 寫的自身／友方輔助。
  - 211 沒選目標時送自己的 ID 與座標。
  - 1412 加進全域的 `last_skill_rejection` 名單（與批 9 相同）。
- 需要 Codex 本機執行 `native_skills_211_220`（8 案），並回歸 `native_skill_selection`、`native_skills_181_188`、`native_selector_only`、`native_skills`，回報 `test result:` 原文。
- 批 8（#158）、批 9（#159）合併進 main 之後，本分支要再同步一次；衝突位置見上面「與批 9 重疊的部分」。如果批 9 的 `tests/native_skills_201_210.rs` 有斷言 `illusion_targeted` 的完整清單，它只掃 1–220 並期待六個編號，合併後要改成只掃 201–210，或把期待值改成兩批的聯集。
- T022 十張全部合併後，`native_skills::request` 的 `_ =>`（「請求格式尚未接入」）應該只剩 E149 沒有 catalog 列的編號。
